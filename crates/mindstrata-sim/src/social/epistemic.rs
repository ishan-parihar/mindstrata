//! Epistemic/informational substrate (§8.2) — agents have partial, distorted, local knowledge.
//!
//! This module implements:
//! - Epistemic styles (how agents process information)
//! - Belief updating from events with psychological biases
//! - Rumor propagation through social networks
//! - Information asymmetry between agents
//! - Source credibility tracking
//! - Attention filtering (agents notice different things)
//!
//! The key insight from the architecture:
//! > Agents should not know the global world state.
//! > They know: their body, needs, emotions, memories, relationships,
//! > local perception, institutional memberships, rumors, messages,
//! > and beliefs — which may be false.

use crate::person::{Belief, DiscreteEmotions, Personality};
use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Epistemic style — how an agent processes and evaluates information.
///
/// Different styles produce different patterns of belief formation,
/// rumor acceptance, and information seeking behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EpistemicStyle {
    /// Trusts direct experience and empirical evidence.
    Empirical,
    /// Defers to tradition and ancestral knowledge.
    Traditional,
    /// Accepts information from authority figures uncritically.
    Authoritative,
    /// Seeks hidden explanations, distrusts official narratives.
    Conspiratorial,
    /// Evaluates information by practical utility.
    Pragmatic,
    /// Seeks meaning through spiritual or mystical experience.
    Mystical,
    /// Doubts all claims, demands rigorous proof.
    Skeptical,
    /// Accepts information easily without scrutiny.
    #[default]
    Gullible,
}

/// Source of information — affects how much an agent trusts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InformationSource {
    /// Direct personal observation (highest trust).
    DirectObservation,
    /// Trusted friend or family member.
    TrustedPeer,
    /// Known community member.
    CommunityMember,
    /// Institutional announcement (council, temple).
    Institutional,
    /// Unknown source (rumor, hearsay).
    Unknown,
    /// Written record or inscription.
    WrittenRecord,
}

impl InformationSource {
    /// Base trust level for this source type.
    pub fn base_trust(&self) -> Fixed {
        match self {
            Self::DirectObservation => Fixed::from_f64(0.9),
            Self::TrustedPeer => Fixed::from_f64(0.7),
            Self::CommunityMember => Fixed::from_f64(0.5),
            Self::Institutional => Fixed::from_f64(0.6),
            Self::Unknown => Fixed::from_f64(0.2),
            Self::WrittenRecord => Fixed::from_f64(0.65),
        }
    }
}

/// Information chunk — a piece of knowledge an agent has acquired.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationChunk {
    /// What this information is about (proposition id).
    pub proposition_id: u64,
    /// How strongly the agent believes this (0-1).
    pub confidence: Fixed,
    /// Where this information came from.
    pub source: InformationSource,
    /// Tick when this information was acquired.
    pub acquired_tick: u64,
    /// How many times this information has been reinforced.
    pub reinforcement_count: u32,
    /// Emotional charge associated with this information.
    pub emotional_charge: Fixed,
    /// Whether this information is actually accurate in the world.
    pub is_accurate: bool,
}

/// Trust network — tracks how much an agent trusts different sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustNetwork {
    /// Trust in specific agents (agent_id -> trust level).
    pub agent_trust: Vec<(u64, Fixed)>,
    /// Trust in institutions (institution_id -> trust level).
    pub institution_trust: Vec<(u64, Fixed)>,
    /// General skepticism toward unknown sources.
    pub skepticism: Fixed,
}

impl Default for TrustNetwork {
    fn default() -> Self {
        Self {
            agent_trust: Vec::new(),
            institution_trust: Vec::new(),
            skepticism: Fixed::from_f64(0.3),
        }
    }
}

impl TrustNetwork {
    /// Get trust level for a specific agent.
    pub fn trust_for_agent(&self, agent_id: u64) -> Fixed {
        self.agent_trust
            .iter()
            .find(|(id, _)| *id == agent_id)
            .map_or(Fixed::from_f64(0.5), |(_, t)| *t) // default trust for unknown agents
    }

