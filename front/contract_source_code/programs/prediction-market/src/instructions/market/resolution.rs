//! 市场指令：结算市场（由管理员触发）

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

/// 账户集合：市场结算所需账户
#[derive(Accounts)]
pub struct Resolution<'info> {
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

    /// 全局金库（PDA，存放SOL）
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

    /// 用户信息（领取奖励时用）
    #[account(
        mut,
        seeds = [USERINFO.as_bytes(), &user.key().to_bytes(), &market.key().to_bytes()],
        bump
    )]
    pub user_info: Box<Account<'info, UserInfo>>,

    /// 用户（用于接收奖励等场景）
    #[account(mut)]
    pub user: AccountInfo<'info>,

    /// 管理员（必须为全局authority）
    #[account(mut)]
    pub authority: Signer<'info>,

    /// 系统/代币/ATA程序
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Resolution<'info>{
    /// 结算入口；当前为权限校验与占位逻辑
    pub fn handler(&mut self, _yes_amount: u64, _no_amount: u64 ,token_type: u8, is_completed: bool ,global_vault_bump:u8)-> Result<()> {
        // 仅限管理员
        require!(
            self.authority.key() == self.global_config.authority.key(),
            PredictionMarketError::InvalidMigrationAuthority
        );

        let signer_seeds: &[&[&[u8]]] = &[&[
            GLOBAL.as_bytes(),
            &[global_vault_bump],
        ]];

        // 兑付 + 完成
        self.market.resolution(
            &mut self.global_vault.to_account_info(),
            &mut self.user,
            signer_seeds,
            &mut self.user_info,
            token_type,
            &self.system_program,
        )?;

        if is_completed {
            self.market.is_completed = true;
        }

        Ok(())
    }
}
