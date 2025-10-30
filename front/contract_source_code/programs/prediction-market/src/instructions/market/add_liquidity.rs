//! 市场指令：添加流动性（LP）

use crate::{
    constants::{CONFIG, GLOBAL, MARKET, USERINFO},
    errors::PredictionMarketError,
    state::{config::*, market::*},
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token},
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

    /// 团队钱包（需与配置一致，用于收取平台费）
    /// CHECK: should be same with the address in the global_config
    #[account(
        mut,
        constraint = global_config.team_wallet == team_wallet.key() @PredictionMarketError::IncorrectAuthority
    )]
    pub team_wallet: AccountInfo<'info>,

    /// 市场账户（以YES/NO mint作为种子）
    #[account(
        mut,
        seeds = [MARKET.as_bytes(), &yes_token.key().to_bytes(), &no_token.key().to_bytes()], 
        bump
    )]
    market: Account<'info, Market>,

    /// 全局金库（存放SOL）
    /// CHECK: global vault pda which stores SOL
    #[account(
        mut,
        seeds = [GLOBAL.as_bytes()],
        bump,
    )]
    pub global_vault: AccountInfo<'info>,

    /// YES/NO 代币mint
    pub yes_token: Box<Account<'info, Mint>>,
    pub no_token: Box<Account<'info, Mint>>,

    /// 用户信息（按需初始化）
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

impl<'info> AddLiquidity<'info> { 
    /// 处理添加LP；当前为占位实现，预留LP逻辑
    pub fn handler(&mut self, amount: u64) -> Result<()> {
        require!(self.market.is_completed == false, PredictionMarketError::MarketIsCompleted);
        require!(amount > 0, PredictionMarketError::WITHDRAWLIQUIDITYSOLAMOUNTERROR);

        self.market.add_liquidity(
            &mut self.global_vault.to_account_info(),
            &self.user,
            amount,
            &mut self.user_info,
            &self.system_program,
        )
    }
}