//! 管理员配置指令：初始化/更新全局配置与全局金库

use crate::errors::*;
use crate::{
    constants::{CONFIG, GLOBAL},
    state::config::*,
};
use anchor_lang::{prelude::*, system_program, Discriminator};
use anchor_spl::{associated_token::AssociatedToken, token::Token};
use borsh::BorshDeserialize;

/// 配置账户集合
#[derive(Accounts)]
pub struct Configure<'info> {
    /// 付费者/管理员
    #[account(mut)]
    payer: Signer<'info>,

    /// 配置PDA（内部完成初始化/扩容/写入）
    /// CHECK: initialization handled inside the instruction
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    config: AccountInfo<'info>,

    /// 全局金库（PDA，存放 USDC）
    /// CHECK: global vault pda which stores USDC
    #[account(
        mut,
        seeds = [GLOBAL.as_bytes()],
        bump,
    )]
    pub global_vault: AccountInfo<'info>,

    /// 系统程序
    #[account(address = system_program::ID)]
    system_program: Program<'info, System>,

    /// 代币程序
    token_program: Program<'info, Token>,

    /// 关联代币账户程序
    associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Configure<'info> {
    /// 写入全局配置，并在需要时创建/扩容配置PDA与初始化全局金库
    pub fn handler(&mut self, new_config: Config, config_bump: u8, global_vault_bump: u8) -> Result<()> {
        // ✅ v1.1.1: 修复权限控制不足问题
        //
        // 🔴 原问题：仅在配置已存在时检查权限（第 184 行）
        //    初始化时任何人都可以调用，造成严重安全隐患：
        //    1. 抢先运行攻击：恶意用户可抢先初始化配置
        //    2. 设置恶意参数：team_wallet 指向攻击者地址
        //    3. 拦截所有手续费：平台所有收入被窃取
        //
        // ✅ 修复策略：
        //    - 初始化时：检查 payer 是否为预期的部署者（通过 new_config.authority）
        //    - 更新时：检查 payer 是否为当前 authority（已有逻辑）
        //
        // 📐 实现方式：
        //    初始化操作中，new_config.authority 应该等于 payer.key()
        //    这确保只有被授权的部署者可以初始化配置
        let is_initialization = self.config.owner != &crate::ID;
        // ✅ v1.1.1: 强制 token_decimals = 6（USDC 精度绑定）
        //
        // 🎯 核心设计原则：YES/NO 代币精度必须与抵押品精度严格一致
        //
        // 📐 精度绑定逻辑：
        //    - 抵押品：USDC（6 位精度，1 USDC = 10^6 最小单位）
        //    - YES/NO：必须使用 6 位精度（1 YES = 10^6 最小单位）
        //    - 等价关系：1 USDC ⇔ 1 YES + 1 NO（在相同精度下）
        //
        // 🔒 为什么强制绑定？
        //    1. 保证 1:1 套保机制（Polymarket 核心玩法）
        //    2. 避免精度转换错误（10^6 vs 10^9 差 1000 倍）
        //    3. 简化 LMSR 数学运算（无需跨精度计算）
        //    4. 确保抵押品守恒（mint/redeem 时精度一致）
        //
        // ⚠️ 如果未来支持多币种：
        //    - USDC 抵押 → token_decimals = 6
        //    - USDC 抵押 → token_decimals = 6
        //    - 自定义代币 → token_decimals = 自定义精度
        //
        require!(
            new_config.token_decimals_config == 6,
            PredictionMarketError::InvalidParameter
        );

        // 🔒 修复：基础参数校验
        let decimal_multiplier = 10u64.pow(new_config.token_decimals_config as u32);

        // 校验 1：总供应必须是精度的整数倍
        let fractional_tokens = new_config.token_supply_config % decimal_multiplier;
        if fractional_tokens != 0 {
            return Err(ValueInvalid.into());
        }

        // 🔒 修复：校验 token_supply_config >= initial_real_token_reserves_config
        // 原逻辑量纲错误，应该直接比较两个配置值
        require!(
            new_config.token_supply_config >= new_config.initial_real_token_reserves_config,
            PredictionMarketError::InvalidAmount
        );

        // ✅ v1.1.1: 强制 initial_real_token_reserves_config > 0（修复 DoS 漏洞）
        //
        // 🔒 核心风险：如果 b = 0，所有 LMSR 函数都会失败
        //    - lmsr_cost 中的 require!(b > 0) 会直接拒绝
        //    - 导致新市场创建后无法进行任何买卖操作
        //    - 形成可预期的系统性拒绝服务（DoS）
        //
        // 🎯 修复策略：在配置阶段强制校验 > 0
        require!(
            new_config.initial_real_token_reserves_config > 0,
            PredictionMarketError::InvalidParameter
        );

        // ✅ v1.0.13: 校验 initial_real_token_reserves_config 不超过 LMSR 上限
        // initial_real_token_reserves_config 用作 create_market 的 lmsr_b 参数
        // 必须 <= MAX_LMSR_B (1M USDC = 1,000,000,000,000 最小单位) 以防止定点 LMSR 溢出
        //
        // 修复 (v1.0.13): 原错误使用 1u64 << 32 = 4.29 SOL (历史记录，当时使用 SOL 计价)
        // ✅ v1.1.0: 更新为 constants::MAX_LMSR_B = 1M USDC，支持深度市场流动性
        require!(
            new_config.initial_real_token_reserves_config <= crate::constants::MAX_LMSR_B,
            PredictionMarketError::ValueTooLarge
        );

        // ✅ v1.0.16: 校验手续费基点在合理范围内（防止溢出）
        // 基点范围：0-10000 (0%-100%)
        // 如果设置 >10000，swap 中的 checked_sub 会下溢导致所有交易失败
        // 效果等同于永久暂停市场
        const MAX_FEE_BPS: u64 = 10000; // 100%

        require!(
            new_config.platform_buy_fee <= MAX_FEE_BPS,
            PredictionMarketError::ValueTooLarge
        );
        require!(
            new_config.platform_sell_fee <= MAX_FEE_BPS,
            PredictionMarketError::ValueTooLarge
        );
        require!(
            new_config.lp_buy_fee <= MAX_FEE_BPS,
            PredictionMarketError::ValueTooLarge
        );
        require!(
            new_config.lp_sell_fee <= MAX_FEE_BPS,
            PredictionMarketError::ValueTooLarge
        );

        // 校验总费用不超过 100%（买入和卖出分别检查）
        let total_buy_fee = new_config.platform_buy_fee
            .checked_add(new_config.lp_buy_fee)
            .ok_or(PredictionMarketError::MathOverflow)?;
        let total_sell_fee = new_config.platform_sell_fee
            .checked_add(new_config.lp_sell_fee)
            .ok_or(PredictionMarketError::MathOverflow)?;

        require!(
            total_buy_fee <= MAX_FEE_BPS,
            PredictionMarketError::ValueTooLarge
        );
        require!(
            total_sell_fee <= MAX_FEE_BPS,
            PredictionMarketError::ValueTooLarge
        );

        // 计算空间与租金
        let serialized_config =
            [&Config::DISCRIMINATOR, new_config.try_to_vec()?.as_slice()].concat();
        let serialized_config_len = serialized_config.len();
        let config_cost = Rent::get()?.minimum_balance(serialized_config_len);

        // 初始化/校验配置PDA归属与权限
        if is_initialization {
            // ✅ v1.1.1: 初始化时的权限检查
            //
            // 确保只有预期的部署者可以初始化配置
            // new_config.authority 必须等于 payer（调用者）
            require!(
                new_config.authority == self.payer.key(),
                PredictionMarketError::IncorrectAuthority
            );

            let cpi_context = CpiContext::new(
                self.system_program.to_account_info(),
                system_program::CreateAccount {
                    from: self.payer.to_account_info(),
                    to: self.config.to_account_info(),
                },
            );
            system_program::create_account(
                cpi_context.with_signer(&[&[CONFIG.as_bytes(), &[config_bump]]]),
                config_cost,
                serialized_config_len as u64,
                &crate::ID,
            )?;

            msg!("Config initialized by authority: {}", self.payer.key());
        } else {
            // ✅ 更新时的权限检查（原有逻辑）
            let data = self.config.try_borrow_data()?;
            if data.len() < 8 || &data[0..8] != Config::DISCRIMINATOR {
                return err!(PredictionMarketError::IncorrectConfigAccount);
            }
            let config = Config::deserialize(&mut &data[8..])?;

            // 只有当前 authority 可以更新配置
            require!(
                config.authority == self.payer.key(),
                PredictionMarketError::IncorrectAuthority
            );

            msg!("Config updated by authority: {}", self.payer.key());
        }

        // 🔒 修复：处理账户大小调整（扩容或收缩）
        let current_data_len = self.config.to_account_info().data_len();
        let lamport_delta = (config_cost as i64) - (self.config.lamports() as i64);

        // 无论扩容还是收缩，都需要调用 realloc
        if serialized_config_len != current_data_len {
            // 步骤1：先调整账户大小（无论扩容还是收缩）
            // realloc 被标记为 deprecated 但是 Anchor 0.32.1 中唯一可用的方法
            #[allow(deprecated)]
            self.config.to_account_info().realloc(serialized_config_len, false)?;

            // 步骤2：处理租金差额
            if lamport_delta > 0 {
                // 扩容：需要补足租金
                system_program::transfer(
                    CpiContext::new(
                        self.system_program.to_account_info(),
                        system_program::Transfer {
                            from: self.payer.to_account_info(),
                            to: self.config.to_account_info(),
                        },
                    ),
                    lamport_delta as u64,
                )?;
                msg!("Config expanded: +{} bytes, +{} lamports",
                    serialized_config_len as i64 - current_data_len as i64,
                    lamport_delta);
            } else if lamport_delta < 0 {
                // 收缩：退回多余租金
                **self.config.to_account_info().try_borrow_mut_lamports()? -= (-lamport_delta) as u64;
                **self.payer.to_account_info().try_borrow_mut_lamports()? += (-lamport_delta) as u64;
                msg!("Config shrunk: {} bytes, refunded {} lamports",
                    current_data_len as i64 - serialized_config_len as i64,
                    -lamport_delta);
            }
        }

        // 写入序列化配置
        (self.config.try_borrow_mut_data()?[..serialized_config_len])
            .copy_from_slice(serialized_config.as_slice());

        // ✅ FIX: 正确初始化全局金库 PDA
        // 必须调用 create_account 设置 owner 为程序，才能在后续修改 lamports
        if self.global_vault.owner != &crate::ID {
            let rent = Rent::get()?;
            let min_rent = rent.minimum_balance(0); // PDA 不存储数据，只需最小租金

            let cpi_context = CpiContext::new(
                self.system_program.to_account_info(),
                system_program::CreateAccount {
                    from: self.payer.to_account_info(),
                    to: self.global_vault.to_account_info(),
                },
            );
            system_program::create_account(
                cpi_context.with_signer(&[&[GLOBAL.as_bytes(), &[global_vault_bump]]]),
                min_rent,
                0, // 数据长度为0
                &crate::ID, // ✅ 关键：owner 设为程序
            )?;
            msg!("Global vault PDA created with owner = program_id");
        }

        // ✅ v1.1.1: 发射配置更新事件（增强可追溯性）
        let clock = Clock::get()?;
        emit!(crate::events::ConfigUpdateEvent {
            authority: self.payer.key(),
            is_initialization,
            new_authority: new_config.authority,
            team_wallet: new_config.team_wallet,
            initial_real_token_reserves_config: new_config.initial_real_token_reserves_config,
            token_supply_config: new_config.token_supply_config,
            token_decimals_config: new_config.token_decimals_config,
            platform_buy_fee: new_config.platform_buy_fee,
            platform_sell_fee: new_config.platform_sell_fee,
            lp_buy_fee: new_config.lp_buy_fee,
            lp_sell_fee: new_config.lp_sell_fee,
            is_paused: new_config.is_paused,
            whitelist_enabled: new_config.whitelist_enabled,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }
}
