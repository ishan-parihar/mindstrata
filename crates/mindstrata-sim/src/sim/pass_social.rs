//! Tick pass 5: social interactions.

use super::{
    AgentBundle, AgentId, Fixed, InstitutionKind, InteractionKind, SimEvent, Simulation, Tick,
};
use crate::norms;
use crate::social;

impl Simulation {
    pub(super) fn tick_social_pass(
        ctx: &mut crate::systems::SystemContext,
        agents: &mut [AgentBundle],
        tick_u64: u64,
        emotions: &mut [crate::person::DiscreteEmotions],
        pre_tick_events: usize,
        tick: Tick,
        active_courtships: &mut [crate::social::courtship::Courtship],
        params: &crate::parameters::SimParameters,
        relationships: &mut [crate::person::Relationship],
        institutions_reg: &[crate::institutions::Institution],
        norms_reg: &crate::norms::NormRegistry,
    ) {
        // ── 5. Social interactions ────────────────────────────────
        {
            // §2.4: Pass agent positions for proximity-based social interactions
            let agent_positions: Vec<(i32, i32)> = agents
                .iter()
                .map(|a| (a.position.x, a.position.y))
                .collect();
            // §8.1.10 (Iteration 85): thread each agent's internalized
            // no-violence norm strength into the interaction decision —
            // `choose_interaction` scales its threat thresholds by
            // (1 − resistance). Resolved by id (`NO_VIOLENCE_NORM_ID`,
            // the same constant the check_violation sites use) so a
            // scenario that re-registers norms_reg with renamed descriptions
            // still gates correctly; a registry without the norm resolves
            // to zero resistance (legacy behavior). Zero-at-zero: before
            // the first monthly ritual (tick 4320) no agent holds any
            // internalized norm, so the thresholds are unchanged and the
            // golden baseline stays byte-identical.
            let no_violence_name = norms_reg
                .norms()
                .iter()
                .find(|n| n.id == norms::NO_VIOLENCE_NORM_ID)
                .map(|n| n.name.as_str());
            // §8.1.10 (Iteration 89): thread each agent's internalized
            // "Help Neighbors" norm strength into the interaction
            // decision — `choose_interaction` scales its high-affection
            // Help window by (1 + propensity). The prescriptive norm
            // uses the same id-resolved-by-description read as the
            // prohibitive ones (`norm_resistance` returns the internalized
            // strength); a registry without the norm resolves to zero
            // propensity (legacy behavior). Zero-at-zero: no internalized
            // norm before the first monthly ritual (tick 4320) → the
            // Help bound stays 0.5 and the golden baseline stays
            // byte-identical.
            let help_neighbors_name = norms_reg
                .norms()
                .iter()
                .find(|n| n.id == norms::HELP_NEIGHBORS_NORM_ID)
                .map(|n| n.name.as_str());
            // §8.1.10 (Iteration 91): thread each agent's internalized
            // "Respect Elders" norm strength plus the elder designation
            // into the interaction decision — `choose_interaction`
            // suppresses disrespectful acts toward the community's
            // designated elder (the Council "Elder" role holder, the live
            // elder anchor; age-based elders > 60 never appear at the
            // sim's 35040-tick-per-year timescale). Resolved once per
            // tick, deterministic, no RNG. Zero-at-zero: no internalized
            // norm before the first monthly ritual (tick 4320) → the
            // elder flag alone changes nothing → golden byte-identical.
            let respect_elders_name = norms_reg
                .norms()
                .iter()
                .find(|n| n.id == norms::RESPECT_ELDERS_NORM_ID)
                .map(|n| n.name.as_str());
            let elder_idx: Option<usize> = institutions_reg
                .iter()
                .find(|i| i.kind == InstitutionKind::Council)
                .and_then(|c| c.get_role_holder("Elder"))
                .map(|id| id.as_u64() as usize);
            // §8.1.10 (Iteration 93): the authority anchor for the
            // "Obey Ruler" gate — the Council "Guard Captain" role
            // holder, the ruler's enforcement arm. Distinct from the
            // Elder (separate role holders assigned at populate), so
            // the two gates never compound on the same target. Resolved
            // once per tick, deterministic, no RNG. A deceased captain
            // clears the anchor (the gate naturally goes inert — no
            // authority to defy).
            let obey_ruler_name = norms_reg
                .norms()
                .iter()
                .find(|n| n.id == norms::OBEY_RULER_NORM_ID)
                .map(|n| n.name.as_str());
            let authority_idx: Option<usize> = institutions_reg
                .iter()
                .find(|i| i.kind == InstitutionKind::Council)
                .and_then(|c| c.get_role_holder("Guard Captain"))
                .map(|id| id.as_u64() as usize);
            let agent_info: Vec<_> = agents
                .iter()
                .enumerate()
                .map(|(i, a)| {
                    let resistance = no_violence_name
                        .map_or(Fixed::ZERO, |n| a.moral_cognition.norm_resistance(n));
                    // §8.1.4 (Iteration 99 + 127): the help propensity
                    // — the combined prosocial push behind the Help
                    // interaction (the tuple element the interaction
                    // system destructures as `help_neighbors_propensity`
                    // and widens the Help window [0.2, 0.5×(1+p)) with).
                    // Norm resistance (Iter-89 Help Neighbors) plus
                    // tenderness (Iter-99, warmth→caregiving) plus
                    // gratitude (Iter-127, reciprocity→caregiving —
                    // appraisal producer `positive × (1 − expectedness)`,
                    // LIVE in calibrated windows, so this is the first
                    // CALIBRATED §8.1.4 consumer: golden + snapshots
                    // regenerated with before/after evidence). Zero
                    // emotions → the norm-only legacy value. The
                    // Help-window consumer clamps to [0.5, 1.0], so the
                    // sum can never overflow the decision window.
                    let help_propensity = crate::appraisal::help_propensity(
                        help_neighbors_name
                            .map_or(Fixed::ZERO, |n| a.moral_cognition.norm_resistance(n)),
                        emotions[i].tenderness,
                        emotions[i].gratitude,
                        // §8.1.6 (Iteration 180): the core altruism
                        // trait — the LAST decision-less core trait now
                        // has its consumer (the disposition→caregiving
                        // channel, ONE-SIDED identity-at-zero like the
                        // emotion tiers).
                        a.personality.altruism,
                        params.social_tenderness_help_multiplier.to_f64(),
                        params.social_gratitude_help_multiplier.to_f64(),
                        params.social_altruism_help_multiplier.to_f64(),
                    );
                    let respect_propensity = respect_elders_name
                        .map_or(Fixed::ZERO, |n| a.moral_cognition.norm_resistance(n));
                    let obey_propensity = obey_ruler_name
                        .map_or(Fixed::ZERO, |n| a.moral_cognition.norm_resistance(n));
                    (
                        AgentId::new(i as u64),
                        a.personality.openness,
                        a.personality.agreeableness,
                        a.personality.extraversion,
                        emotions[i].anger,
                        resistance,
                        help_propensity,
                        respect_propensity,
                        Some(i) == elder_idx,
                        obey_propensity,
                        Some(i) == authority_idx,
                        emotions[i].loneliness, // §8.1.4 (Iteration 98)
                        // §8.1.6 (Iteration 162): the sociability
                        // deviation from the trait-derived baseline —
                        // zero at construction, drifted by the
                        // plasticity pass. The interaction gate's third
                        // channel (socially-tempered agents clear the
                        // gate more often). Zero-at-zero by construction
                        // → calibrated runs byte-identical until the
                        // layer drifts.
                        a.personality.temperament.sociability
                            - crate::person::Temperament::from_traits(&a.personality).sociability,
                        // §17.2 (Iteration 158): Background-tier agents
                        // are excluded from the individual social
                        // interaction pass (aggregate simulation).
                        // Zero-blast by construction — no agent is ever
                        // Background in any calibrated window (Iter-145
                        // probe: 0B at every size/seed).
                        // §8.1.16 (Iteration 187 — the psychopathology
                        // consumer): a mental-health collapse
                        // (`is_impaired()`: overall_health < 0.5 — the
                        // sustained stress/trauma crisis band, not daily
                        // mood) also withdraws the agent from the social
                        // pass — depression and paranoia isolate. The
                        // `social_modifier()` interface is thereby live
                        // (its gate semantics: participation is cut once
                        // the impairment threshold is crossed).
                        a.agent_tier.tier.runs_social_interactions()
                            && !a.psychopathology.is_impaired(),
                    )
                })
                .collect();

            // §5.4: Compute faction membership matrix — same_faction_matrix[i][j] = true if agents i,j share a faction
            let faction_members: Vec<Vec<usize>> = institutions_reg
                .iter()
                .filter(|i| i.kind == InstitutionKind::Faction)
                .map(|f| f.members.iter().map(|m| m.as_u64() as usize).collect())
                .collect();
            let n = agents.len();
            let mut same_faction_matrix = vec![vec![false; n]; n];
            for member_list in &faction_members {
                for &a in member_list {
                    for &b in member_list {
                        if a < n && b < n {
                            same_faction_matrix[a][b] = true;
                        }
                    }
                }
            }

            social::system_social_interactions(
                &agent_info,
                &agent_positions,
                &same_faction_matrix,
                relationships,
                ctx.events,
                tick,
                ctx.rng,
                params.bonding_rate,
                params.conflict_escalation_rate,
                params,
            );

            // §8.1.11: Record speech acts — the structured linguistic frame
            // on the interaction system just run. Every InteractionOccurred
            // event this tick is interpreted into a SpeechAct and pushed onto
            // the speaker's bounded speech_log. Write-only observational
            // state (Iteration 40), so this cannot perturb calibrated runs.
            // Verified invariant: only process_interaction emits
            // InteractionOccurred (update_witnesses mutates witness
            // relationships directly and emits nothing), so every such event
            // in the window is a genuine speaker act.
            let mut spoken: Vec<(u64, u64, InteractionKind)> = Vec::new();
            for ev in &ctx.events[pre_tick_events..] {
                if let SimEvent::InteractionOccurred { from, to, kind, .. } = ev {
                    spoken.push((from.as_u64(), to.as_u64(), *kind));
                }
            }
            if !spoken.is_empty() {
                let n = agents.len();
                for (from_u, to_u, kind) in spoken {
                    let fi = from_u as usize;
                    if fi >= n {
                        continue;
                    }
                    // §10.4: familiarity grows with repeated interaction —
                    // both participants become more familiar with each
                    // other, scaled by how prosocial the interaction kind
                    // is. Previously `update_familiarity` had zero
                    // production callers, so the attraction model's
                    // familiarity term was pinned at 0 forever and
                    // `total_attraction()` (which gates the §8.1.16 D4
                    // courtship scenario) never responded to contact.
                    // Deterministic, no RNG: quality is a pure function
                    // of the kind.
                    let quality = crate::social::attraction::familiarity_quality_for(kind);
                    agents[fi].attraction.update_familiarity(quality);
                    let ti = to_u as usize;
                    if ti < n {
                        agents[ti].attraction.update_familiarity(quality);
                    }
                    // §10.4 (Iteration 76): The courtship stage ladder was
                    // dead — Courtship::record_positive/record_negative had
                    // zero production callers, so every courtship stayed
                    // pinned at Awareness until daily decay. Feed each
                    // interaction to the active courtship (either
                    // direction): prosocial acts advance the romantic
                    // ladder (gated by the pair's current trust and the
                    // speaker's attraction readiness), hostile acts regress
                    // it — deterministic, no RNG. The pursuer's
                    // AttractionModel.reciprocity mirrors the courtship's
                    // reciprocity tracker so total_attraction() finally
                    // responds to returned interest (the §8.1.16 D4 gate
                    // future-wiring documented at Iteration 75).
                    // Speaker's current trust in the listener — computed
                    // once, reused by both the courtship wiring below and
                    // the speech-act credibility.
                    let credibility = relationships
                        .iter()
                        .find(|r| r.from.as_u64() == from_u && r.to.as_u64() == to_u)
                        .map_or(Fixed::from_f64(0.5), |r| r.trust);
                    let kind_is_hostile = matches!(
                        kind,
                        mindstrata_core::event::InteractionKind::Threaten
                            | mindstrata_core::event::InteractionKind::Insult
                    );
                    for courtship in active_courtships.iter_mut() {
                        if !courtship.active {
                            continue;
                        }
                        let pair_matches = (courtship.pursuer == fi && courtship.pursued == ti)
                            || (courtship.pursuer == ti && courtship.pursued == fi);
                        if !pair_matches {
                            continue;
                        }
                        if kind_is_hostile {
                            courtship.record_negative(tick_u64);
                        } else {
                            // The speaker's attraction readiness feeds the
                            // courtship's mutual-attraction bookkeeping.
                            courtship.record_positive(
                                tick_u64,
                                credibility,
                                agents[fi].attraction.total_attraction(),
                            );
                        }
                        break;
                    }
                    // §19.5.D (Iteration 78): norm enforcement is NOT
                    // wired here — the pre-existing sites already record
                    // offenses through check_violation (the theft-fine
                    // path in the black-market/market block, the
                    // violence-conflict block, and the norm-evaluation
                    // pass's check_violation(1, …) call), so adding a
                    // fourth speech-act site would inflate offense_count
                    // and raise the punishment multiplier at those sites
                    // — drifting the snapshot-hashed fines/trust-erosion
                    // baselines. The Iteration-78 contribution to §19.5.D
                    // lives in CrimeRecord::record_offense: notoriety (the
                    // §10.4 social-cost driver) is episode-gated there
                    // (ENFORCEMENT_EPISODE_TICKS) — offense_count still
                    // grows every call (punishment trajectories
                    // byte-identical), but notoriety climbs once per
                    // distinct conflict episode instead of per raw act
                    // (probe: 57–316/agent saturated it at 1.0 for
                    // everyone; the gate keeps it a differentiating
                    // 0–0.9 signal).
                    // Tone from current affect valence (0..1, 0.5 neutral).
                    let tone = (agents[fi].affect.valence + Fixed::ONE) / Fixed::from_f64(2.0);
                    let act = crate::social::speech_act::SpeechAct::from_interaction(
                        kind,
                        AgentId::new(from_u),
                        AgentId::new(to_u),
                        tone,
                        credibility,
                        tick_u64,
                    );
                    let effect = act.resolve_effect();
                    let log = &mut agents[fi].speech_log;
                    log.push(act);
                    if log.len() > crate::social::speech_act::SPEECH_LOG_CAPACITY {
                        log.remove(0);
                    }
                    // §8.1.11 (Iteration 95): apply the speech act's
                    // relational effects — the trust/affection channels
                    // are already live (`system_social_interactions`'
                    // per-kind deltas equal the grounded `base_delta`
                    // values, so applying them here would double-count);
                    // status/obligation/reputation were computed by
                    // `resolve_effect` but never applied anywhere. Pure
                    // deltas on existing state, no RNG — the replay
                    // stream is untouched. The listener owes obligation
                    // to the speaker (promises/requests create debt —
                    // the daily power-balance pass reads it), admires
                    // the speaker per their reputation (aggression costs
                    // standing, informing builds it), and the speaker's
                    // respect for the listener tracks the listener's
                    // status in the speaker's eyes (comfort raises it;
                    // insult/threaten demeans — feeds `quality()`).
                    if ti < n {
                        // §10.2 (P5 audit, Iteration 184): the V2 deep
                        // dimensions were structurally disconnected —
                        // `record_positive`/`record_negative` had ZERO
                        // production call sites, so intimacy (and until
                        // now commitment) stayed 0.000 in every probed
                        // window. The interaction's signed magnitude now
                        // flows through the designed channels: positive
                        // interactions build trust/affection/intimacy/
                        // commitment/gratitude + memory weights, hostile
                        // ones erode them. These deltas are on the V2
                        // array — independent of the legacy
                        // `relationships` per-kind deltas, so nothing is
                        // double-counted. Zero-RNG, deterministic.
                        let magnitude = match kind {
                            mindstrata_core::event::InteractionKind::Help
                            | mindstrata_core::event::InteractionKind::Comfort => {
                                Fixed::from_f64(0.6)
                            }
                            mindstrata_core::event::InteractionKind::Threaten
                            | mindstrata_core::event::InteractionKind::Insult => {
                                Fixed::from_f64(0.5)
                            }
                            mindstrata_core::event::InteractionKind::Gossip
                            | mindstrata_core::event::InteractionKind::Talk => Fixed::from_f64(0.3),
                            _ => Fixed::from_f64(0.2),
                        };
                        // Iteration 197: a fully-socialized agent (the §17
                        // developmental state — grows through childhood)
                        // registers interactions more strongly, so its
                        // bonds advance faster. Baseline-corrected: scale
                        // is exactly 1.0 at the default 0.5 (all adult
                        // populations are byte-identical), rising to 1.25
                        // at full socialization, falling to 0.75 at
                        // unsocialized 0 — only agents whose socialization
                        // actually developed diverge.
                        let ti_social = Fixed::ONE
                            + (agents[ti].developmental.socialization_completeness
                                - Fixed::from_f64(0.5))
                                * Fixed::from_f64(0.5);
                        let fi_social = Fixed::ONE
                            + (agents[fi].developmental.socialization_completeness
                                - Fixed::from_f64(0.5))
                                * Fixed::from_f64(0.5);
                        // Iteration 198: the §8.1.9 Theory of Mind model
                        // was write-only AND its intents were structurally
                        // unreachable (the update derived pos/neg from
                        // trust with scaled-down multipliers, so Friendly/
                        // Threatening could never fire). The speaker's
                        // perceived intent of the listener now modulates
                        // the interaction's relational weight: an agent
                        // who models the listener as Friendly extends
                        // MORE bond (×1.25 on positive), as
                        // Threatening/Deceptive LESS (×0.75); hostile
                        // acts reverse the sign. Identity at Unknown/
                        // Neutral (scale exactly 1.0), so default
                        // populations stay byte-identical.
                        let fi_model = agents[fi]
                            .mind_models
                            .get(mindstrata_core::id::AgentId::new(to_u));
                        let fi_intent = match fi_model {
                            Some(m) => m.perceived_intent,
                            None => crate::psychology::theory_of_mind::IntentPerception::Unknown,
                        };
                        let fi_tom = match fi_intent {
                            crate::psychology::theory_of_mind::IntentPerception::Friendly => {
                                Fixed::from_f64(1.25)
                            }
                            crate::psychology::theory_of_mind::IntentPerception::Threatening
                            | crate::psychology::theory_of_mind::IntentPerception::Deceptive => {
                                Fixed::from_f64(0.75)
                            }
                            crate::psychology::theory_of_mind::IntentPerception::Unknown
                            | crate::psychology::theory_of_mind::IntentPerception::Neutral => {
                                Fixed::ONE
                            }
                        };
                        let l_pos = Self::relationship_v2_pos(ti, fi);
                        let rv2 = &mut agents[ti].relationship_v2s[l_pos];
                        rv2.obligation = (rv2.obligation + effect.obligation_delta).clamp_01();
                        rv2.admiration = (rv2.admiration + effect.reputation_delta).clamp_01();
                        let magnitude_ti = (magnitude * ti_social).clamp_01();
                        if kind_is_hostile {
                            rv2.record_negative(tick_u64, magnitude_ti);
                        } else {
                            rv2.record_positive(tick_u64, magnitude_ti);
                        }
                        let s_pos = Self::relationship_v2_pos(fi, ti);
                        let rv2s = &mut agents[fi].relationship_v2s[s_pos];
                        rv2s.respect = (rv2s.respect + effect.status_delta).clamp_01();
                        let magnitude_fi = (magnitude * fi_social * fi_tom).clamp_01();
                        if kind_is_hostile {
                            rv2s.record_negative(tick_u64, magnitude_fi);
                        } else {
                            rv2s.record_positive(tick_u64, magnitude_fi);
                        }
                    }
                }
            }
        }
    }
}
