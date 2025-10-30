//! Pool 初始化指令：为新创建的市场注入初始流动性
//! ✅ 双账本系统：只操作 Pool Ledger
//!
//! 解决问题：
//! - 市场刚创建时，pool_*_reserve 全为 0
//! - Swap 无法使用（没有库存）
//! - LP 添加流动性需要自带 YES/NO 代币（鸡蛋问题）
//!
//! 方案：
//! - 管理员/创建者提供 USDC
//! - 自动铸造等量 YES + NO 代币到 Pool
//! - 初始化 LMSR 参数
//! - 可选：给管理员铸造初始 LP 份额

use crate::{
    constants::{CONFIG, GLOBAL, LPPOSITION, MARKET},
    errors::PredictionMarketError,
    state::{config::*, market::*},
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

/// 账户集合：Pool 种子流动性注入
#[derive(Accounts)]
pub struct SeedPool<'info> {
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
        constraint = !market.is_completed @PredictionMarketError::CurveAlreadyCompleted,
        constraint = market.pool_collateral_reserve == 0 @PredictionMarketError::PoolAlreadySeeded,
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
    #[account(mut)]
    pub yes_token: Box<Account<'info, Mint>>,
    #[account(mut)]
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

    /// ✅ v1.1.0: 种子提供者 USDC ATA
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = seeder,
    )]
    pub seeder_usdc_ata: Box<Account<'info, TokenAccount>>,

    /// 种子提供者的 LP Position（可选，如果想给初始 LP 份额）
    #[account(
        init_if_needed,
        payer = seeder,
        space = 8 + std::mem::size_of::<LPPosition>(),
        seeds = [LPPOSITION.as_bytes(), &seeder.key().to_bytes(), &market.key().to_bytes()],
        bump
    )]
    pub seeder_lp_position: Box<Account<'info, LPPosition>>,

    /// 种子提供者（通常是管理员或市场创建者）
    #[account(
        mut,
        constraint = global_config.authority == seeder.key() || market.creator == seeder.key() @PredictionMarketError::IncorrectAuthority
    )]
    pub seeder: Signer<'info>,

    /// 系统/代币/ATA程序
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> SeedPool<'info> {
    /// 处理 Pool 种子流动性注入
    ///
    /// ✅ 双账本系统：只操作 Pool Ledger
    ///
    /// # 参数
    /// - usdc_amount: 注入的 USDC 数量
    /// - _issue_lp_shares: 🔒 已废弃（现在总是发行 LP 份额）
    /// - global_vault_bump: PDA bump seed
    ///
    /// # 安全修复 (v1.0.3)
    /// - 🔒 **强制发行 LP 份额**，防止首位 LP 盗取种子流动性
    /// - 原漏洞：issue_lp_shares=false 时，total_lp_shares=0 但池子有储备
    /// - 攻击场景：首位 LP 用 1 USDC 获得 100% 份额，立即提走全部种子资产
    /// - 修复：忽略 issue_lp_shares 参数，总是铸造 LP 份额给种子提供者
    pub fn handler(
        &mut self,
        usdc_amount: u64,
        _issue_lp_shares: bool,
        global_vault_bump: u8,
    ) -> Result<()> {
        msg!(
            "🔒 SeedPool handler start: sol={} (ALWAYS issue LP shares)",
            usdc_amount
        );

        // ═══════════════════════════════════════════════════════════
        // 1. 验证前置条件
        // ═══════════════════════════════════════════════════════════

        require!(
            !self.global_config.is_paused,
            PredictionMarketError::ContractPaused
        );

        require!(usdc_amount > 0, PredictionMarketError::InvalidAmount);

        // ═══════════════════════════════════════════════════════════
        // 🔒 v1.1.0: SECURITY - 强制最小种子流动性
        // ═══════════════════════════════════════════════════════════
        //
        // **风险**: 种子流动性过低会导致价格失真
        //
        // **场景**:
        // - 管理员意外注入 1 USDC 种子流动性
        // - LMSR b参数设置为 1000 USDC
        // - 导致价格曲线计算异常，用户无法正常交易
        //
        // **修复**: 使用配置的 min_usdc_liquidity 作为最小种子流动性要求
        // - 典型值: 100 USDC (100_000_000 最小单位)
        // - 确保初始流动性足够支撑正常交易
        require!(
            usdc_amount >= self.global_config.min_usdc_liquidity,
            PredictionMarketError::ValueTooSmall
        );

        msg!(
            "✅ Seed liquidity check: {} >= {} (min_usdc_liquidity)",
            usdc_amount,
            self.global_config.min_usdc_liquidity
        );

        // 验证 Pool 尚未被初始化
        require!(
            self.market.pool_collateral_reserve == 0,
            PredictionMarketError::PoolAlreadySeeded
        );
        require!(
            self.market.total_lp_shares == 0,
            PredictionMarketError::PoolAlreadySeeded
        );

        // ✅ v1.1.0: 2. 转移 USDC 到 global_usdc_vault
        msg!("Transferring {} USDC to vault", usdc_amount);
        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.seeder_usdc_ata.to_account_info(),
                    to: self.global_usdc_vault.to_account_info(),
                    authority: self.seeder.to_account_info(),
                },
            ),
            usdc_amount,
        )?;

        // ═══════════════════════════════════════════════════════════
        // 3. 铸造 YES + NO 代币到 Pool
        // ═══════════════════════════════════════════════════════════

        let signer_seeds: &[&[&[u8]]] = &[&[
            crate::constants::GLOBAL.as_bytes(),
            &[global_vault_bump],
        ]];

        // ✅ v1.1.0: 铸造 YES 代币（数量 = USDC 数量，保持 1:1）
        msg!("Minting {} YES tokens to pool", usdc_amount);
        token::mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::MintTo {
                    mint: self.yes_token.to_account_info(),
                    to: self.global_yes_ata.to_account_info(),
                    authority: self.global_vault.to_account_info(),
                },
                signer_seeds,
            ),
            usdc_amount,
        )?;

        // ✅ v1.1.0: 铸造 NO 代币（数量 = USDC 数量，保持 1:1）
        msg!("Minting {} NO tokens to pool", usdc_amount);
        token::mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::MintTo {
                    mint: self.no_token.to_account_info(),
                    to: self.global_no_ata.to_account_info(),
                    authority: self.global_vault.to_account_info(),
                },
                signer_seeds,
            ),
            usdc_amount,
        )?;

        // ═══════════════════════════════════════════════════════════
        // 4. 更新 Pool Ledger
        // ═══════════════════════════════════════════════════════════

        self.market.pool_collateral_reserve = usdc_amount;
        self.market.pool_yes_reserve = usdc_amount;
        self.market.pool_no_reserve = usdc_amount;

        // ═══════════════════════════════════════════════════════════
        // 4.5. ✅ v1.0.12: 同步更新 Settlement Ledger (CRITICAL FIX)
        // ═══════════════════════════════════════════════════════════
        //
        // **问题**: 之前只更新 Pool Ledger，导致 total_collateral_locked = 0
        // **影响**: 用户从池中买齐 YES+NO 后无法通过 redeem_complete_set 赎回 USDC
        // **原因**: redeem_complete_set 校验 total_collateral_locked >= amount
        // **修复**: 与 mint_complete_set 保持一致的会计处理
        //
        // 参考：Polymarket 的核心套利机制依赖"完整套件可随时 1:1 赎回"

        self.market.total_collateral_locked = self
            .market
            .total_collateral_locked
            .checked_add(usdc_amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        self.market.total_yes_minted = self
            .market
            .total_yes_minted
            .checked_add(usdc_amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        self.market.total_no_minted = self
            .market
            .total_no_minted
            .checked_add(usdc_amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        self.market.token_yes_total_supply = self
            .market
            .token_yes_total_supply
            .checked_add(usdc_amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        self.market.token_no_total_supply = self
            .market
            .token_no_total_supply
            .checked_add(usdc_amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        msg!(
            "✅ Settlement Ledger synced: collateral_locked={}, yes_minted={}, no_minted={}",
            self.market.total_collateral_locked,
            self.market.total_yes_minted,
            self.market.total_no_minted
        );

        // ═══════════════════════════════════════════════════════════
        // 5. 🔒 强制铸造初始 LP 份额给种子提供者（安全修复）
        // ═══════════════════════════════════════════════════════════

        // 🔒 安全修复：总是发行 LP 份额，防止首位 LP 盗取种子流动性
        // ✅ v1.1.0: 初始 LP 份额 = USDC 数量
        self.market.total_lp_shares = usdc_amount;

        // 初始化 LP Position
        if self.seeder_lp_position.lp_shares == 0 {
            self.seeder_lp_position.user = self.seeder.key();
            self.seeder_lp_position.market = self.market.key();
            self.seeder_lp_position.last_fee_claim_slot = Clock::get()?.slot;
            // ✅ 初始化 last_fee_per_share 为当前值
            self.seeder_lp_position.last_fee_per_share = self.market.fee_per_share_cumulative;
        }

        self.seeder_lp_position.lp_shares = usdc_amount;
        self.seeder_lp_position.deposited_sol = usdc_amount;
        self.seeder_lp_position.deposited_yes = usdc_amount;
        self.seeder_lp_position.deposited_no = usdc_amount;

        msg!(
            "✅ Issued {} LP shares to seeder (FORCED for security)",
            usdc_amount
        );

        // ═══════════════════════════════════════════════════════════
        // 6. 初始化 LMSR 参数（如果尚未设置）
        // ═══════════════════════════════════════════════════════════

        // LMSR q 参数初始化为 0（中立价格）
        // 后续 swap 会自动调整
        if self.market.lmsr_q_yes == 0 && self.market.lmsr_q_no == 0 {
            self.market.lmsr_q_yes = 0;
            self.market.lmsr_q_no = 0;
            msg!("✅ Initialized LMSR q parameters to neutral (0, 0)");
        }

        msg!(
            "✅ Pool seeded: collateral={}, yes={}, no={}, lp_shares={}",
            self.market.pool_collateral_reserve,
            self.market.pool_yes_reserve,
            self.market.pool_no_reserve,
            self.market.total_lp_shares
        );

        // ✅ v1.1.1: 发射种子流动性事件（增强可追溯性）
        let clock = Clock::get()?;
        emit!(crate::events::SeedPoolEvent {
            seeder: self.seeder.key(),
            market: self.market.key(),
            usdc_amount,
            yes_amount: usdc_amount,  // seed_pool: YES = USDC 数量
            no_amount: usdc_amount,   // seed_pool: NO = USDC 数量
            lp_shares_minted: usdc_amount,  // 初始 LP 份额 = USDC 数量
            timestamp: clock.unix_timestamp,
        });

        msg!("SeedPool completed successfully");
        Ok(())
    }
}
