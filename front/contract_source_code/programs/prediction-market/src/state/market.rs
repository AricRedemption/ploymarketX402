//! 市场状态与AMM逻辑：预测市场的核心实现
//! 
//! 面向Solidity工程师的说明：
//! - 在EVM中，这相当于一个合约的存储结构 + 内部函数
//! - Account<'info, Market> 类似于 Solidity 中的 storage 变量
//! - PDA 类似于合约地址，但由程序ID+种子推导，无需私钥控制
//! - CPI (Cross Program Invocation) 类似于 EVM 中的外部合约调用

// Top-level imports
use crate::state::config::*;
use crate::errors::PredictionMarketError;
use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};
use anchor_spl::token::{Mint, Token};

// use anchor_spl::token::{self};

/// 用户信息账户：记录用户在特定市场中的状态
#[account]
pub struct UserInfo {
    /// 用户公钥
    pub user: Pubkey,     // User's public key
    /// 持有的YES代币数量
    pub yes_balance: u64, // Amount of YES tokens purchased
    /// 持有的NO代币数量
    pub no_balance: u64,  // Amount of NO tokens purchased
    /// 是否为流动性提供者
    pub is_lp: bool,
    /// 是否已初始化
    pub is_initialized: bool,
}

/// 流动性提供者信息
#[account]
pub struct LpInfo {
    /// 用户公钥
    pub user: Pubkey,    // User's public key
    /// 初始提供的SOL数量
    pub sol_amount: u64, // Amount of init lp
}

/// 市场账户：存储预测市场的所有状态
#[account]
pub struct Market {
    /// YES代币铸造地址
    pub yes_token_mint: Pubkey,
    /// NO代币铸造地址
    pub no_token_mint: Pubkey,

    /// 市场创建者
    pub creator: Pubkey,

    /// YES代币相关储备
    pub initial_yes_token_reserves: u64,
    pub real_yes_token_reserves: u64,
    pub real_yes_sol_reserves: u64,
    pub token_yes_total_supply: u64,

    /// NO代币相关储备
    pub initial_no_token_reserves: u64,
    pub real_no_token_reserves: u64,
    pub real_no_sol_reserves: u64,
    pub token_no_total_supply: u64,

    /// 市场状态
    pub is_completed: bool,
    /// 开始槽位（可选）
    pub start_slot: Option<u64>,
    /// 结束槽位（可选）
    pub ending_slot: Option<u64>,

    /// 流动性提供者列表
    pub lps: Vec<LpInfo>,
    /// 总流动性数量
    pub total_lp_amount: u64,
}

/// 卖出结果：包含价格计算和储备更新
#[derive(Debug, Clone)]
pub struct SellResult {
    /// 卖出的代币数量
    pub token_amount: u64,
    /// 获得的SOL数量
    pub change_amount: u64,
    /// 当前YES储备
    pub current_yes_reserves: u64,
    /// 当前NO储备
    pub current_no_reserves: u64,
    /// 新的YES储备
    pub new_yes_reserves: u64,
    /// 新的NO储备
    pub new_no_reserves: u64,
}

/// 买入结果：包含价格计算和储备更新
#[derive(Debug, Clone)]
pub struct BuyResult {
    /// 买入的代币数量
    pub token_amount: u64,
    /// 支付的SOL数量
    pub change_amount: u64,
    /// 当前YES储备
    pub current_yes_reserves: u64,
    /// 当前NO储备
    pub current_no_reserves: u64,
    /// 新的YES储备
    pub new_yes_reserves: u64,
    /// 新的NO储备
    pub new_no_reserves: u64,
}

/// 创建市场参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateMarketParams {
    /// YES代币符号
    pub yes_symbol: String,
    /// YES代币元数据URI
    pub yes_uri: String,

    /// 开始槽位（可选）
    pub start_slot: Option<u64>,
    /// 结束槽位（可选）
    pub ending_slot: Option<u64>,
}