    /// Update trust for an agent based on interaction outcome.
    pub fn update_agent_trust(&mut self, agent_id: u64, delta: Fixed) {
        if let Some((_, trust)) = self.agent_trust.iter_mut().find(|(id, _)| *id == agent_id) {
            *trust = (*trust + delta).clamp_01();
        } else {
            let new_trust = (Fixed::from_f64(0.5) + delta).clamp_01();
            self.agent_trust.push((agent_id, new_trust));
        }
    }

    /// Get trust level for an institution.
    pub fn trust_for_institution(&self, inst_id: u64) -> Fixed {
        self.institution_trust
            .iter()
            .find(|(id, _)| *id == inst_id)
            .map_or(Fixed::from_f64(0.5), |(_, t)| *t)
    }

    /// Update trust for an institution.
    pub fn update_institution_trust(&mut self, inst_id: u64, delta: Fixed) {
        if let Some((_, trust)) = self.institution_trust.iter_mut().find(|(id, _)| *id == inst_id) {
            *trust = (*trust + delta).clamp_01();
        } else {
            let new_trust = (Fixed::from_f64(0.5) + delta).clamp_01();
            self.institution_trust.push((inst_id, new_trust));
        }
    }
}

/// Full epistemic state for an agent (§8.2).
///
/// This represents the agent's informational mind — what they know,
/// how they process information, and how they evaluate sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicState {
    /// The agent's epistemic style.
    pub style: EpistemicStyle,
    /// Trust network — tracks trust in agents and institutions.
    pub trust_network: TrustNetwork,
    /// How curious the agent is about new information (0-1).
    pub curiosity: Fixed,
    /// How dogmatic the agent is — resists changing existing beliefs (0-1).
    pub dogmatism: Fixed,
    /// Susceptibility to conspiracy theories (0-1).
    pub conspiracy_susceptibility: Fixed,
    /// How well the agent monitors source credibility (0-1).
    pub source_monitoring: Fixed,
    /// Need for cognitive closure — discomfort with ambiguity (0-1).
    pub need_for_closure: Fixed,
    /// Information overload level — too much info reduces processing quality.
    pub information_overload: Fixed,
    /// Number of information chunks processed this tick.
    pub info_processed_this_tick: u32,
    /// Maximum information processing capacity per tick.
    pub processing_capacity: u32,
}

impl Default for EpistemicState {
    fn default() -> Self {
        Self {
            style: EpistemicStyle::default(),
            trust_network: TrustNetwork::default(),
            curiosity: Fixed::from_f64(0.5),
            dogmatism: Fixed::from_f64(0.3),
            conspiracy_susceptibility: Fixed::from_f64(0.2),
            source_monitoring: Fixed::from_f64(0.4),
            need_for_closure: Fixed::from_f64(0.5),
            information_overload: Fixed::ZERO,
            info_processed_this_tick: 0,
            processing_capacity: 10,
        }
    }
}

impl EpistemicState {
    /// Initialize epistemic state from personality traits.
    pub fn from_personality(personality: &Personality) -> Self {
        let style = if personality.openness > Fixed::from_f64(0.7) {
            EpistemicStyle::Empirical
        } else if personality.traditionalism > Fixed::from_f64(0.7) {
            EpistemicStyle::Traditional
        } else if personality.conscientiousness > Fixed::from_f64(0.7) {
            EpistemicStyle::Pragmatic
        } else if personality.neuroticism > Fixed::from_f64(0.7) {
            EpistemicStyle::Conspiratorial
        } else if personality.agreeableness > Fixed::from_f64(0.7) {
            EpistemicStyle::Gullible
        } else {
            EpistemicStyle::Skeptical
        };

        Self {
            style,
            curiosity: personality.openness,
            dogmatism: Fixed::ONE - personality.openness,
            conspiracy_susceptibility: personality.neuroticism * Fixed::from_f64(0.5),
            source_monitoring: personality.conscientiousness,
            need_for_closure: Fixed::ONE - personality.openness,
            ..Self::default()
        }
    }

