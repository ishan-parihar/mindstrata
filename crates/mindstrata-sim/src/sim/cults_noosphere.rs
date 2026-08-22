//! Cults, meme aggregation and noosphere dynamics.

use super::{Fixed, Simulation};

impl Simulation {
    /// §12.4: Daily cult emergence and dissolution.
    ///
    /// Previously the cult registry was constructed but never written —
    /// `register()` had zero call sites, so the §12.4 system was dead.
    /// Emergence is deterministic (trait-ordered selection, no RNG) so fresh
    /// and replayed runs match: gated by low institutional legitimacy and a
    /// meaning deficit among agents, with a formation cooldown.
    pub(super) fn tick_cults(&mut self, tick: u64) {
        // ── Dissolution ──
        let n_inst = self.institutions.len().max(1);
        let legitimacy = self
            .institutions
            .iter()
            .map(|i| i.legitimacy)
            .fold(Fixed::ZERO, |acc, l| acc + l)
            * Fixed::from_f64(1.0 / n_inst as f64);
        let n_agents = self.agents.len().max(1);
        let avg_fatigue = self
            .agents
            .iter()
            .map(|a| a.body.fatigue)
            .fold(Fixed::ZERO, |acc, f| acc + f)
            * Fixed::from_f64(1.0 / n_agents as f64);
        // Dissolution carries member fallout: when a cult dissolves, its
        // former members lose the belonging that was satisfying their meaning
        // need (the deficit rebounds into a crisis) and their cult-entrenched
        // beliefs partially decay. This completes the §12.4 lifecycle:
        // formation → recruitment → entrenchment → dissolution → fallout.
        // Economic stress — the same scarcity proxy the motivation system uses
        // (world totals vs expected per-capita rations, ~line 2176).
        let expected_food = crate::sim::EXPECTED_GRAIN_PER_AGENT as i64 * self.agents.len() as i64;
        let expected_water = crate::sim::EXPECTED_WATER_PER_AGENT as i64 * self.agents.len() as i64;
        let food_scarcity = (Fixed::ONE
            - self.world.total_food() / Fixed::from_int(expected_food.max(1)))
        .clamp_01();
        let water_scarcity = (Fixed::ONE
            - self.world.total_water() / Fixed::from_int(expected_water.max(1)))
        .clamp_01();
        let economic_stress = food_scarcity.max(water_scarcity);

        let mut former_members: Vec<usize> = Vec::new();
        for cult in &mut self.cult_registry.cults {
            // §12.4: compute the full dissolution signal set. Leader competence
            // is proxied by the leader's conscientiousness (no dedicated
            // competence model); prophecy disconfirmation fires when the
            // sacred meme's credibility collapses or the meme vanishes;
            // internal betrayal fires when most members' meaning need is
            // satisfied (belonging no longer binds them) — note `needs.meaning`
            // is a DEFICIT scale: HIGH = craving, LOW = satisfied = betrayal
            // risk; rival narrative pressure is the excess emotional charge of
            // the hottest competing meme over the sacred one.
            let leader_competence = self
                .agents
                .get(cult.charismatic_leader)
                .map_or(Fixed::from_f64(0.5), |a| a.personality.conscientiousness);
            let prophecy_disconfirmed = self
                .meme_registry
                .memes
                .iter()
                .find(|m| m.id == cult.sacred_narrative_id as usize)
                .is_none_or(|m| !m.active || m.credibility < Fixed::from_f64(0.3));
            let member_count = cult.members.len().max(1);
            let satisfied = cult
                .members
                .iter()
                .filter(|&&m| {
                    self.agents
                        .get(m)
                        .is_some_and(|a| a.needs.meaning <= Fixed::from_f64(0.2))
                })
                .count();
            let internal_betrayal = satisfied > member_count / 2;
            let sacred_charge = self
                .meme_registry
                .memes
                .iter()
                .find(|m| m.id == cult.sacred_narrative_id as usize)
                .map_or(Fixed::ZERO, |m| m.emotional_charge);
            let rival_charge = self
                .meme_registry
                .memes
                .iter()
                .filter(|m| m.active && m.id != cult.sacred_narrative_id as usize)
                .map(|m| m.emotional_charge)
                .fold(Fixed::ZERO, Fixed::max);
            let rival_narrative_pressure = (rival_charge - sacred_charge).clamp_01();
            let signals = crate::social::cult::CultDissolutionSignals {
                leader_competence,
                institutional_legitimacy: legitimacy,
                member_average_fatigue: avg_fatigue,
                cult_age_ticks: tick.saturating_sub(cult.formed_tick),
                prophecy_disconfirmed,
                internal_betrayal,
                rival_narrative_pressure,
                economic_stress,
            };
            if cult.active && cult.should_dissolve(&signals) {
                cult.active = false;
                former_members.extend(
                    std::iter::once(&cult.charismatic_leader)
                        .chain(cult.members.iter())
                        .copied(),
                );
            }
        }
        for i in former_members {
            if i >= self.agents.len() {
                continue;
            }
            let agent = &mut self.agents[i];
            // Meaning crisis rebound: belonging was fulfilling the need.
            agent.needs.meaning =
                (agent.needs.meaning * Fixed::from_f64(0.5) + Fixed::from_f64(0.5)).clamp_01();
            // Cult-entrenched beliefs decay once the narrative grip is gone.
            if let Some(hot) = agent.beliefs.iter_mut().max_by_key(|b| b.emotional_charge) {
                hot.confidence = (hot.confidence - Fixed::from_f64(0.05)).max(Fixed::ZERO);
                hot.emotional_charge =
                    (hot.emotional_charge - Fixed::from_f64(0.05)).max(Fixed::ZERO);
            }
        }

        // ── Emergence ──
        const CULT_COOLDOWN: u64 = 2880; // ~20 days at 144 ticks/day
        const CULT_FORM_MEMBERS: usize = 4; // initial core (leader + up to 4)
        if tick.saturating_sub(self.last_cult_formation_tick) < CULT_COOLDOWN {
            return;
        }
        if legitimacy > Fixed::from_f64(0.45) {
            return; // institutions too strong — no fertile ground
        }
        // Candidates: agents suffering a meaning deficit.
        let candidates: Vec<usize> = (0..self.agents.len())
            .filter(|&i| self.agents[i].needs.meaning > Fixed::from_f64(0.3))
            .collect();
        if candidates.len() < 3 {
            return;
        }
        // Charismatic leader: highest extraversion + agreeableness among the
        // meaning-starved (deterministic: agent order is identical in fresh
        // and replayed runs).
        let Some(leader) = candidates.iter().copied().max_by(|&a, &b| {
            let sa =
                self.agents[a].personality.extraversion + self.agents[a].personality.agreeableness;
            let sb =
                self.agents[b].personality.extraversion + self.agents[b].personality.agreeableness;
            sa.cmp(&sb)
        }) else {
            return; // unreachable: len >= 3 guard above
        };
        // Members: other meaning-deficit agents, capped for cost.
        let members: Vec<usize> = candidates
            .iter()
            .copied()
            .filter(|&i| i != leader)
            .take(CULT_FORM_MEMBERS)
            .collect();
        if members.len() < 2 {
            return;
        }
        // Sacred narrative: the hottest active meme.
        let sacred_id = self
            .meme_registry
            .memes
            .iter()
            .filter(|m| m.active)
            .max_by_key(|m| m.emotional_charge)
            .map_or(0, |m| m.id as u32);
        let mut cult = crate::social::cult::CultDynamics::new(leader, sacred_id, tick);
        cult.members = members;
        self.cult_registry.register(cult);
        self.last_cult_formation_tick = tick;
    }

