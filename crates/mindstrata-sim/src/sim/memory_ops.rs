//! Trauma/founding memory recording and memory encoding.

use super::skill_gain_next;
use super::{
    skill_milestone_crossed, AgentId, Fixed, MemoryKind, MemoryTag, PerceptKind, RngStream,
    SimEvent, Simulation,
};

/// §2.4 (i334): the perception anchors of one tick-event — the positions the
/// event *happened at*, one per agent lane.
///
/// Before i334 the memory-encoding pass ran `compute_salience` (and with it a
/// habituation increment) for **every** event for **every** agent, i.e. N·E
/// salience evaluations per tick, regardless of whether the agent could
/// possibly have perceived the event. i333 sized that: at N=192 only **0.9%**
/// of those pairs involve the agent, so 99.1% of the pass was work no
/// perception model would license. §2.4 already fixes the radius
/// (`DEFAULT_PERCEPTION_RADIUS`, the same one the interaction engine uses) —
/// this is that radius applied to attention.
///
/// An agent perceives an event when it sits within the radius of an anchor;
/// an involved agent is its own anchor at distance 0, so involvement is the
/// degenerate case and the gates compose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EventAnchors {
    /// The event names no agent lane — perceived by every agent. This is the
    /// pre-i334 behavior, kept only for `InstitutionChangedPolicy` (an
    /// institution's policy change is public village news).
    Everywhere,
    /// Every agent lane pointed outside the living population (a stale id),
    /// so there is nothing to stand near — nobody perceives it.
    Nowhere,
    One((i32, i32)),
    Two((i32, i32), (i32, i32)),
    /// Three or more lanes (births, rituals, witness lists) — rare.
    Many(Vec<(i32, i32)>),
}

impl EventAnchors {
    /// Does an agent standing at `me` perceive this event?
    pub(crate) fn perceived_from(&self, me: (i32, i32), radius: i32) -> bool {
        let near = |p: &(i32, i32)| (p.0 - me.0).abs() + (p.1 - me.1).abs() <= radius;
        match self {
            EventAnchors::Everywhere => true,
            EventAnchors::Nowhere => false,
            EventAnchors::One(a) => near(a),
            EventAnchors::Two(a, b) => near(a) || near(b),
            EventAnchors::Many(ps) => ps.iter().any(near),
        }
    }
}

/// The position of agent `id`, when that id is still inside the population.
///
/// A lane whose agent died earlier in the tick can point outside `positions`;
/// its anchor is dropped (the event is still perceived by every lane that
/// survived). This is the same stale-id guard the i331 social-status fold
/// uses.
fn lane_position(positions: &[(i32, i32)], id: AgentId) -> Option<(i32, i32)> {
    positions.get(id.as_u64() as usize).copied()
}

