//! Norm evaluation, moral panic lifecycle and derived belief states.

use super::{
    AgentId, Belief, ConflictKind, Fixed, InstitutionKind, JournalEntryKind, RngStream, SimEvent,
    Simulation, Tick, BELIEF_CULTURAL_FLOOR, BELIEF_CULTURAL_RATE, VIOLENCE_SHAME_TABOO_RATE,
};
use crate::belief_update;
use crate::conflict;
use crate::factions;
use crate::gossip;
use crate::health;
use crate::institutions;
use crate::norms;

use rand::Rng;
impl Simulation {
    /// Section 11: Norm evaluation — threats and insults are norm violations.
    pub(super) fn tick_norm_evaluation(
        &mut self,
        pre_tick_events: usize,
        tick_u64: u64,
        tick: Tick,
    ) {
        // ── 11. Norm evaluation: threats and insults are norm violations ──
        let snap_count = self.events.len();
        for idx in pre_tick_events..snap_count {
            let ev = &self.events[idx];
            if let SimEvent::InteractionOccurred {
                from,
                to,
                kind:
                    mindstrata_core::event::InteractionKind::Threaten
                    | mindstrata_core::event::InteractionKind::Insult,
                ..
            } = ev
            {
                let from_id = *from;
                let to_id = *to;

                // §19.5.H: Route threats through conflict system for trauma, combat fatigue tracking
                let is_threat = matches!(
                    ev,
                    SimEvent::InteractionOccurred {
                        kind: mindstrata_core::event::InteractionKind::Threaten,
                        ..
                    }
                );
                if is_threat {
                    let from_idx = from_id.as_u64() as usize;
                    let to_idx = to_id.as_u64() as usize;
                    if from_idx < self.agents.len()
                        && to_idx < self.agents.len()
                        && to_idx < self.agent_diseases.len()
                    {
                        let rng_val =
                            Fixed::from_f64(self.rng.get_mut(RngStream::Social).random::<f64>());
                        let conflict_result = conflict::resolve_conflict(
                            ConflictKind::Threat,
                            &self.agents[from_idx].personality,
                            &self.agents[to_idx].emotions,
                            self.agents[to_idx].body.health,
                            &self.agents[to_idx].conflict,
                            rng_val,
                            &self.params,
                        );
                        if conflict_result.occurred {
                            // Record conflict state: trauma, combat fatigue
                            self.agents[to_idx]
                                .conflict
                                .record_conflict(ConflictKind::Threat, &self.params);
                            // Apply fear induction
                            self.agents[to_idx].emotions.fear = (self.agents[to_idx].emotions.fear
                                + conflict_result.fear_induced)
                                .clamp_01();
                            // Emit conflict event for observability
                            self.events.push(SimEvent::ConflictOccurred {
                                aggressor: from_id,
                                target: to_id,
                                kind: ConflictKind::Threat,
                                injury: conflict_result.injury,
                                fear_induced: conflict_result.fear_induced,
                                tick,
                            });

                            // §19.5.H: Violence escalation — when threat fails to deter
                            // (target fear remains low after threat), aggressive aggressors
                            // escalate to physical violence with real injury.
                            let target_fear_after = self.agents[to_idx].emotions.fear;
                            let threat_failed =
                                target_fear_after < self.params.conflict_escalation_fear_threshold;
                            // §7.2.2 (Iteration 191 — the documented deferral
                            // closed): the endocrine dominance axis (mean-
                            // reverting toward the §11.1 effective-status
                            // composite, wired daily) now feeds the escalation
                            // decision. Pre-Iter-191 the axis was write-only —
                            // the S2-2-2 fix ran `dominance.update` but its
                            // accessor `dominance_aggression_modifier` had ZERO
                            // consumers, so a status-risen elite and a
                            // status-drained subordinate pressed failed threats
                            // identically. The fold uses the same
                            // `conflict_dominance_weight` (0.3) the injury
                            // calc applies to personality dominance: a hormonally
                            // dominant aggressor escalates a failed threat more
                            // readily ("I can win this"), a drained one backs
                            // down. Deterministic, no RNG — the draw stream is
                            // untouched (only the gate threshold moves).
                            let endo_dominance = self.agents[from_idx]
                                .embodied
                                .endocrine
                                .dominance_aggression_modifier();
                            // §8.1.10 (Iteration 191 — impulse control wired):
                            // `can_inhibit` was dead code whose underlying
                            // `effective_inhibition` (computed every tick from
                            // inhibition × (1 − stress_load)) had NO other
                            // reader — a write-only field. The escalation
                            // decision is its designed consumer: an agent that
                            // cannot inhibit impulses presses a failed threat to
                            // violence more readily; a high-inhibition agent
                            // holds back (the stress-degraded inhibition
                            // channel is now behaviorally live).
                            let inhibition_hold =
                                if self.agents[from_idx].cognitive_runtime.can_inhibit() {
                                    Fixed::from_f64(0.0)
                                } else {
                                    Fixed::from_f64(0.15)
                                };
                            let aggressor_aggression = self.agents[from_idx].personality.dominance
                                + self.agents[from_idx].personality.risk_tolerance
                                + endo_dominance * self.params.conflict_dominance_weight
                                + inhibition_hold;
                            // §10.8: Enemy clans escalate failed threats at twice
                            // the base rate (should_escalate / escalation_chance).
                            let escalate = self.should_escalate(
                                from_idx,
                                to_idx,
                                threat_failed,
                                aggressor_aggression,
                            );

                            if escalate {
                                let violence_result = conflict::resolve_conflict(
                                    ConflictKind::Violence,
                                    &self.agents[from_idx].personality,
                                    &self.agents[to_idx].emotions,
                                    self.agents[to_idx].body.health,
                                    &self.agents[to_idx].conflict,
                                    rng_val,
                                    &self.params,
                                );
                                if violence_result.occurred {
                                    self.agents[to_idx]
                                        .conflict
                                        .record_conflict(ConflictKind::Violence, &self.params);
                                    self.agents[to_idx].emotions.fear =
                                        (self.agents[to_idx].emotions.fear
                                            + violence_result.fear_induced)
                                            .clamp_01();
                                    // Apply injury to target's health.
                                    // §32 (Iteration 187 — the apply_injury
                                    // wiring): the lone WoundInfection source
                                    // (`apply_injury`) was dead code with a
                                    // hardcoded RNG; it now runs here with a
                                    // real seeded draw. Severity is passed as
                                    // `injury × 10` so `apply_injury`'s
                                    // `severity × 0.1` damage reproduces the
                                    // legacy inline `health -= injury` EXACTLY
                                    // (the pre-Iter-187 damage path is
                                    // unchanged); the new energy drain
                                    // (`injury × 0.5`) is the honest addition
                                    // — injuries sap strength. The wound-
                                    // infection roll uses the agent's live
                                    // `injury_chance` config.
                                    let injury = violence_result.injury;
                                    if injury > Fixed::ZERO {
                                        let wound_roll = Fixed::from_f64(
                                            self.rng.get_mut(RngStream::Behavior).random::<f64>(),
                                        );
                                        // Destructure the body borrow so the
                                        // two disjoint fields (health, energy)
                                        // can be passed alongside the separate
                                        // `agent_diseases` vec.
                                        let body = &mut self.agents[to_idx].body;
                                        let diseases = &mut self.agent_diseases[to_idx];
                                        health::apply_injury(
                                            &mut body.health,
                                            &mut body.energy,
                                            diseases,
                                            injury * Fixed::from_f64(10.0),
                                            &self.health_config,
                                            wound_roll,
                                        );
                                    }
                                    // Record injury
                                    self.agents[to_idx].conflict.record_injury();
                                    // Attacker accumulates trauma too
                                    self.agents[from_idx]
                                        .conflict
                                        .record_conflict(ConflictKind::Violence, &self.params);
                                    // Reduce trust and affection between the two
                                    // §19.5.J: Record relationship trace for provenance
                                    if let Some(rel) = self
                                        .relationships
                                        .iter_mut()
                                        .find(|r| r.from == from_id && r.to == to_id)
                                    {
                                        let old_trust = rel.trust;
                                        let old_affection = rel.affection;
                                        rel.trust =
                                            (rel.trust - Fixed::from_f64(0.3)).max(Fixed::ZERO);
                                        rel.affection =
                                            (rel.affection - Fixed::from_f64(0.2)).max(Fixed::ZERO);
                                        self.provenance.record_relationship(
                                            crate::provenance::RelationshipTrace {
                                                from: from_id,
                                                to: to_id,
                                                tick: tick_u64,
                                                cause: "violence".into(),
                                                old_trust,
                                                new_trust: rel.trust,
                                                old_affection,
                                                new_affection: rel.affection,
                                                description: format!(
                                                    "Violence destroyed trust ({} -> {})",
                                                    old_trust.to_f64(),
                                                    rel.trust.to_f64()
                                                ),
                                            },
                                        );
                                    }
                                    if let Some(rel) = self
                                        .relationships
                                        .iter_mut()
                                        .find(|r| r.from == to_id && r.to == from_id)
                                    {
                                        let old_trust = rel.trust;
                                        let old_affection = rel.affection;
                                        rel.trust =
                                            (rel.trust - Fixed::from_f64(0.3)).max(Fixed::ZERO);
                                        rel.affection =
                                            (rel.affection - Fixed::from_f64(0.2)).max(Fixed::ZERO);
                                        self.provenance.record_relationship(
                                            crate::provenance::RelationshipTrace {
                                                from: to_id,
                                                to: from_id,
                                                tick: tick_u64,
                                                cause: "violence".into(),
                                                old_trust,
                                                new_trust: rel.trust,
                                                old_affection,
                                                new_affection: rel.affection,
                                                description: format!(
                                                    "Fear response — trust dropped ({} -> {})",
                                                    old_trust.to_f64(),
                                                    rel.trust.to_f64()
                                                ),
                                            },
                                        );
                                    }
                                    // §10.2 (AP2): Record the violence as a betrayal in
                                    // both directions of the RelationshipV2 history
                                    // (bounded, new fields only — the trust/affection
                                    // damage is applied above, so calibrated runs stay
                                    // byte-identical).
                                    if from_idx != to_idx {
                                        let f_pos = Self::relationship_v2_pos(from_idx, to_idx);
                                        let t_pos = Self::relationship_v2_pos(to_idx, from_idx);
                                        if f_pos < self.agents[from_idx].relationship_v2s.len() {
                                            self.agents[from_idx].relationship_v2s[f_pos].record_betrayal(
                                                tick_u64,
                                                crate::social::relationship_v2::BetrayalKind::Violence,
                                                violence_result.fear_induced,
                                            );
                                        }
                                        if t_pos < self.agents[to_idx].relationship_v2s.len() {
                                            self.agents[to_idx].relationship_v2s[t_pos].record_betrayal(
                                                tick_u64,
                                                crate::social::relationship_v2::BetrayalKind::Violence,
                                                violence_result.fear_induced,
                                            );
                                        }
                                    }
                                    // Record in journal
                                    self.journal.record(
                                        tick_u64,
                                        from_id,
                                        JournalEntryKind::CommittedViolence {
                                            target: to_id.as_u64(),
                                            injury: violence_result.injury.to_f64(),
                                        },
                                    );
                                    // Emit violence event
                                    self.events.push(SimEvent::ConflictOccurred {
                                        aggressor: from_id,
                                        target: to_id,
                                        kind: ConflictKind::Violence,
                                        injury: violence_result.injury,
                                        fear_induced: violence_result.fear_induced,
                                        tick,
                                    });
                                    // §19.5.D: Violence is a severe norm violation.
                                    let _ = self.norms.check_violation(
                                        crate::norms::NO_VIOLENCE_NORM_ID,
                                        from_id,
                                        tick_u64,
                                    );
                                    // §8.1.10/§19.5.D (Iteration 88): the
                                    // no-violence witnessed-enforcement audit —
                                    // unlike sneaky theft (which needs a
                                    // detection roll), a violent act is
                                    // inherently public, so no detection roll
                                    // is drawn (zero new RNG — the golden
                                    // baseline stays byte-identical). Every
                                    // agent holding the internalized
                                    // no-violence norm witnesses the village's
                                    // enforcement response (the check_violation
                                    // record — with the attacker's public shame
                                    // applied below) and records it, exactly
                                    // as the Iteration-86
                                    // theft audit does. Gated on holders:
                                    // before the first monthly ritual (tick
                                    // 4320) no agent holds any norm, so the
                                    // audit is a no-op in the golden window.
                                    // Resolved by id (`NO_VIOLENCE_NORM_ID`); a
                                    // registry without the norm is a no-op.
                                    let no_violence_name = self
                                        .norms
                                        .norms()
                                        .iter()
                                        .find(|n| n.id == norms::NO_VIOLENCE_NORM_ID)
                                        .map(|n| n.name.as_str());
                                    if let Some(name) = no_violence_name {
                                        for bundle in &mut self.agents {
                                            bundle
                                                .moral_cognition
                                                .record_witnessed_enforcement(name);
                                        }
                                    }
                                    // §19.5.G: Feud tracking is handled in dedicated section 18

                                    // Attacker gains shame from violence
                                    // §8.1.18 (Iteration 167): the attacker's
                                    // own no-violence taboo amplifies the
                                    // shame — an agent whose culture forbids
                                    // violence more strongly (higher
                                    // traditionalism → stronger taboo →
                                    // higher violation_cost) feels more
                                    // shame per violent act (sacred
                                    // severity). ONE-SIDED factor
                                    // (1 + violation_cost × 0.5): identity
                                    // at zero taboo (legacy/empty vecs),
                                    // bounded above ≈1.23 at the seeded
                                    // max (Violence base 0.45 ×
                                    // traditionalism 1.0). Deterministic:
                                    // pure arithmetic on existing state,
                                    // zero new RNG — the violence decision
                                    // was already drawn.
                                    let taboo_shame_factor = Fixed::ONE
                                        + self.agents[from_idx]
                                            .cultural_cognition
                                            .taboo_violation_cost_for("violence")
                                            * VIOLENCE_SHAME_TABOO_RATE;
                                    self.agents[from_idx].emotions.shame =
                                        (self.agents[from_idx].emotions.shame
                                            + Fixed::from_f64(0.15) * taboo_shame_factor)
                                            .clamp_01();

                                    // §19.5.H: Check if violence was lethal
                                    if self.agents[to_idx].body.health <= Fixed::ZERO {
                                        self.agents[to_idx].conflict.record_casualty();
                                        self.events.push(SimEvent::AgentDied {
                                            agent: to_id,
                                            cause: mindstrata_core::event::DeathCause::Violence,
                                            tick,
                                        });
                                        tracing::warn!(
                                            agent = to_id.as_u64(),
                                            attacker = from_id.as_u64(),
                                            tick = tick_u64,
                                            "Agent killed by violence"
                                        );
                                    }

                                    tracing::warn!(
                                        aggressor = from_id.as_u64(),
                                        target = to_id.as_u64(),
                                        injury = violence_result.injury.to_f64(),
                                        tick = tick_u64,
                                        "Violence escalated from failed threat"
                                    );
                                }
                            }
                        }
                    }
                }

                let punishment = self.norms.check_violation(1, from_id, tick_u64);
                if punishment > Fixed::ZERO {
                    if let Some(rel) = self
                        .relationships
                        .iter_mut()
                        .find(|r| r.from == to_id && r.to == from_id)
                    {
                        let old_trust = rel.trust;
                        rel.trust =
                            (rel.trust - punishment * Fixed::from_f64(0.15)).max(Fixed::ZERO);
                        // §19.5.J: Record norm violation relationship trace
                        self.provenance
                            .record_relationship(crate::provenance::RelationshipTrace {
                                from: to_id,
                                to: from_id,
                                tick: tick_u64,
                                cause: "norm_violation".into(),
                                old_trust,
                                new_trust: rel.trust,
                                old_affection: rel.affection,
                                new_affection: rel.affection,
                                description: format!(
                                    "Trust eroded by norm violation ({} -> {})",
                                    old_trust.to_f64(),
                                    rel.trust.to_f64()
                                ),
                            });
                    }
                    let from_idx = from_id.as_u64() as usize;
                    if from_idx < self.agents.len() {
                        // §22.1: Moral outrage modulated by moral foundations.
                        // High care/fairness → more shame from own violations.
                        let outrage = self.agents[from_idx].moral_values.moral_outrage(punishment);
                        self.agents[from_idx].emotions.shame =
                            (self.agents[from_idx].emotions.shame
                                + outrage * Fixed::from_f64(0.10))
                            .clamp_01();
                    }
                    // §12.4: Successful norm enforcement increases institutional legitimacy
                    // Modulated by enforcement_capacity — low-capacity councils gain less.
                    for inst in &mut self.institutions {
                        if inst.kind == institutions::InstitutionKind::Council {
                            let legitimacy_gain =
                                inst.enforcement_capacity * Fixed::from_f64(0.005);
                            inst.increase_legitimacy(legitimacy_gain);
                        }
                    }
                }
            }
        }
    }

