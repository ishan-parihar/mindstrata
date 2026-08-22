//! Sexual / Reproductive system — puberty, fertility, pair-bonding, pregnancy, birth.
//!
//! Handled abstractly, age-gated, and non-explicit. Reproduction is probabilistic;
//! pregnancy is a biological and social event.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Biological sex (from genome).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiologicalSex {
    Male,
    Female,
}

/// Puberty stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PubertyStage {
    Prepubescent,
    Early,
    Mid,
    Late,
    Complete,
}

/// Tunable parameters for reproductive system update.
///
/// Grouped into a `Copy` struct to avoid transposition-prone positional args
/// (Apollo Rust best practices Ch. 1: prefer structured data over positional
/// args of the same type).
#[must_use]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReproductiveUpdateParams {
    /// Stress suppression of fertility (0 = no effect, 1 = infertile under stress). Range: 0.0–1.0, default 0.3.
    pub stress_suppression: Fixed,
    /// Age-based fertility decline rate per year past 35. Range: 0.01–0.1, default 0.03.
    pub age_decline_rate: Fixed,
    /// Gestation rate multiplier (higher = faster pregnancy). Range: 0.5–2.0, default 1.0.
    pub gestation_rate_mult: Fixed,
}

impl Default for ReproductiveUpdateParams {
    fn default() -> Self {
        Self {
            stress_suppression: Fixed::from_f64(0.3),
            age_decline_rate: Fixed::from_f64(0.03),
            gestation_rate_mult: Fixed::from_f64(1.0),
        }
    }
}

/// §7.2.6: Gestation stage derived from progress — Early < 0.33, Mid < 0.66,
/// Late < 1.0, FullTerm at 1.0 (birth due).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GestationStage {
    Early,
    Mid,
    Late,
    FullTerm,
}

/// §7.2.6: First-class pregnancy state — the plan's `Option<PregnancyState>`
/// inside `ReproductiveState`. Iteration 42 upgraded the flat
/// `pregnant: bool + pregnancy_progress: Fixed` pair into this struct.
///
/// The biological pregnancy lifecycle (conception → gestation → birth) is
/// fully wired (Iteration 92): `attempt_conception` → `PregnancyState` →
/// gestation tick → `complete_pregnancy` → birth + kinship mirror. The
/// probabilistic demography path (`should_birth`) provides a separate,
/// non-pregnancy birth channel for background fertility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PregnancyState {
    /// Gestation progress (0 = conception, 1 = birth) — maps 1:1 from the old
    /// `pregnancy_progress` field.
    pub gestation_progress: Fixed,
    /// Derived gestation stage.
    pub gestation_stage: GestationStage,
    /// Maternal strain from pregnancy (0..1) — observational, tracks gestation.
    pub maternal_strain: Fixed,
    /// Complication risk (0..1) — observational; rises with maternal age and
    /// poor health (reproduction.ron: health_risk_base + elder risk).
    pub complications_risk: Fixed,
    /// Tick when conception occurred.
    pub conception_tick: u64,
}

impl PregnancyState {
    /// Start a pregnancy at conception.
    #[must_use]
    pub fn new(conception_tick: u64) -> Self {
        Self {
            gestation_progress: Fixed::ZERO,
            gestation_stage: GestationStage::Early,
            maternal_strain: Fixed::ZERO,
            complications_risk: Fixed::from_f64(0.05),
            conception_tick,
        }
    }
}

/// Reproductive state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproductiveState {
    /// Biological sex.
    pub sex: BiologicalSex,
    /// Sexual maturity (0 = child, 1 = fully mature).
    pub sexual_maturity: Fixed,
    /// Current puberty stage.
    pub puberty_stage: PubertyStage,
    /// Current fertility (0 = infertile, 1 = peak fertility).
    pub fertility: Fixed,
    /// Libido level (0 = none, 1 = high).
    pub libido: Fixed,
    /// Pair-bond strength with current partner (0 = none, 1 = deep bond).
    pub pair_bond_strength: Fixed,
    /// §7.2.6: Current pregnancy, if any. Set by `attempt_conception` (Iter 92);
    /// `None` when not pregnant.
    #[serde(default)]
    pub pregnancy: Option<PregnancyState>,
    /// Number of children born.
    pub children_born: u32,
    /// Parental drive (0 = none, 1 = strong desire for children).
    pub parental_drive: Fixed,
}

