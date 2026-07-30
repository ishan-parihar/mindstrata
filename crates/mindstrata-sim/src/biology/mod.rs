//! Biological layer — the embodied substrate of each agent.
//!
//! Biology modulates psychology but does not replace it. Hormones create
//! pressures and biases; psychology still appraises, interprets, and chooses.
//!
//! Architecture:
//! ```text
//! Genome → Endocrine Axes → Nervous System → Interoception → Emotion
//!         Metabolic → Energy → Cognition
//!         Growth → Developmental Stage → Trait Expression
//! ```

pub mod endocrine;
pub mod genome;
pub mod nervous;

use mindstrata_core::fixed::Fixed;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub use endocrine::EndocrineState;
pub use genome::Genome;
pub use nervous::NervousSystemState;

/// EmbodiedState — the full biological substrate of an agent.
///
/// Replaces the abstract `BodyState` with richer biological modeling.
/// Provides a compatibility facade so existing systems keep working.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbodiedState {
    /// Heritable genome.
    pub genome: Genome,
    /// Hormonal axes connecting body to mind.
    pub endocrine: EndocrineState,
    /// Nervous system — arousal, pain, trauma.
    pub nervous: NervousSystemState,
    /// Current health (0–1).
    pub health: Fixed,
    /// Current energy (0–1).
    pub energy: Fixed,
    /// Hunger pressure (0 = satisfied, 1 = starving).
    pub hunger: Fixed,
    /// Thirst pressure (0 = satisfied, 1 = dehydrated).
    pub thirst: Fixed,
    /// Fatigue (0 = rested, 1 = exhausted).
    pub fatigue: Fixed,
    /// Sickness (0 = healthy, 1 = severely ill).
    pub sickness: Fixed,
    /// Injury (0 = uninjured, 1 = severely injured).
    pub injury: Fixed,
    /// Fertility (None = not applicable, Some(0–1) = fertility level).
    pub fertility: Option<Fixed>,
    /// Age in years.
    pub age: Fixed,
}

impl EmbodiedState {
    /// Generate a random embodied state for a new agent.
    pub fn random(age: Fixed, rng: &mut impl Rng) -> Self {
        let genome = Genome::random(rng);
        Self {
            endocrine: EndocrineState::random(rng),
            nervous: NervousSystemState::default(),
            health: Fixed::from_f64(0.9) + Fixed::from_f64(rng.random_range(0.0..0.1)),
            energy: Fixed::from_f64(0.7) + Fixed::from_f64(rng.random_range(0.0..0.2)),
            hunger: Fixed::from_f64(0.2),
            thirst: Fixed::from_f64(0.1),
            fatigue: Fixed::from_f64(0.1),
            sickness: Fixed::ZERO,
            injury: Fixed::ZERO,
            fertility: Some(genome.fertility_predispositions.base_fertility),
            age,
            genome,
        }
    }

    // ── Compatibility facade ──────────────────────────────────────────
    // These methods allow existing systems to read BodyState-like fields
    // from the richer EmbodiedState without modification.

    /// Derived health from biological subsystems.
    /// Maps immune_strength to 0.7–1.0 range so default genome (0.6) → 0.88.
    /// This ensures default agents start near full health like the legacy BodyState.
    pub fn derived_health(&self) -> Fixed {
        let base = self.health;
        let immune_modifier = Fixed::from_f64(0.7) + self.genome.health_predispositions.immune_strength * Fixed::from_f64(0.3);
        let stress_penalty = self.endocrine.stress.level * Fixed::from_f64(0.2);
        let pain_penalty = self.nervous.pain.effective_pain() * Fixed::from_f64(0.1);
        (base * immune_modifier - stress_penalty - pain_penalty).clamp_01()
    }

    /// Derived energy from metabolic and sleep state.
    pub fn derived_energy(&self) -> Fixed {
        let base = self.energy;
        let sleep_penalty = self.nervous.sleep_pressure * Fixed::from_f64(0.3);
        let stress_penalty = self.endocrine.stress.level * Fixed::from_f64(0.1);
        let metabolic_boost = self.endocrine.metabolic.energy * Fixed::from_f64(0.2);
        (base - sleep_penalty - stress_penalty + metabolic_boost).clamp_01()
    }

    /// Derived hunger from metabolic predispositions.
    pub fn derived_hunger(&self) -> Fixed {
        let base = self.hunger;
        let sensitivity = self.genome.metabolic_predispositions.hunger_sensitivity;
        (base * sensitivity).clamp_01()
    }

