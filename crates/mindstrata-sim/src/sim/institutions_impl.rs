//! Institutional psychology, theology, military and technology discovery ticks.

use super::{
    AgentId, Fixed, InstitutionKind, JournalEntryKind, MilitiaMember, RngStream, Simulation,
    Temperament, TheologicalBelief, ELDER_AGE, MUSTER_CAP,
};
use crate::factions;
use crate::institutions;
use crate::military::conscription_eligible;
use crate::military::drill_readiness;
use crate::theology::drift_conviction;
use crate::theology::seed_conviction;
use crate::theology::should_convert;

use rand::Rng;
impl Simulation {
    /// Section 12: Institutional collective psychology derivation.
    pub(super) fn tick_institutional_psychology(
        &mut self,
        tick_u64: u64,
        phases: crate::scheduler::TickPhases,
    ) {
        // ── 12. Institutional collective psychology derivation ────
        // §23: Derive collective psychology from member states each tick.
        //
        // §29.2: Compute the population's average grievance once per tick.
        // Council legitimacy must reflect popular sentiment. Before this fix,
        // the recovery target was max(morale, 0.3) with no grievance term — and
        // since morale = f(legitimacy) (a positive feedback loop), legitimacy
        // pinned at ~1.0 and the faction-formation trigger (legitimacy < 0.5)
        // was never armed even when 9/12 agents were above the grievance
        // threshold. An aggrieved population must withdraw legitimacy from the
        // council, which is what actually arms the faction trigger.
        let market_gini = self.market.inequality;
        let avg_grievance: Fixed = if self.agents.is_empty() {
            Fixed::ZERO
        } else {
            let sum: Fixed = self
                .agents
                .iter()
                .map(|a| {
                    factions::compute_grievance(
                        a.derived.resentment,
                        a.emotions.fear,
                        a.emotions.anger,
                        a.needs.autonomy,
                        a.needs.meaning,
                        a.moral_values.fairness,
                        market_gini,
                    )
                })
                .fold(Fixed::ZERO, |a, b| a + b);
            sum / Fixed::from_int(self.agents.len() as i64)
        };
        for institution in &mut self.institutions {
            let member_morales: Vec<Fixed> = institution
                .members
                .iter()
                .filter_map(|m| {
                    let idx = m.as_u64() as usize;
                    if idx < self.agents.len() {
                        Some(self.agents[idx].affect.valence)
                    } else {
                        None
                    }
                })
                .collect();
            let member_trusts: Vec<Fixed> = institution
                .members
                .iter()
                .filter_map(|m| {
                    let idx = m.as_u64() as usize;
                    if idx < self.agents.len() {
                        Some(self.agents[idx].emotions.trust)
                    } else {
                        None
                    }
                })
                .collect();
            institution.derive_collective_psychology(&member_morales, &member_trusts);
            // §26: Legitimacy is dynamic, not a one-way drain. Unreinforced / empty
            // institutions decay slowly, but functioning institutions (members present)
            // regain legitimacy toward their collective morale. Before this fix the
            // unconditional per-tick decay pinned council legitimacy at 0.000, which
            // kept the faction-formation trigger (`legitimacy < 0.5`) permanently armed.
            if institution.members.is_empty() {
                institution.decay_legitimacy(Fixed::from_f64(0.0001));
            } else {
                // §29.2: An aggrieved population withdraws legitimacy from the
                // council. Suppress the recovery target proportionally to average
                // grievance (scale 0.6: grievance 0.8 → target cut ~48%, enough
                // to arm the faction trigger). Only the Council is exposed to
                // popular grievance — it is the institution the faction trigger
                // reads and the one held accountable for governance. Note: in
                // practice high market inequality (Gini ≈ 0.75) keeps avg
                // grievance above 0.5 in most villages, so the council's
                // equilibrium legitimacy sits below 0.5 and factions form in
                // unequal settlements — which is the intended emergent signal.
                let mut target = if institution.kind == institutions::InstitutionKind::Council {
                    // §29.2 (Iteration 186): the council's legitimacy equilibrium
                    // must sit ABOVE the faction-formation gate (< 0.5) in
                    // ordinary villages. The old formula —
                    // `morale.max(0.3) × (1 − grievance×0.6)` — was tuned when
                    // grievance ran 0.82–0.88 (pre coin-sink-fix, poverty-
                    // ratcheted); once the Iteration-186 dividend closed the
                    // sink, calm/famine grievance settled at 0.55–0.64 and the
                    // SAME scale suppressed the target to ~0.27–0.46 —
                    // permanently below the gate — so the faction trigger was
                    // always armed and the post-revolution honeymoon became a
                    // fixed ~3,500-tick coup clock (probe: calm-42 = 28
                    // revolutions/100K at 3,515-tick gaps; calm-7, a genuinely
                    // happy village, churned 5 by 20K). The floor now maxes
                    // with 0.6 (an ordinary functioning council keeps a healthy
                    // mandate — a fresh mandate cannot read "barely tolerated")
                    // and the suppression scale drops to 0.25, so grievance
                    // must genuinely exceed ~0.67 (pestilence / collapse / a
                    // famine peak) to drag the equilibrium under 0.5.
                    // Revolutions become crisis-driven, not clock-driven.
                    (institution.collective.morale.max(Fixed::from_f64(0.6))
                        * (Fixed::ONE - avg_grievance * Fixed::from_f64(0.25)))
                    .clamp_01()
                } else {
                    (institution.collective.morale.max(Fixed::from_f64(0.3))).clamp_01()
                };
                // §7.3 (Iteration 186): post-revolution honeymoon — a fresh
                // regime that just seized power has a popular mandate: floor
                // its legitimacy target at 0.5 for a window so the faction
                // trigger (`legitimacy < 0.5`) stays disarmed while the new
                // regime governs. Pre-fix the 0.5 reset immediately decayed
                // back under 0.5 (~500 ticks, grievance still pinned high),
                // a new faction formed, and calm villages churned 42–129
                // revolutions per 100K. The honeymoon lets a regime actually
                // govern (and, with the Iteration-186 coin-sink fix, let
                // grievance genuinely decay) before the next crisis can arm.
                if institution.kind == institutions::InstitutionKind::Council
                    && tick_u64.saturating_sub(self.last_revolution_tick)
                        < crate::factions::REVOLUTION_HONEYMOON_TICKS
                {
                    target = target.max(Fixed::from_f64(0.5));
                }
                // §26: Converge symmetrically toward the target. The old code
                // rose at 0.002×gap but decayed at a fixed 0.0001/tick, so
                // legitimacy ratcheted to 1.0 early (before grievance builds)
                // and ritual/norm-enforcement boosts kept it pinned there —
                // the faction trigger (legitimacy < 0.5) never re-armed.
                let diff = target - institution.legitimacy;
                // Convergence rate: 0.01/tick ≈ 70-tick half-life. The old
                // 0.002 rate let additive boosts (norm enforcement +0.0025/violation,
                // ritual stabilization +0.02/centum) dominate the decay gradient,
                // so legitimacy ratcheted to 1.0 and the faction trigger never
                // re-armed. At 0.01 the grievance-suppressed target is actually
                // tracked instead of being overwhelmed.
                if diff > Fixed::ZERO {
                    institution.increase_legitimacy(diff * Fixed::from_f64(0.01));
                } else {
                    institution.decay_legitimacy(-diff * Fixed::from_f64(0.01));
                }
            }

            // §19.5.C: Process pending policies — move ready ones to active
            institution.process_policies(tick_u64);

            // §19.5.C: Council auto-proposes 'Fine Theft' policy when legitimacy is high
            // and enforcement capacity is moderate.
            if institution.kind == institutions::InstitutionKind::Council
                && institution.legitimacy > Fixed::from_f64(0.5)
                && institution.active_policies.is_empty()
                && institution.pending_policies.is_empty()
                && tick_u64 > institutions::INITIAL_PROPOSAL_DELAY
            {
                institution.propose_policy(
                    "Fine Theft Policy".into(),
                    Fixed::from_f64(0.1), // tax effect
                    tick_u64,
                );
            }

            // §19.5.C: Record active policy effects (check name first to avoid borrow conflict)
            let has_fine_theft = institution
                .active_policies
                .iter()
                .any(|p| p.name == "Fine Theft Policy");
            if has_fine_theft && phases.is_centum {
                let members = institution.members.clone();
                // Iteration 219: eliminate double-clone — move into record_action,
                // clone once for the provenance trace.
                let trace_affected = members.clone();
                institution.record_action(
                    tick_u64,
                    "Fine Theft Policy enacted".into(),
                    members,
                    true,
                );
                // §19.5.B: Record institutional provenance trace
                self.provenance
                    .record_institutional(crate::provenance::InstitutionalTrace {
                        institution_name: institution.kind.name().into(),
                        tick: tick_u64,
                        decision_kind: "policy_enacted".into(),
                        description: "Fine Theft Policy enacted".into(),
                        affected: trace_affected,
                        success: true,
                    });
            }

            // Trim old records to prevent unbounded growth (keep last 1000)
            if institution.records.len() > institutions::MAX_RECORDS {
                let drain_count = institution.records.len() - institutions::MAX_RECORDS;
                institution.records.drain(..drain_count);
            }

            // §19.5.C: Institutional tax collection — all institutions collect taxes
            if phases.is_centum && tick_u64 > 0 && !institution.members.is_empty() {
                let tax_rate = match institution.kind {
                    institutions::InstitutionKind::Council => {
                        Fixed::from_f64(institutions::COUNCIL_TAX_RATE)
                    }
                    institutions::InstitutionKind::Market => {
                        Fixed::from_f64(institutions::MARKET_FEE_RATE)
                    }
                    institutions::InstitutionKind::Temple => {
                        Fixed::from_f64(institutions::TEMPLE_TITHE_RATE)
                    }
                    _ => Fixed::ZERO,
                };
                if tax_rate > Fixed::ZERO {
                    let mut member_wealth: Vec<(AgentId, Fixed)> = institution
                        .members
                        .iter()
                        .filter_map(|m| {
                            let idx = m.as_u64() as usize;
                            if idx < self.agents.len() {
                                Some((*m, self.agents[idx].wealth.coin))
                            } else {
                                None
                            }
                        })
                        .collect();
                    let collected = institution.collect_taxes(tax_rate, &mut member_wealth);
                    // Write back tax deductions (AgentId::new(i) == index i)
                    for (agent_id, new_wealth) in &member_wealth {
                        let idx = agent_id.as_u64() as usize;
                        if idx < self.agents.len() {
                            self.agents[idx].wealth.coin = *new_wealth;
                        }
                    }
                    if collected > Fixed::ZERO {
                        let members = institution.members.clone();
                        let action_name = match institution.kind {
                            institutions::InstitutionKind::Temple => {
                                format!("Temple tithe: {:.1} coins", collected.to_f64())
                            }
                            institutions::InstitutionKind::Market => {
                                format!("Market merchant fee: {:.1} coins", collected.to_f64())
                            }
                            _ => {
                                format!("Council tax: {:.1} coins", collected.to_f64())
                            }
                        };
                        let inst_name = institution.kind.name().to_string();
                        // Iteration 219: eliminate double-clone.
                        let trace_affected = members.clone();
                        let trace_action = action_name.clone();
                        institution.record_action(tick_u64, action_name, members, true);
                        // §19.5.B: Record institutional provenance trace for tax collection
                        self.provenance.record_institutional(
                            crate::provenance::InstitutionalTrace {
                                institution_name: inst_name,
                                tick: tick_u64,
                                decision_kind: "tax_collection".into(),
                                description: trace_action,
                                affected: trace_affected,
                                success: true,
                            },
                        );
                    }
                }
            }

            // §19.5.C: All institutions pay role holders wages periodically
            if phases.is_quincent && tick_u64 > 0 {
                // Iteration 186: the Market is the village's trading commons —
                // recirculate its treasury surplus back to the village as a
                // dividend. Pre-fix the treasury was a coin SINK: consumption
                // payments accumulated thousands of coins (probe: 2138–4999 by
                // 10K) while agents sank below the 3-coin poverty line, the
                // poverty channel ratcheted fear/resentment, grievance hit
                // 0.82–0.88, council legitimacy was suppressed below the 0.5
                // faction-formation gate, and calm villages churned 42–129
                // revolutions per 100K (famine ACCIDENTALLY suppressed the
                // sink — fewer purchases, so agents kept coin and read calmer
                // than calm — inverting the intended scenario stress). Keep a
                // small operating reserve and pay the surplus out equally so
                // coin circulates and a thriving village stays solvent.
                if institution.kind == institutions::InstitutionKind::Market
                    && !self.agents.is_empty()
                {
                    let reserve = Fixed::from_f64(50.0);
                    let surplus = (institution.treasury - reserve).max(Fixed::ZERO);
                    if surplus > Fixed::ZERO {
                        let dividend = surplus * Fixed::from_f64(0.5);
                        // Iteration 186: PROGRESSIVE dividend. The first pass
                        // paid the surplus out equally, which PRESERVED the
                        // wealth ratio (everyone gets the same absolute amount,
                        // so the rich stay relatively rich — Gini barely moved:
                        // 0.89 → 0.75) and the inequality→grievance channel
                        // stayed pinned. Weight each share inversely to current
                        // wealth (w = 1/(1+coin)) so the surplus flows to the
                        // poorest: the wealth gap shrinks, Gini drops, and the
                        // `wealth_inequality` term in `compute_grievance`
                        // weakens. Deterministic: weights are a pure function
                        // of agent state in fixed order (no RNG, no
                        // order-dependent accumulation beyond the fixed
                        // iteration order).
                        let dividend_f = dividend.to_f64();
                        let weights: Vec<f64> = self
                            .agents
                            .iter()
                            .map(|a| 1.0 / (1.0 + a.wealth.coin.to_f64()))
                            .collect();
                        let total_w: f64 = weights.iter().sum();
                        if total_w > 0.0 {
                            for (agent, w) in self.agents.iter_mut().zip(weights) {
                                let share = Fixed::from_f64(dividend_f * w / total_w);
                                agent.wealth.coin += share;
                            }
                            institution.treasury =
                                (institution.treasury - dividend).max(Fixed::ZERO);
                        }
                    }
                }
                // Iteration 261 (audit E6): council poor relief — the
                // council recirculates a capped slice of its treasury to
                // relatively destitute members each quincenntial cycle.
                // Probe evidence (i261_gini): Gini climbs monotonically
                // toward an oligarchic asymptote (seed 7: 0.68 @20K → 0.81
                // @35K → 0.82 @50K) because trade/market returns compound
                // for the wealthy while the existing counterforces
                // (progressive market dividend, role wages, patronage)
                // miss members whose coin share collapsed — nobody sits
                // below the ABSOLUTE 1-coin floor anymore (population mean
                // 400-1700 coins), so relief keys off relative poverty:
                // below 10% of the membership's median coin. Sized as a
                // small deterministic drip (≤0.5 coins per recipient per
                // cycle, treasury reserve preserved) to stay inside the
                // calibrated grievance/legitimacy envelope — same
                // discipline as the §10.9 patronage rate.
                if institution.kind == institutions::InstitutionKind::Council {
                    let reserve = Fixed::from_f64(20.0);
                    let budget = (institution.treasury - reserve).max(Fixed::ZERO);
                    if budget > Fixed::ZERO && !self.agents.is_empty() {
                        // Relief is a COUNCIL mandate — it covers the whole
                        // village, not just council members (probe: the
                        // member roster held 3 of 12 agents, none poor).
                        let mut coins: Vec<(AgentId, Fixed)> = self
                            .agents
                            .iter()
                            .enumerate()
                            .map(|(idx, a)| (AgentId::new(idx as u64), a.wealth.coin))
                            .collect();
                        // Relative destitution floor: 10% of the median
                        // member holding (deterministic; sorted-copy
                        // median at even count averages the middle pair).
                        let mut sorted_coins: Vec<f64> =
                            coins.iter().map(|(_, c)| c.to_f64()).collect();
                        sorted_coins.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        let mid = sorted_coins.len() / 2;
                        let median = if sorted_coins.len() % 2 == 1 {
                            sorted_coins[mid]
                        } else {
                            f64::midpoint(sorted_coins[mid - 1], sorted_coins[mid])
                        };
                        /// ponytail: single-site constant; promote to a
                        /// named sim parameter when a second consumer exists
                        const RELIEF_FLOOR_MEDIAN_SHARE: f64 = 0.10;
                        let floor = median * RELIEF_FLOOR_MEDIAN_SHARE;
                        coins.retain(|(_, c)| c.to_f64() < floor);
                        if !coins.is_empty() {
                            // Poorest-first top-up toward the floor, capped
                            // at a quarter of the treasury surplus per
                            // cycle so relief equalizes without draining
                            // the council's operating capacity (and the
                            // legitimacy trajectory it feeds).
                            coins.sort_by(|a, b| a.1.to_f64().partial_cmp(&b.1.to_f64()).unwrap());
                            let mut cycle_budget = ((institution.treasury - reserve)
                                .max(Fixed::ZERO))
                                * Fixed::from_f64(0.25);
                            let mut paid_total = Fixed::ZERO;
                            for (agent_id, coin) in &mut coins {
                                if cycle_budget <= Fixed::ZERO {
                                    break;
                                }
                                let idx = agent_id.as_u64() as usize;
                                let need = Fixed::from_f64(floor - coin.to_f64()).max(Fixed::ZERO);
                                let pay = need.min(cycle_budget);
                                self.agents[idx].wealth.coin = *coin + pay;
                                *coin = self.agents[idx].wealth.coin;
                                cycle_budget -= pay;
                                paid_total += pay;
                            }
                            if paid_total > Fixed::ZERO {
                                institution.treasury -= paid_total;
                                let recipients: Vec<AgentId> =
                                    coins.iter().map(|(id, _)| *id).collect();
                                let msg = format!(
                                    "Council poor relief: {:.1} coins to {} members below \
                                     the relative-subsistence floor",
                                    paid_total.to_f64(),
                                    recipients.len()
                                );
                                let trace_msg = msg.clone();
                                let trace_affected = recipients.clone();
                                institution.record_action(tick_u64, msg, recipients, true);
                                self.provenance.record_institutional(
                                    crate::provenance::InstitutionalTrace {
                                        institution_name: "council".into(),
                                        tick: tick_u64,
                                        decision_kind: "poor_relief".into(),
                                        description: trace_msg,
                                        affected: trace_affected,
                                        success: true,
                                    },
                                );
                            }
                        }
                    }
                }
                let wage = Fixed::from_f64(institutions::BASE_WAGE);
                let role_holder_count = institution
                    .roles
                    .iter()
                    .filter(|r| r.holder.is_some())
                    .count() as i64;
                let total_wage_cost = wage * Fixed::from_int(role_holder_count);
                if institution.treasury >= total_wage_cost && role_holder_count > 0 {
                    let mut member_wealth: Vec<(AgentId, Fixed)> = institution
                        .roles
                        .iter()
                        .filter_map(|r| {
                            r.holder.map(|h| {
                                let idx = h.as_u64() as usize;
                                let wealth = if idx < self.agents.len() {
                                    self.agents[idx].wealth.coin
                                } else {
                                    Fixed::ZERO
                                };
                                (h, wealth)
                            })
                        })
                        .collect();
                    let paid = institution.pay_wages(wage, &mut member_wealth);
                    for (agent_id, new_wealth) in &member_wealth {
                        let idx = agent_id.as_u64() as usize;
                        if idx < self.agents.len() {
                            self.agents[idx].wealth.coin = *new_wealth;
                        }
                    }
                    if paid > Fixed::ZERO {
                        let members = institution.members.clone();
                        let role_names: Vec<&str> = institution
                            .roles
                            .iter()
                            .filter(|r| r.holder.is_some())
                            .map(|r| r.name.as_str())
                            .collect();
                        let kind_name = institution.kind.name().to_string();
                        let wage_msg = format!(
                            "{} wage payment: {:.1} coins to {}",
                            kind_name,
                            paid.to_f64(),
                            role_names.join(", ")
                        );
                        // Iteration 219: eliminate double-clone.
                        let trace_affected = members.clone();
                        let trace_msg = wage_msg.clone();
                        institution.record_action(tick_u64, wage_msg, members, true);
                        // §19.5.B: Record institutional provenance trace for wage payment
                        self.provenance.record_institutional(
                            crate::provenance::InstitutionalTrace {
                                institution_name: kind_name,
                                tick: tick_u64,
                                decision_kind: "wage_payment".into(),
                                description: trace_msg,
                                affected: trace_affected,
                                success: true,
                            },
                        );
                    }
                }
            }

            // §19.5.C: Taxation legitimacy feedback — taxed agents accumulate anger,
            // and high tax rates periodically erode institutional legitimacy.
            // NOTE: The per-tick decay_legitimacy(0.0001) above is a slow continuous baseline;
            // this tax-rate erosion is an additional periodic penalty applied only on collection ticks.
            if phases.is_centum && tick_u64 > 0 && !institution.members.is_empty() {
                // Tax burden affects legitimacy: higher tax rate → faster legitimacy decay
                let tax_rate = match institution.kind {
                    institutions::InstitutionKind::Council => institutions::COUNCIL_TAX_RATE,
                    institutions::InstitutionKind::Market => institutions::MARKET_FEE_RATE,
                    institutions::InstitutionKind::Temple => institutions::TEMPLE_TITHE_RATE,
                    _ => 0.0,
                };
                if tax_rate > 0.0 {
                    // Legitimacy erodes proportional to tax rate (taxation is a cost to subjects)
                    let legitimacy_erosion = Fixed::from_f64(tax_rate * 0.01);
                    institution.decay_legitimacy(legitimacy_erosion);

                    // Each taxed agent loses a tiny amount of trust/affection toward the institution
                    for m in &institution.members {
                        let idx = m.as_u64() as usize;
                        if idx < self.agents.len() {
                            // Tax burden → anger if agent is already unhappy, shame if compliant
                            let tax_fatigue =
                                self.agents[idx].wealth.coin * Fixed::from_f64(tax_rate);
                            if tax_fatigue > Fixed::from_f64(1.0) {
                                // Heavy taxation → anger
                                self.agents[idx].emotions.anger = (self.agents[idx].emotions.anger
                                    + tax_fatigue * Fixed::from_f64(0.01))
                                .clamp_01();
                            }
                        }
                    }
                }
            }

            // §11.3: Hierarchy destabilization — corruption/low legitimacy erodes cohesion
            // Gated to every 100 ticks for performance (slow process).
            if phases.is_centum {
                let destabilization = crate::social::hierarchy::destabilization_pressure(
                    institution.legitimacy,
                    institution.corruption,
                    institution.cohesion,
                    institution.collective.morale,
                    institution.collective.fear,
                    institution.treasury,
                );
                if destabilization > Fixed::from_f64(0.5) {
                    institution.cohesion = (institution.cohesion
                        - destabilization * Fixed::from_f64(0.01))
                    .max(Fixed::ZERO);
                    institution.legitimacy = (institution.legitimacy
                        - destabilization * Fixed::from_f64(0.005))
                    .max(Fixed::ZERO);
                }

                // §11.3: Hierarchy stabilization — ritual participation builds legitimacy
                // §11.3: Ritual participation builds legitimacy, but it must
                // not out-compete the grievance signal. The un-scaled sum of
                // synchrony (0.5 per sponsored ritual) added +0.01/centum —
                // just enough to hold Council legitimacy above the faction
                // trigger (< 0.5) in high-grievance villages, silently
                // disabling revolution (Iteration-6 regression). Halving the
                // contribution keeps rituals meaningful (cohesion, trust,
                // provenance) while letting aggrieved populations still
                // withdraw legitimacy and arm the faction trigger.
                let ritual_strength = self
                    .ritual_registry
                    .rituals
                    .iter()
                    .filter(|r| r.sponsor == institution.id as usize)
                    .map(|r| r.synchrony)
                    .fold(Fixed::ZERO, |a, b| a + b)
                    * Fixed::from_f64(0.5);
                // §13.4: Narrative strength derives from the institution's
                // active propaganda campaigns instead of a static placeholder.
                // Average the effectiveness of campaigns sponsored by this
                // institution (0.3 fallback when no campaigns are active
                // preserves the calibrated baseline). The propaganda system
                // already computes effectiveness from legitimacy × audience_fear
                // × repetition × identity_congruence, so this wiring means
                // propaganda → cohesion is now a live feedback loop: strong
                // campaigns strengthen cohesion, weak/corrupted ones do not.
                let narrative_strength = {
                    let active: Vec<Fixed> = self
                        .propaganda_registry
                        .campaigns
                        .iter()
                        .filter(|c| c.active && c.sponsor == institution.id as usize)
                        .map(|c| c.effectiveness)
                        .collect();
                    if active.is_empty() {
                        Fixed::from_f64(0.3)
                    } else {
                        let sum: Fixed = active.iter().fold(Fixed::ZERO, |acc, &v| acc + v);
                        sum / Fixed::from_int(active.len() as i64)
                    }
                };
                crate::social::hierarchy::stabilize_hierarchy(
                    institution,
                    ritual_strength.min(Fixed::ONE),
                    institution.enforcement_capacity,
                    narrative_strength,
                );

                // §16.1: Record ritual provenance trace (format! hoisted outside loop)
                if ritual_strength > Fixed::ZERO {
                    let desc = format!("Ritual participation in {}", institution.name);
                    let magnitude = ritual_strength.min(Fixed::ONE);
                    for &member in &institution.members {
                        self.provenance
                            .record_system(crate::provenance::SystemTrace {
                                agent: member,
                                tick: tick_u64,
                                category: crate::provenance::ProvenanceCategory::Ritual,
                                description: desc.clone(),
                                magnitude,
                                cause: "hierarchy_stabilization".into(),
                            });
                    }
                }
            }

            // §11.3: Leadership challenges — elections and coups
            // Gated to every 500 ticks for performance.
            if tick_u64 > 1000 && phases.is_quincent && institution.kind == InstitutionKind::Council
            {
                // Find current leader
                let current_leader = institution
                    .roles
                    .iter()
                    .find(|r| r.name == "Leader")
                    .and_then(|r| r.holder);
                if let Some(leader_id) = current_leader {
                    let leader_idx = leader_id.as_u64() as usize;
                    if leader_idx < self.agents.len() {
                        let leader_score = crate::social::hierarchy::leadership_score(
                            self.agents[leader_idx].personality.ambition,
                            self.agents[leader_idx].personality.dominance,
                            self.agents[leader_idx].personality.extraversion,
                            self.agents[leader_idx].personality.conscientiousness,
                            self.agents[leader_idx].age,
                            self.agents[leader_idx].wealth.coin,
                            self.agents[leader_idx].status_v2.network_centrality,
                            self.agents[leader_idx].status_v2.moral_reputation,
                            self.agents[leader_idx].embodied.nervous.trauma_load,
                        );

                        // Find best challenger among members (non-leader)
                        let mut best_challenger: Option<(AgentId, Fixed)> = None;
                        for member_id in &institution.members {
                            let mid = member_id.as_u64() as usize;
                            if mid < self.agents.len() && mid != leader_idx {
                                let score = crate::social::hierarchy::leadership_score(
                                    self.agents[mid].personality.ambition,
                                    self.agents[mid].personality.dominance,
                                    self.agents[mid].personality.extraversion,
                                    self.agents[mid].personality.conscientiousness,
                                    self.agents[mid].age,
                                    self.agents[mid].wealth.coin,
                                    self.agents[mid].status_v2.network_centrality,
                                    self.agents[mid].status_v2.moral_reputation,
                                    self.agents[mid].embodied.nervous.trauma_load,
                                );
                                if best_challenger.is_none_or(|(_, s)| score > s) {
                                    best_challenger = Some((*member_id, score));
                                }
                            }
                        }

                        if let Some((challenger_id, challenger_score)) = best_challenger {
                            if crate::social::hierarchy::should_challenge(
                                institution,
                                challenger_score,
                                leader_score,
                                tick_u64,
                                self.last_revolution_tick,
                            ) {
                                let result = crate::social::hierarchy::attempt_challenge(
                                    institution,
                                    leader_id,
                                    challenger_id,
                                    challenger_score,
                                    leader_score,
                                    institution.corruption,
                                );
                                match &result {
                                    crate::social::hierarchy::ChallengeResult::Election {
                                        ..
                                    } => {
                                        tracing::warn!(
                                            tick = tick_u64,
                                            "Leadership election occurred"
                                        );
                                    }
                                    crate::social::hierarchy::ChallengeResult::Coup { .. } => {
                                        tracing::warn!(
                                            tick = tick_u64,
                                            "Leadership coup attempted"
                                        );
                                    }
                                    crate::social::hierarchy::ChallengeResult::None => {}
                                }
                            }
                        }
                    }
                }
            }
        } // ── 15. Faction dynamics — grievance, formation, recruitment, protests (Phase 8) ──
    }

