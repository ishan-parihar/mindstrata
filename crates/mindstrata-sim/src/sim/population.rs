//! Population lifecycle: constructors, scenario/snapshot loading, initial seeding and agent population.

use super::{
    ActionKind, Affect, AgentBundle, AgentId, AgentSkills, AttentionState, BodyState,
    CausalProvenance, Clock, CognitiveState, ConflictState, CulturalState, DailyRoutine,
    DerivedMentalState, DiplomacyRegistry, DiscreteEmotions, EmbodiedState, EntityId, EventJournal,
    Fixed, IdentityKind, IdentityState, Knowledge, KnowledgeCategory, LegalRegistry, MarketState,
    MemoryStore, MilitaryRegistry, MoralValues, NeedState, NormRegistry, Personality, Position,
    Relationship, RelationshipKind, RelationshipV2, RngStreams, Scenario, SchoolRegistry,
    ShockKind, SimConfig, Simulation, StatusState, TechnologyTree, TheologyRegistry, WealthState,
    World,
};
use crate::demography;
use crate::ecology;
use crate::health;
use crate::institutions;
use crate::norms;
use crate::world_gen;

use rand::Rng;
use rand::SeedableRng;
impl Simulation {
    /// Create a new simulation from config.
    pub fn new(config: SimConfig) -> Self {
        let rng = RngStreams::new(config.seed);
        let world = World::new(config.world_width, config.world_height);

        let sim = Self {
            config,
            params: crate::parameters::SimParameters::default(),
            clock: Clock::new(),
            rng,
            world,
            agents: Vec::new(),
            relationships: Vec::new(),
            events: Vec::new(),
            total_event_count: 0,
            journal: EventJournal::new(),
            norms: NormRegistry::new(),
            institutions: Vec::new(),
            provenance: CausalProvenance::new(),
            scenario: None,
            season: ecology::SeasonTracker::new(8760),
            weather: ecology::WeatherTracker::new(ecology::WeatherConfig::default()),
            ecology_config: ecology::EcologyConfig::default(),
            health_config: health::HealthConfig::default(),
            demography_config: demography::DemographyConfig::default(),
            agent_diseases: Vec::new(),
            site_work_ticks: Vec::new(),
            market: MarketState::new(&crate::parameters::SimParameters::default()),
            knowledge_store: Vec::new(),
            technology: TechnologyTree::riverford(),
            legal: LegalRegistry::new(),
            diplomacy: DiplomacyRegistry::new(),
            school: SchoolRegistry::new(),
            theology: TheologyRegistry::new(),
            military: MilitaryRegistry::new(),
            metric_history: Vec::new(),
            last_revolution_tick: 0,
            last_moral_panic_tick: 0,
            last_cult_formation_tick: 0,
            famine_until: 0,
            drought_until: 0,
            faction_crisis_pressure: 0.0,
            last_faction_formation_tick: 0,
            famine_production_factor: Fixed::ONE,
            black_market: crate::black_market::BlackMarketState::default(),
            kinship_graph: crate::social::kinship::KinshipGraph::default(),
            households: Vec::new(),
            meme_registry: crate::culture::MemeRegistry::default(),
            meme_aggregator: crate::culture::MemeAggregator::default(),
            propaganda_registry: crate::culture::PropagandaRegistry::default(),
            ritual_registry: crate::culture::RitualRegistry::default(),
            rumor_registry: crate::culture::RumorRegistry::default(),
            collective_memory_registry: crate::culture::CollectiveMemoryRegistry::default(),
            collective_field: mindstrata_development::collective::CollectiveField::default(),
            echo_chamber: crate::culture::EchoChamberState::new(),
            clan_registry: crate::social::clan::ClanRegistry::new(),
            marriage_registry: crate::social::marriage::MarriageRegistry::new(),
            cult_registry: crate::social::cult::CultRegistry::new(),
            moral_panic_registry: crate::noosphere::MoralPanicRegistry::new(),
            noospheric_field: crate::noosphere::NoosphericField::new(),
            faction_v2_registry: crate::social::faction_v2::FactionV2Registry::new(),
            active_courtships: Vec::new(),
            patronage_registry: crate::social::patronage::PatronageRegistry::new(),
            group_registry: crate::social::group_formation::GroupRegistry::new(),
            exposure_samples: Vec::new(),
            tick_trust_deltas: Vec::new(),
            tick_agent_ages: Vec::new(),
            tick_rel_snapshot: Vec::new(),
            tick_action_starts: Vec::new(),
        };
        // Meme seeding moved to `populate()` (Iteration 174): seeding here
        // would read `params.meme_virality_scaling` before the caller's
        // pre-populate override lands, making the tuning knob a no-op.
        sim
    }

    /// Seed the meme registry with the village's founding memes.
    ///
    /// The registry previously started empty, so the meme aggregation and
    /// spread loops early-returned and no cultural dynamics ever emerged
    /// (active_meme_count pinned at 0). Seed a small set of culturally
    /// grounded memes — practical, moral, theological, and political — with
    /// varied emotional charge and identity relevance so transmission is
    /// non-trivial. Seeded with `created_tick = 0` so both `new()` and
    /// `from_snapshot()` produce an identical registry (the snapshot does
    /// not serialize the meme registry), keeping golden replays deterministic.
    fn seed_initial_memes(&mut self) {
        use crate::culture::{Meme, MemeContent};
        let seeds: &[(&str, MemeContent, f64, f64, f64)] = &[
            // (description, content, emotional_charge, identity_relevance, mutation_base)
            (
                "The river feeds the village; honor it",
                MemeContent::Theological,
                0.6,
                0.7,
                0.05,
            ),
            (
                "Hard work before the harvest brings plenty",
                MemeContent::Moral,
                0.4,
                0.5,
                0.04,
            ),
            (
                "A famine is coming — the elders say so",
                MemeContent::Prophecy,
                0.8,
                0.4,
                0.06,
            ),
            (
                "The council is hoarding the well's water",
                MemeContent::Political,
                0.7,
                0.6,
                0.07,
            ),
            (
                "Dry fields mean the spirits are angry",
                MemeContent::Theological,
                0.9,
                0.3,
                0.08,
            ),
        ];
        // Iteration 256 (audit Phase 4 — de-scripted founding): grievance
        // memes are CONDITIONAL on actual world stress. A stable world
        // (no scenario, or no deprivation shocks) never starts with
        // "the council is hoarding the well's water" or "a famine is
        // coming" — grievances must be earned by events. Theological/
        // moral/cultural seeds stay universal (every culture has ritual
        // cosmology), but their descriptions still key off water context
        // below.
        let world_has_water_stress = self.scenario.as_ref().is_some_and(|sc| {
            sc.shocks
                .iter()
                .any(|sh| matches!(sh.kind, ShockKind::Drought | ShockKind::Famine))
        });
        let world_has_hard_times = self
            .scenario
            .as_ref()
            .is_some_and(|sc| !sc.shocks.is_empty());
        for (i, (desc, content, emotional, identity, mutation)) in seeds.iter().enumerate() {
            // Skip unearned-grievance seeds in comfortable worlds.
            let is_grievance_seed = desc.contains("hoarding") || desc.contains("famine");
            if is_grievance_seed && !world_has_hard_times {
                continue;
            }
            let _ = world_has_water_stress;
            self.meme_registry.register(Meme::new(
                i,
                desc.to_string(),
                *content,
                Fixed::from_f64(*emotional),
                Fixed::from_f64(*identity),
                0, // created_tick — deterministic across new()/from_snapshot()
                self.params.meme_virality_scaling, // Iteration 174: the §13.1
                // virality knob is now LIVE — previously hardcoded 0.8, so the
                // tuning parameter was a no-op. Default preserved at 0.8 (the
                // probe-verified envelope: all memes active, differentiated
                // host spread, novelty held by transmission reinforcement).
                Fixed::from_f64(*mutation),
            ));
        }
    }

