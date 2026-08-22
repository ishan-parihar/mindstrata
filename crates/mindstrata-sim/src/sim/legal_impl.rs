//! Theft enforcement, council judgement and prosecution.

use super::{
    AgentId, Fixed, JournalEntryKind, LegalCase, RngStream, SimEvent, Simulation, Tick, Verdict,
    GRAIN_RESOURCE_ID,
};
use crate::norms;

use rand::Rng;
impl Simulation {
    /// §19.5.D: Detect and punish theft — consumes resource, fines agent,
    /// records norm violation with enforcement probability, emits event, reduces trust with owner.
    /// Returns (taken > 0, was_caught): theft succeeded and whether it was detected.
    pub(super) fn enforce_theft(
        &mut self,
        agent_idx: usize,
        agent_id: AgentId,
        site_idx: usize,
        resource_id: u64,
        amount: Fixed,
        tick_u64: u64,
        tick: Tick,
    ) -> bool {
        let owner = self.world.sites[site_idx].owner;
        // §8.1.10/§19.5.D (Iteration 84): an agent who has internalized the
        // no-theft norm takes less — the norm's strength scales the amount
        // taken continuously (no threshold cliff). At full internalization the
        // scaled amount reaches zero and the agent refuses the theft outright
        // (early return: nothing consumed, no enforcement run). Resolved by id
        // (`NO_THEFT_NORM_ID`, the same constant the check_violation site
        // uses) so a scenario that re-registers norms with renamed
        // descriptions still gates correctly; a registry without the norm
        // resolves to zero resistance (legacy behavior). Zero-at-zero: before
        // the first monthly ritual (tick 4320) no agent holds any internalized
        // norm, so resistance = 0 and the golden baseline stays byte-identical.
        // The gate draws no RNG — the enforcement detection roll's stream
        // position is unchanged whenever a theft still occurs.
        let no_theft_name = self
            .norms
            .norms()
            .iter()
            .find(|n| n.id == norms::NO_THEFT_NORM_ID)
            .map(|n| n.name.as_str());
        let resistance = no_theft_name.map_or(Fixed::ZERO, |name| {
            self.agents[agent_idx].moral_cognition.norm_resistance(name)
        });
        // §8.1.10 (Iteration 87): hypocrisy compounds the resistance gate —
        // an agent who has witnessed the no-theft norm enforced
        // (`enforcement_count`, populated by the Iteration-86 audit) and is
        // sensitive to hypocrisy resists the act further: "I have seen
        // people punished for this; doing it myself would make me a
        // hypocrite." `hypocrisy_factor` is zero-at-zero (no witnessed
        // enforcement before any caught theft → legacy take), continuous,
        // and saturating at 5 witnessed enforcements; it draws no RNG (the
        // enforcement detection roll's stream position is unchanged
        // whenever a theft still occurs). The two mechanisms (internal
        // conviction strength vs social-learning shame) compound
        // multiplicatively, and either at full weight refuses the theft
        // outright.
        let hypocrisy = no_theft_name.map_or(Fixed::ZERO, |name| {
            self.agents[agent_idx]
                .moral_cognition
                .hypocrisy_factor(name)
        });
        // §10.1.3 (Iteration 111): the noospheric field's perceived
        // legitimacy deters theft — an agent who believes the institution
        // rules rightfully takes less ("the grain belongs to a legitimate
        // authority"). ONE-SIDED: identity at/below the 0.5 construction
        // anchor, so a merely-default institution deters nothing; the
        // consumer activates only when legitimacy is genuinely earned above
        // the baseline (rituals, propaganda, institutional strength).
        // Zero-blast by construction: legitimacy decays to ~0.04 in every
        // calibrated window, so the factor is exactly 1.0 throughout the
        // golden/snapshot horizons. The gate draws no RNG — the enforcement
        // detection roll's stream position is unchanged whenever a theft
        // still occurs.
        // The three `Fixed` conversions run once per theft attempt — and
        // theft itself never fires in calibrated windows (0 NormViolated at
        // every horizon, probe-pinned), so the cost is even rarer than the
        // Iter-110 escalation path; the shared Fixed-domain helper's
        // unit-test precision wins over hoisting to file-scope constants.
        let legitimacy_factor =
            crate::social::relational_field::RelationalFields::legitimacy_deterrence_factor(
                self.agents[agent_idx]
                    .relational_fields
                    .legitimacy_perceived,
                Fixed::from_f64(crate::social::relational_field::LEGITIMACY_DETERRENCE_ANCHOR),
                Fixed::from_f64(crate::social::relational_field::LEGITIMACY_DETERRENCE_RATE),
                Fixed::from_f64(crate::social::relational_field::LEGITIMACY_DETERRENCE_CAP),
            );
        let amount =
            amount * (Fixed::ONE - resistance) * (Fixed::ONE - hypocrisy) * legitimacy_factor;
        if amount <= Fixed::ZERO {
            return false; // refused the theft — nothing taken, no enforcement
        }
        let taken = self.world.consume_resource(site_idx, resource_id, amount);
        // Early return if nothing was actually taken — don't run enforcement for zero-resource thefts
        if taken <= Fixed::ZERO {
            return false;
        }
        let resource_name = if resource_id == GRAIN_RESOURCE_ID {
            "grain"
        } else {
            "water"
        };

        // §19.5.D: Compute enforcement capacity from Council's enforcement capacity
        let enforcement = self.council_enforcement();

        // §4.4: Black market reduces detection probability for qualifying agents
        let agent_can_black_market = self
            .black_market
            .can_participate(&self.agents[agent_idx].personality);
        let effective_enforcement = if agent_can_black_market {
            (enforcement * Fixed::from_f64(0.5)).max(Fixed::ZERO) // black market halves enforcement
        } else {
            enforcement
        };

        // Generate detection roll
        let detection_roll = Fixed::from_f64(self.rng.get_mut(RngStream::Social).random::<f64>());

        // §19.5.D: Apply enforcement probability — not all thefts are caught
        let (_punishment, was_caught) = self.norms.check_violation_with_enforcement(
            norms::NO_THEFT_NORM_ID,
            agent_id,
            tick_u64,
            effective_enforcement,
            detection_roll,
        );

        if was_caught {
            // Theft was detected — apply fine (black market goods priced at premium)
            let base_price = if agent_can_black_market {
                self.black_market
                    .black_market_price(self.market.price(resource_id))
            } else {
                self.market.price(resource_id)
            };
            let fine = taken * base_price * Fixed::from_f64(2.0);
            self.agents[agent_idx].wealth.coin =
                (self.agents[agent_idx].wealth.coin - fine).max(Fixed::ZERO);
            self.journal.record(
                tick_u64,
                agent_id,
                JournalEntryKind::TheftDetected {
                    resource: resource_name.into(),
                    amount: taken.to_f64(),
                    fine: fine.to_f64(),
                },
            );
            self.events.push(SimEvent::NormViolated {
                agent: agent_id,
                norm_id: norms::NO_THEFT_NORM_ID,
                witnesses: Vec::new(),
                tick,
            });
            // §19.5.B: Record institutional enforcement provenance
            self.provenance
                .record_institutional(crate::provenance::InstitutionalTrace {
                    institution_name: "Council".into(),
                    tick: tick_u64,
                    decision_kind: "theft_enforcement".into(),
                    description: format!(
                        "{} stolen {} ({:.1} coins fine)",
                        agent_id.as_u64(),
                        resource_name,
                        fine.to_f64()
                    ),
                    affected: vec![agent_id],
                    success: true,
                });
            if let Some(owner_id) = owner {
                if let Some(rel) = self
                    .relationships
                    .iter_mut()
                    .find(|r| r.from == agent_id && r.to == owner_id)
                {
                    rel.trust = (rel.trust - Fixed::from_f64(0.2)).max(Fixed::ZERO);
                }
                self.agents[agent_idx].emotions.shame =
                    (self.agents[agent_idx].emotions.shame + Fixed::from_f64(0.1)).clamp_01();
            }
            // §8.1.10/§19.5.D (Iteration 86): a caught theft is public
            // enforcement — the Council fines the thief and the whole village
            // knows it. Every agent holding the no-theft norm witnesses the
            // community enforce it, incrementing their documented
            // `enforcement_count` (previously created at 0 by
            // `internalize_norm` and never written in production — the
            // witnessed-enforcement audit channel was dead). The violator is
            // included among the witnesses: they experience the punishment
            // most directly, so "witnessed enforcement" deliberately covers
            // both observation and direct experience of the public fine.
            // Resolved by id (`NO_THEFT_NORM_ID`, same convention as the
            // gate above); a registry without the norm is a no-op.
            // Observational: no production consumer reads
            // `enforcement_count`, so calibrated runs carry zero drift — and
            // the default world's farms are `AccessRight::Public`, so thefts
            // never fire there at all.
            let no_theft_name = self
                .norms
                .norms()
                .iter()
                .find(|n| n.id == norms::NO_THEFT_NORM_ID)
                .map(|n| n.name.as_str());
            if let Some(name) = no_theft_name {
                for bundle in &mut self.agents {
                    bundle.moral_cognition.record_witnessed_enforcement(name);
                }
            }
            // §5 (Iteration 149): the judicial layer — the caught theft is
            // prosecuted: the court files a case, weighs evidence
            // (deterministically, no RNG — an owned site is strong
            // evidence), and adds a supplemental court fine on a Guilty
            // verdict. The path executes only when a theft is caught
            // (probe-pinned zero in every calibrated window), so golden and
            // snapshots stay byte-identical. A self-theft (owner == thief)
            // is not a crime — no case is filed.
            if owner != Some(agent_id) {
                self.prosecute_violation(
                    norms::NO_THEFT_NORM_ID,
                    agent_id,
                    owner,
                    Some(site_idx),
                    fine,
                    tick_u64,
                );
            }
        }

        // §4.4: Track black market transaction volume
        if agent_can_black_market {
            self.black_market.transactions_this_tick += 1;
        }

        // §19.5.D: Theft succeeds regardless of detection — resource is consumed
        // But consequences only apply if caught
        taken > Fixed::ZERO
    }

