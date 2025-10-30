use anchor_lang::prelude::*;
use crate::constants::WHITELIST;

#[account]
#[derive(InitSpace, Debug, Default)]
pub struct Whitelist {
    pub creator: Pubkey,
}

impl Whitelist {
    /// ✅ 使用与constants.rs一致的种子前缀
    pub const SEED_PREFIX: &'static str = WHITELIST;
}
