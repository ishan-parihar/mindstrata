//! Relational fields and social cluster dynamics.

use super::{
    ActionKind, ConflictKind, DeathCause, Fixed, InstitutionKind, RngStream, SimEvent, Simulation,
    Tick, ATTACHMENT_COMFORT_DAMPING, COURTSHIP_TRUST_THRESHOLD, DEMOGRAPHY_TICK_INTERVAL,
    EXPECTED_GRAIN_PER_AGENT, KNOWLEDGE_TABOO_FLOOR, KNOWLEDGE_TABOO_RATE,
    PATRONAGE_AFFECTION_THRESHOLD, PATRONAGE_CHANCE_SCALE, PATRONAGE_MAX_CLIENTS_PER_PATRON,
    PATRONAGE_STATUS_GAP, PATRONAGE_TRUST_THRESHOLD,
};
use crate::demography;
use crate::gossip;
use crate::institutions;
use crate::logistics;
use crate::market;

use rand::Rng;
impl Simulation {
    /// Relationship quality (0–1) between two agents, from the source's
    /// relationship_v2 view; defaults to a neutral 0.5 when absent.
    pub(super) fn relationship_quality(&self, from: usize, to: usize) -> Fixed {
        self.agents[from]
            .relationship_v2s
            .iter()
            .find(|r| r.to.as_u64() as usize == to)
            .map_or(Fixed::from_f64(0.5), |r| {
                (r.trust + r.affection) * Fixed::from_f64(0.5)
            })
    }

    /// §10.1: Refresh each agent's three relational fields.
    ///
    /// Sensory: proximity (agents within `PERCEPTION_RADIUS`) and the
    /// perceived emotional expression of neighbors (mean (fear+anger)/2).
    /// Social: the relationship-graph summary (mean trust, mean obligation,
    /// kin-stage count, highest peer status). Noospheric: held beliefs
    /// (mean confidence, hottest emotional charge), perceived legitimacy, and
    /// the world's collective fear.
    ///
    /// Deterministic: index-order iteration, no RNG, no hash-map iteration.
    /// Observational: no decisional consumer is wired yet, so calibrated runs
    /// carry zero drift.
    pub(super) fn refresh_relational_fields(&mut self) {
        use crate::social::relational_field::{RelationalFields, PERCEPTION_RADIUS};
        let n = self.agents.len();
        if n == 0 {
            return;
        }
        let collective_fear = self
            .agents
            .iter()
            .map(|a| a.emotions.fear)
            .fold(Fixed::ZERO, |acc, f| acc + f)
            / Fixed::from_int(n as i64);
        // Single-pass accumulation per agent — no intermediate Vec allocations
        // (this runs per agent per daily tick; the plan targets large
        // populations, §17.4).
        for i in 0..n {
            // ── Sensory field: neighbor scan accumulates count / min distance /
            // stress sum / max peer status in one pass. ──
            let mut nearby_count: u32 = 0;
            let mut nearest = i32::MAX;
            let mut stress_sum = Fixed::ZERO;
            let mut peer_status = Fixed::ZERO;
            for j in 0..n {
                if i == j {
                    continue;
                }
                let d = self.agents[i]
                    .position
                    .manhattan_distance(&self.agents[j].position);
                if d <= PERCEPTION_RADIUS {
                    nearby_count += 1;
                    nearest = nearest.min(d);
                    stress_sum += (self.agents[j].emotions.fear + self.agents[j].emotions.anger)
                        * Fixed::from_f64(0.5);
                    peer_status = peer_status.max(self.agents[j].status_v2.effective_status());
                }
            }
            // ── Social field ──
            let mut trust_sum = Fixed::ZERO;
            let mut oblig_sum = Fixed::ZERO;
            let mut kin_count: u32 = 0;
            for r in &self.agents[i].relationship_v2s {
                trust_sum += r.trust;
                oblig_sum += r.obligation;
                if crate::social::relationship_stages::is_kin_stage(r.stage) {
                    kin_count += 1;
                }
            }
            let rel_count = self.agents[i].relationship_v2s.len().max(1);
            // ── Noospheric field ──
            let mut belief_sum = Fixed::ZERO;
            let mut hottest_charge = Fixed::ZERO;
            for b in &self.agents[i].beliefs {
                belief_sum += b.confidence;
                hottest_charge = hottest_charge.max(b.emotional_charge);
            }
            let belief_count = self.agents[i].beliefs.len().max(1);
            self.agents[i].relational_fields = RelationalFields {
                nearby_agents: nearby_count,
                nearest_closeness: if nearest == i32::MAX {
                    Fixed::ZERO
                } else {
                    RelationalFields::closeness(nearest)
                },
                perceived_stress: if nearby_count == 0 {
                    Fixed::ZERO
                } else {
                    stress_sum / Fixed::from_int(nearby_count as i64)
                },
                social_trust: trust_sum / Fixed::from_int(rel_count as i64),
                social_obligation: oblig_sum / Fixed::from_int(rel_count as i64),
                kin_count,
                peer_status,
                belief_confidence: belief_sum / Fixed::from_int(belief_count as i64),
                hottest_charge,
                legitimacy_perceived: self.agents[i].legitimacy_field.overall,
                collective_fear,
            };
        }
    }