impl Default for ReproductiveState {
    fn default() -> Self {
        Self {
            sex: BiologicalSex::Male,
            sexual_maturity: Fixed::ZERO,
            puberty_stage: PubertyStage::Prepubescent,
            fertility: Fixed::ZERO,
            libido: Fixed::ZERO,
            pair_bond_strength: Fixed::ZERO,
            pregnancy: None,
            children_born: 0,
            parental_drive: Fixed::from_f64(0.3),
        }
    }
}

impl ReproductiveState {
    /// Update reproductive state based on age and hormonal signals.
    ///
    /// Uses `ReproductiveUpdateParams` to group tunable parameters,
    /// preventing transposition errors with 8 positional args of the same type.
    pub fn tick_update(
        &mut self,
        age_years: Fixed,
        health: Fixed,
        stress_level: Fixed,
        bonding_axis: Fixed,
        nutrition: Fixed,
        params: ReproductiveUpdateParams,
    ) {
        // Puberty progression based on age
        let puberty_age = Fixed::from_f64(13.0);
        let maturity_age = Fixed::from_f64(18.0);

        if age_years < puberty_age {
            self.puberty_stage = PubertyStage::Prepubescent;
            self.sexual_maturity = Fixed::ZERO;
            self.fertility = Fixed::ZERO;
            self.libido = Fixed::ZERO;
        } else if age_years < maturity_age {
            // Progress through puberty stages
            let progress = (age_years - puberty_age) / (maturity_age - puberty_age);
            self.sexual_maturity = progress.clamp_01();
            self.puberty_stage = if progress < Fixed::from_f64(0.33) {
                PubertyStage::Early
            } else if progress < Fixed::from_f64(0.66) {
                PubertyStage::Mid
            } else {
                PubertyStage::Late
            };
            self.fertility = progress * Fixed::from_f64(0.5); // partial fertility
            self.libido = progress * Fixed::from_f64(0.4);
        } else {
            self.sexual_maturity = Fixed::ONE;
            self.puberty_stage = PubertyStage::Complete;
            // Fertility peaks in young adulthood, declines with age
            let age_factor = if age_years < Fixed::from_f64(35.0) {
                Fixed::ONE
            } else {
                let decline = (age_years - Fixed::from_f64(35.0)) * params.age_decline_rate;
                (Fixed::ONE - decline).max(Fixed::from_f64(0.1))
            };
            self.fertility =
                age_factor * health * (Fixed::ONE - stress_level * params.stress_suppression);
            self.libido = (Fixed::from_f64(0.5) + bonding_axis * Fixed::from_f64(0.3)
                - stress_level * Fixed::from_f64(0.2))
            .clamp_01();
        }

        // Pregnancy progression (§7.2.6) — identical gestation dynamics to the
        // pre-Iter-42 flat `pregnancy_progress`, now inside `Option<PregnancyState>`.
        //
        // Iteration 242 (Fixed-truncation fix, audit finding E4): the rate
        // product computed in Fixed truncates to ZERO whenever
        // `0.001 × health × nutrition < 5e-5` — i.e. at the 0.1 world-nutrition
        // floor with any plausible health, every tick (probe: two pregnancies
        // frozen at exactly 0.0 progress through thousands of ticks on seed
        // 51; a third crawled at ~1e-6/tick on sporadic quantization luck).
        // Pregnancies were conceived but could never reach term in depleted
        // worlds. The increment is computed in f64 (the `should_birth` /
        // nervous-recovery / thermal-239 precedent for sub-resolution rates)
        // and stored once per tick.
        if let Some(p) = &mut self.pregnancy {
            let gestation_rate = 0.001_f64
                * health.to_f64()
                * nutrition.to_f64()
                * params.gestation_rate_mult.to_f64();
            p.gestation_progress =
                Fixed::from_f64((p.gestation_progress.to_f64() + gestation_rate).min(1.0));
            p.gestation_stage = gestation_stage_of(p.gestation_progress);
            // Observational maternal burden (never consumed — zero drift):
            // strain tracks gestation; complication risk rises with maternal
            // age and poor health (reproduction.ron health_risk_base 0.05).
            p.maternal_strain = (p.gestation_progress * Fixed::from_f64(0.4)).clamp_01();
            let age_penalty = if age_years > Fixed::from_f64(35.0) {
                (age_years - Fixed::from_f64(35.0)) * Fixed::from_f64(0.005)
            } else {
                Fixed::ZERO
            };
            p.complications_risk = (Fixed::from_f64(0.05)
                + (Fixed::ONE - health) * Fixed::from_f64(0.3)
                + age_penalty)
                .clamp(Fixed::from_f64(0.05), Fixed::from_f64(0.8));
            // Pregnancy increases parental drive
            self.parental_drive = (self.parental_drive + Fixed::from_f64(0.001)).clamp_01();
        }

        // Parental drive increases with pair bonding
        if self.pair_bond_strength > Fixed::from_f64(0.5) {
            self.parental_drive =
                (self.parental_drive + bonding_axis * Fixed::from_f64(0.0005)).clamp_01();
        }
    }

