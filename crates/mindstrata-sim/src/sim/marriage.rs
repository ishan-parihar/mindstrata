//! Marriage formation, pair bonds and spousal household merge/split.

use super::{AgentId, Belief, Fixed, MoralValues, RngStream, SimEvent, Simulation, Tick};

use rand::Rng;
impl Simulation {
    /// Section 19: Marriage formation.
    pub(super) fn tick_marriage_formation(&mut self, tick_u64: u64, tick: Tick) {
        // ── 19. §19.5.F Marriage formation — compatible age + high affection ──
        // §10.5 (P5 audit, Iteration 184): same-pass bigamy guard. The
        // formation loop previously collected `new_marriages` and applied
        // them only AFTER the full scan, so two agents (i and i') could both
        // claim the same unpartnered j in one pass — agent j's `partner`
        // field is still None mid-scan, and the last write won, leaving j
        // in TWO active marriages with TWO pair bonds (a §10.5 monogamy
        // violation). Probe-caught on seed 46 (marriages 1↔6 AND 4↔6 both
        // formed tick 191; the bigamous same-sex couple then fired the
        // conception lottery, converting the pregnancy-path birth into a
        // legacy immediate birth). The fix marks both ends of every newly
        // claimed pair as taken for the REST of this pass, so each agent
        // can appear in at most one new marriage per tick.
        {
            let n = self.agents.len();
            let mut new_marriages: Vec<(usize, usize)> = Vec::new();
            // Both ends of every pair claimed this pass — consulted mid-scan
            // because `partner` is only written in the apply phase below.
            let mut claimed: Vec<bool> = vec![false; n];

            for i in 0..n {
                if self.agents[i].partner.is_some() {
                    continue; // already partnered
                }
                if self.agents[i].age < Fixed::from_f64(18.0) {
                    continue; // too young
                }
                if claimed[i] {
                    continue; // claimed by an earlier agent this pass
                }

                // Find candidate: compatible age, high affection, no partner
                for j in (i + 1)..n {
                    if self.agents[j].partner.is_some() {
                        continue;
                    }
                    if claimed[j] {
                        continue; // claimed by an earlier agent this pass
                    }
                    if self.agents[j].age < Fixed::from_f64(18.0) {
                        continue;
                    }
                    // Check age compatibility (within 15 years)
                    let age_diff = (self.agents[i].age - self.agents[j].age).abs();
                    if age_diff > Fixed::from_f64(15.0) {
                        continue;
                    }
                    // Check relationship affection
                    let affection = self
                        .relationships
                        .iter()
                        .find(|r| {
                            r.from == AgentId::new(i as u64) && r.to == AgentId::new(j as u64)
                        })
                        .map_or(Fixed::ZERO, |r| r.affection);
                    // Architecture-plan-2 §10.4: Compute attraction score.
                    // Personality compatibility (agreeableness similarity), physical proximity,
                    // and social approval feed into the AttractionModel.
                    let personality_compat = Fixed::ONE
                        - (self.agents[i].personality.agreeableness
                            - self.agents[j].personality.agreeableness)
                            .abs();
                    let pos_i = self.agents[i].position;
                    let pos_j = self.agents[j].position;
                    let distance = Fixed::from_int(pos_i.manhattan_distance(&pos_j) as i64);
                    // Architecture-plan-2 §10.4: Max distance for marriage proximity scoring.
                    const MAX_MARRIAGE_DISTANCE: f64 = 20.0;
                    let proximity = (Fixed::ONE
                        - distance / Fixed::from_f64(MAX_MARRIAGE_DISTANCE))
                    .max(Fixed::ZERO);
                    let att = crate::social::attraction::AttractionModel {
                        personality_attraction: personality_compat,
                        familiarity: affection,
                        physical_attraction: proximity,
                        reciprocity: affection, // reciprocity from mutual affection
                        ..crate::social::attraction::AttractionModel::default()
                    };
                    let attraction_score = att.total_attraction();
                    // Marriage probability: attraction * health * trust
                    let health = (self.agents[i].body.health + self.agents[j].body.health)
                        * Fixed::from_f64(0.5);
                    let trust = self
                        .relationships
                        .iter()
                        .find(|r| {
                            r.from == AgentId::new(i as u64) && r.to == AgentId::new(j as u64)
                        })
                        .map_or(Fixed::ZERO, |r| r.trust);
                    // Marriage probability: attraction * health * trust, scaled to a
                    // daily cadence. Previously the 0.001 scalar made the effective
                    // chance ~1e-4/pair/day — a 12-agent village needed ~20K ticks
                    // (200 days) to pair everyone off. 0.01 yields a marriage every
                    // few days of eligible pairs, which is realistic for a small
                    // pre-modern village without flooding instant marriages.
                    // §10.8: Enemy clans do not intermarry (feud boundary =
                    // marriage boundary); allied clans intermarry preferentially.
                    // The chance is zeroed rather than skipping the pair, so the
                    // RNG stream is untouched (replay determinism).
                    let clan_factor = if self.clans_are_enemies(i, j) {
                        Fixed::ZERO
                    } else if self.clans_are_allies(i, j) {
                        Fixed::from_f64(1.5)
                    } else {
                        Fixed::ONE
                    };
                    let marriage_chance = attraction_score
                        * health
                        * trust
                        * self.params.marriage_formation_rate
                        * clan_factor;
                    let rng_val =
                        Fixed::from_f64(self.rng.get_mut(RngStream::Social).random::<f64>());
                    if rng_val < marriage_chance {
                        new_marriages.push((i, j));
                        claimed[i] = true;
                        claimed[j] = true;
                        break; // only one marriage per agent per tick
                    }
                }
            }

            for (a, b) in new_marriages {
                // §10.4 (Iteration 76): Marriage concludes the courtship — any
                // active courtship between the newlyweds is removed so its
                // record ends at Betrothal instead of lingering past the
                // wedding (deterministic, no events, no RNG).
                let mut wed_pursuers: Vec<usize> = Vec::new();
                self.active_courtships.retain(|c| {
                    let pair_matches =
                        (c.pursuer == a && c.pursued == b) || (c.pursuer == b && c.pursued == a);
                    if pair_matches {
                        wed_pursuers.push(c.pursuer);
                    }
                    !pair_matches
                });
                // The newlywed pursuer's attraction model reverts to zero
                // reciprocity — the courtship (and its mirror) is concluded.
                for p in wed_pursuers {
                    self.agents[p].attraction.reciprocity = Fixed::ZERO;
                }
                self.agents[a].partner = Some(b);
                self.agents[b].partner = Some(a);
                // §10.7 (P5 audit, Iteration 184): co-residence — the newly
                // married spouses merge their (populate-time singleton)
                // households into one joint household. The populate-time
                // household pass runs before any marriage exists, so runtime
                // marriages never co-resided and `tick_household_food_pooling`
                // (multi-member only) never fired. Deterministic, no RNG.
                let merged_household = self.merge_spouse_households(a, b);
                // §10.8: Marriage forges a clan alliance between the spouses' clans.
                self.forge_clan_alliance(a, b, tick_u64);
                self.events.push(SimEvent::MarriageFormed {
                    spouse_a: AgentId::new(a as u64),
                    spouse_b: AgentId::new(b as u64),
                    tick,
                });
                // §10.5 (AP2): Marriage is an institution, not just a partner
                // pointer — instantiate the Marriage record with its institutional
                // dimensions, deterministically derived (no RNG, so the golden
                // baseline stays byte-identical; only new registry entries are
                // written, nothing pre-existing is touched).
                let marriage_type = if self.active_courtships.iter().any(|c| {
                    (c.pursuer == a && c.pursued == b) || (c.pursuer == b && c.pursued == a)
                }) {
                    crate::social::marriage::MarriageType::Chosen
                } else if self.marriage_registry.marriages.iter().any(|m| {
                    !m.active
                        && (m.partner_a == a
                            || m.partner_b == a
                            || m.partner_a == b
                            || m.partner_b == b)
                }) {
                    crate::social::marriage::MarriageType::Remarriage
                } else {
                    crate::social::marriage::MarriageType::Arranged
                };
                let mut marriage =
                    crate::social::marriage::Marriage::new(a, b, marriage_type, tick_u64);
                // Kin alliance: the marriage itself forges a Spouse tie between
                // the families; existing InLaw edges involving either spouse are
                // folded in (deduped, deterministic edge order).
                let mut kin_alliance = vec![crate::social::kinship::KinshipLink::Spouse];
                for edge in &self.kinship_graph.edges {
                    let touches = edge.from == a || edge.from == b || edge.to == a || edge.to == b;
                    if touches
                        && matches!(edge.link, crate::social::kinship::KinshipLink::InLaw)
                        && !kin_alliance.contains(&edge.link)
                    {
                        kin_alliance.push(edge.link);
                    }
                }
                marriage.derive_institution(
                    kin_alliance,
                    self.agents[a].wealth.coin,
                    self.agents[b].wealth.coin,
                );
                marriage.household_id = merged_household;
                self.marriage_registry.marriages.push(marriage);
                // §10.4 (AP2): Instantiate the romantic pair bond the marriage
                // institutionalizes. The bond was previously NEVER created in
                // production — the `pair_bonds` registry stayed empty and the
                // whole romantic subsystem (bond_strength / strain /
                // jealousy_load dynamics) was dead code. It forms 1:1 with
                // marriages and evolves in the daily `tick_pair_bonds` pass.
                self.marriage_registry.pair_bonds.push(
                    crate::social::marriage::PairBond::new_married(a, b, tick_u64),
                );
                // §10.6 (AP2): Marriage writes real kinship consequences —
                // Spouse ties between the couple and InLaw ties to each
                // other's parents/siblings. Previously marriage created NO
                // kinship edges (only the institution's kin_alliance
                // metadata), so the Spouse link and the §10.3 InLaw stage
                // were unreachable in production. All marital edges carry
                // coefficient ZERO (inert for the incest taboo; their only
                // reach is the observational kin-stage pass), and initial
                // adults hold no kin edges, so default runs see only inert
                // Spouse ties — the golden baseline stays byte-identical.
                self.kinship_graph.add_marital_links(a, b, tick_u64);
                // Marriage boosts trust and affection
                // §19.5.J: Record marriage relationship traces
                if let Some(rel) = self
                    .relationships
                    .iter_mut()
                    .find(|r| r.from == AgentId::new(a as u64) && r.to == AgentId::new(b as u64))
                {
                    let old_trust = rel.trust;
                    let old_affection = rel.affection;
                    rel.trust = (rel.trust + Fixed::from_f64(0.2)).clamp_01();
                    rel.affection = (rel.affection + Fixed::from_f64(0.3)).clamp_01();
                    self.provenance
                        .record_relationship(crate::provenance::RelationshipTrace {
                            from: AgentId::new(a as u64),
                            to: AgentId::new(b as u64),
                            tick: tick_u64,
                            cause: "marriage".into(),
                            old_trust,
                            new_trust: rel.trust,
                            old_affection,
                            new_affection: rel.affection,
                            description: format!(
                                "Marriage bond formed (trust {} -> {}, affection {} -> {})",
                                old_trust.to_f64(),
                                rel.trust.to_f64(),
                                old_affection.to_f64(),
                                rel.affection.to_f64()
                            ),
                        });
                }
                if let Some(rel) = self
                    .relationships
                    .iter_mut()
                    .find(|r| r.from == AgentId::new(b as u64) && r.to == AgentId::new(a as u64))
                {
                    let old_trust = rel.trust;
                    let old_affection = rel.affection;
                    rel.trust = (rel.trust + Fixed::from_f64(0.2)).clamp_01();
                    rel.affection = (rel.affection + Fixed::from_f64(0.3)).clamp_01();
                    self.provenance
                        .record_relationship(crate::provenance::RelationshipTrace {
                            from: AgentId::new(b as u64),
                            to: AgentId::new(a as u64),
                            tick: tick_u64,
                            cause: "marriage".into(),
                            old_trust,
                            new_trust: rel.trust,
                            old_affection,
                            new_affection: rel.affection,
                            description: format!(
                                "Marriage bond formed (trust {} -> {}, affection {} -> {})",
                                old_trust.to_f64(),
                                rel.trust.to_f64(),
                                old_affection.to_f64(),
                                rel.affection.to_f64()
                            ),
                        });
                }
                tracing::info!(
                    spouse_a = a,
                    spouse_b = b,
                    tick = tick_u64,
                    "Marriage formed"
                );
            }
        }
    }

