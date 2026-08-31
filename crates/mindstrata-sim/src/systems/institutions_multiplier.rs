//! Institutions read-side multiplier groundwork (SIM 4.25, Era IV).
//!
//! Inert plumbing only — no behavior, no tick wiring, golden-proven.
//! Follow-up for Era IV planning (AP3 WP-I): when `mindstrata-institutions`
//! exposes read-side signals (e.g., faction pressure, norm salience), this
//! module will map them to per-agent action multipliers via the same
//! `zero-at-zero` law as `development` (neutral inputs → identity).
//!
//! Today: one type + one neutral constructor + one identity pin.
//! Not called from `core.rs` tick order; `cargo test` proves it compiles
//! and `golden_replay 5/5` stays byte-identical.

use mindstrata_core::fixed::Fixed;

/// Inert institutions multiplier — neutral at `1.0` (identity), `0.0` delta.
///
/// Shape mirrors `development` stage gating: `neutral()` is the zero-at-zero
/// anchor; future `apply` will be `value * multiplier` with `multiplier == 1.0`
/// at neutral institutions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstitutionsMultiplier {
    /// Work multiplier (neutral `1.0`).
    pub work: Fixed,
    /// Rest multiplier (neutral `1.0`).
    pub rest: Fixed,
}

impl InstitutionsMultiplier {
    /// Neutral — identity at zero institutions signal.
    pub fn neutral() -> Self {
        Self {
            work: Fixed::ONE,
            rest: Fixed::ONE,
        }
    }

    /// True when both axes are identity (no institutions pressure).
    pub fn is_neutral(self) -> bool {
        self.work == Fixed::ONE && self.rest == Fixed::ONE
    }
}

impl Default for InstitutionsMultiplier {
    fn default() -> Self {
        Self::neutral()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_is_identity() {
        let m = InstitutionsMultiplier::neutral();
        assert!(m.is_neutral());
        assert_eq!(m.work, Fixed::ONE);
        assert_eq!(m.rest, Fixed::ONE);
    }

    #[test]
    fn default_is_neutral() {
        assert!(InstitutionsMultiplier::default().is_neutral());
    }
}
