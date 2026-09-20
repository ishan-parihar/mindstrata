//! Simulation core: construction, population seeding, the fixed-tick loop, commands and accessors.

use super::{
    ActionKind, Affect, AgentId, BodyState, DiscreteEmotions, Fixed, Goal, JournalEntryKind,
    NeedState, Personality, RngStream, SimEvent, Simulation, SystemContext, KNOWLEDGE_TABOO_FLOOR,
    KNOWLEDGE_TABOO_RATE,
};
use crate::systems;

use super::snapshot_metrics::MAX_METRIC_HISTORY;
use super::MAX_EVENTS;

use rand::Rng;
impl Simulation {
    /// Compute the index into relationship_v2s for agent a → agent b.
    ///
    /// The relationship_v2s vector for agent `a` contains entries for all other
    /// agents in a fixed order. This helper guarantees the same index formula
    /// is used everywhere (populate, ritual bonding, tick loop).
    ///
    /// # Panics
    /// Panics if `a == b` or `a >= n_agents`.
    #[inline]
    pub(super) fn relationship_v2_index(a: usize, b: usize, n_agents: usize) -> usize {
        debug_assert!(a != b, "self-relationship index requested");
        debug_assert!(a < n_agents && b < n_agents, "agent index out of bounds");
        a * (n_agents - 1) + if b > a { b - 1 } else { b }
    }

    /// Per-agent position of target `b` inside agent `a`'s `relationship_v2s` vec.
    ///
    /// `relationship_v2s[a]` holds one entry per *other* agent, ordered by
    /// target index with `a` itself skipped. So target `b` sits at position
    /// `b` when `b < a`, else `b - 1`. This is *not* the same as
    /// [`Self::relationship_v2_index`], which indexes the legacy flat
    /// `self.relationships` matrix of `n·(n−1)` entries.
    #[inline]
    // Arc-D: pub(crate) — the verbatim-moved cognitive pass (systems/) also reads it.
    pub(crate) fn relationship_v2_pos(a: usize, b: usize) -> usize {
        debug_assert!(a != b, "self-relationship position requested");
        if b > a {
            b - 1
        } else {
            b
        }
    }

    /// Run the simulation for `n` ticks.
    pub fn run(&mut self, n: u64) {
        for _ in 0..n {
            self.tick();
        }
    }

    /// Execute a single simulation tick.
    pub fn tick(&mut self) {
        let tick = self.clock.advance();
        let tick_u64 = tick.as_u64();
        // §6: Compute multi-timescale phase flags once per tick.
        let phases = crate::scheduler::TickPhases::compute(tick_u64);

        // ── Scenario shocks (verbatim-extracted to sim/pass_scenario.rs) ──
        self.apply_scenario_shocks(tick_u64, tick);

        // Capture event count before systems run (before SystemContext borrows self.events)
        let pre_tick_events = self.events.len();

        // §19.5.J: Capture relationship snapshot before any systems modify relationships
        // Iteration 218: reuse pre-allocated buffer to avoid per-tick heap allocation.
        self.tick_rel_snapshot.clear();
        self.tick_rel_snapshot.extend(
            self.relationships
                .iter()
                .map(|r| (r.from, r.to, r.trust, r.affection)),
        );

        // Collect action starts to defer resource operations outside the ctx block
        // Iteration 218: reuse pre-allocated buffer.
        self.tick_action_starts.clear();

        // §8.1.5: Precompute world food/water totals before SystemContext borrows
        // self.world mutably (the motivation block reads them later, so they must
        // be captured here to avoid a borrow conflict).
        let world_food_total = self.world.total_food();
        let world_water_total = self.world.total_water();

        // Prepare system context and run systems that need mutable borrow on events/rng
        {
            let mut ctx = SystemContext {
                tick: tick_u64,
                rng: &mut self.rng,
                world: &mut self.world,
                events: &mut self.events,
            };

            // Extract agent data for systems (std::mem::take avoids cloning for mutable fields;
            // personality is read-only so we keep .clone())
            let mut bodies: Vec<BodyState> = self
                .agents
                .iter_mut()
                .map(|a| std::mem::take(&mut a.body))
                .collect();
            let mut needs: Vec<NeedState> = self
                .agents
                .iter_mut()
                .map(|a| std::mem::take(&mut a.needs))
                .collect();
            let personalities: Vec<Personality> =
                self.agents.iter().map(|a| a.personality.clone()).collect();
            let mut goals: Vec<Vec<Goal>> = self
                .agents
                .iter_mut()
                .map(|a| std::mem::take(&mut a.goals))
                .collect();
            let mut affects: Vec<Affect> = self
                .agents
                .iter_mut()
                .map(|a| std::mem::take(&mut a.affect))
                .collect();
            let mut emotions: Vec<DiscreteEmotions> = self
                .agents
                .iter_mut()
                .map(|a| std::mem::take(&mut a.emotions))
                .collect();

            // Architecture-plan-2 §8.2: Epistemic reset and trust sync.
            // §8.2: Reset per-tick processing counters and sync trust network from relationships.
            // Trust sync convergence rate: ~63% aligned after 10 ticks, ~86% after 20 ticks.
            // §8.2: Trust sync convergence rate: ~63% aligned after 10 ticks, ~86% after 20 ticks.
            let trust_sync_rate = Fixed::from_f64(0.1);
            for agent in &mut self.agents {
                agent.epistemic.reset_tick();
                // §17.2: Reset cognitive budget tracker for this tick.
                agent.agent_tier.reset_tick_budget();
            }
            // Sync TrustNetwork from existing relationship trust values.
            // O(N) pre-computation: build a per-agent trust delta map from relationships.
            // This bridges the social interaction system's trust with the epistemic layer.
            // Iteration 218: reuse pre-allocated buffer — resize if population changed
            // (births can add agents mid-tick), then clear inner Vecs and refill.
            let n_agents = self.agents.len();
            if self.tick_trust_deltas.len() < n_agents {
                self.tick_trust_deltas.resize_with(n_agents, Vec::new);
            }
            for td in &mut self.tick_trust_deltas[..n_agents] {
                td.clear();
            }
            for rel in &self.relationships {
                let from_idx = rel.from.as_u64() as usize;
                if from_idx < n_agents {
                    let target_id = rel.to.as_u64();
                    self.tick_trust_deltas[from_idx].push((target_id, rel.trust));
                }
            }
            #[allow(
                clippy::needless_range_loop,
                reason = "intentional parallel-array indexing: trust_deltas[i] + agents[i]"
            )]
            for i in 0..self.agents.len() {
                for &(target_id, rel_trust) in &self.tick_trust_deltas[i] {
                    let current_trust = self.agents[i]
                        .epistemic
                        .trust_network
                        .trust_for_agent(target_id);
                    let delta = (rel_trust - current_trust) * trust_sync_rate;
                    self.agents[i]
                        .epistemic
                        .trust_network
                        .update_agent_trust(target_id, delta);
                }
            }

