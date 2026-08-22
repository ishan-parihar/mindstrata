//! Clan alliances, enmities and dominance escalation.

use super::{
    Fixed, RngStream, Simulation, VIOLENCE_TABOO_AVERSION_FLOOR, VIOLENCE_TABOO_AVERSION_RATE,
};
use crate::norms;

use rand::Rng;
impl Simulation {
    /// §10.8: Clan membership lookup — which clan holds this agent index
    /// (core_households stores agent indices). Deterministic registry scan.
    pub(super) fn clan_of(&self, agent_idx: usize) -> Option<usize> {
        self.clan_registry
            .clans
            .iter()
            .find(|c| c.core_households.contains(&agent_idx))
            .map(|c| c.id)
    }

    /// §10.8: Marriage forges a clan alliance — the design doc's stated
    /// alliance source. Symmetric (both clans declare each other);
    /// declare_ally dedupes and refuses existing enemies.
    pub(super) fn forge_clan_alliance(&mut self, a: usize, b: usize, tick: u64) {
        let ca = match self.clan_of(a) {
            Some(c) => c,
            None => return,
        };
        let cb = match self.clan_of(b) {
            Some(c) => c,
            None => return,
        };
        if ca == cb {
            return;
        }
        for clan in &mut self.clan_registry.clans {
            if clan.id == ca {
                clan.declare_ally(cb);
                clan.last_interaction_tick = tick;
            } else if clan.id == cb {
                clan.declare_ally(ca);
                clan.last_interaction_tick = tick;
            }
        }
    }

    /// §10.8: Repeated violence forges a clan enmity (feud boundary) and
    /// breaks any existing alliance between the pair. Symmetric.
    pub(super) fn forge_clan_enmity(&mut self, a: usize, b: usize, tick: u64) {
        let ca = match self.clan_of(a) {
            Some(c) => c,
            None => return,
        };
        let cb = match self.clan_of(b) {
            Some(c) => c,
            None => return,
        };
        if ca == cb {
            return;
        }
        for clan in &mut self.clan_registry.clans {
            if clan.id == ca {
                clan.allies.retain(|&x| x != cb);
                clan.declare_enemy(cb);
                clan.last_interaction_tick = tick;
            } else if clan.id == cb {
                clan.allies.retain(|&x| x != ca);
                clan.declare_enemy(ca);
                clan.last_interaction_tick = tick;
            }
        }
    }

    /// §10.8: Are two agents in mutually enemy clans? (Symmetric by
    /// construction — forge_clan_enmity declares both ways.)
    pub(super) fn clans_are_enemies(&self, a: usize, b: usize) -> bool {
        let (Some(ca), Some(cb)) = (self.clan_of(a), self.clan_of(b)) else {
            return false;
        };
        if ca == cb {
            return false;
        }
        self.clan_registry
            .clans
            .iter()
            .any(|c| c.id == ca && c.is_enemy(cb))
    }

    /// §10.8: Are two agents in mutually allied clans? (Symmetric by
    /// construction — forge_clan_alliance declares both ways.)
    pub(super) fn clans_are_allies(&self, a: usize, b: usize) -> bool {
        let (Some(ca), Some(cb)) = (self.clan_of(a), self.clan_of(b)) else {
            return false;
        };
        if ca == cb {
            return false;
        }
        self.clan_registry
            .clans
            .iter()
            .any(|c| c.id == ca && c.is_ally(cb))
    }

    /// §10.8/§19.5.G: When every feud that forged a clan enmity has decayed
    /// (no active feud between any member of the two clans), the enmity
    /// clears — feuds can heal into peace, and later marriages can forge an
    /// alliance. Deterministic: clans and agents are scanned in registry
    /// order, no RNG.
    pub(super) fn decay_clan_enmities(&mut self) {
        let n = self.clan_registry.clans.len();
        for i in 0..n {
            for j in (i + 1)..n {
                if !self.clan_registry.clans[i].is_enemy(j) {
                    continue;
                }
                // Any active feud between a member of clan i and clan j?
                let active = self.agents.iter().enumerate().any(|(idx, agent)| {
                    if self.clan_of(idx) != Some(i) {
                        return false;
                    }
                    agent.feuds.iter().any(|&f| self.clan_of(f) == Some(j))
                });
                if active {
                    continue;
                }
                if let Some(clan) = self.clan_registry.clans.iter_mut().find(|c| c.id == i) {
                    clan.clear_enemy(j);
                }
                if let Some(clan) = self.clan_registry.clans.iter_mut().find(|c| c.id == j) {
                    clan.clear_enemy(i);
                }
            }
        }
    }

