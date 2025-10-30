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

    /// NO代币地址
    pub token_no: Pubkey,

    /// NO代币元数据地址
    pub metadata_no: Pubkey,

    /// NO代币总供应量
    pub token_no_total_supply: u64,

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

    /// ✅ v1.1.0: USDC 交易数量（原 sol_amount）
    pub usdc_amount: u64,

    /// 代币交易数量
    pub token_amount: u64,

    /// ✅ v1.1.0: 手续费（USDC 单位，原 fee_lamports）
    pub fee_usdc: u64,
    
    /// 是否为买入操作
    pub is_buy: bool,
    
    /// 是否为YES代币交易
    pub is_yes_no: bool,

    /// ✅ v1.1.0: 真实 USDC 储备（原 real_sol_reserves）
    pub real_usdc_reserves: u64,

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
    
    /// ✅ v1.1.0: 虚拟 USDC 储备（原 virtual_sol_reserves）
    pub virtual_usdc_reserves: u64,

    /// 虚拟代币储备
    pub virtual_token_reserves: u64,

    /// ✅ v1.1.0: 真实 USDC 储备（原 real_sol_reserves）
    pub real_usdc_reserves: u64,

    /// 真实代币储备
    pub real_token_reserves: u64,
    
    /// 完成时间戳
    pub timestamp: i64,
}

/// 添加流动性事件
///
/// 当LP添加流动性时发出
#[event]
pub struct AddLiquidityEvent {
    /// LP用户
    pub user: Pubkey,

    /// 市场账户
    pub market: Pubkey,

    /// ✅ v1.1.0: 添加的 USDC 数量（原 sol_amount）
    pub usdc_amount: u64,

    /// 添加的YES代币数量
    pub yes_amount: u64,

    /// 添加的NO代币数量
    pub no_amount: u64,

    /// 铸造的LP份额数量
    pub lp_shares_minted: u64,

    /// 时间戳
    pub timestamp: i64,
}

/// 提取流动性事件
///
/// 当LP提取流动性时发出
#[event]
pub struct WithdrawLiquidityEvent {
    /// LP用户
    pub user: Pubkey,

    /// 市场账户
    pub market: Pubkey,

    /// 销毁的LP份额数量
    pub lp_shares_burned: u64,

    /// ✅ v1.1.0: 返还的 USDC 数量（原 sol_amount）
    pub usdc_amount: u64,

    /// 返还的YES代币数量
    pub yes_amount: u64,

    /// 返还的NO代币数量
    pub no_amount: u64,

    /// 时间戳
    pub timestamp: i64,
}

/// 市场解决事件
///
/// 当市场结算时发出
#[event]
pub struct ResolutionEvent {
    /// 管理员
    pub authority: Pubkey,

    /// 市场账户
    pub market: Pubkey,

    /// 获胜方代币类型 (0=NO, 1=YES, 2=平局)
    pub winner_token_type: u8,

    /// YES方比例
    pub yes_ratio: u64,

    /// NO方比例
    pub no_ratio: u64,

    /// 时间戳
    pub timestamp: i64,
}

/// 用户领取奖励事件
///
/// 当用户在市场结算后领取奖励时发出
#[event]
pub struct ClaimRewardsEvent {
    /// 用户
    pub user: Pubkey,

    /// 市场账户
    pub market: Pubkey,

    /// 销毁的 YES 代币数量
    pub yes_burned: u64,

    /// 销毁的 NO 代币数量
    pub no_burned: u64,

    /// ✅ v1.1.0: 用户收到的 USDC 奖励（原 sol_payout）
    pub usdc_payout: u64,

    /// 时间戳
    pub timestamp: i64,
}

/// Pool 结算事件
///
/// 当管理员结算 Pool 时发出
#[event]
pub struct SettlePoolEvent {
    /// 管理员
    pub authority: Pubkey,

    /// 市场账户
    pub market: Pubkey,

    /// 获胜方代币类型 (0=NO, 1=YES, 2=平局)
    pub winner_token_type: u8,

    /// Pool 中销毁的输家代币数量 (v1.0.28: 改为销毁而非转移)
    pub loser_tokens_burned: u64,

    /// ✅ v1.1.0: Pool 释放的 USDC 数量（原 sol_released）
    /// 注意：settle_pool 不释放 USDC，保留给 LP 提取，此字段保持为 0
    pub usdc_released: u64,

    /// 时间戳
    pub timestamp: i64,
}

