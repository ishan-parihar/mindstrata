//! Simulation core: construction, population seeding, the fixed-tick loop, commands and accessors.

use super::{
    ActionKind, Affect, AgentId, BodyState, DeathCause, DiscreteEmotions, Fixed, Goal,
    JournalEntryKind, NeedState, Personality, RngStream, ShockKind, SimEvent, Simulation,
    SystemContext, COLD_SEASON_RATE, COLD_SEASON_TEMP, FAMINE_GRAIN_DRAIN, FAMINE_WINDOW_TICKS,
    FEVER_ESCALATION_RATE, GRAIN_RESOURCE_ID, KNOWLEDGE_TABOO_FLOOR, KNOWLEDGE_TABOO_RATE,
    WATER_RESOURCE_ID,
};
use crate::ecology;
use crate::health;
use crate::systems;

use super::snapshot_metrics::MAX_METRIC_HISTORY;

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
    pub(super) fn relationship_v2_pos(a: usize, b: usize) -> usize {
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

        // Check for scenario shocks
        if let Some(ref scenario) = self.scenario {
            if let Some(shock) = scenario.shock_at(tick_u64) {
                tracing::warn!(
                    shock = ?shock.kind,
                    magnitude = shock.magnitude.to_f64(),
                    tick = tick_u64,
                    "Scenario shock applied"
                );
                match shock.kind {
                    ShockKind::Drought => {
                        // A drought dries the water supply (WATER_RESOURCE_ID = 1).
                        // It previously drained grain (resource_id == 0) — a drought
                        // that destroys food instead of water.
                        //
                        // The drain is proportional to the stocked quantity, not a
                        // fixed magnitude × 10: the village well starts with 200
                        // water, so an absolute drain of 3.0 vs 7.0 was a 1.5% vs
                        // 3.5% difference — both drowned out by agent consumption,
                        // making riverford (0.3) and drought (0.7) indistinguishable.
                        // A proportional drain leaves 70% vs 30% of remaining water,
                        // so the shock magnitude now genuinely differentiates.
                        //
                        // NB: no `.clamp_01()` here — a well holding 200 water is
                        // legitimately above 1.0, and clamping to [0,1] would erase
                        // the magnitude difference (60 and 140 both → 1.0). We only
                        // floor at zero to keep quantities non-negative.
                        for site in &mut self.world.sites {
                            for stock in &mut site.inventory {
                                if stock.resource_id == WATER_RESOURCE_ID {
                                    stock.quantity = (stock.quantity
                                        * (Fixed::ONE - shock.magnitude))
                                        .max(Fixed::ZERO);
                                }
                            }
                        }
                        // Iteration 228: drought-pressure suppresses aquifer
                        // recharge for 3000 ticks (~1 year). Without this, the
                        // shock drains 70% of water at tick 500, but aquifer
                        // recharge during Normal weather restores it by tick 3000
                        // — making drought and vanilla baselines identical
                        // (both 38,226 events at 3K).
                        // Iteration 232: set drought window to suppress
                        // aquifer recharge for 3000 ticks.
                        self.drought_until = tick_u64 + 3000;
                    }
                    ShockKind::Famine => {
                        // A famine destroys the grain supply (GRAIN_RESOURCE_ID).
                        // Same proportional-drain semantics as Drought: a fixed
                        // absolute drain would be drowned out by the farm's large
                        // stock, so the magnitude must scale with the stocked
                        // quantity to genuinely differentiate scenarios.
                        for site in &mut self.world.sites {
                            for stock in &mut site.inventory {
                                if stock.resource_id == GRAIN_RESOURCE_ID {
                                    stock.quantity = (stock.quantity
                                        * (Fixed::ONE - shock.magnitude))
                                        .max(Fixed::ZERO);
                                }
                            }
                        }
                        // §8.1.4 (P3-6): a famine is a CROP FAILURE, not a
                        // one-shot store drain. Without production suppression,
                        // Work refilled the destroyed grain within ~1.5K ticks
                        // and body hunger never rose (probe: needs.hunger
                        // 0.006–0.041 in every scenario, famine ≈ calm — the
                        // goal-incongruence branch was unreachable). Open a
                        // production-suppression window (FAMINE_WINDOW_TICKS)
                        // during which Work grain output is scaled by
                        // `1 − magnitude` (0.3 for the standard 0.7 famine):
                        // stored grain genuinely depletes, Eat fails, and
                        // hunger accumulates toward the 0.5 goal-congruence
                        // gate.
                        self.famine_until = tick_u64 + FAMINE_WINDOW_TICKS;
                        // Near-total crop failure: production collapses to
                        // 0.05 × magnitude (3.5% for the standard 0.7 famine).
                        // `1 − magnitude` = 0.3 was NOT enough — the surviving
                        // trickle still yielded ~2 full portions/day and the
                        // village scraped by (probe: top hunger 0.488, never
                        // crossing the 0.5 gate). A famine starves.
                        self.famine_production_factor =
                            shock.magnitude * Fixed::from_f64(0.05).max(Fixed::from_f64(0.01));
                    }
                    ShockKind::Pestilence => {
                        // A pestilence strikes in two composed waves, both routed
                        // through EXISTING machinery rather than fabricated:
                        //  1. IMMEDIATE MORTALITY — a seeded roll with
                        //     p = magnitude × (1 − health × 0.5) is deadly at
                        //     every health level yet deadlier for the weak
                        //     (health 1.0 → 0.35 vs health 0.5 → 0.525 at
                        //     mag 0.7). Deaths flow
                        //     through handle_agent_death — the same path the §31
                        //     demography pass uses — so inheritance, journaling,
                        //     event recording, social cleanup, and in-place
                        //     newborn replacement all stay consistent. This is
                        //     safe here: the shock block runs before the
                        //     SystemContext borrow (self is fully mutable), and
                        //     the dead are replaced IN PLACE so the
                        //     AgentId == index invariant survives.
                        //  2. SURVIVING EPIDEMIC — survivors (and the newborns
                        //     who replaced the dead) carry a virulent Epidemic
                        //     that block 17b spreads by proximity contagion and
                        //     block 17 drains health from over its 800-tick course.
                        // Copy the magnitude first: handle_agent_death borrows
                        // self mutably, so `shock` (borrowing self.scenario) must
                        // not be live across those calls.
                        let magnitude = shock.magnitude;
                        let n = self.agents.len();
                        let mut deaths: Vec<usize> = Vec::new();
                        for i in 0..n {
                            let health = self.agents[i].body.health;
                            let p = magnitude * (Fixed::ONE - health * Fixed::from_f64(0.5));
                            let roll = Fixed::from_f64(
                                self.rng.get_mut(RngStream::Behavior).random::<f64>(),
                            );
                            if roll < p {
                                deaths.push(i);
                            }
                        }
                        tracing::warn!(
                            deaths = deaths.len(),
                            "Pestilence: immediate mortality wave"
                        );
                        // Note: the newborns that replace the dead run the
                        // remainder of this tick's systems (the demography path
                        // kills at tick end instead) — an accepted divergence,
                        // invisible to calibrated runs since no existing
                        // scenario carries a Pestilence shock.
                        for &idx in &deaths {
                            self.handle_agent_death(
                                idx,
                                &deaths,
                                tick_u64,
                                tick,
                                DeathCause::Disease,
                            );
                        }
                        // Seed the virulent epidemic: every agent rolls an
                        // independent infection check (p = magnitude), so the
                        // selection is index-fair — a plague spares no one.
                        let mut infected = 0usize;
                        for i in 0..n {
                            let roll = Fixed::from_f64(
                                self.rng.get_mut(RngStream::Behavior).random::<f64>(),
                            );
                            if roll < magnitude {
                                let already = self.agent_diseases[i]
                                    .iter()
                                    .any(|d| d.kind == health::DiseaseKind::Epidemic);
                                if !already {
                                    self.agent_diseases[i].push(health::ActiveDisease {
                                        kind: health::DiseaseKind::Epidemic,
                                        ticks_infected: 0,
                                        severity_modifier: Fixed::ONE, // virulent strain
                                    });
                                    infected += 1;
                                }
                            }
                        }
                        tracing::debug!(
                            infected = infected,
                            "Pestilence: epidemic seeded into the population"
                        );
                    }
                    ShockKind::Festival => {
                        for agent in &mut self.agents {
                            agent.emotions.joy = (agent.emotions.joy
                                + shock.magnitude * Fixed::from_f64(0.3))
                            .clamp_01();
                        }
                    }
                }
            }
        }

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

        // ── 4a. Weather: continuous temperature/rainfall + emergent regimes ──
        // The Ecology RNG stream has no other consumer (virgin), so weather's
        // two draws per tick cannot shift any existing stream's position —
        // replays stay byte-identical. Weather is computed before production
        // (growth factor) and spoilage (temperature factor) read it. NB: the
        // season baseline used here rolled at the PREVIOUS tick's block 16
        // (season advances after weather), so weather lags a season boundary
        // by one tick — a deliberate, negligible ordering artifact.
        let weather_event = self
            .weather
            .advance(self.season.current, self.rng.get_mut(RngStream::Ecology));
        if let Some(ev) = weather_event {
            tracing::info!(
                event = ?ev,
                rainfall = self.weather.rainfall.to_f64(),
                temperature = self.weather.temperature.to_f64(),
                "Weather regime change"
            );
        }
        // Regime-gated well water: an emergent drought drains each well's
        // water every tick; an emergent flood recharges it. Normal weather
        // touches nothing — wells only move through consumption and these
        // regimes, so calibrated windows are unaffected by this block.
        match self.weather.regime {
            ecology::WeatherRegime::Drought => {
                let drain = self.weather.config.drought_water_drain;
                for site in &mut self.world.sites {
                    for stock in &mut site.inventory {
                        if stock.resource_id == WATER_RESOURCE_ID && stock.quantity > Fixed::ZERO {
                            stock.quantity =
                                (stock.quantity * (Fixed::ONE - drain)).max(Fixed::ZERO);
                        }
                    }
                }
            }
            ecology::WeatherRegime::Flood => {
                // Iteration 238: suppress flood recharge during the drought
                // shock window — a drought desiccates the aquifer, so even
                // flood weather cannot restore water until the drought
                // pressure window expires. Without this, the weather tracker
                // can enter Flood mode during a drought scenario, recharging
                // water past the shock drain (water goes from 120 back to
                // 643 in 500 ticks, defeating the drought test).
                if tick_u64 >= self.drought_until {
                    let recharge = self.weather.config.flood_water_recharge;
                    for site in &mut self.world.sites {
                        // Cap recharged wells at the site's storage capacity so a
                        // sustained flood cannot balloon water past the §19.5.E
                        // storage contract (the cap only binds during floods,
                        // which never fire in calibrated windows).
                        let cap = site.storage_capacity;
                        for stock in &mut site.inventory {
                            if stock.resource_id == WATER_RESOURCE_ID
                                && stock.quantity > Fixed::ZERO
                            {
                                stock.quantity =
                                    (stock.quantity * (Fixed::ONE + recharge)).min(cap);
                            }
                        }
                    }
                }
            }
            ecology::WeatherRegime::Normal => {
                // Iteration 222: natural aquifer recharge — wells slowly
                // refill from groundwater during normal weather. Without
                // this, wells drain to 0 within ~2K ticks (12 agents ×
                // daily consumption) and never recover (only floods
                // recharged them). The recharge rate is slow (0.05%
                // per tick = ~50% recovery over 1K ticks), so the well
                // acts as a buffer rather than an infinite source.
                // Deterministic, no RNG.
                // Iteration 228: drought-pressure suppresses recharge so
                // a Drought shock's water drain persists for the pressure
                // window (3000 ticks). Without this, aquifer recharge
                // restores water by tick 3000, making drought and vanilla
                // baselines identical.
                // Iteration 232: suppress recharge during drought window
                if tick_u64 >= self.drought_until {
                    let recharge_rate = Fixed::from_f64(0.0005);
                    for site in &mut self.world.sites {
                        // Iteration 237: only Wells receive aquifer recharge.
                        // Non-Well sites (Farms, generic) must not gain water
                        // from groundwater — the isolation-chamber test (and
                        // realism) demands that water at a non-Well site stays
                        // exactly stable absent explicit production.
                        if site.kind != crate::world::SiteKind::Well {
                            continue;
                        }
                        let cap = site.storage_capacity;
                        for stock in &mut site.inventory {
                            if stock.resource_id == WATER_RESOURCE_ID && stock.quantity < cap {
                                let deficit = cap - stock.quantity;
                                let recharge = deficit * recharge_rate;
                                stock.quantity = (stock.quantity + recharge).min(cap);
                            }
                        }
                    }
                }
            }
        }
        // §8.1.4 (P3-6): famine grain drain — while a Famine shock's
        // production-suppression window is open, stored grain additionally
        // decays per tick (a famine consumes the granary: eating outpaces the
        // failed harvest). Mirrors the drought regime's per-tick water drain
        // above. With only the one-shot 70% store drain + production
        // suppression, grain plateaued at ~2.1 (production at 0.3× still
        // matched consumption) and body hunger never rose; the drain drives
        // stored grain to zero mid-window so Eat fails, hunger accumulates
        // toward the 0.5 goal-congruence gate, and the goal-incongruence
        // branch (sadness → despair → depression-from-deprivation) becomes
        // reachable. Outside an open window the factor is 1.0 — calibrated
        // windows (riverford/calm/pestilence/golden) are untouched.
        if tick_u64 < self.famine_until {
            let drain = FAMINE_GRAIN_DRAIN;
            for site in &mut self.world.sites {
                for stock in &mut site.inventory {
                    if stock.resource_id == GRAIN_RESOURCE_ID && stock.quantity > Fixed::ZERO {
                        stock.quantity = (stock.quantity * (Fixed::ONE - drain)).max(Fixed::ZERO);
                    }
                }
            }
        }

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

        // ── 16. Ecology: season advance and fertility dynamics ──
        let new_season = self.season.advance();
        if new_season {
            tracing::info!(
                season = self.season.current.name(),
                year = self.season.year,
                "New season"
            );
        }
        // ── 16a. §19.5.I (Iteration 148): Technology discovery — the yearly
        // innovation pass, on the same 4320-tick cadence as the monthly
        // ritual. Calibrated windows (golden @2000, snapshots ≤2000) contain
        // no pass at all; draws come from the virgin Narrative stream, so a
        // pass cannot perturb any other subsystem.
        if tick_u64 > 0 && tick_u64.is_multiple_of(4320) {
            self.tick_technology_discovery(tick_u64);
        }
        // §5 (Iteration 150): the yearly diplomacy pass — off-map neighbors
        // act (caravans / raids) on the same cadence; draws come from the
        // virgin Economy stream, so no other subsystem's RNG can shift.
        if tick_u64 > 0 && tick_u64.is_multiple_of(crate::diplomacy::DIPLOMACY_CADENCE) {
            self.tick_diplomacy(tick_u64);
        }
        // §5 (Iteration 151): the yearly school term — formal cohort
        // instruction, gated on a School site existing. No default world
        // places one, so the pass is a structural no-op in every calibrated
        // window, and it draws no randomness even when a school exists.
        // Both this pass and the per-tick apprenticeship guard idempotently
        // (a `has_learned` check and a `contains` before any push), so
        // whichever runs first in a tick wins and the other skips the
        // student — the same-tick interplay is self-consistent either way.
        if tick_u64 > 0 && tick_u64.is_multiple_of(crate::schools::SCHOOL_CADENCE) {
            self.tick_school_term(tick_u64);
        }
        // §5 (Iteration 152): the half-year religious pass — conversions and
        // conviction drift on the yearly mark, festivals on the mid-year
        // mark. Gated on a seeded Religion (no default world seeds one, so
        // the pass is a structural no-op in every calibrated window) and
        // fully deterministic (no RNG drawn even when a religion exists).
        if tick_u64 > 0 && tick_u64.is_multiple_of(crate::theology::RELIGIOUS_CADENCE) {
            self.tick_theology(tick_u64);
        }
        // §5 (Iteration 153): the yearly military pass — conscription and
        // drills, gated on a Barracks site existing. No default world places
        // one, so the pass is a structural no-op in every calibrated window,
        // and it draws no randomness even when a barracks exists.
        if tick_u64 > 0 && tick_u64.is_multiple_of(crate::military::MILITARY_CADENCE) {
            self.tick_military(tick_u64);
        }
        // Reset work ticks tracker for this tick
        for tick in &mut self.site_work_ticks {
            *tick = 0;
        }
        // Count work actions per site this tick
        for (_agent_idx, action) in &self.tick_action_starts {
            if let ActionKind::Work = action {
                if let Some(farm_idx) = self.world.best_farm_for_work() {
                    if farm_idx < self.site_work_ticks.len() {
                        self.site_work_ticks[farm_idx] += 1;
                    }
                }
            }
        }
        // Apply ecology system to tile fertility
        let mut fertilities: Vec<Fixed> = self.world.tiles.iter().map(|t| t.fertility).collect();
        ecology::system_ecology(
            &self.season,
            &mut fertilities,
            &self.site_work_ticks,
            &self.ecology_config,
        );
        for (i, tile) in self.world.tiles.iter_mut().enumerate() {
            if i < fertilities.len() {
                tile.fertility = fertilities[i];
            }
        }

        // ── 17. Health: disease effects on agents ──
        {
            let n = self.agents.len();
            for i in 0..n {
                let hunger = self.agents[i].needs.hunger;
                let fatigue = self.agents[i].needs.fatigue;
                let body = &mut self.agents[i].body;
                health::system_health(
                    &mut body.health,
                    &mut body.energy,
                    &mut self.agent_diseases[i],
                    hunger,
                    fatigue,
                    &self.health_config,
                );
            }
        }

        // ── 17b. §7.5: Epidemic disease spread — proximity-based contagion ──
        // Contagious diseases (Cold, Fever, Epidemic) spread to nearby agents.
        // Uses Manhattan distance ≤ 2 as "close contact" radius.
        //
        // Iteration 185 (P5 100K audit): same-kind dedup. The old code
        // checked `already_infected` only against the PRE-TICK disease vecs;
        // `new_infections` is applied after the whole scan, so within ONE
        // tick every source's every disease could push the same (j, kind)
        // repeatedly — a freshly-recovered agent absorbed pushes from all
        // sources' all diseases at once, and the nested per-disease loops
        // multiplied the pile (probe: 100K pestilence seed 42 carried
        // 809,303 concurrent Epidemic entries across 12 agents — 8,396 on
        // one agent alone — and per-tick cost grew with the pile, blowing
        // up the run). Dedup against the pending infections caps each
        // agent at ONE new infection per kind per tick; combined with the
        // already_infected cross-tick guard that caps concurrent
        // same-kind diseases at 1, so the vecs stay bounded and the
        // contagion cost linear. Deterministic (Vec preserves push order;
        // the dedup check runs before the RNG draw, so the draw sequence
        // is a deterministic function of the new state).
        {
            let n = self.agents.len();
            let mut new_infections: Vec<(usize, health::DiseaseKind)> = Vec::new();
            for i in 0..n {
                if self.agent_diseases[i].is_empty() {
                    continue;
                }
                for disease in &self.agent_diseases[i] {
                    let rate = disease.kind.transmission_rate();
                    if rate <= Fixed::ZERO {
                        continue; // not contagious
                    }
                    // Check proximity to all other agents
                    for j in 0..n {
                        if i == j || j >= self.agents.len() {
                            continue;
                        }
                        let dist = self.agents[i]
                            .position
                            .manhattan_distance(&self.agents[j].position);
                        if dist > 2 {
                            continue; // too far
                        }
                        // Already infected with this kind (pre-tick state)?
                        let already_infected = self.agent_diseases[j]
                            .iter()
                            .any(|d| d.kind == disease.kind);
                        if already_infected {
                            continue;
                        }
                        // Iteration 185: also skip if a same-kind infection is
                        // already pending for j THIS tick (see the block
                        // comment above).
                        if new_infections
                            .iter()
                            .any(|(aj, ak)| *aj == j && *ak == disease.kind)
                        {
                            continue;
                        }
                        // Probability check — reduced by distance and target health
                        let distance_factor = Fixed::from_f64(1.0)
                            - Fixed::from_f64(dist as f64) * Fixed::from_f64(0.25);
                        let health_factor = self.agents[j].body.health;
                        let transmission_prob = rate
                            * distance_factor
                            * (Fixed::ONE - health_factor * Fixed::from_f64(0.5));
                        let roll =
                            Fixed::from_f64(self.rng.get_mut(RngStream::Social).random::<f64>());
                        if roll < transmission_prob {
                            new_infections.push((j, disease.kind));
                        }
                    }
                }
            }
            // Apply new infections
            for (agent_idx, kind) in new_infections {
                if agent_idx < self.agent_diseases.len() {
                    self.agent_diseases[agent_idx].push(health::ActiveDisease::new(kind));
                    tracing::debug!(
                        agent = agent_idx,
                        disease = ?kind,
                        "Epidemic: disease transmitted"
                    );
                }
            }
        }

        // ── 17b-2. §32 seasonal Cold/Fever vector (Iteration 187) ──
        // The disease system's Cold/Fever kinds were unreachable in real
        // runs (defined + unit-tested, never seeded — audit FINDING 3). This
        // daily pass makes them seasonal: when the weather is cold
        // (temperature < 0.4, the Winter band), each susceptible agent rolls
        // a small chance to catch a Cold, and a Cold can escalate into a
        // Fever (an untreated cold's secondary infection). Deterministic
        // (the seeded Social stream), sparse (daily gate + tiny rates: at
        // Winter temperature 0.2, ~0.2%/day per agent → ~17%/agent/season),
        // and same-kind dedup-guarded like the epidemic pass. Cold/Fever
        // resolve through the normal `system_health` duration path, so the
        // per-kind cap (one of each kind) holds.
        if phases.is_daily {
            let n = self.agents.len();
            let cold_season =
                (Fixed::from_f64(COLD_SEASON_TEMP) - self.weather.temperature).clamp_01();
            if cold_season > Fixed::ZERO {
                for i in 0..n {
                    if self.agent_diseases[i].len() >= health::MAX_CONCURRENT_DISEASES {
                        continue;
                    }
                    let has_cold = self.agent_diseases[i]
                        .iter()
                        .any(|d| d.kind == health::DiseaseKind::Cold);
                    let has_fever = self.agent_diseases[i]
                        .iter()
                        .any(|d| d.kind == health::DiseaseKind::Fever);
                    if !has_cold {
                        let cold_roll =
                            Fixed::from_f64(self.rng.get_mut(RngStream::Social).random::<f64>());
                        if cold_roll < COLD_SEASON_RATE * cold_season {
                            self.agent_diseases[i]
                                .push(health::ActiveDisease::new(health::DiseaseKind::Cold));
                            tracing::debug!(agent = i, "Seasonal cold contracted");
                        }
                    } else if !has_fever {
                        let fever_roll =
                            Fixed::from_f64(self.rng.get_mut(RngStream::Social).random::<f64>());
                        if fever_roll < FEVER_ESCALATION_RATE {
                            self.agent_diseases[i]
                                .push(health::ActiveDisease::new(health::DiseaseKind::Fever));
                            tracing::debug!(agent = i, "Cold escalated into fever");
                        }
                    }
                }
            }
        }

        // ── 17c. §Phase 1.5: Ecology migration pressure ──
        // Agents under high resource scarcity may migrate to a different site.
        {
            let total_grain = self.world.total_food();
            let local_scarcity = (Fixed::ONE - total_grain).clamp_01();
            let season = self.season.current;
            for i in 0..self.agents.len() {
                let has_shelter = self.agents[i].home_site.is_some();
                let pressure = ecology::migration_pressure(local_scarcity, season, has_shelter);
                if pressure > Fixed::from_f64(0.7) {
                    // Agent migrates — teleport to a random site (simplified)
                    if let Some(farm_idx) = self.world.best_farm_for_work() {
                        if farm_idx < self.world.sites.len() {
                            let pos = self
                                .world
                                .site_position(farm_idx)
                                .map_or(crate::sim::Position::new(8, 8), |(x, y)| {
                                    crate::sim::Position::new(x, y)
                                });
                            self.agents[i].position = pos;
                        }
                    }
                }
            }
        }

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
            let mut socialization_updates: Vec<(usize, u64)> = Vec::new();
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
                                socialization_updates.push((i, knowledge_id));
                            }
                        }
                    }
                }
            }
            for (ci, knowledge_id) in socialization_updates {
                self.agents[ci].cultural.knowledge.push(knowledge_id);
                if let Some(k) = self
                    .knowledge_store
                    .iter_mut()
                    .find(|k| k.id == knowledge_id)
                {
                    k.holders += 1;
                }
                // §19.5.F: Record socialization in journal for observability
                self.journal.record(
                    tick_u64,
                    AgentId::new(ci as u64),
                    JournalEntryKind::KnowledgeSocialized { knowledge_id },
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
            let mut innovations: Vec<(usize, u64, String)> = Vec::new();
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
                            innovations.push((idx, k.id, k.name.clone()));
                            break; // one discovery per tick per agent
                        }
                    }
                }
            }
            for (agent_idx, knowledge_id, name) in innovations {
                self.agents[agent_idx].cultural.knowledge.push(knowledge_id);
                if let Some(ks) = self
                    .knowledge_store
                    .iter_mut()
                    .find(|ks| ks.id == knowledge_id)
                {
                    ks.holders += 1;
                }
                self.journal.record(
                    tick_u64,
                    AgentId::new(agent_idx as u64),
                    JournalEntryKind::KnowledgeDiscovered { knowledge_id, name },
                );
                self.events.push(SimEvent::KnowledgeTransferred {
                    source: AgentId::new(agent_idx as u64),
                    target: AgentId::new(agent_idx as u64),
                    knowledge_id,
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

        // ── §6 + §10.6/§10.7: Kinship & Household daily update ──
        self.tick_kinship_household_daily(tick_u64, phases);

        // ── Architecture-plan-2 §8.1.5: Motivation relieve() calls ──
        // Relieve psychological needs based on current actions.
        // Amounts calibrated so multiple actions needed for full satisfaction
        // (relieve ≈ 3–5× growth_rate per tick, creating realistic behavioral pressure).
        for i in 0..self.agents.len() {
            match self.agents[i].current_action {
                ActionKind::Socialize => {
                    self.agents[i]
                        .motivation
                        .belonging
                        .relieve(Fixed::from_f64(0.01));
                    self.agents[i]
                        .motivation
                        .attachment
                        .relieve(Fixed::from_f64(0.008));
                }
                ActionKind::Worship => {
                    self.agents[i]
                        .motivation
                        .meaning
                        .relieve(Fixed::from_f64(0.008));
                    self.agents[i]
                        .motivation
                        .certainty
                        .relieve(Fixed::from_f64(0.005));
                }
                ActionKind::Trade => {
                    self.agents[i]
                        .motivation
                        .competence
                        .relieve(Fixed::from_f64(0.008));
                    self.agents[i]
                        .motivation
                        .recognition
                        .relieve(Fixed::from_f64(0.005));
                }
                ActionKind::Work => {
                    self.agents[i]
                        .motivation
                        .competence
                        .relieve(Fixed::from_f64(0.006));
                    self.agents[i]
                        .motivation
                        .recognition
                        .relieve(Fixed::from_f64(0.006));
                }
                ActionKind::Rest => {
                    self.agents[i]
                        .motivation
                        .sleep
                        .relieve(Fixed::from_f64(0.02));
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
                self.metric_history.remove(0);
            }
        }
    }

    // ── Tick subsystem methods ──────────────────────────────────────
    // Extracted from tick() for readability and maintainability.
    // Each method handles one coherent subsystem tick.
}
