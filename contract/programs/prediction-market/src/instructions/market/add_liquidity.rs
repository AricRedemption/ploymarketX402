//! 市场指令：添加流动性（LP）
//! ✅ 双账本系统：只操作 Pool Ledger

use crate::{
    constants::{CONFIG, GLOBAL, LPPOSITION, MARKET},
    errors::PredictionMarketError,
    events::AddLiquidityEvent,
    state::{config::*, market::*},
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

/// 账户集合：添加LP所需账户
#[derive(Accounts)]
pub struct AddLiquidity<'info> {
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
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<LPPosition>(),
        seeds = [LPPOSITION.as_bytes(), &user.key().to_bytes(), &market.key().to_bytes()],
        bump
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

impl<'info> AddLiquidity<'info> {
    /// 处理添加LP：向市场添加流动性
    ///
    /// ✅ 双账本系统：只操作 Pool Ledger
    /// 参数：
    /// - usdc_amount: 用户提供的 USDC 数量
    /// - yes_amount: 用户提供的 YES 代币数量
    /// - no_amount: 用户提供的 NO 代币数量
    /// - global_vault_bump: PDA bump seed
    pub fn handler(
        &mut self,
        usdc_amount: u64,
        yes_amount: u64,
        no_amount: u64,
        _global_vault_bump: u8,
    ) -> Result<()> {
        msg!(
            "AddLiquidity handler start: usdc={}, yes={}, no={}",
            usdc_amount,
            yes_amount,
            no_amount
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

        // 验证市场未完成
        require!(
            !self.market.is_completed,
            PredictionMarketError::CurveAlreadyCompleted
        );

        // 验证金额有效
        require!(usdc_amount > 0, PredictionMarketError::InvalidAmount);
        require!(yes_amount > 0, PredictionMarketError::InvalidAmount);
        require!(no_amount > 0, PredictionMarketError::InvalidAmount);

        // 验证满足最小流动性要求
        require!(
            usdc_amount >= self.global_config.min_usdc_liquidity,
            PredictionMarketError::InsufficientLiquidity
        );

        // ═══════════════════════════════════════════════════════════
        // 2. 计算 LP 份额并验证比例
        // ═══════════════════════════════════════════════════════════

        let lp_shares_to_mint = if self.market.total_lp_shares == 0 {
            // 首次添加流动性：LP份额 = USDC数量
            msg!("First liquidity provision, minting {} LP shares", usdc_amount);
            usdc_amount
        } else {
            // 后续添加流动性：按比例计算并验证三种资产比例一致
            require!(
                self.market.pool_collateral_reserve > 0,
                PredictionMarketError::InsufficientLiquidity
            );
            require!(
                self.market.pool_yes_reserve > 0,
                PredictionMarketError::InsufficientLiquidity
            );
            require!(
                self.market.pool_no_reserve > 0,
                PredictionMarketError::InsufficientLiquidity
            );

            // 🔒 安全修复：计算三种资产各自对应的份额，取最小值
            // 这确保用户必须按池子当前比例投入，防止用少量代币+大量USDC套利
            let shares_from_usdc = (usdc_amount as u128)
                .checked_mul(self.market.total_lp_shares as u128)
                .ok_or(PredictionMarketError::MathOverflow)?
                .checked_div(self.market.pool_collateral_reserve as u128)
                .ok_or(PredictionMarketError::MathOverflow)?;

            let shares_from_yes = (yes_amount as u128)
                .checked_mul(self.market.total_lp_shares as u128)
                .ok_or(PredictionMarketError::MathOverflow)?
                .checked_div(self.market.pool_yes_reserve as u128)
                .ok_or(PredictionMarketError::MathOverflow)?;

            let shares_from_no = (no_amount as u128)
                .checked_mul(self.market.total_lp_shares as u128)
                .ok_or(PredictionMarketError::MathOverflow)?
                .checked_div(self.market.pool_no_reserve as u128)
                .ok_or(PredictionMarketError::MathOverflow)?;

            // 🔒 取三者最小值，防止用户用不成比例的资产获得超额份额
            let shares = shares_from_usdc.min(shares_from_yes).min(shares_from_no);

            // 🔒 额外验证：三种资产的份额计算结果不能相差太大（容忍 1% 误差）
            let max_shares = shares_from_usdc.max(shares_from_yes).max(shares_from_no);
            let ratio = if shares > 0 {
                max_shares.checked_mul(10000).unwrap_or(u128::MAX) / shares
            } else {
                u128::MAX
            };
            require!(
                ratio <= 10100, // 最大相差 1%
                PredictionMarketError::InvalidLiquidityRatio
            );

            require!(shares <= u64::MAX as u128, PredictionMarketError::MathOverflow);

            let shares_u64 = shares as u64;
            msg!(
                "✅ Proportional liquidity provision (ratio-verified), minting {} LP shares",
                shares_u64
            );
            shares_u64
        };

        require!(lp_shares_to_mint > 0, PredictionMarketError::InvalidAmount);

        // ═══════════════════════════════════════════════════════════
        // 3. 转移资产：用户 → Pool
        // ═══════════════════════════════════════════════════════════

        // ✅ v1.1.0: 3.1 转移 USDC 到 global_usdc_vault
        msg!("Transferring {} USDC from user to vault", usdc_amount);
        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.user_usdc_ata.to_account_info(),
                    to: self.global_usdc_vault.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            usdc_amount,
        )?;

        // 3.2 转移 YES 代币到 global_yes_ata
        msg!("Transferring {} YES tokens from user to pool", yes_amount);
        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.user_yes_ata.to_account_info(),
                    to: self.global_yes_ata.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            yes_amount,
        )?;

        // 3.3 转移 NO 代币到 global_no_ata
        msg!("Transferring {} NO tokens from user to pool", no_amount);
        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                token::Transfer {
                    from: self.user_no_ata.to_account_info(),
                    to: self.global_no_ata.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            no_amount,
        )?;

        // ═══════════════════════════════════════════════════════════
        // 4. 更新 Pool Ledger（Market 状态）
        // ═══════════════════════════════════════════════════════════

        self.market.pool_collateral_reserve = self.market.pool_collateral_reserve
            .checked_add(usdc_amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        self.market.pool_yes_reserve = self.market.pool_yes_reserve
            .checked_add(yes_amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        self.market.pool_no_reserve = self.market.pool_no_reserve
            .checked_add(no_amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        self.market.total_lp_shares = self.market.total_lp_shares
            .checked_add(lp_shares_to_mint)
            .ok_or(PredictionMarketError::MathOverflow)?;

        msg!(
            "Updated Pool Ledger: collateral={}, yes={}, no={}, total_lp={}",
            self.market.pool_collateral_reserve,
            self.market.pool_yes_reserve,
            self.market.pool_no_reserve,
            self.market.total_lp_shares
        );

        // ═══════════════════════════════════════════════════════════
        // 5. 铸造 LP 份额（更新 LPPosition）
        // ═══════════════════════════════════════════════════════════

        // 初始化 LPPosition（如果是新账户）
        if self.lp_position.lp_shares == 0 {
            self.lp_position.user = self.user.key();
            self.lp_position.market = self.market.key();
            self.lp_position.last_fee_claim_slot = Clock::get()?.slot;
            // ✅ 初始化 last_fee_per_share 为当前值（避免领取历史费用）
            self.lp_position.last_fee_per_share = self.market.fee_per_share_cumulative;
        }

        // 增加用户的 LP 份额
        self.lp_position.lp_shares = self.lp_position.lp_shares
            .checked_add(lp_shares_to_mint)
            .ok_or(PredictionMarketError::MathOverflow)?;

        // 记录累计存入的资产（用于统计）
        self.lp_position.deposited_sol = self.lp_position.deposited_sol
            .checked_add(usdc_amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        self.lp_position.deposited_yes = self.lp_position.deposited_yes
            .checked_add(yes_amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        self.lp_position.deposited_no = self.lp_position.deposited_no
            .checked_add(no_amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        msg!(
            "Minted {} LP shares to user. Total user shares: {}",
            lp_shares_to_mint,
            self.lp_position.lp_shares
        );

        // ═══════════════════════════════════════════════════════════════
        // ✅ 发射添加流动性事件
        // ═══════════════════════════════════════════════════════════════
        let clock = Clock::get()?;
        emit!(AddLiquidityEvent {
            user: self.user.key(),
            market: self.market.key(),
            usdc_amount: usdc_amount,  // ✅ v1.1.0: 字段名改为 usdc_amount
            yes_amount,
            no_amount,
            lp_shares_minted: lp_shares_to_mint,
            timestamp: clock.unix_timestamp,
        });

        msg!("AddLiquidity completed successfully");
        Ok(())
    }
}