    /// Attempt conception (called when reproductive-age adults pair-bond).
    /// Returns true if conception occurs.
    ///
    /// NOTE (Iteration 42): not yet called from the sim — wiring it would
    /// introduce RNG draws (and a second birth path) that perturb calibrated
    /// trajectories; births remain demography-driven (§31). The documented
    /// consumer is a future calibrated conception wiring.
    pub fn attempt_conception(
        &mut self,
        partner_fertility: Fixed,
        conception_multiplier: Fixed,
        rng: &mut impl rand::Rng,
    ) -> bool {
        if self.pregnancy.is_some() || self.sex == BiologicalSex::Male {
            return false;
        }
        if self.fertility < Fixed::from_f64(0.2) || partner_fertility < Fixed::from_f64(0.2) {
            return false;
        }
        // Conception probability = product of both fertilities × multiplier × random factor
        let base_prob = (self.fertility * partner_fertility).to_f64();
        let roll: f64 = rng.random();
        roll < base_prob * 0.05 * conception_multiplier.to_f64()
    }

    /// Complete pregnancy — returns true if birth occurs.
    pub fn complete_pregnancy(&mut self) -> bool {
        let full_term = matches!(
            self.pregnancy.as_ref().map(|p| p.gestation_progress),
            Some(progress) if progress >= Fixed::ONE
        );
        if full_term {
            self.pregnancy = None;
            self.children_born += 1;
            true
        } else {
            false
        }
    }
}

