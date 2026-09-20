//! Field dynamics (WP-C): resonance weighting and the ratified 4-fold
//! pathology operator as pure deterministic functions.
//!
//! Identity-at-neutral law (FR-023/FR-012): zero catalyst pressure leaves a
//! neutral field exactly unchanged — consumers pass zero-at-zero gates so
//! goldens stay byte-identical until a value actually moves.
//!
//! All curve parameters are canon constants marked CALIBRATION-PENDING(AP3)
//! in [`crate::canon`]; this module defines the OPERATOR SHAPE only.

use crate::line::LineId;

/// Resonance affinity of one catalyst for one field reading.
///
/// Same-line catalysts resonate at full magnitude; cross-line resonance is
/// deliberately ZERO in v1 — inventing inter-line affinities without vault
/// data would be fabrication, and the extension point is documented.
#[must_use]
pub fn resonance_weight(catalyst_line: LineId, reading_line: LineId, magnitude: f64) -> f64 {
    if catalyst_line == reading_line {
        magnitude.clamp(0.0, 1.0)
    } else {
        // CALIBRATION-PENDING(AP3): cross-line affinity matrix awaits vendor
        // coupling data (probe plan i<iter>_cross_line_resonance).
        0.0
    }
}

/// Growth scale of the Allergy absence-accumulation term (the always-step
/// law in `development.rs`): Allergy grows at `growth × this × headroom` on a
/// tick with no pressure.
///
/// CALIBRATION-PENDING(AP3): 0.1× the pending growth keeps a 5 000-tick
/// window near ~0.4 mean rather than saturating in 20 ticks (`pathology-
/// curves.md` Q4 0.02–0.04 vs Q1 0.04–0.08).
pub const ALLERGY_ABSENCE_SCALE: f64 = 0.1;

/// Resting relaxation of an Allergy quadrant, as a fraction of the quadrant's
/// `decay`, applied EVERY tick regardless of pressure.
///
/// Iteration 318 (difficulty-levers row 3 residual). Before this, pressure was
/// the ONLY decay channel on an Allergy quadrant (`− decay·pressure·I`), so a
/// tick with no pressure was pure monotone growth toward `ceiling`: the
/// long-horizon equilibrium was set by the horizon, not by the agent's
/// reconciliation diet, and the quadrant lost dynamic range (i318 probe: 0% of
/// Q2 agents within 5% of the ceiling at 20K, 50% at 50K, **64% at 100K** —
/// the lever's own catalog horizon). Tying the relaxation to `decay` keeps it
/// on the row-3 difficulty band (a brittle village relaxes slower) without
/// adding an `OperatorParams` field.
///
/// Equilibrium under pure absence: `growth·SCALE·(ceiling−I) = decay·RELAX·I`,
/// i.e. `I* = growth·SCALE·ceiling / (growth·SCALE + decay·RELAX)` — strictly
/// below `ceiling` for every quadrant, giving the ceiling/centre band residual
/// headroom at every horizon.
///
/// CALIBRATION-PENDING(AP3).
pub const ALLERGY_RESTING_RELAXATION: f64 = 0.05;

/// Pathology polarity axis of the ratified operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Polarity {
    /// Dark pole — contraction under deficit/threat.
    Dark,
    /// Golden pole — distortion of genuine opening.
    Golden,
}

/// Metabolism axis: how the polarity processes exposure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Metabolism {
    /// Addiction — intensity GROWS with repeated exposure.
    Addiction,
    /// Allergy — intensity grows with avoidance/recoil patterns; resolves
    /// through sustained golden-path engagement (Agape side).
    Allergy,
}

/// One quadrant's curve parameters (all CALIBRATION-PENDING(AP3)).
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct OperatorParams {
    /// Per-tick growth fraction applied to remaining headroom.
    pub growth: f64,
    /// Per-tick decay fraction applied to current intensity.
    pub decay: f64,
    /// Intensity ceiling before saturation clamps.
    pub ceiling: f64,
}