    /// Derived fatigue from sleep pressure and energy.
    pub fn derived_fatigue(&self) -> Fixed {
        let base = self.fatigue;
        let sleep_factor = self.nervous.sleep_pressure * Fixed::from_f64(0.4);
        let energy_factor = (Fixed::ONE - self.energy) * Fixed::from_f64(0.3);
        (base + sleep_factor + energy_factor).clamp_01()
    }

    /// Update biological systems each tick.
    pub fn tick_update(
        &mut self,
        threat_level: Fixed,
        social_safety: Fixed,
        is_sleeping: bool,
    ) {
        // Update nervous system
        self.nervous.update(
            threat_level,
            social_safety,
            self.injury,
            is_sleeping,
        );

        // Update endocrine stress axis
        let parasympathetic = self.nervous.parasympathetic_tone;
        let acute_stress = self.nervous.pain.effective_pain()
            + self.hunger * Fixed::from_f64(0.3)
            + self.thirst * Fixed::from_f64(0.2);
        self.endocrine.stress.update(acute_stress, parasympathetic);

        // Update metabolic axis
        self.endocrine.metabolic.energy = self.energy;
        self.endocrine.metabolic.appetite = self.hunger;
        self.endocrine.metabolic.satiety = Fixed::ONE - self.hunger;

        // Update arousal from stress
        self.endocrine.arousal.update(self.endocrine.stress.level);

        // Sleep pressure management
        if is_sleeping {
            self.nervous.sleep_pressure = (self.nervous.sleep_pressure - Fixed::from_f64(0.05)).max(Fixed::ZERO);
        }
    }
}

/// Convert EmbodiedState to legacy BodyState for backward compatibility.
impl From<&EmbodiedState> for crate::person::BodyState {
    fn from(embodied: &EmbodiedState) -> Self {
        Self {
            health: embodied.derived_health(),
            energy: embodied.derived_energy(),
            hunger: embodied.hunger,
            thirst: embodied.thirst,
            fatigue: embodied.derived_fatigue(),
            sickness: embodied.sickness,
            injury: embodied.injury,
            fertility: embodied.fertility,
        }
    }
}

/// Convert legacy BodyState fields into EmbodiedState for migration.
/// Note: Sex and age cannot be recovered from BodyState; defaults are used.
/// Callers should override genome.sex and age after migration if known.
impl From<&crate::person::BodyState> for EmbodiedState {
    fn from(body: &crate::person::BodyState) -> Self {
        Self {
            genome: Genome {
                sex: genome::Sex::Male, // TODO: sex not stored in BodyState; override after migration
                trait_predispositions: genome::TraitPredispositions::default(),
                health_predispositions: genome::HealthPredispositions::default(),
                metabolic_predispositions: genome::MetabolicPredispositions::default(),
                physical_potential: genome::PhysicalPotential::default(),
                fertility_predispositions: genome::FertilityPredispositions::default(),
            },
            endocrine: EndocrineState::default(),
            nervous: NervousSystemState::default(),
            health: body.health,
            energy: body.energy,
            hunger: body.hunger,
            thirst: body.thirst,
            fatigue: body.fatigue,
            sickness: body.sickness,
            injury: body.injury,
            fertility: body.fertility,
            age: Fixed::from_f64(25.0), // TODO: age not stored in BodyState; override after migration
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn embodied_state_facade_matches_body_state() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let embodied = EmbodiedState::random(Fixed::from_f64(25.0), &mut rng);
        let body: crate::person::BodyState = (&embodied).into();
        assert_eq!(body.hunger, embodied.hunger);
        assert_eq!(body.thirst, embodied.thirst);
        assert_eq!(body.injury, embodied.injury);
    }

    #[test]
    fn body_state_migration_roundtrip() {
        let original = crate::person::BodyState::default();
        let embodied: EmbodiedState = (&original).into();
        let migrated: crate::person::BodyState = (&embodied).into();
        // Core fields should survive migration
        assert_eq!(original.hunger, migrated.hunger);
        assert_eq!(original.thirst, migrated.thirst);
    }

    #[test]
    fn derived_health_accounts_for_stress() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut embodied = EmbodiedState::random(Fixed::from_f64(25.0), &mut rng);
        let calm_health = embodied.derived_health();
        embodied.endocrine.stress.level = Fixed::from_f64(0.9);
        let stressed_health = embodied.derived_health();
        assert!(calm_health > stressed_health);
    }
}