/// §2.4 (i334): build the perception anchors of `event` from the tick's final
/// agent positions.
///
/// The match is **exhaustive by design**: a new `SimEvent` variant must state
/// its perception lanes explicitly rather than inherit a silent default.
/// Positions are the tick's *final* ones, so an anchor is where the agent
/// ended the tick, not where it stood at event time — an approximation that is
/// invisible at the r=5 scale the interaction engine already uses.
pub(crate) fn anchors_of(event: &SimEvent, positions: &[(i32, i32)]) -> EventAnchors {
    let one = |id: AgentId| lane_position(positions, id);
    let pair = |a: AgentId, b: AgentId| match (one(a), one(b)) {
        (Some(x), Some(y)) => EventAnchors::Two(x, y),
        (Some(x), None) | (None, Some(x)) => EventAnchors::One(x),
        (None, None) => EventAnchors::Nowhere,
    };
    match event {
        // ── Two lanes ────────────────────────────────────────────────
        SimEvent::RelationshipChanged { from, to, .. }
        | SimEvent::InteractionOccurred { from, to, .. } => pair(*from, *to),
        SimEvent::TradeOccurred { buyer, seller, .. } => pair(*buyer, *seller),
        SimEvent::RumorSpread { source, target, .. }
        | SimEvent::KnowledgeTransferred { source, target, .. } => pair(*source, *target),
        SimEvent::ConflictOccurred {
            aggressor, target, ..
        } => pair(*aggressor, *target),
        SimEvent::FeudFormed {
            party_a, party_b, ..
        } => pair(*party_a, *party_b),
        SimEvent::MarriageFormed {
            spouse_a, spouse_b, ..
        } => pair(*spouse_a, *spouse_b),
        // ── One lane ─────────────────────────────────────────────────
        SimEvent::AgentAte { agent, .. }
        | SimEvent::AgentDrank { agent, .. }
        | SimEvent::AgentRested { agent, .. }
        | SimEvent::AgentSpawned { agent, .. }
        | SimEvent::AgentDied { agent, .. }
        | SimEvent::AgentMoved { agent, .. } => match one(*agent) {
            Some(p) => EventAnchors::One(p),
            None => EventAnchors::Nowhere,
        },
        SimEvent::GriefStruck { mourner, .. } => match one(*mourner) {
            Some(p) => EventAnchors::One(p),
            None => EventAnchors::Nowhere,
        },
        // ── Three or more lanes ──────────────────────────────────────
        SimEvent::ChildBorn {
            child,
            parent_a,
            parent_b,
            ..
        } => EventAnchors::Many(
            [one(*child), one(*parent_a), one(*parent_b)]
                .into_iter()
                .flatten()
                .collect(),
        ),
        SimEvent::NormViolated {
            agent, witnesses, ..
        } => EventAnchors::Many(
            std::iter::once(one(*agent))
                .chain(witnesses.iter().map(|w| one(*w)))
                .flatten()
                .collect(),
        ),
        SimEvent::RitualPerformed { participants, .. }
        | SimEvent::MourningObserved { participants, .. } => {
            EventAnchors::Many(participants.iter().filter_map(|p| one(*p)).collect())
        }
        // ── No lane: public institutional news ───────────────────────
        SimEvent::InstitutionChangedPolicy { .. } => EventAnchors::Everywhere,
    }
}

/// §19.5.G (i331): per-agent `(positive, total)` relationship counts.
///
/// A relationship is "positive" when `trust > 0.6`. Each row contributes to
/// exactly its own `from` agent, so this single O(R) pass yields the same
/// per-agent counts as the old per-agent `relationships.iter().filter(..)`
/// fold — but at O(R) instead of O(N·R) = O(N³)/tick. Rows whose `from` is
/// outside the current population (a stale id after a death) are skipped,
/// matching the old fold (which could only match an in-range index).
///
pub(crate) fn social_status_counts(
    relationships: &[crate::person::Relationship],
    n: usize,
) -> Vec<(u32, u32)> {
    let mut counts = vec![(0u32, 0u32); n];
    for r in relationships {
        let fi = r.from.as_u64() as usize;
        if fi < n {
            counts[fi].1 += 1;
            if r.trust > Fixed::from_f64(0.6) {
                counts[fi].0 += 1;
            }
        }
    }
    counts
}

/// i342: per-agent **contacted degree** — how many ordered rows carry any
/// interaction state (`interaction_count > 0`).
///
/// The i341 coupling map found two appraisal channels reading the agent's
/// relationship-list LENGTH as its number of social connections. That length is
/// the complete graph's N−1 for every agent (the i335/i336 pin), so the proxy
/// measures the population, not sociality: probe `i342_social_count_proxy`
/// measured `min(len, 4) == 4` for **every agent at every N** (lists of 11/47/95),
/// pinning `social_visibility` at a constant 0.900/0.400 and the normal-life
/// anxiety term at exactly 0.000 for the whole village.
///
/// Contacted degree is the honest count, it differentiates an isolate from the
/// village's best-connected agent, and it is the same quantity a sparse store
/// must define (i341).
///
/// i403: re-pointed from the legacy v1 matrix to the DYADIC rows — the v1
/// `interaction_count` stamps died with the v1 interaction writes, so the
/// old predicate read a frozen surface (every agent's degree → 0), collapsed
/// `update_narrative_importance`'s network bonus, and mass-demoted the
/// village to Background (the tier-mix pin caught it: "Background appeared
/// at 6 agents", and the interaction volume fell 35–65% because demoted
/// agents are excluded from the social pass). `RelationshipV2.interaction_count`
/// is stamped by the same `record_positive/record_negative` path that now
/// carries every per-act write, so the predicate measures the same thing it
/// always did. One O(N²) pass over the per-agent rows — the i331 pattern for
/// replacing per-agent matrix scans.
pub(crate) fn contacted_degrees(agents: &[crate::sim::AgentBundle], n: usize) -> Vec<u32> {
    let mut degrees = vec![0u32; n];
    for (i, agent) in agents.iter().enumerate().take(n) {
        degrees[i] = agent
            .relationship_v2s
            .iter()
            .filter(|r| r.interaction_count > 0)
            .count() as u32;
    }
    degrees
}