    /// Sections 14+14b: Moral panic detection and revolution mechanism.
    pub(super) fn tick_moral_panic_and_revolution(&mut self, tick_u64: u64, tick: Tick) {
        // ── 14. §7.2: Moral panic detection — rumor cascades collapse institutional trust ──
        // Check if gossip about institutions has accumulated enough emotional charge
        // to trigger a moral panic. This happens when many agents hold high-emotion
        // beliefs about an institution, causing sudden legitimacy collapse.
        {
            // §7.2: Moral panics are discrete crises, not a per-tick loop.
            // Enforce a cooldown so a saturated belief-charge pool cannot fire a
            // panic every single tick (previously ~19K events with legitimacy
            // pinned at 0.000). Between panics the belief charges decay toward
            // their resting level (see below), letting the crisis resolve.
            const MORAL_PANIC_COOLDOWN: u64 = 300;
            let panic_cooldown_ok =
                tick_u64.saturating_sub(self.last_moral_panic_tick) >= MORAL_PANIC_COOLDOWN;

            // Check propositions 0 ("the_market_is_fair") and 1 ("the_council_protects_us")
            let belief_refs: Vec<&[Belief]> =
                self.agents.iter().map(|a| a.beliefs.as_slice()).collect();
            for prop_id in 0..=1 {
                let mut panic_result = gossip::detect_moral_panic(&belief_refs, prop_id);
                if panic_result.triggered && panic_cooldown_ok {
                    self.last_moral_panic_tick = tick_u64;
                    tracing::warn!(
                        proposition = prop_id,
                        avg_charge = panic_result.avg_charge.to_f64(),
                        tick = tick_u64,
                        "§7.2: Moral panic triggered!"
                    );
                    // §10.1.3 (Iteration 112): a genuinely terrified
                    // population amplifies the panic's legitimacy damage —
                    // collective fear (world mean agent fear, refreshed
                    // daily into the relational fields) scales the damage
                    // by `collective_fear_panic_amplifier`. ONE-SIDED:
                    // identity at/below the 0.95 anchor (above the
                    // calibrated peak of 0.903), so this consumer is
                    // provably zero-blast in every calibrated window — the
                    // single seed-42 panic at ~4,320 ticks sits at
                    // collective fear 0.90 and is likewise unamplified.
                    // The `Fixed` conversions below run only when a panic
                    // actually fires (sub-tick-frequency), so the cost is
                    // negligible.
                    let collective_fear = self
                        .agents
                        .iter()
                        .map(|a| a.emotions.fear)
                        .fold(Fixed::ZERO, |acc, f| acc + f)
                        / Fixed::from_int(self.agents.len() as i64);
                    let panic_amplifier = crate::social::relational_field::
                        RelationalFields::collective_fear_panic_amplifier(
                            collective_fear,
                            Fixed::from_f64(crate::social::relational_field::
                                COLLECTIVE_FEAR_PANIC_ANCHOR),
                            Fixed::from_f64(crate::social::relational_field::
                                COLLECTIVE_FEAR_PANIC_RATE),
                            Fixed::from_f64(crate::social::relational_field::
                                COLLECTIVE_FEAR_PANIC_CAP),
                        );
                    panic_result.legitimacy_damage =
                        panic_result.legitimacy_damage * panic_amplifier;
                    // Apply legitimacy damage to all matching institutions
                    for inst in &mut self.institutions {
                        let inst_prop = match inst.kind {
                            InstitutionKind::Council => Some(1u64), // council → proposition 1
                            InstitutionKind::Market => Some(0u64),  // market → proposition 0
                            _ => None,
                        };
                        if inst_prop == Some(prop_id) {
                            inst.legitimacy =
                                (inst.legitimacy - panic_result.legitimacy_damage).max(Fixed::ZERO);
                        }
                    }
                    // Boost faction grievance from panic
                    for inst in &mut self.institutions {
                        if inst.kind == InstitutionKind::Faction {
                            inst.collective.morale =
                                (inst.collective.morale + panic_result.grievance_boost).clamp_01();
                        }
                    }
                    self.events.push(SimEvent::ConflictOccurred {
                        aggressor: AgentId::new(0),
                        target: AgentId::new(0),
                        kind: ConflictKind::MoralPanic,
                        injury: Fixed::ZERO,
                        fear_induced: panic_result.avg_charge,
                        tick,
                    });
                    // §13.5 (Iteration 115): record the panic so its
                    // lifecycle runs daily — escalation/de-escalation from
                    // `escalation_pressure` + fatigue, and the legitimacy
                    // drain. Previously NOTHING ever registered a panic:
                    // the registry stayed empty and `daily_update` was a
                    // no-op (unit-test-only dead machinery). `start_tick`
                    // lets the daily driver skip same-tick processing (the
                    // §7.2 legitimacy damage above already applied).
                    let trigger = match prop_id {
                        // proposition 1 = "the_council_protects_us" → council
                        // corruption; proposition 0 = "the_market_is_fair" →
                        // a perceived moral failing of the market.
                        0 => crate::noosphere::moral_panic::PanicTrigger::MoralViolation,
                        _ => crate::noosphere::moral_panic::PanicTrigger::InstitutionalCorruption,
                    };
                    self.moral_panic_registry.register(
                        crate::noosphere::moral_panic::MoralPanic::new(trigger, None, tick_u64),
                    );
                }
            }

            // §7.2: Let panic resolve — decay the emotional charge of the
            // affected propositions so the loop does not re-trigger instantly.
            if tick_u64.saturating_sub(self.last_moral_panic_tick) < MORAL_PANIC_COOLDOWN
                && tick_u64.saturating_sub(self.last_moral_panic_tick) > 0
            {
                let post_panic_charge = Fixed::from_f64(0.1);
                for agent in &mut self.agents {
                    for belief in &mut agent.beliefs {
                        if belief.proposition_id <= 1 {
                            belief.emotional_charge = belief
                                .emotional_charge
                                .lerp(post_panic_charge, Fixed::from_f64(0.02));
                        }
                    }
                }
            }
        }

        // ── 14b. §7.3: Revolution mechanism — faction seizes control ──
        // When a faction has enough grievance, membership, and the council's
        // legitimacy is critically low, the faction stages a revolution.
        {
            let council_legitimacy: Fixed = self
                .institutions
                .iter()
                .filter(|i| i.kind == InstitutionKind::Council)
                .map(|i| i.legitimacy)
                .fold(Fixed::ZERO, std::cmp::Ord::max);

            for inst_idx in 0..self.institutions.len() {
                if self.institutions[inst_idx].kind != InstitutionKind::Faction {
                    continue;
                }
                let faction = &self.institutions[inst_idx];
                let faction_grievance = faction.collective.morale;
                let faction_size = faction.members.len();
                let total_pop = self.agents.len();

                // Revolution score: weighted combination of grievance, size,
                // council weakness — and (Iteration 240/242) accumulated
                // crisis pressure. Without the pressure term the score could
                // never cross the 0.6 threshold again once formation stopped
                // requiring a legitimacy cliff: factions now organize at
                // legitimacy ~0.55 (probe: pestilence seed 5 holds 3-4
                // protest blocs through 160K, council recovering to ~0.56,
                // ZERO revolts - the §7.3 mechanism was dead). The same
                // accumulator that armed formation measures how much
                // sustained discontent has built since the last answer;
                // a bloc backed by a full pressure tank (1.0 -> +0.2)
                // carries enough momentum to revolt, while calm-world
                // residue (+<=0.06) stays safely sub-threshold.
                let revolution_score = faction_grievance * factions::REVOLUTION_GRIEVANCE_WEIGHT
                    + Fixed::from_int(faction_size as i64)
                        / Fixed::from_int(total_pop.max(1) as i64)
                        * factions::REVOLUTION_SIZE_WEIGHT
                    + (Fixed::ONE - council_legitimacy) * factions::REVOLUTION_LEGITIMACY_WEIGHT
                    + Fixed::from_f64(
                        self.faction_crisis_pressure.min(1.0)
                            * factions::REVOLUTION_PRESSURE_WEIGHT,
                    );

                let revolution_cooldown_ok = tick_u64.saturating_sub(self.last_revolution_tick)
                    >= factions::REVOLUTION_COOLDOWN;
                // §7.3 (Iteration 240): mobilization window — a faction must
                // have existed long enough to organize before it can revolt.
                let entrenchment_ok = tick_u64.saturating_sub(faction.formed_tick)
                    >= factions::FACTION_ENTRENCHMENT_MIN_TICKS;
                if revolution_score >= factions::REVOLUTION_SCORE_THRESHOLD
                    && tick_u64 > factions::REVOLUTION_MIN_TICK
                    && revolution_cooldown_ok
                    && entrenchment_ok
                {
                    tracing::warn!(
                        faction = inst_idx,
                        score = revolution_score.to_f64(),
                        council_legitimacy = council_legitimacy.to_f64(),
                        tick = tick_u64,
                        "§7.3: Revolution! Faction seizes control"
                    );
                    // §7.3: A successful coup is a regime change — the faction
                    // becomes the new establishment. The old council is deposed
                    // and its seats taken by the faction's leadership; the
                    // faction itself dissolves (it achieved its goal). Previously
                    // the faction merely gained legitimacy and kept its members,
                    // so derive_collective_psychology rebuilt its morale from
                    // member valence + legitimacy each tick and it revolted again
                    // every REVOLUTION_COOLDOWN ticks (6 coups in 1400 ticks
                    // observed). With the faction dissolved, a fresh grievance
                    // cycle must build a new faction to revolt again.
                    let faction_members = self.institutions[inst_idx].members.clone();
                    let faction_roles = self.institutions[inst_idx].roles.clone();
                    let leader_id = self.institutions[inst_idx]
                        .get_role_holder("Leader")
                        .unwrap_or_else(|| AgentId::new(inst_idx as u64));
                    // §5.2 (Iteration 89): Deactivate *every* FactionV2 record
                    // — the retain below dissolves all v1 factions, so each
                    // active v2 record's institution is being wiped and must
                    // stop reporting as active (stale threat/agent-tier).
                    // Previously only the revolting leader's record was
                    // deactivated, leaving the other factions' records
                    // stale-active and breaking the v1↔v2 1:1 invariant.
                    self.faction_v2_registry.deactivate_all();
                    self.institutions
                        .retain(|i| i.kind != InstitutionKind::Faction);
                    // The faction leadership becomes the new council.
                    for inst in &mut self.institutions {
                        if inst.kind == InstitutionKind::Council {
                            inst.members.clone_from(&faction_members);
                            for role in &faction_roles {
                                inst.add_role(role.clone());
                            }
                            inst.legitimacy = Fixed::from_f64(0.5);
                            inst.collective.morale = Fixed::from_f64(0.5);
                        }
                    }
                    // Track revolution tick for cooldown
                    self.last_revolution_tick = tick_u64;
                    // Record revolution event (leader captured before the
                    // faction was dissolved by the retain above)
                    self.events.push(SimEvent::ConflictOccurred {
                        aggressor: leader_id,
                        target: AgentId::new(0),
                        kind: ConflictKind::Revolution,
                        injury: Fixed::ZERO,
                        fear_induced: Fixed::from_f64(0.5),
                        tick,
                    });
                    break; // only one revolution per tick
                }
            }
        }
    }