    /// Create from a scenario definition.
    pub fn from_scenario(scenario: Scenario) -> Self {
        let config = scenario.to_sim_config();
        let mut sim = Self::new(config);
        // Iteration 238: pre-set drought_until if the scenario has a
        // drought shock, so that aquifer and flood recharge are suppressed
        // from tick 0 (not just after the shock fires). Without this,
        // weather-driven flood mode during ticks 0..shock_tick recharges
        // water past the shock drain (400 → 2000+ before the shock, then
        // 70% drain leaves 600 — still above the 150 threshold).
        if let Some(drought_shock) = scenario
            .shocks
            .iter()
            .find(|s| matches!(s.kind, ShockKind::Drought))
        {
            sim.drought_until = drought_shock.at_tick + 3000;
        }
        sim.scenario = Some(scenario);
        sim
    }

    /// §16.1: Restore from a snapshot (deterministic replay).
    pub fn from_snapshot(snapshot: crate::snapshot::Snapshot) -> Self {
        // Warn if loading a snapshot older than v6 (which added group_registry).
        if snapshot.version < crate::snapshot::SNAPSHOT_VERSION {
            tracing::warn!(
                snapshot_version = snapshot.version,
                expected = crate::snapshot::SNAPSHOT_VERSION,
                "Loading old snapshot (v{}) without group_registry — registry will be empty after restore",
                snapshot.version,
            );
        }
        let mut clock = Clock::new();
        clock.set(snapshot.clock_tick);
        let rng = RngStreams::new(snapshot.master_seed);
        let mut sim = Self {
            config: snapshot.config,
            clock,
            params: crate::parameters::SimParameters::default(),
            rng,
            world: snapshot.world,
            agents: snapshot.agents,
            relationships: snapshot.relationships,
            total_event_count: snapshot.events.len() as u64,
            events: snapshot.events,
            journal: snapshot.journal,
            norms: snapshot.norms,
            institutions: snapshot.institutions,
            provenance: snapshot.provenance,
            scenario: None,
            season: snapshot.season,
            // §5 (Iteration 147): snapshots predate weather — restore a fresh
            // tracker; the rng is reseeded from master_seed so weather replays
            // deterministically from the restore tick onward.
            weather: ecology::WeatherTracker::new(ecology::WeatherConfig::default()),
            ecology_config: snapshot.ecology_config,
            health_config: snapshot.health_config,
            demography_config: snapshot.demography_config,
            agent_diseases: snapshot.agent_diseases,
            site_work_ticks: snapshot.site_work_ticks,
            market: snapshot.market,
            // §5 (Iteration 148): snapshots predate technology — restore a
            // fresh riverford tree and reconcile the discovered set with the
            // serialized knowledge_store (a mid-run snapshot may include
            // discovered tier-1+ nodes).
            technology: {
                let mut t = TechnologyTree::riverford();
                t.reconcile_discovered(&snapshot.knowledge_store);
                t
            },
            // §5 (Iteration 149): snapshots predate the court — restore a
            // fresh registry; violations (and hence cases) are probe-pinned
            // absent from every calibrated window, so no mid-run snapshot
            // can contain a case today.
            legal: LegalRegistry::new(),
            // §5 (Iteration 150): snapshots predate diplomacy — restore a
            // fresh network; the rng is reseeded from master_seed so the
            // pass replays deterministically from the restore tick onward.
            diplomacy: DiplomacyRegistry::new(),
            school: SchoolRegistry::new(),
            theology: TheologyRegistry::new(),
            military: MilitaryRegistry::new(),
            knowledge_store: snapshot.knowledge_store,
            metric_history: snapshot.metric_history,
            last_revolution_tick: snapshot.last_revolution_tick,
            last_moral_panic_tick: snapshot.last_moral_panic_tick,
            last_cult_formation_tick: snapshot.last_cult_formation_tick,
            famine_until: 0,
            drought_until: 0,
            faction_crisis_pressure: 0.0,
            last_faction_formation_tick: 0,
            famine_production_factor: Fixed::ONE,
            black_market: snapshot.black_market,
            // §10.6: Serialized since v10 — restore the exact marriage/birth-
            // forged edges; pre-v10 snapshots deserialize an empty graph (the
            // serde default), identical to the pre-v10 replay behavior.
            kinship_graph: snapshot.kinship_graph,
            households: Vec::new(),
            // §13.2: Serialized since v9 — restore the exact mutated lineage
            // state; pre-v9 snapshots deserialize an empty registry and fall
            // back to re-seeding founding memes below.
            meme_registry: if snapshot.meme_registry.memes.is_empty() {
                crate::culture::MemeRegistry::default()
            } else {
                snapshot.meme_registry
            },
            meme_aggregator: crate::culture::MemeAggregator::default(),
            propaganda_registry: crate::culture::PropagandaRegistry::default(),
            ritual_registry: crate::culture::RitualRegistry::default(),
            rumor_registry: crate::culture::RumorRegistry::default(),
            collective_memory_registry: snapshot.collective_memory_registry,
            // DC-1 STORY 11: collective field is fresh on restore (the
            // field is per-tick emergent, not serialized); the daily pass
            // recomputes it deterministically from the catalyst stream.
            collective_field: mindstrata_development::collective::CollectiveField::default(),
            echo_chamber: crate::culture::EchoChamberState::new(),
            clan_registry: crate::social::clan::ClanRegistry::new(),
            marriage_registry: crate::social::marriage::MarriageRegistry::new(),
            cult_registry: crate::social::cult::CultRegistry::new(),
            moral_panic_registry: crate::noosphere::MoralPanicRegistry::new(),
            noospheric_field: crate::noosphere::NoosphericField::new(),
            faction_v2_registry: crate::social::faction_v2::FactionV2Registry::new(),
            active_courtships: Vec::new(),
            patronage_registry: crate::social::patronage::PatronageRegistry::new(),
            group_registry: crate::social::group_formation::GroupRegistry::new(),
            exposure_samples: Vec::new(),
            tick_trust_deltas: Vec::new(),
            tick_agent_ages: Vec::new(),
            tick_rel_snapshot: Vec::new(),
            tick_action_starts: Vec::new(),
        };
        // Rebuild the GroupRegistry membership cache (skipped by serde).
        sim.group_registry.rebuild_cache();
        // §13.2: Memes are serialized in the snapshot (v9+), so replays
        // restore the exact mutated lineage state. Pre-v9 snapshots carry an
        // empty registry and fall back to re-seeding the founding memes
        // (created_tick = 0) to stay deterministic.
        if sim.meme_registry.memes.is_empty() {
            sim.seed_initial_memes();
        }
        // Re-seed rituals/campaigns for the same reason (registries are not
        // serialized; `populate()` and `from_snapshot()` must produce
        // identical registries so replays match fresh runs).
        sim.seed_initial_rituals_and_campaigns();
        // §10.8/§13.5/§13: Re-seed the collective-structure registries
        // (clans from agent home-sites, village collective memory, meme
        // concept nodes) so replays match fresh runs.
        sim.seed_initial_clans();
        sim.seed_initial_collective_memory();
        sim.seed_initial_noosphere();
        sim
    }