/// §7.2.6: Derive the gestation stage from progress.
#[must_use]
pub fn gestation_stage_of(progress: Fixed) -> GestationStage {
    if progress >= Fixed::ONE {
        GestationStage::FullTerm
    } else if progress >= Fixed::from_f64(0.66) {
        GestationStage::Late
    } else if progress >= Fixed::from_f64(0.33) {
        GestationStage::Mid
    } else {
        GestationStage::Early
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepubescent_has_no_fertility() {
        let mut r = ReproductiveState::default();
        r.tick_update(
            Fixed::from_f64(10.0),
            Fixed::ONE,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ONE,
            ReproductiveUpdateParams::default(),
        );
        assert_eq!(r.fertility, Fixed::ZERO);
        assert_eq!(r.puberty_stage, PubertyStage::Prepubescent);
    }

    #[test]
    fn adult_has_full_maturity() {
        let mut r = ReproductiveState::default();
        r.tick_update(
            Fixed::from_f64(25.0),
            Fixed::ONE,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ONE,
            ReproductiveUpdateParams::default(),
        );
        assert_eq!(r.sexual_maturity, Fixed::ONE);
        assert_eq!(r.puberty_stage, PubertyStage::Complete);
    }

    #[test]
    fn pregnancy_defaults_to_none() {
        let r = ReproductiveState::default();
        assert!(r.pregnancy.is_none());
    }

    #[test]
    fn pregnancy_progresses() {
        let mut r = ReproductiveState {
            pregnancy: Some(PregnancyState::new(0)),
            ..ReproductiveState::default()
        };
        if let Some(p) = &mut r.pregnancy {
            p.gestation_progress = Fixed::from_f64(0.9);
        }
        r.tick_update(
            Fixed::from_f64(25.0),
            Fixed::ONE,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ONE,
            ReproductiveUpdateParams::default(),
        );
        let p = r.pregnancy.as_ref().expect("pregnancy persists");
        assert!(p.gestation_progress > Fixed::from_f64(0.9));
        // Observational burden tracks gestation.
        assert!(p.maternal_strain > Fixed::ZERO);
        assert!(p.complications_risk >= Fixed::from_f64(0.05));
    }

    #[test]
    fn gestation_stage_derivation() {
        assert_eq!(gestation_stage_of(Fixed::ZERO), GestationStage::Early);
        assert_eq!(
            gestation_stage_of(Fixed::from_f64(0.32)),
            GestationStage::Early
        );
        assert_eq!(
            gestation_stage_of(Fixed::from_f64(0.5)),
            GestationStage::Mid
        );
        assert_eq!(
            gestation_stage_of(Fixed::from_f64(0.9)),
            GestationStage::Late
        );
        assert_eq!(gestation_stage_of(Fixed::ONE), GestationStage::FullTerm);
    }

    #[test]
    fn complete_pregnancy_clears_on_full_term() {
        let mut r = ReproductiveState {
            pregnancy: Some(PregnancyState {
                gestation_progress: Fixed::from_f64(0.9),
                ..PregnancyState::new(0)
            }),
            ..ReproductiveState::default()
        };
        assert!(!r.complete_pregnancy(), "not full term yet");
        assert!(r.pregnancy.is_some());
        // Advance to full term through the field rather than reassigning the
        // outer struct (keeps clippy::field_reassign_with_default quiet).
        if let Some(p) = &mut r.pregnancy {
            p.gestation_progress = Fixed::ONE;
        }
        assert!(r.complete_pregnancy());
        assert!(r.pregnancy.is_none());
        assert_eq!(r.children_born, 1);
    }

    #[test]
    fn conception_blocked_when_pregnant() {
        use rand::SeedableRng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(7);
        let mut r = ReproductiveState {
            sex: BiologicalSex::Female,
            fertility: Fixed::ONE,
            pregnancy: Some(PregnancyState::new(0)),
            ..ReproductiveState::default()
        };
        assert!(!r.attempt_conception(Fixed::ONE, Fixed::ONE, &mut rng));
    }

    #[test]
    fn pregnancy_lifecycle_round_trips_via_tick_updates() {
        use rand::SeedableRng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(7);
        let mut r = ReproductiveState {
            sex: BiologicalSex::Female,
            fertility: Fixed::ONE,
            pregnancy: Some(PregnancyState::new(0)),
            ..ReproductiveState::default()
        };
        // Walk the full gestation purely through tick_update increments
        // (rate 0.001/tick at health = nutrition = 1, multiplier = 1 →
        // ~1000 ticks to term), proving the `>= Fixed::ONE` boundary is
        // reached on the incremental path — not only when progress is
        // set to 1.0 by hand.
        for _ in 0..1001 {
            r.tick_update(
                Fixed::from_f64(25.0),
                Fixed::ONE,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ONE,
                ReproductiveUpdateParams::default(),
            );
        }
        let progress = r
            .pregnancy
            .as_ref()
            .expect("pregnancy persists through gestation")
            .gestation_progress;
        assert!(
            progress >= Fixed::ONE,
            "gestation must reach full term via increments (got {progress})"
        );
        assert!(r.complete_pregnancy(), "full term completes");
        assert!(r.pregnancy.is_none(), "slot cleared");
        assert_eq!(r.children_born, 1);
        // The Option lifecycle round-trips: a fresh conception is now possible.
        let reconceived = (0..300).any(|_| r.attempt_conception(Fixed::ONE, Fixed::ONE, &mut rng));
        assert!(
            reconceived,
            "re-conception possible after full-term completion"
        );
    }

    #[test]
    fn maternal_age_raises_complication_risk() {
        let mut young = ReproductiveState {
            pregnancy: Some(PregnancyState::new(0)),
            ..ReproductiveState::default()
        };
        young.tick_update(
            Fixed::from_f64(25.0),
            Fixed::from_f64(0.9),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ONE,
            ReproductiveUpdateParams::default(),
        );
        let mut elder = ReproductiveState {
            pregnancy: Some(PregnancyState::new(0)),
            ..ReproductiveState::default()
        };
        elder.tick_update(
            Fixed::from_f64(45.0),
            Fixed::from_f64(0.9),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ONE,
            ReproductiveUpdateParams::default(),
        );
        let young_risk = young.pregnancy.unwrap().complications_risk;
        let elder_risk = elder.pregnancy.unwrap().complications_risk;
        assert!(elder_risk > young_risk, "elder mothers carry higher risk");
    }
}
