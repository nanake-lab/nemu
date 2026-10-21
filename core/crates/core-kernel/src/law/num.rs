//! Binary64 helpers with a fixed evaluation order (v2.0 §2.2; D9, V2-44).
//!
//! The group mean is computed exactly: the binary64 inputs are summed without
//! rounding in a fixed-point accumulator and the quotient is rounded once to
//! the nearest binary64 (ties to even). The result does not depend on member
//! order, and repeating every member of a group any number of times leaves it
//! bit-for-bit unchanged (LP3, AC2-03).

use super::error::{LawError, LawResult};

/// The four nature channels in the fixed order of v2.0 §3.
pub type Nature4 = [f64; 4];

/// Reject NaN and ±Inf; store −0 as +0 (CanonicalV1 normalises the sign of zero).
pub fn finite(value: f64, what: &'static str) -> LawResult<f64> {
    if !value.is_finite() {
        return Err(LawError::NonFinite(what));
    }
    Ok(if value == 0.0 { 0.0 } else { value })
}

/// ⟨x, d⟩ accumulated left to right in channel order.
pub fn dot(x: &Nature4, d: &Nature4) -> f64 {
    ((x[0] * d[0] + x[1] * d[1]) + x[2] * d[2]) + x[3] * d[3]
}

/// clip(x, lo, hi) for lo ≤ hi; x is finite.
pub fn clip(x: f64, lo: f64, hi: f64) -> f64 {
    x.max(lo).min(hi)
}

/// Inputs to the exact mean must satisfy |x| ≤ 4 (differences of two values in
/// [−1, 1] are at most 2 in magnitude).
pub const MEAN_INPUT_BOUND: f64 = 4.0;

const LIMBS: usize = 18; // 1152 bits: inputs reach bit 1077; 2^32 terms add 32 bits.

/// Nonnegative fixed-point integer in units of 2^−1074 (the smallest subnormal).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Fixed([u64; LIMBS]); // limb 0 is least significant

impl Fixed {
    const ZERO: Self = Self([0; LIMBS]);

    fn add_shifted(&mut self, mantissa: u64, shift: u32) {
        // mantissa < 2^53 and shift ≤ 1077, so the addend spans limbs `limb`, `limb + 1`.
        let limb = (shift / 64) as usize;
        let wide = u128::from(mantissa) << (shift % 64);
        let parts = [wide as u64, (wide >> 64) as u64];
        let mut carry = false;
        for i in limb..LIMBS {
            let add = if i - limb < 2 { parts[i - limb] } else { 0 };
            let (s1, c1) = self.0[i].overflowing_add(add);
            let (s2, c2) = s1.overflowing_add(u64::from(carry));
            self.0[i] = s2;
            carry = c1 || c2;
            if i > limb && !carry {
                break;
            }
        }
        debug_assert!(!carry, "fixed accumulator overflow");
    }

    fn sub(&self, other: &Self) -> Self {
        // Requires self ≥ other.
        let mut out = [0u64; LIMBS];
        let mut borrow = 0u64;
        for (i, slot) in out.iter_mut().enumerate() {
            let (d1, b1) = self.0[i].overflowing_sub(other.0[i]);
            let (d2, b2) = d1.overflowing_sub(borrow);
            *slot = d2;
            borrow = u64::from(b1 || b2);
        }
        Self(out)
    }

    fn cmp_mag(&self, other: &Self) -> std::cmp::Ordering {
        self.0.iter().rev().cmp(other.0.iter().rev())
    }

    /// (quotient, remainder) of division by a nonzero u64.
    fn div_rem(&self, n: u64) -> (Self, u64) {
        let mut q = [0u64; LIMBS];
        let mut rem = 0u128;
        for i in (0..LIMBS).rev() {
            let cur = (rem << 64) | u128::from(self.0[i]);
            q[i] = (cur / u128::from(n)) as u64;
            rem = cur % u128::from(n);
        }
        (Self(q), rem as u64)
    }

    fn bit_length(&self) -> u32 {
        for i in (0..LIMBS).rev() {
            if self.0[i] != 0 {
                return 64 * i as u32 + (64 - self.0[i].leading_zeros());
            }
        }
        0
    }

    fn bits(&self, from: u32, count: u32) -> u64 {
        // count ≤ 64; bits [from, from + count).
        let mut out = 0u64;
        for k in 0..count {
            let b = from + k;
            let limb = (b / 64) as usize;
            if limb < LIMBS && (self.0[limb] >> (b % 64)) & 1 == 1 {
                out |= 1 << k;
            }
        }
        out
    }

    fn any_below(&self, bit: u32) -> bool {
        (0..bit).any(|b| (self.0[(b / 64) as usize] >> (b % 64)) & 1 == 1)
    }
}

/// Split a finite binary64 into (negative, mantissa, shift) with |x| = mantissa · 2^(shift − 1074).
fn decompose(x: f64) -> (bool, u64, u32) {
    let bits = x.to_bits();
    let negative = bits >> 63 == 1;
    let exp = ((bits >> 52) & 0x7ff) as u32;
    let frac = bits & ((1u64 << 52) - 1);
    if exp == 0 {
        (negative, frac, 0)
    } else {
        (negative, frac | (1u64 << 52), exp - 1)
    }
}