    /// Populate the world with terrain, sites, and agents.
    pub fn populate(&mut self) {
        // §13.1 (Iteration 174): Seed the founding memes here — the
        // behavioral-delta contract applies parameter overrides between
        // `new()` and `populate()`, so `meme_virality_scaling` (read at
        // seeding) is only effective at this point. Idempotent: an already-
        // seeded registry (e.g. post-snapshot re-seed) is left untouched.
        if self.meme_registry.memes.is_empty() {
            self.seed_initial_memes();
        }
        // §31: Clamp requested population to the settlement cap so starting above
        // MAX_POPULATION can never silently disable the birth system (the birth
        // gate `n + new_births < MAX_POPULATION` would otherwise never fire).
        let requested = self.config.num_agents as usize;
        self.config.num_agents = requested.min(crate::population_cap::MAX_POPULATION) as u32;
        if self.config.num_agents as usize != requested {
            tracing::warn!(
                requested,
                clamped = self.config.num_agents,
                "Clamping initial population to MAX_POPULATION"
            );
        }

        world_gen::generate_village(&mut self.world, &mut self.rng);

        let mut populate_rng =
            rand_chacha::ChaCha8Rng::seed_from_u64(self.config.seed.wrapping_add(1000));

        let names = [
            "Anna", "Bran", "Cara", "Dane", "Elise", "Finn", "Greta", "Hans", "Ines", "Jorik",
            "Kira", "Lars", "Mira", "Nils", "Opal", "Poul", "Quinn", "Rosa", "Sven", "Tova", "Ulf",
            "Vera", "Wulf", "Xena",
        ];

        let house_indices: Vec<usize> = self
            .world
            .sites
            .iter()
            .enumerate()
            .filter(|(_, s)| s.kind == crate::world::SiteKind::House)
            .map(|(i, _)| i)
            .collect();

        for i in 0..self.config.num_agents {
            let needs = NeedState::default();
            let personality = Personality::random(&mut populate_rng);
            let neuroticism = personality.neuroticism;
            let openness = personality.openness;
            let name = names[i as usize % names.len()].to_string();
            // Architecture-plan-2 §7.4: Generate rich biological substrate
            let agent_age = Fixed::from_f64(populate_rng.random_range(18.0..55.0));
            let embodied = EmbodiedState::random(agent_age, &mut populate_rng);
            // Extract genome values before embodied is moved into AgentBundle
            let attachment_vulnerability = embodied
                .genome
                .trait_predispositions
                .attachment_vulnerability;

            let home_site = if house_indices.is_empty() {
                None
            } else {
                Some(house_indices[i as usize % house_indices.len()])
            };

            if let Some(site_idx) = home_site {
                self.world.sites[site_idx].owner = Some(EntityId::new(i as u64));
            }

            // Foundational beliefs (props 0–2, including the
            // NeighborTrustworthy proposition the daily belief-update gate
            // targets — §8.1.18, Iteration 168 revives that structurally-dead
            // gate by seeding its target belief). Shared with newborns and
            // snapshot restores via `foundational_beliefs()` so every agent
            // participates in belief propagation identically.
            let beliefs = Self::foundational_beliefs();

            // §6: Initialize agent position from home site coordinates.
            let position = if let Some(site_idx) = home_site {
                self.world
                    .site_position(site_idx)
                    .map_or_else(|| Position::new(8, 8), |(x, y)| Position::new(x, y))
            // center fallback
            } else {
                Position::new(8, 8) // center of 16x16 world
            };

            let epistemic_state =
                crate::social::epistemic::EpistemicState::from_personality(&personality);
            let personality_clone = personality.clone();
            // The populate-RNG draws for identity strengths, moral values,
            // wealth, and the developmental (caregiver/trauma) pair are hoisted
            // HERE — before the literal, in their exact current stream order —
            // so the shared `trauma_history` can feed BOTH the interoception's
            // documented `trauma_load` input (§8.1, previously hardcoded ZERO
            // — deadening the ×0.3 trauma term in `negative_bias`) and the
            // attachment system, with the populate stream byte-identical.
            let identity_farmer = Fixed::from_f64(populate_rng.random_range(0.3..0.8));
            let identity_parent = Fixed::from_f64(populate_rng.random_range(0.0..0.5));
            let identity_believer = Fixed::from_f64(populate_rng.random_range(0.2..0.7));
            let moral_values = MoralValues::random(&mut populate_rng);
            let wealth_coin = Fixed::from_f64(populate_rng.random_range(5.0..20.0));
            let caregiver_security = Fixed::from_f64(populate_rng.random_range(0.3..0.8));
            let trauma_history = Fixed::from_f64(populate_rng.random_range(0.0..0.3));
            self.agents.push(AgentBundle {
                body: BodyState::from(&embodied),
                needs,
                personality,
                goals: Vec::new(),
                beliefs,
                affect: Affect::default(),
                emotions: DiscreteEmotions::default(),
                current_action: ActionKind::Idle,
                action_progress: 0,
                name,
                home_site,
                position,
                memory: MemoryStore::new(200),
                identity: IdentityState {
                    identities: vec![
                        crate::person::AgentIdentity {
                            kind: IdentityKind::Farmer,
                            strength: identity_farmer,
                        },
                        crate::person::AgentIdentity {
                            kind: IdentityKind::Parent,
                            strength: identity_parent,
                        },
                        crate::person::AgentIdentity {
                            kind: IdentityKind::Believer,
                            strength: identity_believer,
                        },
                    ],
                },
                attention: AttentionState::default(),
                intention: None,
                routine: DailyRoutine::village_routine(),
                moral_values,
                cognitive: CognitiveState::default(),
                derived: DerivedMentalState::default(),
                relational_fields: Default::default(),
                recent_successes: 0,
                recent_attempts: 0,
                age: agent_age,
                wealth: WealthState { coin: wealth_coin },
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
                embodied,
                interoception: {
                    let mut inter = crate::psychology::InteroceptiveState::default();
                    inter.initialize_from_personality(neuroticism, openness, trauma_history);
                    inter
                },
                self_model: crate::psychology::SelfModel::default(),
                attachment: {
                    let mut att = crate::psychology::AttachmentSystem::default();
                    att.initialize(caregiver_security, trauma_history, attachment_vulnerability);
                    att
                },
                emotion_regulation: {
                    let mut reg = crate::psychology::EmotionRegulationState::default();
                    // §8.1.8 (P3-4): personality-driven strategy preference
                    // — previously never assigned, so every agent was
                    // permanently Reappraisal (probe-pinned 12/12). Uses the
                    // clone (personality itself is moved into the bundle
                    // below).
                    reg.initialize_from_personality(&personality_clone);
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
                relationship_v2s: Vec::new(), // populated after agents are created
                speech_log: Vec::new(),
                attraction: crate::social::attraction::AttractionModel::default(),
                status_v2: crate::social::status_dims::StatusDimensions::default(),
                epistemic: epistemic_state,
                cognitive_runtime: crate::psychology::CognitiveRuntime::from_personality(
                    &personality_clone,
                ),
                motivation: crate::psychology::MotivationState::from_personality(
                    &personality_clone,
                ),
                neural_like: crate::psychology::neural_like::NeuralLikeState::default(),
                decision_policy: crate::psychology::DecisionPolicy::from_personality(
                    personality_clone.neuroticism,
                    personality_clone.extraversion,
                    personality_clone.conscientiousness,
                    personality_clone.openness,
                    personality_clone.agreeableness,
                    personality_clone.risk_tolerance,
                ),
                agent_tier: crate::agent_tier::AgentTierState::new(
                    crate::agent_tier::AgentTier::Secondary,
                    0,
                ),
                narrative_frames: crate::culture::narrative_frame::NarrativeFrameSet::default(),
                ideology: crate::culture::ideology::Ideology {
                    axes: vec![
                        crate::culture::ideology::IdeologyAxis {
                            name: "tradition".into(),
                            position: Fixed::from_f64(populate_rng.random_range(0.2..0.8)),
                            conviction: Fixed::from_f64(0.5),
                        },
                        crate::culture::ideology::IdeologyAxis {
                            name: "authority".into(),
                            position: Fixed::from_f64(populate_rng.random_range(0.2..0.8)),
                            conviction: Fixed::from_f64(0.5),
                        },
                    ],
                    dogmatism: Fixed::from_f64(populate_rng.random_range(0.2..0.7)),
                    echo_chamber_strength: Fixed::from_f64(0.3),
                    polarization_tendency: Fixed::from_f64(populate_rng.random_range(0.1..0.5)),
                },
                sacred_values: {
                    let mut sv = crate::culture::sacred::SacredValues::default();
                    sv.add_or_strengthen(
                        "family_honor".into(),
                        Fixed::from_f64(populate_rng.random_range(0.3..0.8)),
                        Fixed::from_f64(populate_rng.random_range(0.3..0.7)),
                    );
                    sv.add_or_strengthen(
                        "community_safety".into(),
                        Fixed::from_f64(populate_rng.random_range(0.3..0.7)),
                        Fixed::from_f64(populate_rng.random_range(0.2..0.6)),
                    );
                    sv
                },
                // §11.1: Perceived legitimacy starts at the mean-zero anchor.
                legitimacy_field: crate::noosphere::LegitimacyField::new(Fixed::from_f64(0.5)),
                education: crate::culture::education::EducationState {
                    teaching_skill: Fixed::from_f64(populate_rng.random_range(0.1..0.5)),
                    learning_aptitude: Fixed::from_f64(populate_rng.random_range(0.3..0.8)),
                    teaching_patience: Fixed::from_f64(0.5),
                    ..crate::culture::education::EducationState::default()
                },
                // AP3 DC-1 (task 3.1): placeholder neutral field; the founder
                // endowment is drawn in the post-loop pass below, at the very
                // end of the populate draw sequence (order contract).
                development: crate::psychology::DevelopmentFieldState::neutral(0),
                // DC-1 STORY 9-10: founders start with empty polarity_claims
                // (no catalysts encountered yet); `project_catalyst` fills in
                // the daily pass.
                polarity_claims: Vec::new(),
                // DC-2.7: lore archetype history, parallel to polarity_claims.
                lore_archetypes: Vec::new(),
            });
        }

        for i in 0..self.agents.len() {
            for j in 0..self.agents.len() {
                if i != j {
                    let trust = Fixed::from_f64(populate_rng.random_range(0.3..0.7));
                    let affection = Fixed::from_f64(populate_rng.random_range(0.2..0.6));
                    self.relationships.push(Relationship {
                        from: AgentId::new(i as u64),
                        to: AgentId::new(j as u64),
                        trust,
                        affection,
                        respect: Fixed::from_f64(0.3),
                        fear: Fixed::ZERO,
                        obligation: Fixed::ZERO,
                        last_interaction_tick: 0,
                        kind: RelationshipKind::Stranger,
                        interaction_count: 0,
                        last_positive_tick: 0,
                        last_negative_tick: 0,
                    });
                }
            }
        }

        // ── AP3 DC-1 (task 3.1): founder development-field endowment ──
        // Draw-order contract (AGENTS.md §5 RNG discipline): appended at the
        // VERY END of the populate sequence — after every draw above — one
        // U(0,1) per registered line in stable registry order, per agent in
        // index order. The local populate generator is dropped at function
        // exit (never shared with the tick loop), so no pre-existing draw
        // sequence moves and goldens stay byte-identical until first
        // consumption by the development pass (task 3.2).
        let line_count = mindstrata_development::line::all_lines().count();
        for agent in &mut self.agents {
            agent.development = crate::psychology::DevelopmentFieldState::founder_drawn(
                &mut populate_rng,
                line_count,
            );
        }

        // Architecture-plan-2 §10.2: Initialize RelationshipV2 for each directed pair.
        // Each agent gets a Vec<RelationshipV2> containing enriched relationships to all others.
        // Relationships are pushed in the same order as self.relationships, so we can use
        // index arithmetic for O(1) lookup instead of linear search.
        let n_agents = self.agents.len();
        for i in 0..n_agents {
            let mut v2s = Vec::with_capacity(n_agents.saturating_sub(1));
            for j in 0..n_agents {
                if i != j {
                    let mut rv2 =
                        RelationshipV2::new(AgentId::new(i as u64), AgentId::new(j as u64));
                    // O(1) lookup: relationships are pushed as (0,1),(0,2),...,(0,N-1),(1,0),(1,2),...
                    let rel_idx = Self::relationship_v2_index(i, j, n_agents);
                    if rel_idx < self.relationships.len() {
                        rv2.trust = self.relationships[rel_idx].trust;
                        rv2.affection = self.relationships[rel_idx].affection;
                    }
                    // Advance stage based on initial trust
                    if rv2.trust > Fixed::from_f64(0.5) {
                        rv2.stage = crate::social::relationship_v2::RelationshipStage::Acquaintance;
                    }
                    if rv2.trust > Fixed::from_f64(0.6) {
                        rv2.stage = crate::social::relationship_v2::RelationshipStage::Familiar;
                    }
                    v2s.push(rv2);
                }
            }
            self.agents[i].relationship_v2s = v2s;
        }

        // Initialize disease tracking
        self.agent_diseases = vec![Vec::new(); self.agents.len()];
        self.site_work_ticks = vec![0; self.world.tiles.len()];

        // Register default norms
        for norm in norms::default_norms() {
            self.norms.register(norm);
        }

        // §12: Create default institutions
        self.institutions = institutions::default_institutions();

        // §19.5.I: Seed cultural knowledge store with initial knowledge
        self.knowledge_store = vec![
            Knowledge {
                id: 0,
                name: "Crop Rotation".into(),
                category: KnowledgeCategory::Agricultural,
                difficulty: Fixed::from_f64(0.3),
                utility: Fixed::from_f64(0.8),
                holders: 0,
                discovered_tick: 0,
            },
            Knowledge {
                id: 1,
                name: "Well Maintenance".into(),
                category: KnowledgeCategory::Craft,
                difficulty: Fixed::from_f64(0.4),
                utility: Fixed::from_f64(0.6),
                holders: 0,
                discovered_tick: 0,
            },
            Knowledge {
                id: 2,
                name: "Herbal Medicine".into(),
                category: KnowledgeCategory::Medical,
                difficulty: Fixed::from_f64(0.7),
                utility: Fixed::from_f64(0.9),
                holders: 0,
                discovered_tick: 0,
            },
            Knowledge {
                id: 3,
                name: "Harvest Prayer".into(),
                category: KnowledgeCategory::Philosophical,
                difficulty: Fixed::from_f64(0.2),
                utility: Fixed::from_f64(0.4),
                holders: 0,
                discovered_tick: 0,
            },
            Knowledge {
                id: 4,
                name: "Grain Storage".into(),
                category: KnowledgeCategory::Agricultural,
                difficulty: Fixed::from_f64(0.3),
                utility: Fixed::from_f64(0.7),
                holders: 0,
                discovered_tick: 0,
            },
        ];
        // Seed agents with initial knowledge based on personality traits
        for agent in &mut self.agents {
            // All agents know Crop Rotation (id=0)
            agent.cultural.knowledge.push(0);
            // Traditional agents know Harvest Prayer (id=3)
            if agent.personality.traditionalism > Fixed::from_f64(0.5) {
                agent.cultural.knowledge.push(3);
            }
            // Conscientious agents know Well Maintenance (id=1) or Grain Storage (id=4)
            if agent.personality.conscientiousness > Fixed::from_f64(0.5) {
                agent.cultural.knowledge.push(1);
            }
            // Open agents know Herbal Medicine (id=2)
            if agent.personality.openness > Fixed::from_f64(0.5) {
                agent.cultural.knowledge.push(2);
            }
            if agent.personality.conscientiousness > Fixed::from_f64(0.6) {
                agent.cultural.knowledge.push(4);
            }
        }

        // §8.1.18 (Iteration 165): Seed the shared village taboo set into
        // every agent's cultural cognition, scaled by traditionalism. This is
        // the taboo layer's first production writer — before Iteration 165
        // every agent held an empty `taboos` vec and the §8.1.18 taboo
        // machinery (Taboo, violation_cost, tabo_violated_by,
        // change_resistance) was write-only dead code with zero consumers.
        // The set is deterministic (a pure function of traditionalism, no
        // RNG), so identical seeds produce identical taboo profiles. The
        // daily pass below then folds the strongest taboo
        // (`max_taboo_strength`) into `AttractionModel.taboo_penalty` (the
        // RWR row-#11 consumer). Iteration 167 wired `violation_cost` (via
        // `taboo_violation_cost_for`) into the violence-block shame boost;
        // Iteration 168 wired `change_resistance` into the daily
        // belief-update gate (the §8.1.18 sacred-boundary defense on the
        // belief layer — see block 13). `tabo_violated_by` remains
        // consumer-free (unit-tested only).
        for agent in &mut self.agents {
            agent
                .cultural_cognition
                .seed_village_taboos(agent.personality.traditionalism);
        }

        // §12: Assign agents to institutional roles based on personality traits
        // Elder: high conscientiousness + high agreeableness
        // Priest: high traditionalism + high agreeableness
        if self.agents.len() >= 3 {
            // Find best Elder candidate (highest conscientiousness + agreeableness)
            let elder_candidate = self
                .agents
                .iter()
                .enumerate()
                .max_by_key(|(_, a)| {
                    let score = a.personality.conscientiousness + a.personality.agreeableness;
                    score.to_raw() as u64
                })
                .map(|(i, _)| i);

            // Find best Priest candidate (highest traditionalism + agreeableness, not already elder)
            let priest_candidate = self
                .agents
                .iter()
                .enumerate()
                .filter(|(i, _)| Some(*i) != elder_candidate)
                .max_by_key(|(_, a)| {
                    let score = a.personality.traditionalism + a.personality.agreeableness;
                    score.to_raw() as u64
                })
                .map(|(i, _)| i);

            if let Some(elder_idx) = elder_candidate {
                if let Some(council) = self
                    .institutions
                    .iter_mut()
                    .find(|i| i.kind == institutions::InstitutionKind::Council)
                {
                    council.assign_role("Elder", AgentId::new(elder_idx as u64));
                    council.add_member(AgentId::new(elder_idx as u64));
                    // Add a second member (next highest score)
                    let second = self
                        .agents
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != elder_idx)
                        .max_by_key(|(_, a)| {
                            (a.personality.conscientiousness + a.personality.agreeableness).to_raw()
                                as u64
                        })
                        .map(|(i, _)| i);
                    if let Some(second_idx) = second {
                        council.add_member(AgentId::new(second_idx as u64));
                    }
                }
            }

            if let Some(priest_idx) = priest_candidate {
                if let Some(temple) = self
                    .institutions
                    .iter_mut()
                    .find(|i| i.kind == institutions::InstitutionKind::Temple)
                {
                    temple.assign_role("Priest", AgentId::new(priest_idx as u64));
                    temple.add_member(AgentId::new(priest_idx as u64));
                }
            }

            // §19.5.C: Assign Merchant to Market — high ambition + high dominance
            let merchant_candidate = self
                .agents
                .iter()
                .enumerate()
                .filter(|(i, _)| Some(*i) != elder_candidate && Some(*i) != priest_candidate)
                .max_by_key(|(_, a)| {
                    let score = a.personality.ambition + a.personality.dominance;
                    score.to_raw() as u64
                })
                .map(|(i, _)| i);
            if let Some(merchant_idx) = merchant_candidate {
                if let Some(market) = self
                    .institutions
                    .iter_mut()
                    .find(|i| i.kind == institutions::InstitutionKind::Market)
                {
                    market.assign_role("Merchant", AgentId::new(merchant_idx as u64));
                    market.add_member(AgentId::new(merchant_idx as u64));
                }
            }

            // §12: Assign Guard Captain to Council — high dominance + high conscientiousness
            let guard_candidate = self
                .agents
                .iter()
                .enumerate()
                .filter(|(i, _)| {
                    Some(*i) != elder_candidate
                        && Some(*i) != priest_candidate
                        && Some(*i) != merchant_candidate
                })
                .max_by_key(|(_, a)| {
                    let score = a.personality.dominance + a.personality.conscientiousness;
                    score.to_raw() as u64
                })
                .map(|(i, _)| i);
            if let Some(guard_idx) = guard_candidate {
                if let Some(council) = self
                    .institutions
                    .iter_mut()
                    .find(|i| i.kind == institutions::InstitutionKind::Council)
                {
                    council.assign_role("Guard Captain", AgentId::new(guard_idx as u64));
                    council.add_member(AgentId::new(guard_idx as u64));
                }
            }

            // §19.5.G: Set role_status for agents with institutional roles
            for (i, agent) in self.agents.iter_mut().enumerate() {
                let agent_id = AgentId::new(i as u64);
                let has_role = self
                    .institutions
                    .iter()
                    .any(|inst| inst.has_member(agent_id));
                if has_role {
                    agent.status.role_status = Fixed::from_f64(0.6);
                    agent.status.recompute();
                }
            }
        }

        // Architecture-plan-2 §10.6: Build kinship graph from parent/child data.
        for (i, agent) in self.agents.iter().enumerate() {
            if let Some(parent_a) = agent.parent_a {
                self.kinship_graph.add_link(
                    parent_a,
                    i,
                    crate::social::kinship::KinshipLink::ParentChild,
                    0,
                );
            }
            if let Some(parent_b) = agent.parent_b {
                self.kinship_graph.add_link(
                    parent_b,
                    i,
                    crate::social::kinship::KinshipLink::ParentChild,
                    0,
                );
            }
            // Siblings share at least one parent
            let parents: [Option<usize>; 2] = [agent.parent_a, agent.parent_b];
            for &parent in parents.iter().flatten() {
                for j in 0..self.agents.len() {
                    if j != i {
                        let shares_parent = self.agents[j].parent_a == Some(parent)
                            || self.agents[j].parent_b == Some(parent);
                        if shares_parent {
                            let already_linked = self.kinship_graph.edges.iter().any(|e| {
                                e.from == i
                                    && e.to == j
                                    && e.link == crate::social::kinship::KinshipLink::Sibling
                            });
                            if !already_linked {
                                self.kinship_graph.add_link(
                                    i,
                                    j,
                                    crate::social::kinship::KinshipLink::Sibling,
                                    0,
                                );
                            }
                        }
                    }
                }
            }
        }

        // Architecture-plan-2 §10.7: Create households from partner pairs + singles.
        let mut household_assigned = vec![false; self.agents.len()];
        for (i, agent) in self.agents.iter().enumerate() {
            if household_assigned[i] {
                continue;
            }
            if let Some(spouse_idx) = agent.partner {
                if spouse_idx < self.agents.len() && !household_assigned[spouse_idx] {
                    let mut h = crate::social::household::Household::new(i, agent.home_site, 0);
                    h.add_member(spouse_idx);
                    h.head = Some(i);
                    household_assigned[i] = true;
                    household_assigned[spouse_idx] = true;
                    self.households.push(h);
                }
            }
            if !household_assigned[i] {
                self.households
                    .push(crate::social::household::Household::new(
                        i,
                        agent.home_site,
                        0,
                    ));
                household_assigned[i] = true;
            }
        }

        tracing::info!(
            agents = self.agents.len(),
            sites = self.world.sites.len(),
            relationships = self.relationships.len(),
            norms = self.norms.norms().len(),
            institutions = self.institutions.len(),
            kinship_edges = self.kinship_graph.edges.len(),
            households = self.households.len(),
            "World populated"
        );

        // Seed recurring rituals and institutional propaganda campaigns.
        // Both registries previously started empty (constructed via `default()`
        // in `new()`), so the daily propaganda loop and the duodeca ritual loop
        // early-returned and neither system ever ran — same dead-end class as
        // the empty meme registry fixed in Iteration 2.
        self.seed_initial_rituals_and_campaigns();
        // §10.8/§13.5/§13: Seed the collective-structure registries (clans
        // from agent home-sites, the village's founding collective memory,
        // and noospheric concept nodes mirroring the founding memes) so
        // fresh runs match replayed runs from snapshots.
        self.seed_initial_clans();
        self.seed_initial_collective_memory();
        self.seed_initial_noosphere();

        // Iteration 218: Pre-allocate tick-scoped scratch buffers.
        // These were previously created and dropped every tick, causing
        // ~15.7M unnecessary heap allocations over a 100K run.
        let n = self.agents.len();
        self.tick_trust_deltas = (0..n).map(|_| Vec::new()).collect();
        self.tick_agent_ages = Vec::with_capacity(n);
        self.tick_rel_snapshot = Vec::with_capacity(self.relationships.len());
        self.tick_action_starts = Vec::with_capacity(n);
    }

