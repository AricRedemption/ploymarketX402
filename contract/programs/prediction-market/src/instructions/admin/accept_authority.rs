//! 管理员接受权限指令：由被提名者接受管理员角色

use constants::CONFIG;
use errors::PredictionMarketError;

use crate::*;

/// 账户集合：新管理员与全局配置
#[derive(Accounts)]
pub struct AcceptAuthority<'info> {
    /// 待接受管理员（必须与全局配置中 pending_authority 一致）
    #[account(
        mut,
        constraint = global_config.pending_authority == new_admin.key() @PredictionMarketError::IncorrectAuthority
    )]
    pub new_admin: Signer<'info>,

    /// 全局配置（存储当前/待定管理员）
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    global_config: Box<Account<'info, Config>>,
}

impl AcceptAuthority<'_> {
    /// 将待定管理员提升为正式管理员，并清空 pending 字段
    pub fn process(&mut self) -> Result<()> {
        self.global_config.authority = self.new_admin.key();
        self.global_config.pending_authority = Pubkey::default();

        Ok(())
    }
}
