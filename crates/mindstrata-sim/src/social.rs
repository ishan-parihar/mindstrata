//! Social interaction system — agents talk, trade, gossip, form relationships.
//!
//! This is the social layer.  Interactions between agents update
//! relationships, spread information, and generate social events.

use crate::person::{Relationship, RelationshipKind};
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
///
/// §5.1: Trust and affection deltas are scaled by `bonding_rate` (positive)
/// and `conflict_escalation_rate` (negative) from `SimParameters`.
pub fn process_interaction(
    interaction: &Interaction,
    relationships: &mut [Relationship],
    events: &mut Vec<SimEvent>,
    tick: Tick,
    same_faction: bool, // §5.4: in-group bias modifier
    bonding_rate: Fixed,          // §5.1: from SimParameters
    conflict_escalation_rate: Fixed, // §5.1: from SimParameters
) {
    let trust_delta;
    let affection_delta;

    match interaction.kind {
        InteractionKind::Talk => {
            trust_delta = Fixed::from_f64(0.01) * bonding_rate * Fixed::from_f64(0.2);
            affection_delta = Fixed::from_f64(0.005) * bonding_rate * Fixed::from_f64(0.1);
        }
        InteractionKind::Help => {
            trust_delta = Fixed::from_f64(0.05) * bonding_rate;
            affection_delta = Fixed::from_f64(0.03) * bonding_rate;
        }
        InteractionKind::Threaten => {
            trust_delta = Fixed::from_f64(-0.1) * conflict_escalation_rate * Fixed::from_f64(1.25);
            affection_delta = Fixed::from_f64(-0.05) * conflict_escalation_rate * Fixed::from_f64(0.625);
        }
        InteractionKind::Trade => {
            trust_delta = Fixed::from_f64(0.02) * bonding_rate * Fixed::from_f64(0.4);
            affection_delta = Fixed::ZERO;
        }
        InteractionKind::Gossip => {
            trust_delta = Fixed::from_f64(0.005) * bonding_rate * Fixed::from_f64(0.1);
            affection_delta = Fixed::from_f64(0.01) * bonding_rate * Fixed::from_f64(0.2);
        }
        InteractionKind::Comfort => {
            trust_delta = Fixed::from_f64(0.03) * bonding_rate * Fixed::from_f64(0.6);
            affection_delta = Fixed::from_f64(0.05) * bonding_rate;
        }
        InteractionKind::Insult => {
            trust_delta = Fixed::from_f64(-0.08) * conflict_escalation_rate;
            affection_delta = Fixed::from_f64(-0.1) * conflict_escalation_rate * Fixed::from_f64(1.25);
        }
        InteractionKind::Teach => {
            trust_delta = Fixed::from_f64(0.04) * bonding_rate * Fixed::from_f64(0.8);
            affection_delta = Fixed::from_f64(0.02) * bonding_rate * Fixed::from_f64(0.4);
        }
    }

    // §5.4: In-group/out-group bias — faction membership modifies trust delta
    // In-group: positive interactions get a trust bonus
    // Out-group: negative interactions are penalized (positive interactions are neutral)
    let trust_delta = if same_faction {
        if trust_delta > Fixed::ZERO { trust_delta + INGROUP_TRUST_BONUS } else { trust_delta }
    } else if trust_delta < Fixed::ZERO {
        trust_delta - OUTGROUP_TRUST_PENALTY // out-group: negative interactions are worse
    } else {
        trust_delta // out-group: positive interactions are neutral (no bonus)
    };

    let tick_u64 = tick.as_u64();
    let is_positive = trust_delta > Fixed::ZERO;
    let is_negative = trust_delta < Fixed::ZERO;

    // Update the relationship from → to
    if let Some(rel) = relationships
        .iter_mut()
        .find(|r| r.from == interaction.from && r.to == interaction.to)
    {
        rel.trust = (rel.trust + trust_delta).clamp_01();
        rel.affection = (rel.affection + affection_delta).clamp_01();
        rel.last_interaction_tick = tick_u64;
        rel.interaction_count += 1;
        if is_positive {
            rel.last_positive_tick = tick_u64;
        }
        if is_negative {
            rel.last_negative_tick = tick_u64;
        }
        // §19.5.G: Relationship evolution — evolve kind based on interaction history
        evolve_relationship_kind(rel);
    }

    // Reciprocal relationship update (weaker)
    if let Some(rel) = relationships
        .iter_mut()
        .find(|r| r.from == interaction.to && r.to == interaction.from)
    {
        rel.trust = (rel.trust + trust_delta * Fixed::from_f64(0.3)).clamp_01();
        rel.affection = (rel.affection + affection_delta * Fixed::from_f64(0.3)).clamp_01();
        rel.last_interaction_tick = tick_u64;
        rel.interaction_count += 1;
        if is_positive {
            rel.last_positive_tick = tick_u64;
        }
        if is_negative {
            rel.last_negative_tick = tick_u64;
        }
        evolve_relationship_kind(rel);
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

/// §2.4: Default perception radius — agents can only interact within this Manhattan distance.
pub const DEFAULT_PERCEPTION_RADIUS: i32 = 5;

/// Select a random nearby agent for interaction, respecting perception radius.
///
/// §2.4: "Agents only perceive events within a radius (e.g., 5 tiles).
/// Social interactions only possible between agents within perception radius.
/// This creates natural neighborhoods and social clusters."
pub fn select_interaction_target(
    agent_idx: usize,
    num_agents: usize,
    agent_positions: &[(i32, i32)], // agent (x, y) positions
    perception_radius: i32,
    rng: &mut RngStreams,
) -> Option<usize> {
    if num_agents <= 1 {
        return None;
    }

    // Filter to agents within perception radius
    let ax = agent_positions[agent_idx].0;
    let ay = agent_positions[agent_idx].1;
    let nearby: Vec<usize> = (0..num_agents)
        .filter(|&t| {
            if t == agent_idx {
                return false;
            }
            let dx = (agent_positions[t].0 - ax).abs();
            let dy = (agent_positions[t].1 - ay).abs();
            (dx + dy) <= perception_radius
        })
        .collect();

    if nearby.is_empty() {
        return None;
    }

    let social_rng = rng.get_mut(RngStream::Social);
    let target = nearby[social_rng.random_range(0..nearby.len())];
    Some(target)
}

/// §5.4: In-group bias factor — faction members get bonus on positive interactions.
pub const INGROUP_TRUST_BONUS: Fixed = Fixed::from_raw(1000); // 0.1

/// §5.4: Out-group penalty factor — faction members distrust outsiders.
pub const OUTGROUP_TRUST_PENALTY: Fixed = Fixed::from_raw(500); // 0.05

/// Choose an interaction type based on personality and relationship.
///
/// §5.4: Faction in-group bias — members prefer positive interactions
/// with fellow members and negative ones with outsiders.
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

/// Update witness relationships when an interaction is observed.
/// Witnesses update their trust in both parties based on what they saw.
///
/// §5.1: Witness trust deltas scaled by `bonding_rate` (positive) and
/// `conflict_escalation_rate` (negative) from `SimParameters`.
pub fn update_witnesses(
    interaction: &Interaction,
    relationships: &mut [Relationship],
    num_agents: usize,
    tick: Tick,
    bonding_rate: Fixed,              // §5.1
    conflict_escalation_rate: Fixed,  // §5.1
) {
    for w in 0..num_agents {
        let witness = AgentId::new(w as u64);
        if witness == interaction.from || witness == interaction.to {
            continue;
        }

        match interaction.kind {
            // Negative interactions: witnesses reduce trust in perpetrator
            InteractionKind::Threaten | InteractionKind::Insult => {
                if let Some(rel) = relationships.iter_mut().find(|r| r.from == witness && r.to == interaction.from) {
                    let delta = Fixed::from_f64(-0.03) * conflict_escalation_rate * Fixed::from_f64(0.375);
                    rel.trust = (rel.trust + delta).max(Fixed::ZERO);
                    rel.last_interaction_tick = tick.as_u64();
                }
            }
            // Positive interactions: witnesses increase trust in helper
            InteractionKind::Help | InteractionKind::Comfort => {
                if let Some(rel) = relationships.iter_mut().find(|r| r.from == witness && r.to == interaction.from) {
                    let delta = Fixed::from_f64(0.02) * bonding_rate * Fixed::from_f64(0.4);
                    rel.trust = (rel.trust + delta).clamp_01();
                    rel.last_interaction_tick = tick.as_u64();
                }
            }
            // Neutral interactions: no witness effect
            _ => {}
        }
    }
}

/// §19.5.G: Evolve relationship kind based on accumulated interactions.
/// Stranger → Neighbor (5+ interactions) → Friend (high trust+affection) or Rival (low trust).
fn evolve_relationship_kind(rel: &mut Relationship) {
    // Don't evolve if already a special kind
    if matches!(rel.kind, RelationshipKind::Kin) {
        return;
    }

    let count = rel.interaction_count;
    let trust = rel.trust;
    let affection = rel.affection;

    // Upgrade from Stranger to Neighbor after 5 interactions
    if rel.kind == RelationshipKind::Stranger && count >= 5 {
        rel.kind = RelationshipKind::Neighbor;
    }

    // Upgrade from Neighbor to Friend or Rival based on trust/affection
    if rel.kind == RelationshipKind::Neighbor {
        if trust > Fixed::from_f64(0.7) && affection > Fixed::from_f64(0.5) {
            rel.kind = RelationshipKind::Friend;
        } else if trust < Fixed::from_f64(0.2) {
            rel.kind = RelationshipKind::Rival;
        }
    }

    // Friend can downgrade to Neighbor if trust drops
    if rel.kind == RelationshipKind::Friend && trust < Fixed::from_f64(0.4) {
        rel.kind = RelationshipKind::Neighbor;
    }

    // Rival can be repaired if trust recovers
    if rel.kind == RelationshipKind::Rival && trust > Fixed::from_f64(0.5) {
        rel.kind = RelationshipKind::Neighbor;
    }
}

/// Run the social interaction system for all agents.
///
/// §2.4: Social interactions are proximity-based — agents within perception
/// radius can interact, creating natural neighborhoods and social clusters.
/// §5.4: Faction in-group bias — members of the same faction get trust bonuses.
/// §5.1: Bonding and conflict rates from `SimParameters`.
pub fn system_social_interactions(
    agents: &[(AgentId, Fixed, Fixed, Fixed)], // (id, openness, agreeableness, extraversion)
    agent_positions: &[(i32, i32)],            // §2.4: agent (x, y) positions
    same_faction_matrix: &[Vec<bool>],         // §5.4: same_faction_matrix[i][j] = true if agents i,j share a faction
    relationships: &mut [Relationship],
    events: &mut Vec<SimEvent>,
    tick: Tick,
    rng: &mut RngStreams,
    bonding_rate: Fixed,              // §5.1: from SimParameters
    conflict_escalation_rate: Fixed,  // §5.1: from SimParameters
) {
    let num_agents = agents.len();

    for (i, (agent_id, openness, agreeableness, extraversion)) in agents.iter().enumerate() {
        // Extraversion affects interaction frequency
        let interact_chance = Fixed::from_f64(0.3) + *extraversion * Fixed::from_f64(0.4);
        let roll = Fixed::from_f64(rng.get_mut(RngStream::Social).random_range(0.0..1.0));

        if roll > interact_chance {
            continue;
        }

        // §2.4: Select target within perception radius
        if let Some(target_idx) = select_interaction_target(i, num_agents, agent_positions, DEFAULT_PERCEPTION_RADIUS, rng) {
            let target_id = agents[target_idx].0;

            // Find existing relationship or create default
            let trust = relationships
                .iter()
                .find(|r| r.from == *agent_id && r.to == target_id)
                .map_or(Fixed::from_f64(0.5), |r| r.trust);

            let affection = relationships
                .iter()
                .find(|r| r.from == *agent_id && r.to == target_id)
                .map_or(Fixed::from_f64(0.3), |r| r.affection);

            // §5.4: In-group/out-group — check if both agents share a faction
            let same_faction = same_faction_matrix[i][target_idx];
            let kind = choose_interaction(trust, affection, *openness, *agreeableness, rng);

            let interaction = Interaction {
                from: *agent_id,
                to: target_id,
                kind,
            };

            // Update witnesses before processing the interaction
            update_witnesses(&interaction, relationships, num_agents, tick, bonding_rate, conflict_escalation_rate);

            process_interaction(&interaction, relationships, events, tick, same_faction, bonding_rate, conflict_escalation_rate);
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
            kind: crate::person::RelationshipKind::Stranger,
            interaction_count: 0,
            last_positive_tick: 0,
            last_negative_tick: 0,
        }];
        let mut events = Vec::new();

        let interaction = Interaction {
            from: AgentId::new(0),
            to: AgentId::new(1),
            kind: InteractionKind::Help,
        };

        process_interaction(&interaction, &mut relationships, &mut events, Tick::new(1), false, Fixed::from_f64(0.05), Fixed::from_f64(0.08));

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
            kind: crate::person::RelationshipKind::Stranger,
            interaction_count: 0,
            last_positive_tick: 0,
            last_negative_tick: 0,
        }];
        let mut events = Vec::new();

        let interaction = Interaction {
            from: AgentId::new(0),
            to: AgentId::new(1),
            kind: InteractionKind::Threaten,
        };

        process_interaction(&interaction, &mut relationships, &mut events, Tick::new(1), false, Fixed::from_f64(0.05), Fixed::from_f64(0.08));

        assert!(relationships[0].trust < Fixed::from_f64(0.5));
    }
}