    /// Seed the village's recurring rituals and founding propaganda campaigns.
    ///
    /// Mirrors `seed_initial_memes`: the registries are not serialized in
    /// snapshots, so both `populate()` (fresh runs) and `from_snapshot()`
    /// (replays) must re-seed identically to keep golden replays deterministic.
    /// Ritual participants are deterministic personality-based congregations
    /// (agent count AND personalities are identical at the same tick in fresh
    /// and replayed runs); campaign targets are all agents.
    ///
    /// - Rituals: a weekly Temple seasonal prayer and a monthly Council communal
    ///   meal. `synchrony` defaults to 0.5 in `Ritual::new`, feeding the §11.3
    ///   hierarchy-stabilization term (ritual participation builds legitimacy)
    ///   and the §12.5 duodeca bonding loop (trust/affection between
    ///   participants). Bonding is small enough that trust mean-reversion
    ///   (Iteration 3) still differentiates relationships.
    /// - Campaigns: a Council edict and a Temple sermon, each targeting all
    ///   agents. Durations are in *days* (tick_all runs on the daily phase):
    ///   360 ≈ one year, so the §13.4 loop is observable for a full-year run
    ///   and expires afterward (institutions would relaunch; out of scope).
    pub fn seed_initial_rituals_and_campaigns(&mut self) {
        use crate::culture::{PropagandaCampaign, PropagandaChannel, Ritual, RitualKind};
        let n = self.agents.len();
        // Campaign targets = everyone; propaganda touches affect only and
        // never writes trust/ties, so it cannot disturb the §13.6 metrics.
        let all: Vec<usize> = (0..n).collect();
        // Institution indices from institutions::default_institutions(): 0 =
        // Council, 1 = Temple, 2 = Market.
        //
        // Bonding calibration: the §12.5 duodeca loop applies `bonding × 0.3`
        // to trust for EVERY participant pair on every fire, and rv2 decay is
        // only 0.0005/tick — so a weekly all-pairs ritual with bonding ~0.45
        // saturated avg trust to 1.0 in 20K ticks, erasing the relationship
        // differentiation Iter-3's mean reversion exists to preserve. These
        // seeds use low intensity/sacredness (bonding ~0.1-0.15) and monthly
        // cadence so cohesion rises measurably (~0.2 over 20K ticks) but trust
        // stays differentiated.
        //
        // Congregations, not the whole village: all-pairs bonding ALSO pushed
        // every cross-cluster rv2 trust above the §13.6 cross-cutting-tie
        // threshold (0.5), collapsing tie_ratio to 1.0 — echo_strength halved
        // (daily belief reinforcement stopped outpacing confidence decay:
        // `echo_chambers_reinforce_agent_beliefs` failed) and polarization
        // pinned at 0.0000 (`polarization_index_emerges_from_gossip_fed_beliefs`
        // failed). Rituals now bond WITHIN the §13.6 belief clusters
        // (pro-institution vs anti-institution, split on traditionalism +
        // agreeableness > 1.0), deepening in-group trust while cross-cutting
        // ties stay at baseline:
        // - Temple SeasonalPrayer: the institutional faithful (pro cluster).
        // - Council CommunalMeal: the disaffected (anti cluster) — the Council
        //   feeds its opposition to pacify it (bread and circuses), bonding
        //   the future faction pool into a civic in-group.
        let pro: Vec<usize> = (0..n)
            .filter(|&i| {
                self.agents[i].personality.traditionalism + self.agents[i].personality.agreeableness
                    > Fixed::from_f64(1.0)
            })
            .collect();
        let anti: Vec<usize> = (0..n).filter(|&i| !pro.contains(&i)).collect();
        let mut seasonal = Ritual::new(
            0,
            RitualKind::SeasonalPrayer,
            "Seasonal Prayer".into(),
            1,                     // Temple sponsor
            Fixed::from_f64(0.2),  // emotional intensity
            Fixed::from_f64(0.25), // sacredness
            Fixed::from_f64(0.05), // cost
            4320,                  // monthly interval
            0,                     // last_occurrence — deterministic across replays
        );
        seasonal.participants = pro;
        self.ritual_registry.register(seasonal);

        let mut meal = Ritual::new(
            0,
            RitualKind::CommunalMeal,
            "Communal Meal".into(),
            0,                     // Council sponsor
            Fixed::from_f64(0.12), // emotional intensity
            Fixed::from_f64(0.15), // sacredness
            Fixed::from_f64(0.1),  // cost
            4320,                  // monthly interval
            0,
        );
        meal.participants = anti;
        self.ritual_registry.register(meal);

        // Council edict — reassures the village (raises valence when legitimate).
        self.propaganda_registry.register(PropagandaCampaign::new(
            0,
            0, // Council sponsor
            all.clone(),
            "The council protects the well".into(),
            Fixed::from_f64(0.5), // intensity
            vec![PropagandaChannel::Edict],
            360, // ~1 year (daily tick)
            0,
        ));
        // Temple sermon — reinforces piety (raises valence when legitimate).
        self.propaganda_registry.register(PropagandaCampaign::new(
            0,
            1, // Temple sponsor
            all,
            "Honor the spirits for rain".into(),
            Fixed::from_f64(0.4), // intensity
            vec![PropagandaChannel::Sermon],
            180, // ~half year (daily tick)
            0,
        ));
    }

