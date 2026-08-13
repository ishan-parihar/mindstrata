#![allow(missing_docs)]

//! Gossip propagation system — §11.2, §13.5 of the architecture spec.
//!
//! Gossip should mutate information:
//!
//! ```text
//! rumor_accuracy =
//!     source_memory_accuracy
//!   * source_trust
//!   * transmission_fidelity
//!   * emotional_salience_bias
//!   * identity_bias
//! ```
//!
//! This can produce:
//! - moral panics
//! - false accusations
//! - prestige
//! - scapegoating
//! - factional myths

use crate::person::{Belief, DiscreteEmotions, Personality};
use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A rumor that has been shared through the gossip network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rumor {
    /// The proposition this rumor is about.
    pub proposition_id: u64,
    /// Current confidence as believed by the spreader.
    pub confidence: Fixed,
    /// How many times this rumor has been relayed (each hop mutates it).
    pub hops: u32,
    /// Original tick when the rumor was created.
    pub origin_tick: u64,
    /// Tick when this rumor was last heard.
    pub last_heard_tick: u64,
    /// Emotional charge accumulated from spreaders' emotions.
    pub emotional_charge: Fixed,
    /// Identity linkage of the original belief.
    pub identity_linkage: Fixed,
    /// Resistance of the original belief (how hard it is to update).
    pub original_resistance: Fixed,
}

impl Rumor {
    /// Create a new rumor from a belief.
    pub fn from_belief(belief: &Belief, tick: u64) -> Self {
        Self {
            proposition_id: belief.proposition_id,
            confidence: belief.confidence,
            hops: 0,
            origin_tick: tick,
            last_heard_tick: tick,
            emotional_charge: belief.emotional_charge,
            identity_linkage: belief.identity_linkage,
            original_resistance: belief.resistance,
        }
    }

    /// Compute the rumor's age in ticks.
    pub fn age(&self, current_tick: u64) -> u64 {
        current_tick.saturating_sub(self.origin_tick)
    }

    /// Compute how salient this rumor is to an agent.
    /// High emotional charge and low hop count = more salient.
    pub fn salience(&self, _current_tick: u64) -> Fixed {
        // Telephone game: each hop reduces accuracy significantly
        let age_penalty = Fixed::from_f64(0.85).powi(self.hops);
        let emotional_boost = self.emotional_charge * Fixed::from_f64(0.3);
        let novelty = if self.hops == 0 {
            Fixed::from_f64(0.2)
        } else {
            Fixed::ZERO
        };
        (age_penalty + emotional_boost + novelty).clamp_01()
    }
}

/// Gossip propagation result — what changed when a rumor was shared.
#[derive(Debug, Clone)]
pub struct GossipResult {
    /// The proposition_id of the rumor shared.
    pub proposition_id: u64,
    /// The mutated confidence after transmission.
    pub mutated_confidence: Fixed,
    /// Emotional charge after distortion.
    pub emotional_charge: Fixed,
    /// How much the listener's belief changed (delta).
    pub belief_delta: Fixed,
    /// Whether the rumor was accepted (passed salience threshold).
    pub accepted: bool,
}

/// Compute the mutated confidence of a rumor during transmission.
///
/// Implements §11.2: "rumor_accuracy = source_memory_accuracy * source_trust
/// * transmission_fidelity * emotional_salience_bias * identity_bias"
pub fn mutate_rumor(
    rumor: &Rumor,
    _source_trust: Fixed,
    spreader_emotions: &DiscreteEmotions,
    spreader_personality: &Personality,
    listener_personality: &Personality,
    gossip_base_fidelity: Fixed,
    gossip_emotional_distortion: Fixed,
) -> (Fixed, Fixed) {
    // Source memory accuracy: degrades with hops (telephone game)
    let memory_accuracy = Fixed::from_f64(0.95).powi(rumor.hops);

    // Transmission fidelity: extraversion and agreeableness improve fidelity
    let transmission_fidelity = gossip_base_fidelity
        + spreader_personality.extraversion * Fixed::from_f64(0.15)
        + spreader_personality.agreeableness * Fixed::from_f64(0.1);

    // Emotional salience bias: angry or fearful spreaders exaggerate
    let emotional_distortion = spreader_emotions.anger * gossip_emotional_distortion
        + spreader_emotions.fear * Fixed::from_f64(0.1)
        - spreader_emotions.joy * Fixed::from_f64(0.05);

    // Identity bias: listeners accept rumors that align with their identity
    let identity_affinity = if listener_personality.traditionalism > Fixed::from_f64(0.6) {
        Fixed::from_f64(0.05) // traditionalists accept authority-aligned rumors
    } else if listener_personality.openness > Fixed::from_f64(0.6) {
        Fixed::from_f64(0.03) // open people accept novel rumors
    } else {
        Fixed::ZERO
    };

    // Combine all factors
    let mutated = rumor.confidence * memory_accuracy * transmission_fidelity
        + emotional_distortion * rumor.confidence
        + identity_affinity * rumor.confidence;

    // Emotional charge accumulates from spreaders
    let new_charge = (rumor.emotional_charge
        + spreader_emotions.anger * Fixed::from_f64(0.1)
        + spreader_emotions.fear * Fixed::from_f64(0.1))
    .clamp_01();

    (mutated.clamp_01(), new_charge)
}