    /// Process incoming information and update beliefs.
    ///
    /// Returns the belief delta (how much the agent's belief changed).
    pub fn process_information(
        &mut self,
        belief: &mut Belief,
        info: &InformationChunk,
        personality: &Personality,
        emotions: &DiscreteEmotions,
        tick: u64,
    ) -> Fixed {
        // Check processing capacity
        if self.info_processed_this_tick >= self.processing_capacity {
            return Fixed::ZERO; // information overload
        }

        // Source trust depends on epistemic style
        let source_trust = self.compute_source_trust(info, personality);

        // Emotional fit: does this information match the agent's emotional state?
        let emotional_fit = self.compute_emotional_fit(info, emotions);

        // Identity protection: beliefs with high identity linkage resist change
        let identity_protection = if belief.identity_linkage > Fixed::from_f64(0.5) {
            self.dogmatism * Fixed::from_f64(0.5)
        } else {
            Fixed::ZERO
        };

        // Source monitoring: how carefully does the agent evaluate this source?
        let monitoring_bonus = if info.source == InformationSource::DirectObservation {
            Fixed::from_f64(0.2) // direct observation bypasses source monitoring
        } else {
            self.source_monitoring * Fixed::from_f64(0.1)
        };

        // Compute the evidence weight
        let evidence_weight = source_trust + emotional_fit + monitoring_bonus - identity_protection;

        // Apply belief update
        let old_confidence = belief.confidence;
        let resistance = belief.resistance + self.dogmatism * Fixed::from_f64(0.2);

        // §8.2: Belief updating formula
        // new_confidence = old * resistance + evidence * weight
        let new_confidence = (belief.confidence * resistance
            + info.confidence * evidence_weight * (Fixed::ONE - resistance))
            .clamp_01();

        let delta = new_confidence - old_confidence;
        belief.confidence = new_confidence;
        belief.emotional_charge = info.emotional_charge;
        belief.last_reinforced_tick = tick;

        self.info_processed_this_tick += 1;

        delta
    }

    /// Compute how much an agent trusts a particular information source.
    fn compute_source_trust(&self, info: &InformationChunk, personality: &Personality) -> Fixed {
        let base_trust = info.source.base_trust();

        // Style modifiers
        let style_modifier = match self.style {
            EpistemicStyle::Empirical => {
                if info.source == InformationSource::DirectObservation {
                    Fixed::from_f64(0.3) // empirical agents love direct observation
                } else {
                    -Fixed::from_f64(0.1) // distrust non-empirical sources
                }
            }
            EpistemicStyle::Traditional => {
                if info.source == InformationSource::WrittenRecord {
                    Fixed::from_f64(0.2) // traditional agents trust old records
                } else if info.source == InformationSource::Institutional {
                    Fixed::from_f64(0.1) // trust established institutions
                } else {
                    Fixed::ZERO
                }
            }
            EpistemicStyle::Authoritative => {
                if info.source == InformationSource::Institutional {
                    Fixed::from_f64(0.3) // authoritative agents love institutional sources
                } else {
                    -Fixed::from_f64(0.1) // distrust non-authority sources
                }
            }
            EpistemicStyle::Conspiratorial => {
                if info.source == InformationSource::Institutional {
                    -Fixed::from_f64(0.3) // conspiratorial agents distrust institutions
                } else if info.source == InformationSource::Unknown {
                    Fixed::from_f64(0.2) // but trust "hidden" sources
                } else {
                    Fixed::ZERO
                }
            }
            EpistemicStyle::Skeptical => {
                // skeptical agents are hard to convince from any source
                -Fixed::from_f64(0.15)
            }
            EpistemicStyle::Gullible => {
                // gullible agents trust everything
                Fixed::from_f64(0.15)
            }
            _ => Fixed::ZERO,
        };

        // Personality modifiers
        let personality_modifier = if personality.openness > Fixed::from_f64(0.6) {
            Fixed::from_f64(0.05) // open people are more receptive
        } else if personality.neuroticism > Fixed::from_f64(0.6) {
            -Fixed::from_f64(0.05) // neurotic people are more distrustful
        } else {
            Fixed::ZERO
        };

        (base_trust + style_modifier + personality_modifier).clamp_01()
    }

