//! # 错误定义模块
//! 
//! 定义预测市场合约中可能出现的各种错误类型
//! 使用Anchor的错误处理机制提供详细的错误信息

use anchor_lang::prelude::*;

// 导出所有错误类型以便使用
pub use PredictionMarketError::*;

/// 预测市场错误枚举
/// 
/// 定义了合约中可能出现的所有错误类型
/// 每个错误都有对应的错误码和描述信息
#[error_code]
pub enum PredictionMarketError {
    /// 数值过小错误
    /// 当输入值小于允许的最小值时触发
    #[msg("ValueTooSmall")]
    ValueTooSmall,

    /// 数值过大错误
    /// 当输入值大于允许的最大值时触发
    #[msg("ValueTooLarge")]
    ValueTooLarge,

    /// 数值无效错误
    /// 当输入值不在允许的范围内时触发
    #[msg("ValueInvalid")]
    ValueInvalid,

    /// 配置账户错误
    /// 当提供的配置账户不正确时触发
    #[msg("IncorrectConfigAccount")]
    IncorrectConfigAccount,

    /// 权限错误
    /// 当调用者没有足够权限时触发
    #[msg("IncorrectAuthority")]
    IncorrectAuthority,

    /// 溢出或下溢错误
    /// 当数值计算发生溢出或下溢时触发
    #[msg("Overflow or underflow occured")]
    OverflowOrUnderflowOccurred,

    /// 无效金额错误
    /// 当提供的金额无效时触发
    #[msg("Amount is invalid")]
    InvalidAmount,

    /// 团队钱包地址错误
    /// 当团队钱包地址不正确时触发
    #[msg("Incorrect team wallet address")]
    IncorrectTeamWallet,

    /// 曲线未完成错误
    /// 当尝试在曲线未完成时执行某些操作时触发
    #[msg("Curve is not completed")]
    CurveNotCompleted,

    /// 曲线已完成错误
    /// 当尝试在曲线完成后执行交易时触发
    #[msg("Can not swap after the curve is completed")]
    CurveAlreadyCompleted,

    /// 铸造权限未撤销错误
    /// 当代币的铸造权限未被撤销时触发
    #[msg("Mint authority should be revoked")]
    MintAuthorityEnabled,

    /// 冻结权限未撤销错误
    /// 当代币的冻结权限未被撤销时触发
    #[msg("Freeze authority should be revoked")]
    FreezeAuthorityEnabled,

    /// 返回金额过小错误
    /// 当实际返回金额小于最小接收金额时触发（滑点保护）
    #[msg("Return amount is too small compared to the minimum received amount")]
    ReturnAmountTooSmall,

    /// AMM已存在错误
    /// 当尝试创建已存在的AMM时触发
    #[msg("AMM is already exist")]
    AmmAlreadyExists,

    /// 未初始化错误
    /// 当全局配置未初始化时触发
    #[msg("Global Not Initialized")]
    NotInitialized,

    /// 无效的全局权限错误
    /// 当全局权限验证失败时触发
    #[msg("Invalid Global Authority")]
    InvalidGlobalAuthority,

    /// 白名单错误
    /// 当创建者不在白名单中时触发
    #[msg("This creator is not in whitelist")]
    NotWhiteList,

    /// 启动阶段错误
    /// 当操作不在正确的启动阶段时触发
    #[msg("IncorrectLaunchPhase")]
    IncorrectLaunchPhase,

    /// 代币余额不足错误
    /// 当没有足够的代币完成卖出订单时触发
    #[msg("Not enough tokens to complete the sell order.")]
    InsufficientTokens,

    /// SOL余额不足错误
    /// 当没有足够的SOL完成操作时触发
    #[msg("Not enough SOL received to be valid.")]
    InsufficientSol,

    /// 卖出失败错误
    /// 当卖出操作失败时触发
    #[msg("Sell Failed")]
    SellFailed,

    /// 买入失败错误
    /// 当买入操作失败时触发
    #[msg("Buy Failed")]
    BuyFailed,

    /// 非绑定曲线代币错误
    /// 当代币不是绑定曲线代币时触发
    #[msg("This token is not a bonding curve token")]
    NotBondingCurveMint,

    /// 非SOL代币错误
    /// 当代币不是SOL时触发
    #[msg("Not quote mint")]
    NotSOL,

    /// 无效的迁移权限错误
    /// 当迁移权限验证失败时触发
    #[msg("Invalid Migration Authority")]
    InvalidMigrationAuthority,

    /// 绑定曲线未完成错误
    /// 当绑定曲线未完成时触发
    #[msg("Bonding curve is not completed")]
    NotCompleted,

    /// 无效的Meteora程序错误
    /// 当Meteora程序验证失败时触发
    #[msg("Invalid Meteora Program")]
    InvalidMeteoraProgram,

    /// 算术错误
    /// 当算术运算出现错误时触发
    #[msg("Arithmetic Error")]
    ArithmeticError,

    /// 无效参数错误
    /// 当提供的参数无效时触发
    #[msg("Invalid Parameter")]
    InvalidParameter,

    /// 无效开始时间错误
    /// 当开始时间在过去时触发
    #[msg("Start time is in the past")]
    InvalidStartTime,

    /// 无效结束时间错误
    /// 当结束时间在过去时触发
    #[msg("End time is in the past")]
    InvalidEndTime,

    /// 已初始化错误
    /// 当尝试重复初始化时触发
    #[msg("Global Already Initialized")]
    AlreadyInitialized,

    /// 无效权限错误
    /// 当权限验证失败时触发
    #[msg("Invalid Authority")]
    InvalidAuthority,

    /// 无效参数错误
    /// 当提供的参数无效时触发
    #[msg("Invalid Argument")]
    InvalidArgument,

    /// 市场未完成错误
    /// 当市场尚未结束时触发
    #[msg("The market has already ended.")]
    MarketNotCompleted,

    /// 市场已完成错误
    /// 当市场已经结束时触发
    #[msg("The market already ended.")]
    MarketIsCompleted,

    /// 获胜代币类型错误
    /// 当获胜代币类型设置错误时触发
    #[msg("The winner token type error.")]
    RESOLUTIONTOKEYTYPEERROR,

    /// 获胜YES代币数量错误
    /// 当YES代币奖励数量设置错误时触发
    #[msg("The winner yes token amount error.")]
    RESOLUTIONYESAMOUNTERROR,

    /// 获胜NO代币数量错误
    /// 当NO代币奖励数量设置错误时触发
    #[msg("The winner no token amount error.")]
    RESOLUTIONNOAMOUNTERROR,

    /// 提取SOL数量错误
    /// 当提取的SOL数量无效时触发
    #[msg("The withdraw sol amount error.")]
    WITHDRAWLIQUIDITYSOLAMOUNTERROR,

    /// 非LP提取错误
    /// 当非流动性提供者尝试提取流动性时触发
    #[msg("The withdraw: not lp error.")]
    WITHDRAWNOTLPERROR,
}
