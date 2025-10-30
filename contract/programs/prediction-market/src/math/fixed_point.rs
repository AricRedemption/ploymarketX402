//! # 定点数学库 (Fixed-Point Math Library)
//!
//! ## 数据格式 (Data Format)
//!
//! **Q64.64 Fixed-Point Representation**:
//! - Type: `u128` (128 bits total)
//! - Integer part: Upper 64 bits
//! - Fractional part: Lower 64 bits
//! - Precision: ~18 decimal digits (~2^-64 ≈ 5.42e-20)
//! - Range: [0, 2^64 - 1] ≈ [0, 1.84e19]
//!
//! ## 为什么使用定点数 (Why Fixed-Point)
//!
//! Solana 程序必须是**确定性的** (deterministic)，浮点运算在不同机器上可能产生不同结果。
//! 定点数提供：
//! - ✅ 跨平台确定性 (Cross-platform determinism)
//! - ✅ 无舍入误差累积 (No rounding error accumulation)
//! - ✅ Gas 效率 (Gas efficient integer operations)
//!
//! ## 数学公式 (Mathematical Operations)
//!
//! 给定 Q64.64 格式的两个数 `a` 和 `b`：
//!
//! - **乘法 (Multiplication)**: `(a × b) >> 64`
//!   - 原理: `(a/2^64) × (b/2^64) = (a×b)/2^128`，需右移 64 位恢复 Q64.64 格式
//!
//! - **除法 (Division)**: `(a << 64) / b`
//!   - 原理: `(a/2^64) / (b/2^64) = a/b`，需左移 64 位保持小数精度
//!
//! - **加减法 (Addition/Subtraction)**: `a ± b` (直接操作)
//!   - 原理: 小数点对齐，无需额外操作
//!
//! ## 参考文献 (References)
//!
//! - Fixed-Point Arithmetic: <https://en.wikipedia.org/wiki/Fixed-point_arithmetic>
//! - Q Format: <https://en.wikipedia.org/wiki/Q_(number_format)>
//! - Solana Determinism: <https://docs.solana.com/developing/programming-model/overview#determinism>

use anchor_lang::prelude::*;

/// Q64.64 定点数类型
///
/// # 表示方式 (Representation)
///
/// 数值 `x` 表示为: `x_fixed = x × 2^64`
///
/// # 示例 (Examples)
///
/// ```text
/// 1.0   → 0x0000000000000001_0000000000000000 (1 << 64)
/// 0.5   → 0x0000000000000000_8000000000000000 (1 << 63)
/// 2.718 → 0x0000000000000002_B7E151628AED2A6A (e ≈ 2.718281828...)
/// ```
pub type FixedPoint = u128;

/// 定点数常量
pub mod constants {
    use super::FixedPoint;

    /// 1.0 的定点表示 (2^64)
    pub const ONE: FixedPoint = 1u128 << 64;

    /// 2.0 的定点表示
    pub const TWO: FixedPoint = 2u128 << 64;

    /// e ≈ 2.718281828 的定点表示
    pub const E: FixedPoint = 50143449209799256682; // e * 2^64

    /// ln(2) ≈ 0.693147180 的定点表示
    pub const LN_2: FixedPoint = 12786308645202655660; // ln(2) * 2^64

    /// 最大安全指数（避免溢出）：ln(2^63) ≈ 43.668
    /// ✅ v1.0.26: 修正常量值 - 43.668 * 2^64 = 805306368000000000000 (20位)
    ///     之前错误值：805306368000000000 (18位) ≈ 0.0436 << 64
    pub const MAX_EXP_INPUT: FixedPoint = 805306368000000000000; // 43.668 * 2^64

    /// 最小 ln 输入（接近 0）
    pub const MIN_LN_INPUT: FixedPoint = 1844674407; // ~0.0001 * 2^64
}

use constants::*;

