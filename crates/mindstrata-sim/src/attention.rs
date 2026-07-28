//! Attention and salience system.
//!
//! Agents cannot appraise everything. Attention is limited by:
//! - attention budget (cognitive capacity)
//! - habituation (repeated stimuli become less salient)
//! - emotional relevance (emotionally charged events grab attention)
//! - survival relevance (threats to needs grab attention)
//! - novelty (new things are more salient)
//!
//! §22.5 of the architecture spec.

use crate::person::{Affect, NeedState};
use mindstrata_core::event::{InteractionKind, SimEvent};
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// An agent's attention state.
///
/// Controls which events the agent notices and appraises.
/// §22.5: "Agents cannot appraise everything."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionState {
    /// Current focus — what the agent is paying attention to.
    pub focus: AttentionFocus,
    /// Attention budget: how much cognitive capacity is available (0..1).
    /// Depleted by stress, fatigue, emotional overwhelm.
    pub budget: Fixed,
    /// Habituation levels per event category (repeated stimuli become less salient).
    pub habituation: Habituation,
    /// Overall salience bias: shifts how easily things grab attention.
    /// High = easily distracted, low = focused.
    pub salience_bias: Fixed,
    /// Residual attention from the last tick (carries over).
    pub residual: Fixed,
}

/// What the agent is currently focused on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttentionFocus {
    /// Focused on internal state (needs, emotions).
    Internal,
    /// Focused on a specific agent.
    Agent(AgentId),
    /// Focused on a site/location.
    Site(usize),
    /// Scanning the environment broadly.
    Scanning,
}

impl Default for AttentionState {
    fn default() -> Self {
        Self {
            focus: AttentionFocus::Scanning,
            budget: Fixed::from_f64(0.8),
            habituation: Habituation::default(),
            salience_bias: Fixed::from_f64(0.5),
            residual: Fixed::ZERO,
        }
    }
}

/// Habituation levels for different event categories.
/// Repeated exposure reduces salience (§22.5).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Habituation {
    /// Habituation to social interactions (talk, trade).
    pub social: Fixed,
    /// Habituation to threats and violence.
    pub threat: Fixed,
    /// Habituation to resource-related events (eating, drinking).
    pub resource: Fixed,
    /// Habituation to emotional events.
    pub emotional: Fixed,
}

impl AttentionState {
    /// Compute the salience of an event for this agent.
    ///
    /// salience = intensity * novelty * relevance * bias - habituation
    ///
    /// Returns a value in [0, 1] indicating how much attention this event deserves.
    pub fn compute_salience(
        &mut self,
        event: &SimEvent,
        agent_id: AgentId,
        needs: &NeedState,
        affect: &Affect,
    ) -> Fixed {
        let base_intensity = Self::event_base_intensity(event, agent_id);
        let novelty = Self::event_novelty(event, &self.habituation);
        let relevance = Self::event_relevance(event, agent_id, needs, affect);
        let habituation_penalty = Self::event_habituation(event, &mut self.habituation);

        let raw_salience = base_intensity * novelty * relevance * self.salience_bias;
        let salience = (raw_salience - habituation_penalty).clamp_01();

        // Update habituation (exposure increases habituation)
        self.update_habituation(event, salience);

        salience
    }

    /// Base intensity of an event type (how inherently attention-grabbing it is).
    fn event_base_intensity(event: &SimEvent, _agent_id: AgentId) -> Fixed {
        match event {
            // Threats and insults are highly intense
            SimEvent::InteractionOccurred {
                kind: InteractionKind::Threaten | InteractionKind::Insult,
                ..
            } => Fixed::from_f64(0.9),
            // Help and comfort are moderately intense
            SimEvent::InteractionOccurred {
                kind: InteractionKind::Help | InteractionKind::Comfort,
                ..
            } => Fixed::from_f64(0.6),
            // Other interactions are mild
            SimEvent::InteractionOccurred { .. } => Fixed::from_f64(0.4),
            // Resource events are moderate
            SimEvent::AgentAte { .. } | SimEvent::AgentDrank { .. } => {
                Fixed::from_f64(0.3)
            }
            // Relationship changes are moderate
            SimEvent::RelationshipChanged { trust_delta, .. } => {
                // Large trust changes are more intense
                let magnitude = trust_delta.abs();
                Fixed::from_f64(0.3) + magnitude * Fixed::from_f64(0.5)
            }
            // Rumors are moderately intense
            SimEvent::RumorSpread { .. } => Fixed::from_f64(0.5),
            // Everything else is mild
            _ => Fixed::from_f64(0.2),
        }
    }