    /// Sections 18-21: Feud tracking, market, demography, conflict, carrying cost.
    pub(super) fn tick_social_cluster(
        &mut self,
        pre_tick_events: usize,
        tick_u64: u64,
        tick: Tick,
        phases: crate::scheduler::TickPhases,
    ) {
        // ── 18. §19.5.G Feud tracking — repeated violence creates persistent feuds ──
        let snap_count = self.events.len();
        for idx in pre_tick_events..snap_count {
            let ev = &self.events[idx];
            if let SimEvent::ConflictOccurred {
                aggressor,
                target,
                kind,
                ..
            } = ev
            {
                if *kind == ConflictKind::Violence {
                    let a_idx = aggressor.as_u64() as usize;
                    let t_idx = target.as_u64() as usize;
                    if a_idx < self.agents.len() && t_idx < self.agents.len() {
                        let was_new = !self.agents[a_idx].feuds.contains(&t_idx);
                        if was_new {
                            self.agents[a_idx].feuds.push(t_idx);
                            self.agents[a_idx].feud_ticks.push(tick_u64);
                            self.agents[t_idx].feuds.push(a_idx);
                            self.agents[t_idx].feud_ticks.push(tick_u64);
                            self.events.push(SimEvent::FeudFormed {
                                party_a: *aggressor,
                                party_b: *target,
                                tick,
                            });
                            // §10.8: Repeated violence forges a clan enmity (feud boundary).
                            self.forge_clan_enmity(a_idx, t_idx, tick_u64);
                        }
                    }
                }
            }
        }

        // ── 20. Conflict state update — trauma decay, combat fatigue, feud decay ──
        for agent in &mut self.agents {
            agent.conflict.update(&self.params);
            // §19.5.G: Feud decay — remove feuds older than 500 ticks
            let feud_decay_threshold = tick_u64.saturating_sub(500);
            let mut keep = 0;
            for j in 0..agent.feuds.len() {
                if agent.feud_ticks[j] > feud_decay_threshold {
                    agent.feuds[keep] = agent.feuds[j];
                    agent.feud_ticks[keep] = agent.feud_ticks[j];
                    keep += 1;
                }
            }
            agent.feuds.truncate(keep);
            agent.feud_ticks.truncate(keep);
        }

        // §10.8/§19.5.G: Clear clan enmities whose feuds have fully decayed —
        // a feud that ends can heal into peace (and later, a marriage alliance).
        self.decay_clan_enmities();

        // ── 22. Courtship creation — eligible agents begin romantic courtship ──
        // Architecture-plan-2 §10.4: When two unpartnered adult agents with sufficient
        // attraction and trust interact, a Courtship is initiated.
        // Runs every 12 ticks (~2 hours) to avoid O(N²) overhead every tick.
        if phases.is_duodeca {
            let n = self.agents.len();
            // Build HashSet of agents already in courtships for O(1) lookup.
            let mut in_courtship: std::collections::HashSet<usize> =
                std::collections::HashSet::new();
            for c in &self.active_courtships {
                if c.active {
                    in_courtship.insert(c.pursuer);
                    in_courtship.insert(c.pursued);
                }
            }
            for i in 0..n {
                // Skip agents already in a courtship, partnered, or minors
                if in_courtship.contains(&i) {
                    continue;
                }
                if self.agents[i].partner.is_some() {
                    continue;
                }
                if self.agents[i].age < Fixed::from_f64(16.0) {
                    continue;
                }

                for j in (i + 1)..n {
                    if in_courtship.contains(&j) {
                        continue;
                    }
                    if self.agents[j].partner.is_some() {
                        continue;
                    }
                    if self.agents[j].age < Fixed::from_f64(16.0) {
                        continue;
                    }

                    // Check kinship — skip close relatives
                    let kinship_coeff = self.kinship_graph.coefficient_between(i, j);

                    // Compute mutual trust from relationship_v2
                    let idx_ij = Self::relationship_v2_pos(i, j);
                    let idx_ji = Self::relationship_v2_pos(j, i);
                    let trust_ij = if idx_ij < self.agents[i].relationship_v2s.len() {
                        self.agents[i].relationship_v2s[idx_ij].trust
                    } else {
                        Fixed::ZERO
                    };
                    let trust_ji = if idx_ji < self.agents[j].relationship_v2s.len() {
                        self.agents[j].relationship_v2s[idx_ji].trust
                    } else {
                        Fixed::ZERO
                    };
                    let avg_trust = (trust_ij + trust_ji) * Fixed::from_f64(0.5);

                    // Delegate all eligibility checks to the function
                    let already_bonded = self.marriage_registry.find_bond(i, j).is_some();
                    let eligible = crate::social::courtship::eligible_for_courtship(
                        self.agents[i].age,
                        self.agents[j].age,
                        kinship_coeff,
                        already_bonded,
                        COURTSHIP_TRUST_THRESHOLD,
                        avg_trust,
                    );
                    if !eligible {
                        continue;
                    }

                    // Deterministic chance to start courtship — based on trust
                    let chance = avg_trust * Fixed::from_f64(0.02);
                    let roll =
                        Fixed::from_f64(self.rng.get_mut(RngStream::Social).random_range(0.0..1.0));
                    if roll < chance {
                        let mut courtship =
                            crate::social::courtship::Courtship::new(i, j, tick_u64);
                        courtship.mutual_attraction = avg_trust;
                        courtship.stage = crate::social::marriage::RomanticStage::Awareness;
                        // Update the HashSet immediately to prevent double-matching
                        // within the same tick (agent in two courtships simultaneously).
                        in_courtship.insert(i);
                        in_courtship.insert(j);
                        self.active_courtships.push(courtship);
                    }
                }
            }
        }

        // ── 22b. Patronage creation (architecture-plan-2 §10.9) ──────────
        // When a high-status agent has repeated positive interactions with a low-status
        // agent, a patronage relationship may form. Runs every 12 ticks to limit O(N²).
        if phases.is_duodeca {
            // Pre-cache effective status for all agents (O(N) instead of O(N²) recomputation).
            let status_cache: Vec<Fixed> = self
                .agents
                .iter()
                .map(|a| a.status_v2.effective_status())
                .collect();
            // Pre-build HashSet of agents already in a patronage relation as client.
            // O(M) build + O(1) lookups vs O(N²×M) with inline patron_of() calls.
            let patronage_clients: std::collections::HashSet<usize> = self
                .agents
                .iter()
                .enumerate()
                .filter(|(idx, _)| self.patronage_registry.patron_of(*idx).is_some())
                .map(|(idx, _)| idx)
                .collect();
            // Track how many clients each patron has acquired this tick (max 3).
            // Vec indexed by agent ID — simpler and avoids HashMap overhead.
            let mut patron_client_counts: Vec<usize> = vec![0; self.agents.len()];
            #[expect(
                clippy::needless_range_loop,
                reason = "intentional parallel-array indexing: status_cache[i/j] + patron_client_counts[i] + relationship_v2s[i]"
            )]
            for i in 0..self.agents.len() {
                let patron_status = status_cache[i];
                if patron_client_counts[i] >= PATRONAGE_MAX_CLIENTS_PER_PATRON {
                    continue;
                }
                for j in 0..self.agents.len() {
                    if i == j {
                        continue;
                    }
                    // Patron must be notably higher status than client
                    if patron_status <= status_cache[j] + PATRONAGE_STATUS_GAP {
                        continue;
                    }
                    // Must not already have a patronage link (O(1) HashSet lookup)
                    if patronage_clients.contains(&j) {
                        continue;
                    }
                    // Check trust from relationship_v2
                    let rv2_idx = Self::relationship_v2_pos(i, j);
                    if rv2_idx >= self.agents[i].relationship_v2s.len() {
                        continue;
                    }
                    let trust = self.agents[i].relationship_v2s[rv2_idx].trust;
                    let affection = self.agents[i].relationship_v2s[rv2_idx].affection; // Need moderate trust and positive affect to form patronage
                    if trust < PATRONAGE_TRUST_THRESHOLD
                        || affection < PATRONAGE_AFFECTION_THRESHOLD
                    {
                        continue;
                    }
                    let chance = (trust * PATRONAGE_CHANCE_SCALE).clamp_01();
                    let roll =
                        Fixed::from_f64(self.rng.get_mut(RngStream::Social).random_range(0.0..1.0));
                    if roll < chance {
                        let rel = crate::social::patronage::PatronageRelation::new(i, j, tick_u64);
                        self.patronage_registry.register(rel);
                        patron_client_counts[i] += 1;
                        tracing::info!(
                            patron = i,
                            client = j,
                            trust = trust.to_f64(),
                            tick = tick_u64,
                            "Patronage relationship formed"
                        );
                    }
                }
            }
        }

        // ── 23. §19.5.E Carrying cost — heavy loads increase fatigue ──
        // Agents who performed physical actions (Work) incur carrying cost based on load.
        // Since agents don't carry items yet, this serves as a workload fatigue modifier.
        for i in 0..self.agents.len() {
            if self.agents[i].current_action == ActionKind::Work {
                let load_ratio = Fixed::from_f64(0.4); // moderate load for work
                let fatigue_cost = logistics::carrying_cost(
                    load_ratio,
                    Fixed::from_f64(1.0),
                    self.agents[i].needs.fatigue,
                );
                self.agents[i].needs.fatigue =
                    (self.agents[i].needs.fatigue + fatigue_cost).clamp_01();
            }
        }

        // ── 19. Market system: price formation, inequality tracking ──
        {
            let agent_economic_state: Vec<_> = self
                .agents
                .iter()
                .map(|a| (a.needs.clone(), a.wealth.clone()))
                .collect();
            market::system_market(
                &self.world,
                &agent_economic_state,
                &mut self.market,
                tick_u64,
                &self.params,
            );

            // §4.4: Update black market state based on current scarcity and enforcement
            // Scarcity is normalized: 0 = abundant, 1 = critically scarce
            let total_grain = self.world.total_food();
            let expected_capacity = Fixed::from_int(self.agents.len() as i64)
                * Fixed::from_int(EXPECTED_GRAIN_PER_AGENT as i64);
            let scarcity = if expected_capacity > Fixed::ZERO {
                (Fixed::ONE - total_grain / expected_capacity).clamp_01()
            } else {
                Fixed::ZERO
            };
            let council_enforcement = self
                .institutions
                .iter()
                .filter(|i| i.kind == InstitutionKind::Council)
                .map(|i| i.enforcement_capacity)
                .fold(Fixed::ZERO, std::cmp::Ord::max);
            self.black_market.update(scarcity, council_enforcement);
        }

        // ── 18. Demography: aging and mortality ──
        if phases.is_deca {
            // process demography every 10 ticks to reduce overhead
            let mut deaths = Vec::new();
            for i in 0..self.agents.len() {
                // §31: Probabilistic death roll — uniform RNG vs age/health-based chance.
                let roll = Fixed::from_f64(self.rng.get_mut(RngStream::Behavior).random::<f64>());
                let (new_age, died) = demography::age_agent(
                    self.agents[i].age,
                    self.agents[i].body.health,
                    self.agents[i].body.energy,
                    DEMOGRAPHY_TICK_INTERVAL,
                    &self.demography_config,
                    roll,
                );
                self.agents[i].age = new_age;
                if died {
                    deaths.push(i);
                }
            }
            // §31: Dead agents are replaced IN PLACE by newborns so the
            // `AgentId::new(i) == index i` invariant is never broken.
            for &idx in &deaths {
                self.handle_agent_death(idx, &deaths, tick_u64, tick, DeathCause::OldAge);
            }
        }

        // §19.5.J: Trim provenance traces to prevent unbounded growth
        self.provenance.trim(institutions::MAX_PROVENANCE_RECORDS);
    }
}