    /// Section 19b: Birth mechanics.
    /// §31: Build a fresh newborn AgentBundle in place of a dead agent.
    ///
    /// Because the sim relies on `AgentId::new(i) == index i` (relationship_v2s
    /// O(1) layout, agent_diseases parallel vec, partner/parent indices), dead
    /// agents are NOT removed — their slot is repopulated by a newborn child.
    /// This keeps every index-stable registry consistent.
    /// Foundational beliefs seeded into every agent at populate, and now also
    /// into newborns (born or replacement) so they can participate in belief
    /// propagation instead of starting with an empty set.
    /// Iteration 246: the village's living-community moral prior — the
    /// per-foundation mean across all living agents. Births blend this in
    /// at 30% so each generation re-anchors slightly toward the current
    /// consensus instead of drifting freely.
    pub(super) fn community_moral_prior(&self) -> MoralValues {
        if self.agents.is_empty() {
            return MoralValues::default();
        }
        let n = Fixed::from_int(self.agents.len() as i64);
        let mean = |pick: fn(&MoralValues) -> Fixed| -> Fixed {
            self.agents
                .iter()
                .fold(Fixed::ZERO, |acc, a| acc + pick(&a.moral_values))
                / n
        };
        MoralValues {
            care: mean(|m| m.care),
            fairness: mean(|m| m.fairness),
            loyalty: mean(|m| m.loyalty),
            authority: mean(|m| m.authority),
            purity: mean(|m| m.purity),
            liberty: mean(|m| m.liberty),
        }
    }

