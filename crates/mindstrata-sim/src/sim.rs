//! Simulation orchestrator — the fixed-tick loop.

use crate::actions::{self, ActionKind};
use crate::conflict::{self, ConflictKind, ConflictState};
use crate::cultural::{CulturalState, Knowledge, KnowledgeCategory};
use crate::logistics;
use crate::gossip;
use crate::market::{self, MarketState, WealthState};
use crate::appraisal::{self, Agency, Appraisal};
use crate::belief_update;
use crate::journal::{EventJournal, JournalEntryKind};
use crate::memory::{MemoryKind, MemoryStore, MemoryTag};
use crate::norms::{self, NormRegistry};
use crate::attention::AttentionState;
use crate::demography;
use crate::ecology;
use crate::factions;
use crate::health;
use crate::institutions::{self, Institution, InstitutionKind};
use crate::routines::DailyRoutine;
pub use crate::attention;
pub use crate::person::Intention;
use crate::person::{Affect, BodyState, Belief, CognitiveState, DerivedMentalState, DiscreteEmotions, Goal, GoalKind, GoalSource, IdentityKind, IdentityState, MoralValues, NeedState, Personality, Relationship, RelationshipKind, StatusState};
use crate::biology::EmbodiedState;
use crate::psychology::InteroceptiveState;
use crate::psychology::SelfModel;
use crate::psychology::AttachmentSystem;
use crate::psychology::EmotionRegulationState;
use crate::psychology::MoralCognition;
use crate::psychology::ProspectionState;
use crate::psychology::NarrativeIdentity;
use crate::psychology::DevelopmentalPsychState;
use crate::psychology::PsychopathologyState;
use crate::psychology::SkillState as PsychSkillState;
use crate::relationship_v2::RelationshipV2;
use crate::attraction::AttractionModel;
use crate::status_dims::StatusDimensions;
use crate::provenance::{CausalProvenance, DecisionFactor, DecisionTrace};
use crate::scenario::{Scenario, ShockKind};
use crate::snapshot::Snapshot;
use crate::social;
use crate::systems::{self, SystemContext};
use crate::world::{GRAIN_RESOURCE_ID, WATER_RESOURCE_ID, World};
use crate::world_gen;
use mindstrata_core::clock::{Clock, Tick};
use mindstrata_core::event::SimEvent;
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::{AgentId, EntityId, ResourceId};
use mindstrata_core::rng::{RngStream, RngStreams};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

// ── Named constants for magic numbers ──────────────────────────────
/// Expected grain capacity per agent — used for scarcity normalization.
pub const EXPECTED_GRAIN_PER_AGENT: u32 = 10;
/// Skill improvement per tick of practice.
pub const SKILL_GAIN_PER_TICK: Fixed = Fixed::from_raw(10); // 0.001
/// Belief resistance decay rate per tick.
pub const BELIEF_RESISTANCE_DECAY: Fixed = Fixed::from_raw(10); // 0.001

/// Configuration for a simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    pub seed: u64,
    pub max_ticks: u64,
    pub world_width: u32,
    pub world_height: u32,
    pub num_agents: u32,
    pub snapshot_interval: Option<u64>,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            max_ticks: 1000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: Some(100),
        }
    }
}

/// §6: Agent's spatial position in the world grid.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Manhattan distance to another position.
    pub fn manhattan_distance(&self, other: &Position) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

/// An agent's full state bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBundle {
    pub body: BodyState,
    pub needs: NeedState,
    pub personality: Personality,
    pub goals: Vec<Goal>,
    pub beliefs: Vec<Belief>,
    pub affect: Affect,
    pub emotions: DiscreteEmotions,
    pub current_action: ActionKind,
    pub action_progress: u32,
    pub name: String,
    pub home_site: Option<usize>,
    /// §6: Agent's spatial position in the world grid.
    pub position: Position,
    pub memory: MemoryStore,
    pub identity: IdentityState,
    /// §22.5: Attention controls which events the agent notices.
    pub attention: AttentionState,
    /// §24.5: Current intention — a committed plan to achieve a goal.
    pub intention: Option<crate::person::Intention>,
    /// §10.3: Daily routine creating behavioral stability.
    pub routine: DailyRoutine,
    /// §22.1: Moral foundations affecting norm compliance and emotional appraisal.
    pub moral_values: MoralValues,
    /// §22.1: Bounded rationality — stress, fatigue reduce planning and increase heuristic reliance.
    pub cognitive: CognitiveState,
    /// §22: Derived mental states — trauma, depression, ambition, resilience, resentment.
    pub derived: DerivedMentalState,
    /// Track success rate for derived mental state computation.
    pub recent_successes: u32,
    pub recent_attempts: u32,
    /// §31: Agent age in years.
    pub age: Fixed,
    /// §13.3: Per-agent wealth — coin and economic state.
    pub wealth: WealthState,
    /// §19.5.H: Per-agent conflict state — trauma, injuries, combat fatigue.
    pub conflict: ConflictState,
    /// §19.5.I: Per-agent cultural state — practices, knowledge, ideology.
    pub cultural: CulturalState,
    /// §19.5.F: Partner index (if married/partnered).
    pub partner: Option<usize>,
    /// §19.5.F: Parent indices (for children).
    pub parent_a: Option<usize>,
    pub parent_b: Option<usize>,
    /// §19.5.G: Active feuds — set of agent indices this agent is feuding with.
    pub feuds: Vec<usize>,
    /// §19.5.G: Tick when each feud started (for decay).
    pub feud_ticks: Vec<u64>,
    /// §19.5.G: Per-agent status — derived from wealth, role, social connections.
    pub status: StatusState,
    /// §3.2: Goals abandoned due to critical need override or timeout.
    pub rejected_goals: Vec<Goal>,
    /// §3.2: Goals successfully completed.
    pub completed_goals: Vec<Goal>,
    /// §4.2: Per-agent skill levels — improve with repeated actions.
    pub skills: AgentSkills,
    /// Architecture-plan-2 §7.4: Rich biological substrate.
    /// Provides genome, endocrine axes, nervous system alongside legacy BodyState.
    pub embodied: EmbodiedState,
    /// Architecture-plan-2 §8.1.1: Translates body signals into felt experience.
    pub interoception: InteroceptiveState,
    /// Architecture-plan-2 §8.1.7: Internal self-model with identity, roles, values.
    pub self_model: SelfModel,
    /// Architecture-plan-2 §8.1.14: Attachment style and security.
    pub attachment: AttachmentSystem,
    /// Architecture-plan-2 §8.1.4: Emotion regulation strategies.
    pub emotion_regulation: EmotionRegulationState,
    /// Architecture-plan-2 §8.1.10: Moral foundations and norm internalization.
    pub moral_cognition: MoralCognition,
    /// Architecture-plan-2 §8.1.16: Prospection and mental simulation.
    pub prospection: ProspectionState,
    /// Architecture-plan-2 §8.1.17: Life narrative and meaning-making.
    pub narrative: NarrativeIdentity,
    /// Architecture-plan-2 §8.1.13: Developmental psychology.
    pub developmental: DevelopmentalPsychState,
    /// Architecture-plan-2 §8.1.15: Mental health risk dynamics.
    pub psychopathology: PsychopathologyState,
    /// Architecture-plan-2 §8.1.19: Skills and habits.
    pub psych_skills: PsychSkillState,
    /// Architecture-plan-2 §10.2: Enriched relationship model.
    pub relationship_v2s: Vec<RelationshipV2>,
    /// Architecture-plan-2 §10.4: Attraction model for potential partners.
    pub attraction: AttractionModel,
    /// Architecture-plan-2 §11.1: Multi-dimensional status.
    pub status_v2: StatusDimensions,
}

/// §4.2: Skill levels that improve through repeated practice.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentSkills {
    /// Farming skill — increases productivity when working.
    pub farming: Fixed,
    /// Trading skill — improves trade prices and success rate.
    pub trading: Fixed,
    /// Social skill — improves relationship gains from interactions.
    pub social: Fixed,
}

/// The simulation state.
pub struct Simulation {
    config: SimConfig,
    clock: Clock,
    rng: RngStreams,
    pub world: World,
    pub agents: Vec<AgentBundle>,
    pub relationships: Vec<Relationship>,
    events: Vec<SimEvent>,
    journal: EventJournal,
    norms: NormRegistry,
    /// §12: Institutions are first-class entities with legitimacy, roles, and collective psychology.
    pub institutions: Vec<Institution>,
    /// §34: Causal provenance — tracks decision traces and event causality chains.
    provenance: CausalProvenance,
    scenario: Option<Scenario>,
    /// §28: Season tracker for ecology.
    pub season: ecology::SeasonTracker,
    /// §28: Ecology configuration.
    pub ecology_config: ecology::EcologyConfig,
    /// §9: Health configuration.
    pub health_config: health::HealthConfig,
    /// §31: Demography configuration.
    pub demography_config: demography::DemographyConfig,
    /// §32: Active diseases per agent (parallel to agents vec).
    pub agent_diseases: Vec<Vec<health::ActiveDisease>>,
    /// Track work ticks per site for ecology (overfarming).
    pub site_work_ticks: Vec<u32>,
    /// §13.3: Market state — prices, inequality, trade volume.
    pub market: MarketState,
    /// §19.5.I: Cultural knowledge store — all knowledge in the world.
    pub knowledge_store: Vec<Knowledge>,
    /// §6.5: Per-tick metric history for observability and CSV export.
    pub metric_history: Vec<MetricsSnapshot>,
    /// §7.3: Tick when last revolution occurred (for cooldown tracking).
    last_revolution_tick: u64,
    /// §4.4: Black market state — activates under scarcity with weak enforcement.
    pub black_market: crate::black_market::BlackMarketState,
}

impl Simulation {
    /// Create a new simulation from config.
    pub fn new(config: SimConfig) -> Self {
        let rng = RngStreams::new(config.seed);
        let world = World::new(config.world_width, config.world_height);

        Self {
            config,
            clock: Clock::new(),
            rng,
            world,
            agents: Vec::new(),
            relationships: Vec::new(),
            events: Vec::new(),
            journal: EventJournal::new(),
            norms: NormRegistry::new(),
            institutions: Vec::new(),
            provenance: CausalProvenance::new(),
            scenario: None,
            season: ecology::SeasonTracker::new(8760),
            ecology_config: ecology::EcologyConfig::default(),
            health_config: health::HealthConfig::default(),
            demography_config: demography::DemographyConfig::default(),
            agent_diseases: Vec::new(),
            site_work_ticks: Vec::new(),
            market: MarketState::new(),
            knowledge_store: Vec::new(),
            metric_history: Vec::new(),
            last_revolution_tick: 0,
            black_market: crate::black_market::BlackMarketState::default(),
        }
    }

    /// Create from a scenario definition.
    pub fn from_scenario(scenario: Scenario) -> Self {
        let config = scenario.to_sim_config();
        let mut sim = Self::new(config);
        sim.scenario = Some(scenario);
        sim
    }

    /// §16.1: Restore from a snapshot (deterministic replay).
    pub fn from_snapshot(snapshot: crate::snapshot::Snapshot) -> Self {
        let mut clock = Clock::new();
        clock.set(snapshot.clock_tick);
        let rng = RngStreams::new(snapshot.master_seed);
        Self {
            config: snapshot.config,
            clock,
            rng,
            world: snapshot.world,
            agents: snapshot.agents,
            relationships: snapshot.relationships,
            events: snapshot.events,
            journal: snapshot.journal,
            norms: snapshot.norms,
            institutions: snapshot.institutions,
            provenance: snapshot.provenance,
            scenario: None,
            season: snapshot.season,
            ecology_config: snapshot.ecology_config,
            health_config: snapshot.health_config,
            demography_config: snapshot.demography_config,
            agent_diseases: snapshot.agent_diseases,
            site_work_ticks: snapshot.site_work_ticks,
            market: snapshot.market,
            knowledge_store: snapshot.knowledge_store,
            metric_history: snapshot.metric_history,
            last_revolution_tick: snapshot.last_revolution_tick,
            black_market: snapshot.black_market,
        }
    }

    /// §16.1: Capture a snapshot of the current simulation state.
    pub fn capture_snapshot(&self) -> crate::snapshot::Snapshot {
        crate::snapshot::Snapshot::capture(&crate::snapshot::CaptureContext {
            config: &self.config,
            tick: self.clock.tick(),
            master_seed: self.config.seed,
            world: &self.world,
            agents: &self.agents,
            relationships: &self.relationships,
            events: &self.events,
            journal: &self.journal,
            norms: &self.norms,
            institutions: &self.institutions,
            provenance: &self.provenance,
            season: &self.season,
            ecology_config: &self.ecology_config,
            health_config: &self.health_config,
            demography_config: &self.demography_config,
            agent_diseases: &self.agent_diseases,
            market: &self.market,
            knowledge_store: &self.knowledge_store,
            metric_history: &self.metric_history,
            last_revolution_tick: self.last_revolution_tick,
            black_market: &self.black_market,
            site_work_ticks: &self.site_work_ticks,
        })
    }