    /// Compute how well information fits the agent's current emotional state.
    fn compute_emotional_fit(&self, info: &InformationChunk, emotions: &DiscreteEmotions) -> Fixed {
        // Information that matches the agent's emotional state is more believable
        let emotional_charge = info.emotional_charge;

        // Fearful agents accept fearful information more readily
        let fear_fit = if emotions.fear > Fixed::from_f64(0.5) && emotional_charge > Fixed::ZERO {
            Fixed::from_f64(0.1)
        } else {
            Fixed::ZERO
        };

        // Angry agents accept angry information more readily
        let anger_fit = if emotions.anger > Fixed::from_f64(0.5) && emotional_charge > Fixed::ZERO {
            Fixed::from_f64(0.1)
        } else {
            Fixed::ZERO
        };

        // Conspiracy-susceptible agents accept emotionally charged information
        let conspiracy_fit = self.conspiracy_susceptibility * emotional_charge * Fixed::from_f64(0.1);

        (fear_fit + anger_fit + conspiracy_fit).clamp_01()
    }

    /// Reset per-tick processing counters.
    pub fn reset_tick(&mut self) {
        self.info_processed_this_tick = 0;
        // Information overload decays each tick
        self.information_overload = (self.information_overload - Fixed::from_f64(0.05)).max(Fixed::ZERO);
    }

