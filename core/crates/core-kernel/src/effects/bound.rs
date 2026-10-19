//! Conservative binary64 arithmetic for budget checks.
//!
//! S 6: budget computations use conservative upper bounds; floating error
//! must not let a boundary decision pass that the exact value would fail.
//! Plain left-to-right binary64 addition can round a total down onto the
//! budget (1 + 2^-53 + 2^-53 rounds to 1), so every budget sum and bound
//! product goes through these functions instead.
//!
//! Each function returns a value ≥ the exact real result of the same
//! operation on its inputs. Exact operations are not inflated: the rounding
//! error is recovered with an error-free transformation (TwoSum for addition,
//! a fused multiply-add residual for multiplication), and the result moves up
//! one ulp only when the exact value lies above it. Inputs are finite and
//! non-negative; an overflow becomes +∞, which no budget admits.

/// Smallest magnitude at which a product residual is guaranteed to be
/// representable; below it the residual may underflow, so the product is
/// rounded up unconditionally.
const RESIDUAL_SAFE: f64 = f64::MIN_POSITIVE * 9_007_199_254_740_992.0; // 2^-1022 · 2^53

/// Upper bound of `a + b`.
pub fn add_up(a: f64, b: f64) -> f64 {
    let s = a + b;
    if !s.is_finite() {
        return f64::INFINITY;
    }
    // TwoSum (Knuth): s + err == a + b exactly for finite, non-overflowing s.
    let bb = s - a;
    let err = (a - (s - bb)) + (b - bb);
    if err > 0.0 { s.next_up() } else { s }
}

/// Upper bound of `a · b`.
pub fn mul_up(a: f64, b: f64) -> f64 {
    let p = a * b;
    if !p.is_finite() {
        return f64::INFINITY;
    }
    if p < RESIDUAL_SAFE {
        return p.next_up();
    }
    // a·b − p, computed exactly by a correctly rounded fused multiply-add
    // (pure-Rust libm, so the result does not depend on the platform).
    let err = libm::fma(a, b, -p);
    if err > 0.0 { p.next_up() } else { p }
}

/// Upper bound of a sum, accumulated in the given (canonical) order.
pub fn sum_up(values: impl IntoIterator<Item = f64>) -> f64 {
    values.into_iter().fold(0.0, add_up)
}
