use crate::state::config::*;

use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};
use anchor_spl::token::{Mint, Token, TokenAccount};

// use anchor_spl::token::{self};

#[account]
pub struct UserInfo {
    pub user: Pubkey,     // User's public key
    // ✅ FIX CRITICAL-2: 删除冗余的余额字段
    // yes_balance 和 no_balance 是冗余的，因为 SPL Token ATA 已经追踪余额
    // 保留单一真相来源：用户的 ATA 余额
    // pub yes_balance: u64, // ❌ REMOVED
    // pub no_balance: u64,  // ❌ REMOVED
    pub is_lp: bool,
    pub is_initialized: bool,
}

/// LP Position（LP 持仓信息）
/// 用于新的 LP Token 系统
#[account]
pub struct LPPosition {
    pub user: Pubkey,           // LP 用户
    pub market: Pubkey,          // 所属市场
    pub lp_shares: u64,          // LP 份额
    pub deposited_sol: u64,      // 存入的 SOL 数量（记录）
    pub deposited_yes: u64,      // 存入的 YES 数量（记录）
    pub deposited_no: u64,       // 存入的 NO 数量（记录）
    pub last_fee_claim_slot: u64, // 上次领取费用的 slot（已废弃，改用 last_fee_per_share）
    /// ✅ 上次领取时的 fee_per_share 值（用于计算未领取的费用）
    pub last_fee_per_share: u128, // 精度：* 10^18
}

/// ✅ v1.0.12: Swap 交易结果
/// 用于在 swap 函数和事件发射之间传递详细交易数据
/// ✅ v1.1.0: 更新为 USDC 字段名
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct SwapResult {
    pub usdc_amount: u64,       // ✅ v1.1.0: 实际的 USDC 数量（买单=输入，卖单=输出税后）
    pub token_amount: u64,      // 实际的代币数量（买单=输出，卖单=输入）
    pub fee_usdc: u64,          // ✅ v1.1.0: 总手续费（platform_fee + lp_fee，USDC）
}

#[account]
pub struct Market {
    pub yes_token_mint: Pubkey,
    pub no_token_mint: Pubkey,

    pub creator: Pubkey,

    // ═══════════════════════════════════════════════════════════════
    // Settlement Ledger（结算账本）
    // 用于 mint_complete_set / redeem_complete_set / claim_rewards
    // ═══════════════════════════════════════════════════════════════

    /// 条件代币的 1:1 抵押品锁定量
    pub total_collateral_locked: u64,

    /// 通过 mint_complete_set 创建的 YES 代币总量
    pub total_yes_minted: u64,

    /// 通过 mint_complete_set 创建的 NO 代币总量
    pub total_no_minted: u64,

    // ═══════════════════════════════════════════════════════════════
    // AMM Pool Ledger（池子账本）
    // 用于 add_liquidity / withdraw_liquidity / swap
    // ═══════════════════════════════════════════════════════════════

    /// Pool 中的 SOL 储备金（流动性）
    pub pool_collateral_reserve: u64,

    /// Pool 中的 YES 代币库存（用于 swap）
    pub pool_yes_reserve: u64,

    /// Pool 中的 NO 代币库存（用于 swap）
    pub pool_no_reserve: u64,

    /// LP Token 总供应量
    pub total_lp_shares: u64,

    // ═══════════════════════════════════════════════════════════════
    // LMSR 定价参数（用于 Pool）
    // ═══════════════════════════════════════════════════════════════

    /// 流动性参数（决定市场深度）
    pub lmsr_b: u64,

    /// YES 的净持仓量（用于 LMSR 定价）
    pub lmsr_q_yes: i64,

    /// NO 的净持仓量（用于 LMSR 定价）
    pub lmsr_q_no: i64,

    // ═══════════════════════════════════════════════════════════════
    // 已废弃字段（保留以保持向后兼容）
    // ═══════════════════════════════════════════════════════════════

