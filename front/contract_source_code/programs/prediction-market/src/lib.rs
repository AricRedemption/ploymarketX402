//! # Solana 预测市场合约主程序
//! 
//! 这是一个基于Solana区块链的去中心化预测市场平台，灵感来源于Polymarket。
//! 该平台允许用户创建市场、交易头寸，并根据现实世界事件解决结果。
//! 
//! ## 主要功能
//! - 创建预测市场
//! - 买卖YES/NO代币
//! - 流动性管理
//! - 市场结算
//! - 权限管理

use anchor_lang::prelude::*;

// 模块声明
pub mod constants;  // 常量定义
pub mod errors;     // 错误类型定义
pub mod events;     // 事件定义
pub mod instructions; // 指令实现
pub mod state;      // 状态结构定义
pub mod utils;      // 工具函数

// 导入指令模块
use instructions::{
    accept_authority::*, add_liquidity::*, configure::*, create_market::*, mint_no_token::*,
    nominate_authority::*, resolution::*, swap::*, withdraw_liquidity::*,
};

// 导入状态模块
use state::config::*;
use state::market::*;

// 声明程序ID
declare_id!("EgEc7fuse6eQ3UwqeWGFncDtbTwozWCy4piydbeRaNrU");

/// 预测市场程序主模块
#[program]
pub mod prediction_market {
    use super::*;

    /// 配置全局设置
    /// 
    /// 由管理员调用，用于设置全局配置参数
    /// 需要验证调用者是否为授权管理员
    /// 
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `new_config` - 新的配置参数
    /// 
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn configure(ctx: Context<Configure>, new_config: Config) -> Result<()> {
        msg!("configure: {:#?}", new_config);
        ctx.accounts.handler(new_config, ctx.bumps.config)
    }

    /// 提名新的管理员
    /// 
    /// 当前管理员可以将管理员角色转移给其他账户
    /// 这是一个两步过程，需要新管理员接受才能完成转移
    /// 
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `new_admin` - 新管理员的公钥
    /// 
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn nominate_authority(ctx: Context<NominateAuthority>, new_admin: Pubkey) -> Result<()> {
        ctx.accounts.process(new_admin)
    }

    /// 接受管理员角色
    /// 
    /// 被提名的管理员调用此函数来接受管理员角色
    /// 只有在被提名后才能调用此函数
    /// 
    /// # 参数
    /// * `ctx` - 指令上下文
    /// 
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn accept_authority(ctx: Context<AcceptAuthority>) -> Result<()> {
        ctx.accounts.process()
    }

    /// 铸造NO代币
    /// 
    /// 为预测市场创建NO代币（表示"不同意"的代币）
    /// 每个市场都需要一对YES和NO代币
    /// 
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `no_symbol` - NO代币的符号
    /// * `no_uri` - NO代币的元数据URI
    /// 
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn mint_no_token(
        ctx: Context<MintNoToken>,
        no_symbol: String,
        no_uri: String,
    ) -> Result<()> {
        ctx.accounts
            .handler(no_symbol, no_uri, ctx.bumps.global_vault)
    }

    /// 创建预测市场
    /// 
    /// 创建一个新的预测市场，包括YES代币的铸造
    /// 市场创建者需要提供市场的基本信息
    /// 
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `params` - 创建市场的参数
    /// 
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn create_market(ctx: Context<CreateMarket>, params: CreateMarketParams) -> Result<()> {
        ctx.accounts.handler(params, ctx.bumps.global_vault)
    }

    /// 交易代币
    /// 
    /// 在预测市场中买卖YES或NO代币
    /// 使用AMM（自动做市商）机制进行价格发现
    /// 
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `amount` - 交易数量
    /// * `direction` - 交易方向（0=买入，1=卖出）
    /// * `token_type` - 代币类型（0=YES，1=NO）
    /// * `minimum_receive_amount` - 最小接收数量（滑点保护）
    /// 
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn swap(
        ctx: Context<Swap>,
        amount: u64,
        direction: u8,
        token_type: u8,
        minimum_receive_amount: u64,
    ) -> Result<()> {
        ctx.accounts.handler(
            amount,
            direction,
            token_type,
            minimum_receive_amount,
            ctx.bumps.global_vault,
        )
    }

    /// 市场结算
    /// 
    /// 由管理员调用，用于结算预测市场的结果
    /// 根据实际结果分配奖励给持有正确代币的用户
    /// 
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `yes_amount` - YES代币的奖励数量
    /// * `no_amount` - NO代币的奖励数量
    /// * `token_type` - 获胜的代币类型
    /// * `is_completed` - 市场是否完成
    /// 
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn resolution(
        ctx: Context<Resolution>,
        yes_amount: u64,
        no_amount: u64,
        token_type: u8,
        is_completed: bool,
    ) -> Result<()> {
        ctx.accounts.handler(
            yes_amount,
            no_amount,
            token_type,
            is_completed,
            ctx.bumps.global_vault,
        )
    }

    /// 添加流动性
    /// 
    /// 用户可以向市场添加流动性，成为流动性提供者（LP）
    /// LP可以获得交易手续费分成
    /// 
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `amount` - 添加的SOL数量
    /// 
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn add_liquidity(ctx: Context<AddLiquidity>, amount: u64) -> Result<()> {
        ctx.accounts.handler(amount)
    }

    /// 提取流动性
    /// 
    /// 流动性提供者可以提取之前添加的流动性
    /// 只能提取自己添加的流动性
    /// 
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `amount` - 提取的SOL数量
    /// 
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn withdraw_liquidity(ctx: Context<WithdrawLiquidity>, amount: u64) -> Result<()> {
        ctx.accounts.handler(amount, ctx.bumps.global_vault)
    }
}
