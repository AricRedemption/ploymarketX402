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
pub mod math;       // 数学库（定点数、LMSR）
pub mod state;      // 状态结构定义
pub mod utils;      // 工具函数

// 导入指令模块
use instructions::{
    accept_authority::*, add_liquidity::*, add_to_whitelist::*, claim_lp_fees::*, claim_rewards::*,
    configure::*, create_market::*, mint_complete_set::*, mint_no_token::*, nominate_authority::*,
    pause::*, redeem_complete_set::*, remove_from_whitelist::*, resolution::*, seed_pool::*,
    settle_pool::*, swap::*, withdraw_liquidity::*,
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
        ctx.accounts.handler(new_config, ctx.bumps.config, ctx.bumps.global_vault)
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
    /// * `token_type` - 代币类型（0=NO，1=YES）
    /// * `minimum_receive_amount` - 最小接收数量（滑点保护）
    /// * `deadline` - 交易截止时间戳（Unix timestamp），设为 0 则不检查
    ///
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn swap(
        ctx: Context<Swap>,
        amount: u64,
        direction: u8,
        token_type: u8,
        minimum_receive_amount: u64,
        deadline: i64,
    ) -> Result<()> {
        ctx.accounts.handler(
            amount,
            direction,
            token_type,
            minimum_receive_amount,
            deadline,
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
    /// ✅ 双账本系统：只操作 Pool Ledger
    /// 用户向 AMM Pool 添加 USDC + YES + NO 代币，获得 LP 份额
    /// LP 可以获得交易手续费分成
    ///
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `usdc_amount` - 添加的 USDC 数量
    /// * `yes_amount` - 添加的YES代币数量
    /// * `no_amount` - 添加的NO代币数量
    ///
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        usdc_amount: u64,
        yes_amount: u64,
        no_amount: u64,
    ) -> Result<()> {
        ctx.accounts.handler(
            usdc_amount,
            yes_amount,
            no_amount,
            ctx.bumps.global_vault,
        )
    }

    /// 提取流动性
    ///
    /// ✅ 双账本系统：只操作 Pool Ledger
    /// LP 赎回份额，获得按比例的 USDC + YES + NO 代币
    ///
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `lp_shares_to_burn` - 要赎回的 LP 份额数量
    ///
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn withdraw_liquidity(
        ctx: Context<WithdrawLiquidity>,
        lp_shares_to_burn: u64,
    ) -> Result<()> {
        ctx.accounts
            .handler(lp_shares_to_burn, ctx.bumps.global_vault)
    }

    /// 铸造完整集合（条件代币核心功能）
    ///
    /// 用户存入 USDC，获得等量的 YES + NO 代币
    /// 这是 Polymarket 条件代币机制的核心：1 USDC = 1 YES + 1 NO
    ///
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `amount` - USDC 数量
    ///
    /// # 返回
    /// * `Result<()>` - 操作结果
    ///
    /// # 示例
    /// 用户存入 1 USDC → 获得 1 YES + 1 NO
    /// 这确保了 YES + NO 的价值等于抵押品
    pub fn mint_complete_set(ctx: Context<MintCompleteSet>, amount: u64) -> Result<()> {
        ctx.accounts.handler(amount, ctx.bumps.global_vault)
    }

    /// 赎回完整集合（条件代币核心功能）
    ///
    /// 用户销毁等量的 YES + NO 代币，赎回 USDC
    /// 与 mint_complete_set 相反的操作
    ///
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `amount` - 赎回数量
    ///
    /// # 返回
    /// * `Result<()>` - 操作结果
    ///
    /// # 示例
    /// 用户提供 1 YES + 1 NO → 赎回 1 USDC
    /// 这是套利者平衡市场价格的关键机制
    pub fn redeem_complete_set(ctx: Context<RedeemCompleteSet>, amount: u64, global_vault_bump: u8) -> Result<()> {
        ctx.accounts.handler(amount, global_vault_bump)
    }

    /// 暂停合约
    ///
    /// 管理员调用以紧急暂停所有市场操作
    ///
    /// # 参数
    /// * `ctx` - 指令上下文
    ///
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        ctx.accounts.pause()
    }

    /// 恢复合约
    ///
    /// 管理员调用以恢复合约操作
    ///
    /// # 参数
    /// * `ctx` - 指令上下文
    ///
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn unpause(ctx: Context<Pause>) -> Result<()> {
        ctx.accounts.unpause()
    }

    /// 添加创建者到白名单
    ///
    /// ✅ v1.0.16: 新增白名单管理指令
    /// 管理员调用以将创建者地址添加到白名单
    /// 只有在 whitelist_enabled=true 时才需要白名单
    ///
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `creator` - 要添加到白名单的创建者公钥
    ///
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn add_to_whitelist(ctx: Context<AddToWhitelist>, creator: Pubkey) -> Result<()> {
        ctx.accounts.handler(creator)
    }

    /// 从白名单移除创建者
    ///
    /// ✅ v1.0.16: 新增白名单管理指令
    /// 管理员调用以从白名单移除创建者地址
    /// 移除后该创建者将无法创建新市场（如果 whitelist_enabled=true）
    ///
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `creator` - 要从白名单移除的创建者公钥
    ///
    /// # 返回
    /// * `Result<()>` - 操作结果
    pub fn remove_from_whitelist(ctx: Context<RemoveFromWhitelist>, creator: Pubkey) -> Result<()> {
        ctx.accounts.handler(creator)
    }

    /// 领取奖励
    ///
    /// 用户在市场结算后调用，根据 resolution 比例领取奖励
    ///
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `global_vault_bump` - 全局金库 bump
    ///
    /// # 返回
    /// * `Result<()>` - 操作结果
    ///
    /// # 示例
    /// 市场结算后，YES获胜(100%)，用户持有10 YES → 获得10 USDC
    /// 如果是平局(50%/50%)，用户持有10 YES + 10 NO → 获得10 USDC
    pub fn claim_rewards(ctx: Context<ClaimRewards>, global_vault_bump: u8) -> Result<()> {
        ctx.accounts.handler(global_vault_bump)
    }

    /// Pool 初始化
    ///
    /// ✅ 双账本系统：只操作 Pool Ledger
    /// 为新创建的市场注入初始流动性，解决"鸡蛋问题"
    /// - 自动铸造 YES + NO 代币到 Pool
    /// - 初始化 LMSR 参数
    /// - 可选给种子提供者铸造 LP 份额
    ///
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `usdc_amount` - 注入的 USDC 数量
    /// * `issue_lp_shares` - 是否给种子提供者铸造 LP 份额
    /// * `global_vault_bump` - 全局金库 PDA bump
    ///
    /// # 返回
    /// * `Result<()>` - 操作结果
    ///
    /// # 注意
    /// - 只能由管理员或市场创建者调用
    /// - 每个市场只能调用一次
    pub fn seed_pool(
        ctx: Context<SeedPool>,
        usdc_amount: u64,
        issue_lp_shares: bool,
        global_vault_bump: u8,
    ) -> Result<()> {
        ctx.accounts
            .handler(usdc_amount, issue_lp_shares, global_vault_bump)
    }

    /// Pool 结算
    ///
    /// ✅ 双账本系统：只操作 Pool Ledger
    /// 市场结束后，处理 Pool 中剩余的代币资产
    /// - 获胜方代币：保留给 LP 提取
    /// - 失败方代币：转移给团队钱包
    /// - USDC 储备：保留给 LP 提取
    ///
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `global_vault_bump` - 全局金库 PDA bump
    ///
    /// # 返回
    /// * `Result<()>` - 操作结果
    ///
    /// # 注意
    /// - 只能由管理员在市场结束后调用
    /// - LP 仍可通过 withdraw_liquidity 提取剩余资产
    pub fn settle_pool(ctx: Context<SettlePool>, global_vault_bump: u8) -> Result<()> {
        ctx.accounts.handler(global_vault_bump)
    }

    /// LP 费用领取
    ///
    /// ✅ 双账本系统：只操作 Pool Ledger
    /// LP 按比例领取累积的交易手续费
    /// - 手续费来自 swap 交易中收取的 LP 费用部分
    /// - 按 LP 份额占比分配
    /// - 更新 last_fee_claim_slot 防止重复领取
    ///
    /// # 参数
    /// * `ctx` - 指令上下文
    /// * `global_vault_bump` - 全局金库 PDA bump
    ///
    /// # 返回
    /// * `Result<()>` - 操作结果
    ///
    /// # 注意
    /// - LP 可随时领取累积的手续费
    /// - 手续费从 accumulated_lp_fees 中扣除
    /// - 建议定期领取，避免累积过多
    pub fn claim_lp_fees(ctx: Context<ClaimLpFees>, global_vault_bump: u8) -> Result<()> {
        ctx.accounts.handler(global_vault_bump)
    }
}
