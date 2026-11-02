//! Explicit binary64 operations. No tolerance, nextafter, FMA or fast-math.
use super::{Fault, Result};

pub(super) fn finite(x: f64) -> Result<f64> {
    if !x.is_finite() {
        return Err(Fault::NonFinite);
    }
    Ok(if x == 0.0 { 0.0 } else { x })
}
pub(super) fn add(a: f64, b: f64) -> Result<f64> {
    finite(a + b)
}
pub(super) fn sub(a: f64, b: f64) -> Result<f64> {
    finite(a - b)
}
pub(super) fn mul(a: f64, b: f64) -> Result<f64> {
    let x = finite(a * b)?;
    if a != 0.0 && b != 0.0 && x == 0.0 {
        return Err(Fault::Underflow);
    }
    Ok(x)
}
pub(super) fn div(a: f64, b: f64) -> Result<f64> {
    if b == 0.0 {
        return Err(Fault::ZeroDenominator);
    }
    let x = finite(a / b)?;
    if a != 0.0 && x == 0.0 {
        return Err(Fault::Underflow);
    }
    Ok(x)
}
pub(super) fn count(n: u64) -> Result<f64> {
    if n >= (1_u64 << 53) {
        return Err(Fault::CounterExhausted);
    }
    Ok(n as f64)
}
pub(super) fn increment(n: u64) -> Result<u64> {
    let n = n.checked_add(1).ok_or(Fault::CounterExhausted)?;
    count(n)?;
    Ok(n)
}
pub(super) fn sign(x: f64) -> i8 {
    if x > 0.0 {
        1
    } else if x < 0.0 {
        -1
    } else {
        0
    }
}
