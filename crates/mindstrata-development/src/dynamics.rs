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
        // Zero-at-zero identity: untouched fields never move.
        if pressure == 0.0 && self.intensity == 0.0 {
            return *self;
        }
        let headroom = (p.ceiling - self.intensity).max(0.0);
        let next = match metabolism {
            Metabolism::Addiction => {
                self.intensity + p.growth * headroom * pressure - p.decay * self.intensity
            }
            Metabolism::Allergy => {
                self.intensity + p.growth * headroom * (1.0 - pressure)
                    - p.decay * pressure * self.intensity
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
        let field = PathologyField::neutral();
        assert!(field.is_neutral());
        let stepped_field = field.step(
            [
                (Polarity::Dark, Metabolism::Addiction, 0.0),
                (Polarity::Dark, Metabolism::Allergy, 0.0),
                (Polarity::Golden, Metabolism::Addiction, 0.0),
                (Polarity::Golden, Metabolism::Allergy, 0.0),
            ],
            &OperatorParams::pending(),
        );
        assert_eq!(stepped_field, field);
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