/// 市场账户特征：定义市场相关的所有操作
/// 
/// 面向Solidity工程师：
/// - 这类似于 Solidity 中的 interface 或 abstract contract
/// - 每个方法都相当于合约的内部函数
/// - AccountInfo<'info> 类似于 Solidity 中的 address 类型
/// - CpiContext 类似于 Solidity 中的外部调用上下文
// Replace the erroneous impl-without-bodies block with a trait definition.
// This belongs to: pub trait MarketAccount<'info>
pub trait MarketAccount<'info> {
    /// 执行代币交换（买入/卖出）
    fn swap(
        &mut self,
        global_config: &Account<'info, Config>,

        yes_token_mint: &Account<'info, Mint>,
        global_yes_ata: &mut AccountInfo<'info>,
        user_yes_ata: &mut AccountInfo<'info>,

        no_token_mint: &Account<'info, Mint>,
        global_no_ata: &mut AccountInfo<'info>,
        user_no_ata: &mut AccountInfo<'info>,

        source: &mut AccountInfo<'info>,
        team_wallet: &mut AccountInfo<'info>,

        amount: u64,
        direction: u8,
        token_type: u8,
        minimum_receive_amount: u64,

        user: &Signer<'info>,
        signer: &[&[&[u8]]],

        user_info_pda: &mut Account<'info, UserInfo>,

        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()>;

    fn get_tokens_for_buy_sol(&self, change_amount: u64, token_type: u8) -> Option<BuyResult>;
    fn get_tokens_for_sell_sol(&self, change_amount: u64, token_type: u8) -> Option<SellResult>;

    fn apply_buy(&mut self, change_amount: u64, token_type: u8) -> Option<BuyResult>;
    fn apply_sell(&mut self, change_amount: u64, token_type: u8) -> Option<SellResult>;

    fn resolution(
        &mut self,

        source: &mut AccountInfo<'info>,

        user: &mut AccountInfo<'info>,
        signer: &[&[&[u8]]],
        user_info_pda: &mut Account<'info, UserInfo>,

        token_type: u8,

        system_program: &Program<'info, System>,
    ) -> Result<()>;

    fn add_liquidity(
        &mut self,

        source: &mut AccountInfo<'info>,

        user: &Signer<'info>,
        sol_amount: u64,
        user_info_pda: &mut Account<'info, UserInfo>,

        system_program: &Program<'info, System>,
    ) -> Result<()>;

    fn withdraw_liquidity(
        &mut self,

        source: &mut AccountInfo<'info>,

        user: &Signer<'info>,
        sol_amount: u64,

        signer: &[&[&[u8]]],
        user_info_pda: &mut Account<'info, UserInfo>,

        system_program: &Program<'info, System>,
    ) -> Result<()>;
}

