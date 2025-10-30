//! 赎回完整集合指令：用户销毁等量的 YES + NO 代币，赎回 USDC
//!
//! 这是条件代币的核心机制（反向操作）：
//! - 用户提供 X YES + X NO
//! - 系统销毁这些代币
//! - 返还 X USDC 给用户

use crate::{
    constants::{CONFIG, GLOBAL, MARKET, USERINFO},
    errors::PredictionMarketError,
    state::{config::*, market::*},
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

/// 账户集合：赎回完整集合所需账户
#[derive(Accounts)]
pub struct RedeemCompleteSet<'info> {
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

    /// ✅ v1.1.0: 全局 USDC 金库（存放 USDC 抵押品）
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = global_vault,
    )]
    pub global_usdc_vault: Box<Account<'info, TokenAccount>>,

    /// YES 代币 mint
    /// ✅ FIX HIGH-4: 验证 mint authority 必须是 global_vault PDA
    #[account(
        mut,
        constraint = yes_token.mint_authority == anchor_lang::solana_program::program_option::COption::Some(global_vault.key())
            @ PredictionMarketError::InvalidAuthority
    )]
    pub yes_token: Box<Account<'info, Mint>>,

    /// NO 代币 mint
    /// ✅ FIX HIGH-4: 验证 mint authority 必须是 global_vault PDA
    #[account(
        mut,
        constraint = no_token.mint_authority == anchor_lang::solana_program::program_option::COption::Some(global_vault.key())
            @ PredictionMarketError::InvalidAuthority
    )]
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

    /// ✅ v1.1.0: 用户 USDC ATA（接收赎回的抵押品）
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = user,
    )]
    pub user_usdc_ata: Box<Account<'info, TokenAccount>>,

    /// 用户信息（自动创建以支持被动持有者）
    /// 🔒 修复：使用 init_if_needed 允许仅通过链上转账收到代币的用户赎回
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

