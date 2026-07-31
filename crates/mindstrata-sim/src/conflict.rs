//! Conflict mechanics — §19.5.H of the architecture spec.
//!
//! Implements threats, intimidation, violence, combat, casualties,
//! trauma, occupation, repression, rebellion, and feuds.
//!
//! §19.5.H: "Threats, intimidation, violence, combat, casualties,
//! trauma, occupation, repression, rebellion, feuds."

use crate::person::{DiscreteEmotions, Personality};
use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

// ConflictKind is now defined in mindstrata-core and re-exported here for backwards compatibility.
pub use mindstrata_core::ConflictKind;

/// Conflict tracking state for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictState {
    /// Accumulated trauma from all conflicts (0 = no trauma, 1 = severe).
    pub trauma: Fixed,
    /// Number of conflicts this agent has been involved in.
    pub conflict_count: u32,
    /// Number of injuries received.
    pub injuries_received: u32,
    /// Number of casualties caused.
    pub casualties_caused: u32,
    /// Whether the agent is currently in combat.
    pub in_combat: bool,
    /// Combat fatigue (reduces effectiveness in ongoing combat).
    pub combat_fatigue: Fixed,
}

impl Default for ConflictState {
    fn default() -> Self {
        Self {
            trauma: Fixed::ZERO,
            conflict_count: 0,
            injuries_received: 0,
            casualties_caused: 0,
            in_combat: false,
            combat_fatigue: Fixed::ZERO,
        }
    }
}

impl ConflictState {
    /// Update conflict state each tick.
    pub fn update(&mut self) {
        // Combat fatigue decays when not in combat
        if !self.in_combat {
            self.combat_fatigue = (self.combat_fatigue - Fixed::from_f64(0.02)).max(Fixed::ZERO);
        }
        // Trauma decays very slowly (years to recover)
        self.trauma = (self.trauma - Fixed::from_f64(0.0001)).max(Fixed::ZERO);
    }

    /// Record a conflict incident.
    pub fn record_conflict(&mut self, kind: ConflictKind) {
        self.conflict_count += 1;
        self.trauma = (self.trauma + kind.trauma_rate()).clamp_01();
        if kind == ConflictKind::Combat || kind == ConflictKind::Violence {
            self.in_combat = true;
            self.combat_fatigue = (self.combat_fatigue + Fixed::from_f64(0.1)).clamp_01();
        }
    }

    /// Record an injury received.
    pub fn record_injury(&mut self) {
        self.injuries_received += 1;
    }

    /// Record a casualty caused.
    pub fn record_casualty(&mut self) {
        self.casualties_caused += 1;
    }
}

/// Result of a conflict interaction.
#[derive(Debug, Clone)]
pub struct ConflictResult {
    /// Whether the conflict occurred.
    pub occurred: bool,
    /// Kind of conflict.
    pub kind: ConflictKind,
    /// Injury applied to target (0 = no injury).
    pub injury: Fixed,
    /// Fear induced in target.
    pub fear_induced: Fixed,
    /// Trauma accumulated by target.
    pub trauma_accumulated: Fixed,
    /// Whether the target was killed.
    pub target_died: bool,
}

