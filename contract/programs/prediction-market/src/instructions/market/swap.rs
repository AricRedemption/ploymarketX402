//! 市场指令：代币交换（买/卖 YES 或 NO）

use crate::{
    constants::{CONFIG, GLOBAL, MARKET, USERINFO},
    errors::PredictionMarketError,
    events::TradeEvent,
    state::{config::*, market::*},
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

/// 账户集合：交易所需账户
#[derive(Accounts)]
pub struct Swap<'info> {
    /// 全局配置
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    global_config: Box<Account<'info, Config>>,

    /// ✅ v1.1.0: 团队钱包（仅用于验证 team_usdc_ata 的 authority）
    /// CHECK: Verified against global_config.team_wallet
    #[account(
        constraint = global_config.team_wallet == team_wallet.key() @ PredictionMarketError::IncorrectAuthority
    )]
    pub team_wallet: AccountInfo<'info>,

    /// 市场账户
    #[account(
        mut,
        seeds = [MARKET.as_bytes(), &yes_token.key().to_bytes(), &no_token.key().to_bytes()], 
        bump
    )]
    market: Account<'info, Market>,

    /// ✅ v1.1.0: 全局金库（PDA，用于验证 mint authority 和 USDC 转账）
    /// CHECK: global vault pda used as authority
    #[account(
        mut,
        seeds = [GLOBAL.as_bytes()],
        bump,
    )]
    pub global_vault: AccountInfo<'info>,

    /// YES/NO 代币mint
    pub yes_token: Box<Account<'info, Mint>>,
    pub no_token: Box<Account<'info, Mint>>,

    /// 全局金库的YES/NO ATA（按需使用）
    /// CHECK: ata of global vault
    #[account(
        mut,
        seeds = [
            global_vault.key().as_ref(),
            anchor_spl::token::spl_token::ID.as_ref(),
            yes_token.key().as_ref(),
        ],
        bump,
        seeds::program = anchor_spl::associated_token::ID
    )]
    global_yes_ata: AccountInfo<'info>,

     /// CHECK: ata of global vault
     #[account(
        mut,
        seeds = [
            global_vault.key().as_ref(),
            anchor_spl::token::spl_token::ID.as_ref(),
            no_token.key().as_ref(),
        ],
        bump,
        seeds::program = anchor_spl::associated_token::ID
    )]
    global_no_ata: AccountInfo<'info>,

    /// 用户的YES/NO ATA（不存在则创建）
    /// CHECK: ata of user
    #[account(
        mut,
        seeds = [
            user.key().as_ref(),
            anchor_spl::token::spl_token::ID.as_ref(),
            yes_token.key().as_ref(),
        ],
        bump,
        seeds::program = anchor_spl::associated_token::ID
    )]
    user_yes_ata: AccountInfo<'info>,

     /// CHECK: ata of user
     #[account(
        mut,
        seeds = [
            user.key().as_ref(),
            anchor_spl::token::spl_token::ID.as_ref(),
            no_token.key().as_ref(),
        ],
        bump,
        seeds::program = anchor_spl::associated_token::ID
    )]
    user_no_ata: AccountInfo<'info>,

    /// 用户信息（按需初始化）
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserInfo>(),
        seeds = [USERINFO.as_bytes(), &user.key().to_bytes(), &market.key().to_bytes()],
        bump
    )]
    pub user_info: Box<Account<'info, UserInfo>>,

    // ═══════════════════════════════════════════════════════════════
    // ✅ v1.1.0: USDC 相关账户
    // ═══════════════════════════════════════════════════════════════

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

    /// ✅ v1.1.0: 用户 USDC ATA
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = user,
    )]
    pub user_usdc_ata: Box<Account<'info, TokenAccount>>,

    /// ✅ v1.1.0: 团队钱包 USDC ATA（用于接收平台手续费）
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = team_wallet,
    )]
    pub team_usdc_ata: Box<Account<'info, TokenAccount>>,

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

