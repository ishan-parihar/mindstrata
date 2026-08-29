//! Births, in-place generational replacement and death handling.

use super::{
    ActionKind, Affect, AgentBundle, AgentId, AgentSkills, AttentionState, BodyState,
    CognitiveState, ConflictState, CulturalState, DailyRoutine, DeathCause, DerivedMentalState,
    DiscreteEmotions, EmbodiedState, Fixed, IdentityState, JournalEntryKind, MemoryStore,
    MoralValues, NeedState, Personality, Relationship, RelationshipKind, RelationshipV2, RngStream,
    SimEvent, Simulation, StatusState, Tick, WealthState, DEMOGRAPHY_TICK_INTERVAL,
};
use crate::demography;

use rand::Rng;
use rand::SeedableRng;
impl Simulation {
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
            // Iteration 247 (Arc A carry-over): the replacement newborn
            // inherits the deceased household's ideology as sole parent.
            ideology: {
                let mut ideo_rng = rand_chacha::ChaCha8Rng::seed_from_u64(
                    self.config.seed ^ tick_u64.wrapping_mul(0xA1) ^ (idx as u64),
                );
                self.agents[idx]
                    .ideology
                    .clone()
                    .inherit(None, &mut ideo_rng)
            },
            // Iteration 260 (Arc-A carry-over closure): the replacement
            // newborn inherits the deceased household's earned sacred
            // values verbatim (sole-parent pattern, mirroring the ideology
            // inheritance above). Deterministic clone - cultural fidelity
            // needs no noise draw; jitter can be added when evidence
            // demands it.
            sacred_values: self.agents[idx].sacred_values.clone(),
            legitimacy_field: crate::noosphere::LegitimacyField::new(Fixed::from_f64(0.5)),
            education: crate::culture::education::EducationState {
                learning_aptitude: Fixed::from_f64(0.6),
                ..crate::culture::education::EducationState::default()
            },
            // AP3 DC-1 (task 3.6): replacement heredity — sole-parent
            // vertical transmission with one draw per line (stream
            // discipline §5); pathology resets to neutral. Derived RNG
            // with 0xB2 domain separation from the ideology 0xA1 stream
            // so the two inheritances do not share draws.
            development: {
                let mut dev_rng = rand_chacha::ChaCha8Rng::seed_from_u64(
                    self.config.seed ^ tick_u64.wrapping_mul(0xB2) ^ (idx as u64),
                );
                crate::psychology::DevelopmentFieldState::inherited(
                    &self.agents[idx].development,
                    None,
                    &mut dev_rng,
                )
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
                    // Iteration 247 (Arc A carry-over): ideology transmits
                    // from both parents with enculturation noise. Derived
                    // RNG stream — zero interference with child_rng draw
                    // order downstream.
                    ideology: {
                        let mut ideo_rng = rand_chacha::ChaCha8Rng::seed_from_u64(
                            self.config.seed ^ tick_u64.wrapping_mul(0xA1) ^ (child_idx as u64),
                        );
                        let mother_ideo = self.agents[parent_a].ideology.clone();
                        let father_ideo = parent_b.map(|p| self.agents[p].ideology.clone());
                        mother_ideo.inherit(father_ideo.as_ref(), &mut ideo_rng)
                    },
                    // Iteration 260 (Arc-A carry-over closure): maternal
                    // vertical transmission of the community's earned
                    // sacred values - children no longer start
                    // sacred-less while every founder holds seeded ones.
                    // Maternal line matches the surname convention;
                    // deterministic clone, no RNG draws.
                    sacred_values: self.agents[parent_a].sacred_values.clone(),
                    legitimacy_field: crate::noosphere::LegitimacyField::new(Fixed::from_f64(0.5)),
                    education: crate::culture::education::EducationState {
                        learning_aptitude: Fixed::from_f64(0.6),
                        ..crate::culture::education::EducationState::default()
                    },
                    // AP3 DC-1 (task 3.6): vertical transmission — blended
                    // mid-parent altitudes plus per-line noise, one draw per
                    // line (stream discipline §5, matching ideology). Separate
                    // 0xB2 RNG domain from ideology's 0xA1 so streams do not
                    // share draws; pathology resets to neutral at birth.
                    development: {
                        let mut dev_rng = rand_chacha::ChaCha8Rng::seed_from_u64(
                            self.config.seed ^ tick_u64.wrapping_mul(0xB2) ^ (child_idx as u64),
                        );
                        let mother_dev = &self.agents[parent_a].development;
                        let father_dev = parent_b.map(|p| &self.agents[p].development);
                        crate::psychology::DevelopmentFieldState::inherited(
                            mother_dev,
                            father_dev,
                            &mut dev_rng,
                        )
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
}
