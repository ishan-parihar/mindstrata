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
//!
//! **Declared-vs-operative divergence in that spec field** (`chronic_pain_accumulation_rate`):
//! the spec declares `0.001`; the operative neutral rate is
//! [`CHRONIC_PAIN_ACCUMULATION_RATE`] = `0.005`. The spec value cannot work at
//! this state's resolution — its product with a moderate `fracture_risk`
//! (0.05 × 0.001 = 5e-5) is below `Fixed`'s 1e-4 quantum and truncates to zero
//! (§5), so the accumulation would be permanently dead. The spec field is
//! **not** read by any code (the identifier appears only in comments), so the
//! divergence is currently inert; aligning the two is recorded as spec debt in
//! `docs/PLAN_DC5_DEVELOPMENT.md` rather than silently rescaling here.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Neutral chronic-pain accumulation rate — accumulated chronic pain per tick
/// per unit of fracture risk. This is the value the law hardcoded before i392
/// row 3; it is now the **gene-neutral** point of
/// [`chronic_pain_accumulation_rate`], so agents whose gene sits at the draw
/// midpoint reproduce it exactly.
///
/// Kept as f64 (not `Fixed`) because the gene multiplier is evaluated in f64 and
/// quantized once, per §5's sub-resolution rule.
pub const CHRONIC_PAIN_ACCUMULATION_RATE: f64 = 0.005;

/// The draw midpoint of `chronic_pain_risk` (`U(0.0, 0.5)` → 0.25) — the gene
/// value at which the multiplier is exactly 1.0 (§4.6 midpoint neutrality).
pub const CHRONIC_PAIN_GENE_MIDPOINT: f64 = 0.25;

/// How far the gene swings the accumulation rate. Sized to the response's
/// *shape*: chronic pain is a pure linear accumulator (no gate, no threshold),
/// so a ±40% rate change buys a ±40% state change — measured in
/// `i393_chronic_pain_gene` leg D (0.30 / 0.50 / 0.70 after 500 steps at
/// fracture_risk 0.2 for genes 0.0 / 0.25 / 0.5).
pub const CHRONIC_PAIN_GENE_SPAN: f64 = 1.6;

/// Map the heritable `health_predispositions.chronic_pain_risk` gene onto the
/// accumulation rate (i392 row 3).
///
/// The gene was dead from i391's census: drawn, defaulted, blended at `inherit`,
/// read by nothing. It now scales the law that its name describes.
///
/// Computed in f64 and quantized **once** (§5): a `Fixed`-only chain would
/// quantize the multiplier to 1e-4 before multiplying by the rate, discarding
/// most of the gene's resolution at low fracture risk.
#[must_use]
pub fn chronic_pain_accumulation_rate(gene: Fixed) -> Fixed {
    let multiplier = 1.0 + (gene.to_f64() - CHRONIC_PAIN_GENE_MIDPOINT) * CHRONIC_PAIN_GENE_SPAN;
    Fixed::from_f64(CHRONIC_PAIN_ACCUMULATION_RATE * multiplier)
}

/// Tunables for [`SkeletalState::tick_update`], grouped so the four `Fixed`
/// arguments cannot be transposed positionally.
#[derive(Debug, Clone, Copy)]
pub struct SkeletalUpdateParams {
    /// Accumulation rate of chronic pain per unit of fracture risk — the gene's
    /// consumer; see [`chronic_pain_accumulation_rate`].
    pub chronic_pain_accumulation_rate: Fixed,
}

