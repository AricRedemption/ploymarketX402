//! 管理员指令：将创建者添加到白名单

use crate::{
    constants::CONFIG,
    errors::PredictionMarketError,
    events::WhitelistUpdateEvent,
    state::{config::*, whitelist::*},
};
use anchor_lang::prelude::*;

/// 添加到白名单的账户集合
#[derive(Accounts)]
#[instruction(creator: Pubkey)]
pub struct AddToWhitelist<'info> {
    /// 全局配置
    #[account(
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    pub global_config: Account<'info, Config>,

    /// 白名单 PDA（将被创建）
    /// ✅ 使用 Whitelist::SEED_PREFIX 保持与 create_market 一致
    #[account(
        init,
        payer = authority,
        space = 8 + Whitelist::INIT_SPACE,
        seeds = [Whitelist::SEED_PREFIX.as_bytes(), creator.as_ref()],
        bump
    )]
    pub whitelist: Account<'info, Whitelist>,

    /// 管理员（必须为全局 authority）
    #[account(
        mut,
        constraint = authority.key() == global_config.authority @ PredictionMarketError::IncorrectAuthority
    )]
    pub authority: Signer<'info>,

    /// 系统程序
    pub system_program: Program<'info, System>,
}

impl<'info> AddToWhitelist<'info> {
    /// 添加创建者到白名单
    pub fn handler(&mut self, creator: Pubkey) -> Result<()> {
        self.whitelist.creator = creator;

        msg!("Added creator to whitelist: {}", creator);

        // 发射白名单更新事件
        let clock = Clock::get()?;
        emit!(WhitelistUpdateEvent {
            authority: self.authority.key(),
            target: creator,
            is_add: true,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }
}
