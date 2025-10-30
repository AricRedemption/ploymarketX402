//! 全局状态：平台级参数与权限（面向Solidity工程师的说明见文件底部）

use crate::{
    errors::PredictionMarketError,
    events::{GlobalUpdateEvent, IntoEvent},
};
use anchor_lang::{prelude::*, solana_program::sysvar::last_restart_slot::LastRestartSlot};

/// 更新全局管理员的输入参数
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GlobalAuthorityInput {
    /// 新的全局管理员（可选）
    pub global_authority: Option<Pubkey>,
}

/// 全局账户：存放平台级别配置
#[account]
#[derive(InitSpace, Debug)]
pub struct Global {
    /// 是否已初始化
    pub initialized: bool,
    /// 全局管理员（可更新全局设置）
    pub global_authority: Pubkey, // can update settings

    /// 团队钱包（平台费用接收者）
    pub team_wallet: Pubkey,

    /// 平台手续费（买/卖），单位：百分比（例如 1.0 表示 1%）
    pub platform_buy_fee: f64, //  platform fee percentage
    pub platform_sell_fee: f64,

    // 预测市场初始参数
    pub initial_real_token_reserves: u64,
    pub token_total_supply: u64,
    pub mint_decimals: u8,

    /// 最近一次更新对应的槽位（Solana上的区块计数）
    pub last_updated_slot: u64,
}

impl Default for Global {
    fn default() -> Self {
        Self {
            initialized: true,
            global_authority: Pubkey::default(),
            team_wallet: Pubkey::default(),
            platform_buy_fee: 1.0,
            platform_sell_fee: 1.0,

            // prediction-market initial values
            initial_real_token_reserves: 1000000000000000,
            token_total_supply: 1000000000000000,
            mint_decimals: 6,
            last_updated_slot: 0,
        }
    }
}

/// 全局设置的输入结构
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct GlobalSettingsInput {
    pub initial_real_token_reserves: u64,
    pub token_total_supply: u64,
    pub mint_decimals: u8,
    pub team_wallet: Pubkey,
}

impl Global {
    /// 全局PDA种子前缀
    pub const SEED_PREFIX: &'static str = "globalconfig";

    /// 组合成PDA签名者的seeds（[seed, bump]）
    pub fn get_signer<'a>(bump: &'a u8) -> [&'a [u8]; 2] {
        let prefix_bytes = Self::SEED_PREFIX.as_bytes();
        let bump_slice: &'a [u8] = std::slice::from_ref(bump);
        [prefix_bytes, bump_slice]
    }

    /// 校验全局设置的有效性（范围/非零/非默认）
    pub fn validate_settings(&self, params: &GlobalSettingsInput) -> Result<()> {
        require!(params.mint_decimals <= 9, PredictionMarketError::InvalidParameter);
        require!(
            params.token_total_supply <= u64::MAX / 2,
            PredictionMarketError::InvalidParameter
        );
        require!(
            params.team_wallet != Pubkey::default(),
            PredictionMarketError::InvalidParameter
        );
        require!(
            params.initial_real_token_reserves > 0,
            PredictionMarketError::InvalidParameter
        );

        // 确保总供应量大于初始真实储备
        require!(
            params.token_total_supply > params.initial_real_token_reserves,
            PredictionMarketError::InvalidParameter
        );
        Ok(())
    }

    /// 更新全局设置并记录当前槽位
    pub fn update_settings(&mut self, params: GlobalSettingsInput, slot: u64) {
        self.mint_decimals = params.mint_decimals;
        self.initial_real_token_reserves = params.initial_real_token_reserves;
        self.token_total_supply = params.token_total_supply;
        self.team_wallet = params.team_wallet;

        // Set last updated slot to the slot of the update
        self.last_updated_slot = slot;
    }

    /// 更新全局管理员
    pub fn update_authority(&mut self, params: GlobalAuthorityInput) {
        if let Some(global_authority) = params.global_authority {
            self.global_authority = global_authority;
        }
    }

    /// 判断当前配置是否过期（基于系统last_restart_slot）
    pub fn is_config_outdated(&self) -> Result<bool> {
        let last_restart_slot = LastRestartSlot::get()?;
        Ok(self.last_updated_slot <= last_restart_slot.last_restart_slot)
    }
}

impl IntoEvent<GlobalUpdateEvent> for Global {
    fn into_event(&self) -> GlobalUpdateEvent {
        GlobalUpdateEvent {
            global_authority: self.global_authority,
            initial_real_token_reserves: self.initial_real_token_reserves,
            token_total_supply: self.token_total_supply,
            mint_decimals: self.mint_decimals,
        }
    }
}

/*
面向Solidity工程师的说明：
- 账户模型：
  在Solana/Anchor中，状态保存在独立的“账户”(Account)里，类似于EVM中的合约存储，但以账户对象形式存在，需显式传入指令。
- PDA（Program Derived Address）：
  等价于“合约控制的地址”，由程序ID+种子推导，无法被私钥控制，常用于金库/配置等安全地址。
- Slot与区块时间：
  Solana使用slot作为时间单位（约400ms一个slot），这里通过sysvar(last_restart_slot)读到最近的重启槽位以判断配置是否过期。
- 权限与校验：
  通过#[account(...)]约束、require!宏以及显式传入的Signer来完成“onlyOwner风格”的权限控制。
*/
