//! 市场指令：结算市场（由管理员触发）

use crate::{
    constants::{CONFIG, GLOBAL, MARKET},
    errors::PredictionMarketError,
    events::ResolutionEvent,
    state::{config::*, market::*},
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token},
};

/// 账户集合：市场结算所需账户
#[derive(Accounts)]
pub struct Resolution<'info> {
    /// 全局配置
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    global_config: Box<Account<'info, Config>>,

    /// 市场账户
    #[account(
        mut,
        seeds = [MARKET.as_bytes(), &yes_token.key().to_bytes(), &no_token.key().to_bytes()], 
        bump
    )]
    market: Account<'info, Market>,

    /// 全局金库（PDA，存放 USDC）
    /// CHECK: global vault pda which stores USDC
    #[account(
        mut,
        seeds = [GLOBAL.as_bytes()],
        bump,
    )]
    pub global_vault: AccountInfo<'info>,

    /// YES/NO 代币mint
    pub yes_token: Box<Account<'info, Mint>>,
    pub no_token: Box<Account<'info, Mint>>,

    /// ✅ FIX CRITICAL: 添加 PDA 的代币账户用于清算
    /// global_yes_ata - PDA 持有的 YES 代币（swap 卖出时积累）
    #[account(
        mut,
        associated_token::mint = yes_token,
        associated_token::authority = global_vault,
    )]
    pub global_yes_ata: Box<Account<'info, token::TokenAccount>>,

    /// global_no_ata - PDA 持有的 NO 代币（swap 卖出时积累）
    #[account(
        mut,
        associated_token::mint = no_token,
        associated_token::authority = global_vault,
    )]
    pub global_no_ata: Box<Account<'info, token::TokenAccount>>,

    // ✅ v1.0.23: 移除未使用的 user/user_info 账户（感谢审计发现!）
    //
    // 🔴 原问题：Resolution 强制要求 user 和 user_info 账户
    //    但在 handler 函数中完全未使用这些账户
    //    导致无人交易的市场无法结算（因为不存在 UserInfo PDA）
    //
    // ✅ 修复：移除这些账户约束
    //    Resolution 是管理员操作，不需要用户账户
    //    PDA 代币清算由 global_vault 签名执行
    //
    // 已删除：
    // - user_info: Box<Account<'info, UserInfo>>
    // - user: AccountInfo<'info>

    /// 管理员（必须为全局authority）
    /// 🔒 v1.1.0: CRITICAL - 强制验证 authority 必须是 global_config.authority
    #[account(
        mut,
        constraint = authority.key() == global_config.authority @ PredictionMarketError::InvalidAuthority
    )]
    pub authority: Signer<'info>,

    /// 系统/代币/ATA程序
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Resolution<'info>{
    /// 市场结算：由管理员调用，设置获胜方
    ///
    /// # 参数
    /// * `yes_amount` - YES代币的赎回比例（0-10000，10000=100%）
    /// * `no_amount` - NO代币的赎回比例（0-10000，10000=100%）
    /// * `token_type` - 获胜方（0=NO, 1=YES, 2=平局）
    /// * `is_completed` - 是否完成结算
    /// * `global_vault_bump` - 全局金库bump
    pub fn handler(&mut self, yes_amount: u64, no_amount: u64, token_type: u8, is_completed: bool, _global_vault_bump: u8) -> Result<()> {
        msg!("Resolution start: token_type={}, yes_amount={}, no_amount={}", token_type, yes_amount, no_amount);

        // 1. 验证权限：仅限管理员
        require!(
            self.authority.key() == self.global_config.authority.key(),
            PredictionMarketError::InvalidMigrationAuthority
        );

        // 2. 验证市场未结算
        require!(
            !self.market.is_completed,
            PredictionMarketError::MarketIsCompleted
        );

        // ✅ FIX HIGH-2: 验证市场已到达结束时间
        // 防止管理员在市场结束前提前结算
        if let Some(ending_slot) = self.market.ending_slot {
            let clock = Clock::get()?;
            let current_slot = clock.slot;
            require!(
                current_slot >= ending_slot,
                PredictionMarketError::MarketNotEnded
            );
            msg!(
                "✅ Time check passed: current_slot={}, ending_slot={}",
                current_slot,
                ending_slot
            );
        }

        // 3. 验证token_type有效（0=NO获胜, 1=YES获胜, 2=平局）
        require!(
            token_type <= 2,
            PredictionMarketError::RESOLUTIONTOKEYTYPEERROR
        );

        // 4. 验证赎回比例（使用基点：10000 = 100%）
        const MAX_BASIS_POINTS: u64 = 10000;

        match token_type {
            0 => {
                // NO获胜：NO=100%, YES=0%
                require!(
                    no_amount == MAX_BASIS_POINTS,
                    PredictionMarketError::RESOLUTIONNOAMOUNTERROR
                );
                require!(
                    yes_amount == 0,
                    PredictionMarketError::RESOLUTIONYESAMOUNTERROR
                );
                msg!("Resolution: NO wins (NO holders get 100%)");
            },
            1 => {
                // YES获胜：YES=100%, NO=0%
                require!(
                    yes_amount == MAX_BASIS_POINTS,
                    PredictionMarketError::RESOLUTIONYESAMOUNTERROR
                );
                require!(
                    no_amount == 0,
                    PredictionMarketError::RESOLUTIONNOAMOUNTERROR
                );
                msg!("Resolution: YES wins (YES holders get 100%)");
            },
            2 => {
                // 平局：YES=50%, NO=50%（或自定义比例）
                require!(
                    yes_amount + no_amount == MAX_BASIS_POINTS,
                    PredictionMarketError::RESOLUTIONYESAMOUNTERROR
                );
                msg!("Resolution: Draw (YES={}, NO={})", yes_amount, no_amount);
            },
            _ => {
                return Err(PredictionMarketError::RESOLUTIONTOKEYTYPEERROR.into());
            }
        }

        // 5. 设置结算参数（存储在market中）
        self.market.resolution_yes_ratio = yes_amount;
        self.market.resolution_no_ratio = no_amount;
        self.market.winner_token_type = token_type;

        // 6. 标记市场完成
        if is_completed {
            self.market.is_completed = true;
            msg!("Market marked as completed");
        }

        // ✅ FIX CRITICAL: 清算 PDA 持有的代币，释放抵押品
        //
        // 问题：
        //   - swap 卖出时，代币转入 global_yes_ata/global_no_ata（授权者是 PDA）
        //   - PDA 无法调用 claim_rewards（需要 user.is_signer）
        //   - PDA 持有的胜方代币永远无法销毁
        //   - total_collateral_locked 永远无法归零
        //   - LP 无法提取流动性（available = vault - locked 始终 < 0）
        //
        // 解决方案：
        //   - 在 resolution 时由管理员代表 PDA 清算代币
        //   - 销毁 PDA 持有的所有 YES 和 NO 代币
        //   - 按结算比例释放抵押品
        //   - 确保 total_collateral_locked 能够归零
        //
        // ✅ FIX CRITICAL-10: 处理混合代币架构
        // 问题：
        //   - mint_complete_set 创建的代币有抵押品支持（tracked in total_*_minted）
        //   - mint_no_token 创建的初始流动性代币没有抵押品（现在已修复统计）
        //   - 如果 total_*_minted < pda_balance 会导致 checked_sub 下溢
        // 解决：
        //   - 只销毁有抵押品支持的代币（min(pda_balance, total_minted)）
        //   - 只释放对应的抵押品
        //   - 确保统计数据不会下溢

        let global_yes_balance = self.global_yes_ata.amount;
        let global_no_balance = self.global_no_ata.amount;

        msg!(
            "PDA token balances: YES={}, NO={}, total_yes_minted={}, total_no_minted={}",
            global_yes_balance,
            global_no_balance,
            self.market.total_yes_minted,
            self.market.total_no_minted
        );

        // ✅ 计算可销毁的代币数量（不超过有抵押品支持的数量）
        let yes_burnable = global_yes_balance.min(self.market.total_yes_minted);
        let no_burnable = global_no_balance.min(self.market.total_no_minted);

        msg!(
            "Burnable amounts (with collateral backing): YES={}, NO={}",
            yes_burnable,
            no_burnable
        );

        // 7. 销毁 PDA 持有的 YES 代币（只销毁有抵押品支持的部分）
        if yes_burnable > 0 {
            let signer_seeds: &[&[&[u8]]] = &[&[
                crate::constants::GLOBAL.as_bytes(),
                &[_global_vault_bump],
            ]];

            token::burn(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    token::Burn {
                        mint: self.yes_token.to_account_info(),
                        from: self.global_yes_ata.to_account_info(),
                        authority: self.global_vault.to_account_info(),
                    },
                    signer_seeds,
                ),
                yes_burnable,
            )?;
            msg!("✅ Burned {} YES tokens from PDA (with collateral)", yes_burnable);
        }

        // 8. 销毁 PDA 持有的 NO 代币（只销毁有抵押品支持的部分）
        if no_burnable > 0 {
            let signer_seeds: &[&[&[u8]]] = &[&[
                crate::constants::GLOBAL.as_bytes(),
                &[_global_vault_bump],
            ]];

            token::burn(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    token::Burn {
                        mint: self.no_token.to_account_info(),
                        from: self.global_no_ata.to_account_info(),
                        authority: self.global_vault.to_account_info(),
                    },
                    signer_seeds,
                ),
                no_burnable,
            )?;
            msg!("✅ Burned {} NO tokens from PDA (with collateral)", no_burnable);
        }

        // 9. 按结算比例释放抵押品（只释放已销毁代币的抵押品）
        // 注意：只计算有抵押品支持的代币的支付额
        const BASIS_POINTS: u64 = 10000;

        let yes_payout = yes_burnable
            .checked_mul(yes_amount)
            .ok_or(PredictionMarketError::MathOverflow)?
            .checked_div(BASIS_POINTS)
            .ok_or(PredictionMarketError::MathOverflow)?;

        let no_payout = no_burnable
            .checked_mul(no_amount)
            .ok_or(PredictionMarketError::MathOverflow)?
            .checked_div(BASIS_POINTS)
            .ok_or(PredictionMarketError::MathOverflow)?;

        let total_pda_payout = yes_payout
            .checked_add(no_payout)
            .ok_or(PredictionMarketError::MathOverflow)?;

        // 10. 释放 PDA 代币对应的抵押品
        self.market.total_collateral_locked = self.market.total_collateral_locked
            .checked_sub(total_pda_payout)
            .ok_or(PredictionMarketError::InsufficientLiquidity)?;

        // 11. 更新代币统计（只减去实际销毁的数量）
        self.market.total_yes_minted = self.market.total_yes_minted
            .checked_sub(yes_burnable)
            .ok_or(PredictionMarketError::MathOverflow)?;

        self.market.total_no_minted = self.market.total_no_minted
            .checked_sub(no_burnable)
            .ok_or(PredictionMarketError::MathOverflow)?;

        self.market.token_yes_total_supply = self.market.token_yes_total_supply
            .checked_sub(yes_burnable)
            .ok_or(PredictionMarketError::InsufficientBalance)?;

        self.market.token_no_total_supply = self.market.token_no_total_supply
            .checked_sub(no_burnable)
            .ok_or(PredictionMarketError::InsufficientBalance)?;

        // ✅ CRITICAL FIX v1.0.17: 同步更新 Pool Ledger
        //
        // 问题：
        //   - resolution 销毁 global_yes_ata/global_no_ata 中的所有代币
        //   - 但这些 ATA 同时存储 Pool reserves（LP 提供的流动性）
        //   - 如果不同步减少 pool_yes_reserve/pool_no_reserve
        //   - settle_pool 和 withdraw_liquidity 会认为池中还有代币
        //   - 导致账面数 > 实际余额，提现失败（InsufficientLiquidity）
        //
        // 解决：
        //   - 销毁代币后，同步减少对应的 pool reserves
        //   - 确保 pool_*_reserve 反映实际可用余额
        //
        // 注意：只减少实际销毁的数量（yes_burnable/no_burnable）
        //       不能减少超过 pool reserves 的数量（否则会下溢）

        let yes_pool_reduction = yes_burnable.min(self.market.pool_yes_reserve);
        let no_pool_reduction = no_burnable.min(self.market.pool_no_reserve);

        if yes_pool_reduction > 0 {
            self.market.pool_yes_reserve = self.market.pool_yes_reserve
                .checked_sub(yes_pool_reduction)
                .ok_or(PredictionMarketError::MathOverflow)?;
            msg!("✅ Reduced pool_yes_reserve by {}", yes_pool_reduction);
        }

        if no_pool_reduction > 0 {
            self.market.pool_no_reserve = self.market.pool_no_reserve
                .checked_sub(no_pool_reduction)
                .ok_or(PredictionMarketError::MathOverflow)?;
            msg!("✅ Reduced pool_no_reserve by {}", no_pool_reduction);
        }

        msg!(
            "✅ PDA liquidation: burned {} YES + {} NO (with collateral), released {} USDC collateral",
            yes_burnable,
            no_burnable,
            total_pda_payout
        );

        msg!(
            "✅ Resolution completed: token_type={}, YES ratio={}, NO ratio={}, total_collateral_locked={}",
            token_type,
            yes_amount,
            no_amount,
            self.market.total_collateral_locked
        );

        // ═══════════════════════════════════════════════════════════════
        // ✅ 发射市场解决事件
        // ═══════════════════════════════════════════════════════════════
        let clock = Clock::get()?;
        emit!(ResolutionEvent {
            authority: self.authority.key(),
            market: self.market.key(),
            winner_token_type: token_type,
            yes_ratio: yes_amount,
            no_ratio: no_amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }
}