    /// Add information overload from processing too much.
    pub fn add_overload(&mut self, amount: Fixed) {
        self.information_overload = (self.information_overload + amount).clamp_01();
        // Overload reduces processing capacity using Fixed arithmetic for determinism
        let capacity_fixed = Fixed::from_f64(10.0) * (Fixed::ONE - self.information_overload * Fixed::from_f64(0.5));
        self.processing_capacity = capacity_fixed.to_raw() as u32 / 1000; // Fixed has 1000 scaling
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
            fear: Fixed::ZERO,
            anger: Fixed::ZERO,
            joy: Fixed::ZERO,
            sadness: Fixed::ZERO,
            trust: Fixed::from_f64(0.5),
            shame: Fixed::ZERO,
            pride: Fixed::ZERO,
            guilt: Fixed::ZERO,
        }
    }

    fn make_belief(prop_id: u64, confidence: f64) -> Belief {
        Belief {
            proposition_id: prop_id,
            confidence: Fixed::from_f64(confidence),
            emotional_charge: Fixed::ZERO,
            identity_linkage: Fixed::ZERO,
            resistance: Fixed::from_f64(0.5),
            last_reinforced_tick: 0,
            source: crate::person::EvidenceSource::PersonalExperience,
            social_reinforcement: 0,
            is_accurate: true,
        }
    }

    fn make_info(source: InformationSource, confidence: f64) -> InformationChunk {
        InformationChunk {
            proposition_id: 0,
            confidence: Fixed::from_f64(confidence),
            source,
            acquired_tick: 100,
            reinforcement_count: 1,
            emotional_charge: Fixed::ZERO,
            is_accurate: true,
        }
    }

    #[test]
    fn epistemic_state_from_personality_open_is_empirical() {
        let mut p = make_personality();
        p.openness = Fixed::from_f64(0.8);
        let state = EpistemicState::from_personality(&p);
        assert_eq!(state.style, EpistemicStyle::Empirical);
    }

    #[test]
    fn epistemic_state_from_personality_neurotic_is_conspiratorial() {
        let mut p = make_personality();
        p.neuroticism = Fixed::from_f64(0.8);
        let state = EpistemicState::from_personality(&p);
        assert_eq!(state.style, EpistemicStyle::Conspiratorial);
    }

    #[test]
    fn direct_observation_trusted_more_than_unknown() {
        let state = EpistemicState::default();
        let personality = make_personality();
        let emotions = make_emotions();

        let mut belief = make_belief(0, 0.5);
        let info_direct = make_info(InformationSource::DirectObservation, 0.7);
        let delta_direct = state.process_information(
            &mut belief, &info_direct, &personality, &emotions, 100,
        );

        let mut belief2 = make_belief(0, 0.5);
        let info_unknown = make_info(InformationSource::Unknown, 0.7);
        let delta_unknown = EpistemicState::default().process_information(
            &mut belief2, &info_unknown, &personality, &emotions, 100,
        );

        assert!(delta_direct > delta_unknown,
            "Direct observation should produce larger belief change than unknown source");
    }

    #[test]
    fn trust_network_agent_trust() {
        let mut network = TrustNetwork::default();
        assert_eq!(network.trust_for_agent(1), Fixed::from_f64(0.5)); // default

        network.update_agent_trust(1, Fixed::from_f64(0.2));
        assert!(network.trust_for_agent(1) > Fixed::from_f64(0.5));

        network.update_agent_trust(1, Fixed::from_f64(-0.4));
        assert!(network.trust_for_agent(1) < Fixed::from_f64(0.5));
    }

    #[test]
    fn trust_network_institution_trust() {
        let mut network = TrustNetwork::default();
        assert_eq!(network.trust_for_institution(1), Fixed::from_f64(0.5));

        network.update_institution_trust(1, Fixed::from_f64(0.3));
        assert!(network.trust_for_institution(1) > Fixed::from_f64(0.5));
    }

    #[test]
    fn processing_capacity_limits_information() {
        let mut state = EpistemicState::default();
        state.processing_capacity = 2;

        let personality = make_personality();
        let emotions = make_emotions();
        let info = make_info(InformationSource::DirectObservation, 0.7);

        // Process using the SAME state (not clone) so info_processed_this_tick accumulates
        let mut belief1 = make_belief(0, 0.3);
        let delta1 = state.process_information(
            &mut belief1, &info, &personality, &emotions, 100,
        );
        assert_ne!(delta1, Fixed::ZERO, "First info should be processed");

        let mut belief2 = make_belief(1, 0.3);
        let delta2 = state.process_information(
            &mut belief2, &info, &personality, &emotions, 100,
        );
        assert_ne!(delta2, Fixed::ZERO, "Second info should be processed");

        let mut belief3 = make_belief(2, 0.3);
        let delta3 = state.process_information(
            &mut belief3, &info, &personality, &emotions, 100,
        );
        assert_eq!(delta3, Fixed::ZERO, "Third info should be rejected (over capacity)");
    }

    #[test]
    fn reset_tick_clears_counters() {
        let mut state = EpistemicState::default();
        state.info_processed_this_tick = 5;
        state.information_overload = Fixed::from_f64(0.3);
        state.reset_tick();
        assert_eq!(state.info_processed_this_tick, 0);
        assert!(state.information_overload < Fixed::from_f64(0.3));
    }

    #[test]
    fn information_source_base_trust_values() {
        assert!(InformationSource::DirectObservation.base_trust() > InformationSource::Unknown.base_trust());
        assert!(InformationSource::TrustedPeer.base_trust() > InformationSource::Unknown.base_trust());
        assert!(InformationSource::Institutional.base_trust() > InformationSource::Unknown.base_trust());
    }

    #[test]
    fn dogmatic_agent_resists_belief_change() {
        let mut state = EpistemicState::default();
        state.dogmatism = Fixed::from_f64(0.9);

        let personality = make_personality();
        let emotions = make_emotions();
        let info = make_info(InformationSource::DirectObservation, 0.8);

        let mut belief = make_belief(0, 0.3);
        let delta = state.process_information(
            &mut belief, &info, &personality, &emotions, 100,
        );

        // Dogmatic agent should have smaller belief change
        let mut state2 = EpistemicState::default();
        state2.dogmatism = Fixed::from_f64(0.1);
        let mut belief2 = make_belief(0, 0.3);
        let delta2 = state2.process_information(
            &mut belief2, &info, &personality, &emotions, 100,
        );

        assert!(delta.abs() < delta2.abs(),
            "Dogmatic agent should have smaller belief change: {} vs {}",
            delta.to_f64(), delta2.to_f64());
    }
}