/// 2^k for −1074 ≤ k ≤ 1023, exactly.
fn pow2(k: i32) -> f64 {
    if k >= -1022 {
        f64::from_bits(((k + 1023) as u64) << 52)
    } else {
        f64::from_bits(1u64 << (k + 1074))
    }
}

/// Round (q + rem/n) · 2^−1074 to the nearest binary64, ties to even.
fn round_fixed(q: &Fixed, rem: u64, n: u64) -> f64 {
    let h = q.bit_length();
    if h <= 53 {
        // Below 2^−1021 the spacing of binary64 is 2^−1074: round at the unit.
        let mut m = q.bits(0, 53);
        let twice = u128::from(rem) * 2;
        let n = u128::from(n);
        if twice > n || (twice == n && m & 1 == 1) {
            m += 1;
        }
        return m as f64 * pow2(-1074);
    }
    let mut shift = h - 53;
    let mut m = q.bits(shift, 53);
    let half_bit = q.bits(shift - 1, 1) == 1;
    let sticky = q.any_below(shift - 1) || rem != 0;
    if half_bit && (sticky || m & 1 == 1) {
        m += 1;
        if m == 1u64 << 53 {
            m >>= 1;
            shift += 1;
        }
    }
    m as f64 * pow2(shift as i32 - 1074)
}

/// Exact mean of finite values with |x| ≤ MEAN_INPUT_BOUND, rounded once.
pub fn exact_mean(values: &[f64]) -> LawResult<f64> {
    if values.is_empty() {
        return Err(LawError::InvalidInput("mean of an empty group".into()));
    }
    let n = u64::try_from(values.len()).map_err(|_| LawError::Overflow("group size"))?;
    if n > u64::from(u32::MAX) {
        return Err(LawError::Overflow("group size"));
    }
    let (mut pos, mut neg) = (Fixed::ZERO, Fixed::ZERO);
    for &x in values {
        if !x.is_finite() || x.abs() > MEAN_INPUT_BOUND {
            return Err(LawError::InvalidInput(format!(
                "mean input {x} out of range"
            )));
        }
        let (negative, mantissa, shift) = decompose(x);
        if mantissa == 0 {
            continue;
        }
        if negative {
            neg.add_shifted(mantissa, shift);
        } else {
            pos.add_shifted(mantissa, shift);
        }
    }
    let (negative, magnitude) = match pos.cmp_mag(&neg) {
        std::cmp::Ordering::Less => (true, neg.sub(&pos)),
        _ => (false, pos.sub(&neg)),
    };
    let (q, rem) = magnitude.div_rem(n);
    let v = round_fixed(&q, rem, n);
    Ok(if negative && v != 0.0 { -v } else { v })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_mean_of_single_value_is_the_value() {
        for x in [
            0.0,
            1.0,
            -1.0,
            0.1,
            -0.3,
            2.0,
            -2.0,
            1e-300,
            f64::from_bits(1),
        ] {
            assert_eq!(exact_mean(&[x]).unwrap().to_bits(), x.to_bits(), "{x}");
        }
        assert_eq!(exact_mean(&[-0.0]).unwrap().to_bits(), 0.0f64.to_bits());
    }

    #[test]
    fn exact_mean_rounds_ties_to_even() {
        let up = f64::from_bits(1.0f64.to_bits() + 1);
        // (1 + (1 + ulp))/2 = 1 + ulp/2: a tie, even is 1.0.
        assert_eq!(exact_mean(&[1.0, up]).unwrap(), 1.0);
        let up2 = f64::from_bits(1.0f64.to_bits() + 2);
        let up3 = f64::from_bits(1.0f64.to_bits() + 3);
        // (1 + 2ulp + 1 + 3ulp)/2 = 1 + 2.5 ulp: tie between 2 and 3 ulp, even is 2.
        assert_eq!(exact_mean(&[up2, up3]).unwrap(), up2);
    }

    #[test]
    fn exact_mean_is_order_and_repetition_invariant() {
        let xs = [0.1, -0.7, 0.3333333333333333, 1.9, -1.25e-5];
        let base = exact_mean(&xs).unwrap();
        let mut rev = xs;
        rev.reverse();
        assert_eq!(exact_mean(&rev).unwrap().to_bits(), base.to_bits());
        for k in 2..10 {
            let rep: Vec<f64> = xs.iter().copied().cycle().take(xs.len() * k).collect();
            assert_eq!(exact_mean(&rep).unwrap().to_bits(), base.to_bits(), "k={k}");
        }
    }

    #[test]
    fn exact_mean_rejects_bad_inputs() {
        assert!(exact_mean(&[]).is_err());
        assert!(exact_mean(&[f64::NAN]).is_err());
        assert!(exact_mean(&[f64::INFINITY]).is_err());
        assert!(exact_mean(&[4.5]).is_err());
    }

    #[test]
    fn finite_normalises_negative_zero_and_rejects_nonfinite() {
        assert_eq!(finite(-0.0, "z").unwrap().to_bits(), 0);
        assert!(finite(f64::NAN, "n").is_err());
        assert!(finite(f64::NEG_INFINITY, "i").is_err());
    }
}