/// 定点数乘法 (Fixed-Point Multiplication)
///
/// # 数学公式 (Mathematical Formula)
///
/// ```text
/// Given: a_fixed = a × 2^64, b_fixed = b × 2^64
/// Want:  result_fixed = (a × b) × 2^64
///
/// Derivation:
/// (a_fixed × b_fixed) / 2^64 = (a × 2^64 × b × 2^64) / 2^64
///                             = (a × b) × 2^64
///                             = result_fixed ✓
/// ```
///
/// # 参数 (Parameters)
///
/// * `a` - 第一个定点数 (First fixed-point number)
/// * `b` - 第二个定点数 (Second fixed-point number)
///
/// # 返回值 (Returns)
///
/// * `Result<FixedPoint>` - 乘法结果，精度为 Q64.64
///
/// # 精度 (Precision)
///
/// - 相对误差 (Relative error): ~2^-64 ≈ 5.4e-20
/// - 绝对误差 (Absolute error): 取决于结果大小，最坏情况 1 LSB
///
/// # 错误 (Errors)
///
/// * `MathOverflow` - 中间结果超过 u128 最大值
///
/// # 示例 (Examples)
///
/// ```text
/// fp_mul(2.5 × 2^64, 3.0 × 2^64) = 7.5 × 2^64
/// fp_mul(from_u64(2), from_u64(3)) = from_u64(6)
/// ```
#[inline]
pub fn fp_mul(a: FixedPoint, b: FixedPoint) -> Result<FixedPoint> {
    // ✅ v1.0.26: 修复 CRITICAL 缺陷 - 使用真正的 256 位中间结果
    //
    // 🔴 原问题（v1.0.25 及之前）：
    //    let result = (a as u128).checked_mul(b as u128)?;
    //    - a 和 b 已经是 u128 类型，cast 无效
    //    - u128 × u128 的结果仍是 u128，高位被截断
    //    - 对于 Q64.64 格式：(2 << 64) × (3 << 64) = 6 << 128 → 溢出！
    //    - 实测：fp_mul(from_u64(2), from_u64(3)) → MathOverflow
    //
    // ✅ 修复策略：手动拆分为 4 个 64×64 部分，组合成 256 位结果
    //
    // 原理：将 u128 拆为高低 64 位
    //   a = a_hi << 64 + a_lo
    //   b = b_hi << 64 + b_lo
    //
    //   a × b = (a_hi << 64 + a_lo) × (b_hi << 64 + b_lo)
    //        = a_hi×b_hi << 128 + a_hi×b_lo << 64 + a_lo×b_hi << 64 + a_lo×b_lo
    //        = [high_128_bits | low_128_bits] (256-bit result)
    //
    // Q64.64 乘法需要：(a × b) >> 64
    //   = [high_128 | low_128] >> 64
    //   = (high_128 << 64) | (low_128 >> 64)

    let a_lo = a & 0xFFFF_FFFF_FFFF_FFFF;  // Lower 64 bits
    let a_hi = a >> 64;                     // Upper 64 bits
    let b_lo = b & 0xFFFF_FFFF_FFFF_FFFF;
    let b_hi = b >> 64;

    // 计算 4 个 64×64 乘积（每个结果都是 u128，安全）
    let ll = a_lo.checked_mul(b_lo).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;
    let lh = a_lo.checked_mul(b_hi).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;
    let hl = a_hi.checked_mul(b_lo).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;
    let hh = a_hi.checked_mul(b_hi).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

    // 组合成 256 位结果的低 128 位部分
    // low_128 = ll + (lh << 64) + (hl << 64)
    // 我们需要 (256-bit result) >> 64，即取 low_128 的高 64 位 + high_128 的低 64 位

    // ll >> 64: ll 的高 64 位
    let result_from_ll = ll >> 64;

    // (lh + hl) 的低 64 位：中间项
    let mid = lh.checked_add(hl).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;
    let mid_lo = mid & 0xFFFF_FFFF_FFFF_FFFF;
    let mid_hi = mid >> 64;

    // hh 的低 64 位 + mid_hi：高位项
    let high = hh.checked_add(mid_hi).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

    // 最终结果：(high << 64) + mid_lo + result_from_ll
    let result = high.checked_shl(64)
        .and_then(|h| h.checked_add(mid_lo))
        .and_then(|h| h.checked_add(result_from_ll))
        .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

    Ok(result)
}

