//! 市场指令：提取流动性（LP提取）
//! ✅ 双账本系统：只操作 Pool Ledger

use crate::{
    constants::{CONFIG, GLOBAL, LPPOSITION, MARKET},
    errors::PredictionMarketError,
    events::WithdrawLiquidityEvent,
    state::{config::*, market::*},
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

/// 账户集合：提取LP所需账户
#[derive(Accounts)]
pub struct WithdrawLiquidity<'info> {
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

    /// 全局金库（存放 USDC）
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

    /// 全局 YES/NO 代币账户（Pool 库存）
    #[account(
        mut,
        associated_token::mint = yes_token,
        associated_token::authority = global_vault,
    )]
    pub global_yes_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = no_token,
        associated_token::authority = global_vault,
    )]
    pub global_no_ata: Box<Account<'info, TokenAccount>>,

    /// 用户 YES/NO 代币账户
    #[account(
        mut,
        associated_token::mint = yes_token,
        associated_token::authority = user,
    )]
    pub user_yes_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = no_token,
        associated_token::authority = user,
    )]
    pub user_no_ata: Box<Account<'info, TokenAccount>>,

    /// ✅ v1.1.0: USDC Mint
    #[account(
        constraint = usdc_mint.key() == global_config.usdc_mint @ PredictionMarketError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    /// ✅ v1.1.0: 全局 USDC 金库（Pool 的 USDC 储备）
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = global_vault,
    )]
    pub global_usdc_vault: Box<Account<'info, TokenAccount>>,

    /// ✅ v1.1.0: 用户 USDC ATA
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = user,
    )]
    pub user_usdc_ata: Box<Account<'info, TokenAccount>>,

    /// LP Position 账户（记录 LP 份额）
    #[account(
        mut,
        seeds = [LPPOSITION.as_bytes(), &user.key().to_bytes(), &market.key().to_bytes()],
        bump,
        constraint = lp_position.lp_shares > 0 @PredictionMarketError::WITHDRAWNOTLPERROR
    )]
    pub lp_position: Box<Account<'info, LPPosition>>,

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

