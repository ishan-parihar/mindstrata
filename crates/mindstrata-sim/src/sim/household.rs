//! Household food pooling and daily kinship economics.

use super::{
    Fixed, InstitutionKind, MemoryKind, MemoryTag, Simulation, Tick,
    GROUP_COUNCIL_LEGITIMACY_DEFAULT, GROUP_SOCIAL_COST_SCALE, GROUP_SUPPRESSION_SCALE,
};
use crate::institutions;

impl Simulation {
    /// §10.7 (AP2): Household food pooling — the plan's "resource pooling"
    /// and "division of labor" dimensions, now decisional (Iteration 119;
    /// previously `pool_food`/`distribute_food` had ZERO production call
    /// sites and `HouseholdRole` had ZERO consumers). Every day, well-fed
    /// adults of a multi-member household contribute a share of their
    /// surplus to the communal pot (`pool_food`), then hungry members are
    /// fed dependents-first — children and elders (the childcare / elder
    /// care dimensions) before adults — via `distribute_food`. Hunger
    /// relief is exact (`min(hunger, ration)`), never overshoots, and the
    /// pool never goes negative. Contributions are deliberately savings-only
    /// (inverse to need): a starving adult contributes nothing, so a
    /// prolonged famine drains the founding cache to zero — the fold is an
    /// emergency buffer, not a subsistence model. Singleton households are
    /// skipped entirely
    /// (identity by construction — one member has no one to pool with or
    /// care for), and every current calibrated window is all-singleton
    /// (probe-pinned: no marriages form in ≤2000-tick windows), so the
    /// golden, all snapshots, and every trajectory pin stay byte-identical.
    /// Deterministic: no RNG, fixed member order.
    pub fn tick_household_food_pooling(&mut self) {
        use crate::social::household::{
            HouseholdRole, HOUSEHOLD_ADULT_FEED_RATIO, HOUSEHOLD_FEED_RELIEF,
            HOUSEHOLD_HUNGER_FEED_THRESHOLD, HOUSEHOLD_POOL_RATE,
        };
        let threshold = Fixed::from_f64(HOUSEHOLD_HUNGER_FEED_THRESHOLD);
        let dependent_ration = Fixed::from_f64(HOUSEHOLD_FEED_RELIEF);
        let adult_ration = Fixed::from_f64(HOUSEHOLD_FEED_RELIEF * HOUSEHOLD_ADULT_FEED_RATIO);
        let pool_rate = Fixed::from_f64(HOUSEHOLD_POOL_RATE);
        for h in &mut self.households {
            if h.members.len() < 2 {
                continue; // singleton — no pooling partner; identity by construction
            }
            // Guard: roles must run parallel to members (production and
            // add/remove/derive all maintain this; a malformed or
            // deserialized household must not panic a pub method).
            if h.roles.len() != h.members.len() {
                continue;
            }
            // Clone the small member list so `h.pool_food`/`h.distribute_food`
            // (mutable) don't fight the iterator's immutable borrow.
            let members: Vec<usize> = h.members.clone();
            // Dependents consume but do not produce: children, elders, and
            // household dependents. (`HouseholdRole::Dependent` is included
            // for completeness — `derive_roles` never assigns it today.)
            let dependent: Vec<bool> = h
                .roles
                .iter()
                .map(|r| {
                    matches!(
                        r,
                        HouseholdRole::Child | HouseholdRole::Elder | HouseholdRole::Dependent
                    )
                })
                .collect();
            // Division of labor: adults (Head/Partner/Adult) contribute a
            // share of their surplus — how far they sit below the feed
            // threshold — to the communal pot.
            for (idx, &member) in members.iter().enumerate() {
                if dependent[idx] {
                    continue; // dependents consume, they do not produce
                }
                let hunger = self.agents[member].needs.hunger;
                if hunger < threshold {
                    h.pool_food((threshold - hunger) * pool_rate);
                }
            }
            // Childcare / elder care: dependents are fed first, then hungry
            // adults from the residual pot.
            for pass in 0..2 {
                for (idx, &member) in members.iter().enumerate() {
                    let hunger = self.agents[member].needs.hunger;
                    if hunger < threshold {
                        continue;
                    }
                    let is_dep = dependent[idx];
                    let want_pass = if pass == 0 { is_dep } else { !is_dep };
                    if !want_pass {
                        continue;
                    }
                    let ration = if is_dep {
                        dependent_ration
                    } else {
                        adult_ration
                    };
                    let given = h.distribute_food(hunger.min(ration));
                    if given > Fixed::ZERO {
                        self.agents[member].needs.hunger = (hunger - given).max(Fixed::ZERO);
                    }
                }
            }
        }
    }

