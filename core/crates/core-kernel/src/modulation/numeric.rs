use super::JFault;

pub(super) fn finite(value: f64, field: &'static str) -> Result<f64, JFault> {
    if !value.is_finite() {
        return Err(JFault::NonFinite(field));
    }
    Ok(if value == 0.0 { 0.0 } else { value })
}

pub(super) fn nonnegative(value: f64, field: &'static str) -> Result<f64, JFault> {
    let value = finite(value, field)?;
    if value < 0.0 {
        return Err(JFault::InvalidConfig(field));
    }
    Ok(value)
}

/// Outward-rounded addition of nonnegative finite binary64 bounds. FastTwoSum
/// recovers the rounding residual; step upwards only if rounded below the
/// exact sum. Exact additions (including 0 + 0) are not inflated. No epsilon
/// allowance can admit an actually over-budget sum, including MAX + minsub.
pub(super) fn add_bound(a: f64, b: f64) -> Result<f64, JFault> {
    let (hi, lo) = if a >= b { (a, b) } else { (b, a) };
    let sum = finite(hi + lo, "beta_sum")?;
    let residual = lo - (sum - hi);
    finite(if residual > 0.0 { sum.next_up() } else { sum }, "beta_sum")
}
