//! ✅ v1.1.0: 铸造完整集合指令：用户存入 USDC，获得等量的 YES + NO 代币
//!
//! 这是条件代币的核心机制：
//! - 用户存入 X USDC
//! - 系统铸造 X YES + X NO
//! - 确保 YES + NO 价值 = X USDC

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

/// 账户集合：铸造完整集合所需账户
#[derive(Accounts)]
pub struct MintCompleteSet<'info> {
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

    /// ✅ v1.1.0: 全局金库（PDA，用于验证 mint authority）
    /// CHECK: global vault pda used as mint authority
    #[account(
        mut,
        seeds = [GLOBAL.as_bytes()],
        bump,
    )]
    pub global_vault: AccountInfo<'info>,

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
        init_if_needed,
        payer = user,
        associated_token::mint = yes_token,
        associated_token::authority = user,
    )]
    pub user_yes_ata: Box<Account<'info, TokenAccount>>,

    /// 用户的 NO ATA
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = no_token,
        associated_token::authority = user,
    )]
    pub user_no_ata: Box<Account<'info, TokenAccount>>,

    /// ✅ v1.1.0: USDC Mint
    #[account(
        constraint = usdc_mint.key() == global_config.usdc_mint @ PredictionMarketError::InvalidMint
    )]
    pub usdc_mint: Box<Account<'info, Mint>>,

    /// ✅ v1.1.0: 全局 USDC 金库（存储 USDC 抵押品）
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

    /// 用户信息
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

impl<'info> MintCompleteSet<'info> {
    /// 处理铸造完整集合
    ///
    /// # 参数
    /// * `amount` - USDC 数量（6 位精度）
    /// * `global_vault_bump` - 全局金库的 bump
    ///
    /// # 流程
    /// 1. ✅ v1.1.0: 用户转 USDC 到全局 USDC 金库（抵押）
    /// 2. 铸造等量的 YES 代币给用户
    /// 3. 铸造等量的 NO 代币给用户
    /// 4. 更新市场统计
    pub fn handler(&mut self, amount: u64, global_vault_bump: u8) -> Result<()> {
        msg!("MintCompleteSet start: amount={}", amount);

        // ✅ v1.0.17: 验证 global_vault 已正确初始化（owner = program_id）
        require!(
            self.global_vault.owner == &crate::ID,
            PredictionMarketError::InvalidAuthority
        );

        // ✅ 检查合约是否暂停
        require!(
            !self.global_config.is_paused,
            PredictionMarketError::ContractPaused
        );

        // 验证金额
        require!(amount > 0, PredictionMarketError::InvalidAmount);

        // 验证市场未完成
        require!(
            !self.market.is_completed,
            PredictionMarketError::CurveAlreadyCompleted
        );

        // 初始化用户信息（如果需要）
        if !self.user_info.is_initialized {
            self.user_info.user = self.user.key();
            // ✅ FIX CRITICAL-2: 不再初始化余额字段（已删除）
            self.user_info.is_lp = false;
            self.user_info.is_initialized = true;
        }

        // ✅ v1.1.0: 1. 用户转 USDC 到全局 USDC 金库（作为抵押品）
        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.user_usdc_ata.to_account_info(),
                    to: self.global_usdc_vault.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            amount,
        )?;
        msg!("✅ Locked {} USDC as collateral", amount);

        // PDA 签名种子
        let signer_seeds: &[&[&[u8]]] = &[&[GLOBAL.as_bytes(), &[global_vault_bump]]];

        // 2. 铸造等量的 YES 代币给用户
        token::mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::MintTo {
                    mint: self.yes_token.to_account_info(),
                    to: self.user_yes_ata.to_account_info(),
                    authority: self.global_vault.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
        msg!("✅ Minted {} YES tokens", amount);

        // 3. 铸造等量的 NO 代币给用户
        token::mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                token::MintTo {
                    mint: self.no_token.to_account_info(),
                    to: self.user_no_ata.to_account_info(),
                    authority: self.global_vault.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
        msg!("✅ Minted {} NO tokens", amount);

        // 4. 更新市场状态
        let market = &mut self.market;
        market.total_collateral_locked = market
            .total_collateral_locked
            .checked_add(amount)
            .ok_or(PredictionMarketError::MathOverflow)?;
        market.total_yes_minted = market
            .total_yes_minted
            .checked_add(amount)
            .ok_or(PredictionMarketError::MathOverflow)?;
        market.total_no_minted = market
            .total_no_minted
            .checked_add(amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        // ✅ FIX: 同步 AMM 供应量计数，否则 swap 卖出时会因 checked_sub 下溢失败
        market.token_yes_total_supply = market
            .token_yes_total_supply
            .checked_add(amount)
            .ok_or(PredictionMarketError::MathOverflow)?;
        market.token_no_total_supply = market
            .token_no_total_supply
            .checked_add(amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        // ✅ FIX CRITICAL-2: 不再更新 user_info 余额（已删除）
        // 余额由 SPL Token ATA 自动追踪，无需在 user_info 中重复

        msg!(
            "✅ MintCompleteSet completed: {} USDC → {} YES + {} NO",
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

        // ✅ v1.1.1: 发射铸造事件（增强可追溯性）
        let clock = Clock::get()?;
        emit!(crate::events::MintCompleteSetEvent {
            user: self.user.key(),
            market: self.market.key(),
            usdc_locked: amount,
            yes_minted: amount,
            no_minted: amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }
}