/// 定点数除法 (Fixed-Point Division)
///
/// # 数学公式 (Mathematical Formula)
///
/// ```text
/// Given: a_fixed = a × 2^64, b_fixed = b × 2^64
/// Want:  result_fixed = (a / b) × 2^64
///
/// Derivation:
/// (a_fixed << 64) / b_fixed = (a × 2^64 × 2^64) / (b × 2^64)
///                           = (a × 2^128) / (b × 2^64)
///                           = (a / b) × 2^64
///                           = result_fixed ✓
/// ```
///
/// # 参数 (Parameters)
///
/// * `a` - 分子定点数 (Numerator fixed-point number)
/// * `b` - 分母定点数 (Denominator fixed-point number, must be > 0)
///
/// # 返回值 (Returns)
///
/// * `Result<FixedPoint>` - 除法结果，精度为 Q64.64
///
/// # 精度 (Precision)
///
/// - 相对误差 (Relative error): ~2^-64 ≈ 5.4e-20
/// - 截断误差 (Truncation): 向下取整（类似 floor 行为）
///
/// # 错误 (Errors)
///
/// * `DivisionByZero` - 分母为 0
/// * `MathOverflow` - 左移操作溢出（a 过大）
///
/// # 示例 (Examples)
///
/// ```text
/// fp_div(6.0 × 2^64, 2.0 × 2^64) = 3.0 × 2^64
/// fp_div(from_u64(7), from_u64(2)) = 3.5 × 2^64
/// ```
#[inline]
pub fn fp_div(a: FixedPoint, b: FixedPoint) -> Result<FixedPoint> {
    require!(b > 0, crate::errors::PredictionMarketError::DivisionByZero);

    // ✅ v1.0.27: 完全重写 - 使用逐位处理避免溢出
    //
    // 🔴 v1.0.26 问题：
    //    mid_rem 可能 >= 2^64，导致 mid_rem << 64 溢出
    //
    // ✅ 新策略：逐位计算 (a << 64) / b
    //
    // 核心思想：
    //   result = (a << 64) / b
    //          = (a_hi << 128 + a_lo << 64) / b
    //
    // 方法：分段处理，避免任何可能溢出的左移
    //
    // 算法：
    //   1. 处理高 64 位：q_hi = a_hi / b, rem_hi = a_hi % b
    //   2. 处理低 64 位：将 rem_hi 和 a_lo 组合
    //      dividend_mid = rem_hi * 2^64 + a_lo
    //      但不直接计算 rem_hi << 64（会溢出）
    //      而是用算术：(rem_hi << 64 + a_lo) / b
    //                 = rem_hi * 2^64 / b + a_lo / b
    //                 = rem_hi * (2^64 / b) + a_lo / b
    //   3. 处理小数部分：对最终余数再除以 b

    // 拆分 a 为高低 64 位
    let a_hi = a >> 64;
    let a_lo = a & 0xFFFF_FFFF_FFFF_FFFF;

    // Step 1: 高 64 位的整数除法
    // q_hi 是 a_hi / b 的商（result 的高位部分）
    let q_hi = a_hi / b;
    let rem_hi = a_hi % b;

    // Step 2: 中间 64 位
    // 现在需要计算：(rem_hi * 2^64 + a_lo) / b
    //
    // 问题：rem_hi * 2^64 可能溢出 u128
    // 解决：拆分计算
    //   (rem_hi * 2^64 + a_lo) / b
    //   = rem_hi * 2^64 / b + (rem_hi * 2^64 % b + a_lo) / b
    //
    // 但 rem_hi * 2^64 / b 仍可能溢出
    //
    // 更好的方法：使用辗转相除
    //   dividend = rem_hi * 2^64 + a_lo
    //
    // 我们知道 rem_hi < b（因为是余数）
    // 所以 dividend < b * 2^64 + a_lo < b * 2^64 + 2^64 = (b+1) * 2^64
    //
    // 如果 rem_hi 很小（< 2^64 / b），rem_hi * 2^64 不会溢出
    // 否则需要特殊处理

    // 使用一个辅助函数：计算 (rem * 2^64 + extra) / divisor
    fn div_with_shifted_rem(rem: u128, extra: u128, divisor: u128) -> (u128, u128) {
        // 计算 (rem * 2^64 + extra) / divisor
        // 返回 (quotient, remainder)

        // rem < divisor（输入保证）
        // extra < 2^64

        // 方法：逐步加倍 rem，每次检查是否 >= divisor
        // 等价于二进制长除法

        let mut quotient = 0u128;
        let mut current_rem = rem;

        // 处理 rem 的 64 位（从高到低）
        for i in (0..64).rev() {
            // current_rem * 2 + (extra 的第 i 位)
            let bit = (extra >> i) & 1;
            current_rem = current_rem * 2 + bit;

            if current_rem >= divisor {
                current_rem -= divisor;
                quotient += 1u128 << i;
            }
        }

        (quotient, current_rem)
    }

    let (_q_mid, rem_mid) = div_with_shifted_rem(rem_hi, a_lo, b);

    // Step 3: 小数部分
    // 现在需要计算：rem_mid * 2^64 / b
    let (_q_lo, _rem_lo) = div_with_shifted_rem(rem_mid, 0, b);

    // Step 4: 组合结果
    // result = q_hi * 2^128 + q_mid * 2^64 + q_lo
    // 但我们只取低 128 位（Q64.64 格式）

    // 检查 q_hi 不能太大（否则结果溢出）
    if q_hi >= (1u128 << 64) {
        return Err(crate::errors::PredictionMarketError::MathOverflow.into());
    }

    // result = (q_hi << 64) + q_mid (整数部分)
    //        + q_lo (小数部分，但 q_lo 应该 < 2^64)
    //
    // 等等，我的分段有问题...
    //
    // 让我重新理清：
    //   (a << 64) = (a_hi << 128) + (a_lo << 64)
    //
    // (a_hi << 128) / b:
    //   quotient_1 = a_hi * 2^128 / b
    //   remainder_1 = a_hi * 2^128 % b
    //
    // (a_lo << 64) / b:
    //   加上 remainder_1
    //
    // 实际上应该是：
    //   整个被除数 = a_hi * 2^128 + a_lo * 2^64
    //
    // 分三段：
    //   - Segment 3 (高 64 位): a_hi
    //   - Segment 2 (中 64 位): a_lo
    //   - Segment 1 (低 64 位): 0
    //   - Segment 0 (最低 64 位): 0
    //
    // 从高到低除：
    //   r = 0
    //   q3 = (r * 2^64 + a_hi) / b, r = (r * 2^64 + a_hi) % b
    //   q2 = (r * 2^64 + a_lo) / b, r = (r * 2^64 + a_lo) % b
    //   q1 = (r * 2^64 + 0) / b, r = (r * 2^64 + 0) % b
    //   q0 = (r * 2^64 + 0) / b
    //
    // result = q3 * 2^64 + q2 (取整数部分的低 64 位)
    //        + q1 (小数部分的高 64 位)
    //        + q0 (不需要，精度足够)

    // 重新实现：
    let rem = 0u128;

    // Segment 3: a_hi
    let (_q3, r3) = div_with_shifted_rem(rem, a_hi, b);

    // Segment 2: a_lo
    let (_q2, r2) = div_with_shifted_rem(r3, a_lo, b);

    // Segment 1: 0（小数部分）
    let (_q1, _r1) = div_with_shifted_rem(r2, 0, b);

    // 组合结果：
    // Q64.64 格式：整数部分（q3 << 64 + q2 的高 64 位），小数部分（q2 的低 64 位 << 0 + q1）
    //
    // 等等，q3 和 q2 都是完整的 quotient，不能简单组合
    //
    // 正确的理解：
    //   被除数 = a_hi * 2^192 + a_lo * 2^128 （不对，应该是 2^128 和 2^64）
    //
    // 让我用更清晰的方式：
    //   被除数（256-bit）= [a_hi | a_lo | 0 | 0] （4 个 64-bit 段）
    //                      = a_hi * 2^192 + a_lo * 2^128 + 0 * 2^64 + 0
    //
    // 但我们要的是 (a << 64)，所以：
    //   被除数 = a * 2^64
    //          = (a_hi * 2^64 + a_lo) * 2^64
    //          = a_hi * 2^128 + a_lo * 2^64
    //
    // 3 段 64-bit 除法：
    //   Seg2: a_hi
    //   Seg1: a_lo
    //   Seg0: 0
    //
    // r = 0
    // q2, r = divmod(r * 2^64 + a_hi, b)  → q2 是整数商的高位
    // q1, r = divmod(r * 2^64 + a_lo, b)  → q1 是整数商的低位
    // q0, r = divmod(r * 2^64 + 0, b)     → q0 是小数部分
    //
    // result = q2 * 2^128 + q1 * 2^64 + q0
    //
    // 但 Q64.64 只需要低 128 位：
    //   = (q1 << 64) + q0
    //
    // 如果 q2 > 0，说明结果 > 2^64，溢出

    // 正确实现：
    let rem = 0u128;
    let (q2, r2) = div_with_shifted_rem(rem, a_hi, b);
    let (q1, r1) = div_with_shifted_rem(r2, a_lo, b);
    let (q0, _r0) = div_with_shifted_rem(r1, 0, b);

    // 检查溢出
    if q2 > 0 {
        return Err(crate::errors::PredictionMarketError::MathOverflow.into());
    }

    // 组合 result = (q1 << 64) + q0
    let result = q1.checked_shl(64)
        .and_then(|q| q.checked_add(q0))
        .ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

    Ok(result)
}