impl Default for SkeletalUpdateParams {
    fn default() -> Self {
        Self {
            chronic_pain_accumulation_rate: Fixed::from_f64(CHRONIC_PAIN_ACCUMULATION_RATE),
        }
    }
}

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
    pub fn tick_update(
        &mut self,
        age: Fixed,
        injury: Fixed,
        nutrition_quality: Fixed,
        params: SkeletalUpdateParams,
    ) {
        // Elder frailty — structural integrity erodes past the onset age and
        // recovers to full integrity below it (calibrated runs: ages < 60).
        if age > Fixed::from_f64(60.0) {
            self.developmental_stage = SkeletalMaturity::Frail;
            let frailty = ((age - Fixed::from_f64(60.0)) * Fixed::from_f64(0.002)).clamp_01();
            self.structural_integrity = (Fixed::ONE - frailty).max(Fixed::from_f64(0.3));
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
            // The rate must stay above Fixed's 4-decimal resolution across the
            // intended risk range: with a 0.0005 rate, even fracture_risk 0.05 →
            // 0.000025/tick would round to zero; the neutral 0.005 keeps even
            // moderate risk (0.05 → 0.00025) visible. i392 row 3: the rate is now
            // the genotype's, passed in by the caller (`biology/mod.rs` feeds
            // `genome.health_predispositions.chronic_pain_risk`).
            self.chronic_pain = (self.chronic_pain
                + self.fracture_risk * params.chronic_pain_accumulation_rate)
                .clamp_01();
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
        assert_eq!(
            SkeletalState::from_age(Fixed::from_f64(30.0)).developmental_stage,
            SkeletalMaturity::Adult
        );
        assert_eq!(
            SkeletalState::from_age(Fixed::from_f64(8.0)).developmental_stage,
            SkeletalMaturity::Growing
        );
        assert_eq!(
            SkeletalState::from_age(Fixed::from_f64(15.0)).developmental_stage,
            SkeletalMaturity::Maturing
        );
        assert_eq!(
            SkeletalState::from_age(Fixed::from_f64(70.0)).developmental_stage,
            SkeletalMaturity::Frail
        );
    }

    #[test]
    fn elder_frailty_reduces_integrity_and_mobility() {
        let mut s = SkeletalState::from_age(Fixed::from_f64(65.0));
        s.tick_update(
            Fixed::from_f64(70.0),
            Fixed::ZERO,
            Fixed::from_f64(0.6),
            SkeletalUpdateParams::default(),
        );
        assert!(s.structural_integrity < Fixed::ONE);
        assert!(s.effective_mobility() < Fixed::ONE);
        assert_eq!(s.developmental_stage, SkeletalMaturity::Frail);
    }

    #[test]
    fn severe_injury_accumulates_fracture_risk_and_chronic_pain() {
        let mut s = SkeletalState::default();
        for _ in 0..50 {
            s.tick_update(
                Fixed::from_f64(40.0),
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.6),
                SkeletalUpdateParams::default(),
            );
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
            s.tick_update(
                Fixed::from_f64(40.0),
                Fixed::ZERO,
                Fixed::from_f64(0.1),
                SkeletalUpdateParams::default(),
            );
        }
        assert!(s.bone_density < Fixed::from_f64(0.7));
    }

    #[test]
    fn healthy_adult_ticks_stay_neutral() {
        // The calibrated-run invariant: a healthy adult under moderate
        // nutrition must remain exactly neutral tick after tick.
        let mut s = SkeletalState::from_age(Fixed::from_f64(40.0));
        for _ in 0..100 {
            s.tick_update(
                Fixed::from_f64(40.0),
                Fixed::ZERO,
                Fixed::from_f64(0.6),
                SkeletalUpdateParams::default(),
            );
        }
        assert_eq!(s.effective_mobility(), Fixed::ONE);
        assert_eq!(s.health_factor(), Fixed::ONE);
        assert_eq!(s.mobility_penalty, Fixed::ZERO);
    }

    /// i392 row 3 — the gene is *live*: same injury, different gene, different
    /// chronic pain. Measured in `i393_chronic_pain_gene` leg D as 0.30 / 0.50 /
    /// 0.70 at genes 0.0 / 0.25 / 0.5 (risk 0.2, 500 steps).
    #[test]
    fn chronic_pain_risk_gene_scales_accumulation() {
        let risk = Fixed::from_f64(0.2);
        let mut values = Vec::new();
        for gene in [0.0f64, 0.25, 0.5] {
            let mut s = SkeletalState {
                fracture_risk: risk,
                ..Default::default()
            };
            for _ in 0..500 {
                // injury below the 0.5 threshold leaves fracture_risk alone.
                s.tick_update(
                    Fixed::from_f64(40.0),
                    Fixed::ZERO,
                    Fixed::from_f64(0.6),
                    SkeletalUpdateParams {
                        chronic_pain_accumulation_rate: chronic_pain_accumulation_rate(
                            Fixed::from_f64(gene),
                        ),
                    },
                );
            }
            values.push(s.chronic_pain);
        }
        assert!(
            values[0] < values[1] && values[1] < values[2],
            "gene must order chronic pain accumulation: {values:?}"
        );
    }

    /// The mid-point identity the wiring rests on (§4.6): the draw's midpoint maps
    /// to exactly the rate the law hardcoded, so the population mean is neutral and
    /// the wiring is byte-identical wherever the band is closed.
    #[test]
    fn gene_midpoint_reproduces_the_retired_constant() {
        assert_eq!(
            chronic_pain_accumulation_rate(Fixed::from_f64(CHRONIC_PAIN_GENE_MIDPOINT)),
            Fixed::from_f64(CHRONIC_PAIN_ACCUMULATION_RATE)
        );
        assert_eq!(
            SkeletalUpdateParams::default().chronic_pain_accumulation_rate,
            Fixed::from_f64(CHRONIC_PAIN_ACCUMULATION_RATE)
        );
    }
}