impl Simulation {
    /// §13.5: Record a famine (scarcity) trauma in the village's collective
    /// memory when hunger or thirst runs critically high. Episode-guarded:
    /// a scarcity trauma is not re-recorded while a recent one exists, so
    /// one crisis yields one memory. Deterministic (reads agent needs).
    pub(super) fn record_famine_memory(&mut self, tick: u64) {
        let n = self.agents.len().max(1);
        let inv = Fixed::from_f64(1.0 / n as f64);
        let avg_hunger = self
            .agents
            .iter()
            .map(|a| a.needs.hunger)
            .fold(Fixed::ZERO, |acc, h| acc + h)
            * inv;
        let avg_thirst = self
            .agents
            .iter()
            .map(|a| a.needs.thirst)
            .fold(Fixed::ZERO, |acc, t| acc + t)
            * inv;
        crate::culture::maybe_record_famine(
            &mut self.collective_memory_registry,
            avg_hunger,
            avg_thirst,
            tick,
        );
    }

    /// §13.5: Record a faction's founding myth — group 1 + faction id keeps
    /// faction memory separate from the village's (group 0).
    pub(super) fn record_faction_founding_memory(&mut self, faction_id: usize, tick: u64) {
        crate::culture::record_faction_founding(
            &mut self.collective_memory_registry,
            faction_id,
            tick,
        );
    }

