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
    /// Whether currently pregnant.
    pub pregnant: bool,
    /// Pregnancy progress (0 = conception, 1 = birth).
    pub pregnancy_progress: Fixed,
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
            pregnant: false,
            pregnancy_progress: Fixed::ZERO,
            children_born: 0,
            parental_drive: Fixed::from_f64(0.3),
        }
    }
}

impl ReproductiveState {
    /// Update reproductive state based on age and hormonal signals.
    pub fn tick_update(
        &mut self,
        age_years: Fixed,
        health: Fixed,
        stress_level: Fixed,
        bonding_axis: Fixed,
        nutrition: Fixed,
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
                let decline = (age_years - Fixed::from_f64(35.0)) * Fixed::from_f64(0.03);
                (Fixed::ONE - decline).max(Fixed::from_f64(0.1))
            };
            self.fertility = age_factor * health * (Fixed::ONE - stress_level * Fixed::from_f64(0.3));
            self.libido = (Fixed::from_f64(0.5) + bonding_axis * Fixed::from_f64(0.3)
                - stress_level * Fixed::from_f64(0.2))
                .clamp_01();
        }

        // Pregnancy progression
        if self.pregnant {
            let gestation_rate = Fixed::from_f64(0.001) * health * nutrition;
            self.pregnancy_progress = (self.pregnancy_progress + gestation_rate).clamp_01();
            // Pregnancy increases parental drive
            self.parental_drive = (self.parental_drive + Fixed::from_f64(0.001)).clamp_01();
        }

        // Parental drive increases with pair bonding
        if self.pair_bond_strength > Fixed::from_f64(0.5) {
            self.parental_drive = (self.parental_drive + bonding_axis * Fixed::from_f64(0.0005))
                .clamp_01();
        }
    }

    /// Attempt conception (called when reproductive-age adults pair-bond).
    /// Returns true if conception occurs.
    pub fn attempt_conception(
        &mut self,
        partner_fertility: Fixed,
        rng: &mut impl rand::Rng,
    ) -> bool {
        if self.pregnant || self.sex == BiologicalSex::Male {
            return false;
        }
        if self.fertility < Fixed::from_f64(0.2) || partner_fertility < Fixed::from_f64(0.2) {
            return false;
        }
        // Conception probability = product of both fertilities × random factor
        let base_prob = (self.fertility * partner_fertility).to_f64();
        let roll: f64 = rng.random();
        roll < base_prob * 0.05 // low per-tick probability
    }

    /// Complete pregnancy — returns true if birth occurs.
    pub fn complete_pregnancy(&mut self) -> bool {
        if self.pregnant && self.pregnancy_progress >= Fixed::ONE {
            self.pregnant = false;
            self.pregnancy_progress = Fixed::ZERO;
            self.children_born += 1;
            true
        } else {
            false
        }
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
        );
        assert_eq!(r.sexual_maturity, Fixed::ONE);
        assert_eq!(r.puberty_stage, PubertyStage::Complete);
    }

    #[test]
    fn pregnancy_progresses() {
        let mut r = ReproductiveState::default();
        r.pregnant = true;
        r.pregnancy_progress = Fixed::from_f64(0.9);
        r.tick_update(
            Fixed::from_f64(25.0),
            Fixed::ONE,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ONE,
        );
        assert!(r.pregnancy_progress > Fixed::from_f64(0.9));
    }
}
