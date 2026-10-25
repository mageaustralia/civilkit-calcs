//! Exact rational numbers on `i128` with overflow checks. Every operation returns an error instead of
//! wrapping, so a number that does not fit is reported rather than silently wrong.

use crate::error::{CasError, CasResult};
use std::cmp::Ordering;
use std::fmt;

/// A rational number `n/d` in lowest terms with `d > 0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rat {
    n: i128,
    d: i128,
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

impl Rat {
    /// Zero.
    pub const ZERO: Rat = Rat { n: 0, d: 1 };
    /// One.
    pub const ONE: Rat = Rat { n: 1, d: 1 };

    /// Builds `n/d` in lowest terms. A zero denominator is `DivisionByZero`.
    pub fn new(n: i128, d: i128) -> CasResult<Rat> {
        if d == 0 {
            return Err(CasError::DivisionByZero);
        }
        let (mut n, mut d) = (n, d);
        if d < 0 {
            n = n.checked_neg().ok_or(CasError::Overflow)?;
            d = d.checked_neg().ok_or(CasError::Overflow)?;
        }
        let g = gcd(n.unsigned_abs(), d as u128);
        if g > 1 {
            n /= g as i128;
            d /= g as i128;
        }
        Ok(Rat { n, d })
    }

    /// The integer `n`.
    pub fn int(n: i64) -> Rat {
        Rat { n: n as i128, d: 1 }
    }

    /// The numerator (sign carried here).
    pub fn numer(&self) -> i128 {
        self.n
    }

    /// The denominator (always positive).
    pub fn denom(&self) -> i128 {
        self.d
    }

    /// True for zero.
    pub fn is_zero(&self) -> bool {
        self.n == 0
    }

    /// True for one.
    pub fn is_one(&self) -> bool {
        self.n == 1 && self.d == 1
    }

    /// True when the denominator is 1.
    pub fn is_integer(&self) -> bool {
        self.d == 1
    }

    /// True when below zero.
    pub fn is_negative(&self) -> bool {
        self.n < 0
    }

    /// True when above zero.
    pub fn is_positive(&self) -> bool {
        self.n > 0
    }

    /// `self + o`.
    pub fn add(&self, o: Rat) -> CasResult<Rat> {
        let g = gcd(self.d as u128, o.d as u128) as i128;
        let a = self.n.checked_mul(o.d / g).ok_or(CasError::Overflow)?;
        let b = o.n.checked_mul(self.d / g).ok_or(CasError::Overflow)?;
        let d = (self.d / g).checked_mul(o.d).ok_or(CasError::Overflow)?;
        Rat::new(a.checked_add(b).ok_or(CasError::Overflow)?, d)
    }

    /// `-self`.
    pub fn neg(&self) -> CasResult<Rat> {
        Ok(Rat {
            n: self.n.checked_neg().ok_or(CasError::Overflow)?,
            d: self.d,
        })
    }

    /// `self - o`.
    pub fn sub(&self, o: Rat) -> CasResult<Rat> {
        self.add(o.neg()?)
    }

    /// `self * o`.
    pub fn mul(&self, o: Rat) -> CasResult<Rat> {
        let g1 = gcd(self.n.unsigned_abs(), o.d as u128).max(1) as i128;
        let g2 = gcd(o.n.unsigned_abs(), self.d as u128).max(1) as i128;
        let n = (self.n / g1).checked_mul(o.n / g2).ok_or(CasError::Overflow)?;
        let d = (self.d / g2).checked_mul(o.d / g1).ok_or(CasError::Overflow)?;
        Ok(Rat { n, d })
    }

    /// `1 / self`; zero is `DivisionByZero`.
    pub fn recip(&self) -> CasResult<Rat> {
        Rat::new(self.d, self.n)
    }

    /// `self / o`.
    pub fn div(&self, o: Rat) -> CasResult<Rat> {
        self.mul(o.recip()?)
    }

