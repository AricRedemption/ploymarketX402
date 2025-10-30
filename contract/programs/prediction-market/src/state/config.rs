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

    /// ⚠️ 已废弃：代币总供应量配置（v1.1.0+）
    ///
    /// **废弃原因**：
    /// - 预测市场采用动态铸造模型，无固定总供应量上限
    /// - YES/NO 代币通过 mint_complete_set 按需铸造，通过 redeem_complete_set 销毁
    /// - 代币供应量由市场需求决定，不需要预设上限
    ///
    /// **历史背景**：
    /// - 此字段原本用于 bonding curve 固定供应量模型
    /// - 预测市场模型不适用固定供应量约束
    ///
    /// **当前状态**：
    /// - 此字段未在任何指令中使用
    /// - 保留以维持账户结构兼容性
    /// - 部署时应设置为 0 以明确表示未启用
    ///
    /// **未来计划**：
    /// - v2.0 可考虑移除（需要账户迁移）
    /// - 或者用于限制单个市场的最大代币铸造量（需要实现额度校验）
    ///
    /// 默认值: 0 (未启用)
    pub token_supply_config: u64,
    
    /// 代币精度配置
    /// ✅ v1.1.0: 强制要求为 6（USDC decimals）
    /// 在 configure 指令中验证 token_decimals_config == 6
    pub token_decimals_config: u8,

    /// 初始真实代币储备配置
    pub initial_real_token_reserves_config: u64,

    /// ⚠️ 已废弃：最小流动性要求（保留以兼容旧版本）
    /// 实际使用 min_usdc_liquidity 和 min_trading_liquidity
    pub min_sol_liquidity: u64,

    /// ⚠️ 最小交易流动性要求（当前未使用）
    ///
    /// **预期用途**: 限制 swap 操作的最小池子流动性，防止池子过度枯竭
    /// **当前状态**: 字段已定义但未在 swap 中强制执行
    /// **风险**: 前端/运维可能误认为存在流动性保护
    ///
    /// **实现选项**:
    /// - 选项 A: 在 swap.rs 中添加校验（推荐用于 v2.0）
    ///   ```rust,ignore
    ///   // Example (not currently implemented):
    ///   require!(
    ///       market.pool_collateral_reserve >= config.min_trading_liquidity,
    ///       InsufficientLiquidity
    ///   );
    ///   ```
    /// - 选项 B: 移除此字段以避免混淆（保持向后兼容需要账户迁移）
    ///
    /// **当前建议**: 部署时设置为 0 以明确表示未启用
    /// 默认值: 0 (未启用)
    pub min_trading_liquidity: u64,

    /// 配置是否已初始化
    pub initialized: bool,

    /// ✅ 紧急暂停开关
    pub is_paused: bool,

    /// ✅ 白名单开关（true=强制创建者需白名单）
    pub whitelist_enabled: bool,

    // ═══════════════════════════════════════════════════════════════
    // ✅ v1.1.0: USDC 迁移相关字段
    // ═══════════════════════════════════════════════════════════════

    /// USDC Token Mint 地址
    /// 用于验证所有 USDC 操作使用正确的 USDC mint
    pub usdc_mint: Pubkey,

    /// USDC 金库最小余额（用于租金豁免保护）
    /// 防止 USDC 金库余额低于租金豁免要求
    /// 建议值：2000 USDC 最小单位（约 0.002 USDC）
    pub usdc_vault_min_balance: u64,

    /// 最小 USDC 流动性要求（用于 add_liquidity 验证）
    /// 防止添加过少的流动性
    /// 建议值：100 USDC（100 * 10^6）
    pub min_usdc_liquidity: u64,
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
