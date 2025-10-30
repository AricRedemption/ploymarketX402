//! # 配置状态模块
//! 
//! 定义预测市场合约的全局配置结构
//! 包括管理员权限、手续费设置、代币配置等

use crate::errors::*;
use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};
use core::fmt::Debug;

/// 全局配置账户
/// 
/// 存储预测市场合约的全局配置参数
/// 包括管理员权限、手续费设置、代币配置等
#[account]
#[derive(Debug)]
pub struct Config {
    /// 当前管理员公钥
    pub authority: Pubkey,
    
    /// 待确认的管理员公钥（用于两步权限转移）
    /// 当前管理员提名新管理员后，新管理员需要调用accept_authority来确认
    pub pending_authority: Pubkey,

    /// 团队钱包地址
    /// 用于接收平台手续费
    pub team_wallet: Pubkey,

    /// 平台买入手续费（基点，如1000表示10%）
    pub platform_buy_fee: u64,
    
    /// 平台卖出手续费（基点，如1000表示10%）
    pub platform_sell_fee: u64,

    /// 流动性提供者买入手续费（基点）
    pub lp_buy_fee: u64,
    
    /// 流动性提供者卖出手续费（基点）
    pub lp_sell_fee: u64,

    /// 代币总供应量配置
    pub token_supply_config: u64,
    
    /// 代币精度配置
    pub token_decimals_config: u8,

    /// 初始真实代币储备配置
    pub initial_real_token_reserves_config: u64,

    /// 最小SOL流动性要求
    pub min_sol_liquidity: u64,

    /// 配置是否已初始化
    pub initialized: bool,
}

/// 数量配置枚举
/// 
/// 用于验证输入值是否在允许的范围内
/// 支持范围验证和枚举值验证两种模式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum AmountConfig<T: PartialEq + PartialOrd + Debug> {
    /// 范围验证模式
    /// min: 最小值（可选）
    /// max: 最大值（可选）
    Range { min: Option<T>, max: Option<T> },
    
    /// 枚举值验证模式
    /// 只允许指定的值列表中的值
    Enum(Vec<T>),
}

impl<T: PartialEq + PartialOrd + Debug> AmountConfig<T> {
    /// 验证输入值是否符合配置要求
    /// 
    /// # 参数
    /// * `value` - 要验证的值
    /// 
    /// # 返回
    /// * `Result<()>` - 验证结果，如果不符合要求则返回错误
    pub fn validate(&self, value: &T) -> Result<()> {
        match self {
            Self::Range { min, max } => {
                // 检查最小值限制
                if let Some(min) = min {
                    if value < min {
                        return Err(ValueTooSmall.into());
                    }
                }
                
                // 检查最大值限制
                if let Some(max) = max {
                    if value > max {
                        return Err(ValueTooLarge.into());
                    }
                }

                Ok(())
            }
            Self::Enum(options) => {
                // 检查值是否在允许的枚举列表中
                if options.contains(value) {
                    Ok(())
                } else {
                    Err(ValueInvalid.into())
                }
            }
        }
    }
}
