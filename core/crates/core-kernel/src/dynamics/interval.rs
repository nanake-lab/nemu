//! Outward enclosures used ONLY to derive V2-96 fault bounds.
//! Adjacent floats widen proof endpoints; they never repair an estimator value.
use std::ops::{Add, Div, Mul, Sub};

#[derive(Clone, Copy, Debug)]
pub(super) struct I {
    pub lo: f64,
    pub hi: f64,
}
pub(super) fn p(x: f64) -> I {
    I { lo: x, hi: x }
}
impl I {
    fn outward(lo: f64, hi: f64) -> Self {
        Self {
            lo: lo.next_down(),
            hi: hi.next_up(),
        }
    }
    pub fn valid(self) -> bool {
        self.lo.is_finite() && self.hi.is_finite() && self.lo <= self.hi
    }
    pub fn abs(self) -> f64 {
        if !self.valid() {
            return f64::NAN;
        }
        self.lo.abs().max(self.hi.abs())
    }
    pub fn width(self) -> f64 {
        (self.hi - self.lo).next_up()
    }
    pub fn sqrt(self) -> Self {
        if self.lo <= 0.0 {
            return p(f64::NAN);
        }
        Self::outward(libm::sqrt(self.lo), libm::sqrt(self.hi))
    }
    pub fn exp(self) -> Self {
        // libm 0.2.16 exp's documented <1 ulp error. Two neighbours also
        // cover the unequal spacings on either side of a binade boundary.
        Self::outward(libm::exp(self.lo).next_down(), libm::exp(self.hi).next_up())
    }
    pub fn clip(self, low: f64, high: f64) -> Self {
        if !self.valid() {
            return p(f64::NAN);
        }
        Self {
            lo: self.lo.clamp(low, high),
            hi: self.hi.clamp(low, high),
        }
    }
}
impl Add for I {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::outward(self.lo + rhs.lo, self.hi + rhs.hi)
    }
}
impl Sub for I {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::outward(self.lo - rhs.hi, self.hi - rhs.lo)
    }
}
impl Mul for I {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let terms = [
            self.lo * rhs.lo,
            self.lo * rhs.hi,
            self.hi * rhs.lo,
            self.hi * rhs.hi,
        ];
        if terms.iter().any(|x| !x.is_finite()) {
            return p(f64::NAN);
        }
        Self::outward(
            terms.into_iter().fold(f64::INFINITY, f64::min),
            terms.into_iter().fold(f64::NEG_INFINITY, f64::max),
        )
    }
}
impl Div for I {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        if rhs.lo <= 0.0 && rhs.hi >= 0.0 {
            return p(f64::NAN);
        }
        let terms = [
            self.lo / rhs.lo,
            self.lo / rhs.hi,
            self.hi / rhs.lo,
            self.hi / rhs.hi,
        ];
        if terms.iter().any(|x| !x.is_finite()) {
            return p(f64::NAN);
        }
        Self::outward(
            terms.into_iter().fold(f64::INFINITY, f64::min),
            terms.into_iter().fold(f64::NEG_INFINITY, f64::max),
        )
    }
}