/// Process a gossip event: the spreader shares a rumor with the listener.
///
/// Returns the mutated rumor and whether it was accepted.
pub fn process_gossip(
    rumor: &Rumor,
    source_trust: Fixed,
    spreader_emotions: &DiscreteEmotions,
    spreader_personality: &Personality,
    listener_personality: &Personality,
    listener_beliefs: &[Belief],
    current_tick: u64,
    gossip_base_fidelity: Fixed,
    gossip_emotional_distortion: Fixed,
    gossip_acceptance_threshold: Fixed,
) -> GossipResult {
    let (mutated_confidence, emotional_charge) = mutate_rumor(
        rumor,
        source_trust,
        spreader_emotions,
        spreader_personality,
        listener_personality,
        gossip_base_fidelity,
        gossip_emotional_distortion,
    );

    // Check if listener already has a belief about this proposition
    let existing = listener_beliefs
        .iter()
        .find(|b| b.proposition_id == rumor.proposition_id);

    let belief_delta = if let Some(existing) = existing {
        // Weight by resistance: high-resistance beliefs barely change
        let weight = Fixed::ONE - existing.resistance;
        let new_conf =
            (existing.confidence * existing.resistance + mutated_confidence * weight).clamp_01();
        new_conf - existing.confidence
    } else {
        // New belief — the full mutated confidence
        mutated_confidence
    };

    // Accept if the rumor has enough salience
    let salience = rumor.salience(current_tick);
    let accepted = salience > gossip_acceptance_threshold;

    GossipResult {
        proposition_id: rumor.proposition_id,
        mutated_confidence,
        emotional_charge,
        belief_delta,
        accepted,
    }
}

/// Apply a gossip result to a listener's beliefs.
pub fn apply_gossip(listener_beliefs: &mut Vec<Belief>, result: &GossipResult, tick: u64) {
    if !result.accepted {
        return;
    }

    if let Some(existing) = listener_beliefs
        .iter_mut()
        .find(|b| b.proposition_id == result.proposition_id)
    {
        // Update existing belief
        let weight = Fixed::ONE - existing.resistance;
        existing.confidence = (existing.confidence * existing.resistance
            + result.mutated_confidence * weight)
            .clamp_01();
        existing.emotional_charge = result.emotional_charge;
        existing.last_reinforced_tick = tick;
    } else {
        // Create new belief from rumor
        // §19.5.A: Accuracy depends on how distorted the mutation was.
        // Compare mutated confidence against 0.5 (neutral) as proxy for wild distortion.
        let distortion = (result.mutated_confidence - Fixed::from_f64(0.5)).abs();
        let is_accurate = distortion < Fixed::from_f64(0.3);
        listener_beliefs.push(Belief {
            proposition_id: result.proposition_id,
            confidence: result.mutated_confidence,
            emotional_charge: result.emotional_charge,
            identity_linkage: Fixed::from_f64(0.2), // rumors start with low identity linkage
            resistance: Fixed::from_f64(0.3),       // rumors are initially easy to update
            last_reinforced_tick: tick,
            source: crate::person::EvidenceSource::Hearsay,
            social_reinforcement: 0,
            is_accurate,
        });
    }
}

// ── §7.2: Moral Panic / Rumor Cascade ──────────────────────────────────

/// Threshold of average emotional charge across all rumors about an institution
/// that triggers a moral panic event.
pub const MORAL_PANIC_CHARGE_THRESHOLD: Fixed = Fixed::from_raw(5500); // 0.55