    pub initial_yes_token_reserves: u64,
    pub real_yes_token_reserves: u64,
    pub real_yes_sol_reserves: u64,
    pub token_yes_total_supply: u64,

    pub initial_no_token_reserves: u64,
    pub real_no_token_reserves: u64,
    pub real_no_sol_reserves: u64,
    pub token_no_total_supply: u64,

    // ═══════════════════════════════════════════════════════════════
    // 市场状态
    // ═══════════════════════════════════════════════════════════════

    pub is_completed: bool,
    pub start_slot: Option<u64>,
    pub ending_slot: Option<u64>,

    /// Resolution 结算参数
    pub resolution_yes_ratio: u64,  // YES代币赎回比例（基点，10000=100%）
    pub resolution_no_ratio: u64,   // NO代币赎回比例（基点，10000=100%）
    pub winner_token_type: u8,      // 获胜方（0=NO, 1=YES, 2=平局）

    /// 重入保护标志
    pub swap_in_progress: bool,

    // ═══════════════════════════════════════════════════════════════
    // LP 管理（已废弃，改用 LPPosition）
    // ═══════════════════════════════════════════════════════════════

    /// LP 累计费用（总额）
    pub accumulated_lp_fees: u64,

    /// ✅ 累计每份额收益（用于公平分配 LP 费用）
    /// 公式：fee_per_share_cumulative += new_fees / total_lp_shares
    /// 精度：使用 u128 存储，实际值 * 10^18
    pub fee_per_share_cumulative: u128,

    // ═══════════════════════════════════════════════════════════════
    // 🔒 v1.0.5+ 新增字段（追加到末尾以保持向后兼容）
    // ═══════════════════════════════════════════════════════════════

    /// Pool 结算标志（settle_pool 完成后设为 true）
    /// 用于允许 LP 在市场完成后安全提取流动性
    /// ⚠️ 重要：此字段追加到结构体末尾，旧账户需迁移
    pub pool_settled: bool,
}

#[derive(Debug, Clone)]
pub struct SellResult {
    pub token_amount: u64,
    pub change_amount: u64,
    pub current_yes_reserves: u64,
    pub current_no_reserves: u64,
    pub new_yes_reserves: u64,
    pub new_no_reserves: u64,
}

#[derive(Debug, Clone)]
pub struct BuyResult {
    pub token_amount: u64,
    pub change_amount: u64,
    pub current_yes_reserves: u64,
    pub current_no_reserves: u64,
    pub new_yes_reserves: u64,
    pub new_no_reserves: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateMarketParams {
    pub yes_symbol: String,
    pub yes_uri: String,

    pub start_slot: Option<u64>,
    pub ending_slot: Option<u64>,
}
pub trait MarketAccount<'info> {
    #[allow(clippy::too_many_arguments)]
    fn swap(
        &mut self,
        global_config: &Account<'info, Config>,

        yes_token_mint: &Account<'info, Mint>,
        _global_yes_ata: &mut AccountInfo<'info>,
        user_yes_ata: &mut AccountInfo<'info>,

        no_token_mint: &Account<'info, Mint>,
        _global_no_ata: &mut AccountInfo<'info>,
        user_no_ata: &mut AccountInfo<'info>,

        source: &mut AccountInfo<'info>,
        _team_wallet: &mut AccountInfo<'info>,  // ✅ v1.1.0: 不再使用（改用 team_usdc_ata）

        amount: u64,
        direction: u8,
        token_type: u8,
        minimum_receive_amount: u64,

        user: &Signer<'info>,
        signer: &[&[&[u8]]],

        user_info_pda: &mut Account<'info, UserInfo>,

        token_program: &Program<'info, Token>,
        _system_program: &Program<'info, System>,

        // ✅ v1.1.0: USDC 相关账户
        _usdc_mint: &Account<'info, Mint>,
        global_usdc_vault: &Account<'info, TokenAccount>,
        user_usdc_ata: &Account<'info, TokenAccount>,
        team_usdc_ata: &Account<'info, TokenAccount>,
    ) -> Result<SwapResult>;