    /// Novelty factor: new/unexpected events are more salient.
    fn event_novelty(event: &SimEvent, habituation: &Habituation) -> Fixed {
        let habituation_level = match event {
            SimEvent::InteractionOccurred {
                kind: InteractionKind::Threaten | InteractionKind::Insult,
                ..
            } => habituation.threat,
            SimEvent::InteractionOccurred { .. } => habituation.social,
            SimEvent::AgentAte { .. } | SimEvent::AgentDrank { .. } => habituation.resource,
            SimEvent::RelationshipChanged { .. } => habituation.emotional,
            _ => Fixed::from_f64(0.3),
        };
        // Novelty = 1 - habituation (habituated events are less novel)
        Fixed::ONE - habituation_level
    }

    /// Relevance: how important is this event to the agent's current state?
    fn event_relevance(
        event: &SimEvent,
        agent_id: AgentId,
        needs: &NeedState,
        affect: &Affect,
    ) -> Fixed {
        let mut relevance = Fixed::from_f64(0.3); // base relevance

        match event {
            // Events involving the agent directly are highly relevant
            SimEvent::InteractionOccurred { from, to, .. }
            | SimEvent::RelationshipChanged { from, to, .. } => {
                if *from == agent_id || *to == agent_id {
                    relevance = Fixed::from_f64(0.9);
                }
            }
            SimEvent::AgentAte { agent, .. } | SimEvent::AgentDrank { agent, .. } => {
                if *agent == agent_id {
                    relevance = Fixed::from_f64(0.8);
                }
            }
            _ => {}
        }

        // Hungry agents find food-related events more relevant
        if needs.hunger > Fixed::from_f64(0.7)
            && matches!(event, SimEvent::AgentAte { .. })
        {
            relevance = (relevance + Fixed::from_f64(0.3)).clamp_01();
        }

        // Thirsty agents find water-related events more relevant
        if needs.thirst > Fixed::from_f64(0.7)
            && matches!(event, SimEvent::AgentDrank { .. })
        {
            relevance = (relevance + Fixed::from_f64(0.3)).clamp_01();
        }

        // Negative affect makes threat events more relevant (anxiety bias)
        if affect.valence < Fixed::ZERO
            && matches!(
                event,
                SimEvent::InteractionOccurred {
                    kind: InteractionKind::Threaten | InteractionKind::Insult,
                    ..
                }
            )
        {
            relevance = (relevance + Fixed::from_f64(0.2)).clamp_01();
        }

        relevance
    }

    /// Habituation penalty for a specific event type.
    fn event_habituation(event: &SimEvent, habituation: &mut Habituation) -> Fixed {
        match event {
            SimEvent::InteractionOccurred {
                kind: InteractionKind::Threaten | InteractionKind::Insult,
                ..
            } => habituation.threat,
            SimEvent::InteractionOccurred { .. } => habituation.social,
            SimEvent::AgentAte { .. } | SimEvent::AgentDrank { .. } => habituation.resource,
            SimEvent::RelationshipChanged { .. } => habituation.emotional,
            _ => Fixed::ZERO,
        }
    }

    /// Update habituation after exposure to an event.
    fn update_habituation(&mut self, event: &SimEvent, salience: Fixed) {
        let increment = salience * Fixed::from_f64(0.02); // slow habituation
        match event {
            SimEvent::InteractionOccurred {
                kind: InteractionKind::Threaten | InteractionKind::Insult,
                ..
            } => {
                self.habituation.threat =
                    (self.habituation.threat + increment).clamp_01();
            }
            SimEvent::InteractionOccurred { .. } => {
                self.habituation.social =
                    (self.habituation.social + increment).clamp_01();
            }
            SimEvent::AgentAte { .. } | SimEvent::AgentDrank { .. } => {
                self.habituation.resource =
                    (self.habituation.resource + increment).clamp_01();
            }
            SimEvent::RelationshipChanged { .. } => {
                self.habituation.emotional =
                    (self.habituation.emotional + increment).clamp_01();
            }
            _ => {}
        }
    }

    /// Decay habituation over time (agents "forget" to ignore things).
    pub fn decay_habituation(&mut self, rate: Fixed) {
        self.habituation.social = (self.habituation.social - rate).max(Fixed::ZERO);
        self.habituation.threat = (self.habituation.threat - rate).max(Fixed::ZERO);
        self.habituation.resource = (self.habituation.resource - rate).max(Fixed::ZERO);
        self.habituation.emotional = (self.habituation.emotional - rate).max(Fixed::ZERO);
    }

    /// Replenish attention budget (rest, low stress).
    pub fn replenish_budget(&mut self, stress: Fixed, fatigue: Fixed) {
        let replenishment = Fixed::from_f64(0.05) * (Fixed::ONE - stress) * (Fixed::ONE - fatigue);
        self.budget = (self.budget + replenishment).clamp_01();
    }

