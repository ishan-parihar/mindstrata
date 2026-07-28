//! Deterministic fixed-point arithmetic.
//!
//! All simulation values use [`Fixed`] instead of `f32`/`f64` to guarantee
//! cross-platform determinism.  The internal representation is `i64` scaled by
//! [`SCALE`] (10 000), giving four decimal digits of precision over the range
//! roughly −922 337 203 685 477 ..= +922 337 203 685 477.
//!
//! For the simulation's typical range of 0.0 ..= 1.0 this maps to 0 ..= 10 000.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Rem, Sub, SubAssign};

/// Internal scale factor: 1.0 == 10_000.
pub const SCALE: i64 = 10_000;

/// One unit in the fixed-point domain.
pub const ONE: i64 = SCALE;

/// A deterministic fixed-point number.
///
/// Internally stored as `i64` with 4 decimal places of precision.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Fixed(i64);

impl Default for Fixed {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Fixed {
    // ── Constructors ──────────────────────────────────────────────────

    /// Wrap a raw scaled `i64` value.
    #[inline]
    pub const fn from_raw(v: i64) -> Self {
        Self(v)
    }

    /// Create from an integer (e.g. `Fixed::from_int(5)` == 5.0).
    #[inline]
    pub const fn from_int(v: i64) -> Self {
        Self(v * SCALE)
    }

    /// Create from a float.  Prefer `const` constructors in hot paths.
    #[inline]
    pub fn from_f64(v: f64) -> Self {
        Self((v * SCALE as f64).round() as i64)
    }

    // ── Constants ─────────────────────────────────────────────────────

    /// 0.0
    pub const ZERO: Self = Self(0);
    /// 1.0
    pub const ONE: Self = Self(SCALE);
    /// 0.5
    pub const HALF: Self = Self(SCALE / 2);
    /// The maximum representable value (practical).
    pub const MAX: Self = Self(i64::MAX);
    /// The minimum representable value (practical).
    pub const MIN: Self = Self(i64::MIN);

    // ── Conversion ────────────────────────────────────────────────────

    /// Raw scaled integer.
    #[inline]
    pub const fn to_raw(self) -> i64 {
        self.0
    }

    /// Convert to `f64`.
    #[inline]
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / SCALE as f64
    }

    // ── Clamping / Ranges ─────────────────────────────────────────────

    /// Clamp to `0.0 ..= 1.0`.
    #[inline]
    pub fn clamp_01(self) -> Self {
        Self(self.0.clamp(0, SCALE))
    }

    /// Clamp to an arbitrary range.
    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self(self.0.clamp(min.0, max.0))
    }

    /// Returns `true` if the value is in `0.0 ..= 1.0`.
    #[inline]
    pub fn is_unit(self) -> bool {
        (0..=SCALE).contains(&self.0)
    }

    // ── Math helpers ──────────────────────────────────────────────────

    /// Power via repeated multiplication (integer exponent).
    /// For fractional exponents use `powf`.
    pub fn powi(self, exp: u32) -> Self {
        if exp == 0 {
            return Self::ONE;
        }
        let mut result = self;
        for _ in 1..exp {
            result = result * self;
        }
        result
    }

    /// Raise to a floating-point power (approximate, via `f64` conversion).
    pub fn powf(self, exp: f64) -> Self {
        Self::from_f64(self.to_f64().powf(exp))
    }

    /// Linear interpolation between `self` and `target` by factor `t` in [0, 1].
    pub fn lerp(self, target: Self, t: Fixed) -> Self {
        self + (target - self) * t
    }

    /// Absolute value.
    #[inline]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// Returns `self` rounded down to the nearest integer.
    #[inline]
    pub fn floor(self) -> Self {
        Self((self.0 / SCALE) * SCALE)
    }
}

// ── Display ──────────────────────────────────────────────────────────────

impl fmt::Debug for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fixed({})", self.to_f64())
    }
}

impl fmt::Display for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.to_f64())
    }
}

// ── Operator impls ───────────────────────────────────────────────────────

impl Add for Fixed {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl AddAssign for Fixed {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_add(rhs.0);
    }
}

impl Sub for Fixed {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl SubAssign for Fixed {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_sub(rhs.0);
    }
}

impl Mul for Fixed {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        // Use i128 intermediates to avoid overflow during multiply-then-divide.
        Self(((self.0 as i128 * rhs.0 as i128) / SCALE as i128) as i64)
    }
}

impl Mul<i64> for Fixed {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: i64) -> Self {
        Self(self.0.wrapping_mul(rhs))
    }
}

impl Div for Fixed {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self {
        Self(((self.0 as i128 * SCALE as i128) / rhs.0 as i128) as i64)
    }
}

impl Div<i64> for Fixed {
    type Output = Self;
    #[inline]
    fn div(self, rhs: i64) -> Self {
        Self(self.0 / rhs)
    }
}

impl Rem for Fixed {
    type Output = Self;
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Self(self.0 % rhs.0)
    }
}

impl Neg for Fixed {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

// ── From conversions ─────────────────────────────────────────────────────

impl From<i32> for Fixed {
    fn from(v: i32) -> Self {
        Self::from_int(v as i64)
    }
}

impl From<i64> for Fixed {
    fn from(v: i64) -> Self {
        Self::from_int(v)
    }
}

impl From<f64> for Fixed {
    fn from(v: f64) -> Self {
        Self::from_f64(v)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_arithmetic() {
        let a = Fixed::from_f64(0.5);
        let b = Fixed::from_f64(0.25);
        assert_eq!(a + b, Fixed::from_f64(0.75));
        assert_eq!(a - b, Fixed::from_f64(0.25));
        assert_eq!(a * b, Fixed::from_f64(0.125));
        assert_eq!(a / b, Fixed::from_f64(2.0));
    }

    #[test]
    fn clamp_01() {
        let over = Fixed::from_f64(1.5);
        let under = Fixed::from_f64(-0.3);
        assert_eq!(over.clamp_01(), Fixed::ONE);
        assert_eq!(under.clamp_01(), Fixed::ZERO);
    }

    #[test]
    fn display() {
        let v = Fixed::from_f64(0.1234);
        assert_eq!(format!("{v}"), "0.1234");
    }

    #[test]
    fn lerp() {
        let a = Fixed::from_f64(0.0);
        let b = Fixed::from_f64(1.0);
        let t = Fixed::from_f64(0.25);
        let result = a.lerp(b, t);
        assert_eq!(result.to_f64(), 0.25);
    }
}