    pub(super) fn foundational_beliefs() -> Vec<Belief> {
        vec![
            Belief {
                proposition_id: 0,
                confidence: Fixed::from_f64(0.6),
                emotional_charge: Fixed::ZERO,
                identity_linkage: Fixed::from_f64(0.3),
                resistance: Fixed::from_f64(0.5),
                last_reinforced_tick: 0,
                source: crate::person::EvidenceSource::PersonalExperience,
                social_reinforcement: 0,
                is_accurate: true,
            },
            Belief {
                proposition_id: 1,
                confidence: Fixed::from_f64(0.5),
                emotional_charge: Fixed::ZERO,
                identity_linkage: Fixed::from_f64(0.2),
                resistance: Fixed::from_f64(0.4),
                last_reinforced_tick: 0,
                source: crate::person::EvidenceSource::InstitutionalRecord,
                social_reinforcement: 0,
                is_accurate: true,
            },
            // §8.1.18 (Iteration 168): NeighborTrustworthy — the belief the
            // daily belief-update gate targets (sim.rs block 13 updates
            // proposition 2 from interaction trust evidence). Pre-Iter-168
            // agents never held it, so the gate was structurally dead (probed
            // 0/12 agents across seeds). Seeding it here (shared by populate,
            // newborns, and snapshot restores) revives the gate: interaction
            // trust now genuinely updates the belief, and §8.1.18
            // `change_resistance` dampens the update (a uniform cultural
            // factor — resistance totals ~0.55 for every agent: 0.15 from
            // conservatism 0.5 × 0.3 plus 0.4 from the saturated taboo term
            // min(1.0) × 0.4, category rigidity ~0; honestly framed as a
            // standing dampening, not a differential).
            Belief {
                proposition_id: 2,
                confidence: Fixed::from_f64(0.5),
                emotional_charge: Fixed::ZERO,
                identity_linkage: Fixed::from_f64(0.2),
                resistance: Fixed::from_f64(0.4),
                last_reinforced_tick: 0,
                source: crate::person::EvidenceSource::PersonalExperience,
                social_reinforcement: 0,
                is_accurate: true,
            },
        ]
    }