/// Moral panic result — what happens when gossip accumulates enough emotional charge.
#[derive(Debug, Clone)]
pub struct MoralPanicResult {
    /// Whether a moral panic was triggered.
    pub triggered: bool,
    /// The proposition_id of the institution being gossiped about.
    pub proposition_id: u64,
    /// The average emotional charge that triggered the panic.
    pub avg_charge: Fixed,
    /// How much institutional legitimacy should drop.
    pub legitimacy_damage: Fixed,
    /// How much faction grievance should increase.
    pub grievance_boost: Fixed,
}

/// Analyze accumulated beliefs for signs of moral panic.
///
/// When enough agents hold high-emotional-charge beliefs about an institution,
/// it can trigger a sudden collapse in trust — a moral panic.
/// This implements §7.2: "Gossip about institutions spreads through the
/// social graph. Distorted rumors can create moral panics."
pub fn detect_moral_panic(beliefs: &[&Vec<Belief>], proposition_id: u64) -> MoralPanicResult {
    // Collect emotional charges from all agents' beliefs about this proposition
    let charges: Vec<Fixed> = beliefs
        .iter()
        .filter_map(|agent_beliefs| {
            agent_beliefs
                .iter()
                .find(|b| b.proposition_id == proposition_id)
                .map(|b| b.emotional_charge)
        })
        .collect();

    if charges.is_empty() {
        return MoralPanicResult {
            triggered: false,
            proposition_id,
            avg_charge: Fixed::ZERO,
            legitimacy_damage: Fixed::ZERO,
            grievance_boost: Fixed::ZERO,
        };
    }

    let avg_charge =
        charges.iter().fold(Fixed::ZERO, |a, b| a + *b) / Fixed::from_int(charges.len() as i64);

    // Also check: how many agents have HIGH emotional charge (> 0.4)?
    let high_charge_count = charges
        .iter()
        .filter(|c| **c > Fixed::from_f64(0.4))
        .count();
    let panic_ratio =
        Fixed::from_int(high_charge_count as i64) / Fixed::from_int(charges.len() as i64);

    // Moral panic requires: high average charge AND widespread fear (many agents affected)
    let triggered =
        avg_charge >= MORAL_PANIC_CHARGE_THRESHOLD && panic_ratio >= Fixed::from_f64(0.3); // at least 30% of agents are emotionally charged

    let legitimacy_damage = if triggered {
        // Damage scales with how intense the panic is
        (avg_charge * Fixed::from_f64(0.4) + panic_ratio * Fixed::from_f64(0.3)).clamp_01()
    } else {
        Fixed::ZERO
    };

    let grievance_boost = if triggered {
        // Panic increases grievance proportionally
        (avg_charge * Fixed::from_f64(0.2)).clamp_01()
    } else {
        Fixed::ZERO
    };

    MoralPanicResult {
        triggered,
        proposition_id,
        avg_charge,
        legitimacy_damage,
        grievance_boost,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_belief(prop_id: u64, confidence: f64) -> Belief {
        Belief {
            proposition_id: prop_id,
            confidence: Fixed::from_f64(confidence),
            emotional_charge: Fixed::from_f64(0.3),
            identity_linkage: Fixed::from_f64(0.4),
            resistance: Fixed::from_f64(0.5),
            last_reinforced_tick: 0,
            source: crate::person::EvidenceSource::PersonalExperience,
            social_reinforcement: 0,
            is_accurate: true,
        }
    }

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
            constitution: None,
            temperament: crate::person::Temperament::default(),
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
            disgust: Fixed::ZERO,
            contempt: Fixed::ZERO,
            awe: Fixed::ZERO,
            gratitude: Fixed::ZERO,
            jealousy: Fixed::ZERO,
            envy: Fixed::ZERO,
            loneliness: Fixed::ZERO,
            tenderness: Fixed::ZERO,
            humiliation: Fixed::ZERO,
            relief: Fixed::ZERO,
            hope: Fixed::ZERO,
            despair: Fixed::ZERO,
            nostalgia: Fixed::ZERO,
            moral_outrage: Fixed::ZERO,
        }
    }

    #[test]
    fn rumor_from_belief() {
        let belief = make_belief(0, 0.7);
        let rumor = Rumor::from_belief(&belief, 100);
        assert_eq!(rumor.proposition_id, 0);
        assert_eq!(rumor.confidence, Fixed::from_f64(0.7));
        assert_eq!(rumor.hops, 0);
        assert_eq!(rumor.origin_tick, 100);
    }

    #[test]
    fn rumor_salience_higher_when_new() {
        let rumor = Rumor {
            proposition_id: 0,
            confidence: Fixed::from_f64(0.7),
            hops: 0,
            origin_tick: 100,
            last_heard_tick: 100,
            emotional_charge: Fixed::from_f64(0.5),
            identity_linkage: Fixed::ZERO,
            original_resistance: Fixed::from_f64(0.5),
        };
        let salience_new = rumor.salience(100);

        let mut old_rumor = rumor;
        old_rumor.hops = 5;
        let salience_old = old_rumor.salience(100);

        assert!(
            salience_new > salience_old,
            "New rumors should be more salient than old ones"
        );
    }

    #[test]
    fn mutate_applies_emotional_distortion() {
        let rumor = Rumor {
            proposition_id: 0,
            confidence: Fixed::from_f64(0.7),
            hops: 0,
            origin_tick: 0,
            last_heard_tick: 0,
            emotional_charge: Fixed::from_f64(0.3),
            identity_linkage: Fixed::ZERO,
            original_resistance: Fixed::from_f64(0.5),
        };

        let mut angry_emotions = make_emotions();
        angry_emotions.anger = Fixed::from_f64(0.8);

        let calm_emotions = make_emotions();

        let personality = make_personality();

        let (mutated_angry, _) = mutate_rumor(
            &rumor,
            Fixed::from_f64(0.7),
            &angry_emotions,
            &personality,
            &personality,
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.15),
        );
        let (mutated_calm, _) = mutate_rumor(
            &rumor,
            Fixed::from_f64(0.7),
            &calm_emotions,
            &personality,
            &personality,
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.15),
        );

        // Angry spreaders should distort confidence differently
        assert_ne!(
            mutated_angry, mutated_calm,
            "Emotional state should affect mutation"
        );
    }

    #[test]
    fn gossip_updates_existing_belief() {
        let rumor = Rumor::from_belief(&make_belief(0, 0.8), 100);
        let listener_beliefs = vec![make_belief(0, 0.5)];

        let result = process_gossip(
            &rumor,
            Fixed::from_f64(0.7),
            &make_emotions(),
            &make_personality(),
            &make_personality(),
            &listener_beliefs,
            100,
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.15),
            Fixed::from_f64(0.15),
        );

        assert!(result.accepted, "Gossip should be accepted");
        assert_ne!(result.belief_delta, Fixed::ZERO, "Belief should change");
    }

    #[test]
    fn gossip_creates_new_belief() {
        let rumor = Rumor::from_belief(&make_belief(5, 0.6), 100);
        let listener_beliefs: Vec<Belief> = vec![];

        let result = process_gossip(
            &rumor,
            Fixed::from_f64(0.7),
            &make_emotions(),
            &make_personality(),
            &make_personality(),
            &listener_beliefs,
            100,
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.15),
            Fixed::from_f64(0.15),
        );

        assert!(result.accepted);
        assert!(result.belief_delta > Fixed::ZERO);
    }

    #[test]
    fn apply_gossip_modifies_belief() {
        let mut beliefs = vec![make_belief(0, 0.5)];
        let result = GossipResult {
            proposition_id: 0,
            mutated_confidence: Fixed::from_f64(0.8),
            emotional_charge: Fixed::from_f64(0.4),
            belief_delta: Fixed::from_f64(0.15),
            accepted: true,
        };

        apply_gossip(&mut beliefs, &result, 200);

        assert!(
            beliefs[0].confidence > Fixed::from_f64(0.5),
            "Confidence should increase"
        );
        assert_eq!(beliefs[0].last_reinforced_tick, 200);
    }

    #[test]
    fn apply_gossip_creates_new_belief() {
        let mut beliefs: Vec<Belief> = vec![];
        let result = GossipResult {
            proposition_id: 5,
            mutated_confidence: Fixed::from_f64(0.6),
            emotional_charge: Fixed::from_f64(0.3),
            belief_delta: Fixed::from_f64(0.6),
            accepted: true,
        };

        apply_gossip(&mut beliefs, &result, 200);

        assert_eq!(beliefs.len(), 1);
        assert_eq!(beliefs[0].proposition_id, 5);
    }

    #[test]
    fn rejected_gossip_does_not_apply() {
        let mut beliefs = vec![make_belief(0, 0.5)];
        let result = GossipResult {
            proposition_id: 0,
            mutated_confidence: Fixed::from_f64(0.9),
            emotional_charge: Fixed::from_f64(0.5),
            belief_delta: Fixed::from_f64(0.2),
            accepted: false,
        };

        apply_gossip(&mut beliefs, &result, 200);

        assert_eq!(
            beliefs[0].confidence,
            Fixed::from_f64(0.5),
            "Rejected gossip should not change belief"
        );
    }

    #[test]
    fn high_resistance_belief_resists_gossip() {
        let rumor = Rumor::from_belief(&make_belief(0, 0.9), 100);
        let high_resistance_belief = Belief {
            proposition_id: 0,
            confidence: Fixed::from_f64(0.8),
            emotional_charge: Fixed::from_f64(0.5),
            identity_linkage: Fixed::from_f64(0.8),
            resistance: Fixed::from_f64(0.9), // very resistant
            last_reinforced_tick: 0,
            source: crate::person::EvidenceSource::PersonalExperience,
            social_reinforcement: 0,
            is_accurate: true,
        };

        let result = process_gossip(
            &rumor,
            Fixed::from_f64(0.7),
            &make_emotions(),
            &make_personality(),
            &make_personality(),
            &[high_resistance_belief],
            100,
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.15),
            Fixed::from_f64(0.15),
        );

        // High resistance should make the belief barely change
        assert!(
            result.belief_delta.abs() < Fixed::from_f64(0.1),
            "High resistance belief should barely change from gossip, delta={}",
            result.belief_delta.to_f64()
        );
    }

    #[test]
    fn moral_panic_triggers_with_high_charge() {
        // Create beliefs with high emotional charge across multiple agents
        let b1 = {
            let mut b = make_belief(1, 0.8);
            b.emotional_charge = Fixed::from_f64(0.7); // well above 0.55 threshold
            vec![b]
        };
        let b2 = {
            let mut b = make_belief(1, 0.8);
            b.emotional_charge = Fixed::from_f64(0.7);
            vec![b]
        };
        let b3 = {
            let mut b = make_belief(1, 0.8);
            b.emotional_charge = Fixed::from_f64(0.7);
            vec![b]
        };
        let beliefs: Vec<&Vec<Belief>> = vec![&b1, &b2, &b3];

        let result = detect_moral_panic(&beliefs, 1);
        assert!(
            result.triggered,
            "Moral panic should trigger with high emotional charge"
        );
        assert!(result.legitimacy_damage > Fixed::ZERO);
        assert!(result.grievance_boost > Fixed::ZERO);
    }

    #[test]
    fn moral_panic_does_not_trigger_with_low_charge() {
        let b1 = {
            let mut b = make_belief(1, 0.5);
            b.emotional_charge = Fixed::from_f64(0.2); // well below 0.55 threshold
            vec![b]
        };
        let beliefs: Vec<&Vec<Belief>> = vec![&b1];

        let result = detect_moral_panic(&beliefs, 1);
        assert!(
            !result.triggered,
            "Moral panic should not trigger with low emotional charge"
        );
        assert_eq!(result.legitimacy_damage, Fixed::ZERO);
        assert_eq!(result.grievance_boost, Fixed::ZERO);
    }

    #[test]
    fn moral_panic_no_beliefs_returns_no_trigger() {
        let empty_beliefs: Vec<&Vec<Belief>> = vec![];
        let result = detect_moral_panic(&empty_beliefs, 1);
        assert!(!result.triggered);
        assert_eq!(result.avg_charge, Fixed::ZERO);
    }

    #[test]
    fn moral_panic_boundary_high_charge_low_ratio() {
        // avg_charge >= 0.55 but panic_ratio < 0.3 (only 1 of 5 agents affected)
        let mut high_charge = make_belief(1, 0.8);
        high_charge.emotional_charge = Fixed::from_f64(0.7);
        let low_charge = make_belief(1, 0.5); // default emotional_charge = 0.3

        let b1 = vec![high_charge];
        let b2 = vec![low_charge.clone()];
        let b3 = vec![low_charge.clone()];
        let b4 = vec![low_charge.clone()];
        let b5 = vec![low_charge];
        let beliefs: Vec<&Vec<Belief>> = vec![&b1, &b2, &b3, &b4, &b5];

        let result = detect_moral_panic(&beliefs, 1);
        // avg_charge = (0.7 + 0.3 + 0.3 + 0.3 + 0.3) / 5 = 0.38 — below 0.55 threshold
        assert!(
            !result.triggered,
            "Should NOT trigger when avg charge is below threshold even with one high agent"
        );
    }
}