/// Resolve a conflict between two agents.
///
/// Computes injury, fear, trauma, and whether the target dies.
pub fn resolve_conflict(
    kind: ConflictKind,
    aggressor_personality: &Personality,
    target_emotions: &DiscreteEmotions,
    target_health: Fixed,
    target_conflict_state: &ConflictState,
    rng_value: Fixed,
) -> ConflictResult {
    let base_injury = kind.injury_severity();

    // Personality modifiers
    let aggression = aggressor_personality.dominance * Fixed::from_f64(0.3)
        + aggressor_personality.risk_tolerance * Fixed::from_f64(0.2);
    let injury = (base_injury + aggression * Fixed::from_f64(0.1)).clamp_01();

    // Fear induction: already fearful targets are more affected
    let fear_sensitivity = target_emotions.fear * Fixed::from_f64(0.3) + Fixed::from_f64(0.7);
    let fear_induced = (kind.fear_induction() * fear_sensitivity).clamp_01();

    // Trauma: prior trauma makes new trauma worse (PTSD-like)
    let trauma_multiplier = Fixed::ONE + target_conflict_state.trauma * Fixed::from_f64(0.5);
    let trauma = (kind.trauma_rate() * trauma_multiplier).clamp_01();

    // Casualty check: very low health + severe injury = death
    let lethal = injury > Fixed::from_f64(0.3)
        && target_health < Fixed::from_f64(0.2)
        && rng_value < Fixed::from_f64(0.3);

    ConflictResult {
        occurred: true,
        kind,
        injury,
        fear_induced,
        trauma_accumulated: trauma,
        target_died: lethal,
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn make_personality() -> Personality {
        Personality {
            openness: Fixed::from_f64(0.5),
            conscientiousness: Fixed::from_f64(0.5),
            extraversion: Fixed::from_f64(0.5),
            agreeableness: Fixed::from_f64(0.5),
            neuroticism: Fixed::from_f64(0.5),
            risk_tolerance: Fixed::from_f64(0.5),
            conformity: Fixed::from_f64(0.5),
            ambition: Fixed::from_f64(0.5),
            altruism: Fixed::from_f64(0.5),
            traditionalism: Fixed::from_f64(0.5),
            dominance: Fixed::from_f64(0.5),
            impulsivity: Fixed::from_f64(0.5),
        }
    }

    fn make_emotions() -> DiscreteEmotions {
        DiscreteEmotions {
            fear: Fixed::from_f64(0.2),
            anger: Fixed::from_f64(0.1),
            joy: Fixed::from_f64(0.3),
            sadness: Fixed::ZERO,
            trust: Fixed::from_f64(0.5),
            shame: Fixed::ZERO,
            pride: Fixed::ZERO,
            guilt: Fixed::ZERO,
        }
    }

    #[test]
    fn conflict_severity_increases_with_kind() {
        assert!(ConflictKind::Threat.injury_severity() < ConflictKind::Violence.injury_severity());
        assert!(ConflictKind::Violence.injury_severity() < ConflictKind::Combat.injury_severity());
    }

    #[test]
    fn trauma_accumulates() {
        let mut state = ConflictState::default();
        state.record_conflict(ConflictKind::Violence);
        assert!(state.trauma > Fixed::ZERO);
        assert_eq!(state.conflict_count, 1);
    }

    #[test]
    fn trauma_multiplier_increases_with_prior_trauma() {
        let personality = make_personality();
        let emotions = make_emotions();
        let state_no_trauma = ConflictState::default();
        let state_with_trauma = ConflictState {
            trauma: Fixed::from_f64(0.5),
            ..ConflictState::default()
        };

        let r1 = resolve_conflict(
            ConflictKind::Violence,
            &personality,
            &emotions,
            Fixed::from_f64(0.8),
            &state_no_trauma,
            Fixed::from_f64(0.5),
        );
        let r2 = resolve_conflict(
            ConflictKind::Violence,
            &personality,
            &emotions,
            Fixed::from_f64(0.8),
            &state_with_trauma,
            Fixed::from_f64(0.5),
        );

        assert!(r2.trauma_accumulated > r1.trauma_accumulated,
            "Prior trauma should increase new trauma accumulation");
    }

    #[test]
    fn lethal_check_requires_low_health() {
        let personality = make_personality();
        let emotions = make_emotions();
        let state = ConflictState::default();

        // High health: should not die
        let r_healthy = resolve_conflict(
            ConflictKind::Combat,
            &personality,
            &emotions,
            Fixed::from_f64(0.9), // high health
            &state,
            Fixed::from_f64(0.1), // favorable rng
        );
        assert!(!r_healthy.target_died, "Healthy agents should not die easily");

        // Low health: might die
        let mut deaths = 0;
        for _ in 0..100 {
            let rng_val = Fixed::from_f64(0.1);
            let r = resolve_conflict(
                ConflictKind::Combat,
                &personality,
                &emotions,
                Fixed::from_f64(0.1), // low health
                &state,
                rng_val,
            );
            if r.target_died {
                deaths += 1;
            }
        }
        assert!(deaths > 0, "Low health agents should sometimes die in combat");
    }

    #[test]
    fn combat_fatigue_accumulates() {
        let mut state = ConflictState::default();
        state.record_conflict(ConflictKind::Combat);
        assert!(state.combat_fatigue > Fixed::ZERO);
        assert!(state.in_combat);
    }

    #[test]
    fn combat_fatigue_decays_when_not_in_combat() {
        let mut state = ConflictState {
            combat_fatigue: Fixed::from_f64(0.5),
            in_combat: false,
            ..ConflictState::default()
        };

        state.update();
        assert!(state.combat_fatigue < Fixed::from_f64(0.5));
    }
}