    /// §5 (Iteration 152): One religious pass — conversions and conviction
    /// drift on the yearly mark (4320k), festivals on the mid-year mark
    /// (2160 + 4320k). Gated on a seeded [`crate::theology::Religion`]: no
    /// default world seeds one, so this is a structural no-op in every
    /// calibrated window. Fully deterministic — conversion and drift follow
    /// fixed rules and no RNG is drawn — so the pass can never perturb any
    /// RNG stream, even in a world with a live religion.
    pub fn tick_theology(&mut self, tick_u64: u64) {
        // Zero-blast gate: a world without a religion has no theology.
        if self.theology.religion.is_none() {
            return;
        }
        let n = self.agents.len();
        if n == 0 {
            return;
        }
        if self.theology.beliefs.len() < n {
            self.theology.beliefs.resize(n, None);
        }

        // Mid-year festival (2160 + 4320k): hallow the doctrine's sacred
        // value among believers and boost their conviction. A festival with
        // no believers yet is not held (nothing to celebrate).
        if tick_u64.is_multiple_of(2160) && !tick_u64.is_multiple_of(4320) {
            let attenders = self.theology.believer_count();
            if attenders == 0 {
                return;
            }
            let sacred_value = self
                .theology
                .religion
                .as_ref()
                .map(|r| r.doctrine.sacred_value.clone())
                .unwrap_or_default();
            for i in 0..n {
                if self.theology.beliefs[i].is_none() {
                    continue;
                }
                if let Some(b) = self.theology.beliefs[i].as_mut() {
                    b.conviction = (b.conviction * Fixed::from_f64(1.05)).clamp_01();
                }
                self.agents[i].sacred_values.add_or_strengthen(
                    sacred_value.clone(),
                    Fixed::from_f64(0.15),
                    Fixed::from_f64(0.15),
                );
            }
            self.theology.festivals_held += 1;
            self.journal.record(
                tick_u64,
                AgentId::new(0),
                JournalEntryKind::TheologyFestival { attenders },
            );
            return;
        }

        // Yearly mark (4320k): conversion + conviction drift. Elders
        // convert freely; everyone else needs social contact with a
        // believer from BEFORE this pass (the neutral-default relationship
        // view makes contagion spread fast once the first believer exists —
        // a two-stage spread: elders adopt, then the village follows).
        let believers_before: Vec<bool> =
            self.theology.beliefs.iter().map(Option::is_some).collect();
        let elder_age = Fixed::from_f64(ELDER_AGE);
        // The theodicy every new convert adopts — constant across the pass.
        let temperament = self
            .theology
            .religion
            .as_ref()
            .map_or(Temperament::Indifferent, |r| r.deity.temperament);
        let mut converts_this_year: u64 = 0;
        for i in 0..n {
            if self.theology.beliefs[i].is_some() {
                continue;
            }
            let is_elder = self.agents[i].age >= elder_age;
            let has_believer_contact = (0..n).any(|j| {
                j != i
                    && believers_before[j]
                    && self.relationship_quality(i, j) >= Fixed::from_f64(0.3)
            });
            if !should_convert(is_elder, has_believer_contact) {
                continue;
            }
            self.theology.beliefs[i] = Some(TheologicalBelief {
                conviction: seed_conviction(is_elder, self.agents[i].age),
                temperament_held: temperament,
                since_tick: tick_u64,
            });
            self.theology.converts += 1;
            converts_this_year += 1;
        }

        // One year of conviction drift: every belief pulls toward the
        // community mean.
        let believers: Vec<usize> = (0..n)
            .filter(|&i| self.theology.beliefs[i].is_some())
            .collect();
        if !believers.is_empty() {
            let mean = believers
                .iter()
                .map(|&i| {
                    self.theology.beliefs[i]
                        .as_ref()
                        .map_or(Fixed::ZERO, |b| b.conviction)
                        .to_f64()
                })
                .sum::<f64>()
                / believers.len() as f64;
            let mean_f = Fixed::from_f64(mean);
            for &i in &believers {
                if let Some(b) = self.theology.beliefs[i].as_mut() {
                    b.conviction = drift_conviction(b.conviction, mean_f, 0.1);
                }
            }
        }

        if converts_this_year > 0 {
            self.journal.record(
                tick_u64,
                AgentId::new(0),
                JournalEntryKind::TheologyConversion {
                    converts: converts_this_year,
                },
            );
        }
    }

