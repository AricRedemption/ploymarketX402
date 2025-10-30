//! 市场指令：结算 Pool（市场结束后处理 Pool 资产）
//! ✅ 双账本系统：只操作 Pool Ledger

use crate::{
    constants::{CONFIG, GLOBAL, MARKET},
    errors::PredictionMarketError,
    events::SettlePoolEvent,
    state::{config::*, market::*},
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

/// 账户集合：Pool 结算所需账户
#[derive(Accounts)]
pub struct SettlePool<'info> {
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
        bump,
        constraint = market.is_completed @PredictionMarketError::MarketNotCompleted,
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

    /// 权限签名者（管理员）
    #[account(
        mut,
        constraint = global_config.authority == authority.key() @PredictionMarketError::IncorrectAuthority
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

impl<'info> SettlePool<'info> {
    /// 处理 Pool 结算
    ///
    /// ✅ 双账本系统：只操作 Pool Ledger
    ///
    /// 市场结束后，Pool 中剩余的代币需要处理：
    /// - 获胜方代币：保留在 Pool 中给 LP 提取（LP 可选择 claim_rewards 赎回或持有）
    /// - 失败方代币：**直接销毁**（v1.0.28 治理决策）
    ///   * 更加去中心化，避免中心化收入风险
    ///   * 失败方代币理论价值为0，销毁符合经济逻辑
    ///   * 减少总供应量，符合代币经济学原则
    /// - USDC储备：保留给 LP 提取（通过 withdraw_liquidity）
    ///
    /// # 参数
    /// * global_vault_bump: PDA bump seed
    pub fn handler(&mut self, global_vault_bump: u8) -> Result<()> {
        msg!("SettlePool handler start");

        // ═══════════════════════════════════════════════════════════
        // 1. 验证前置条件
        // ═══════════════════════════════════════════════════════════

        // 验证市场已完成
        require!(
            self.market.is_completed,
            PredictionMarketError::MarketNotCompleted
        );

        // 验证有获胜方设置（0=NO, 1=YES, 2=平局）
        require!(
            self.market.winner_token_type <= 2,
            PredictionMarketError::RESOLUTIONTOKEYTYPEERROR
        );

        // ═══════════════════════════════════════════════════════════
        // 2. 根据结算类型处理代币
        // ═══════════════════════════════════════════════════════════

        let signer_seeds: &[&[&[u8]]] = &[&[
            crate::constants::GLOBAL.as_bytes(),
            &[global_vault_bump],
        ]];

        // 保存输家代币数量用于事件发射
        let loser_tokens_transferred: u64;

        if self.market.winner_token_type == 2 {
            // ═══════════════════════════════════════════════════════════
            // 平局场景：两种代币都有价值，按比例保留
            // ═══════════════════════════════════════════════════════════

            loser_tokens_transferred = 0; // 平局没有输家

            msg!(
                "Draw scenario: YES ratio={}, NO ratio={}, preserving both tokens for LP withdrawal",
                self.market.resolution_yes_ratio,
                self.market.resolution_no_ratio
            );

            // 平局时，Pool 中的代币都保留给 LP 提取
            // LP 提取后可以通过 claim_rewards 按比例赎回 USDC
            // 不需要在这里做任何代币转移

            msg!(
                "Pool reserves preserved - YES: {}, NO: {}, USDC: {}",
                self.market.pool_yes_reserve,
                self.market.pool_no_reserve,
                self.market.pool_collateral_reserve
            );

        } else {
            // ═══════════════════════════════════════════════════════════
            // 单方获胜场景：处理获胜方和失败方代币
            // ═══════════════════════════════════════════════════════════

            let (winner_reserve, loser_reserve, loser_ata) =
                if self.market.winner_token_type == 0 {
                    // NO 方获胜
                    msg!("Winner: NO token, processing YES tokens as loser");
                    (
                        self.market.pool_no_reserve,
                        self.market.pool_yes_reserve,
                        &self.global_yes_ata,
                    )
                } else {
                    // YES 方获胜
                    msg!("Winner: YES token, processing NO tokens as loser");
                    (
                        self.market.pool_yes_reserve,
                        self.market.pool_no_reserve,
                        &self.global_no_ata,
                    )
                };

            msg!(
                "Pool reserves - Winner: {}, Loser: {}, USDC: {}",
                winner_reserve,
                loser_reserve,
                self.market.pool_collateral_reserve
            );

            // 保存输家代币数量用于事件发射
            loser_tokens_transferred = loser_reserve;

            // ═══════════════════════════════════════════════════════════
            // 3. 处理失败方代币：直接销毁
            // ═══════════════════════════════════════════════════════════
            //
            // ✅ 治理决策：失败方代币直接销毁（v1.0.28）
            //
            // 原因：
            // 1. 市场结算后，失败方代币理论上价值为0（根据结算比例）
            // 2. 销毁机制更加去中心化，避免中心化收入
            // 3. 减少代币总供应量，符合代币经济学原则
            // 4. 透明且不可逆，无中心化风险
            //
            // 经济影响：
            // - 失败方代币永久从流通中移除
            // - 不影响获胜方代币或 USDC 储备
            // - LP 仍可正常提取流动性和获胜方代币
            //
            // 披露：
            // - 销毁的代币数量会在 SettlePoolEvent 中公开
            // - 前端应向用户明确展示此机制

            if loser_reserve > 0 {
                msg!(
                    "Burning {} loser tokens from pool",
                    loser_reserve
                );

                token::burn(
                    CpiContext::new_with_signer(
                        self.token_program.to_account_info(),
                        token::Burn {
                            mint: if self.market.winner_token_type == 0 {
                                self.yes_token.to_account_info()
                            } else {
                                self.no_token.to_account_info()
                            },
                            from: loser_ata.to_account_info(),
                            authority: self.global_vault.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    loser_reserve,
                )?;

                // 更新 Pool Ledger：清空失败方代币储备
                if self.market.winner_token_type == 0 {
                    // NO 获胜，清空 YES
                    self.market.pool_yes_reserve = 0;
                } else {
                    // YES 获胜，清空 NO
                    self.market.pool_no_reserve = 0;
                }
            }

            // ═══════════════════════════════════════════════════════════
            // 4. 处理获胜方代币：保留在 Pool 中等待 LP 提取
            // ═══════════════════════════════════════════════════════════

            // 注意：获胜方代币暂时保留在 Pool 中
            // 当 LP 提取流动性时，会获得这些代币
            // LP 可以选择：
            // 1. 通过 claim_rewards 赎回这些代币（如果市场已结算）
            // 2. 或者直接持有（如果代币还有市场价值）

            msg!(
                "Winner tokens ({}) remain in pool for LP withdrawal",
                winner_reserve
            );
        }

        // ═══════════════════════════════════════════════════════════
        // 5. USDC 储备保留给 LP
        // ═══════════════════════════════════════════════════════════

        // USDC 储备保留在 global_vault 中
        // LP 可以通过 withdraw_liquidity 按比例提取
        // 不需要在这里做任何操作

        msg!(
            "USDC reserve ({}) remains for LP withdrawal",
            self.market.pool_collateral_reserve
        );

        // ═══════════════════════════════════════════════════════════
        // 6. 🔒 设置 pool_settled 标志，允许 LP 提取流动性
        // ═══════════════════════════════════════════════════════════

        self.market.pool_settled = true;
        msg!("✅ Pool settled flag set to true, LP withdrawal now allowed");

        // ═══════════════════════════════════════════════════════════════
        // ✅ v1.0.11: 发射 Pool 结算事件
        // ═══════════════════════════════════════════════════════════════
        let clock = Clock::get()?;

        emit!(SettlePoolEvent {
            authority: self.authority.key(),
            market: self.market.key(),
            winner_token_type: self.market.winner_token_type,
            loser_tokens_burned: loser_tokens_transferred, // v1.0.28: 改为销毁
            usdc_released: 0, // ✅ v1.1.0: USDC 保留在 global_usdc_vault 给 LP 提取
            timestamp: clock.unix_timestamp,
        });

        msg!("SettlePool completed successfully");
        Ok(())
    }
}