    /// §12.4: Daily cult member dynamics — recruitment and psychological feedback.
    ///
    /// Active cults recruit new meaning-starved agents (up to a membership
    /// cap) and entrench their members: belonging satisfies the meaning need
    /// (deficit shrinks), the hottest belief gains confidence/charge (cult
    /// narrative grip), and identity fusion / fear-load are reinforced against
    /// the daily decay in `CultDynamics::daily_update`. This turns the
    /// previously observational cult registry (Iteration 9) into an emergent
    /// force on member psychology. Fully deterministic (agent order, no RNG)
    /// so fresh and replayed runs match.
    pub(super) fn tick_cult_dynamics(&mut self) {
        let n = self.agents.len();
        let member_cap = (n / 2).max(4);

        // ── Recruitment ──
        // Collect (cult_idx, agent_idx) targets under an immutable borrow of
        // the registry, then apply them to avoid borrow conflicts.
        let mut recruits: Vec<(usize, usize)> = Vec::new();
        let active: Vec<usize> = (0..self.cult_registry.cults.len())
            .filter(|&ci| self.cult_registry.cults[ci].active)
            .collect();
        for &ci in &active {
            let cult = &self.cult_registry.cults[ci];
            let mut slots = member_cap.saturating_sub(1 + cult.members.len());
            if slots == 0 {
                continue;
            }
            for i in 0..n {
                if slots == 0 {
                    break;
                }
                if self.agents[i].needs.meaning <= Fixed::from_f64(0.3) {
                    continue;
                }
                if i == cult.charismatic_leader || cult.members.contains(&i) {
                    continue;
                }
                let already_in = self
                    .cult_registry
                    .cults
                    .iter()
                    .any(|c| c.active && c.members.contains(&i));
                if already_in {
                    continue;
                }
                recruits.push((ci, i));
                slots -= 1;
            }
        }
        for (ci, i) in recruits {
            self.cult_registry.cults[ci].add_member(i);
        }

        // ── Member psychological feedback ──
        // Snapshot (leader, members) per active cult, then apply to agents.
        let member_sets: Vec<(usize, Vec<usize>)> = self
            .cult_registry
            .cults
            .iter()
            .filter(|c| c.active)
            .map(|c| (c.charismatic_leader, c.members.clone()))
            .collect();
        for (leader, members) in member_sets {
            for &i in std::iter::once(&leader).chain(members.iter()) {
                if i >= n {
                    continue;
                }
                let agent = &mut self.agents[i];
                // Belonging satisfies the meaning need (deficit shrinks).
                agent.needs.meaning = (agent.needs.meaning * Fixed::from_f64(0.98)
                    - Fixed::from_f64(0.01))
                .max(Fixed::ZERO);
                // Entrench the hottest belief — the cult's narrative grip.
                if let Some(hot) = agent.beliefs.iter_mut().max_by_key(|b| b.emotional_charge) {
                    hot.confidence = (hot.confidence + Fixed::from_f64(0.01)).clamp_01();
                    hot.emotional_charge =
                        (hot.emotional_charge + Fixed::from_f64(0.01)).clamp_01();
                }
            }
            // Cult-level reinforcement against the daily_update decay.
            // The leader index uniquely identifies an active cult here.
            if let Some(cult) = self
                .cult_registry
                .cults
                .iter_mut()
                .find(|c| c.active && c.charismatic_leader == leader)
            {
                cult.identity_fusion = (cult.identity_fusion + Fixed::from_f64(0.004)).clamp_01();
                cult.fear_load = (cult.fear_load + Fixed::from_f64(0.002)).clamp_01();
            }
        }
    }

