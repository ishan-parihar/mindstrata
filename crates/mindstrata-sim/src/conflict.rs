//! Conflict mechanics — §19.5.H of the architecture spec.
//!
//! Implements threats, intimidation, violence, combat, casualties,
//! trauma, occupation, repression, rebellion, and feuds.
//!
//! §19.5.H: "Threats, intimidation, violence, combat, casualties,
//! trauma, occupation, repression, rebellion, feuds."

use crate::person::{DiscreteEmotions, Personality};
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// Types of conflict interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictKind {
    /// Verbal threat — fear-inducing but no physical harm.
    Threat,
    /// Intimidation — repeated threats that erode will.
    Intimidation,
    /// Physical violence — direct harm.
    Violence,
    /// Organized combat — between factions or groups.
    Combat,
    /// Feud — ongoing conflict between two parties.
    Feud,
}

impl ConflictKind {
    /// Base injury severity from this conflict type.
    pub fn injury_severity(self) -> Fixed {
        match self {
            ConflictKind::Threat => Fixed::ZERO, // no physical harm
            ConflictKind::Intimidation => Fixed::from_f64(0.05),
            ConflictKind::Violence => Fixed::from_f64(0.3),
            ConflictKind::Combat => Fixed::from_f64(0.5),
            ConflictKind::Feud => Fixed::from_f64(0.2), // cumulative over time
        }
    }

    /// Trauma accumulation rate per incident.
    pub fn trauma_rate(self) -> Fixed {
        match self {
            ConflictKind::Threat => Fixed::from_f64(0.02),
            ConflictKind::Intimidation => Fixed::from_f64(0.05),
            ConflictKind::Violence => Fixed::from_f64(0.15),
            ConflictKind::Combat => Fixed::from_f64(0.25),
            ConflictKind::Feud => Fixed::from_f64(0.08),
        }
    }

    /// Fear induced in the target.
    pub fn fear_induction(self) -> Fixed {
        match self {
            ConflictKind::Threat => Fixed::from_f64(0.15),
            ConflictKind::Intimidation => Fixed::from_f64(0.25),
            ConflictKind::Violence => Fixed::from_f64(0.4),
            ConflictKind::Combat => Fixed::from_f64(0.6),
            ConflictKind::Feud => Fixed::from_f64(0.2),
        }
    }
}

/// A feud between two agents or groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feud {
    /// The two parties in conflict.
    pub party_a: AgentId,
    pub party_b: AgentId,
    /// Intensity of the feud (0 = dormant, 1 = active).
    pub intensity: Fixed,
    /// Number of incidents so far.
    pub incidents: u32,
    /// Tick when the feud started.
    pub started_tick: u64,
    /// Tick of last incident.
    pub last_incident_tick: u64,
    /// Whether the feud has been resolved.
    pub resolved: bool,
}

impl Feud {
    /// Create a new feud.
    pub fn new(party_a: AgentId, party_b: AgentId, tick: u64) -> Self {
        Self {
            party_a,
            party_b,
            intensity: Fixed::from_f64(0.5),
            incidents: 1,
            started_tick: tick,
            last_incident_tick: tick,
            resolved: false,
        }
    }

    /// Escalate the feud after an incident.
    pub fn escalate(&mut self, tick: u64) {
        self.incidents += 1;
        self.last_incident_tick = tick;
        // Intensity grows with incidents but caps at 1.0
        let escalation = Fixed::from_f64(0.1) / Fixed::from_int(self.incidents as i64);
        self.intensity = (self.intensity + escalation).clamp_01();
    }

    /// Decay feud intensity over time without incidents.
    pub fn decay(&mut self, decay_rate: Fixed) {
        self.intensity = (self.intensity - decay_rate).max(Fixed::ZERO);
        if self.intensity <= Fixed::from_f64(0.05) {
            self.resolved = true;
        }
    }
}

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

/// Compute the probability of a conflict occurring based on relationship and context.
pub fn conflict_probability(
    trust: Fixed,
    grievance: Fixed,
    fear: Fixed,
    personality_aggression: Fixed,
) -> Fixed {
    // Low trust + high grievance + low fear + aggressive personality = high probability
    let trust_factor = Fixed::ONE - trust;
    let grievance_factor = grievance;
    let fear_deterrent = Fixed::ONE - fear * Fixed::from_f64(0.5); // fear deters aggression
    let personality_factor = personality_aggression;

    (trust_factor * Fixed::from_f64(0.3)
        + grievance_factor * Fixed::from_f64(0.3)
        + fear_deterrent * Fixed::from_f64(0.2)
        + personality_factor * Fixed::from_f64(0.2))
        .clamp_01()
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
        let mut state_with_trauma = ConflictState::default();
        state_with_trauma.trauma = Fixed::from_f64(0.5);

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
    fn feud_escalates() {
        let a = AgentId::new(0);
        let b = AgentId::new(1);
        let mut feud = Feud::new(a, b, 100);
        let initial_intensity = feud.intensity;

        feud.escalate(200);
        assert!(feud.intensity > initial_intensity);
        assert_eq!(feud.incidents, 2);
    }

    #[test]
    fn feud_decays() {
        let a = AgentId::new(0);
        let b = AgentId::new(1);
        let mut feud = Feud::new(a, b, 100);
        feud.intensity = Fixed::from_f64(0.8);

        feud.decay(Fixed::from_f64(0.1));
        assert!(feud.intensity < Fixed::from_f64(0.8));
    }

    #[test]
    fn conflict_probability_increases_with_low_trust() {
        let p_low_trust = conflict_probability(
            Fixed::from_f64(0.1), // low trust
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::from_f64(0.5),
        );
        let p_high_trust = conflict_probability(
            Fixed::from_f64(0.9), // high trust
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::from_f64(0.5),
        );

        assert!(p_low_trust > p_high_trust,
            "Low trust should increase conflict probability");
    }

    #[test]
    fn fear_deters_conflict() {
        let p_fearful = conflict_probability(
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.8), // high fear
            Fixed::from_f64(0.5),
        );
        let p_brave = conflict_probability(
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::ZERO, // no fear
            Fixed::from_f64(0.5),
        );

        assert!(p_brave > p_fearful,
            "Fear should deter conflict");
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
        let mut state = ConflictState::default();
        state.combat_fatigue = Fixed::from_f64(0.5);
        state.in_combat = false;

        state.update();
        assert!(state.combat_fatigue < Fixed::from_f64(0.5));
    }
}
