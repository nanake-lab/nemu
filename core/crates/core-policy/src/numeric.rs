//! Numeric context values in the field's own fixed-point units.
//!
//! A numeric field is fixed-point in units of 10^-d (catalog). A binary64
//! context value is first quantised, exactly, to the nearest unit (ties to
//! even), and the atom `[lo, hi]` is then an integer comparison. Overlap is
//! proven on the same integer units, so two atoms proven `Disjoint` can never
//! both hit one context value. Dividing the bounds in binary64 instead would
//! round neighbouring units onto one float near 2^53 and let a disjointness
//! proof and an evaluation disagree (S 6: floating error must not flip a
//! boundary decision).

/// `lo ≤ units(v) ≤ hi`. Non-finite values never hit.
pub fn in_units(v: f64, lo: i64, hi: i64, decimals: u8) -> bool {
    if !v.is_finite() {
        return false;
    }
    let u = to_units(v, decimals);
    i128::from(lo) <= u && u <= i128::from(hi)
}

/// Saturation far outside every i64 bound; beyond it only the sign matters.
const SATURATED: i128 = 1 << 100;

/// v·10^d rounded to the nearest integer, ties to even, computed exactly for
/// finite `v` and saturated to ±2^100.
pub fn to_units(v: f64, decimals: u8) -> i128 {
    let bits = v.to_bits();
    let exponent_bits = ((bits >> 52) & 0x7ff) as i32;
    let fraction = i128::from(bits & ((1u64 << 52) - 1));
    // v = mantissa · 2^exponent exactly; subnormals have no implicit bit.
    let (mut mantissa, exponent) = if exponent_bits == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1 << 52), exponent_bits - 1075)
    };
    if bits >> 63 == 1 {
        mantissa = -mantissa;
    }
    // |mantissa| < 2^53 and 10^d ≤ 10^12 < 2^40, so this cannot overflow.
    let scaled = mantissa * pow10(decimals);
    if exponent >= 0 {
        let exact = if exponent < 100 {
            scaled.checked_mul(1i128 << exponent)
        } else {
            None
        };
        return exact.map_or(SATURATED * scaled.signum(), |x| {
            x.clamp(-SATURATED, SATURATED)
        });
    }
    let shift = -exponent;
    if shift >= 120 {
        // |scaled / 2^shift| < 2^-27 rounds to zero.
        return 0;
    }
    let divisor = 1i128 << shift;
    let floor = scaled.div_euclid(divisor);
    let twice_remainder = 2 * scaled.rem_euclid(divisor);
    match twice_remainder.cmp(&divisor) {
        std::cmp::Ordering::Less => floor,
        std::cmp::Ordering::Greater => floor + 1,
        std::cmp::Ordering::Equal if floor.rem_euclid(2) == 0 => floor,
        std::cmp::Ordering::Equal => floor + 1,
    }
}

fn pow10(decimals: u8) -> i128 {
    (0..decimals).fold(1i128, |acc, _| acc * 10)
}

#[cfg(test)]
mod tests {
    use super::{in_units, to_units};

    #[test]
    fn decimal_values_land_on_their_unit() {
        assert!(in_units(0.1, 1, 1, 1));
        assert!(in_units(0.3, 3, 3, 1));
        assert!(in_units(-2.5, -25, -25, 1));
        assert!(!in_units(-2.5, -24, 0, 1));
        assert!(in_units(123.456, 123_456, 123_456, 3));
        assert!(in_units(0.0, 0, 0, 12));
        assert!(in_units(-0.0, 0, 0, 0));
        assert!(!in_units(f64::NAN, i64::MIN, i64::MAX, 0));
        assert!(!in_units(f64::INFINITY, i64::MIN, i64::MAX, 0));
    }

    #[test]
    fn ties_go_to_even_in_both_signs() {
        assert_eq!(to_units(0.25, 1), 2);
        assert_eq!(to_units(0.75, 1), 8);
        assert_eq!(to_units(-0.25, 1), -2);
        assert_eq!(to_units(-0.75, 1), -8);
        assert_eq!(to_units(2.5, 0), 2);
        assert_eq!(to_units(3.5, 0), 4);
    }

    #[test]
    fn extremes_saturate_or_vanish() {
        assert!(!in_units(f64::MAX, i64::MIN, i64::MAX, 12));
        assert!(!in_units(f64::MIN, i64::MIN, i64::MAX, 12));
        assert_eq!(to_units(f64::MIN_POSITIVE / 4.0, 12), 0);
        assert_eq!(to_units(-f64::MIN_POSITIVE, 0), 0);
    }

    #[test]
    fn neighbouring_units_near_2_pow_53_get_one_owner() {
        // n/10 and (n+1)/10 round onto one binary64 value; quantising puts
        // that value into exactly one unit.
        let n = 9_007_199_254_740_952i64;
        let v = n as f64 / 10.0;
        assert!(in_units(v, n, n, 1) ^ in_units(v, n + 1, n + 1, 1));
    }
}