    /// Section 9: Memory encoding from this tick's events.
    pub(super) fn tick_memory_encoding(
        &mut self,
        pre_tick_events: usize,
        tick_u64: u64,
        phases: crate::scheduler::TickPhases,
    ) {
        // ── 9. Memory encoding from this tick's events ───────────────
        // §22.5: Use attention system to compute salience instead of hardcoded values.
        // Only events that pass the attention threshold are encoded into memory.
        //
        // i331: the §19.5.G social-status fold below used to re-scan the whole
        // relationship matrix *per agent* (`iter().filter(r.from == i)`), i.e.
        // O(N·R) = O(N³)/tick inside the tick's largest pass. Hoisted to one
        // O(R) pass (each row contributes to exactly its own `from` agent), so
        // the counts are identical by construction.
        let rel_counts = social_status_counts(&self.relationships, self.agents.len());
        // §2.4 (i334): perception gate. One O(E) pass builds each tick-event's
        // anchors from the tick's final positions; the inner loop then tests a
        // Manhattan distance instead of computing salience for events the
        // agent could not have perceived. i333 measured 0.9% of the pre-i334
        // pairs as agent-involving at N=192, so this removes ~99% of the
        // pass's work *and* the causal implausibility it stood on (before: a
        // village-wide agent habituated to every event in the village).
        let positions: Vec<(i32, i32)> = self
            .agents
            .iter()
            .map(|a| (a.position.x, a.position.y))
            .collect();
        let radius = crate::social::interaction::DEFAULT_PERCEPTION_RADIUS;
        let tick_events = &self.events[pre_tick_events..];
        let anchors: Vec<EventAnchors> = tick_events
            .iter()
            .map(|ev| anchors_of(ev, &positions))
            .collect();
        for (i, agent) in self.agents.iter_mut().enumerate() {
            // §17: Background agents skip memory encoding entirely
            // §17.2: Also check memory retrieval budget
            if !agent.agent_tier.tier.runs_memory_encoding()
                || !agent.agent_tier.budget_tracker.can_memory_op()
            {
                continue;
            }
            // §8.1.2: Fresh salience competition each tick — the map records
            // this tick's percepts, not a rolling top-N.
            agent.attention.salience_map.clear();
            let me = (agent.position.x, agent.position.y);
            for (ev, footprint) in tick_events.iter().zip(anchors.iter()) {
                if !footprint.perceived_from(me, radius) {
                    continue;
                }
                // §22.5: Attention computes salience based on intensity, novelty, relevance
                let salience = agent.attention.compute_salience(
                    ev,
                    AgentId::new(i as u64),
                    &agent.needs,
                    &agent.affect,
                );

                // §8.1.2: Record the percept into the salience competition
                // (write-only observational state — keeps the top percepts by
                // computed salience, so the salience gate itself is untouched).
                let percept_agent = match ev {
                    SimEvent::InteractionOccurred { from, to, .. } => {
                        Some(if from.as_u64() == i as u64 {
                            *to
                        } else {
                            *from
                        })
                    }
                    SimEvent::AgentAte { agent: a, .. }
                    | SimEvent::AgentDrank { agent: a, .. }
                    | SimEvent::RelationshipChanged { from: a, .. } => Some(*a),
                    _ => None,
                };
                agent.attention.record_salience(
                    PerceptKind::of(ev),
                    percept_agent,
                    salience,
                    tick_u64,
                );

                // Only encode events that exceed the attention threshold
                if salience < Fixed::from_f64(0.2) {
                    continue;
                }

                // Emotional intensity scales with arousal
                let emotional = agent.affect.arousal * Fixed::from_f64(0.6) + Fixed::from_f64(0.1);
                match ev {
                    SimEvent::AgentAte { agent: a, .. } if a.as_u64() == i as u64 => {
                        if agent.agent_tier.budget_tracker.can_memory_op() {
                            let _ = agent.agent_tier.budget_tracker.consume_memory_op();
                            agent.memory.encode(
                                MemoryKind::Somatic,
                                tick_u64,
                                salience,
                                emotional,
                                None,
                                MemoryTag::AteFood,
                            );
                        }
                    }
                    SimEvent::AgentDrank { agent: a, .. } if a.as_u64() == i as u64 => {
                        if agent.agent_tier.budget_tracker.can_memory_op() {
                            let _ = agent.agent_tier.budget_tracker.consume_memory_op();
                            agent.memory.encode(
                                MemoryKind::Somatic,
                                tick_u64,
                                salience,
                                emotional,
                                None,
                                MemoryTag::DrankWater,
                            );
                        }
                    }
                    SimEvent::AgentRested { agent: a, .. } if a.as_u64() == i as u64 => {
                        if agent.agent_tier.budget_tracker.can_memory_op() {
                            let _ = agent.agent_tier.budget_tracker.consume_memory_op();
                            agent.memory.encode(
                                MemoryKind::Emotional,
                                tick_u64,
                                salience,
                                emotional,
                                None,
                                MemoryTag::Rested,
                            );
                        }
                    }
                    SimEvent::InteractionOccurred { from, to, kind, .. } => {
                        if from.as_u64() == i as u64 {
                            let tag = match kind {
                                mindstrata_core::event::InteractionKind::Help => {
                                    MemoryTag::HelpedBy
                                }
                                mindstrata_core::event::InteractionKind::Threaten => {
                                    MemoryTag::ThreatenedBy
                                }
                                mindstrata_core::event::InteractionKind::Insult => {
                                    MemoryTag::InsultedBy
                                }
                                mindstrata_core::event::InteractionKind::Gossip => {
                                    MemoryTag::GossipedAbout
                                }
                                mindstrata_core::event::InteractionKind::Trade => {
                                    MemoryTag::TradedWith
                                }
                                _ => MemoryTag::TalkedTo,
                            };
                            let kind = match kind {
                                mindstrata_core::event::InteractionKind::Threaten
                                | mindstrata_core::event::InteractionKind::Insult => {
                                    MemoryKind::Traumatic
                                }
                                mindstrata_core::event::InteractionKind::Help
                                | mindstrata_core::event::InteractionKind::Comfort => {
                                    MemoryKind::Emotional
                                }
                                _ => MemoryKind::Social,
                            };
                            if agent.agent_tier.budget_tracker.can_memory_op() {
                                let _ = agent.agent_tier.budget_tracker.consume_memory_op();
                                agent.memory.encode(
                                    kind,
                                    tick_u64,
                                    salience,
                                    emotional,
                                    Some(to.as_u64() as u32),
                                    tag,
                                );
                            }
                        }
                        if to.as_u64() == i as u64
                            && agent.agent_tier.budget_tracker.can_memory_op()
                        {
                            let _ = agent.agent_tier.budget_tracker.consume_memory_op();
                            agent.memory.encode(
                                MemoryKind::Social,
                                tick_u64,
                                salience,
                                emotional,
                                Some(from.as_u64() as u32),
                                MemoryTag::TalkedTo,
                            );
                        }
                    }
                    _ => {}
                }
            }
            // Probabilistic memory rehearsal (every 10 ticks)
            if phases.is_deca {
                agent
                    .memory
                    .rehearse_random(tick_u64, self.rng.get_mut(RngStream::Behavior));
            }
            // Decay memories
            agent.memory.decay(tick_u64);

            // §22.5: Attention maintenance — decay habituation, replenish budget
            agent.attention.decay_habituation(Fixed::from_f64(0.005));
            let stress = agent.emotions.fear + agent.emotions.anger;
            agent
                .attention
                .replenish_budget(stress, agent.needs.fatigue);
            // §8.1.2: Recompute the perceptual biases from current agent state
            // (pure function, no RNG — write-only observational state, so
            // calibrated trajectories are untouched).
            agent.attention.recompute_biases(
                agent.emotions.fear,
                agent.emotions.anger,
                agent.derived.trauma_risk,
                agent.personality.extraversion,
                agent.personality.openness,
                agent.attachment.security,
                // §9.2: yesterday's surprise raises today's novelty-seeking
                // attention (the neural pass below updates
                // `last_prediction_error`; the value seen here is the prior
                // tick's). Zero in calm windows → byte-identical.
                agent.neural_like.expectation.last_prediction_error,
                // §7.2.2 (Iteration 187 — the S2-2-2 arousal consumer): the
                // biological endocrine arousal level now feeds the threat
                // bias — a physiologically aroused agent is hypervigilant.
                // Mean-zero at the 0.5 anchor inside `recompute_biases`.
                agent.embodied.endocrine.arousal.level,
            );

            // §19.5.G: Update status from current wealth and social connections
            agent.status.wealth_status = (agent.wealth.coin / Fixed::from_f64(20.0)).clamp_01();
            // Social status: ratio of positive relationships to total relationships
            // §19.5.G: A relationship is "positive" when trust exceeds 0.6
            let (positive_rels, total_rels) = rel_counts[i];
            if total_rels > 0 {
                agent.status.social_status =
                    Fixed::from_f64(positive_rels as f64 / total_rels as f64);
            }
            agent.status.recompute();

            // §22 / §8.1.3: Memory reconsolidation — current emotions bias recalled
            // memories and erode their accuracy (recording distortion events).
            agent
                .memory
                .reconsolidate(tick_u64, agent.emotions.anger, agent.emotions.joy);

            // §8.1.3: Retrieval reconstructs — the most accessible trace
            // (salience argmax: strength + charge/2 + identity/4) is recalled
            // and re-encoded under the current emotional state: rehearsal
            // strengthens it while an emotion-mismatched reconstruction erodes
            // accuracy and records a RetrievalReconstruction distortion event.
            // High-charge (traumatic) traces always win the argmax, so they
            // intrude preferentially. Deterministic (no RNG), purely
            // observational memory state — calibrated runs stay byte-identical.
            agent.memory.retrieve_and_reconsolidate(
                tick_u64,
                agent.emotions.anger,
                agent.emotions.joy,
            );

            // §9.2: Deterministic neural-like runtime. The concept vector is
            // derived deterministically from the agent's live state (no RNG)
            // and spread through the association network. Since Iteration
            // 183d the prediction error feeds three live consumers —
            // attention (novelty_bias, continuous PE × 0.2 via
            // recompute_biases), belief reinforcement, and emotional
            // intensity (arousal, both gated at 0.3). All are exactly zero
            // in calm windows (zero prediction error), so calibrated
            // trajectories stay byte-identical; the learned RL values
            // remain observational (consumed only via `learned_delta` in the
            // action-value fold, actions.rs).
            let concepts = crate::psychology::neural_like::ConceptVector {
                safety: Fixed::ONE - agent.emotions.fear,
                sacredness: if agent.cultural.ideology.is_some() {
                    Fixed::from_f64(0.5)
                } else {
                    Fixed::ZERO
                },
                status: agent.status.social_status,
                kinship: if agent.partner.is_some() || agent.parent_a.is_some() {
                    Fixed::from_f64(0.5)
                } else {
                    Fixed::ZERO
                },
                scarcity: Fixed::ONE - agent.needs.safety,
                threat: agent.emotions.fear,
                purity: agent.moral_values.purity,
                freedom: Fixed::ONE - agent.personality.conformity,
                loyalty: agent.moral_values.loyalty,
                pleasure: agent.emotions.joy,
                shame: agent.emotions.shame,
                hope: agent.emotions.hope,
            };
            agent
                .neural_like
                .network
                .spread(concepts, Fixed::from_f64(0.8));
            // Predictive error: observe the current success rate as the
            // outcome of the agent's recent attempts.
            let attempts = agent.recent_attempts;
            if attempts > 0 {
                let success_rate = Fixed::from_int(agent.recent_successes as i64)
                    / Fixed::from_int(attempts as i64);
                agent.neural_like.expectation.observe(success_rate);
            }
            // §9.2 (Iteration 183d): LARGE prediction error creates emotional
            // intensity — a violated world model spikes arousal (the
            // intensity axis), amplifying the emotional weight of subsequent
            // decisions. Gated at 0.3 (calm windows carry zero prediction
            // error, so the golden baseline stays byte-identical); at 0.1
            // gain a 0.3 surprise adds 0.03 arousal — a modest, honest
            // intensity bump.
            let surprise = agent.neural_like.expectation.last_prediction_error;
            if surprise > Fixed::from_f64(0.3) {
                agent.affect.arousal =
                    (agent.affect.arousal + surprise * Fixed::from_f64(0.1)).clamp_01();
            }
            // §9.2 script grammar: partnered agents replay the courtship script
            // as an observational narrative track (no behavioral effect).
            if agent.partner.is_some() {
                let script = agent
                    .neural_like
                    .script
                    .get_or_insert_with(crate::psychology::neural_like::BehaviorScript::courtship);
                script.next_step();
            }

            // §4.2: Skill improvement — agents improve skills through repeated
            // practice (Iteration 260: power-law curve at the practice site
            // below; the flat SKILL_GAIN_PER_TICK increment is retired).
            // §8.1.3: Procedural memory — practice crossing a 0.1-proficiency
            // milestone encodes the mastered routine (sparse by design: with a
            // 0.001 gain a milestone needs ~100 practice ticks, keeping the
            // 200-capacity store focused on the vivid events).
            let practiced: Option<&mut Fixed> = match agent.current_action {
                crate::actions::ActionKind::Work => Some(&mut agent.skills.farming),
                crate::actions::ActionKind::Trade => Some(&mut agent.skills.trading),
                crate::actions::ActionKind::Socialize | crate::actions::ActionKind::Worship => {
                    Some(&mut agent.skills.social)
                }
                _ => None,
            };
            // §8.1.19 (P3-1, August 14, 2026): the psychology SkillState was
            // structurally unwired — `practice`/`form_habit`/`execute_habit`
            // had ZERO production call sites, so the skills and habits maps
            // stayed permanently empty (probe-pinned: skill_count/habit_count
            // 0.000 for 12/12 agents in every window). The same action
            // practice now feeds BOTH the §4.2 person-level proficiency above
            // AND the psychology skill system below: repeated practice grows
            // proficiency, and crossing the 0.1-proficiency milestone forms
            // (or reinforces) the corresponding habit.
            let psych_skill = match agent.current_action {
                crate::actions::ActionKind::Work => {
                    Some((crate::psychology::skill::SKILL_FARMING, "Work", "hunger"))
                }
                crate::actions::ActionKind::Trade => {
                    Some((crate::psychology::skill::SKILL_TRADING, "Trade", "thirst"))
                }
                crate::actions::ActionKind::Socialize => Some((
                    crate::psychology::skill::SKILL_SPEAKING,
                    "Socialize",
                    "social",
                )),
                crate::actions::ActionKind::Worship => {
                    Some((crate::psychology::skill::SKILL_RITUAL, "Worship", "meaning"))
                }
                crate::actions::ActionKind::Eat => {
                    Some((crate::psychology::skill::SKILL_COOKING, "Eat", "hunger"))
                }
                // Iteration 234: expanded action-to-skill mappings so
                // more skill types are practiced during normal behavior.
                crate::actions::ActionKind::Wander => Some((
                    crate::psychology::skill::SKILL_LEADERSHIP,
                    "Wander",
                    "safety",
                )),
                crate::actions::ActionKind::Rest => {
                    Some((crate::psychology::skill::SKILL_PARENTING, "Rest", "fatigue"))
                }
                crate::actions::ActionKind::Idle => {
                    Some((crate::psychology::skill::SKILL_DIPLOMACY, "Idle", "social"))
                }
                _ => None,
            };
            if let Some((skill_id, action_name, trigger)) = psych_skill {
                if practiced.is_some() {
                    // §8.1.19: practice grows proficiency (difficulty-scaled,
                    // clamped); neuroplasticity is the agent's own trait.
                    // Iteration 197: cognitive_development (the §17
                    // life-stage state — 0.1 infant → 1.0 adult → decline
                    // past 50) modulates how easily the agent learns.
                    // Baseline-corrected: exactly 1.0 at the default 0.7,
                    // rising to 1.3 at full adult cognition, falling to 0.85
                    // at infant 0.1 — populate-default agents are exact
                    // no-ops (the developmental pass only rewrites
                    // cognitive_development for Focal agents), and only
                    // agents whose cognition actually developed diverge.
                    let cognitive_scale = Fixed::ONE
                        + (agent.developmental.cognitive_development - Fixed::from_f64(0.7))
                            * Fixed::from_f64(0.5);
                    // Iteration 260: capture pre-practice proficiency for the
                    // milestone gate — psych proficiency and the person-level
                    // skill now follow different curves, so the old
                    // `prof - skill_gain` proxy no longer tracks this tick's
                    // own increment.
                    let prof_before = agent.psych_skills.proficiency(skill_id);
                    agent.psych_skills.practice(
                        skill_id,
                        tick_u64,
                        (agent.psych_skills.neuroplasticity * cognitive_scale).clamp_01(),
                    );
                    // §8.1.19: habit formation at the 0.1-proficiency
                    // milestone — the same gate the procedural-memory encode
                    // uses, so habits arrive with genuine mastery (~100
                    // practice ticks). Deterministic, no RNG.
                    let prof = agent.psych_skills.proficiency(skill_id);
                    let crossed =
                        prof >= Fixed::from_f64(0.1) && prof_before < Fixed::from_f64(0.1);
                    if crossed {
                        agent.psych_skills.form_habit(
                            action_name.to_string(),
                            trigger.to_string(),
                            Fixed::from_f64(0.3),
                            tick_u64,
                        );
                    } else {
                        // §8.1.19 (P3-1 completion, re-audit): practicing
                        // the action exercises any matching habit — refresh
                        // recency so the centum decay pass cannot grind
                        // formed habits to zero while the agent keeps
                        // performing the action (the pre-fix habit_count
                        // collapsed 1.17 @2K → 0.000 @10K because
                        // `execute_habit`'s automaticity > 0.5 gate was
                        // unreachable and `last_performed` never refreshed).
                        agent.psych_skills.refresh_habit(trigger, tick_u64);
                    }
                }
            }
            if let Some(skill) = practiced {
                // Iteration 260 (audit E8): power-law learning curve — the
                // gain scales with remaining headroom, so mastery asymptotes
                // below the clamp instead of pinning population-wide at
                // exactly 1.0. Probe evidence (i260_skills, pre-change,
                // 20K ticks ×3 seeds): farming mean=1.000 sd=0.000 for
                // every agent — the flat increment carried zero information
                // about practice frequency. Quantized sequence saturates
                // ≈0.988; differentiation now tracks practice share.
                // f64 core + single quantize per §5 (sub-resolution rates).
                let before = *skill;
                *skill = skill_gain_next(before);
                if skill_milestone_crossed(before, *skill)
                    && agent.agent_tier.budget_tracker.can_memory_op()
                {
                    let _ = agent.agent_tier.budget_tracker.consume_memory_op();
                    let emotional =
                        agent.affect.arousal * Fixed::from_f64(0.6) + Fixed::from_f64(0.1);
                    agent.memory.encode(
                        MemoryKind::Procedural,
                        tick_u64,
                        Fixed::from_f64(0.3),
                        emotional,
                        None,
                        MemoryTag::SkillMastered,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const R: i32 = 5;

    fn pos(n: usize) -> Vec<(i32, i32)> {
        (0..n).map(|i| (i as i32 * 2, 0)).collect()
    }

    fn interaction(from: u64, to: u64) -> SimEvent {
        SimEvent::InteractionOccurred {
            from: AgentId::new(from),
            to: AgentId::new(to),
            kind: mindstrata_core::event::InteractionKind::Talk,
            tick: mindstrata_core::clock::Tick::new(1),
        }
    }

    /// The gate's core contract: an agent perceives what it is part of, and
    /// what happens within the §2.4 radius of an involved agent — nothing else.
    #[test]
    fn perception_gate_is_involvement_or_radius() {
        let positions = pos(4); // (0,0) (2,0) (4,0) (6,0)
        let anchors = anchors_of(&interaction(0, 2), &positions);
        assert_eq!(anchors, EventAnchors::Two((0, 0), (4, 0)));
        // Involved at distance 0.
        assert!(anchors.perceived_from((0, 0), R));
        assert!(anchors.perceived_from((4, 0), R));
        // A bystander within the radius of one lane perceives it.
        assert!(anchors.perceived_from((6, 0), R));
        // ...and a distant one does not.
        assert!(!anchors.perceived_from((20, 0), R));
        assert!(!anchors.perceived_from((0, 40), R));
    }

    /// The radius is inclusive at exactly `r` tiles (the boundary the
    /// interaction engine's own selector uses).
    #[test]
    fn perception_radius_boundary_is_inclusive() {
        let anchors = anchors_of(&interaction(0, 0), &pos(1));
        assert!(anchors.perceived_from((R, 0), R), "distance r is in range");
        assert!(
            !anchors.perceived_from((R + 1, 0), R),
            "distance r + 1 is not"
        );
    }

    /// Lane extraction covers the shapes the sim actually emits: one lane,
    /// two lanes, three lanes (a birth), a witness list, and the single
    /// positionless institutional variant.
    #[test]
    fn lane_extraction_covers_every_event_shape() {
        use mindstrata_core::clock::Tick;
        let positions = pos(6);
        let ate = SimEvent::AgentAte {
            agent: AgentId::new(3),
            food: mindstrata_core::id::EntityId::new(0),
            tick: Tick::new(1),
        };
        assert_eq!(anchors_of(&ate, &positions), EventAnchors::One((6, 0)));

        let born = SimEvent::ChildBorn {
            child: AgentId::new(0),
            parent_a: AgentId::new(1),
            parent_b: AgentId::new(2),
            tick: Tick::new(1),
        };
        assert_eq!(
            anchors_of(&born, &positions),
            EventAnchors::Many(vec![(0, 0), (2, 0), (4, 0)])
        );

        let violation = SimEvent::NormViolated {
            agent: AgentId::new(1),
            norm_id: 0,
            witnesses: vec![AgentId::new(4)],
            tick: Tick::new(1),
        };
        assert_eq!(
            anchors_of(&violation, &positions),
            EventAnchors::Many(vec![(2, 0), (8, 0)]),
            "witnesses are lanes too"
        );

        let policy = SimEvent::InstitutionChangedPolicy {
            institution: mindstrata_core::id::EntityId::new(0),
            policy_id: 0,
            tick: Tick::new(1),
        };
        assert_eq!(anchors_of(&policy, &positions), EventAnchors::Everywhere);
        assert!(anchors_of(&policy, &positions).perceived_from((99, 99), R));
    }

    /// A lane pointing outside the living population contributes no anchor.
    /// With no surviving lane the event is perceived by nobody — the pre-i334
    /// behavior (everyone) is deliberately NOT restored, since a phantom
    /// position would anchor attention to a dead id.
    #[test]
    fn stale_lanes_anchor_nowhere() {
        let positions = pos(2);
        let dead = SimEvent::AgentDied {
            agent: AgentId::new(9),
            cause: mindstrata_core::event::DeathCause::Unknown,
            tick: mindstrata_core::clock::Tick::new(1),
        };
        assert_eq!(anchors_of(&dead, &positions), EventAnchors::Nowhere);
        assert!(!anchors_of(&dead, &positions).perceived_from((0, 0), R));

        // One survivor lane keeps the event perceivable from that lane.
        let half = anchors_of(&interaction(1, 9), &positions);
        assert_eq!(half, EventAnchors::One((2, 0)));
        assert!(half.perceived_from((2, 0), R));
        assert!(!half.perceived_from((0, 0), 0));
    }
}