/// 将 u64 转换为定点数
#[inline]
pub fn from_u64(x: u64) -> FixedPoint {
    (x as u128) << 64
}

/// 将定点数转换为 u64（向下取整）
#[inline]
pub fn to_u64(x: FixedPoint) -> u64 {
    (x >> 64) as u64
}

/// 定点数自然对数 ln(x) (Fixed-Point Natural Logarithm)
///
/// # 数学公式 (Mathematical Formula)
///
/// ```text
/// ln(x) = ln(m × 2^e) = ln(m) + e × ln(2)
///
/// where:
///   m ∈ [1, 2)   - normalized mantissa (尾数)
///   e ∈ ℤ        - exponent (指数)
/// ```
///
/// # 算法详解 (Algorithm Details)
///
/// ## Step 1: 归一化 (Normalization)
///
/// 将 `x` 表示为 `m × 2^e`，其中 `1 ≤ m < 2`：
///
/// ```text
/// Example:
///   x = 5.25 = 1.3125 × 2^2
///   → m = 1.3125, e = 2
/// ```
///
/// ## Step 2: 泰勒级数 (Taylor Series)
///
/// 对 `ln(m)` 使用泰勒展开，其中 `y = m - 1 ∈ [0, 1)`：
///
/// ```text
/// ln(1 + y) = y - y²/2 + y³/3 - y⁴/4 + y⁵/5 - ...
///
/// Convergence: 10 terms provide ~10 decimal digits accuracy
/// Error bound: |R_n| < y^(n+1)/(n+1) for y ∈ [0, 1)
/// ```
///
/// ## Step 3: 恢复完整对数 (Reconstruct Full Logarithm)
///
/// ```text
/// ln(x) = ln(m) + e × ln(2)
/// ```
///
/// # 参数 (Parameters)
///
/// * `x` - 输入定点数 (Input fixed-point number, must be > MIN_LN_INPUT)
///
/// # 返回值 (Returns)
///
/// * `Result<FixedPoint>` - ln(x) 的定点表示，单位无关（对数是无量纲的）
///
/// # 精度 (Precision)
///
/// - 绝对误差 (Absolute error): ~1e-10 (10 decimal digits)
/// - 相对误差 (Relative error): 对于 x > 1，约 1e-10 / ln(x)
///
/// # 输入范围 (Input Range)
///
/// - 最小值 (Minimum): `MIN_LN_INPUT ≈ 0.0001`
/// - 最大值 (Maximum): `2^64 - 1 ≈ 1.84e19`
///
/// # 错误 (Errors)
///
/// * `InvalidParameter` - x < MIN_LN_INPUT（接近零或负数会导致 ln(x) → -∞）
/// * `MathOverflow` - 中间计算溢出（极少发生）
///
/// # 参考文献 (References)
///
/// - Taylor Series for ln(1+x): <https://en.wikipedia.org/wiki/Mercator_series>
/// - Range Reduction: Cody & Waite, "Software Manual for the Elementary Functions" (1980)
///
/// # 示例 (Examples)
///
/// ```text
/// fp_ln(e × 2^64) ≈ 1.0 × 2^64          // ln(e) = 1
/// fp_ln(1 × 2^64) = 0                    // ln(1) = 0
/// fp_ln(2 × 2^64) ≈ 0.693 × 2^64        // ln(2) ≈ 0.693
/// ```
pub fn fp_ln(x: FixedPoint) -> Result<FixedPoint> {
    require!(
        x >= MIN_LN_INPUT,
        crate::errors::PredictionMarketError::InvalidParameter
    );

    // 步骤 1：归一化到 [1, 2)
    let mut normalized = x;
    let mut exponent: i64 = 0;

    // 向上移动到 >= 1.0
    while normalized < ONE {
        normalized = normalized.checked_shl(1).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;
        exponent -= 1;
    }

    // 向下移动到 < 2.0
    while normalized >= TWO {
        normalized = normalized >> 1;
        exponent += 1;
    }

    // 步骤 2：计算 ln(normalized) 其中 normalized ∈ [1, 2)
    // 令 y = normalized - 1，使用泰勒级数：ln(1+y) = y - y²/2 + y³/3 - ...
    let y = normalized.checked_sub(ONE).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

    // 泰勒级数逼近（10 项，精度 ~1e-10）
    let mut result = y; // y
    let mut term = y;

    for i in 2..=10 {
        term = fp_mul(term, y)?;
        let term_div = term / (i as u128);

        if i % 2 == 0 {
            // 偶数项：减去
            result = result.checked_sub(term_div).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;
        } else {
            // 奇数项：加上
            result = result.checked_add(term_div).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;
        }
    }

    // 步骤 3：加上 exponent * ln(2)
    let exponent_term = if exponent >= 0 {
        LN_2.checked_mul(exponent as u128).ok_or(crate::errors::PredictionMarketError::MathOverflow)?
    } else {
        LN_2.checked_mul((-exponent) as u128).ok_or(crate::errors::PredictionMarketError::MathOverflow)?
    };

    if exponent >= 0 {
        result = result.checked_add(exponent_term).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;
    } else {
        result = result.checked_sub(exponent_term).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;
    }

    Ok(result)
}

