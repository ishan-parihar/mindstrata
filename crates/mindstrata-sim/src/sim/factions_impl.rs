//! Faction grievance pressure and faction dynamics.

use super::{
    AgentId, Fixed, InstitutionKind, Simulation, Tick, LEGITIMACY_GRIEVANCE_DAMPEN,
    PATRONAGE_DESTITUTION_FLOOR, PATRONAGE_GRIEVANCE_DAMPEN, PATRONAGE_TRANSFER_RATE,
};
use crate::factions;

impl Simulation {
    /// §29.2 (Iteration 240): Current accumulated faction crisis pressure
    /// (0..1). Observability accessor for probes/inspectors — mutation
    /// happens only inside the faction-dynamics tick.
    pub fn faction_crisis_pressure(&self) -> f64 {
        self.faction_crisis_pressure
    }

    /// §29.2 (Iteration 240): Mean faction grievance across the unorganized
    /// pool (agents not currently in a faction) — the same aggregate the
    /// crisis-pressure accumulator consumes. Observability for probes,
    /// inspectors, and calibration.
    pub fn faction_pool_mean_grievance(&self) -> f64 {
        let grievances = self.unorganized_grievances();
        if grievances.is_empty() {
            return 0.0;
        }
        let sum: f64 = grievances.iter().map(|(_, g)| g.to_f64()).sum();
        sum / grievances.len() as f64
    }

    /// §29.2: Per-agent faction grievance over every agent NOT currently in a
    /// faction (factions are exclusive — members are never re-recruited).
    fn unorganized_grievances(&self) -> Vec<(AgentId, Fixed)> {
        let already_factioned: std::collections::HashSet<AgentId> = self
            .institutions
            .iter()
            .filter(|i| i.kind == InstitutionKind::Faction)
            .flat_map(|f| f.members.iter().copied())
            .collect();
        self.agents
            .iter()
            .enumerate()
            .filter(|(i, _)| !already_factioned.contains(&AgentId::new(*i as u64)))
            .map(|(i, _)| (AgentId::new(i as u64), self.faction_grievance(i)))
            .collect()
    }

    /// §29.2/§10.9: Per-agent faction grievance, dampened when the agent is a
    /// dependent patronage client — material security from the patron softens
    /// resentment/anger (the patron buys political quiescence). Deterministic.
    pub(super) fn faction_grievance(&self, idx: usize) -> Fixed {
        let agent = &self.agents[idx];
        let g = factions::compute_grievance(
            agent.derived.resentment,
            agent.emotions.fear,
            agent.emotions.anger,
            agent.needs.autonomy,
            agent.needs.meaning,
            agent.moral_values.fairness,
            self.market.inequality,
        );
        let dampen = self
            .patronage_registry
            .patron_of(idx)
            .map_or(Fixed::ZERO, |r| r.client_dependence)
            * PATRONAGE_GRIEVANCE_DAMPEN;
        // §11.1: Perceived legitimacy of the council modulates grievance — an
        // agent who believes the council rules rightfully is quiescent (dampen),
        // one who sees it as illegitimate is more grievance-prone (amplify, so
        // the dampen term must stay signed — the grievance is clamped below).
        // Mean-zero at the 0.5 construction anchor: no effect until the field
        // moves with ritual participation or witnessed violations.
        let legitimacy_deviation = agent.legitimacy_field.overall - Fixed::from_f64(0.5);
        let dampen = dampen + legitimacy_deviation * LEGITIMACY_GRIEVANCE_DAMPEN;
        (g * (Fixed::ONE - dampen)).max(Fixed::ZERO)
    }