    /// §5 (Iteration 153): One military pass — conscription and drill.
    /// Gated on a `SiteKind::Barracks` existing (no default world places
    /// one, so this is a structural no-op in every calibrated window). Fully
    /// deterministic — conscription and drills follow fixed rules and no RNG
    /// is drawn — so the pass can never perturb any RNG stream, even in a
    /// world with a barracks.
    pub fn tick_military(&mut self, tick_u64: u64) {
        // Zero-blast gate: without a barracks there is no militia to muster.
        if !self
            .world
            .sites
            .iter()
            .any(|s| s.kind == crate::world::SiteKind::Barracks)
        {
            return;
        }
        let n = self.agents.len();
        if n == 0 {
            return;
        }
        if self.military.roster.len() < n {
            self.military.roster.resize(n, None);
        }

        // Muster: conscript the most dominant eligible adults up to the
        // barracks cap (dominance is the same combat-power proxy
        // `resolve_conflict` uses for aggression). Deterministic tie-break:
        // stable order by descending dominance. The cap comes from the
        // barracks' own capacity, never exceeding the hard constant.
        let site_cap = self
            .world
            .sites
            .iter()
            .find(|s| s.kind == crate::world::SiteKind::Barracks)
            .map_or(MUSTER_CAP, |s| (s.capacity as usize).min(MUSTER_CAP));
        let mut candidates: Vec<usize> = (0..n)
            .filter(|&i| conscription_eligible(self.agents[i].age))
            .filter(|&i| self.military.roster[i].is_none())
            .collect();
        candidates.sort_by(|&a, &b| {
            self.agents[b]
                .personality
                .dominance
                .cmp(&self.agents[a].personality.dominance)
        });
        candidates.truncate(site_cap);
        let mut conscripted: u64 = 0;
        for i in candidates {
            self.military.roster[i] = Some(MilitiaMember {
                enlisted_since: tick_u64,
                dominance_at_enlistment: self.agents[i].personality.dominance,
            });
            self.military.conscripts += 1;
            conscripted += 1;
        }
        if conscripted > 0 {
            self.military.musters += 1;
            self.journal.record(
                tick_u64,
                AgentId::new(0),
                JournalEntryKind::MilitaryMuster {
                    conscripts: conscripted,
                },
            );
        }

        // Yearly drill: the militia trains, building collective readiness
        // (the decay applies every year, whether or not they drill).
        let attenders = self.military.militia_size() as usize;
        self.military.readiness = drill_readiness(self.military.readiness, attenders, site_cap);
        if attenders > 0 {
            self.military.drills += 1;
            self.journal.record(
                tick_u64,
                AgentId::new(0),
                JournalEntryKind::MilitaryDrill {
                    attenders: attenders as u64,
                    readiness: self.military.readiness.to_f64(),
                },
            );
        }
    }

