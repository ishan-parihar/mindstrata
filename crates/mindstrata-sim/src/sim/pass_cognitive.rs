//! Tick pass 0b: cognitive state update.

use super::life_chapter_crossed;
use super::{
    AgentBundle, AgentId, Fixed, MemoryKind, MemoryTag, Simulation,
    ATTACHMENT_LOW_SUPPORT_THRESHOLD,
};

use super::snapshot_metrics::self_esteem_support;

impl Simulation {
    pub(super) fn tick_cognitive_pass(
        agents: &mut [AgentBundle],
        emotions: &mut [crate::person::DiscreteEmotions],
        needs: &mut [crate::person::NeedState],
        personalities: &[crate::person::Personality],
        phases: crate::scheduler::TickPhases,
        tick_u64: u64,
        world_food_total: Fixed,
        world_water_total: Fixed,
        params: &crate::parameters::SimParameters,
        tick_agent_ages: &mut Vec<Fixed>,
        relationships: &mut [crate::person::Relationship],
        institutions: &[crate::institutions::Institution],
        provenance: &mut crate::provenance::CausalProvenance,
        norms: &crate::norms::NormRegistry,
        kinship_graph: &crate::social::kinship::KinshipGraph,
        households: &mut [crate::social::household::Household],
        faction_v2_registry: &crate::social::faction_v2::FactionV2Registry,
    ) -> Vec<crate::psychology::emotion_regulation::RegulationStrategy> {
        // §8.1.4: Collect regulation strategies from step 0b, applied after appraisal in step 6.
        // Pre-allocated Vec avoids repeated allocation; capacity matches agent count.
        let num_agents = agents.len();
        let mut reg_strategies: Vec<crate::psychology::emotion_regulation::RegulationStrategy> =
            Vec::with_capacity(num_agents);

        // ── 0b. Cognitive state update (§22.1) ────────────────────
        // §22.1: Stress reduces planning horizon, increases heuristic bias.
        // This must happen before action selection to affect decision-making.
        // Iteration 108 (§10.1.2): kin support buffers the stress input —
        // the social field's kin_count scales (fear + anger) down before
        // the EMA. Zero-at-zero: no kin → factor 1.0 → byte-identical
        // (kinship forms only via births, earliest at ~4,170 ticks, so
        // every ≤2000-tick calibrated window is untouched). Note the
        // channel: this lowers cognitive.stress → planning_horizon /
        // heuristic_bias (§22.1); the metrics' avg_stress (mean of
        // fear + anger) is NOT changed by the buffer.
        let kin_rate = Fixed::from_f64(crate::social::relational_field::KIN_STRESS_RATE);
        let kin_cap = Fixed::from_f64(crate::social::relational_field::KIN_STRESS_CAP);
        let narr_rate = Fixed::from_f64(crate::psychology::NARRATIVE_STRESS_RESILIENCE_RATE);
        for i in 0..agents.len() {
            // §8.1.3 (Iteration 181): the narrative scripts' first decision
            // consumer — the plan's "narrative feeds ... resilience". A
            // redemptive story (redemption/heroism/coherence up,
            // contamination/victimhood/shame down) buffers the perceived
            // stress input; a contaminated story amplifies it. One-sided
            // identity-at-birth (the Iter-99/127 pattern): factor is ~1.0
            // at the birth envelope (coherence drift alone moves it
            // ~±1–4%, probe-pinned — never decision-granularity), so
            // default-envelope agents are effectively untouched — and only
            // focal agents ever move their scripts (runs_narrative), so
            // the tiered blast is contained to focal agents. Same channel
            // as the kin buffer: lowers cognitive.stress →
            // planning_horizon / heuristic_bias (§22.1); the metrics'
            // avg_stress (mean of fear + anger) is NOT changed.
            let stress = (emotions[i].fear + emotions[i].anger)
                * crate::social::relational_field::RelationalFields::kin_stress_factor(
                    agents[i].relational_fields.kin_count,
                    kin_rate,
                    kin_cap,
                )
                * agents[i].narrative.stress_resilience_factor(
                    narr_rate,
                    agents[i].embodied.endocrine.stress.chronic_load,
                );
            let need_fatigue = needs[i].fatigue.max(needs[i].hunger).max(needs[i].thirst);
            agents[i].cognitive.update(stress, need_fatigue);

            // Architecture-plan-2 §8.1.12: Executive function update.
            // Degrades effective capacity, inhibition, flexibility under stress.
            let pain_level = agents[i].embodied.nervous.pain.effective_pain();
            let trauma = agents[i].embodied.nervous.trauma_load;
            // §17: Only focal agents run full executive function update
            // §17.2: Budget check — executive function counts as an appraisal.
            if agents[i].agent_tier.tier.runs_full_psychology()
                && agents[i].agent_tier.budget_tracker.can_appraise()
            {
                agents[i]
                    .cognitive_runtime
                    .update(stress, need_fatigue, pain_level, trauma);
                let _ = agents[i].agent_tier.budget_tracker.consume_appraisal();
            }

            // Architecture-plan-2 §8.1.14: Attachment system daily decay.
            // Separation distress decays slowly; security stabilizes.
            // §17: Only agents with relationship updates run attachment decay
            if phases.is_daily && agents[i].agent_tier.tier.runs_relationship_updates() {
                // §8.1.14: A partnered agent separated from their attachment
                // figure accumulates separation distress. Previously distress
                // only ever decayed (and reunions barely happen), so the
                // attachment → emotion coupling never activated.
                // Iteration 173: the separation accumulation and decay now
                // read the tunable parameters (attachment_separation_rate,
                // attachment_decay_rate) instead of hardcoded 0.02/0.95,
                // making the §8.1.14 dynamics calibratable. The defaults
                // preserve the probe-verified envelope: a calibration
                // sweep showed rates above 0.03 make the coupling dominate
                // (inverting taboo suppression, the kin-support buffer and
                // scenario deltas — violating the Phase-5 acceptance), so
                // the strength is deliberately left at the calibrated
                // level while the knobs become live.
                let is_partnered = agents[i].partner.is_some();
                let att = &mut agents[i].attachment;
                if is_partnered {
                    let closeness = Fixed::from_f64(0.8);
                    att.on_separation(closeness, params.attachment_separation_rate);
                }
                // Decay factor is clamped non-negative so an out-of-range
                // decay_rate (>= 1.0) degrades to instant drain, never to
                // a sign flip on the distress accumulator.
                let decay_factor = (Fixed::ONE - params.attachment_decay_rate).max(Fixed::ZERO);
                att.separation_distress = (att.separation_distress * decay_factor).max(Fixed::ZERO);
            }

            // Architecture-plan-2 §8.1.5: Motivation update.
            // Bridge biological needs from existing NeedState, then grow and update.
            // §8.1.5 full formula: the three missing context factors are
            // derived per tick from existing state — emotional amplification
            // from affect, cultural legitimacy from the agent's legitimacy
            // field (mean-zero 0.5 anchor), situational affordance from
            // world food/water scarcity. Write-only observational: the
            // richer pressure feeds only `dominant_need`, which has no
            // behavioral consumer yet, so calibrated runs carry zero drift.
            // Scarcity hoisted to the agent-loop scope (Iteration 71): the
            // same deterministic values feed both the motivation context
            // (below) and the daily prospection scenario generation.
            let expected_food = crate::sim::EXPECTED_GRAIN_PER_AGENT as i64 * agents.len() as i64;
            let expected_water = crate::sim::EXPECTED_WATER_PER_AGENT as i64 * agents.len() as i64;
            // Use the world totals precomputed before the ctx block.
            let food_scarcity =
                (Fixed::ONE - world_food_total / Fixed::from_int(expected_food.max(1))).clamp_01();
            let water_scarcity = (Fixed::ONE
                - world_water_total / Fixed::from_int(expected_water.max(1)))
            .clamp_01();
            {
                // NOTE (Iteration 124): the LIVE LOCAL emotions vec —
                // the tick's step-0b parallel-array machinery took each
                // agent's emotions out via `std::mem::take` (sim.rs:2337)
                // and only writes back at step-8 (sim.rs:4105), so
                // `agents[i].emotions.*` is all-ZERO here. The
                // pre-iteration code read the EMPTIED field, so
                // `set_context` received all zeros and the §8.1.5
                // emotional pressure amplifications (`1 + fear×0.4`
                // Safety/Certainty, `1 + anger×0.4` Justice,
                // `1 + joy×0.3` Romance, `1 − sadness×0.3` Meaning)
                // were silently DEAD — the dominant-need machinery has
                // operated with an emotionless context since the
                // parallel-array refactor (audit-found in Iteration
                // 123). SEMANTIC: at this site the local vec holds the
                // value TAKEN from the agent at tick start — the
                // previous tick's accumulated writeback, not this
                // tick's fresh appraise deltas — "yesterday's emotion
                // drives today's motivation", the same carried-over
                // treatment the emotion machinery itself gives the
                // vec. Probe-pinned blast radius: fear mean 0.83–1.0
                // and joy mean 0.65–0.89 across calibrated windows
                // (anger ~0, sadness 0), so this restores a genuinely
                // live emotional context. The net Safety amplification
                // measures ~1.03× (not 1.35×): the ×0.8 legitimacy
                // dampener at live legitimacy≈0 partially cancels the
                // fear factor — which is exactly why the full gate is
                // green with ZERO regeneration (live-but-observationally-
                // inert in every test window; Safety stays dominant and
                // the Iter-96 exclusive urgency boost scales its
                // channels uniformly). Deterministic, no RNG.
                let fear = emotions[i].fear;
                let anger = emotions[i].anger;
                let joy = emotions[i].joy;
                let sadness = emotions[i].sadness;
                let legitimacy = agents[i].legitimacy_field.overall;
                agents[i].motivation.set_context(
                    fear,
                    anger,
                    joy,
                    sadness,
                    legitimacy,
                    food_scarcity,
                    water_scarcity,
                );
            }
            // Iteration 247 (Arc B — interoception activation): the mind
            // receives the body's signals through the felt-signal filters,
            // not raw deficits. Baseline-corrected channels are exact
            // identities at default configuration (zero golden blast) and
            // amplify/distort only via personality-driven interoceptive
            // deviation — an anxious agent feels the same hunger as more
            // dire, feeding stronger need pressure into action selection.
            agents[i].motivation.hunger.deficit = agents[i]
                .interoception
                .corrected_felt_hunger(needs[i].hunger);
            agents[i].motivation.thirst.deficit = agents[i]
                .interoception
                .corrected_felt_thirst(needs[i].thirst);
            // §7.3.1 (Iteration 187 — the circadian behavioral consumer):
            // the body's sleep-pressure clock now drives the sleep drive
            // instead of running underneath it. `should_sleep()` fires at
            // night with pressure > 0.4; `sleep_deprived()` at debt > 0.3.
            // When either fires, the fatigue need is floored so the Rest
            // action's utility (keyed on `needs.fatigue` via the
            // DecisionContext) and the §8.1.5 sleep motive both respond —
            // the agent actually lies down when its circadian clock says
            // so. Deterministic, no RNG; identity-at-zero (a rested,
            // daytime agent keeps its raw fatigue).
            let circadian = &agents[i].embodied.circadian;
            let sleep_deficit = if circadian.sleep_deprived() {
                Fixed::from_f64(0.8)
            } else if circadian.should_sleep() {
                Fixed::from_f64(0.6)
            } else {
                needs[i].fatigue
            };
            needs[i].fatigue = needs[i].fatigue.max(sleep_deficit);
            agents[i].motivation.sleep.deficit = sleep_deficit;
            // §8.1.5 (P2/P3 re-audit — safety-need ratchet): the legacy
            // `needs.safety` accumulator has ZERO consumers anywhere in
            // the sim (no action relieves it; SeekSafety has no aligned
            // action), so it grew +0.0003/tick from birth and saturated
            // at 1.0 for every agent by ~10K ticks (probe: 0.702 at 2K →
            // 1.000 at 10K, 12/12 identical, no spread) — monopolizing
            // `dominant_need` (Safety:7 at 10K) and hiding the real
            // thirst/fatigue drives. The §8.1.5 safety deficit now
            // tracks the agent's LIVE felt danger (fear + anger) instead
            // of the unbounded clock: a calm agent is safe (deficit low),
            // a threatened agent is unsafe (deficit high). Mean-zero
            // anchor: the fear/anger blend replaces the accumulator, so
            // the axis differentiates and dominant_need reflects actual
            // drives. Deterministic, no RNG.
            let safety_deficit = (emotions[i].fear + emotions[i].anger) * Fixed::from_f64(0.5);
            agents[i].motivation.safety.deficit = safety_deficit.clamp_01();
            agents[i].motivation.update();

            // Architecture-plan-2 §8.1.4: Emotion regulation
            // Compute social support from relationships (average trust of top-3 closest agents)
            // Uses a fixed-size stack array to avoid heap allocation per agent per tick.
            let social_support = {
                let mut top3: [Fixed; 3] = [Fixed::ZERO; 3];
                let mut count: usize = 0;
                for r in relationships.iter() {
                    if r.from == AgentId::new(i as u64) {
                        // Insert into sorted top-3 (descending)
                        let pos = top3.iter().position(|t| r.trust > *t);
                        if let Some(p) = pos {
                            // Shift smaller values right, insert at p
                            for j in (p + 1..3).rev() {
                                top3[j] = top3[j - 1];
                            }
                            top3[p] = r.trust;
                        }
                        count += 1;
                    }
                }
                if count > 0 {
                    let n = count.min(3);
                    let sum: Fixed = top3[..n].iter().fold(Fixed::ZERO, |acc, t| acc + *t);
                    sum / Fixed::from_int(n as i64)
                } else {
                    Fixed::from_f64(0.3) // baseline when no relationships
                }
            };
            // §17: Only focal agents run full emotion regulation
            if agents[i].agent_tier.tier.runs_full_psychology() {
                let regulation_strategy = agents[i].emotion_regulation.select_strategy(
                    stress,
                    social_support,
                    personalities[i].extraversion,
                );
                // §8.1.4: Store regulation strategy for post-appraisal application.
                // The actual apply_strategy() call happens after appraisal (step 6)
                // so the skill-scaled boost uses the fresh, appraisal-derived affect values.
                // §16.2: Record attachment-proximate provenance when social support is low.
                if social_support < ATTACHMENT_LOW_SUPPORT_THRESHOLD {
                    provenance.record_system(crate::provenance::SystemTrace {
                        agent: AgentId::new(i as u64),
                        tick: tick_u64,
                        category: crate::provenance::ProvenanceCategory::Attachment,
                        description: format!(
                            "Low social support ({}) activating attachment-driven regulation",
                            social_support.to_f64()
                        ),
                        magnitude: (Fixed::ONE - social_support).clamp_01(),
                        cause: "emotion_regulation_selection".into(),
                    });
                }
                reg_strategies.push(regulation_strategy);
                // §8.1: A healthy self-model (high self-esteem earned from
                // a redemptive narrative) sustains regulation capacity.
                let self_support = self_esteem_support(agents[i].self_model.self_esteem);
                agents[i].emotion_regulation.update_capacity(
                    stress,
                    need_fatigue,
                    social_support + self_support,
                );
            } else {
                // §17: Background/secondary agents use default regulation strategy
                reg_strategies.push(Default::default());
            }

            // §17: Only focal agents run moral cognition update
            if agents[i].agent_tier.tier.runs_full_psychology() {
                // Architecture-plan-2 §8.1.10: Moral cognition update
                // §8.1.10: Compute witnessed violations from this tick's negative interactions.
                // High anger = agent perceived a norm violation; threat/insult events contribute.
                let witnessed_violations = (emotions[i].anger * Fixed::from_f64(0.6)
                    + emotions[i].fear * Fixed::from_f64(0.2))
                .clamp_01();
                // §8.1.7: Sacred values amplify witnessed violations — the
                // same observed violation triggers proportionally more moral
                // outrage the more sacred the agent's values are. Zero-at-zero
                // anchor: no violations -> no amplification (typical calm
                // agents are unaffected; only moral events diverge).
                let witnessed_violations = agents[i]
                    .sacred_values
                    .amplify_witnessed_violations(witnessed_violations);
                // §11.1: Witnessed violations erode perceived legitimacy —
                // the council that fails to prevent wrongdoing loses its
                // perceived rightfulness. Zero-at-zero anchor: no violations
                // -> no erosion.
                // §8.1.4 (Iter-130): awe — overwhelming, unexpected
                // significance — shields this erosion: an awe-struck
                // population experiences institutional failings as smaller
                // than they are ("reverence forgives"). One-sided: identity
                // at zero awe. The producer is U-shaped on |future_implication|
                // (= 1 − threat − need_pressure), so awe fires on significance
                // in EITHER direction (wonder or dread) — the zero baseline
                // blast (golden + 14 snapshots byte-identical) is consumer-
                // side: erosion floors legitimacy at 0 in every calibrated
                // window, the 0.5-anchored grievance/theft consumers never
                // activate, and the continuous motivation-context shift stays
                // below decision granularity.
                let awe_reverence = crate::appraisal::awe_reverence_factor(
                    emotions[i].awe,
                    crate::appraisal::AWE_REVERENCE_RATE,
                    crate::appraisal::AWE_REVERENCE_FLOOR,
                );
                agents[i]
                    .legitimacy_field
                    .scandal_damage(witnessed_violations * awe_reverence);
                let personal_violations = (emotions[i].guilt * Fixed::from_f64(0.4)
                    + emotions[i].shame * Fixed::from_f64(0.3))
                .clamp_01();
                let moral_achievements = (emotions[i].pride * Fixed::from_f64(0.3)
                    + emotions[i].trust * Fixed::from_f64(0.2))
                .clamp_01();
                // Iteration 195: the §17 developmental psychology
                // state was 100% write-only (computed every centum
                // for Focal agents, consumed by NOTHING — the whole
                // subsystem was a dead ratchet). `moral_development`
                // (Kohlberg stage) is the natural modulator of the
                // internalized shame/pride response in the moral
                // cognition update that runs on the SAME Focal gate,
                // so the developmental pipeline finally feeds
                // behavior. Zero-blast: moral emotions are
                // probe-confirmed 0.000 in calm/famine/pestilence
                // windows, and this consumer is Focal-gated like the
                // producer.
                let moral_development = agents[i].developmental.moral_development;
                agents[i].moral_cognition.update_moral_emotions(
                    witnessed_violations,
                    personal_violations,
                    moral_achievements,
                    moral_development,
                );
            }

            // §17: Only focal agents run prospection (mental simulation of futures)
            // §17.2: Also check cognitive budget — skip if prospection budget exhausted.
            if agents[i].agent_tier.tier.runs_prospection()
                && agents[i].agent_tier.budget_tracker.can_prospect()
            {
                let agent_fear = emotions[i].fear;
                let agent_ambition = agents[i].personality.ambition;
                let agent_trauma = agents[i].embodied.nervous.trauma_load;
                let agent_depression = agents[i].psychopathology.depression_risk;

                // Architecture-plan-2 §8.1.16 (Iteration 71): the plan's
                // core mechanic — "agents simulate possible futures" —
                // was dead: generate_scenario/evaluate_scenarios had zero
                // production callers, so the scenario set stayed empty.
                // One scenario per daily pass, derived deterministically
                // from live state (no RNG) and bounded by capacity; then
                // evaluate_scenarios() grounds hope/dread in the set.
                // Ordering matters (reviewer-mandated): generation runs
                // FIRST — it uses the standing emotion biases from the
                // previous tick's update — then evaluation regrounds
                // hope/dread, and only then does update() apply today's
                // emotions, so the plan's "depression reduces hope" drain
                // operates on the scenario-grounded hope instead of being
                // clobbered by the overwrite. Observational — prospection
                // is absent from the AgentBundle projection and has no
                // behavioral consumer, so calibrated runs stay
                // byte-identical.
                if phases.is_daily {
                    let inputs = crate::psychology::imagination::ScenarioInputs {
                        hunger: needs[i].hunger,
                        thirst: needs[i].thirst,
                        safety: needs[i].safety,
                        fear: agent_fear,
                        anger: emotions[i].anger,
                        ambition: agent_ambition,
                        food_scarcity,
                        water_scarcity,
                        has_partner: agents[i].partner.is_some(),
                        total_attraction: agents[i].attraction.total_attraction(),
                        // §8.1.12 (Iteration 80): the D5 ambition gate —
                        // long-term planning requires intact executive
                        // function (stress/fatigue/pain/trauma degrade it).
                        // Deliberate hard gate (not a soft scale): matches
                        // CognitiveRuntime::can_plan_long_term's documented
                        // categorical contract; the alternative (scaling D5's
                        // vividness by EF depth) is unit-testable but less
                        // legible. Note: cognitive_runtime.update runs under
                        // the can_appraise() budget gate while this read sits
                        // in the can_prospect() block — with mismatched
                        // budget exhaustion the read reflects last-tick EF.
                        // Harmless at 12 focal agents (budgets never exhaust).
                        can_plan_long_term: agents[i].cognitive_runtime.can_plan_long_term(),
                    };
                    agents[i]
                        .prospection
                        .generate_daily_scenario(tick_u64, inputs);
                    agents[i].prospection.evaluate_scenarios();
                }
                agents[i].prospection.update(
                    agent_fear,
                    agent_ambition,
                    agent_trauma,
                    agent_depression,
                );
                // §8.1.12 (Iteration 80): mirror the executive function's
                // effective planning depth into planning_confidence — the
                // plan's "confidence in ability to plan and execute"
                // should track the agent's actual degraded capacity, not
                // just fear/ambition. Deliberate 0.5/0.5 simplification:
                // the emotion term (update()) captures motivational
                // confidence while EF depth captures cognitive capacity;
                // equal weight is the documented default, revisable via
                // calibration. Both operands are provably in [0,1] (clamped
                // by their producers), so the blend needs no re-clamp.
                // Observational; deterministic; same budget-gate note as
                // the D5 gate above. Runs inside the can_prospect() block,
                // consistent with update() — both skip together on
                // budget exhaustion.
                let ef_depth = agents[i].cognitive_runtime.effective_planning_depth();
                // §8.1.16 (Iteration 187 → 211 graduated): the binary
                // `is_impaired()` gate (health < 0.5 → halve) is replaced
                // by `cognitive_modifier()` which returns `overall_health`
                // directly — a smooth gradient instead of a cliff. Fully
                // healthy agents (health ≈ 1.0) keep near-full planning
                // depth; severely impaired agents (health → 0.0) get
                // near-zero. Deterministic, no RNG.
                let ef_depth = ef_depth * agents[i].psychopathology.cognitive_modifier();
                agents[i].prospection.planning_confidence =
                    (agents[i].prospection.planning_confidence + ef_depth) * Fixed::from_f64(0.5);
                let _ = agents[i].agent_tier.budget_tracker.consume_prospection();
            }

            // §17: Compute negative_events once (shared by narrative + psychopathology)
            let negative_events = if emotions[i].fear + emotions[i].sadness > Fixed::from_f64(0.3) {
                (emotions[i].fear + emotions[i].sadness) * Fixed::from_f64(0.1)
            } else {
                Fixed::ZERO
            };

            // §17: Only focal agents run narrative identity updates
            // §17.2: Budget check — narrative interpretation counts as an appraisal.
            if agents[i].agent_tier.tier.runs_narrative()
                && agents[i].agent_tier.budget_tracker.can_appraise()
            {
                // §8.1.3 (Iteration 181): the per-event ratchets are
                // write-only — without decay every script saturated at
                // ~1.0 within ~10K ticks (probe-pinned), flattening the
                // identity. Pull each script back toward its birth-narrative
                // value every tick so movement is event-driven but bounded
                // (the Iter-179 pull pattern). Runs before interpretation
                // so today's events act on today's re-anchored envelope.
                agents[i].narrative.decay_scripts(Fixed::from_f64(
                    crate::psychology::NARRATIVE_SCRIPT_DECAY_RATE,
                ));
                let positive_event_magnitude = if emotions[i].joy > Fixed::from_f64(0.3) {
                    emotions[i].joy * Fixed::from_f64(0.1)
                } else {
                    Fixed::ZERO
                };
                let events_before = agents[i].narrative.events_integrated;
                if negative_events > Fixed::ZERO {
                    agents[i].narrative.interpret_negative_event(
                        negative_events,
                        social_support,
                        emotions[i].anger > Fixed::from_f64(0.3),
                    );
                }
                if positive_event_magnitude > Fixed::ZERO {
                    agents[i]
                        .narrative
                        .interpret_positive_event(positive_event_magnitude, social_support);
                }
                // Iteration 186: feed the agent's temperament into the
                // theme mapping so a thriving calm village does not
                // collapse every agent onto Mission (the script-balance-
                // only mapping saturated positive scripts uniformly).
                // Agency = ambition/extraversion/conscientiousness
                // blend; neuroticism discounts the positive balance.
                let (neuro, agency) = {
                    let p = &agents[i].personality;
                    let agency = p.ambition * Fixed::from_f64(0.4)
                        + p.extraversion * Fixed::from_f64(0.3)
                        + p.conscientiousness * Fixed::from_f64(0.3);
                    (p.neuroticism, agency)
                };
                agents[i]
                    .narrative
                    .update_theme(crate::psychology::narrative::ThemeTemperament {
                        neuroticism: neuro,
                        agency,
                    });
                // §8.1.3: Episodic memory — every chapter boundary of
                // integrated life events closes a chapter worth remembering
                // (sparse by design; salience scales with the current
                // event's emotional magnitude, so crises recall louder).
                // The block is Focal-only (runs_narrative), a subset of
                // the memory-encoding tiers, so no extra tier gate needed.
                let events_after = agents[i].narrative.events_integrated;
                if life_chapter_crossed(events_before, events_after)
                    && agents[i].agent_tier.budget_tracker.can_memory_op()
                {
                    let _ = agents[i].agent_tier.budget_tracker.consume_memory_op();
                    let event_magnitude = negative_events.max(positive_event_magnitude);
                    // Ceiling is 0.6 by construction: negative_events ≤ 0.2
                    // ((fear+sadness)×0.1) → 1.5×0.2+0.3; positive ≤ 0.1 → 0.45.
                    let salience = event_magnitude * Fixed::from_f64(1.5) + Fixed::from_f64(0.3);
                    let emotional =
                        agents[i].affect.arousal * Fixed::from_f64(0.6) + Fixed::from_f64(0.1);
                    agents[i].memory.encode(
                        MemoryKind::Episodic,
                        tick_u64,
                        salience,
                        emotional,
                        None,
                        MemoryTag::LifeEvent,
                    );
                }
                // §8.1: The self-model tracks the same life events, and
                // self-esteem mean-reverts toward the narrative balance.
                agents[i].self_model.update_narrative(
                    negative_events,
                    positive_event_magnitude,
                    social_support,
                );
                agents[i].self_model.reconcile_self_esteem();
                // §8.1.7 (P3-2): coherence and security now have
                // producers. Coherence feeds the yearly plasticity gate
                // (identity_integration → plastic_update); security is
                // observational (no behavioral consumer — see
                // psych_probe.rs for diagnostics).
                agents[i].self_model.update_coherence();
                agents[i].self_model.update_security(
                    negative_events,
                    positive_event_magnitude,
                    social_support,
                );
                // §8.1.17: The same life events reshape the meaning-making
                // frames themselves — suffering crystallizes punitive frames
                // and erodes just-world optimism, success does the reverse.
                agents[i]
                    .narrative_frames
                    .daily_update(negative_events, positive_event_magnitude);
                let _ = agents[i].agent_tier.budget_tracker.consume_appraisal();
            }

            // §17: Only focal agents run developmental psychology
            if phases.is_centum && agents[i].agent_tier.tier.runs_full_psychology() {
                let agent_age = agents[i].age;
                // Iteration 196: education quality is the agent's own
                // learning aptitude (how well education benefits THIS
                // agent — the same capacity `effective_learning` weights
                // at 0.3 in the apprenticeship pass) instead of the
                // hardcoded 0.5 placeholder, so identity formation
                // genuinely scales with the education pipeline.
                let education_quality = agents[i].education.learning_aptitude;
                agents[i]
                    .developmental
                    .tick_update(agent_age, social_support, education_quality);
            }

            // §17: Only focal agents run psychopathology update
            if agents[i].agent_tier.tier.runs_full_psychology() {
                let chronic_stress = agents[i].embodied.endocrine.stress.chronic_load;
                let trauma_load = agents[i].embodied.nervous.trauma_load;
                let chronic_pain = agents[i].embodied.nervous.pain.chronic;
                let sleep_debt = agents[i].embodied.circadian.sleep_debt;
                let depression_vuln = agents[i]
                    .embodied
                    .genome
                    .trait_predispositions
                    .depression_vulnerability;
                let addiction_risk = agents[i]
                    .embodied
                    .genome
                    .trait_predispositions
                    .addiction_risk;
                let social_isolation = (Fixed::ONE - social_support).max(Fixed::ZERO);
                agents[i].psychopathology.tick_update(
                    chronic_stress,
                    trauma_load,
                    social_isolation,
                    // Iteration 193: humiliation_recent was hardcoded
                    // ZERO — the paranoia channel's humiliation leg
                    // (×0.25) and resentment's humiliation leg (×0.2)
                    // were structurally dead. The live emotion
                    // `emotions[i].humiliation` (status_threat ×
                    // identity_relevance in appraisal, already consumed
                    // by the escalation fold) is the natural producer:
                    // public status loss is exactly the humiliation the
                    // risk model expects. Zero-at-zero in calm/famine/
                    // pestilence windows (probe: 0.000 — status_threat
                    // needs a threat event), so calibrated runs stay
                    // byte-identical.
                    emotions[i].humiliation,
                    negative_events,
                    // Iteration 193: moral_injury was hardcoded ZERO —
                    // resentment's moral-injury leg (×0.25) was
                    // structurally dead (only chronic_stress×0.1
                    // remained). The live emotion
                    // `emotions[i].moral_outrage` (sacredness_violation ×
                    // goal_relevance in appraisal — the agent's emotional
                    // response to witnessing its sacred order violated) is
                    // the natural producer: the morally-outraged agent IS
                    // morally injured. Zero-at-zero in calibrated windows
                    // (probe: 0.000 — sacredness_violation needs an
                    // unfairness event), so calibrated runs stay
                    // byte-identical.
                    emotions[i].moral_outrage,
                    chronic_pain,
                    sleep_debt,
                    depression_vuln,
                    addiction_risk,
                    social_support,
                    // Iteration 192: despair is the emotional hopelessness
                    // of an unmet need — the documented
                    // "sadness -> despair -> depression-from-deprivation"
                    // chain's last link, which previously had no consumer
                    // into depression_risk. Zero-at-zero: calm agents
                    // (despair 0.000) contribute nothing.
                    emotions[i].despair,
                );
            }

            // §17: Only focal agents run cultural cognition + decision policy
            if phases.is_daily && agents[i].agent_tier.tier.runs_full_psychology() {
                let positive_exposure = social_support * Fixed::from_f64(0.5);
                let negative_exposure = stress * Fixed::from_f64(0.3);
                agents[i]
                    .cultural_cognition
                    .tick_update(positive_exposure, negative_exposure);
                agents[i].decision_policy.daily_update();
            }

            // §17: Skill/habit update runs for focal + secondary (heuristic cognition)
            if agents[i].agent_tier.tier.runs_heuristic_cognition() {
                agents[i]
                    .psych_skills
                    .update_automaticity(stress, need_fatigue);
                if phases.is_centum {
                    agents[i].psych_skills.decay_habits(tick_u64);
                }
            }
        }

        // §17: Verify reg_strategies Vec alignment after per-agent loop
        debug_assert_eq!(
            reg_strategies.len(),
            agents.len(),
            "reg_strategies must be aligned with agents after tier-gated psychology loop"
        );

        // Compute avg_wealth once before status updates (O(N) instead of O(N²))
        let avg_wealth: Fixed = if agents.is_empty() {
            Fixed::ZERO
        } else {
            agents
                .iter()
                .map(|a| a.wealth.coin)
                .fold(Fixed::ZERO, |acc, c| acc + c)
                / Fixed::from_int(agents.len() as i64)
        }; // Precomputed ages for the kinship max-relatedness scan below —
           // the outer loop iterates `&mut agents`, so a nested
           // `agents[j].age` read would be a borrow conflict; the ages
           // are pure reads collected once (deterministic, no RNG).
           // Iteration 218: reuse pre-allocated buffer.
        tick_agent_ages.clear();
        tick_agent_ages.extend(agents.iter().map(|a| a.age));
        for (i, agent) in agents.iter_mut().enumerate() {
            // Architecture-plan-2 §11.1: Status dimensions update.
            // Decay shame and honor gradually; update wealth_rank from relative wealth.
            agent.status_v2.decay();
            // Sync authority from existing institutional role_status
            agent.status_v2.authority = agent.status.role_status;
            // §11.1 (AP2): Derive institutional_rank from the agent's highest
            // institutional role authority (deterministic scan of the
            // institutions registry — no RNG). Zero when the agent holds no
            // institutional role. Only the new field is written, so the
            // golden baseline stays byte-identical.
            let agent_id = AgentId::new(i as u64);
            let mut best_role_authority = Fixed::ZERO;
            for institution in institutions {
                for role in &institution.roles {
                    if role.holder == Some(agent_id) && role.authority > best_role_authority {
                        best_role_authority = role.authority;
                    }
                }
            }
            agent.status_v2.institutional_rank = best_role_authority;
            if avg_wealth > Fixed::ZERO {
                agent.status_v2.wealth_rank =
                    (agent.wealth.coin / avg_wealth * Fixed::from_f64(0.5)).clamp_01();
            }
            // Update moral_reputation from moral pride and gratitude (weighted average)
            agent.status_v2.moral_reputation = (agent.moral_cognition.moral_emotions.pride
                * Fixed::from_f64(0.5)
                + agent.moral_cognition.moral_emotions.gratitude * Fixed::from_f64(0.5))
            .clamp_01();
            // Update prestige from relationship count (social connections)
            let num_connections = agent.relationship_v2s.len() as i64;
            if num_connections > 0 {
                let avg_quality: Fixed = agent
                    .relationship_v2s
                    .iter()
                    .map(crate::social::relationship_v2::RelationshipV2::quality)
                    .fold(Fixed::ZERO, |acc, q| acc + q)
                    / Fixed::from_int(num_connections);
                agent.status_v2.network_centrality = avg_quality;
            }
            // §10.4 (Iteration 77): status_attraction was a declared
            // AttractionModel field with zero production writers — the
            // last of the three attraction factors (with familiarity and
            // reciprocity) that feed the §8.1.16 D4 courtship gate. The
            // agent's freshly-refreshed composite standing feeds their
            // attraction profile: role holders and the wealthy carry the
            // standing that makes them a better match. The mirror is
            // intentionally exact (status_attraction == effective_status,
            // pinned by the integration test): a future relative/scaled
            // derivation is a conscious re-design, not an incidental one.
            // Note: status_attraction tracks whatever effective_status()
            // captures — since Iteration 106 the §11.1 institutional_rank
            // field IS weighted into the composite (× 0.1), so role
            // holders' status_attraction reflects their institutional
            // power, flowing through automatically. Deterministic, no RNG;
            // status_attraction itself feeds only the D4 scenario gate,
            // but the composite also feeds §10.9 patronage formation
            // (behavioral), so role-holder status now genuinely matters.
            let eff_status = agent.status_v2.effective_status();
            agent.attraction.update_status_attraction(eff_status);
            // §7.2.2 (S2-2-2 fix): the endocrine dominance axis now
            // tracks the agent's standing — the plan's "status victory
            // raises dominance" channel. Previously write-once:
            // `dominance.update` had zero call sites (the axis sat frozen
            // at birth values). Mean-reverts toward `effective_status`
            // (the §11.1 composite, refreshed just above): elites hold
            // higher dominance, low-status agents drain toward their
            // standing — rises with status gains, falls with losses, and
            // cannot saturate (the target is the agent's real status).
            // Daily cadence: status fields refresh on the daily
            // decay/sync pattern in this same loop (taboo_penalty,
            // kinship_penalty below use the same `phases.is_daily`
            // fold). Deterministic, no RNG; dominance feeds
            // `dominance_aggression_modifier` into `should_escalate`
            // (Iteration 187), so the level is decision-live.
            if phases.is_daily {
                agent
                    .embodied
                    .endocrine
                    .dominance
                    .update(eff_status, params.endocrine_dominance_response);
            }
            // §10.4 (Iteration 78): the two per-agent aversion channels.
            // social_cost mirrors the agent's criminal notoriety from the
            // live norm-enforcement crime records (offenders are socially
            // costly partners — the channel only rises for agents who
            // actually commit hostile acts, so it differentiates without
            // depressing the D4-reachability that Iteration 77 wired).
            // moral_disgust mirrors the disgust emotion — the §8.1.4
            // appraisal's purity-violation aversion, 0 in peaceful runs by
            // design (the same situational class as the Iter-65 jealousy
            // channel), firing when the agent appraises a moral violation.
            // Both deterministic, no RNG, observational (only the D4
            // scenario gate reads total_attraction).
            let notoriety = norms
                .crime_record(AgentId::new(i as u64))
                .map_or(Fixed::ZERO, |r| r.notoriety);
            agent.attraction.update_social_cost(notoriety);
            // NOTE (Iteration 123): both channels below read the LIVE
            // local `emotions[i]` vec, not `agent.emotions` —
            // the tick takes each agent's emotions out via
            // `std::mem::take` at the step-0b parallel-array setup, so
            // the AGENT field is all-zero until the step-8 writeback
            // (sim.rs:4105). The pre-Iteration-123 `update_moral_disgust`
            // line read the emptied field and was therefore a LATENT
            // dead channel (moral_disgust pinned at 0 in production
            // despite live disgust producers); it is fixed here to read
            // the local vec — behaviorally identical until a purity
            // violation actually fires the emotion (which calm worlds
            // never do → zero-blast).
            let disgust = emotions[i].disgust;
            agent.attraction.update_moral_disgust(disgust);
            // §8.1.4 (Iteration 123): the envy aversion channel — the
            // coveting emotion (`status_threat × incongruent` — wanting
            // what the higher-status other has) poisons courtship
            // interest: the envious mind is consumed with status
            // comparison, so romantic salience drops. Mirrors the
            // moral_disgust channel exactly (same update-*-cost pattern,
            // same 0.2 weight tier, and — per the note above — the
            // same live local-vec read). ONE-SIDED: identity at zero —
            // envy is never produced in calibrated windows
            // (probe-pinned: every seed-42 golden agent at envy == 0,
            // Iter-122 Leg C), so `envy_cost` stays exactly 0 and
            // `total_attraction()` is bit-identical (provably
            // zero-blast).
            let envy = emotions[i].envy;
            agent.attraction.update_envy_cost(envy);
            // §8.1.18/§10.4 (Iteration 165): the taboo-restraint channel —
            // the strongest internalized taboo in the agent's cultural
            // cognition (`CulturalCognition::max_taboo_strength`) becomes
            // a standing cost on courtship pursuit (the RWR row-#11
            // `taboo_penalty` consumer, and the §8.1.18 taboo layer's
            // first production read). Traditional agents hold stronger
            // seeded taboos (populate scales strength by traditionalism),
            // so the channel differentiates: the most taboo-bound agents
            // court less readily. Deterministic, no RNG; identity at zero
            // for taboo-free agents (pre-Iter-165 snapshot restores).
            // Daily cadence: taboo strength only changes on the daily
            // cultural-cognition tick_update decay, so a daily fold is
            // the honest cadence (mirrors the kinship_penalty fold's
            // daily pattern below).
            if phases.is_daily {
                let max_taboo = agent.cultural_cognition.max_taboo_strength();
                agent.attraction.update_taboo_penalty(max_taboo);
            }
            // §10.4 (Iteration 79): the last unwired attraction channel —
            // kinship_penalty, the soft taboo against courting within the
            // local pool. Derived as the max transitive genetic
            // relatedness to any other adult (the plan's §10.6 BFS model:
            // parent/sibling 0.5, grandparent 0.25, first cousin 0.125),
            // which the direct-edge 0.25 hard eligibility gate does not
            // catch. Reading: a pool-inbreeding signal — the 0.5 tier
            // (parent/sibling, hard-gated out of courtship anyway) raises
            // the cost of ALL matches in a kin-dense village, while the
            // actionable taboo tier (cousins 0.125–0.24, which pass the
            // hard gate) is exactly what the soft channel suppresses.
            // 0 for founding villages, rising as families form (probe:
            // seeds 42/43/44 → 0, seed 51 → 0.50). Daily cadence: kinship
            // edges only change on birth/marriage, so the O(N²) BFS runs
            // once a day (the patronage block below uses the same
            // is_duodeca pattern for its O(N²) work). Deterministic, no
            // RNG, observational (only the D4 scenario reads it).
            if phases.is_daily {
                let adult_age = Fixed::from_f64(16.0);
                let mut max_relatedness = Fixed::ZERO;
                for (j, age) in tick_agent_ages.iter().enumerate() {
                    if j == i || *age < adult_age {
                        continue;
                    }
                    let coeff = kinship_graph.transitive_coefficient(i, j);
                    if coeff > max_relatedness {
                        max_relatedness = coeff;
                    }
                }
                agent.attraction.update_kinship_penalty(max_relatedness);
            }

            // Architecture-plan-2 §17.3: Relationship caching.
            // Only decay relationships that have been recently active (dirty)
            // or are due for the daily full update. Dormant relationships
            // receive no per-tick decay — they only decay during the daily pass.
            for rv2 in &mut agent.relationship_v2s {
                if rv2.is_active_this_tick(tick_u64) {
                    rv2.decay(1);
                    // After daily boundary decay, clear dirty so dormant
                    // relationships return to sleep on the next tick.
                    if tick_u64 > 0 && tick_u64.is_multiple_of(144) {
                        rv2.clear_dirty(tick_u64);
                    }
                }
            }

            // §5.1: Legacy relationship mean reversion — trust drifts toward a
            // neutral baseline so it cannot pin at 1.0 for every pair. Without
            // this, positive interactions ratcheted all trust to 1.0 once agents
            // were healthy/wealthy, erasing the differentiation the witness
            // system produces. `relationship_dormant_decay` was previously dead
            // parameter. Mean reversion: trust -= (trust - 0.5) * decay per day.
            if tick_u64.is_multiple_of(144) && params.relationship_dormant_decay > Fixed::ZERO {
                for rel in relationships.iter_mut() {
                    if rel.from == AgentId::new(i as u64) {
                        let drift =
                            (rel.trust - Fixed::from_f64(0.5)) * params.relationship_dormant_decay;
                        rel.trust = (rel.trust - drift).clamp_01();
                    }
                }
            }
        }

        // §11.2: Recompute relational power balance on the daily boundary.
        // Fills `RelationshipV2.power_balance`, declared since Iter 1 but
        // never written — the dead-field class this closes. Purely a
        // function of existing relationship + status state (no RNG), so it
        // cannot perturb calibrated runs; the field is not consumed
        // downstream or by agent_summaries().
        if tick_u64 > 0 && tick_u64.is_multiple_of(144) {
            let n = agents.len();
            let statuses: Vec<Fixed> = (0..n)
                .map(|i| agents[i].status_v2.effective_status())
                .collect();
            for i in 0..n {
                let rel_count = agents[i].relationship_v2s.len();
                for pos in 0..rel_count {
                    let (dependence_a_on_b, to_idx) = {
                        let rv2 = &agents[i].relationship_v2s[pos];
                        (rv2.dependence, rv2.to.as_u64() as usize)
                    };
                    if to_idx >= n || to_idx == i {
                        // Defensive: population never creates self-relationships
                        // (the i == j pair is skipped), but guard anyway —
                        // relationship_v2_pos debug_asserts a != b and would
                        // silently misbehave in release.
                        continue;
                    }
                    // B's reverse view toward A: relationship_v2_pos(b, a).
                    let rev_pos = Self::relationship_v2_pos(to_idx, i);
                    let (
                        commitment_b_to_a,
                        attachment_b_to_a,
                        fear_b_of_a,
                        obligation_b_to_a,
                        moral_debt_b_to_a,
                    ) = {
                        let rev = &agents[to_idx].relationship_v2s[rev_pos];
                        (
                            rev.commitment,
                            rev.attachment_security,
                            rev.fear,
                            rev.obligation,
                            rev.moral_debt,
                        )
                    };
                    let status_advantage = statuses[i] - statuses[to_idx];
                    agents[i].relationship_v2s[pos].update_power_balance(
                        dependence_a_on_b,
                        commitment_b_to_a,
                        attachment_b_to_a,
                        status_advantage,
                        fear_b_of_a,
                        obligation_b_to_a,
                        moral_debt_b_to_a,
                    );
                }
            }
        }

        // §10.2 (AP2): Relationship identity metadata + structural links,
        // refreshed on the daily boundary. Fills the plan-mandated fields
        // the struct was missing: public/private labels (§10.3 taxonomy),
        // role expectation (authority/kin branch), kinship coefficient,
        // and the shared household/faction/institution links. Purely a
        // deterministic function of existing state (no RNG) that only
        // writes the new §10.2 fields, so calibrated runs stay
        // byte-identical.
        if tick_u64 > 0 && tick_u64.is_multiple_of(144) {
            let n = agents.len();
            // First-match memberships per agent (deterministic).
            let agent_household: Vec<Option<usize>> = (0..n)
                .map(|i| {
                    households
                        .iter()
                        .find(|h| h.members.contains(&i))
                        .map(|h| h.id)
                })
                .collect();
            let agent_faction: Vec<Option<usize>> = (0..n)
                .map(|i| {
                    faction_v2_registry
                        .factions
                        .iter()
                        .position(|f| f.active && f.members.contains(&i))
                })
                .collect();
            let agent_institution: Vec<Option<u64>> = (0..n)
                .map(|i| {
                    let id = AgentId::new(i as u64);
                    institutions
                        .iter()
                        .find(|inst| inst.members.contains(&id))
                        .map(|inst| inst.id)
                })
                .collect();
            for i in 0..n {
                let rel_count = agents[i].relationship_v2s.len();
                for pos in 0..rel_count {
                    let to_idx = agents[i].relationship_v2s[pos].to.as_u64() as usize;
                    if to_idx >= n || to_idx == i {
                        continue;
                    }
                    let rv2 = &mut agents[i].relationship_v2s[pos];
                    rv2.update_identity_metadata();
                    // §10.2: Reconciliation — when a betrayal has been
                    // repaired by recovered trust (no reconciliation since
                    // the last betrayal, trust back above 0.5), record one
                    // Apology event. Deterministic, writes only new §10.2
                    // fields (magnitude ∝ recovered trust), so calibrated
                    // runs stay byte-identical. This closes the plan's
                    // betrayal→reconciliation arc in live runs.
                    {
                        let last_betrayal_tick = rv2.betrayal_history.last().map_or(0, |e| e.tick);
                        let last_reconciliation_tick =
                            rv2.reconciliation_history.last().map_or(0, |e| e.tick);
                        if last_betrayal_tick > last_reconciliation_tick
                            && rv2.trust > Fixed::from_f64(0.5)
                        {
                            rv2.record_reconciliation(
                                tick_u64,
                                crate::social::relationship_v2::ReconciliationKind::Apology,
                                rv2.trust,
                            );
                        }
                    }
                    rv2.household_link = if agent_household[i].is_some()
                        && agent_household[i] == agent_household[to_idx]
                    {
                        agent_household[i]
                    } else {
                        None
                    };
                    rv2.faction_link = if agent_faction[i].is_some()
                        && agent_faction[i] == agent_faction[to_idx]
                    {
                        agent_faction[i]
                    } else {
                        None
                    };
                    rv2.institutional_link = if agent_institution[i].is_some()
                        && agent_institution[i] == agent_institution[to_idx]
                    {
                        agent_institution[i]
                    } else {
                        None
                    };
                }
            }
        }
        reg_strategies
    }
}
