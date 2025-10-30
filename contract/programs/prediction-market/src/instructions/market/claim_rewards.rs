//! 市场指令：领取奖励（Resolution 后）

use crate::{
    constants::{CONFIG, GLOBAL, MARKET, USERINFO},
    errors::PredictionMarketError,
    events::ClaimRewardsEvent,
    state::{config::*, market::*},
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

/// 账户集合：领取奖励所需账户
#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    /// 全局配置
    #[account(
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    pub global_config: Box<Account<'info, Config>>,

    /// 市场账户
    #[account(
        mut,
        seeds = [MARKET.as_bytes(), &yes_token.key().to_bytes(), &no_token.key().to_bytes()],
        bump
    )]
    pub market: Box<Account<'info, Market>>,

    /// 全局金库（PDA，用于签名）
    /// CHECK: global vault pda for signing
    #[account(
        mut,
        seeds = [GLOBAL.as_bytes()],
        bump,
    )]
    pub global_vault: AccountInfo<'info>,

    /// ✅ v1.1.0: USDC Mint
    #[account(
        constraint = usdc_mint.key() == global_config.usdc_mint @ PredictionMarketError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    /// ✅ v1.1.0: 全局 USDC 金库
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = global_vault,
    )]
    pub global_usdc_vault: Box<Account<'info, TokenAccount>>,

    /// YES 代币 mint
    #[account(mut)]
    pub yes_token: Box<Account<'info, Mint>>,

    /// NO 代币 mint
    #[account(mut)]
    pub no_token: Box<Account<'info, Mint>>,

    /// 用户的 YES ATA
    #[account(
        mut,
        associated_token::mint = yes_token,
        associated_token::authority = user,
    )]
    pub user_yes_ata: Box<Account<'info, TokenAccount>>,

    /// 用户的 NO ATA
    #[account(
        mut,
        associated_token::mint = no_token,
        associated_token::authority = user,
    )]
    pub user_no_ata: Box<Account<'info, TokenAccount>>,

    /// ✅ v1.1.0: 用户 USDC ATA（接收奖励）
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = user,
    )]
    pub user_usdc_ata: Box<Account<'info, TokenAccount>>,

    /// 用户信息（自动创建以支持被动持有者）
    /// 🔒 修复：使用 init_if_needed 允许仅通过链上转账收到代币的用户领取奖励
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserInfo>(),
        seeds = [USERINFO.as_bytes(), &user.key().to_bytes(), &market.key().to_bytes()],
        bump
    )]
    pub user_info: Box<Account<'info, UserInfo>>,

    /// 用户签名者
    #[account(mut)]
    pub user: Signer<'info>,

    /// 系统/代币/ATA程序
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ClaimRewards<'info> {
    /// 处理领取奖励
    ///
    /// # 流程
    /// 1. 验证市场已结算（is_completed = true）
    /// 2. 根据 resolution_yes/no_ratio 计算用户可领取的 USDC
    /// 3. 销毁用户的 YES 和 NO 代币
    /// 4. 从全局金库转 USDC 给用户
    /// 5. 清空用户余额记录
    pub fn handler(&mut self, _global_vault_bump: u8) -> Result<()> {
        msg!("ClaimRewards start");

        // ✅ v1.0.17: 验证 global_vault 已正确初始化（owner = program_id）
        require!(
            self.global_vault.owner == &crate::ID,
            PredictionMarketError::InvalidAuthority
        );

        // 0. 🔒 初始化 user_info（如果是新创建的）
        if !self.user_info.is_initialized {
            self.user_info.user = self.user.key();
            self.user_info.is_lp = false;
            self.user_info.is_initialized = true;
            msg!("✅ UserInfo auto-created for passive token holder");
        }

        // 1. 验证市场已结算
        require!(
            self.market.is_completed,
            PredictionMarketError::MarketNotCompleted
        );

        // ✅ FIX MEDIUM-4: 如果市场已完成，即使合约暂停也允许领取奖励（紧急提取）
        // 注意：市场未完成时的暂停检查已在其他指令中处理
        // 这里只要求市场必须已完成，不检查暂停状态
        // 这确保用户资金在紧急情况下不会被永久锁定

        // 2. 获取用户持有的代币数量
        let yes_balance = self.user_yes_ata.amount;
        let no_balance = self.user_no_ata.amount;

        require!(
            yes_balance > 0 || no_balance > 0,
            PredictionMarketError::InsufficientBalance
        );

        msg!(
            "User has {} YES and {} NO tokens",
            yes_balance,
            no_balance
        );

        // 3. 根据 resolution ratios 计算奖励
        // resolution_yes_ratio 和 resolution_no_ratio 是基点（10000 = 100%）
        const BASIS_POINTS: u64 = 10000;

        let yes_payout = yes_balance
            .checked_mul(self.market.resolution_yes_ratio)
            .ok_or(PredictionMarketError::MathOverflow)?
            .checked_div(BASIS_POINTS)
            .ok_or(PredictionMarketError::MathOverflow)?;

        let no_payout = no_balance
            .checked_mul(self.market.resolution_no_ratio)
            .ok_or(PredictionMarketError::MathOverflow)?
            .checked_div(BASIS_POINTS)
            .ok_or(PredictionMarketError::MathOverflow)?;

        let total_payout = yes_payout
            .checked_add(no_payout)
            .ok_or(PredictionMarketError::MathOverflow)?;

        // ✅ FIX: 允许零兑付销毁（输家代币清理）
        // 场景：用户只持有输家代币（ratio = 0），total_payout = 0
        // 原问题：require!(total_payout > 0) 阻止销毁，导致输家代币永久存在
        // 新逻辑：允许 total_payout = 0，使输家能清理代币并更新统计
        //
        // 示例：YES 全胜 (100-0)
        //   - Alice 持有 1000 NO（输家）
        //   - total_payout = 0
        //   - 允许销毁 1000 NO，不支付任何 USDC
        //   - 更新 total_no_minted -= 1000 ✅

        msg!(
            "Calculated payouts: YES={}, NO={}, Total={}",
            yes_payout,
            no_payout,
            total_payout
        );

        // ✅ v1.1.0: 4. 验证 USDC 金库有足够余额（如果有支付）
        if total_payout > 0 {
            let usdc_balance = self.global_usdc_vault.amount;
            require!(
                usdc_balance >= total_payout,
                PredictionMarketError::InsufficientLiquidity
            );

            // ✅ v1.1.0: USDC 金库最小余额校验
            let remaining_balance = usdc_balance
                .checked_sub(total_payout)
                .ok_or(PredictionMarketError::MathOverflow)?;

            require!(
                remaining_balance >= self.global_config.usdc_vault_min_balance,
                PredictionMarketError::InsufficientBalance
            );

            msg!(
                "USDC balance check: remaining {} >= min_balance {}",
                remaining_balance,
                self.global_config.usdc_vault_min_balance
            );
        }

        // 5. 销毁用户的 YES 代币（如果有）
        if yes_balance > 0 {
            token::burn(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    token::Burn {
                        mint: self.yes_token.to_account_info(),
                        from: self.user_yes_ata.to_account_info(),
                        authority: self.user.to_account_info(),
                    },
                ),
                yes_balance,
            )?;
            msg!("✅ Burned {} YES tokens", yes_balance);
        }

        // 6. 销毁用户的 NO 代币（如果有）
        if no_balance > 0 {
            token::burn(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    token::Burn {
                        mint: self.no_token.to_account_info(),
                        from: self.user_no_ata.to_account_info(),
                        authority: self.user.to_account_info(),
                    },
                ),
                no_balance,
            )?;
            msg!("✅ Burned {} NO tokens", no_balance);
        }

        // ✅ v1.1.0: 7. 从全局 USDC 金库转给用户（仅在有支付时）
        if total_payout > 0 {
            let signer_seeds: &[&[&[u8]]] = &[&[
                crate::constants::GLOBAL.as_bytes(),
                &[_global_vault_bump],
            ]];

            token::transfer(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    token::Transfer {
                        from: self.global_usdc_vault.to_account_info(),
                        to: self.user_usdc_ata.to_account_info(),
                        authority: self.global_vault.to_account_info(),
                    },
                    signer_seeds,
                ),
                total_payout,
            )?;

            msg!("✅ Transferred {} USDC to user", total_payout);
        } else {
            msg!("✅ Zero payout (losing tokens) - no USDC transfer");
        }

        // ✅ v1.0.17: 支持所有代币类型的claim（包括swap代币）
        //
        // 设计理念：
        //   - mint_complete_set 用户: 有 1:1 抵押品 (total_collateral_locked)
        //   - swap 买入用户: 支付的 USDC 进入 LP pool (pool_collateral_reserve)
        //   - 两类用户在市场结算后都应该能够claim奖励
        //
        // 资金来源优先级：
        //   1. 优先使用 total_collateral_locked (mint用户的抵押品)
        //   2. 不足时从 pool_collateral_reserve 支付 (LP承担做市成本)
        //
        // LP 风险说明：
        //   - LP 做市收取手续费（platform_fee + lp_fee）和价格滑点
        //   - 但同时承担结算时的赔付义务（pool_collateral_reserve被用于支付奖励）
        //   - 这是AMM/LMSR预测市场的标准设计
        //   - LP应理解：收益来自手续费，风险来自结算赔付
        //
        // 流动性保护：
        //   - withdraw_liquidity 要求 pool_settled=true（在settle_pool之后）
        //   - 这给用户足够的时间窗口先claim
        //   - 但如果LP在settle前大量提现，可能导致资金不足
        //   - 建议：前端引导LP等待结算，或添加流动性锁定期
        //
        // 资金不足处理：
        //   - 如果 total_collateral_locked + pool_collateral_reserve < total_payout
        //   - 返回 InsufficientLiquidity
        //   - 这是流动性枯竭，需要治理层介入

        // 步骤1: 从 total_collateral_locked 释放（优先）
        let collateral_released = total_payout.min(self.market.total_collateral_locked);
        self.market.total_collateral_locked = self.market.total_collateral_locked
            .checked_sub(collateral_released)
            .ok_or(PredictionMarketError::InsufficientLiquidity)?;

        // 步骤2: 计算需要从 pool_collateral_reserve 支付的数量
        let from_liquidity = total_payout
            .checked_sub(collateral_released)
            .ok_or(PredictionMarketError::MathOverflow)?;

        // 步骤3: 如果需要从流动性支付，验证并扣减
        if from_liquidity > 0 {
            require!(
                self.market.pool_collateral_reserve >= from_liquidity,
                PredictionMarketError::InsufficientLiquidity
            );

            self.market.pool_collateral_reserve = self.market.pool_collateral_reserve
                .checked_sub(from_liquidity)
                .ok_or(PredictionMarketError::MathOverflow)?;

            msg!(
                "⚠️ Using LP pool funds: {} USDC (pool_collateral_reserve: {} -> {})",
                from_liquidity,
                self.market.pool_collateral_reserve + from_liquidity,
                self.market.pool_collateral_reserve
            );
        }

        msg!(
            "✅ Claim payout breakdown: total={}, from_collateral={}, from_pool={}",
            total_payout,
            collateral_released,
            from_liquidity
        );

        // 同步更新代币铸造统计（只减少有统计记录的部分）
        // 注意：swap 买入的代币可能超过 total_*_minted，需要用 min() 避免下溢
        let yes_minted_decrease = yes_balance.min(self.market.total_yes_minted);
        let no_minted_decrease = no_balance.min(self.market.total_no_minted);

        self.market.total_yes_minted = self.market.total_yes_minted
            .checked_sub(yes_minted_decrease)
            .ok_or(PredictionMarketError::MathOverflow)?;

        self.market.total_no_minted = self.market.total_no_minted
            .checked_sub(no_minted_decrease)
            .ok_or(PredictionMarketError::MathOverflow)?;

        // 同步减少 AMM 供应量计数
        // ⚠️ 注意：swap 买入的代币可能导致用户余额 > total_supply 统计
        // 使用 min() 避免下溢（与上面 total_*_minted 的处理一致）
        let yes_supply_decrease = yes_balance.min(self.market.token_yes_total_supply);
        let no_supply_decrease = no_balance.min(self.market.token_no_total_supply);

        self.market.token_yes_total_supply = self.market.token_yes_total_supply
            .checked_sub(yes_supply_decrease)
            .ok_or(PredictionMarketError::InsufficientBalance)?;

        self.market.token_no_total_supply = self.market.token_no_total_supply
            .checked_sub(no_supply_decrease)
            .ok_or(PredictionMarketError::InsufficientBalance)?;

        msg!(
            "✅ Released {} USDC collateral (paid {} USDC, burned {} YES + {} NO tokens). Remaining locked: {}",
            total_payout,
            total_payout,
            yes_balance,
            no_balance,
            self.market.total_collateral_locked
        );

        // ✅ FIX CRITICAL-2: 不再清空 user_info 余额（已删除字段）
        // 代币已销毁，SPL Token ATA 余额已自动更新为 0

        msg!(
            "✅ ClaimRewards completed: {} YES + {} NO → {} USDC (ratios: YES={}, NO={})",
            yes_balance,
            no_balance,
            total_payout,
            self.market.resolution_yes_ratio,
            self.market.resolution_no_ratio
        );

        // ═══════════════════════════════════════════════════════════════
        // ✅ v1.0.11: 发射领取奖励事件
        // ═══════════════════════════════════════════════════════════════
        let clock = Clock::get()?;
        emit!(ClaimRewardsEvent {
            user: self.user.key(),
            market: self.market.key(),
            yes_burned: yes_balance,
            no_burned: no_balance,
            usdc_payout: total_payout,  // ✅ v1.1.0: 字段名改为 usdc_payout
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }
}