/// 市场账户的具体实现
/// 
/// 面向Solidity工程师：
/// - impl 类似于 Solidity 中的 contract 实现
/// - 这里实现了简化的AMM逻辑（实际项目中需要更复杂的公式）
/// - CPI调用类似于 Solidity 中的外部合约调用
impl<'info> MarketAccount<'info> for Account<'info, Market> {
    /// 交换实现：当前为占位，实际需要完整的AMM逻辑
    fn swap(
        &mut self,
        _global_config: &Account<'info, Config>,

        _yes_token_mint: &Account<'info, Mint>,
        _global_yes_ata: &mut AccountInfo<'info>,
        _user_yes_ata: &mut AccountInfo<'info>,

        _no_token_mint: &Account<'info, Mint>,
        _global_no_ata: &mut AccountInfo<'info>,
        _user_no_ata: &mut AccountInfo<'info>,

        _source: &mut AccountInfo<'info>,
        _team_wallet: &mut AccountInfo<'info>,

        amount: u64,
        direction: u8,
        token_type: u8,
        minimum_receive_amount: u64,

        _user: &Signer<'info>,
        _signer: &[&[&[u8]]],

        user_info_pda: &mut Account<'info, UserInfo>,

        _token_program: &Program<'info, Token>,
        _system_program: &Program<'info, System>,
    ) -> Result<()> {
        if direction == 0 {
            let Some(buy) = self.apply_buy(amount, token_type) else {
                return Err(error!(PredictionMarketError::ArithmeticError));
            };

            if buy.token_amount < minimum_receive_amount {
                return Err(error!(PredictionMarketError::ReturnAmountTooSmall));
            }

            if token_type == 0 {
                user_info_pda.yes_balance = user_info_pda
                    .yes_balance
                    .saturating_add(buy.token_amount);
            } else {
                user_info_pda.no_balance = user_info_pda
                    .no_balance
                    .saturating_add(buy.token_amount);
            }
        } else {
            let Some(sell) = self.apply_sell(amount, token_type) else {
                return Err(error!(PredictionMarketError::ArithmeticError));
            };

            if sell.change_amount < minimum_receive_amount {
                return Err(error!(PredictionMarketError::ReturnAmountTooSmall));
            }

            if token_type == 0 {
                user_info_pda.yes_balance = user_info_pda
                    .yes_balance
                    .saturating_sub(sell.token_amount);
            } else {
                user_info_pda.no_balance = user_info_pda
                    .no_balance
                    .saturating_sub(sell.token_amount);
            }
        }

        Ok(())
    }

    /// 计算买入代币数量（简化AMM公式）
    fn get_tokens_for_buy_sol(&self, change_amount: u64, token_type: u8) -> Option<BuyResult> {
        let current_yes_reserves = self.real_yes_sol_reserves;
        let current_no_reserves = self.real_no_sol_reserves;
        
        // 根据代币类型计算代币数量
        let token_amount = if token_type == 0 {
            // 买入YES代币
            let new_yes_reserves = current_yes_reserves + change_amount;
            let new_no_reserves = current_no_reserves;
            
            // 简化计算 - 实际实现中需要使用正确的AMM公式
            let token_amount = change_amount; // Simplified calculation
            
            BuyResult {
                token_amount,
                change_amount,
                current_yes_reserves,
                current_no_reserves,
                new_yes_reserves,
                new_no_reserves,
            }
        } else {
            // 买入NO代币
            let new_yes_reserves = current_yes_reserves;
            let new_no_reserves = current_no_reserves + change_amount;
            
            // 简化计算 - 实际实现中需要使用正确的AMM公式
            let token_amount = change_amount; // Simplified calculation
            
            BuyResult {
                token_amount,
                change_amount,
                current_yes_reserves,
                current_no_reserves,
                new_yes_reserves,
                new_no_reserves,
            }
        };
        
        Some(token_amount)
    }

    /// 应用买入操作（更新储备）
    fn apply_buy(&mut self, change_amount: u64, token_type: u8) -> Option<BuyResult> {
        // 计算代币数量
        let result = self.get_tokens_for_buy_sol(change_amount, token_type)?;

        Some(result)
    }

    /// 应用卖出操作（更新储备）
    fn apply_sell(&mut self, change_amount: u64, token_type: u8) -> Option<SellResult> {
        // 计算SOL数量
        let result = self.get_tokens_for_sell_sol(change_amount, token_type)?;

        Some(result)
    }

    /// 计算卖出SOL数量（简化AMM公式）
    fn get_tokens_for_sell_sol(&self, token_amount: u64, token_type: u8) -> Option<SellResult> {
        let current_yes_reserves = self.real_yes_sol_reserves;
        let current_no_reserves = self.real_no_sol_reserves;
        
        // 根据代币类型计算SOL数量
        let change_amount = if token_type == 0 {
            // 卖出YES代币
            let new_yes_reserves = current_yes_reserves.saturating_sub(token_amount);
            let new_no_reserves = current_no_reserves;
            
            // 简化计算 - 实际实现中需要使用正确的AMM公式
            let change_amount = token_amount; // Simplified calculation
            
            SellResult {
                token_amount,
                change_amount,
                current_yes_reserves,
                current_no_reserves,
                new_yes_reserves,
                new_no_reserves,
            }
        } else {
            // 卖出NO代币
            let new_yes_reserves = current_yes_reserves;
            let new_no_reserves = current_no_reserves.saturating_sub(token_amount);
            
            // 简化计算 - 实际实现中需要使用正确的AMM公式
            let change_amount = token_amount; // Simplified calculation
            
            SellResult {
                token_amount,
                change_amount,
                current_yes_reserves,
                current_no_reserves,
                new_yes_reserves,
                new_no_reserves,
            }
        };
        
        Some(change_amount)
    }

    /// 市场结算实现（占位）
    fn resolution(
        &mut self,

        _source: &mut AccountInfo<'info>,

        _user: &mut AccountInfo<'info>,
        _signer: &[&[&[u8]]],
        _user_info_pda: &mut Account<'info, UserInfo>,

        _token_type: u8,

        _system_program: &Program<'info, System>,
    ) -> Result<()> {
        Ok(())
    }

    /// 添加流动性实现（占位）
    fn add_liquidity(
        &mut self,

        _source: &mut AccountInfo<'info>,

        _user: &Signer<'info>,
        _sol_amount: u64,

        _user_info_pda: &mut Account<'info, UserInfo>,

        _system_program: &Program<'info, System>,
    ) -> Result<()> {
        Ok(())
    }

    /// 提取流动性实现（占位）
    fn withdraw_liquidity(
        &mut self,

        _source: &mut AccountInfo<'info>,

        _user: &Signer<'info>,
        _sol_amount: u64,
        _signer: &[&[&[u8]]],
        _user_info_pda: &mut Account<'info, UserInfo>,

        _system_program: &Program<'info, System>,
    ) -> Result<()> {
        Ok(())
    }
}