    fn apply_buy(&mut self, sol_amount: u64, token_type: u8) -> Option<BuyResult>;

    fn apply_sell(&mut self, token_amount: u64, token_type: u8) -> Option<SellResult>;

    fn get_tokens_for_buy_sol(&self, sol_amount: u64, token_type: u8) -> Option<BuyResult>;

    fn get_tokens_for_sell_sol(&self, token_amount: u64, token_type: u8) -> Option<SellResult>;

    /// ⚠️ DEPRECATED (v1.1.0): 该方法已废弃，空实现
    /// 实际结算逻辑在 instructions/market/resolution.rs::Resolution::handler
    /// 保留仅为了 trait 完整性，建议未来版本删除整个 trait 方法
    fn resolution(
        &mut self,

        source: &mut AccountInfo<'info>,

        user: &mut AccountInfo<'info>,
        signer: &[&[&[u8]]],
        user_info_pda: &mut Account<'info, UserInfo>,

        token_type: u8,

        system_program: &Program<'info, System>,
    ) -> Result<()>;
}

impl<'info> MarketAccount<'info> for Account<'info, Market> {
    #[allow(clippy::too_many_arguments)]
    fn swap(
        &mut self,
        global_config: &Account<'info, Config>,

        _yes_token_mint: &Account<'info, Mint>,
        _global_yes_ata: &mut AccountInfo<'info>,
        user_yes_ata: &mut AccountInfo<'info>,

        _no_token_mint: &Account<'info, Mint>,
        _global_no_ata: &mut AccountInfo<'info>,
        user_no_ata: &mut AccountInfo<'info>,

        source: &mut AccountInfo<'info>,
        _team_wallet: &mut AccountInfo<'info>,  // ✅ v1.1.0: 不再使用（改用 team_usdc_ata）

        amount: u64,
        direction: u8,
        token_type: u8,
        minimum_receive_amount: u64,

        user: &Signer<'info>,
        signer: &[&[&[u8]]],

        _user_info_pda: &mut Account<'info, UserInfo>,

        token_program: &Program<'info, Token>,
        _system_program: &Program<'info, System>,

        // ✅ v1.1.0: USDC 相关账户
        _usdc_mint: &Account<'info, Mint>,
        global_usdc_vault: &Account<'info, TokenAccount>,
        user_usdc_ata: &Account<'info, TokenAccount>,
        team_usdc_ata: &Account<'info, TokenAccount>,
    ) -> Result<SwapResult> {
        use anchor_spl::token;

        msg!("Swap: direction={}, token_type={}, amount={}", direction, token_type, amount);

        // ✅ FIX CRITICAL-1: 所有验证必须在状态修改之前完成
        // 1. 先做所有基础验证
        require!(!self.swap_in_progress, crate::errors::PredictionMarketError::InvalidParameter);
        require!(!self.is_completed, crate::errors::PredictionMarketError::CurveAlreadyCompleted);
        require!(amount > 0, crate::errors::PredictionMarketError::InvalidAmount);
        require!(token_type <= 1, crate::errors::PredictionMarketError::InvalidParameter);
        require!(direction <= 1, crate::errors::PredictionMarketError::InvalidParameter);

        // 🔒 P0 修复：校验市场交易时间窗口
        let current_slot = Clock::get()?.slot;

        // 校验市场已开始
        if let Some(start_slot) = self.start_slot {
            require!(
                current_slot >= start_slot,
                crate::errors::PredictionMarketError::MarketNotStarted
            );
        }

        // 校验市场未结束
        if let Some(ending_slot) = self.ending_slot {
            require!(
                current_slot < ending_slot,
                crate::errors::PredictionMarketError::MarketEnded
            );
        }

        // 2. 验证通过后才修改状态
        self.swap_in_progress = true;

        // ⚠️ CRITICAL FIX: 使用包装器确保标志一定被重置
        // 将核心逻辑的结果捕获，无论成功失败都重置标志
        // ✅ v1.0.12: 修改返回类型为 SwapResult 以支持准确的事件发射
        let swap_result = (|| -> Result<SwapResult> {
            // direction: 0 = buy, 1 = sell
            // token_type: 0 = NO, 1 = YES

            if direction == 0 {
            // ═══════════════════════════════════════════════════════════
            // BUY 操作：用 USDC 从 Pool 买代币
            // ✅ 双账本系统：只操作 Pool Ledger，不影响 Settlement
            // ═══════════════════════════════════════════════════════════
            msg!("Processing BUY order (Pool)");

            // 计算手续费
            let platform_fee = amount
                .checked_mul(global_config.platform_buy_fee)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?
                .checked_div(10000)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

            let lp_fee = amount
                .checked_mul(global_config.lp_buy_fee)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?
                .checked_div(10000)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

            let total_fee = platform_fee.checked_add(lp_fee)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

            let amount_after_fee = amount.checked_sub(total_fee)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

            msg!("Fees - platform: {}, lp: {}, net amount: {}", platform_fee, lp_fee, amount_after_fee);

            // 计算可获得的代币数量（使用AMM公式）
            let buy_result = self.apply_buy(amount_after_fee, token_type)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

            // 检查滑点保护
            require!(
                buy_result.token_amount >= minimum_receive_amount,
                crate::errors::PredictionMarketError::SlippageExceeded
            );

            msg!("Token amount to receive: {}", buy_result.token_amount);

            // ✅ v1.1.0: 用户转 USDC 到全局 USDC 金库（扣除平台费后的金额 + LP费）
            let usdc_to_vault = amount_after_fee.checked_add(lp_fee)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

            token::transfer(
                CpiContext::new(
                    token_program.to_account_info(),
                    token::Transfer {
                        from: user_usdc_ata.to_account_info(),
                        to: global_usdc_vault.to_account_info(),
                        authority: user.to_account_info(),
                    },
                ),
                usdc_to_vault,
            )?;

            // ✅ v1.1.0: 平台手续费（USDC）转给团队钱包
            if platform_fee > 0 {
                token::transfer(
                    CpiContext::new(
                        token_program.to_account_info(),
                        token::Transfer {
                            from: user_usdc_ata.to_account_info(),
                            to: team_usdc_ata.to_account_info(),
                            authority: user.to_account_info(),
                        },
                    ),
                    platform_fee,
                )?;
            }

            // ✅ 累计 LP 费用到 Pool
            self.accumulated_lp_fees = self.accumulated_lp_fees
                .checked_add(lp_fee)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

            // ✅ 更新累计每份额收益（公平分配关键）
            if self.total_lp_shares > 0 && lp_fee > 0 {
                // fee_per_share_cumulative += (lp_fee * 10^18) / total_lp_shares
                let fee_per_share_increase = (lp_fee as u128)
                    .checked_mul(1_000_000_000_000_000_000) // 10^18 精度
                    .ok_or(crate::errors::PredictionMarketError::MathOverflow)?
                    .checked_div(self.total_lp_shares as u128)
                    .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

                self.fee_per_share_cumulative = self.fee_per_share_cumulative
                    .checked_add(fee_per_share_increase)
                    .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;
            }

            // ═══════════════════════════════════════════════════════════
            // ✅ 双账本系统：只操作 Pool Ledger
            // ═══════════════════════════════════════════════════════════

            // 1. Pool 收到 USDC（增加储备金）
            self.pool_collateral_reserve = self.pool_collateral_reserve
                .checked_add(amount_after_fee)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

            // 2. 从 Pool 转移代币给用户（减少储备）
            if token_type == 0 {
                // 买 NO 代币
                require!(
                    self.pool_no_reserve >= buy_result.token_amount,
                    crate::errors::PredictionMarketError::InsufficientLiquidity
                );

                self.pool_no_reserve = self.pool_no_reserve
                    .checked_sub(buy_result.token_amount)
                    .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

                token::transfer(
                    CpiContext::new_with_signer(
                        token_program.to_account_info(),
                        token::Transfer {
                            from: _global_no_ata.to_account_info(),
                            to: user_no_ata.to_account_info(),
                            authority: source.to_account_info(),
                        },
                        signer,
                    ),
                    buy_result.token_amount,
                )?;

                msg!(
                    "✅ Pool: Sold {} NO to user for {} USDC. Reserves: USDC={}, NO={}",
                    buy_result.token_amount,
                    amount_after_fee,
                    self.pool_collateral_reserve,
                    self.pool_no_reserve
                );
            } else {
                // 买 YES 代币
                require!(
                    self.pool_yes_reserve >= buy_result.token_amount,
                    crate::errors::PredictionMarketError::InsufficientLiquidity
                );

                self.pool_yes_reserve = self.pool_yes_reserve
                    .checked_sub(buy_result.token_amount)
                    .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

                token::transfer(
                    CpiContext::new_with_signer(
                        token_program.to_account_info(),
                        token::Transfer {
                            from: _global_yes_ata.to_account_info(),
                            to: user_yes_ata.to_account_info(),
                            authority: source.to_account_info(),
                        },
                        signer,
                    ),
                    buy_result.token_amount,
                )?;

                msg!(
                    "✅ Pool: Sold {} YES to user for {} USDC. Reserves: USDC={}, YES={}",
                    buy_result.token_amount,
                    amount_after_fee,
                    self.pool_collateral_reserve,
                    self.pool_yes_reserve
                );
            }

            // 注意：不修改 Settlement Ledger 的字段：
            // - total_collateral_locked (不变)
            // - total_yes_minted (不变)
            // - total_no_minted (不变)

            msg!("BUY completed (Pool only, Settlement unchanged)");

            // ✅ v1.0.12: 返回准确的交易数据
            // ✅ v1.1.0: 更新为 USDC 字段名
            Ok(SwapResult {
                usdc_amount: amount_after_fee,      // ✅ v1.1.0: 用户支付的 USDC（税后）
                token_amount: buy_result.token_amount, // 用户获得的代币
                fee_usdc: total_fee,                // ✅ v1.1.0: 总手续费（USDC）
            })

        } else {
            // ═══════════════════════════════════════════════════════════
            // SELL 操作：卖代币换 USDC
            // ✅ 双账本系统：只操作 Pool Ledger
            // ═══════════════════════════════════════════════════════════
            msg!("Processing SELL order (Pool)");

            // 计算可获得的 USDC（使用 LMSR 公式）
            let sell_result = self.apply_sell(amount, token_type)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

            // 计算手续费
            let platform_fee = sell_result.change_amount
                .checked_mul(global_config.platform_sell_fee)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?
                .checked_div(10000)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

            let lp_fee = sell_result.change_amount
                .checked_mul(global_config.lp_sell_fee)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?
                .checked_div(10000)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

            let total_fee = platform_fee.checked_add(lp_fee)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

            let amount_after_fee = sell_result.change_amount.checked_sub(total_fee)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

            // 检查滑点保护
            require!(
                amount_after_fee >= minimum_receive_amount,
                crate::errors::PredictionMarketError::SlippageExceeded
            );

            // ✅ 验证 Pool 有足够的 USDC 储备
            require!(
                self.pool_collateral_reserve >= sell_result.change_amount,
                crate::errors::PredictionMarketError::InsufficientLiquidity
            );

            msg!("Selling {} tokens for {} USDC (after fee)", amount, amount_after_fee);

            // ═══════════════════════════════════════════════════════════
            // ✅ 双账本系统：只操作 Pool Ledger
            // ═══════════════════════════════════════════════════════════

            // 1. Pool 收到代币（增加储备）
            if token_type == 0 {
                // 卖 NO 代币
                self.pool_no_reserve = self.pool_no_reserve
                    .checked_add(amount)
                    .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

                token::transfer(
                    CpiContext::new(
                        token_program.to_account_info(),
                        token::Transfer {
                            from: user_no_ata.to_account_info(),
                            to: _global_no_ata.to_account_info(),
                            authority: user.to_account_info(),
                        },
                    ),
                    amount,
                )?;

                msg!(
                    "✅ Pool: Bought {} NO from user for {} USDC. Reserves: USDC={}, NO={}",
                    amount,
                    sell_result.change_amount,
                    self.pool_collateral_reserve,
                    self.pool_no_reserve
                );
            } else {
                // 卖 YES 代币
                self.pool_yes_reserve = self.pool_yes_reserve
                    .checked_add(amount)
                    .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

                token::transfer(
                    CpiContext::new(
                        token_program.to_account_info(),
                        token::Transfer {
                            from: user_yes_ata.to_account_info(),
                            to: _global_yes_ata.to_account_info(),
                            authority: user.to_account_info(),
                        },
                    ),
                    amount,
                )?;

                msg!(
                    "✅ Pool: Bought {} YES from user for {} USDC. Reserves: USDC={}, YES={}",
                    amount,
                    sell_result.change_amount,
                    self.pool_collateral_reserve,
                    self.pool_yes_reserve
                );
            }

            // 2. Pool 支付 USDC（减少储备）
            self.pool_collateral_reserve = self.pool_collateral_reserve
                .checked_sub(sell_result.change_amount)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

            // ✅ v1.1.0: 从全局 USDC 金库转 USDC 给用户（扣除手续费后）
            token::transfer(
                CpiContext::new_with_signer(
                    token_program.to_account_info(),
                    token::Transfer {
                        from: global_usdc_vault.to_account_info(),
                        to: user_usdc_ata.to_account_info(),
                        authority: source.to_account_info(),
                    },
                    signer,
                ),
                amount_after_fee,
            )?;

            // ✅ v1.1.0: 转平台手续费（USDC）给团队钱包
            if platform_fee > 0 {
                token::transfer(
                    CpiContext::new_with_signer(
                        token_program.to_account_info(),
                        token::Transfer {
                            from: global_usdc_vault.to_account_info(),
                            to: team_usdc_ata.to_account_info(),
                            authority: source.to_account_info(),
                        },
                        signer,
                    ),
                    platform_fee,
                )?;
            }

            // ✅ 累计 LP 费用（卖出时 lp_fee 留在金库中）
            self.accumulated_lp_fees = self.accumulated_lp_fees
                .checked_add(lp_fee)
                .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

            // ✅ 更新累计每份额收益（公平分配关键）
            if self.total_lp_shares > 0 && lp_fee > 0 {
                let fee_per_share_increase = (lp_fee as u128)
                    .checked_mul(1_000_000_000_000_000_000) // 10^18 精度
                    .ok_or(crate::errors::PredictionMarketError::MathOverflow)?
                    .checked_div(self.total_lp_shares as u128)
                    .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

                self.fee_per_share_cumulative = self.fee_per_share_cumulative
                    .checked_add(fee_per_share_increase)
                    .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;
            }

            // 注意：不修改 Settlement Ledger 的字段：
            // - total_collateral_locked (不变)
            // - total_yes_minted (不变)
            // - total_no_minted (不变)

            msg!("SELL completed (Pool only, Settlement unchanged)");

            // ✅ v1.0.12: 返回准确的交易数据
            // ✅ v1.1.0: 更新为 USDC 字段名
            Ok(SwapResult {
                usdc_amount: amount_after_fee,     // ✅ v1.1.0: 用户获得的 USDC（税后）
                token_amount: amount,              // 用户卖出的代币数量
                fee_usdc: total_fee,               // ✅ v1.1.0: 总手续费（USDC）
            })
            }
        })(); // 立即执行闭包

        // ✅ FIX REENTRANCY LOCK: 无论成功或失败，都重置标志
        // 这确保即使上面的逻辑通过 ? 或 require! 失败返回，标志也会被重置
        self.swap_in_progress = false;

        // 返回闭包的结果（成功或错误）
        swap_result
    }

    fn get_tokens_for_buy_sol(&self, sol_amount: u64, token_type: u8) -> Option<BuyResult> {
        // ✅ v1.0.10: 使用定点 LMSR 定价算法
        // LMSR Cost Function: C(q) = b * ln(e^(q_yes/b) + e^(q_no/b))
        // Price: P(YES) = e^(q_yes/b) / (e^(q_yes/b) + e^(q_no/b))

        let b = self.lmsr_b;
        let q_yes = self.lmsr_q_yes;
        let q_no = self.lmsr_q_no;

        // 计算能买多少代币（使用定点数二分法）
        let token_amount = if token_type == 0 {
            // 买NO代币：增加q_no
            self.lmsr_calculate_token_amount_for_sol(sol_amount, q_yes, q_no, b, false).ok()?
        } else {
            // 买YES代币：增加q_yes
            self.lmsr_calculate_token_amount_for_sol(sol_amount, q_yes, q_no, b, true).ok()?
        };

        Some(BuyResult {
            token_amount,
            change_amount: sol_amount,
            current_yes_reserves: 0, // LMSR不使用reserves
            current_no_reserves: 0,
            new_yes_reserves: 0,
            new_no_reserves: 0,
        })
    }

    fn apply_buy(&mut self, change_amount: u64, token_type: u8) -> Option<BuyResult> {
        // ✅ 使用LMSR计算代币数量
        let result = self.get_tokens_for_buy_sol(change_amount, token_type)?;

        // ✅ 更新LMSR状态（q_yes 或 q_no）
        if token_type == 0 {
            // 买NO代币：增加q_no
            self.lmsr_q_no = self.lmsr_q_no.checked_add(result.token_amount as i64)?;
            // ✅ 验证q值在安全范围内
            if self.lmsr_q_no.abs() > crate::constants::MAX_Q_VALUE {
                return None;
            }
        } else {
            // 买YES代币：增加q_yes
            self.lmsr_q_yes = self.lmsr_q_yes.checked_add(result.token_amount as i64)?;
            // ✅ 验证q值在安全范围内
            if self.lmsr_q_yes.abs() > crate::constants::MAX_Q_VALUE {
                return None;
            }
        }

        Some(result)
    }

    fn apply_sell(&mut self, change_amount: u64, token_type: u8) -> Option<SellResult> {
        // ✅ 使用LMSR计算SOL数量
        let result = self.get_tokens_for_sell_sol(change_amount, token_type)?;

        // ✅ 更新LMSR状态（减少q_yes 或 q_no）
        if token_type == 0 {
            // 卖NO代币：减少q_no
            self.lmsr_q_no = self.lmsr_q_no.checked_sub(change_amount as i64)?;
            // ✅ 验证q值在安全范围内
            if self.lmsr_q_no.abs() > crate::constants::MAX_Q_VALUE {
                return None;
            }
        } else {
            // 卖YES代币：减少q_yes
            self.lmsr_q_yes = self.lmsr_q_yes.checked_sub(change_amount as i64)?;
            // ✅ 验证q值在安全范围内
            if self.lmsr_q_yes.abs() > crate::constants::MAX_Q_VALUE {
                return None;
            }
        }

        Some(result)
    }

    fn get_tokens_for_sell_sol(&self, token_amount: u64, token_type: u8) -> Option<SellResult> {
        // ✅ v1.0.10: 使用定点 LMSR 定价算法
        let b = self.lmsr_b;
        let q_yes = self.lmsr_q_yes;
        let q_no = self.lmsr_q_no;

        // 计算卖出token_amount代币能获得多少SOL（使用定点数）
        let sol_amount = if token_type == 0 {
            // 卖NO代币：减少q_no
            self.lmsr_calculate_sol_for_token_amount(token_amount, q_yes, q_no, b, false).ok()?
        } else {
            // 卖YES代币：减少q_yes
            self.lmsr_calculate_sol_for_token_amount(token_amount, q_yes, q_no, b, true).ok()?
        };

        Some(SellResult {
            token_amount,
            change_amount: sol_amount,
            current_yes_reserves: 0, // LMSR不使用reserves
            current_no_reserves: 0,
            new_yes_reserves: 0,
            new_no_reserves: 0,
        })
    }

    /// ⚠️ DEPRECATED (v1.1.0): 空实现，实际不执行任何操作
    /// 实际结算功能已在 instructions/market/resolution.rs 中实现
    fn resolution(
        &mut self,

        _source: &mut AccountInfo<'info>,

        _user: &mut AccountInfo<'info>,
        _signer: &[&[&[u8]]],
        _user_info_pda: &mut Account<'info, UserInfo>,

        _token_type: u8,

        _system_program: &Program<'info, System>,
    ) -> Result<()> {
        // 空实现：实际结算逻辑在 Resolution 指令中
        Ok(())
    }
}