    /// §19.5.H/§10.8: Escalation chance after a failed threat. Enemy clans
    /// escalate at twice the base rate — a feud is a standing state of war,
    /// so deterrence between enemy clans fails roughly twice as often.
    /// (Capped at 1.0; keeps the 12-agent-village collective-action
    /// calibration intact: certainty would let one feud militarize the whole
    /// village and suppress protest — see factions_emerge_from_grievance.)
    pub(super) fn escalation_chance(&self, from_idx: usize, to_idx: usize) -> f64 {
        if self.clans_are_enemies(from_idx, to_idx) {
            (self.params.conflict_escalation_chance.to_f64() * 2.0).min(1.0)
        } else {
            self.params.conflict_escalation_chance.to_f64()
        }
    }

    /// §10.2 (Iteration 102): Continuous dominance scale folded into the
    /// escalation chance. `power_balance` (directed, from→to, ∈ [-1, 1]) is
    /// mapped onto [0.5, 1.5] — identity at zero, so pairs without a
    /// relationship record (or a perfectly balanced one) keep the legacy
    /// chance. Monotone: a dominant aggressor escalates a failed threat
    /// more readily ("I can win this"), a subordinate backs down ("picking
    /// a fight with them would go badly"). Pure function of the field — no
    /// RNG, deterministic.
    #[inline]
    pub(super) fn dominance_escalation_scale(power_balance: Fixed) -> f64 {
        (1.0 + power_balance.to_f64() * 0.5).clamp(0.5, 1.5)
    }

