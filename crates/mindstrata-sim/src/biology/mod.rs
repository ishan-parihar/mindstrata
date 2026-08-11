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
//!         Cardiovascular → Stamina → Physical Agency
//!         Immune → Disease Resistance → Recovery
//!         Respiratory → Exertion Limits → Endurance
//!         Musculoskeletal → Strength → Labor Capacity
//!         Reproductive → Fertility → Pair-Bonding → Kinship
//!         Circadian → Sleep/Wake → Alertness → Rhythm
//!         Development → Life Stage → Aging → Milestones
//! ```

pub mod cardiovascular;
pub mod circadian;
pub mod development;
pub mod digestive;
pub mod endocrine;
pub mod genome;
pub mod immune;
pub mod metabolism;
pub mod musculoskeletal;
pub mod nervous;
pub mod reproductive;
pub mod respiratory;
pub mod skeletal;
pub mod thermal;

use mindstrata_core::fixed::Fixed;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub use cardiovascular::CardiovascularState;
pub use circadian::CircadianState;
pub use development::DevelopmentalState;
pub use digestive::DigestiveState;
pub use endocrine::EndocrineState;
pub use genome::Genome;
pub use immune::ImmuneState;
pub use metabolism::MetabolicState;
pub use musculoskeletal::MuscularState;
pub use nervous::NervousSystemState;
pub use reproductive::ReproductiveState;
pub use respiratory::RespiratoryState;
pub use skeletal::SkeletalState;
pub use thermal::ThermalState;

