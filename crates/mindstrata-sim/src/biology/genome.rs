//! Genetic system — heritable individuality and developmental predispositions.
//!
//! Genes are not direct traits. They are potentials moderated by environment:
//!
//! ```text
//! expressed_trait = genetic_potential
//!                 × developmental_environment
//!                 × nutrition
//!                 × stress_history
//!                 × cultural_shaping
//!                 × random_epigenetic_noise
//! ```

use mindstrata_core::fixed::Fixed;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Biological sex of an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sex {
    Male,
    Female,
}

/// Predispositions encoded in the genome.
/// These are potentials (0–1) that interact with environment to produce expressed traits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitPredispositions {
    /// Baseline stress reactivity (high = more cortisol response).
    pub stress_reactivity: Fixed,
    /// Aggression threshold (low = easier to anger).
    pub aggression_threshold: Fixed,
    /// Openness to novelty (biological component).
    pub novelty_seeking: Fixed,
    /// Attachment vulnerability (anxious/avoidant predisposition).
    pub attachment_vulnerability: Fixed,
    /// Addiction risk.
    pub addiction_risk: Fixed,
    /// Depression vulnerability.
    pub depression_vulnerability: Fixed,
}

impl Default for TraitPredispositions {
    fn default() -> Self {
        Self {
            stress_reactivity: Fixed::from_f64(0.5),
            aggression_threshold: Fixed::from_f64(0.5),
            novelty_seeking: Fixed::from_f64(0.5),
            attachment_vulnerability: Fixed::from_f64(0.5),
            addiction_risk: Fixed::from_f64(0.3),
            depression_vulnerability: Fixed::from_f64(0.3),
        }
    }
}

/// Health predispositions — disease resistance and vulnerability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthPredispositions {
    /// Immune system baseline strength.
    pub immune_strength: Fixed,
    /// Disease susceptibility.
    pub disease_susceptibility: Fixed,
    /// Recovery rate modifier.
    pub recovery_rate: Fixed,
    /// Chronic pain predisposition.
    pub chronic_pain_risk: Fixed,
}

impl Default for HealthPredispositions {
    fn default() -> Self {
        Self {
            immune_strength: Fixed::from_f64(0.6),
            disease_susceptibility: Fixed::from_f64(0.4),
            recovery_rate: Fixed::from_f64(0.5),
            chronic_pain_risk: Fixed::from_f64(0.2),
        }
    }
}

/// Metabolic predispositions — energy processing efficiency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolicPredispositions {
    /// Baseline metabolic rate.
    pub metabolic_rate: Fixed,
    /// Fat storage efficiency (high = gains weight easily).
    pub fat_storage: Fixed,
    /// Hunger sensitivity.
    pub hunger_sensitivity: Fixed,
}

impl Default for MetabolicPredispositions {
    fn default() -> Self {
        Self {
            metabolic_rate: Fixed::from_f64(0.5),
            fat_storage: Fixed::from_f64(0.5),
            hunger_sensitivity: Fixed::from_f64(0.5),
        }
    }
}

/// Physical potential — strength, endurance, sensory acuity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalPotential {
    /// Maximum achievable strength.
    pub strength_ceiling: Fixed,
    /// Maximum achievable endurance.
    pub endurance_ceiling: Fixed,
    /// Sensory acuity baseline.
    pub sensory_acuity: Fixed,
}

impl Default for PhysicalPotential {
    fn default() -> Self {
        Self {
            strength_ceiling: Fixed::from_f64(0.6),
            endurance_ceiling: Fixed::from_f64(0.6),
            sensory_acuity: Fixed::from_f64(0.5),
        }
    }
}

/// Fertility predispositions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FertilityPredispositions {
    /// Baseline fertility.
    pub base_fertility: Fixed,
    /// Age at puberty onset (in years, abstracted).
    pub puberty_age: Fixed,
}

impl Default for FertilityPredispositions {
    fn default() -> Self {
        Self {
            base_fertility: Fixed::from_f64(0.7),
            puberty_age: Fixed::from_f64(13.0),
        }
    }
}