    /// §19.5.H/§10.8: Whether a failed threat escalates to violence. The RNG
    /// draw happens under exactly the same conditions as the original inline
    /// logic, so the stream is untouched (replay determinism).
    pub(super) fn should_escalate(
        &mut self,
        from_idx: usize,
        to_idx: usize,
        threat_failed: bool,
        aggressor_aggression: Fixed,
    ) -> bool {
        if !threat_failed
            || aggressor_aggression <= self.params.conflict_escalation_aggression_threshold
        {
            return false;
        }
        // §8.1.10: An agent who has internalized the no-violence norm resists
        // escalating a failed threat to physical violence — the norm's strength
        // scales the escalation chance continuously (no threshold cliff).
        // The norm is resolved by id (`NO_VIOLENCE_NORM_ID`, the same constant
        // the check_violation sites use) so a scenario that re-registers norms
        // with renamed descriptions still gates correctly; a registry without
        // the norm resolves to zero resistance (legacy behavior).
        // Zero-at-zero: before the first monthly ritual (tick 4320) no agent
        // holds any internalized norm, so resistance = 0 and the golden
        // baseline stays byte-identical. The RNG draw remains unconditional
        // (same stream position), so replay determinism holds at every
        // resistance value.
        let no_violence_name = self
            .norms
            .norms()
            .iter()
            .find(|n| n.id == norms::NO_VIOLENCE_NORM_ID)
            .map(|n| n.name.as_str());
        let resistance = no_violence_name.map_or(0.0, |name| {
            self.agents[from_idx]
                .moral_cognition
                .norm_resistance(name)
                .to_f64()
        });
        // §8.1.10 (Iteration 88): hypocrisy compounds the resistance gate —
        // an agent who has witnessed the no-violence norm enforced
        // (`enforcement_count`, populated by the Iteration-88 violence
        // audit — violence is inherently public, unlike sneaky theft) is
        // additionally restrained: "I have seen this punished; doing it
        // myself would make me a hypocrite." Zero-at-zero (no witnessed
        // enforcement before the audit's holders exist → legacy chance),
        // continuous, no cliff, and no extra RNG (the draw below stays
        // unconditional — only the comparison threshold changes).
        let hypocrisy = no_violence_name.map_or(0.0, |name| {
            self.agents[from_idx]
                .moral_cognition
                .hypocrisy_factor(name)
                .to_f64()
        });
        // §8.1.18 (Iteration 169): the Violence taboo is a PRE-COMMITMENT
        // cultural brake — an agent whose culture forbids violence hesitates
        // before escalating a failed threat ("my people forbid this"). The
        // taboo-resolution helper `tabo_violated_by("violence")` finds the
        // forbidden set (case-insensitive substring on description); the
        // summed violation_cost scales the chance via the ONE-SIDED
        // aversion factor (1 − cost × rate, floored). This is the
        // behavioral counterpoint to the Iter-167 shame boost on the same
        // decision family: Iter-167 amplifies the shame AFTER a violent act,
        // this dampens the willingness BEFORE it. Deterministic, zero new
        // RNG — the draw below stays unconditional (same stream position),
        // so replay determinism holds at every aversion value — only the
        // comparison threshold changes. Identity at zero: taboo-free agents
        // (pre-Iter-165 snapshot restores) keep the legacy chance exactly.
        let violence_taboo_cost = self.agents[from_idx]
            .cultural_cognition
            .taboo_violation_cost_sum("violence");
        let taboo_aversion = (Fixed::ONE - violence_taboo_cost * VIOLENCE_TABOO_AVERSION_RATE)
            .max(VIOLENCE_TABOO_AVERSION_FLOOR)
            .clamp_01();
        // §10.2 (Iteration 102): Relational dominance feeds the escalation
        // decision. `power_balance` (directed, from→to) was updated on the
        // daily boundary (tick % 144) but had zero consumers — a write-only
        // field. The aggressor's directed power over the target scales the
        // chance continuously (0.5× subordinate → 1.5× dominant); pairs
        // without a relationship record resolve to zero → scale 1.0 →
        // legacy behavior. The RNG draw below stays unconditional (same
        // stream position), so replay determinism holds at every
        // power-balance value — only the comparison threshold changes.
        let dominance_scale = {
            let pos = Self::relationship_v2_pos(from_idx, to_idx);
            let power_balance = self.agents[from_idx]
                .relationship_v2s
                .get(pos)
                .map_or(Fixed::ZERO, |r| r.power_balance);
            Self::dominance_escalation_scale(power_balance)
        };
        // §10.1.2 (Iteration 110): The social field's mean trust pacifies
        // the escalation decision — an agent embedded in a trusting
        // relationship graph escalates a failed threat to violence less
        // readily ("the social fabric restrains me"). The aggressor's own
        // mean `social_trust` (refreshed daily by refresh_relational_fields,
        // zero at tick 0 → factor 1.0) scales the chance continuously
        // (0.3/unit trust, capped at 0.6 so the pacification never erases
        // the escalation chance entirely). The RNG draw below stays
        // unconditional (same stream position), so replay determinism holds
        // at every trust value — only the comparison threshold changes.
        // The two `Fixed` conversions run once per failed-threat escalation
        // opportunity (not per-tick-per-agent — orders of magnitude below the
        // Iter-108 kin fold), so the shared Fixed-domain helper's unit-test
        // precision wins over hoisting them to file-scope computed constants.
        let trust_factor = crate::social::relational_field::RelationalFields::trust_pacify_factor(
            self.agents[from_idx].relational_fields.social_trust,
            Fixed::from_f64(crate::social::relational_field::SOCIAL_TRUST_PACIFY_RATE),
            Fixed::from_f64(crate::social::relational_field::SOCIAL_TRUST_PACIFY_CAP),
        );
        // §10.1.2 (Iteration 114): the social field's mean obligation also
        // pacifies the escalation decision — an agent bound by deep
        // reciprocal obligations escalates a failed threat to violence less
        // readily ("I owe this web; attacking dishonors my debts"). This is
        // the SECOND §19.5.H pacifier alongside the trust factor above, but
        // a distinct semantic layer: trust is relational confidence ("I
        // believe they won't harm me"), obligation is moral constraint ("I
        // have duties I must honor") — §10.1.2 lists both as social-field
        // components. ONE-SIDED at the 0.5 anchor — identity in every
        // golden/snapshot horizon (seed-42 max obligation 0.456@5000,
        // proven by the byte-identical gates), so the golden/snapshot
        // windows are untouched (zero-blast — no regeneration); deep-debt worlds cross it by
        // design (seed-42 mean obligation reaches 0.69@20K), activating the
        // restraint. The RNG draw below stays unconditional (same stream
        // position), so replay determinism holds at every obligation value
        // — only the comparison threshold changes. The `Fixed` conversions
        // run once per failed-threat escalation opportunity (same cost
        // class as the Iter-110 trust factor).
        let obligation_factor =
            crate::social::relational_field::RelationalFields::obligation_restraint_factor(
                self.agents[from_idx].relational_fields.social_obligation,
                Fixed::from_f64(crate::social::relational_field::OBLIGATION_RESTRAINT_ANCHOR),
                Fixed::from_f64(crate::social::relational_field::OBLIGATION_RESTRAINT_RATE),
                Fixed::from_f64(crate::social::relational_field::OBLIGATION_RESTRAINT_CAP),
            );
        // §8.1.4 (Iteration 116): a humiliated agent escalates a failed
        // threat to violence MORE readily — `humiliation_escalation_factor`
        // (1 + humiliation × rate, factor 1.30 at full humiliation with the
        // shipped constant) multiplies the chance chain. This is the
        // AMPLIFIER counterpoint to the Iter-110 trust / Iter-114 obligation
        // pacifiers on the same §19.5.H decision. ONE-SIDED: identity at
        // zero — the calibration probe shows humiliation is never produced
        // in calibrated windows (its appraisal inputs — status threat +
        // identity relevance — don't fire in calm worlds; probe mean/max
        // 0.0000 across seeds and scenarios), so the factor is exactly 1.0
        // throughout the golden/snapshot horizons (provably zero-blast). The
        // RNG draw below stays unconditional (same stream position), so
        // replay determinism holds at every humiliation value — only the
        // comparison threshold changes.
        let humiliation_factor = crate::appraisal::humiliation_escalation_factor(
            self.agents[from_idx].emotions.humiliation,
            crate::appraisal::HUMILIATION_ESCALATION_RATE,
        );
        // §8.1.4 (Iteration 122): a contemptuous aggressor escalates a
        // failed threat to violence MORE readily — `contempt_escalation_factor`
        // (1 + contempt × rate, factor 1.30 at full contempt) multiplies the
        // chance chain. This is the SECOND AMPLIFIER alongside the Iter-116
        // humiliation factor on the same §19.5.H decision, from a distinct
        // semantic layer: humiliation is public status loss ("I was shamed"),
        // contempt is secure superiority ("the target is beneath me — no
        // restraint needed"). ONE-SIDED: identity at zero — the calibration
        // probe shows contempt is never produced in calibrated windows (its
        // appraisal inputs — `(1 − status_threat) × unfair` — don't fire in
        // calm worlds; probe mean/max 0.0000 across seeds and scenarios), so
        // the factor is exactly 1.0 throughout the golden/snapshot horizons
        // (provably zero-blast). The RNG draw below stays unconditional
        // (same stream position), so replay determinism holds at every
        // contempt value — only the comparison threshold changes. The
        // combined-amplifier ceiling is clamped at 1.0 by
        // `escalation_chance` (the enemy-clan tests exercise the same
        // clamp), so even both amplifiers (humiliation × contempt) at
        // saturation cannot exceed it.
        let contempt_factor = crate::appraisal::contempt_escalation_factor(
            self.agents[from_idx].emotions.contempt,
            crate::appraisal::CONTEMPT_ESCALATION_RATE,
        );
        // §8.1.4 (Iteration 122): a despairing aggressor escalates a failed
        // threat to violence LESS readily — `despair_pacify_factor`
        // (1 − despair × rate floored at 0.5, factor 0.75 at half despair,
        // 0.5 at full) multiplies the chance chain. This is the PACIFIER
        // counterpoint to the Iter-116/122 amplifiers on the same §19.5.H
        // decision, from the distinct layer of hopelessness ("nothing I do
        // matters — violence cannot change this"); the floor guarantees it
        // never fully erases the escalation chance (same never-zero design
        // as the Iter-110 trust / Iter-114 obligation pacifiers). ONE-SIDED:
        // identity at zero — the calibration probe shows despair is never
        // produced in calibrated windows (its appraisal inputs —
        // `future_negative × (1 − coping_potential)` — don't fire in calm
        // worlds; probe mean/max 0.0000 across seeds and scenarios), so the
        // factor is exactly 1.0 throughout the golden/snapshot horizons
        // (provably zero-blast). The RNG draw below stays unconditional
        // (same stream position), so replay determinism holds at every
        // despair value — only the comparison threshold changes. The
        // multipliers compose multiplicatively on this chain: a
        // simultaneously contemptuous AND despairing aggressor nets
        // ×(1.3 × 0.5) = 0.65, i.e. despair dominates the decision
        // (hopelessness demobilizes even the secure-superior). The
        // violent-despair counter-hypothesis (nothing-to-lose violence) is
        // acknowledged and left as a future calibration knob — the
        // rate/floor consts make it trivially tunable.
        // §8.1.4 (Iteration 125): a morally outraged aggressor escalates a
        // failed threat to violence MORE readily — `moral_outrage_escalation_factor`
        // (1 + outrage × rate, factor 1.30 at full outrage) multiplies the
        // chance chain. This is the THIRD AMPLIFIER alongside the Iter-116
        // humiliation and Iter-122 contempt factors on the same §19.5.H
        // decision, from the distinct righteous-indignation layer ("the
        // violation of the sacred demands retaliation" — the AP2 §8.1.10
        // honor-feud path): humiliation is public status loss, contempt is
        // secure superiority, outrage is moral condemnation of the target's
        // act. ONE-SIDED: identity at zero — the Iter-125 calibration
        // probe shows moral_outrage is never produced in calibrated windows
        // (its appraisal input — `sacredness_violation × goal_relevance`
        // = `witnessed_unfairness × max_sacredness × max(hunger, thirst,
        // threat)` — fires at ANY positive witnessed unfairness: there is
        // NO 0.05 gate on this channel — that threshold only flips the
        // sign of the separate `fairness` appraisal field — so the
        // exact-zero probe pins `witnessed_unfairness == 0` exactly, i.e.
        // calm worlds witness no injustice; probe mean/max 0.0000 across
        // seeds 1/42/99 at 1000–5000 ticks despite every agent carrying
        // live sacred values 0.55–0.79 AND live goal_relevance — the
        // channel is ARMED, the zero is the unfairness input, not absent
        // machinery), so the factor is exactly 1.0 throughout the
        // golden/snapshot horizons (provably zero-blast).
        // The RNG draw below stays unconditional (same stream position), so
        // replay determinism holds at every outrage value — only the
        // comparison threshold changes. The combined-amplifier ceiling is
        // clamped at 1.0 by `escalation_chance`, so even all three
        // amplifiers at saturation cannot exceed it.
        let outrage_factor = crate::appraisal::moral_outrage_escalation_factor(
            self.agents[from_idx].emotions.moral_outrage,
            crate::appraisal::MORAL_OUTRAGE_ESCALATION_RATE,
        );
        let despair_factor = crate::appraisal::despair_pacify_factor(
            self.agents[from_idx].emotions.despair,
            crate::appraisal::DESPAIR_PACIFY_RATE,
            crate::appraisal::DESPAIR_PACIFY_FLOOR,
        );
        // §8.1.4 (Iteration 128): a relieved aggressor escalates a failed
        // threat to violence MORE readily — `relief_escalation_factor`
        // (1 + relief × 0.1, factor 1.10 at full) multiplies the chance
        // chain. The emotional counterpoint to the Iter-122 despair
        // pacifier on the same §19.5.H decision, from the distinct
        // recovery layer ("the uncontrollable outcome turned out positive
        // — I survived, I am emboldened"): despair is hopelessness,
        // relief is post-danger risk-acceptance (the lucky-escape
        // appraisal, producer `positive × (1 − controllability)`).
        // CALIBRATED: relief is LIVE in recovery windows (0.1-rate probe:
        // mean > 0.5 at 5000; factor ≈ 1.05–1.08 there), but the factor is
        // SELECTIVE — relief's producer (positive × (1 − controllability))
        // is dormant in famine/stress windows, so golden/snapshots stay
        // byte-identical while long-horizon emergent runs shift (seed-42
        // panic fire 17713 → 18577, Iter-128). Despair (Iter-122) is
        // anti-correlated: live in stress windows, zero in recovery — the
        // two factors never silently cancel in the same window. The RNG
        // draw below stays unconditional (same stream position), so
        // replay determinism holds at every relief value — only the
        // comparison threshold changes. The combined-amplifier ceiling is
        // clamped at 1.0 by `escalation_chance`.
        let relief_factor = crate::appraisal::relief_escalation_factor(
            self.agents[from_idx].emotions.relief,
            crate::appraisal::RELIEF_ESCALATION_RATE,
        );
        let chance = self.escalation_chance(from_idx, to_idx)
            * (1.0 - resistance)
            * (1.0 - hypocrisy)
            * taboo_aversion.to_f64()
            * dominance_scale
            * trust_factor.to_f64()
            * obligation_factor.to_f64()
            * humiliation_factor.to_f64()
            * contempt_factor.to_f64()
            * outrage_factor.to_f64()
            * relief_factor.to_f64()
            * despair_factor.to_f64();
        self.rng.get_mut(RngStream::Social).random::<f64>() < chance
    }
}