impl<'info> Swap<'info> {
    /// 处理交易：校验市场时间/状态，准备用户账户，委托给 `Market::swap`
    ///
    /// # 参数
    /// * `deadline` - 交易过期时间戳（Unix timestamp in seconds），如果为 0 则不检查
    pub fn handler(&mut self, amount: u64, direction: u8, token_type: u8 ,minimum_receive_amount: u64, deadline: i64, global_vault_bump:u8) -> Result<()> {
        // ✅ v1.0.17: 验证 global_vault 已正确初始化（owner = program_id）
        // 防止在未执行 configure 初始化的情况下调用指令
        require!(
            self.global_vault.owner == &crate::ID,
            PredictionMarketError::InvalidAuthority
        );

        // ✅ FIX: 检查合约是否暂停
        require!(
            !self.global_config.is_paused,
            PredictionMarketError::ContractPaused
        );

        // ✅ FIX MEDIUM-2: 检查交易是否已过期（防止交易长时间在 mempool 中等待）
        if deadline > 0 {
            let clock = Clock::get()?;
            require!(
                clock.unix_timestamp <= deadline,
                PredictionMarketError::TransactionExpired
            );
        }

        let market = &mut self.market;

        // 校验结束时间
        let clock = Clock::get()?;
        // 🔒 v1.1.0: 统一使用 MarketEnded 错误码（与 market.rs 保持一致）
        if let Some(ending_slot) = market.ending_slot {
            require!(
                ending_slot >= clock.slot,
                PredictionMarketError::MarketEnded
            )
        }

        // 不能在完成后再交易
        require!(
            market.is_completed == false,
            PredictionMarketError::CurveAlreadyCompleted
        );

        // ✅ v1.0.19 + v1.0.22: 强制检查 min_trading_liquidity（感谢审计发现!）
        //
        // 🔴 原问题：配置项 min_trading_liquidity 存在但未实际检查
        //    运营/前端容易误以为启用了最小流动性保护
        //
        // ✅ v1.0.19 修复：在交易前检查 pool_collateral_reserve 是否满足最小要求
        // ✅ v1.0.22 优化：使用更精确的错误类型 MarketBelowMinLiquidity
        //
        // 🔍 错误语义区分：
        //   - InsufficientLiquidity: 临时状态，池中资金不足完成此次交易
        //   - MarketBelowMinLiquidity: 市场总储备低于安全阈值，需管理员介入
        require!(
            market.pool_collateral_reserve >= self.global_config.min_trading_liquidity,
            PredictionMarketError::MarketBelowMinLiquidity
        );

        let user_info_pda = &mut self.user_info;

        // 初始化用户信息（如未初始化）
        if user_info_pda.is_initialized == false {
            msg!("User info does not exist, initializing...");
            user_info_pda.user = self.user.key();
            // ✅ FIX CRITICAL-2: 不再初始化已删除的余额字段
            user_info_pda.is_lp = false;
            user_info_pda.is_initialized = true;
            msg!("User info initialized.");
        } else {
            msg!("User info already exists.");
        }

        msg!(
            "Swap started. amount: {}, direction: {}, token_type: {}, minimum_receive_amount: {}, global_vault_bump: {}",
            amount,
            direction,
            token_type,
            minimum_receive_amount,
            global_vault_bump
        );

        // ✅ v1.0.18 + v1.0.21: ATA 验证最佳实践（感谢审计确认!）
        //
        // 🛡️ 双层防护策略：
        //   1. 声明式层 (#[derive(Accounts)]): seeds + seeds::program 验证 PDA 地址
        //   2. 运行时层 (handler): 手动验证 mint 和 authority 字段
        //
        // 🔍 为何需要运行时验证？
        //   - AccountInfo 类型无法在 constraint 中访问 TokenAccount 字段
        //   - 虽然 PDA seeds 验证已经足够安全（ATA 地址唯一对应 owner+mint）
        //   - 但运行时验证提供额外的纵深防御，防止意外情况
        //
        // ✅ 审计确认：此实现符合 Anchor 最佳实践
        //
        // 先保存需要的 keys 以避免借用冲突
        let no_token_key = self.no_token.key();
        let yes_token_key = self.yes_token.key();
        let user_key = self.user.key();

        // 确保用户ATA存在并属于正确的 mint 和 authority
        if token_type == 0 {
            // NO ATA
            if self.user_no_ata.data_is_empty() {
                anchor_spl::associated_token::create(CpiContext::new(
                    self.associated_token_program.to_account_info(),
                    anchor_spl::associated_token::Create {
                        payer: self.user.to_account_info(),
                        associated_token: self.user_no_ata.to_account_info(),
                        authority: self.user.to_account_info(),
                        mint: self.no_token.to_account_info(),
                        system_program: self.system_program.to_account_info(),
                        token_program: self.token_program.to_account_info(),
                    }
                ))?;
            } else {
                // 验证已存在的 NO ATA
                let user_no_token_account = anchor_spl::token::TokenAccount::try_deserialize(
                    &mut &self.user_no_ata.data.borrow()[..]
                )?;
                require!(
                    user_no_token_account.mint == no_token_key,
                    PredictionMarketError::InvalidMint
                );
                require!(
                    user_no_token_account.owner == user_key,
                    PredictionMarketError::InvalidAuthority
                );
            }
        } else {
            // YES ATA
            if self.user_yes_ata.data_is_empty() {
                anchor_spl::associated_token::create(CpiContext::new(
                    self.associated_token_program.to_account_info(),
                    anchor_spl::associated_token::Create {
                        payer: self.user.to_account_info(),
                        associated_token: self.user_yes_ata.to_account_info(),
                        authority: self.user.to_account_info(),
                        mint: self.yes_token.to_account_info(),
                        system_program: self.system_program.to_account_info(),
                        token_program: self.token_program.to_account_info(),
                    }
                ))?;
            } else {
                // 验证已存在的 YES ATA
                let user_yes_token_account = anchor_spl::token::TokenAccount::try_deserialize(
                    &mut &self.user_yes_ata.data.borrow()[..]
                )?;
                require!(
                    user_yes_token_account.mint == yes_token_key,
                    PredictionMarketError::InvalidMint
                );
                require!(
                    user_yes_token_account.owner == user_key,
                    PredictionMarketError::InvalidAuthority
                );
            }
        }

        // 现在创建引用用于后续操作
        let source = &mut self.global_vault.to_account_info();
        let team_wallet = &mut self.team_wallet;

        let yes_token = &mut self.yes_token;
        let user_yes_ata = &mut self.user_yes_ata;

        let no_token = &mut self.no_token;
        let user_no_ata = &mut self.user_no_ata;

        // PDA种子
        let signer_seeds: &[&[&[u8]]] = &[&[
            GLOBAL.as_bytes(),
            &[global_vault_bump],
        ]];

        // 交由市场逻辑处理具体交换
        // ✅ v1.0.12: 捕获 SwapResult 用于准确的事件发射
        // ✅ v1.1.0: 添加 USDC 相关账户
        let swap_result = market.swap(
            &*self.global_config,
            yes_token.as_ref(),
            &mut self.global_yes_ata,
            user_yes_ata,
            no_token.as_ref(),
            &mut self.global_no_ata,
            user_no_ata,
            source,
            team_wallet,
            amount,
            direction,
            token_type,
            minimum_receive_amount,
            &self.user,
            signer_seeds,
            user_info_pda,
            &self.token_program,
            &self.system_program,
            // ✅ v1.1.0: USDC 相关账户
            &self.usdc_mint,
            &self.global_usdc_vault,
            &self.user_usdc_ata,
            &self.team_usdc_ata,
        )?;

        // ═══════════════════════════════════════════════════════════════
        // ✅ v1.0.12: 发射准确的交易事件
        // ═══════════════════════════════════════════════════════════════
        let clock = Clock::get()?;
        emit!(TradeEvent {
            user: self.user.key(),
            token_yes: self.yes_token.key(),
            token_no: self.no_token.key(),
            market_info: self.market.key(),
            usdc_amount: swap_result.usdc_amount,       // ✅ v1.1.0: 实际 USDC 数量（买=支付，卖=收到）
            token_amount: swap_result.token_amount,     // ✅ 实际代币数量（买=收到，卖=支付）
            fee_usdc: swap_result.fee_usdc,             // ✅ v1.1.0: 实际手续费（USDC）
            is_buy: direction == 0,
            is_yes_no: token_type == 1,
            real_usdc_reserves: self.market.pool_collateral_reserve,  // ✅ v1.1.0: USDC 储备
            real_token_yes_reserves: self.market.pool_yes_reserve,
            real_token_no_reserves: self.market.pool_no_reserve,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }
}