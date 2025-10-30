//! # 工具函数模块
//! 
//! 提供预测市场合约中使用的各种工具函数
//! 包括数值转换、代币转账、SOL转账等功能

use crate::*;
use anchor_spl::token::{self, Token};
use anchor_lang::solana_program::program::{invoke, invoke_signed};
use std::ops::{Div, Mul};

/// 将代币数量转换为浮点数
/// 
/// 根据代币的精度将原始数量转换为可读的浮点数
/// 
/// # 参数
/// * `value` - 原始代币数量
/// * `decimals` - 代币精度
/// 
/// # 返回
/// * `f64` - 转换后的浮点数
/// 
/// # 示例
/// ```rust
/// let tokens = convert_to_float(1000000000, 9); // 1.0
/// ```
pub fn convert_to_float(value: u64, decimals: u8) -> f64 {
    (value as f64).div(f64::powf(10.0, decimals as f64))
}

/// 将浮点数转换为代币数量
/// 
/// 根据代币的精度将浮点数转换为原始代币数量
/// 
/// # 参数
/// * `value` - 浮点数值
/// * `decimals` - 代币精度
/// 
/// # 返回
/// * `u64` - 转换后的代币数量
/// 
/// # 示例
/// ```rust
/// let amount = convert_from_float(1.5, 9); // 1500000000
/// ```
pub fn convert_from_float(value: f64, decimals: u8) -> u64 {
    value.mul(f64::powf(10.0, decimals as f64)) as u64
}

/// 从用户账户转账SOL
/// 
/// 使用系统程序从签名者账户向目标账户转账SOL
/// 
/// # 参数
/// * `signer` - 签名者账户（资金源）
/// * `destination` - 目标账户
/// * `system_program` - 系统程序
/// * `amount` - 转账金额（lamports）
/// 
/// # 返回
/// * `Result<()>` - 操作结果
pub fn sol_transfer_from_user<'info>(
    signer: &Signer<'info>,
    destination: AccountInfo<'info>,
    system_program: &Program<'info, System>,
    amount: u64,
) -> Result<()> {
    // 创建转账指令
    let ix = anchor_lang::solana_program::system_instruction::transfer(signer.key, destination.key, amount);
    
    // 执行转账
    invoke(
        &ix,
        &[
            signer.to_account_info(),
            destination,
            system_program.to_account_info(),
        ],
    )?;
    Ok(())
}

/// 从用户账户转账代币
/// 
/// 使用SPL代币程序从用户账户转账代币
/// 
/// # 参数
/// * `from` - 源代币账户
/// * `authority` - 授权签名者
/// * `to` - 目标代币账户
/// * `token_program` - SPL代币程序
/// * `amount` - 转账数量
/// 
/// # 返回
/// * `Result<()>` - 操作结果
pub fn token_transfer_user<'info>(
    from: AccountInfo<'info>,
    authority: &Signer<'info>,
    to: AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    // 创建CPI上下文
    let cpi_ctx: CpiContext<_> = CpiContext::new(
        token_program.to_account_info(),
        token::Transfer {
            from,
            authority: authority.to_account_info(),
            to,
        },
    );
    
    // 执行代币转账
    token::transfer(cpi_ctx, amount)?;
    Ok(())
}

/// 从PDA账户转账代币
/// 
/// 使用PDA作为授权者进行代币转账
/// 需要提供PDA的签名种子
/// 
/// # 参数
/// * `from` - 源代币账户
/// * `authority` - PDA授权账户
/// * `to` - 目标代币账户
/// * `token_program` - SPL代币程序
/// * `signer_seeds` - PDA签名种子
/// * `amount` - 转账数量
/// 
/// # 返回
/// * `Result<()>` - 操作结果
pub fn token_transfer_with_signer<'info>(
    from: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    to: AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    signer_seeds: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    // 创建带签名的CPI上下文
    let cpi_ctx: CpiContext<_> = CpiContext::new_with_signer(
        token_program.to_account_info(),
        token::Transfer {
            from,
            to,
            authority,
        },
        signer_seeds,
    );
    
    // 执行代币转账
    token::transfer(cpi_ctx, amount)?;
    Ok(())
}

/// 从PDA账户转账SOL
/// 
/// 使用PDA作为签名者进行SOL转账
/// 需要提供PDA的签名种子
/// 
/// # 参数
/// * `source` - 源账户（PDA）
/// * `destination` - 目标账户
/// * `system_program` - 系统程序
/// * `signers_seeds` - PDA签名种子
/// * `amount` - 转账金额（lamports）
/// 
/// # 返回
/// * `Result<()>` - 操作结果
pub fn sol_transfer_with_signer<'info>(
    source: AccountInfo<'info>,
    destination: AccountInfo<'info>,
    system_program: &Program<'info, System>,
    signers_seeds: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    // 创建转账指令
    let ix = anchor_lang::solana_program::system_instruction::transfer(source.key, destination.key, amount);
    
    // 使用PDA签名执行转账
    invoke_signed(
        &ix,
        &[source, destination, system_program.to_account_info()],
        signers_seeds,
    )?;
    Ok(())
}

/// 从PDA账户销毁代币
/// 
/// 使用PDA作为授权者销毁代币
/// 销毁的代币将从总供应量中永久移除
/// 
/// # 参数
/// * `from` - 要销毁代币的账户
/// * `authority` - PDA授权账户
/// * `token_program` - SPL代币程序
/// * `signer_seeds` - PDA签名种子
/// * `amount` - 销毁数量
/// 
/// # 返回
/// * `Result<()>` - 操作结果
pub fn token_burn_with_signer<'info>(
    from: AccountInfo<'info>, // 要销毁代币的账户
    authority: AccountInfo<'info>, // PDA授权账户
    token_program: &Program<'info, Token>, // SPL代币程序
    signer_seeds: &[&[&[u8]]], // PDA签名种子
    amount: u64, // 销毁数量
) -> Result<()> {
    // 创建带签名的CPI上下文
    let cpi_ctx: CpiContext<_> = CpiContext::new_with_signer(
        token_program.to_account_info(),
        token::Burn {
            mint: from.to_account_info(), // 代币铸造账户
            from, // 要销毁的账户
            authority, // PDA授权账户
        },
        signer_seeds,
    );

    // 执行代币销毁
    token::burn(cpi_ctx, amount)?;
    Ok(())
}

/// 计算基点（BPS）乘法
/// 
/// 用于计算手续费等基于基点的计算
/// 防止溢出并提供安全的数值计算
/// 
/// # 参数
/// * `bps` - 基点值（如1000表示10%）
/// * `value` - 基础值
/// * `divisor` - 除数（通常为10000）
/// 
/// # 返回
/// * `Option<u64>` - 计算结果，如果溢出则返回None
pub fn bps_mul(bps: u64, value: u64, divisor: u64) -> Option<u64> {
    bps_mul_raw(bps, value, divisor).unwrap().try_into().ok()
}

/// 基点乘法的原始实现
/// 
/// 使用u128进行中间计算以防止溢出
/// 
/// # 参数
/// * `bps` - 基点值
/// * `value` - 基础值
/// * `divisor` - 除数
/// 
/// # 返回
/// * `Option<u128>` - 计算结果
pub fn bps_mul_raw(bps: u64, value: u64, divisor: u64) -> Option<u128> {
    (value as u128)
        .checked_mul(bps as u128)?
        .checked_div(divisor as u128)
}