    /// §13: Noospheric field update (every tick) — refresh node metadata from
    /// the live meme registry, feed meme emotional charge as spreading
    /// activation, then apply natural decay (the pre-existing behavior).
    pub(super) fn tick_noosphere(&mut self) {
        for meme in &self.meme_registry.memes {
            let node_id = meme.id as u32;
            if let Some(node) = self
                .noospheric_field
                .nodes
                .iter_mut()
                .find(|n| n.id == node_id)
            {
                node.emotional_charge = meme.emotional_charge;
                node.identity_relevance = meme.identity_relevance;
                node.institutional_backing = meme.credibility;
            }
            self.noospheric_field
                .spread_activation(node_id, meme.emotional_charge * Fixed::from_f64(0.05));
        }
        // Natural decay — preserved baseline behavior.
        self.noospheric_field.decay_all(Fixed::from_f64(0.001));
    }

    /// §13: Daily belief-ecology projection — the noosphere feeds back into
    /// individual psychology, closing the loop that previously ran one-way
    /// (memes/beliefs → nodes only). The field's most-activated node is the
    /// dominant symbolic mood (the "zeitgeist"); project it onto every
    /// agent's hottest belief as a small, capped emotional-charge boost so
    /// the collective atmosphere amplifies conviction. Structurally distinct
    /// from the §13.6 echo chamber: that is per-cluster (who you are tied
    /// to); this is global (the mood of the whole field). Deterministic.
    pub(super) fn tick_noosphere_belief_projection(&mut self) {
        let zeitgeist = self
            .noospheric_field
            .nodes
            .iter()
            .map(crate::noosphere::field::SymbolicNode::effective_activation)
            .fold(Fixed::ZERO, std::cmp::Ord::max);
        if zeitgeist <= Fixed::from_f64(0.4) {
            return; // dormant field — no atmospheric pressure
        }
        let boost = zeitgeist * Fixed::from_f64(0.0005);
        for agent in &mut self.agents {
            if let Some(hot) = agent.beliefs.iter_mut().max_by_key(|b| b.emotional_charge) {
                hot.emotional_charge = (hot.emotional_charge + boost).clamp_01();
            }
        }
    }