    /// Populate the world with terrain, sites, and agents.
    pub fn populate(&mut self) {
        world_gen::generate_village(&mut self.world, &mut self.rng);

        let mut populate_rng =
            rand_chacha::ChaCha8Rng::seed_from_u64(self.config.seed.wrapping_add(1000));

        let names = [
            "Anna", "Bran", "Cara", "Dane", "Elise", "Finn", "Greta", "Hans",
            "Ines", "Jorik", "Kira", "Lars", "Mira", "Nils", "Opal", "Poul",
            "Quinn", "Rosa", "Sven", "Tova", "Ulf", "Vera", "Wulf", "Xena",
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
            let attachment_vulnerability = embodied.genome.trait_predispositions.attachment_vulnerability;

            let home_site = if house_indices.is_empty() {
                None
            } else {
                Some(house_indices[i as usize % house_indices.len()])
            };

            if let Some(site_idx) = home_site {
                self.world.sites[site_idx].owner =
                    Some(EntityId::new(i as u64));
            }

            let beliefs = vec![
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
            ];

            // §6: Initialize agent position from home site coordinates.
            let position = if let Some(site_idx) = home_site {
                self.world.site_position(site_idx).map_or_else(|| Position::new(8, 8), |(x, y)| Position::new(x, y)) // center fallback
            } else {
                Position::new(8, 8) // center of 16x16 world
            };

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
                            strength: Fixed::from_f64(populate_rng.random_range(0.3..0.8)),
                        },
                        crate::person::AgentIdentity {
                            kind: IdentityKind::Parent,
                            strength: Fixed::from_f64(populate_rng.random_range(0.0..0.5)),
                        },
                        crate::person::AgentIdentity {
                            kind: IdentityKind::Believer,
                            strength: Fixed::from_f64(populate_rng.random_range(0.2..0.7)),
                        },
                    ],
                },
                attention: AttentionState::default(),
                intention: None,
                routine: DailyRoutine::village_routine(),
                moral_values: MoralValues::random(&mut populate_rng),
                cognitive: CognitiveState::default(),
                derived: DerivedMentalState::default(),
                recent_successes: 0,
                recent_attempts: 0,
                age: agent_age,
                wealth: WealthState {
                    coin: Fixed::from_f64(populate_rng.random_range(5.0..20.0)),
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
                embodied,
                interoception: {
                    let mut inter = crate::psychology::InteroceptiveState::default();
                    inter.initialize_from_personality(neuroticism, openness, Fixed::ZERO);
                    inter
                },
                self_model: crate::psychology::SelfModel::default(),
                attachment: {
                    let mut att = crate::psychology::AttachmentSystem::default();
                    att.initialize(
                        Fixed::from_f64(populate_rng.random_range(0.3..0.8)), // caregiver security
                        Fixed::from_f64(populate_rng.random_range(0.0..0.3)), // trauma history
                        attachment_vulnerability,
                    );
                    att
                },
                emotion_regulation: crate::psychology::EmotionRegulationState::default(),
                moral_cognition: crate::psychology::MoralCognition::default(),
                prospection: crate::psychology::ProspectionState::default(),
                narrative: crate::psychology::NarrativeIdentity::default(),
                developmental: crate::psychology::DevelopmentalPsychState::default(),
                psychopathology: crate::psychology::PsychopathologyState::default(),
                psych_skills: crate::psychology::SkillState::default(),
                relationship_v2s: Vec::new(), // populated after agents are created
                attraction: crate::attraction::AttractionModel::default(),
                status_v2: crate::status_dims::StatusDimensions::default(),
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

        // Architecture-plan-2 §10.2: Initialize RelationshipV2 for each directed pair.
        // Each agent gets a Vec<RelationshipV2> containing enriched relationships to all others.
        // Relationships are pushed in the same order as self.relationships, so we can use
        // index arithmetic for O(1) lookup instead of linear search.
        let n_agents = self.agents.len();
        for i in 0..n_agents {
            let mut v2s = Vec::with_capacity(n_agents.saturating_sub(1));
            for j in 0..n_agents {
                if i != j {
                    let mut rv2 = RelationshipV2::new(AgentId::new(i as u64), AgentId::new(j as u64));
                    // O(1) lookup: relationships are pushed as (0,1),(0,2),...,(0,N-1),(1,0),(1,2),...
                    let rel_idx = i * (n_agents - 1) + if j > i { j - 1 } else { j };
                    if rel_idx < self.relationships.len() {
                        rv2.trust = self.relationships[rel_idx].trust;
                        rv2.affection = self.relationships[rel_idx].affection;
                    }
                    // Advance stage based on initial trust
                    if rv2.trust > Fixed::from_f64(0.5) {
                        rv2.stage = crate::relationship_v2::RelationshipStage::Acquaintance;
                    }
                    if rv2.trust > Fixed::from_f64(0.6) {
                        rv2.stage = crate::relationship_v2::RelationshipStage::Familiar;
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
            Knowledge { id: 0, name: "Crop Rotation".into(), category: KnowledgeCategory::Agricultural, difficulty: Fixed::from_f64(0.3), utility: Fixed::from_f64(0.8), holders: 0, discovered_tick: 0 },
            Knowledge { id: 1, name: "Well Maintenance".into(), category: KnowledgeCategory::Craft, difficulty: Fixed::from_f64(0.4), utility: Fixed::from_f64(0.6), holders: 0, discovered_tick: 0 },
            Knowledge { id: 2, name: "Herbal Medicine".into(), category: KnowledgeCategory::Medical, difficulty: Fixed::from_f64(0.7), utility: Fixed::from_f64(0.9), holders: 0, discovered_tick: 0 },
            Knowledge { id: 3, name: "Harvest Prayer".into(), category: KnowledgeCategory::Philosophical, difficulty: Fixed::from_f64(0.2), utility: Fixed::from_f64(0.4), holders: 0, discovered_tick: 0 },
            Knowledge { id: 4, name: "Grain Storage".into(), category: KnowledgeCategory::Agricultural, difficulty: Fixed::from_f64(0.3), utility: Fixed::from_f64(0.7), holders: 0, discovered_tick: 0 },
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

        // §12: Assign agents to institutional roles based on personality traits
        // Elder: high conscientiousness + high agreeableness
        // Priest: high traditionalism + high agreeableness
        if self.agents.len() >= 3 {
            // Find best Elder candidate (highest conscientiousness + agreeableness)
            let elder_candidate = self.agents.iter().enumerate()
                .max_by_key(|(_, a)| {
                    let score = a.personality.conscientiousness + a.personality.agreeableness;
                    score.to_raw() as u64
                })
                .map(|(i, _)| i);

            // Find best Priest candidate (highest traditionalism + agreeableness, not already elder)
            let priest_candidate = self.agents.iter().enumerate()
                .filter(|(i, _)| Some(*i) != elder_candidate)
                .max_by_key(|(_, a)| {
                    let score = a.personality.traditionalism + a.personality.agreeableness;
                    score.to_raw() as u64
                })
                .map(|(i, _)| i);

            if let Some(elder_idx) = elder_candidate {
                if let Some(council) = self.institutions.iter_mut().find(|i| i.kind == institutions::InstitutionKind::Council) {
                    council.assign_role("Elder", AgentId::new(elder_idx as u64));
                    council.add_member(AgentId::new(elder_idx as u64));
                    // Add a second member (next highest score)
                    let second = self.agents.iter().enumerate()
                        .filter(|(i, _)| *i != elder_idx)
                        .max_by_key(|(_, a)| (a.personality.conscientiousness + a.personality.agreeableness).to_raw() as u64)
                        .map(|(i, _)| i);
                    if let Some(second_idx) = second {
                        council.add_member(AgentId::new(second_idx as u64));
                    }
                }
            }

            if let Some(priest_idx) = priest_candidate {
                if let Some(temple) = self.institutions.iter_mut().find(|i| i.kind == institutions::InstitutionKind::Temple) {
                    temple.assign_role("Priest", AgentId::new(priest_idx as u64));
                    temple.add_member(AgentId::new(priest_idx as u64));
                }
            }

            // §19.5.C: Assign Merchant to Market — high ambition + high dominance
            let merchant_candidate = self.agents.iter().enumerate()
                .filter(|(i, _)| Some(*i) != elder_candidate && Some(*i) != priest_candidate)
                .max_by_key(|(_, a)| {
                    let score = a.personality.ambition + a.personality.dominance;
                    score.to_raw() as u64
                })
                .map(|(i, _)| i);
            if let Some(merchant_idx) = merchant_candidate {
                if let Some(market) = self.institutions.iter_mut().find(|i| i.kind == institutions::InstitutionKind::Market) {
                    market.assign_role("Merchant", AgentId::new(merchant_idx as u64));
                    market.add_member(AgentId::new(merchant_idx as u64));
                }
            }

            // §12: Assign Guard Captain to Council — high dominance + high conscientiousness
            let guard_candidate = self.agents.iter().enumerate()
                .filter(|(i, _)| Some(*i) != elder_candidate && Some(*i) != priest_candidate && Some(*i) != merchant_candidate)
                .max_by_key(|(_, a)| {
                    let score = a.personality.dominance + a.personality.conscientiousness;
                    score.to_raw() as u64
                })
                .map(|(i, _)| i);
            if let Some(guard_idx) = guard_candidate {
                if let Some(council) = self.institutions.iter_mut().find(|i| i.kind == institutions::InstitutionKind::Council) {
                    council.assign_role("Guard Captain", AgentId::new(guard_idx as u64));
                    council.add_member(AgentId::new(guard_idx as u64));
                }
            }

            // §19.5.G: Set role_status for agents with institutional roles
            for (i, agent) in self.agents.iter_mut().enumerate() {
                let agent_id = AgentId::new(i as u64);
                let has_role = self.institutions.iter().any(|inst| inst.has_member(agent_id));
                if has_role {
                    agent.status.role_status = Fixed::from_f64(0.6);
                    agent.status.recompute();
                }
            }
        }

        tracing::info!(
            agents = self.agents.len(),
            sites = self.world.sites.len(),
            relationships = self.relationships.len(),
            norms = self.norms.norms().len(),
            institutions = self.institutions.len(),
            "World populated"
        );
    }

    /// Run the simulation for `n` ticks.
    pub fn run(&mut self, n: u64) {
        for _ in 0..n {
            self.tick();
        }
    }

    /// Execute a single simulation tick.
    pub fn tick(&mut self) {
        let tick = self.clock.advance();
        let tick_u64 = tick.as_u64();



        // Check for scenario shocks
        if let Some(ref scenario) = self.scenario {
            if let Some(shock) = scenario.shock_at(tick_u64) {
                tracing::warn!(
                    shock = ?shock.kind,
                    magnitude = shock.magnitude.to_f64(),
                    tick = tick_u64,
                    "Scenario shock applied"
                );
                match shock.kind {
                    ShockKind::Drought => {
                        for site in &mut self.world.sites {
                            for stock in &mut site.inventory {
                                if stock.resource_id == 0 {
                                    stock.quantity =
                                        (stock.quantity - shock.magnitude * Fixed::from_f64(10.0))
                                            .clamp_01();
                                }
                            }
                        }
                    }
                    ShockKind::Festival => {
                        for agent in &mut self.agents {
                            agent.emotions.joy =
                                (agent.emotions.joy + shock.magnitude * Fixed::from_f64(0.3))
                                    .clamp_01();
                        }
                    }
                }
            }
        }

        // Capture event count before systems run (before SystemContext borrows self.events)
        let pre_tick_events = self.events.len();

        // §19.5.J: Capture relationship snapshot before any systems modify relationships
        let rel_snapshot: Vec<(AgentId, AgentId, Fixed, Fixed)> = self.relationships.iter()
            .map(|r| (r.from, r.to, r.trust, r.affection))
            .collect();

        // Collect action starts to defer resource operations outside the ctx block
        let mut action_starts: Vec<(usize, ActionKind)> = Vec::new();

        // Prepare system context and run systems that need mutable borrow on events/rng
        {
            let mut ctx = SystemContext {
                tick: tick_u64,
                rng: &mut self.rng,
                world: &mut self.world,
                events: &mut self.events,
            };

            // Extract agent data for systems (std::mem::take avoids cloning for mutable fields;
            // personality is read-only so we keep .clone())
            let mut bodies: Vec<BodyState> = self.agents.iter_mut().map(|a| std::mem::take(&mut a.body)).collect();
            let mut needs: Vec<NeedState> = self.agents.iter_mut().map(|a| std::mem::take(&mut a.needs)).collect();
            let personalities: Vec<Personality> =
                self.agents.iter().map(|a| a.personality.clone()).collect();
            let mut goals: Vec<Vec<Goal>> = self.agents.iter_mut().map(|a| std::mem::take(&mut a.goals)).collect();
            let mut affects: Vec<Affect> = self.agents.iter_mut().map(|a| std::mem::take(&mut a.affect)).collect();
            let mut emotions: Vec<DiscreteEmotions> =
                self.agents.iter_mut().map(|a| std::mem::take(&mut a.emotions)).collect();

            // ── 0. Biological update (architecture-plan-2 §7) ──────────
            // Tick the rich biological substrate before cognitive processing.
            // Uses previous tick's emotions — biology reacts to current felt state.
            // EmbodiedState feeds endocrine/nervous signals into the legacy BodyState.
            for i in 0..self.agents.len() {
                let threat_level = emotions[i].fear + emotions[i].anger;
                let social_safety = Fixed::ONE - threat_level;
                let is_sleeping = matches!(self.agents[i].current_action, ActionKind::Rest);
                // Compute activity level from current action
                let activity_level = match self.agents[i].current_action {
                    ActionKind::Work => Fixed::from_f64(0.8),
                    ActionKind::Wander | ActionKind::Move { .. } => Fixed::from_f64(0.4),
                    ActionKind::Trade => Fixed::from_f64(0.3),
                    ActionKind::Socialize | ActionKind::Worship => Fixed::from_f64(0.2),
                    _ => Fixed::from_f64(0.1),
                };
                let ambient_temperature = Fixed::from_f64(0.5); // temperate default
                let crowding = Fixed::from_f64(0.3); // village-level crowding
                let hygiene = Fixed::from_f64(0.6); // moderate hygiene
                self.agents[i].embodied.tick_update(
                    threat_level, social_safety, is_sleeping,
                    activity_level, ambient_temperature, crowding, hygiene,
                );
                // Sync derived body fields from EmbodiedState back to legacy BodyState.
                // Compute values first to avoid borrow conflicts between embodied and body.
                // Boundary: hunger/thirst/sickness/injury are managed by health.rs and
                // routines.rs, not synced from embodied — they operate on separate tracks.
                let derived_health = self.agents[i].embodied.derived_health();
                let derived_energy = self.agents[i].embodied.derived_energy();
                let derived_fatigue = self.agents[i].embodied.derived_fatigue();
                self.agents[i].body.health = derived_health;
                self.agents[i].body.energy = derived_energy;
                self.agents[i].body.fatigue = derived_fatigue;

            }

            // §8.1.4: Collect regulation strategies from step 0b, applied after appraisal in step 6.
            // Pre-allocated Vec avoids repeated allocation; capacity matches agent count.
            let num_agents = self.agents.len();
            let mut reg_strategies: Vec<crate::psychology::emotion_regulation::RegulationStrategy> =
                Vec::with_capacity(num_agents);

            // ── 0b. Cognitive state update (§22.1) ────────────────────
            // §22.1: Stress reduces planning horizon, increases heuristic bias.
            // This must happen before action selection to affect decision-making.
            for i in 0..self.agents.len() {
                let stress = emotions[i].fear + emotions[i].anger;
                let need_fatigue = needs[i].fatigue.max(needs[i].hunger).max(needs[i].thirst);
                self.agents[i].cognitive.update(stress, need_fatigue);

                // Architecture-plan-2 §8.1.4: Emotion regulation
                // Compute social support from relationships (average trust of top-3 closest agents)
                // Uses a fixed-size stack array to avoid heap allocation per agent per tick.
                let social_support = {
                    let mut top3: [Fixed; 3] = [Fixed::ZERO; 3];
                    let mut count: usize = 0;
                    for r in self.relationships.iter() {
                        if r.from == AgentId::new(i as u64) {
                            // Insert into sorted top-3 (descending)
                            let pos = top3.iter().position(|t| r.trust > *t);
                            if let Some(p) = pos {
                                // Shift smaller values right, insert at p
                                for j in (p + 1..3).rev() {
                                    top3[j] = top3[j - 1];
                                }
                                top3[p] = r.trust;
                            }
                            count += 1;
                        }
                    }
                    if count > 0 {
                        let n = count.min(3);
                        let sum: Fixed = top3[..n].iter().fold(Fixed::ZERO, |acc, t| acc + *t);
                        sum / Fixed::from_int(n as i64)
                    } else {
                        Fixed::from_f64(0.3) // baseline when no relationships
                    }
                };
                let regulation_strategy = self.agents[i].emotion_regulation.select_strategy(
                    stress,
                    social_support,
                    personalities[i].extraversion,
                );
                // §8.1.4: Store regulation strategy for post-appraisal application.
                // The actual apply_strategy() call happens after appraisal (step 6)
                // so the skill-scaled boost uses the fresh, appraisal-derived affect values.
                reg_strategies.push(regulation_strategy);
                self.agents[i].emotion_regulation.update_capacity(
                    stress, need_fatigue, social_support,
                );

                // Architecture-plan-2 §8.1.10: Moral cognition update
                // §8.1.10: Compute witnessed violations from this tick's negative interactions.
                // High anger = agent perceived a norm violation; threat/insult events contribute.
                // Witnessed violations: anger signals perceived norm violation; fear of injustice adds
                let witnessed_violations = (emotions[i].anger * Fixed::from_f64(0.6)
                    + emotions[i].fear * Fixed::from_f64(0.2))
                    .clamp_01();
                // Personal violations: agent's own guilt/shame signal
                let personal_violations = (emotions[i].guilt * Fixed::from_f64(0.4)
                    + emotions[i].shame * Fixed::from_f64(0.3))
                    .clamp_01();
                // Moral achievements: pride and gratitude from prosocial interactions
                let moral_achievements = (emotions[i].pride * Fixed::from_f64(0.3)
                    + emotions[i].trust * Fixed::from_f64(0.2))
                    .clamp_01();
                self.agents[i].moral_cognition.update_moral_emotions(
                    witnessed_violations,
                    personal_violations,
                    moral_achievements,
                );

                // Architecture-plan-2 §8.1.16: Prospection update
                let agent_fear = emotions[i].fear;
                let agent_ambition = self.agents[i].personality.ambition;
                let agent_trauma = self.agents[i].embodied.nervous.trauma_load;
                let agent_depression = self.agents[i].psychopathology.depression_risk;
                self.agents[i].prospection.update(
                    agent_fear, agent_ambition, agent_trauma, agent_depression,
                );

                // Architecture-plan-2 §8.1.17: Narrative update
                // §8.1.17: Feed actual emotional events into narrative identity.
                // Positive events increase redemption_script; negative events increase victimhood/contamination.
                let positive_event_magnitude = if emotions[i].joy > Fixed::from_f64(0.3) {
                    emotions[i].joy * Fixed::from_f64(0.1)
                } else {
                    Fixed::ZERO
                };
                let negative_events = if emotions[i].fear + emotions[i].sadness > Fixed::from_f64(0.3) {
                    (emotions[i].fear + emotions[i].sadness) * Fixed::from_f64(0.1)
                } else {
                    Fixed::ZERO
                };
                if negative_events > Fixed::ZERO {
                    self.agents[i].narrative.interpret_negative_event(
                        negative_events,
                        social_support,
                        emotions[i].anger > Fixed::from_f64(0.3), // has_blame_target if angry
                    );
                }
                if positive_event_magnitude > Fixed::ZERO {
                    self.agents[i].narrative.interpret_positive_event(
                        positive_event_magnitude,
                        social_support, // social_recognition tied to social support
                    );
                }
                self.agents[i].narrative.update_theme();

                // Architecture-plan-2 §8.1.13: Developmental update
                // §8.1.13: Only tick developmental psychology every 100 ticks (≈daily).
                // Identity formation, moral development, and socialization are slow processes.
                if tick_u64.is_multiple_of(100) {
                    let agent_age = self.agents[i].age;
                    self.agents[i].developmental.tick_update(
                        agent_age,
                        social_support,
                        Fixed::from_f64(0.5), // education quality placeholder
                    );
                }

                // Architecture-plan-2 §8.1.15: Psychopathology update
                let chronic_stress = self.agents[i].embodied.endocrine.stress.chronic_load;
                let trauma_load = self.agents[i].embodied.nervous.trauma_load;
                let chronic_pain = self.agents[i].embodied.nervous.pain.chronic;
                let sleep_debt = self.agents[i].embodied.circadian.sleep_debt;
                let depression_vuln = self.agents[i].embodied.genome.trait_predispositions.depression_vulnerability;
                let addiction_risk = self.agents[i].embodied.genome.trait_predispositions.addiction_risk;
                // §8.1.15: Compute social isolation from relationship quality
                let social_isolation = (Fixed::ONE - social_support).max(Fixed::ZERO);
                self.agents[i].psychopathology.tick_update(
                    chronic_stress, trauma_load,
                    social_isolation,
                    Fixed::ZERO, // humiliation_recent — TODO: detect from interactions
                    negative_events,
                    Fixed::ZERO, // moral_injury — TODO: detect from moral_cognition
                    chronic_pain, sleep_debt,
                    depression_vuln, addiction_risk,
                    social_support,
                );

                // Architecture-plan-2 §8.1.19: Skill/habit update
                self.agents[i].psych_skills.update_automaticity(stress, need_fatigue);
                if tick_u64.is_multiple_of(100) {
                    self.agents[i].psych_skills.decay_habits(tick_u64);
                }
            }

            // Compute avg_wealth once before status updates (O(N) instead of O(N²))
            let avg_wealth: Fixed = if self.agents.is_empty() {
                Fixed::ZERO
            } else {
                self.agents.iter().map(|a| a.wealth.coin).fold(Fixed::ZERO, |acc, c| acc + c)
                    / Fixed::from_int(self.agents.len() as i64)
            };

            for i in 0..self.agents.len() {
                // Architecture-plan-2 §11.1: Status dimensions update.
                // Decay shame and honor gradually; update wealth_rank from relative wealth.
                self.agents[i].status_v2.decay();
                // Sync authority from existing institutional role_status
                self.agents[i].status_v2.authority = self.agents[i].status.role_status;
                if avg_wealth > Fixed::ZERO {
                    self.agents[i].status_v2.wealth_rank =
                        (self.agents[i].wealth.coin / avg_wealth * Fixed::from_f64(0.5)).clamp_01();
                }
                // Update moral_reputation from moral pride and gratitude (weighted average)
                self.agents[i].status_v2.moral_reputation =
                    (self.agents[i].moral_cognition.moral_emotions.pride * Fixed::from_f64(0.5)
                        + self.agents[i].moral_cognition.moral_emotions.gratitude * Fixed::from_f64(0.5))
                    .clamp_01();
                // Update prestige from relationship count (social connections)
                let num_connections = self.agents[i].relationship_v2s.len() as i64;
                if num_connections > 0 {
                    let avg_quality: Fixed = self.agents[i].relationship_v2s.iter()
                        .map(crate::relationship_v2::RelationshipV2::quality)
                        .fold(Fixed::ZERO, |acc, q| acc + q)
                        / Fixed::from_int(num_connections);
                    self.agents[i].status_v2.network_centrality = avg_quality;
                }

                // Architecture-plan-2 §10.2: RelationshipV2 decay.
                // Decay all relationship V2s once per tick.
                for rv2 in &mut self.agents[i].relationship_v2s {
                    rv2.decay(1);
                }
            }

            // ── 1. Need decay (nonlinear pressure, §9.1) ────────────────
            tracing::debug!(tick = tick_u64, "systems::need_decay");
            systems::system_need_decay(&mut ctx, &bodies, &mut needs);

            // ── 2. Body state update ──────────────────────────────────
            tracing::debug!(tick = tick_u64, "systems::body_update");
            systems::system_body_update(&mut ctx, &mut bodies, &needs);

            // ── 3. Goal generation ────────────────────────────────────
            tracing::debug!(tick = tick_u64, "systems::goal_generation");
            systems::system_goal_generation(&mut ctx, &personalities, &needs, &mut goals, &emotions);

            // ── 4. Action execution (per-tick effects) ────────────────
            for i in 0..self.agents.len() {
                // Track causal provenance flags per agent per tick
                let mut was_interrupted_by_critical_needs = false;
                let mut intention_abandoned_this_tick = false;

                // §24.5: Intention commitment — check if current intention should be abandoned
                if let Some(ref intention) = self.agents[i].intention {
                    if intention.completed {
                        // §3.2: Goal completed — move to completed list for learning
                        let completed_goal = Goal {
                            kind: intention.goal_kind,
                            priority: Fixed::from_f64(0.5),
                            commitment: Fixed::ONE,
                            created_tick: intention.formed_tick,
                            source: GoalSource::Identity, // default source for completed goals
                        };
                        self.agents[i].completed_goals.push(completed_goal);
                        if self.agents[i].completed_goals.len() > 50 {
                            self.agents[i].completed_goals.remove(0);
                        }
                        self.agents[i].intention = None;
                        intention_abandoned_this_tick = true;
                    } else {
                        let stress = emotions[i].fear + emotions[i].anger;
                        let emotional_shock = emotions[i].anger > Fixed::from_f64(0.5) || emotions[i].fear > Fixed::from_f64(0.5);
                        if intention.should_abandon(tick_u64, stress, emotional_shock) {
                            // §3.2: Goal abandoned — move to rejected list for learning
                            let rejected_goal = Goal {
                                kind: intention.goal_kind,
                                priority: Fixed::from_f64(0.5),
                                commitment: Fixed::ZERO,
                                created_tick: intention.formed_tick,
                                source: GoalSource::Identity,
                            };
                            self.agents[i].rejected_goals.push(rejected_goal);
                            if self.agents[i].rejected_goals.len() > 50 {
                                self.agents[i].rejected_goals.remove(0);
                            }
                            self.agents[i].intention = None;
                            self.agents[i].action_progress = 0; // force reselection
                            intention_abandoned_this_tick = true;
                        }
                    }
                }

                // Action interruption: force address critical needs (> 0.9)
                if self.agents[i].action_progress > 0 {
                    let critical = needs[i].hunger > Fixed::from_f64(0.9)
                        || needs[i].thirst > Fixed::from_f64(0.9)
                        || needs[i].fatigue > Fixed::from_f64(0.95);
                    if critical {
                        // §3.2: Goal interrupted by critical needs — move to rejected list
                        if let Some(ref intention) = self.agents[i].intention {
                            let rejected_goal = Goal {
                                kind: intention.goal_kind,
                                priority: Fixed::from_f64(0.5),
                                commitment: Fixed::ZERO,
                                created_tick: intention.formed_tick,
                                source: GoalSource::Identity,
                            };
                            self.agents[i].rejected_goals.push(rejected_goal);
                            if self.agents[i].rejected_goals.len() > 50 {
                                self.agents[i].rejected_goals.remove(0);
                            }
                        }
                        self.agents[i].action_progress = 0;
                        self.agents[i].intention = None; // critical needs override intentions
                        was_interrupted_by_critical_needs = true;
                        intention_abandoned_this_tick = true;
                    }
                }

                if self.agents[i].action_progress == 0 {
                    let total_grain = ctx.world.total_food();
                    let total_water = ctx.world.total_water();
                    // Compute norm pressure using the registry's formula (identity-aware)
                    let conformity = personalities[i].conformity;
                    let avg_identity_strength = self.agents[i].identity.strength_of(IdentityKind::Farmer)
                        .max(self.agents[i].identity.strength_of(IdentityKind::Parent))
                        .max(self.agents[i].identity.strength_of(IdentityKind::Believer));
                    // §12: Institution enforcement amplifies norm pressure.
                    // Council enforcement capacity multiplies the pressure felt by agents.
                    let enforcement_multiplier = self.institutions.iter()
                        .filter(|i| i.kind == institutions::InstitutionKind::Council)
                        .map(|i| Fixed::ONE + i.enforcement_capacity * Fixed::from_f64(0.5))
                        .fold(Fixed::ONE, std::cmp::Ord::max);

                    // §22.1: Moral values modulate norm pressure.
                    // High fairness/authority → stronger norm compliance.
                    // High liberty → weaker norm compliance.
                    let moral_modifier = self.agents[i].moral_values.fairness
                        * Fixed::from_f64(0.15)
                        + self.agents[i].moral_values.authority * Fixed::from_f64(0.1)
                        - self.agents[i].moral_values.liberty * Fixed::from_f64(0.08);
                    let norm_pressure = self.norms
                        .norms()
                        .iter()
                        .map(|n| self.norms.compute_pressure(conformity, avg_identity_strength, n.id))
                        .fold(Fixed::ZERO, |a, b| a + b) * enforcement_multiplier
                        - moral_modifier;

                    // §10.3: Routine bias — agents follow daily schedules when stable.
                    // §24: Personality modulates routine adherence — conscientiousness
                    // increases it, openness decreases it.
                    let (routine_action, routine_strength) = self.agents[i].routine.preferred_action(tick);
                    let stress = emotions[i].fear + emotions[i].anger;
                    let follow_routine = self.agents[i].routine.should_follow(
                        needs[i].hunger, needs[i].thirst, needs[i].fatigue, stress
                    );
                    // Personality-modulated routine strength: conscientiousness boosts, openness reduces
                    let personality_routine_modifier = personalities[i].conscientiousness
                        * Fixed::from_f64(0.3)
                        - personalities[i].openness * Fixed::from_f64(0.2);
                    let effective_routine_strength = (routine_strength + personality_routine_modifier).clamp_01();

                    // §22.1: When stressed, agents favor habit/routine over utility.
                    // High heuristic bias → agents follow routine more readily.
                    let effective_routine_threshold = if self.agents[i].cognitive.heuristic_bias > Fixed::from_f64(0.5) {
                        Fixed::from_f64(0.3) // stressed agents follow routine at lower threshold
                    } else {
                        Fixed::from_f64(0.5) // normal agents need stronger routine signal
                    };

                    let action = if follow_routine && effective_routine_strength > effective_routine_threshold {
                        // §10.3: Routine creates behavioral stability — prefer scheduled action
                        routine_action
                    } else if !self.agents[i].feuds.is_empty()
                        && emotions[i].anger > Fixed::from_f64(0.4)
                        && needs[i].hunger < Fixed::from_f64(0.85)
                        && needs[i].thirst < Fixed::from_f64(0.85)
                    {
                        // §19.5.G: Angry feuding agents Move toward their feud target, NOT when critical needs demand attention
                        let feud_target = self.agents[i].feuds[0];
                        ActionKind::Move {
                            target_x: self.agents[feud_target].position.x,
                            target_y: self.agents[feud_target].position.y,
                        }
                    } else {
                        // §12.3: Institution collective morale modulates norm compliance.
                        // Members of institutions with high morale are more norm-compliant.
                        // Note: norm_pressure is negative = compliant, positive = violating.
                        let mut adjusted_pressure = norm_pressure;
                        for inst in &self.institutions {
                            if inst.has_member(AgentId::new(i as u64)) {
                                // High morale → more compliant (pressure becomes more negative)
                                let morale_bonus = inst.collective.morale * Fixed::from_f64(0.1);
                                adjusted_pressure -= morale_bonus;
                            }
                        }
                        actions::select_action(
                            &needs[i],
                            &personalities[i],
                            &goals[i],
                            ctx.rng,
                            total_grain,
                            total_water,
                            &self.agents[i].identity,
                            adjusted_pressure,
                        )
                    };
                    self.agents[i].current_action = action;
                    self.agents[i].action_progress = action.definition().duration_ticks;
                    action_starts.push((i, action));

                    let agent_id = AgentId::new(i as u64);

                    // §34: Record decision trace for causal provenance
                    {
                        let mut factors = Vec::new();
                        factors.push(DecisionFactor {
                            kind: "need_hunger".into(),
                            magnitude: needs[i].hunger,
                            description: format!("Hunger: {:.2}", needs[i].hunger.to_f64()),
                        });
                        factors.push(DecisionFactor {
                            kind: "need_thirst".into(),
                            magnitude: needs[i].thirst,
                            description: format!("Thirst: {:.2}", needs[i].thirst.to_f64()),
                        });
                        factors.push(DecisionFactor {
                            kind: "need_fatigue".into(),
                            magnitude: needs[i].fatigue,
                            description: format!("Fatigue: {:.2}", needs[i].fatigue.to_f64()),
                        });
                        factors.push(DecisionFactor {
                            kind: "norm_pressure".into(),
                            magnitude: norm_pressure,
                            description: format!("Norm pressure: {:.2}", norm_pressure.to_f64()),
                        });
                        if follow_routine && effective_routine_strength > Fixed::from_f64(0.5) {
                            factors.push(DecisionFactor {
                                kind: "routine".into(),
                                magnitude: effective_routine_strength,
                                description: format!("Routine strength: {:.2}", effective_routine_strength.to_f64()),
                            });
                        }
                        self.provenance.record_decision(DecisionTrace {
                            agent: agent_id,
                            tick: tick_u64,
                            action_name: format!("{action:?}"),
                            factors,
                            from_routine: follow_routine && effective_routine_strength > Fixed::from_f64(0.5),
                            interrupted_by_critical_needs: was_interrupted_by_critical_needs,
                            intention_abandoned: intention_abandoned_this_tick,
                        });
                    }

                    // §24.5: Create intention for the selected action's goal
                    let goal_kind = match action {
                        ActionKind::Eat => GoalKind::Eat,
                        ActionKind::Drink => GoalKind::Drink,
                        ActionKind::Rest => GoalKind::Rest,
                        ActionKind::Work => GoalKind::Work,
                        ActionKind::Socialize => GoalKind::Socialize,
                        ActionKind::Worship => GoalKind::Worship,
                        _ => GoalKind::Eat, // fallback
                    };
                    self.agents[i].intention = Some(
                        crate::person::Intention::new(goal_kind, tick_u64, personalities[i].conscientiousness)
                    );

                    // Push events (no resource ops yet — those happen after ctx drops)
                    match action {
                        ActionKind::Eat => {
                            ctx.events.push(SimEvent::AgentAte {
                                agent: agent_id,
                                food: EntityId::new(0),
                                tick,
                            });
                        }
                        ActionKind::Drink => {
                            ctx.events.push(SimEvent::AgentDrank {
                                agent: agent_id,
                                source: EntityId::new(1),
                                tick,
                            });
                        }
                        ActionKind::Rest => {
                            ctx.events.push(SimEvent::AgentRested {
                                agent: agent_id,
                                tick,
                            });
                        }
                        _ => {}
                    }
                }

                let action = self.agents[i].current_action;
                actions::apply_action_tick(action, &mut bodies[i], &mut needs[i]);

                self.agents[i].action_progress =
                    self.agents[i].action_progress.saturating_sub(1);
            }

            // ── 5. Social interactions ────────────────────────────────
            {
                // §2.4: Pass agent positions for proximity-based social interactions
                let agent_positions: Vec<(i32, i32)> = self.agents.iter()
                    .map(|a| (a.position.x, a.position.y))
                    .collect();
                let agent_info: Vec<_> = self
                    .agents
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        (
                            AgentId::new(i as u64),
                            a.personality.openness,
                            a.personality.agreeableness,
                            a.personality.extraversion,
                        )
                    })
                    .collect();

                // §5.4: Compute faction membership matrix — same_faction_matrix[i][j] = true if agents i,j share a faction
                let faction_members: Vec<Vec<usize>> = self.institutions.iter()
                    .filter(|i| i.kind == InstitutionKind::Faction)
                    .map(|f| f.members.iter().map(|m| m.as_u64() as usize).collect())
                    .collect();
                let n = self.agents.len();
                let mut same_faction_matrix = vec![vec![false; n]; n];
                for member_list in &faction_members {
                    for &a in member_list {
                        for &b in member_list {
                            if a < n && b < n {
                                same_faction_matrix[a][b] = true;
                            }
                        }
                    }
                }

                social::system_social_interactions(
                    &agent_info,
                    &agent_positions,
                    &same_faction_matrix,
                    &mut self.relationships,
                    ctx.events,
                    tick,
                    ctx.rng,
                );
            }

            // ── 6. Appraisal — emotions from state ────────────────────
            for i in 0..self.agents.len() {
                let appraisal = Appraisal {
                    goal_relevance: needs[i].hunger.max(needs[i].thirst),
                    goal_congruence: if needs[i].hunger < Fixed::from_f64(0.5) {
                        Fixed::from_f64(0.3)
                    } else {
                        Fixed::from_f64(-0.3)
                    },
                    coping_potential: personalities[i].conscientiousness,
                    expectedness: Fixed::from_f64(0.5),
                    fairness: Fixed::from_f64(0.5),
                    agency: Agency::Circumstance,
                    social_visibility: Fixed::ZERO,
                    identity_relevance: Fixed::from_f64(0.2),
                };

                let delta = appraisal::appraise(&appraisal, tick);

                emotions[i].fear = (emotions[i].fear + delta.fear).clamp_01();
                emotions[i].anger = (emotions[i].anger + delta.anger).clamp_01();
                emotions[i].joy = (emotions[i].joy + delta.joy).clamp_01();
                emotions[i].sadness = (emotions[i].sadness + delta.sadness).clamp_01();
                emotions[i].trust = (emotions[i].trust + delta.trust).clamp_01();
                emotions[i].shame = (emotions[i].shame + delta.shame).clamp_01();
                emotions[i].pride = (emotions[i].pride + delta.pride).clamp_01();
                emotions[i].guilt = (emotions[i].guilt + delta.guilt).clamp_01();

                affects[i].valence = (emotions[i].joy - emotions[i].sadness - emotions[i].fear
                    - emotions[i].anger)
                    .clamp_01();
                affects[i].arousal = (emotions[i].fear + emotions[i].anger + emotions[i].joy)
                    * Fixed::from_f64(0.5);
                // §8.1.4: Apply emotion regulation AFTER appraisal.
                // Now apply_strategy() uses the fresh, appraisal-derived affect values
                // as input — the skill-scaled boost is computed from the correct target state.
                if i < reg_strategies.len() {
                    let (reg_vd, reg_ad) = self.agents[i].emotion_regulation.apply_strategy(
                        reg_strategies[i],
                        affects[i].valence,
                        affects[i].arousal,
                    );
                    affects[i].valence = (affects[i].valence + reg_vd).clamp_01();
                    affects[i].arousal = (affects[i].arousal + reg_ad).clamp_01();
                }
            }

            // ── 7. Emotion decay ──────────────────────────────────────
            // §22.1: Emotions decay slowly — 0.002/tick ≈ full decay in ~500 ticks.
            // This allows emotions to accumulate and create meaningful dynamics.
            for emotion in &mut emotions {
                let decay = Fixed::from_f64(0.002);
                emotion.fear = (emotion.fear - decay).clamp_01();
                emotion.anger = (emotion.anger - decay).clamp_01();
                emotion.joy = (emotion.joy - decay).clamp_01();
                emotion.sadness = (emotion.sadness - decay).clamp_01();
                emotion.shame = (emotion.shame - decay).clamp_01();
                emotion.pride = (emotion.pride - decay).clamp_01();
                emotion.guilt = (emotion.guilt - decay).clamp_01();
            }

            // ── 8. Belief resistance decay ────────────────────────────
            for agent_beliefs in &mut self.agents {
                belief_update::decay_belief_resistance(
                    &mut agent_beliefs.beliefs,
                    BELIEF_RESISTANCE_DECAY,
                );
            }

            // ── Write back (std::mem::take avoids cloning) ────────────
            for (i, agent) in self.agents.iter_mut().enumerate() {
                agent.body = std::mem::take(&mut bodies[i]);
                agent.needs = std::mem::take(&mut needs[i]);
                agent.goals = std::mem::take(&mut goals[i]);
                agent.affect = std::mem::take(&mut affects[i]);
                agent.emotions = std::mem::take(&mut emotions[i]);
            }

        } // ctx is dropped here, releasing mutable borrows on self.events and self.rng

        // §6: Spatial movement — update agent positions AFTER ctx drops (no borrow conflicts)
        {
            let world_w = self.world.width as i32;
            let world_h = self.world.height as i32;
            for i in 0..self.agents.len() {
                let action = self.agents[i].current_action;
                match action {
                    ActionKind::Wander => {
                        let dx: i32 = self.rng.get_mut(RngStream::Behavior).random_range(-1..=1);
                        let dy: i32 = self.rng.get_mut(RngStream::Behavior).random_range(-1..=1);
                        self.agents[i].position.x = (self.agents[i].position.x + dx).clamp(0, world_w - 1);
                        self.agents[i].position.y = (self.agents[i].position.y + dy).clamp(0, world_h - 1);
                    }
                    ActionKind::Move { target_x, target_y } => {
                        // §6: Move one step toward target (Manhattan pathfinding)
                        let dx = (target_x - self.agents[i].position.x).clamp(-1, 1);
                        let dy = (target_y - self.agents[i].position.y).clamp(-1, 1);
                        self.agents[i].position.x = (self.agents[i].position.x + dx).clamp(0, world_w - 1);
                        self.agents[i].position.y = (self.agents[i].position.y + dy).clamp(0, world_h - 1);
                        // §6: Completion detection — finish action when at target
                        if self.agents[i].position.x == target_x && self.agents[i].position.y == target_y {
                            self.agents[i].action_progress = 0;
                        }
                    }
                    _ => {}
                }
            }
        }

        // §19.5.J: Record relationship traces for any changes from social interactions
        // This must happen after ctx is dropped so we can read self.events.
        for (i, rel) in self.relationships.iter().enumerate() {
            if i < rel_snapshot.len() {
                let (from, to, old_trust, old_affection) = rel_snapshot[i];
                let trust_changed = (rel.trust - old_trust).abs() > Fixed::from_f64(0.001);
                let affection_changed = (rel.affection - old_affection).abs() > Fixed::from_f64(0.001);
                if trust_changed || affection_changed {
                    let cause = self.events[pre_tick_events..].iter()
                        .find_map(|ev| {
                            if let SimEvent::InteractionOccurred { from: ef, to: et, kind, .. } = ev {
                                if *ef == from && *et == to {
                                    Some(format!("{kind:?}"))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "social_interaction".into());
                    self.provenance.record_relationship(crate::provenance::RelationshipTrace {
                        from,
                        to,
                        tick: tick_u64,
                        cause,
                        old_trust,
                        new_trust: rel.trust,
                        old_affection,
                        new_affection: rel.affection,
                        description: format!("Social interaction changed trust {}->{}, affection {}->{}",
                            old_trust.to_f64(), rel.trust.to_f64(), old_affection.to_f64(), rel.affection.to_f64()),
                    });
                }
            }
        }

        // ── 4b. Resource operations, journal recording, intention tracking ──

        for (agent_idx, action) in &action_starts {
            let agent_id = AgentId::new(*agent_idx as u64);
            // §24.5: Track intention success/failure when action completes.
            // The resource operation outcome (e.g., Eat failing because no grain)
            // is known here, so we record intention success/failure based on
            // the actual resource operation result.
            let action_succeeded = match action {
                ActionKind::Eat => {
                    let amount = Fixed::from_f64(0.1);
                    // §19.5.E: Use access-checking method to enforce resource access rights.
                    if let Some(farm_idx) = self.world.accessible_farm_with_grain(agent_id, &self.institutions) {
                        let taken = self.world.consume_resource(farm_idx, GRAIN_RESOURCE_ID, amount);
                        // §13.3: Consumers pay coin for resources consumed
                        let cost = taken * self.market.price(GRAIN_RESOURCE_ID);
                        self.agents[*agent_idx].wealth.coin = (self.agents[*agent_idx].wealth.coin - cost).max(Fixed::ZERO);
                        self.journal.record(tick_u64, agent_id, JournalEntryKind::Consumed { resource: "grain".into(), amount: taken.to_f64() });
                        taken > Fixed::ZERO
                    } else {
                        // §19.5.D: Theft detection — no accessible farm, but inaccessible farms with grain exist?
                        if let Some(thief_idx) = self.world.inaccessible_farm_with_grain(agent_id, &self.institutions) {
                            self.enforce_theft(*agent_idx, agent_id, thief_idx, GRAIN_RESOURCE_ID, amount, tick_u64, tick)
                        } else {
                            false
                        }
                    }
                }
                ActionKind::Drink => {
                    let amount = Fixed::from_f64(0.15);
                    // §19.5.E: Use access-checking method to enforce resource access rights.
                    if let Some(well_idx) = self.world.accessible_well_with_water(agent_id, &self.institutions) {
                        let taken = self.world.consume_resource(well_idx, WATER_RESOURCE_ID, amount);
                        // §13.3: Consumers pay coin for water consumed
                        let cost = taken * self.market.price(WATER_RESOURCE_ID);
                        self.agents[*agent_idx].wealth.coin = (self.agents[*agent_idx].wealth.coin - cost).max(Fixed::ZERO);
                        self.journal.record(tick_u64, agent_id, JournalEntryKind::Consumed { resource: "water".into(), amount: taken.to_f64() });
                        taken > Fixed::ZERO
                    } else {
                        // §19.5.D: Water theft detection — no accessible well, but inaccessible wells with water exist?
                        if let Some(thief_idx) = self.world.inaccessible_well_with_water(agent_id, &self.institutions) {
                            self.enforce_theft(*agent_idx, agent_id, thief_idx, WATER_RESOURCE_ID, amount, tick_u64, tick)
                        } else {
                            false
                        }
                    }
                }
                ActionKind::Work => {
                    // §4.2: Productivity = base (conscientiousness) + skill bonus
                    let skill_bonus = self.agents[*agent_idx].skills.farming * Fixed::from_f64(0.02);
                    let productivity = (self.agents[*agent_idx].personality.conscientiousness * Fixed::from_f64(0.05) + skill_bonus).clamp_01();
                    if let Some(farm_idx) = self.world.best_farm_for_work() {
                        self.world.produce_resource(farm_idx, GRAIN_RESOURCE_ID, productivity);
                        // §13.3: Workers earn coin proportional to productivity
                        let wage = productivity * self.market.price(GRAIN_RESOURCE_ID) * Fixed::from_f64(0.3);
                        self.agents[*agent_idx].wealth.coin += wage;

                        self.journal.record(tick_u64, agent_id, JournalEntryKind::Worked { productivity: productivity.to_f64() });
                        true
                    } else {
                        false
                    }
                }
                ActionKind::Rest => {
                    self.journal.record(tick_u64, agent_id, JournalEntryKind::Rested);
                    true
                }
                ActionKind::Worship => {
                    self.journal.record(tick_u64, agent_id, JournalEntryKind::Worshiped);
                    true
                }                    ActionKind::Trade => {
                    // §13.3: Agent-to-agent trade — find a nearby seller at a market site
                    let buyer_coin = self.agents[*agent_idx].wealth.coin;
                    // Find a seller: nearby agent who owns a farm with grain
                    let seller_info = self.agents.iter().enumerate()
                        .find(|(j, a)| {
                            *j != *agent_idx
                                && self.agents[*agent_idx].position.manhattan_distance(&a.position) <= 3
                                && a.home_site.is_some() // seller has a home/owned site
                        })
                        .and_then(|(j, a)| {
                            // Check if seller's home is a farm with grain
                            a.home_site.and_then(|hs| {
                                if hs < self.world.sites.len()
                                    && self.world.sites[hs].kind == crate::world::SiteKind::Farm
                                {
                                    let grain = self.world.sites[hs].inventory.iter()
                                        .find(|s| s.resource_id == GRAIN_RESOURCE_ID)
                                        .map_or(Fixed::ZERO, |s| s.quantity);
                                    if grain > Fixed::ZERO {
                                        Some((j, hs, grain))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                        });
                    if let Some((seller, farm_idx, _available)) = seller_info {
                        let trust = self.relationships.iter()
                            .find(|r| r.from == AgentId::new(*agent_idx as u64) && r.to == AgentId::new(seller as u64))
                            .map_or(Fixed::from_f64(0.5), |r| r.trust);
                        let quantity = Fixed::from_f64(0.1);
                        // §13.3: Execute trade — buyer pays coin, seller's farm provides grain
                        let base_price = self.market.price(GRAIN_RESOURCE_ID);
                        // Trust discount: high trust → lower price
                        let trust_modifier = Fixed::from_f64(1.0) - trust * Fixed::from_f64(0.2) + (Fixed::ONE - trust) * Fixed::from_f64(0.1);
                        let price = (base_price * trust_modifier).max(Fixed::from_f64(1.0));
                        let cost = price * quantity;
                        if buyer_coin >= cost {
                            let taken = self.world.consume_resource(farm_idx, GRAIN_RESOURCE_ID, quantity);
                            if taken > Fixed::ZERO {
                                let actual_cost = price * taken;
                                // Buyer pays coin to seller (farm owner)
                                self.agents[*agent_idx].wealth.coin = (self.agents[*agent_idx].wealth.coin - actual_cost).max(Fixed::ZERO);
                                self.agents[seller].wealth.coin += actual_cost;
                                self.events.push(SimEvent::TradeOccurred {
                                    buyer: AgentId::new(*agent_idx as u64),
                                    seller: AgentId::new(seller as u64),
                                    good: ResourceId::new(GRAIN_RESOURCE_ID),
                                    quantity: taken,
                                    price,
                                    tick,
                                });
                                self.journal.record(tick_u64, agent_id, JournalEntryKind::Consumed { resource: "grain_via_trade".into(), amount: taken.to_f64() });
                                // §19.5.J: Record relationship trace — trade builds trust
                                if let Some(rel) = self.relationships.iter_mut()
                                    .find(|r| r.from == AgentId::new(*agent_idx as u64) && r.to == AgentId::new(seller as u64))
                                {
                                    let old_trust = rel.trust;
                                    rel.trust = (rel.trust + Fixed::from_f64(0.02)).clamp_01();
                                    self.provenance.record_relationship(crate::provenance::RelationshipTrace {
                                        from: AgentId::new(*agent_idx as u64),
                                        to: AgentId::new(seller as u64),
                                        tick: tick_u64,
                                        cause: "trade".into(),
                                        old_trust,
                                        new_trust: rel.trust,
                                        old_affection: rel.affection,
                                        new_affection: rel.affection,
                                        description: format!("Trade built trust ({} -> {})", old_trust.to_f64(), rel.trust.to_f64()),
                                    });
                                }
                                true
                            } else {
                                false
                            }
                        } else {
                            false // insufficient funds
                        }
                    } else {
                        false // no nearby seller with grain
                    }
                }
                _ => false,
            };

            // Record intention success/failure based on actual resource outcome
            if let Some(ref mut intention) = self.agents[*agent_idx].intention {
                if action_succeeded {
                    intention.record_success();
                } else {
                    intention.record_failure();
                }
            }
            // §22: Track success rate for derived mental state computation
            self.agents[*agent_idx].recent_attempts += 1;
            if action_succeeded {
                self.agents[*agent_idx].recent_successes += 1;
            }
        }

        // ── 9. Memory encoding from this tick's events ───────────────
        // §22.5: Use attention system to compute salience instead of hardcoded values.
        // Only events that pass the attention threshold are encoded into memory.
        for (i, agent) in self.agents.iter_mut().enumerate() {
            for ev in &self.events[pre_tick_events..] {
                // §22.5: Attention computes salience based on intensity, novelty, relevance
                let salience = agent.attention.compute_salience(
                    ev,
                    AgentId::new(i as u64),
                    &agent.needs,
                    &agent.affect,
                );

                // Only encode events that exceed the attention threshold
                if salience < Fixed::from_f64(0.2) {
                    continue;
                }

                // Emotional intensity scales with arousal
                let emotional = agent.affect.arousal * Fixed::from_f64(0.6) + Fixed::from_f64(0.1);
                match ev {
                    SimEvent::AgentAte { agent: a, .. } if a.as_u64() == i as u64 => {
                        agent.memory.encode(MemoryKind::Consumption, tick_u64, salience, emotional, None, MemoryTag::AteFood);
                    }
                    SimEvent::AgentDrank { agent: a, .. } if a.as_u64() == i as u64 => {
                        agent.memory.encode(MemoryKind::Consumption, tick_u64, salience, emotional, None, MemoryTag::DrankWater);
                    }
                    SimEvent::AgentRested { agent: a, .. } if a.as_u64() == i as u64 => {
                        agent.memory.encode(MemoryKind::Positive, tick_u64, salience, emotional, None, MemoryTag::Rested);
                    }
                    SimEvent::InteractionOccurred { from, to, kind, .. } => {
                        if from.as_u64() == i as u64 {
                            let tag = match kind {
                                mindstrata_core::event::InteractionKind::Help => MemoryTag::HelpedBy,
                                mindstrata_core::event::InteractionKind::Threaten => MemoryTag::ThreatenedBy,
                                mindstrata_core::event::InteractionKind::Insult => MemoryTag::InsultedBy,
                                mindstrata_core::event::InteractionKind::Gossip => MemoryTag::GossipedAbout,
                                mindstrata_core::event::InteractionKind::Trade => MemoryTag::TradedWith,
                                _ => MemoryTag::TalkedTo,
                            };
                            let kind = match kind {
                                mindstrata_core::event::InteractionKind::Threaten | mindstrata_core::event::InteractionKind::Insult => MemoryKind::Negative,
                                mindstrata_core::event::InteractionKind::Help | mindstrata_core::event::InteractionKind::Comfort => MemoryKind::Positive,
                                _ => MemoryKind::Social,
                            };
                            agent.memory.encode(kind, tick_u64, salience, emotional, Some(to.as_u64() as u32), tag);
                        }
                        if to.as_u64() == i as u64 {
                            agent.memory.encode(MemoryKind::Social, tick_u64, salience, emotional, Some(from.as_u64() as u32), MemoryTag::TalkedTo);
                        }
                    }
                    _ => {}
                }
            }
            // Probabilistic memory rehearsal (every 10 ticks)
            if tick_u64.is_multiple_of(10) {
                agent.memory.rehearse_random(self.rng.get_mut(RngStream::Behavior));
            }
            // Decay memories
            agent.memory.decay(tick_u64);

            // §22.5: Attention maintenance — decay habituation, replenish budget
            agent.attention.decay_habituation(Fixed::from_f64(0.005));
            let stress = agent.emotions.fear + agent.emotions.anger;
            agent.attention.replenish_budget(stress, agent.needs.fatigue);

            // §19.5.G: Update status from current wealth and social connections
            agent.status.wealth_status = (agent.wealth.coin / Fixed::from_f64(20.0)).clamp_01();
            // Social status: ratio of positive relationships to total relationships
            // §19.5.G: A relationship is "positive" when trust exceeds 0.6
            let agent_id = AgentId::new(i as u64);
            let (positive_rels, total_rels) = self.relationships.iter()
                .filter(|r| r.from == agent_id)
                .fold((0u32, 0u32), |(pos, total), r| {
                    (pos + if r.trust > Fixed::from_f64(0.6) { 1 } else { 0 }, total + 1)
                });
            if total_rels > 0 {
                agent.status.social_status = Fixed::from_f64(positive_rels as f64 / total_rels as f64);
            }
            agent.status.recompute();

            // §22: Memory reconsolidation — current emotions bias recalled memories
            agent.memory.reconsolidate(agent.emotions.anger, agent.emotions.joy);

            // §4.2: Skill improvement — agents improve skills through repeated practice.
            // Small increments per tick; skills cap at 1.0.
            let skill_gain = SKILL_GAIN_PER_TICK;
            match agent.current_action {
                crate::actions::ActionKind::Work => {
                    agent.skills.farming = (agent.skills.farming + skill_gain).clamp_01();
                }
                crate::actions::ActionKind::Trade => {
                    agent.skills.trading = (agent.skills.trading + skill_gain).clamp_01();
                }
                crate::actions::ActionKind::Socialize | crate::actions::ActionKind::Worship => {
                    agent.skills.social = (agent.skills.social + skill_gain).clamp_01();
                }
                _ => {}
            }
        }

        // ── 16. Ecology: season advance and fertility dynamics ──
        let new_season = self.season.advance();
        if new_season {
            tracing::info!(
                season = self.season.current.name(),
                year = self.season.year,
                "New season"
            );
        }
        // Reset work ticks tracker for this tick
        for tick in &mut self.site_work_ticks {
            *tick = 0;
        }
        // Count work actions per site this tick
        for (_agent_idx, action) in &action_starts {
            if let ActionKind::Work = action {
                if let Some(farm_idx) = self.world.best_farm_for_work() {
                    if farm_idx < self.site_work_ticks.len() {
                        self.site_work_ticks[farm_idx] += 1;
                    }
                }
            }
        }
        // Apply ecology system to tile fertility
        let mut fertilities: Vec<Fixed> = self.world.tiles.iter().map(|t| t.fertility).collect();
        ecology::system_ecology(&self.season, &mut fertilities, &self.site_work_ticks, &self.ecology_config);
        for (i, tile) in self.world.tiles.iter_mut().enumerate() {
            if i < fertilities.len() {
                tile.fertility = fertilities[i];
            }
        }

        // ── 17. Health: disease effects on agents ──
        {
            let health_cfg = self.health_config.clone();
            let n = self.agents.len();
            for i in 0..n {
                let hunger = self.agents[i].needs.hunger;
                let fatigue = self.agents[i].needs.fatigue;
                let body = &mut self.agents[i].body;
                health::system_health(
                    &mut body.health,
                    &mut body.energy,
                    &mut self.agent_diseases[i],
                    hunger,
                    fatigue,
                    &health_cfg,
                );
            }
        }

        // ── 17b. §7.5: Epidemic disease spread — proximity-based contagion ──
        // Contagious diseases (Cold, Fever, Epidemic) spread to nearby agents.
        // Uses Manhattan distance ≤ 2 as "close contact" radius.
        {
            let n = self.agents.len();
            let mut new_infections: Vec<(usize, health::DiseaseKind)> = Vec::new();
            for i in 0..n {
                if self.agent_diseases[i].is_empty() {
                    continue;
                }
                for disease in &self.agent_diseases[i] {
                    let rate = disease.kind.transmission_rate();
                    if rate <= Fixed::ZERO {
                        continue; // not contagious
                    }
                    // Check proximity to all other agents
                    for j in 0..n {
                        if i == j || j >= self.agents.len() {
                            continue;
                        }
                        let dist = self.agents[i].position.manhattan_distance(&self.agents[j].position);
                        if dist > 2 {
                            continue; // too far
                        }
                        // Already infected with this kind?
                        let already_infected = self.agent_diseases[j].iter().any(|d| d.kind == disease.kind);
                        if already_infected {
                            continue;
                        }
                        // Probability check — reduced by distance and target health
                        let distance_factor = Fixed::from_f64(1.0) - Fixed::from_f64(dist as f64) * Fixed::from_f64(0.25);
                        let health_factor = self.agents[j].body.health;
                        let transmission_prob = rate * distance_factor * (Fixed::ONE - health_factor * Fixed::from_f64(0.5));
                        let roll = Fixed::from_f64(
                            self.rng.get_mut(RngStream::Social).random::<f64>()
                        );
                        if roll < transmission_prob {
                            new_infections.push((j, disease.kind));
                        }
                    }
                }
            }
            // Apply new infections
            for (agent_idx, kind) in new_infections {
                if agent_idx < self.agent_diseases.len() {
                    self.agent_diseases[agent_idx].push(health::ActiveDisease::new(kind));
                    tracing::debug!(
                        agent = agent_idx,
                        disease = ?kind,
                        "Epidemic: disease transmitted"
                    );
                }
            }
        }

        // ── 17c. §Phase 1.5: Ecology migration pressure ──
        // Agents under high resource scarcity may migrate to a different site.
        {
            let total_grain = self.world.total_food();
            let local_scarcity = (Fixed::ONE - total_grain).clamp_01();
            let season = self.season.current;
            for i in 0..self.agents.len() {
                let has_shelter = self.agents[i].home_site.is_some();
                let pressure = ecology::migration_pressure(local_scarcity, season, has_shelter);
                if pressure > Fixed::from_f64(0.7) {
                    // Agent migrates — teleport to a random site (simplified)
                    if let Some(farm_idx) = self.world.best_farm_for_work() {
                        if farm_idx < self.world.sites.len() {
                            let pos = self.world.site_position(farm_idx)
                                .map_or(crate::sim::Position::new(8, 8), |(x, y)| crate::sim::Position::new(x, y));
                            self.agents[i].position = pos;
                        }
                    }
                }
            }
        }

        // ── 10. Resource spoilage (perishable goods degrade each tick) ──
        // Apply season spoilage modifier
        let spoilage_modifier = self.season.current.spoilage_modifier();
        // Extract resources ref before mutable borrow of sites (split borrowing)
        let resource_defs = &self.world.resources;
        for site in &mut self.world.sites {
            for stock in &mut site.inventory {
                if let Some(res_def) = resource_defs.iter().find(|r| r.id == stock.resource_id) {
                    if res_def.perishable && res_def.spoilage_rate > Fixed::ZERO {
                        let spoilage = stock.quantity * res_def.spoilage_rate * spoilage_modifier;
                        stock.quantity = (stock.quantity - spoilage).max(Fixed::ZERO);
                    }
                }
            }
        }

        // ── 11. Norm evaluation: threats and insults are norm violations ──
        for ev in &self.events[pre_tick_events..].to_vec() {
            if let SimEvent::InteractionOccurred {
                from,
                to,
                kind: mindstrata_core::event::InteractionKind::Threaten
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
                        kind: mindstrata_core::event::InteractionKind::Threaten, ..
                    }
                );
                if is_threat {
                    let from_idx = from_id.as_u64() as usize;
                    let to_idx = to_id.as_u64() as usize;
                    if from_idx < self.agents.len()
                        && to_idx < self.agents.len()
                        && to_idx < self.agent_diseases.len()
                    {
                        let rng_val = Fixed::from_f64(
                            self.rng.get_mut(RngStream::Social).random::<f64>()
                        );
                        let conflict_result = conflict::resolve_conflict(
                            ConflictKind::Threat,
                            &self.agents[from_idx].personality,
                            &self.agents[to_idx].emotions,
                            self.agents[to_idx].body.health,
                            &self.agents[to_idx].conflict,
                            rng_val,
                        );
                        if conflict_result.occurred {
                            // Record conflict state: trauma, combat fatigue
                            self.agents[to_idx].conflict.record_conflict(ConflictKind::Threat);
                            // Apply fear induction
                            self.agents[to_idx].emotions.fear = (
                                self.agents[to_idx].emotions.fear + conflict_result.fear_induced
                            ).clamp_01();
                            // Emit conflict event for observability
                            self.events.push(SimEvent::ConflictOccurred {
                                aggressor: from_id,
                                target: to_id,
                                kind: format!("{:?}", ConflictKind::Threat),
                                injury: conflict_result.injury,
                                fear_induced: conflict_result.fear_induced,
                                tick,
                            });

                            // §19.5.H: Violence escalation — when threat fails to deter
                            // (target fear remains low after threat), aggressive aggressors
                            // escalate to physical violence with real injury.
                            let target_fear_after = self.agents[to_idx].emotions.fear;
                            let threat_failed = target_fear_after < Fixed::from_f64(0.3);
                            let aggressor_aggression = self.agents[from_idx].personality.dominance
                                + self.agents[from_idx].personality.risk_tolerance;
                            let escalate = threat_failed
                                && aggressor_aggression > Fixed::from_f64(1.2)
                                && self.rng.get_mut(RngStream::Social).random::<f64>() < 0.3; // probabilistic

                            if escalate {
                                let violence_result = conflict::resolve_conflict(
                                    ConflictKind::Violence,
                                    &self.agents[from_idx].personality,
                                    &self.agents[to_idx].emotions,
                                    self.agents[to_idx].body.health,
                                    &self.agents[to_idx].conflict,
                                    rng_val,
                                );
                                if violence_result.occurred {
                                    self.agents[to_idx].conflict.record_conflict(ConflictKind::Violence);
                                    self.agents[to_idx].emotions.fear = (
                                        self.agents[to_idx].emotions.fear + violence_result.fear_induced
                                    ).clamp_01();
                                    // Apply injury to target's health
                                    self.agents[to_idx].body.health = (
                                        self.agents[to_idx].body.health - violence_result.injury
                                    ).max(Fixed::ZERO);
                                    // Record injury
                                    self.agents[to_idx].conflict.record_injury();
                                    // Attacker accumulates trauma too
                                    self.agents[from_idx].conflict.record_conflict(ConflictKind::Violence);
                                    // Reduce trust and affection between the two
                                    // §19.5.J: Record relationship trace for provenance
                                    if let Some(rel) = self.relationships.iter_mut()
                                        .find(|r| r.from == from_id && r.to == to_id)
                                    {
                                        let old_trust = rel.trust;
                                        let old_affection = rel.affection;
                                        rel.trust = (rel.trust - Fixed::from_f64(0.3)).max(Fixed::ZERO);
                                        rel.affection = (rel.affection - Fixed::from_f64(0.2)).max(Fixed::ZERO);
                                        self.provenance.record_relationship(crate::provenance::RelationshipTrace {
                                            from: from_id,
                                            to: to_id,
                                            tick: tick_u64,
                                            cause: "violence".into(),
                                            old_trust,
                                            new_trust: rel.trust,
                                            old_affection,
                                            new_affection: rel.affection,
                                            description: format!("Violence destroyed trust ({} -> {})", old_trust.to_f64(), rel.trust.to_f64()),
                                        });
                                    }
                                    if let Some(rel) = self.relationships.iter_mut()
                                        .find(|r| r.from == to_id && r.to == from_id)
                                    {
                                        let old_trust = rel.trust;
                                        let old_affection = rel.affection;
                                        rel.trust = (rel.trust - Fixed::from_f64(0.3)).max(Fixed::ZERO);
                                        rel.affection = (rel.affection - Fixed::from_f64(0.2)).max(Fixed::ZERO);
                                        self.provenance.record_relationship(crate::provenance::RelationshipTrace {
                                            from: to_id,
                                            to: from_id,
                                            tick: tick_u64,
                                            cause: "violence".into(),
                                            old_trust,
                                            new_trust: rel.trust,
                                            old_affection,
                                            new_affection: rel.affection,
                                            description: format!("Fear response — trust dropped ({} -> {})", old_trust.to_f64(), rel.trust.to_f64()),
                                        });
                                    }
                                    // Record in journal
                                    self.journal.record(tick_u64, from_id, JournalEntryKind::CommittedViolence {
                                        target: to_id.as_u64(),
                                        injury: violence_result.injury.to_f64(),
                                    });
                                    // Emit violence event
                                    self.events.push(SimEvent::ConflictOccurred {
                                        aggressor: from_id,
                                        target: to_id,
                                        kind: format!("{:?}", ConflictKind::Violence),
                                        injury: violence_result.injury,
                                        fear_induced: violence_result.fear_induced,
                                        tick,
                                    });
                                    // §19.5.D: Violence is a severe norm violation
                                    let _ = self.norms.check_violation(4, from_id, tick_u64); // norm id=4: No Violence
                                    // §19.5.G: Feud tracking is handled in dedicated section 18

                                    // Attacker gains shame from violence
                                    self.agents[from_idx].emotions.shame = (
                                        self.agents[from_idx].emotions.shame + Fixed::from_f64(0.15)
                                    ).clamp_01();

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
                            self.provenance.record_relationship(crate::provenance::RelationshipTrace {
                                from: to_id,
                                to: from_id,
                                tick: tick_u64,
                                cause: "norm_violation".into(),
                                old_trust,
                                new_trust: rel.trust,
                                old_affection: rel.affection,
                                new_affection: rel.affection,
                                description: format!("Trust eroded by norm violation ({} -> {})", old_trust.to_f64(), rel.trust.to_f64()),
                            });
                        }
                        let from_idx = from_id.as_u64() as usize;
                        if from_idx < self.agents.len() {
                            // §22.1: Moral outrage modulated by moral foundations.
                            // High care/fairness → more shame from own violations.
                            let outrage = self.agents[from_idx].moral_values.moral_outrage(punishment);
                            self.agents[from_idx].emotions.shame = (self.agents[from_idx]
                                .emotions
                                .shame
                                + outrage * Fixed::from_f64(0.10))
                                .clamp_01();
                        }
                        // §12.4: Successful norm enforcement increases institutional legitimacy
                        // Modulated by enforcement_capacity — low-capacity councils gain less.
                        for inst in &mut self.institutions {
                            if inst.kind == institutions::InstitutionKind::Council {
                                let legitimacy_gain = inst.enforcement_capacity * Fixed::from_f64(0.005);
                                inst.increase_legitimacy(legitimacy_gain);
                            }
                        }
                    }
            }
        }

        // ── 11b. §11.2 Gossip propagation with mutation ──
        for ev in &self.events[pre_tick_events..].to_vec() {
            if let SimEvent::InteractionOccurred {
                from,
                to,
                kind: mindstrata_core::event::InteractionKind::Gossip,
                ..
            } = ev
            {
                let from_idx = from.as_u64() as usize;
                let to_idx = to.as_u64() as usize;
                if from_idx < self.agents.len() && to_idx < self.agents.len() {
                    let from_beliefs = &self.agents[from_idx].beliefs;
                    if from_beliefs.is_empty() {
                        continue;
                    }
                    let pick = self.rng.get_mut(RngStream::Social).random_range(0..from_beliefs.len());
                    let source_trust = self.relationships
                        .iter()
                        .find(|r| r.from == *from && r.to == *to)
                        .map_or(Fixed::from_f64(0.5), |r| r.trust);
                    let rumor = gossip::Rumor::from_belief(&from_beliefs[pick], tick_u64);
                    let listener_beliefs = self.agents[to_idx].beliefs.clone();
                    let result = gossip::process_gossip(
                        &rumor,
                        source_trust,
                        &self.agents[from_idx].emotions,
                        &self.agents[from_idx].personality,
                        &self.agents[to_idx].personality,
                        &listener_beliefs,
                        tick_u64,
                    );
                    // §19.5.B: Capture old confidence before gossip update for provenance trace
                    let old_conf = self.agents[to_idx].beliefs.iter()
                        .find(|b| b.proposition_id == result.proposition_id)
                        .map_or(Fixed::from_f64(0.5), |b| b.confidence);
                    gossip::apply_gossip(&mut self.agents[to_idx].beliefs, &result, tick_u64);
                    if result.accepted {
                        // §19.5.B: Record belief update trace for provenance
                        let new_conf = self.agents[to_idx].beliefs.iter()
                            .find(|b| b.proposition_id == result.proposition_id)
                            .map_or(result.mutated_confidence, |b| b.confidence);
                        self.provenance.record_belief_update(
                            crate::provenance::BeliefUpdateTrace {
                                agent: *to,
                                tick: tick_u64,
                                proposition_id: result.proposition_id,
                                old_confidence: old_conf,
                                new_confidence: new_conf,
                                cause: "gossip".into(),
                                delta: (new_conf - old_conf),
                            },
                        );
                        // Emit RumorSpread event for memory encoding
                        self.events.push(SimEvent::RumorSpread {
                            source: *from,
                            target: *to,
                            content_hash: result.proposition_id,
                            distortion: (result.mutated_confidence - rumor.confidence).abs(),
                            tick,
                        });
                    }
                }
            }
        }

        // ── 11c. §19.5.I Knowledge diffusion — cultural knowledge spreads through social networks ──
        for ev in &self.events[pre_tick_events..].to_vec() {
            if let SimEvent::InteractionOccurred {
                from,
                to,
                kind: mindstrata_core::event::InteractionKind::Gossip
                | mindstrata_core::event::InteractionKind::Help
                | mindstrata_core::event::InteractionKind::Trade,
                ..
            } = ev
            {
                let from_idx = from.as_u64() as usize;
                let to_idx = to.as_u64() as usize;
                if from_idx < self.agents.len() && to_idx < self.agents.len() {
                    let source_trust = self.relationships
                        .iter()
                        .find(|r| r.from == *from && r.to == *to)
                        .map_or(Fixed::from_f64(0.5), |r| r.trust);
                    // Teacher teaches a random piece of knowledge they have
                    if !self.agents[from_idx].cultural.knowledge.is_empty() {
                        let pick = self.rng.get_mut(RngStream::Social)
                            .random_range(0..self.agents[from_idx].cultural.knowledge.len());
                        let knowledge_id = self.agents[from_idx].cultural.knowledge[pick];
                        // Simple diffusion: probabilistic transfer based on trust and openness
                        let trust_factor = source_trust;
                        let openness = self.agents[to_idx].cultural.openness;
                        let acceptance = (trust_factor * Fixed::from_f64(0.5)
                            + openness * Fixed::from_f64(0.5)).clamp_01();
                        if !self.agents[to_idx].cultural.knowledge.contains(&knowledge_id)
                            && acceptance > Fixed::from_f64(0.5)
                        {
                            self.agents[to_idx].cultural.knowledge.push(knowledge_id);
                            // §19.5.I: Update holder count in knowledge store
                            if let Some(k) = self.knowledge_store.iter_mut().find(|k| k.id == knowledge_id) {
                                k.holders += 1;
                            }
                            // Emit knowledge transfer event for observability
                            self.events.push(SimEvent::KnowledgeTransferred {
                                source: *from,
                                target: *to,
                                knowledge_id,
                                tick,
                            });
                            // §19.5.B: Record knowledge transfer in provenance
                            self.provenance.record_institutional(
                                crate::provenance::InstitutionalTrace {
                                    institution_name: "Community".into(),
                                    tick: tick_u64,
                                    decision_kind: "knowledge_transfer".into(),
                                    description: format!("Agent {} taught knowledge {} to Agent {}", from.as_u64(), knowledge_id, to.as_u64()),
                                    affected: vec![*from, *to],
                                    success: true,
                                },
                            );
                        }
                    }
                }
            }
        }

        // ── 11d. §19.5.F Childhood socialization — children learn from parents ──
        {
            let mut socialization_updates: Vec<(usize, u64)> = Vec::new();
            for i in 0..self.agents.len() {
                let a = &self.agents[i];
                if a.age >= Fixed::from_f64(18.0) {
                    continue;
                }
                // Pick the first available parent to learn from
                let parent_idx = a.parent_a.or(a.parent_b);
                if let Some(pi) = parent_idx {
                    if pi < self.agents.len() && !self.agents[pi].cultural.knowledge.is_empty() {
                        let pick = self.rng.get_mut(RngStream::Social)
                            .random_range(0..self.agents[pi].cultural.knowledge.len());
                        let knowledge_id = self.agents[pi].cultural.knowledge[pick];
                        if !self.agents[i].cultural.knowledge.contains(&knowledge_id) {
                            let acceptance = (self.agents[i].cultural.openness * Fixed::from_f64(0.6)
                                + Fixed::from_f64(0.4)).clamp_01();
                            if acceptance > Fixed::from_f64(0.3) {
                                socialization_updates.push((i, knowledge_id));
                            }
                        }
                    }
                }
            }
            for (ci, knowledge_id) in socialization_updates {
                self.agents[ci].cultural.knowledge.push(knowledge_id);
                if let Some(k) = self.knowledge_store.iter_mut().find(|k| k.id == knowledge_id) {
                    k.holders += 1;
                }
                // §19.5.F: Record socialization in journal for observability
                self.journal.record(tick_u64, AgentId::new(ci as u64),
                    JournalEntryKind::KnowledgeSocialized { knowledge_id });
            }
        }

        // ── 11e. §19.5.I Innovation — agents discover new knowledge through work ──
        {
            let work_agents: Vec<usize> = self.agents.iter().enumerate()
                .filter(|(_, a)| matches!(a.current_action, crate::actions::ActionKind::Work))
                .map(|(i, _)| i)
                .collect();
            let mut innovations: Vec<(usize, u64, String)> = Vec::new();
            for idx in work_agents {
                // Discovery chance based on openness and conscientiousness
                let discovery_chance = (self.agents[idx].personality.openness * Fixed::from_f64(0.3)
                    + self.agents[idx].personality.conscientiousness * Fixed::from_f64(0.2)
                    + Fixed::from_f64(0.01)).clamp_01();
                let roll = Fixed::from_f64(
                    self.rng.get_mut(RngStream::Social).random::<f64>()
                );
                if roll < discovery_chance {
                    // Try to discover a knowledge item not yet known by this agent
                    for k in &self.knowledge_store {
                        if !self.agents[idx].cultural.knowledge.contains(&k.id)
                            && k.difficulty < self.agents[idx].personality.openness
                        {
                            innovations.push((idx, k.id, k.name.clone()));
                            break; // one discovery per tick per agent
                        }
                    }
                }
            }
            for (agent_idx, knowledge_id, name) in innovations {
                self.agents[agent_idx].cultural.knowledge.push(knowledge_id);
                if let Some(ks) = self.knowledge_store.iter_mut().find(|ks| ks.id == knowledge_id) {
                    ks.holders += 1;
                }
                self.journal.record(tick_u64, AgentId::new(agent_idx as u64),
                    JournalEntryKind::KnowledgeDiscovered {
                        knowledge_id,
                        name,
                    });
                self.events.push(SimEvent::KnowledgeTransferred {
                    source: AgentId::new(agent_idx as u64),
                    target: AgentId::new(agent_idx as u64),
                    knowledge_id,
                    tick,
                });
            }
        }

        // ── 12. Institutional collective psychology derivation ────
        // §23: Derive collective psychology from member states each tick.
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
            // Slow legitimacy decay without reinforcement
            institution.decay_legitimacy(Fixed::from_f64(0.0001));

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
            let has_fine_theft = institution.active_policies.iter()
                .any(|p| p.name == "Fine Theft Policy");
            if has_fine_theft && tick_u64.is_multiple_of(institutions::POLICY_RECORD_INTERVAL) {
                let members = institution.members.clone();
                institution.record_action(
                    tick_u64,
                    "Fine Theft Policy enacted".into(),
                    members.clone(),
                    true,
                );
                // §19.5.B: Record institutional provenance trace
                self.provenance.record_institutional(
                    crate::provenance::InstitutionalTrace {
                        institution_name: institution.kind.name().into(),
                        tick: tick_u64,
                        decision_kind: "policy_enacted".into(),
                        description: "Fine Theft Policy enacted".into(),
                        affected: members,
                        success: true,
                    },
                );
            }

            // Trim old records to prevent unbounded growth (keep last 1000)
            if institution.records.len() > institutions::MAX_RECORDS {
                let drain_count = institution.records.len() - institutions::MAX_RECORDS;
                institution.records.drain(..drain_count);
            }

            // §19.5.C: Institutional tax collection — all institutions collect taxes
            if tick_u64.is_multiple_of(institutions::TAX_COLLECTION_INTERVAL) && tick_u64 > 0 && !institution.members.is_empty() {
                let tax_rate = match institution.kind {
                    institutions::InstitutionKind::Council => Fixed::from_f64(institutions::COUNCIL_TAX_RATE),
                    institutions::InstitutionKind::Market => Fixed::from_f64(institutions::MARKET_FEE_RATE),
                    institutions::InstitutionKind::Temple => Fixed::from_f64(institutions::TEMPLE_TITHE_RATE),
                    _ => Fixed::ZERO,
                };
                if tax_rate > Fixed::ZERO {
                    let mut member_wealth: Vec<(AgentId, Fixed)> = institution.members.iter()
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
                        institution.record_action(tick_u64, action_name.clone(), members.clone(), true);
                        // §19.5.B: Record institutional provenance trace for tax collection
                        self.provenance.record_institutional(
                            crate::provenance::InstitutionalTrace {
                                institution_name: inst_name,
                                tick: tick_u64,
                                decision_kind: "tax_collection".into(),
                                description: action_name,
                                affected: members,
                                success: true,
                            },
                        );
                    }
                }
            }

            // §19.5.C: All institutions pay role holders wages periodically
            if tick_u64.is_multiple_of(institutions::WAGE_PAYMENT_INTERVAL) && tick_u64 > 0 {
                let wage = Fixed::from_f64(institutions::BASE_WAGE);
                let role_holder_count = institution.roles.iter().filter(|r| r.holder.is_some()).count() as i64;
                let total_wage_cost = wage * Fixed::from_int(role_holder_count);
                if institution.treasury >= total_wage_cost && role_holder_count > 0 {
                    let mut member_wealth: Vec<(AgentId, Fixed)> = institution.roles.iter()
                        .filter_map(|r| r.holder.map(|h| {
                            let idx = h.as_u64() as usize;
                            let wealth = if idx < self.agents.len() {
                                self.agents[idx].wealth.coin
                            } else {
                                Fixed::ZERO
                            };
                            (h, wealth)
                        }))
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
                        let role_names: Vec<&str> = institution.roles.iter()
                            .filter(|r| r.holder.is_some())
                            .map(|r| r.name.as_str())
                            .collect();
                        let kind_name = institution.kind.name().to_string();
                        let wage_msg = format!("{} wage payment: {:.1} coins to {}", kind_name, paid.to_f64(), role_names.join(", "));
                        institution.record_action(tick_u64, wage_msg.clone(), members.clone(), true);
                        // §19.5.B: Record institutional provenance trace for wage payment
                        self.provenance.record_institutional(
                            crate::provenance::InstitutionalTrace {
                                institution_name: kind_name,
                                tick: tick_u64,
                                decision_kind: "wage_payment".into(),
                                description: wage_msg,
                                affected: members,
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
            if tick_u64.is_multiple_of(institutions::TAX_COLLECTION_INTERVAL) && tick_u64 > 0 && !institution.members.is_empty() {
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
                            let tax_fatigue = self.agents[idx].wealth.coin * Fixed::from_f64(tax_rate);
                            if tax_fatigue > Fixed::from_f64(1.0) {
                                // Heavy taxation → anger
                                self.agents[idx].emotions.anger = (
                                    self.agents[idx].emotions.anger + tax_fatigue * Fixed::from_f64(0.01)
                                ).clamp_01();
                            }
                        }
                    }
                }
            }
        }        // ── 15. Faction dynamics — grievance, formation, recruitment, protests (Phase 8) ──
        // §29.2: Factions emerge from collective grievance. §Phase 8: legitimacy crisis → rebellion or reform.
        {
            let faction_count = self.institutions.iter()
                .filter(|i| i.kind == InstitutionKind::Faction)
                .count();

            // Compute grievance for each agent
            let market_gini = self.market.inequality;
            let grievances: Vec<(AgentId, Fixed)> = self.agents.iter().enumerate()
                .map(|(i, agent)| {
                    let g = factions::compute_grievance(
                        agent.derived.resentment,
                        agent.emotions.fear,
                        agent.emotions.anger,
                        agent.needs.autonomy,
                        agent.needs.meaning,
                        agent.moral_values.fairness,
                        market_gini,
                    );
                    (AgentId::new(i as u64), g)
                })
                .collect();

            // Find council legitimacy (average across councils)
            let avg_council_legitimacy: Fixed = {
                let councils: Vec<_> = self.institutions.iter()
                    .filter(|i| i.kind == InstitutionKind::Council)
                    .collect();
                if councils.is_empty() {
                    Fixed::from_f64(0.5)
                } else {
                    let sum: Fixed = councils.iter().map(|c| c.legitimacy).fold(Fixed::ZERO, |a, b| a + b);
                    sum / Fixed::from_int(councils.len() as i64)
                }
            };

            // Faction formation: when grievance is high and council legitimacy is low
            let should_form = faction_count < factions::MAX_FACTIONS
                && avg_council_legitimacy < Fixed::from_f64(0.5)
                && tick_u64 > factions::FORMATION_COOLDOWN;

            if should_form {
                let recruitable = factions::find_recruitable_agents(
                    &grievances, factions::RECRUITMENT_GRIEVANCE_THRESHOLD,
                );

                let avg_grievance = if recruitable.is_empty() {
                    Fixed::ZERO
                } else {
                    let sum: Fixed = recruitable.iter().map(|(_, g)| *g).fold(Fixed::ZERO, |a, b| a + b);
                    sum / Fixed::from_int(recruitable.len() as i64)
                };

                if avg_grievance >= factions::FORMATION_GRIEVANCE_THRESHOLD && recruitable.len() >= 2 {
                    // Find leader candidates: high ambition, high dominance, low anger
                    let leader_candidates: Vec<_> = recruitable.iter()
                        .filter_map(|(id, _)| {
                            let idx = id.as_u64() as usize;
                            if idx < self.agents.len() {
                                let a = &self.agents[idx];
                                if a.personality.ambition >= factions::LEADER_AMBITION_THRESHOLD {
                                    Some((*id, a.personality.ambition, a.personality.dominance,
                                          a.personality.extraversion, a.emotions.anger))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();

                    if let Some(leader) = factions::select_leader(&leader_candidates) {
                        let members: Vec<AgentId> = recruitable.iter()
                            .map(|(id, _)| *id)
                            .collect();

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
                        self.provenance.record_decision(crate::provenance::DecisionTrace {
                            agent: leader,
                            tick: tick_u64,
                            action_name: "FactionFormation".into(),
                            factors: vec![
                                crate::provenance::DecisionFactor {
                                    kind: "grievance".into(),
                                    magnitude: avg_grievance,
                                    description: format!("Avg grievance: {:.2}", avg_grievance.to_f64()),
                                },
                                crate::provenance::DecisionFactor {
                                    kind: "council_legitimacy".into(),
                                    magnitude: avg_council_legitimacy,
                                    description: format!("Council legitimacy: {:.2}", avg_council_legitimacy.to_f64()),
                                },
                            ],
                            from_routine: false,
                            interrupted_by_critical_needs: false,
                            intention_abandoned: false,
                        });

                        self.institutions.push(new_faction);
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
                    let council_enforcement = self.institutions.iter()
                        .filter(|i| i.kind == InstitutionKind::Council)
                        .map(|i| i.enforcement_capacity)
                        .fold(Fixed::ZERO, std::cmp::Ord::max);

                    let (suppressed, legitimacy_effect) = factions::council_response(
                        council_enforcement, protest_size, total_pop,
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
                            faction.collective.morale = (faction.collective.morale + Fixed::from_f64(0.05)).clamp_01();
                        }
                    }

                    // Record in provenance (use leader of the protesting faction)
                    let protest_agent = self.institutions[inst_idx].get_role_holder("Leader")
                        .unwrap_or(AgentId::new(0));
                    self.provenance.record_decision(crate::provenance::DecisionTrace {
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

        // ── 14. §7.2: Moral panic detection — rumor cascades collapse institutional trust ──
        // Check if gossip about institutions has accumulated enough emotional charge
        // to trigger a moral panic. This happens when many agents hold high-emotion
        // beliefs about an institution, causing sudden legitimacy collapse.
        {
            // Check propositions 0 ("the_market_is_fair") and 1 ("the_council_protects_us")
            let belief_refs: Vec<&Vec<Belief>> = self.agents.iter().map(|a| &a.beliefs).collect();
            for prop_id in 0..=1 {
                let panic_result = gossip::detect_moral_panic(&belief_refs, prop_id);
                if panic_result.triggered {
                    tracing::warn!(
                        proposition = prop_id,
                        avg_charge = panic_result.avg_charge.to_f64(),
                        tick = tick_u64,
                        "§7.2: Moral panic triggered!"
                    );
                    // Apply legitimacy damage to all matching institutions
                    for inst in &mut self.institutions {
                        let inst_prop = match inst.kind {
                            InstitutionKind::Council => Some(1u64),  // council → proposition 1
                            InstitutionKind::Market => Some(0u64),   // market → proposition 0
                            _ => None,
                        };
                        if inst_prop == Some(prop_id) {
                            inst.legitimacy = (inst.legitimacy - panic_result.legitimacy_damage).max(Fixed::ZERO);
                        }
                    }
                    // Boost faction grievance from panic
                    for inst in &mut self.institutions {
                        if inst.kind == InstitutionKind::Faction {
                            inst.collective.morale = (inst.collective.morale + panic_result.grievance_boost).clamp_01();
                        }
                    }
                    self.events.push(SimEvent::ConflictOccurred {
                        aggressor: AgentId::new(0),
                        target: AgentId::new(0),
                        kind: "MoralPanic".into(),
                        injury: Fixed::ZERO,
                        fear_induced: panic_result.avg_charge,
                        tick,
                    });
                }
            }
        }

        // ── 14b. §7.3: Revolution mechanism — faction seizes control ──
        // When a faction has enough grievance, membership, and the council's
        // legitimacy is critically low, the faction stages a revolution.
        {
            let council_legitimacy: Fixed = self.institutions.iter()
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

                // Revolution score: weighted combination of grievance, size, and council weakness
                let revolution_score = faction_grievance * factions::REVOLUTION_GRIEVANCE_WEIGHT
                    + Fixed::from_int(faction_size as i64) / Fixed::from_int(total_pop.max(1) as i64) * factions::REVOLUTION_SIZE_WEIGHT
                    + (Fixed::ONE - council_legitimacy) * factions::REVOLUTION_LEGITIMACY_WEIGHT;

                let revolution_cooldown_ok = tick_u64.saturating_sub(self.last_revolution_tick) >= factions::REVOLUTION_COOLDOWN;
                if revolution_score >= factions::REVOLUTION_SCORE_THRESHOLD
                    && tick_u64 > factions::REVOLUTION_MIN_TICK
                    && revolution_cooldown_ok
                {
                    tracing::warn!(
                        faction = inst_idx,
                        score = revolution_score.to_f64(),
                        council_legitimacy = council_legitimacy.to_f64(),
                        tick = tick_u64,
                        "§7.3: Revolution! Faction seizes control"
                    );
                    // Council loses all legitimacy
                    for inst in &mut self.institutions {
                        if inst.kind == InstitutionKind::Council {
                            inst.legitimacy = Fixed::ZERO;
                            inst.collective.morale = Fixed::from_f64(0.1);
                        }
                    }
                    // Faction gains legitimacy
                    self.institutions[inst_idx].legitimacy = Fixed::from_f64(0.6);
                    self.institutions[inst_idx].collective.morale = Fixed::from_f64(0.8);
                    // Track revolution tick for cooldown
                    self.last_revolution_tick = tick_u64;
                    // Record revolution event
                    self.events.push(SimEvent::ConflictOccurred {
                        aggressor: self.institutions[inst_idx].get_role_holder("Leader").unwrap_or(AgentId::new(0)),
                        target: AgentId::new(0),
                        kind: "Revolution".into(),
                        injury: Fixed::ZERO,
                        fear_induced: Fixed::from_f64(0.5),
                        tick,
                    });
                    break; // only one revolution per tick
                }
            }
        }

        // ── 15. Derived mental state computation (§22) ─────────────
        for agent in &mut self.agents {
            let stress = agent.emotions.fear + agent.emotions.anger;
            let need_deficit_avg = (agent.needs.hunger + agent.needs.thirst + agent.needs.fatigue + agent.needs.safety) * Fixed::from_f64(0.25);
            let social_support = Fixed::ONE - agent.needs.social;
            let success_rate = if agent.recent_attempts > 0 {
                Fixed::from_int(agent.recent_successes as i64) / Fixed::from_int(agent.recent_attempts as i64)
            } else {
                Fixed::from_f64(0.5)
            };
            let justice_perception = agent.moral_values.fairness;
            agent.derived.compute(&crate::person::MentalStateInput {
                stress,
                coping_potential: agent.personality.conscientiousness,
                social_support,
                need_deficit_avg,
                meaning: agent.needs.meaning,
                autonomy: agent.needs.autonomy,
                success_rate,
                neuroticism: agent.personality.neuroticism,
                justice_perception,
            });
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
                    let trust = self.relationships
                        .iter()
                        .find(|r| r.from == *from && r.to == *to)
                        .map_or(Fixed::from_f64(0.5), |r| r.trust);

                    let evidence_strength = trust - Fixed::from_f64(0.5);
                    let source_trust = Fixed::from_f64(0.6);

                    belief_update::update_beliefs(
                        &mut self.agents[from_idx].beliefs,
                        &[(2, evidence_strength, source_trust)],
                        Fixed::ZERO,
                        Fixed::ZERO,
                        tick_u64,
                    );
                }
            }
        }

        // ── 18. §19.5.G Feud tracking — repeated violence creates persistent feuds ──
        for ev in &self.events[pre_tick_events..].to_vec() {
            if let SimEvent::ConflictOccurred {
                aggressor,
                target,
                kind,
                ..
            } = ev
            {
                if *kind == format!("{:?}", ConflictKind::Violence) {
                    let a_idx = aggressor.as_u64() as usize;
                    let t_idx = target.as_u64() as usize;
                    if a_idx < self.agents.len() && t_idx < self.agents.len() {
                        let was_new = !self.agents[a_idx].feuds.contains(&t_idx);
                        if was_new {
                            self.agents[a_idx].feuds.push(t_idx);
                            self.agents[a_idx].feud_ticks.push(tick_u64);
                            self.agents[t_idx].feuds.push(a_idx);
                            self.agents[t_idx].feud_ticks.push(tick_u64);
                            self.events.push(SimEvent::FeudFormed {
                                party_a: *aggressor,
                                party_b: *target,
                                tick,
                            });
                        }
                    }
                }
            }
        }

        // ── 19. §19.5.F Marriage formation — compatible age + high affection ──
        {
            let n = self.agents.len();
            let mut new_marriages: Vec<(usize, usize)> = Vec::new();

            for i in 0..n {
                if self.agents[i].partner.is_some() {
                    continue; // already partnered
                }
                if self.agents[i].age < Fixed::from_f64(18.0) {
                    continue; // too young
                }

                // Find candidate: compatible age, high affection, no partner
                for j in (i + 1)..n {
                    if self.agents[j].partner.is_some() {
                        continue;
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
                    let affection = self.relationships.iter()
                        .find(|r| r.from == AgentId::new(i as u64) && r.to == AgentId::new(j as u64))
                        .map_or(Fixed::ZERO, |r| r.affection);
                    // Architecture-plan-2 §10.4: Compute attraction score.
                    // Personality compatibility (agreeableness similarity), physical proximity,
                    // and social approval feed into the AttractionModel.
                    let personality_compat = Fixed::ONE - (self.agents[i].personality.agreeableness
                        - self.agents[j].personality.agreeableness).abs();
                    let pos_i = self.agents[i].position;
                    let pos_j = self.agents[j].position;
                    let distance = Fixed::from_int(pos_i.manhattan_distance(&pos_j) as i64);
                    // Architecture-plan-2 §10.4: Max distance for marriage proximity scoring.
                    const MAX_MARRIAGE_DISTANCE: f64 = 20.0;
                    let proximity = (Fixed::ONE - distance / Fixed::from_f64(MAX_MARRIAGE_DISTANCE)).max(Fixed::ZERO);
                    let mut att = crate::attraction::AttractionModel::default();
                    att.personality_attraction = personality_compat;
                    att.familiarity = affection;
                    att.physical_attraction = proximity;
                    att.reciprocity = affection; // reciprocity from mutual affection
                    let attraction_score = att.total_attraction();
                    // Marriage probability: attraction * health * trust
                    let health = (self.agents[i].body.health + self.agents[j].body.health) * Fixed::from_f64(0.5);
                    let trust = self.relationships.iter()
                        .find(|r| r.from == AgentId::new(i as u64) && r.to == AgentId::new(j as u64))
                        .map_or(Fixed::ZERO, |r| r.trust);
                    let marriage_chance = attraction_score * health * trust * Fixed::from_f64(0.001);
                    let rng_val = Fixed::from_f64(
                        self.rng.get_mut(RngStream::Social).random::<f64>()
                    );
                    if rng_val < marriage_chance {
                        new_marriages.push((i, j));
                        break; // only one marriage per agent per tick
                    }
                }
            }

            for (a, b) in new_marriages {
                self.agents[a].partner = Some(b);
                self.agents[b].partner = Some(a);
                self.events.push(SimEvent::MarriageFormed {
                    spouse_a: AgentId::new(a as u64),
                    spouse_b: AgentId::new(b as u64),
                    tick,
                });
                // Marriage boosts trust and affection
                // §19.5.J: Record marriage relationship traces
                if let Some(rel) = self.relationships.iter_mut()
                    .find(|r| r.from == AgentId::new(a as u64) && r.to == AgentId::new(b as u64))
                {
                    let old_trust = rel.trust;
                    let old_affection = rel.affection;
                    rel.trust = (rel.trust + Fixed::from_f64(0.2)).clamp_01();
                    rel.affection = (rel.affection + Fixed::from_f64(0.3)).clamp_01();
                    self.provenance.record_relationship(crate::provenance::RelationshipTrace {
                        from: AgentId::new(a as u64),
                        to: AgentId::new(b as u64),
                        tick: tick_u64,
                        cause: "marriage".into(),
                        old_trust,
                        new_trust: rel.trust,
                        old_affection,
                        new_affection: rel.affection,
                        description: format!("Marriage bond formed (trust {} -> {}, affection {} -> {})", old_trust.to_f64(), rel.trust.to_f64(), old_affection.to_f64(), rel.affection.to_f64()),
                    });
                }
                if let Some(rel) = self.relationships.iter_mut()
                    .find(|r| r.from == AgentId::new(b as u64) && r.to == AgentId::new(a as u64))
                {
                    let old_trust = rel.trust;
                    let old_affection = rel.affection;
                    rel.trust = (rel.trust + Fixed::from_f64(0.2)).clamp_01();
                    rel.affection = (rel.affection + Fixed::from_f64(0.3)).clamp_01();
                    self.provenance.record_relationship(crate::provenance::RelationshipTrace {
                        from: AgentId::new(b as u64),
                        to: AgentId::new(a as u64),
                        tick: tick_u64,
                        cause: "marriage".into(),
                        old_trust,
                        new_trust: rel.trust,
                        old_affection,
                        new_affection: rel.affection,
                        description: format!("Marriage bond formed (trust {} -> {}, affection {} -> {})", old_trust.to_f64(), rel.trust.to_f64(), old_affection.to_f64(), rel.affection.to_f64()),
                    });
                }
                tracing::info!(
                    spouse_a = a, spouse_b = b, tick = tick_u64,
                    "Marriage formed"
                );
            }
        }

        // ── 19b. §19.5.F Birth mechanics — new agents from partnered couples ──
        {
            let n = self.agents.len();
            let mut new_births: Vec<(usize, usize, usize)> = Vec::new(); // (parent_a, parent_b, child_idx)

            for i in 0..n {
                if let Some(partner_idx) = self.agents[i].partner {
                    if partner_idx <= i {
                        continue; // only process each couple once
                    }
                    let age_a = self.agents[i].age.to_f64();
                    let age_b = self.agents[partner_idx].age.to_f64();
                    let avg_age = (age_a + age_b) * 0.5;
                    let health_a = self.agents[i].body.health;
                    let health_b = self.agents[partner_idx].body.health;
                    let min_health = health_a.min(health_b);
                    let rng_val = Fixed::from_f64(
                        self.rng.get_mut(RngStream::Social).random::<f64>()
                    );
                    // Count existing children for this specific couple
                    let existing_children = self.agents.iter().filter(|a| {
                        (a.parent_a == Some(i) && a.parent_b == Some(partner_idx))
                            || (a.parent_a == Some(partner_idx) && a.parent_b == Some(i))
                    }).count();
                    let should = demography::should_birth(
                        true, min_health, avg_age, existing_children,
                        &self.demography_config, rng_val,
                    );
                    if should && n + new_births.len() < crate::population_cap::MAX_POPULATION {
                        new_births.push((i, partner_idx, n + new_births.len()));
                    }
                }
            }

            for (parent_a, parent_b, child_idx) in new_births {
                let child_name = format!("Child_{child_idx}");
                // Inherit personality traits with noise
                let mut child_rng = rand_chacha::ChaCha8Rng::seed_from_u64(
                    self.config.seed.wrapping_add(tick_u64).wrapping_add(child_idx as u64)
                );
                let child_personality = Personality::random(&mut child_rng);
                let child_age = Fixed::from_f64(0.0); // newborn

                // Allocate home site
                let home_site = self.agents[parent_a].home_site;

                let agent_id = AgentId::new(child_idx as u64);

                let child_embodied = EmbodiedState::random(child_age, &mut child_rng);
                let child_neuroticism = child_personality.neuroticism;
                let child_openness = child_personality.openness;
                self.agents.push(AgentBundle {
                    body: BodyState::from(&child_embodied),
                    needs: NeedState::default(),
                    personality: child_personality,
                    goals: Vec::new(),
                    beliefs: Vec::new(),
                    affect: Affect::default(),
                    emotions: DiscreteEmotions::default(),
                    current_action: ActionKind::Idle,
                    action_progress: 0,
                    name: child_name,
                    position: self.agents[parent_a].position,
                    home_site,
                    memory: MemoryStore::new(200),
                    identity: IdentityState {
                        identities: vec![],
                    },
                    attention: AttentionState::default(),
                    intention: None,
                    routine: DailyRoutine::village_routine(),
                    moral_values: MoralValues::random(&mut child_rng),
                    cognitive: CognitiveState::default(),
                    derived: DerivedMentalState::default(),
                    recent_successes: 0,
                    recent_attempts: 0,
                    age: child_age,
                    wealth: WealthState { coin: Fixed::from_f64(1.0) },
                    conflict: ConflictState::default(),
                    cultural: CulturalState::default(),
                    partner: None,
                    parent_a: Some(parent_a),
                    parent_b: Some(parent_b),
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
                    attachment: crate::psychology::AttachmentSystem::default(),
                    emotion_regulation: crate::psychology::EmotionRegulationState::default(),
                    moral_cognition: crate::psychology::MoralCognition::default(),
                    prospection: crate::psychology::ProspectionState::default(),
                    narrative: crate::psychology::NarrativeIdentity::default(),
                    developmental: crate::psychology::DevelopmentalPsychState::default(),
                    psychopathology: crate::psychology::PsychopathologyState::default(),
                    psych_skills: crate::psychology::SkillState::default(),
                    relationship_v2s: Vec::new(),
                    attraction: crate::attraction::AttractionModel::default(),
                    status_v2: crate::status_dims::StatusDimensions::default(),
                });                // Add relationships to all existing agents
                for existing_idx in 0..self.agents.len() - 1 {
                    let trust = if existing_idx == parent_a || existing_idx == parent_b {
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

                // Disease tracking for new agent
                self.agent_diseases.push(Vec::new());

                self.events.push(SimEvent::ChildBorn {
                    child: agent_id,
                    parent_a: AgentId::new(parent_a as u64),
                    parent_b: AgentId::new(parent_b as u64),
                    tick,
                });

                tracing::info!(
                    child = child_idx,
                    parent_a = parent_a,
                    parent_b = parent_b,
                    tick = tick_u64,
                    "Child born"
                );
            }
        }

        // ── 20. Conflict state update — trauma decay, combat fatigue, feud decay ──
        for agent in &mut self.agents {
            agent.conflict.update();
            // §19.5.G: Feud decay — remove feuds older than 500 ticks
            let feud_decay_threshold = tick_u64.saturating_sub(500);
            let mut keep = 0;
            for j in 0..agent.feuds.len() {
                if agent.feud_ticks[j] > feud_decay_threshold {
                    agent.feuds[keep] = agent.feuds[j];
                    agent.feud_ticks[keep] = agent.feud_ticks[j];
                    keep += 1;
                }
            }
            agent.feuds.truncate(keep);
            agent.feud_ticks.truncate(keep);
        }

        // ── 21. §19.5.E Carrying cost — heavy loads increase fatigue ──
        // Agents who performed physical actions (Work) incur carrying cost based on load.
        // Since agents don't carry items yet, this serves as a workload fatigue modifier.
        for i in 0..self.agents.len() {
            if self.agents[i].current_action == ActionKind::Work {
                let load_ratio = Fixed::from_f64(0.4); // moderate load for work
                let fatigue_cost = logistics::carrying_cost(
                    load_ratio,
                    Fixed::from_f64(1.0),
                    self.agents[i].needs.fatigue,
                );
                self.agents[i].needs.fatigue = (
                    self.agents[i].needs.fatigue + fatigue_cost
                ).clamp_01();
            }
        }

        // ── 19. Market system: price formation, inequality tracking ──
        {
            let agent_economic_state: Vec<_> = self.agents.iter()
                .map(|a| (a.needs.clone(), a.wealth.clone()))
                .collect();
            market::system_market(&self.world, &agent_economic_state, &mut self.market, tick_u64);

            // §4.4: Update black market state based on current scarcity and enforcement
            // Scarcity is normalized: 0 = abundant, 1 = critically scarce
            let total_grain = self.world.total_food();
            let expected_capacity = Fixed::from_int(self.agents.len() as i64) * Fixed::from_int(EXPECTED_GRAIN_PER_AGENT as i64);
            let scarcity = if expected_capacity > Fixed::ZERO {
                (Fixed::ONE - total_grain / expected_capacity).clamp_01()
            } else {
                Fixed::ZERO
            };
            let council_enforcement = self.institutions.iter()
                .filter(|i| i.kind == InstitutionKind::Council)
                .map(|i| i.enforcement_capacity)
                .fold(Fixed::ZERO, std::cmp::Ord::max);
            self.black_market.update(scarcity, council_enforcement);
        }

        // ── 18. Demography: aging and mortality ──
        if tick_u64.is_multiple_of(10) { // process demography every 10 ticks to reduce overhead
            let mut deaths = Vec::new();
            for i in 0..self.agents.len() {
                let (new_age, died) = demography::age_agent(
                    self.agents[i].age,
                    self.agents[i].body.health,
                    self.agents[i].body.energy,
                    &self.demography_config,
                );
                self.agents[i].age = new_age;
                if died {
                    deaths.push(i);
                }
            }
            // Remove dead agents (in reverse order to preserve indices)
            for &idx in deaths.iter().rev() {
                let agent_id = AgentId::new(idx as u64);
                tracing::warn!(
                    agent = self.agents[idx].name.as_str(),
                    age = self.agents[idx].age.to_f64(),
                    tick = tick_u64,
                    "Agent died"
                );                    // §19.5.F: Inheritance — transfer wealth to spouse and children
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
                    self.journal.record(tick_u64, agent_id, JournalEntryKind::Inheritance {
                        heir_count: heirs.len() as u64,
                        amount: inherited_wealth.to_f64(),
                    });
                }
                self.agents.remove(idx);
                self.agent_diseases.remove(idx);
                // Remove from relationships
                self.relationships.retain(|r| r.from != agent_id && r.to != agent_id);
                // Remove from institutions
                for inst in &mut self.institutions {
                    inst.remove_member(agent_id);
                }
            }
        }

        // §19.5.J: Trim provenance traces to prevent unbounded growth
        self.provenance.trim(institutions::MAX_PROVENANCE_RECORDS);

        // ── 19. §6.5: Record metric history for observability ──
        // Every 10 ticks, snapshot key metrics for CSV export and analysis.
        // Cap at MAX_METRIC_HISTORY entries to prevent unbounded growth.
        if tick_u64.is_multiple_of(10) {
            let snapshot = self.metrics_snapshot();
            self.metric_history.push(snapshot);
            if self.metric_history.len() > MAX_METRIC_HISTORY {
                self.metric_history.remove(0);
            }
        }
    }

    /// Access the current tick.
    pub fn current_tick(&self) -> Tick {
        self.clock.tick()
    }

    /// Number of agents.
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Total events generated.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Get a reference to the world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get a reference to all relationships.
    pub fn relationships(&self) -> &[Relationship] {
        &self.relationships
    }

    /// Get recent events (last n).
    pub fn recent_events(&self, n: usize) -> &[SimEvent] {
        let start = self.events.len().saturating_sub(n);
        &self.events[start..]
    }

    /// §19.5.D: Detect and punish theft — consumes resource, fines agent,
    /// records norm violation with enforcement probability, emits event, reduces trust with owner.
    /// Returns (taken > 0, was_caught): theft succeeded and whether it was detected.
    fn enforce_theft(&mut self, agent_idx: usize, agent_id: AgentId, site_idx: usize, resource_id: u64, amount: Fixed, tick_u64: u64, tick: Tick) -> bool {
        let owner = self.world.sites[site_idx].owner;
        let taken = self.world.consume_resource(site_idx, resource_id, amount);
        // Early return if nothing was actually taken — don't run enforcement for zero-resource thefts
        if taken <= Fixed::ZERO {
            return false;
        }
        let resource_name = if resource_id == GRAIN_RESOURCE_ID { "grain" } else { "water" };

        // §19.5.D: Compute enforcement capacity from Council's enforcement capacity
        let enforcement = self.institutions.iter()
            .filter(|i| i.kind == crate::institutions::InstitutionKind::Council)
            .map(|i| i.enforcement_capacity)
            .fold(Fixed::ZERO, |a, b| a + b)
            .min(Fixed::ONE);

        // §4.4: Black market reduces detection probability for qualifying agents
        let agent_can_black_market = self.black_market.can_participate(&self.agents[agent_idx].personality);
        let effective_enforcement = if agent_can_black_market {
            (enforcement * Fixed::from_f64(0.5)).max(Fixed::ZERO) // black market halves enforcement
        } else {
            enforcement
        };

        // Generate detection roll
        let detection_roll = Fixed::from_f64(
            self.rng.get_mut(RngStream::Social).random::<f64>()
        );

        // §19.5.D: Apply enforcement probability — not all thefts are caught
        let (_punishment, was_caught) = self.norms.check_violation_with_enforcement(
            0, agent_id, tick_u64, effective_enforcement, detection_roll
        );

        if was_caught {
            // Theft was detected — apply fine (black market goods priced at premium)
            let base_price = if agent_can_black_market {
                self.black_market.black_market_price(self.market.price(resource_id))
            } else {
                self.market.price(resource_id)
            };
            let fine = taken * base_price * Fixed::from_f64(2.0);
            self.agents[agent_idx].wealth.coin = (self.agents[agent_idx].wealth.coin - fine).max(Fixed::ZERO);
            self.journal.record(tick_u64, agent_id, JournalEntryKind::TheftDetected {
                resource: resource_name.into(),
                amount: taken.to_f64(),
                fine: fine.to_f64(),
            });
            self.events.push(SimEvent::NormViolated { agent: agent_id, norm_id: 0, witnesses: Vec::new(), tick });
            // §19.5.B: Record institutional enforcement provenance
            self.provenance.record_institutional(
                crate::provenance::InstitutionalTrace {
                    institution_name: "Council".into(),
                    tick: tick_u64,
                    decision_kind: "theft_enforcement".into(),
                    description: format!("{} stolen {} ({:.1} coins fine)", agent_id.as_u64(), resource_name, fine.to_f64()),
                    affected: vec![agent_id],
                    success: true,
                },
            );
            if let Some(owner_id) = owner {
                if let Some(rel) = self.relationships.iter_mut().find(|r| r.from == agent_id && r.to == owner_id) {
                    rel.trust = (rel.trust - Fixed::from_f64(0.2)).max(Fixed::ZERO);
                }
                self.agents[agent_idx].emotions.shame = (self.agents[agent_idx].emotions.shame + Fixed::from_f64(0.1)).clamp_01();
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

    /// Get a snapshot of agent summaries.
    pub fn agent_summaries(&self) -> Vec<AgentSummary> {
        self.agents
            .iter()
            .enumerate()
            .map(|(i, a)| AgentSummary {
                index: i,
                name: a.name.clone(),
                hunger: a.needs.hunger,
                thirst: a.needs.thirst,
                fatigue: a.needs.fatigue,
                health: a.body.health,
                energy: a.body.energy,
                valence: a.affect.valence,
                joy: a.emotions.joy,
                fear: a.emotions.fear,
                anger: a.emotions.anger,
                goal_count: a.goals.len(),
                current_action: format!("{:?}", a.current_action),
                attention_budget: a.attention.budget,
                has_intention: a.intention.is_some(),
            })
            .collect()
    }

    /// Get a reference to the event journal.
    pub fn journal(&self) -> &EventJournal {
        &self.journal
    }

    /// Get journal entry count.
    pub fn journal_len(&self) -> usize {
        self.journal.len()
    }

    /// Get total grain across all farms.
    pub fn total_grain(&self) -> Fixed {
        self.world.total_food()
    }

    /// Get total water across all wells.
    pub fn total_water(&self) -> Fixed {
        self.world.total_water()
    }

    /// Get a reference to the norm registry.
    pub fn norms(&self) -> &NormRegistry {
        &self.norms
    }

    /// §34: Get a reference to the causal provenance store.
    pub fn provenance(&self) -> &CausalProvenance {
        &self.provenance
    }

    /// §16.1: Capture a full deterministic snapshot of the simulation state.
    /// RNG streams are not serialized — only the master seed is stored.
    /// §16.1: Convenience alias — delegates to capture_snapshot.
    pub fn save_snapshot(&self) -> Snapshot {
        self.capture_snapshot()
    }

    /// Capture a snapshot of key metrics at the current tick.
    pub fn metrics_snapshot(&self) -> MetricsSnapshot {
        let summaries = self.agent_summaries();
        let n = summaries.len() as f64;
        MetricsSnapshot {
            tick: self.current_tick().as_u64(),
            avg_hunger: summaries.iter().map(|s| s.hunger.to_f64()).sum::<f64>() / n,
            avg_thirst: summaries.iter().map(|s| s.thirst.to_f64()).sum::<f64>() / n,
            avg_fatigue: summaries.iter().map(|s| s.fatigue.to_f64()).sum::<f64>() / n,
            avg_valence: summaries.iter().map(|s| s.valence.to_f64()).sum::<f64>() / n,
            avg_joy: summaries.iter().map(|s| s.joy.to_f64()).sum::<f64>() / n,
            avg_fear: summaries.iter().map(|s| s.fear.to_f64()).sum::<f64>() / n,
            total_grain: self.total_grain().to_f64(),
            total_water: self.total_water().to_f64(),
            event_count: self.event_count() as u64,
            journal_len: self.journal_len() as u64,
            agent_count: self.agent_count() as u64,
        }
    }
}

/// Maximum number of metric snapshots to retain (last MAX_METRIC_HISTORY * 10 ticks).
const MAX_METRIC_HISTORY: usize = 500;

/// A snapshot of key simulation metrics at a specific tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub tick: u64,
    pub avg_hunger: f64,
    pub avg_thirst: f64,
    pub avg_fatigue: f64,
    pub avg_valence: f64,
    pub avg_joy: f64,
    pub avg_fear: f64,
    pub total_grain: f64,
    pub total_water: f64,
    pub event_count: u64,
    pub journal_len: u64,
    pub agent_count: u64,
}

/// A lightweight summary of an agent's state for display.
#[derive(Debug, Clone)]
pub struct AgentSummary {
    pub index: usize,
    pub name: String,
    pub hunger: Fixed,
    pub thirst: Fixed,
    pub fatigue: Fixed,
    pub health: Fixed,
    pub energy: Fixed,
    pub valence: Fixed,
    pub joy: Fixed,
    pub fear: Fixed,
    pub anger: Fixed,
    pub goal_count: usize,
    pub current_action: String,
    pub attention_budget: Fixed,
    pub has_intention: bool,
}
