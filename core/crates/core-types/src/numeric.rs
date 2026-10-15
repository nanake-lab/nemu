//! Source: System Plan §6 and Operators §19. No serialization or runtime wiring.
//! Limits are explicit inputs; this module does not choose deployment limits.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericError {
    NonFinite,
    Overflow,
    LimitExceeded,
}

/// A finite binary64 value with a unique positive-zero representation.
/// No arithmetic traits: every computed result must pass validation again.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FiniteBinary64(f64);

impl FiniteBinary64 {
    pub fn new(value: f64) -> Result<Self, NumericError> {
        if !value.is_finite() {
            return Err(NumericError::NonFinite);
        }
        Ok(Self(if value == 0.0 { 0.0 } else { value }))
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

/// Nonnegative elapsed interval, not a wall-clock timestamp or clock epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NanosecondInterval {
    value: u64,
    limit: u64,
}

impl NanosecondInterval {
    pub fn new(value: u128, limit: u64) -> Result<Self, NumericError> {
        let value = u64::try_from(value).map_err(|_| NumericError::Overflow)?;
        if value > limit {
            return Err(NumericError::LimitExceeded);
        }
        Ok(Self { value, limit })
    }

    pub fn get(self) -> u64 {
        self.value
    }

    pub fn limit(self) -> u64 {
        self.limit
    }

    /// Failure leaves the caller's original value intact.
    pub fn checked_add(self, increment: u64) -> Result<Self, NumericError> {
        Self::new(u128::from(self.value) + u128::from(increment), self.limit)
    }
}

/// Checked count with an explicit cap. Never wraps or saturates silently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundedCount {
    value: u64,
    limit: u64,
}

impl BoundedCount {
    pub fn new(value: u128, limit: u64) -> Result<Self, NumericError> {
        let value = u64::try_from(value).map_err(|_| NumericError::Overflow)?;
        if value > limit {
            return Err(NumericError::LimitExceeded);
        }
        Ok(Self { value, limit })
    }

    pub fn get(self) -> u64 {
        self.value
    }

    pub fn limit(self) -> u64 {
        self.limit
    }

    pub fn checked_add(self, increment: u64) -> Result<Self, NumericError> {
        Self::new(u128::from(self.value) + u128::from(increment), self.limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sys21_finite_values_preserve_binary64_bits() {
        for value in [
            0.0,
            1.5,
            -1.5,
            f64::MAX,
            f64::MIN,
            f64::MIN_POSITIVE,
            f64::from_bits(1),
        ] {
            assert_eq!(
                FiniteBinary64::new(value).unwrap().get().to_bits(),
                value.to_bits()
            );
        }
    }

    #[test]
    fn sys21_rejects_all_nonfinite_categories() {
        for value in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::from_bits(0x7ff0000000000001),
        ] {
            assert_eq!(FiniteBinary64::new(value), Err(NumericError::NonFinite));
        }
    }

    #[test]
    fn sys21_normalizes_negative_zero_only() {
        assert_eq!(FiniteBinary64::new(-0.0).unwrap().get().to_bits(), 0);
        assert_eq!(
            FiniteBinary64::new(-f64::from_bits(1))
                .unwrap()
                .get()
                .to_bits(),
            0x8000000000000001
        );
    }

    #[test]
    fn sys21_count_accepts_boundaries_rejects_limit_and_overflow() {
        assert_eq!(BoundedCount::new(0, 0).unwrap().get(), 0);
        let count = BoundedCount::new(2, 3).unwrap();
        assert_eq!(count.checked_add(1).unwrap().get(), 3);
        assert_eq!(count.checked_add(2), Err(NumericError::LimitExceeded));
        assert_eq!(count.get(), 2);
        assert_eq!(count.limit(), 3);
        let max = BoundedCount::new(u128::from(u64::MAX), u64::MAX).unwrap();
        assert_eq!(max.checked_add(1), Err(NumericError::Overflow));
        assert_eq!(
            BoundedCount::new(u128::MAX, u64::MAX),
            Err(NumericError::Overflow)
        );
    }

    #[test]
    fn sys21_intervals_are_explicitly_bounded_and_never_wrap() {
        assert_eq!(NanosecondInterval::new(0, 0).unwrap().get(), 0);
        let interval = NanosecondInterval::new(10, 20).unwrap();
        assert_eq!(interval.checked_add(10).unwrap().get(), 20);
        assert_eq!(interval.checked_add(11), Err(NumericError::LimitExceeded));
        assert_eq!(interval.get(), 10);
        assert_eq!(interval.limit(), 20);
        let max = NanosecondInterval::new(u128::from(u64::MAX), u64::MAX).unwrap();
        assert_eq!(max.checked_add(1), Err(NumericError::Overflow));
        assert_eq!(
            NanosecondInterval::new(u128::MAX, u64::MAX),
            Err(NumericError::Overflow)
        );
    }
}