impl OperatorParams {
    /// Neutral placeholder set — compiles inert, moves nothing at zero input.
    #[must_use]
    pub const fn pending() -> Self {
        Self {
            growth: 0.05,
            decay: 0.02,
            ceiling: 1.0,
        }
    }
}

/// One quadrant of a person's pathology state.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct QuadrantState {
    /// Current intensity in [0, ceiling].
    pub intensity: f64,
}

impl QuadrantState {
    /// Neutral state.
    #[must_use]
    pub const fn neutral() -> Self {
        Self { intensity: 0.0 }
    }

    /// Advance one tick under `pressure` in [0,1] (the resonated exposure).
    ///
    /// Zero pressure AND zero intensity is the identity point: returns the
    /// state unchanged bit-for-bit (zero-at-zero).
    /// Addiction: pressure feeds growth toward the ceiling; absence decays.
    /// Allergy: pressure suppresses (Agape resolution); its ABSENCE lets
    /// intensity grow toward the ceiling (recoil accumulation).
    #[must_use]
    pub fn step(&self, metabolism: Metabolism, pressure: f64, p: &OperatorParams) -> Self {
        let pressure = pressure.clamp(0.0, 1.0);
        // Zero-at-zero identity for Addiction only — Addiction needs
        // pressure to leave neutral.  Allergy uses `1−pressure` so
        // absence (pressure 0) must still accumulate from neutral;
        // the early return would pin Q2/Q4 at 0.0000 forever (i293
        // 20-seed sweep, i294 N=48/20K).  See development.rs
        // absence-driven growth for the per-tick stepping.
        if metabolism == Metabolism::Addiction && pressure == 0.0 && self.intensity == 0.0 {
            return *self;
        }
        let headroom = (p.ceiling - self.intensity).max(0.0);
        let next = match metabolism {
            Metabolism::Addiction => {
                self.intensity + p.growth * headroom * pressure - p.decay * self.intensity
            }
            Metabolism::Allergy => {
                // Ponytail: Allergy growth is slower than Addiction —
                // pathology-curves.md Q4 0.02–0.04 vs Q1 0.04–0.08, and
                // the always-step absence-driven accumulation in
                // development.rs would saturate in 20 ticks at 0.05.
                // Use 0.1× the pending growth for Allergy so the
                // 5000-tick horizon shows ~0.4 mean not 1.0, preserving
                // dynamic range for differentiation.  CALIBRATION-PENDING.
                //
                // i318 resting relaxation: bounds the absence attractor
                // strictly below `ceiling` (see ALLERGY_RESTING_RELAXATION)
                // so the quadrant keeps headroom at the 100K lever horizon
                // instead of piling on its cap.
                self.intensity + p.growth * ALLERGY_ABSENCE_SCALE * headroom * (1.0 - pressure)
                    - p.decay * pressure * self.intensity
                    - p.decay * ALLERGY_RESTING_RELAXATION * self.intensity
            }
        };
        Self {
            intensity: next.clamp(0.0, p.ceiling),
        }
    }
}

/// The full 4-fold pathology state for one person.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PathologyField {
    /// Dark-addiction quadrant.
    pub dark_addiction: QuadrantState,
    /// Dark-allergy quadrant.
    pub dark_allergy: QuadrantState,
    /// Golden-addiction quadrant.
    pub golden_addiction: QuadrantState,
    /// Golden-allergy quadrant.
    pub golden_allergy: QuadrantState,
}

impl Default for PathologyField {
    fn default() -> Self {
        Self::neutral()
    }
}

impl PathologyField {
    /// Fully neutral field — the founder default (FR-023).
    #[must_use]
    pub const fn neutral() -> Self {
        Self {
            dark_addiction: QuadrantState::neutral(),
            dark_allergy: QuadrantState::neutral(),
            golden_addiction: QuadrantState::neutral(),
            golden_allergy: QuadrantState::neutral(),
        }
    }