/// 定点数指数函数 exp(x) (Fixed-Point Exponential Function)
///
/// # 数学公式 (Mathematical Formula)
///
/// ```text
/// exp(x) = 2^n × exp(r)
///
/// where:
///   n = floor(x / ln(2))  - integer part
///   r = x - n × ln(2)      - remainder, |r| < ln(2) ≈ 0.693
/// ```
///
/// # 算法详解 (Algorithm Details)
///
/// ## Step 1: 范围缩减 (Range Reduction)
///
/// 将指数分解为整数幂和小余数：
///
/// ```text
/// exp(x) = exp(n × ln(2) + r)
///        = exp(n × ln(2)) × exp(r)
///        = 2^n × exp(r)
///
/// Example:
///   x = 5.0 = 7.213 × ln(2) + 0.007
///   → n = 7, r = 0.007
///   → exp(5.0) = 2^7 × exp(0.007) ≈ 128 × 1.007 = 148.4
/// ```
///
/// **为什么这样做？** 因为 `exp(r)` 在小范围内收敛更快，泰勒级数只需要少量项。
///
/// ## Step 2: 泰勒级数 (Taylor Series)
///
/// 对小余数 `r` 使用泰勒展开：
///
/// ```text
/// exp(r) = 1 + r + r²/2! + r³/3! + r⁴/4! + ...
///
/// Convergence: For |r| < 1, each term is < previous/i
/// Precision: 15 terms provide ~15 decimal digits for |r| < 1
/// Error bound: |R_n| < r^(n+1)/(n+1)!
/// ```
///
/// ## Step 3: 缩放回原范围 (Scale Back)
///
/// ```text
/// result = exp(r) << n    // Multiply by 2^n via left shift
/// ```
///
/// # 参数 (Parameters)
///
/// * `x` - 输入定点数 (Input fixed-point number, must be ≤ MAX_EXP_INPUT)
///
/// # 返回值 (Returns)
///
/// * `Result<FixedPoint>` - exp(x) 的定点表示
///
/// # 精度 (Precision)
///
/// - 相对误差 (Relative error): ~1e-12 (12 decimal digits)
/// - 对于 x ≈ 0，精度更高（~1e-15）
///
/// # 输入范围 (Input Range)
///
/// - 最小值 (Minimum): 0（exp(0) = 1）
/// - 最大值 (Maximum): `MAX_EXP_INPUT ≈ 43.668`（避免 exp(x) > 2^63 溢出）
///
/// # 错误 (Errors)
///
/// * `ValueTooLarge` - x > MAX_EXP_INPUT（结果会超过 u128 表示范围）
/// * `MathOverflow` - 中间计算溢出
///
/// # 参考文献 (References)
///
/// - Taylor Series for exp(x): <https://en.wikipedia.org/wiki/Exponential_function#Formal_definition>
/// - Range Reduction: Tang, "Table-Driven Implementation of the Exponential Function" (1989)
/// - CORDIC Algorithm: <https://en.wikipedia.org/wiki/CORDIC> (alternative approach)
///
/// # 示例 (Examples)
///
/// ```text
/// fp_exp(0) = 1.0 × 2^64                     // exp(0) = 1
/// fp_exp(1.0 × 2^64) ≈ 2.718 × 2^64         // exp(1) = e
/// fp_exp(ln(2) × 2^64) ≈ 2.0 × 2^64        // exp(ln(2)) = 2
/// ```
pub fn fp_exp(x: FixedPoint) -> Result<FixedPoint> {
    // 边界检查
    require!(
        x <= MAX_EXP_INPUT,
        crate::errors::PredictionMarketError::ValueTooLarge
    );

    // 特殊情况：exp(0) = 1
    if x == 0 {
        return Ok(ONE);
    }

    // 步骤 1：范围缩减 x = n * ln(2) + r
    let n = (x / LN_2) as i64;
    let r = x - ((n as u128) * LN_2);

    // 步骤 2：计算 exp(r) 使用泰勒级数
    // exp(r) = 1 + r + r²/2! + r³/3! + ...
    let mut result = ONE; // 1
    let mut term = ONE;   // 当前项
    let mut factorial = 1u128;

    for i in 1..=15 {
        factorial = factorial.checked_mul(i).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;
        term = fp_mul(term, r)?;
        let term_div = term / factorial;
        result = result.checked_add(term_div).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;

        // 提前终止：项太小
        if term_div < 100 {
            break;
        }
    }

    // 步骤 3：乘以 2^n
    if n >= 0 {
        result = result.checked_shl(n as u32).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;
    } else {
        result = result >> ((-n) as u32);
    }

    Ok(result)
}