/*
面向Solidity工程师的详细对照说明：

1. 账户模型 vs 存储：
   Solidity: contract MyContract { mapping(address => uint256) balances; }
   Anchor:   #[account] pub struct UserInfo { pub user: Pubkey; pub balance: u64; }
   
   - Solidity的存储是合约内部的变量
   - Anchor的账户是独立的、可传递的对象

2. 权限控制：
   Solidity: modifier onlyOwner() { require(msg.sender == owner); }
   Anchor:   #[account(mut, constraint = global_config.authority == *admin.key)]
   
   - Solidity使用modifier和require
   - Anchor使用账户约束和require!宏

3. 外部调用：
   Solidity: IERC20(token).transfer(to, amount);
   Anchor:   token::transfer(cpi_ctx, amount)?;
   
   - Solidity直接调用接口
   - Anchor使用CPI (Cross Program Invocation)

4. 事件：
   Solidity: event Trade(address user, uint256 amount);
   Anchor:   #[event] pub struct TradeEvent { pub user: Pubkey; pub amount: u64; }
   
   - 两者概念相同，但Anchor需要显式定义结构体

5. PDA vs 合约地址：
   Solidity: address constant VAULT = 0x123...;
   Anchor:   let (vault, bump) = Pubkey::find_program_address(&[b"vault"], program_id);
   
   - Solidity使用固定地址或CREATE2
   - Anchor使用PDA，由程序ID+种子推导

6. 错误处理：
   Solidity: revert("Insufficient balance");
   Anchor:   return Err(ErrorCode::InsufficientBalance.into());
   
   - Solidity使用revert
   - Anchor使用Result<()>和自定义错误枚举

7. 初始化：
   Solidity: constructor() { owner = msg.sender; }
   Anchor:   #[account(init, payer = user, space = 8 + std::mem::size_of::<Market>())]
   
   - Solidity在构造函数中初始化
   - Anchor使用init约束和显式的空间分配
*/

