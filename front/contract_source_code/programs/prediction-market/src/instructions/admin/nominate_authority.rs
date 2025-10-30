//! 管理员指令：提名新管理员（两步交接的第一步）

use constants::CONFIG;
use errors::PredictionMarketError;
// use state::config::*;

use crate::*;

/// 账户集合：由当前管理员发起提名
#[derive(Accounts)]
pub struct NominateAuthority<'info> {
    /// 当前管理员（必须与全局配置中的 authority 匹配）
    #[account(
        mut,
        constraint = global_config.authority == *admin.key @PredictionMarketError::IncorrectAuthority
    )]
    pub admin: Signer<'info>,

    /// 全局配置（写入 pending_authority）
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    global_config: Box<Account<'info, Config>>,
}

impl NominateAuthority<'_> {
    /// 将新管理员地址写入 pending_authority，等待其调用 accept_authority 接受
    pub fn process(&mut self, new_admin: Pubkey) -> Result<()> {
        self.global_config.pending_authority = new_admin;
        Ok(())
    }
}
