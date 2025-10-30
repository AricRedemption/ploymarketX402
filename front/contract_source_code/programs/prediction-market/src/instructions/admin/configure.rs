//! 管理员配置指令：初始化/更新全局配置与全局金库

use crate::errors::*;
use crate::{
    constants::{CONFIG, GLOBAL},
    state::config::*,
    utils::sol_transfer_from_user,
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

    /// 全局金库（PDA，存放SOL）
    /// CHECK: global vault pda which stores SOL
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
    pub fn handler(&mut self, new_config: Config, config_bump: u8) -> Result<()> {
        // 基础参数校验：总供应与精度、初始储备的一致性
        let decimal_multiplier = 10u64.pow(new_config.token_decimals_config as u32);
        let fractional_tokens = new_config.token_supply_config % decimal_multiplier;
        if fractional_tokens != 0 {
            return Err(ValueInvalid.into());
        }

        require!(
            new_config.token_supply_config
                >= (new_config.initial_real_token_reserves_config / decimal_multiplier
                    * new_config.initial_real_token_reserves_config),
            PredictionMarketError::InvalidAmount
        );

        require!(
            new_config.token_supply_config
                >= (new_config.initial_real_token_reserves_config / decimal_multiplier
                    * new_config.initial_real_token_reserves_config),
            PredictionMarketError::InvalidAmount
        );

        // 计算空间与租金
        let serialized_config =
            [&Config::DISCRIMINATOR, new_config.try_to_vec()?.as_slice()].concat();
        let serialized_config_len = serialized_config.len();
        let config_cost = Rent::get()?.minimum_balance(serialized_config_len);

        // 初始化/校验配置PDA归属与权限
        if self.config.owner != &crate::ID {
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
        } else {
            let data = self.config.try_borrow_data()?;
            if data.len() < 8 || &data[0..8] != Config::DISCRIMINATOR {
                return err!(PredictionMarketError::IncorrectConfigAccount);
            }
            let config = Config::deserialize(&mut &data[8..])?;

            if config.authority != self.payer.key() {
                return err!(PredictionMarketError::IncorrectAuthority);
            }
        }

        // 如需补足租金并调整AccountInfo大小
        let lamport_delta = (config_cost as i64) - (self.config.lamports() as i64);
        if lamport_delta > 0 {
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
            self.config.resize(serialized_config_len)?;
        }

        // 写入序列化配置
        (self.config.try_borrow_mut_data()?[..serialized_config_len])
            .copy_from_slice(serialized_config.as_slice());

        // 初始化全局金库（如未初始化则转入少量租金）
        if self.global_vault.lamports() == 0 {
            sol_transfer_from_user(
                &self.payer,
                self.global_vault.clone(),
                &self.system_program,
                1000000,
            )?;
        }
        Ok(())
    }
}
