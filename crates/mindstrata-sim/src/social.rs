//! Social interaction system — agents talk, trade, gossip, form relationships.
//!
//! This is the social layer.  Interactions between agents update
//! relationships, spread information, and generate social events.

use crate::person::Relationship;
use mindstrata_core::clock::Tick;
use mindstrata_core::event::{InteractionKind, SimEvent};
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use mindstrata_core::rng::{RngStreams, RngStream};
use rand::Rng;

/// A social interaction between two agents.
#[derive(Debug, Clone)]
pub struct Interaction {
    pub from: AgentId,
    pub to: AgentId,
    pub kind: InteractionKind,
}

/// Process a social interaction between two agents.
///
/// Updates their relationship and generates events.
pub fn process_interaction(
    interaction: &Interaction,
    relationships: &mut [Relationship],
    events: &mut Vec<SimEvent>,
    tick: Tick,
) {
    let trust_delta;
    let affection_delta;

    match interaction.kind {
        InteractionKind::Talk => {
            trust_delta = Fixed::from_f64(0.01);
            affection_delta = Fixed::from_f64(0.005);
        }
        InteractionKind::Help => {
            trust_delta = Fixed::from_f64(0.05);
            affection_delta = Fixed::from_f64(0.03);
        }
        InteractionKind::Threaten => {
            trust_delta = Fixed::from_f64(-0.1);
            affection_delta = Fixed::from_f64(-0.05);
        }
        InteractionKind::Trade => {
            trust_delta = Fixed::from_f64(0.02);
            affection_delta = Fixed::ZERO;
        }
        InteractionKind::Gossip => {
            trust_delta = Fixed::from_f64(0.005);
            affection_delta = Fixed::from_f64(0.01);
        }
        InteractionKind::Comfort => {
            trust_delta = Fixed::from_f64(0.03);
            affection_delta = Fixed::from_f64(0.05);
        }
        InteractionKind::Insult => {
            trust_delta = Fixed::from_f64(-0.08);
            affection_delta = Fixed::from_f64(-0.1);
        }
        InteractionKind::Teach => {
            trust_delta = Fixed::from_f64(0.04);
            affection_delta = Fixed::from_f64(0.02);
        }
    }

    // Update the relationship from → to
    if let Some(rel) = relationships
        .iter_mut()
        .find(|r| r.from == interaction.from && r.to == interaction.to)
    {
        rel.trust = (rel.trust + trust_delta).clamp_01();
        rel.affection = (rel.affection + affection_delta).clamp_01();
        rel.last_interaction_tick = tick.as_u64();
    }

    // Reciprocal relationship update (weaker)
    if let Some(rel) = relationships
        .iter_mut()
        .find(|r| r.from == interaction.to && r.to == interaction.from)
    {
        rel.trust = (rel.trust + trust_delta * Fixed::from_f64(0.3)).clamp_01();
        rel.affection = (rel.affection + affection_delta * Fixed::from_f64(0.3)).clamp_01();
        rel.last_interaction_tick = tick.as_u64();
    }

    // Generate event
    events.push(SimEvent::InteractionOccurred {
        from: interaction.from,
        to: interaction.to,
        kind: interaction.kind,
        tick,
    });

    // Generate relationship change event
    events.push(SimEvent::RelationshipChanged {
        from: interaction.from,
        to: interaction.to,
        trust_delta,
        affection_delta,
        tick,
    });
}

/// Select a random nearby agent for interaction.
pub fn select_interaction_target(
    agent_idx: usize,
    num_agents: usize,
    rng: &mut RngStreams,
) -> Option<usize> {
    if num_agents <= 1 {
        return None;
    }
    let social_rng = rng.get_mut(RngStream::Social);
    let target = loop {
        let t: usize = social_rng.random_range(0..num_agents);
        if t != agent_idx {
            break t;
        }
    };
    Some(target)
}

