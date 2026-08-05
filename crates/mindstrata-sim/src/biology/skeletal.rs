//! Skeletal system — structural body capacity, injury, aging, physical limitation.
//!
//! AP2 §7.2.3: elders become respected but physically limited; injured workers
//! become dependent; child malnutrition produces lifelong weakness; combat
//! injuries create chronic pain. The system is designed to be **exactly
//! neutral at baseline** (structural_integrity = 1.0, chronic_pain = 0,
//! fracture_risk = 0, mobility_penalty = 0), so calibrated runs are untouched:
//! the penalties only move under elder frailty (age > 60), severe injury, or
//! malnutrition — none of which occur in the calibrated horizons.
//!
//! Alignment: `specs/biology/organs.ron` §skeletal declares the tunable
//! parameters (base_frame_size, base_bone_density, adult_peak_age,
//! elder_frailty_onset_age, malnutrition_bone_density_penalty,
//! fracture_risk_from_fall, chronic_pain_accumulation_rate).

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Skeletal maturity stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkeletalMaturity {
    /// Childhood growth phase (bone density still rising).
    Growing,
    /// Adolescent maturation toward adult peak.
    Maturing,
    /// Adult peak structural capacity.
    Adult,
    /// Elder frailty — integrity and density declining.
    Frail,
}

/// Skeletal state (§7.2.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkeletalState {
    /// Frame size (0 = small, 1 = large). Individual variation, not a penalty.
    pub frame_size: Fixed,
    /// Bone density (0 = brittle, 1 = peak). Eroded by malnutrition and age.
    pub bone_density: Fixed,
    /// Structural integrity (0 = compromised, 1 = intact). Declines with elder
    /// frailty and severe fractures. **Starts at exactly 1.0.**
    pub structural_integrity: Fixed,
    /// Fracture risk from severe injury (0 = none, 1 = high).
    pub fracture_risk: Fixed,
    /// Chronic pain accumulated from unresolved fractures (0 = none, 1 = severe).
    pub chronic_pain: Fixed,
    /// Mobility penalty derived from frailty + chronic pain (0 = none, 1 = immobile).
    pub mobility_penalty: Fixed,
    /// Current skeletal maturity stage.
    pub developmental_stage: SkeletalMaturity,
}

impl Default for SkeletalState {
    fn default() -> Self {
        Self {
            frame_size: Fixed::from_f64(0.5),
            bone_density: Fixed::from_f64(0.7),
            // Neutral values — the multipliers below must be exactly 1.0 / 0.0
            // at baseline so calibrated runs carry zero drift.
            structural_integrity: Fixed::ONE,
            fracture_risk: Fixed::ZERO,
            chronic_pain: Fixed::ZERO,
            mobility_penalty: Fixed::ZERO,
            developmental_stage: SkeletalMaturity::Adult,
        }
    }
}

impl SkeletalState {
    /// Initialize skeletal state from age (adult peak for the founding population).
    pub fn from_age(age: Fixed) -> Self {
        let mut skeletal = Self::default();
        // Founding agents are 18–55: all at or past adult peak. Bone density
        // ramps through childhood/adolescence (Growing → Maturing) and holds
        // at adult peak until elder frailty onset.
        let stage = if age < Fixed::from_f64(12.0) {
            SkeletalMaturity::Growing
        } else if age < Fixed::from_f64(20.0) {
            SkeletalMaturity::Maturing
        } else if age > Fixed::from_f64(60.0) {
            SkeletalMaturity::Frail
        } else {
            SkeletalMaturity::Adult
        };
        skeletal.developmental_stage = stage;
        if stage == SkeletalMaturity::Growing {
            skeletal.bone_density = Fixed::from_f64(0.5);
        } else if stage == SkeletalMaturity::Maturing {
            skeletal.bone_density = Fixed::from_f64(0.6);
        } else if stage == SkeletalMaturity::Frail {
            // Elder frailty — integrity and density below adult peak.
            let frailty = ((age - Fixed::from_f64(60.0)) * Fixed::from_f64(0.002)).clamp_01();
            skeletal.structural_integrity = (Fixed::ONE - frailty).max(Fixed::from_f64(0.3));
            skeletal.bone_density =
                (Fixed::from_f64(0.7) - frailty * Fixed::from_f64(0.4)).max(Fixed::from_f64(0.2));
        }
        skeletal
    }

