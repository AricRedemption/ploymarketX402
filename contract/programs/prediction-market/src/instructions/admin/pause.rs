//! 管理员指令：紧急暂停和恢复合约

use crate::{
    constants::CONFIG,
    errors::PredictionMarketError,
    events::{PauseEvent, UnpauseEvent},
    state::config::*,
};
use anchor_lang::prelude::*;

/// 暂停/恢复合约的账户
#[derive(Accounts)]
pub struct Pause<'info> {
    /// 全局配置
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,

    /// 管理员签名者
    #[account(
        constraint = authority.key() == config.authority @ PredictionMarketError::InvalidAuthority
    )]
    pub authority: Signer<'info>,
}

impl<'info> Pause<'info> {
    /// 暂停合约
    pub fn pause(&mut self) -> Result<()> {
        // 🔒 v1.1.0: 使用语义明确的错误码
        require!(!self.config.is_paused, PredictionMarketError::AlreadyPaused);
        self.config.is_paused = true;
        msg!("Contract PAUSED by admin: {}", self.authority.key());

        // 发射暂停事件
        let clock = Clock::get()?;
        emit!(PauseEvent {
            authority: self.authority.key(),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// 恢复合约
    pub fn unpause(&mut self) -> Result<()> {
        // 🔒 v1.1.0: 使用语义明确的错误码
        require!(self.config.is_paused, PredictionMarketError::NotPaused);
        self.config.is_paused = false;
        msg!("Contract UNPAUSED by admin: {}", self.authority.key());

        // 发射恢复事件
        let clock = Clock::get()?;
        emit!(UnpauseEvent {
            authority: self.authority.key(),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }
}