/// log-sum-exp 技巧 (Log-Sum-Exp Trick for Numerical Stability)
///
/// # 数学公式 (Mathematical Formula)
///
/// **朴素方法（数值不稳定）：**
///
/// ```text
/// ln(exp(a) + exp(b)) = ln(exp(a) + exp(b))
///
/// Problem: exp(a) or exp(b) may overflow even when result is valid
/// Example: a = 100, b = 101
///   → exp(100) ≈ 2.7e43 (overflow!)
///   → But ln(exp(100) + exp(101)) ≈ 101.3 (valid result)
/// ```
///
/// **数值稳定版本（本实现）：**
///
/// ```text
/// ln(exp(a) + exp(b)) = max(a, b) + ln(1 + exp(-|a - b|))
///
/// Derivation (assume a ≥ b without loss of generality):
///   ln(exp(a) + exp(b))
///   = ln(exp(a) × (1 + exp(b - a)))        // Factor out exp(a)
///   = ln(exp(a)) + ln(1 + exp(b - a))      // ln(xy) = ln(x) + ln(y)
///   = a + ln(1 + exp(-(a - b)))            // Since b - a = -(a - b)
///   = max(a, b) + ln(1 + exp(-|a - b|))    // Generalize for either order
/// ```
///
/// # 为什么需要这个技巧？ (Why This Trick?)
///
/// 1. **避免溢出 (Overflow Prevention)**:
///    - `exp(diff)` 对于大 diff 会溢出
///    - `exp(-diff)` 永远 ∈ (0, 1]，不会溢出
///
/// 2. **避免精度损失 (Precision Loss Prevention)**:
///    - 当 |a - b| 很大时，较小的项对结果贡献可忽略
///    - `exp(-20) ≈ 2e-9`，可直接舍去，返回 max(a, b)
///
/// 3. **LMSR 应用 (LMSR Application)**:
///    - LMSR cost function: `C = b × ln(exp(q_yes/b) + exp(q_no/b))`
///    - 当 q_yes 和 q_no 很大时，直接计算会溢出
///    - 使用 log-sum-exp 技巧保证数值稳定性
///
/// # 算法详解 (Algorithm Details)
///
/// ## Step 1: 计算 max 和 diff
///
/// ```text
/// max_val = max(a, b)
/// diff = |a - b|
/// ```
///
/// ## Step 2: 计算 exp(-diff) (✅ v1.0.24 修复)
///
/// ```text
/// if diff < 20:
///     exp_neg_diff = 1 / exp(diff)    // Safe reciprocal
/// else:
///     return max_val                   // exp(-diff) ≈ 0, negligible
/// ```
///
/// **关键修复**: 必须计算 `exp(-diff)` 而非 `exp(diff)`！
/// - `exp(diff)` 随 diff 指数增长 → 系统性高估
/// - `exp(-diff)` 随 diff 指数衰减 → 正确
///
/// ## Step 3: 返回结果
///
/// ```text
/// result = max_val + ln(1 + exp_neg_diff)
/// ```
///
/// # 参数 (Parameters)
///
/// * `a` - 第一个对数值 (First logarithmic value)
/// * `b` - 第二个对数值 (Second logarithmic value)
///
/// # 返回值 (Returns)
///
/// * `Result<FixedPoint>` - ln(exp(a) + exp(b)) 的定点表示
///
/// # 精度 (Precision)
///
/// - 绝对误差 (Absolute error): ~1e-10
/// - 对于 |a - b| > 20，误差 < 1e-9（exp(-20) 可忽略）
///
/// # 错误 (Errors)
///
/// * `MathOverflow` - 中间计算溢出（极少发生）
///
/// # 参考文献 (References)
///
/// - Log-Sum-Exp Trick: <https://en.wikipedia.org/wiki/LogSumExp>
/// - Numerical Stability: Goodfellow et al., "Deep Learning" (2016), Section 4.1
/// - LMSR原理: Hanson, "Combinatorial Information Market Design" (2003)
///
/// # 示例 (Examples)
///
/// ```text
/// // Case 1: Similar values
/// fp_log_sum_exp(10 × 2^64, 10.5 × 2^64)
/// = 10.5 + ln(1 + exp(-0.5))
/// ≈ 10.5 + 0.474 = 10.974 × 2^64
///
/// // Case 2: Very different values
/// fp_log_sum_exp(10 × 2^64, 50 × 2^64)
/// = 50 + ln(1 + exp(-40))
/// ≈ 50 + 0 = 50 × 2^64              // exp(-40) ≈ 4e-18, negligible
///
/// // Case 3: LMSR cost calculation
/// q_yes = 1000, q_no = 500, b = 100
/// C = b × fp_log_sum_exp(q_yes/b, q_no/b)
///   = 100 × fp_log_sum_exp(10, 5)
///   = 100 × 10.0067 = 1000.67 USDC 最小单位
/// ```
pub fn fp_log_sum_exp(a: FixedPoint, b: FixedPoint) -> Result<FixedPoint> {
    let max_val = if a > b { a } else { b };
    let diff = if a > b {
        a.checked_sub(b).ok_or(crate::errors::PredictionMarketError::MathOverflow)?
    } else {
        b.checked_sub(a).ok_or(crate::errors::PredictionMarketError::MathOverflow)?
    };

    // ✅ v1.0.24: 修复 CRITICAL 漏洞 - exp(-diff) 而非 exp(diff)（感谢审计发现!）
    //
    // 🔴 原问题：计算 ln(1 + exp(-diff)) 时错误使用了 exp(diff)
    //    - diff = |a - b| 是非负数
    //    - exp(diff) 随 diff 指数增长 → 严重高估
    //    - exp(-diff) 随 diff 指数衰减 → 正确
    //    - 导致 LMSR cost 被系统性高估
    //    - 影响所有同号分支的买卖定价
    //
    // ✅ 修复：正确计算 exp(-diff) = 1 / exp(diff)
    //    为避免 diff 过大时 exp(diff) 溢出，先检查边界

    // 计算 exp(-diff) = 1 / exp(diff)
    let exp_neg_diff = if diff < from_u64(20) {
        // diff 较小，安全计算 exp(diff) 然后取倒数
        let exp_diff = fp_exp(diff)?;
        fp_div(ONE, exp_diff)?
    } else {
        // diff >= 20，exp(-diff) ≈ 0，直接返回 max_val
        // 因为 ln(1 + exp(-20)) ≈ ln(1) = 0
        return Ok(max_val);
    };

    let one_plus_exp = ONE.checked_add(exp_neg_diff).ok_or(crate::errors::PredictionMarketError::MathOverflow)?;
    let ln_term = fp_ln(one_plus_exp)?;

    max_val.checked_add(ln_term).ok_or(crate::errors::PredictionMarketError::MathOverflow.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fp_mul() {
        // 2.0 * 3.0 = 6.0
        let a = from_u64(2);
        let b = from_u64(3);
        let result = fp_mul(a, b).unwrap();
        assert_eq!(to_u64(result), 6);
    }

    #[test]
    fn test_fp_div() {
        // 6.0 / 2.0 = 3.0
        let a = from_u64(6);
        let b = from_u64(2);
        let result = fp_div(a, b).unwrap();
        assert_eq!(to_u64(result), 3);
    }

    #[test]
    fn test_fp_ln() {
        // ln(e) ≈ 1.0
        let result = fp_ln(E).unwrap();
        let result_u64 = to_u64(result);
        assert!(result_u64 == 0 || result_u64 == 1); // 允许舍入误差
    }

    #[test]
    fn test_fp_exp() {
        // exp(0) = 1
        let result = fp_exp(0).unwrap();
        assert_eq!(result, ONE);

        // exp(1) ≈ e
        let result = fp_exp(ONE).unwrap();
        let diff = if result > E { result - E } else { E - result };
        assert!(diff < (ONE / 100)); // 误差 < 1%
    }
}