impl Simulation {
    /// §11.2 + §19.5.I: Gossip propagation, attachment reunion, ToM,
    /// cultural encounter, knowledge diffusion, meme transmission.
    pub(super) fn tick_gossip_and_knowledge(
        &mut self,
        pre_tick_events: usize,
        tick_u64: u64,
        tick: Tick,
    ) {
        // Collect event indices to process (avoids .to_vec() heap allocation)
        let event_range = pre_tick_events..self.events.len();
        // Snapshot event count to avoid re-processing newly-pushed events
        let snap_count = self.events.len();

        for idx in event_range.clone() {
            if idx >= snap_count {
                break;
            }
            if let SimEvent::InteractionOccurred { from, to, kind, .. } = self.events[idx] {
                if !matches!(
                    kind,
                    mindstrata_core::event::InteractionKind::Gossip
                        | mindstrata_core::event::InteractionKind::Help
                        | mindstrata_core::event::InteractionKind::Comfort
                        | mindstrata_core::event::InteractionKind::Trade
                ) {
                    continue;
                }
                let from_idx = from.as_u64() as usize;
                let to_idx = to.as_u64() as usize;
                if from_idx >= self.agents.len() || to_idx >= self.agents.len() {
                    continue;
                }

                // §8.1.14: Attachment reunion (Gossip only)
                if matches!(kind, mindstrata_core::event::InteractionKind::Gossip) {
                    self.agents[from_idx].attachment.on_reunion(
                        self.params.attachment_secure_recovery,
                        self.params.attachment_anxious_recovery,
                        self.params.attachment_avoidant_recovery,
                        self.params.attachment_disorganized_recovery,
                    );
                    self.agents[to_idx].attachment.on_reunion(
                        self.params.attachment_secure_recovery,
                        self.params.attachment_anxious_recovery,
                        self.params.attachment_avoidant_recovery,
                        self.params.attachment_disorganized_recovery,
                    );
                }

                // §8.1.14 (Iteration 191 — the active soothing path): a
                // Comfort interaction (the high-affection context's softest
                // signal) now calls `receive_comfort` on the RECIPIENT —
                // pre-Iter-191 the method had ZERO call sites, so separation
                // distress only ever decayed passively (the daily
                // `× decay_factor` fold) and the designed comfort mechanics
                // (style-matched distress relief scaled by the agent's
                // soothing_receptivity × the interaction's quality, plus a
                // security gain for the secure style) never fired. Comfort
                // quality scales with the existing relationship trust
                // between the pair (a trusted figure soothes better; a
                // stranger's comfort barely registers). Deterministic, no
                // RNG — the draw stream is untouched (only state deltas on
                // the recipient's attachment).
                if matches!(kind, mindstrata_core::event::InteractionKind::Comfort) {
                    let rel_trust = self
                        .relationships
                        .iter()
                        .find(|r| r.from == from && r.to == to)
                        .map_or(Fixed::from_f64(0.5), |r| r.trust);
                    // Iteration 191 calibration: the raw recovery rates
                    // (0.3/0.6/0.4/0.5) made a SINGLE comfort event reduce
                    // distress by up to ~0.24 (effectiveness = receptivity ×
                    // quality up to 0.8, × the 0.3 secure rate) while the
                    // daily separation gain is ~0.008 — the active path
                    // erased the coupling entirely (probe: mean 0.000, 0/46
                    // partnered agents nonzero, at EVERY seed × rate, and the
                    // rate-response ratio collapsed to NaN). The comfort
                    // reduction is damped by
                    // `ATTACHMENT_COMFORT_DAMPING` (0.05) so a comfort event
                    // soothes ~1–2 days of accumulated separation (reduction
                    // ~0.01–0.02) — the coupling stays live population-wide
                    // and the separation-rate parameter still drives
                    // distress, but a supported partner's distress genuinely
                    // sits lower than an unsupported one's.
                    let comfort_quality =
                        (Fixed::from_f64(0.4) + rel_trust * Fixed::from_f64(0.6)).clamp_01();
                    let damp = Fixed::from_f64(ATTACHMENT_COMFORT_DAMPING);
                    self.agents[to_idx].attachment.receive_comfort(
                        comfort_quality,
                        self.params.attachment_secure_recovery * damp,
                        self.params.attachment_anxious_recovery * damp,
                        self.params.attachment_avoidant_recovery * damp,
                        self.params.attachment_disorganized_recovery * damp,
                        Fixed::from_f64(0.05),
                    );
                }

                // §8.1.9: Theory of Mind update
                // §17.2: Gate social inference budget — ToM is the most expensive per-interaction op.
                let trust_from_to = self
                    .relationships
                    .iter()
                    .find(|r| r.from == from && r.to == to)
                    .map_or(Fixed::from_f64(0.5), |r| r.trust);
                let trust_to_from = self
                    .relationships
                    .iter()
                    .find(|r| r.from == to && r.to == from)
                    .map_or(Fixed::from_f64(0.5), |r| r.trust);
                // Iteration 198: the observed-behavior inputs were derived
                // from trust with scaled-down multipliers (pos = trust × 0.3,
                // neg = (1−trust) × 0.1), so `infer_intent`'s Friendly
                // (>0.3 positive) and Threatening (>0.3 negative) branches
                // were STRUCTURALLY UNREACHABLE — a model could essentially
                // only ever say Neutral, and the whole §8.1.9 system was
                // write-only. The honest inputs are the actual interaction:
                // Help/Comfort is strongly helpful (0.6), Trade mildly
                // helpful (0.3), Gossip neutral (0.1); none of the four kinds
                // this pass carries is harmful, so negative stays 0 (the
                // hostile kinds never reach this pass). Intent now genuinely
                // differentiates: repeated Help at trust > 0.5 → Friendly.
                let observed_helpful = match kind {
                    mindstrata_core::event::InteractionKind::Help
                    | mindstrata_core::event::InteractionKind::Comfort => Fixed::from_f64(0.6),
                    mindstrata_core::event::InteractionKind::Trade => Fixed::from_f64(0.3),
                    _ => Fixed::from_f64(0.1),
                };
                if self.agents[from_idx].agent_tier.tier.runs_theory_of_mind()
                    && self.agents[from_idx]
                        .agent_tier
                        .budget_tracker
                        .can_social_infer()
                {
                    let model_a = self.agents[from_idx].mind_models.get_or_create(to);
                    model_a.update_from_observation(
                        observed_helpful,
                        Fixed::ZERO,
                        trust_to_from,
                        Fixed::from_f64(0.5),
                    );
                    model_a.infer_intent(trust_to_from, observed_helpful, Fixed::ZERO);
                    let _ = self.agents[from_idx]
                        .agent_tier
                        .budget_tracker
                        .consume_social_inference();
                }
                if self.agents[to_idx].agent_tier.tier.runs_theory_of_mind()
                    && self.agents[to_idx]
                        .agent_tier
                        .budget_tracker
                        .can_social_infer()
                {
                    let model_b = self.agents[to_idx].mind_models.get_or_create(from);
                    model_b.update_from_observation(
                        observed_helpful,
                        Fixed::ZERO,
                        trust_from_to,
                        Fixed::from_f64(0.5),
                    );
                    model_b.infer_intent(trust_from_to, observed_helpful, Fixed::ZERO);
                    let _ = self.agents[to_idx]
                        .agent_tier
                        .budget_tracker
                        .consume_social_inference();
                }

                // §8.1.18: Cultural category encounter
                let (from_cat, to_cat) = {
                    let fc = self.agents[from_idx]
                        .cultural_cognition
                        .categories
                        .iter()
                        .max_by_key(|c| c.identification.to_raw() as u64)
                        .map(|c| c.name.clone());
                    let tc = self.agents[to_idx]
                        .cultural_cognition
                        .categories
                        .iter()
                        .max_by_key(|c| c.identification.to_raw() as u64)
                        .map(|c| c.name.clone());
                    (fc, tc)
                };
                if let (Some(ref fn_name), Some(ref tn_name)) = (&from_cat, &to_cat) {
                    let same = fn_name == tn_name;
                    if let Some(c) = self.agents[from_idx]
                        .cultural_cognition
                        .categories
                        .iter_mut()
                        .find(|c| &c.name == tn_name)
                    {
                        c.encounter(same);
                    }
                    if let Some(c) = self.agents[to_idx]
                        .cultural_cognition
                        .categories
                        .iter_mut()
                        .find(|c| &c.name == fn_name)
                    {
                        c.encounter(same);
                    }
                }

                // Gossip-specific: belief sharing
                if !matches!(kind, mindstrata_core::event::InteractionKind::Gossip) {
                    continue;
                }
                let from_beliefs = &self.agents[from_idx].beliefs;
                if from_beliefs.is_empty() {
                    continue;
                }
                let pick = self
                    .rng
                    .get_mut(RngStream::Social)
                    .random_range(0..from_beliefs.len());
                let source_trust = trust_from_to;
                let rumor = gossip::Rumor::from_belief(&from_beliefs[pick], tick_u64);
                let result = gossip::process_gossip(
                    &rumor,
                    source_trust,
                    &self.agents[from_idx].emotions,
                    &self.agents[from_idx].personality,
                    &self.agents[to_idx].personality,
                    &self.agents[to_idx].beliefs,
                    tick_u64,
                    self.params.gossip_base_fidelity,
                    self.params.gossip_emotional_distortion,
                    self.params.gossip_acceptance_threshold,
                );
                let old_conf = self.agents[to_idx]
                    .beliefs
                    .iter()
                    .find(|b| b.proposition_id == result.proposition_id)
                    .map_or(Fixed::from_f64(0.5), |b| b.confidence);
                gossip::apply_gossip(&mut self.agents[to_idx].beliefs, &result, tick_u64);
                if result.accepted {
                    let new_conf = self.agents[to_idx]
                        .beliefs
                        .iter()
                        .find(|b| b.proposition_id == result.proposition_id)
                        .map_or(result.mutated_confidence, |b| b.confidence);
                    self.provenance
                        .record_belief_update(crate::provenance::BeliefUpdateTrace {
                            agent: to,
                            tick: tick_u64,
                            proposition_id: result.proposition_id,
                            old_confidence: old_conf,
                            new_confidence: new_conf,
                            cause: "gossip".into(),
                            delta: (new_conf - old_conf),
                        });
                    self.events.push(SimEvent::RumorSpread {
                        source: from,
                        target: to,
                        content_hash: result.proposition_id,
                        distortion: (result.mutated_confidence - rumor.confidence).abs(),
                        tick,
                    });
                    // §13.1 + §13.2: Meme transmission using agent epistemic traits.
                    // susceptibility: gullible agents (high conspiracy_susceptibility) accept
                    //   memes more readily; floor of 0.2 ensures realistic information permeability.
                    // skepticism: critical agents (high source_monitoring, low dogmatism) resist
                    //   memes more; multiplier 0.6 gives meaningful differentiation between
                    //   gullible (≈0.08) and critical (≈0.6) agents.
                    let listener_susceptibility =
                        (self.agents[to_idx].epistemic.conspiracy_susceptibility
                            * Fixed::from_f64(0.6)
                            + Fixed::from_f64(0.2))
                        .clamp_01();
                    let listener_skepticism = (self.agents[to_idx].epistemic.source_monitoring
                        * (Fixed::ONE
                            - self.agents[to_idx].epistemic.dogmatism * Fixed::from_f64(0.5))
                        * Fixed::from_f64(0.6))
                    .clamp_01();
                    let nb = Fixed::from_f64(0.05);
                    let mut sampled = 0u32;
                    for meme in &mut self.meme_registry.memes {
                        if !meme.active || sampled >= 3 {
                            break;
                        }
                        let chance = meme.transmission_chance(
                            source_trust,
                            listener_susceptibility,
                            listener_skepticism,
                            self.params.meme_transmission_multiplier,
                        );
                        let roll = self
                            .rng
                            .get_mut(RngStream::Social)
                            .random_range(0.0f64..1.0);
                        if Fixed::from_f64(roll) < chance {
                            // host_count is "number of hosting agents" — clamp to
                            // population so repeated transmissions of the same meme
                            // to the same village can't inflate it past the agent
                            // count (previously unbounded, made visible once memes
                            // could actually spread).
                            let pop = self.agents.len() as u32;
                            meme.host_count = (meme.host_count.saturating_add(1)).min(pop);
                            meme.novelty = (meme.novelty + nb).clamp_01();
                            sampled += 1;
                            // §13.2 (AP2): Meme mutation during transmission —
                            // the five-factor drift (memory error, emotional
                            // exaggeration, identity bias, narrative
                            // simplification, audience tailoring). Gated on the
                            // master multiplier: at ZERO no mutation ever fires
                            // and the decision draw is not consumed (the
                            // identity factor that pinned the golden baseline
                            // while the feature was off — the baseline has
                            // since been intentionally recalibrated with the
                            // feature live at 0.3).
                            if self.params.meme_mutation_rate_base > Fixed::ZERO {
                                let m_roll = self
                                    .rng
                                    .get_mut(RngStream::Social)
                                    .random_range(0.0f64..1.0);
                                if meme.should_mutate_scaled(
                                    Fixed::from_f64(m_roll),
                                    self.params.meme_mutation_rate_base,
                                ) {
                                    let magnitude = meme.mutation_magnitude();
                                    meme.mutate(magnitude);
                                }
                            }
                            // §17.4: Record exposure sample for meme aggregation.
                            let meme_ids = [meme.id];
                            self.meme_aggregator.sample_exposures(
                                to_idx,
                                &meme_ids,
                                source_trust,
                                true,
                                &mut self.exposure_samples,
                            );
                        }
                    }
                }
                // §13.3: Rumor creation from emotionally charged gossip
                if result.accepted && result.emotional_charge > Fixed::from_f64(0.3) {
                    let mut r = crate::culture::RumorV2::new(
                        0,
                        format!("gossip about prop_{}", result.proposition_id),
                        Some(to.as_u64() as usize),
                        result.emotional_charge * Fixed::from_f64(0.5),
                        source_trust,
                        result.emotional_charge,
                        tick_u64,
                    );
                    // §13.3: The originator is in the source chain but has not
                    // retold the rumor — record_source skips the evidence
                    // degradation so stored evidence stays aligned with the
                    // actual hop count.
                    r.record_source(from.as_u64() as usize, tick_u64);
                    self.rumor_registry.register(r);
                }
            }
        }

        // §19.5.I: Knowledge diffusion
        for idx in event_range {
            if idx >= snap_count {
                break;
            }
            if let SimEvent::InteractionOccurred {
                from,
                to,
                kind:
                    mindstrata_core::event::InteractionKind::Gossip
                    | mindstrata_core::event::InteractionKind::Help
                    | mindstrata_core::event::InteractionKind::Trade,
                ..
            } = self.events[idx]
            {
                let fi = from.as_u64() as usize;
                let ti = to.as_u64() as usize;
                if fi >= self.agents.len() || ti >= self.agents.len() {
                    continue;
                }
                let source_trust = self
                    .relationships
                    .iter()
                    .find(|r| r.from == from && r.to == to)
                    .map_or(Fixed::from_f64(0.5), |r| r.trust);
                if !self.agents[fi].cultural.knowledge.is_empty() {
                    let pick = self
                        .rng
                        .get_mut(RngStream::Social)
                        .random_range(0..self.agents[fi].cultural.knowledge.len());
                    let kid = self.agents[fi].cultural.knowledge[pick];
                    // §8.1.18 (Iteration 166): the recipient's taboo layer
                    // dampens absorption of knowledge spread through social
                    // contact — a taboo-bound agent resists novel practices
                    // from peers (sacred boundary defense). ONE-SIDED
                    // factor, identity at zero; the recipient's acceptance
                    // must still clear the 0.5 floor.
                    let resistance = crate::psychology::cultural_cognition::taboo_knowledge_factor(
                        self.agents[ti].cultural_cognition.max_taboo_strength(),
                        KNOWLEDGE_TABOO_RATE,
                        KNOWLEDGE_TABOO_FLOOR,
                    );
                    let acceptance = (source_trust * Fixed::from_f64(0.5)
                        + self.agents[ti].cultural.openness * Fixed::from_f64(0.5))
                    .clamp_01()
                        * resistance;
                    if !self.agents[ti].cultural.knowledge.contains(&kid)
                        && acceptance > Fixed::from_f64(0.5)
                        // §5 (Iteration 148): interaction diffusion is gated
                        // by the technology tree's prerequisites.
                        && self.technology.can_learn(&self.agents[ti].cultural.knowledge, kid)
                    {
                        self.agents[ti].cultural.knowledge.push(kid);
                        if let Some(k) = self.knowledge_store.iter_mut().find(|k| k.id == kid) {
                            k.holders += 1;
                        }
                        // §8.1.7: Successful knowledge acquisition is evidence
                        // exposure — the absorbed knowledge desacralizes the
                        // recipient's sacred values in proportion to absorption
                        // strength (acceptance) and reasoning capacity
                        // (executive function). Zero-at-zero anchor: no
                        // acquisition -> no pressure, and the internal >0.1
                        // gate inside attempt_desacred keeps very sacred values
                        // inert (only mid-sacredness values slowly secularize).
                        let reasoning = self.agents[ti].cognitive.executive_capacity;
                        self.agents[ti]
                            .sacred_values
                            .desacralize_through_exposure(acceptance, reasoning);
                        self.events.push(SimEvent::KnowledgeTransferred {
                            source: from,
                            target: to,
                            knowledge_id: kid,
                            tick,
                        });
                        self.provenance.record_institutional(
                            crate::provenance::InstitutionalTrace {
                                institution_name: "Community".into(),
                                tick: tick_u64,
                                decision_kind: "knowledge_transfer".into(),
                                description: format!(
                                    "Agent {} taught knowledge {} to Agent {}",
                                    from.as_u64(),
                                    kid,
                                    to.as_u64()
                                ),
                                affected: vec![from, to],
                                success: true,
                            },
                        );
                    }
                }
            }
        }
    }
}
