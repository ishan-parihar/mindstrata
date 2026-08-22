//! Digestive system — eating as a biological process (AP2 §7.2.7).
//!
//! Food takes time to digest, poor-quality (spoiled) food causes nausea and
//! gut damage, satiety tracks recent intake, and gut health recovers when fed
//! well. The system is designed to be **exactly neutral at baseline**:
//! `effective_digestion() = 1.0` for a healthy, fed agent, so calibrated runs
//! are untouched. Only spoiled food (quality < 0.4) or prolonged under-eating
//! moves the state away from neutral.
//!
//! Alignment: `specs/biology/organs.ron` (digestive section added in
//! Iteration 38) declares the tunable parameters.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A food craving (optional, abstract — currently observational).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Craving {
    /// Craving kind (e.g. "salt", "sweet", "meat").
    pub kind: String,
    /// Craving strength (0 = none, 1 = intense).
    pub strength: Fixed,
}

/// Digestive state (§7.2.7).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestiveState {
    /// Food currently in the stomach (0 = empty, 1 = full).
    pub stomach_contents: Fixed,
    /// Nutrient absorption from digestion (0 = none, 1 = fully absorbed).
    pub nutrient_absorption: Fixed,
    /// Gut health (0 = damaged, 1 = healthy). **Starts at exactly 1.0.**
    pub gut_health: Fixed,
    /// Food safety perception (1 = safe, 0 = unsafe).
    pub food_safety: Fixed,
    /// Nausea from spoiled food (0 = none, 1 = severe).
    pub nausea: Fixed,
    /// Satiety from recent intake (0 = hungry, 1 = full).
    pub satiety: Fixed,
    /// Current cravings (observational for now).
    pub cravings: Vec<Craving>,
}

impl Default for DigestiveState {
    fn default() -> Self {
        Self {
            stomach_contents: Fixed::ZERO,
            nutrient_absorption: Fixed::from_f64(0.5),
            // Neutral values — the multiplier must be exactly 1.0 at baseline.
            gut_health: Fixed::ONE,
            food_safety: Fixed::ONE,
            nausea: Fixed::ZERO,
            satiety: Fixed::ZERO,
            cravings: Vec::new(),
        }
    }
}

impl DigestiveState {
    /// Consume food of a given quality (0 = spoiled, 1 = excellent).
    ///
    /// Spoiled food (quality < 0.4) damages gut health and induces nausea;
    /// good food soothes the gut and settles nausea.
    pub fn consume_food(&mut self, quality: Fixed) {
        self.stomach_contents = (self.stomach_contents + Fixed::from_f64(0.5)).clamp_01();
        if quality < Fixed::from_f64(0.4) {
            let hazard = (Fixed::from_f64(0.4) - quality) * Fixed::from_f64(0.05);
            self.gut_health = (self.gut_health - hazard).max(Fixed::from_f64(0.1));
            self.nausea = (self.nausea + hazard).clamp_01();
            self.food_safety = (self.food_safety - hazard).max(Fixed::ZERO);
        } else {
            // Good food: gut recovers toward full health, nausea subsides.
            self.gut_health = (self.gut_health + Fixed::from_f64(0.01)).min(Fixed::ONE);
            self.nausea = (self.nausea - Fixed::from_f64(0.02)).max(Fixed::ZERO);
            self.food_safety = (self.food_safety + Fixed::from_f64(0.005)).min(Fixed::ONE);
        }
    }

    /// Per-tick digestion: stomach contents convert into absorbed nutrients,
    /// satiety tracks intake and decays, absorption slowly integrates.
    pub fn tick_update(&mut self) {
        let digested = self.stomach_contents * Fixed::from_f64(0.1);
        self.stomach_contents = (self.stomach_contents - digested).max(Fixed::ZERO);
        self.nutrient_absorption =
            (self.nutrient_absorption + digested * Fixed::from_f64(0.5)).clamp_01();
        self.satiety = (self.satiety + digested).clamp_01();
        self.satiety = (self.satiety - Fixed::from_f64(0.01)).max(Fixed::ZERO);
    }

    /// Nutrition multiplier — exactly 1.0 at baseline (healthy gut, no nausea);
    /// falls under gut damage or spoiled-food nausea (floored at 0.2).
    #[must_use]
    pub fn effective_digestion(&self) -> Fixed {
        (self.gut_health - self.nausea * Fixed::from_f64(0.5))
            .clamp(Fixed::from_f64(0.2), Fixed::ONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_is_exactly_neutral() {
        let d = DigestiveState::default();
        assert_eq!(d.effective_digestion(), Fixed::ONE);
        assert_eq!(d.gut_health, Fixed::ONE);
        assert_eq!(d.nausea, Fixed::ZERO);
        assert_eq!(d.food_safety, Fixed::ONE);
    }

    #[test]
    fn good_food_keeps_digestion_neutral() {
        let mut d = DigestiveState::default();
        for _ in 0..100 {
            d.consume_food(Fixed::from_f64(0.6));
            d.tick_update();
        }
        assert_eq!(d.effective_digestion(), Fixed::ONE);
        assert_eq!(d.gut_health, Fixed::ONE);
        assert_eq!(d.nausea, Fixed::ZERO);
    }

    #[test]
    fn spoiled_food_damages_gut_and_causes_nausea() {
        let mut d = DigestiveState::default();
        for _ in 0..50 {
            d.consume_food(Fixed::from_f64(0.2));
        }
        assert!(d.gut_health < Fixed::ONE);
        assert!(d.nausea > Fixed::ZERO);
        assert!(d.food_safety < Fixed::ONE);
        assert!(d.effective_digestion() < Fixed::ONE);
    }

    #[test]
    fn digestion_converts_stomach_to_satiety() {
        let mut d = DigestiveState::default();
        d.consume_food(Fixed::from_f64(0.8));
        d.tick_update();
        assert!(d.stomach_contents > Fixed::ZERO);
        assert!(d.satiety > Fixed::ZERO);
        d.tick_update();
        assert!(d.nutrient_absorption > Fixed::from_f64(0.5));
    }
}