    /// Per-tick update: elder frailty, severe-injury fracture risk, and
    /// malnutrition bone loss. All penalties start at zero and only move under
    /// conditions absent from calibrated runs.
    pub fn tick_update(&mut self, age: Fixed, injury: Fixed, nutrition_quality: Fixed) {
        // Elder frailty — structural integrity erodes past the onset age and
        // recovers to full integrity below it (calibrated runs: ages < 60).
        if age > Fixed::from_f64(60.0) {
            self.developmental_stage = SkeletalMaturity::Frail;
            let frailty = ((age - Fixed::from_f64(60.0)) * Fixed::from_f64(0.002)).clamp_01();
            self.structural_integrity =
                (Fixed::ONE - frailty).max(Fixed::from_f64(0.3));
        } else {
            if self.developmental_stage == SkeletalMaturity::Frail {
                // (Only reachable if an agent were re-aged down; defensive.)
                self.developmental_stage = SkeletalMaturity::Adult;
            }
            self.structural_integrity = Fixed::ONE;
        }

        // Severe injuries raise fracture risk; fracture risk begets chronic pain.
        if injury > Fixed::from_f64(0.5) {
            self.fracture_risk = (self.fracture_risk
                + (injury - Fixed::from_f64(0.5)) * Fixed::from_f64(0.01))
                .clamp_01();
        } else {
            self.fracture_risk = (self.fracture_risk - Fixed::from_f64(0.0005)).max(Fixed::ZERO);
        }
        if self.fracture_risk > Fixed::ZERO {
            // chronic_pain_accumulation_rate from organs.ron. Must stay above
            // Fixed's 4-decimal resolution across the intended risk range: with
            // a 0.0005 rate, even fracture_risk 0.05 → 0.000025/tick would round
            // to zero; 0.005 keeps even moderate risk (0.05 → 0.00025) visible.
            self.chronic_pain =
                (self.chronic_pain + self.fracture_risk * Fixed::from_f64(0.005)).clamp_01();
        } else {
            self.chronic_pain = (self.chronic_pain - Fixed::from_f64(0.0005)).max(Fixed::ZERO);
        }

        // Malnutrition erodes bone density; feeding remineralizes toward adult peak.
        if nutrition_quality < Fixed::from_f64(0.3) {
            let loss = (Fixed::from_f64(0.3) - nutrition_quality) * Fixed::from_f64(0.002);
            self.bone_density = (self.bone_density - loss).clamp_01();
        } else if self.bone_density < Fixed::from_f64(0.7) {
            self.bone_density =
                (self.bone_density + Fixed::from_f64(0.0005)).min(Fixed::from_f64(0.7));
        }

        // Mobility penalty: frailty + chronic pain. Exactly 0.0 at baseline.
        self.mobility_penalty = ((Fixed::ONE - self.structural_integrity) * Fixed::from_f64(0.5)
            + self.chronic_pain * Fixed::from_f64(0.3))
            .clamp_01();
    }

    /// Mobility multiplier — exactly 1.0 at baseline; falls under frailty,
    /// unresolved fractures, or chronic pain (floored so a frail elder can
    /// still drag themselves to the field).
    #[must_use]
    pub fn effective_mobility(&self) -> Fixed {
        (Fixed::ONE - self.mobility_penalty).max(Fixed::from_f64(0.2))
    }

    /// Health multiplier — exactly 1.0 at baseline; integrity loss and chronic
    /// pain drain health (floored at 0.3 for survivability).
    #[must_use]
    pub fn health_factor(&self) -> Fixed {
        (self.structural_integrity - self.chronic_pain * Fixed::from_f64(0.2))
            .clamp(Fixed::from_f64(0.3), Fixed::ONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_is_exactly_neutral() {
        let s = SkeletalState::default();
        assert_eq!(s.effective_mobility(), Fixed::ONE);
        assert_eq!(s.health_factor(), Fixed::ONE);
        assert_eq!(s.mobility_penalty, Fixed::ZERO);
        assert_eq!(s.chronic_pain, Fixed::ZERO);
        assert_eq!(s.fracture_risk, Fixed::ZERO);
    }

    #[test]
    fn adult_stage_from_age() {
        assert_eq!(SkeletalState::from_age(Fixed::from_f64(30.0)).developmental_stage, SkeletalMaturity::Adult);
        assert_eq!(SkeletalState::from_age(Fixed::from_f64(8.0)).developmental_stage, SkeletalMaturity::Growing);
        assert_eq!(SkeletalState::from_age(Fixed::from_f64(15.0)).developmental_stage, SkeletalMaturity::Maturing);
        assert_eq!(SkeletalState::from_age(Fixed::from_f64(70.0)).developmental_stage, SkeletalMaturity::Frail);
    }

    #[test]
    fn elder_frailty_reduces_integrity_and_mobility() {
        let mut s = SkeletalState::from_age(Fixed::from_f64(65.0));
        s.tick_update(Fixed::from_f64(70.0), Fixed::ZERO, Fixed::from_f64(0.6));
        assert!(s.structural_integrity < Fixed::ONE);
        assert!(s.effective_mobility() < Fixed::ONE);
        assert_eq!(s.developmental_stage, SkeletalMaturity::Frail);
    }

    #[test]
    fn severe_injury_accumulates_fracture_risk_and_chronic_pain() {
        let mut s = SkeletalState::default();
        for _ in 0..50 {
            s.tick_update(Fixed::from_f64(40.0), Fixed::from_f64(0.8), Fixed::from_f64(0.6));
        }
        assert!(s.fracture_risk > Fixed::ZERO);
        assert!(s.chronic_pain > Fixed::ZERO);
        assert!(s.effective_mobility() < Fixed::ONE);
        assert!(s.health_factor() < Fixed::ONE);
    }

    #[test]
    fn malnutrition_reduces_bone_density() {
        let mut s = SkeletalState::default();
        for _ in 0..100 {
            s.tick_update(Fixed::from_f64(40.0), Fixed::ZERO, Fixed::from_f64(0.1));
        }
        assert!(s.bone_density < Fixed::from_f64(0.7));
    }

    #[test]
    fn healthy_adult_ticks_stay_neutral() {
        // The calibrated-run invariant: a healthy adult under moderate
        // nutrition must remain exactly neutral tick after tick.
        let mut s = SkeletalState::from_age(Fixed::from_f64(40.0));
        for _ in 0..100 {
            s.tick_update(Fixed::from_f64(40.0), Fixed::ZERO, Fixed::from_f64(0.6));
        }
        assert_eq!(s.effective_mobility(), Fixed::ONE);
        assert_eq!(s.health_factor(), Fixed::ONE);
        assert_eq!(s.mobility_penalty, Fixed::ZERO);
    }
}