impl<'info> RedeemCompleteSet<'info> {
    /// 处理赎回完整集合
    ///
    /// # 参数
    /// * `amount` - 赎回数量（必须同时持有等量的 YES 和 NO）
    /// * `global_vault_bump` - 全局金库的 bump
    ///
    /// # 流程
    /// 1. 验证用户有足够的 YES 和 NO 代币
    /// 2. 销毁用户的 YES 代币
    /// 3. 销毁用户的 NO 代币
    /// 4. 从全局 USDC 金库返还 USDC 给用户
    /// 5. 更新市场统计
    pub fn handler(&mut self, amount: u64, global_vault_bump: u8) -> Result<()> {
        msg!("RedeemCompleteSet start: amount={}", amount);

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

        // 验证金额
        require!(amount > 0, PredictionMarketError::InvalidAmount);

        // ✅ FIX: 设计决策 - 市场完成后不允许 redeem_complete_set
        // 原因：
        // 1. 市场完成后，用户应该使用 claim_rewards（按结算比例领取）
        // 2. redeem_complete_set 是 1:1 赎回（1 YES + 1 NO = 1 USDC）
        // 3. 如果允许市场完成后 1:1 赎回，用户可能错失获胜方的额外奖励
        //
        // 例子：YES 全胜 (100-0)
        //   - 用户持有 100 YES + 100 NO
        //   - 如果 redeem: 获得 100 USDC（错失 100 USDC 奖励！）
        //   - 如果 claim: 获得 100 USDC（YES）+ 0 USDC（NO）= 100 USDC
        //   - 但用户本应先赎回 NO（0价值），然后 claim YES（100 USDC）
        //
        // 正确流程：
        //   - 市场完成前：可以随时 redeem_complete_set
        //   - 市场完成后：必须使用 claim_rewards
        //
        // 紧急提取机制：
        //   - claim_rewards 在市场完成后允许执行（即使合约暂停）
        //   - 这确保用户资金不会被永久锁定
        require!(
            !self.market.is_completed,
            PredictionMarketError::CurveAlreadyCompleted
        );

        // ✅ FIX MEDIUM-4: 只在市场未完成时检查暂停状态
        // 注意：上面已经确认市场未完成，所以这里必定执行
        require!(
            !self.global_config.is_paused,
            PredictionMarketError::ContractPaused
        );

        // 验证用户有足够的代币
        require!(
            self.user_yes_ata.amount >= amount,
            PredictionMarketError::InsufficientBalance
        );
        require!(
            self.user_no_ata.amount >= amount,
            PredictionMarketError::InsufficientBalance
        );

        // 验证市场有足够的抵押品（账本记录）
        let market = &self.market;
        require!(
            market.total_collateral_locked >= amount,
            PredictionMarketError::InsufficientLiquidity
        );

        // ✅ v1.1.0: 验证全局 USDC 金库有足够的实际 USDC 余额
        let usdc_balance = self.global_usdc_vault.amount;
        require!(
            usdc_balance >= amount,
            PredictionMarketError::InsufficientLiquidity
        );

        // 1. 销毁用户的 YES 代币
        token::burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Burn {
                    mint: self.yes_token.to_account_info(),
                    from: self.user_yes_ata.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            amount,
        )?;
        msg!("✅ Burned {} YES tokens", amount);

        // 2. 销毁用户的 NO 代币
        token::burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Burn {
                    mint: self.no_token.to_account_info(),
                    from: self.user_no_ata.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            amount,
        )?;
        msg!("✅ Burned {} NO tokens", amount);

        // ✅ v1.1.0: 3. 从全局 USDC 金库返还 USDC 给用户
        // ✅ v1.1.0: USDC 金库最小余额校验
        let remaining_balance = usdc_balance
            .checked_sub(amount)
            .ok_or(PredictionMarketError::InsufficientLiquidity)?;

        require!(
            remaining_balance >= self.global_config.usdc_vault_min_balance,
            PredictionMarketError::InsufficientBalance
        );

        msg!(
            "USDC balance check: remaining {} >= min_balance {}",
            remaining_balance,
            self.global_config.usdc_vault_min_balance
        );

        let signer_seeds: &[&[&[u8]]] = &[&[
            crate::constants::GLOBAL.as_bytes(),
            &[global_vault_bump],
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
            amount,
        )?;
        msg!("✅ Returned {} USDC to user", amount);

        // 4. 更新市场状态
        let market = &mut self.market;
        market.total_collateral_locked = market
            .total_collateral_locked
            .checked_sub(amount)
            .ok_or(PredictionMarketError::MathOverflow)?;
        market.total_yes_minted = market
            .total_yes_minted
            .checked_sub(amount)
            .ok_or(PredictionMarketError::MathOverflow)?;
        market.total_no_minted = market
            .total_no_minted
            .checked_sub(amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        // ✅ FIX: 同步减少 AMM 供应量计数
        market.token_yes_total_supply = market
            .token_yes_total_supply
            .checked_sub(amount)
            .ok_or(PredictionMarketError::InsufficientBalance)?;
        market.token_no_total_supply = market
            .token_no_total_supply
            .checked_sub(amount)
            .ok_or(PredictionMarketError::InsufficientBalance)?;

        // ✅ FIX CRITICAL-2: 不再更新 user_info 余额（已删除）
        // 余额由 SPL Token ATA 自动追踪，销毁操作已经更新了 ATA 余额

        msg!(
            "✅ RedeemCompleteSet completed: {} YES + {} NO → {} USDC",
            amount,
            amount,
            amount
        );
        msg!(
            "   Market totals: collateral={}, yes_minted={}, no_minted={}",
            market.total_collateral_locked,
            market.total_yes_minted,
            market.total_no_minted
        );

        // ✅ v1.1.1: 发射赎回事件（增强可追溯性）
        let clock = Clock::get()?;
        emit!(crate::events::RedeemCompleteSetEvent {
            user: self.user.key(),
            market: self.market.key(),
            yes_burned: amount,
            no_burned: amount,
            usdc_returned: amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }
}
