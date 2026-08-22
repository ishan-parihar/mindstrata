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
    /// Male. (doc added at S2 extraction)
    Male,
    /// Female. (doc added at S2 extraction)
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
    /// Sex. (doc added at S2 extraction)
    pub sex: Sex,
    /// Trait predispositions. (doc added at S2 extraction)
    pub trait_predispositions: TraitPredispositions,
    /// Health predispositions. (doc added at S2 extraction)
    pub health_predispositions: HealthPredispositions,
    /// Metabolic predispositions. (doc added at S2 extraction)
    pub metabolic_predispositions: MetabolicPredispositions,
    /// Physical potential. (doc added at S2 extraction)
    pub physical_potential: PhysicalPotential,
    /// Fertility predispositions. (doc added at S2 extraction)
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

    /// Blend two parent genomes into a child genome (Iteration 244).
    ///
    /// Quantitative-genetics midpoint model: every gene is the parental
    /// midpoint plus small mutation noise (±`MUTATION_NOISE`), clamped to
    /// [0,1]. Sex is a fresh 50/50 draw. With `b == a` (single known
    /// ancestor — the household-continuity path for replacement newborns)
    /// the child clusters around that ancestor with mutation spread.
    /// Deterministic from the passed RNG stream; consumes exactly one RNG
    /// draw per gene plus one for sex, in field order.
    pub fn blend(a: &Genome, b: &Genome, rng: &mut impl Rng) -> Self {
        let sex = if rng.random_bool(0.5) {
            Sex::Male
        } else {
            Sex::Female
        };
        Self {
            sex,
            trait_predispositions: TraitPredispositions {
                stress_reactivity: blend_gene(
                    a.trait_predispositions.stress_reactivity,
                    b.trait_predispositions.stress_reactivity,
                    rng,
                ),
                aggression_threshold: blend_gene(
                    a.trait_predispositions.aggression_threshold,
                    b.trait_predispositions.aggression_threshold,
                    rng,
                ),
                novelty_seeking: blend_gene(
                    a.trait_predispositions.novelty_seeking,
                    b.trait_predispositions.novelty_seeking,
                    rng,
                ),
                attachment_vulnerability: blend_gene(
                    a.trait_predispositions.attachment_vulnerability,
                    b.trait_predispositions.attachment_vulnerability,
                    rng,
                ),
                addiction_risk: blend_gene(
                    a.trait_predispositions.addiction_risk,
                    b.trait_predispositions.addiction_risk,
                    rng,
                ),
                depression_vulnerability: blend_gene(
                    a.trait_predispositions.depression_vulnerability,
                    b.trait_predispositions.depression_vulnerability,
                    rng,
                ),
            },
            health_predispositions: HealthPredispositions {
                immune_strength: blend_gene(
                    a.health_predispositions.immune_strength,
                    b.health_predispositions.immune_strength,
                    rng,
                ),
                disease_susceptibility: blend_gene(
                    a.health_predispositions.disease_susceptibility,
                    b.health_predispositions.disease_susceptibility,
                    rng,
                ),
                recovery_rate: blend_gene(
                    a.health_predispositions.recovery_rate,
                    b.health_predispositions.recovery_rate,
                    rng,
                ),
                chronic_pain_risk: blend_gene(
                    a.health_predispositions.chronic_pain_risk,
                    b.health_predispositions.chronic_pain_risk,
                    rng,
                ),
            },
            metabolic_predispositions: MetabolicPredispositions {
                metabolic_rate: blend_gene(
                    a.metabolic_predispositions.metabolic_rate,
                    b.metabolic_predispositions.metabolic_rate,
                    rng,
                ),
                fat_storage: blend_gene(
                    a.metabolic_predispositions.fat_storage,
                    b.metabolic_predispositions.fat_storage,
                    rng,
                ),
                hunger_sensitivity: blend_gene(
                    a.metabolic_predispositions.hunger_sensitivity,
                    b.metabolic_predispositions.hunger_sensitivity,
                    rng,
                ),
            },
            physical_potential: PhysicalPotential {
                strength_ceiling: blend_gene(
                    a.physical_potential.strength_ceiling,
                    b.physical_potential.strength_ceiling,
                    rng,
                ),
                endurance_ceiling: blend_gene(
                    a.physical_potential.endurance_ceiling,
                    b.physical_potential.endurance_ceiling,
                    rng,
                ),
                sensory_acuity: blend_gene(
                    a.physical_potential.sensory_acuity,
                    b.physical_potential.sensory_acuity,
                    rng,
                ),
            },
            fertility_predispositions: FertilityPredispositions {
                base_fertility: blend_gene(
                    a.fertility_predispositions.base_fertility,
                    b.fertility_predispositions.base_fertility,
                    rng,
                ),
                puberty_age: blend_gene(
                    a.fertility_predispositions.puberty_age,
                    b.fertility_predispositions.puberty_age,
                    rng,
                ),
            },
        }
    }
}