            Self::tick_biology_pass(
                &mut self.agents,
                &emotions,
                &self.params,
                &self.weather,
                &mut self.provenance,
                tick_u64,
                world_food_total,
                world_water_total,
            );
            let reg_strategies = Self::tick_cognitive_pass(
                &mut self.agents,
                &mut emotions,
                &mut needs,
                &personalities,
                phases,
                tick_u64,
                world_food_total,
                world_water_total,
                &self.params,
                &mut self.tick_agent_ages,
                &mut self.relationships,
                &self.institutions,
                &mut self.provenance,
                &self.norms,
                &self.kinship_graph,
                &mut self.households,
                &self.faction_v2_registry,
            );
            // ── 1. Need decay (nonlinear pressure, §9.1) ────────────────
            // §5.1: Use configurable parameters for decay rates.
            let params = self.params;
            systems::system_need_decay_with_params(&params, &mut needs);

            // ── 2. Body state update ──────────────────────────────────
            tracing::debug!(tick = tick_u64, "systems::body_update");
            systems::system_body_update(&mut ctx, &mut bodies, &needs);

            // ── 3. Goal generation ────────────────────────────────────
            tracing::debug!(tick = tick_u64, "systems::goal_generation");
            systems::system_goal_generation(
                &mut ctx,
                &personalities,
                &needs,
                &mut goals,
                &emotions,
                &params,
            );

