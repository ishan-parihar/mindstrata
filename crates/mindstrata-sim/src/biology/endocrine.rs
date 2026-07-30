//! Hormonal / Endocrine system — connects body to mind.
//!
//! Hormones modulate emotion, motivation, trust, aggression, bonding, appetite,
//! sleep, and reproduction. We simulate functional axes, not individual hormones.
//!
//! Each axis has:
//! - inputs (what raises/lowers it)
//! - current level (0–1)
//! - effects on psychology and behavior

use mindstrata_core::fixed::Fixed;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Stress axis (cortisol-like).
/// Raised by: threat, pain, hunger, sleep debt, humiliation, uncertainty.
/// Effects: increases fear/anger, reduces planning, increases heuristic bias.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressAxis {
    /// Current level (0 = calm, 1 = panicked).
    pub level: Fixed,
    /// Baseline reactivity — how quickly this agent responds to stress.
    pub reactivity: Fixed,
    /// Chronic stress load — accumulates over time.
    pub chronic_load: Fixed,
}

impl Default for StressAxis {
    fn default() -> Self {
        Self {
            level: Fixed::from_f64(0.2),
            reactivity: Fixed::from_f64(0.5),
            chronic_load: Fixed::ZERO,
        }
    }
}

impl StressAxis {
    /// Update stress level based on acute input and recovery.
    pub fn update(&mut self, acute_input: Fixed, parasympathetic_tone: Fixed) {
        let recovery_rate = Fixed::from_f64(0.05) * parasympathetic_tone;
        let acute_delta = acute_input * self.reactivity;
        let chronic_delta = self.chronic_load * Fixed::from_f64(0.1);
        self.level = (self.level + acute_delta + chronic_delta - recovery_rate).clamp_01();
        // Chronic load accumulates slowly from high acute stress
        if self.level > Fixed::from_f64(0.6) {
            self.chronic_load = (self.chronic_load + Fixed::from_f64(0.001)).clamp_01();
        } else {
            // Chronic load recovers very slowly
            self.chronic_load = (self.chronic_load - Fixed::from_f64(0.0005)).max(Fixed::ZERO);
        }
    }
}

/// Bonding axis (oxytocin-like).
/// Raised by: positive touch, comforting, caregiving, shared meals, ritual.
/// Effects: increases trust, attachment, in-group favoritism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondingAxis {
    pub level: Fixed,
    /// Baseline receptivity to bonding.
    pub receptivity: Fixed,
}

impl Default for BondingAxis {
    fn default() -> Self {
        Self {
            level: Fixed::from_f64(0.4),
            receptivity: Fixed::from_f64(0.5),
        }
    }
}

impl BondingAxis {
    pub fn update(&mut self, social_input: Fixed) {
        let recovery = Fixed::from_f64(0.02);
        let delta = social_input * self.receptivity;
        self.level = (self.level + delta - recovery).clamp_01();
    }
}

/// Dominance axis (testosterone-like).
/// Raised by: status victory, competition, public respect.
/// Effects: modulates confidence, aggression, risk tolerance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DominanceAxis {
    pub level: Fixed,
}

impl Default for DominanceAxis {
    fn default() -> Self {
        Self {
            level: Fixed::from_f64(0.4),
        }
    }
}

impl DominanceAxis {
    pub fn update(&mut self, status_change: Fixed) {
        // Rises with status gains, falls with status losses
        self.level = (self.level + status_change * Fixed::from_f64(0.1)
            - Fixed::from_f64(0.01))
            .clamp_01();
    }
}

/// Fertility axis — reproductive state modulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FertilityAxis {
    /// Current fertility level.
    pub level: Fixed,
    /// Libido.
    pub libido: Fixed,
    /// Whether agent is pregnant.
    pub pregnant: bool,
    /// Pregnancy progress (0–1, 1 = birth).
    pub pregnancy_progress: Fixed,
}

impl Default for FertilityAxis {
    fn default() -> Self {
        Self {
            level: Fixed::from_f64(0.5),
            libido: Fixed::from_f64(0.3),
            pregnant: false,
            pregnancy_progress: Fixed::ZERO,
        }
    }
}

/// Metabolic axis — energy processing, hunger, satiety.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolicAxis {
    /// Energy availability.
    pub energy: Fixed,
    /// Appetite pressure (high = hungry).
    pub appetite: Fixed,
    /// Satiety (high = full).
    pub satiety: Fixed,
}

impl Default for MetabolicAxis {
    fn default() -> Self {
        Self {
            energy: Fixed::from_f64(0.7),
            appetite: Fixed::from_f64(0.3),
            satiety: Fixed::from_f64(0.5),
        }
    }
}

/// Arousal axis — sympathetic activation for fight/flight/freeze.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArousalAxis {
    /// Current arousal level.
    pub level: Fixed,
    /// Baseline arousal.
    pub baseline: Fixed,
}

impl Default for ArousalAxis {
    fn default() -> Self {
        Self {
            level: Fixed::from_f64(0.3),
            baseline: Fixed::from_f64(0.3),
        }
    }
}