// ✅ v1.0.10: 定点 LMSR 实现（替换 f64/exp/ln）
impl Market {
    /// LMSR成本函数: C(q) = b * ln(e^(q_yes/b) + e^(q_no/b))
    ///
    /// ✅ 使用定点数替代 f64，确保确定性和安全性
    pub fn lmsr_cost(&self, q_yes: i64, q_no: i64, b: u64) -> Result<u64> {
        crate::math::lmsr::lmsr_cost(b, q_yes, q_no)
    }

    /// 计算给定 USDC 能买多少代币
    ///
    /// ✅ 使用定点数二分法，最大迭代次数 50（Gas 限制）
    pub fn lmsr_calculate_token_amount_for_sol(
        &self,
        sol_amount: u64,
        q_yes: i64,
        q_no: i64,
        b: u64,
        is_yes: bool,
    ) -> Result<u64> {
        crate::math::lmsr::lmsr_tokens_for_usdc(b, q_yes, q_no, sol_amount, is_yes)
    }

    /// 计算卖出代币能获得多少 USDC
    ///
    /// ✅ 使用定点数二分法
    pub fn lmsr_calculate_sol_for_token_amount(
        &self,
        token_amount: u64,
        q_yes: i64,
        q_no: i64,
        b: u64,
        is_yes: bool,
    ) -> Result<u64> {
        crate::math::lmsr::lmsr_sell_payout(b, q_yes, q_no, token_amount, is_yes)
    }

    /// 获取当前YES边际价格
    ///
    /// ✅ 返回定点数价格（需要转换为百分比）
    pub fn lmsr_get_yes_price(&self) -> Result<crate::math::FixedPoint> {
        let b = self.lmsr_b;
        let q_yes = self.lmsr_q_yes;
        let q_no = self.lmsr_q_no;

        crate::math::lmsr::lmsr_marginal_price(b, q_yes, q_no)
    }

    /// 获取当前NO边际价格
    ///
    /// ✅ P(NO) = 1 - P(YES)
    pub fn lmsr_get_no_price(&self) -> Result<crate::math::FixedPoint> {
        let yes_price = self.lmsr_get_yes_price()?;
        let one = crate::math::fixed_point::constants::ONE;

        Ok(one.checked_sub(yes_price).ok_or(crate::errors::PredictionMarketError::MathOverflow)?)
    }
}