    /// Deplete attention budget (high stress, emotional overwhelm).
    pub fn deplete_budget(&mut self, amount: Fixed) {
        self.budget = (self.budget - amount).max(Fixed::ZERO);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mindstrata_core::clock::Tick;

    #[test]
    fn threat_events_are_highly_salient() {
        let mut attention = AttentionState::default();
        let needs = NeedState::default();
        let affect = Affect::default();
        let agent_id = AgentId::new(0);

        let threat_event = SimEvent::InteractionOccurred {
            from: AgentId::new(1),
            to: AgentId::new(0),
            kind: InteractionKind::Threaten,
            tick: Tick::new(1),
        };

        let salience = attention.compute_salience(&threat_event, agent_id, &needs, &affect);
        // base_intensity(0.9) * novelty(1.0) * relevance(0.9) * bias(0.5) = 0.405
        // This is the highest salience category — threat events dominate.
        assert!(salience > Fixed::from_f64(0.3),
            "Threat events should be highly salient, got {}", salience.to_f64());
    }

    #[test]
    fn direct_events_are_more_relevant() {
        let mut attention = AttentionState::default();
        let needs = NeedState::default();
        let affect = Affect::default();
        let agent_id = AgentId::new(0);

        // Event involving the agent
        let direct_event = SimEvent::InteractionOccurred {
            from: AgentId::new(1),
            to: AgentId::new(0),
            kind: InteractionKind::Talk,
            tick: Tick::new(1),
        };

        // Event not involving the agent
        let indirect_event = SimEvent::InteractionOccurred {
            from: AgentId::new(2),
            to: AgentId::new(3),
            kind: InteractionKind::Talk,
            tick: Tick::new(1),
        };

        let salience_direct = attention.compute_salience(&direct_event, agent_id, &needs, &affect);
        let salience_indirect = attention.compute_salience(&indirect_event, agent_id, &needs, &affect);

        assert!(salience_direct > salience_indirect,
            "Direct events should be more salient: direct={}, indirect={}",
            salience_direct.to_f64(), salience_indirect.to_f64());
    }

    #[test]
    fn habituation_reduces_salience() {
        let mut attention = AttentionState::default();
        let needs = NeedState::default();
        let affect = Affect::default();
        let agent_id = AgentId::new(0);

        let event = SimEvent::InteractionOccurred {
            from: AgentId::new(1),
            to: AgentId::new(0),
            kind: InteractionKind::Talk,
            tick: Tick::new(1),
        };

        let salience_first = attention.compute_salience(&event, agent_id, &needs, &affect);
        let salience_second = attention.compute_salience(&event, agent_id, &needs, &affect);

        assert!(salience_second <= salience_first,
            "Habituation should reduce salience: first={}, second={}",
            salience_first.to_f64(), salience_second.to_f64());
    }

    #[test]
    fn hunger_increases_food_relevance() {
        let mut attention = AttentionState::default();
        let agent_id = AgentId::new(0);
        let affect = Affect::default();

        let hungry_needs = NeedState {
            hunger: Fixed::from_f64(0.9),
            ..Default::default()
        };
        let satiated_needs = NeedState {
            hunger: Fixed::from_f64(0.1),
            ..Default::default()
        };

        let event = SimEvent::AgentAte {
            agent: AgentId::new(2),
            food: mindstrata_core::id::EntityId::new(0),
            tick: Tick::new(1),
        };

        let salience_hungry = attention.compute_salience(&event, agent_id, &hungry_needs, &affect);
        let salience_satiated = attention.compute_salience(&event, agent_id, &satiated_needs, &affect);

        assert!(salience_hungry > salience_satiated,
            "Hungry agents should find food events more salient: hungry={}, satiated={}",
            salience_hungry.to_f64(), salience_satiated.to_f64());
    }

    #[test]
    fn habituation_decays() {
        let mut attention = AttentionState::default();
        attention.habituation.social = Fixed::from_f64(0.5);

        attention.decay_habituation(Fixed::from_f64(0.1));
        assert_eq!(attention.habituation.social, Fixed::from_f64(0.4));
    }

    #[test]
    fn budget_depletes_and_replenishes() {
        let mut attention = AttentionState::default();
        let initial = attention.budget;

        attention.deplete_budget(Fixed::from_f64(0.3));
        assert!(attention.budget < initial);

        attention.replenish_budget(Fixed::ZERO, Fixed::ZERO); // no stress, no fatigue
        assert!(attention.budget > initial - Fixed::from_f64(0.3));
    }
}
