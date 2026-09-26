//! Social interaction system — agents talk, trade, gossip, form relationships.
//!
//! This is the social layer.  Interactions between agents update
//! relationships, spread information, and generate social events.

use mindstrata_core::clock::Tick;
use mindstrata_core::event::{InteractionKind, SimEvent};
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use mindstrata_core::rng::{RngStream, RngStreams};
use mindstrata_person::person::Relationship;
use rand::Rng;

/// A social interaction between two agents.
#[derive(Debug, Clone)]
pub struct Interaction {
    /// From. (doc added at S3 extraction)
    pub from: AgentId,
    /// To. (doc added at S3 extraction)
    pub to: AgentId,
    /// Kind. (doc added at S3 extraction)
    pub kind: InteractionKind,
}

/// Process a social interaction between two agents.
///
/// §5.1: the emitted deltas are scaled by `bonding_rate` (positive) and
/// `conflict_escalation_rate` (negative) from `SimParameters`.
///
/// i403: the legacy-matrix store writes are DELETED — the per-act relationship
/// write lives on the dyadic store (pass_social's speech-act block applies
/// `RelationshipV2::record_positive/record_negative` with credibility-based
/// magnitudes in BOTH directions, so the direct and the reciprocal channels
/// both have a dyadic home). The §5.1 kind table below now feeds the
/// `RelationshipChanged` PERCEPT only — attention intensity (|delta|), memory
/// pair-lanes, salience — and the `model_sign_matches_applied_deltas` contract
/// (speech_act.rs) still asserts these emitted deltas equal the grounded
/// model's, so the percept keeps meaning what it always meant.
pub fn process_interaction(
    interaction: &Interaction,
    events: &mut Vec<SimEvent>,
    tick: Tick,
    same_faction: bool,              // §5.4: in-group bias modifier
    bonding_rate: Fixed,             // §5.1: scales positive emitted deltas
    conflict_escalation_rate: Fixed, // §5.1: scales negative emitted deltas
) {
    let (trust_delta, affection_delta) = match interaction.kind {
        InteractionKind::Talk => (
            Fixed::from_f64(0.01) * bonding_rate,
            Fixed::from_f64(0.005) * bonding_rate,
        ),
        InteractionKind::Help => (
            Fixed::from_f64(0.05) * bonding_rate,
            Fixed::from_f64(0.03) * bonding_rate,
        ),
        InteractionKind::Threaten => (
            Fixed::from_f64(-0.1) * conflict_escalation_rate,
            Fixed::from_f64(-0.05) * conflict_escalation_rate,
        ),
        InteractionKind::Trade => (Fixed::from_f64(0.02) * bonding_rate, Fixed::ZERO),
        InteractionKind::Gossip => (
            Fixed::from_f64(0.005) * bonding_rate,
            Fixed::from_f64(0.01) * bonding_rate,
        ),
        InteractionKind::Comfort => (
            Fixed::from_f64(0.03) * bonding_rate,
            Fixed::from_f64(0.05) * bonding_rate,
        ),
        InteractionKind::Insult => (
            Fixed::from_f64(-0.08) * conflict_escalation_rate,
            Fixed::from_f64(-0.1) * conflict_escalation_rate,
        ),
        InteractionKind::Teach => (
            Fixed::from_f64(0.04) * bonding_rate,
            Fixed::from_f64(0.02) * bonding_rate,
        ),
    };

    // §5.4: In-group/out-group bias — faction membership modifies trust delta
    // In-group: positive interactions get a trust bonus
    // Out-group: negative interactions are penalized (positive interactions are neutral)
    let trust_delta = if same_faction {
        if trust_delta > Fixed::ZERO {
            trust_delta + INGROUP_TRUST_BONUS
        } else {
            trust_delta
        }
    } else if trust_delta < Fixed::ZERO {
        trust_delta - OUTGROUP_TRUST_PENALTY // out-group: negative interactions are worse
    } else {
        trust_delta // out-group: positive interactions are neutral (no bonus)
    };

    // i403: the two legacy-matrix write blocks (direct + reciprocal ×
    // social_reciprocal_factor) are deleted here — the per-act store write
    // lives on the dyadic path (the speech-act block writes BOTH the
    // listener's and the speaker's rows, so the weaker reverse-direction
    // update is carried with its own credibility weighting instead of a
    // flat ×0.3).

    // Generate event — the speech-act block and memory_ops consume this;
    // it is the interaction path's live producer signal.
    events.push(SimEvent::InteractionOccurred {
        from: interaction.from,
        to: interaction.to,
        kind: interaction.kind,
        tick,
    });

    // Generate relationship change event — post-i403 a PERCEPT record of the
    // act's modeled significance (attention intensity, memory pair-lanes,
    // salience), not a ledger update; the store deltas live on the dyadic
    // write path.
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
///
/// §17.2 (Iteration 158): `eligible` is the per-agent social-participation
/// mask — Background-tier agents (aggregate simulation, no individual
/// relationships) are never selected as interaction targets. The mask is
/// aligned with `agent_positions`, so index mapping is unchanged.
pub fn select_interaction_target(
    agent_idx: usize,
    num_agents: usize,
    agent_positions: &[(i32, i32)], // agent (x, y) positions
    perception_radius: i32,
    rng: &mut RngStreams,
    eligible: &[bool], // §17.2: false = Background tier, never targeted
) -> Option<usize> {
    if num_agents <= 1 {
        return None;
    }

    // Filter to agents within perception radius who participate in social
    // interactions (Background-tier agents are excluded as targets).
    let ax = agent_positions[agent_idx].0;
    let ay = agent_positions[agent_idx].1;
    let nearby: Vec<usize> = (0..num_agents)
        .filter(|&t| {
            if t == agent_idx || !eligible[t] {
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
    no_violence_resistance: Fixed,    // §8.1.10 (Iteration 85)
    help_neighbors_propensity: Fixed, // §8.1.10 (Iteration 89)
    respect_elders_propensity: Fixed, // §8.1.10 (Iteration 91)
    target_is_elder: bool,            // §8.1.10 (Iteration 91)
    obey_ruler_propensity: Fixed,     // §8.1.10 (Iteration 93)
    target_is_authority: bool,        // §8.1.10 (Iteration 93)
    rng: &mut RngStreams,
    params: &mindstrata_core::parameters::SimParameters,
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

    // §8.1.10 (Iteration 93): the prescriptive "Obey Ruler" norm
    // suppresses *defiance* (threat/insult) toward the community's
    // authority — the Council "Guard Captain" role holder (the ruler's
    // enforcement arm). The gate mirrors the Iteration-91 elder gate:
    // target-conditional (a non-authority target or a norm-less agent
    // leaves both negative windows untouched → obey_scale 1.0 → legacy
    // probabilities, golden baselines byte-identical), zero-at-zero, and
    // the RNG draw stays unconditional (only thresholds change → replay
    // determinism). The Guard Captain is a distinct anchor from the Elder
    // (separate role holders, probe-verified), so this gate never
    // compounds with the elder gate on the same target.
    let obey_scale = if target_is_authority {
        (Fixed::ONE - obey_ruler_propensity.clamp_01()).to_f64()
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
    // Iteration 185 (P5 emergent re-audit) recalibration: the previous
    // 0.10/0.12 rates, multiplied by the daily interaction volume (~57
    // interactions/agent/day), produced ~2372 threats in 2000 calm ticks
    // (19% of ALL interactions — ~9.5 threats/agent/day) and a
    // threat→violence→low-trust→more-threats death spiral that collapsed
    // 5/6 seeds to ≤4/12 alive by 20K (every death cause = Violence).
    // Calm-baseline hostility is now rare-but-present: low agreeableness
    // alone yields 2.5% (a genuinely aggressive disposition), elevated
    // anger 5% (stress-coupled, the drama channel). Conflict remains
    // reachable (probe: ~250 insults / ~200 threats per 2000 ticks), but
    // the village no longer massacres itself.
    let negativity_prob = if personality_agreeableness < Fixed::from_f64(0.35) {
        0.025
    } else if anger > Fixed::from_f64(0.5) {
        0.05
    } else {
        0.0
    };
    if roll < negativity_prob * elder_scale * obey_scale {
        return if roll < negativity_prob * 0.4 * threat_scale * elder_scale * obey_scale {
            InteractionKind::Threaten
        } else {
            InteractionKind::Insult
        };
    }

    if trust < params.social_low_trust_threshold {
        // Low trust: threaten or avoid (an internalized no-violence norm
        // converts the threat into cautious talk; toward the designated
        // elder a Respect Elders norm does the same, and toward the Guard
        // Captain an Obey Ruler norm suppresses the defiance)
        // Iteration 185 recalibration: 0.3 → 0.12. The 0.3 threaten rate on
        // the low-trust branch compounded the negativity branch's threat
        // stream (violence destroys trust, pushing pairs below the 0.2
        // threshold, which fired more threats → more violence). 0.12 keeps
        // low-trust pairs hostile-but-cautious (mostly avoidant talk) while
        // breaking the death spiral.
        if roll < 0.12 * threat_scale * elder_scale * obey_scale {
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
        let help_bound = f64::midpoint(1.0, help_neighbors_propensity.to_f64()).clamp(0.5, 1.0);
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
/// i330: `rel_lookup` is the dense `(from·n + to) → relationships position`
/// index built by the caller. The per-witness `iter_mut().find(..)` below was
/// `O(R)` = `O(N²)` **per witness per interaction**, i.e. `O(I·N·R) ≈ O(N⁴)`
/// per tick — the dominant term in the whole tick at N=192 (probe i330:
/// `social_pass` local exponent ≈3.0). With the lookup each witness update is
/// i403 scope decision: the witness channel STAYS on the legacy v1 matrix.
/// The plan row deletes the interaction direct+reciprocal writes (the
/// ratchet); moving the ±0.02/0.03 witness deltas onto the dyadic rows was
/// tried in this arc and REVERTED by measurement: the v1 bump was TRANSIENT
/// (erased daily by the i376 convergence sync), so making it persistent on
/// v2 is a MAGNITUDE change (the i398 class) — the +0.02/act inflow (most
/// of the village witnesses most acts, i349 locality notwithstanding)
/// pinned v2 trust at 1.0 and saturated the meme-transmission baseline
/// (36 hosts at multiplier 1.2 AND 3.0). The channel keeps its i349-
/// calibrated transient semantics on v1 until its own re-sized move.
pub fn update_witnesses(
    interaction: &Interaction,
    relationships: &mut [Relationship],
    rel_lookup: &[u32],
    num_agents: usize,
    tick: Tick,
    bonding_rate: Fixed,             // §5.1
    conflict_escalation_rate: Fixed, // §5.1
    agent_positions: &[(i32, i32)],
    perception_radius: i32,
) {
    // i349: witnesses must be able to PERCEIVE the act (Manhattan distance
    // from the act site) — the locality model that killed the village-wide
    // trust ratchet.
    let (fx, fy) = agent_positions[interaction.from.as_u64() as usize];
    for (w, &(wx, wy)) in agent_positions.iter().enumerate().take(num_agents) {
        let witness = AgentId::new(w as u64);
        if witness == interaction.from || witness == interaction.to {
            continue;
        }
        let dx = (wx - fx).abs();
        let dy = (wy - fy).abs();
        if dx + dy > perception_radius {
            continue;
        }

        // Neutral interactions have no witness effect — skip before any lookup.
        let negative = matches!(
            interaction.kind,
            InteractionKind::Threaten | InteractionKind::Insult
        );
        let positive = matches!(
            interaction.kind,
            InteractionKind::Help | InteractionKind::Comfort
        );
        if !negative && !positive {
            continue;
        }

        // The witness always looks at `(witness → actor)` — hostile actors
        // lose witness trust, helpful actors gain it.
        let pos = resolve_rel_pos(
            relationships,
            rel_lookup,
            num_agents,
            witness,
            interaction.from,
        );

        match interaction.kind {
            // Negative interactions: witnesses reduce trust in perpetrator
            InteractionKind::Threaten | InteractionKind::Insult => {
                if let Some(p) = pos {
                    let delta = Fixed::from_f64(-0.03) * conflict_escalation_rate;
                    let rel = &mut relationships[p];
                    rel.trust = (rel.trust + delta).max(Fixed::ZERO);
                    rel.last_interaction_tick = tick.as_u64();
                }
            }
            // Positive interactions: witnesses increase trust in helper
            InteractionKind::Help | InteractionKind::Comfort => {
                if let Some(p) = pos {
                    let delta = Fixed::from_f64(0.02) * bonding_rate;
                    let rel = &mut relationships[p];
                    rel.trust = (rel.trust + delta).clamp_01();
                    rel.last_interaction_tick = tick.as_u64();
                }
            }
            // Neutral interactions: no witness effect
            _ => {}
        }
    }
}

/// i330: O(1) position of the `from → to` edge in `relationships`, via the
/// dense caller-built lookup with a **revalidating fallback** to the linear
/// scan (correct when the lookup is stale — the pass runs on a slice, so
/// positions are stable, but a mid-tick population change can shift ids).
/// Records first-occurrence semantics, matching `iter().find(..)` exactly.
fn resolve_rel_pos(
    relationships: &[Relationship],
    rel_lookup: &[u32],
    num_agents: usize,
    from: AgentId,
    to: AgentId,
) -> Option<usize> {
    let fi = from.as_u64() as usize;
    let ti = to.as_u64() as usize;
    if fi < num_agents && ti < num_agents {
        if let Some(&p) = rel_lookup.get(fi * num_agents + ti) {
            if p != u32::MAX {
                let p = p as usize;
                if relationships
                    .get(p)
                    .is_some_and(|r| r.from == from && r.to == to)
                {
                    return Some(p);
                }
            }
        }
    }
    relationships
        .iter()
        .position(|r| r.from == from && r.to == to)
}

/// Run the social interaction system for all agents.
///
/// §2.4: Social interactions are proximity-based — agents within perception
/// radius can interact, creating natural neighborhoods and social clusters.
/// §5.4: Faction in-group bias — members of the same faction get trust bonuses.
/// §5.1: Bonding and conflict rates from `SimParameters`.
///
/// The per-agent info tuple grows with every norm wiring (Iterations 85, 89,
/// 91, 158, 162) — an intentional flat data-passing shape, not a public API.
#[expect(clippy::type_complexity)]
pub fn system_social_interactions(
    // (id, openness, agreeableness, extraversion, anger, no_violence_resistance,
    //  help_neighbors_propensity, respect_elders_propensity, is_elder,
    //  obey_ruler_propensity, is_authority, loneliness, sociability_dev, runs_social)
    agents: &[(
        AgentId,
        Fixed,
        Fixed,
        Fixed,
        Fixed,
        Fixed,
        Fixed,
        Fixed,
        bool,
        Fixed,
        bool,
        Fixed,
        Fixed,
        bool,
    )],
    agent_positions: &[(i32, i32)],    // §2.4: agent (x, y) positions
    same_faction_matrix: &[Vec<bool>], // §5.4: same_faction_matrix[i][j] = true if agents i,j share a faction
    // i428: the kind schedule reads the legacy matrix again (the i403 read
    // move's kind-mix shift was measured as the conviction-collapse cause at
    // i427); the witness channel always stayed on legacy (scope decision,
    // see update_witnesses). The v2 rows are not needed here at all.
    relationships: &mut [Relationship],
    rel_lookup: &[u32],
    events: &mut Vec<SimEvent>,
    tick: Tick,
    rng: &mut RngStreams,
    bonding_rate: Fixed,             // §5.1: from SimParameters
    conflict_escalation_rate: Fixed, // §5.1: from SimParameters
    params: &mindstrata_core::parameters::SimParameters,
) {
    let num_agents = agents.len();

    // §17.2 (Iteration 158): the social-participation mask — Background-tier
    // agents are excluded both as initiators and as targets. When no agent is
    // Background (every calibrated window — Iter-145 probe pins 0B at every
    // size/seed), the mask is all-true and behavior is byte-identical.
    let runs_social_mask: Vec<bool> = agents.iter().map(|a| a.13).collect();

    // The source agent's own elder/authority flags are unused here — the
    // gates read the *target's* status (`agents[target_idx].8` for elder,
    // `agents[target_idx].10` for authority) when the interaction is chosen.
    for (
        i,
        (
            agent_id,
            openness,
            agreeableness,
            extraversion,
            anger,
            no_violence_resistance,
            help_neighbors_propensity,
            respect_elders_propensity,
            _is_elder,
            obey_ruler_propensity,
            _is_authority,
            loneliness,
            sociability_dev,
            runs_social,
        ),
    ) in agents.iter().enumerate()
    {
        // §17.2 (Iteration 158): Background agents run aggregate simulation —
        // they neither initiate nor receive individual social interactions.
        // The gate sits BEFORE the RNG roll, so a Background source consumes
        // zero Social-stream draws (the `continue` also skips target
        // selection). Focal/Secondary agents are unaffected.
        if !runs_social {
            continue;
        }
        // §8.1.4 (Iteration 98): a lonely agent seeks social contact more —
        // loneliness adds to the interaction-chance gate (previously the
        // family was computed by appraisal every tick and never read).
        // Zero-at-zero (loneliness 0 → legacy gate, byte-identical), the RNG
        // draw stays unconditional (only the threshold moves — replay
        // determinism holds), and extraversion still dominates the
        // personality channel.
        //
        // §8.1.6 (Iteration 162): sociability deviation (live temperament −
        // trait-derived baseline, accumulated by the plasticity pass) adds a
        // third gate channel — a socially-tempered agent clears the gate
        // more often. Zero-at-zero (deviation 0 at construction → legacy
        // gate, byte-identical until plasticity drifts the layer), RNG draw
        // stays unconditional (threshold-only change → replay determinism).
        let interact_chance = params.social_interaction_base_chance
            + *extraversion * params.social_extraversion_multiplier
            + *loneliness * params.social_loneliness_multiplier
            + *sociability_dev * params.social_sociability_multiplier;
        let roll = Fixed::from_f64(rng.get_mut(RngStream::Social).random_range(0.0..1.0));

        if roll > interact_chance {
            continue;
        }

        // §2.4: Select target within perception radius — Background-tier
        // agents are excluded from the candidate set (mask from §17.2).
        if let Some(target_idx) = select_interaction_target(
            i,
            num_agents,
            agent_positions,
            DEFAULT_PERCEPTION_RADIUS,
            rng,
            &runs_social_mask,
        ) {
            let target_id = agents[target_idx].0;

            // i428: the kind schedule reads v1 again — i427's event-stream
            // measurements show the trust bars are already 89–100% open on v2
            // (the conviction collapse came from this read move's kind-mix
            // shift), and i402 measured the schedule's input saturated on
            // both stores — the real repair is re-deriving the 0.7/0.2
            // thresholds against v2's event distribution, its own iteration.
            // The i330 dense lookup is restored verbatim.
            let pair = rel_lookup
                .get(i * num_agents + target_idx)
                .copied()
                .filter(|&p| p != u32::MAX)
                .map(|p| p as usize)
                .filter(|&p| {
                    relationships
                        .get(p)
                        .is_some_and(|r| r.from == *agent_id && r.to == target_id)
                })
                .or_else(|| {
                    relationships
                        .iter()
                        .position(|r| r.from == *agent_id && r.to == target_id)
                });
            let (trust, affection) = pair.map_or(
                (params.social_default_trust, params.social_default_affection),
                |p| (relationships[p].trust, relationships[p].affection),
            );

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
                *obey_ruler_propensity,
                agents[target_idx].10, // the target's is_authority flag
                rng,
                params,
            );

            let interaction = Interaction {
                from: *agent_id,
                to: target_id,
                kind,
            };

            // Update witnesses before processing the interaction — the
            // witness channel stays on v1 (i403 scope decision).
            update_witnesses(
                &interaction,
                relationships,
                rel_lookup,
                num_agents,
                tick,
                bonding_rate,
                conflict_escalation_rate,
                agent_positions,
                DEFAULT_PERCEPTION_RADIUS,
            );

            process_interaction(
                &interaction,
                events,
                tick,
                same_faction,
                bonding_rate,
                conflict_escalation_rate,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// i403: the v1 store writes are deleted; `process_interaction` now
    /// emits the percept pair (InteractionOccurred + RelationshipChanged)
    /// with the §5.1 model deltas. The old lookup/write tests are retired
    /// with the code they pinned; these assert the surviving contract —
    /// the emitted delta's sign and scaling, which the
    /// `model_sign_matches_applied_deltas` guard in speech_act.rs pins to
    /// the grounded model.
    #[test]
    fn help_emits_positive_model_delta() {
        let mut events = Vec::new();
        process_interaction(
            &Interaction {
                from: AgentId::new(0),
                to: AgentId::new(1),
                kind: InteractionKind::Help,
            },
            &mut events,
            Tick::new(1),
            false,
            Fixed::ONE,
            Fixed::ONE,
        );
        assert!(!events.is_empty());
        let delta = events.iter().find_map(|ev| match ev {
            SimEvent::RelationshipChanged { trust_delta, .. } => Some(*trust_delta),
            _ => None,
        });
        assert!(delta.is_some(), "RelationshipChanged must be emitted");
        assert!(
            delta.unwrap() > Fixed::ZERO,
            "help's model delta is positive"
        );
    }

    #[test]
    fn threat_emits_negative_model_delta() {
        let mut events = Vec::new();
        process_interaction(
            &Interaction {
                from: AgentId::new(0),
                to: AgentId::new(1),
                kind: InteractionKind::Threaten,
            },
            &mut events,
            Tick::new(1),
            false,
            Fixed::ONE,
            Fixed::ONE,
        );
        let delta = events.iter().find_map(|ev| match ev {
            SimEvent::RelationshipChanged { trust_delta, .. } => Some(*trust_delta),
            _ => None,
        });
        assert!(
            delta.unwrap() < Fixed::ZERO,
            "threat's model delta is negative"
        );
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
        let params = mindstrata_core::parameters::SimParameters::default();
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
                    Fixed::ZERO, // no Help Neighbors norm in this setup
                    Fixed::ZERO, // no Respect Elders norm in this setup
                    false,       // no designated-elder target
                    Fixed::ZERO, // no Obey Ruler norm in this setup
                    false,       // no authority target in this setup
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
        assert!(
            baseline > 0,
            "low-trust threats must occur without the norm"
        );
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
        let params = mindstrata_core::parameters::SimParameters::default();
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
                    Fixed::ZERO, // no Obey Ruler norm in this setup
                    false,       // no authority target in this setup
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

    /// §8.1.6 (Iteration 180): the core altruism trait — the last
    /// decision-less core trait — widens the Help window THROUGH the shared
    /// fold: `help_propensity` (norm + tenderness × 0.5 + gratitude × 0.5 +
    /// altruism × 0.3) feeds `help_neighbors_propensity`, so a high-
    /// altruism agent helps strictly more in high-affection pairs than an
    /// altruism-zero agent under the SAME seed. The RNG draw stays
    /// unconditional — only the window threshold changes (the Iter-89
    /// pattern), so the [0, 0.2) Comfort window must stay identical.
    #[test]
    fn altruism_widens_help_decision_via_fold() {
        let params = mindstrata_core::parameters::SimParameters::default();
        // High-affection setup isolates the Help branch (mirrors the
        // Iter-89 test): trust 1.0 and affection 1.0 clear both thresholds,
        // high agreeableness + zero anger keep the negativity branch off.
        let trust = Fixed::ONE;
        let affection = Fixed::ONE;
        let openness = Fixed::from_f64(0.5);
        let agreeableness = Fixed::from_f64(0.8);
        let anger = Fixed::ZERO;
        let alt_rate = params.social_altruism_help_multiplier.to_f64();
        let count_help = |altruism: f64| -> (u32, u32) {
            let mut rng = RngStreams::new(42);
            let mut help = 0u32;
            let mut comfort = 0u32;
            for _ in 0..2000 {
                // Zero emotions + zero norm isolates the dispositional
                // channel — the propensity is purely the altruism term.
                let propensity = mindstrata_psych::appraisal::help_propensity(
                    Fixed::ZERO,
                    Fixed::ZERO,
                    Fixed::ZERO,
                    Fixed::from_f64(altruism),
                    0.5,
                    0.5,
                    alt_rate,
                );
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
                    Fixed::ZERO, // no Obey Ruler norm in this setup
                    false,       // no authority target in this setup
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
        let (baseline_help, baseline_comfort) = count_help(0.0);
        let (full_help, full_comfort) = count_help(1.0);
        let (half_help, _) = count_help(0.5);
        assert!(
            baseline_help > 0,
            "high-affection rolls in [0.2, 0.5) must produce Help even at zero altruism"
        );
        assert!(
            full_help > baseline_help,
            "full altruism must strictly amplify Help (rolls in the widened window convert Talk → Help)"
        );
        assert!(
            half_help > baseline_help && half_help < full_help,
            "half altruism must land strictly between baseline and full"
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
        let params = mindstrata_core::parameters::SimParameters::default();
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
                    Fixed::ZERO, // no Obey Ruler norm in this setup
                    false,       // no authority target in this setup
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
        assert!(
            baseline > 0,
            "low-trust threats must occur without the norm"
        );
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

    /// §8.1.10 (Iteration 93): the prescriptive "Obey Ruler" norm
    /// suppresses *defiance* (threat/insult) toward the community's
    /// authority — the Council "Guard Captain" role holder. Mirrors the
    /// Iteration-91 elder gate: `choose_interaction` scales both negative
    /// windows by obey_scale = (1 − propensity) when the target is the
    /// authority. Same seed-42 roll sequence across all propensities.
    #[test]
    fn obey_ruler_norm_suppresses_defiance_toward_guard_captain() {
        let params = mindstrata_core::parameters::SimParameters::default();
        // Low-trust isolated setup (mirrors the Respect Elders test): trust 0
        // sits below the low-trust threshold, high agreeableness + zero anger
        // keep the stress-driven negativity branch off, so every Threaten
        // comes from the low-trust `roll < 0.3` gate.
        let trust = Fixed::ZERO;
        let affection = Fixed::ZERO;
        let openness = Fixed::from_f64(0.5);
        let agreeableness = Fixed::from_f64(0.8);
        let anger = Fixed::ZERO;
        let count_threats = |propensity: Fixed, target_is_authority: bool| -> u32 {
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
                    Fixed::ZERO,
                    false, // no designated-elder target
                    propensity,
                    target_is_authority,
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
        let authority_no_norm = count_threats(Fixed::ZERO, true);
        let authority_full = count_threats(Fixed::ONE, true);
        let authority_partial = count_threats(Fixed::from_f64(0.7), true);
        assert!(
            baseline > 0,
            "low-trust threats must occur without the norm"
        );
        assert_eq!(
            authority_no_norm, baseline,
            "the authority designation alone must be inert (zero-at-zero)"
        );
        assert_eq!(
            authority_full, 0,
            "full Obey Ruler internalization must eliminate threats toward the authority"
        );
        assert!(
            authority_partial < baseline,
            "partial internalization must leave a strictly reduced threat rate"
        );
    }

    /// §8.1.4 (Iteration 99): a tenderness-boosted help propensity converts
    /// more high-affection rolls into Help — the same window the Iter-89
    /// Help Neighbors norm amplifies, so the norm and the emotion compose
    /// additively (clamped by the [0.5, 1.0] window bound). Zero-at-zero:
    /// propensity 0 → legacy Help window.
    #[test]
    fn tenderness_boosted_propensity_amplifies_help() {
        let params = mindstrata_core::parameters::SimParameters::default();
        let trust = Fixed::ONE;
        let affection = Fixed::ONE;
        let openness = Fixed::from_f64(0.5);
        let agreeableness = Fixed::from_f64(0.8);
        let anger = Fixed::ZERO;
        let count_help = |propensity: Fixed| -> u32 {
            let mut rng = RngStreams::new(42);
            let mut help = 0u32;
            for _ in 0..2000 {
                if choose_interaction(
                    trust,
                    affection,
                    openness,
                    agreeableness,
                    anger,
                    Fixed::ZERO,
                    propensity,
                    Fixed::ZERO,
                    false,
                    Fixed::ZERO,
                    false,
                    &mut rng,
                    &params,
                ) == InteractionKind::Help
                {
                    help += 1;
                }
            }
            help
        };
        // Tenderness 0.9 × multiplier 0.5 = 0.45 propensity (no norm).
        let baseline = count_help(Fixed::ZERO);
        let tender = count_help(Fixed::from_f64(0.45));
        let full = count_help(Fixed::ONE);
        assert!(baseline > 0, "high-affection rolls must produce some Help");
        assert!(
            tender > baseline,
            "tenderness-boosted propensity must amplify Help: {tender} vs {baseline}"
        );
        assert!(
            full > tender,
            "higher propensity must amplify Help strictly more: {full} vs {tender}"
        );
    }

    /// §8.1.4 (Iteration 98): a lonely agent clears the interaction gate
    /// more often — `system_social_interactions` adds `loneliness ×
    /// social_loneliness_multiplier` to `interact_chance` (the family was
    /// computed by appraisal every tick and never read). Zero-at-zero: at
    /// loneliness 0 the gate is exactly the legacy formula. The gate roll
    /// itself is drawn unconditionally, but once a lonely agent converts an
    /// extra roll, `select_interaction_target`/`choose_interaction` consume
    /// more downstream RNG — so the runs are replay-deterministic per seed,
    /// not stream-identical; the higher threshold still converts strictly
    /// more rolls into interactions.
    #[test]
    fn loneliness_raises_interaction_gate() {
        let params = mindstrata_core::parameters::SimParameters::default();
        let count_interactions = |loneliness: Fixed| -> u32 {
            let mut rng = RngStreams::new(42);
            let mut events: Vec<SimEvent> = Vec::new();
            let mut relationships: Vec<Relationship> = Vec::new();
            let agent_info = vec![
                (
                    AgentId::new(0),
                    Fixed::from_f64(0.5), // openness
                    Fixed::from_f64(0.5), // agreeableness
                    Fixed::ZERO,          // extraversion (0 isolates loneliness)
                    Fixed::ZERO,          // anger
                    Fixed::ZERO,          // no_violence_resistance
                    Fixed::ZERO,          // help_neighbors_propensity
                    Fixed::ZERO,          // respect_elders_propensity
                    false,                // is_elder
                    Fixed::ZERO,          // obey_ruler_propensity
                    false,                // is_authority
                    loneliness,           // §8.1.4 (Iteration 98)
                    Fixed::ZERO,          // §8.1.6 (Iteration 162): sociability_dev
                    true,                 // §17.2 (Iteration 158): runs social
                ),
                (
                    AgentId::new(1),
                    Fixed::from_f64(0.5),
                    Fixed::from_f64(0.5),
                    Fixed::ZERO,
                    Fixed::ZERO,
                    Fixed::ZERO,
                    Fixed::ZERO,
                    Fixed::ZERO,
                    false,
                    Fixed::ZERO,
                    false,
                    Fixed::ZERO,
                    Fixed::ZERO,
                    true,
                ),
            ];
            let positions = vec![(0i32, 0i32), (1i32, 0i32)]; // both within radius 5
            let same_faction = vec![vec![false, false], vec![false, false]];
            let mut interactions = 0u32;
            for _ in 0..1000 {
                system_social_interactions(
                    &agent_info,
                    &positions,
                    &same_faction,
                    &mut relationships,
                    &[],
                    &mut events,
                    Tick::new(1),
                    &mut rng,
                    params.bonding_rate,
                    params.conflict_escalation_rate,
                    &params,
                );
            }
            for ev in &events {
                if let SimEvent::InteractionOccurred { .. } = ev {
                    interactions += 1;
                }
            }
            interactions
        };
        let baseline = count_interactions(Fixed::ZERO);
        let lonely = count_interactions(Fixed::from_f64(0.9));
        assert!(baseline > 0, "non-lonely agents must still interact");
        assert!(
            lonely > baseline,
            "lonely agents must clear the gate more often: {lonely} vs {baseline}"
        );
    }

    /// §8.1.4 (Iteration 98): loneliness alone — with zero extraversion and
    /// zero loneliness the gate is exactly the base chance; the multiplier
    /// only ever *raises* the gate (never lowers it), so a calmer agent is
    /// never penalized by this channel.
    #[test]
    fn loneliness_never_lowers_the_gate() {
        let params = mindstrata_core::parameters::SimParameters::default();
        let chance = |loneliness: Fixed| -> f64 {
            (params.social_interaction_base_chance
                + Fixed::ZERO * params.social_extraversion_multiplier
                + loneliness * params.social_loneliness_multiplier)
                .to_f64()
        };
        assert_eq!(
            chance(Fixed::ZERO),
            params.social_interaction_base_chance.to_f64()
        );
        assert!(chance(Fixed::from_f64(0.5)) > chance(Fixed::ZERO));
        assert!(chance(Fixed::ONE) >= chance(Fixed::from_f64(0.5)));
    }

    /// §8.1.6 (Iteration 162): a socially-tempered agent clears the
    /// interaction gate more often — `system_social_interactions` adds
    /// `sociability_dev × social_sociability_multiplier` to
    /// `interact_chance`. The deviation is zero at construction (byte-
    /// identical until the plasticity pass drifts the temperament layer),
    /// the RNG draw stays unconditional, and once a social agent converts an
    /// extra roll the downstream streams diverge per seed (replay-
    /// deterministic, not stream-identical).
    #[test]
    fn sociability_raises_interaction_gate() {
        let params = mindstrata_core::parameters::SimParameters::default();
        let count_interactions = |sociability_dev: Fixed| -> u32 {
            let mut rng = RngStreams::new(42);
            let mut events: Vec<SimEvent> = Vec::new();
            let mut relationships: Vec<Relationship> = Vec::new();
            let agent_info = vec![
                (
                    AgentId::new(0),
                    Fixed::from_f64(0.5), // openness
                    Fixed::from_f64(0.5), // agreeableness
                    Fixed::ZERO,          // extraversion (0 isolates sociability)
                    Fixed::ZERO,          // anger
                    Fixed::ZERO,          // no_violence_resistance
                    Fixed::ZERO,          // help_neighbors_propensity
                    Fixed::ZERO,          // respect_elders_propensity
                    false,                // is_elder
                    Fixed::ZERO,          // obey_ruler_propensity
                    false,                // is_authority
                    Fixed::ZERO,          // loneliness (0 isolates sociability)
                    sociability_dev,      // §8.1.6 (Iteration 162)
                    true,                 // §17.2: runs social
                ),
                (
                    AgentId::new(1),
                    Fixed::from_f64(0.5),
                    Fixed::from_f64(0.5),
                    Fixed::ZERO,
                    Fixed::ZERO,
                    Fixed::ZERO,
                    Fixed::ZERO,
                    Fixed::ZERO,
                    false,
                    Fixed::ZERO,
                    false,
                    Fixed::ZERO,
                    Fixed::ZERO,
                    true,
                ),
            ];
            let positions = vec![(0i32, 0i32), (1i32, 0i32)]; // both within radius 5
            let same_faction = vec![vec![false, false], vec![false, false]];
            let mut interactions = 0u32;
            for _ in 0..1000 {
                system_social_interactions(
                    &agent_info,
                    &positions,
                    &same_faction,
                    &mut relationships,
                    &[],
                    &mut events,
                    Tick::new(1),
                    &mut rng,
                    params.bonding_rate,
                    params.conflict_escalation_rate,
                    &params,
                );
            }
            for ev in &events {
                if let SimEvent::InteractionOccurred { .. } = ev {
                    interactions += 1;
                }
            }
            interactions
        };
        let baseline = count_interactions(Fixed::ZERO);
        let social = count_interactions(Fixed::from_f64(0.5));
        assert!(baseline > 0, "non-social agents must still interact");
        assert!(
            social > baseline,
            "socially-tempered agents must clear the gate more often: {social} vs {baseline}"
        );
    }

    /// §8.1.6 (Iteration 162): the sociability channel is a pure amplifier —
    /// with zero extraversion and zero loneliness the legacy gate is
    /// untouched at zero deviation, and the term only ever raises it.
    #[test]
    fn sociability_never_lowers_the_gate() {
        let params = mindstrata_core::parameters::SimParameters::default();
        let chance = |sociability_dev: Fixed| -> f64 {
            (params.social_interaction_base_chance
                + Fixed::ZERO * params.social_extraversion_multiplier
                + Fixed::ZERO * params.social_loneliness_multiplier
                + sociability_dev * params.social_sociability_multiplier)
                .to_f64()
        };
        assert_eq!(
            chance(Fixed::ZERO),
            params.social_interaction_base_chance.to_f64()
        );
        assert!(chance(Fixed::from_f64(0.5)) > chance(Fixed::ZERO));
        assert!(chance(Fixed::ONE) >= chance(Fixed::from_f64(0.5)));
    }

    // ── §17.2 (Iteration 158): Background-tier social-participation gate ──

    /// §17.2 (Iteration 158): `select_interaction_target` excludes
    /// ineligible (Background-tier) candidates — an eligible source whose
    /// only nearby neighbor is ineligible gets no target at all, and an
    /// ineligible candidate is never drawn even when mixed with eligible
    /// ones (the deterministic draw must stay within the eligible subset).
    #[test]
    fn target_selection_excludes_ineligible_agents() {
        // Positions: (0,0) and (1,0) — both within radius 5 of each other.
        let positions = [(0i32, 0i32), (1i32, 0i32)];

        // Source 0 eligible, target 1 ineligible → no candidate at all.
        let mut rng = RngStreams::new(42);
        let none = select_interaction_target(
            0,
            2,
            &positions,
            DEFAULT_PERCEPTION_RADIUS,
            &mut rng,
            &[true, false],
        );
        assert_eq!(
            none, None,
            "ineligible-only neighborhood must yield no target"
        );

        // Source 0 eligible, targets 1 (ineligible) + 2 (eligible): the draw
        // must always land on 2 across many rolls.
        let positions3 = [(0i32, 0i32), (1i32, 0i32), (1i32, 1i32)];
        let mut rng = RngStreams::new(7);
        for _ in 0..200 {
            let picked = select_interaction_target(
                0,
                3,
                &positions3,
                DEFAULT_PERCEPTION_RADIUS,
                &mut rng,
                &[true, false, true],
            );
            assert_eq!(picked, Some(2), "eligible subset must be the only pool");
        }
    }

    /// §17.2 (Iteration 158): an ineligible (Background) source neither
    /// initiates interactions nor is selected as a target — `
    /// system_social_interactions` skips it before the gate roll (so it
    /// consumes no Social-stream RNG) and the eligibility mask removes it
    /// from every other agent's candidate set. With a lone ineligible agent
    /// the pass is a complete no-op (zero events, zero relationships); with
    /// one ineligible + one eligible pair the eligible pair still
    /// interacts, proving the gate is differential, not a global freeze.
    #[test]
    fn background_sources_never_initiate_or_receive_interactions() {
        let params = mindstrata_core::parameters::SimParameters::default();
        // Element type inferred from the `social`/`background` builders and
        // the `system_social_interactions` call below (the 13-tuple shape).
        let run = |agent_info: Vec<_>| -> (u32, u32) {
            let mut rng = RngStreams::new(42);
            let mut events: Vec<SimEvent> = Vec::new();
            let mut relationships: Vec<Relationship> = Vec::new();
            let n = agent_info.len();
            let positions: Vec<(i32, i32)> = (0..n).map(|i| (i as i32, 0)).collect();
            let same_faction = vec![vec![false; n]; n];
            for _ in 0..1000 {
                system_social_interactions(
                    &agent_info,
                    &positions,
                    &same_faction,
                    &mut relationships,
                    &[],
                    &mut events,
                    Tick::new(1),
                    &mut rng,
                    params.bonding_rate,
                    params.conflict_escalation_rate,
                    &params,
                );
            }
            let bg_involved = events
                .iter()
                .filter(|ev| {
                    matches!(ev, SimEvent::InteractionOccurred { from, to, .. }
                        if from.as_u64() == 0 || to.as_u64() == 0)
                })
                .count() as u32;
            let total = events
                .iter()
                .filter(|ev| matches!(ev, SimEvent::InteractionOccurred { .. }))
                .count() as u32;
            (bg_involved, total)
        };

        let social = |id: u64| {
            (
                AgentId::new(id),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.5),
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                false,
                Fixed::ZERO,
                false,
                Fixed::ZERO,
                Fixed::ZERO, // §8.1.6 (Iteration 162): sociability_dev
                true,        // §17.2: participates
            )
        };
        let background = |id: u64| {
            let mut t = social(id);
            t.13 = false; // §17.2: aggregate tier — no individual interactions
            t
        };

        // Lone Background agent: complete no-op.
        let (bg, total) = run(vec![background(0)]);
        assert_eq!(
            (bg, total),
            (0, 0),
            "lone Background agent must not interact"
        );

        // One Background + two social agents: the eligible pair still
        // interacts (the Background's ineligibility must not freeze the
        // rest of the population).
        let (bg, total) = run(vec![background(0), social(1), social(2)]);
        assert_eq!(bg, 0, "Background agent must never be involved");
        assert!(total > 0, "eligible agents must still interact normally");

        // Two social agents (the all-eligible control): full interaction.
        let (_, total) = run(vec![social(0), social(1)]);
        assert!(total > 0, "all-eligible control must interact");
    }
}