    /// §10.4 (AP2): Daily pair-bond dynamics.
    ///
    /// Pair bonds form 1:1 with marriages in the daily social pass; here we
    /// charge each bond's jealousy load from the partners' appraised jealousy
    /// emotion (weighted by relationship dependence) and drive the
    /// post-marriage romantic-stage ladder (Married → HouseholdFormed →
    /// Parenthood → Strain ↔ Stabilization). Iteration 126 corrected this
    /// comment: the pass is NOT observational-only — `should_dissolve()`
    /// (bond_strength < 0.1 && strain > 0.8) below dissolves the bond and
    /// its marriage (Separation), so the jealousy → load → strain →
    /// dissolution chain is a fully decisional §10.4 consumer. In
    /// calibrated windows both producer inputs (status_threat ×
    /// attachment_threat) are dormant, so the emotion and the load stay
    /// exactly zero (probe-pinned, Iter-126) and the golden baseline is
    /// byte-identical.
    pub(super) fn tick_pair_bonds(&mut self, tick_u64: u64) {
        let n = self.agents.len();
        if n == 0 {
            return;
        }
        // Precompute canonical shared-child parent pairs once per pass
        // (O(n) once, O(bonds) lookups — §17.4 population scale discipline).
        let parent_pairs: Vec<(usize, usize)> = self
            .agents
            .iter()
            .filter_map(|ag| {
                let (pa, pb) = (ag.parent_a?, ag.parent_b?);
                Some((pa.min(pb), pa.max(pb)))
            })
            .collect();
        for bond in &mut self.marriage_registry.pair_bonds {
            let a = bond.partner_a;
            let b = bond.partner_b;
            if a >= n || b >= n {
                continue;
            }
            // Dependence = trust from a's perspective of b (the same v1
            // relationship layer the marriage pass uses).
            let trust = self
                .relationships
                .iter()
                .find(|r| r.from == AgentId::new(a as u64) && r.to == AgentId::new(b as u64))
                .map_or(Fixed::ZERO, |r| r.trust);
            // Appraised jealousy already folds attachment anxiety, status
            // threat and fear of abandonment into one emotion (appraisal.rs).
            let jealousy = (self.agents[a].emotions.jealousy + self.agents[b].emotions.jealousy)
                * Fixed::from_f64(0.5);
            bond.charge_jealousy(jealousy, trust);
            // Effects (§10.4): sustained jealousy load strains the bond; a
            // calm, trusting bond gets a small maintenance boost (calibrated
            // small — a large magnitude saturated bond_strength to 1.0 within
            // ~40 days, erasing bond differentiation; 0.1 keeps the
            // equilibrium ~0.002/day so healthy bonds plateau near ~0.8 and
            // stay differentiated).
            if bond.jealousy_load > Fixed::from_f64(0.6) {
                bond.record_negative(tick_u64, bond.jealousy_load - Fixed::from_f64(0.6));
            } else if bond.jealousy_load < Fixed::from_f64(0.2) && trust > Fixed::from_f64(0.5) {
                bond.record_positive(tick_u64, Fixed::from_f64(0.1));
            }
            // Shared children drive the Parenthood stage of the ladder.
            let has_children = parent_pairs.contains(&(a.min(b), a.max(b)));
            bond.advance_stage(has_children);
        }
        // §10.4 (AP2): Jealousy-driven breakup — the decisional consumer.
        // A bond crushed past the dissolution threshold (strength < 0.1 AND
        // strain > 0.8) ends its marriage: the marriage record is dissolved
        // with the Separation cause (legitimacy damaged, strain maxed — the
        // history stays in the registry with active=false) and the bond
        // itself is removed so it cannot re-fire daily. Pairs are collected
        // first and applied after the loop (no reentrant borrow of the
        // registry). In peaceful villages jealousy stays ~0 so this never
        // fires — the golden baseline stays byte-identical.
        let dissolving: Vec<(usize, usize)> = self
            .marriage_registry
            .pair_bonds
            .iter()
            .filter(|pb| pb.should_dissolve())
            .map(|pb| {
                (
                    pb.partner_a.min(pb.partner_b),
                    pb.partner_a.max(pb.partner_b),
                )
            })
            .collect();
        if !dissolving.is_empty() {
            for (a, b) in &dissolving {
                if let Some(m) = self.marriage_registry.marriages.iter_mut().find(|m| {
                    m.active
                        && ((m.partner_a == *a && m.partner_b == *b)
                            || (m.partner_a == *b && m.partner_b == *a))
                }) {
                    m.dissolve(&crate::social::marriage::DissolutionCause::Separation);
                }
                // §10.7 (P5 audit, Iteration 184): separation ends
                // co-residence — the joint household splits back into
                // singletons (children stay with the first spouse).
                self.split_spouse_household(*a, *b);
            }
            self.marriage_registry.pair_bonds.retain(|pb| {
                let pair = (
                    pb.partner_a.min(pb.partner_b),
                    pb.partner_a.max(pb.partner_b),
                );
                !dissolving.contains(&pair)
            });
        }
    }