            Self::tick_action_pass(
                &mut ctx,
                &mut self.agents,
                tick_u64,
                &mut bodies,
                &mut emotions,
                &mut goals,
                &mut needs,
                &personalities,
                tick,
                &self.institutions,
                &self.norms,
                &self.collective_field,
                &mut self.provenance,
                &self.season,
                &mut self.tick_action_starts,
            );
            Self::tick_social_pass(
                &mut ctx,
                &mut self.agents,
                tick_u64,
                &mut emotions,
                pre_tick_events,
                tick,
                &mut self.active_courtships,
                &self.params,
                &mut self.relationships,
                &self.institutions,
                &self.norms,
            );
            Self::tick_appraisal_pass(
                &mut ctx,
                &mut self.agents,
                tick_u64,
                &mut affects,
                &mut emotions,
                &mut needs,
                &personalities,
                pre_tick_events,
                &reg_strategies,
                tick,
                &self.params,
            );
            Self::tick_decay_pass(
                &mut self.agents,
                tick_u64,
                &mut affects,
                &mut emotions,
                &mut goals,
                &mut needs,
                phases,
                &self.params,
            );
            // ── Write back (std::mem::take avoids cloning) ────────────
            //
            // Iteration 242 (root-cause fix, audit finding E3/E4): since the
            // Iteration-218 mem::take buffer refactor, the biology pass's
            // legacy-field sync (health/energy/fatigue <- derived) wrote into
            // the TAKEN placeholder that this write-back then discarded — the
            // sync has been dead code ever since. Consequences (probe-pinned,
            // seed 51): an agent whose thirst crossed 0.9 for ~300 ticks
            // decayed to body.health == 0.000 EXACTLY (clamp pin, no restoring
            // force) and stayed there permanently — blocking conception
            // (min-health < 0.3 hard gate), marriage (health product), and
            // work, while the EmbodiedState substrate reported 0.97/derived
            // 0.81. The fix converges the legacy track toward the biological
            // envelope (tau ~10 ticks) after the systems deltas of this tick:
            // acute damage (dehydration/starvation/injury) still bites within
            // the tick and repeated deprivation still collapses health, but a
            // fed, hydrated agent recovers instead of ratcheting to zero.
            for (i, agent) in self.agents.iter_mut().enumerate() {
                let derived_health = agent.embodied.derived_health();
                let h = &mut bodies[i];
                h.health += (derived_health - h.health) * Fixed::from_f64(0.1);
                h.health = h.health.clamp_01();
                // i313: the legacy facade's `injury` field was never refreshed
                // from the biological substrate either (like the dead health
                // sync above), so consumers of `BodyState.injury` read a
                // birth-time zero forever. Mirror the authoritative value.
                h.injury = agent.embodied.injury;
                agent.body = std::mem::take(&mut bodies[i]);
                agent.needs = std::mem::take(&mut needs[i]);
                agent.goals = std::mem::take(&mut goals[i]);
                agent.affect = std::mem::take(&mut affects[i]);
                agent.emotions = std::mem::take(&mut emotions[i]);
            }
        } // ctx is dropped here, releasing mutable borrows on self.events and self.rng

        // §6: Spatial movement — update agent positions AFTER ctx drops (no borrow conflicts)
        {
            let world_w = self.world.width as i32;
            let world_h = self.world.height as i32;
            for i in 0..self.agents.len() {
                let action = self.agents[i].current_action;
                match action {
                    ActionKind::Wander => {
                        let dx: i32 = self.rng.get_mut(RngStream::Behavior).random_range(-1..=1);
                        let dy: i32 = self.rng.get_mut(RngStream::Behavior).random_range(-1..=1);
                        self.agents[i].position.x =
                            (self.agents[i].position.x + dx).clamp(0, world_w - 1);
                        self.agents[i].position.y =
                            (self.agents[i].position.y + dy).clamp(0, world_h - 1);
                    }
                    ActionKind::Move { target_x, target_y } => {
                        // §6: Move one step toward target (Manhattan pathfinding)
                        let dx = (target_x - self.agents[i].position.x).clamp(-1, 1);
                        let dy = (target_y - self.agents[i].position.y).clamp(-1, 1);
                        self.agents[i].position.x =
                            (self.agents[i].position.x + dx).clamp(0, world_w - 1);
                        self.agents[i].position.y =
                            (self.agents[i].position.y + dy).clamp(0, world_h - 1);
                        // §6: Completion detection — finish action when at target
                        if self.agents[i].position.x == target_x
                            && self.agents[i].position.y == target_y
                        {
                            self.agents[i].action_progress = 0;
                        }
                    }
                    _ => {}
                }
            }
        }

        // §19.5.J: Record relationship traces for any changes from social interactions
        // This must happen after ctx is dropped so we can read self.events.
        for (i, rel) in self.relationships.iter().enumerate() {
            if i < self.tick_rel_snapshot.len() {
                let (from, to, old_trust, old_affection) = self.tick_rel_snapshot[i];
                let trust_changed = (rel.trust - old_trust).abs() > Fixed::from_f64(0.001);
                let affection_changed =
                    (rel.affection - old_affection).abs() > Fixed::from_f64(0.001);
                if trust_changed || affection_changed {
                    let cause = self.events[pre_tick_events..]
                        .iter()
                        .find_map(|ev| {
                            if let SimEvent::InteractionOccurred {
                                from: ef,
                                to: et,
                                kind,
                                ..
                            } = ev
                            {
                                if *ef == from && *et == to {
                                    Some(format!("{kind:?}"))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "social_interaction".into());
                    self.provenance
                        .record_relationship(crate::provenance::RelationshipTrace {
                            from,
                            to,
                            tick: tick_u64,
                            cause,
                            old_trust,
                            new_trust: rel.trust,
                            old_affection,
                            new_affection: rel.affection,
                            description: format!(
                                "Social interaction changed trust {}->{}, affection {}->{}",
                                old_trust.to_f64(),
                                rel.trust.to_f64(),
                                old_affection.to_f64(),
                                rel.affection.to_f64()
                            ),
                        });
                }
            }
        }

        // ── 4a. Weather (verbatim-extracted to sim/pass_weather.rs) ──
        self.weather_pass(tick_u64);
        // ── 4b. Resource operations (extracted) ──
        // Iteration 218: drain the pre-allocated buffer into a local Vec
        // so tick_resource_operations can take &mut self without borrow conflict.
        // The buffer is refilled on the next tick via clear()+push().
        let action_starts_local: Vec<(usize, ActionKind)> =
            std::mem::take(&mut self.tick_action_starts);
        self.tick_resource_operations(&action_starts_local, pre_tick_events, tick_u64, tick);
        // Restore the buffer (now empty) for the next tick's clear()+push() cycle.
        self.tick_action_starts = action_starts_local;

        // ── 9. Memory encoding (extracted) ──
        self.tick_memory_encoding(pre_tick_events, tick_u64, phases);

        // ── 16. Ecology: season advance + yearly cadence gates
        // (verbatim-extracted to sim/pass_ecology.rs) ──
        self.ecology_season_pass(tick_u64);

        // ── 17./17b./17b-2. Health, epidemic contagion, seasonal vector
        // (verbatim-extracted to sim/pass_health.rs) ──
        self.health_disease_pass(tick_u64, phases);

        // ── 17c. Migration pressure (verbatim-extracted to
        // sim/pass_ecology.rs) ──
        self.ecology_migration_pass();
        // ── 10. Resource spoilage (perishable goods degrade each tick) ──
        // Apply season spoilage modifier
        let spoilage_modifier = self.season.current.spoilage_modifier();
        // Extract resources ref before mutable borrow of sites (split borrowing)
        let resource_defs = &self.world.resources;
        for site in &mut self.world.sites {
            for stock in &mut site.inventory {
                if let Some(res_def) = resource_defs.iter().find(|r| r.id == stock.resource_id) {
                    if res_def.perishable && res_def.spoilage_rate > Fixed::ZERO {
                        // §5 (Iteration 147): weather scales spoilage — ≈1.03
                        // in normal weather, ×1.2 in a flood, ×0.8 in a drought.
                        let spoilage = stock.quantity
                            * res_def.spoilage_rate
                            * spoilage_modifier
                            * self.weather.temperature_factor();
                        stock.quantity = (stock.quantity - spoilage).max(Fixed::ZERO);
                    }
                }
            }
        }

        // ── 10b. Storage overflow — goods stored beyond a site's capacity rot/spill ──
        self.apply_storage_overflow();

        // ── 11. Norm evaluation (extracted) ──
        self.tick_norm_evaluation(pre_tick_events, tick_u64, tick);

        // ── 11b. §11.2 Gossip propagation with mutation ── (extracted)
        self.tick_gossip_and_knowledge(pre_tick_events, tick_u64, tick);

        // ── 11d. §19.5.F Childhood socialization — children learn from parents ──
        {
            self.tick_socialization_updates.clear();
            for i in 0..self.agents.len() {
                let a = &self.agents[i];
                if a.age >= Fixed::from_f64(18.0) {
                    continue;
                }
                // Pick the first available parent to learn from
                let parent_idx = a.parent_a.or(a.parent_b);
                if let Some(pi) = parent_idx {
                    if pi < self.agents.len() && !self.agents[pi].cultural.knowledge.is_empty() {
                        let pick = self
                            .rng
                            .get_mut(RngStream::Social)
                            .random_range(0..self.agents[pi].cultural.knowledge.len());
                        let knowledge_id = self.agents[pi].cultural.knowledge[pick];
                        // §5 (Iteration 148): the technology tree gates peer
                        // transmission — a learner must hold every
                        // prerequisite of the knowledge before absorbing it.
                        if !self.agents[i].cultural.knowledge.contains(&knowledge_id)
                            && self
                                .technology
                                .can_learn(&self.agents[i].cultural.knowledge, knowledge_id)
                        {
                            // §8.1.18 (Iteration 166): the child's own taboo
                            // layer dampens absorption of the parent's
                            // knowledge — a child whose culture forbids more
                            // is more resistant to novel practices (sacred
                            // boundary defense). ONE-SIDED factor, identity
                            // at zero; acceptance still needs to clear the
                            // 0.3 floor.
                            let resistance =
                                crate::psychology::cultural_cognition::taboo_knowledge_factor(
                                    self.agents[i].cultural_cognition.max_taboo_strength(),
                                    KNOWLEDGE_TABOO_RATE,
                                    KNOWLEDGE_TABOO_FLOOR,
                                );
                            let acceptance = (self.agents[i].cultural.openness
                                * Fixed::from_f64(0.6)
                                + Fixed::from_f64(0.4))
                            .clamp_01()
                                * resistance;
                            if acceptance > Fixed::from_f64(0.3) {
                                self.tick_socialization_updates.push((i, knowledge_id));
                            }
                        }
                    }
                }
            }
            for (ci, knowledge_id) in &self.tick_socialization_updates {
                self.agents[*ci].cultural.knowledge.push(*knowledge_id);
                if let Some(k) = self
                    .knowledge_store
                    .iter_mut()
                    .find(|k| k.id == *knowledge_id)
                {
                    k.holders += 1;
                }
                // §19.5.F: Record socialization in journal for observability
                self.journal.record(
                    tick_u64,
                    AgentId::new(*ci as u64),
                    JournalEntryKind::KnowledgeSocialized {
                        knowledge_id: *knowledge_id,
                    },
                );
            }
        }

        // ── 11e. §19.5.I Innovation — agents discover new knowledge through work ──
        {
            let work_agents: Vec<usize> = self
                .agents
                .iter()
                .enumerate()
                .filter(|(_, a)| matches!(a.current_action, crate::actions::ActionKind::Work))
                .map(|(i, _)| i)
                .collect();
            self.tick_innovations.clear();
            for idx in work_agents {
                // Discovery chance based on openness and conscientiousness
                let mut discovery_chance = (self.agents[idx].personality.openness
                    * Fixed::from_f64(0.3)
                    + self.agents[idx].personality.conscientiousness * Fixed::from_f64(0.2)
                    + Fixed::from_f64(0.01))
                .clamp_01();
                // §8.1.18 (Iteration 166): the taboo knowledge-resistance
                // factor dampens innovation for taboo-bound agents — a
                // culture that forbids more accepts fewer novel practices
                // (sacred boundary defense). ONE-SIDED: a taboo-free agent's
                // factor is exactly 1.0 (zero drift at the pre-Iter-165
                // snapshot shape). The RNG draw below is UNCONDITIONAL, so
                // replay determinism holds — the factor only scales the
                // chance, never the stream.
                let resistance = crate::psychology::cultural_cognition::taboo_knowledge_factor(
                    self.agents[idx].cultural_cognition.max_taboo_strength(),
                    KNOWLEDGE_TABOO_RATE,
                    KNOWLEDGE_TABOO_FLOOR,
                );
                discovery_chance = discovery_chance * resistance;
                let roll = Fixed::from_f64(self.rng.get_mut(RngStream::Social).random::<f64>());
                if roll < discovery_chance {
                    // Try to discover a knowledge item not yet known by this agent
                    for k in &self.knowledge_store {
                        // §5 (Iteration 148): work-innovation is also gated —
                        // an agent cannot innovate a node whose prerequisites
                        // they do not hold.
                        if !self.agents[idx].cultural.knowledge.contains(&k.id)
                            && k.difficulty < self.agents[idx].personality.openness
                            && self
                                .technology
                                .can_learn(&self.agents[idx].cultural.knowledge, k.id)
                        {
                            self.tick_innovations.push((idx, k.id, k.name.clone()));
                            break; // one discovery per tick per agent
                        }
                    }
                }
            }
            for (agent_idx, knowledge_id, name) in &self.tick_innovations {
                self.agents[*agent_idx]
                    .cultural
                    .knowledge
                    .push(*knowledge_id);
                if let Some(ks) = self
                    .knowledge_store
                    .iter_mut()
                    .find(|ks| ks.id == *knowledge_id)
                {
                    ks.holders += 1;
                }
                self.journal.record(
                    tick_u64,
                    AgentId::new(*agent_idx as u64),
                    JournalEntryKind::KnowledgeDiscovered {
                        knowledge_id: *knowledge_id,
                        name: name.clone(),
                    },
                );
                self.events.push(SimEvent::KnowledgeTransferred {
                    source: AgentId::new(*agent_idx as u64),
                    target: AgentId::new(*agent_idx as u64),
                    knowledge_id: *knowledge_id,
                    tick,
                });
            }
        }

        // ── 12. Institutional collective psychology (extracted) ────
        self.tick_institutional_psychology(tick_u64, phases);

        // ── 15. Faction dynamics (extracted) ────
        self.tick_faction_dynamics(tick_u64, tick);

        // ── 14. Moral panic + Revolution (extracted) ──
        self.tick_moral_panic_and_revolution(tick_u64, tick);

        // ── 15+13. Derived mental state + Belief updates (extracted) ──
        self.tick_derived_states_and_beliefs(pre_tick_events, tick_u64);

        // ── 18+19+20+21. Social cluster (extracted) ──
        self.tick_social_cluster(pre_tick_events, tick_u64, tick, phases);

        // Architecture-plan-2 §13.6: Per-tick echo chamber drift.
        // After the daily cluster rebuild (in tick_social_cluster), this accumulates
        // gradual outgroup hostility and identity fusion between rebuilds.
        // Skip on daily ticks — tick_social_cluster already rebuilds clusters and
        // recomputes polarization, so tick_update() would be redundant.
        if !phases.is_daily {
            self.echo_chamber.tick_update();
        }

        // ── 19. Marriage formation (extracted) ──
        self.tick_marriage_formation(tick_u64, tick);

        // ── 19b. Birth mechanics (extracted) — demography cadence ──
        // Births roll on the same 10-tick cadence as aging/mortality so the
        // annual birth rate is scaled by the correct elapsed-year fraction.
        if phases.is_deca {
            self.tick_birth_mechanics(tick_u64, tick);
        }

        // ── Architecture-plan-2 §12.5 (moved, Iteration-274): ritual
        // executions. Previously ran inside tick_kinship_household_daily at
        // the tick tail — AFTER system_development / polarity / collective
        // passes had consumed the same tick's catalyst window, so a ritual
        // event pushed there would never be observed. Called BEFORE the
        // development pass so `RitualPerformed` events are inside the window
        // and route as per-participant Bond catalysts. Ritual cadence is
        // monthly (4320), beyond all golden/snapshot horizons → zero blast on
        // calibrated runs.
        self.tick_ritual_executions(tick_u64, tick);

        // ── AP3 DC-1 (task 3.2): daily development pass — consumes catalysts
        // via frozen IC-1 types and pure field engine (zero-at-zero identity
        // when window empty). Hooked after birth mechanics so all demographic
        // events of the tick are visible, before kinship rebuild.
        // Window is pre_tick_events captured at tick start (read-only slice).
        // i304: parameterized entry — the difficulty-levers row-3 pathology
        // band is read from the run's `SimParameters` once per tick.
        crate::systems::development::system_development_with_params(
            &mut self.agents,
            &self.events[pre_tick_events..],
            &self.params,
        );

        // ── DC-1 STORY 9-10: polarity data-path wire. Reuse the same
        // window that the development pass just consumed, so the wiring
        // cost is one extra walk over the same slice. For each catalyst
        // observed in the window, project the claim via the pure
        // `project_catalyst` function and append to the agent's
        // `polarity_claims` list. Read-side append only — no
        // reconciliation yet (DC-2; pure `reconcile_claims` is in the
        // dev crate, but emitting reconciled transitions requires a
        // policy decision that needs a contract-freeze first). The list
        // is bounded by the per-tick event volume; no growth explosion.
        let polarity_window = &self.events[pre_tick_events..];
        crate::systems::development::system_polarity_claim_emit(
            &mut self.agents,
            polarity_window,
            tick_u64,
        );

        // ── DC-1 STORY 11 + WP-I (Iter-266): collective-field step.
        // Reuses the same pre_tick_events window that the dev + polarity
        // passes just consumed — zero extra allocation, one extra walk. The
        // village `CollectiveField` lives on `Simulation` (per-village, not
        // per-agent); pressure derivation maps catalyst buckets onto the
        // vendored collective lines by vault kind (see
        // system_collective_field_step). The step is LIVE since WP-I: press
        // integrates per line, stages advance on saturation. The field is
        // not in any golden/snapshot metric projection (FR-023), so
        // calibrated windows stay byte-identical; the behavioral consumer is
        // anchored above the probe-measured fulfillment peak (Iter-112
        // pattern) so golden runs stay inert until a crisis concentrates
        // catalysts.
        crate::systems::development::system_collective_field_step(
            &mut self.collective_field,
            polarity_window,
            self.agents.len(),
        );

        // ── Iter-296 (UM-3 core): per-polity collective holons. Same press
        // law, same window, per-capita WITHIN each polity. Additive by
        // construction: `polity_fields` is empty unless `assign_polities`
        // populated it, and the legacy whole-village field above stays the
        // sole consumer-facing field (zero blast on golden/calibrated runs —
        // identity-at-isolation pin in `sim/tests/development.rs`).
        for (field, members) in self
            .polity_fields
            .iter_mut()
            .zip(self.polity_members.iter())
        {
            crate::systems::development::system_polity_collective_field_step(
                field,
                polarity_window,
                members,
            );
        }

        // ── Iter-299 (UM-3 leg 3): cross-polity cultural diffusion via
        // trade. Trade/Trade-interaction events between members of
        // DIFFERENT polities expose the receiver's polity to the sender's
        // namespaced genesis memes (damped host-count growth — the §13.1
        // trade weight 0.3 vs gossip). Zero-at-zero: no polities or no
        // cross-polity trades → no-op; the pass is pure over the event
        // window and touches no golden-projected state. Runs BEFORE the
        // genesis/field passes so this tick's trades inform this tick's
        // culture step (event order: trades pushed by the economy pass
        // earlier in this same tick are in the window).
        crate::systems::trade_diffusion::system_trade_diffusion(
            polarity_window,
            tick,
            &self.polity_members,
            &mut self.meme_registry,
        );

        // ── Iter-297 (UM-3 leg 1): territory-anchored per-polity genesis.
        // Each polity's field generates culture citing ITS territory (site
        // ownership by nearest home-site centroid; institutions by member
        // majority) — the referent leg i296 deferred. The whole-village
        // genesis call below still runs first in tick order and wins the
        // shared-registry dedup on (bucket, class, epoch) tags, so worlds
        // with polities assigned keep every whole-village meme and gain only
        // NEW epochs that the whole-village field has not yet reached — which
        // is exactly the divergence channel: a polity whose holon advanced
        // past the village's stage commemorates it with ITS OWN places.
        // Inert without `assign_polities` (empty polity_fields → early
        // return; zero allocation beyond the empty check).

        // ── Era IV (i273): collective-line meme genesis. The first consumer
        // that GENERATES culture from collective development instead of
        // modulating an existing channel (substrate §5: replaces the
        // seed_initial_memes fixed roster over time). Inert while every line
        // sits at its founding stage — golden/snapshot horizons have all
        // lines at exactly 1.0 (probe i273, CV 0.000) — so zero blast on
        // calibrated windows; fires when a bucket's deepest line crosses
        // stage 2.0 (first observed at ~5-8K ticks in the probe).
        if !self.polity_fields.is_empty() {
            let site_positions: Vec<(i32, i32)> = self
                .world
                .sites
                .iter()
                .enumerate()
                .map(|(idx, _)| self.world.site_position(idx).unwrap_or((8, 8)))
                .collect();
            let agent_home_site: Vec<Option<usize>> =
                self.agents.iter().map(|a| a.home_site).collect();
            crate::systems::genesis::system_polity_genesis(
                &self.polity_fields,
                &self.polity_members,
                &mut self.meme_registry,
                self.params.meme_virality_scaling,
                tick,
                &self.institutions,
                &self.world.sites,
                &site_positions,
                &agent_home_site,
            );
        }
        crate::systems::genesis::system_collective_genesis(
            &self.collective_field,
            &mut self.meme_registry,
            self.params.meme_virality_scaling,
            tick,
            &self.institutions,
            &self.world.sites,
        );

        // ── Iter-286 (WP-H3 second half): norm proposals from reconciled
        // polarity clusters. Reads the claim state the polarity pass just
        // advanced; proposes registry norms when a surviving Integrated
        // synthesis holds majority consensus AND the collective Safety
        // stage has crossed the i277 band-III gate (4.0). Inert at every
        // pinned horizon by construction (max stage at 2K = 1.0 exactly,
        // probe i273) and gated again by the synthesis+quorum scan — the
        // only consumers are the monthly-ritual reinforcement loop (§12.5,
        // iterates all registry norms; first fires at 4320) and §12.3.
        let _proposed = crate::systems::development::system_norm_proposal(
            &self.agents,
            &self.collective_field,
            &mut self.norms,
            self.agents.len(),
        );

        // ── §6 + §10.6/§10.7: Kinship & Household daily update ──
        self.tick_kinship_household_daily(tick_u64, phases);

        // ── Architecture-plan-2 §8.1.5: Motivation relieve() calls ──
        // Relieve psychological needs based on current actions.
        // Amounts calibrated so multiple actions needed for full satisfaction
        // (relieve ≈ 3–5× growth_rate per tick, creating realistic behavioral pressure).
        //
        // DC-1 H6 close (CO-2026-002): the original 6/19 need coverage left
        // 13 needs saturating to deficit 1.0 (dead-channel debt — see
        // `runbooks/calibration-audit-v2.md` §H6 and
        // `docs/balance/change-orders/CO-2026-002.md`). Each action now
        // carries a **secondary relief** at 0.4–0.6× primary magnitude, so
        // the secondary need is alive when the action is chosen without
        // giving it dominant-need weight. Dominant-need gate semantics
        // preserved: action selection is unchanged; only the relief map
        // widened. Magnitudes chosen so a starving agent still feels
        // hunger as dominant (Hunger 0.008/tick vs Health 0.002/tick
        // secondary means Health won't dominate Hunger via secondary
        // alone).
        for i in 0..self.agents.len() {
            let action = self.agents[i].current_action;
            let m = &mut self.agents[i].motivation;
            match action {
                ActionKind::Socialize => {
                    // Primary: belonging/attachment (the action's purpose).
                    m.belonging.relieve(Fixed::from_f64(0.01));
                    m.attachment.relieve(Fixed::from_f64(0.008));
                    // Secondary: novelty (new face), care (empathic listening).
                    m.novelty.relieve(Fixed::from_f64(0.003));
                    m.care.relieve(Fixed::from_f64(0.002));
                }
                ActionKind::Worship => {
                    // Primary: meaning/certainty.
                    m.meaning.relieve(Fixed::from_f64(0.008));
                    m.certainty.relieve(Fixed::from_f64(0.005));
                    // Secondary: justice (moral order), recognition (communal).
                    m.justice.relieve(Fixed::from_f64(0.002));
                    m.recognition.relieve(Fixed::from_f64(0.002));
                }
                ActionKind::Trade => {
                    // Primary: competence/recognition.
                    m.competence.relieve(Fixed::from_f64(0.008));
                    m.recognition.relieve(Fixed::from_f64(0.005));
                    // Secondary: novelty (new goods), justice (fair exchange).
                    m.novelty.relieve(Fixed::from_f64(0.003));
                    m.justice.relieve(Fixed::from_f64(0.002));
                }
                ActionKind::Work => {
                    // Primary: competence/recognition.
                    m.competence.relieve(Fixed::from_f64(0.006));
                    m.recognition.relieve(Fixed::from_f64(0.006));
                    // Secondary: autonomy (self-direction), esteem (mastery).
                    m.autonomy.relieve(Fixed::from_f64(0.002));
                    m.esteem.relieve(Fixed::from_f64(0.002));
                }
                ActionKind::Rest => {
                    // Primary: sleep.
                    m.sleep.relieve(Fixed::from_f64(0.02));
                    // Secondary: health (recovery), safety (sheltered).
                    m.health.relieve(Fixed::from_f64(0.0015));
                    m.safety.relieve(Fixed::from_f64(0.002));
                }
                _ => {}
            }
        }

        // ── 19. §6.5: Record metric history for observability ──
        // Every 10 ticks, snapshot key metrics for CSV export and analysis.
        // Cap at MAX_METRIC_HISTORY entries to prevent unbounded growth.
        if phases.is_deca {
            let snapshot = self.metrics_snapshot();
            self.metric_history.push(snapshot);
            if self.metric_history.len() > MAX_METRIC_HISTORY {
                // ponytail: drain_to_ring — keep capacity, shift only the
                // tail. This avoids `remove(0)` O(n) shift that regressed
                // N=48 10K by ~30%.
                let drop_n = self.metric_history.len() - MAX_METRIC_HISTORY;
                self.metric_history.drain(..drop_n);
            }
        }
        // ── End-of-tick: bump the cumulative event counter. The counter is
        // read by `event_count()` and the metrics snapshot's `event_count`
        // field, so it tracks the full history independently of the rolling
        // buffer (trimmed just below — i327).
        let events_pushed_this_tick = self.events.len().saturating_sub(pre_tick_events);
        self.total_event_count = self
            .total_event_count
            .saturating_add(events_pushed_this_tick as u64);

        // ── i327: bound the rolling event buffer (amortized bulk drop) ──
        // The buffer used to grow without bound (probe i326: 1 049 162 events
        // ≈ 56 MiB at N=48 @20K, ~O(N) per tick — a memory wall, not a time
        // one). The recorded fix was `VecDeque<SimEvent>` for an O(1) front
        // pop; measured against the real churn that is a much larger refactor
        // than the win (≈40 `.push` sites need `push_back`, ~15 slice-read
        // sites need reworking, and `VecDeque` has no range indexing) — it is
        // recorded as the upgrade path, not attempted piecemeal.
        //
        // Bounded here instead by an AMORTIZED bulk drop: let the buffer reach
        // 2×MAX_EVENTS, then one `drain` back to MAX. The memmove is O(MAX),
        // but fires only once per (MAX / events-per-tick) ticks — at the
        // measured ~52 events/tick (N=48) that is one ~14 MiB move every
        // ~5 000 ticks, i.e. amortized ≈O(1) per event, with peak memory
        // 2×MAX ≈ 28 MiB instead of unbounded. Trimmed HERE, at tick end,
        // after every pass read its `pre_tick_events..` window, so no in-tick
        // index is disturbed; the next tick re-reads `self.events.len()` as
        // its own base. `total_event_count` carries the cumulative reading, so
        // the public `event_count()` and the golden `metric_hash` are unmoved.
        //
        // ponytail: ceiling = the periodic bulk memmove (a sub-ms tick-jitter
        // stall every ~5K ticks); upgrade path = the recorded
        // `VecDeque<SimEvent>` ring if a pass ever needs jitter-free ticks.
        Self::trim_event_buffer(&mut self.events);
    }

    /// i327: the amortized bulk drop that bounds the rolling event buffer.
    /// Extracted so the bound has a runnable check without a 100K-tick run.
    /// No-op until the buffer exceeds `2×`[`MAX_EVENTS`]; then one drain back
    /// to `MAX_EVENTS` (see the call site for the cost/benefit rationale).
    pub(super) fn trim_event_buffer(events: &mut Vec<SimEvent>) {
        if events.len() > 2 * MAX_EVENTS {
            let drop_n = events.len() - MAX_EVENTS;
            events.drain(..drop_n);
        }
    }

    // ── Tick subsystem methods ──────────────────────────────────────
    // Extracted from tick() for readability and maintainability.
    // Each method handles one coherent subsystem tick.
}