    /// `self` to an integer power (negative powers take the reciprocal).
    pub fn pow_i(&self, e: i128) -> CasResult<Rat> {
        if e == 0 {
            return Ok(Rat::ONE);
        }
        let base = if e < 0 { self.recip()? } else { *self };
        let mut k = e.unsigned_abs();
        if k > 4096 && !(base.n.abs() == 1 && base.d == 1) {
            return Err(CasError::Overflow);
        }
        let mut result = Rat::ONE;
        let mut sq = base;
        while k > 0 {
            if k & 1 == 1 {
                result = result.mul(sq)?;
            }
            k >>= 1;
            if k > 0 {
                sq = sq.mul(sq)?;
            }
        }
        Ok(result)
    }

    /// The largest integer not above the value.
    pub fn floor(&self) -> i128 {
        self.n.div_euclid(self.d)
    }

    /// The nearest `f64`.
    pub fn to_f64(&self) -> f64 {
        self.n as f64 / self.d as f64
    }
}

/// Compares `a/b` with `c/d` (both denominators positive) without overflow, by continued fractions.
fn cmp_frac(a: i128, b: i128, c: i128, d: i128) -> Ordering {
    let (fa, fb) = (a.div_euclid(b), c.div_euclid(d));
    if fa != fb {
        return fa.cmp(&fb);
    }
    let (ra, rb) = (a - fa * b, c - fb * d);
    match (ra == 0, rb == 0) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        // ra/b vs rb/d  ==  reverse of  b/ra vs d/rb
        (false, false) => cmp_frac(d, rb, b, ra),
    }
}

impl Ord for Rat {
    fn cmp(&self, o: &Rat) -> Ordering {
        cmp_frac(self.n, self.d, o.n, o.d)
    }
}

impl PartialOrd for Rat {
    fn partial_cmp(&self, o: &Rat) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl fmt::Display for Rat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.d == 1 {
            write!(f, "{}", self.n)
        } else {
            write!(f, "{}/{}", self.n, self.d)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(n: i128, d: i128) -> Rat {
        Rat::new(n, d).unwrap()
    }

    #[test]
    fn normalises() {
        assert_eq!(r(2, 4), r(1, 2));
        assert_eq!(r(1, -2), r(-1, 2));
        assert_eq!(r(0, 5), Rat::ZERO);
        assert_eq!(r(6, 3).numer(), 2);
    }

    #[test]
    fn zero_denominator_is_error() {
        assert_eq!(Rat::new(1, 0), Err(CasError::DivisionByZero));
        assert_eq!(Rat::ZERO.recip(), Err(CasError::DivisionByZero));
    }

    #[test]
    fn arithmetic() {
        assert_eq!(r(1, 2).add(r(1, 3)).unwrap(), r(5, 6));
        assert_eq!(r(1, 2).sub(r(1, 2)).unwrap(), Rat::ZERO);
        assert_eq!(r(2, 3).mul(r(3, 4)).unwrap(), r(1, 2));
        assert_eq!(r(1, 2).div(r(1, 4)).unwrap(), Rat::int(2));
        assert_eq!(r(2, 3).pow_i(-2).unwrap(), r(9, 4));
        assert_eq!(r(-2, 1).pow_i(3).unwrap(), Rat::int(-8));
    }

    #[test]
    fn overflow_is_reported() {
        let big = Rat::new(i128::MAX / 2, 1).unwrap();
        assert_eq!(big.mul(Rat::int(4)), Err(CasError::Overflow));
        assert_eq!(Rat::int(10).pow_i(60), Err(CasError::Overflow));
    }

    #[test]
    fn ordering() {
        assert!(r(1, 3) < r(1, 2));
        assert!(r(-1, 2) < r(-1, 3));
        assert!(r(7, 2) > r(3, 1));
        let a = Rat::new(i128::MAX - 1, i128::MAX - 2).unwrap();
        let b = Rat::new(i128::MAX - 2, i128::MAX - 3).unwrap();
        assert!(a < b);
    }

    #[test]
    fn floor_and_display() {
        assert_eq!(r(-1, 2).floor(), -1);
        assert_eq!(r(5, 2).floor(), 2);
        assert_eq!(format!("{}", r(3, 6)), "1/2");
        assert_eq!(format!("{}", Rat::int(4)), "4");
        assert!((r(1, 3).to_f64() - 0.333333333).abs() < 1e-8);
    }
}