/// 暂停合约事件
///
/// 当管理员暂停合约时发出
#[event]
pub struct PauseEvent {
    /// 管理员
    pub authority: Pubkey,

    /// 时间戳
    pub timestamp: i64,
}

/// 恢复合约事件
///
/// 当管理员恢复合约时发出
#[event]
pub struct UnpauseEvent {
    /// 管理员
    pub authority: Pubkey,

    /// 时间戳
    pub timestamp: i64,
}

/// 白名单更新事件
///
/// 当管理员添加或移除白名单地址时发出
#[event]
pub struct WhitelistUpdateEvent {
    /// 管理员
    pub authority: Pubkey,

    /// 被添加/移除的地址
    pub target: Pubkey,

    /// 操作类型: true=添加, false=移除
    pub is_add: bool,

    /// 时间戳
    pub timestamp: i64,
}

/// LP 费用领取事件
///
/// 当 LP 领取手续费时发出
#[event]
pub struct ClaimLpFeesEvent {
    /// LP 地址
    pub lp: Pubkey,

    /// 市场账户
    pub market: Pubkey,

    /// ✅ v1.1.0: 领取的费用数量（USDC 最小单位）
    pub fees_claimed: u64,

    /// LP 持有的份额
    pub lp_shares: u64,

    /// 领取前的累积费用
    pub accumulated_fees_before: u64,

    /// 领取后的累积费用
    pub accumulated_fees_after: u64,

    /// 时间戳
    pub timestamp: i64,
}

/// ✅ v1.1.1: 配置更新事件（修复审计发现的日志记录不全问题）
///
/// 当全局配置被初始化或更新时发出
/// 用于追踪关键参数变更，便于监控和审计
#[event]
pub struct ConfigUpdateEvent {
    /// 操作者（初始化者或管理员）
    pub authority: Pubkey,

    /// 是否为初始化操作（true=首次创建，false=更新）
    pub is_initialization: bool,

    /// 新的权限地址
    pub new_authority: Pubkey,

    /// 团队钱包地址
    pub team_wallet: Pubkey,

    /// ✅ LMSR b 参数（流动性深度配置）
    pub initial_real_token_reserves_config: u64,

    /// 代币总供应配置
    pub token_supply_config: u64,

    /// 代币精度配置
    pub token_decimals_config: u8,

    /// 平台买入手续费（基点）
    pub platform_buy_fee: u64,

    /// 平台卖出手续费（基点）
    pub platform_sell_fee: u64,

    /// LP 买入手续费（基点）
    pub lp_buy_fee: u64,

    /// LP 卖出手续费（基点）
    pub lp_sell_fee: u64,

    /// 是否暂停
    pub is_paused: bool,

    /// 是否启用白名单
    pub whitelist_enabled: bool,

    /// 时间戳
    pub timestamp: i64,
}

/// ✅ v1.1.1: 铸造完整集合事件（修复事件记录缺失）
///
/// 当用户铸造 YES+NO 完整集合时发出
#[event]
pub struct MintCompleteSetEvent {
    /// 用户
    pub user: Pubkey,

    /// 市场账户
    pub market: Pubkey,

    /// USDC 抵押数量
    pub usdc_locked: u64,

    /// 铸造的 YES 代币数量
    pub yes_minted: u64,

    /// 铸造的 NO 代币数量
    pub no_minted: u64,

    /// 时间戳
    pub timestamp: i64,
}

/// ✅ v1.1.1: 赎回完整集合事件
///
/// 当用户赎回 YES+NO 换回 USDC 时发出
#[event]
pub struct RedeemCompleteSetEvent {
    /// 用户
    pub user: Pubkey,

    /// 市场账户
    pub market: Pubkey,

    /// 销毁的 YES 代币数量
    pub yes_burned: u64,

    /// 销毁的 NO 代币数量
    pub no_burned: u64,

    /// 返还的 USDC 数量
    pub usdc_returned: u64,

    /// 时间戳
    pub timestamp: i64,
}

/// ✅ v1.1.1: 种子流动性注入事件
///
/// 当管理员或创建者为 Pool 注入初始流动性时发出
#[event]
pub struct SeedPoolEvent {
    /// 种子提供者
    pub seeder: Pubkey,

    /// 市场账户
    pub market: Pubkey,

    /// 注入的 USDC 数量
    pub usdc_amount: u64,

    /// 注入的 YES 代币数量
    pub yes_amount: u64,

    /// 注入的 NO 代币数量
    pub no_amount: u64,

    /// 铸造的 LP 份额
    pub lp_shares_minted: u64,

    /// 时间戳
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