/// EmbodiedState — the full biological substrate of an agent.
///
/// Replaces the abstract `BodyState` with richer biological modeling.
/// Provides a compatibility facade so existing systems keep working.
///
/// The subsystems interact:
/// ```text
/// Genome → predispositions for all subsystems
/// Endocrine → modulates psychology via hormone axes
/// Nervous → arousal, pain, trauma → feeds interoception
/// Metabolic → energy, hunger, thermoregulation
/// Cardiovascular → stamina, shock risk, fitness
/// Respiratory → exertion limits, disease vulnerability
/// Immune → disease resistance, inflammation
/// Musculoskeletal → strength, fatigue, conditioning
/// Reproductive → fertility, puberty, pregnancy
/// Circadian → sleep/wake cycle, alertness
/// Development → life stage, aging, milestones
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbodiedState {
    /// Heritable genome.
    pub genome: Genome,
    /// Hormonal axes connecting body to mind.
    pub endocrine: EndocrineState,
    /// Nervous system — arousal, pain, trauma.
    pub nervous: NervousSystemState,
    /// Metabolic system — energy, hunger, hydration (thermoregulation now
    /// lives in the dedicated `thermal: ThermalState` field, Iter 45).
    pub metabolic: MetabolicState,
    /// Cardiovascular system — stamina, shock, fitness.
    pub cardiovascular: CardiovascularState,
    /// Respiratory system — exertion, disease vulnerability.
    pub respiratory: RespiratoryState,
    /// Immune system — disease resistance, inflammation.
    pub immune: ImmuneState,
    /// Musculoskeletal system — strength, fatigue, conditioning.
    pub muscular: MuscularState,
    /// Skeletal system (§7.2.3) — frame, bone density, integrity, fracture risk,
    /// chronic pain. Exactly neutral at baseline (identity multipliers); only
    /// elder frailty, severe injury, or malnutrition move it.
    /// `#[serde(default)]`: added in Iter 38, so snapshots captured before it
    /// still deserialize (deterministic-replay mandate for old saves).
    #[serde(default)]
    pub skeletal: SkeletalState,
    /// Digestive system (§7.2.7) — stomach contents, absorption, gut health,
    /// nausea, satiety. Exactly neutral at baseline; only spoiled food or
    /// prolonged under-eating moves it.
    /// `#[serde(default)]`: added in Iter 38, so snapshots captured before it
    /// still deserialize (deterministic-replay mandate for old saves).
    #[serde(default)]
    pub digestive: DigestiveState,
    /// Reproductive system — fertility, puberty, pregnancy.
    pub reproductive: ReproductiveState,
    /// Circadian system — sleep/wake cycle, alertness.
    pub circadian: CircadianState,
    /// Thermoregulation (§7.3.3) — body temperature, cold/heat stress.
    /// `#[serde(default)]`: added in Iter 45 (extracted from MetabolicState),
    /// so pre-Iter-45 snapshots still restore.
    #[serde(default)]
    pub thermal: ThermalState,
    /// Developmental system — life stage, aging, milestones.
    pub development: DevelopmentalState,
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
        let mut development = DevelopmentalState::default();
        development.age = age;
        development.life_stage = crate::biology::development::LifeStage::from_age(age);

        Self {
            endocrine: EndocrineState::random(rng),
            nervous: NervousSystemState::default(),
            metabolic: MetabolicState::default(),
            cardiovascular: CardiovascularState::default(),
            respiratory: RespiratoryState::default(),
            immune: ImmuneState::default(),
            muscular: MuscularState::default(),
            skeletal: SkeletalState::from_age(age),
            digestive: DigestiveState::default(),
            reproductive: ReproductiveState {
                sex: match genome.sex {
                    genome::Sex::Male => reproductive::BiologicalSex::Male,
                    genome::Sex::Female => reproductive::BiologicalSex::Female,
                },
                ..ReproductiveState::default()
            },
            circadian: CircadianState::default(),
            thermal: ThermalState::default(),
            development,
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
    ///
    /// §7.2.3: Multiplied by the skeletal `health_factor` — exactly 1.0 for a
    /// healthy adult, so calibrated runs are untouched; elder frailty, chronic
    /// pain, or compromised structural integrity drain health.
    pub fn derived_health(&self) -> Fixed {
        let base = self.health;
        let immune_modifier = Fixed::from_f64(0.7)
            + self.genome.health_predispositions.immune_strength * Fixed::from_f64(0.3);
        let stress_penalty = self.endocrine.stress.level * Fixed::from_f64(0.2);
        let pain_penalty = self.nervous.pain.effective_pain() * Fixed::from_f64(0.1);
        let sickness_penalty = self.immune.sickness_level() * Fixed::from_f64(0.15);
        let shock_penalty = self.cardiovascular.shock_risk * Fixed::from_f64(0.1);
        let skeletal_factor = self.skeletal.health_factor();
        ((base * immune_modifier
            - stress_penalty
            - pain_penalty
            - sickness_penalty
            - shock_penalty)
            * skeletal_factor)
            .clamp_01()
    }

    /// Derived energy from metabolic, cardiovascular, and sleep state.
    ///
    /// §7.2.7: Multiplied by the digestive `effective_digestion` — exactly 1.0
    /// for a healthy gut, so calibrated runs are untouched; spoiled food or
    /// gut damage depress available energy.
    pub fn derived_energy(&self) -> Fixed {
        let base = self.energy;
        let sleep_penalty = self.nervous.sleep_pressure * Fixed::from_f64(0.3);
        let stress_penalty = self.endocrine.stress.level * Fixed::from_f64(0.1);
        let metabolic_boost = self.endocrine.metabolic.energy * Fixed::from_f64(0.2);
        let cardio_boost = self.cardiovascular.effective_stamina() * Fixed::from_f64(0.1);
        let fatigue_penalty = self.muscular.fatigue * Fixed::from_f64(0.15);
        let digestion_factor = self.digestive.effective_digestion();
        ((base - sleep_penalty - stress_penalty + metabolic_boost + cardio_boost - fatigue_penalty)
            * digestion_factor)
            .clamp_01()
    }

    /// Derived hunger from metabolic predispositions.
    pub fn derived_hunger(&self) -> Fixed {
        let base = self.hunger;
        let sensitivity = self.genome.metabolic_predispositions.hunger_sensitivity;
        let satiety_reduce = self.metabolic.satiety * Fixed::from_f64(0.3);
        ((base * sensitivity - satiety_reduce).max(Fixed::ZERO)).clamp_01()
    }

    /// Derived fatigue from sleep pressure, muscular fatigue, and energy.
    pub fn derived_fatigue(&self) -> Fixed {
        let base = self.fatigue;
        let sleep_factor = self.nervous.sleep_pressure * Fixed::from_f64(0.4);
        let energy_factor = (Fixed::ONE - self.energy) * Fixed::from_f64(0.3);
        let muscular_factor = self.muscular.fatigue * Fixed::from_f64(0.2);
        (base + sleep_factor + energy_factor + muscular_factor).clamp_01()
    }

    /// Update all biological systems each tick.
    ///
    /// This is the main biological update loop. It updates subsystems in causal order:
    /// 1. Circadian (time of day)
    /// 2. Nervous (arousal, pain, trauma)
    /// 3. Endocrine (hormonal axes)
    /// 4. Metabolic (energy, hunger, thermoregulation)
    /// 5. Cardiovascular (fitness, shock)
    /// 6. Respiratory (exertion, disease)
    /// 7. Immune (infection, inflammation)
    /// 8. Musculoskeletal (strength, fatigue)
    /// 9. Reproductive (fertility, pregnancy)
    /// 10. Development (aging, milestones)
    pub fn tick_update(
        &mut self,
        threat_level: Fixed,
        social_safety: Fixed,
        is_sleeping: bool,
        activity_level: Fixed,
        ambient_temperature: Fixed,
        crowding: Fixed,
        hygiene: Fixed,
        params: &crate::parameters::SimParameters,
    ) {
        // 1. Circadian — advances time of day
        self.circadian.tick_update(144, is_sleeping); // 144 ticks per day

        // 2. Nervous — arousal, pain, trauma
        self.nervous.update(
            threat_level,
            social_safety,
            self.injury,
            is_sleeping,
            nervous::NervousUpdateParams {
                sympathetic_recovery_rate: params.nervous_sympathetic_recovery,
                parasympathetic_buildup_rate: params.nervous_parasympathetic_buildup,
                trauma_accumulation_rate: params.nervous_trauma_accumulation,
                trauma_decay_rate: params.nervous_trauma_decay,
            },
        );

        // 3. Endocrine — hormonal axes
        let parasympathetic = self.nervous.parasympathetic_tone;
        let acute_stress = self.nervous.pain.effective_pain()
            + self.hunger * Fixed::from_f64(0.3)
            + self.thirst * Fixed::from_f64(0.2);
        self.endocrine.stress.update(
            acute_stress,
            parasympathetic,
            params.endocrine_stress_recovery,
            params.endocrine_stress_chronic_rate,
            params.endocrine_stress_chronic_recovery,
        );
        self.endocrine.metabolic.energy = self.energy;
        self.endocrine.metabolic.appetite = self.hunger;
        self.endocrine.metabolic.satiety = Fixed::ONE - self.hunger;
        self.endocrine.arousal.update(
            self.endocrine.stress.level,
            params.endocrine_arousal_rise,
            params.endocrine_arousal_decay,
        );

        // 4. Metabolic — energy, hunger (thermoregulation now in ThermalState)
        self.metabolic.tick_update(activity_level);

        // 4b. Thermal — body temperature homeostasis, cold/heat stress.
        // Runs immediately after metabolic with the post-update energy
        // reserves (identical to the pre-Iter-45 inlined block).
        self.thermal
            .tick_update(ambient_temperature, self.metabolic.energy_reserves);

        // 5. Cardiovascular — fitness, shock
        let age_modifier = self.development.physical_modifier();
        self.cardiovascular.tick_update(
            activity_level,
            self.injury,
            self.endocrine.stress.level,
            Fixed::from_f64(0.6), // nutrition quality placeholder
            age_modifier,
        );

        // 6. Respiratory — exertion, disease
        self.respiratory.tick_update(
            activity_level,
            self.thermal.cold_stress,
            Fixed::ZERO, // smoke exposure (placeholder)
            Fixed::ZERO, // damp housing (placeholder)
            age_modifier,
        );

        // 7. Immune — infection, inflammation
        let sleep_quality = if is_sleeping {
            Fixed::from_f64(0.8)
        } else {
            Fixed::from_f64(0.3)
        };
        self.immune.tick_update(
            Fixed::from_f64(0.6), // nutrition quality placeholder
            self.endocrine.stress.level,
            sleep_quality,
            self.injury,
            crowding,
            hygiene,
            age_modifier,
        );

        // 8. Musculoskeletal — strength, fatigue
        self.muscular.tick_update(
            activity_level,
            Fixed::from_f64(0.6), // nutrition quality placeholder
            if is_sleeping {
                Fixed::ONE
            } else {
                Fixed::from_f64(0.2)
            },
            age_modifier,
        );

        // 8b. Skeletal (§7.2.3) — frailty, fracture, malnutrition
        self.skeletal.tick_update(
            self.age,
            self.injury,
            Fixed::from_f64(0.6), // nutrition quality placeholder
        );

        // 8c. Digestive (§7.2.7) — stomach processing, gut health
        self.digestive.tick_update();

        // 9. Reproductive — fertility, pregnancy
        let bonding = self.endocrine.bonding.level;
        self.reproductive.tick_update(
            self.age,
            self.derived_health(),
            self.endocrine.stress.level,
            bonding,
            Fixed::from_f64(0.6), // nutrition placeholder
            reproductive::ReproductiveUpdateParams {
                stress_suppression: params.reproduction_stress_suppression,
                age_decline_rate: params.reproduction_age_decline_rate,
                gestation_rate_mult: params.reproduction_gestation_rate,
            },
        );

        // 10. Development — aging (advance age every 144 ticks = 1 day)
        // Note: age advancement is handled externally by the simulation tick loop
        // to avoid double-counting. Development only updates environment quality here.
        self.development.update_environment(
            Fixed::from_f64(0.6), // nutrition
            social_safety,
            social_safety, // social support ≈ social safety for now
        );

        // Sleep pressure management
        if is_sleeping {
            self.nervous.sleep_pressure =
                (self.nervous.sleep_pressure - Fixed::from_f64(0.05)).max(Fixed::ZERO);
        }

        // Sync derived fields back to legacy fields.
        // NOTE: base health is NOT overwritten with the acute derived value.
        // `derived_health()` subtracts stress/pain/sickness/shock penalties, and
        // writing it back each tick fed those penalties into the base — since
        // immune_modifier < 1.0, any sustained stress ratcheted health to 0
        // permanently (the avg_health metric read ~0.06 while agents were
        // actually at ~1.0). Base health now recovers slowly and only chronic
        // factors (injury, sickness) reduce it; acute penalties remain visible
        // transiently through derived_health().
        let chronic_damage = (self.injury * Fixed::from_f64(0.5)
            + self.immune.sickness_level() * Fixed::from_f64(0.3))
        .clamp_01();
        let health_target = (Fixed::ONE - chronic_damage).clamp_01();
        let recovery_rate = Fixed::from_f64(0.002); // slow recovery per tick
        self.health = (self.health + (health_target - self.health) * recovery_rate).clamp_01();
        self.energy = self.derived_energy();
        self.fatigue = self.derived_fatigue();
        self.sickness = self.immune.sickness_level();
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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn derived_health_matches_body_state_fields() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let embodied = EmbodiedState::random(Fixed::from_f64(25.0), &mut rng);
        let body: crate::person::BodyState = (&embodied).into();
        assert_eq!(body.hunger, embodied.hunger);
        assert_eq!(body.thirst, embodied.thirst);
        assert_eq!(body.injury, embodied.injury);
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

    #[test]
    fn tick_update_runs_all_subsystems() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut embodied = EmbodiedState::random(Fixed::from_f64(25.0), &mut rng);
        embodied.tick_update(
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.7),
            false,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.7),
            &crate::parameters::SimParameters::default(),
        );
        // Should not panic and values should remain in range
        assert!(derived_health_in_range(&embodied));
    }

    #[test]
    fn tick_update_sleep_reduces_pressure() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut embodied = EmbodiedState::random(Fixed::from_f64(25.0), &mut rng);
        embodied.nervous.sleep_pressure = Fixed::from_f64(0.8);
        embodied.tick_update(
            Fixed::ZERO,
            Fixed::ONE,
            true,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::ONE,
            &crate::parameters::SimParameters::default(),
        );
        assert!(embodied.nervous.sleep_pressure < Fixed::from_f64(0.8));
    }

    fn derived_health_in_range(e: &EmbodiedState) -> bool {
        let h = e.derived_health();
        h >= Fixed::ZERO && h <= Fixed::ONE
    }
}
