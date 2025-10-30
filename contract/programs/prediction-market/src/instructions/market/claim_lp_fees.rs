//! LP 费用领取指令（公平分配版）
//! ✅ 双账本系统：只操作 Pool Ledger
//!
//! 功能：
//! - LP 按累计每份额收益公平领取手续费
//! - 使用 fee_per_share_cumulative 模型防止抢跑
//! - 更新 last_fee_per_share 防止重复领取
//! - 手续费从 accumulated_lp_fees 扣除
//!
//! 公平分配原理：
//! - 全局维护 fee_per_share_cumulative（每次 swap 后更新）
//! - 每个 LP 记录 last_fee_per_share（上次领取时的全局值）
//! - 可领取费用 = lp_shares * (current_fee_per_share - last_fee_per_share)
//! - 这样无论谁先领取，每个 LP 每份额只能领取一次对应收益
//!
//! 精度说明：
//! - fee_per_share_cumulative 使用 u128 存储，精度为 10^18
//! - 前端显示时需要除以 10^18 转换为实际 USDC 值
//! - 例如：fee_per_share_cumulative = 5 * 10^18，表示每份额累计收益 5 USDC
//!
//! 金库余额保护：
//! - 领取前检查 global_vault 余额是否充足
//! - 检查 market.accumulated_lp_fees >= 可领取金额
//! - 两级验证确保不会超额支付
//! - 即使 fee_per_share 计算值大于实际可用余额，也会被余额检查拦截