impl ArousalAxis {
    pub fn update(&mut self, acute_input: Fixed) {
        // Rapid rise, slower decay back to baseline
        let rise = acute_input * Fixed::from_f64(0.3);
        let decay = (self.level - self.baseline) * Fixed::from_f64(0.1);
        self.level = (self.level + rise - decay).clamp_01();
    }
}

/// Growth axis — development, repair, aging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthAxis {
    /// Current growth/repair capacity.
    pub capacity: Fixed,
    /// Developmental stage modifier (child=high, adult=moderate, elder=low).
    pub developmental_modifier: Fixed,
}

impl Default for GrowthAxis {
    fn default() -> Self {
        Self {
            capacity: Fixed::from_f64(0.6),
            developmental_modifier: Fixed::from_f64(1.0),
        }
    }
}

/// Complete endocrine state — all hormonal axes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EndocrineState {
    pub stress: StressAxis,
    pub bonding: BondingAxis,
    pub dominance: DominanceAxis,
    pub fertility: FertilityAxis,
    pub metabolic: MetabolicAxis,
    pub arousal: ArousalAxis,
    pub growth: GrowthAxis,
}

impl EndocrineState {
    /// Generate a random endocrine state.
    pub fn random(rng: &mut impl Rng) -> Self {
        Self {
            stress: StressAxis {
                level: Fixed::from_f64(0.2),
                reactivity: Fixed::from_f64(rng.random_range(0.2..0.8)),
                chronic_load: Fixed::ZERO,
            },
            bonding: BondingAxis {
                level: Fixed::from_f64(rng.random_range(0.3..0.6)),
                receptivity: Fixed::from_f64(rng.random_range(0.2..0.8)),
            },
            dominance: DominanceAxis {
                level: Fixed::from_f64(rng.random_range(0.2..0.6)),
            },
            fertility: FertilityAxis::default(),
            metabolic: MetabolicAxis {
                energy: Fixed::from_f64(rng.random_range(0.5..0.8)),
                appetite: Fixed::from_f64(rng.random_range(0.2..0.5)),
                satiety: Fixed::from_f64(rng.random_range(0.3..0.6)),
            },
            arousal: ArousalAxis {
                level: Fixed::from_f64(rng.random_range(0.2..0.4)),
                baseline: Fixed::from_f64(rng.random_range(0.2..0.4)),
            },
            growth: GrowthAxis {
                capacity: Fixed::from_f64(0.6),
                developmental_modifier: Fixed::from_f64(1.0),
            },
        }
    }

    /// Compute parasympathetic tone from stress and arousal (inverse relationship).
    /// High stress/arousal = low parasympathetic tone.
    pub fn parasympathetic_tone(&self) -> Fixed {
        (Fixed::ONE - self.stress.level * Fixed::from_f64(0.5)
            - self.arousal.level * Fixed::from_f64(0.3))
            .clamp_01()
    }

    /// Stress modifier for cognitive function.
    /// High stress = more heuristic bias, less planning.
    pub fn stress_cognitive_modifier(&self) -> Fixed {
        self.stress.level
    }

    /// Bonding modifier for trust formation.
    pub fn bonding_trust_modifier(&self) -> Fixed {
        self.bonding.level
    }

    /// Dominance modifier for aggression.
    pub fn dominance_aggression_modifier(&self) -> Fixed {
        self.dominance.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn stress_axis_recovers() {
        let mut axis = StressAxis::default();
        axis.level = Fixed::from_f64(0.8);
        let tone = Fixed::from_f64(0.5);
        // Acute input 0, so only recovery happens
        axis.update(Fixed::ZERO, tone);
        assert!(axis.level < Fixed::from_f64(0.8));
    }

    #[test]
    fn stress_axis_increases_with_input() {
        let mut axis = StressAxis::default();
        axis.level = Fixed::from_f64(0.2);
        let tone = Fixed::from_f64(0.5);
        axis.update(Fixed::from_f64(0.8), tone);
        assert!(axis.level > Fixed::from_f64(0.2));
    }

    #[test]
    fn parasympathetic_tone_inversely_relates_to_stress() {
        let mut endo = EndocrineState::default();
        endo.stress.level = Fixed::from_f64(0.1);
        let calm_tone = endo.parasympathetic_tone();
        endo.stress.level = Fixed::from_f64(0.9);
        let stressed_tone = endo.parasympathetic_tone();
        assert!(calm_tone > stressed_tone);
    }

    #[test]
    fn arousal_decays_to_baseline() {
        let mut axis = ArousalAxis {
            level: Fixed::from_f64(0.9),
            baseline: Fixed::from_f64(0.3),
        };
        // No acute input, should decay
        axis.update(Fixed::ZERO);
        assert!(axis.level < Fixed::from_f64(0.9));
        assert!(axis.level > axis.baseline);
    }

    #[test]
    fn endocrine_state_deterministic() {
        let mut rng1 = rand::rngs::StdRng::seed_from_u64(42);
        let mut rng2 = rand::rngs::StdRng::seed_from_u64(42);
        let e1 = EndocrineState::random(&mut rng1);
        let e2 = EndocrineState::random(&mut rng2);
        assert_eq!(
            e1.stress.reactivity.to_raw(),
            e2.stress.reactivity.to_raw()
        );
    }
}