    /// §13.5 (Iteration 115): daily moral-panic lifecycle driver.
    ///
    /// For each ACTIVE panic (skipping panics registered this very tick —
    /// their §7.2 legitimacy damage already applied at trigger), the
    /// resolve-first cycle runs:
    /// 1. `should_resolve(institutional_response, fatigue)` — with the
    ///    institutional response never set by any system, fatigue
    ///    (`panic_fatigue`, 0.02/day → crosses the 0.7 resolve threshold
    ///    after 35 days) is the resolution driver: the panic ends when the
    ///    population tires of the crisis;
    /// 2. else `escalation_pressure(mean_legitimacy, community_fear)` above
    ///    the 0.5 threshold → `escalate` (self-reinforcing: as the panic
    ///    erodes legitimacy, the legitimacy-vacuum term keeps pressure
    ///    high);
    /// 3. else natural `daily_update` decay.
    ///
    /// Then the DECISIONAL CONSUMER: the panic's (post-update) intensity
    /// erodes the legitimacy of the institution it erupted about
    /// (`panic_legitimacy_drain`) each day until resolved. Deterministic —
    /// no RNG — so replay determinism holds. Zero-blast by construction:
    /// no panic fires before ~4,320 ticks in seed-42 runs, so the golden
    /// (1,000) and snapshot (≤2,000) horizons never see this consumer.
    pub(super) fn tick_moral_panic_lifecycle(&mut self, tick_u64: u64) {
        use crate::noosphere::moral_panic::{
            panic_fatigue, panic_legitimacy_drain, PanicTrigger, PANIC_ESCALATION_THRESHOLD,
            PANIC_FATIGUE_RATE, PANIC_LEGITIMACY_DRAIN_RATE, TICKS_PER_DAY,
        };
        if self.moral_panic_registry.panics.is_empty() {
            return;
        }
        // Mean legitimacy over the two panic-relevant institutions and the
        // world mean fear (community fear) drive `escalation_pressure`.
        let mut legit_sum = Fixed::ZERO;
        let mut legit_count = 0u32;
        for inst in &self.institutions {
            if inst.kind == InstitutionKind::Council || inst.kind == InstitutionKind::Market {
                legit_sum += inst.legitimacy;
                legit_count += 1;
            }
        }
        let mean_legitimacy = if legit_count > 0 {
            legit_sum / Fixed::from_int(legit_count as i64)
        } else {
            Fixed::ZERO
        };
        let n = self.agents.len();
        let community_fear = if n > 0 {
            self.agents
                .iter()
                .map(|a| a.emotions.fear)
                .fold(Fixed::ZERO, |acc, f| acc + f)
                / Fixed::from_int(n as i64)
        } else {
            Fixed::ZERO
        };
        let panics = &mut self.moral_panic_registry.panics;
        for panic in panics {
            if !panic.active || panic.start_tick == tick_u64 {
                continue;
            }
            let elapsed_days = (tick_u64 - panic.start_tick) / TICKS_PER_DAY;
            let fatigue = panic_fatigue(elapsed_days, PANIC_FATIGUE_RATE);
            let pressure = panic.escalation_pressure(mean_legitimacy, community_fear);
            if panic.should_resolve(Fixed::ZERO, fatigue) {
                panic.deescalate();
            } else if pressure > Fixed::from_f64(PANIC_ESCALATION_THRESHOLD) {
                panic.escalate(tick_u64);
            } else {
                panic.daily_update();
            }
            // Decisional consumer: active intensity erodes the institution
            // the panic erupted about ("panic → distrust → more panic").
            let target = match panic.trigger {
                PanicTrigger::InstitutionalCorruption => Some(InstitutionKind::Council),
                PanicTrigger::MoralViolation => Some(InstitutionKind::Market),
                _ => None,
            };
            if let Some(target) = target {
                let drain = panic_legitimacy_drain(panic.intensity, PANIC_LEGITIMACY_DRAIN_RATE);
                for inst in &mut self.institutions {
                    if inst.kind == target {
                        inst.legitimacy = (inst.legitimacy - drain).max(Fixed::ZERO);
                    }
                }
            }
        }
    }