use crate::{
    constants::{CONFIG, GLOBAL, LPPOSITION, MARKET},
    errors::PredictionMarketError,
    state::{config::*, market::*},
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

/// 账户集合：LP 费用领取
#[derive(Accounts)]
pub struct ClaimLpFees<'info> {
    /// 全局配置
    #[account(
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    global_config: Box<Account<'info, Config>>,

    /// YES/NO 代币mint（用于 PDA 推导）
    pub yes_token: AccountInfo<'info>,
    pub no_token: AccountInfo<'info>,

    /// 市场账户
    #[account(
        mut,
        seeds = [MARKET.as_bytes(), &yes_token.key().to_bytes(), &no_token.key().to_bytes()],
        bump,
    )]
    market: Account<'info, Market>,

    /// 全局金库（支付手续费）
    /// CHECK: global vault pda which stores USDC
    #[account(
        mut,
        seeds = [GLOBAL.as_bytes()],
        bump,
    )]
    pub global_vault: AccountInfo<'info>,

    /// LP Position 账户
    #[account(
        mut,
        seeds = [LPPOSITION.as_bytes(), &lp.key().to_bytes(), &market.key().to_bytes()],
        bump,
        constraint = lp_position.lp_shares > 0 @PredictionMarketError::WITHDRAWNOTLPERROR
    )]
    pub lp_position: Box<Account<'info, LPPosition>>,

    /// LP 用户（手续费接收者）
    #[account(mut)]
    pub lp: Signer<'info>,

    /// ✅ v1.1.0: USDC Mint
    #[account(
        constraint = usdc_mint.key() == global_config.usdc_mint @ PredictionMarketError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    /// ✅ v1.1.0: 全局 USDC 金库（支付费用）
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = global_vault,
    )]
    pub global_usdc_vault: Box<Account<'info, TokenAccount>>,

    /// ✅ v1.1.0: LP 的 USDC ATA（接收费用）
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = lp,
    )]
    pub lp_usdc_ata: Box<Account<'info, TokenAccount>>,

    /// 系统程序
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ClaimLpFees<'info> {
    /// 处理 LP 费用领取（公平分配版）
    ///
    /// ✅ 双账本系统：只操作 Pool Ledger
    /// ✅ 公平分配：基于累计每份额收益模型
    ///
    /// # 参数
    /// - _global_vault_bump: PDA bump seed (unused, reserved for future use)
    pub fn handler(&mut self, _global_vault_bump: u8) -> Result<()> {
        msg!("ClaimLpFees handler start (fair distribution model)");

        // ═══════════════════════════════════════════════════════════
        // 1. 验证前置条件
        // ═══════════════════════════════════════════════════════════

        require!(
            !self.global_config.is_paused,
            PredictionMarketError::ContractPaused
        );

        require!(
            self.market.total_lp_shares > 0,
            PredictionMarketError::InsufficientLiquidity
        );

        // ═══════════════════════════════════════════════════════════
        // 2. 计算可领取的手续费（公平分配模型）
        // ═══════════════════════════════════════════════════════════

        // 自上次领取以来，每份额累计收益增加量
        let fee_per_share_delta = self.market.fee_per_share_cumulative
            .checked_sub(self.lp_position.last_fee_per_share)
            .ok_or(PredictionMarketError::MathOverflow)?;

        // 该 LP 的总可领取费用 = lp_shares * fee_per_share_delta / 10^18
        let claimable_fees = (self.lp_position.lp_shares as u128)
            .checked_mul(fee_per_share_delta)
            .ok_or(PredictionMarketError::MathOverflow)?
            .checked_div(1_000_000_000_000_000_000) // 除以 10^18 精度
            .ok_or(PredictionMarketError::MathOverflow)?;

        require!(
            claimable_fees <= u64::MAX as u128,
            PredictionMarketError::MathOverflow
        );

        let fees_amount = claimable_fees as u64;

        // 如果没有可领取费用，直接返回
        if fees_amount == 0 {
            msg!("No fees to claim for this LP");
            return Ok(());
        }

        msg!(
            "LP {} can claim {} USDC (lp_shares: {}, fee_delta: {})",
            self.lp.key(),
            fees_amount,
            self.lp_position.lp_shares,
            fee_per_share_delta
        );

        // ═══════════════════════════════════════════════════════════
        // 3. 验证金库和累积费用充足
        // ═══════════════════════════════════════════════════════════

        // ✅ v1.1.0: 检查 USDC 余额（非 Solana 原生 SOL lamports）
        let vault_usdc_balance = self.global_usdc_vault.amount;
        require!(
            vault_usdc_balance >= fees_amount,
            PredictionMarketError::InsufficientLiquidity
        );

        require!(
            self.market.accumulated_lp_fees >= fees_amount,
            PredictionMarketError::InsufficientBalance
        );

        // ═══════════════════════════════════════════════════════════
        // 4. 转移手续费给 LP
        // ═══════════════════════════════════════════════════════════

        // ✅ v1.1.0: 使用 USDC token::transfer 而不是 lamports 操作
        let signer_seeds: &[&[&[u8]]] = &[&[
            crate::constants::GLOBAL.as_bytes(),
            &[_global_vault_bump],
        ]];

        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.global_usdc_vault.to_account_info(),
                    to: self.lp_usdc_ata.to_account_info(),
                    authority: self.global_vault.to_account_info(),
                },
                signer_seeds,
            ),
            fees_amount,
        )?;

        // ═══════════════════════════════════════════════════════════
        // 5. 更新状态
        // ═══════════════════════════════════════════════════════════

        // 从累积费用中扣除
        self.market.accumulated_lp_fees = self.market.accumulated_lp_fees
            .checked_sub(fees_amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        // ✅ 关键：更新 last_fee_per_share 为当前值（防止重复领取）
        self.lp_position.last_fee_per_share = self.market.fee_per_share_cumulative;

        // 更新时间戳（可选，用于统计）
        self.lp_position.last_fee_claim_slot = Clock::get()?.slot;

        msg!(
            "✅ LP claimed {} USDC (fair share). Remaining accumulated fees: {}",
            fees_amount,
            self.market.accumulated_lp_fees
        );

        msg!("ClaimLpFees completed successfully");
        Ok(())
    }
}
