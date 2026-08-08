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
    params: &crate::parameters::SimParameters,
) {
    let trust_delta;
    let affection_delta;

    match interaction.kind {
        InteractionKind::Talk => {
            trust_delta = Fixed::from_f64(0.01) * bonding_rate;
            affection_delta = Fixed::from_f64(0.005) * bonding_rate;
        }
        InteractionKind::Help => {
            trust_delta = Fixed::from_f64(0.05) * bonding_rate;
            affection_delta = Fixed::from_f64(0.03) * bonding_rate;
        }
        InteractionKind::Threaten => {
            trust_delta = Fixed::from_f64(-0.1) * conflict_escalation_rate;
            affection_delta = Fixed::from_f64(-0.05) * conflict_escalation_rate;
        }
        InteractionKind::Trade => {
            trust_delta = Fixed::from_f64(0.02) * bonding_rate;
            affection_delta = Fixed::ZERO;
        }
        InteractionKind::Gossip => {
            trust_delta = Fixed::from_f64(0.005) * bonding_rate;
            affection_delta = Fixed::from_f64(0.01) * bonding_rate;
        }
        InteractionKind::Comfort => {
            trust_delta = Fixed::from_f64(0.03) * bonding_rate;
            affection_delta = Fixed::from_f64(0.05) * bonding_rate;
        }
        InteractionKind::Insult => {
            trust_delta = Fixed::from_f64(-0.08) * conflict_escalation_rate;
            affection_delta = Fixed::from_f64(-0.1) * conflict_escalation_rate;
        }
        InteractionKind::Teach => {
            trust_delta = Fixed::from_f64(0.04) * bonding_rate;
            affection_delta = Fixed::from_f64(0.02) * bonding_rate;
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
        evolve_relationship_kind(rel, params);
    }

    // Reciprocal relationship update (weaker)
    if let Some(rel) = relationships
        .iter_mut()
        .find(|r| r.from == interaction.to && r.to == interaction.from)
    {
        rel.trust = (rel.trust + trust_delta * params.social_reciprocal_factor).clamp_01();
        rel.affection = (rel.affection + affection_delta * params.social_reciprocal_factor).clamp_01();
        rel.last_interaction_tick = tick_u64;
        rel.interaction_count += 1;
        if is_positive {
            rel.last_positive_tick = tick_u64;
        }
        if is_negative {
            rel.last_negative_tick = tick_u64;
        }
        evolve_relationship_kind(rel, params);
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
    anger: Fixed,
    no_violence_resistance: Fixed, // §8.1.10 (Iteration 85)
    help_neighbors_propensity: Fixed, // §8.1.10 (Iteration 89)
    respect_elders_propensity: Fixed, // §8.1.10 (Iteration 91)
    target_is_elder: bool,          // §8.1.10 (Iteration 91)
    rng: &mut RngStreams,
    params: &crate::parameters::SimParameters,
) -> InteractionKind {
    let social_rng = rng.get_mut(RngStream::Social);
    let roll: f64 = social_rng.random_range(0.0..1.0);

    // §8.1.10 (Iteration 85): the no-violence norm suppresses the threat
    // *decision* itself — an agent who has internalized the norm is less
    // likely to issue a threat in the first place. Both threat paths scale
    // their thresholds by (1 − resistance): the stress-driven negativity
    // branch converts the threatened act into an insult (verbal hostility
    // persists, the violent act is suppressed), and the low-trust branch
    // converts the threat into cautious talk. Continuous (no cliff),
    // zero-at-zero (no internalized norm before the first monthly ritual at
    // tick 4320 → legacy probabilities, golden baselines byte-identical),
    // and the RNG draw stays unconditional (same stream position at every
    // resistance value — replay determinism holds).
    let threat_scale = (Fixed::ONE - no_violence_resistance).to_f64();

    // §8.1.10 (Iteration 91): the prescriptive "Respect Elders" norm
    // suppresses *disrespectful* acts (threat/insult) toward the community's
    // designated elder (the Council "Elder" role holder). Target-conditional
    // and zero-at-zero: a non-elder target or a norm-less agent leaves both
    // negative windows untouched (elder_scale = 1.0 → legacy probabilities,
    // golden baselines byte-identical); full internalization toward the
    // elder eliminates disrespect entirely; partial leaves a strictly
    // reduced continuous rate (no cliff). The RNG draw stays unconditional
    // — only the thresholds change, so replay determinism holds.
    let elder_scale = if target_is_elder {
        (Fixed::ONE - respect_elders_propensity.clamp_01()).to_f64()
    } else {
        1.0
    };

    // Stress-driven negativity: angry agents (low agreeableness OR elevated
    // anger) occasionally lash out even at moderate trust. Without this, the
    // negative-interaction branch was structurally unreachable (low-trust
    // threshold 0.2 never re-attained after initialization), so trust
    // ratcheted to 1.0 for every pair and the witness-reputation system had
    // nothing to witness. Gating on anger keeps conflict correlated with
    // stress, preserving the stress → conflict coupling the system models.
    let negativity_prob = if personality_agreeableness < Fixed::from_f64(0.35) {
        0.10
    } else if anger > Fixed::from_f64(0.5) {
        0.12
    } else {
        0.0
    };
    if roll < negativity_prob * elder_scale {
        return if roll < negativity_prob * 0.4 * threat_scale * elder_scale {
            InteractionKind::Threaten
        } else {
            InteractionKind::Insult
        };
    }

    if trust < params.social_low_trust_threshold {
        // Low trust: threaten or avoid (an internalized no-violence norm
        // converts the threat into cautious talk; toward the designated
        // elder a Respect Elders norm does the same)
        if roll < 0.3 * threat_scale * elder_scale {
            InteractionKind::Threaten
        } else {
            InteractionKind::Talk // cautious talk
        }
    } else if affection > params.social_high_affection_threshold {
        // High affection: comfort, help, talk
        // §8.1.10 (Iteration 89): the prescriptive "Help Neighbors" norm
        // amplifies the Help *decision* — the Help window [0.2, help_bound)
        // grows with the internalized norm's strength: at full
        // internalization the window reaches [0.2, 1.0), so the agent
        // always helps or comforts in high-affection contexts (never merely
        // talks), mirroring the prohibitive norms' full-suppression at 1.0
        // without a cliff (continuous scaling). Zero-at-zero: before the
        // first monthly ritual (tick 4320) no agent holds the norm, so the
        // bound is 0.5 and the golden baseline stays byte-identical. The
        // RNG draw stays unconditional — only the threshold changes.
        // Clamped to [0.5, 1.0]: the prescriptive norm can amplify but never
        // suppress the Help window, even for a degenerate out-of-range
        // strength (mirrors the Iter-87 hypocrisy-factor clamp rationale).
        let help_bound = (0.5 * (1.0 + help_neighbors_propensity.to_f64())).clamp(0.5, 1.0);
        if roll < 0.2 {
            InteractionKind::Comfort
        } else if roll < help_bound {
            InteractionKind::Help
        } else {
            InteractionKind::Talk
        }
    } else if personality_openness > params.social_openness_threshold {
        // Open personality: gossip, teach
        if roll < 0.3 {
            InteractionKind::Gossip
        } else if roll < 0.5 && personality_agreeableness > params.social_agreeableness_threshold {
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
                    let delta = Fixed::from_f64(-0.03) * conflict_escalation_rate;
                    rel.trust = (rel.trust + delta).max(Fixed::ZERO);
                    rel.last_interaction_tick = tick.as_u64();
                }
            }
            // Positive interactions: witnesses increase trust in helper
            InteractionKind::Help | InteractionKind::Comfort => {
                if let Some(rel) = relationships.iter_mut().find(|r| r.from == witness && r.to == interaction.from) {
                    let delta = Fixed::from_f64(0.02) * bonding_rate;
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
fn evolve_relationship_kind(rel: &mut Relationship, params: &crate::parameters::SimParameters) {
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
        if trust > params.social_friend_trust_threshold && affection > params.social_friend_affection_threshold {
            rel.kind = RelationshipKind::Friend;
        } else if trust < params.social_rival_trust_threshold {
            rel.kind = RelationshipKind::Rival;
        }
    }

    // Friend can downgrade to Neighbor if trust drops
    if rel.kind == RelationshipKind::Friend && trust < params.social_friend_downgrade_threshold {
        rel.kind = RelationshipKind::Neighbor;
    }

    // Rival can be repaired if trust recovers
    if rel.kind == RelationshipKind::Rival && trust > params.social_rival_repair_trust {
        rel.kind = RelationshipKind::Neighbor;
    }
}

/// Run the social interaction system for all agents.
///
/// §2.4: Social interactions are proximity-based — agents within perception
/// radius can interact, creating natural neighborhoods and social clusters.
/// §5.4: Faction in-group bias — members of the same faction get trust bonuses.
/// §5.1: Bonding and conflict rates from `SimParameters`.
///
/// The per-agent info tuple grows with every norm wiring (Iterations 85, 89,
/// 91) — an intentional flat data-passing shape, not a public API.
#[expect(clippy::type_complexity)]
pub fn system_social_interactions(
    // (id, openness, agreeableness, extraversion, anger, no_violence_resistance,
    //  help_neighbors_propensity, respect_elders_propensity, is_elder)
    agents: &[(AgentId, Fixed, Fixed, Fixed, Fixed, Fixed, Fixed, Fixed, bool)],
    agent_positions: &[(i32, i32)],            // §2.4: agent (x, y) positions
    same_faction_matrix: &[Vec<bool>],         // §5.4: same_faction_matrix[i][j] = true if agents i,j share a faction
    relationships: &mut [Relationship],
    events: &mut Vec<SimEvent>,
    tick: Tick,
    rng: &mut RngStreams,
    bonding_rate: Fixed,              // §5.1: from SimParameters
    conflict_escalation_rate: Fixed,  // §5.1: from SimParameters
    params: &crate::parameters::SimParameters,
) {
    let num_agents = agents.len();

    // The source agent's own elder flag is unused here — the gate reads the
    // *target's* elder status (`agents[target_idx].8`) when the interaction
    // is chosen.
    for (i, (agent_id, openness, agreeableness, extraversion, anger, no_violence_resistance, help_neighbors_propensity, respect_elders_propensity, _is_elder)) in agents.iter().enumerate() {
        // Extraversion affects interaction frequency
        let interact_chance = params.social_interaction_base_chance + *extraversion * params.social_extraversion_multiplier;
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
                .map_or(params.social_default_trust, |r| r.trust);

            let affection = relationships
                .iter()
                .find(|r| r.from == *agent_id && r.to == target_id)
                .map_or(params.social_default_affection, |r| r.affection);

            // §5.4: In-group/out-group — check if both agents share a faction
            let same_faction = same_faction_matrix[i][target_idx];
            let kind = choose_interaction(
                trust,
                affection,
                *openness,
                *agreeableness,
                *anger,
                *no_violence_resistance,
                *help_neighbors_propensity,
                *respect_elders_propensity,
                agents[target_idx].8, // the target's is_elder flag
                rng,
                params,
            );

            let interaction = Interaction {
                from: *agent_id,
                to: target_id,
                kind,
            };

            // Update witnesses before processing the interaction
            update_witnesses(&interaction, relationships, num_agents, tick, bonding_rate, conflict_escalation_rate);

            process_interaction(&interaction, relationships, events, tick, same_faction, bonding_rate, conflict_escalation_rate, params);
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

        process_interaction(&interaction, &mut relationships, &mut events, Tick::new(1), false, Fixed::from_f64(0.05), Fixed::from_f64(0.08), &crate::parameters::SimParameters::default());

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

        process_interaction(&interaction, &mut relationships, &mut events, Tick::new(1), false, Fixed::from_f64(0.05), Fixed::from_f64(0.08), &crate::parameters::SimParameters::default());

        assert!(relationships[0].trust < Fixed::from_f64(0.5));
    }

    /// §8.1.10 (Iteration 85): an agent who has internalized the no-violence
    /// norm suppresses the threat *decision* itself — `choose_interaction`
    /// scales both threat thresholds by (1 − resistance). The RNG draw stays
    /// unconditional, so the same seed yields the same roll sequence at every
    /// resistance value: full internalization must eliminate threats
    /// entirely, partial internalization must leave a strictly reduced rate,
    /// and zero resistance (the golden window) must match legacy behavior.
    #[test]
    fn no_violence_norm_suppresses_threat_decision() {
        let params = crate::parameters::SimParameters::default();
        // Low-trust setup isolates the threat branch: trust 0 sits below the
        // low-trust threshold, high agreeableness keeps the stress-driven
        // negativity branch off, so every Threaten comes from the low-trust
        // `roll < 0.3` gate.
        let trust = Fixed::ZERO;
        let affection = Fixed::ZERO;
        let openness = Fixed::from_f64(0.5);
        let agreeableness = Fixed::from_f64(0.8);
        let anger = Fixed::ZERO;
        let count_threats = |resistance: Fixed| -> u32 {
            let mut rng = RngStreams::new(42);
            let mut threats = 0u32;
            for _ in 0..2000 {
                if choose_interaction(
                    trust,
                    affection,
                    openness,
                    agreeableness,
                    anger,
                    resistance,
                    Fixed::ZERO,  // no Help Neighbors norm in this setup
                    Fixed::ZERO,  // no Respect Elders norm in this setup
                    false,        // no designated-elder target
                    &mut rng,
                    &params,
                ) == InteractionKind::Threaten
                {
                    threats += 1;
                }
            }
            threats
        };
        let baseline = count_threats(Fixed::ZERO);
        let suppressed = count_threats(Fixed::ONE);
        let partial = count_threats(Fixed::from_f64(0.7));
        assert!(baseline > 0, "low-trust threats must occur without the norm");
        assert_eq!(
            suppressed, 0,
            "full no-violence internalization must eliminate threats"
        );
        assert!(
            partial < baseline,
            "partial internalization must leave a reduced threat rate"
        );
    }

    /// §8.1.10 (Iteration 89): the prescriptive "Help Neighbors" norm
    /// amplifies the Help *decision* — `choose_interaction` grows the
    /// high-affection Help window from [0.2, 0.5) toward [0.2, 1.0) as the
    /// internalized strength rises. The RNG draw stays unconditional, so
    /// the same seed yields the same roll sequence at every propensity:
    /// full internalization must strictly increase Help (any roll in the
    /// [0.5, 1.0) ring converts Talk → Help), partial must land strictly
    /// between, and the untouched [0, 0.2) Comfort window must stay
    /// identical (the golden window's unaffected-path zero-drift pin).
    #[test]
    fn help_neighbors_norm_amplifies_help_decision() {
        let params = crate::parameters::SimParameters::default();
        // High-affection setup isolates the Help branch: trust 1.0 and
        // affection 1.0 clear both the low-trust and high-affection
        // thresholds, and high agreeableness + zero anger keep the
        // stress-driven negativity branch off, so every roll lands in the
        // high-affection window.
        let trust = Fixed::ONE;
        let affection = Fixed::ONE;
        let openness = Fixed::from_f64(0.5);
        let agreeableness = Fixed::from_f64(0.8);
        let anger = Fixed::ZERO;
        let count_help = |propensity: Fixed| -> (u32, u32) {
            let mut rng = RngStreams::new(42);
            let mut help = 0u32;
            let mut comfort = 0u32;
            for _ in 0..2000 {
                match choose_interaction(
                    trust,
                    affection,
                    openness,
                    agreeableness,
                    anger,
                    Fixed::ZERO,
                    propensity,
                    Fixed::ZERO,
                    false,
                    &mut rng,
                    &params,
                ) {
                    InteractionKind::Help => help += 1,
                    InteractionKind::Comfort => comfort += 1,
                    _ => {}
                }
            }
            (help, comfort)
        };
        let (baseline_help, baseline_comfort) = count_help(Fixed::ZERO);
        let (full_help, full_comfort) = count_help(Fixed::ONE);
        let (partial_help, _) = count_help(Fixed::from_f64(0.5));
        assert!(
            baseline_help > 0,
            "high-affection rolls in [0.2, 0.5) must produce Help without the norm"
        );
        assert!(
            full_help > baseline_help,
            "full internalization must strictly amplify Help (rolls in [0.5, 1.0) convert Talk → Help)"
        );
        assert!(
            partial_help > baseline_help && partial_help < full_help,
            "partial internalization must land strictly between baseline and full"
        );
        assert_eq!(
            baseline_comfort, full_comfort,
            "the untouched [0, 0.2) Comfort window must be identical — same roll sequence"
        );
    }

    /// §8.1.10 (Iteration 91): the prescriptive "Respect Elders" norm
    /// suppresses *disrespectful* acts toward the community's designated
    /// elder — `choose_interaction` scales both negative windows (the
    /// stress-driven negativity branch and the low-trust threat gate) by
    /// elder_scale = (1 − propensity) when the target is the elder. The RNG
    /// draw stays unconditional, so the same seed yields the same roll
    /// sequence at every propensity: full internalization must eliminate
    /// threats toward the elder, partial must leave a strictly reduced rate,
    /// and the elder flag alone (zero propensity) must be inert — the
    /// zero-at-zero pin that keeps the golden baseline byte-identical.
    #[test]
    fn respect_elders_norm_suppresses_disrespect_toward_elders() {
        let params = crate::parameters::SimParameters::default();
        // Low-trust isolated setup: trust 0 sits below the low-trust
        // threshold, high agreeableness + zero anger keep the stress-driven
        // negativity branch off, so every Threaten comes from the low-trust
        // `roll < 0.3` gate.
        let trust = Fixed::ZERO;
        let affection = Fixed::ZERO;
        let openness = Fixed::from_f64(0.5);
        let agreeableness = Fixed::from_f64(0.8);
        let anger = Fixed::ZERO;
        let count_threats = |propensity: Fixed, target_is_elder: bool| -> u32 {
            let mut rng = RngStreams::new(42);
            let mut threats = 0u32;
            for _ in 0..2000 {
                if choose_interaction(
                    trust,
                    affection,
                    openness,
                    agreeableness,
                    anger,
                    Fixed::ZERO,
                    Fixed::ZERO,
                    propensity,
                    target_is_elder,
                    &mut rng,
                    &params,
                ) == InteractionKind::Threaten
                {
                    threats += 1;
                }
            }
            threats
        };
        let baseline = count_threats(Fixed::ZERO, false);
        let elder_no_norm = count_threats(Fixed::ZERO, true);
        let elder_full = count_threats(Fixed::ONE, true);
        let elder_partial = count_threats(Fixed::from_f64(0.7), true);
        assert!(baseline > 0, "low-trust threats must occur without the norm");
        assert_eq!(
            elder_no_norm, baseline,
            "the elder designation alone must be inert (zero-at-zero)"
        );
        assert_eq!(
            elder_full, 0,
            "full Respect Elders internalization must eliminate threats toward the elder"
        );
        assert!(
            elder_partial < baseline,
            "partial internalization must leave a strictly reduced threat rate"
        );
    }
}
