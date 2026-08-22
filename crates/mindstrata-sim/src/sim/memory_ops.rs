//! Trauma/founding memory recording and memory encoding.

use super::skill_milestone_crossed;
use super::{
    AgentId, Fixed, MemoryKind, MemoryTag, PerceptKind, RngStream, SimEvent, Simulation,
    SKILL_GAIN_PER_TICK,
};

impl Simulation {
    /// §13.5: Record a famine (scarcity) trauma in the village's collective
    /// memory when hunger or thirst runs critically high. Episode-guarded:
    /// a scarcity trauma is not re-recorded while a recent one exists, so
    /// one crisis yields one memory. Deterministic (reads agent needs).
    pub(super) fn record_famine_memory(&mut self, tick: u64) {
        use crate::culture::SharedMemoryKind;
        // Episode guard — skip while a *live* scarcity trauma exists. Seeded
        // origin memories have event_tick == 0 and never block.
        let recent = self.collective_memory_registry.get(0).is_some_and(|cm| {
            cm.memories.iter().any(|m| {
                m.kind == SharedMemoryKind::Trauma
                    && m.event_tick > 0
                    && tick.saturating_sub(m.event_tick) < 43200 // ~300 days
            })
        });
        if recent {
            return;
        }
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
        let (desc, charge) = if avg_thirst > Fixed::from_f64(0.6) {
            ("Thirst gripped the village when the wells ran low", 0.8)
        } else if avg_hunger > Fixed::from_f64(0.6) {
            ("The village went hungry when the harvest failed", 0.8)
        } else {
            return;
        };
        self.collective_memory_registry.get_or_create(0).add_memory(
            desc.into(),
            SharedMemoryKind::Trauma,
            tick,
            Fixed::from_f64(charge),
        );
    }

    /// §13.5: Record a faction's founding myth — group 1 + faction id keeps
    /// faction memory separate from the village's (group 0).
    pub(super) fn record_faction_founding_memory(&mut self, faction_id: usize, tick: u64) {
        use crate::culture::SharedMemoryKind;
        self.collective_memory_registry
            .get_or_create(1 + faction_id)
            .add_memory(
                "The faction was born of shared grievance against the council".into(),
                SharedMemoryKind::Founding,
                tick,
                Fixed::from_f64(0.7),
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
            for ev in &self.events[pre_tick_events..] {
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
            let agent_id = AgentId::new(i as u64);
            let (positive_rels, total_rels) = self
                .relationships
                .iter()
                .filter(|r| r.from == agent_id)
                .fold((0u32, 0u32), |(pos, total), r| {
                    (
                        pos + if r.trust > Fixed::from_f64(0.6) { 1 } else { 0 },
                        total + 1,
                    )
                });
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

            // §4.2: Skill improvement — agents improve skills through repeated practice.
            // Small increments per tick; skills cap at 1.0.
            let skill_gain = SKILL_GAIN_PER_TICK;
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
                        prof >= Fixed::from_f64(0.1) && prof - skill_gain < Fixed::from_f64(0.1);
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
                let before = *skill;
                *skill = (before + skill_gain).clamp_01();
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