    /// §10.8: Seed the village's founding clans from agent home-sites.
    ///
    /// Households are not serialized in snapshots, so this cannot derive
    /// from `self.households` — it partitions agents by `home_site` parity
    /// instead. `home_site` IS serialized and identical at the same tick in
    /// fresh and replayed runs, so `populate()` and `from_snapshot()`
    /// produce identical registries (same pattern as `seed_initial_memes`).
    /// Clans are currently observational: `clan_registry.daily_update()`
    /// decays prestige/cohesion/grievance and `clan_count` is exported.
    fn seed_initial_clans(&mut self) {
        use crate::social::clan::Clan;
        if !self.clan_registry.clans.is_empty() {
            return;
        }
        // Iteration 256 (audit Phase 4): clan count DERIVES FROM
        // SETTLEMENT CLUSTERING instead of a hardcoded 2 - occupied home
        // sites are grouped into contiguous runs of at most
        // MAX_HOUSEHOLDS_PER_CLAN, and every household inherits its
        // run's clan. Households therefore NEVER straddle clan boundaries
        // (the first cut used `site % count`, which scattered co-residents
        // into rival clans and starved intra-clan belief reinforcement -
        // probe: pestilence s99 belief charges collapsed max 0.87 -> 0.23,
        // zero panics). Village scale still drives the count: 3 households
        // -> 1 clan, 7 -> 3 clans.
        const MAX_HOUSEHOLDS_PER_CLAN: usize = 3;
        let mut occupied_sites: Vec<usize> =
            self.agents.iter().filter_map(|a| a.home_site).collect();
        occupied_sites.sort_unstable();
        occupied_sites.dedup();
        let clan_count = occupied_sites
            .len()
            .div_ceil(MAX_HOUSEHOLDS_PER_CLAN)
            .max(1);
        let mut site_to_bucket: std::collections::HashMap<usize, usize> =
            std::collections::HashMap::new();
        for (run_idx, site) in occupied_sites.iter().enumerate() {
            site_to_bucket.insert(*site, run_idx / MAX_HOUSEHOLDS_PER_CLAN);
        }
        let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); clan_count];
        for (i, agent) in self.agents.iter().enumerate() {
            let bucket = agent
                .home_site
                .and_then(|hs| site_to_bucket.get(&hs).copied())
                .unwrap_or(i % clan_count);
            buckets[bucket].push(i);
        }
        let meme_count = self.meme_registry.memes.len();
        for members in buckets.into_iter().filter(|m| !m.is_empty()) {
            let clan_id = self.clan_registry.clans.len();
            let mut clan = Clan::new(
                clan_id,
                Some(members[0]), // founder — first member in agent order
                None,
                members,
                0, // formed_tick — identical across new()/from_snapshot()
            );
            // §10.8: Founding identity — each clan adopts a founding myth
            // drawn deterministically from the village meme pool (memes are
            // re-seeded before clans in both populate() and from_snapshot(),
            // so the same meme id is chosen in fresh and replayed runs) plus
            // an honor code derived from it. This makes the plan's "myths
            // justify identity / elders preserve honor codes" layer live from
            // formation instead of leaving `myths`/`honor_codes` empty.
            if meme_count > 0 {
                if let Some(m) = self.meme_registry.memes.get(clan_id % meme_count) {
                    clan.add_myth(
                        format!("Founding myth: {}", m.description),
                        m.credibility,
                        m.emotional_charge,
                        matches!(
                            m.content_type,
                            crate::culture::MemeContent::Accusation
                                | crate::culture::MemeContent::Conspiracy
                        ),
                    );
                    clan.add_honor_code(
                        format!("Honor the founding principle: {}", m.description),
                        (Fixed::from_f64(0.4) + m.identity_relevance * Fixed::from_f64(0.3))
                            .clamp_01(),
                        (m.emotional_charge * Fixed::from_f64(0.25)).clamp_01(),
                    );
                }
            }
            self.clan_registry.register(clan);
        }
    }

    /// §13.5: Seed the village's founding collective memory (group 0).
    ///
    /// Deterministic (hardcoded narratives, created at tick 0) so fresh and
    /// replayed runs match. `collective_memory_registry.tick_all` already
    /// runs on the daily phase, decaying salience/cohesion — the memory
    /// system now has content to maintain.
    fn seed_initial_collective_memory(&mut self) {
        use crate::culture::SharedMemoryKind;
        if self
            .collective_memory_registry
            .entries
            .iter()
            .any(|e| e.group_id == 0 && !e.memories.is_empty())
        {
            return;
        }
        let village = self.collective_memory_registry.get_or_create(0); // 0 = the village
        if village.memories.is_empty() {
            // Iteration 256 (audit Phase 4): the founding myth derives from
            // ACTUAL world generation — river-followers vs well-diggers —
            // and traumas are no longer pre-seeded: they must be earned by
            // events (the drought trauma here was a lie about a history
            // that never happened).
            let water_stress = self.scenario.as_ref().is_some_and(|sc| {
                sc.shocks
                    .iter()
                    .any(|sh| matches!(sh.kind, ShockKind::Drought | ShockKind::Famine))
            });
            let myth = if water_stress {
                "The founders dug their wells where the old ones said water hides".to_string()
            } else {
                "The village was founded by the first settlers who followed the river".to_string()
            };
            village.add_memory(myth, SharedMemoryKind::Founding, 0, Fixed::from_f64(0.8));
        }
    }

    /// §13: Seed the noospheric field with symbolic nodes mirroring the
    /// founding memes, plus associative edges between consecutive concepts.
    ///
    /// Deterministic (mirrors the same hardcoded meme list as
    /// `seed_initial_memes`, at tick 0) so fresh and replayed runs match.
    /// Without this, `tick_noosphere` ran spreading/decay on an empty field.
    fn seed_initial_noosphere(&mut self) {
        use crate::culture::MemeContent;
        use crate::noosphere::field::{ConceptVector, SymbolicEdge, SymbolicNode};
        if !self.noospheric_field.nodes.is_empty() {
            return;
        }
        for meme in &self.meme_registry.memes {
            let mut concept = ConceptVector::default();
            match meme.content_type {
                MemeContent::Theological => {
                    concept.sacredness = meme.sacredness.max(Fixed::from_f64(0.5));
                    concept.hope = meme.emotional_charge;
                }
                MemeContent::Moral => {
                    concept.loyalty = meme.emotional_charge;
                    concept.purity = meme.identity_relevance;
                }
                MemeContent::Prophecy => {
                    concept.threat = meme.emotional_charge;
                    concept.hope = meme.identity_relevance;
                }
                MemeContent::Political => {
                    concept.status = meme.emotional_charge;
                    concept.freedom = meme.identity_relevance;
                }
                _ => {
                    concept.scarcity = meme.emotional_charge;
                }
            }
            let mut node = SymbolicNode::new(meme.id as u32, concept);
            node.emotional_charge = meme.emotional_charge;
            node.identity_relevance = meme.identity_relevance;
            node.sacredness = meme.sacredness;
            node.institutional_backing = meme.credibility;
            self.noospheric_field.add_node(node);
        }
        // Associative chain between consecutive founding concepts so
        // spreading activation propagates between them.
        let ids: Vec<u32> = self.noospheric_field.nodes.iter().map(|n| n.id).collect();
        for w in ids.windows(2) {
            self.noospheric_field.add_edge(SymbolicEdge {
                from: w[0],
                to: w[1],
                weight: Fixed::from_f64(0.4),
                decay: Fixed::from_f64(0.2),
            });
            self.noospheric_field.add_edge(SymbolicEdge {
                from: w[1],
                to: w[0],
                weight: Fixed::from_f64(0.4),
                decay: Fixed::from_f64(0.2),
            });
        }
    }
}