    /// Sections 15+13: Derived mental state computation and belief updates.
    pub(super) fn tick_derived_states_and_beliefs(
        &mut self,
        pre_tick_events: usize,
        tick_u64: u64,
    ) {
        // ── 15. Derived mental state computation (§22) ─────────────
        // §17: Periodic tier reclassification (every 100 ticks)
        if tick_u64.is_multiple_of(100) && tick_u64 > 0 {
            for (idx, agent) in self.agents.iter_mut().enumerate() {
                let agent_id = mindstrata_core::id::AgentId::new(idx as u64);
                let has_role = self
                    .institutions
                    .iter()
                    .any(|inst| inst.has_member(agent_id));
                let emotional_intensity =
                    agent.emotions.fear + agent.emotions.anger + agent.emotions.joy;
                agent.agent_tier.update_narrative_importance(
                    has_role,
                    agent.relationship_v2s.len(),
                    emotional_intensity,
                    0,
                );
                agent.agent_tier.reclassify(
                    agent.status.effective,
                    has_role,
                    false, // is_faction_leader — would need to check faction leadership
                    agent.emotions.fear,
                    agent.emotions.anger,
                    agent.relationship_v2s.len(),
                    tick_u64,
                    100, // min reassign interval
                );
            }
        }
        for agent in &mut self.agents {
            // §13.3: Wealth → psychology — poverty produces fear, and relative
            // deprivation produces resentment. Previously agents could be taxed
            // down to 0.39 coins with zero emotional feedback, so wealth loss
            // never registered psychologically.
            let poverty = (Fixed::from_f64(3.0) - agent.wealth.coin).max(Fixed::ZERO);
            if poverty > Fixed::ZERO {
                let poverty_fear = (poverty * Fixed::from_f64(0.05)).clamp_01();
                agent.emotions.fear = (agent.emotions.fear + poverty_fear).clamp_01();
                agent.derived.resentment =
                    (agent.derived.resentment + poverty * Fixed::from_f64(0.02)).clamp_01();
            }

            // §17: Background agents skip derived states — use simple decay instead
            if !agent.agent_tier.tier.runs_belief_updates() {
                continue;
            }
            let stress = agent.emotions.fear + agent.emotions.anger;
            // §8.1: Interoception filters what the agent *feels* of its raw need
            // deficit — anxious (high negative-bias) agents register the same body
            // state as more dire, feeding higher depression risk downstream.
            // Mean-zero at the default interoceptive configuration.
            let need_deficit_avg = agent.interoception.felt_need_deficit(
                agent.needs.hunger,
                agent.needs.thirst,
                agent.needs.fatigue,
                agent.needs.safety,
            );
            let social_support = Fixed::ONE - agent.needs.social;
            let success_rate = if agent.recent_attempts > 0 {
                Fixed::from_int(agent.recent_successes as i64)
                    / Fixed::from_int(agent.recent_attempts as i64)
            } else {
                Fixed::from_f64(0.5)
            };
            // §8.1.7 -> §22: Outrage at witnessed violations of sacred values
            // erodes perceived justice — the world feels less fair when what an
            // agent holds sacred is violated — feeding derived.resentment.
            // Mean-zero at zero outrage (fairness unchanged for calm agents).
            let justice_perception = (agent.moral_values.fairness
                - agent.moral_cognition.moral_emotions.outrage * Fixed::from_f64(0.3))
            .clamp_01();
            // §5.1: Use configurable smoothing/accumulation from SimParameters
            agent.derived.compute(
                &crate::person::MentalStateInput {
                    stress,
                    coping_potential: agent.personality.conscientiousness,
                    social_support,
                    need_deficit_avg,
                    meaning: agent.needs.meaning,
                    autonomy: agent.needs.autonomy,
                    success_rate,
                    neuroticism: agent.personality.neuroticism,
                    justice_perception,
                },
                self.params.mental_state_smoothing,
                self.params.mental_state_accumulation,
            );
            // Decay success tracking window
            agent.recent_attempts = agent.recent_attempts.saturating_sub(1);
            agent.recent_successes = agent.recent_successes.saturating_sub(1);
        }

        // ── 13. Belief updates from this tick's social interactions ────
        for ev in &self.events[pre_tick_events..] {
            if let SimEvent::InteractionOccurred { from, to, .. } = ev {
                let from_idx = from.as_u64() as usize;
                let to_idx = to.as_u64() as usize;
                if from_idx < self.agents.len() && to_idx < self.agents.len() {
                    let trust = self
                        .relationships
                        .iter()
                        .find(|r| r.from == *from && r.to == *to)
                        .map_or(Fixed::from_f64(0.5), |r| r.trust);

                    let evidence_strength = trust - Fixed::from_f64(0.5);
                    let source_trust = Fixed::from_f64(0.6);

                    // §10.1.3 (Iteration 109): the noospheric field's mean
                    // belief confidence scales update resistance — the
                    // dogmatism factor (identity at 1.0 when the agent
                    // holds no beliefs, since the field reads zero there).
                    // Read before the mutable `beliefs` borrow.
                    let rigidity_factor =
                        crate::social::relational_field::RelationalFields::belief_rigidity_factor(
                            self.agents[from_idx].relational_fields.belief_confidence,
                            Fixed::from_f64(
                                crate::social::relational_field::BELIEF_CONFIDENCE_RIGIDITY,
                            ),
                        );
                    // §8.1.18 (Iteration 168): the agent's cultural
                    // change-resistance dampens the update evidence — a
                    // conservative/taboo-bound culture absorbs less
                    // counter- and confirming evidence (sacred boundary
                    // defense on the belief layer). ONE-SIDED factor
                    // (1 − resistance × 0.3) floored at 0.8; near-uniform
                    // ~0.55 resistance → a ~17% standing dampening in
                    // calibrated windows (honestly framed as uniform — the
                    // differential needs a resistance spread the current
                    // seeding does not produce). Read before the mutable
                    // `beliefs` borrow; deterministic, zero new RNG.
                    let cultural_dampening = (Fixed::ONE
                        - self.agents[from_idx].cultural_cognition.change_resistance()
                            * BELIEF_CULTURAL_RATE)
                        .max(BELIEF_CULTURAL_FLOOR)
                        .clamp_01();

                    // §9.2 (Iteration 183d): LARGE prediction error updates
                    // beliefs — surprise amplifies the emotional
                    // reinforcement of the evidence (an agent whose world
                    // model is strongly violated moves its beliefs more per
                    // interaction). Gated at 0.3 (zero prediction error in
                    // calm windows → byte-identical); at 0.3 gain a 0.3
                    // surprise adds 0.09 reinforcement — a modest honest
                    // nudge on top of the evidence term.
                    let from_surprise = self.agents[from_idx]
                        .neural_like
                        .expectation
                        .last_prediction_error;
                    let emotional_reinforcement = if from_surprise > Fixed::from_f64(0.3) {
                        from_surprise * Fixed::from_f64(0.3)
                    } else {
                        Fixed::ZERO
                    };

                    belief_update::update_beliefs(
                        &mut self.agents[from_idx].beliefs,
                        &[(2, evidence_strength, source_trust)],
                        emotional_reinforcement,
                        Fixed::ZERO,
                        tick_u64,
                        &self.params,
                        rigidity_factor,
                        cultural_dampening,
                    );
                }
            }
        }

        // §7.2 (Iteration 241): Lived-experience emotional reinforcement of
        // institution-directed beliefs — the missing production half of the
        // moral-panic pipeline. `detect_moral_panic` requires avg belief
        // charge >= 0.55 across propositions 0/1 ("the market is fair",
        // "the council protects us"), but NOTHING systematically charged
        // them: beliefs are born at charge 0, `update_belief` never touches
        // emotional_charge, and the only live writer (echo-chamber feedback)
        // adds ~5e-4/day — probed equilibrium ~0.10–0.13 vs the 0.55
        // trigger, so panics stopped firing in every anchored window.
        //
        // The realistic mechanism: chronic distress heats grievance
        // narratives. Each day, an agent's emotional climate (fear +
        // negative valence + anger) drives its institution-directed beliefs
        // toward a distress-proportional equilibrium (eq = 1.2 x distress:
        // calm ~0.32 -> eq ~0.38, safely sub-threshold; epidemic distress
        // ~0.63 -> eq ~0.76, panic territory); sustained calm cools them
        // again. Daily increments (~1e-2..1e-3) clear the Fixed quantum;
        // deterministic, zero RNG.
        if tick_u64.is_multiple_of(144) && tick_u64 > 0 {
            const CHARGE_SCALE: f64 = 0.075;
            const CHARGE_DECAY: f64 = 0.05;
            for agent in &mut self.agents {
                let distress = (0.6 * agent.emotions.fear.to_f64()
                    + 0.4 * (-agent.affect.valence.to_f64()).max(0.0)
                    + 0.2 * agent.emotions.anger.to_f64())
                .clamp(0.0, 1.0);
                for b in agent.beliefs.iter_mut().filter(|b| b.proposition_id <= 1) {
                    let c = b.emotional_charge.to_f64();
                    let next = c + CHARGE_SCALE * distress - CHARGE_DECAY * c;
                    b.emotional_charge = Fixed::from_f64(next).clamp_01();
                }
            }
        }

        // §10.1: Refresh each agent's three relational fields (sensory /
        // social / noospheric) on the daily cadence.
        self.refresh_relational_fields();
    }
}