/// Choose an interaction type based on personality and relationship.
pub fn choose_interaction(
    trust: Fixed,
    affection: Fixed,
    personality_openness: Fixed,
    personality_agreeableness: Fixed,
    rng: &mut RngStreams,
) -> InteractionKind {
    let social_rng = rng.get_mut(RngStream::Social);
    let roll: f64 = social_rng.random_range(0.0..1.0);

    if trust < Fixed::from_f64(0.2) {
        // Low trust: threaten or avoid
        if roll < 0.3 {
            InteractionKind::Threaten
        } else {
            InteractionKind::Talk // cautious talk
        }
    } else if affection > Fixed::from_f64(0.7) {
        // High affection: comfort, help, talk
        if roll < 0.2 {
            InteractionKind::Comfort
        } else if roll < 0.5 {
            InteractionKind::Help
        } else {
            InteractionKind::Talk
        }
    } else if personality_openness > Fixed::from_f64(0.6) {
        // Open personality: gossip, teach
        if roll < 0.3 {
            InteractionKind::Gossip
        } else if roll < 0.5 && personality_agreeableness > Fixed::from_f64(0.5) {
            InteractionKind::Teach
        } else {
            InteractionKind::Talk
        }
    } else {
        // Default: talk or trade
        if roll < 0.3 {
            InteractionKind::Trade
        } else {
            InteractionKind::Talk
        }
    }
}

/// Run the social interaction system for all agents.
pub fn system_social_interactions(
    agents: &[(AgentId, Fixed, Fixed, Fixed)], // (id, openness, agreeableness, extraversion)
    relationships: &mut [Relationship],
    events: &mut Vec<SimEvent>,
    tick: Tick,
    rng: &mut RngStreams,
) {
    let num_agents = agents.len();

    for (i, (agent_id, openness, agreeableness, extraversion)) in agents.iter().enumerate() {
        // Extraversion affects interaction frequency
        let interact_chance = Fixed::from_f64(0.3) + *extraversion * Fixed::from_f64(0.4);
        let roll = Fixed::from_f64(rng.get_mut(RngStream::Social).random_range(0.0..1.0));

        if roll > interact_chance {
            continue;
        }

        // Select target
        if let Some(target_idx) = select_interaction_target(i, num_agents, rng) {
            let target_id = agents[target_idx].0;

            // Find existing relationship or create default
            let trust = relationships
                .iter()
                .find(|r| r.from == *agent_id && r.to == target_id)
                .map(|r| r.trust)
                .unwrap_or(Fixed::from_f64(0.5));

            let affection = relationships
                .iter()
                .find(|r| r.from == *agent_id && r.to == target_id)
                .map(|r| r.affection)
                .unwrap_or(Fixed::from_f64(0.3));

            let kind = choose_interaction(trust, affection, *openness, *agreeableness, rng);

            let interaction = Interaction {
                from: *agent_id,
                to: target_id,
                kind,
            };

            process_interaction(&interaction, relationships, events, tick);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_increases_trust() {
        let mut relationships = vec![Relationship {
            from: AgentId::new(0),
            to: AgentId::new(1),
            trust: Fixed::from_f64(0.5),
            affection: Fixed::from_f64(0.3),
            respect: Fixed::ZERO,
            fear: Fixed::ZERO,
            obligation: Fixed::ZERO,
            last_interaction_tick: 0,
        }];
        let mut events = Vec::new();

        let interaction = Interaction {
            from: AgentId::new(0),
            to: AgentId::new(1),
            kind: InteractionKind::Help,
        };

        process_interaction(&interaction, &mut relationships, &mut events, Tick::new(1));

        assert!(relationships[0].trust > Fixed::from_f64(0.5));
        assert!(!events.is_empty());
    }

    #[test]
    fn threat_decreases_trust() {
        let mut relationships = vec![Relationship {
            from: AgentId::new(0),
            to: AgentId::new(1),
            trust: Fixed::from_f64(0.5),
            affection: Fixed::from_f64(0.3),
            respect: Fixed::ZERO,
            fear: Fixed::ZERO,
            obligation: Fixed::ZERO,
            last_interaction_tick: 0,
        }];
        let mut events = Vec::new();

        let interaction = Interaction {
            from: AgentId::new(0),
            to: AgentId::new(1),
            kind: InteractionKind::Threaten,
        };

        process_interaction(&interaction, &mut relationships, &mut events, Tick::new(1));

        assert!(relationships[0].trust < Fixed::from_f64(0.5));
    }
}