impl<'info> WithdrawLiquidity<'info> {
    /// 处理提取流动性：LP 赎回份额，获得 USDC + YES + NO 代币
    ///
    /// ✅ 双账本系统：只操作 Pool Ledger
    /// 参数：
    /// - lp_shares_to_burn: 要赎回的 LP 份额数量
    /// - global_vault_bump: PDA bump seed
    pub fn handler(&mut self, lp_shares_to_burn: u64, global_vault_bump: u8) -> Result<()> {
        msg!(
            "WithdrawLiquidity handler start: lp_shares={}",
            lp_shares_to_burn
        );

        // ═══════════════════════════════════════════════════════════
        // 1. 验证前置条件
        // ═══════════════════════════════════════════════════════════

        // ✅ v1.0.17: 验证 global_vault 已正确初始化（owner = program_id）
        require!(
            self.global_vault.owner == &crate::ID,
            PredictionMarketError::InvalidAuthority
        );

        // 检查合约是否暂停
        require!(
            !self.global_config.is_paused,
            PredictionMarketError::ContractPaused
        );

        // 🔒 安全修复：市场已结算后，必须等待 settle_pool 完成才能提取
        // 防止LP在用户claim_rewards前抢先提走pool_collateral_reserve
        // 一旦 pool_settled=true，说明失败方代币已被清理，LP 可以安全提取
        //
        // ⚠️ LP 风险提示（v1.0.17）：
        //   - LP 做市的收益来自手续费（platform_fee + lp_fee）和价格滑点
        //   - 但同时承担结算时的赔付义务
        //   - claim_rewards 会从 pool_collateral_reserve 支付 swap 用户的奖励
        //   - 这是 AMM/LMSR 预测市场的标准设计
        //   - 建议：等待大部分用户 claim 后再提现，避免流动性不足
        require!(
            !self.market.is_completed || self.market.pool_settled,
            PredictionMarketError::MarketResolvedLpLocked
        );

        // 验证金额有效
        require!(
            lp_shares_to_burn > 0,
            PredictionMarketError::InvalidAmount
        );

        // 验证用户有足够的 LP 份额
        require!(
            self.lp_position.lp_shares >= lp_shares_to_burn,
            PredictionMarketError::InsufficientBalance
        );

        // 验证市场 total_lp_shares 有效
        require!(
            self.market.total_lp_shares > 0,
            PredictionMarketError::InsufficientLiquidity
        );

        // ═══════════════════════════════════════════════════════════
        // 🔒 P0 修复：提现前自动结算 LP 费用（防止收益流失）
        // ═══════════════════════════════════════════════════════════

        // 计算自上次领取以来的累计收益
        let fee_per_share_delta = self.market.fee_per_share_cumulative
            .checked_sub(self.lp_position.last_fee_per_share)
            .ok_or(PredictionMarketError::MathOverflow)?;

        if fee_per_share_delta > 0 {
            // 计算该 LP 的应计费用
            let claimable_fees = (self.lp_position.lp_shares as u128)
                .checked_mul(fee_per_share_delta)
                .ok_or(PredictionMarketError::MathOverflow)?
                .checked_div(1_000_000_000_000_000_000) // 除以 10^18
                .ok_or(PredictionMarketError::MathOverflow)?;

            if claimable_fees > 0 && claimable_fees <= u64::MAX as u128 {
                let fees_amount = claimable_fees as u64;

                // ✅ v1.1.0: CRITICAL FIX - 使用 USDC 转账而非 SOL lamports
                //
                // 原错误逻辑 (v1.0.8-v1.0.31):
                //   检查 global_vault.lamports()
                //   使用 try_borrow_mut_lamports() 操作 SOL
                //   ❌ 但在 v1.1+ 中，LP 费用存放在 global_usdc_vault (SPL-USDC)
                //
                // 问题: 手续费已全部在 USDC 金库，但代码仍检查 SOL 余额
                //       导致 LP 无法提取流动性（一旦有应计手续费就会失败）
                //
                // 修复: 改用与 claim_lp_fees 一致的 USDC 转账流程
                //       从 global_usdc_vault 转账到 user_usdc_ata

                // 检查 USDC 余额充足
                let vault_usdc_balance = self.global_usdc_vault.amount;
                require!(
                    vault_usdc_balance >= fees_amount,
                    PredictionMarketError::InsufficientLiquidity
                );

                require!(
                    self.market.accumulated_lp_fees >= fees_amount,
                    PredictionMarketError::InsufficientBalance
                );

                // 使用 USDC token::transfer 转移费用给 LP
                let signer_seeds: &[&[&[u8]]] = &[&[
                    crate::constants::GLOBAL.as_bytes(),
                    &[global_vault_bump],
                ]];

                anchor_spl::token::transfer(
                    CpiContext::new_with_signer(
                        self.token_program.to_account_info(),
                        anchor_spl::token::Transfer {
                            from: self.global_usdc_vault.to_account_info(),
                            to: self.user_usdc_ata.to_account_info(),
                            authority: self.global_vault.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    fees_amount,
                )?;

                // 更新市场累计费用
                self.market.accumulated_lp_fees = self.market.accumulated_lp_fees
                    .checked_sub(fees_amount)
                    .ok_or(PredictionMarketError::MathOverflow)?;

                // ✅ 只有在成功转账后才更新 last_fee_per_share
                self.lp_position.last_fee_per_share = self.market.fee_per_share_cumulative;

                msg!("✅ Auto-settled LP fees before withdrawal: {} USDC (smallest units)", fees_amount);
            }
        }

        // ═══════════════════════════════════════════════════════════
        // 2. 计算按比例返还的资产数量
        // ═══════════════════════════════════════════════════════════

        // 计算用户份额占比：user_shares / total_shares
        // 返还资产 = pool_reserve * (lp_shares_to_burn / total_lp_shares)

        let usdc_to_return = (self.market.pool_collateral_reserve as u128)
            .checked_mul(lp_shares_to_burn as u128)
            .ok_or(PredictionMarketError::MathOverflow)?
            .checked_div(self.market.total_lp_shares as u128)
            .ok_or(PredictionMarketError::MathOverflow)?;

        let yes_to_return = (self.market.pool_yes_reserve as u128)
            .checked_mul(lp_shares_to_burn as u128)
            .ok_or(PredictionMarketError::MathOverflow)?
            .checked_div(self.market.total_lp_shares as u128)
            .ok_or(PredictionMarketError::MathOverflow)?;

        let no_to_return = (self.market.pool_no_reserve as u128)
            .checked_mul(lp_shares_to_burn as u128)
            .ok_or(PredictionMarketError::MathOverflow)?
            .checked_div(self.market.total_lp_shares as u128)
            .ok_or(PredictionMarketError::MathOverflow)?;

        require!(
            usdc_to_return <= u64::MAX as u128,
            PredictionMarketError::MathOverflow
        );
        require!(
            yes_to_return <= u64::MAX as u128,
            PredictionMarketError::MathOverflow
        );
        require!(
            no_to_return <= u64::MAX as u128,
            PredictionMarketError::MathOverflow
        );

        let usdc_amount = usdc_to_return as u64;
        let yes_amount = yes_to_return as u64;
        let no_amount = no_to_return as u64;

        msg!(
            "Calculated return amounts: usdc={}, yes={}, no={}",
            usdc_amount,
            yes_amount,
            no_amount
        );

        // 验证 Pool 有足够的储备
        require!(
            self.market.pool_collateral_reserve >= usdc_amount,
            PredictionMarketError::InsufficientLiquidity
        );
        require!(
            self.market.pool_yes_reserve >= yes_amount,
            PredictionMarketError::InsufficientLiquidity
        );
        require!(
            self.market.pool_no_reserve >= no_amount,
            PredictionMarketError::InsufficientLiquidity
        );

        // ═══════════════════════════════════════════════════════════
        // 3. 转移资产：Pool → 用户
        // ═══════════════════════════════════════════════════════════

        // PDA 签名种子
        let signer_seeds: &[&[&[u8]]] = &[&[
            crate::constants::GLOBAL.as_bytes(),
            &[global_vault_bump],
        ]];

        // ✅ v1.1.0: 3.1 转移 USDC 到用户
        if usdc_amount > 0 {
            // ✅ v1.1.0: USDC 金库最小余额校验
            let usdc_balance = self.global_usdc_vault.amount;
            require!(
                usdc_balance >= usdc_amount,
                PredictionMarketError::InsufficientLiquidity
            );

            let remaining_balance = usdc_balance
                .checked_sub(usdc_amount)
                .ok_or(PredictionMarketError::MathOverflow)?;

            require!(
                remaining_balance >= self.global_config.usdc_vault_min_balance,
                PredictionMarketError::InsufficientBalance
            );

            msg!("Transferring {} USDC from vault to user", usdc_amount);
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
                usdc_amount,
            )?;
        }

        // 3.2 转移 YES 代币到用户
        if yes_amount > 0 {
            msg!("Transferring {} YES tokens from pool to user", yes_amount);
            token::transfer(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    token::Transfer {
                        from: self.global_yes_ata.to_account_info(),
                        to: self.user_yes_ata.to_account_info(),
                        authority: self.global_vault.to_account_info(),
                    },
                    signer_seeds,
                ),
                yes_amount,
            )?;
        }

        // 3.3 转移 NO 代币到用户
        if no_amount > 0 {
            msg!("Transferring {} NO tokens from pool to user", no_amount);
            token::transfer(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    token::Transfer {
                        from: self.global_no_ata.to_account_info(),
                        to: self.user_no_ata.to_account_info(),
                        authority: self.global_vault.to_account_info(),
                    },
                    signer_seeds,
                ),
                no_amount,
            )?;
        }