/// Heritable genome for each agent.
/// Genes bias but do not determine — environment moderates expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genome {
    pub sex: Sex,
    pub trait_predispositions: TraitPredispositions,
    pub health_predispositions: HealthPredispositions,
    pub metabolic_predispositions: MetabolicPredispositions,
    pub physical_potential: PhysicalPotential,
    pub fertility_predispositions: FertilityPredispositions,
}

impl Genome {
    /// Generate a random genome.
    pub fn random(rng: &mut impl Rng) -> Self {
        let sex = if rng.random_bool(0.5) {
            Sex::Male
        } else {
            Sex::Female
        };
        Self {
            sex,
            trait_predispositions: TraitPredispositions {
                stress_reactivity: Fixed::from_f64(rng.random_range(0.1..0.9)),
                aggression_threshold: Fixed::from_f64(rng.random_range(0.2..0.9)),
                novelty_seeking: Fixed::from_f64(rng.random_range(0.1..0.9)),
                attachment_vulnerability: Fixed::from_f64(rng.random_range(0.1..0.8)),
                addiction_risk: Fixed::from_f64(rng.random_range(0.0..0.6)),
                depression_vulnerability: Fixed::from_f64(rng.random_range(0.0..0.6)),
            },
            health_predispositions: HealthPredispositions {
                immune_strength: Fixed::from_f64(rng.random_range(0.2..0.9)),
                disease_susceptibility: Fixed::from_f64(rng.random_range(0.1..0.7)),
                recovery_rate: Fixed::from_f64(rng.random_range(0.2..0.8)),
                chronic_pain_risk: Fixed::from_f64(rng.random_range(0.0..0.5)),
            },
            metabolic_predispositions: MetabolicPredispositions {
                metabolic_rate: Fixed::from_f64(rng.random_range(0.3..0.8)),
                fat_storage: Fixed::from_f64(rng.random_range(0.1..0.8)),
                hunger_sensitivity: Fixed::from_f64(rng.random_range(0.2..0.8)),
            },
            physical_potential: PhysicalPotential {
                strength_ceiling: Fixed::from_f64(rng.random_range(0.3..0.9)),
                endurance_ceiling: Fixed::from_f64(rng.random_range(0.3..0.9)),
                sensory_acuity: Fixed::from_f64(rng.random_range(0.2..0.8)),
            },
            fertility_predispositions: FertilityPredispositions {
                base_fertility: Fixed::from_f64(rng.random_range(0.3..0.9)),
                puberty_age: Fixed::from_f64(rng.random_range(11.0..15.0)),
            },
        }
    }

    /// Express a trait given environmental moderators.
    /// `genetic_potential` is the genome value, `environment` is 0–1 modifier.
    pub fn express_trait(genetic_potential: Fixed, environment: Fixed) -> Fixed {
        // expressed = genetic × (0.5 + 0.5 × environment)
        // Environment can halve or fully realize genetic potential
        let env_modifier = Fixed::from_f64(0.5) + environment * Fixed::from_f64(0.5);
        (genetic_potential * env_modifier).clamp_01()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn genome_random_is_deterministic() {
        let mut rng1 = rand::rngs::StdRng::seed_from_u64(42);
        let mut rng2 = rand::rngs::StdRng::seed_from_u64(42);
        let g1 = Genome::random(&mut rng1);
        let g2 = Genome::random(&mut rng2);
        assert_eq!(g1.sex, g2.sex);
        assert_eq!(
            g1.trait_predispositions.stress_reactivity.to_raw(),
            g2.trait_predispositions.stress_reactivity.to_raw()
        );
    }

    #[test]
    fn express_trait_respects_environment() {
        let potential = Fixed::from_f64(0.8);
        let low_env = Fixed::from_f64(0.2);
        let high_env = Fixed::from_f64(0.9);
        let low = Genome::express_trait(potential, low_env);
        let high = Genome::express_trait(potential, high_env);
        assert!(high > low);
    }
}
