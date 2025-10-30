//! # 常量定义模块
//! 
//! 定义预测市场合约中使用的各种常量
//! 包括PDA种子、代币名称、时间限制等

/// 全局配置PDA种子
pub const CONFIG: &str = "config";

/// 全局金库PDA种子
pub const GLOBAL: &str = "global";

/// 市场PDA种子
pub const MARKET: &str = "market";

/// 用户信息PDA种子
pub const USERINFO: &str = "userinfo";

/// LP仓位PDA种子（记录用户在市场中的LP份额）
pub const LPPOSITION: &str = "lp_position";

/// 白名单PDA种子（✅ v1.0.16 新增）
pub const WHITELIST: &str = "prediction_market_creator_whitelist";

/// 代币元数据PDA种子
pub const METADATA: &str = "metadata";

/// YES代币名称（表示"同意"）
pub const YES_NAME: &str = "agree";

/// NO代币名称（表示"不同意"）
pub const NO_NAME: &str = "disagree";

/// 最大开始时间延迟（约1周，以槽位计算）
/// 每个槽位约400毫秒
pub const MAX_START_SLOT_DELAY: u64 = 1_512_000; // ~1 week in slots (400ms each)

/// ✅ LMSR b参数最大值
/// ✅ v1.1.0: 更新为 USDC 单位（6 位精度）
/// 1M USDC = 1,000,000 USDC * 10^6
pub const MAX_LMSR_B: u64 = 1_000_000_000_000; // 1M USDC in smallest units (6 decimals)

/// ✅ LPs最大数量（防止Vec无限增长）
pub const MAX_LPS: usize = 100;

/// ✅ LMSR q参数最大绝对值（防止精度损失和溢出）
/// ✅ v1.1.0: 更新为 USDC 单位（6 位精度）
/// 1B USDC = 1,000,000,000 USDC * 10^6（远超实际使用场景）
pub const MAX_Q_VALUE: i64 = 1_000_000_000_000_000; // 1B USDC in smallest units (6 decimals)

/// ✅ 最小流动性要求（防止流动性枯竭和除零错误）
/// 设置为 1000 USDC（1000 * 10^6）
/// 参考 Uniswap V2 的 MINIMUM_LIQUIDITY = 1000
/// ✅ v1.1.0: 更新为 6 位精度（匹配 USDC）
pub const MIN_LIQUIDITY: u64 = 1_000_000_000; // 1000 USDC in smallest units (6 decimals: 1000 * 10^6)