        // ═══════════════════════════════════════════════════════════
        // 4. 更新 Pool Ledger（Market 状态）
        // ═══════════════════════════════════════════════════════════

        self.market.pool_collateral_reserve = self
            .market
            .pool_collateral_reserve
            .checked_sub(usdc_amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        self.market.pool_yes_reserve = self
            .market
            .pool_yes_reserve
            .checked_sub(yes_amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        self.market.pool_no_reserve = self
            .market
            .pool_no_reserve
            .checked_sub(no_amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        self.market.total_lp_shares = self
            .market
            .total_lp_shares
            .checked_sub(lp_shares_to_burn)
            .ok_or(PredictionMarketError::MathOverflow)?;

        msg!(
            "Updated Pool Ledger: collateral={}, yes={}, no={}, total_lp={}",
            self.market.pool_collateral_reserve,
            self.market.pool_yes_reserve,
            self.market.pool_no_reserve,
            self.market.total_lp_shares
        );

        // ═══════════════════════════════════════════════════════════
        // 5. 销毁 LP 份额（更新 LPPosition）
        // ═══════════════════════════════════════════════════════════

        self.lp_position.lp_shares = self
            .lp_position
            .lp_shares
            .checked_sub(lp_shares_to_burn)
            .ok_or(PredictionMarketError::MathOverflow)?;

        msg!(
            "Burned {} LP shares from user. Remaining user shares: {}",
            lp_shares_to_burn,
            self.lp_position.lp_shares
        );

        // ═══════════════════════════════════════════════════════════════
        // ✅ 发射提取流动性事件
        // ═══════════════════════════════════════════════════════════════
        let clock = Clock::get()?;
        emit!(WithdrawLiquidityEvent {
            user: self.user.key(),
            market: self.market.key(),
            lp_shares_burned: lp_shares_to_burn,
            usdc_amount,  // ✅ v1.1.0: 字段名改为 usdc_amount
            yes_amount,
            no_amount,
            timestamp: clock.unix_timestamp,
        });

        msg!("WithdrawLiquidity completed successfully");
        Ok(())
    }
}