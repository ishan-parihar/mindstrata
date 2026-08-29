//! Village `CollectiveField` (STORY 10-11, AP3 03-substrate §2/WP-I).
//!
//! The collective holon mirrors the per-person `DevelopmentField` but over the
//! 8 collective lines (sourced from the vendored `LineId` registry per FR-020).
//! Reads are pure and deterministic; writes are inert in v1 (`step_collective`
//! returns the same `CollectiveField` it received when no catalysts are queued,
//! preserving the zero-at-zero identity law).
//!
//! Consumers (`pass_development`, sim-side `sim/collective.rs`) wire this in
//! behind the `--collectives` flag once the per-quadrant collective pressure
//! derivation lands; until then, this module is the typed scaffold.

use crate::dynamics::OperatorParams;
use crate::line::{all_lines, LineId, Scope};

/// The vendored collective line count (WP-A registry). The plan's "8 collective
/// lines" was a placeholder; the actual KosmOS registry carries N=28. The
/// holon is fixed-size once we know the count — change in WP-0A lockstep with
/// the registry bump.
pub const COLLECTIVE_LINE_COUNT: usize = 29;

fn collect_collective_slugs() -> Vec<LineId> {
    all_lines()
        .filter(|l| l.scope() == Scope::Collective)
        .collect()
}

/// A single collective line's state on the village holon.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CollectiveLineState {
    /// Shadow stage coordinate in [1..=17]; quantized view via `StageCoord::floor`.
    pub stage: f64,
    /// Transition accumulator toward the next stage.
    pub press: f64,
    /// EMA of stage-adequate signal — drives the transcend-and-include gate.
    pub fulfillment: f64,
}

impl CollectiveLineState {
    /// Neutral collective line (per AP3 03-substrate §2 — `lambda=1.0`, `press=0.0`).
    #[must_use]
    pub const fn neutral() -> Self {
        Self {
            stage: 1.0,
            press: 0.0,
            fulfillment: 0.0,
        }
    }

    /// Advance one tick under `pressure` in [0,1]. Identity-at-zero: zero
    /// pressure on a neutral line returns the line unchanged.
    #[must_use]
    pub fn step(&self, pressure: f64) -> Self {
        if pressure == 0.0 && self.press == 0.0 {
            return *self;
        }
        let p = pressure.clamp(0.0, 1.0);
        // Tiny press integration — full growth/ceiling canon lands with WP-I wiring.
        Self {
            stage: self.stage,
            press: (self.press + p * 0.05).clamp(0.0, 1.0),
            fulfillment: self.fulfillment,
        }
    }
}

/// Village holon state (8 collective lines + Λ frontier).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CollectiveField {
    /// One `CollectiveLineState` per collective line, in vendored order.
    pub lines: [CollectiveLineState; COLLECTIVE_LINE_COUNT],
    /// Λ window width (AP3 03-substrate §6 — starts `1.0`, widen only with probe).
    pub lambda: f64,
}

impl Default for CollectiveField {
    fn default() -> Self {
        Self::neutral()
    }
}

impl CollectiveField {
    /// Fully neutral village holon — the founder default.
    #[must_use]
    pub const fn neutral() -> Self {
        Self {
            lines: [CollectiveLineState::neutral(); COLLECTIVE_LINE_COUNT],
            lambda: 1.0,
        }
    }

    /// All-N collective line slugs the vendored registry references (FR-020).
    #[must_use]
    pub fn line_slugs() -> Vec<LineId> {
        collect_collective_slugs()
    }

    /// True when every collective line is at the founder neutral (used by
    /// `sim/collective.rs` to skip the step when no collective catalysts are
    /// queued).
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.lambda == 1.0
            && self
                .lines
                .iter()
                .all(|l| l.stage == 1.0 && l.press == 0.0 && l.fulfillment == 0.0)
    }

    /// Inert step: returns `self` unchanged. Full step lands with WP-I pressure
    /// derivation; the per-line `step` above is the shape.
    #[must_use]
    pub fn step_collective(
        &self,
        _pressures: &[f64; COLLECTIVE_LINE_COUNT],
        _params: &OperatorParams,
    ) -> Self {
        // ponytail: no pressure derivation yet (WP-I). Stay inert and
        // zero-at-zero so goldens stay byte-identical.
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_collective_is_identity() {
        let cf = CollectiveField::neutral();
        assert!(cf.is_neutral());
        let p = OperatorParams::pending();
        let stepped = cf.step_collective(&[0.0; COLLECTIVE_LINE_COUNT], &p);
        assert_eq!(stepped, cf);
    }

    #[test]
    fn line_slugs_match_vendored_registry() {
        let slugs = CollectiveField::line_slugs();
        assert_eq!(slugs.len(), COLLECTIVE_LINE_COUNT);
        for s in slugs {
            assert!(!s.slug().is_empty());
        }
    }

    #[test]
    fn single_line_step_is_identity_at_zero() {
        let l = CollectiveLineState::neutral();
        let stepped = l.step(0.0);
        assert_eq!(stepped, l);
    }

    #[test]
    fn single_line_step_accumulates_press() {
        let l = CollectiveLineState::neutral();
        let stepped = l.step(0.5);
        assert!(stepped.press > 0.0);
        assert!(stepped.press < 1.0);
    }
}
