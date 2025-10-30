//! 白名单状态：控制市场创建者权限

use anchor_lang::prelude::*;

/// 白名单账户：记录允许创建市场的用户
#[account]
#[derive(InitSpace, Debug, Default)]
pub struct Whitelist {
    /// 白名单中的创建者地址
    pub creator: Pubkey,
}

impl Whitelist {
    /// 白名单PDA种子前缀
    pub const SEED_PREFIX: &'static str = "wl-seed";
}
