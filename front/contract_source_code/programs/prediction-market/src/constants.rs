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

/// 代币元数据PDA种子
pub const METADATA: &str = "metadata";

/// YES代币名称（表示"同意"）
pub const YES_NAME: &str = "agree";

/// NO代币名称（表示"不同意"）
pub const NO_NAME: &str = "disagree";

/// 最大开始时间延迟（约1周，以槽位计算）
/// 每个槽位约400毫秒
pub const MAX_START_SLOT_DELAY: u64 = 1_512_000; // ~1 week in slots (400ms each)