    /// Sum of the Councils' enforcement capacity — the court's investigative
    /// power — capped at one. Shared by theft enforcement and prosecution.
    fn council_enforcement(&self) -> Fixed {
        self.institutions
            .iter()
            .filter(|i| i.kind == crate::institutions::InstitutionKind::Council)
            .map(|i| i.enforcement_capacity)
            .fold(Fixed::ZERO, |a, b| a + b)
            .min(Fixed::ONE)
    }

    /// §5 (Iteration 149): The court's entry point — prosecute a violation.
    /// Files a `LegalCase`, weighs evidence deterministically (Council
    /// enforcement × owned-site bonus × repeat-offender record — NO RNG is
    /// drawn, so adjudication cannot perturb any subsystem's stream), applies
    /// the supplemental court fine on a Guilty verdict, and records the
    /// outcome in the journal and the provenance store. Public so the
    /// integration tests can drive the court directly.
    pub fn prosecute_violation(
        &mut self,
        norm_id: u64,
        accused: AgentId,
        victim: Option<AgentId>,
        site_idx: Option<usize>,
        base_fine: Fixed,
        tick_u64: u64,
    ) -> Option<LegalCase> {
        let owned_site = site_idx.is_some_and(|si| self.world.sites[si].owner.is_some());
        let enforcement = self.council_enforcement();
        let case = self.legal.prosecute(
            norm_id,
            accused,
            victim,
            site_idx,
            owned_site,
            enforcement,
            base_fine,
            tick_u64,
        );
        if case.verdict == Some(Verdict::Guilty) && case.sentence > Fixed::ZERO {
            // The `AgentId::new(i) == index i` invariant (documented at
            // AgentBundle) lets us map the accused id straight to its slot.
            let idx = accused.as_u64() as usize;
            if idx < self.agents.len() {
                self.agents[idx].wealth.coin =
                    (self.agents[idx].wealth.coin - case.sentence).max(Fixed::ZERO);
            }
            self.provenance
                .record_institutional(crate::provenance::InstitutionalTrace {
                    institution_name: "Council".into(),
                    tick: tick_u64,
                    decision_kind: "court_verdict".into(),
                    description: format!(
                        "case {}: agent {} guilty (evidence {:.2}) — {:.2} coin court fine",
                        case.case_id,
                        accused.as_u64(),
                        case.evidence_strength.to_f64(),
                        case.sentence.to_f64()
                    ),
                    affected: vec![accused],
                    success: true,
                });
        }
        // The verdict is journaled to the accused whether guilty or innocent
        // — justice is fully observable.
        self.journal.record(
            tick_u64,
            accused,
            JournalEntryKind::LegalVerdict {
                case_id: case.case_id,
                guilty: case.verdict == Some(Verdict::Guilty),
                sentence: case.sentence.to_f64(),
            },
        );
        Some(case)
    }
}