/// Mutation noise half-width for `Genome::blend`. Wide enough that sibling
/// pairs differ measurably across a few generations, narrow enough that a
/// family's expressed traits stay visibly correlated within one generation
/// (the Iteration-245 probe pins parent-child slope in [0.3, 0.7]).
const MUTATION_NOISE: f64 = 0.06;

/// Midpoint + uniform mutation noise, clamped to [0,1]. f64 arithmetic with
/// one final quantization — same determinism discipline as every other
/// sub-resolution rate in this workspace (Fixed::mul truncates).
fn blend_gene(a: Fixed, b: Fixed, rng: &mut impl Rng) -> Fixed {
    let mid = (a.to_f64() + b.to_f64()) * 0.5;
    let noise = rng.random_range(-MUTATION_NOISE..MUTATION_NOISE);
    Fixed::from_f64((mid + noise).clamp(0.0, 1.0))
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

    #[test]
    fn blend_is_deterministic() {
        let mut r1 = rand::rngs::StdRng::seed_from_u64(7);
        let mut r2 = rand::rngs::StdRng::seed_from_u64(7);
        let a = Genome::random(&mut rand::rngs::StdRng::seed_from_u64(1));
        let b = Genome::random(&mut rand::rngs::StdRng::seed_from_u64(2));
        let c1 = Genome::blend(&a, &b, &mut r1);
        let c2 = Genome::blend(&a, &b, &mut r2);
        assert_eq!(
            c1.metabolic_predispositions.metabolic_rate.to_raw(),
            c2.metabolic_predispositions.metabolic_rate.to_raw()
        );
    }

    /// The Iteration-244 heredity contract: blended children sit closer to
    /// the parental midpoint than to unrelated genomes, across many draws.
    #[test]
    fn blend_clusters_around_parental_midpoint() {
        let parent_a = Genome::random(&mut rand::rngs::StdRng::seed_from_u64(11));
        let parent_b = Genome::random(&mut rand::rngs::StdRng::seed_from_u64(22));
        let stranger = Genome::random(&mut rand::rngs::StdRng::seed_from_u64(33));
        let mid = |x: f64, y: f64| (x + y) * 0.5;
        let mut child_dist = 0.0f64;
        let mut stranger_dist = 0.0f64;
        for seed in 100u64..140 {
            let child = Genome::blend(
                &parent_a,
                &parent_b,
                &mut rand::rngs::StdRng::seed_from_u64(seed),
            );
            child_dist += (child.trait_predispositions.stress_reactivity.to_f64()
                - mid(
                    parent_a.trait_predispositions.stress_reactivity.to_f64(),
                    parent_b.trait_predispositions.stress_reactivity.to_f64(),
                ))
            .abs();
            stranger_dist += (child.trait_predispositions.stress_reactivity.to_f64()
                - stranger.trait_predispositions.stress_reactivity.to_f64())
            .abs();
        }
        assert!(
            child_dist < stranger_dist,
            "child must resemble parents ({child_dist:.3}) more than a stranger ({stranger_dist:.3})"
        );
    }

    /// Single-ancestor path (household continuity): self-blend children
    /// cluster around that ancestor with only mutation spread.
    #[test]
    fn self_blend_stays_near_ancestor() {
        let a = Genome::random(&mut rand::rngs::StdRng::seed_from_u64(44));
        let anchor = a.health_predispositions.immune_strength.to_f64();
        let mut max_dev = 0.0f64;
        for seed in 200u64..230 {
            let child = Genome::blend(&a, &a, &mut rand::rngs::StdRng::seed_from_u64(seed));
            max_dev =
                max_dev.max((child.health_predispositions.immune_strength.to_f64() - anchor).abs());
        }
        assert!(
            max_dev <= 0.061,
            "self-blend deviation {max_dev:.4} exceeds mutation half-width"
        );
    }
}