    /// Section 15: Faction dynamics - grievance, formation, recruitment, protests.
    pub(super) fn tick_faction_dynamics(&mut self, tick_u64: u64, _tick: Tick) {
        // §29.2: Factions emerge from collective grievance. §Phase 8: legitimacy crisis → rebellion or reform.
        {
            let faction_count = self
                .institutions
                .iter()
                .filter(|i| i.kind == InstitutionKind::Faction)
                .count();

            // Compute grievance for each agent.
            // §29.2: Factions must be exclusive — an agent already in a faction
            // cannot be recruited again. Without this, every new faction pulls
            // from the same grievance pool, producing overlapping memberships
            // (e.g. 4 factions × 22 members on 48 agents). Enforced inside
            // `unorganized_grievances()` (Iteration 240 extraction).
            let grievances: Vec<(AgentId, Fixed)> = self.unorganized_grievances();

            // Find council legitimacy (average across councils)
            let avg_council_legitimacy: Fixed = {
                let councils: Vec<_> = self
                    .institutions
                    .iter()
                    .filter(|i| i.kind == InstitutionKind::Council)
                    .collect();
                if councils.is_empty() {
                    Fixed::from_f64(0.5)
                } else {
                    let sum: Fixed = councils
                        .iter()
                        .map(|c| c.legitimacy)
                        .fold(Fixed::ZERO, |a, b| a + b);
                    sum / Fixed::from_int(councils.len() as i64)
                }
            };

            // ── Iteration 240: crisis-pressure hazard accumulator (§29.2) ──
            // Sustained grievance above the recruitment anchor and council
            // illegitimacy below the gate build pressure; genuine peace bleeds
            // it off. Crossing the threshold arms formation through the exact
            // legacy path below — same recruitment, leader selection,
            // registration, provenance — so a crisis that never produces a
            // single-tick legitimacy/grievance cliff still reliably organizes
            // opposition, while transient spikes are erased by peace and can
            // no longer compound into clock-driven coups. Zero RNG-stream
            // consumption (pure arithmetic on already-computed aggregates).
            {
                use factions::{CRISIS_PRESSURE_BLEED_RATE, CRISIS_PRESSURE_BUILD_RATE};
                let grievance_excess = {
                    // Mean over the unorganized pool (already excludes current
                    // faction members): exactly the population a new faction
                    // would recruit from.
                    let mean = if grievances.is_empty() {
                        0.0
                    } else {
                        let sum: f64 = grievances.iter().map(|(_, g)| g.to_f64()).sum();
                        sum / grievances.len() as f64
                    };
                    (mean - factions::CRISIS_GRIEVANCE_ANCHOR).max(0.0)
                };
                let legitimacy_deficit = (0.5 - avg_council_legitimacy.to_f64()).max(0.0);
                if grievance_excess > 0.0 || legitimacy_deficit > 0.0 {
                    self.faction_crisis_pressure += grievance_excess * CRISIS_PRESSURE_BUILD_RATE
                        + legitimacy_deficit * CRISIS_PRESSURE_BUILD_RATE;
                } else {
                    self.faction_crisis_pressure *= 1.0 - CRISIS_PRESSURE_BLEED_RATE;
                }
                self.faction_crisis_pressure = self.faction_crisis_pressure.clamp(0.0, 1.0);
            }
            let pressure_arms = self.faction_crisis_pressure
                >= factions::FACTION_FORMATION_PRESSURE_THRESHOLD
                && tick_u64.saturating_sub(self.last_faction_formation_tick)
                    >= factions::FACTION_REARM_COOLDOWN_TICKS;

            // Faction formation: instant arm on the legitimacy cliff (legacy
            // path) OR accumulated crisis pressure (Iteration 240).
            let should_form = faction_count < factions::MAX_FACTIONS
                && tick_u64 > factions::FORMATION_COOLDOWN
                && (avg_council_legitimacy < Fixed::from_f64(0.5) || pressure_arms);

            if should_form {
                let recruitable = factions::find_recruitable_agents(
                    &grievances,
                    factions::RECRUITMENT_GRIEVANCE_THRESHOLD,
                );

                let avg_grievance = if recruitable.is_empty() {
                    Fixed::ZERO
                } else {
                    let sum: Fixed = recruitable
                        .iter()
                        .map(|(_, g)| *g)
                        .fold(Fixed::ZERO, |a, b| a + b);
                    sum / Fixed::from_int(recruitable.len() as i64)
                };

                let pool_ok = if pressure_arms {
                    // Pressure-certified: the accumulator has proven sustained
                    // collective discontent, so a real recruitment pool is the
                    // only remaining requirement — demanding the legacy 0.5
                    // recruitable mean here would re-impose the same
                    // single-tick cliff the accumulator exists to remove.
                    recruitable.len() >= 2
                } else {
                    avg_grievance >= factions::FORMATION_GRIEVANCE_THRESHOLD
                        && recruitable.len() >= 2
                };

                if pool_ok {
                    // Find leader candidates: high ambition, high dominance, low anger
                    let leader_candidates: Vec<_> = recruitable
                        .iter()
                        .filter_map(|(id, _)| {
                            let idx = id.as_u64() as usize;
                            if idx < self.agents.len() {
                                let a = &self.agents[idx];
                                if a.personality.ambition >= factions::LEADER_AMBITION_THRESHOLD {
                                    Some((
                                        *id,
                                        a.personality.ambition,
                                        a.personality.dominance,
                                        a.personality.extraversion,
                                        a.emotions.anger,
                                    ))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();

                    if let Some(leader) = factions::select_leader(&leader_candidates) {
                        let members: Vec<AgentId> = recruitable.iter().map(|(id, _)| *id).collect();

                        let new_faction = factions::create_faction(
                            1000 + faction_count as u64,
                            leader,
                            members.clone(),
                            avg_grievance,
                        );

                        tracing::warn!(
                            leader = leader.as_u64(),
                            members = members.len(),
                            grievance = avg_grievance.to_f64(),
                            tick = tick_u64,
                            "Faction formed"
                        );

                        // Record in provenance
                        self.provenance
                            .record_decision(crate::provenance::DecisionTrace {
                                agent: leader,
                                tick: tick_u64,
                                action_name: "FactionFormation".into(),
                                factors: vec![
                                    crate::provenance::DecisionFactor {
                                        kind: "grievance".into(),
                                        magnitude: avg_grievance,
                                        description: format!(
                                            "Avg grievance: {:.2}",
                                            avg_grievance.to_f64()
                                        ),
                                    },
                                    crate::provenance::DecisionFactor {
                                        kind: "council_legitimacy".into(),
                                        magnitude: avg_council_legitimacy,
                                        description: format!(
                                            "Council legitimacy: {:.2}",
                                            avg_council_legitimacy.to_f64()
                                        ),
                                    },
                                ],
                                from_routine: false,
                                interrupted_by_critical_needs: false,
                                intention_abandoned: false,
                            });

                        // Iteration 240: record the formation tick so the
                        // §7.3 revolution gate can enforce the mobilization
                        // (entrenchment) window. Set pre-push: no post-push
                        // unwrap needed.
                        let mut new_faction = new_faction;
                        new_faction.formed_tick = tick_u64;
                        self.institutions.push(new_faction);

                        // Iteration 240: the crisis that armed this formation
                        // has been answered — bleed the accumulator and start
                        // the rearm refractory so one long crisis cannot stack
                        // formations back-to-back (the FORMATION_COOLDOWN
                        // alone was ~1K-tick-scale; probed churn hit 20
                        // form/dissolve cycles per 30K epidemic ticks).
                        self.faction_crisis_pressure = 0.0;
                        self.last_faction_formation_tick = tick_u64;

                        // Architecture-plan-2 §5.2: Also register a FactionV2 in the upgraded registry.
                        let member_indices: Vec<usize> = members
                            .iter()
                            .map(|id| id.as_u64() as usize)
                            .filter(|&idx| idx < self.agents.len())
                            .collect();
                        let mut fv2 = crate::social::faction_v2::FactionV2::new(
                            leader.as_u64() as usize,
                            member_indices,
                            avg_grievance,
                            tick_u64,
                        );
                        // Architecture-plan-2 §12.3: attachment styles scale
                        // upward — the faction takes the modal member style,
                        // which modulates its daily dynamics (trust in
                        // leadership, fragmentation under stress, cohesion
                        // volatility, panic under scarcity). Mirrors the peer-
                        // group derivation; deterministic, no RNG.
                        {
                            let member_styles: Vec<_> = fv2
                                .members
                                .iter()
                                .map(|&m| self.agents[m].attachment.style)
                                .collect();
                            fv2.derive_attachment_style(&member_styles);
                        }
                        let fv2_id = self.faction_v2_registry.register(fv2);
                        // §13.5: Factions found their own mythic past — a
                        // founding memory for group 1 + faction id (0 = village).
                        self.record_faction_founding_memory(fv2_id, tick_u64);
                        tracing::info!(
                            faction_v2_id = fv2_id,
                            leader = leader.as_u64(),
                            members = members.len(),
                            grievance = avg_grievance.to_f64(),
                            tick = tick_u64,
                            "FactionV2 registered alongside v1"
                        );
                    }
                }
            }

            // ── Iteration 240: faction RECRUITMENT — the lifecycle's missing
            // stage ──
            // The AP2 lifecycle is formation → recruitment → entrenchment →
            // dissolution, but `FactionV2::recruitment_chance` had ZERO call
            // sites: factions formed with their founding membership and could
            // only shrink (deaths), so crisis factions dissolved within days
            // of plague attrition (probe: pestilence-5 formed 3 factions /
            // 30K, ALL dissolved before every 5K sample). One admission pass
            // per day per faction: the highest-grievance unorganized agent
            // joins when the faction's morale × the candidate's grievance
            // clears a conviction threshold — demoralized factions stop
            // growing, content agents never join, and replenishment offsets
            // casualty attrition. Deterministic (threshold-gated on
            // already-computed quantities, stable tie-break on agent order):
            // zero RNG-stream consumption.
            if tick_u64.is_multiple_of(144) {
                let village_cap = self.agents.len() / 2;
                let mut admissions: Vec<(usize, AgentId)> = Vec::new();
                for fv2 in &mut self.faction_v2_registry.factions {
                    if !fv2.active || fv2.members.len() >= village_cap {
                        continue;
                    }
                    // The single most convincing remaining candidate: highest
                    // grievance among the unorganized pool above the
                    // recruitment floor whose conviction (morale × grievance)
                    // clears the threshold. Stable max by (grievance, then
                    // lowest agent id) keeps selection deterministic.
                    let recruit_id = grievances
                        .iter()
                        .copied()
                        .filter(|(id, g)| {
                            *g >= factions::RECRUITMENT_GRIEVANCE_THRESHOLD
                                && fv2.morale * *g >= factions::FACTION_RECRUIT_CONVICTION_THRESHOLD
                                && !fv2.members.contains(&(id.as_u64() as usize))
                        })
                        .max_by_key(|(id, g)| (*g, std::cmp::Reverse(*id)))
                        .map(|(id, _)| id);
                    let Some(recruit) = recruit_id else {
                        continue;
                    };
                    let recruit_idx = recruit.as_u64() as usize;
                    if recruit_idx >= self.agents.len() {
                        continue;
                    }
                    fv2.members.push(recruit_idx);
                    admissions.push((fv2.leader, recruit));
                }
                for (leader_idx, recruit) in admissions {
                    // Mirror into the paired v1 institution (matched by its
                    // leader's membership — formation registers both with the
                    // same leader).
                    if let Some(inst) = self.institutions.iter_mut().find(|i| {
                        i.kind == InstitutionKind::Faction
                            && i.members.contains(&AgentId::new(leader_idx as u64))
                    }) {
                        inst.members.push(recruit);
                    }
                }
            }

            // Faction dynamics: check for protests from existing factions
            for inst_idx in 0..self.institutions.len() {
                if self.institutions[inst_idx].kind != InstitutionKind::Faction {
                    continue;
                }

                let faction_grievance = self.institutions[inst_idx].collective.morale;
                let faction_cohesion = self.institutions[inst_idx].collective.unity;

                if factions::should_protest(faction_grievance, faction_cohesion, tick_u64) {
                    let protest_size = self.institutions[inst_idx].members.len();
                    let total_pop = self.agents.len();

                    tracing::warn!(
                        faction = self.institutions[inst_idx].name.as_str(),
                        protest_size,
                        total_pop,
                        grievance = faction_grievance.to_f64(),
                        tick = tick_u64,
                        "Protest occurred"
                    );

                    // Council response
                    let council_enforcement = self
                        .institutions
                        .iter()
                        .filter(|i| i.kind == InstitutionKind::Council)
                        .map(|i| i.enforcement_capacity)
                        .fold(Fixed::ZERO, std::cmp::Ord::max);

                    // Architecture-plan-2 §29.2: an armed, mobilized, radicalized
                    // faction resists suppression — the protesting v1 faction's
                    // v2 record (matched by leader, registered 1:1 at formation)
                    // feeds its full threat model into the suppression decision:
                    // suppression_resistance blends the armed core (fighting
                    // strength) with the cohesion/grievance-modulated threat
                    // level and amplifies by legitimacy of violence (Iteration
                    // 100 — previously only raw fighting_strength was read;
                    // threat_level() and legitimacy_of_violence had zero
                    // consumers). Zero if no v2 record exists (legacy behavior).
                    let protest_leader = self.institutions[inst_idx]
                        .get_role_holder("Leader")
                        .map(|id| id.as_u64() as usize);
                    let protest_resistance = protest_leader
                        .and_then(|leader_idx| {
                            self.faction_v2_registry
                                .factions
                                .iter()
                                .find(|f| f.active && f.leader == leader_idx)
                        })
                        .map_or(
                            Fixed::ZERO,
                            crate::social::faction_v2::FactionV2::suppression_resistance,
                        );

                    let (suppressed, legitimacy_effect) = factions::council_response(
                        council_enforcement,
                        protest_size,
                        total_pop,
                        protest_resistance,
                    );

                    // Apply legitimacy effect to council
                    for inst in &mut self.institutions {
                        if inst.kind == InstitutionKind::Council {
                            if legitimacy_effect > Fixed::ZERO {
                                inst.increase_legitimacy(legitimacy_effect);
                            } else {
                                inst.decay_legitimacy(-legitimacy_effect);
                            }
                        }
                    }

                    // If not suppressed, faction gains legitimacy and morale boost
                    if !suppressed {
                        if let Some(faction) = self.institutions.get_mut(inst_idx) {
                            faction.increase_legitimacy(Fixed::from_f64(0.02));
                            faction.collective.morale =
                                (faction.collective.morale + Fixed::from_f64(0.05)).clamp_01();
                        }
                    }

                    // Record in provenance (use leader of the protesting faction)
                    let protest_agent = self.institutions[inst_idx]
                        .get_role_holder("Leader")
                        .unwrap_or(AgentId::new(0));
                    self.provenance
                        .record_decision(crate::provenance::DecisionTrace {
                            agent: protest_agent,
                            tick: tick_u64,
                            action_name: "Protest".into(),
                            factors: vec![
                                crate::provenance::DecisionFactor {
                                    kind: "protest_size".into(),
                                    magnitude: Fixed::from_int(protest_size as i64),
                                    description: format!("Protest size: {protest_size}"),
                                },
                                crate::provenance::DecisionFactor {
                                    kind: "suppressed".into(),
                                    magnitude: if suppressed { Fixed::ONE } else { Fixed::ZERO },
                                    description: format!("Suppressed: {suppressed}"),
                                },
                            ],
                            from_routine: false,
                            interrupted_by_critical_needs: false,
                            intention_abandoned: false,
                        });
                }
            }
        }
    }

    /// §10.9: Patrons provision destitute clients — the economic side of
    /// patronage. When a client's coin falls below the destitution floor and
    /// the patron can afford the stipend, a small daily transfer moves wealth
    /// through the social structure (a safety net that softens inequality).
    /// Deterministic (no RNG): reads are collected, then applied.
    pub(super) fn tick_patronage_provision(&mut self) {
        let mut transfers: Vec<(usize, usize, Fixed)> = Vec::new();
        for rel in &self.patronage_registry.relations {
            if !rel.active {
                continue;
            }
            let (patron, client) = (rel.patron, rel.client);
            if patron >= self.agents.len() || client >= self.agents.len() {
                continue;
            }
            if self.agents[client].wealth.coin >= PATRONAGE_DESTITUTION_FLOOR {
                continue;
            }
            let transfer = rel.provision * PATRONAGE_TRANSFER_RATE;
            if self.agents[patron].wealth.coin < transfer {
                continue;
            }
            transfers.push((patron, client, transfer));
        }
        for (patron, client, transfer) in transfers {
            self.agents[patron].wealth.coin -= transfer;
            self.agents[client].wealth.coin += transfer;
        }
    }
}
