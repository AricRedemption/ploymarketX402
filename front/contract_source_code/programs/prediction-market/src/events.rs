//! # 事件定义模块
//! 
//! 定义预测市场合约中发出的各种事件
//! 事件用于记录重要的状态变化和操作，便于前端监听和索引

use anchor_lang::prelude::*;

/// 全局更新事件
/// 
/// 当全局配置发生更新时发出
/// 包括管理员变更、代币配置更新等
#[event]
pub struct GlobalUpdateEvent {
    /// 全局管理员公钥
    pub global_authority: Pubkey,
    
    /// 初始真实代币储备
    pub initial_real_token_reserves: u64,
    
    /// 代币总供应量
    pub token_total_supply: u64,
    
    /// 代币精度
    pub mint_decimals: u8,
}

/// 市场创建事件
/// 
/// 当新的预测市场被创建时发出
/// 包含市场的基本信息和代币配置
#[event]
pub struct CreateEvent {
    /// 市场创建者
    pub creator: Pubkey,
    
    /// 市场账户地址
    pub market: Pubkey,

    /// YES代币地址
    pub token_yes: Pubkey,
    
    /// YES代币元数据地址
    pub metadata_yes: Pubkey,
    
    /// YES代币总供应量
    pub token_yes_total_supply: u64,
    
    /// YES代币真实SOL储备
    pub real_yes_sol_reserves: u64,

    /// NO代币地址
    pub token_no: Pubkey,
    
    /// NO代币元数据地址
    pub metadata_no: Pubkey,
    
    /// NO代币总供应量
    pub token_no_total_supply: u64,
    
    /// NO代币真实SOL储备
    pub real_no_sol_reserves: u64,

    /// 开始槽位
    pub start_slot: u64,
    
    /// 结束槽位
    pub ending_slot: u64,
}

/// 提取事件
/// 
/// 当从金库提取资金时发出
/// 用于记录手续费提取等操作
#[event]
pub struct WithdrawEvent {
    /// 提取授权者
    pub withdraw_authority: Pubkey,
    
    /// 代币铸造地址
    pub mint: Pubkey,
    
    /// 手续费金库地址
    pub fee_vault: Pubkey,

    /// 本次提取数量
    pub withdrawn: u64,
    
    /// 累计提取数量
    pub total_withdrawn: u64,

    /// 提取时间戳
    pub withdraw_time: i64,
}

/// 交易事件
/// 
/// 当用户进行代币交易时发出
/// 包含交易的详细信息，便于分析和索引
#[event]
pub struct TradeEvent {
    /// 交易用户
    pub user: Pubkey,
    
    /// YES代币地址
    pub token_yes: Pubkey,
    
    /// NO代币地址
    pub token_no: Pubkey,
    
    /// 市场信息账户
    pub market_info: Pubkey,

    /// SOL交易数量
    pub sol_amount: u64,
    
    /// 代币交易数量
    pub token_amount: u64,
    
    /// 手续费（lamports）
    pub fee_lamports: u64,
    
    /// 是否为买入操作
    pub is_buy: bool,
    
    /// 是否为YES代币交易
    pub is_yes_no: bool,

    /// 真实SOL储备
    pub real_sol_reserves: u64,
    
    /// 真实YES代币储备
    pub real_token_yes_reserves: u64,
    
    /// 真实NO代币储备
    pub real_token_no_reserves: u64,

    /// 交易时间戳
    pub timestamp: i64,
}

/// 完成事件
/// 
/// 当市场完成或曲线完成时发出
/// 记录最终的状态信息
#[event]
pub struct CompleteEvent {
    /// 操作用户
    pub user: Pubkey,
    
    /// 代币铸造地址
    pub mint: Pubkey,
    
    /// 虚拟SOL储备
    pub virtual_sol_reserves: u64,
    
    /// 虚拟代币储备
    pub virtual_token_reserves: u64,
    
    /// 真实SOL储备
    pub real_sol_reserves: u64,
    
    /// 真实代币储备
    pub real_token_reserves: u64,
    
    /// 完成时间戳
    pub timestamp: i64,
}

/// 事件转换特征
/// 
/// 提供将结构体转换为事件的通用接口
/// 用于简化事件创建过程
pub trait IntoEvent<T: anchor_lang::Event> {
    /// 将当前结构体转换为指定的事件类型
    /// 
    /// # 返回
    /// * `T` - 转换后的事件
    fn into_event(&self) -> T;
}
