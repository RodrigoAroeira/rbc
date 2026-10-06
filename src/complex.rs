use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const ONE: Self = Self::new(1.0, 0.0);

    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// Magnitude of the complex number. Only the test suite reads this today.
    #[cfg_attr(not(test), allow(dead_code))]
    #[inline]
    pub fn abs(self) -> f64 {
        self.mag2().sqrt()
    }

    #[inline]
    pub const fn mag2(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    #[inline]
    pub const fn is_zero(self) -> bool {
        self.re == 0.0 && self.im == 0.0
    }

    #[inline]
    pub const fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }

    /// Multiplicative inverse `1/z`.
    #[inline]
    pub fn recip(self) -> Self {
        // 1/z = conj(z) / |z|^2
        let mag_sq = self.mag2();
        self.conj() / mag_sq
        // Self::new(self.re / mag_sq, -self.im / mag_sq)
    }
}

impl AddAssign for Complex {
    fn add_assign(&mut self, rhs: Self) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

impl Add for Complex {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl SubAssign for Complex {
    fn sub_assign(&mut self, rhs: Self) {
        self.re -= rhs.re;
        self.im -= rhs.im;
    }
}

impl Sub for Complex {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl DivAssign for Complex {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl Div for Complex {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        // (a + bi) / (c + di) = ((ac + bd) + (bc - ad)i) / (c^2 + d^2)
        let denom = rhs.re * rhs.re + rhs.im * rhs.im;
        Self::new(
            (self.re * rhs.re + self.im * rhs.im) / denom,
            (self.im * rhs.re - self.re * rhs.im) / denom,
        )
    }
}

impl Div<f64> for Complex {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.re / rhs, self.im / rhs)
    }
}

impl From<f64> for Complex {
    fn from(re: f64) -> Self {
        Self::new(re, 0.0)
    }
}

impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Drop a zero part rather than printing `0+3i` or `4+0i`.
        match (self.re == 0.0, self.im == 0.0) {
            (true, true) => write!(f, "0"),
            (true, false) => write!(f, "{}j", self.im),
            (false, true) => write!(f, "{}", self.re),
            (false, false) => {
                write!(f, "{}{}", self.re, if self.im < 0.0 { "-" } else { "+" })?;
                write!(f, "{}j", self.im.abs())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_arithmetic() {
        assert_eq!(
            Complex::new(1.0, 2.0) + Complex::new(3.0, -1.0),
            Complex::new(4.0, 1.0)
        );
        assert_eq!(
            Complex::new(1.0, 2.0) - Complex::new(3.0, -1.0),
            Complex::new(-2.0, 3.0)
        );
    }

    #[test]
    fn test_division() {
        assert_eq!(
            Complex::new(1.0, 1.0) / Complex::new(1.0, 0.0),
            Complex::new(1.0, 1.0)
        );
        assert_eq!(
            Complex::new(0.0, 2.0) / Complex::new(0.0, 2.0),
            Complex::new(1.0, 0.0)
        );
        assert_eq!(
            Complex::new(1.0, 1.0) / Complex::new(1.0, -1.0),
            Complex::new(0.0, 1.0)
        );
    }

    #[test]
    fn test_recip() {
        assert!(Complex::ZERO.is_zero());
        assert!(!Complex::new(0.0, 2.0).is_zero());
        assert_eq!(Complex::new(2.0, 0.0).recip(), Complex::new(0.5, 0.0));
        assert_eq!(Complex::new(0.0, 2.0).recip(), Complex::new(0.0, -0.5));
        // recip() must agree with the general division it is built on
        assert_eq!(
            Complex::new(3.0, 4.0).recip(),
            Complex::ONE / Complex::new(3.0, 4.0)
        );
    }

    #[test]
    fn test_abs() {
        assert_eq!(Complex::new(3.0, 4.0).abs(), 5.0);
        assert_eq!(Complex::new(-3.0, 0.0).abs(), 3.0);
    }

    #[test]
    fn test_display() {
        assert_eq!(Complex::new(4700.0, 0.0).to_string(), "4700");
        assert_eq!(Complex::new(3.0, 4.0).to_string(), "3+4i");
        assert_eq!(Complex::new(3.0, -4.0).to_string(), "3-4i");
        assert_eq!(Complex::new(-3.0, 4.0).to_string(), "-3+4i");
        // zero parts are dropped rather than shown as `0+3i` / `4+0i`
        assert_eq!(Complex::ZERO.to_string(), "0");
        assert_eq!(Complex::new(0.0, 3.0).to_string(), "3i");
        assert_eq!(Complex::new(0.0, -3.0).to_string(), "-3i");
        assert_eq!(Complex::new(0.0, 1.0).to_string(), "1i");
    }

    #[test]
    fn test_from_f64() {
        assert_eq!(Complex::from(4.7), Complex::new(4.7, 0.0));
        assert_eq!(Complex::ZERO, Complex::from(0.0));
    }
}