    /// True when every quadrant sits at exact zero (the golden-untouched
    /// condition downstream gates consume).
    #[must_use]
    pub const fn is_neutral(&self) -> bool {
        self.dark_addiction.intensity == 0.0
            && self.dark_allergy.intensity == 0.0
            && self.golden_addiction.intensity == 0.0
            && self.golden_allergy.intensity == 0.0
    }

    /// Advance all quadrants one tick under their respective pressures.
    #[must_use]
    pub fn step(&self, pressures: [(Polarity, Metabolism, f64); 4], p: &OperatorParams) -> Self {
        let mut out = *self;
        for (pol, met, pressure) in pressures {
            let slot = match (pol, met) {
                (Polarity::Dark, Metabolism::Addiction) => &mut out.dark_addiction,
                (Polarity::Dark, Metabolism::Allergy) => &mut out.dark_allergy,
                (Polarity::Golden, Metabolism::Addiction) => &mut out.golden_addiction,
                (Polarity::Golden, Metabolism::Allergy) => &mut out.golden_allergy,
            };
            *slot = slot.step(met, pressure, p);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lid(slug: &'static str) -> LineId {
        LineId::new(slug).expect("registered")
    }

    #[test]
    fn same_line_resonance_carries_magnitude() {
        let w = resonance_weight(lid("cognitive"), lid("cognitive"), 0.7);
        assert!((w - 0.7).abs() < 1e-12);
    }

    #[test]
    fn cross_line_resonance_is_zero_until_calibrated() {
        assert_eq!(resonance_weight(lid("cognitive"), lid("values"), 0.9), 0.0);
    }

    #[test]
    fn resonance_clamps_out_of_range_magnitude() {
        assert_eq!(
            resonance_weight(lid("cognitive"), lid("cognitive"), 5.0),
            1.0
        );
    }

    #[test]
    fn zero_pressure_zero_intensity_is_bit_identity() {
        let q = QuadrantState::neutral();
        let stepped = q.step(Metabolism::Addiction, 0.0, &OperatorParams::pending());
        assert_eq!(stepped.intensity, q.intensity);
    }

    /// RE-CONTRACT per AGENTS.md §4.4 (Iter-266 audit finding): the old bulk
    /// pin asserted `PathologyField::step` at all-zero pressures returns the
    /// field bit-identical. Commit 34e4ca7 (Allergy Q2/Q4 always-step) made
    /// absence-driven growth the LAW — `QuadrantState::step` for Allergy
    /// deliberately grows from neutral under zero pressure (0.05 × 0.1 ×
    /// headroom = 0.005/tick), because the alternative (early return) pinned
    /// Q2/Q4 at 0.0000 forever (i293 20-seed, i294 N=48/20K). The old
    /// assertion therefore tests something the always-step semantics
    /// legitimately invalidate — this is a re-contract, not a magnitude
    /// re-pin. The REAL invariants, guarded below: (1) Addiction quadrants
    /// are bit-identical at zero pressure (the zero-at-zero law for Q1/Q3);
    /// (2) Allergy quadrants move by EXACTLY the documented absence rate
    /// (deterministic, monotone, bounded by the ceiling); (3) the field's
    /// `is_neutral()` still marks the founder default that golden windows
    /// relied on (observability, not dynamics). The calm-world byte-identity
    /// contract was never carried by this unit pin — it is enforced by the
    /// fact that the person-level pass (`system_development`) only steps
    /// Allergy quadrants on real-catalyst ticks (triggered_q2/triggered_q4
    /// guards), so golden windows with zero catalysts stay untouched.
    #[test]
    fn pathology_step_respects_addiction_zero_at_zero_and_allergy_absence_law() {
        let p = OperatorParams::pending();
        let field = PathologyField::neutral();
        assert!(field.is_neutral());
        let stepped_field = field.step(
            [
                (Polarity::Dark, Metabolism::Addiction, 0.0),
                (Polarity::Dark, Metabolism::Allergy, 0.0),
                (Polarity::Golden, Metabolism::Addiction, 0.0),
                (Polarity::Golden, Metabolism::Allergy, 0.0),
            ],
            &p,
        );
        // (1) Addiction zero-at-zero: Q1/Q3 bit-identical under zero pressure.
        assert_eq!(stepped_field.dark_addiction.intensity, 0.0);
        assert_eq!(stepped_field.golden_addiction.intensity, 0.0);
        // (2) Allergy absence law: exact documented increment per untouched
        // tick — 0.05 growth × 0.1 Allergy scale × headroom (1.0) = 0.005.
        let expected = 0.05 * 0.1;
        assert_eq!(stepped_field.dark_allergy.intensity, expected);
        assert_eq!(stepped_field.golden_allergy.intensity, expected);
        // Monotone + bounded: repeated absence ticks accumulate toward the
        // ceiling without exceeding it.
        let mut q = QuadrantState::neutral();
        for _ in 0..500 {
            q = q.step(Metabolism::Allergy, 0.0, &p);
        }
        assert!(q.intensity > 0.5, "absence must accumulate Allergy");
        assert!(q.intensity <= p.ceiling, "ceiling clamp holds");
    }

    #[test]
    fn addiction_grows_under_pressure_and_decays_without() {
        let p = OperatorParams::pending();
        let mut q = QuadrantState::neutral();
        for _ in 0..50 {
            q = q.step(Metabolism::Addiction, 1.0, &p);
        }
        assert!(q.intensity > 0.5, "sustained pressure must grow addiction");
        for _ in 0..300 {
            q = q.step(Metabolism::Addiction, 0.0, &p);
        }
        assert!(q.intensity < 0.01, "pressure removal must decay addiction");
    }

    #[test]
    fn allergy_resolves_through_engagement_and_grows_in_absence() {
        let p = OperatorParams::pending();
        let start = QuadrantState { intensity: 0.2 };
        let engaged = start.step(Metabolism::Allergy, 1.0, &p);
        assert!(
            engaged.intensity < start.intensity,
            "engagement resolves allergy"
        );
        let avoided = start.step(Metabolism::Allergy, 0.0, &p);
        assert!(
            avoided.intensity > start.intensity,
            "avoidance accumulates allergy"
        );
    }

    /// i318 (difficulty-levers row 3 residual): before the resting relaxation,
    /// pressure was the ONLY decay channel on an Allergy quadrant, so a tick
    /// with no pressure was monotone growth to `ceiling` and the long-horizon
    /// equilibrium was the horizon, not the agent's reconciliation diet — the
    /// i318 probe measured 0% of Q2 agents within 5% of the ceiling at 20K but
    /// 64% at 100K. The attraction point must now sit strictly below the
    /// ceiling at `I* = growth·SCALE·ceiling / (growth·SCALE + decay·RELAX)`.
    #[test]
    fn allergy_absence_attractor_stays_below_ceiling() {
        let p = OperatorParams::pending();
        let g_eff = p.growth * ALLERGY_ABSENCE_SCALE;
        let relax = p.decay * ALLERGY_RESTING_RELAXATION;
        let predicted = g_eff * p.ceiling / (g_eff + relax);
        let mut q = QuadrantState::neutral();
        for _ in 0..20_000 {
            q = q.step(Metabolism::Allergy, 0.0, &p);
        }
        assert!(
            (q.intensity - predicted).abs() < 1e-3,
            "absence equilibrium {:.4} must match the leak-balance prediction {:.4}",
            q.intensity,
            predicted
        );
        assert!(
            q.intensity < p.ceiling - 0.1,
            "absence must leave headroom below the ceiling: {:.4} vs ceiling {:.4}",
            q.intensity,
            p.ceiling
        );
    }

    #[test]
    fn intensity_never_exceeds_ceiling() {
        let p = OperatorParams {
            ceiling: 0.6,
            ..OperatorParams::pending()
        };
        let mut q = QuadrantState::neutral();
        for _ in 0..500 {
            q = q.step(Metabolism::Addiction, 1.0, &p);
        }
        assert!(q.intensity <= 0.6 + 1e-12);
    }
}