    /// Section 4b: Resource operations, journal recording, intention tracking.
    /// §19.5.I (Iteration 148): One technology-discovery pass. For each
    /// undiscovered tier-1+ node, rolls the discovery chance derived from the
    /// population's prerequisite mass; a hit adds the node to the knowledge
    /// store and marks it discovered, unlocking its dependents for later
    /// passes. Draws come from the virgin Narrative stream — the only
    /// consumer — so the pass can never perturb any other subsystem's RNG.
    /// The pass is idempotent and safe to call from tests.
    pub fn tick_technology_discovery(&mut self, tick_u64: u64) {
        let known: Vec<Vec<u64>> = self
            .agents
            .iter()
            .map(|a| a.cultural.knowledge.clone())
            .collect();
        let ids: Vec<u64> = self.technology.undiscovered().collect();
        let mut hits: Vec<u64> = Vec::new();
        for id in ids {
            let chance = self.technology.discovery_chance(id, &known);
            if chance <= Fixed::ZERO {
                continue;
            }
            let roll = Fixed::from_f64(self.rng.get_mut(RngStream::Narrative).random::<f64>());
            if roll < chance {
                hits.push(id);
            }
        }
        for id in hits {
            let name = self
                .technology
                .nodes
                .get(&id)
                .map_or_else(|| format!("tech-{id}"), |n| n.name.clone());
            if let Some(k) = self.technology.discover(id, tick_u64) {
                self.knowledge_store.push(k);
                self.journal.record(
                    tick_u64,
                    AgentId::new(0),
                    JournalEntryKind::KnowledgeDiscovered {
                        knowledge_id: id,
                        name,
                    },
                );
            }
        }
    }
}