    /// §10.7 (P5 audit, Iteration 184): co-residence — merge the two
    /// spouses' households into one joint household at marriage formation.
    /// The merge is in-place (B's members fold into A, B's vec entry is
    /// emptied as a tombstone) so Vec indices never shift and the O(1)
    /// household lookups elsewhere stay valid. Deterministic, no RNG.
    /// Returns the merged household index (None if either spouse has none).
    fn merge_spouse_households(&mut self, a: usize, b: usize) -> Option<usize> {
        let hh_a = self.households.iter().position(|h| h.members.contains(&a));
        let hh_b = self.households.iter().position(|h| h.members.contains(&b));
        if let (Some(ia), Some(ib)) = (hh_a, hh_b) {
            if ia != ib {
                let members_b: Vec<usize> = self.households[ib].members.clone();
                for m in members_b {
                    if m != b {
                        self.households[ia].add_member(m);
                    }
                }
                self.households[ia].add_member(b);
                self.households[ia].head = Some(a);
                self.households[ib].members.clear();
                self.households[ib].roles.clear();
            }
            return Some(ia);
        }
        None
    }

    /// §10.7 (P5 audit, Iteration 184): separation ends co-residence —
    /// remove spouse B from the joint household (children stay with A); if
    /// the household empties it stays as a tombstone. Deterministic, no RNG.
    fn split_spouse_household(&mut self, a: usize, b: usize) {
        for hh in &mut self.households {
            if hh.members.contains(&a) && hh.members.contains(&b) {
                if let Some(pos) = hh.members.iter().position(|m| *m == b) {
                    hh.members.remove(pos);
                    if pos < hh.roles.len() {
                        hh.roles.remove(pos);
                    }
                }
                if hh.head == Some(b) {
                    hh.head = hh.members.first().copied();
                }
            }
        }
    }
}