    /// §6 + §10.6/§10.7: Kinship & Household daily update.
    pub(super) fn tick_kinship_household_daily(
        &mut self,
        tick_u64: u64,
        phases: crate::scheduler::TickPhases,
    ) {
        // §10.6: Decay kinship edges daily.
        // §10.7: Tick household dynamics (cohesion, conflict, reputation).
        if phases.is_daily {
            self.kinship_graph.decay_daily();
            // §10.7 (AP2): Refresh household roles + traditions — the plan's
            // division-of-labor and `traditions: Vec<PracticeId>` dimensions.
            // Deterministic pure functions of existing agent state (no RNG),
            // writing only the new §10.7 fields, so calibrated runs stay
            // byte-identical.
            let ages: Vec<Fixed> = self.agents.iter().map(|a| a.age).collect();
            let partners: Vec<Option<usize>> = self.agents.iter().map(|a| a.partner).collect();
            let practices: Vec<Vec<u64>> = self
                .agents
                .iter()
                .map(|a| a.cultural.practices.clone())
                .collect();
            for household in &mut self.households {
                household.tick_update();
                household.derive_roles(&ages, &partners);
                household.collect_traditions(&practices);
            }
            // §10.7 (AP2): Household food pooling — division of labor
            // (adults contribute surplus) + childcare/elder care (dependents
            // fed first). Multi-member households only; singleton worlds
            // stay byte-identical by construction.
            self.tick_household_food_pooling();
            // Architecture-plan-2 §10.9: Tick patronage relations daily.
            self.patronage_registry.daily_update();
            // §10.9: Patrons provision destitute clients — patronage as an
            // economic safety net (wealth flows through the social structure).
            self.tick_patronage_provision();
            // Architecture-plan-2 §13.1: Decay meme novelty daily. Iteration
            // 174: the novelty-decay RATE (meme_novelty_decay, 0.002) is now
            // the live knob — the factor is derived (ONE - rate); the old
            // meme_novelty_decay_factor field was a redundant complement
            // (0.998 = 1 - 0.002) and is removed.
            self.meme_registry
                .tick_all(Fixed::ONE - self.params.meme_novelty_decay);
            // §13.1 (AP2): Derive institutional_backing deterministically — a
            // meme is backed by the first institution whose domain matches its
            // content type (theological→temple, political/moral→council,
            // practical→market). Only the new field is written (no RNG), so
            // the golden baseline stays byte-identical. Suppression stays at
            // its default ZERO today; campaign wiring can raise it later.
            {
                let temple = self
                    .institutions
                    .iter()
                    .find(|i| i.kind == InstitutionKind::Temple)
                    .map(|i| i.id);
                let council = self
                    .institutions
                    .iter()
                    .find(|i| i.kind == InstitutionKind::Council)
                    .map(|i| i.id);
                let market = self
                    .institutions
                    .iter()
                    .find(|i| i.kind == InstitutionKind::Market)
                    .map(|i| i.id);
                for meme in &mut self.meme_registry.memes {
                    meme.institutional_backing = match meme.content_type {
                        crate::culture::meme::MemeContent::Theological
                        | crate::culture::meme::MemeContent::Prophecy
                        | crate::culture::meme::MemeContent::Historical => temple,
                        crate::culture::meme::MemeContent::Political
                        | crate::culture::meme::MemeContent::Moral
                        | crate::culture::meme::MemeContent::Slogan => council,
                        crate::culture::meme::MemeContent::Practical
                        | crate::culture::meme::MemeContent::Rumor => market,
                        _ => None,
                    };
                }
            }
            // §13.6 (AP2): Derive narrative dominance from the meme pool — a
            // narrative dominates when it is both credible and viral. A
            // BTreeMap (vs the plan's HashMap) keeps the ordering deterministic
            // for serialization. Only the new field is written (no RNG).
            self.echo_chamber.narrative_dominance = self
                .meme_registry
                .memes
                .iter()
                .map(|m| (m.id as u64, (m.credibility * m.virality).clamp_01()))
                .collect();
            // §13.5 (AP2): Derive the village's founding myths — memes
            // carrying the group's origin narrative (Historical content) are
            // the founding myths (plan types this `Vec<MemeId>`).
            self.collective_memory_registry
                .get_or_create(0)
                .founding_myths = self
                .meme_registry
                .memes
                .iter()
                .filter(|m| m.content_type == crate::culture::meme::MemeContent::Historical)
                .map(|m| m.id as u64)
                .collect();
            // §17.4: Aggregate meme metrics for large-population observability.
            self.wire_meme_aggregation(tick_u64);
            // Architecture-plan-2 §13.3: Decay rumor prevalence daily.
            self.rumor_registry.tick_all(tick_u64);
            // Architecture-plan-2 §13.3 + §12.3: Deterministic rumor transmission
            // pass — each active rumor spreads to its most receptive listener,
            // with the transmitter's group-level attachment style scaling the
            // spread (anxious groups escalate rumors, avoidant groups suppress
            // them). Fully deterministic (argmax, no RNG): the golden baseline
            // stays byte-identical. Trust matrix read from relationship_v2s;
            // susceptibility from openness (open agents are more receptive),
            // skepticism from the epistemic substrate.
            {
                let n = self.agents.len();
                if n > 0
                    && !self.rumor_registry.rumors.is_empty()
                    && self.rumor_registry.rumors.iter().any(|r| r.active)
                {
                    // §17.4 lazy trust lookup: read trust[listener][source]
                    // straight from the packed per-agent relationship_v2s
                    // instead of allocating an n×n matrix on every daily
                    // pass (O(n²) fill at 12–48 agents, the queue's flagged
                    // O(n²) pass). The diagonal is never read — the source is
                    // always in the rumor's chain, and chain members are
                    // skipped as listeners — so it is ZERO exactly as in the
                    // old matrix. Missing entries (e.g. newborn agents)
                    // degrade to ZERO, matching the old unfilled cells.
                    let agents = &self.agents;
                    let trust = move |listener: usize, source: usize| -> Fixed {
                        if listener == source {
                            Fixed::ZERO
                        } else {
                            agents[listener]
                                .relationship_v2s
                                .get(Self::relationship_v2_pos(listener, source))
                                .map_or(Fixed::ZERO, |rv2| rv2.trust)
                        }
                    };
                    let susceptibility: Vec<Fixed> = self
                        .agents
                        .iter()
                        .map(|a| {
                            (Fixed::from_f64(0.4) + a.personality.openness * Fixed::from_f64(0.6))
                                .clamp_01()
                        })
                        .collect();
                    let skepticism: Vec<Fixed> = self
                        .agents
                        .iter()
                        .map(|a| a.epistemic.trust_network.skepticism)
                        .collect();
                    // §12.3: group-level attachment style escalates/suppresses
                    // rumor spread by the transmitter's groups (factions then
                    // peer groups; the maximum applies for multi-membership).
                    let mut escalation = vec![Fixed::ONE; n];
                    let style_scale =
                        |style: crate::social::group_formation::GroupAttachmentStyle| {
                            match style {
                            crate::social::group_formation::GroupAttachmentStyle::Secure => {
                                Fixed::from_f64(1.0)
                            }
                            crate::social::group_formation::GroupAttachmentStyle::Anxious => {
                                // §12.3: "Anxious groups... escalate rumors."
                                Fixed::from_f64(1.5)
                            }
                            crate::social::group_formation::GroupAttachmentStyle::Avoidant => {
                                // §12.3: avoidant groups "suppress emotion" —
                                // rumors travel slower through them.
                                Fixed::from_f64(0.5)
                            }
                            crate::social::group_formation::GroupAttachmentStyle::Disorganized => {
                                // §12.3 lists rumor escalation under Anxious;
                                // volatile/purge-prone groups get a mild 1.25×
                                // (interpreted from "volatile leadership").
                                Fixed::from_f64(1.25)
                            }
                        }
                        };
                    for faction in &self.faction_v2_registry.factions {
                        if !faction.active {
                            continue;
                        }
                        let scale = style_scale(faction.attachment_style);
                        for &m in &faction.members {
                            if m < n {
                                escalation[m] = escalation[m].max(scale);
                            }
                        }
                    }
                    for group in &self.group_registry.groups {
                        if !group.active {
                            continue;
                        }
                        let scale = style_scale(group.attachment_style);
                        for &m in &group.members {
                            if m < n {
                                escalation[m] = escalation[m].max(scale);
                            }
                        }
                    }
                    self.rumor_registry.transmission_pass_lazy(
                        n,
                        trust,
                        &susceptibility,
                        &skepticism,
                        &escalation,
                        n as u32,
                        tick_u64,
                    );
                }
            }
            // Architecture-plan-2 §13.4: Tick propaganda campaigns daily.
            self.propaganda_registry
                .tick_all(self.params.propaganda_resistance_growth);
            // §13.4 emergent (Iteration 263): institution-owned runtime producer.
            // Prior to this, only the two founding campaigns (seeded at tick 0)
            // existed — the registry's daily loop was live but its input was
            // static, so propaganda died after ~360 ticks and never re-emerged.
            // Each institution with sufficient legitimacy now spawns a fresh
            // campaign when its prior one expires, keeping the live loop fed
            // without perturbing the calibrated window (first spawn at tick 600,
            // after both founders expire, with legitimacy-gated intensity and
            // deterministic narrative selection — no RNG, no stream drift).
            if tick_u64 != 0 && tick_u64.is_multiple_of(600) {
                let n = self.agents.len();
                if n > 0 {
                    let all_targets: Vec<usize> = (0..n).collect();
                    // Council: legitimacy-gated, one active at a time.
                    let council_legit = self
                        .institutions
                        .iter()
                        .find(|i| i.kind == InstitutionKind::Council)
                        .map_or(Fixed::ZERO, |i| i.legitimacy);
                    let council_active = self
                        .propaganda_registry
                        .campaigns
                        .iter()
                        .filter(|c| c.active && c.sponsor == 0)
                        .count();
                    if council_legit > Fixed::from_f64(0.55) && council_active == 0 {
                        let narratives = [
                            "The council secures the harvest",
                            "Trust the council's well decree",
                            "The council feeds the hungry",
                        ];
                        let idx = ((tick_u64 / 600) as usize) % narratives.len();
                        let intensity = (council_legit * Fixed::from_f64(0.6)
                            + Fixed::from_f64(0.2))
                        .clamp_01();
                        self.propaganda_registry
                            .register(crate::culture::PropagandaCampaign::new(
                                0,
                                0,
                                all_targets.clone(),
                                narratives[idx].into(),
                                intensity,
                                vec![crate::culture::PropagandaChannel::Edict],
                                300,
                                tick_u64,
                            ));
                    }
                    // Temple: same gate, slightly lower intensity.
                    let temple_legit = self
                        .institutions
                        .iter()
                        .find(|i| i.kind == InstitutionKind::Temple)
                        .map_or(Fixed::ZERO, |i| i.legitimacy);
                    let temple_active = self
                        .propaganda_registry
                        .campaigns
                        .iter()
                        .filter(|c| c.active && c.sponsor == 1)
                        .count();
                    if temple_legit > Fixed::from_f64(0.55) && temple_active == 0 {
                        let narratives = [
                            "Honor the spirits for rain",
                            "The ancestors watch over the fields",
                            "Piety brings the harvest",
                        ];
                        let idx = ((tick_u64 / 600 + 1) as usize) % narratives.len();
                        let intensity =
                            (temple_legit * Fixed::from_f64(0.5) + Fixed::from_f64(0.2)).clamp_01();
                        self.propaganda_registry
                            .register(crate::culture::PropagandaCampaign::new(
                                0,
                                1,
                                all_targets,
                                narratives[idx].into(),
                                intensity,
                                vec![crate::culture::PropagandaChannel::Sermon],
                                240,
                                tick_u64,
                            ));
                    }
                }
            }
            // Architecture-plan-2 §13.4: Apply propaganda effects to target agents.
            // For each active campaign, compute effectiveness using institutional legitimacy
            // and audience fear, then nudge target agents' emotional state.
            let n_agents = self.agents.len();
            let n_insts = self.institutions.len();
            let campaign_ids: Vec<usize> = self
                .propaganda_registry
                .campaigns
                .iter()
                .filter(|c| c.active)
                .map(|c| c.id)
                .collect();
            for campaign_id in campaign_ids {
                let (sponsor, targets_clone, intensity, coercion) = {
                    let c = &self.propaganda_registry.campaigns[campaign_id];
                    (c.sponsor, c.targets.clone(), c.intensity, c.coercion)
                };
                // Get sponsor institution's legitimacy (0.5 if sponsor out of bounds)
                let legitimacy = if sponsor < n_insts {
                    self.institutions[sponsor].legitimacy
                } else {
                    Fixed::from_f64(0.5)
                };
                // Compute average audience fear from target agents
                let audience_fear = if !targets_clone.is_empty() {
                    let total: Fixed = targets_clone
                        .iter()
                        .filter(|&&t| t < n_agents)
                        .map(|&t| self.agents[t].emotions.fear)
                        .fold(Fixed::ZERO, |acc, f| acc + f);
                    let count = targets_clone.iter().filter(|&&t| t < n_agents).count() as i64;
                    if count > 0 {
                        total / Fixed::from_int(count)
                    } else {
                        Fixed::ZERO
                    }
                } else {
                    Fixed::ZERO
                };
                // Compute effectiveness (mutates the campaign). The global
                // §13.4 effectiveness multiplier is applied here so the knob
                // scales the whole product (identity 1.0 at default).
                if let Some(campaign) = self.propaganda_registry.campaigns.get_mut(campaign_id) {
                    campaign.compute_effectiveness(
                        legitimacy,
                        audience_fear,
                        self.params.propaganda_effectiveness,
                    );
                }
                // Apply propaganda effect to each target agent
                let effectiveness = self.propaganda_registry.campaigns[campaign_id].effectiveness;
                if effectiveness > Fixed::from_f64(0.1) {
                    let propaganda_push = effectiveness * intensity * Fixed::from_f64(0.05);
                    let coercion_push = coercion * Fixed::from_f64(0.03);
                    let mut pushback_accum = Fixed::ZERO;
                    for &target in &targets_clone {
                        if target < n_agents {
                            // Nudge affect toward propaganda's emotional tone
                            self.agents[target].affect.valence =
                                (self.agents[target].affect.valence
                                    + propaganda_push * Fixed::from_f64(0.02))
                                .clamp_01();
                            // Coercion increases fear
                            if coercion > Fixed::from_f64(0.3) {
                                self.agents[target].emotions.fear =
                                    (self.agents[target].emotions.fear + coercion_push).clamp_01();
                                // Agents with high autonomy push back against coercion
                                let autonomy = self.agents[target].personality.dominance
                                    + self.agents[target].personality.openness;
                                if autonomy > Fixed::from_f64(1.0) {
                                    pushback_accum =
                                        (pushback_accum + Fixed::from_f64(0.01)).clamp_01();
                                }
                            }
                        }
                    }
                    // Record accumulated pushback to increase campaign resistance
                    if pushback_accum > Fixed::ZERO {
                        if let Some(c) = self.propaganda_registry.campaigns.get_mut(campaign_id) {
                            c.record_pushback(pushback_accum);
                        }
                    }
                }
            }
            // Architecture-plan-2 §13.5: Decay collective memory daily.
            // §8.1.4 (Iteration 129): a nostalgic population preserves its
            // shared past — the daily fade is scaled by the population mean
            // nostalgia's preservation factor (1 − mean × 0.3, floored at
            // 0.7 — never fully frozen, per the Iter-12 linear-decay
            // design). Deterministic (pure function of the mean, no RNG),
            // and collective-memory salience is NOT part of the golden
            // metric hash nor any snapshot — zero baseline blast by
            // construction, while long-horizon memories fade measurably
            // slower. NOTE (probe-pinned, Iter-129): the daily pass reads
            // the emotion field at a point where nostalgia saturates at
            // 1.0, so the factor pins at exactly the floor 0.7 — a UNIFORM
            // 30% slowdown in every calibrated window (floor-pinned, NOT a
            // differential consumer like gratitude/relief; the
            // near-saturation dead-signal risk flagged in planning
            // applies — the honest value is the baseline
            // memory-preservation effect, and stress/famine worlds where
            // nostalgia drops would show the differential).
            let mean_nostalgia = if self.agents.is_empty() {
                Fixed::ZERO
            } else {
                let sum: Fixed = self
                    .agents
                    .iter()
                    .map(|a| a.emotions.nostalgia)
                    .fold(Fixed::ZERO, |acc, v| acc + v);
                sum / Fixed::from_int(self.agents.len() as i64)
            };
            let preservation = crate::appraisal::nostalgia_preservation_factor(
                mean_nostalgia,
                crate::appraisal::NOSTALGIA_PRESERVATION_RATE,
                crate::appraisal::NOSTALGIA_PRESERVATION_FLOOR,
            );
            self.collective_memory_registry
                .tick_all_preserved(tick_u64, preservation);
            // §13.5: Capture scarcity crises as trauma memories (episode-
            // guarded — one crisis yields one memory).
            self.record_famine_memory(tick_u64);
            // Architecture-plan-2 §13.6: Update echo chamber polarization daily.
            // Assign agents to belief clusters based on dominant belief orientation.
            // Agents with similar beliefs cluster together; cross-cutting ties are
            // social connections between agents in different clusters.
            {
                // Phase 1: Create clusters from agents' dominant belief orientation.
                // Use belief confidence sum as a simple proxy for belief orientation.
                let n = self.agents.len();
                if n >= 2 {
                    // Clear existing clusters and rebuild daily
                    self.echo_chamber.clusters.clear();
                    // Create 2 default clusters (pro-institution vs anti-institution)
                    let c_pro = self.echo_chamber.create_cluster("pro_institution".into());
                    let c_anti = self.echo_chamber.create_cluster("anti_institution".into());
                    // §13.6 (AP2): narrative-dominance → cluster assignment
                    // (remaining-work row 14). When a single narrative truly
                    // dominates the meme pool (dominance > 0.6), the village
                    // tips toward the institutional pole: the momentum bonus
                    // is added to every pro_score, pulling marginal agents
                    // (within 0.1 below the 1.0 boundary at full dominance)
                    // into the pro-institution cluster. The threshold is
                    // anchored ABOVE the probe-pinned calibrated ceiling
                    // (max dominance = 0.52 in every seed × horizon swept),
                    // so in every current calibrated world the momentum is
                    // exactly ZERO and the assignment is byte-identical.
                    let nd_threshold = Fixed::from_f64(
                        crate::culture::echo_chamber::NARRATIVE_DOMINANCE_THRESHOLD,
                    );
                    let nd_rate =
                        Fixed::from_f64(crate::culture::echo_chamber::NARRATIVE_MOMENTUM_RATE);
                    let momentum = self.echo_chamber.narrative_momentum(nd_threshold, nd_rate);
                    for i in 0..n {
                        // Simple heuristic: high traditionalism + high agreeableness = pro-institution
                        let pro_score = self.agents[i].personality.traditionalism
                            + self.agents[i].personality.agreeableness
                            + momentum;
                        let cluster_id = if pro_score > Fixed::from_f64(1.0) {
                            c_pro
                        } else {
                            c_anti
                        };
                        if let Some(cluster) = self.echo_chamber.get_cluster_mut(cluster_id) {
                            cluster.add_member(i);
                            // §13.6 feed: emotional charge comes from the agent's
                            // strongest belief — gossip acceptance and meme
                            // transmission now write real charge into beliefs, so
                            // polarization tracks cultural dynamics instead of
                            // affect.arousal (which stays ~0 in calm villages and
                            // previously pinned polarization at 0.0000 forever).
                            let belief_charge = self.agents[i]
                                .beliefs
                                .iter()
                                .map(|b| b.emotional_charge)
                                .fold(Fixed::ZERO, Fixed::max);
                            // Accumulate WITHOUT clamping: Phase 3 divides by
                            // member count, so an early clamp would saturate the
                            // running sum at 1.0 and understate the per-cluster
                            // average for large clusters (e.g. ~24 members at
                            // MAX_POPULATION).
                            cluster.emotional_charge += belief_charge * Fixed::from_f64(0.5);
                            // Identity fusion from ideological commitment: an
                            // agent's echo-chamber strength and polarization
                            // tendency — the actual drivers §13.6 names — rather
                            // than raw conformity.
                            let fusion_input = self.agents[i].ideology.echo_chamber_strength
                                + self.agents[i].ideology.polarization_tendency;
                            cluster.identity_fusion += fusion_input * Fixed::from_f64(0.5);
                        }
                    }
                    // Phase 2: Compute cross-cutting ties from relationship_v2s.
                    // Pre-compute membership map for O(1) cluster lookup per agent.
                    // An agent has a cross-cutting tie if they have a high-trust relationship
                    // with someone in a different cluster. Each edge counted once (j > i).
                    let membership: Vec<Option<usize>> = (0..n)
                        .map(|i| {
                            self.echo_chamber
                                .clusters
                                .iter()
                                .find(|c| c.members.contains(&i))
                                .map(|c| c.id)
                        })
                        .collect();
                    #[expect(
                        clippy::needless_range_loop,
                        reason = "intentional parallel-array indexing: membership[i/j] + relationship_v2s[i]"
                    )]
                    for i in 0..n {
                        if let Some(my_c) = membership[i] {
                            for j in (i + 1)..n {
                                if let Some(their_c) = membership[j] {
                                    if my_c != their_c {
                                        // Check relationship trust (i→j)
                                        let idx = Self::relationship_v2_pos(i, j);
                                        if idx < self.agents[i].relationship_v2s.len() {
                                            let trust = self.agents[i].relationship_v2s[idx].trust;
                                            if trust > Fixed::from_f64(0.5) {
                                                // Count once per cluster involved
                                                if let Some(c) =
                                                    self.echo_chamber.get_cluster_mut(my_c)
                                                {
                                                    c.cross_cutting_ties =
                                                        c.cross_cutting_ties.saturating_add(1);
                                                }
                                                if let Some(c) =
                                                    self.echo_chamber.get_cluster_mut(their_c)
                                                {
                                                    c.cross_cutting_ties =
                                                        c.cross_cutting_ties.saturating_add(1);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Phase 3: Normalize cluster emotional charge and compute polarization
                    for cluster in &mut self.echo_chamber.clusters {
                        let member_count = cluster.members.len() as i64;
                        if member_count > 0 {
                            cluster.emotional_charge = (cluster.emotional_charge
                                / Fixed::from_int(member_count))
                            .clamp_01();
                            cluster.identity_fusion = (cluster.identity_fusion
                                / Fixed::from_int(member_count))
                            .clamp_01();
                            cluster.outgroup_hostility = cluster.cohesion * cluster.identity_fusion;
                        }
                    }
                    self.echo_chamber.compute_polarization();

                    // Phase 4: §13.6 feedback — echo chambers reinforce beliefs.
                    // A cluster's echo strength (cohesion × fusion × outgroup
                    // hostility / tie penalty) drives daily belief reinforcement:
                    // members of strong echo chambers have their hottest belief
                    // nudged toward certainty and their resistance raised
                    // (closed-mindedness to dissent). This closes the loop that
                    // §13.6 describes — polarized clusters entrench their own
                    // beliefs instead of polarization being a dead-end metric.
                    // Calibrated to the daily cadence (144 ticks/day): a typical
                    // cohesive village has echo ~0.0014 (cohesion 0.5 × fusion
                    // 0.3 × hostility 0.14 × tie penalty 1/15), so × 1.0 yields
                    // ~0.5 confidence/yr on the hot belief — clearly above the
                    // belief_update `confidence × resistance` decay floor
                    // (~0.3-0.4), so the loop observably entrenches beliefs
                    // instead of merely offsetting decay. The original 0.5
                    // coefficient left entrenchment a knife-edge race once
                    // Iteration-8 ritual congregations changed trust/gossip
                    // dynamics (echo test grew 3-5/12, below the >6/12 bar).
                    // Strong realistic chambers (echo ≤ ~0.06, few cross-ties)
                    // entrench to certainty in days; clamps at 1.0 bound it.
                    for cluster in &self.echo_chamber.clusters {
                        let echo = cluster.echo_strength();
                        // Feedback is proportional to echo — no absolute gate.
                        // In a cohesive village echo_strength is small (~0.001:
                        // cohesion 0.5 × fusion 0.3 × hostility 0.14 × tie
                        // penalty 1/15), so an absolute threshold like 0.01
                        // would silence the loop everywhere; scaling keeps the
                        // nudge visible in strong chambers and near-zero in
                        // weak ones.
                        if echo <= Fixed::ZERO {
                            continue;
                        }
                        let reinforcement = echo * Fixed::from_f64(1.0);
                        let resistance_gain = echo * Fixed::from_f64(0.2);
                        let charge_gain = reinforcement * Fixed::from_f64(0.5);
                        // NOTE: cluster membership is personality-driven
                        // (traditionalism + agreeableness, Phase 1), so the
                        // hottest belief reinforced here may not be the
                        // pro/anti-institution stance that defines the cluster.
                        // This approximates §13.6 (chambers entrench beliefs);
                        // aligning reinforcement with the cluster's orientation
                        // belief is future work.
                        for &member in &cluster.members {
                            if member >= self.agents.len() {
                                continue;
                            }
                            let agent = &mut self.agents[member];
                            // Reinforce the agent's hottest (most emotionally
                            // charged) belief — the one the echo chamber amplifies.
                            if let Some(hot) =
                                agent.beliefs.iter_mut().max_by_key(|b| b.emotional_charge)
                            {
                                hot.confidence = (hot.confidence + reinforcement).clamp_01();
                                hot.emotional_charge =
                                    (hot.emotional_charge + charge_gain).clamp_01();
                                hot.resistance = (hot.resistance + resistance_gain).clamp_01();
                            }
                        }
                    }
                }
            }
            // §13: Noosphere belief-ecology projection (daily, after the echo
            // chamber feed) — the zeitgeist amplifies conviction.
            self.tick_noosphere_belief_projection();

            // Architecture-plan-2 §10.8: Clan daily update.
            // Decay grievance, adjust cohesion, decay myth belief — then let
            // the clan's living identity (myths + honor codes) feed cohesion
            // and prestige back.
            self.clan_registry.daily_update();
            self.clan_registry.myth_resonance_day();

            // Architecture-plan-2 §10.4-§10.5: Marriage registry daily update.
            self.marriage_registry.daily_update();
            // §10.4 (AP2): Pair-bond dynamics — charge jealousy from the
            // partners' live emotional + relational state, advance the
            // romantic stage. (Decay runs inside daily_update above; writes
            // go only to `pair_bonds`, which no behavioral system reads, so
            // the golden baseline is untouched.)
            self.tick_pair_bonds(tick_u64);

            // Architecture-plan-2 §12.4: Cult registry daily update + emergence.
            self.cult_registry.daily_update();
            self.tick_cults(tick_u64);
            self.tick_cult_dynamics();

            // Architecture-plan-2 §13.5 (Iteration 115): Moral panic
            // lifecycle — escalation pressure drives daily escalation or
            // fatigue-driven resolution, and an ACTIVE panic erodes its
            // target institution's legitimacy each day ("panic → distrust →
            // more panic"). Previously the registry was unit-test-only dead
            // code: nothing ever registered a panic, so this was a no-op.
            self.tick_moral_panic_lifecycle(tick_u64);

            self.faction_v2_registry.daily_update();
            // Architecture-plan-2 §12.2: Peer group daily update — decay cohesion.
            self.group_registry.daily_update();
            // Architecture-plan-2 §10.4 (Iteration 76): Courtship daily updates.
            // The romantic ladder was dead — try_advance only ever fired from
            // record_positive, and courting pairs sit beyond each other's
            // perception radius, so a courtship formed at Awareness and rotted
            // there. Each daily pass now refreshes the courtship's mutual
            // attraction to the pair's CURRENT average trust (deterministic,
            // no RNG, no events — zero golden impact) and advances the
            // trust-gated ladder one rung per pass, stalling at the pair's
            // trust ceiling. daily_update() is self-contained and short-circuits
            // on !active.
            for courtship in &mut self.active_courtships {
                let i = courtship.pursuer;
                let j = courtship.pursued;
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
                // §10.4 (Iteration 101): the ladder is now attraction-gated
                // like the interaction path — `record_positive` passes the
                // speaker's real `total_attraction()` (familiarity, status,
                // reciprocity, moral disgust, social cost, kinship penalty
                // all fold in), but the daily pass substituted `avg_trust`
                // for the attraction arg, so the 0.6× attraction floor
                // (`min_attraction = next.base_trust() × 0.6`) NEVER bound
                // here — trust was the only real gate and the plan's
                // attraction→courtship consumer stayed dead on the daily
                // path. The pursuer's live total attraction now feeds both
                // the bookkeeping field and the ladder gate: a pair with
                // high trust but low attraction (disgusted, socially
                // costly, kinship-penalized) stalls below the floor instead
                // of climbing. Trust remains the primary gate; attraction
                // binds only below its own 0.6× floor (zero-at-zero: at or
                // above the floor the advance conditions are unchanged).
                let pursuer_attraction = self.agents[i].attraction.total_attraction();
                courtship.mutual_attraction = pursuer_attraction;
                courtship.try_advance(avg_trust, pursuer_attraction);
                courtship.daily_update();
                // Mirror the courtship's reciprocity (which daily_update just
                // decayed) into the pursuer's attraction model, so the mirror
                // decays in lockstep instead of snapping and going stale.
                self.agents[courtship.pursuer].attraction.reciprocity = courtship.reciprocity;
            }
            // §10.4 (Iteration 118): "Seek proximity" — the pursuer
            // actively walks toward the pursued whenever they sit beyond
            // perception radius. Probe-pinned stall (Iter-76 limitation):
            // courtships form without any distance check (trust-gated
            // formation), and pairs beyond radius never interact — the
            // interaction path is radius-gated, so trust never grows and
            // the ladder rots at Awareness (seed 42 golden holds a pair at
            // distance 8 for the whole 1000-tick window; seeds 1/99 the
            // same at 5000). One deterministic Manhattan step per day
            // (no RNG, world-clamped) converges the pair into perception
            // range in ⌈distance − radius⌉ days, after which the normal
            // interaction path takes over. Identity at or within the
            // radius — zero-at-zero for already-proximate pairs.
            for courtship in &self.active_courtships {
                if !courtship.active {
                    continue;
                }
                let i = courtship.pursuer;
                let j = courtship.pursued;
                if i >= self.agents.len() || j >= self.agents.len() {
                    continue;
                }
                let ax = self.agents[i].position.x;
                let ay = self.agents[i].position.y;
                let bx = self.agents[j].position.x;
                let by = self.agents[j].position.y;
                if let Some((dx, dy)) = crate::social::courtship::seek_step(
                    ax,
                    ay,
                    bx,
                    by,
                    crate::social::interaction::DEFAULT_PERCEPTION_RADIUS,
                ) {
                    let w = self.world.width as i32;
                    let h = self.world.height as i32;
                    self.agents[i].position.x = (ax + dx).clamp(0, w - 1);
                    self.agents[i].position.y = (ay + dy).clamp(0, h - 1);
                }
            }
            // Courtships that failed (net-negative at Awareness) free their
            // pursuers' attraction models back to zero reciprocity.
            let mut dead_pursuers: Vec<usize> = Vec::new();
            self.active_courtships.retain(|c| {
                if !c.active {
                    dead_pursuers.push(c.pursuer);
                }
                c.active
            });
            for p in dead_pursuers {
                self.agents[p].attraction.reciprocity = Fixed::ZERO;
            }
        }

        // Architecture-plan-2 §13: Noospheric field — meme-driven spreading
        // activation plus natural decay (every tick).
        self.tick_noosphere();

        // Per-agent daily updates for new systems.
        if phases.is_daily {
            // §10.3 (AP2): Assign kin-branch stages from the kinship graph.
            // The stage tables' contract is "Kin stages are assigned, not
            // advanced" — but nothing ever wired the assignment. This pass
            // assigns ParentChild/Sibling/InLaw from direct kinship links and
            // AncestorDescendant from 2-hop ancestry (grandparent ↔ grandchild),
            // refreshing the derived identity metadata (labels, role
            // expectation, kinship coefficient) for the pairs it touches.
            // Deterministic Vec scans, no RNG; only writes pairs not already
            // at a kin stage, so non-kin behavior is untouched and the golden
            // baseline stays byte-identical (default runs have no kin edges).
            {
                let n_agents = self.agents.len();
                for i in 0..n_agents {
                    for j in 0..n_agents {
                        if i == j {
                            continue;
                        }
                        let rv2_idx = Self::relationship_v2_pos(i, j);
                        if rv2_idx >= self.agents[i].relationship_v2s.len() {
                            continue;
                        }
                        let current = self.agents[i].relationship_v2s[rv2_idx].stage;
                        // Direct kinship link: assign (or confirm) its kin stage.
                        // Iteration 185 (emergent-quality audit — blood-over-
                        // marital precedence): a marital InLaw edge must NOT
                        // shadow a genuine blood chain. A pair can carry both
                        // (a grandparent who married into the family is a
                        // 2-hop blood ancestor AND an in-law via the marriage);
                        // the direct-link branch used to assign InLaw from
                        // whichever edge inserted first, mislabeling blood
                        // kin. Non-marital links (ParentChild, Sibling,
                        // Adoptive, Godparent, OathSibling) commit
                        // immediately; an InLaw link is remembered and only
                        // assigned after the 2-hop ancestry and cousin scans
                        // below find no blood chain. Deterministic, no RNG —
                        // and default golden windows (no kin edges) are
                        // untouched.
                        let mut marital_inlaw = false;
                        if let Some(link) = self.kinship_graph.link_between(i, j) {
                            if let Some(stage) =
                                crate::social::relationship_stages::kin_stage_for_link(link)
                            {
                                if link == crate::social::kinship::KinshipLink::InLaw {
                                    marital_inlaw = true;
                                } else {
                                    if current != stage {
                                        let rv2 = &mut self.agents[i].relationship_v2s[rv2_idx];
                                        rv2.stage = stage;
                                        rv2.update_identity_metadata();
                                    }
                                    // Direct blood/ritual link: commit and move
                                    // on (never falls into the orphaned reset).
                                    continue;
                                }
                            }
                        }
                        // 2-hop ancestry — j is a grandparent of i (or i of j)
                        // when a parent of i is a child of j (or vice versa).
                        let parents_of = |x: usize| -> Vec<usize> {
                            self.kinship_graph
                                .edges
                                .iter()
                                .filter(|e| {
                                    e.link == crate::social::kinship::KinshipLink::ParentChild
                                        && e.to == x
                                })
                                .map(|e| e.from)
                                .collect()
                        };
                        let gps_i = parents_of(i)
                            .iter()
                            .flat_map(|&p| parents_of(p))
                            .collect::<Vec<usize>>();
                        let gps_j = parents_of(j)
                            .iter()
                            .flat_map(|&p| parents_of(p))
                            .collect::<Vec<usize>>();
                        if gps_i.contains(&j) || gps_j.contains(&i) {
                            let rv2 = &mut self.agents[i].relationship_v2s[rv2_idx];
                            rv2.stage = crate::social::relationship_v2::RelationshipStage::
                                AncestorDescendant;
                            rv2.update_identity_metadata();
                            continue;
                        }
                        // First cousins share a grandparent: both are 2 hops
                        // above the same ancestor (both are grandchildren of
                        // the same person). Siblings are caught by the direct-
                        // link branch above, and an uncle/niece pair does NOT
                        // share a grandparent (the uncle's grandparents are
                        // the niece's great-grandparents), so this scan cannot
                        // misfire on the collateral-ascendant direction.
                        if gps_i.iter().any(|&g| gps_j.contains(&g)) {
                            let rv2 = &mut self.agents[i].relationship_v2s[rv2_idx];
                            rv2.stage = crate::social::relationship_v2::RelationshipStage::Cousin;
                            rv2.update_identity_metadata();
                            continue;
                        }
                        // No blood chain: only now does a marital InLaw edge
                        // (or a direct InLaw stage) apply.
                        if marital_inlaw {
                            if current != crate::social::relationship_v2::RelationshipStage::InLaw {
                                let rv2 = &mut self.agents[i].relationship_v2s[rv2_idx];
                                rv2.stage =
                                    crate::social::relationship_v2::RelationshipStage::InLaw;
                                rv2.update_identity_metadata();
                            }
                            continue;
                        }
                        // Orphaned kin stage: the link was removed (death clears
                        // edges at sim.rs ~5312, and the deceased's slot is
                        // replaced in place by a stranger). A terminal kin stage
                        // must not permanently mislabel a stranger — reset to
                        // the undifferentiated default. (Checked AFTER the 2-hop
                        // scan so genuine grandparent chains — which have no
                        // direct link — are not misread as orphaned.)
                        if crate::social::relationship_stages::is_kin_stage(current) {
                            let rv2 = &mut self.agents[i].relationship_v2s[rv2_idx];
                            rv2.stage =
                                crate::social::relationship_v2::RelationshipStage::Unnoticed;
                            rv2.update_identity_metadata();
                        }
                    }
                }
            }
            // Architecture-plan-2 §10.3: Relationship stage transitions.
            // Evaluate whether any RelationshipV2 should advance or regress based on
            // interaction count, trust, and affection. Runs once daily for all pairs.
            let n_agents = self.agents.len();
            for i in 0..n_agents {
                for j in 0..n_agents {
                    if i == j {
                        continue;
                    }
                    let rv2_idx = Self::relationship_v2_pos(i, j);
                    if rv2_idx >= self.agents[i].relationship_v2s.len() {
                        continue;
                    }
                    let current_stage = self.agents[i].relationship_v2s[rv2_idx].stage;
                    // Short-circuit: skip pairs with 0 interactions at entry-level stages.
                    // No advancement is possible without interactions; regression only matters
                    // for Established+ stages or when fear is high.
                    let interactions = self.agents[i].relationship_v2s[rv2_idx].interaction_count;
                    // Kin and authority stages are assigned, not advanced —
                    // skip them outright.
                    if crate::social::relationship_stages::is_kin_stage(current_stage)
                        || crate::social::relationship_stages::is_authority_stage(current_stage)
                    {
                        continue;
                    }
                    if interactions == 0
                        && matches!(
                            current_stage,
                            crate::social::relationship_v2::RelationshipStage::Unnoticed
                                | crate::social::relationship_v2::RelationshipStage::Noticed
                                | crate::social::relationship_v2::RelationshipStage::Disliked
                        )
                    {
                        continue;
                    }
                    let trust = self.agents[i].relationship_v2s[rv2_idx].trust;
                    let affection = self.agents[i].relationship_v2s[rv2_idx].affection;
                    let fear = self.agents[i].relationship_v2s[rv2_idx].fear;
                    // Iteration 201 (write-side closure): produce the
                    // continuous within-stage progress toward the next
                    // stage's thresholds — the previously-dead
                    // `stage_progress` field (never computed, never
                    // consumed, pinned at ZERO). Deterministic, no RNG;
                    // write-only today (the behavioral read is deferred —
                    // the §10.3 ladder is dense in every calibrated
                    // window, so a live consumer would re-pace the
                    // golden, the Iter-199 class).
                    let rv2 = &mut self.agents[i].relationship_v2s[rv2_idx];
                    if let Some(progress) =
                        crate::social::relationship_stages::stage_progress_toward_next(
                            rv2.stage,
                            rv2.interaction_count,
                            rv2.trust,
                            rv2.affection,
                        )
                    {
                        rv2.stage_progress = progress;
                    }
                    // Try advancement first; if not, try regression
                    if let Some(new_stage) = crate::social::relationship_stages::try_advance_stage(
                        current_stage,
                        interactions,
                        trust,
                        affection,
                    ) {
                        self.agents[i].relationship_v2s[rv2_idx].stage = new_stage;
                        // P5 audit (Iteration 184): stages now move daily
                        // (V2 trust/affection are interaction-live), so the
                        // identity metadata must refresh in the same pass —
                        // otherwise public/private labels lag a full day
                        // behind the stage and end-state snapshots catch
                        // the desync. Deterministic, no RNG.
                        self.agents[i].relationship_v2s[rv2_idx].update_identity_metadata();
                    } else if let Some(new_stage) =
                        crate::social::relationship_stages::try_regress_stage(
                            current_stage,
                            trust,
                            fear,
                        )
                    {
                        self.agents[i].relationship_v2s[rv2_idx].stage = new_stage;
                        self.agents[i].relationship_v2s[rv2_idx].update_identity_metadata();
                    }
                }
            }
            // Architecture-plan-2 §10.3: Authority-stage assignment. The
            // authority branch (patron/client, lord/vassal, master/apprentice,
            // priest/layperson, elder/junior, guard/citizen) labels bonds that
            // are *structural* — created by the patronage registry, the
            // apprenticeship learning records, cult leadership, and household
            // headship — rather than grown through social interaction. The
            // pass runs AFTER the transition pass so pairs that can socially
            // progress do so first; only pairs still at the social baseline
            // (Unnoticed/Noticed) receive the structural label. Authority
            // stages are terminal (never advanced) and are written
            // symmetrically, so both directions carry the authority label.
            // LordVassal and GuardCitizen have no live producer yet (reserved
            // taxonomy). Deterministic Vec scans in registry order, no RNG.
            // Note: the pass runs before run_apprenticeship_pass, so learning
            // events created by today's apprenticeship pass are labeled on the
            // NEXT daily tick (a self-healing one-tick lag). When a pair
            // belongs to multiple producers, the last-applying producer wins
            // (household > cult > apprenticeship > patronage).
            {
                let mut authority_pairs: Vec<(
                    usize,
                    usize,
                    crate::social::relationship_v2::RelationshipStage,
                )> = Vec::new();
                // Patronage: patron ↔ client.
                for rel in &self.patronage_registry.relations {
                    let p = rel.patron;
                    let c = rel.client;
                    authority_pairs.push((
                        p,
                        c,
                        crate::social::relationship_v2::RelationshipStage::PatronClient,
                    ));
                    authority_pairs.push((
                        c,
                        p,
                        crate::social::relationship_v2::RelationshipStage::PatronClient,
                    ));
                }
                // Apprenticeship: each learning event labels teacher ↔ student.
                for agent in &self.agents {
                    for ev in &agent.education.learning_events {
                        let t = ev.teacher;
                        let s = ev.student;
                        authority_pairs.push((
                            t,
                            s,
                            crate::social::relationship_v2::RelationshipStage::MasterApprentice,
                        ));
                        authority_pairs.push((
                            s,
                            t,
                            crate::social::relationship_v2::RelationshipStage::MasterApprentice,
                        ));
                    }
                }
                // Cults: charismatic leader ↔ members.
                for cult in &self.cult_registry.cults {
                    let l = cult.charismatic_leader;
                    for &m in &cult.members {
                        authority_pairs.push((
                            l,
                            m,
                            crate::social::relationship_v2::RelationshipStage::PriestLayperson,
                        ));
                        authority_pairs.push((
                            m,
                            l,
                            crate::social::relationship_v2::RelationshipStage::PriestLayperson,
                        ));
                    }
                }
                // Households: head ↔ members.
                for hh in &self.households {
                    if let Some(head) = hh.head {
                        for &m in &hh.members {
                            if m == head {
                                continue;
                            }
                            authority_pairs.push((
                                head,
                                m,
                                crate::social::relationship_v2::RelationshipStage::ElderJunior,
                            ));
                            authority_pairs.push((
                                m,
                                head,
                                crate::social::relationship_v2::RelationshipStage::ElderJunior,
                            ));
                        }
                    }
                }
                // Apply: baseline gate, then write the authority label.
                for &(a, b, stage) in &authority_pairs {
                    if a >= self.agents.len() || b >= self.agents.len() {
                        continue;
                    }
                    let rv2_idx = Self::relationship_v2_pos(a, b);
                    if rv2_idx >= self.agents[a].relationship_v2s.len() {
                        continue;
                    }
                    let rv2 = &mut self.agents[a].relationship_v2s[rv2_idx];
                    // Only label pairs that have not already socially progressed.
                    if !matches!(
                        rv2.stage,
                        crate::social::relationship_v2::RelationshipStage::Unnoticed
                            | crate::social::relationship_v2::RelationshipStage::Noticed
                    ) {
                        continue;
                    }
                    rv2.stage = stage;
                    rv2.update_identity_metadata();
                }
                // Orphaned authority stage: the producer disappeared without a
                // death-path rv2 rebuild. (Death itself fully rebuilds v2s in
                // rebuild_relationship_v2s_after_death, but a cult disbanding,
                // household dissolution, or registry cleanup leaves the
                // terminal authority label behind — and since the transition
                // pass skips authority stages, nothing else would ever reset
                // it.) Mirror the kin-stage orphan reset: any authority label
                // whose pair no longer has a live producer returns to the
                // social baseline.
                let produced: std::collections::BTreeSet<(usize, usize)> =
                    authority_pairs.iter().map(|&(a, b, _)| (a, b)).collect();
                for i in 0..self.agents.len() {
                    let mut reset: Vec<usize> = Vec::new();
                    for (idx, rv2) in self.agents[i].relationship_v2s.iter().enumerate() {
                        if crate::social::relationship_stages::is_authority_stage(rv2.stage)
                            && !produced.contains(&(i, rv2.to.as_u64() as usize))
                        {
                            reset.push(idx);
                        }
                    }
                    for idx in reset {
                        let rv2 = &mut self.agents[i].relationship_v2s[idx];
                        rv2.stage = crate::social::relationship_v2::RelationshipStage::Unnoticed;
                        rv2.update_identity_metadata();
                    }
                }
            }
            for agent in &mut self.agents {
                // §8.1.17: Narrative frame update — resilience factor modulates
                // how narrative meaning-making buffers against adversity.
                // (Computed but applied through narrative identity system above.)
                // §8.1.18: Education daily update
                agent.education.daily_update();
            }
            // §8.1.18: Apprenticeship pass — deliberate knowledge transmission.
            // The education module (attempt_teaching/record_learning) was fully
            // implemented but never called from the tick loop; this pass makes
            // adults teach knowledge they hold to agents who lack it, through
            // the teaching-skill × relationship-quality pipeline.
            self.run_apprenticeship_pass(tick_u64, Tick::new(tick_u64));
        }

        // Architecture-plan-2 §12.5: Execute due rituals every 12 ticks (~2 hours).
        // Apply bonding effect to participating agents' relationship_v2s.
        if phases.is_duodeca {
            let due: Vec<usize> = self
                .ritual_registry
                .due_rituals(tick_u64)
                .into_iter()
                .map(|r| r.id)
                .collect();
            let ritual_fired = !due.is_empty();
            for ritual_id in due {
                if let Some(ritual) = self
                    .ritual_registry
                    .rituals
                    .iter_mut()
                    .find(|r| r.id == ritual_id)
                {
                    let bonding = ritual.execute(tick_u64);
                    // §11.1: Ritual participation builds perceived legitimacy —
                    // communal ritual rehearses the rightfulness of the order.
                    // Zero-at-zero anchor: no ritual -> no boost.
                    let ritual_legitimacy = bonding;
                    // Apply bonding to all participant pairs
                    for i in 0..ritual.participants.len() {
                        // Boost each participant's perceived legitimacy once.
                        let p = ritual.participants[i];
                        if p < self.agents.len() {
                            self.agents[p]
                                .legitimacy_field
                                .ritual_boost(ritual_legitimacy);
                            // §7.2.2 (S2-2-2 fix): ritual participation feeds
                            // the endocrine bonding axis — the plan's "ritual
                            // increases bonding" channel. Previously
                            // write-once: `bonding.update` had zero call sites
                            // (the axis sat frozen at birth values in every
                            // probe window). Input scaled by 0.5 (mirroring
                            // the trust channel's `bonding × 0.3` in this
                            // loop): the seasonal effect 0.225 × 0.5 ×
                            // receptivity (mean ~0.5) with the 0.02 recovery
                            // moves the axis ≈ +0.04/fire — differentiated
                            // across pro/anti congregations. Saturation is
                            // bounded by the axis's own logistic (1 − level)
                            // gain (equilibrium ≈ 0.78 at max receptivity,
                            // probe-verified across 5K/20K/50K horizons).
                            // Rituals fire only at monthly intervals (4320),
                            // beyond every golden (1000) / snapshot (≤2000)
                            // horizon, so calibrated runs stay byte-identical.
                            self.agents[p].embodied.endocrine.bonding.update(
                                bonding * Fixed::from_f64(0.5),
                                self.params.endocrine_bonding_recovery,
                            );
                            // §8.1.3: Cultural memory — shared ritual
                            // participation is the canonical cultural episode
                            // (sparse: rituals fire on their interval; salience
                            // follows the ritual's intensity and sacredness).
                            let participant = &mut self.agents[p];
                            if participant.agent_tier.tier.runs_memory_encoding()
                                && participant.agent_tier.budget_tracker.can_memory_op()
                            {
                                let _ = participant.agent_tier.budget_tracker.consume_memory_op();
                                let salience = ((ritual.emotional_intensity + ritual.sacredness)
                                    * Fixed::from_f64(0.8))
                                .clamp_01();
                                let emotional = participant.affect.arousal * Fixed::from_f64(0.6)
                                    + Fixed::from_f64(0.1);
                                participant.memory.encode(
                                    MemoryKind::Cultural,
                                    tick_u64,
                                    salience,
                                    emotional,
                                    None,
                                    MemoryTag::RitualParticipated,
                                );
                            }
                            // §12.5: Rituals "reinforce norms" — each
                            // participant internalizes (or strengthens) the
                            // community's registry norms, scaled by the norm's
                            // community internalization. Deterministic (no RNG)
                            // and observationally isolated (norm_resistance has
                            // no production consumer), so the golden baseline
                            // stays byte-identical: rituals fire only at
                            // monthly intervals (4320 ticks) while every
                            // snapshot/golden horizon is ≤ 2000.
                            //
                            // §19.5.D (Iteration 90): the sponsor
                            // institution's declared norms (`Institution.
                            // norm_ids` — the temple declares "Obey Ruler" = 3)
                            // are reinforced preferentially at its ritual; the
                            // field's documented purpose ("Obey Ruler norm
                            // reinforced by temple") is now honored. A missing
                            // sponsor or empty declaration keeps the legacy
                            // all-equal reinforcement. The declared norm has no
                            // behavioral consumer, so this only changes the
                            // growth rate of observational strength.
                            let sponsor_norms: std::collections::BTreeSet<u64> = self
                                .institutions
                                .get(ritual.sponsor)
                                .map_or_else(std::collections::BTreeSet::new, |inst| {
                                    inst.norm_ids.iter().copied().collect()
                                });
                            // §8.1.10 (P3-11): per-agent internalization
                            // scaling. The norm count was UNIFORM (5.000 for
                            // 12/12 at 10K, zero spread) because every
                            // participant internalized every community norm at
                            // the identical rate — conformity had no seat at
                            // the ritual. Scale the reinforcement by the
                            // agent's conformity (0.5 + conformity × 0.5, so
                            // a maximally conformist agent internalizes at
                            // full strength and a maximally independent one at
                            // half): the collective ritual still internalizes
                            // norms, but per-agent strength (and the
                            // 0.1-strength first-exposure threshold in
                            // `reinforce_norm`'s internalize path) now
                            // differentiates the population.
                            let conformity = self.agents[p].personality.conformity;
                            let internalize_scale =
                                Fixed::from_f64(0.5) + conformity * Fixed::from_f64(0.5);
                            for norm in self.norms.norms() {
                                let reinforcement = ritual.norm_reinforcement_for_institutional(
                                    norm.internalization,
                                    sponsor_norms.contains(&norm.id),
                                );
                                let scaled = (reinforcement * internalize_scale).clamp_01();
                                if scaled > Fixed::ZERO {
                                    self.agents[p]
                                        .moral_cognition
                                        .reinforce_norm(&norm.name, scaled);
                                }
                            }
                        }
                        for j in (i + 1)..ritual.participants.len() {
                            let a = ritual.participants[i];
                            let b = ritual.participants[j];
                            if a < self.agents.len() && b < self.agents.len() {
                                // Increase trust and affection between participants
                                let idx_a = Self::relationship_v2_pos(a, b);
                                if idx_a < self.agents[a].relationship_v2s.len() {
                                    self.agents[a].relationship_v2s[idx_a].trust =
                                        (self.agents[a].relationship_v2s[idx_a].trust
                                            + bonding * Fixed::from_f64(0.3))
                                        .clamp_01();
                                    self.agents[a].relationship_v2s[idx_a].affection =
                                        (self.agents[a].relationship_v2s[idx_a].affection
                                            + bonding * Fixed::from_f64(0.2))
                                        .clamp_01();
                                }
                                let idx_b = Self::relationship_v2_pos(b, a);
                                if idx_b < self.agents[b].relationship_v2s.len() {
                                    self.agents[b].relationship_v2s[idx_b].trust =
                                        (self.agents[b].relationship_v2s[idx_b].trust
                                            + bonding * Fixed::from_f64(0.3))
                                        .clamp_01();
                                    self.agents[b].relationship_v2s[idx_b].affection =
                                        (self.agents[b].relationship_v2s[idx_b].affection
                                            + bonding * Fixed::from_f64(0.2))
                                        .clamp_01();
                                }
                            }
                        }
                    }
                }
            }

            // §13.5: Rituals rehearse the village's collective memory — ritual
            // repetition is the memory-maintenance mechanism. Without this the
            // seeded memories only ever decayed (salience → 0 within ~2 weeks;
            // the daily decay previously accumulated quadratically, fixed in
            // Iteration 12). Monthly rehearsal (+0.05) outpaces daily decay
            // (0.001/day), so memories stay vivid while rituals are held.
            if ritual_fired {
                self.collective_memory_registry
                    .get_or_create(0)
                    .rehearse_all(tick_u64);
            }

            // §13.5 (AP2): Refresh the derived plan fields (traumas,
            // sacred_events) from the shared-memory log. Deterministic — reads
            // only memories, writes only the new fields, so the golden baseline
            // stays byte-identical.
            for cm in &mut self.collective_memory_registry.entries {
                cm.refresh_derived_views();
            }

            // Architecture-plan-2 §12.2: Evaluate group formation pressure.
            // After ritual bonding, check if any agent clusters have sufficient
            // shared identity, repeated interaction, or external threat to form a
            // formal group. Gated by is_duodeca to limit O(N²) overhead.
            let n_agents = self.agents.len();
            // Pre-compute institution-level values once (not per-agent).
            // social_cost: higher membership density → higher cost to break away.
            let social_cost = {
                let total_members: usize = self
                    .institutions
                    .iter()
                    .map(|inst| inst.members.len())
                    .sum();
                let density = if self.agents.is_empty() {
                    Fixed::ZERO
                } else {
                    Fixed::from_f64(total_members as f64 / self.agents.len() as f64).clamp_01()
                };
                density * GROUP_SOCIAL_COST_SCALE
            };
            // institutional_suppression: high council legitimacy → institutions suppress groups.
            let institutional_suppression = self
                .institutions
                .iter()
                .find_map(|inst| {
                    (inst.kind == institutions::InstitutionKind::Council).then_some(inst.legitimacy)
                })
                .unwrap_or(GROUP_COUNCIL_LEGITIMACY_DEFAULT)
                * GROUP_SUPPRESSION_SCALE;
            for i in 0..n_agents {
                // Find peers: agents with high mutual trust and shared location
                let mut peers: Vec<usize> = Vec::new();
                for j in 0..n_agents {
                    if i == j {
                        continue;
                    }
                    let rv2_idx = Self::relationship_v2_pos(i, j);
                    if rv2_idx < self.agents[i].relationship_v2s.len()
                        && self.agents[i].relationship_v2s[rv2_idx].trust > Fixed::from_f64(0.5)
                        && self.agents[i].position == self.agents[j].position
                    {
                        peers.push(j);
                    }
                }
                // Need at least 2 peers (3 total including i) for group formation
                if peers.len() < 2 {
                    continue;
                }
                // Check shared identity from relationship_v2 solidarity
                let shared_identity: Fixed = peers
                    .iter()
                    .map(|&j| {
                        let rv2_idx = Self::relationship_v2_pos(i, j);
                        if rv2_idx < self.agents[i].relationship_v2s.len() {
                            self.agents[i].relationship_v2s[rv2_idx].solidarity
                        } else {
                            Fixed::ZERO
                        }
                    })
                    .fold(Fixed::ZERO, |acc, s| acc + s)
                    / Fixed::from_int(peers.len() as i64);
                // Shared grievance from derived mental state
                let shared_grievance: Fixed = peers
                    .iter()
                    .map(|&j| self.agents[j].derived.resentment)
                    .fold(Fixed::ZERO, |acc, r| acc + r)
                    / Fixed::from_int(peers.len() as i64);
                // Compute emotional synchrony from mean valence similarity.
                // Valence = (joy + trust + pride) - (fear + anger + sadness + guilt + shame).
                let emotional_synchrony: Fixed = peers
                    .iter()
                    .map(|&j| {
                        let vi = (self.agents[i].emotions.joy
                            + self.agents[i].emotions.trust
                            + self.agents[i].emotions.pride)
                            - (self.agents[i].emotions.fear
                                + self.agents[i].emotions.anger
                                + self.agents[i].emotions.sadness
                                + self.agents[i].emotions.guilt
                                + self.agents[i].emotions.shame);
                        let vj = (self.agents[j].emotions.joy
                            + self.agents[j].emotions.trust
                            + self.agents[j].emotions.pride)
                            - (self.agents[j].emotions.fear
                                + self.agents[j].emotions.anger
                                + self.agents[j].emotions.sadness
                                + self.agents[j].emotions.guilt
                                + self.agents[j].emotions.shame);
                        let diff = (vi - vj).abs();
                        (Fixed::ONE - diff).clamp_01()
                    })
                    .fold(Fixed::ZERO, |acc, s| acc + s)
                    / Fixed::from_int(peers.len() as i64);
                // Count recent interactions as proxy for repeated interaction
                let repeated_interaction: Fixed = peers
                    .iter()
                    .map(|&j| {
                        let rv2_idx = Self::relationship_v2_pos(i, j);
                        if rv2_idx < self.agents[i].relationship_v2s.len() {
                            let ic = self.agents[i].relationship_v2s[rv2_idx].interaction_count;
                            // Normalize: 10+ interactions → ~1.0
                            Fixed::from_f64(ic as f64 / 10.0).clamp_01()
                        } else {
                            Fixed::ZERO
                        }
                    })
                    .fold(Fixed::ZERO, |acc, r| acc + r)
                    / Fixed::from_int(peers.len() as i64);
                // §12.2 (AP2, Iteration 121): shared trauma — the mean of the
                // candidate's bodily trauma load (self + peers). The plan's
                // "bonding after shared trauma": trauma feeds the formation
                // pressure like an experienced grievance.
                let members: Vec<usize> = {
                    let mut m = vec![i];
                    m.extend_from_slice(&peers);
                    m
                };
                let shared_trauma: Fixed = members
                    .iter()
                    .map(|&m| self.agents[m].embodied.nervous.trauma_load)
                    .fold(Fixed::ZERO, |acc, t| acc + t)
                    / Fixed::from_int(members.len() as i64);
                let candidate = crate::social::group_formation::GroupCandidate {
                    members,
                    shared_grievance,
                    shared_identity,
                    emotional_synchrony,
                    repeated_interaction,
                    leadership_gravity: Fixed::ZERO,
                    external_threat: Fixed::ZERO,
                    social_cost,
                    institutional_suppression,
                    shared_trauma,
                    identified_tick: tick_u64,
                };
                if candidate.should_form() {
                    // Architecture-plan-2 §12.2: Register the formed peer group.
                    let mut group = crate::social::group_formation::PeerGroup::from_candidate(
                        &candidate,
                        self.group_registry.groups.len(),
                        tick_u64,
                    );
                    // §12.3: individual attachment styles scale upward — the
                    // group takes the modal member style, which drives its
                    // cohesion dynamics.
                    let member_styles: Vec<_> = candidate
                        .members
                        .iter()
                        .map(|&m| self.agents[m].attachment.style)
                        .collect();
                    group.attachment_style =
                        crate::social::group_formation::derive_group_attachment_style(
                            &member_styles,
                        );
                    let group_id = self.group_registry.register(group);
                    tracing::info!(
                        group_id = group_id,
                        members = candidate.members.len(),
                        shared_grievance = shared_grievance.to_f64(),
                        shared_identity = shared_identity.to_f64(),
                        leader_index = i,
                        tick = tick_u64,
                        "Peer group formed and registered"
                    );
                }
            }
        }
    }
}
