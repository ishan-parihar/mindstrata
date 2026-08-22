//! Marriage, pair bonds, births, deaths and household kinship economics.

use super::*;

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
    fn community_moral_prior(&self) -> MoralValues {
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

    fn build_replacement_newborn(&self, idx: usize, tick_u64: u64) -> AgentBundle {
        // Iteration 245: the replacement newborn keeps the deceased's
        // household surname (lineage continuity, matching the self-blend).
        let child_name = crate::person::inherit_surname(
            &self.agents[idx].name,
            crate::person::FIRST_NAMES
                [(idx + self.current_tick().as_u64() as usize) % crate::person::FIRST_NAMES.len()],
        );
        // Inherit the deceased agent's home and position so the newborn keeps
        // the family home and village location (generational continuity).
        let inherited_home = self.agents[idx].home_site;
        let inherited_position = self.agents[idx].position;

        // Inherit personality traits with noise (deterministic from seed + tick)
        let mut child_rng = rand_chacha::ChaCha8Rng::seed_from_u64(
            self.config
                .seed
                .wrapping_add(tick_u64)
                .wrapping_add(idx as u64),
        );
        // Iteration 244: replacement newborns continue the deceased's
        // household lineage — single-ancestor blend (self-blend + mutation)
        // keeps family resemblance through the house without fabricating an
        // unknown second parent.
        let child_personality =
            Personality::inherit(&self.agents[idx].personality, None, &mut child_rng);
        let child_age = Fixed::from_f64(0.0); // newborn
        let child_embodied =
            EmbodiedState::born(&self.agents[idx].embodied, None, child_age, &mut child_rng);
        let child_attachment_vulnerability = child_embodied
            .genome
            .trait_predispositions
            .attachment_vulnerability;
        let child_neuroticism = child_personality.neuroticism;
        let child_openness = child_personality.openness;

        AgentBundle {
            body: BodyState::from(&child_embodied),
            needs: NeedState::default(),
            // Clone: the original is reused below to seed the newborn's
            // emotion-regulation strategy from personality (P3-4 completion).
            personality: child_personality.clone(),
            goals: Vec::new(),
            beliefs: Self::foundational_beliefs(),
            affect: Affect::default(),
            emotions: DiscreteEmotions::default(),
            current_action: ActionKind::Idle,
            action_progress: 0,
            name: child_name,
            position: inherited_position,
            home_site: inherited_home,
            memory: MemoryStore::new(200),
            identity: IdentityState { identities: vec![] },
            attention: AttentionState::default(),
            intention: None,
            routine: DailyRoutine::village_routine(),
            // Iteration 246: the replacement newborn inherits the deceased
            // household's values (sole-parent blend) + community prior.
            moral_values: {
                let prior = Self::community_moral_prior(self);
                MoralValues::inherit(&self.agents[idx].moral_values, None, &prior, &mut child_rng)
            },
            cognitive: CognitiveState::default(),
            derived: DerivedMentalState::default(),
            relational_fields: Default::default(),
            recent_successes: 0,
            recent_attempts: 0,
            age: child_age,
            wealth: WealthState {
                coin: Fixed::from_f64(1.0),
            },
            conflict: ConflictState::default(),
            cultural: CulturalState::default(),
            partner: None,
            parent_a: None,
            parent_b: None,
            feuds: Vec::new(),
            feud_ticks: Vec::new(),
            status: StatusState::default(),
            rejected_goals: Vec::new(),
            completed_goals: Vec::new(),
            skills: AgentSkills::default(),
            embodied: child_embodied,
            interoception: {
                let mut inter = crate::psychology::InteroceptiveState::default();
                inter.initialize_from_personality(child_neuroticism, child_openness, Fixed::ZERO);
                inter
            },
            self_model: crate::psychology::SelfModel::default(),
            attachment: {
                let mut att = crate::psychology::AttachmentSystem::default();
                att.initialize(
                    Fixed::from_f64(0.7), // high caregiver security for child
                    Fixed::ZERO,          // no trauma history
                    child_attachment_vulnerability,
                );
                att
            },
            emotion_regulation: {
                // §8.1.8 (P2/P3 re-audit — P3-4 completion): newborns must get
                // the same personality-driven strategy preference as the
                // initial population. Previously they took
                // `EmotionRegulationState::default()` (permanently Reappraisal),
                // so in death-heavy worlds (pestilence) the population refilled
                // with default-Reappraisal newborns and the probe showed the
                // strategy mix collapsing to Reappraisal:12 at 10K — the exact
                // single-strategy uniformity P3-4 exists to eliminate. The
                // personality traits are already in scope (child_personality
                // was moved into the bundle above), so seed from them.
                let mut reg = crate::psychology::EmotionRegulationState::default();
                reg.initialize_from_personality(&child_personality);
                reg
            },
            moral_cognition: crate::psychology::MoralCognition::default(),
            prospection: crate::psychology::ProspectionState::default(),
            narrative: crate::psychology::NarrativeIdentity::default(),
            developmental: crate::psychology::DevelopmentalPsychState::default(),
            psychopathology: crate::psychology::PsychopathologyState::default(),
            mind_models: crate::psychology::theory_of_mind::MindModels::default(),
            cultural_cognition: crate::psychology::CulturalCognition::default(),
            psych_skills: crate::psychology::SkillState::default(),
            relationship_v2s: Vec::new(),
            speech_log: Vec::new(),
            attraction: crate::social::attraction::AttractionModel::default(),
            status_v2: crate::social::status_dims::StatusDimensions::default(),
            epistemic: crate::social::epistemic::EpistemicState::default(),
            cognitive_runtime: crate::psychology::CognitiveRuntime::default(),
            motivation: crate::psychology::MotivationState::default(),
            neural_like: crate::psychology::neural_like::NeuralLikeState::default(),
            decision_policy: crate::psychology::DecisionPolicy::default(),
            agent_tier: crate::agent_tier::AgentTierState::new(
                crate::agent_tier::AgentTier::Secondary,
                tick_u64,
            ),
            narrative_frames: crate::culture::narrative_frame::NarrativeFrameSet::default(),
            ideology: crate::culture::ideology::Ideology {
                axes: vec![
                    crate::culture::ideology::IdeologyAxis {
                        name: "tradition".into(),
                        position: Fixed::from_f64(0.5),
                        conviction: Fixed::from_f64(0.5),
                    },
                    crate::culture::ideology::IdeologyAxis {
                        name: "authority".into(),
                        position: Fixed::from_f64(0.5),
                        conviction: Fixed::from_f64(0.5),
                    },
                ],
                dogmatism: Fixed::from_f64(0.4),
                echo_chamber_strength: Fixed::from_f64(0.3),
                polarization_tendency: Fixed::from_f64(0.3),
            },
            sacred_values: crate::culture::sacred::SacredValues::default(),
            legitimacy_field: crate::noosphere::LegitimacyField::new(Fixed::from_f64(0.5)),
            education: crate::culture::education::EducationState {
                learning_aptitude: Fixed::from_f64(0.6),
                ..crate::culture::education::EducationState::default()
            },
        }
    }

    /// §31: Handle an agent's death via in-place generational replacement.
    ///
    /// Runs inheritance, dissolves marriages, clears partner/parent/feud
    /// references, removes the dead agent from institutions/households/kinship,
    /// and repopulates the slot with a fresh newborn so index invariants hold.
    pub(super) fn handle_agent_death(
        &mut self,
        idx: usize,
        deaths: &[usize],
        tick_u64: u64,
        tick: Tick,
        cause: DeathCause,
    ) {
        let agent_id = AgentId::new(idx as u64);
        tracing::warn!(
            agent = self.agents[idx].name.as_str(),
            age = self.agents[idx].age.to_f64(),
            tick = tick_u64,
            "Agent died"
        );

        // §19.5.F: Inheritance — transfer wealth to spouse and children
        let inherited_wealth = self.agents[idx].wealth.coin;
        if inherited_wealth > Fixed::ZERO {
            let mut heirs: Vec<usize> = Vec::new();
            // Spouse inherits first — but only if not also dying this tick
            if let Some(spouse_idx) = self.agents[idx].partner {
                if spouse_idx < self.agents.len() && !deaths.contains(&spouse_idx) {
                    heirs.push(spouse_idx);
                }
            }
            // Children inherit second — skip if also dying
            for child_idx in 0..self.agents.len() {
                if child_idx != idx && !deaths.contains(&child_idx) {
                    let is_child = self.agents[child_idx].parent_a == Some(idx)
                        || self.agents[child_idx].parent_b == Some(idx);
                    if is_child {
                        heirs.push(child_idx);
                    }
                }
            }
            // Split wealth among heirs (or give to a random agent if no heirs)
            if !heirs.is_empty() {
                let share = inherited_wealth / Fixed::from_f64(heirs.len() as f64);
                for &heir_idx in &heirs {
                    if heir_idx < self.agents.len() {
                        self.agents[heir_idx].wealth.coin += share;
                    }
                }
            }
            self.journal.record(
                tick_u64,
                agent_id,
                JournalEntryKind::Inheritance {
                    heir_count: heirs.len() as u64,
                    amount: inherited_wealth.to_f64(),
                },
            );
        }
        let cause_label = match cause {
            DeathCause::Disease => "disease",
            _ => "natural",
        };
        self.journal.record(
            tick_u64,
            agent_id,
            JournalEntryKind::Died {
                age: self.agents[idx].age.to_f64(),
                cause: cause_label.into(),
            },
        );
        self.events.push(SimEvent::AgentDied {
            agent: agent_id,
            cause,
            tick,
        });

        // ── Dissolve marriages and pair bonds involving the deceased ──
        for marriage in &mut self.marriage_registry.marriages {
            if marriage.active && (marriage.partner_a == idx || marriage.partner_b == idx) {
                marriage.dissolve(&crate::social::marriage::DissolutionCause::Death(idx));
            }
        }
        self.marriage_registry
            .pair_bonds
            .retain(|pb| pb.partner_a != idx && pb.partner_b != idx);

        // ── Clear partner / parent references pointing at the deceased ──
        for agent in &mut self.agents {
            if agent.partner == Some(idx) {
                agent.partner = None;
            }
            if agent.parent_a == Some(idx) {
                agent.parent_a = None;
            }
            if agent.parent_b == Some(idx) {
                agent.parent_b = None;
            }
            agent.feuds.retain(|f| *f != idx);
        }

        // ── Remove from relationships ──
        self.relationships
            .retain(|r| r.from != agent_id && r.to != agent_id);

        // ── Remove from institutions (membership + role holders) ──
        for inst in &mut self.institutions {
            inst.remove_member(agent_id);
            for role in &mut inst.roles {
                if role.holder == Some(agent_id) {
                    role.holder = None;
                }
            }
        }

        // ── Remove from households ──
        for hh in &mut self.households {
            hh.remove_member(idx);
            if hh.head == Some(idx) {
                hh.head = hh.members.first().copied();
            }
        }

        // ── Remove kinship edges referencing the deceased ──
        self.kinship_graph
            .edges
            .retain(|e| e.from != idx && e.to != idx);

        // ── Remove from faction v2 member lists ──
        for faction in &mut self.faction_v2_registry.factions {
            faction.members.retain(|m| *m != idx);
            if faction.leader == idx {
                faction.leader = faction.members.first().copied().unwrap_or(usize::MAX);
            }
        }
        // Dissolve factions that lost all members — a `usize::MAX` leader
        // sentinel would otherwise dangle until the next daily_update.
        self.faction_v2_registry
            .factions
            .retain(|f| !f.members.is_empty());

        // ── Remove from courtships and patronage ──
        self.active_courtships
            .retain(|c| c.pursuer != idx && c.pursued != idx);
        self.patronage_registry
            .relations
            .retain(|r| r.patron != idx && r.client != idx);

        // ── In-place generational replacement: a newborn takes the slot ──
        self.agent_diseases[idx] = Vec::new();
        self.agents[idx] = self.build_replacement_newborn(idx, tick_u64);

        // Rebuild relationship_v2s for the newborn and for all other agents'
        // entries pointing at this index (they now reference a stranger).
        self.rebuild_relationship_v2s_after_death(idx);
    }

    /// Rebuild the O(1) relationship_v2s matrix around a dead-then-reborn slot.
    ///
    /// The dead agent's v2s entries referencing others are replaced with fresh
    /// stranger-level entries, and every other agent's entry pointing at `idx`
    /// is reset to stranger level (the old high-trust bonds died with them).
    ///
    /// Invariant: `relationship_v2s` rows must stay square (row i lists every
    /// other live agent exactly once, targets ordered 0..n excluding self).
    /// Births preserve this by APPENDING the newborn's row + one entry per
    /// existing agent (see the v2s extension in `tick_birth_mechanics`) — do
    /// not "simplify" that append as redundant; it is what keeps this
    /// O(1)-slot lookup from indexing past a shorter vec after a birth.
    fn rebuild_relationship_v2s_after_death(&mut self, idx: usize) {
        let n = self.agents.len();
        let agent_id = AgentId::new(idx as u64);

        // Fresh stranger-level v2s for the newborn.
        let mut v2s = Vec::with_capacity(n.saturating_sub(1));
        for j in 0..n {
            if j != idx {
                v2s.push(RelationshipV2::new(agent_id, AgentId::new(j as u64)));
            }
        }
        self.agents[idx].relationship_v2s = v2s;

        // Reset other agents' entry that points at idx.
        for j in 0..n {
            if j == idx {
                continue;
            }
            // In agent j's v2s, the entry for target idx sits at a fixed slot:
            // targets are ordered 0..n excluding j, so target idx is at
            // position idx if idx < j else idx - 1.
            let pos = if idx < j { idx } else { idx - 1 };
            if let Some(entry) = self.agents[j].relationship_v2s.get_mut(pos) {
                *entry = RelationshipV2::new(AgentId::new(j as u64), agent_id);
            }
        }
    }

    pub(super) fn tick_birth_mechanics(&mut self, tick_u64: u64, tick: Tick) {
        // ── 19b. §19.5.F Birth mechanics — new agents from partnered couples ──
        // §7.2.6 (Iteration 92): the conception→pregnancy→birth pipeline is
        // now live. The demography roll (`should_birth` — the calibrated
        // annual 0.3/couple rate) is the CONCEPTION decision: when it fires
        // for a couple with a female partner, a pregnancy starts on the
        // female instead of an immediate birth; gestation advances in the
        // per-tick biology pass (`EmbodiedState::tick_update` →
        // `reproductive.tick_update`, rate 0.001 × health × nutrition ×
        // multiplier ≈ 1700–1900 ticks to term at default nutrition 0.6);
        // when the pregnancy reaches full term the newborn is born here.
        // Same-sex couples (no female partner) keep the legacy immediate-
        // birth path — their behavior is unchanged.
        //
        // Zero drift by construction: the unconditional Social roll per
        // couple per deca-pass is preserved (byte-identical RNG stream; the
        // newborn still uses its separately-seeded child_rng and birth adds
        // no draws), and on the default seed-42 world no conception fires
        // before tick 22,010 (probe-verified) — every snapshot/golden
        // horizon ≤ 2000 stays byte-identical.
        {
            let n = self.agents.len();
            // (parent_a: usize, parent_b: Option<usize>, child_idx) — parent_b
            // is None only for a widow birth (the father died mid-gestation).
            let mut new_births: Vec<(usize, Option<usize>, usize)> = Vec::new();

            // ── Conception pass: the should_birth roll now starts pregnancies ──

            for i in 0..n {
                if let Some(partner_idx) = self.agents[i].partner {
                    if partner_idx <= i {
                        continue; // only process each couple once
                    }
                    let age_a = self.agents[i].age.to_f64();
                    let age_b = self.agents[partner_idx].age.to_f64();

                    let health_a = self.agents[i].body.health;
                    let health_b = self.agents[partner_idx].body.health;
                    let min_health = health_a.min(health_b);
                    let rng_val =
                        Fixed::from_f64(self.rng.get_mut(RngStream::Social).random::<f64>());
                    // Count existing children for this specific couple
                    let existing_children = self
                        .agents
                        .iter()
                        .filter(|a| {
                            (a.parent_a == Some(i) && a.parent_b == Some(partner_idx))
                                || (a.parent_a == Some(partner_idx) && a.parent_b == Some(i))
                        })
                        .count();
                    // Iteration 220: Extract biology-driven conception parameters.
                    // fertility and libido from the reproductive state of both partners
                    // (age, health, stress, bonding-dependent from §7.2.6).
                    let fertility_a = self.agents[i].embodied.reproductive.fertility;
                    let fertility_b = self.agents[partner_idx].embodied.reproductive.fertility;
                    let min_fertility = fertility_a.min(fertility_b);
                    let libido_a = self.agents[i].embodied.reproductive.libido;
                    let libido_b = self.agents[partner_idx].embodied.reproductive.libido;
                    let avg_libido = (libido_a + libido_b) * Fixed::from_f64(0.5);
                    // nutrition: the couple's AVERAGE digestive health
                    // (gut_health reflects actual nutritional status, not the
                    // volatile world stock). Iteration 242: was MIN(gut_a,
                    // gut_b) — a single partner whose gut had floored at the
                    // 0.1 chronic-spoilage floor sterilized the COUPLE
                    // outright (probe: seed 5, product 0.016 vs the 0.084
                    // normalization anchor → ~0.06 conceptions/couple/year).
                    // Averaging keeps deficiency meaningful without letting
                    // one gut veto fertility.
                    let gut_a = self.agents[i].embodied.digestive.gut_health;
                    let gut_b = self.agents[partner_idx].embodied.digestive.gut_health;
                    let nutrition = (gut_a + gut_b) * Fixed::from_f64(0.5);
                    // intimacy: relationship quality with partner.
                    let v2_pos = Self::relationship_v2_pos(i, partner_idx);
                    let intimacy = if v2_pos < self.agents[i].relationship_v2s.len() {
                        self.agents[i].relationship_v2s[v2_pos].quality()
                    } else {
                        Fixed::from_f64(0.3) // baseline
                    };
                    // parental_drive: from the reproductive state.
                    let pd_a = self.agents[i].embodied.reproductive.parental_drive;
                    let pd_b = self.agents[partner_idx]
                        .embodied
                        .reproductive
                        .parental_drive;
                    let avg_pd = (pd_a + pd_b) * Fixed::from_f64(0.5);
                    // Iteration 220: use the younger partner's age for the childbearing
                    // check (not avg_age, which filters out couples where one
                    // partner is older). At least one partner must be in range.
                    let younger_age = age_a.min(age_b);
                    let should = demography::should_birth(
                        true,
                        min_health,
                        younger_age,
                        existing_children,
                        DEMOGRAPHY_TICK_INTERVAL,
                        &self.demography_config,
                        rng_val,
                        self.params.reproduction_conception_multiplier.to_f64(),
                        min_fertility,
                        avg_libido,
                        nutrition,
                        intimacy,
                        avg_pd,
                    );

                    if should {
                        // The female partner carries the pregnancy; a couple
                        // with no female partner (same-sex) keeps the legacy
                        // immediate birth (abstraction — behavior unchanged).
                        // An already-pregnant female makes the fired roll a
                        // natural no-op: the pregnancy must complete before a
                        // new conception (birth spacing emerges from the
                        // gestation delay).
                        let female = if self.agents[i].embodied.reproductive.sex
                            == crate::biology::reproductive::BiologicalSex::Female
                        {
                            Some(i)
                        } else if self.agents[partner_idx].embodied.reproductive.sex
                            == crate::biology::reproductive::BiologicalSex::Female
                        {
                            Some(partner_idx)
                        } else {
                            None
                        };
                        match female {
                            Some(mother)
                                if n + new_births.len() < crate::population_cap::MAX_POPULATION
                                    && self.agents[mother]
                                        .embodied
                                        .reproductive
                                        .pregnancy
                                        .is_none() =>
                            {
                                self.agents[mother].embodied.reproductive.pregnancy = Some(
                                    crate::biology::reproductive::PregnancyState::new(tick_u64),
                                );
                            }
                            None if n + new_births.len()
                                < crate::population_cap::MAX_POPULATION =>
                            {
                                new_births.push((i, Some(partner_idx), n + new_births.len()));
                            }
                            _ => {}
                        }
                    }
                }
            }

            // ── Birth pass: full-term pregnancies deliver newborns ──
            // Deterministic (no RNG): the pregnancy's `complete_pregnancy`
            // fires exactly when gestation_progress reaches 1.0, which the
            // per-tick biology pass advances. Iteration 92: the pass is keyed
            // off the pregnancy itself, not the couple — a mother whose
            // husband died mid-gestation (her partner field cleared by
            // handle_agent_death) still delivers. The father is resolved
            // best-effort: live partner → active marriage's other spouse →
            // None (a widow birth is recorded as single-parent).
            let n2 = self.agents.len();
            for mother in 0..n2 {
                let full_term = self.agents[mother]
                    .embodied
                    .reproductive
                    .pregnancy
                    .as_ref()
                    .is_some_and(|p| p.gestation_progress >= Fixed::ONE);
                if full_term
                    && n2 + new_births.len() < crate::population_cap::MAX_POPULATION
                    && self.agents[mother]
                        .embodied
                        .reproductive
                        .complete_pregnancy()
                {
                    let father = self.agents[mother]
                        .partner
                        .filter(|p| *p != mother && *p < n2)
                        .or_else(|| {
                            self.marriage_registry
                                .marriages
                                .iter()
                                .find(|m| {
                                    m.active && (m.partner_a == mother || m.partner_b == mother)
                                })
                                .map(|m| {
                                    if m.partner_a == mother {
                                        m.partner_b
                                    } else {
                                        m.partner_a
                                    }
                                })
                        });
                    new_births.push((mother, father, n2 + new_births.len()));
                }
            }

            for (parent_a, parent_b, child_idx) in new_births {
                // Iteration 245 (Arc A heredity): children carry the
                // maternal family line instead of a "Child_N" placeholder;
                // the given name cycles the shared founder pool.
                let child_name = crate::person::inherit_surname(
                    &self.agents[parent_a].name,
                    crate::person::FIRST_NAMES[child_idx % crate::person::FIRST_NAMES.len()],
                );
                // Inherit personality traits with noise
                let mut child_rng = rand_chacha::ChaCha8Rng::seed_from_u64(
                    self.config
                        .seed
                        .wrapping_add(tick_u64)
                        .wrapping_add(child_idx as u64),
                );
                // Iteration 244 (Arc A heredity): genome + personality now
                // blend from the parents instead of re-rolling — vertical
                // transmission makes family resemblance a population-level
                // property. `Personality::inherit` consumes exactly one RNG
                // draw per trait, so downstream stream alignment matches
                // the old `random` call count.
                let father_embodied = parent_b
                    .and_then(|fb| self.agents.get(fb))
                    .map(|p| &p.embodied);
                let child_personality = Personality::inherit(
                    &self.agents[parent_a].personality,
                    parent_b
                        .and_then(|fb| self.agents.get(fb))
                        .map(|p| &p.personality),
                    &mut child_rng,
                );
                // Iteration 239 (H6): personality-driven emotion-regulation
                // seeding must match the initial population AND
                // build_replacement_newborn (P3-4 contract). The real-birth
                // path previously took EmotionRegulationState::default()
                // (permanent Reappraisal), so death-heavy worlds refilled
                // with default-strategy newborns — the exact single-strategy
                // collapse P3-4 documents.
                let child_reg_personality = child_personality.clone();
                let child_age = Fixed::from_f64(0.0); // newborn

                // Allocate home site
                let home_site = self.agents[parent_a].home_site;

                let agent_id = AgentId::new(child_idx as u64);

                let child_embodied = EmbodiedState::born(
                    &self.agents[parent_a].embodied,
                    father_embodied,
                    child_age,
                    &mut child_rng,
                );
                let child_attachment_vulnerability = child_embodied
                    .genome
                    .trait_predispositions
                    .attachment_vulnerability;
                let child_neuroticism = child_personality.neuroticism;
                let child_openness = child_personality.openness;
                self.agents.push(AgentBundle {
                    body: BodyState::from(&child_embodied),
                    needs: NeedState::default(),
                    personality: child_personality,
                    goals: Vec::new(),
                    beliefs: Self::foundational_beliefs(),
                    affect: Affect::default(),
                    emotions: DiscreteEmotions::default(),
                    current_action: ActionKind::Idle,
                    action_progress: 0,
                    name: child_name,
                    position: self.agents[parent_a].position,
                    home_site,
                    memory: MemoryStore::new(200),
                    identity: IdentityState { identities: vec![] },
                    attention: AttentionState::default(),
                    intention: None,
                    routine: DailyRoutine::village_routine(),
                    // Iteration 246 (Arc A heredity): values transmit from
                    // both parents, blended with the community prior.
                    moral_values: {
                        let prior = self.community_moral_prior();
                        let mother_vals = self.agents[parent_a].moral_values.clone();
                        let father_vals = parent_b.map(|p| self.agents[p].moral_values.clone());
                        MoralValues::inherit(
                            &mother_vals,
                            father_vals.as_ref(),
                            &prior,
                            &mut child_rng,
                        )
                    },
                    cognitive: CognitiveState::default(),
                    derived: DerivedMentalState::default(),
                    relational_fields: Default::default(),
                    recent_successes: 0,
                    recent_attempts: 0,
                    age: child_age,
                    wealth: WealthState {
                        coin: Fixed::from_f64(1.0),
                    },
                    conflict: ConflictState::default(),
                    cultural: CulturalState::default(),
                    partner: None,
                    parent_a: Some(parent_a),
                    // parent_b is None only for widow births (father died
                    // mid-gestation) — recorded as single-parent.
                    parent_b,
                    feuds: Vec::new(),
                    feud_ticks: Vec::new(),
                    status: StatusState::default(),
                    rejected_goals: Vec::new(),
                    completed_goals: Vec::new(),
                    skills: AgentSkills::default(),
                    embodied: child_embodied,
                    interoception: {
                        let mut inter = crate::psychology::InteroceptiveState::default();
                        inter.initialize_from_personality(
                            child_neuroticism,
                            child_openness,
                            Fixed::ZERO,
                        );
                        inter
                    },
                    self_model: crate::psychology::SelfModel::default(),
                    attachment: {
                        let mut att = crate::psychology::AttachmentSystem::default();
                        att.initialize(
                            Fixed::from_f64(0.7), // high caregiver security for child
                            Fixed::ZERO,          // no trauma history
                            child_attachment_vulnerability,
                        );
                        att
                    },
                    emotion_regulation: {
                        let mut reg = crate::psychology::EmotionRegulationState::default();
                        reg.initialize_from_personality(&child_reg_personality);
                        reg
                    },
                    moral_cognition: crate::psychology::MoralCognition::default(),
                    prospection: crate::psychology::ProspectionState::default(),
                    narrative: crate::psychology::NarrativeIdentity::default(),
                    developmental: crate::psychology::DevelopmentalPsychState::default(),
                    psychopathology: crate::psychology::PsychopathologyState::default(),
                    mind_models: crate::psychology::theory_of_mind::MindModels::default(),
                    cultural_cognition: crate::psychology::CulturalCognition::default(),
                    psych_skills: crate::psychology::SkillState::default(),
                    relationship_v2s: Vec::new(),
                    speech_log: Vec::new(),
                    attraction: crate::social::attraction::AttractionModel::default(),
                    status_v2: crate::social::status_dims::StatusDimensions::default(),
                    epistemic: crate::social::epistemic::EpistemicState::default(),
                    cognitive_runtime: crate::psychology::CognitiveRuntime::default(),
                    motivation: crate::psychology::MotivationState::default(),
                    neural_like: crate::psychology::neural_like::NeuralLikeState::default(),
                    decision_policy: crate::psychology::DecisionPolicy::default(),
                    agent_tier: crate::agent_tier::AgentTierState::new(
                        crate::agent_tier::AgentTier::Secondary,
                        tick_u64,
                    ),
                    narrative_frames: crate::culture::narrative_frame::NarrativeFrameSet::default(),
                    ideology: crate::culture::ideology::Ideology {
                        axes: vec![
                            crate::culture::ideology::IdeologyAxis {
                                name: "tradition".into(),
                                position: Fixed::from_f64(0.5),
                                conviction: Fixed::from_f64(0.5),
                            },
                            crate::culture::ideology::IdeologyAxis {
                                name: "authority".into(),
                                position: Fixed::from_f64(0.5),
                                conviction: Fixed::from_f64(0.5),
                            },
                        ],
                        dogmatism: Fixed::from_f64(0.4),
                        echo_chamber_strength: Fixed::from_f64(0.3),
                        polarization_tendency: Fixed::from_f64(0.3),
                    },
                    sacred_values: crate::culture::sacred::SacredValues::default(),
                    legitimacy_field: crate::noosphere::LegitimacyField::new(Fixed::from_f64(0.5)),
                    education: crate::culture::education::EducationState {
                        learning_aptitude: Fixed::from_f64(0.6),
                        ..crate::culture::education::EducationState::default()
                    },
                });
                // §10.2 (Iteration 92): keep the O(1) relationship_v2s matrix
                // complete — the newborn must appear in every agent's vec and
                // carry stranger-level entries for everyone. Previously a
                // birth only appended the agent (with an empty v2s), so a
                // birth after a death-replacement left the matrix asymmetric
                // and the daily power-balance pass could index past a vec's
                // length (latent panic — surfaced by accelerated population
                // testing). The child is the last index, so its slot in any
                // vec is the last position — append.
                let new_idx = self.agents.len() - 1;
                for j in 0..new_idx {
                    self.agents[j]
                        .relationship_v2s
                        .push(RelationshipV2::new(AgentId::new(j as u64), agent_id));
                }
                self.agents[new_idx].relationship_v2s = (0..new_idx)
                    .map(|j| RelationshipV2::new(agent_id, AgentId::new(j as u64)))
                    .collect();
                // Add relationships to all existing agents
                for existing_idx in 0..self.agents.len() - 1 {
                    let trust = if existing_idx == parent_a || parent_b == Some(existing_idx) {
                        Fixed::from_f64(0.8) // high trust for parents
                    } else {
                        Fixed::from_f64(0.4) // moderate trust for others
                    };
                    self.relationships.push(Relationship {
                        from: agent_id,
                        to: AgentId::new(existing_idx as u64),
                        trust,
                        affection: trust * Fixed::from_f64(0.8),
                        respect: Fixed::from_f64(0.3),
                        fear: Fixed::ZERO,
                        obligation: Fixed::ZERO,
                        last_interaction_tick: tick_u64,
                        kind: RelationshipKind::Kin,
                        interaction_count: 1,
                        last_positive_tick: tick_u64,
                        last_negative_tick: 0,
                    });
                    self.relationships.push(Relationship {
                        from: AgentId::new(existing_idx as u64),
                        to: agent_id,
                        trust,
                        affection: trust * Fixed::from_f64(0.5),
                        respect: Fixed::ZERO,
                        fear: Fixed::ZERO,
                        obligation: Fixed::ZERO,
                        last_interaction_tick: tick_u64,
                        kind: RelationshipKind::Stranger,
                        interaction_count: 0,
                        last_positive_tick: 0,
                        last_negative_tick: 0,
                    });
                }

                // §10.6 (AP2): Mirror the new family into the kinship graph.
                // Previously only populate-time edges were created — and the
                // initial adults have no parents — so the graph stayed empty
                // for the entire run: no parent/child edges on birth, no
                // sibling edges, and the §10.3 kin stages + kinship-coefficient
                // social gates had no edges to work with. Wired here,
                // deterministically (no RNG): parent↔child links in both
                // directions, plus sibling links to every prior child of
                // either parent.
                self.kinship_graph.add_link(
                    parent_a,
                    child_idx,
                    crate::social::kinship::KinshipLink::ParentChild,
                    tick_u64,
                );
                self.kinship_graph.add_link(
                    child_idx,
                    parent_a,
                    crate::social::kinship::KinshipLink::ParentChild,
                    tick_u64,
                );
                if let Some(father) = parent_b {
                    self.kinship_graph.add_link(
                        father,
                        child_idx,
                        crate::social::kinship::KinshipLink::ParentChild,
                        tick_u64,
                    );
                    self.kinship_graph.add_link(
                        child_idx,
                        father,
                        crate::social::kinship::KinshipLink::ParentChild,
                        tick_u64,
                    );
                }
                for k in 0..child_idx {
                    if k == parent_a || parent_b == Some(k) {
                        continue;
                    }
                    let shares = self.agents[k].parent_a == Some(parent_a)
                        || self.agents[k].parent_b == Some(parent_a)
                        || parent_b.is_some_and(|fb| {
                            self.agents[k].parent_a == Some(fb)
                                || self.agents[k].parent_b == Some(fb)
                        });
                    if shares {
                        self.kinship_graph.add_link(
                            child_idx,
                            k,
                            crate::social::kinship::KinshipLink::Sibling,
                            tick_u64,
                        );
                        self.kinship_graph.add_link(
                            k,
                            child_idx,
                            crate::social::kinship::KinshipLink::Sibling,
                            tick_u64,
                        );
                    }
                }

                // Disease tracking for new agent
                self.agent_diseases.push(Vec::new());

                self.events.push(SimEvent::ChildBorn {
                    child: agent_id,
                    parent_a: AgentId::new(parent_a as u64),
                    // A widow birth (father unknown) records the mother in
                    // the second slot — the event type has no None variant.
                    parent_b: parent_b
                        .map_or(AgentId::new(parent_a as u64), |p| AgentId::new(p as u64)),
                    tick,
                });

                tracing::info!(
                    child = child_idx,
                    parent_a = parent_a,
                    parent_b = parent_b,
                    tick = tick_u64,
                    "Child born"
                );

                // §7.2.6 (Iteration 92): record the child in the mother's
                // active marriage — `Marriage.children` was write-only (zero
                // consumers); it now records "children produced by this
                // marriage" at birth, observational.
                if let Some(m) = self
                    .marriage_registry
                    .marriages
                    .iter_mut()
                    .find(|m| m.active && (m.partner_a == parent_a || m.partner_b == parent_a))
                {
                    m.children.push(child_idx);
                }
            }
        }
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
    fn tick_pair_bonds(&mut self, tick_u64: u64) {
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