    /// §17.4: Wire meme aggregation into the tick loop.
    ///
    /// Called daily after meme novelty decay. Computes group-level meme
    /// prevalence, most viral meme, spreader centrality, and echo chamber
    /// strength for observability and Phase 5 tuning.
    ///
    /// **Limitation**: Meme host assignment is a placeholder heuristic based
    /// on cultural category indices. Aggregation metrics (prevalence,
    /// homogeneity, echo strength) are deterministic artifacts of initial
    /// category assignment, not emergent from social dynamics. Replace with
    /// real per-agent meme host tracking (`hosted_memes: Vec<usize>` field)
    /// before using these metrics for Phase 5 tuning decisions.
    ///
    pub(super) fn wire_meme_aggregation(&mut self, tick_u64: u64) {
        let n_agents = self.agents.len();
        if n_agents == 0 || self.meme_registry.memes.is_empty() {
            return;
        }

        // §17.4: Flush accumulated exposure samples into the aggregator. This
        // stays DAILY — the tick loop pushes samples every tick, and dropping
        // them would lose the observability data.
        let samples = std::mem::take(&mut self.exposure_samples);
        self.meme_aggregator.set_exposure_samples(samples);

        // §17.4 lazy aggregation: the input builds below are O(agents ×
        // categories) + O(agents × households) — rebuilding them every day
        // when `aggregate()` would immediately early-return its cached
        // metrics is pure waste. Gate them behind `should_compute`, the
        // single source of truth that `aggregate()` itself uses (tick 0
        // always initializes; then every `aggregation_interval` ticks), so
        // the aggregate call happens on exactly the same ticks as before
        // with identical inputs — behavior-preserving by construction.
        if !self.meme_aggregator.should_compute(tick_u64) {
            return;
        }

        // Build per-agent meme host lists (agent_idx → meme ids hosted).
        // §13.2 + §17.4: Track which memes each agent hosts.
        // NOTE: This is a placeholder heuristic — real meme hosting should be
        // tracked via a per-agent `hosted_memes: Vec<usize>` field. For now,
        // we assign memes based on cultural category identification strength.
        let meme_count = self.meme_registry.memes.len();
        let agent_meme_hosts: Vec<Vec<usize>> = self
            .agents
            .iter()
            .map(|a| {
                let mut hosts: Vec<usize> = Vec::new();
                for (cat_idx, cat) in a.cultural_cognition.categories.iter().enumerate() {
                    // Agents host memes whose index aligns with their identified categories
                    if cat.identification > Fixed::from_f64(0.3) {
                        let meme_id = cat_idx % meme_count;
                        if !hosts.contains(&meme_id) {
                            hosts.push(meme_id);
                        }
                    }
                }
                if hosts.is_empty() {
                    // Fallback: all agents host at least meme 0
                    hosts.push(0);
                }
                hosts
            })
            .collect();

        // Build agent group assignments (simplified: household index)
        let agent_groups: Vec<Option<usize>> = self
            .agents
            .iter()
            .enumerate()
            .map(|(i, _)| self.households.iter().position(|h| h.members.contains(&i)))
            .collect();

        // Collect active meme ids and emotional charges
        let meme_ids: Vec<usize> = self
            .meme_registry
            .memes
            .iter()
            .filter(|m| m.active)
            .map(|m| m.id)
            .collect();
        let meme_charges: Vec<Fixed> = meme_ids
            .iter()
            .filter_map(|&mid| self.meme_registry.get(mid).map(|m| m.emotional_charge))
            .collect();

        // Build group labels from households
        let group_labels: Vec<(usize, String)> = self
            .households
            .iter()
            .enumerate()
            .map(|(i, _)| (i, format!("household_{i}")))
            .collect();

        // Simplified centrality: relationship count normalized to [0,1].
        // Typical agent has 3-8 relationships; 10 normalizes well-connected agents to ~1.0.
        let centrality: Vec<Fixed> = self
            .agents
            .iter()
            .map(|a| {
                let n_rels = a.relationship_v2s.len() as f64;
                Fixed::from_f64((n_rels / 10.0).min(1.0))
            })
            .collect();

        self.meme_aggregator.aggregate(
            tick_u64,
            &agent_meme_hosts,
            &agent_groups,
            &meme_ids,
            &meme_charges,
            &group_labels,
            &centrality,
        );
    }
}
