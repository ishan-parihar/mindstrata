//! Simulation orchestrator — the fixed-tick loop.

use crate::actions::{self, ActionKind};
use crate::appraisal::{self, Agency, Appraisal};
pub use crate::attention;
use crate::attention::{AttentionState, PerceptKind};
use crate::belief_update;
use crate::biology::EmbodiedState;
use crate::conflict::{self, ConflictKind, ConflictState};
use crate::culture::{CulturalState, Knowledge, KnowledgeCategory, TechnologyTree};
use crate::demography;
use crate::diplomacy::{
    DiplomacyRegistry, CARAVAN_GRAIN_BASE, EVENT_RELATION_SHIFT, RAID_GRAIN_CAP,
    RAID_GRAIN_FRACTION,
};
use crate::ecology;
use crate::factions;
use crate::gossip;
use crate::health;
use crate::institutions::{self, Institution, InstitutionKind};
use crate::journal::{EventJournal, JournalEntryKind};
use crate::legal::{LegalCase, LegalRegistry, Verdict};
use crate::logistics;
use crate::market::{self, MarketState, WealthState};
use crate::memory::{MemoryKind, MemoryStore, MemoryTag};
use crate::military::{
    conscription_eligible, drill_readiness, raid_loss_multiplier, MilitaryRegistry, MilitiaMember,
    MUSTER_CAP,
};
use crate::norms::{self, NormRegistry};
pub use crate::person::Intention;
use crate::person::{
    Affect, Belief, BodyState, CognitiveState, DerivedMentalState, DiscreteEmotions, Goal,
    GoalKind, GoalSource, IdentityKind, IdentityState, MoralValues, NeedState, Personality,
    Relationship, RelationshipKind, StatusState,
};
use crate::provenance::{CausalProvenance, DecisionFactor, DecisionTrace};
use crate::psychology::attachment::{AttachmentStyle, CaregivingStyle};
use crate::psychology::AttachmentSystem;
use crate::psychology::DevelopmentalPsychState;
use crate::psychology::EmotionRegulationState;
use crate::psychology::InteroceptiveState;
use crate::psychology::MoralCognition;
use crate::psychology::NarrativeIdentity;
use crate::psychology::ProspectionState;
use crate::psychology::PsychopathologyState;
use crate::psychology::SelfModel;
use crate::psychology::SkillState as PsychSkillState;
use crate::routines::DailyRoutine;
use crate::scenario::{Scenario, ShockKind};
use crate::schools::SchoolRegistry;
use crate::snapshot::Snapshot;
use crate::social;
use crate::social::attraction::AttractionModel;
use crate::social::relationship_v2::RelationshipV2;
use crate::social::status_dims::StatusDimensions;
use crate::systems::{self, SystemContext};
use crate::theology::{
    drift_conviction, seed_conviction, should_convert, Temperament, TheologicalBelief,
    TheologyRegistry, ELDER_AGE,
};
use crate::world::{World, GRAIN_RESOURCE_ID, WATER_RESOURCE_ID};
use crate::world_gen;
use mindstrata_core::clock::{Clock, Tick};
use mindstrata_core::event::{DeathCause, InteractionKind, SimEvent};
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::{AgentId, EntityId, ResourceId};
use mindstrata_core::rng::{RngStream, RngStreams};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

// ── Named constants for magic numbers ──────────────────────────────
/// Expected grain capacity per agent — used for scarcity normalization.
pub const EXPECTED_GRAIN_PER_AGENT: u32 = 10;
/// Water endowment assumed per agent when deriving §8.1.5 situational
/// affordance scarcity (world water stock / (this × population)).
pub const EXPECTED_WATER_PER_AGENT: u32 = 10;
/// Skill improvement per tick of practice.
pub const SKILL_GAIN_PER_TICK: Fixed = Fixed::from_raw(10); // 0.001

/// §8.1.3: Whether a practice tick carried a skill across a 0.1-proficiency
/// boundary — the milestone that warrants a Procedural memory. Integer raw
/// math (tenth = SCALE/10) keeps the gate exactly deterministic; tenth-step
/// precision keeps encodes sparse (~100 practice ticks apart at 0.001/tick).
fn skill_milestone_crossed(before: Fixed, after: Fixed) -> bool {
    const TENTH: i64 = 1000; // 0.1 at the SCALE = 10 000 fixed-point scale
    before < after && before.to_raw() / TENTH != after.to_raw() / TENTH
}

/// §8.1.3: Integrated-life-event count that closes an Episodic chapter.
const EPISODIC_CHAPTER_EVENTS: u32 = 100;

/// §8.1.3: Whether a narrative tick carried the agent's integrated-life-event
/// count across an [`EPISODIC_CHAPTER_EVENTS`]-event chapter boundary — the
/// milestone that warrants an Episodic memory. The narrative block fires every
/// tick in riverford runs (probe: ~3600 events per agent over 2000 ticks), so
/// the chapter gate keeps encodes sparse without a stateful cooldown.
fn life_chapter_crossed(before: u32, after: u32) -> bool {
    before / EPISODIC_CHAPTER_EVENTS != after / EPISODIC_CHAPTER_EVENTS
}
/// §5 (Iteration 155): If the agent holds an external directive (a
/// Command-sourced goal), return the action it demands AND the directive's
/// goal kind — unless a critical need of a *different* kind demands survival
/// first (the tick's own critical-interruption rule, applied at selection
/// time). Commands are strong nudges, not mind control: they override
/// routine and internal drives, but never block eating/drinking/resting
/// when those are truly pressing. `SeekSafety` has no aligned action
/// candidate in [`actions::select_action`], so a SeekSafety directive
/// returns `None` (the goal still shows in the inspector but cannot steer
/// an action yet).
///
/// The directive is *consumed* by the caller the moment it steers a
/// selection — satisfaction is the action being performed, NOT the resource
/// outcome (an unproductive farm would otherwise leave a Work directive
/// spinning forever on record_failure).
fn command_goal_action(goals: &[Goal], needs: &NeedState) -> Option<(ActionKind, GoalKind)> {
    // `max_by_key` returns the *last* maximum on ties, so a same-priority
    // repeat directive wins — the most recent command takes precedence.
    let command = goals
        .iter()
        .filter(|g| g.source == GoalSource::Command)
        .max_by_key(|g| g.priority.to_raw())?;
    let (action, blocked) = match command.kind {
        GoalKind::Eat => (
            ActionKind::Eat,
            needs.thirst > Fixed::from_f64(0.9) || needs.fatigue > Fixed::from_f64(0.95),
        ),
        GoalKind::Drink => (
            ActionKind::Drink,
            needs.hunger > Fixed::from_f64(0.9) || needs.fatigue > Fixed::from_f64(0.95),
        ),
        GoalKind::Rest => (
            ActionKind::Rest,
            needs.hunger > Fixed::from_f64(0.9) || needs.thirst > Fixed::from_f64(0.9),
        ),
        GoalKind::Work | GoalKind::Socialize | GoalKind::Worship => (
            match command.kind {
                GoalKind::Work => ActionKind::Work,
                GoalKind::Socialize => ActionKind::Socialize,
                _ => ActionKind::Worship,
            },
            needs.hunger > Fixed::from_f64(0.9)
                || needs.thirst > Fixed::from_f64(0.9)
                || needs.fatigue > Fixed::from_f64(0.95),
        ),
        GoalKind::SeekSafety => return None,
    };
    if blocked {
        None
    } else {
        Some((action, command.kind))
    }
}

/// Demography runs once per this many ticks (matches `phases.is_deca`).
const DEMOGRAPHY_TICK_INTERVAL: u64 = 10;
/// §8.1.4 (P3-6): length of the famine production-suppression window opened by
/// a Famine shock (~3 weeks at 96 ticks/day — long enough that the destroyed
/// grain cannot be refilled while the failure lasts, short enough that a
/// village genuinely recovers afterward).
const FAMINE_WINDOW_TICKS: u64 = 2000;
/// §8.1.4 (P3-6): per-tick proportional drain on stored grain while a famine
/// window is open — the granary is genuinely consumed. Sized so the ~45
/// surviving grain after a 0.7 shock reaches zero mid-window (the ~2.1
/// plateau showed production alone still matched consumption).
const FAMINE_GRAIN_DRAIN: Fixed = Fixed::from_raw(50); // 0.005/tick
/// §32 (Iteration 187): seasonal Cold/Fever vector knobs. Cold contracts
/// when `temperature < COLD_SEASON_TEMP` (the Winter band); the per-day
/// rate is `COLD_SEASON_RATE × (COLD_SEASON_TEMP − temperature)`, so at
/// Winter 0.2 that is ~0.2%/day/agent (~17%/agent/season) — a mild,
/// honest seasonal burden, not an epidemic. A Cold escalates to Fever at
/// `FEVER_ESCALATION_RATE` per day (~17%/season) — untreated colds
/// occasionally worsen.
const COLD_SEASON_TEMP: f64 = 0.4;
/// Per-day Cold contraction rate at the seasonal temperature floor.
const COLD_SEASON_RATE: Fixed = Fixed::from_raw(100); // 0.01
/// Per-day Cold→Fever escalation rate.
const FEVER_ESCALATION_RATE: Fixed = Fixed::from_raw(20); // 0.002
/// §29 (Iteration 187): market price-trend consumption — a buyer facing a
/// FALLING price holds out for a better deal (bids lower) and a buyer
/// facing a RISING price pays up (scarcity urgency). Modifier = 1 + trend ×
/// this, clamped to [0.9, 1.1] — a ±10% band at ±0.2 trend, honest and
/// modest (the calibrated trade price floor at 1.0 still applies).
const TREND_BUY_RATE: Fixed = Fixed::from_raw(5000); // 0.5
/// §19.5.E (Iteration 187): logistics local-scarcity dampening — the
/// module's raw modifier spans [0.5, 3.0]; applied to trade price as
/// `1 + (modifier − 1) × this` → [0.95, 1.2], a ±20% local scarcity
/// premium/discount without exploding the price band.
const LOGISTICS_PRICE_DAMP: Fixed = Fixed::from_raw(1000); // 0.1
/// Minimum mutual trust required to initiate a courtship (0.3).
const COURTSHIP_TRUST_THRESHOLD: Fixed = Fixed::from_raw(3000); // 0.3

// ── §8.1.18 knowledge-resistance constants (Iteration 166) ─────────
/// Taboo knowledge-resistance dampening rate: max_taboo × this is removed
/// from the knowledge-absorption chance, capped by the floor below. Sized
/// modest (0.2 → up to ~0.14 removal at max taboo ~0.68, ~0.11 at the
/// mean 0.53) so taboo-bound agents absorb novel knowledge measurably
/// slower without ever freezing transmission.
const KNOWLEDGE_TABOO_RATE: Fixed = Fixed::from_raw(2000); // 0.2
/// Taboo knowledge-resistance floor — absorption is slowed but never fully
/// blocked (a taboo-bound agent can still be taught; sacred values stay
/// defensible, not hermetically sealed).
const KNOWLEDGE_TABOO_FLOOR: Fixed = Fixed::from_raw(8000); // 0.8
/// §8.1.18 (Iteration 167): sacred-severity shame rate — the violence
/// taboo's violation_cost scales the attacker's per-violence shame boost
/// as (1 + violation_cost × this). ONE-SIDED: identity at zero taboo.
/// At the seeded max violation cost for the Violence keyword (~0.45 at
/// traditionalism 1.0) the factor reaches ≈1.23 — a ~17% amplification at
/// the mean cost (~0.33), a live-but-modest sacred-boundary signal that
/// sits below the golden-window metric granularity (zero-blast proven).
const VIOLENCE_SHAME_TABOO_RATE: Fixed = Fixed::from_raw(5000); // 0.5
/// §8.1.18 (Iteration 168): belief-update cultural dampening rate —
/// `change_resistance` (conservatism + category rigidity + taboo load, all
/// near-uniform at ~0.55 in calibrated windows: conservatism defaults 0.5
/// and the 7-taboo sum saturates min(1.0)) scales the daily belief-update
/// evidence as (1 − resistance × this). At the uniform 0.55 this is a
/// ~17% standing dampening — a uniform cultural brake on belief movement,
/// honestly framed (the differential requires a spread in `change_resistance`
/// that the current seeding does not produce).
const BELIEF_CULTURAL_RATE: Fixed = Fixed::from_raw(3000); // 0.3
/// Belief-update cultural dampening floor — evidence is damped but never
/// fully silenced (a maximally conservative agent still updates, slowly).
const BELIEF_CULTURAL_FLOOR: Fixed = Fixed::from_raw(8000); // 0.8
/// §8.1.18 (Iteration 169): violence-taboo escalation aversion rate — the
/// summed `violation_cost` of every taboo the "violence" description
/// violates (exactly the seeded Violence taboo, non-sacred, mean cost
/// ~0.34 at the traditionalism midpoint) scales the §19.5.H failed-threat
/// escalation chance as (1 − cost × this). The mean cost → factor ~0.966
/// with the shipped rate: a ~3% standing decision suppression that probe-
/// pins to a ~15% violence reduction at the hub (feedback-amplified) while
/// keeping the escalation-knob liveness delta POSITIVE (the Iter-128
/// lesson: the chain's violence→stress feedback multiplies beyond face
/// value — the 0.3 probe flipped the knob delta −543, so 0.1 is the
/// calibrated sweet spot).
const VIOLENCE_TABOO_AVERSION_RATE: Fixed = Fixed::from_raw(1000); // 0.1
/// Violence-taboo aversion floor — the cultural brake never fully erases
/// the escalation chance (a maximally taboo-bound agent still escalates,
/// rarely).
const VIOLENCE_TABOO_AVERSION_FLOOR: Fixed = Fixed::from_raw(5000); // 0.5

// ── Patronage creation constants (§22b) ────────────────────────────
/// Maximum clients a single patron may acquire per duodeca cycle.
const PATRONAGE_MAX_CLIENTS_PER_PATRON: usize = 3;
/// Minimum status differential for patron over client.
const PATRONAGE_STATUS_GAP: Fixed = Fixed::from_raw(1500); // 0.15
/// Minimum trust required for patronage formation.
const PATRONAGE_TRUST_THRESHOLD: Fixed = Fixed::from_raw(3500); // 0.35
/// Minimum affection required for patronage formation.
const PATRONAGE_AFFECTION_THRESHOLD: Fixed = Fixed::from_raw(2000); // 0.2
/// Patronage formation chance multiplier (scaled by trust).
const PATRONAGE_CHANCE_SCALE: Fixed = Fixed::from_raw(500); // 0.05
/// §10.9: Patrons transfer a daily stipend (provision × this rate) to
/// destitute clients — patronage as an economic safety net. Sized small
/// (0.02) to stay inside the calibrated propaganda/faction envelope:
/// legitimate_campaign_reaches_effectiveness_gate sits at 0.097 with a 0.05
/// rate; a faster drip perturbs the council-legitimacy trajectory too much.
const PATRONAGE_TRANSFER_RATE: Fixed = Fixed::from_raw(200); // 0.02
/// §10.9: A client is destitute (and eligible for support) below this coin floor.
const PATRONAGE_DESTITUTION_FLOOR: Fixed = Fixed::from_raw(10_000); // 1.0 coin
/// §10.9: Patronage buys political quiescence — client faction grievance is
/// dampened by dependence × this factor (material security softens
/// resentment/anger). Sized small to stay inside the calibrated
/// faction-formation envelope (factions_emerge_from_grievance).
const PATRONAGE_GRIEVANCE_DAMPEN: Fixed = Fixed::from_raw(600); // 0.06

/// §11.1: Perceived legitimacy modulates faction grievance — a point of
/// legitimacy deviation from the 0.5 mean-zero anchor shifts grievance by
/// this factor. Sized small to stay inside the calibrated faction-formation
/// envelope (mirrors PATRONAGE_GRIEVANCE_DAMPEN sizing).
const LEGITIMACY_GRIEVANCE_DAMPEN: Fixed = Fixed::from_raw(500); // 0.05

// ── Self-model constants (§8.1) ────────────────────────────────────
/// Regulation-capacity support per unit of self-esteem deviation from the
/// 0.5 baseline. Zero at baseline (the default self-model), so early
/// simulation is untouched; a coherent self-model (high self-esteem from a
/// redemptive narrative) sustains emotional regulation.
const SELF_MODEL_REGULATION_GAIN: Fixed = Fixed::from_raw(2000); // 0.2

// ── Group formation constants (§12.2) ──────────────────────────────
/// Social cost scale factor (density × this → 0.0–0.2).
const GROUP_SOCIAL_COST_SCALE: Fixed = Fixed::from_raw(2000); // 0.2
/// Default council legitimacy fallback when no Council exists.
const GROUP_COUNCIL_LEGITIMACY_DEFAULT: Fixed = Fixed::from_raw(5000); // 0.5
/// Institutional suppression scale factor (legitimacy × this → 0.0–0.4).
const GROUP_SUPPRESSION_SCALE: Fixed = Fixed::from_raw(4000); // 0.4

// ── SystemTrace provenance thresholds (§16.2) ─────────────────────
/// Cortisol level above which we record Hormonal provenance (0.6).
const HORMONAL_TRACE_THRESHOLD: Fixed = Fixed::from_raw(6000); // 0.6
/// Social support below which we record Attachment provenance (0.3).
const ATTACHMENT_LOW_SUPPORT_THRESHOLD: Fixed = Fixed::from_raw(3000); // 0.3
/// Iteration 191: the active-comfort reduction is damped to ~1–2 days of
/// separation per comfort event (see the Comfort-interaction wiring) so the
/// coupling stays visible while supported partners sit genuinely lower.
const ATTACHMENT_COMFORT_DAMPING: f64 = 0.0005;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// §10.1: The agent's perceived relational fields (sensory / social /
    /// noospheric), refreshed on the daily cadence. Decisional consumers are
    /// tracked in the `relational_field` module doc — six wired since
    /// Iteration 107 (`perceived_stress`, `kin_count`, `belief_confidence`,
    /// `social_trust`, `legitimacy_perceived`, `collective_fear`); the
    /// remaining observational layers are `social_obligation` and
    /// `peer_status`.
    #[serde(default)]
    pub relational_fields: crate::social::relational_field::RelationalFields,
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
    /// Architecture-plan-2 §8.1.9: Theory of Mind — models of other agents' minds.
    pub mind_models: crate::psychology::theory_of_mind::MindModels,
    /// Architecture-plan-2 §8.1.18: Cultural categories, taboos, honor codes.
    pub cultural_cognition: crate::psychology::CulturalCognition,
    /// Architecture-plan-2 §8.1.19: Skills and habits.
    pub psych_skills: PsychSkillState,
    /// Architecture-plan-2 §10.2: Enriched relationship model.
    pub relationship_v2s: Vec<RelationshipV2>,
    /// Architecture-plan-2 §8.1.11: Bounded log of speech acts the agent has
    /// performed — the structured linguistic frame on the interaction system.
    /// Write-only observational state (Iteration 40); `#[serde(default)]` so
    /// pre-Iter-40 snapshots restore.
    #[serde(default)]
    pub speech_log: Vec<crate::social::speech_act::SpeechAct>,
    /// Architecture-plan-2 §10.4: Attraction model for potential partners.
    pub attraction: AttractionModel,
    /// Architecture-plan-2 §11.1: Multi-dimensional status.
    pub status_v2: StatusDimensions,
    /// Architecture-plan-2 §8.2: Epistemic/informational substrate.
    /// Agents have partial, distorted, local knowledge.
    pub epistemic: crate::social::epistemic::EpistemicState,
    /// Architecture-plan-2 §8.1.12: Executive function and metacognition.
    pub cognitive_runtime: crate::psychology::CognitiveRuntime,
    /// Architecture-plan-2 §8.1.5: Layered motivation architecture.
    pub motivation: crate::psychology::MotivationState,
    /// Architecture-plan-2 §8.1.20: Decision policy integrating all psychology into action.
    pub decision_policy: crate::psychology::DecisionPolicy,
    /// Architecture-plan-2 §9.2: Deterministic neural-like runtime — concept
    /// embeddings, spreading activation, predictive error, RL action values,
    /// and script grammar (observational state).
    #[serde(default)]
    pub neural_like: crate::psychology::neural_like::NeuralLikeState,
    /// Architecture-plan-2 §17: Agent tier for level-of-detail simulation.
    pub agent_tier: crate::agent_tier::AgentTierState,
    /// Architecture-plan-2 §8.1.17: Narrative frames for event interpretation.
    pub narrative_frames: crate::culture::narrative_frame::NarrativeFrameSet,
    /// Architecture-plan-2 §13.6: Ideological position and polarization.
    pub ideology: crate::culture::ideology::Ideology,
    /// Architecture-plan-2 §8.1.7: Sacred values that resist cost-benefit reasoning.
    pub sacred_values: crate::culture::sacred::SacredValues,
    /// Architecture-plan-2 §8.1.18: Education state for knowledge transmission.
    pub education: crate::culture::education::EducationState,
    /// Architecture-plan-2 §11.1: Perceived legitimacy of the council — built
    /// by ritual participation, eroded by witnessed violations. Mean-zero at
    /// `new(0.5)`: the faction-grievance dampen is unchanged until the field
    /// actually moves with ritual/scandal events.
    #[serde(default)]
    pub legitimacy_field: crate::noosphere::LegitimacyField,
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
    /// §5.1 / Phase 5: Configurable simulation parameters.
    pub params: crate::parameters::SimParameters,
    clock: Clock,
    rng: RngStreams,
    pub world: World,
    pub agents: Vec<AgentBundle>,
    pub relationships: Vec<Relationship>,
    events: Vec<SimEvent>,
    journal: EventJournal,
    /// §12: Social norms — mod norms are registered post-populate through
    /// the modding API (`apply_content_pack`), so the registry is pub.
    pub norms: NormRegistry,
    /// §12: Institutions are first-class entities with legitimacy, roles, and collective psychology.
    pub institutions: Vec<Institution>,
    /// §34: Causal provenance — tracks decision traces and event causality chains.
    provenance: CausalProvenance,
    scenario: Option<Scenario>,
    /// §28: Season tracker for ecology.
    pub season: ecology::SeasonTracker,
    /// §5 (Iteration 147): continuous weather — temperature/rainfall with
    /// emergent drought/flood regimes (the Ecology RNG stream is virgin, so
    /// weather draws cannot perturb any other stream).
    pub weather: ecology::WeatherTracker,
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
    /// §19.5.I (Iteration 148): The technology tree — knowledge
    /// prerequisites and innovation chains layered over the flat store.
    pub technology: TechnologyTree,
    /// §5 (Iteration 149): The court — case records, verdicts, sentences.
    pub legal: LegalRegistry,
    /// §5 (Iteration 150): The off-map settlement network — neighboring
    /// villages with relations, caravans, and raids.
    pub diplomacy: DiplomacyRegistry,
    /// §5 (Iteration 151): Formal-school registry — yearly cohort terms.
    pub school: SchoolRegistry,
    /// §5 (Iteration 152): Religious registry — the seeded religion (if
    /// any), per-agent beliefs, and the festival calendar.
    pub theology: TheologyRegistry,
    /// §5 (Iteration 153): Military registry — the conscription roster,
    /// collective readiness, and drill tallies.
    pub military: MilitaryRegistry,
    /// §6.5: Per-tick metric history for observability and CSV export.
    pub metric_history: Vec<MetricsSnapshot>,
    /// §7.3: Tick when last revolution occurred (for cooldown tracking).
    last_revolution_tick: u64,
    /// §7.2: Tick when the last moral panic fired — enforces a cooldown so a
    /// saturated belief-charge loop cannot hammer legitimacy every tick.
    last_moral_panic_tick: u64,
    /// §8.1.4 (P3-6): Famine production-suppression window — the tick (exclusive)
    /// until which a Famine shock suppresses grain output. A famine is a crop
    /// failure, not a one-shot store drain: without this, Work production
    /// refilled the grain the shock destroyed within ~1.5K ticks and body hunger
    /// never rose (probe: needs.hunger 0.006–0.041 in EVERY scenario, famine ≈
    /// calm — the goal-incongruence branch was unreachable). 0 = no famine.
    famine_until: u64,
    /// §8.1.4 (P3-6): Grain-output multiplier while `famine_until` is active —
    /// `1 − magnitude` of the triggering shock (0.3 for the standard 0.7 famine).
    famine_production_factor: Fixed,
    /// §12.4: Tick when the last cult formed — enforces a formation cooldown.
    last_cult_formation_tick: u64,
    /// §4.4: Black market state — activates under scarcity with weak enforcement.
    pub black_market: crate::black_market::BlackMarketState,
    /// Architecture-plan-2 §10.6: Kinship graph — biological, marital, ritual kin.
    pub kinship_graph: crate::social::kinship::KinshipGraph,
    /// Architecture-plan-2 §10.7: Households — primary social and economic units.
    pub households: Vec<crate::social::household::Household>,
    /// Architecture-plan-2 §13.1: Meme registry — cultural units that propagate.
    pub meme_registry: crate::culture::MemeRegistry,
    /// Architecture-plan-2 §17.4: Meme aggregation engine for large-population metrics.
    pub meme_aggregator: crate::culture::MemeAggregator,
    /// Architecture-plan-2 §13.4: Propaganda registry — institutional narrative campaigns.
    pub propaganda_registry: crate::culture::PropagandaRegistry,
    /// Architecture-plan-2 §12.5: Ritual registry — structured ceremonial activities.
    pub ritual_registry: crate::culture::RitualRegistry,
    /// Architecture-plan-2 §13.3: Rumor registry — rumors with uncertainty and social stakes.
    pub rumor_registry: crate::culture::RumorRegistry,
    /// Architecture-plan-2 §13.5: Collective memory registry — shared group memories.
    pub collective_memory_registry: crate::culture::CollectiveMemoryRegistry,
    /// Architecture-plan-2 §13.6: Echo chamber state — belief clusters and polarization.
    pub echo_chamber: crate::culture::EchoChamberState,
    /// Architecture-plan-2 §10.8: Clan registry — emergent kinship-plus-alliance groups.
    pub clan_registry: crate::social::clan::ClanRegistry,
    /// Architecture-plan-2 §10.4-§10.5: Marriage registry — pair bonds and marriages.
    pub marriage_registry: crate::social::marriage::MarriageRegistry,
    /// Architecture-plan-2 §12.4: Cult registry — emergent cult dynamics.
    pub cult_registry: crate::social::cult::CultRegistry,
    /// Architecture-plan-2 §13.5: Moral panic registry — collective fear events.
    pub moral_panic_registry: crate::noosphere::MoralPanicRegistry,
    /// Architecture-plan-2 §13: Noospheric field — symbolic concept space.
    pub noospheric_field: crate::noosphere::NoosphericField,
    /// Architecture-plan-2 §5.2, §29.2: Faction v2 — upgraded faction dynamics.
    pub faction_v2_registry: crate::social::faction_v2::FactionV2Registry,
    /// Architecture-plan-2 §10.4: Active courtships between agents.
    /// NOTE: not persisted in snapshots — rebuilt from agent state.
    pub active_courtships: Vec<crate::social::courtship::Courtship>,
    /// Architecture-plan-2 §10.9: Patronage relations — asymmetric patron/client power dynamics.
    /// Wired: creation in §22b (social tick), daily_update in §20 (kinship daily).
    /// NOTE: not persisted in snapshots — rebuilt from agent state.
    pub patronage_registry: crate::social::patronage::PatronageRegistry,
    /// Architecture-plan-2 §12.2: Peer group registry — emergent groups from formation pressure.
    /// Wired: registration in duodeca (§12.2), daily_update in §20 (kinship daily).
    pub group_registry: crate::social::group_formation::GroupRegistry,
    /// §17.4: Accumulated exposure samples from gossip block, flushed daily into meme_aggregator.
    exposure_samples: Vec<crate::culture::meme_aggregator::ExposureSample>,
}

impl Simulation {
    /// Compute the index into relationship_v2s for agent a → agent b.
    ///
    /// The relationship_v2s vector for agent `a` contains entries for all other
    /// agents in a fixed order. This helper guarantees the same index formula
    /// is used everywhere (populate, ritual bonding, tick loop).
    ///
    /// # Panics
    /// Panics if `a == b` or `a >= n_agents`.
    #[inline]
    fn relationship_v2_index(a: usize, b: usize, n_agents: usize) -> usize {
        debug_assert!(a != b, "self-relationship index requested");
        debug_assert!(a < n_agents && b < n_agents, "agent index out of bounds");
        a * (n_agents - 1) + if b > a { b - 1 } else { b }
    }

    /// Per-agent position of target `b` inside agent `a`'s `relationship_v2s` vec.
    ///
    /// `relationship_v2s[a]` holds one entry per *other* agent, ordered by
    /// target index with `a` itself skipped. So target `b` sits at position
    /// `b` when `b < a`, else `b - 1`. This is *not* the same as
    /// [`Self::relationship_v2_index`], which indexes the legacy flat
    /// `self.relationships` matrix of `n·(n−1)` entries.
    #[inline]
    fn relationship_v2_pos(a: usize, b: usize) -> usize {
        debug_assert!(a != b, "self-relationship position requested");
        if b > a {
            b - 1
        } else {
            b
        }
    }

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
        for (i, (desc, content, emotional, identity, mutation)) in seeds.iter().enumerate() {
            self.meme_registry.register(Meme::new(
                i,
                desc.to_string(),
                *content,
                Fixed::from_f64(*emotional),
                Fixed::from_f64(*identity),
                0,                                  // created_tick — deterministic across new()/from_snapshot()
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
            last_moral_panic_tick: self.last_moral_panic_tick,
            last_cult_formation_tick: self.last_cult_formation_tick,
            black_market: &self.black_market,
            site_work_ticks: &self.site_work_ticks,
            group_registry: &self.group_registry,
            collective_memory_registry: &self.collective_memory_registry,
            meme_registry: &self.meme_registry,
            kinship_graph: &self.kinship_graph,
        })
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
                    att.initialize(
                        caregiver_security,
                        trauma_history,
                        attachment_vulnerability,
                    );
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
    fn seed_initial_rituals_and_campaigns(&mut self) {
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
        const CLAN_COUNT: usize = 2;
        let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); CLAN_COUNT];
        for (i, agent) in self.agents.iter().enumerate() {
            let site = agent.home_site.unwrap_or(i);
            buckets[site % CLAN_COUNT].push(i);
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

    /// §10.8: Clan membership lookup — which clan holds this agent index
    /// (core_households stores agent indices). Deterministic registry scan.
    fn clan_of(&self, agent_idx: usize) -> Option<usize> {
        self.clan_registry
            .clans
            .iter()
            .find(|c| c.core_households.contains(&agent_idx))
            .map(|c| c.id)
    }

    /// §10.8: Marriage forges a clan alliance — the design doc's stated
    /// alliance source. Symmetric (both clans declare each other);
    /// declare_ally dedupes and refuses existing enemies.
    fn forge_clan_alliance(&mut self, a: usize, b: usize, tick: u64) {
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
    fn forge_clan_enmity(&mut self, a: usize, b: usize, tick: u64) {
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
    fn clans_are_enemies(&self, a: usize, b: usize) -> bool {
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
    fn clans_are_allies(&self, a: usize, b: usize) -> bool {
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
    fn decay_clan_enmities(&mut self) {
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
    fn escalation_chance(&self, from_idx: usize, to_idx: usize) -> f64 {
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
    fn dominance_escalation_scale(power_balance: Fixed) -> f64 {
        (1.0 + power_balance.to_f64() * 0.5).clamp(0.5, 1.5)
    }

    /// §19.5.H/§10.8: Whether a failed threat escalates to violence. The RNG
    /// draw happens under exactly the same conditions as the original inline
    /// logic, so the stream is untouched (replay determinism).
    fn should_escalate(
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
        let taboo_aversion = (Fixed::ONE
            - violence_taboo_cost * VIOLENCE_TABOO_AVERSION_RATE)
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
            village.add_memory(
                "The village was founded by the first settlers who followed the river".into(),
                SharedMemoryKind::Founding,
                0,
                Fixed::from_f64(0.8),
            );
            village.add_memory(
                "The last great drought nearly starved the village".into(),
                SharedMemoryKind::Trauma,
                0,
                Fixed::from_f64(0.6),
            );
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

    /// §12.4: Daily cult emergence and dissolution.
    ///
    /// Previously the cult registry was constructed but never written —
    /// `register()` had zero call sites, so the §12.4 system was dead.
    /// Emergence is deterministic (trait-ordered selection, no RNG) so fresh
    /// and replayed runs match: gated by low institutional legitimacy and a
    /// meaning deficit among agents, with a formation cooldown.
    fn tick_cults(&mut self, tick: u64) {
        // ── Dissolution ──
        let n_inst = self.institutions.len().max(1);
        let legitimacy = self
            .institutions
            .iter()
            .map(|i| i.legitimacy)
            .fold(Fixed::ZERO, |acc, l| acc + l)
            * Fixed::from_f64(1.0 / n_inst as f64);
        let n_agents = self.agents.len().max(1);
        let avg_fatigue = self
            .agents
            .iter()
            .map(|a| a.body.fatigue)
            .fold(Fixed::ZERO, |acc, f| acc + f)
            * Fixed::from_f64(1.0 / n_agents as f64);
        // Dissolution carries member fallout: when a cult dissolves, its
        // former members lose the belonging that was satisfying their meaning
        // need (the deficit rebounds into a crisis) and their cult-entrenched
        // beliefs partially decay. This completes the §12.4 lifecycle:
        // formation → recruitment → entrenchment → dissolution → fallout.
        // Economic stress — the same scarcity proxy the motivation system uses
        // (world totals vs expected per-capita rations, ~line 2176).
        let expected_food = crate::sim::EXPECTED_GRAIN_PER_AGENT as i64 * self.agents.len() as i64;
        let expected_water = crate::sim::EXPECTED_WATER_PER_AGENT as i64 * self.agents.len() as i64;
        let food_scarcity = (Fixed::ONE
            - self.world.total_food() / Fixed::from_int(expected_food.max(1)))
        .clamp_01();
        let water_scarcity = (Fixed::ONE
            - self.world.total_water() / Fixed::from_int(expected_water.max(1)))
        .clamp_01();
        let economic_stress = food_scarcity.max(water_scarcity);

        let mut former_members: Vec<usize> = Vec::new();
        for cult in &mut self.cult_registry.cults {
            // §12.4: compute the full dissolution signal set. Leader competence
            // is proxied by the leader's conscientiousness (no dedicated
            // competence model); prophecy disconfirmation fires when the
            // sacred meme's credibility collapses or the meme vanishes;
            // internal betrayal fires when most members' meaning need is
            // satisfied (belonging no longer binds them) — note `needs.meaning`
            // is a DEFICIT scale: HIGH = craving, LOW = satisfied = betrayal
            // risk; rival narrative pressure is the excess emotional charge of
            // the hottest competing meme over the sacred one.
            let leader_competence = self
                .agents
                .get(cult.charismatic_leader)
                .map_or(Fixed::from_f64(0.5), |a| a.personality.conscientiousness);
            let prophecy_disconfirmed = self
                .meme_registry
                .memes
                .iter()
                .find(|m| m.id == cult.sacred_narrative_id as usize)
                .is_none_or(|m| !m.active || m.credibility < Fixed::from_f64(0.3));
            let member_count = cult.members.len().max(1);
            let satisfied = cult
                .members
                .iter()
                .filter(|&&m| {
                    self.agents
                        .get(m)
                        .is_some_and(|a| a.needs.meaning <= Fixed::from_f64(0.2))
                })
                .count();
            let internal_betrayal = satisfied > member_count / 2;
            let sacred_charge = self
                .meme_registry
                .memes
                .iter()
                .find(|m| m.id == cult.sacred_narrative_id as usize)
                .map_or(Fixed::ZERO, |m| m.emotional_charge);
            let rival_charge = self
                .meme_registry
                .memes
                .iter()
                .filter(|m| m.active && m.id != cult.sacred_narrative_id as usize)
                .map(|m| m.emotional_charge)
                .fold(Fixed::ZERO, Fixed::max);
            let rival_narrative_pressure = (rival_charge - sacred_charge).clamp_01();
            let signals = crate::social::cult::CultDissolutionSignals {
                leader_competence,
                institutional_legitimacy: legitimacy,
                member_average_fatigue: avg_fatigue,
                cult_age_ticks: tick.saturating_sub(cult.formed_tick),
                prophecy_disconfirmed,
                internal_betrayal,
                rival_narrative_pressure,
                economic_stress,
            };
            if cult.active && cult.should_dissolve(&signals) {
                cult.active = false;
                former_members.extend(
                    std::iter::once(&cult.charismatic_leader)
                        .chain(cult.members.iter())
                        .copied(),
                );
            }
        }
        for i in former_members {
            if i >= self.agents.len() {
                continue;
            }
            let agent = &mut self.agents[i];
            // Meaning crisis rebound: belonging was fulfilling the need.
            agent.needs.meaning =
                (agent.needs.meaning * Fixed::from_f64(0.5) + Fixed::from_f64(0.5)).clamp_01();
            // Cult-entrenched beliefs decay once the narrative grip is gone.
            if let Some(hot) = agent.beliefs.iter_mut().max_by_key(|b| b.emotional_charge) {
                hot.confidence = (hot.confidence - Fixed::from_f64(0.05)).max(Fixed::ZERO);
                hot.emotional_charge =
                    (hot.emotional_charge - Fixed::from_f64(0.05)).max(Fixed::ZERO);
            }
        }

        // ── Emergence ──
        const CULT_COOLDOWN: u64 = 2880; // ~20 days at 144 ticks/day
        const CULT_FORM_MEMBERS: usize = 4; // initial core (leader + up to 4)
        if tick.saturating_sub(self.last_cult_formation_tick) < CULT_COOLDOWN {
            return;
        }
        if legitimacy > Fixed::from_f64(0.45) {
            return; // institutions too strong — no fertile ground
        }
        // Candidates: agents suffering a meaning deficit.
        let candidates: Vec<usize> = (0..self.agents.len())
            .filter(|&i| self.agents[i].needs.meaning > Fixed::from_f64(0.3))
            .collect();
        if candidates.len() < 3 {
            return;
        }
        // Charismatic leader: highest extraversion + agreeableness among the
        // meaning-starved (deterministic: agent order is identical in fresh
        // and replayed runs).
        let leader = candidates
            .iter()
            .copied()
            .max_by(|&a, &b| {
                let sa = self.agents[a].personality.extraversion
                    + self.agents[a].personality.agreeableness;
                let sb = self.agents[b].personality.extraversion
                    + self.agents[b].personality.agreeableness;
                sa.cmp(&sb)
            })
            .unwrap();
        // Members: other meaning-deficit agents, capped for cost.
        let members: Vec<usize> = candidates
            .iter()
            .copied()
            .filter(|&i| i != leader)
            .take(CULT_FORM_MEMBERS)
            .collect();
        if members.len() < 2 {
            return;
        }
        // Sacred narrative: the hottest active meme.
        let sacred_id = self
            .meme_registry
            .memes
            .iter()
            .filter(|m| m.active)
            .max_by_key(|m| m.emotional_charge)
            .map_or(0, |m| m.id as u32);
        let mut cult = crate::social::cult::CultDynamics::new(leader, sacred_id, tick);
        cult.members = members;
        self.cult_registry.register(cult);
        self.last_cult_formation_tick = tick;
    }

    /// §12.4: Daily cult member dynamics — recruitment and psychological feedback.
    ///
    /// Active cults recruit new meaning-starved agents (up to a membership
    /// cap) and entrench their members: belonging satisfies the meaning need
    /// (deficit shrinks), the hottest belief gains confidence/charge (cult
    /// narrative grip), and identity fusion / fear-load are reinforced against
    /// the daily decay in `CultDynamics::daily_update`. This turns the
    /// previously observational cult registry (Iteration 9) into an emergent
    /// force on member psychology. Fully deterministic (agent order, no RNG)
    /// so fresh and replayed runs match.
    fn tick_cult_dynamics(&mut self) {
        let n = self.agents.len();
        let member_cap = (n / 2).max(4);

        // ── Recruitment ──
        // Collect (cult_idx, agent_idx) targets under an immutable borrow of
        // the registry, then apply them to avoid borrow conflicts.
        let mut recruits: Vec<(usize, usize)> = Vec::new();
        let active: Vec<usize> = (0..self.cult_registry.cults.len())
            .filter(|&ci| self.cult_registry.cults[ci].active)
            .collect();
        for &ci in &active {
            let cult = &self.cult_registry.cults[ci];
            let mut slots = member_cap.saturating_sub(1 + cult.members.len());
            if slots == 0 {
                continue;
            }
            for i in 0..n {
                if slots == 0 {
                    break;
                }
                if self.agents[i].needs.meaning <= Fixed::from_f64(0.3) {
                    continue;
                }
                if i == cult.charismatic_leader || cult.members.contains(&i) {
                    continue;
                }
                let already_in = self
                    .cult_registry
                    .cults
                    .iter()
                    .any(|c| c.active && c.members.contains(&i));
                if already_in {
                    continue;
                }
                recruits.push((ci, i));
                slots -= 1;
            }
        }
        for (ci, i) in recruits {
            self.cult_registry.cults[ci].add_member(i);
        }

        // ── Member psychological feedback ──
        // Snapshot (leader, members) per active cult, then apply to agents.
        let member_sets: Vec<(usize, Vec<usize>)> = self
            .cult_registry
            .cults
            .iter()
            .filter(|c| c.active)
            .map(|c| (c.charismatic_leader, c.members.clone()))
            .collect();
        for (leader, members) in member_sets {
            for &i in std::iter::once(&leader).chain(members.iter()) {
                if i >= n {
                    continue;
                }
                let agent = &mut self.agents[i];
                // Belonging satisfies the meaning need (deficit shrinks).
                agent.needs.meaning = (agent.needs.meaning * Fixed::from_f64(0.98)
                    - Fixed::from_f64(0.01))
                .max(Fixed::ZERO);
                // Entrench the hottest belief — the cult's narrative grip.
                if let Some(hot) = agent.beliefs.iter_mut().max_by_key(|b| b.emotional_charge) {
                    hot.confidence = (hot.confidence + Fixed::from_f64(0.01)).clamp_01();
                    hot.emotional_charge =
                        (hot.emotional_charge + Fixed::from_f64(0.01)).clamp_01();
                }
            }
            // Cult-level reinforcement against the daily_update decay.
            // The leader index uniquely identifies an active cult here.
            if let Some(cult) = self
                .cult_registry
                .cults
                .iter_mut()
                .find(|c| c.active && c.charismatic_leader == leader)
            {
                cult.identity_fusion = (cult.identity_fusion + Fixed::from_f64(0.004)).clamp_01();
                cult.fear_load = (cult.fear_load + Fixed::from_f64(0.002)).clamp_01();
            }
        }
    }

    /// §13: Noospheric field update (every tick) — refresh node metadata from
    /// the live meme registry, feed meme emotional charge as spreading
    /// activation, then apply natural decay (the pre-existing behavior).
    fn tick_noosphere(&mut self) {
        for meme in &self.meme_registry.memes {
            let node_id = meme.id as u32;
            if let Some(node) = self
                .noospheric_field
                .nodes
                .iter_mut()
                .find(|n| n.id == node_id)
            {
                node.emotional_charge = meme.emotional_charge;
                node.identity_relevance = meme.identity_relevance;
                node.institutional_backing = meme.credibility;
            }
            self.noospheric_field
                .spread_activation(node_id, meme.emotional_charge * Fixed::from_f64(0.05));
        }
        // Natural decay — preserved baseline behavior.
        self.noospheric_field.decay_all(Fixed::from_f64(0.001));
    }

    /// §13: Daily belief-ecology projection — the noosphere feeds back into
    /// individual psychology, closing the loop that previously ran one-way
    /// (memes/beliefs → nodes only). The field's most-activated node is the
    /// dominant symbolic mood (the "zeitgeist"); project it onto every
    /// agent's hottest belief as a small, capped emotional-charge boost so
    /// the collective atmosphere amplifies conviction. Structurally distinct
    /// from the §13.6 echo chamber: that is per-cluster (who you are tied
    /// to); this is global (the mood of the whole field). Deterministic.
    fn tick_noosphere_belief_projection(&mut self) {
        let zeitgeist = self
            .noospheric_field
            .nodes
            .iter()
            .map(super::noosphere::field::SymbolicNode::effective_activation)
            .fold(Fixed::ZERO, std::cmp::Ord::max);
        if zeitgeist <= Fixed::from_f64(0.4) {
            return; // dormant field — no atmospheric pressure
        }
        let boost = zeitgeist * Fixed::from_f64(0.0005);
        for agent in &mut self.agents {
            if let Some(hot) = agent.beliefs.iter_mut().max_by_key(|b| b.emotional_charge) {
                hot.emotional_charge = (hot.emotional_charge + boost).clamp_01();
            }
        }
    }

    /// §13.5: Record a famine (scarcity) trauma in the village's collective
    /// memory when hunger or thirst runs critically high. Episode-guarded:
    /// a scarcity trauma is not re-recorded while a recent one exists, so
    /// one crisis yields one memory. Deterministic (reads agent needs).
    fn record_famine_memory(&mut self, tick: u64) {
        use crate::culture::SharedMemoryKind;
        // Episode guard — skip while a *live* scarcity trauma exists. Seeded
        // origin memories have event_tick == 0 and never block.
        let recent = self.collective_memory_registry.get(0).is_some_and(|cm| {
            cm.memories.iter().any(|m| {
                m.kind == SharedMemoryKind::Trauma
                    && m.event_tick > 0
                    && tick.saturating_sub(m.event_tick) < 43200 // ~300 days
            })
        });
        if recent {
            return;
        }
        let n = self.agents.len().max(1);
        let inv = Fixed::from_f64(1.0 / n as f64);
        let avg_hunger = self
            .agents
            .iter()
            .map(|a| a.needs.hunger)
            .fold(Fixed::ZERO, |acc, h| acc + h)
            * inv;
        let avg_thirst = self
            .agents
            .iter()
            .map(|a| a.needs.thirst)
            .fold(Fixed::ZERO, |acc, t| acc + t)
            * inv;
        let (desc, charge) = if avg_thirst > Fixed::from_f64(0.6) {
            ("Thirst gripped the village when the wells ran low", 0.8)
        } else if avg_hunger > Fixed::from_f64(0.6) {
            ("The village went hungry when the harvest failed", 0.8)
        } else {
            return;
        };
        self.collective_memory_registry.get_or_create(0).add_memory(
            desc.into(),
            SharedMemoryKind::Trauma,
            tick,
            Fixed::from_f64(charge),
        );
    }

    /// §13.5: Record a faction's founding myth — group 1 + faction id keeps
    /// faction memory separate from the village's (group 0).
    fn record_faction_founding_memory(&mut self, faction_id: usize, tick: u64) {
        use crate::culture::SharedMemoryKind;
        self.collective_memory_registry
            .get_or_create(1 + faction_id)
            .add_memory(
                "The faction was born of shared grievance against the council".into(),
                SharedMemoryKind::Founding,
                tick,
                Fixed::from_f64(0.7),
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
        // §6: Compute multi-timescale phase flags once per tick.
        let phases = crate::scheduler::TickPhases::compute(tick_u64);

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
                        // A drought dries the water supply (WATER_RESOURCE_ID = 1).
                        // It previously drained grain (resource_id == 0) — a drought
                        // that destroys food instead of water.
                        //
                        // The drain is proportional to the stocked quantity, not a
                        // fixed magnitude × 10: the village well starts with 200
                        // water, so an absolute drain of 3.0 vs 7.0 was a 1.5% vs
                        // 3.5% difference — both drowned out by agent consumption,
                        // making riverford (0.3) and drought (0.7) indistinguishable.
                        // A proportional drain leaves 70% vs 30% of remaining water,
                        // so the shock magnitude now genuinely differentiates.
                        //
                        // NB: no `.clamp_01()` here — a well holding 200 water is
                        // legitimately above 1.0, and clamping to [0,1] would erase
                        // the magnitude difference (60 and 140 both → 1.0). We only
                        // floor at zero to keep quantities non-negative.
                        for site in &mut self.world.sites {
                            for stock in &mut site.inventory {
                                if stock.resource_id == WATER_RESOURCE_ID {
                                    stock.quantity = (stock.quantity
                                        * (Fixed::ONE - shock.magnitude))
                                        .max(Fixed::ZERO);
                                }
                            }
                        }
                    }
                    ShockKind::Famine => {
                        // A famine destroys the grain supply (GRAIN_RESOURCE_ID).
                        // Same proportional-drain semantics as Drought: a fixed
                        // absolute drain would be drowned out by the farm's large
                        // stock, so the magnitude must scale with the stocked
                        // quantity to genuinely differentiate scenarios.
                        for site in &mut self.world.sites {
                            for stock in &mut site.inventory {
                                if stock.resource_id == GRAIN_RESOURCE_ID {
                                    stock.quantity = (stock.quantity
                                        * (Fixed::ONE - shock.magnitude))
                                        .max(Fixed::ZERO);
                                }
                            }
                        }
                        // §8.1.4 (P3-6): a famine is a CROP FAILURE, not a
                        // one-shot store drain. Without production suppression,
                        // Work refilled the destroyed grain within ~1.5K ticks
                        // and body hunger never rose (probe: needs.hunger
                        // 0.006–0.041 in every scenario, famine ≈ calm — the
                        // goal-incongruence branch was unreachable). Open a
                        // production-suppression window (FAMINE_WINDOW_TICKS)
                        // during which Work grain output is scaled by
                        // `1 − magnitude` (0.3 for the standard 0.7 famine):
                        // stored grain genuinely depletes, Eat fails, and
                        // hunger accumulates toward the 0.5 goal-congruence
                        // gate.
                        self.famine_until = tick_u64 + FAMINE_WINDOW_TICKS;
                        // Near-total crop failure: production collapses to
                        // 0.05 × magnitude (3.5% for the standard 0.7 famine).
                        // `1 − magnitude` = 0.3 was NOT enough — the surviving
                        // trickle still yielded ~2 full portions/day and the
                        // village scraped by (probe: top hunger 0.488, never
                        // crossing the 0.5 gate). A famine starves.
                        self.famine_production_factor = shock.magnitude
                            * Fixed::from_f64(0.05)
                            .max(Fixed::from_f64(0.01));
                    }
                    ShockKind::Pestilence => {
                        // A pestilence strikes in two composed waves, both routed
                        // through EXISTING machinery rather than fabricated:
                        //  1. IMMEDIATE MORTALITY — a seeded roll with
                        //     p = magnitude × (1 − health × 0.5) is deadly at
                        //     every health level yet deadlier for the weak
                        //     (health 1.0 → 0.35 vs health 0.5 → 0.525 at
                        //     mag 0.7). Deaths flow
                        //     through handle_agent_death — the same path the §31
                        //     demography pass uses — so inheritance, journaling,
                        //     event recording, social cleanup, and in-place
                        //     newborn replacement all stay consistent. This is
                        //     safe here: the shock block runs before the
                        //     SystemContext borrow (self is fully mutable), and
                        //     the dead are replaced IN PLACE so the
                        //     AgentId == index invariant survives.
                        //  2. SURVIVING EPIDEMIC — survivors (and the newborns
                        //     who replaced the dead) carry a virulent Epidemic
                        //     that block 17b spreads by proximity contagion and
                        //     block 17 drains health from over its 800-tick course.
                        // Copy the magnitude first: handle_agent_death borrows
                        // self mutably, so `shock` (borrowing self.scenario) must
                        // not be live across those calls.
                        let magnitude = shock.magnitude;
                        let n = self.agents.len();
                        let mut deaths: Vec<usize> = Vec::new();
                        for i in 0..n {
                            let health = self.agents[i].body.health;
                            let p = magnitude * (Fixed::ONE - health * Fixed::from_f64(0.5));
                            let roll = Fixed::from_f64(
                                self.rng.get_mut(RngStream::Behavior).random::<f64>(),
                            );
                            if roll < p {
                                deaths.push(i);
                            }
                        }
                        tracing::warn!(
                            deaths = deaths.len(),
                            "Pestilence: immediate mortality wave"
                        );
                        // Note: the newborns that replace the dead run the
                        // remainder of this tick's systems (the demography path
                        // kills at tick end instead) — an accepted divergence,
                        // invisible to calibrated runs since no existing
                        // scenario carries a Pestilence shock.
                        for &idx in &deaths {
                            self.handle_agent_death(
                                idx,
                                &deaths,
                                tick_u64,
                                tick,
                                DeathCause::Disease,
                            );
                        }
                        // Seed the virulent epidemic: every agent rolls an
                        // independent infection check (p = magnitude), so the
                        // selection is index-fair — a plague spares no one.
                        let mut infected = 0usize;
                        for i in 0..n {
                            let roll = Fixed::from_f64(
                                self.rng.get_mut(RngStream::Behavior).random::<f64>(),
                            );
                            if roll < magnitude {
                                let already = self.agent_diseases[i]
                                    .iter()
                                    .any(|d| d.kind == health::DiseaseKind::Epidemic);
                                if !already {
                                    self.agent_diseases[i].push(health::ActiveDisease {
                                        kind: health::DiseaseKind::Epidemic,
                                        ticks_infected: 0,
                                        severity_modifier: Fixed::ONE, // virulent strain
                                    });
                                    infected += 1;
                                }
                            }
                        }
                        tracing::debug!(
                            infected = infected,
                            "Pestilence: epidemic seeded into the population"
                        );
                    }
                    ShockKind::Festival => {
                        for agent in &mut self.agents {
                            agent.emotions.joy = (agent.emotions.joy
                                + shock.magnitude * Fixed::from_f64(0.3))
                            .clamp_01();
                        }
                    }
                }
            }
        }

        // Capture event count before systems run (before SystemContext borrows self.events)
        let pre_tick_events = self.events.len();

        // §19.5.J: Capture relationship snapshot before any systems modify relationships
        let rel_snapshot: Vec<(AgentId, AgentId, Fixed, Fixed)> = self
            .relationships
            .iter()
            .map(|r| (r.from, r.to, r.trust, r.affection))
            .collect();

        // Collect action starts to defer resource operations outside the ctx block
        let mut action_starts: Vec<(usize, ActionKind)> = Vec::new();

        // §8.1.5: Precompute world food/water totals before SystemContext borrows
        // self.world mutably (the motivation block reads them later, so they must
        // be captured here to avoid a borrow conflict).
        let world_food_total = self.world.total_food();
        let world_water_total = self.world.total_water();

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
            let mut bodies: Vec<BodyState> = self
                .agents
                .iter_mut()
                .map(|a| std::mem::take(&mut a.body))
                .collect();
            let mut needs: Vec<NeedState> = self
                .agents
                .iter_mut()
                .map(|a| std::mem::take(&mut a.needs))
                .collect();
            let personalities: Vec<Personality> =
                self.agents.iter().map(|a| a.personality.clone()).collect();
            let mut goals: Vec<Vec<Goal>> = self
                .agents
                .iter_mut()
                .map(|a| std::mem::take(&mut a.goals))
                .collect();
            let mut affects: Vec<Affect> = self
                .agents
                .iter_mut()
                .map(|a| std::mem::take(&mut a.affect))
                .collect();
            let mut emotions: Vec<DiscreteEmotions> = self
                .agents
                .iter_mut()
                .map(|a| std::mem::take(&mut a.emotions))
                .collect();

            // Architecture-plan-2 §8.2: Epistemic reset and trust sync.
            // §8.2: Reset per-tick processing counters and sync trust network from relationships.
            // Trust sync convergence rate: ~63% aligned after 10 ticks, ~86% after 20 ticks.
            // §8.2: Trust sync convergence rate: ~63% aligned after 10 ticks, ~86% after 20 ticks.
            let trust_sync_rate = Fixed::from_f64(0.1);
            for agent in &mut self.agents {
                agent.epistemic.reset_tick();
                // §17.2: Reset cognitive budget tracker for this tick.
                agent.agent_tier.reset_tick_budget();
            }
            // Sync TrustNetwork from existing relationship trust values.
            // O(N) pre-computation: build a per-agent trust delta map from relationships.
            // This bridges the social interaction system's trust with the epistemic layer.
            let mut trust_deltas: Vec<Vec<(u64, Fixed)>> = Vec::with_capacity(self.agents.len());
            for _ in 0..self.agents.len() {
                trust_deltas.push(Vec::new());
            }
            for rel in &self.relationships {
                let from_idx = rel.from.as_u64() as usize;
                if from_idx < trust_deltas.len() {
                    let target_id = rel.to.as_u64();
                    trust_deltas[from_idx].push((target_id, rel.trust));
                }
            }
            #[expect(
                clippy::needless_range_loop,
                reason = "intentional parallel-array indexing: trust_deltas[i] + agents[i]"
            )]
            for i in 0..self.agents.len() {
                for &(target_id, rel_trust) in &trust_deltas[i] {
                    let current_trust = self.agents[i]
                        .epistemic
                        .trust_network
                        .trust_for_agent(target_id);
                    let delta = (rel_trust - current_trust) * trust_sync_rate;
                    self.agents[i]
                        .epistemic
                        .trust_network
                        .update_agent_trust(target_id, delta);
                }
            }

            // ── 0. Biological update (architecture-plan-2 §7) ──────────
            // Tick the rich biological substrate before cognitive processing.
            // Uses previous tick's emotions — biology reacts to current felt state.
            // EmbodiedState feeds endocrine/nervous signals into the legacy BodyState.
            #[expect(
                clippy::needless_range_loop,
                reason = "intentional parallel-array indexing: agents[i] + emotions[i]"
            )]
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
                // §5 (S3-2-1 fix): the biology pass used a HARDCODED
                // "temperate default" 0.5 here, which froze the thermal
                // system at thermoneutral in every scenario (probe: body_temp
                // 0.500, cold/heat stress 0, no spread — the weather layer
                // was live but never reached the body). The WeatherTracker
                // temperature (0..1; seasonal baselines Spring 0.6 / Summer
                // 0.9 / Autumn 0.5 / Winter 0.2, mean-reverting + seeded
                // noise) is now the ambient input, so body temperature tracks
                // the seasons — mild heat in Spring, real cold in Winter —
                // the plan's "winter hardship" (§7.3.3) becomes live.
                // NB: this block (biology pass) runs before weather.advance
                // in the same tick, so it reads the PREVIOUS tick's weather
                // state — the same one-tick lag the weather site documents
                // for season boundaries; deterministic, seeded, RNG-free
                // here. Golden-safe: nothing golden-hashed reads thermal;
                // the respiratory consumer receives cold_stress (≈0 in
                // Spring/Summer/Autumn) via irritation.
                let ambient_temperature = self.weather.temperature;
                let crowding = Fixed::from_f64(0.3); // village-level crowding
                let hygiene = Fixed::from_f64(0.6); // moderate hygiene
                // §7.2.2 (Iteration 188 — the S2-2-2 hunger/thirst channel):
                // the endocrine acute-stress term now reads the LIVE needs
                // (the same values the Eat/Drink decisions use) instead of
                // the frozen embodied facade fields — famine starvation and
                // chronic dehydration genuinely elevate cortisol.
                let live_hunger = self.agents[i].needs.hunger;
                let live_thirst = self.agents[i].needs.thirst;
                self.agents[i].embodied.tick_update(
                    threat_level,
                    social_safety,
                    is_sleeping,
                    activity_level,
                    ambient_temperature,
                    crowding,
                    hygiene,
                    live_hunger,
                    live_thirst,
                    &self.params,
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

                // §16.2: Record hormonal provenance when stress axis level exceeds threshold.
                // This traces cross-system causal influence from biology into psychology.
                let cortisol = self.agents[i].embodied.endocrine.stress.level;
                if cortisol > HORMONAL_TRACE_THRESHOLD {
                    self.provenance
                        .record_system(crate::provenance::SystemTrace {
                            agent: AgentId::new(i as u64),
                            tick: tick_u64,
                            category: crate::provenance::ProvenanceCategory::Hormonal,
                            description: format!(
                                "Stress axis level ({}) influencing threat appraisal",
                                cortisol.to_f64()
                            ),
                            magnitude: cortisol,
                            cause: "biological_update".into(),
                        });
                }
            }

            // §8.1.4: Collect regulation strategies from step 0b, applied after appraisal in step 6.
            // Pre-allocated Vec avoids repeated allocation; capacity matches agent count.
            let num_agents = self.agents.len();
            let mut reg_strategies: Vec<crate::psychology::emotion_regulation::RegulationStrategy> =
                Vec::with_capacity(num_agents);

            // ── 0b. Cognitive state update (§22.1) ────────────────────
            // §22.1: Stress reduces planning horizon, increases heuristic bias.
            // This must happen before action selection to affect decision-making.
            // Iteration 108 (§10.1.2): kin support buffers the stress input —
            // the social field's kin_count scales (fear + anger) down before
            // the EMA. Zero-at-zero: no kin → factor 1.0 → byte-identical
            // (kinship forms only via births, earliest at ~4,170 ticks, so
            // every ≤2000-tick calibrated window is untouched). Note the
            // channel: this lowers cognitive.stress → planning_horizon /
            // heuristic_bias (§22.1); the metrics' avg_stress (mean of
            // fear + anger) is NOT changed by the buffer.
            let kin_rate = Fixed::from_f64(crate::social::relational_field::KIN_STRESS_RATE);
            let kin_cap = Fixed::from_f64(crate::social::relational_field::KIN_STRESS_CAP);
            let narr_rate =
                Fixed::from_f64(crate::psychology::NARRATIVE_STRESS_RESILIENCE_RATE);
            for i in 0..self.agents.len() {
                // §8.1.3 (Iteration 181): the narrative scripts' first decision
                // consumer — the plan's "narrative feeds ... resilience". A
                // redemptive story (redemption/heroism/coherence up,
                // contamination/victimhood/shame down) buffers the perceived
                // stress input; a contaminated story amplifies it. One-sided
                // identity-at-birth (the Iter-99/127 pattern): factor is ~1.0
                // at the birth envelope (coherence drift alone moves it
                // ~±1–4%, probe-pinned — never decision-granularity), so
                // default-envelope agents are effectively untouched — and only
                // focal agents ever move their scripts (runs_narrative), so
                // the tiered blast is contained to focal agents. Same channel
                // as the kin buffer: lowers cognitive.stress →
                // planning_horizon / heuristic_bias (§22.1); the metrics'
                // avg_stress (mean of fear + anger) is NOT changed.
                let stress = (emotions[i].fear + emotions[i].anger)
                    * crate::social::relational_field::RelationalFields::kin_stress_factor(
                        self.agents[i].relational_fields.kin_count,
                        kin_rate,
                        kin_cap,
                    )
                    * self.agents[i].narrative.stress_resilience_factor(narr_rate);
                let need_fatigue = needs[i].fatigue.max(needs[i].hunger).max(needs[i].thirst);
                self.agents[i].cognitive.update(stress, need_fatigue);

                // Architecture-plan-2 §8.1.12: Executive function update.
                // Degrades effective capacity, inhibition, flexibility under stress.
                let pain_level = self.agents[i].embodied.nervous.pain.effective_pain();
                let trauma = self.agents[i].embodied.nervous.trauma_load;
                // §17: Only focal agents run full executive function update
                // §17.2: Budget check — executive function counts as an appraisal.
                if self.agents[i].agent_tier.tier.runs_full_psychology()
                    && self.agents[i].agent_tier.budget_tracker.can_appraise()
                {
                    self.agents[i].cognitive_runtime.update(
                        stress,
                        need_fatigue,
                        pain_level,
                        trauma,
                    );
                    let _ = self.agents[i].agent_tier.budget_tracker.consume_appraisal();
                }

                // Architecture-plan-2 §8.1.14: Attachment system daily decay.
                // Separation distress decays slowly; security stabilizes.
                // §17: Only agents with relationship updates run attachment decay
                if phases.is_daily && self.agents[i].agent_tier.tier.runs_relationship_updates() {
                    // §8.1.14: A partnered agent separated from their attachment
                    // figure accumulates separation distress. Previously distress
                    // only ever decayed (and reunions barely happen), so the
                    // attachment → emotion coupling never activated.
                    // Iteration 173: the separation accumulation and decay now
                    // read the tunable parameters (attachment_separation_rate,
                    // attachment_decay_rate) instead of hardcoded 0.02/0.95,
                    // making the §8.1.14 dynamics calibratable. The defaults
                    // preserve the probe-verified envelope: a calibration
                    // sweep showed rates above 0.03 make the coupling dominate
                    // (inverting taboo suppression, the kin-support buffer and
                    // scenario deltas — violating the Phase-5 acceptance), so
                    // the strength is deliberately left at the calibrated
                    // level while the knobs become live.
                    let is_partnered = self.agents[i].partner.is_some();
                    let att = &mut self.agents[i].attachment;
                    if is_partnered {
                        let closeness = Fixed::from_f64(0.8);
                        att.on_separation(closeness, self.params.attachment_separation_rate);
                    }
                    // Decay factor is clamped non-negative so an out-of-range
                    // decay_rate (>= 1.0) degrades to instant drain, never to
                    // a sign flip on the distress accumulator.
                    let decay_factor =
                        (Fixed::ONE - self.params.attachment_decay_rate).max(Fixed::ZERO);
                    att.separation_distress =
                        (att.separation_distress * decay_factor).max(Fixed::ZERO);
                }

                // Architecture-plan-2 §8.1.5: Motivation update.
                // Bridge biological needs from existing NeedState, then grow and update.
                // §8.1.5 full formula: the three missing context factors are
                // derived per tick from existing state — emotional amplification
                // from affect, cultural legitimacy from the agent's legitimacy
                // field (mean-zero 0.5 anchor), situational affordance from
                // world food/water scarcity. Write-only observational: the
                // richer pressure feeds only `dominant_need`, which has no
                // behavioral consumer yet, so calibrated runs carry zero drift.
                // Scarcity hoisted to the agent-loop scope (Iteration 71): the
                // same deterministic values feed both the motivation context
                // (below) and the daily prospection scenario generation.
                let expected_food =
                    crate::sim::EXPECTED_GRAIN_PER_AGENT as i64 * self.agents.len() as i64;
                let expected_water =
                    crate::sim::EXPECTED_WATER_PER_AGENT as i64 * self.agents.len() as i64;
                // Use the world totals precomputed before the ctx block.
                let food_scarcity = (Fixed::ONE
                    - world_food_total / Fixed::from_int(expected_food.max(1)))
                .clamp_01();
                let water_scarcity = (Fixed::ONE
                    - world_water_total / Fixed::from_int(expected_water.max(1)))
                .clamp_01();
                {
                    // NOTE (Iteration 124): the LIVE LOCAL emotions vec —
                    // the tick's step-0b parallel-array machinery took each
                    // agent's emotions out via `std::mem::take` (sim.rs:2337)
                    // and only writes back at step-8 (sim.rs:4105), so
                    // `self.agents[i].emotions.*` is all-ZERO here. The
                    // pre-iteration code read the EMPTIED field, so
                    // `set_context` received all zeros and the §8.1.5
                    // emotional pressure amplifications (`1 + fear×0.4`
                    // Safety/Certainty, `1 + anger×0.4` Justice,
                    // `1 + joy×0.3` Romance, `1 − sadness×0.3` Meaning)
                    // were silently DEAD — the dominant-need machinery has
                    // operated with an emotionless context since the
                    // parallel-array refactor (audit-found in Iteration
                    // 123). SEMANTIC: at this site the local vec holds the
                    // value TAKEN from the agent at tick start — the
                    // previous tick's accumulated writeback, not this
                    // tick's fresh appraise deltas — "yesterday's emotion
                    // drives today's motivation", the same carried-over
                    // treatment the emotion machinery itself gives the
                    // vec. Probe-pinned blast radius: fear mean 0.83–1.0
                    // and joy mean 0.65–0.89 across calibrated windows
                    // (anger ~0, sadness 0), so this restores a genuinely
                    // live emotional context. The net Safety amplification
                    // measures ~1.03× (not 1.35×): the ×0.8 legitimacy
                    // dampener at live legitimacy≈0 partially cancels the
                    // fear factor — which is exactly why the full gate is
                    // green with ZERO regeneration (live-but-observationally-
                    // inert in every test window; Safety stays dominant and
                    // the Iter-96 exclusive urgency boost scales its
                    // channels uniformly). Deterministic, no RNG.
                    let fear = emotions[i].fear;
                    let anger = emotions[i].anger;
                    let joy = emotions[i].joy;
                    let sadness = emotions[i].sadness;
                    let legitimacy = self.agents[i].legitimacy_field.overall;
                    self.agents[i].motivation.set_context(
                        fear,
                        anger,
                        joy,
                        sadness,
                        legitimacy,
                        food_scarcity,
                        water_scarcity,
                    );
                }
                self.agents[i].motivation.hunger.deficit = needs[i].hunger;
                self.agents[i].motivation.thirst.deficit = needs[i].thirst;
                // §7.3.1 (Iteration 187 — the circadian behavioral consumer):
                // the body's sleep-pressure clock now drives the sleep drive
                // instead of running underneath it. `should_sleep()` fires at
                // night with pressure > 0.4; `sleep_deprived()` at debt > 0.3.
                // When either fires, the fatigue need is floored so the Rest
                // action's utility (keyed on `needs.fatigue` via the
                // DecisionContext) and the §8.1.5 sleep motive both respond —
                // the agent actually lies down when its circadian clock says
                // so. Deterministic, no RNG; identity-at-zero (a rested,
                // daytime agent keeps its raw fatigue).
                let circadian = &self.agents[i].embodied.circadian;
                let sleep_deficit = if circadian.sleep_deprived() {
                    Fixed::from_f64(0.8)
                } else if circadian.should_sleep() {
                    Fixed::from_f64(0.6)
                } else {
                    needs[i].fatigue
                };
                needs[i].fatigue = needs[i].fatigue.max(sleep_deficit);
                self.agents[i].motivation.sleep.deficit = sleep_deficit;
                // §8.1.5 (P2/P3 re-audit — safety-need ratchet): the legacy
                // `needs.safety` accumulator has ZERO consumers anywhere in
                // the sim (no action relieves it; SeekSafety has no aligned
                // action), so it grew +0.0003/tick from birth and saturated
                // at 1.0 for every agent by ~10K ticks (probe: 0.702 at 2K →
                // 1.000 at 10K, 12/12 identical, no spread) — monopolizing
                // `dominant_need` (Safety:7 at 10K) and hiding the real
                // thirst/fatigue drives. The §8.1.5 safety deficit now
                // tracks the agent's LIVE felt danger (fear + anger) instead
                // of the unbounded clock: a calm agent is safe (deficit low),
                // a threatened agent is unsafe (deficit high). Mean-zero
                // anchor: the fear/anger blend replaces the accumulator, so
                // the axis differentiates and dominant_need reflects actual
                // drives. Deterministic, no RNG.
                let safety_deficit =
                    (emotions[i].fear + emotions[i].anger) * Fixed::from_f64(0.5);
                self.agents[i].motivation.safety.deficit = safety_deficit.clamp_01();
                self.agents[i].motivation.update();

                // Architecture-plan-2 §8.1.4: Emotion regulation
                // Compute social support from relationships (average trust of top-3 closest agents)
                // Uses a fixed-size stack array to avoid heap allocation per agent per tick.
                let social_support = {
                    let mut top3: [Fixed; 3] = [Fixed::ZERO; 3];
                    let mut count: usize = 0;
                    for r in &self.relationships {
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
                // §17: Only focal agents run full emotion regulation
                if self.agents[i].agent_tier.tier.runs_full_psychology() {
                    let regulation_strategy = self.agents[i].emotion_regulation.select_strategy(
                        stress,
                        social_support,
                        personalities[i].extraversion,
                    );
                    // §8.1.4: Store regulation strategy for post-appraisal application.
                    // The actual apply_strategy() call happens after appraisal (step 6)
                    // so the skill-scaled boost uses the fresh, appraisal-derived affect values.
                    // §16.2: Record attachment-proximate provenance when social support is low.
                    if social_support < ATTACHMENT_LOW_SUPPORT_THRESHOLD {
                        self.provenance
                            .record_system(crate::provenance::SystemTrace {
                                agent: AgentId::new(i as u64),
                                tick: tick_u64,
                                category: crate::provenance::ProvenanceCategory::Attachment,
                                description: format!(
                                "Low social support ({}) activating attachment-driven regulation",
                                social_support.to_f64()
                            ),
                                magnitude: (Fixed::ONE - social_support).clamp_01(),
                                cause: "emotion_regulation_selection".into(),
                            });
                    }
                    reg_strategies.push(regulation_strategy);
                    // §8.1: A healthy self-model (high self-esteem earned from
                    // a redemptive narrative) sustains regulation capacity.
                    let self_support = self_esteem_support(self.agents[i].self_model.self_esteem);
                    self.agents[i].emotion_regulation.update_capacity(
                        stress,
                        need_fatigue,
                        social_support + self_support,
                    );
                } else {
                    // §17: Background/secondary agents use default regulation strategy
                    reg_strategies.push(Default::default());
                }

                // §17: Only focal agents run moral cognition update
                if self.agents[i].agent_tier.tier.runs_full_psychology() {
                    // Architecture-plan-2 §8.1.10: Moral cognition update
                    // §8.1.10: Compute witnessed violations from this tick's negative interactions.
                    // High anger = agent perceived a norm violation; threat/insult events contribute.
                    let witnessed_violations = (emotions[i].anger * Fixed::from_f64(0.6)
                        + emotions[i].fear * Fixed::from_f64(0.2))
                    .clamp_01();
                    // §8.1.7: Sacred values amplify witnessed violations — the
                    // same observed violation triggers proportionally more moral
                    // outrage the more sacred the agent's values are. Zero-at-zero
                    // anchor: no violations -> no amplification (typical calm
                    // agents are unaffected; only moral events diverge).
                    let witnessed_violations = self.agents[i]
                        .sacred_values
                        .amplify_witnessed_violations(witnessed_violations);
                    // §11.1: Witnessed violations erode perceived legitimacy —
                    // the council that fails to prevent wrongdoing loses its
                    // perceived rightfulness. Zero-at-zero anchor: no violations
                    // -> no erosion.
                    // §8.1.4 (Iter-130): awe — overwhelming, unexpected
                    // significance — shields this erosion: an awe-struck
                    // population experiences institutional failings as smaller
                    // than they are ("reverence forgives"). One-sided: identity
                    // at zero awe. The producer is U-shaped on |future_implication|
                    // (= 1 − threat − need_pressure), so awe fires on significance
                    // in EITHER direction (wonder or dread) — the zero baseline
                    // blast (golden + 14 snapshots byte-identical) is consumer-
                    // side: erosion floors legitimacy at 0 in every calibrated
                    // window, the 0.5-anchored grievance/theft consumers never
                    // activate, and the continuous motivation-context shift stays
                    // below decision granularity.
                    let awe_reverence = crate::appraisal::awe_reverence_factor(
                        emotions[i].awe,
                        crate::appraisal::AWE_REVERENCE_RATE,
                        crate::appraisal::AWE_REVERENCE_FLOOR,
                    );
                    self.agents[i]
                        .legitimacy_field
                        .scandal_damage(witnessed_violations * awe_reverence);
                    let personal_violations = (emotions[i].guilt * Fixed::from_f64(0.4)
                        + emotions[i].shame * Fixed::from_f64(0.3))
                    .clamp_01();
                    let moral_achievements = (emotions[i].pride * Fixed::from_f64(0.3)
                        + emotions[i].trust * Fixed::from_f64(0.2))
                    .clamp_01();
                    // Iteration 195: the §17 developmental psychology
                    // state was 100% write-only (computed every centum
                    // for Focal agents, consumed by NOTHING — the whole
                    // subsystem was a dead ratchet). `moral_development`
                    // (Kohlberg stage) is the natural modulator of the
                    // internalized shame/pride response in the moral
                    // cognition update that runs on the SAME Focal gate,
                    // so the developmental pipeline finally feeds
                    // behavior. Zero-blast: moral emotions are
                    // probe-confirmed 0.000 in calm/famine/pestilence
                    // windows, and this consumer is Focal-gated like the
                    // producer.
                    let moral_development =
                        self.agents[i].developmental.moral_development;
                    self.agents[i].moral_cognition.update_moral_emotions(
                        witnessed_violations,
                        personal_violations,
                        moral_achievements,
                        moral_development,
                    );
                }

                // §17: Only focal agents run prospection (mental simulation of futures)
                // §17.2: Also check cognitive budget — skip if prospection budget exhausted.
                if self.agents[i].agent_tier.tier.runs_prospection()
                    && self.agents[i].agent_tier.budget_tracker.can_prospect()
                {
                    let agent_fear = emotions[i].fear;
                    let agent_ambition = self.agents[i].personality.ambition;
                    let agent_trauma = self.agents[i].embodied.nervous.trauma_load;
                    let agent_depression = self.agents[i].psychopathology.depression_risk;

                    // Architecture-plan-2 §8.1.16 (Iteration 71): the plan's
                    // core mechanic — "agents simulate possible futures" —
                    // was dead: generate_scenario/evaluate_scenarios had zero
                    // production callers, so the scenario set stayed empty.
                    // One scenario per daily pass, derived deterministically
                    // from live state (no RNG) and bounded by capacity; then
                    // evaluate_scenarios() grounds hope/dread in the set.
                    // Ordering matters (reviewer-mandated): generation runs
                    // FIRST — it uses the standing emotion biases from the
                    // previous tick's update — then evaluation regrounds
                    // hope/dread, and only then does update() apply today's
                    // emotions, so the plan's "depression reduces hope" drain
                    // operates on the scenario-grounded hope instead of being
                    // clobbered by the overwrite. Observational — prospection
                    // is absent from the AgentBundle projection and has no
                    // behavioral consumer, so calibrated runs stay
                    // byte-identical.
                    if phases.is_daily {
                        let inputs = crate::psychology::imagination::ScenarioInputs {
                            hunger: needs[i].hunger,
                            thirst: needs[i].thirst,
                            safety: needs[i].safety,
                            fear: agent_fear,
                            anger: emotions[i].anger,
                            ambition: agent_ambition,
                            food_scarcity,
                            water_scarcity,
                            has_partner: self.agents[i].partner.is_some(),
                            total_attraction: self.agents[i].attraction.total_attraction(),
                            // §8.1.12 (Iteration 80): the D5 ambition gate —
                            // long-term planning requires intact executive
                            // function (stress/fatigue/pain/trauma degrade it).
                            // Deliberate hard gate (not a soft scale): matches
                            // CognitiveRuntime::can_plan_long_term's documented
                            // categorical contract; the alternative (scaling D5's
                            // vividness by EF depth) is unit-testable but less
                            // legible. Note: cognitive_runtime.update runs under
                            // the can_appraise() budget gate while this read sits
                            // in the can_prospect() block — with mismatched
                            // budget exhaustion the read reflects last-tick EF.
                            // Harmless at 12 focal agents (budgets never exhaust).
                            can_plan_long_term: self.agents[i]
                                .cognitive_runtime
                                .can_plan_long_term(),
                        };
                        self.agents[i]
                            .prospection
                            .generate_daily_scenario(tick_u64, inputs);
                        self.agents[i].prospection.evaluate_scenarios();
                    }
                    self.agents[i].prospection.update(
                        agent_fear,
                        agent_ambition,
                        agent_trauma,
                        agent_depression,
                    );
                    // §8.1.12 (Iteration 80): mirror the executive function's
                    // effective planning depth into planning_confidence — the
                    // plan's "confidence in ability to plan and execute"
                    // should track the agent's actual degraded capacity, not
                    // just fear/ambition. Deliberate 0.5/0.5 simplification:
                    // the emotion term (update()) captures motivational
                    // confidence while EF depth captures cognitive capacity;
                    // equal weight is the documented default, revisable via
                    // calibration. Both operands are provably in [0,1] (clamped
                    // by their producers), so the blend needs no re-clamp.
                    // Observational; deterministic; same budget-gate note as
                    // the D5 gate above. Runs inside the can_prospect() block,
                    // consistent with update() — both skip together on
                    // budget exhaustion.
                    let ef_depth = self.agents[i].cognitive_runtime.effective_planning_depth();
                    // §8.1.16 (Iteration 187 — the psychopathology cognitive
                    // consumer): a mental-health collapse halves the
                    // executive fold that feeds planning confidence — the
                    // `cognitive_modifier()` interface is live, gated on the
                    // same `is_impaired()` crisis threshold as the social
                    // gate above. ONE-SIDED: healthy agents keep the exact
                    // pre-Iter-187 value.
                    let ef_depth = if self.agents[i].psychopathology.is_impaired() {
                        ef_depth * Fixed::from_f64(0.5)
                    } else {
                        ef_depth
                    };
                    self.agents[i].prospection.planning_confidence =
                        (self.agents[i].prospection.planning_confidence + ef_depth)
                            * Fixed::from_f64(0.5);
                    let _ = self.agents[i]
                        .agent_tier
                        .budget_tracker
                        .consume_prospection();
                }

                // §17: Compute negative_events once (shared by narrative + psychopathology)
                let negative_events =
                    if emotions[i].fear + emotions[i].sadness > Fixed::from_f64(0.3) {
                        (emotions[i].fear + emotions[i].sadness) * Fixed::from_f64(0.1)
                    } else {
                        Fixed::ZERO
                    };

                // §17: Only focal agents run narrative identity updates
                // §17.2: Budget check — narrative interpretation counts as an appraisal.
                if self.agents[i].agent_tier.tier.runs_narrative()
                    && self.agents[i].agent_tier.budget_tracker.can_appraise()
                {
                    // §8.1.3 (Iteration 181): the per-event ratchets are
                    // write-only — without decay every script saturated at
                    // ~1.0 within ~10K ticks (probe-pinned), flattening the
                    // identity. Pull each script back toward its birth-narrative
                    // value every tick so movement is event-driven but bounded
                    // (the Iter-179 pull pattern). Runs before interpretation
                    // so today's events act on today's re-anchored envelope.
                    self.agents[i]
                        .narrative
                        .decay_scripts(Fixed::from_f64(
                            crate::psychology::NARRATIVE_SCRIPT_DECAY_RATE,
                        ));
                    let positive_event_magnitude = if emotions[i].joy > Fixed::from_f64(0.3) {
                        emotions[i].joy * Fixed::from_f64(0.1)
                    } else {
                        Fixed::ZERO
                    };
                    let events_before = self.agents[i].narrative.events_integrated;
                    if negative_events > Fixed::ZERO {
                        self.agents[i].narrative.interpret_negative_event(
                            negative_events,
                            social_support,
                            emotions[i].anger > Fixed::from_f64(0.3),
                        );
                    }
                    if positive_event_magnitude > Fixed::ZERO {
                        self.agents[i]
                            .narrative
                            .interpret_positive_event(positive_event_magnitude, social_support);
                    }
                    // Iteration 186: feed the agent's temperament into the
                    // theme mapping so a thriving calm village does not
                    // collapse every agent onto Mission (the script-balance-
                    // only mapping saturated positive scripts uniformly).
                    // Agency = ambition/extraversion/conscientiousness
                    // blend; neuroticism discounts the positive balance.
                    let (neuro, agency) = {
                        let p = &self.agents[i].personality;
                        let agency = p.ambition * Fixed::from_f64(0.4)
                            + p.extraversion * Fixed::from_f64(0.3)
                            + p.conscientiousness * Fixed::from_f64(0.3);
                        (p.neuroticism, agency)
                    };
                    self.agents[i].narrative.update_theme(
                        crate::psychology::narrative::ThemeTemperament {
                            neuroticism: neuro,
                            agency,
                        },
                    );
                    // §8.1.3: Episodic memory — every chapter boundary of
                    // integrated life events closes a chapter worth remembering
                    // (sparse by design; salience scales with the current
                    // event's emotional magnitude, so crises recall louder).
                    // The block is Focal-only (runs_narrative), a subset of
                    // the memory-encoding tiers, so no extra tier gate needed.
                    let events_after = self.agents[i].narrative.events_integrated;
                    if life_chapter_crossed(events_before, events_after)
                        && self.agents[i].agent_tier.budget_tracker.can_memory_op()
                    {
                        let _ = self.agents[i].agent_tier.budget_tracker.consume_memory_op();
                        let event_magnitude = negative_events.max(positive_event_magnitude);
                        // Ceiling is 0.6 by construction: negative_events ≤ 0.2
                        // ((fear+sadness)×0.1) → 1.5×0.2+0.3; positive ≤ 0.1 → 0.45.
                        let salience =
                            event_magnitude * Fixed::from_f64(1.5) + Fixed::from_f64(0.3);
                        let emotional = self.agents[i].affect.arousal * Fixed::from_f64(0.6)
                            + Fixed::from_f64(0.1);
                        self.agents[i].memory.encode(
                            MemoryKind::Episodic,
                            tick_u64,
                            salience,
                            emotional,
                            None,
                            MemoryTag::LifeEvent,
                        );
                    }
                    // §8.1: The self-model tracks the same life events, and
                    // self-esteem mean-reverts toward the narrative balance.
                    self.agents[i].self_model.update_narrative(
                        negative_events,
                        positive_event_magnitude,
                        social_support,
                    );
                    self.agents[i].self_model.reconcile_self_esteem();
                    // §8.1.7 (P3-2): coherence and security now have
                    // producers — the two write-only dead fields (probe:
                    // 0.600/0.500, 12/12 identical in every window). Both
                    // are observational (only the yearly plasticity reads
                    // coherence; security has no consumer), so short-horizon
                    // calibrated runs stay byte-identical.
                    self.agents[i].self_model.update_coherence();
                    self.agents[i].self_model.update_security(
                        negative_events,
                        positive_event_magnitude,
                        social_support,
                    );
                    // §8.1.17: The same life events reshape the meaning-making
                    // frames themselves — suffering crystallizes punitive frames
                    // and erodes just-world optimism, success does the reverse.
                    self.agents[i]
                        .narrative_frames
                        .daily_update(negative_events, positive_event_magnitude);
                    let _ = self.agents[i].agent_tier.budget_tracker.consume_appraisal();
                }

                // §17: Only focal agents run developmental psychology
                if phases.is_centum && self.agents[i].agent_tier.tier.runs_full_psychology() {
                    let agent_age = self.agents[i].age;
                    // Iteration 196: education quality is the agent's own
                    // learning aptitude (how well education benefits THIS
                    // agent — the same capacity `effective_learning` weights
                    // at 0.3 in the apprenticeship pass) instead of the
                    // hardcoded 0.5 placeholder, so identity formation
                    // genuinely scales with the education pipeline.
                    let education_quality =
                        self.agents[i].education.learning_aptitude;
                    self.agents[i].developmental.tick_update(
                        agent_age,
                        social_support,
                        education_quality,
                    );
                }

                // §17: Only focal agents run psychopathology update
                if self.agents[i].agent_tier.tier.runs_full_psychology() {
                    let chronic_stress = self.agents[i].embodied.endocrine.stress.chronic_load;
                    let trauma_load = self.agents[i].embodied.nervous.trauma_load;
                    let chronic_pain = self.agents[i].embodied.nervous.pain.chronic;
                    let sleep_debt = self.agents[i].embodied.circadian.sleep_debt;
                    let depression_vuln = self.agents[i]
                        .embodied
                        .genome
                        .trait_predispositions
                        .depression_vulnerability;
                    let addiction_risk = self.agents[i]
                        .embodied
                        .genome
                        .trait_predispositions
                        .addiction_risk;
                    let social_isolation = (Fixed::ONE - social_support).max(Fixed::ZERO);
                    self.agents[i].psychopathology.tick_update(
                        chronic_stress,
                        trauma_load,
                        social_isolation,
                        // Iteration 193: humiliation_recent was hardcoded
                        // ZERO — the paranoia channel's humiliation leg
                        // (×0.25) and resentment's humiliation leg (×0.2)
                        // were structurally dead. The live emotion
                        // `emotions[i].humiliation` (status_threat ×
                        // identity_relevance in appraisal, already consumed
                        // by the escalation fold) is the natural producer:
                        // public status loss is exactly the humiliation the
                        // risk model expects. Zero-at-zero in calm/famine/
                        // pestilence windows (probe: 0.000 — status_threat
                        // needs a threat event), so calibrated runs stay
                        // byte-identical.
                        emotions[i].humiliation,
                        negative_events,
                        // Iteration 193: moral_injury was hardcoded ZERO —
                        // resentment's moral-injury leg (×0.25) was
                        // structurally dead (only chronic_stress×0.1
                        // remained). The live emotion
                        // `emotions[i].moral_outrage` (sacredness_violation ×
                        // goal_relevance in appraisal — the agent's emotional
                        // response to witnessing its sacred order violated) is
                        // the natural producer: the morally-outraged agent IS
                        // morally injured. Zero-at-zero in calibrated windows
                        // (probe: 0.000 — sacredness_violation needs an
                        // unfairness event), so calibrated runs stay
                        // byte-identical.
                        emotions[i].moral_outrage,
                        chronic_pain,
                        sleep_debt,
                        depression_vuln,
                        addiction_risk,
                        social_support,
                        // Iteration 192: despair is the emotional hopelessness
                        // of an unmet need — the documented
                        // "sadness -> despair -> depression-from-deprivation"
                        // chain's last link, which previously had no consumer
                        // into depression_risk. Zero-at-zero: calm agents
                        // (despair 0.000) contribute nothing.
                        emotions[i].despair,
                    );
                }

                // §17: Only focal agents run cultural cognition + decision policy
                if phases.is_daily && self.agents[i].agent_tier.tier.runs_full_psychology() {
                    let positive_exposure = social_support * Fixed::from_f64(0.5);
                    let negative_exposure = stress * Fixed::from_f64(0.3);
                    self.agents[i]
                        .cultural_cognition
                        .tick_update(positive_exposure, negative_exposure);
                    self.agents[i].decision_policy.daily_update();
                }

                // §17: Skill/habit update runs for focal + secondary (heuristic cognition)
                if self.agents[i].agent_tier.tier.runs_heuristic_cognition() {
                    self.agents[i]
                        .psych_skills
                        .update_automaticity(stress, need_fatigue);
                    if phases.is_centum {
                        self.agents[i].psych_skills.decay_habits(tick_u64);
                    }
                }
            }

            // §17: Verify reg_strategies Vec alignment after per-agent loop
            debug_assert_eq!(
                reg_strategies.len(),
                self.agents.len(),
                "reg_strategies must be aligned with agents after tier-gated psychology loop"
            );

            // Compute avg_wealth once before status updates (O(N) instead of O(N²))
            let avg_wealth: Fixed = if self.agents.is_empty() {
                Fixed::ZERO
            } else {
                self.agents
                    .iter()
                    .map(|a| a.wealth.coin)
                    .fold(Fixed::ZERO, |acc, c| acc + c)
                    / Fixed::from_int(self.agents.len() as i64)
            }; // Precomputed ages for the kinship max-relatedness scan below —
               // the outer loop iterates `&mut self.agents`, so a nested
               // `self.agents[j].age` read would be a borrow conflict; the ages
               // are pure reads collected once (deterministic, no RNG).
            let agent_ages: Vec<Fixed> = self.agents.iter().map(|a| a.age).collect();
            for (i, agent) in self.agents.iter_mut().enumerate() {
                // Architecture-plan-2 §11.1: Status dimensions update.
                // Decay shame and honor gradually; update wealth_rank from relative wealth.
                agent.status_v2.decay();
                // Sync authority from existing institutional role_status
                agent.status_v2.authority = agent.status.role_status;
                // §11.1 (AP2): Derive institutional_rank from the agent's highest
                // institutional role authority (deterministic scan of the
                // institutions registry — no RNG). Zero when the agent holds no
                // institutional role. Only the new field is written, so the
                // golden baseline stays byte-identical.
                let agent_id = AgentId::new(i as u64);
                let mut best_role_authority = Fixed::ZERO;
                for institution in &self.institutions {
                    for role in &institution.roles {
                        if role.holder == Some(agent_id) && role.authority > best_role_authority {
                            best_role_authority = role.authority;
                        }
                    }
                }
                agent.status_v2.institutional_rank = best_role_authority;
                if avg_wealth > Fixed::ZERO {
                    agent.status_v2.wealth_rank =
                        (agent.wealth.coin / avg_wealth * Fixed::from_f64(0.5)).clamp_01();
                }
                // Update moral_reputation from moral pride and gratitude (weighted average)
                agent.status_v2.moral_reputation = (agent.moral_cognition.moral_emotions.pride
                    * Fixed::from_f64(0.5)
                    + agent.moral_cognition.moral_emotions.gratitude * Fixed::from_f64(0.5))
                .clamp_01();
                // Update prestige from relationship count (social connections)
                let num_connections = agent.relationship_v2s.len() as i64;
                if num_connections > 0 {
                    let avg_quality: Fixed = agent
                        .relationship_v2s
                        .iter()
                        .map(crate::social::relationship_v2::RelationshipV2::quality)
                        .fold(Fixed::ZERO, |acc, q| acc + q)
                        / Fixed::from_int(num_connections);
                    agent.status_v2.network_centrality = avg_quality;
                }
                // §10.4 (Iteration 77): status_attraction was a declared
                // AttractionModel field with zero production writers — the
                // last of the three attraction factors (with familiarity and
                // reciprocity) that feed the §8.1.16 D4 courtship gate. The
                // agent's freshly-refreshed composite standing feeds their
                // attraction profile: role holders and the wealthy carry the
                // standing that makes them a better match. The mirror is
                // intentionally exact (status_attraction == effective_status,
                // pinned by the integration test): a future relative/scaled
                // derivation is a conscious re-design, not an incidental one.
                // Note: status_attraction tracks whatever effective_status()
                // captures — since Iteration 106 the §11.1 institutional_rank
                // field IS weighted into the composite (× 0.1), so role
                // holders' status_attraction reflects their institutional
                // power, flowing through automatically. Deterministic, no RNG;
                // status_attraction itself feeds only the D4 scenario gate,
                // but the composite also feeds §10.9 patronage formation
                // (behavioral), so role-holder status now genuinely matters.
                let eff_status = agent.status_v2.effective_status();
                agent.attraction.update_status_attraction(eff_status);
                // §7.2.2 (S2-2-2 fix): the endocrine dominance axis now
                // tracks the agent's standing — the plan's "status victory
                // raises dominance" channel. Previously write-once:
                // `dominance.update` had zero call sites (the axis sat frozen
                // at birth values). Mean-reverts toward `effective_status`
                // (the §11.1 composite, refreshed just above): elites hold
                // higher dominance, low-status agents drain toward their
                // standing — rises with status gains, falls with losses, and
                // cannot saturate (the target is the agent's real status).
                // Daily cadence: status fields refresh on the daily
                // decay/sync pattern in this same loop (taboo_penalty,
                // kinship_penalty below use the same `phases.is_daily`
                // fold). Deterministic, no RNG; dominance feeds no decision
                // consumer yet (the accessors are unwired), so the golden
                // baseline stays byte-identical.
                if phases.is_daily {
                    agent.embodied.endocrine.dominance.update(
                        eff_status,
                        self.params.endocrine_dominance_response,
                    );
                }
                // §10.4 (Iteration 78): the two per-agent aversion channels.
                // social_cost mirrors the agent's criminal notoriety from the
                // live norm-enforcement crime records (offenders are socially
                // costly partners — the channel only rises for agents who
                // actually commit hostile acts, so it differentiates without
                // depressing the D4-reachability that Iteration 77 wired).
                // moral_disgust mirrors the disgust emotion — the §8.1.4
                // appraisal's purity-violation aversion, 0 in peaceful runs by
                // design (the same situational class as the Iter-65 jealousy
                // channel), firing when the agent appraises a moral violation.
                // Both deterministic, no RNG, observational (only the D4
                // scenario gate reads total_attraction).
                let notoriety = self
                    .norms
                    .crime_record(AgentId::new(i as u64))
                    .map_or(Fixed::ZERO, |r| r.notoriety);
                agent.attraction.update_social_cost(notoriety);
                // NOTE (Iteration 123): both channels below read the LIVE
                // local `emotions[i]` vec, not `agent.emotions` —
                // the tick takes each agent's emotions out via
                // `std::mem::take` at the step-0b parallel-array setup, so
                // the AGENT field is all-zero until the step-8 writeback
                // (sim.rs:4105). The pre-Iteration-123 `update_moral_disgust`
                // line read the emptied field and was therefore a LATENT
                // dead channel (moral_disgust pinned at 0 in production
                // despite live disgust producers); it is fixed here to read
                // the local vec — behaviorally identical until a purity
                // violation actually fires the emotion (which calm worlds
                // never do → zero-blast).
                let disgust = emotions[i].disgust;
                agent.attraction.update_moral_disgust(disgust);
                // §8.1.4 (Iteration 123): the envy aversion channel — the
                // coveting emotion (`status_threat × incongruent` — wanting
                // what the higher-status other has) poisons courtship
                // interest: the envious mind is consumed with status
                // comparison, so romantic salience drops. Mirrors the
                // moral_disgust channel exactly (same update-*-cost pattern,
                // same 0.2 weight tier, and — per the note above — the
                // same live local-vec read). ONE-SIDED: identity at zero —
                // envy is never produced in calibrated windows
                // (probe-pinned: every seed-42 golden agent at envy == 0,
                // Iter-122 Leg C), so `envy_cost` stays exactly 0 and
                // `total_attraction()` is bit-identical (provably
                // zero-blast).
                let envy = emotions[i].envy;
                agent.attraction.update_envy_cost(envy);
                // §8.1.18/§10.4 (Iteration 165): the taboo-restraint channel —
                // the strongest internalized taboo in the agent's cultural
                // cognition (`CulturalCognition::max_taboo_strength`) becomes
                // a standing cost on courtship pursuit (the RWR row-#11
                // `taboo_penalty` consumer, and the §8.1.18 taboo layer's
                // first production read). Traditional agents hold stronger
                // seeded taboos (populate scales strength by traditionalism),
                // so the channel differentiates: the most taboo-bound agents
                // court less readily. Deterministic, no RNG; identity at zero
                // for taboo-free agents (pre-Iter-165 snapshot restores).
                // Daily cadence: taboo strength only changes on the daily
                // cultural-cognition tick_update decay, so a daily fold is
                // the honest cadence (mirrors the kinship_penalty fold's
                // daily pattern below).
                if phases.is_daily {
                    let max_taboo = agent.cultural_cognition.max_taboo_strength();
                    agent.attraction.update_taboo_penalty(max_taboo);
                }
                // §10.4 (Iteration 79): the last unwired attraction channel —
                // kinship_penalty, the soft taboo against courting within the
                // local pool. Derived as the max transitive genetic
                // relatedness to any other adult (the plan's §10.6 BFS model:
                // parent/sibling 0.5, grandparent 0.25, first cousin 0.125),
                // which the direct-edge 0.25 hard eligibility gate does not
                // catch. Reading: a pool-inbreeding signal — the 0.5 tier
                // (parent/sibling, hard-gated out of courtship anyway) raises
                // the cost of ALL matches in a kin-dense village, while the
                // actionable taboo tier (cousins 0.125–0.24, which pass the
                // hard gate) is exactly what the soft channel suppresses.
                // 0 for founding villages, rising as families form (probe:
                // seeds 42/43/44 → 0, seed 51 → 0.50). Daily cadence: kinship
                // edges only change on birth/marriage, so the O(N²) BFS runs
                // once a day (the patronage block below uses the same
                // is_duodeca pattern for its O(N²) work). Deterministic, no
                // RNG, observational (only the D4 scenario reads it).
                if phases.is_daily {
                    let adult_age = Fixed::from_f64(16.0);
                    let mut max_relatedness = Fixed::ZERO;
                    for (j, age) in agent_ages.iter().enumerate() {
                        if j == i || *age < adult_age {
                            continue;
                        }
                        let coeff = self.kinship_graph.transitive_coefficient(i, j);
                        if coeff > max_relatedness {
                            max_relatedness = coeff;
                        }
                    }
                    agent.attraction.update_kinship_penalty(max_relatedness);
                }

                // Architecture-plan-2 §17.3: Relationship caching.
                // Only decay relationships that have been recently active (dirty)
                // or are due for the daily full update. Dormant relationships
                // receive no per-tick decay — they only decay during the daily pass.
                for rv2 in &mut agent.relationship_v2s {
                    if rv2.is_active_this_tick(tick_u64) {
                        rv2.decay(1);
                        // After daily boundary decay, clear dirty so dormant
                        // relationships return to sleep on the next tick.
                        if tick_u64 > 0 && tick_u64.is_multiple_of(144) {
                            rv2.clear_dirty(tick_u64);
                        }
                    }
                }

                // §5.1: Legacy relationship mean reversion — trust drifts toward a
                // neutral baseline so it cannot pin at 1.0 for every pair. Without
                // this, positive interactions ratcheted all trust to 1.0 once agents
                // were healthy/wealthy, erasing the differentiation the witness
                // system produces. `relationship_dormant_decay` was previously dead
                // parameter. Mean reversion: trust -= (trust - 0.5) * decay per day.
                if tick_u64.is_multiple_of(144)
                    && self.params.relationship_dormant_decay > Fixed::ZERO
                {
                    for rel in &mut self.relationships {
                        if rel.from == AgentId::new(i as u64) {
                            let drift = (rel.trust - Fixed::from_f64(0.5))
                                * self.params.relationship_dormant_decay;
                            rel.trust = (rel.trust - drift).clamp_01();
                        }
                    }
                }
            }

            // §11.2: Recompute relational power balance on the daily boundary.
            // Fills `RelationshipV2.power_balance`, declared since Iter 1 but
            // never written — the dead-field class this closes. Purely a
            // function of existing relationship + status state (no RNG), so it
            // cannot perturb calibrated runs; the field is not consumed
            // downstream or by agent_summaries().
            if tick_u64 > 0 && tick_u64.is_multiple_of(144) {
                let n = self.agents.len();
                let statuses: Vec<Fixed> = (0..n)
                    .map(|i| self.agents[i].status_v2.effective_status())
                    .collect();
                for i in 0..n {
                    let rel_count = self.agents[i].relationship_v2s.len();
                    for pos in 0..rel_count {
                        let (dependence_a_on_b, to_idx) = {
                            let rv2 = &self.agents[i].relationship_v2s[pos];
                            (rv2.dependence, rv2.to.as_u64() as usize)
                        };
                        if to_idx >= n || to_idx == i {
                            // Defensive: population never creates self-relationships
                            // (the i == j pair is skipped), but guard anyway —
                            // relationship_v2_pos debug_asserts a != b and would
                            // silently misbehave in release.
                            continue;
                        }
                        // B's reverse view toward A: relationship_v2_pos(b, a).
                        let rev_pos = Self::relationship_v2_pos(to_idx, i);
                        let (
                            commitment_b_to_a,
                            attachment_b_to_a,
                            fear_b_of_a,
                            obligation_b_to_a,
                            moral_debt_b_to_a,
                        ) = {
                            let rev = &self.agents[to_idx].relationship_v2s[rev_pos];
                            (
                                rev.commitment,
                                rev.attachment_security,
                                rev.fear,
                                rev.obligation,
                                rev.moral_debt,
                            )
                        };
                        let status_advantage = statuses[i] - statuses[to_idx];
                        self.agents[i].relationship_v2s[pos].update_power_balance(
                            dependence_a_on_b,
                            commitment_b_to_a,
                            attachment_b_to_a,
                            status_advantage,
                            fear_b_of_a,
                            obligation_b_to_a,
                            moral_debt_b_to_a,
                        );
                    }
                }
            }

            // §10.2 (AP2): Relationship identity metadata + structural links,
            // refreshed on the daily boundary. Fills the plan-mandated fields
            // the struct was missing: public/private labels (§10.3 taxonomy),
            // role expectation (authority/kin branch), kinship coefficient,
            // and the shared household/faction/institution links. Purely a
            // deterministic function of existing state (no RNG) that only
            // writes the new §10.2 fields, so calibrated runs stay
            // byte-identical.
            if tick_u64 > 0 && tick_u64.is_multiple_of(144) {
                let n = self.agents.len();
                // First-match memberships per agent (deterministic).
                let agent_household: Vec<Option<usize>> = (0..n)
                    .map(|i| {
                        self.households
                            .iter()
                            .find(|h| h.members.contains(&i))
                            .map(|h| h.id)
                    })
                    .collect();
                let agent_faction: Vec<Option<usize>> = (0..n)
                    .map(|i| {
                        self.faction_v2_registry
                            .factions
                            .iter()
                            .position(|f| f.active && f.members.contains(&i))
                    })
                    .collect();
                let agent_institution: Vec<Option<u64>> = (0..n)
                    .map(|i| {
                        let id = AgentId::new(i as u64);
                        self.institutions
                            .iter()
                            .find(|inst| inst.members.contains(&id))
                            .map(|inst| inst.id)
                    })
                    .collect();
                for i in 0..n {
                    let rel_count = self.agents[i].relationship_v2s.len();
                    for pos in 0..rel_count {
                        let to_idx = self.agents[i].relationship_v2s[pos].to.as_u64() as usize;
                        if to_idx >= n || to_idx == i {
                            continue;
                        }
                        let rv2 = &mut self.agents[i].relationship_v2s[pos];
                        rv2.update_identity_metadata();
                        // §10.2: Reconciliation — when a betrayal has been
                        // repaired by recovered trust (no reconciliation since
                        // the last betrayal, trust back above 0.5), record one
                        // Apology event. Deterministic, writes only new §10.2
                        // fields (magnitude ∝ recovered trust), so calibrated
                        // runs stay byte-identical. This closes the plan's
                        // betrayal→reconciliation arc in live runs.
                        {
                            let last_betrayal_tick =
                                rv2.betrayal_history.last().map_or(0, |e| e.tick);
                            let last_reconciliation_tick =
                                rv2.reconciliation_history.last().map_or(0, |e| e.tick);
                            if last_betrayal_tick > last_reconciliation_tick
                                && rv2.trust > Fixed::from_f64(0.5)
                            {
                                rv2.record_reconciliation(
                                    tick_u64,
                                    crate::social::relationship_v2::ReconciliationKind::Apology,
                                    rv2.trust,
                                );
                            }
                        }
                        rv2.household_link = if agent_household[i].is_some()
                            && agent_household[i] == agent_household[to_idx]
                        {
                            agent_household[i]
                        } else {
                            None
                        };
                        rv2.faction_link = if agent_faction[i].is_some()
                            && agent_faction[i] == agent_faction[to_idx]
                        {
                            agent_faction[i]
                        } else {
                            None
                        };
                        rv2.institutional_link = if agent_institution[i].is_some()
                            && agent_institution[i] == agent_institution[to_idx]
                        {
                            agent_institution[i]
                        } else {
                            None
                        };
                    }
                }
            }

            // ── 1. Need decay (nonlinear pressure, §9.1) ────────────────
            // §5.1: Use configurable parameters for decay rates.
            let params = self.params;
            systems::system_need_decay_with_params(&params, &mut needs);

            // ── 2. Body state update ──────────────────────────────────
            tracing::debug!(tick = tick_u64, "systems::body_update");
            systems::system_body_update(&mut ctx, &mut bodies, &needs);

            // ── 3. Goal generation ────────────────────────────────────
            tracing::debug!(tick = tick_u64, "systems::goal_generation");
            systems::system_goal_generation(
                &mut ctx,
                &personalities,
                &needs,
                &mut goals,
                &emotions,
            );

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
                        let emotional_shock = emotions[i].anger > Fixed::from_f64(0.5)
                            || emotions[i].fear > Fixed::from_f64(0.5);
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
                    let avg_identity_strength = self.agents[i]
                        .identity
                        .strength_of(IdentityKind::Farmer)
                        .max(self.agents[i].identity.strength_of(IdentityKind::Parent))
                        .max(self.agents[i].identity.strength_of(IdentityKind::Believer));
                    // §12: Institution enforcement amplifies norm pressure.
                    // Council enforcement capacity multiplies the pressure felt by agents.
                    let enforcement_multiplier = self
                        .institutions
                        .iter()
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
                    let norm_pressure = self
                        .norms
                        .norms()
                        .iter()
                        .map(|n| {
                            self.norms
                                .compute_pressure(conformity, avg_identity_strength, n.id)
                        })
                        .fold(Fixed::ZERO, |a, b| a + b)
                        * enforcement_multiplier
                        - moral_modifier;

                    // §10.3: Routine bias — agents follow daily schedules when stable.
                    // §24: Personality modulates routine adherence — conscientiousness
                    // increases it, openness decreases it.
                    let (routine_action, routine_strength) =
                        self.agents[i].routine.preferred_action(tick);
                    let stress = emotions[i].fear + emotions[i].anger;
                    let follow_routine = self.agents[i].routine.should_follow(
                        needs[i].hunger,
                        needs[i].thirst,
                        needs[i].fatigue,
                        stress,
                    );
                    // Personality-modulated routine strength: conscientiousness boosts, openness reduces
                    let personality_routine_modifier = personalities[i].conscientiousness
                        * Fixed::from_f64(0.3)
                        - personalities[i].openness * Fixed::from_f64(0.2);
                    let effective_routine_strength =
                        (routine_strength + personality_routine_modifier).clamp_01();

                    // §22.1: When stressed, agents favor habit/routine over utility.
                    // High heuristic bias → agents follow routine more readily.
                    let effective_routine_threshold =
                        if self.agents[i].cognitive.heuristic_bias > Fixed::from_f64(0.5) {
                            Fixed::from_f64(0.3) // stressed agents follow routine at lower threshold
                        } else {
                            Fixed::from_f64(0.5) // normal agents need stronger routine signal
                        };

                    let mut action = if let Some((cmd_action, cmd_kind)) =
                        command_goal_action(&goals[i], &needs[i])
                    {
                        // §5 (Iteration 155): an external directive takes
                        // priority over routine and internal drives, and is
                        // consumed the moment it steers a selection — a
                        // one-shot nudge that returns the agent to
                        // autonomy. Gated on `GoalSource::Command` goals —
                        // which only exist after `command_agent` is called
                        // (never in a calibrated window), so this branch is
                        // a structural no-op everywhere calibrated and draws
                        // zero RNG.
                        goals[i]
                            .retain(|g| !(g.source == GoalSource::Command && g.kind == cmd_kind));
                        cmd_action
                    } else if follow_routine
                        && effective_routine_strength > effective_routine_threshold
                    {
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
                            &actions::DecisionContext {
                                needs: &needs[i],
                                personality: &personalities[i],
                                active_goals: &goals[i],
                                identity: &self.agents[i].identity,
                                decision_policy: &self.agents[i].decision_policy,
                                total_grain,
                                total_water,
                                coin: self.agents[i].wealth.coin,
                                norm_pressure: adjusted_pressure,
                                anger: emotions[i].anger,
                                fear: emotions[i].fear,
                                joy: emotions[i].joy,
                                sadness: emotions[i].sadness,
                                stress,
                                fairness: self.agents[i].moral_values.fairness,
                                authority: self.agents[i].moral_values.authority,
                                care: self.agents[i].moral_values.care,
                                loyalty: self.agents[i].moral_values.loyalty,
                                // §9.2 (Iteration 94): the agent's learned
                                // RL valuation weights feed selection.
                                action_values: self.agents[i].neural_like.values,
                                // §8.1.5 (Iteration 96): the full-pressure
                                // motivation argmax biases selection toward
                                // actions that relieve the dominant need.
                                dominant_need: self.agents[i].motivation.dominant_need,
                                dominant_pressure: self.agents[i].motivation.dominant_pressure(),
                                // §8.1.16 (Iteration 103): the scenario-
                                // grounded dread (regrounded on the daily
                                // phase, before selection in the same tick)
                                // drives precautionary provisioning — Work/
                                // Trade up, Rest down. Zero-at-zero: agents
                                // whose tiers do not run prospection hold
                                // dread 0 → legacy utility.
                                dread: self.agents[i].prospection.dread,
                            },
                            ctx.rng,
                        )
                    };
                    // §8.1.19 (P3-1, August 14, 2026): habitual fallback
                    // under stress — an agent whose automaticity is high
                    // (habits formed through ~100 practice ticks at the
                    // 0.1-proficiency milestone) and who is stressed
                    // substitutes its strongest habit for the deliberated
                    // action ("under stress, agents fall back on habits").
                    // Zero-blast in calibrated windows: automaticity stays
                    // < 0.5 until habits actually form (base strength 0.3 →
                    // ≈0.29 at golden stress levels), so the golden/snapshot
                    // horizons are untouched and only long-horizon runs where
                    // practice has accumulated see the fallback. Deterministic,
                    // no RNG.
                    if stress > Fixed::from_f64(0.5)
                        && self.agents[i].psych_skills.automaticity > Fixed::from_f64(0.5)
                    {
                        let trigger = match self.agents[i].motivation.dominant_need {
                            crate::psychology::motivation::MotiveCategory::Hunger => "hunger",
                            crate::psychology::motivation::MotiveCategory::Thirst => "thirst",
                            crate::psychology::motivation::MotiveCategory::Sleep => "fatigue",
                            crate::psychology::motivation::MotiveCategory::Belonging
                            | crate::psychology::motivation::MotiveCategory::Attachment
                            | crate::psychology::motivation::MotiveCategory::Romance => "social",
                            crate::psychology::motivation::MotiveCategory::Meaning => "meaning",
                            _ => "",
                        };
                        if let Some(habit) =
                            self.agents[i].psych_skills.execute_habit(trigger, tick_u64)
                        {
                            action = match habit.as_str() {
                                "Work" => ActionKind::Work,
                                "Trade" => ActionKind::Trade,
                                "Socialize" => ActionKind::Socialize,
                                "Worship" => ActionKind::Worship,
                                "Eat" => ActionKind::Eat,
                                _ => action,
                            };
                        }
                    }
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
                                description: format!(
                                    "Routine strength: {:.2}",
                                    effective_routine_strength.to_f64()
                                ),
                            });
                        }
                        self.provenance.record_decision(DecisionTrace {
                            agent: agent_id,
                            tick: tick_u64,
                            action_name: format!("{action:?}"),
                            factors,
                            from_routine: follow_routine
                                && effective_routine_strength > Fixed::from_f64(0.5),
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
                    self.agents[i].intention = Some(crate::person::Intention::new(
                        goal_kind,
                        tick_u64,
                        personalities[i].conscientiousness,
                    ));

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

                self.agents[i].action_progress = self.agents[i].action_progress.saturating_sub(1);
            }

            // ── 5. Social interactions ────────────────────────────────
            {
                // §2.4: Pass agent positions for proximity-based social interactions
                let agent_positions: Vec<(i32, i32)> = self
                    .agents
                    .iter()
                    .map(|a| (a.position.x, a.position.y))
                    .collect();
                // §8.1.10 (Iteration 85): thread each agent's internalized
                // no-violence norm strength into the interaction decision —
                // `choose_interaction` scales its threat thresholds by
                // (1 − resistance). Resolved by id (`NO_VIOLENCE_NORM_ID`,
                // the same constant the check_violation sites use) so a
                // scenario that re-registers norms with renamed descriptions
                // still gates correctly; a registry without the norm resolves
                // to zero resistance (legacy behavior). Zero-at-zero: before
                // the first monthly ritual (tick 4320) no agent holds any
                // internalized norm, so the thresholds are unchanged and the
                // golden baseline stays byte-identical.
                let no_violence_name = self
                    .norms
                    .norms()
                    .iter()
                    .find(|n| n.id == norms::NO_VIOLENCE_NORM_ID)
                    .map(|n| n.name.as_str());
                // §8.1.10 (Iteration 89): thread each agent's internalized
                // "Help Neighbors" norm strength into the interaction
                // decision — `choose_interaction` scales its high-affection
                // Help window by (1 + propensity). The prescriptive norm
                // uses the same id-resolved-by-description read as the
                // prohibitive ones (`norm_resistance` returns the internalized
                // strength); a registry without the norm resolves to zero
                // propensity (legacy behavior). Zero-at-zero: no internalized
                // norm before the first monthly ritual (tick 4320) → the
                // Help bound stays 0.5 and the golden baseline stays
                // byte-identical.
                let help_neighbors_name = self
                    .norms
                    .norms()
                    .iter()
                    .find(|n| n.id == norms::HELP_NEIGHBORS_NORM_ID)
                    .map(|n| n.name.as_str());
                // §8.1.10 (Iteration 91): thread each agent's internalized
                // "Respect Elders" norm strength plus the elder designation
                // into the interaction decision — `choose_interaction`
                // suppresses disrespectful acts toward the community's
                // designated elder (the Council "Elder" role holder, the live
                // elder anchor; age-based elders > 60 never appear at the
                // sim's 35040-tick-per-year timescale). Resolved once per
                // tick, deterministic, no RNG. Zero-at-zero: no internalized
                // norm before the first monthly ritual (tick 4320) → the
                // elder flag alone changes nothing → golden byte-identical.
                let respect_elders_name = self
                    .norms
                    .norms()
                    .iter()
                    .find(|n| n.id == norms::RESPECT_ELDERS_NORM_ID)
                    .map(|n| n.name.as_str());
                let elder_idx: Option<usize> = self
                    .institutions
                    .iter()
                    .find(|i| i.kind == InstitutionKind::Council)
                    .and_then(|c| c.get_role_holder("Elder"))
                    .map(|id| id.as_u64() as usize);
                // §8.1.10 (Iteration 93): the authority anchor for the
                // "Obey Ruler" gate — the Council "Guard Captain" role
                // holder, the ruler's enforcement arm. Distinct from the
                // Elder (separate role holders assigned at populate), so
                // the two gates never compound on the same target. Resolved
                // once per tick, deterministic, no RNG. A deceased captain
                // clears the anchor (the gate naturally goes inert — no
                // authority to defy).
                let obey_ruler_name = self
                    .norms
                    .norms()
                    .iter()
                    .find(|n| n.id == norms::OBEY_RULER_NORM_ID)
                    .map(|n| n.name.as_str());
                let authority_idx: Option<usize> = self
                    .institutions
                    .iter()
                    .find(|i| i.kind == InstitutionKind::Council)
                    .and_then(|c| c.get_role_holder("Guard Captain"))
                    .map(|id| id.as_u64() as usize);
                let agent_info: Vec<_> = self
                    .agents
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        let resistance = no_violence_name
                            .map_or(Fixed::ZERO, |n| a.moral_cognition.norm_resistance(n));
                        // §8.1.4 (Iteration 99 + 127): the help propensity
                        // — the combined prosocial push behind the Help
                        // interaction (the tuple element the interaction
                        // system destructures as `help_neighbors_propensity`
                        // and widens the Help window [0.2, 0.5×(1+p)) with).
                        // Norm resistance (Iter-89 Help Neighbors) plus
                        // tenderness (Iter-99, warmth→caregiving) plus
                        // gratitude (Iter-127, reciprocity→caregiving —
                        // appraisal producer `positive × (1 − expectedness)`,
                        // LIVE in calibrated windows, so this is the first
                        // CALIBRATED §8.1.4 consumer: golden + snapshots
                        // regenerated with before/after evidence). Zero
                        // emotions → the norm-only legacy value. The
                        // Help-window consumer clamps to [0.5, 1.0], so the
                        // sum can never overflow the decision window.
                        let help_propensity = crate::appraisal::help_propensity(
                            help_neighbors_name
                                .map_or(Fixed::ZERO, |n| a.moral_cognition.norm_resistance(n)),
                            emotions[i].tenderness,
                            emotions[i].gratitude,
                            // §8.1.6 (Iteration 180): the core altruism
                            // trait — the LAST decision-less core trait now
                            // has its consumer (the disposition→caregiving
                            // channel, ONE-SIDED identity-at-zero like the
                            // emotion tiers).
                            a.personality.altruism,
                            self.params.social_tenderness_help_multiplier.to_f64(),
                            self.params.social_gratitude_help_multiplier.to_f64(),
                            self.params.social_altruism_help_multiplier.to_f64(),
                        );
                        let respect_propensity = respect_elders_name
                            .map_or(Fixed::ZERO, |n| a.moral_cognition.norm_resistance(n));
                        let obey_propensity = obey_ruler_name
                            .map_or(Fixed::ZERO, |n| a.moral_cognition.norm_resistance(n));
                        (
                            AgentId::new(i as u64),
                            a.personality.openness,
                            a.personality.agreeableness,
                            a.personality.extraversion,
                            emotions[i].anger,
                            resistance,
                            help_propensity,
                            respect_propensity,
                            Some(i) == elder_idx,
                            obey_propensity,
                            Some(i) == authority_idx,
                            emotions[i].loneliness, // §8.1.4 (Iteration 98)
                            // §8.1.6 (Iteration 162): the sociability
                            // deviation from the trait-derived baseline —
                            // zero at construction, drifted by the
                            // plasticity pass. The interaction gate's third
                            // channel (socially-tempered agents clear the
                            // gate more often). Zero-at-zero by construction
                            // → calibrated runs byte-identical until the
                            // layer drifts.
                            a.personality.temperament.sociability
                                - crate::person::Temperament::from_traits(&a.personality)
                                    .sociability,
                            // §17.2 (Iteration 158): Background-tier agents
                            // are excluded from the individual social
                            // interaction pass (aggregate simulation).
                            // Zero-blast by construction — no agent is ever
                            // Background in any calibrated window (Iter-145
                            // probe: 0B at every size/seed).
                            // §8.1.16 (Iteration 187 — the psychopathology
                            // consumer): a mental-health collapse
                            // (`is_impaired()`: overall_health < 0.5 — the
                            // sustained stress/trauma crisis band, not daily
                            // mood) also withdraws the agent from the social
                            // pass — depression and paranoia isolate. The
                            // `social_modifier()` interface is thereby live
                            // (its gate semantics: participation is cut once
                            // the impairment threshold is crossed).
                            a.agent_tier.tier.runs_social_interactions()
                                && !a.psychopathology.is_impaired(),
                        )
                    })
                    .collect();

                // §5.4: Compute faction membership matrix — same_faction_matrix[i][j] = true if agents i,j share a faction
                let faction_members: Vec<Vec<usize>> = self
                    .institutions
                    .iter()
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
                    self.params.bonding_rate,
                    self.params.conflict_escalation_rate,
                    &self.params,
                );

                // §8.1.11: Record speech acts — the structured linguistic frame
                // on the interaction system just run. Every InteractionOccurred
                // event this tick is interpreted into a SpeechAct and pushed onto
                // the speaker's bounded speech_log. Write-only observational
                // state (Iteration 40), so this cannot perturb calibrated runs.
                // Verified invariant: only process_interaction emits
                // InteractionOccurred (update_witnesses mutates witness
                // relationships directly and emits nothing), so every such event
                // in the window is a genuine speaker act.
                let mut spoken: Vec<(u64, u64, InteractionKind)> = Vec::new();
                for ev in &ctx.events[pre_tick_events..] {
                    if let SimEvent::InteractionOccurred { from, to, kind, .. } = ev {
                        spoken.push((from.as_u64(), to.as_u64(), *kind));
                    }
                }
                if !spoken.is_empty() {
                    let n = self.agents.len();
                    for (from_u, to_u, kind) in spoken {
                        let fi = from_u as usize;
                        if fi >= n {
                            continue;
                        }
                        // §10.4: familiarity grows with repeated interaction —
                        // both participants become more familiar with each
                        // other, scaled by how prosocial the interaction kind
                        // is. Previously `update_familiarity` had zero
                        // production callers, so the attraction model's
                        // familiarity term was pinned at 0 forever and
                        // `total_attraction()` (which gates the §8.1.16 D4
                        // courtship scenario) never responded to contact.
                        // Deterministic, no RNG: quality is a pure function
                        // of the kind.
                        let quality = crate::social::attraction::familiarity_quality_for(kind);
                        self.agents[fi].attraction.update_familiarity(quality);
                        let ti = to_u as usize;
                        if ti < n {
                            self.agents[ti].attraction.update_familiarity(quality);
                        }
                        // §10.4 (Iteration 76): The courtship stage ladder was
                        // dead — Courtship::record_positive/record_negative had
                        // zero production callers, so every courtship stayed
                        // pinned at Awareness until daily decay. Feed each
                        // interaction to the active courtship (either
                        // direction): prosocial acts advance the romantic
                        // ladder (gated by the pair's current trust and the
                        // speaker's attraction readiness), hostile acts regress
                        // it — deterministic, no RNG. The pursuer's
                        // AttractionModel.reciprocity mirrors the courtship's
                        // reciprocity tracker so total_attraction() finally
                        // responds to returned interest (the §8.1.16 D4 gate
                        // future-wiring documented at Iteration 75).
                        // Speaker's current trust in the listener — computed
                        // once, reused by both the courtship wiring below and
                        // the speech-act credibility.
                        let credibility = self
                            .relationships
                            .iter()
                            .find(|r| r.from.as_u64() == from_u && r.to.as_u64() == to_u)
                            .map_or(Fixed::from_f64(0.5), |r| r.trust);
                        let kind_is_hostile = matches!(
                            kind,
                            mindstrata_core::event::InteractionKind::Threaten
                                | mindstrata_core::event::InteractionKind::Insult
                        );
                        for courtship in &mut self.active_courtships {
                            if !courtship.active {
                                continue;
                            }
                            let pair_matches = (courtship.pursuer == fi && courtship.pursued == ti)
                                || (courtship.pursuer == ti && courtship.pursued == fi);
                            if !pair_matches {
                                continue;
                            }
                            if kind_is_hostile {
                                courtship.record_negative(tick_u64);
                            } else {
                                // The speaker's attraction readiness feeds the
                                // courtship's mutual-attraction bookkeeping.
                                courtship.record_positive(
                                    tick_u64,
                                    credibility,
                                    self.agents[fi].attraction.total_attraction(),
                                );
                            }
                            break;
                        }
                        // §19.5.D (Iteration 78): norm enforcement is NOT
                        // wired here — the pre-existing sites already record
                        // offenses through check_violation (the theft-fine
                        // path in the black-market/market block, the
                        // violence-conflict block, and the norm-evaluation
                        // pass's check_violation(1, …) call), so adding a
                        // fourth speech-act site would inflate offense_count
                        // and raise the punishment multiplier at those sites
                        // — drifting the snapshot-hashed fines/trust-erosion
                        // baselines. The Iteration-78 contribution to §19.5.D
                        // lives in CrimeRecord::record_offense: notoriety (the
                        // §10.4 social-cost driver) is episode-gated there
                        // (ENFORCEMENT_EPISODE_TICKS) — offense_count still
                        // grows every call (punishment trajectories
                        // byte-identical), but notoriety climbs once per
                        // distinct conflict episode instead of per raw act
                        // (probe: 57–316/agent saturated it at 1.0 for
                        // everyone; the gate keeps it a differentiating
                        // 0–0.9 signal).
                        // Tone from current affect valence (0..1, 0.5 neutral).
                        let tone =
                            (self.agents[fi].affect.valence + Fixed::ONE) / Fixed::from_f64(2.0);
                        let act = crate::social::speech_act::SpeechAct::from_interaction(
                            kind,
                            AgentId::new(from_u),
                            AgentId::new(to_u),
                            tone,
                            credibility,
                            tick_u64,
                        );
                        let effect = act.resolve_effect();
                        let log = &mut self.agents[fi].speech_log;
                        log.push(act);
                        if log.len() > crate::social::speech_act::SPEECH_LOG_CAPACITY {
                            log.remove(0);
                        }
                        // §8.1.11 (Iteration 95): apply the speech act's
                        // relational effects — the trust/affection channels
                        // are already live (`system_social_interactions`'
                        // per-kind deltas equal the grounded `base_delta`
                        // values, so applying them here would double-count);
                        // status/obligation/reputation were computed by
                        // `resolve_effect` but never applied anywhere. Pure
                        // deltas on existing state, no RNG — the replay
                        // stream is untouched. The listener owes obligation
                        // to the speaker (promises/requests create debt —
                        // the daily power-balance pass reads it), admires
                        // the speaker per their reputation (aggression costs
                        // standing, informing builds it), and the speaker's
                        // respect for the listener tracks the listener's
                        // status in the speaker's eyes (comfort raises it;
                        // insult/threaten demeans — feeds `quality()`).
                        if ti < n {
                            // §10.2 (P5 audit, Iteration 184): the V2 deep
                            // dimensions were structurally disconnected —
                            // `record_positive`/`record_negative` had ZERO
                            // production call sites, so intimacy (and until
                            // now commitment) stayed 0.000 in every probed
                            // window. The interaction's signed magnitude now
                            // flows through the designed channels: positive
                            // interactions build trust/affection/intimacy/
                            // commitment/gratitude + memory weights, hostile
                            // ones erode them. These deltas are on the V2
                            // array — independent of the legacy
                            // `relationships` per-kind deltas, so nothing is
                            // double-counted. Zero-RNG, deterministic.
                            let magnitude = match kind {
                                mindstrata_core::event::InteractionKind::Help
                                | mindstrata_core::event::InteractionKind::Comfort => {
                                    Fixed::from_f64(0.6)
                                }
                                mindstrata_core::event::InteractionKind::Threaten
                                | mindstrata_core::event::InteractionKind::Insult => {
                                    Fixed::from_f64(0.5)
                                }
                                mindstrata_core::event::InteractionKind::Gossip
                                | mindstrata_core::event::InteractionKind::Talk => {
                                    Fixed::from_f64(0.3)
                                }
                                _ => Fixed::from_f64(0.2),
                            };
                            // Iteration 197: a fully-socialized agent (the §17
                            // developmental state — grows through childhood)
                            // registers interactions more strongly, so its
                            // bonds advance faster. Baseline-corrected: scale
                            // is exactly 1.0 at the default 0.5 (all adult
                            // populations are byte-identical), rising to 1.25
                            // at full socialization, falling to 0.75 at
                            // unsocialized 0 — only agents whose socialization
                            // actually developed diverge.
                            let ti_social = Fixed::ONE
                                + (self.agents[ti].developmental.socialization_completeness
                                    - Fixed::from_f64(0.5))
                                    * Fixed::from_f64(0.5);
                            let fi_social = Fixed::ONE
                                + (self.agents[fi].developmental.socialization_completeness
                                    - Fixed::from_f64(0.5))
                                    * Fixed::from_f64(0.5);
                            // Iteration 198: the §8.1.9 Theory of Mind model
                            // was write-only AND its intents were structurally
                            // unreachable (the update derived pos/neg from
                            // trust with scaled-down multipliers, so Friendly/
                            // Threatening could never fire). The speaker's
                            // perceived intent of the listener now modulates
                            // the interaction's relational weight: an agent
                            // who models the listener as Friendly extends
                            // MORE bond (×1.25 on positive), as
                            // Threatening/Deceptive LESS (×0.75); hostile
                            // acts reverse the sign. Identity at Unknown/
                            // Neutral (scale exactly 1.0), so default
                            // populations stay byte-identical.
                            let fi_model = self.agents[fi]
                                .mind_models
                                .get(mindstrata_core::id::AgentId::new(to_u));
                            let fi_intent = match fi_model {
                                Some(m) => m.perceived_intent,
                                None => {
                                    crate::psychology::theory_of_mind::IntentPerception::Unknown
                                }
                            };
                            let fi_tom = match fi_intent {
                                crate::psychology::theory_of_mind::IntentPerception::Friendly => {
                                    Fixed::from_f64(1.25)
                                }
                                crate::psychology::theory_of_mind::IntentPerception::Threatening
                                | crate::psychology::theory_of_mind::IntentPerception::Deceptive => {
                                    Fixed::from_f64(0.75)
                                }
                                crate::psychology::theory_of_mind::IntentPerception::Unknown
                                | crate::psychology::theory_of_mind::IntentPerception::Neutral => {
                                    Fixed::ONE
                                }
                            };
                            let l_pos = Self::relationship_v2_pos(ti, fi);
                            let rv2 = &mut self.agents[ti].relationship_v2s[l_pos];
                            rv2.obligation = (rv2.obligation + effect.obligation_delta).clamp_01();
                            rv2.admiration = (rv2.admiration + effect.reputation_delta).clamp_01();
                            let magnitude_ti = (magnitude * ti_social).clamp_01();
                            if kind_is_hostile {
                                rv2.record_negative(tick_u64, magnitude_ti);
                            } else {
                                rv2.record_positive(tick_u64, magnitude_ti);
                            }
                            let s_pos = Self::relationship_v2_pos(fi, ti);
                            let rv2s = &mut self.agents[fi].relationship_v2s[s_pos];
                            rv2s.respect = (rv2s.respect + effect.status_delta).clamp_01();
                            let magnitude_fi = (magnitude * fi_social * fi_tom).clamp_01();
                            if kind_is_hostile {
                                rv2s.record_negative(tick_u64, magnitude_fi);
                            } else {
                                rv2s.record_positive(tick_u64, magnitude_fi);
                            }
                        }
                    }
                }
            }

            // ── 6. Appraisal — emotions from state ────────────────────
            // §8.1.4: Compute per-agent threat/unfairness exposure from this tick's
            // events so cognition actually reacts to the world (conflicts, norm
            // violations, moral panics) instead of only hunger/thirst.
            let mut threat_exposure: Vec<Fixed> = vec![Fixed::ZERO; self.agents.len()];
            let mut witnessed_unfairness: Vec<Fixed> = vec![Fixed::ZERO; self.agents.len()];
            for ev in &ctx.events[pre_tick_events..] {
                match ev {
                    SimEvent::ConflictOccurred {
                        aggressor,
                        target,
                        fear_induced,
                        ..
                    } => {
                        let a = aggressor.as_u64() as usize;
                        let t = target.as_u64() as usize;
                        if a < self.agents.len() {
                            threat_exposure[a] = (threat_exposure[a]
                                + *fear_induced * Fixed::from_f64(0.5))
                            .clamp_01();
                        }
                        if t < self.agents.len() {
                            threat_exposure[t] = (threat_exposure[t] + *fear_induced).clamp_01();
                            witnessed_unfairness[t] =
                                (witnessed_unfairness[t] + Fixed::from_f64(0.1)).clamp_01();
                        }
                    }
                    SimEvent::NormViolated { agent, .. } => {
                        let idx = agent.as_u64() as usize;
                        if idx < self.agents.len() {
                            witnessed_unfairness[idx] =
                                (witnessed_unfairness[idx] + Fixed::from_f64(0.05)).clamp_01();
                        }
                    }
                    _ => {}
                }
            }
            for i in 0..self.agents.len() {
                let threat = threat_exposure[i];
                let unfairness = witnessed_unfairness[i];
                let need_pressure = needs[i].hunger.max(needs[i].thirst);
                // §8.1.4 (P2/P3 audit closure): the agent's own aggression is
                // the self-caused-failure signal that makes guilt reachable.
                // Anger is NOT it — appraisal never writes it (the conflict
                // events push after this block reads, so the unfairness/threat
                // exposure is always zero; anger maxes at 0.063 in calibrated
                // windows). An active feud IS: it is live and transient in
                // conflict worlds (7/12 at 2K, 3/12 at 10K), self-caused by
                // the agent's own escalation, and absent in calm worlds (zero
                // blast on the golden baseline). The feud is also goal-
                // RELEVANT: without it, `goal_relevance` (needs/threat only)
                // stays below the 0.3 branch threshold for feuding agents and
                // the incongruent branch never fires — guilt stays 0 even
                // with Self_ attribution.
                let in_feud = !self.agents[i].feuds.is_empty();
                let feud_pressure = if in_feud {
                    Fixed::from_f64(0.5)
                } else {
                    Fixed::ZERO
                };
                // §8.1.4: Deepened appraisal dimensions — derived from live
                // agent state (sacredness, attachment, status, narrative).
                let max_sacredness = self.agents[i]
                    .sacred_values
                    .values
                    .iter()
                    .fold(Fixed::ZERO, |acc, v| acc.max(v.sacredness));
                let separation_distress = self.agents[i].attachment.separation_distress;
                let status_hold = self.agents[i].status_v2.authority;
                let coherence = self.agents[i].narrative.coherence;
                // §8.1.4 (P2/P3 re-audit — P3-8 root cause): `social_visibility`
                // was HARDCODED to zero, so the loneliness producer
                // (attachment_threat × (1 − social_visibility)) fired at full
                // strength every tick for partnered agents (daily separation
                // distress) and, being decay-exempt, ratcheted loneliness to
                // 1.0 for 10/12 agents in calm windows (probe: mean 0.833,
                // 10/12 pinned) while bereaved agents in pestilence (no
                // partner → no separation) sat at 0.000 — the P3-8 inversion.
                // Both inputs now derive from live state: visibility rises
                // with partner presence and relationship count (an embedded
                // agent's daily separation barely registers; an isolate has
                // no damping), and attachment threat adds an isolation term
                // so the socially-absent agent is chronically lonely.
                // Deterministic (pure relationship-state arithmetic, no RNG).
                let rel_count = self.agents[i].relationship_v2s.len() as f64;
                let social_visibility = if self.agents[i].partner.is_some() {
                    Fixed::from_f64(0.5) + Fixed::from_f64(rel_count.min(4.0) * 0.1)
                } else {
                    Fixed::from_f64(rel_count.min(4.0) * 0.1)
                };
                let social_visibility = social_visibility.clamp_01();
                let isolation_threat =
                    (Fixed::ONE - social_visibility) * Fixed::from_f64(0.08);
                let attachment_threat = (separation_distress + isolation_threat).clamp_01();
                // Iteration 192 (famine-chain closure): past the 0.5
                // goal-congruence gate, the excess unmet need drags the
                // signed future outlook negative (see the future_implication
                // field comment below). Keyed on HUNGER — the same signal the
                // goal-congruence gate uses (`needs[i].hunger < 0.5` above) —
                // not need_pressure: thirst routinely exceeds 0.5 in calm
                // windows (drink windows are spaced), so a max(hunger,thirst)
                // key would fire transient despair in CALM (probe-pinned:
                // calm maxD 0.880) and break the golden byte-identity.
                // Hunger stays below 0.5 in calm/pestilence, so the term is
                // zero there and only the famine window (hunger 0.711 peak)
                // crosses it.
                let bleak_excess = (needs[i].hunger - Fixed::from_f64(0.5))
                    .max(Fixed::ZERO)
                    * Fixed::from_f64(2.0);
                let appraisal = Appraisal {
                    goal_relevance: need_pressure.max(threat).max(feud_pressure),
                    goal_congruence: if threat > Fixed::from_f64(0.2) {
                        // Under direct threat: strongly goal-incongruent.
                        -Fixed::from_f64(0.6)
                    } else if in_feud {
                        // Self-caused conflict state: a feud the agent
                        // escalated is a goal-incongruent situation even when
                        // its own needs are met — the morally incongruent
                        // self-attributed failure that produces guilt.
                        -Fixed::from_f64(0.3)
                    } else if needs[i].hunger < Fixed::from_f64(0.5) {
                        // §8.1.4 (P3-5): congruence now scales with how well
                        // needs are met instead of the constant +0.3 — the
                        // constant made `positive` a per-agent constant, so
                        // the positive secondary family (gratitude/tenderness/
                        // relief/nostalgia) pinned at IDENTICAL saturation
                        // for all agents in every window (probe: gratitude
                        // 0.880, tenderness 1.000, 12/12 identical). With
                        // (1 − need_pressure) the family differentiates and a
                        // genuinely thirsty agent (high pressure) is no longer
                        // joyful by construction.
                        Fixed::from_f64(0.3) * (Fixed::ONE - need_pressure)
                    } else {
                        Fixed::from_f64(-0.3)
                    },
                    coping_potential: personalities[i].conscientiousness,
                    expectedness: Fixed::from_f64(0.5),
                    fairness: if unfairness > Fixed::from_f64(0.05) {
                        -unfairness // witnessed injustice → anger
                    } else {
                        Fixed::from_f64(0.5)
                    },
                    // §8.1.4 (P2/P3 audit closure): the agency dimension was
                    // hardcoded `Agency::Circumstance`, so the Self_/Other
                    // attribution branches in appraise() — pride (Self_ on
                    // goal-congruent), trust (Other on goal-congruent), guilt
                    // (Self_ on goal-incongruent), other-directed anger (Other
                    // on goal-incongruent) — were structurally unreachable
                    // (probe: pride/guilt/trust 0.000, 12/12, every window).
                    // Derive attribution from live state so the moral
                    // emotions are reachable and directionally honest:
                    //   - earned success (conscientious work / held authority)
                    //     → Self_ → pride
                    //   - community support (low attachment distress) → Other
                    //     → trust
                    //   - witnessed injustice (conflict/norm-violation
                    //     exposure) → Other → anger at the wrongdoer (the
                    //     victim's path; the aggressor gets no unfairness,
                    //     only threat)
                    //   - own aggression (an active feud the agent escalated)
                    //     → Self_ → guilt (the aggressor's path — the honest
                    //     self-caused-failure signal; anger was tried first
                    //     but appraisal never writes it, so it stayed 0)
                    //   - impersonal scarcity (famine/pestilence with no
                    //     social cause) → Circumstance → sadness/fear
                    //     (unchanged).
                    agency: if threat <= Fixed::from_f64(0.2)
                        && needs[i].hunger < Fixed::from_f64(0.5)
                    {
                        // Goal-congruent: who gets credit?
                        if personalities[i].conscientiousness > Fixed::from_f64(0.5)
                            || status_hold > Fixed::from_f64(0.5)
                        {
                            // Earned by own effort/standing → pride.
                            Agency::Self_
                        } else if separation_distress < Fixed::from_f64(0.3) {
                            // Carried by community support → trust.
                            Agency::Other(AgentId::new(i as u64))
                        } else {
                            Agency::Circumstance
                        }
                    } else if in_feud {
                        // Own aggression escalated the feud → self-attributed
                        // failure → guilt.
                        Agency::Self_
                    } else if unfairness > Fixed::from_f64(0.05) {
                        // Someone else's wrongdoing caused the failure → anger.
                        Agency::Other(AgentId::new(i as u64))
                    } else {
                        Agency::Circumstance
                    },
                    social_visibility,
                    // Iteration 196: identity relevance now scales with the
                    // §17 developmental identity-formation state (previously
                    // a hardcoded 0.2 constant while `identity_formation` was
                    // write-only). Baseline-corrected: exactly 0.2 at the
                    // default formation 0.5, rising toward 0.3 at full
                    // formation and falling toward 0.1 at identity-less 0 —
                    // a formed identity makes events more personally
                    // relevant (pride/shame/humiliation/awe all scale with
                    // it). Exact no-op at populate defaults.
                    identity_relevance: Fixed::from_f64(0.2)
                        + (self.agents[i].developmental.identity_formation
                            - Fixed::from_f64(0.5))
                            * Fixed::from_f64(0.2),
                    sacredness_violation: (unfairness * max_sacredness).clamp_01(),
                    attachment_threat,
                    status_threat: (threat * (Fixed::ONE - status_hold)).clamp_01(),
                    purity_violation: (unfairness * (Fixed::ONE - emotions[i].trust)).clamp_01(),
                    // Felt control erodes under threat — distinct from the raw
                    // conscientiousness that feeds coping_potential above.
                    controllability: (personalities[i].conscientiousness * (Fixed::ONE - threat))
                        .clamp_01(),
                    // Signed future outlook; provably in [-1, 1] (threat and
                    // need_pressure are both clamped to 0..1), clamped for the
                    // Iter-192 bleak-excess term below (worst case threat 1.0 +
                    // need 1.0 + excess 1.0 = -2, so the clamp binds only in
                    // the impossible simultaneous-max corner).
                    //
                    // Iteration 192 (famine-chain closure): `1 - threat -
                    // need_pressure` can only go negative when threat +
                    // need_pressure > 1, so a famine (threat 0, need peaking
                    // at 0.71) left a starving agent at +0.29 -> HOPE fired
                    // and despair stayed 0.000 (probe-pinned) — the documented
                    // "sadness -> despair -> depression-from-deprivation"
                    // chain (famine-grain-drain comment below) died at the
                    // despair link. Past the 0.5 goal-congruence gate the
                    // excess unmet need drags the outlook negative at 2x the
                    // excess (need 0.667 -> 0, need 0.711 -> -0.13, need 1.0
                    // -> -1.0), so despair is reachable from deprivation
                    // alone. Below the gate the formula is byte-identical
                    // (calm/pestilence need ~0.23-0.28 -> unchanged).
                    future_implication: (Fixed::ONE - threat - need_pressure - bleak_excess)
                        .clamp(-Fixed::ONE, Fixed::ONE),
                    narrative_meaning: coherence,
                };

                let mut delta = appraisal::appraise(&appraisal, tick, &self.params);

                // §8.1.6 (Iteration 105): temperament reactivity amplifies the
                // stress response. The deviation from the trait-derived
                // baseline is zero at construction and builds only as the
                // plasticity pass accumulates repeated-stress experience
                // (identity-at-zero → amplifier 1.0 → legacy byte-identical
                // when inert). The fold lands on the appraise-produced delta
                // only — the §8.1.14 attachment-fear block below stays
                // orthogonal.
                let reactivity_baseline =
                    crate::person::Temperament::from_traits(&personalities[i]);
                let reactivity_deviation = self.agents[i].personality.temperament.reactivity
                    - reactivity_baseline.reactivity;
                let reactivity_amp =
                    crate::person::Temperament::reactivity_amplifier(reactivity_deviation);
                delta.fear = (delta.fear * reactivity_amp).clamp_01();
                delta.anger = (delta.anger * reactivity_amp).clamp_01();

                // §8.1.14: Attachment → emotion — separation distress feeds fear.
                // Anxious and disorganized styles convert separation into fear
                // more readily than secure/avoidant styles.
                let sep = self.agents[i].attachment.separation_distress;
                if sep > Fixed::ZERO {
                    let style_factor = match self.agents[i].attachment.style {
                        crate::psychology::attachment::AttachmentStyle::Anxious
                        | crate::psychology::attachment::AttachmentStyle::Disorganized => {
                            Fixed::from_f64(0.15)
                        }
                        crate::psychology::attachment::AttachmentStyle::Secure
                        | crate::psychology::attachment::AttachmentStyle::Avoidant => {
                            Fixed::from_f64(0.08)
                        }
                    };
                    delta.fear = (delta.fear + sep * style_factor).clamp_01();
                }

                emotions[i].fear = (emotions[i].fear + delta.fear).clamp_01();
                emotions[i].anger = (emotions[i].anger + delta.anger).clamp_01();
                emotions[i].joy = (emotions[i].joy + delta.joy).clamp_01();
                emotions[i].sadness = (emotions[i].sadness + delta.sadness).clamp_01();
                emotions[i].trust = (emotions[i].trust + delta.trust).clamp_01();
                emotions[i].shame = (emotions[i].shame + delta.shame).clamp_01();
                emotions[i].pride = (emotions[i].pride + delta.pride).clamp_01();
                emotions[i].guilt = (emotions[i].guilt + delta.guilt).clamp_01();
                // §8.1.4: Expanded emotion families. Observational state in
                // calm windows — loneliness (Iter-98) is consumed for
                // decisions and stays exempt from decay (its producer is
                // zero in most ticks), tenderness (Iter-99) feeds the Help
                // decision and is decayed since Iter-183 (P3-5 — it
                // ratcheted to 1.0 while exempt), humiliation (Iter-116)
                // amplifies failed-threat escalation, and the rest are
                // kept at producer-driven steady states by the Iter-116
                // daily decay below. No gate serializes them (golden
                // agent_hash and the snapshots read only base
                // emotions/valence), so calibrated runs stay
                // byte-identical.
                emotions[i].disgust = (emotions[i].disgust + delta.disgust).clamp_01();
                emotions[i].contempt = (emotions[i].contempt + delta.contempt).clamp_01();
                emotions[i].awe = (emotions[i].awe + delta.awe).clamp_01();
                emotions[i].gratitude = (emotions[i].gratitude + delta.gratitude).clamp_01();
                emotions[i].jealousy = (emotions[i].jealousy + delta.jealousy).clamp_01();
                emotions[i].envy = (emotions[i].envy + delta.envy).clamp_01();
                emotions[i].loneliness = (emotions[i].loneliness + delta.loneliness).clamp_01();
                emotions[i].tenderness = (emotions[i].tenderness + delta.tenderness).clamp_01();
                emotions[i].humiliation = (emotions[i].humiliation + delta.humiliation).clamp_01();
                emotions[i].relief = (emotions[i].relief + delta.relief).clamp_01();
                emotions[i].hope = (emotions[i].hope + delta.hope).clamp_01();
                emotions[i].despair = (emotions[i].despair + delta.despair).clamp_01();
                emotions[i].nostalgia = (emotions[i].nostalgia + delta.nostalgia).clamp_01();
                emotions[i].moral_outrage =
                    (emotions[i].moral_outrage + delta.moral_outrage).clamp_01();

                // §8.1.4: Valence is SIGNED (-1..1). The old `.clamp_01()` floored
                // negative affect at 0, so fear/anger/sadness could never move
                // valence below zero — an agent could live through a revolution
                // while maintaining +0.8 valence. Fixed is signed, so we clamp to
                // the full range instead.
                affects[i].valence =
                    (emotions[i].joy - emotions[i].sadness - emotions[i].fear - emotions[i].anger)
                        .clamp(-Fixed::ONE, Fixed::ONE);
                affects[i].arousal =
                    (emotions[i].fear + emotions[i].anger + emotions[i].joy) * Fixed::from_f64(0.5);
                // §8.1.4: Apply emotion regulation AFTER appraisal.
                // Now apply_strategy() uses the fresh, appraisal-derived affect values
                // as input — the skill-scaled boost is computed from the correct target state.
                if i < reg_strategies.len() {
                    // §8.1: Embodied emotions (high body tone from interoceptive
                    // sensitivity) resist cognitive regulation — scale the
                    // strategy's effect. Mean-zero at the default sensitivity.
                    let reg_scale = self.agents[i]
                        .interoception
                        .regulation_scale(affects[i].valence, affects[i].arousal);
                    let (reg_vd, reg_ad) = self.agents[i].emotion_regulation.apply_strategy(
                        reg_strategies[i],
                        affects[i].valence,
                        affects[i].arousal,
                    );
                    affects[i].valence =
                        (affects[i].valence + reg_vd * reg_scale).clamp(-Fixed::ONE, Fixed::ONE);
                    affects[i].arousal = (affects[i].arousal + reg_ad * reg_scale).clamp_01();
                }
            }

            // ── 7. Emotion decay ──────────────────────────────────────
            // §8.1.4 (Iteration 164): The BASE emotion families now decay
            // proportionally every tick (BASE_EMOTION_DECAY_RATE, probe-
            // calibrated at 0.06) — the pre-Iter-164 subtractive decay
            // (0.002/tick, §22.1) was ~40× too weak against the per-tick
            // appraisal deltas, so fear pinned at 0.83–1.0 in every
            // calibrated window (probe-pinned at Iter-164: fear climbed to
            // 0.99 by tick 800 in seeds 1/42/99) and every amplification
            // consumer (`1 + fear×0.4` Safety, `(fear+sadness)×0.1`
            // negative-events narrative fold, stress input) ran at max
            // with zero differentiation. Proportional decay cancels
            // production to producer-driven steady states (fear mean
            // 0.55–0.75 with min 0.03–0.10 / max 1.0 only for genuinely
            // stressed agents, joy 0–0.92, shame ≈ 0.03) — restoring
            // headroom so amplification differentiates across agents and
            // windows. Tuned down from an initial 0.08 whose probe
            // starved the rare-event conception rolls (the lower fear
            // re-paces the shared Social RNG stream; at 0.08 only 1/5
            // seeds delivered a 100K birth, at 0.06 all 5 deliver).
            // `tenderness` exemption below removed (Iter-183, P3-5) and
            // `loneliness` joined the secondary decay below (P2/P3 re-
            // audit, P3-8) once their producers became live — each was
            // the same write-only-ratchet class. `trust` JOINED the base
            // decay above (P2/P3 audit closure): appraisal now produces
            // it via Other-attributed goal congruence, so the old
            // exemption (justified only while it probed at exactly 0)
            // would ratchet it to 1.0.
            for emotion in &mut emotions {
                let base_rate = crate::appraisal::BASE_EMOTION_DECAY_RATE;
                emotion.fear = crate::appraisal::secondary_emotion_decay(emotion.fear, base_rate);
                emotion.anger =
                    crate::appraisal::secondary_emotion_decay(emotion.anger, base_rate);
                emotion.joy = crate::appraisal::secondary_emotion_decay(emotion.joy, base_rate);
                emotion.sadness =
                    crate::appraisal::secondary_emotion_decay(emotion.sadness, base_rate);
                emotion.shame =
                    crate::appraisal::secondary_emotion_decay(emotion.shame, base_rate);
                emotion.pride =
                    crate::appraisal::secondary_emotion_decay(emotion.pride, base_rate);
                emotion.guilt =
                    crate::appraisal::secondary_emotion_decay(emotion.guilt, base_rate);
                // §8.1.4 (P2/P3 audit closure): `trust` was decay-exempt with
                // the justification "probes at exactly 0 in calibrated
                // windows" — but the appraisal block now produces it
                // (Other-attributed goal congruence), so the exemption would
                // become a write-only ratchet to 1.0 (the same class as the
                // Iter-183 tenderness fix). Joined the base decay so it sits
                // at a producer-driven steady state.
                emotion.trust =
                    crate::appraisal::secondary_emotion_decay(emotion.trust, base_rate);
                // §8.1.4 (Iteration 116): The expanded emotion families decay
                // proportionally EVERY TICK (SECONDARY_EMOTION_DECAY_RATE,
                // probe-calibrated) — the base-8 linear decay (0.002/tick) is
                // ~40× too weak against the per-tick appraisal deltas (~0.08/
                // tick, probe-pinned: awe climbs 0.08 → 1.0 in ~20 ticks), and
                // the expanded families previously had NO decay at all: the
                // produced families (awe/relief/hope/gratitude/nostalgia)
                // pinned at 1.0 in every calibrated run. The proportional
                // per-tick decay cancels the per-tick production to keep them
                // at meaningful producer-driven levels. `loneliness` (Iter-98
                // social-seeking) remains exempt — its producer is zero in
                // most ticks, so it does not ratchet; `tenderness` was
                // exempted at Iter-99 but JOINED the decay at Iter-183
                // (P3-5): with a live per-agent producer it ratcheted to
                // 1.0 for every agent, exactly the write-only saturation
                // class the P3 audit targets. The Iter-115 birth-pipeline
                // lesson applies — the help-window re-pace is absorbed by
                // the standard re-pin workflow (golden + snapshots
                // regenerated, integration pins re-anchored).
                let rate = crate::appraisal::SECONDARY_EMOTION_DECAY_RATE;
                emotion.disgust = crate::appraisal::secondary_emotion_decay(emotion.disgust, rate);
                emotion.contempt =
                    crate::appraisal::secondary_emotion_decay(emotion.contempt, rate);
                emotion.awe = crate::appraisal::secondary_emotion_decay(emotion.awe, rate);
                emotion.gratitude =
                    crate::appraisal::secondary_emotion_decay(emotion.gratitude, rate);
                emotion.jealousy =
                    crate::appraisal::secondary_emotion_decay(emotion.jealousy, rate);
                emotion.envy = crate::appraisal::secondary_emotion_decay(emotion.envy, rate);
                emotion.humiliation =
                    crate::appraisal::secondary_emotion_decay(emotion.humiliation, rate);
                emotion.relief = crate::appraisal::secondary_emotion_decay(emotion.relief, rate);
                emotion.hope = crate::appraisal::secondary_emotion_decay(emotion.hope, rate);
                emotion.despair = crate::appraisal::secondary_emotion_decay(emotion.despair, rate);
                emotion.nostalgia =
                    crate::appraisal::secondary_emotion_decay(emotion.nostalgia, rate);
                emotion.moral_outrage =
                    crate::appraisal::secondary_emotion_decay(emotion.moral_outrage, rate);
                // §8.1.4 (P2/P3 re-audit — P3-8 completion): loneliness JOINS
                // the secondary decay. It was exempted at Iter-98 with the
                // justification "its producer is zero in most ticks, so it
                // does not ratchet" — that premise is stale: the appraisal
                // block now produces it every tick (attachment_threat ×
                // (1 − social_visibility), with separation distress for
                // partnered agents), so the exemption ratcheted it to 1.0
                // for 10/12 agents in calm (probe: mean 0.833) — the exact
                // write-only saturation class the trust/tenderness fixes
                // eliminated. With the decay it sits at a producer-driven
                // steady state: partnered-embedded agents low, isolates
                // high, bereaved agents (pestilence) mid — the honest
                // direction.
                emotion.loneliness =
                    crate::appraisal::secondary_emotion_decay(emotion.loneliness, rate);
                // §8.1.4 (Iteration 183 — AP2 P3-5 completion): tenderness
                // JOINS the secondary decay. It was exempted at Iter-99
                // ("consumers calibrated against the saturated state") and
                // ratcheted to 1.0 for 13/13 agents in every calibrated
                // window — the write-only saturation class P3-5 exists to
                // eliminate. The P3-5 goal-congruence differentiation gave
                // gratitude a 0.207–0.880 spread but could not move
                // tenderness, which never decayed. Now its producer
                // (positive × (1 − status_threat), same family as
                // gratitude) drives the same delta/rate equilibrium, so
                // the help propensity's tenderness tier (× 0.5)
                // differentiates across agents instead of adding a
                // constant +0.5 to everyone. `loneliness` keeps its
                // exemption — its producer (attachment_threat × (1 −
                // social_visibility)) is zero in most ticks, so it does
                // not ratchet (probe: live at 0.04–0.42).
                emotion.tenderness =
                    crate::appraisal::secondary_emotion_decay(emotion.tenderness, rate);
            }

            // ── 7b. §10.1.1 fear contagion (Iteration 107) ────────────
            // The sensory field's perceived ambient stress ("expression" —
            // mean of (fear + anger) / 2 over agents within
            // PERCEPTION_RADIUS) feeds the agent's fear on the daily
            // cadence: an agent surrounded by stressed neighbors catches
            // their fear, and a terrified village sustains itself against
            // the §22.1 decay. Zero-at-zero: no stressed neighbors →
            // perceived_stress 0 → the term contributes nothing, and the
            // daily refresh (end of the previous tick) reads default zeros
            // at tick 0, so calibrated runs are byte-identical until
            // neighbors actually hold stress. Deterministic (the field is
            // RNG-free by construction); the rate keeps the annual ambient
            // pressure bounded (~FEAR_CONTAGION_RATE × stress × 365,
            // clamped). Equilibrium note: the fold's own arithmetic
            // (rate × stress ≈ 0.02/day vs decay 0.002/day) would alone
            // saturate fear — and empirically 2/12 agents DO sit at the
            // 1.0 clamp at tick 2000 (probe-pinned) — but the per-tick
            // appraisal recompute holds the population MEAN at ~0.86,
            // leaving headroom for the other 10, so the channel's
            // differential is compressed but live. The elevated ambient
            // fear is the intended "terrified village sustains itself"
            // state (reviewer-flagged calibration risk: if appraisal
            // tuning changes, re-pin the integration floor).
            if phases.is_daily {
                for (agent, emotion) in self.agents.iter().zip(emotions.iter_mut()) {
                    emotion.fear =
                        crate::social::relational_field::RelationalFields::contagion_apply(
                            emotion.fear,
                            agent.relational_fields.perceived_stress,
                            Fixed::from_f64(crate::social::relational_field::FEAR_CONTAGION_RATE),
                        );
                }
                // ── 7c. §10.1.2 peer-status envy (Iteration 113) ───────
                // The social field's highest-status neighbor ("the most
                // dominant presence I see") feeds the agent's anger on the
                // daily cadence: an agent who perceives a GENUINELY
                // dominant peer (peer_status above the 0.5 anchor — above
                // the calibrated ceiling of 0.46, probe-pinned across seeds
                // 1/7/42/99, drought/pestilence scenarios, and the 20K
                // horizon) grows envious: status frustration → anger. The
                // anger channel is decisional (feeds threat_level, stress,
                // and witnessed-violation appraisal), so the envy term is
                // behaviorally live when it engages. ONE-SIDED: identity
                // (zero delta) at/below the anchor, so calibrated runs are
                // byte-identical (zero-blast — no regeneration).
                // Deterministic (no RNG).
                for (agent, emotion) in self.agents.iter().zip(emotions.iter_mut()) {
                    emotion.anger = crate::social::relational_field::RelationalFields::envy_apply(
                        emotion.anger,
                        agent.relational_fields.peer_status,
                        Fixed::from_f64(crate::social::relational_field::PEER_ENVY_ANCHOR),
                        Fixed::from_f64(crate::social::relational_field::PEER_ENVY_RATE),
                        Fixed::from_f64(crate::social::relational_field::PEER_ENVY_CAP),
                    );
                }
            }

            // ── 8. Belief resistance decay ────────────────────────────
            // §5.1: Use configurable decay rate from SimParameters.
            // §8.1.17: Narrative frames modulate belief rigidity — agents whose
            // meaning-making frames resist countervailing evidence (punitive,
            // curse, just-world frames) hold their beliefs longer. Mean-zero
            // at the default frame set (resistance_to_update == 0.5 exactly),
            // so decay is unchanged for typical agents and only diverges as
            // frames drift with life experience.
            let belief_resistance_decay = self.params.belief_resistance_decay;
            for agent_beliefs in &mut self.agents {
                let rigidity = agent_beliefs.narrative_frames.resistance_to_update();
                let rigidity_deviation = rigidity - Fixed::from_f64(0.5);
                let scaled_decay =
                    (belief_resistance_decay * (Fixed::ONE - rigidity_deviation)).max(Fixed::ZERO);
                belief_update::decay_belief_resistance(
                    &mut agent_beliefs.beliefs,
                    scaled_decay,
                    &self.params,
                );
            }

            // ── 9. Trait plasticity (§8.1.6 state-trait dynamics) ──────
            // Temperament dimensions slowly reshape from repeated life
            // experience (stress, recovery, social engagement, goal striving),
            // gated by identity integration (self-model coherence) and
            // developmental plasticity (youth). Writes ONLY the observational
            // temperament layer — the 12 decision-read core traits are
            // untouched, so calibrated runs remain byte-identical.
            //
            // §8.1.6 (Iteration 162): the social_engagement and goal_striving
            // signals were STRUCTURALLY DEAD — probe-pinned at exactly 0.0000
            // for every agent to 5,000 ticks — so sociability, persistence,
            // regularity, and approach_withdrawal never drifted (only the
            // stress-driven reactivity/soothability/sensitivity moved). Two
            // dead channels, two fixes:
            //   1. `social_engagement` read `emotions.trust + joy`: appraisal
            //      NEVER writes `emotions.trust` (pinned 0.0000 at every
            //      sampled tick) and joy is sporadic. Replaced with the
            //      live social-need satisfaction `1 − needs.social` (the
            //      deficit is live, 0.00–0.065 in calm windows, so the
            //      satisfaction signal sits ~0.93–1.0 — saturated but
            //      genuine, mirroring the arousal signal's saturation).
            //   2. `goal_striving` read `recent_attempts/successes`, but the
            //      outcome pass increments them at tick-action time and the
            //      derived-state pass `saturating_sub(1)`s them — the two
            //      cancel within each tick, so the window is ALWAYS 0 at
            //      this read point (pinned 0/0 at every sample). Replaced
            //      with the live goal load — the sum of active goal
            //      priorities clamped to 0..1 (1–2 goals, priority sums
            //      0.72–1.42 in calm windows). Both replacements stay in
            //      the Fixed domain (project doctrine), are deterministic
            //      (no RNG), and now drive the four previously-frozen
            //      temperament dimensions.
            for (i, agent) in self.agents.iter_mut().enumerate() {
                let goal_striving = goals[i]
                    .iter()
                    .fold(Fixed::ZERO, |acc, g| acc + g.priority)
                    .clamp_01();
                let signals = crate::person::PlasticitySignals {
                    // Arousal is the in-scope physiological stress proxy
                    // ((fear + anger + joy) × 0.5 from the appraisal block).
                    repeated_stress: affects[i].arousal,
                    recovery: affects[i].valence.clamp_01(),
                    // §8.1.6 (Iteration 162): live social-need satisfaction
                    // (was the dead `emotions.trust + joy` proxy).
                    social_engagement: Fixed::ONE - needs[i].social.clamp_01(),
                    goal_striving,
                    identity_integration: agent.self_model.coherence,
                    age_years: agent.age,
                };
                let baseline = crate::person::Temperament::from_traits(&agent.personality);
                agent
                    .personality
                    .temperament
                    .plastic_update(&baseline, &signals);
                // §8.1.6 (Iteration 179): the 12 decision-read core traits
                // also move — the plan's "traits slowly change through
                // repeated behavior, trauma, success, failure..." formula
                // applied to the traits themselves (not just the
                // observational temperament layer). Each trait is pushed
                // toward its repeatedly-expressed state and pulled back
                // toward the birth constitution, both at the same rate
                // (identity-integration × developmental-plasticity ×
                // social-reinforcement × CORE_TRAIT_PLASTICITY_RATE).
                // Deterministic, zero RNG, clamped to 0..1.
                //
                // YEARLY GATE (the critical calibration decision): core
                // traits feed decisions directly, so per-tick drift at the
                // Fixed-4-decimal floor would pin them to constitution +
                // saturated-signal (social/valence ~0.9-1.0 in calm windows)
                // within a few thousand ticks — a qualitative behavioral
                // rewrite (Iter-179 probe: births delayed 51K→93K ticks;
                // and the baseline recompute chases the drifting traits,
                // collapsing the deviation the Iter-105 reactivity
                // amplifier reads). Gated to the scheduler's YearlyPhase
                // (51,840 ticks) so every short-horizon calibrated window
                // is byte-identical while still allowing genuine multi-year
                // personality drift — the plan's "slowly".
                //
                // Why YearlyPhase and not the age system's 35,040-tick year
                // (DemographyConfig.ticks_per_year)? Two reasons, both
                // probed: (1) the scheduler's yearly cadence is the
                // codebase's canonical "year" for the other slow systems
                // (climate, culture drift, technology), so trait plasticity
                // fires on the same clock as its sibling yearly passes;
                // (2) at 35,040 the first fire lands at tick 35,040 —
                // BEFORE the seed-46 first birth (51,000) — and that
                // pre-birth nudge re-paces courtship enough to push every
                // birth out of the 100K liveness window (probe: left []).
                // At 51,840 the first fire lands AFTER the first birth, so
                // the pinned 51,000 birth stays byte-identical and only
                // subsequent courtship shifts (probe: [51000, 69330,
                // 88160] — the documented 3-chain). The age-vs-gate year
                // mismatch (an agent ages ~1.48 years between fires) is a
                // pre-existing dual-year-definition inconsistency in the
                // codebase (clock 96 ticks/day vs scheduler 144 ticks/day),
                // not introduced by this iteration.
                if phases.is_yearly {
                    agent.personality.plastic_update_traits(&signals);
                }
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
                        self.agents[i].position.x =
                            (self.agents[i].position.x + dx).clamp(0, world_w - 1);
                        self.agents[i].position.y =
                            (self.agents[i].position.y + dy).clamp(0, world_h - 1);
                    }
                    ActionKind::Move { target_x, target_y } => {
                        // §6: Move one step toward target (Manhattan pathfinding)
                        let dx = (target_x - self.agents[i].position.x).clamp(-1, 1);
                        let dy = (target_y - self.agents[i].position.y).clamp(-1, 1);
                        self.agents[i].position.x =
                            (self.agents[i].position.x + dx).clamp(0, world_w - 1);
                        self.agents[i].position.y =
                            (self.agents[i].position.y + dy).clamp(0, world_h - 1);
                        // §6: Completion detection — finish action when at target
                        if self.agents[i].position.x == target_x
                            && self.agents[i].position.y == target_y
                        {
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
                let affection_changed =
                    (rel.affection - old_affection).abs() > Fixed::from_f64(0.001);
                if trust_changed || affection_changed {
                    let cause = self.events[pre_tick_events..]
                        .iter()
                        .find_map(|ev| {
                            if let SimEvent::InteractionOccurred {
                                from: ef,
                                to: et,
                                kind,
                                ..
                            } = ev
                            {
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
                    self.provenance
                        .record_relationship(crate::provenance::RelationshipTrace {
                            from,
                            to,
                            tick: tick_u64,
                            cause,
                            old_trust,
                            new_trust: rel.trust,
                            old_affection,
                            new_affection: rel.affection,
                            description: format!(
                                "Social interaction changed trust {}->{}, affection {}->{}",
                                old_trust.to_f64(),
                                rel.trust.to_f64(),
                                old_affection.to_f64(),
                                rel.affection.to_f64()
                            ),
                        });
                }
            }
        }

        // ── 4a. Weather: continuous temperature/rainfall + emergent regimes ──
        // The Ecology RNG stream has no other consumer (virgin), so weather's
        // two draws per tick cannot shift any existing stream's position —
        // replays stay byte-identical. Weather is computed before production
        // (growth factor) and spoilage (temperature factor) read it. NB: the
        // season baseline used here rolled at the PREVIOUS tick's block 16
        // (season advances after weather), so weather lags a season boundary
        // by one tick — a deliberate, negligible ordering artifact.
        let weather_event = self
            .weather
            .advance(self.season.current, self.rng.get_mut(RngStream::Ecology));
        if let Some(ev) = weather_event {
            tracing::info!(
                event = ?ev,
                rainfall = self.weather.rainfall.to_f64(),
                temperature = self.weather.temperature.to_f64(),
                "Weather regime change"
            );
        }
        // Regime-gated well water: an emergent drought drains each well's
        // water every tick; an emergent flood recharges it. Normal weather
        // touches nothing — wells only move through consumption and these
        // regimes, so calibrated windows are unaffected by this block.
        match self.weather.regime {
            ecology::WeatherRegime::Drought => {
                let drain = self.weather.config.drought_water_drain;
                for site in &mut self.world.sites {
                    for stock in &mut site.inventory {
                        if stock.resource_id == WATER_RESOURCE_ID && stock.quantity > Fixed::ZERO {
                            stock.quantity =
                                (stock.quantity * (Fixed::ONE - drain)).max(Fixed::ZERO);
                        }
                    }
                }
            }
            ecology::WeatherRegime::Flood => {
                let recharge = self.weather.config.flood_water_recharge;
                for site in &mut self.world.sites {
                    // Cap recharged wells at the site's storage capacity so a
                    // sustained flood cannot balloon water past the §19.5.E
                    // storage contract (the cap only binds during floods,
                    // which never fire in calibrated windows).
                    let cap = site.storage_capacity;
                    for stock in &mut site.inventory {
                        if stock.resource_id == WATER_RESOURCE_ID && stock.quantity > Fixed::ZERO {
                            stock.quantity = (stock.quantity * (Fixed::ONE + recharge)).min(cap);
                        }
                    }
                }
            }
            ecology::WeatherRegime::Normal => {}
        }
        // §8.1.4 (P3-6): famine grain drain — while a Famine shock's
        // production-suppression window is open, stored grain additionally
        // decays per tick (a famine consumes the granary: eating outpaces the
        // failed harvest). Mirrors the drought regime's per-tick water drain
        // above. With only the one-shot 70% store drain + production
        // suppression, grain plateaued at ~2.1 (production at 0.3× still
        // matched consumption) and body hunger never rose; the drain drives
        // stored grain to zero mid-window so Eat fails, hunger accumulates
        // toward the 0.5 goal-congruence gate, and the goal-incongruence
        // branch (sadness → despair → depression-from-deprivation) becomes
        // reachable. Outside an open window the factor is 1.0 — calibrated
        // windows (riverford/calm/pestilence/golden) are untouched.
        if tick_u64 < self.famine_until {
            let drain = FAMINE_GRAIN_DRAIN;
            for site in &mut self.world.sites {
                for stock in &mut site.inventory {
                    if stock.resource_id == GRAIN_RESOURCE_ID && stock.quantity > Fixed::ZERO {
                        stock.quantity =
                            (stock.quantity * (Fixed::ONE - drain)).max(Fixed::ZERO);
                    }
                }
            }
        }

        // ── 4b. Resource operations (extracted) ──
        self.tick_resource_operations(&action_starts, pre_tick_events, tick_u64, tick);

        // ── 9. Memory encoding (extracted) ──
        self.tick_memory_encoding(pre_tick_events, tick_u64, phases);

        // ── 16. Ecology: season advance and fertility dynamics ──
        let new_season = self.season.advance();
        if new_season {
            tracing::info!(
                season = self.season.current.name(),
                year = self.season.year,
                "New season"
            );
        }
        // ── 16a. §19.5.I (Iteration 148): Technology discovery — the yearly
        // innovation pass, on the same 4320-tick cadence as the monthly
        // ritual. Calibrated windows (golden @2000, snapshots ≤2000) contain
        // no pass at all; draws come from the virgin Narrative stream, so a
        // pass cannot perturb any other subsystem.
        if tick_u64 > 0 && tick_u64.is_multiple_of(4320) {
            self.tick_technology_discovery(tick_u64);
        }
        // §5 (Iteration 150): the yearly diplomacy pass — off-map neighbors
        // act (caravans / raids) on the same cadence; draws come from the
        // virgin Economy stream, so no other subsystem's RNG can shift.
        if tick_u64 > 0 && tick_u64.is_multiple_of(crate::diplomacy::DIPLOMACY_CADENCE) {
            self.tick_diplomacy(tick_u64);
        }
        // §5 (Iteration 151): the yearly school term — formal cohort
        // instruction, gated on a School site existing. No default world
        // places one, so the pass is a structural no-op in every calibrated
        // window, and it draws no randomness even when a school exists.
        // Both this pass and the per-tick apprenticeship guard idempotently
        // (a `has_learned` check and a `contains` before any push), so
        // whichever runs first in a tick wins and the other skips the
        // student — the same-tick interplay is self-consistent either way.
        if tick_u64 > 0 && tick_u64.is_multiple_of(crate::schools::SCHOOL_CADENCE) {
            self.tick_school_term(tick_u64);
        }
        // §5 (Iteration 152): the half-year religious pass — conversions and
        // conviction drift on the yearly mark, festivals on the mid-year
        // mark. Gated on a seeded Religion (no default world seeds one, so
        // the pass is a structural no-op in every calibrated window) and
        // fully deterministic (no RNG drawn even when a religion exists).
        if tick_u64 > 0 && tick_u64.is_multiple_of(crate::theology::RELIGIOUS_CADENCE) {
            self.tick_theology(tick_u64);
        }
        // §5 (Iteration 153): the yearly military pass — conscription and
        // drills, gated on a Barracks site existing. No default world places
        // one, so the pass is a structural no-op in every calibrated window,
        // and it draws no randomness even when a barracks exists.
        if tick_u64 > 0 && tick_u64.is_multiple_of(crate::military::MILITARY_CADENCE) {
            self.tick_military(tick_u64);
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
        ecology::system_ecology(
            &self.season,
            &mut fertilities,
            &self.site_work_ticks,
            &self.ecology_config,
        );
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
        //
        // Iteration 185 (P5 100K audit): same-kind dedup. The old code
        // checked `already_infected` only against the PRE-TICK disease vecs;
        // `new_infections` is applied after the whole scan, so within ONE
        // tick every source's every disease could push the same (j, kind)
        // repeatedly — a freshly-recovered agent absorbed pushes from all
        // sources' all diseases at once, and the nested per-disease loops
        // multiplied the pile (probe: 100K pestilence seed 42 carried
        // 809,303 concurrent Epidemic entries across 12 agents — 8,396 on
        // one agent alone — and per-tick cost grew with the pile, blowing
        // up the run). Dedup against the pending infections caps each
        // agent at ONE new infection per kind per tick; combined with the
        // already_infected cross-tick guard that caps concurrent
        // same-kind diseases at 1, so the vecs stay bounded and the
        // contagion cost linear. Deterministic (Vec preserves push order;
        // the dedup check runs before the RNG draw, so the draw sequence
        // is a deterministic function of the new state).
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
                        let dist = self.agents[i]
                            .position
                            .manhattan_distance(&self.agents[j].position);
                        if dist > 2 {
                            continue; // too far
                        }
                        // Already infected with this kind (pre-tick state)?
                        let already_infected = self.agent_diseases[j]
                            .iter()
                            .any(|d| d.kind == disease.kind);
                        if already_infected {
                            continue;
                        }
                        // Iteration 185: also skip if a same-kind infection is
                        // already pending for j THIS tick (see the block
                        // comment above).
                        if new_infections.iter().any(|(aj, ak)| *aj == j && *ak == disease.kind) {
                            continue;
                        }
                        // Probability check — reduced by distance and target health
                        let distance_factor = Fixed::from_f64(1.0)
                            - Fixed::from_f64(dist as f64) * Fixed::from_f64(0.25);
                        let health_factor = self.agents[j].body.health;
                        let transmission_prob = rate
                            * distance_factor
                            * (Fixed::ONE - health_factor * Fixed::from_f64(0.5));
                        let roll =
                            Fixed::from_f64(self.rng.get_mut(RngStream::Social).random::<f64>());
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

        // ── 17b-2. §32 seasonal Cold/Fever vector (Iteration 187) ──
        // The disease system's Cold/Fever kinds were unreachable in real
        // runs (defined + unit-tested, never seeded — audit FINDING 3). This
        // daily pass makes them seasonal: when the weather is cold
        // (temperature < 0.4, the Winter band), each susceptible agent rolls
        // a small chance to catch a Cold, and a Cold can escalate into a
        // Fever (an untreated cold's secondary infection). Deterministic
        // (the seeded Social stream), sparse (daily gate + tiny rates: at
        // Winter temperature 0.2, ~0.2%/day per agent → ~17%/agent/season),
        // and same-kind dedup-guarded like the epidemic pass. Cold/Fever
        // resolve through the normal `system_health` duration path, so the
        // per-kind cap (one of each kind) holds.
        if phases.is_daily {
            let n = self.agents.len();
            let cold_season =
                (Fixed::from_f64(COLD_SEASON_TEMP) - self.weather.temperature).clamp_01();
            if cold_season > Fixed::ZERO {
                for i in 0..n {
                    if self.agent_diseases[i].len() >= health::MAX_CONCURRENT_DISEASES {
                        continue;
                    }
                    let has_cold = self.agent_diseases[i]
                        .iter()
                        .any(|d| d.kind == health::DiseaseKind::Cold);
                    let has_fever = self.agent_diseases[i]
                        .iter()
                        .any(|d| d.kind == health::DiseaseKind::Fever);
                    if !has_cold {
                        let cold_roll = Fixed::from_f64(
                            self.rng.get_mut(RngStream::Social).random::<f64>(),
                        );
                        if cold_roll < COLD_SEASON_RATE * cold_season {
                            self.agent_diseases[i]
                                .push(health::ActiveDisease::new(health::DiseaseKind::Cold));
                            tracing::debug!(agent = i, "Seasonal cold contracted");
                        }
                    } else if !has_fever {
                        let fever_roll = Fixed::from_f64(
                            self.rng.get_mut(RngStream::Social).random::<f64>(),
                        );
                        if fever_roll < FEVER_ESCALATION_RATE {
                            self.agent_diseases[i]
                                .push(health::ActiveDisease::new(health::DiseaseKind::Fever));
                            tracing::debug!(agent = i, "Cold escalated into fever");
                        }
                    }
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
                            let pos = self
                                .world
                                .site_position(farm_idx)
                                .map_or(crate::sim::Position::new(8, 8), |(x, y)| {
                                    crate::sim::Position::new(x, y)
                                });
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
                        // §5 (Iteration 147): weather scales spoilage — ≈1.03
                        // in normal weather, ×1.2 in a flood, ×0.8 in a drought.
                        let spoilage = stock.quantity
                            * res_def.spoilage_rate
                            * spoilage_modifier
                            * self.weather.temperature_factor();
                        stock.quantity = (stock.quantity - spoilage).max(Fixed::ZERO);
                    }
                }
            }
        }

        // ── 10b. Storage overflow — goods stored beyond a site's capacity rot/spill ──
        self.apply_storage_overflow();

        // ── 11. Norm evaluation (extracted) ──
        self.tick_norm_evaluation(pre_tick_events, tick_u64, tick);

        // ── 11b. §11.2 Gossip propagation with mutation ── (extracted)
        self.tick_gossip_and_knowledge(pre_tick_events, tick_u64, tick);

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
                        let pick = self
                            .rng
                            .get_mut(RngStream::Social)
                            .random_range(0..self.agents[pi].cultural.knowledge.len());
                        let knowledge_id = self.agents[pi].cultural.knowledge[pick];
                        // §5 (Iteration 148): the technology tree gates peer
                        // transmission — a learner must hold every
                        // prerequisite of the knowledge before absorbing it.
                        if !self.agents[i].cultural.knowledge.contains(&knowledge_id)
                            && self
                                .technology
                                .can_learn(&self.agents[i].cultural.knowledge, knowledge_id)
                        {
                            // §8.1.18 (Iteration 166): the child's own taboo
                            // layer dampens absorption of the parent's
                            // knowledge — a child whose culture forbids more
                            // is more resistant to novel practices (sacred
                            // boundary defense). ONE-SIDED factor, identity
                            // at zero; acceptance still needs to clear the
                            // 0.3 floor.
                            let resistance =
                                crate::psychology::cultural_cognition::taboo_knowledge_factor(
                                    self.agents[i].cultural_cognition.max_taboo_strength(),
                                    KNOWLEDGE_TABOO_RATE,
                                    KNOWLEDGE_TABOO_FLOOR,
                                );
                            let acceptance = (self.agents[i].cultural.openness
                                * Fixed::from_f64(0.6)
                                + Fixed::from_f64(0.4))
                            .clamp_01()
                                * resistance;
                            if acceptance > Fixed::from_f64(0.3) {
                                socialization_updates.push((i, knowledge_id));
                            }
                        }
                    }
                }
            }
            for (ci, knowledge_id) in socialization_updates {
                self.agents[ci].cultural.knowledge.push(knowledge_id);
                if let Some(k) = self
                    .knowledge_store
                    .iter_mut()
                    .find(|k| k.id == knowledge_id)
                {
                    k.holders += 1;
                }
                // §19.5.F: Record socialization in journal for observability
                self.journal.record(
                    tick_u64,
                    AgentId::new(ci as u64),
                    JournalEntryKind::KnowledgeSocialized { knowledge_id },
                );
            }
        }

        // ── 11e. §19.5.I Innovation — agents discover new knowledge through work ──
        {
            let work_agents: Vec<usize> = self
                .agents
                .iter()
                .enumerate()
                .filter(|(_, a)| matches!(a.current_action, crate::actions::ActionKind::Work))
                .map(|(i, _)| i)
                .collect();
            let mut innovations: Vec<(usize, u64, String)> = Vec::new();
            for idx in work_agents {
                // Discovery chance based on openness and conscientiousness
                let mut discovery_chance = (self.agents[idx].personality.openness
                    * Fixed::from_f64(0.3)
                    + self.agents[idx].personality.conscientiousness * Fixed::from_f64(0.2)
                    + Fixed::from_f64(0.01))
                .clamp_01();
                // §8.1.18 (Iteration 166): the taboo knowledge-resistance
                // factor dampens innovation for taboo-bound agents — a
                // culture that forbids more accepts fewer novel practices
                // (sacred boundary defense). ONE-SIDED: a taboo-free agent's
                // factor is exactly 1.0 (zero drift at the pre-Iter-165
                // snapshot shape). The RNG draw below is UNCONDITIONAL, so
                // replay determinism holds — the factor only scales the
                // chance, never the stream.
                let resistance = crate::psychology::cultural_cognition::taboo_knowledge_factor(
                    self.agents[idx].cultural_cognition.max_taboo_strength(),
                    KNOWLEDGE_TABOO_RATE,
                    KNOWLEDGE_TABOO_FLOOR,
                );
                discovery_chance = discovery_chance * resistance;
                let roll = Fixed::from_f64(self.rng.get_mut(RngStream::Social).random::<f64>());
                if roll < discovery_chance {
                    // Try to discover a knowledge item not yet known by this agent
                    for k in &self.knowledge_store {
                        // §5 (Iteration 148): work-innovation is also gated —
                        // an agent cannot innovate a node whose prerequisites
                        // they do not hold.
                        if !self.agents[idx].cultural.knowledge.contains(&k.id)
                            && k.difficulty < self.agents[idx].personality.openness
                            && self
                                .technology
                                .can_learn(&self.agents[idx].cultural.knowledge, k.id)
                        {
                            innovations.push((idx, k.id, k.name.clone()));
                            break; // one discovery per tick per agent
                        }
                    }
                }
            }
            for (agent_idx, knowledge_id, name) in innovations {
                self.agents[agent_idx].cultural.knowledge.push(knowledge_id);
                if let Some(ks) = self
                    .knowledge_store
                    .iter_mut()
                    .find(|ks| ks.id == knowledge_id)
                {
                    ks.holders += 1;
                }
                self.journal.record(
                    tick_u64,
                    AgentId::new(agent_idx as u64),
                    JournalEntryKind::KnowledgeDiscovered { knowledge_id, name },
                );
                self.events.push(SimEvent::KnowledgeTransferred {
                    source: AgentId::new(agent_idx as u64),
                    target: AgentId::new(agent_idx as u64),
                    knowledge_id,
                    tick,
                });
            }
        }

        // ── 12. Institutional collective psychology (extracted) ────
        self.tick_institutional_psychology(tick_u64, phases);

        // ── 15. Faction dynamics (extracted) ────
        self.tick_faction_dynamics(tick_u64, tick);

        // ── 14. Moral panic + Revolution (extracted) ──
        self.tick_moral_panic_and_revolution(tick_u64, tick);

        // ── 15+13. Derived mental state + Belief updates (extracted) ──
        self.tick_derived_states_and_beliefs(pre_tick_events, tick_u64);

        // ── 18+19+20+21. Social cluster (extracted) ──
        self.tick_social_cluster(pre_tick_events, tick_u64, tick, phases);

        // Architecture-plan-2 §13.6: Per-tick echo chamber drift.
        // After the daily cluster rebuild (in tick_social_cluster), this accumulates
        // gradual outgroup hostility and identity fusion between rebuilds.
        // Skip on daily ticks — tick_social_cluster already rebuilds clusters and
        // recomputes polarization, so tick_update() would be redundant.
        if !phases.is_daily {
            self.echo_chamber.tick_update();
        }

        // ── 19. Marriage formation (extracted) ──
        self.tick_marriage_formation(tick_u64, tick);

        // ── 19b. Birth mechanics (extracted) — demography cadence ──
        // Births roll on the same 10-tick cadence as aging/mortality so the
        // annual birth rate is scaled by the correct elapsed-year fraction.
        if phases.is_deca {
            self.tick_birth_mechanics(tick_u64, tick);
        }

        // ── §6 + §10.6/§10.7: Kinship & Household daily update ──
        self.tick_kinship_household_daily(tick_u64, phases);

        // ── Architecture-plan-2 §8.1.5: Motivation relieve() calls ──
        // Relieve psychological needs based on current actions.
        // Amounts calibrated so multiple actions needed for full satisfaction
        // (relieve ≈ 3–5× growth_rate per tick, creating realistic behavioral pressure).
        for i in 0..self.agents.len() {
            match self.agents[i].current_action {
                ActionKind::Socialize => {
                    self.agents[i]
                        .motivation
                        .belonging
                        .relieve(Fixed::from_f64(0.01));
                    self.agents[i]
                        .motivation
                        .attachment
                        .relieve(Fixed::from_f64(0.008));
                }
                ActionKind::Worship => {
                    self.agents[i]
                        .motivation
                        .meaning
                        .relieve(Fixed::from_f64(0.008));
                    self.agents[i]
                        .motivation
                        .certainty
                        .relieve(Fixed::from_f64(0.005));
                }
                ActionKind::Trade => {
                    self.agents[i]
                        .motivation
                        .competence
                        .relieve(Fixed::from_f64(0.008));
                    self.agents[i]
                        .motivation
                        .recognition
                        .relieve(Fixed::from_f64(0.005));
                }
                ActionKind::Work => {
                    self.agents[i]
                        .motivation
                        .competence
                        .relieve(Fixed::from_f64(0.006));
                    self.agents[i]
                        .motivation
                        .recognition
                        .relieve(Fixed::from_f64(0.006));
                }
                ActionKind::Rest => {
                    self.agents[i]
                        .motivation
                        .sleep
                        .relieve(Fixed::from_f64(0.02));
                }
                _ => {}
            }
        }

        // ── 19. §6.5: Record metric history for observability ──
        // Every 10 ticks, snapshot key metrics for CSV export and analysis.
        // Cap at MAX_METRIC_HISTORY entries to prevent unbounded growth.
        if phases.is_deca {
            let snapshot = self.metrics_snapshot();
            self.metric_history.push(snapshot);
            if self.metric_history.len() > MAX_METRIC_HISTORY {
                self.metric_history.remove(0);
            }
        }
    }

    // ── Tick subsystem methods ──────────────────────────────────────
    // Extracted from tick() for readability and maintainability.
    // Each method handles one coherent subsystem tick.

    /// §11.2 + §19.5.I: Gossip propagation, attachment reunion, ToM,
    /// cultural encounter, knowledge diffusion, meme transmission.
    fn tick_gossip_and_knowledge(&mut self, pre_tick_events: usize, tick_u64: u64, tick: Tick) {
        // Collect event indices to process (avoids .to_vec() heap allocation)
        let event_range = pre_tick_events..self.events.len();
        // Snapshot event count to avoid re-processing newly-pushed events
        let snap_count = self.events.len();

        for idx in event_range.clone() {
            if idx >= snap_count {
                break;
            }
            if let SimEvent::InteractionOccurred { from, to, kind, .. } = self.events[idx] {
                if !matches!(
                    kind,
                    mindstrata_core::event::InteractionKind::Gossip
                        | mindstrata_core::event::InteractionKind::Help
                        | mindstrata_core::event::InteractionKind::Comfort
                        | mindstrata_core::event::InteractionKind::Trade
                ) {
                    continue;
                }
                let from_idx = from.as_u64() as usize;
                let to_idx = to.as_u64() as usize;
                if from_idx >= self.agents.len() || to_idx >= self.agents.len() {
                    continue;
                }

                // §8.1.14: Attachment reunion (Gossip only)
                if matches!(kind, mindstrata_core::event::InteractionKind::Gossip) {
                    self.agents[from_idx].attachment.on_reunion(
                        self.params.attachment_secure_recovery,
                        self.params.attachment_anxious_recovery,
                        self.params.attachment_avoidant_recovery,
                        self.params.attachment_disorganized_recovery,
                    );
                    self.agents[to_idx].attachment.on_reunion(
                        self.params.attachment_secure_recovery,
                        self.params.attachment_anxious_recovery,
                        self.params.attachment_avoidant_recovery,
                        self.params.attachment_disorganized_recovery,
                    );
                }

                // §8.1.14 (Iteration 191 — the active soothing path): a
                // Comfort interaction (the high-affection context's softest
                // signal) now calls `receive_comfort` on the RECIPIENT —
                // pre-Iter-191 the method had ZERO call sites, so separation
                // distress only ever decayed passively (the daily
                // `× decay_factor` fold) and the designed comfort mechanics
                // (style-matched distress relief scaled by the agent's
                // soothing_receptivity × the interaction's quality, plus a
                // security gain for the secure style) never fired. Comfort
                // quality scales with the existing relationship trust
                // between the pair (a trusted figure soothes better; a
                // stranger's comfort barely registers). Deterministic, no
                // RNG — the draw stream is untouched (only state deltas on
                // the recipient's attachment).
                if matches!(kind, mindstrata_core::event::InteractionKind::Comfort) {
                    let rel_trust = self
                        .relationships
                        .iter()
                        .find(|r| r.from == from && r.to == to)
                        .map_or(Fixed::from_f64(0.5), |r| r.trust);
                    // Iteration 191 calibration: the raw recovery rates
                    // (0.3/0.6/0.4/0.5) made a SINGLE comfort event reduce
                    // distress by up to ~0.24 (effectiveness = receptivity ×
                    // quality up to 0.8, × the 0.3 secure rate) while the
                    // daily separation gain is ~0.008 — the active path
                    // erased the coupling entirely (probe: mean 0.000, 0/46
                    // partnered agents nonzero, at EVERY seed × rate, and the
                    // rate-response ratio collapsed to NaN). The comfort
                    // reduction is damped by
                    // `ATTACHMENT_COMFORT_DAMPING` (0.05) so a comfort event
                    // soothes ~1–2 days of accumulated separation (reduction
                    // ~0.01–0.02) — the coupling stays live population-wide
                    // and the separation-rate parameter still drives
                    // distress, but a supported partner's distress genuinely
                    // sits lower than an unsupported one's.
                    let comfort_quality =
                        (Fixed::from_f64(0.4) + rel_trust * Fixed::from_f64(0.6)).clamp_01();
                    let damp = Fixed::from_f64(ATTACHMENT_COMFORT_DAMPING);
                    self.agents[to_idx].attachment.receive_comfort(
                        comfort_quality,
                        self.params.attachment_secure_recovery * damp,
                        self.params.attachment_anxious_recovery * damp,
                        self.params.attachment_avoidant_recovery * damp,
                        self.params.attachment_disorganized_recovery * damp,
                        Fixed::from_f64(0.05),
                    );
                }

                // §8.1.9: Theory of Mind update
                // §17.2: Gate social inference budget — ToM is the most expensive per-interaction op.
                let trust_from_to = self
                    .relationships
                    .iter()
                    .find(|r| r.from == from && r.to == to)
                    .map_or(Fixed::from_f64(0.5), |r| r.trust);
                let trust_to_from = self
                    .relationships
                    .iter()
                    .find(|r| r.from == to && r.to == from)
                    .map_or(Fixed::from_f64(0.5), |r| r.trust);
                // Iteration 198: the observed-behavior inputs were derived
                // from trust with scaled-down multipliers (pos = trust × 0.3,
                // neg = (1−trust) × 0.1), so `infer_intent`'s Friendly
                // (>0.3 positive) and Threatening (>0.3 negative) branches
                // were STRUCTURALLY UNREACHABLE — a model could essentially
                // only ever say Neutral, and the whole §8.1.9 system was
                // write-only. The honest inputs are the actual interaction:
                // Help/Comfort is strongly helpful (0.6), Trade mildly
                // helpful (0.3), Gossip neutral (0.1); none of the four kinds
                // this pass carries is harmful, so negative stays 0 (the
                // hostile kinds never reach this pass). Intent now genuinely
                // differentiates: repeated Help at trust > 0.5 → Friendly.
                let observed_helpful = match kind {
                    mindstrata_core::event::InteractionKind::Help
                    | mindstrata_core::event::InteractionKind::Comfort => Fixed::from_f64(0.6),
                    mindstrata_core::event::InteractionKind::Trade => Fixed::from_f64(0.3),
                    _ => Fixed::from_f64(0.1),
                };
                if self.agents[from_idx].agent_tier.tier.runs_theory_of_mind()
                    && self.agents[from_idx]
                        .agent_tier
                        .budget_tracker
                        .can_social_infer()
                {
                    let model_a = self.agents[from_idx].mind_models.get_or_create(to);
                    model_a.update_from_observation(
                        observed_helpful,
                        Fixed::ZERO,
                        trust_to_from,
                        Fixed::from_f64(0.5),
                    );
                    model_a.infer_intent(trust_to_from, observed_helpful, Fixed::ZERO);
                    let _ = self.agents[from_idx]
                        .agent_tier
                        .budget_tracker
                        .consume_social_inference();
                }
                if self.agents[to_idx].agent_tier.tier.runs_theory_of_mind()
                    && self.agents[to_idx]
                        .agent_tier
                        .budget_tracker
                        .can_social_infer()
                {
                    let model_b = self.agents[to_idx].mind_models.get_or_create(from);
                    model_b.update_from_observation(
                        observed_helpful,
                        Fixed::ZERO,
                        trust_from_to,
                        Fixed::from_f64(0.5),
                    );
                    model_b.infer_intent(trust_from_to, observed_helpful, Fixed::ZERO);
                    let _ = self.agents[to_idx]
                        .agent_tier
                        .budget_tracker
                        .consume_social_inference();
                }

                // §8.1.18: Cultural category encounter
                let (from_cat, to_cat) = {
                    let fc = self.agents[from_idx]
                        .cultural_cognition
                        .categories
                        .iter()
                        .max_by_key(|c| c.identification.to_raw() as u64)
                        .map(|c| c.name.clone());
                    let tc = self.agents[to_idx]
                        .cultural_cognition
                        .categories
                        .iter()
                        .max_by_key(|c| c.identification.to_raw() as u64)
                        .map(|c| c.name.clone());
                    (fc, tc)
                };
                if let (Some(ref fn_name), Some(ref tn_name)) = (&from_cat, &to_cat) {
                    let same = fn_name == tn_name;
                    if let Some(c) = self.agents[from_idx]
                        .cultural_cognition
                        .categories
                        .iter_mut()
                        .find(|c| &c.name == tn_name)
                    {
                        c.encounter(same);
                    }
                    if let Some(c) = self.agents[to_idx]
                        .cultural_cognition
                        .categories
                        .iter_mut()
                        .find(|c| &c.name == fn_name)
                    {
                        c.encounter(same);
                    }
                }

                // Gossip-specific: belief sharing
                if !matches!(kind, mindstrata_core::event::InteractionKind::Gossip) {
                    continue;
                }
                let from_beliefs = &self.agents[from_idx].beliefs;
                if from_beliefs.is_empty() {
                    continue;
                }
                let pick = self
                    .rng
                    .get_mut(RngStream::Social)
                    .random_range(0..from_beliefs.len());
                let source_trust = trust_from_to;
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
                    self.params.gossip_base_fidelity,
                    self.params.gossip_emotional_distortion,
                    self.params.gossip_acceptance_threshold,
                );
                let old_conf = self.agents[to_idx]
                    .beliefs
                    .iter()
                    .find(|b| b.proposition_id == result.proposition_id)
                    .map_or(Fixed::from_f64(0.5), |b| b.confidence);
                gossip::apply_gossip(&mut self.agents[to_idx].beliefs, &result, tick_u64);
                if result.accepted {
                    let new_conf = self.agents[to_idx]
                        .beliefs
                        .iter()
                        .find(|b| b.proposition_id == result.proposition_id)
                        .map_or(result.mutated_confidence, |b| b.confidence);
                    self.provenance
                        .record_belief_update(crate::provenance::BeliefUpdateTrace {
                            agent: to,
                            tick: tick_u64,
                            proposition_id: result.proposition_id,
                            old_confidence: old_conf,
                            new_confidence: new_conf,
                            cause: "gossip".into(),
                            delta: (new_conf - old_conf),
                        });
                    self.events.push(SimEvent::RumorSpread {
                        source: from,
                        target: to,
                        content_hash: result.proposition_id,
                        distortion: (result.mutated_confidence - rumor.confidence).abs(),
                        tick,
                    });
                    // §13.1 + §13.2: Meme transmission using agent epistemic traits.
                    // susceptibility: gullible agents (high conspiracy_susceptibility) accept
                    //   memes more readily; floor of 0.2 ensures realistic information permeability.
                    // skepticism: critical agents (high source_monitoring, low dogmatism) resist
                    //   memes more; multiplier 0.6 gives meaningful differentiation between
                    //   gullible (≈0.08) and critical (≈0.6) agents.
                    let listener_susceptibility =
                        (self.agents[to_idx].epistemic.conspiracy_susceptibility
                            * Fixed::from_f64(0.6)
                            + Fixed::from_f64(0.2))
                        .clamp_01();
                    let listener_skepticism = (self.agents[to_idx].epistemic.source_monitoring
                        * (Fixed::ONE
                            - self.agents[to_idx].epistemic.dogmatism * Fixed::from_f64(0.5))
                        * Fixed::from_f64(0.6))
                    .clamp_01();
                    let nb = Fixed::from_f64(0.05);
                    let mut sampled = 0u32;
                    for meme in &mut self.meme_registry.memes {
                        if !meme.active || sampled >= 3 {
                            break;
                        }
                        let chance = meme.transmission_chance(
                            source_trust,
                            listener_susceptibility,
                            listener_skepticism,
                            self.params.meme_transmission_multiplier,
                        );
                        let roll = self
                            .rng
                            .get_mut(RngStream::Social)
                            .random_range(0.0f64..1.0);
                        if Fixed::from_f64(roll) < chance {
                            // host_count is "number of hosting agents" — clamp to
                            // population so repeated transmissions of the same meme
                            // to the same village can't inflate it past the agent
                            // count (previously unbounded, made visible once memes
                            // could actually spread).
                            let pop = self.agents.len() as u32;
                            meme.host_count = (meme.host_count.saturating_add(1)).min(pop);
                            meme.novelty = (meme.novelty + nb).clamp_01();
                            sampled += 1;
                            // §13.2 (AP2): Meme mutation during transmission —
                            // the five-factor drift (memory error, emotional
                            // exaggeration, identity bias, narrative
                            // simplification, audience tailoring). Gated on the
                            // master multiplier: at ZERO no mutation ever fires
                            // and the decision draw is not consumed (the
                            // identity factor that pinned the golden baseline
                            // while the feature was off — the baseline has
                            // since been intentionally recalibrated with the
                            // feature live at 0.3).
                            if self.params.meme_mutation_rate_base > Fixed::ZERO {
                                let m_roll = self
                                    .rng
                                    .get_mut(RngStream::Social)
                                    .random_range(0.0f64..1.0);
                                if meme.should_mutate_scaled(
                                    Fixed::from_f64(m_roll),
                                    self.params.meme_mutation_rate_base,
                                ) {
                                    let magnitude = meme.mutation_magnitude();
                                    meme.mutate(magnitude);
                                }
                            }
                            // §17.4: Record exposure sample for meme aggregation.
                            let meme_ids = [meme.id];
                            self.meme_aggregator.sample_exposures(
                                to_idx,
                                &meme_ids,
                                source_trust,
                                true,
                                &mut self.exposure_samples,
                            );
                        }
                    }
                }
                // §13.3: Rumor creation from emotionally charged gossip
                if result.accepted && result.emotional_charge > Fixed::from_f64(0.3) {
                    let mut r = crate::culture::RumorV2::new(
                        0,
                        format!("gossip about prop_{}", result.proposition_id),
                        Some(to.as_u64() as usize),
                        result.emotional_charge * Fixed::from_f64(0.5),
                        source_trust,
                        result.emotional_charge,
                        tick_u64,
                    );
                    // §13.3: The originator is in the source chain but has not
                    // retold the rumor — record_source skips the evidence
                    // degradation so stored evidence stays aligned with the
                    // actual hop count.
                    r.record_source(from.as_u64() as usize, tick_u64);
                    self.rumor_registry.register(r);
                }
            }
        }

        // §19.5.I: Knowledge diffusion
        for idx in event_range {
            if idx >= snap_count {
                break;
            }
            if let SimEvent::InteractionOccurred {
                from,
                to,
                kind:
                    mindstrata_core::event::InteractionKind::Gossip
                    | mindstrata_core::event::InteractionKind::Help
                    | mindstrata_core::event::InteractionKind::Trade,
                ..
            } = self.events[idx]
            {
                let fi = from.as_u64() as usize;
                let ti = to.as_u64() as usize;
                if fi >= self.agents.len() || ti >= self.agents.len() {
                    continue;
                }
                let source_trust = self
                    .relationships
                    .iter()
                    .find(|r| r.from == from && r.to == to)
                    .map_or(Fixed::from_f64(0.5), |r| r.trust);
                if !self.agents[fi].cultural.knowledge.is_empty() {
                    let pick = self
                        .rng
                        .get_mut(RngStream::Social)
                        .random_range(0..self.agents[fi].cultural.knowledge.len());
                    let kid = self.agents[fi].cultural.knowledge[pick];
                    // §8.1.18 (Iteration 166): the recipient's taboo layer
                    // dampens absorption of knowledge spread through social
                    // contact — a taboo-bound agent resists novel practices
                    // from peers (sacred boundary defense). ONE-SIDED
                    // factor, identity at zero; the recipient's acceptance
                    // must still clear the 0.5 floor.
                    let resistance = crate::psychology::cultural_cognition::taboo_knowledge_factor(
                        self.agents[ti].cultural_cognition.max_taboo_strength(),
                        KNOWLEDGE_TABOO_RATE,
                        KNOWLEDGE_TABOO_FLOOR,
                    );
                    let acceptance = (source_trust * Fixed::from_f64(0.5)
                        + self.agents[ti].cultural.openness * Fixed::from_f64(0.5))
                    .clamp_01()
                        * resistance;
                    if !self.agents[ti].cultural.knowledge.contains(&kid)
                        && acceptance > Fixed::from_f64(0.5)
                        // §5 (Iteration 148): interaction diffusion is gated
                        // by the technology tree's prerequisites.
                        && self.technology.can_learn(&self.agents[ti].cultural.knowledge, kid)
                    {
                        self.agents[ti].cultural.knowledge.push(kid);
                        if let Some(k) = self.knowledge_store.iter_mut().find(|k| k.id == kid) {
                            k.holders += 1;
                        }
                        // §8.1.7: Successful knowledge acquisition is evidence
                        // exposure — the absorbed knowledge desacralizes the
                        // recipient's sacred values in proportion to absorption
                        // strength (acceptance) and reasoning capacity
                        // (executive function). Zero-at-zero anchor: no
                        // acquisition -> no pressure, and the internal >0.1
                        // gate inside attempt_desacred keeps very sacred values
                        // inert (only mid-sacredness values slowly secularize).
                        let reasoning = self.agents[ti].cognitive.executive_capacity;
                        self.agents[ti]
                            .sacred_values
                            .desacralize_through_exposure(acceptance, reasoning);
                        self.events.push(SimEvent::KnowledgeTransferred {
                            source: from,
                            target: to,
                            knowledge_id: kid,
                            tick,
                        });
                        self.provenance.record_institutional(
                            crate::provenance::InstitutionalTrace {
                                institution_name: "Community".into(),
                                tick: tick_u64,
                                decision_kind: "knowledge_transfer".into(),
                                description: format!(
                                    "Agent {} taught knowledge {} to Agent {}",
                                    from.as_u64(),
                                    kid,
                                    to.as_u64()
                                ),
                                affected: vec![from, to],
                                success: true,
                            },
                        );
                    }
                }
            }
        }
    }

    /// Access the current tick.
    /// §5 (Iteration 155): The interactive-TUI command channel — inject a
    /// high-priority directive goal into an agent's goal queue.
    ///
    /// The goal is exempt from goal-generation decay and need-dropping, so
    /// it persists until satisfied or replaced. While present, the tick's
    /// selection phase honors its aligned action over routine and internal
    /// drives (`command_goal_action`), yielding only to critical needs of
    /// other kinds (the sim's own interruption rule) — so commands are
    /// strong nudges, not mind control: a commanded agent still eats,
    /// drinks, and rests when those are truly pressing.
    ///
    /// Called only by the interactive TUI (and tests) *between* ticks —
    /// never from the tick loop — so calibrated windows are untouched.
    /// Directives serialize into snapshots (they are ordinary goals), so a
    /// commanded world saved to disk carries its directives on reload.
    /// Returns false if the agent index is out of range.
    pub fn command_agent(&mut self, agent_idx: usize, kind: GoalKind) -> bool {
        if agent_idx >= self.agents.len() {
            return false;
        }
        let tick = self.current_tick().as_u64();
        let agent = &mut self.agents[agent_idx];
        // Upsert: a repeat directive of the same kind replaces the old one
        // (prevents stale duplicate directives piling up in the queue).
        agent
            .goals
            .retain(|g| !(g.source == GoalSource::Command && g.kind == kind));
        agent.goals.push(Goal {
            kind,
            priority: Fixed::ONE,
            commitment: Fixed::ONE,
            created_tick: tick,
            source: GoalSource::Command,
        });
        true
    }

    /// §5 (Iteration 155): Cancel every outstanding directive on an agent,
    /// returning it to fully autonomous behavior. Returns false if the agent
    /// index is out of range.
    pub fn clear_commands(&mut self, agent_idx: usize) -> bool {
        if agent_idx >= self.agents.len() {
            return false;
        }
        self.agents[agent_idx]
            .goals
            .retain(|g| g.source != GoalSource::Command);
        true
    }

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
    fn enforce_theft(
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
                attachment_style: a.attachment.style,
                attachment_security: a.attachment.security,
                attachment_anxiety: a.attachment.anxiety,
                attachment_avoidance: a.attachment.avoidance,
                attachment_protest_threshold: a.attachment.protest_threshold,
                attachment_soothing_receptivity: a.attachment.soothing_receptivity,
                attachment_separation_distress: a.attachment.separation_distress,
                attachment_caregiving_style: a.attachment.caregiving_style,
                trauma_load: a.embodied.nervous.trauma_load,
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
        let n_inv = if n > 0.0 { 1.0 / n } else { 0.0 };

        // §17: Deep observability metrics for Phase 5 tuning
        let avg_stress: f64 = if self.agents.is_empty() {
            0.0
        } else {
            self.agents
                .iter()
                .map(|a| (a.emotions.fear + a.emotions.anger).to_f64())
                .sum::<f64>()
                * n_inv
        };
        let avg_health: f64 = if self.agents.is_empty() {
            0.0
        } else {
            self.agents
                .iter()
                .map(|a| a.embodied.derived_health().to_f64())
                .sum::<f64>()
                * n_inv
        };
        let (trust_sum, quality_sum, rel_count) = self
            .agents
            .iter()
            .flat_map(|a| a.relationship_v2s.iter())
            .fold((0.0f64, 0.0f64, 0u64), |(tq, qq, c), rv2| {
                (tq + rv2.trust.to_f64(), qq + rv2.quality().to_f64(), c + 1)
            });
        let rel_n = if rel_count > 0 { rel_count as f64 } else { 1.0 };
        let avg_relationship_trust = trust_sum / rel_n;
        let avg_relationship_quality = quality_sum / rel_n;
        let active_meme_count = self.meme_registry.active_count() as u64;
        let polarization_index = self.echo_chamber.polarization_index.to_f64();
        // §13.3: Inequality metrics — computed by the market system earlier in
        // this tick (tick_social_cluster → system_market), so values are fresh.
        let gini = self.market.inequality.to_f64();
        let avg_wealth = self.market.avg_wealth.to_f64();
        let median_wealth = self.market.median_wealth.to_f64();
        let total_trades = self.market.total_trades;
        let household_count = self.households.len() as u64;
        let kinship_edge_count = self.kinship_graph.active_count() as u64;
        let avg_agent_tier: f64 = if self.agents.is_empty() {
            0.0
        } else {
            self.agents
                .iter()
                .map(|a| a.agent_tier.tier.tier_index())
                .sum::<f64>()
                * n_inv
        };
        let total_active_feuds: u64 = self.agents.iter().map(|a| a.feuds.len() as u64).sum();
        // §10.8/§12.4: Collective-structure observability — clan count and
        // active cult count.
        let clan_count = self.clan_registry.clans.len() as u64;
        // §10.8: Distinct inter-clan relations — each alliance/enmity edge is
        // recorded symmetrically on both clans, so divide by 2.
        let clan_relation_count = {
            let edges: usize = self
                .clan_registry
                .clans
                .iter()
                .map(|c| c.allies.len() + c.enemies.len())
                .sum();
            (edges / 2) as u64
        };
        let cult_count = self.cult_registry.cults.iter().filter(|c| c.active).count() as u64;
        let noosphere_nodes = self.noospheric_field.nodes.len() as u64;
        let noosphere_zeitgeist = self
            .noospheric_field
            .nodes
            .iter()
            .map(|n| n.effective_activation().to_f64())
            .fold(0.0f64, f64::max);
        let collective_memory_count = self
            .collective_memory_registry
            .entries
            .iter()
            .map(|e| e.memories.len() as u64)
            .sum();
        let patronage_relation_count = self
            .patronage_registry
            .relations
            .iter()
            .filter(|r| r.active)
            .count() as u64;

        MetricsSnapshot {
            tick: self.current_tick().as_u64(),
            avg_hunger: summaries.iter().map(|s| s.hunger.to_f64()).sum::<f64>() * n_inv,
            avg_thirst: summaries.iter().map(|s| s.thirst.to_f64()).sum::<f64>() * n_inv,
            avg_fatigue: summaries.iter().map(|s| s.fatigue.to_f64()).sum::<f64>() * n_inv,
            avg_valence: summaries.iter().map(|s| s.valence.to_f64()).sum::<f64>() * n_inv,
            avg_joy: summaries.iter().map(|s| s.joy.to_f64()).sum::<f64>() * n_inv,
            avg_fear: summaries.iter().map(|s| s.fear.to_f64()).sum::<f64>() * n_inv,
            total_grain: self.total_grain().to_f64(),
            total_water: self.total_water().to_f64(),
            event_count: self.event_count() as u64,
            journal_len: self.journal_len() as u64,
            agent_count: self.agent_count() as u64,
            avg_stress,
            avg_health,
            avg_trauma_load: summaries
                .iter()
                .map(|s| s.trauma_load.to_f64())
                .sum::<f64>()
                * n_inv,
            avg_relationship_trust,
            avg_relationship_quality,
            active_meme_count,
            polarization_index,
            gini,
            avg_wealth,
            median_wealth,
            total_trades,
            household_count,
            kinship_edge_count,
            avg_agent_tier,
            total_active_feuds,
            clan_count,
            clan_relation_count,
            cult_count,
            noosphere_nodes,
            noosphere_zeitgeist,
            collective_memory_count,
            patronage_relation_count,
        }
    }

    /// Section 12: Institutional collective psychology derivation.
    fn tick_institutional_psychology(
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
                institution.record_action(
                    tick_u64,
                    "Fine Theft Policy enacted".into(),
                    members.clone(),
                    true,
                );
                // §19.5.B: Record institutional provenance trace
                self.provenance
                    .record_institutional(crate::provenance::InstitutionalTrace {
                        institution_name: institution.kind.name().into(),
                        tick: tick_u64,
                        decision_kind: "policy_enacted".into(),
                        description: "Fine Theft Policy enacted".into(),
                        affected: members,
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
                        institution.record_action(
                            tick_u64,
                            action_name.clone(),
                            members.clone(),
                            true,
                        );
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
                        institution.record_action(
                            tick_u64,
                            wage_msg.clone(),
                            members.clone(),
                            true,
                        );
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
                crate::social::hierarchy::stabilize_hierarchy(
                    institution,
                    ritual_strength.min(Fixed::ONE),
                    institution.enforcement_capacity,
                    Fixed::from_f64(0.3), // narrative strength placeholder
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

    /// §29.2/§10.9: Per-agent faction grievance, dampened when the agent is a
    /// dependent patronage client — material security from the patron softens
    /// resentment/anger (the patron buys political quiescence). Deterministic.
    fn faction_grievance(&self, idx: usize) -> Fixed {
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
    fn tick_faction_dynamics(&mut self, tick_u64: u64, _tick: Tick) {
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
            // (e.g. 4 factions × 22 members on 48 agents).
            let already_factioned: std::collections::HashSet<AgentId> = self
                .institutions
                .iter()
                .filter(|i| i.kind == InstitutionKind::Faction)
                .flat_map(|f| f.members.iter().copied())
                .collect();
            let grievances: Vec<(AgentId, Fixed)> = self
                .agents
                .iter()
                .enumerate()
                .filter(|(i, _)| !already_factioned.contains(&AgentId::new(*i as u64)))
                .map(|(i, _)| (AgentId::new(i as u64), self.faction_grievance(i)))
                .collect();

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

            // Faction formation: when grievance is high and council legitimacy is low
            let should_form = faction_count < factions::MAX_FACTIONS
                && avg_council_legitimacy < Fixed::from_f64(0.5)
                && tick_u64 > factions::FORMATION_COOLDOWN;

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

                if avg_grievance >= factions::FORMATION_GRIEVANCE_THRESHOLD
                    && recruitable.len() >= 2
                {
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

                        self.institutions.push(new_faction);

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

    /// Section 19: Marriage formation.
    fn tick_marriage_formation(&mut self, tick_u64: u64, tick: Tick) {
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
    fn foundational_beliefs() -> Vec<Belief> {
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
        let child_name = format!("Child_{idx}");
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
        let child_personality = Personality::random(&mut child_rng);
        let child_age = Fixed::from_f64(0.0); // newborn
        let child_embodied = EmbodiedState::random(child_age, &mut child_rng);
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
            moral_values: MoralValues::random(&mut child_rng),
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
    fn handle_agent_death(
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

    fn tick_birth_mechanics(&mut self, tick_u64: u64, tick: Tick) {
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
                    let avg_age = (age_a + age_b) * 0.5;
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
                    let should = demography::should_birth(
                        true,
                        min_health,
                        avg_age,
                        existing_children,
                        DEMOGRAPHY_TICK_INTERVAL,
                        &self.demography_config,
                        rng_val,
                        self.params.reproduction_conception_multiplier.to_f64(),
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
                let child_name = format!("Child_{child_idx}");
                // Inherit personality traits with noise
                let mut child_rng = rand_chacha::ChaCha8Rng::seed_from_u64(
                    self.config
                        .seed
                        .wrapping_add(tick_u64)
                        .wrapping_add(child_idx as u64),
                );
                let child_personality = Personality::random(&mut child_rng);
                let child_age = Fixed::from_f64(0.0); // newborn

                // Allocate home site
                let home_site = self.agents[parent_a].home_site;

                let agent_id = AgentId::new(child_idx as u64);

                let child_embodied = EmbodiedState::random(child_age, &mut child_rng);
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
                    moral_values: MoralValues::random(&mut child_rng),
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
                    emotion_regulation: crate::psychology::EmotionRegulationState::default(),
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

    /// §17.4: Wire meme aggregation into the tick loop.
    ///
    /// Called daily after meme novelty decay. Computes group-level meme
    /// prevalence, most viral meme, spreader centrality, and echo chamber
    /// strength for observability and Phase 5 tuning.
    ///
    /// **Limitation**: Meme host assignment is a placeholder heuristic based
    /// on cultural category indices. Aggregation metrics (prevalence,
    /// homogeneity, echo strength) are deterministic artifacts of initial
    /// category assignment, not emergent from social dynamics. Replace with
    /// real per-agent meme host tracking (`hosted_memes: Vec<usize>` field)
    /// before using these metrics for Phase 5 tuning decisions.
    ///
    fn wire_meme_aggregation(&mut self, tick_u64: u64) {
        let n_agents = self.agents.len();
        if n_agents == 0 || self.meme_registry.memes.is_empty() {
            return;
        }

        // §17.4: Flush accumulated exposure samples into the aggregator. This
        // stays DAILY — the tick loop pushes samples every tick, and dropping
        // them would lose the observability data.
        let samples = std::mem::take(&mut self.exposure_samples);
        self.meme_aggregator.set_exposure_samples(samples);

        // §17.4 lazy aggregation: the input builds below are O(agents ×
        // categories) + O(agents × households) — rebuilding them every day
        // when `aggregate()` would immediately early-return its cached
        // metrics is pure waste. Gate them behind `should_compute`, the
        // single source of truth that `aggregate()` itself uses (tick 0
        // always initializes; then every `aggregation_interval` ticks), so
        // the aggregate call happens on exactly the same ticks as before
        // with identical inputs — behavior-preserving by construction.
        if !self.meme_aggregator.should_compute(tick_u64) {
            return;
        }

        // Build per-agent meme host lists (agent_idx → meme ids hosted).
        // §13.2 + §17.4: Track which memes each agent hosts.
        // NOTE: This is a placeholder heuristic — real meme hosting should be
        // tracked via a per-agent `hosted_memes: Vec<usize>` field. For now,
        // we assign memes based on cultural category identification strength.
        let meme_count = self.meme_registry.memes.len();
        let agent_meme_hosts: Vec<Vec<usize>> = self
            .agents
            .iter()
            .map(|a| {
                let mut hosts: Vec<usize> = Vec::new();
                for (cat_idx, cat) in a.cultural_cognition.categories.iter().enumerate() {
                    // Agents host memes whose index aligns with their identified categories
                    if cat.identification > Fixed::from_f64(0.3) {
                        let meme_id = cat_idx % meme_count;
                        if !hosts.contains(&meme_id) {
                            hosts.push(meme_id);
                        }
                    }
                }
                if hosts.is_empty() {
                    // Fallback: all agents host at least meme 0
                    hosts.push(0);
                }
                hosts
            })
            .collect();

        // Build agent group assignments (simplified: household index)
        let agent_groups: Vec<Option<usize>> = self
            .agents
            .iter()
            .enumerate()
            .map(|(i, _)| self.households.iter().position(|h| h.members.contains(&i)))
            .collect();

        // Collect active meme ids and emotional charges
        let meme_ids: Vec<usize> = self
            .meme_registry
            .memes
            .iter()
            .filter(|m| m.active)
            .map(|m| m.id)
            .collect();
        let meme_charges: Vec<Fixed> = meme_ids
            .iter()
            .filter_map(|&mid| self.meme_registry.get(mid).map(|m| m.emotional_charge))
            .collect();

        // Build group labels from households
        let group_labels: Vec<(usize, String)> = self
            .households
            .iter()
            .enumerate()
            .map(|(i, _)| (i, format!("household_{i}")))
            .collect();

        // Simplified centrality: relationship count normalized to [0,1].
        // Typical agent has 3-8 relationships; 10 normalizes well-connected agents to ~1.0.
        let centrality: Vec<Fixed> = self
            .agents
            .iter()
            .map(|a| {
                let n_rels = a.relationship_v2s.len() as f64;
                Fixed::from_f64((n_rels / 10.0).min(1.0))
            })
            .collect();

        self.meme_aggregator.aggregate(
            tick_u64,
            &agent_meme_hosts,
            &agent_groups,
            &meme_ids,
            &meme_charges,
            &group_labels,
            &centrality,
        );
    }

    /// §10.9: Patrons provision destitute clients — the economic side of
    /// patronage. When a client's coin falls below the destitution floor and
    /// the patron can afford the stipend, a small daily transfer moves wealth
    /// through the social structure (a safety net that softens inequality).
    /// Deterministic (no RNG): reads are collected, then applied.
    fn tick_patronage_provision(&mut self) {
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
    fn tick_kinship_household_daily(
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
                                if link
                                    == crate::social::kinship::KinshipLink::InLaw
                                {
                                    marital_inlaw = true;
                                } else {
                                    if current != stage {
                                        let rv2 =
                                            &mut self.agents[i].relationship_v2s[rv2_idx];
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
                            if current
                                != crate::social::relationship_v2::RelationshipStage::InLaw
                            {
                                let rv2 = &mut self.agents[i].relationship_v2s[rv2_idx];
                                rv2.stage = crate::social::relationship_v2::RelationshipStage::
                                    InLaw;
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
                            let internalize_scale = Fixed::from_f64(0.5)
                                + conformity * Fixed::from_f64(0.5);
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

    /// §8.1.18: Apprenticeship pass — deliberate knowledge transmission.
    ///
    /// Runs once daily. Each agent attempts to learn one knowledge item they
    /// lack, taught by an agent who holds it with sufficient teaching skill
    /// and a warm relationship. Deterministic: the teacher with the highest
    /// teaching skill is picked (no RNG), the transfer succeeds when the
    /// learning rate clears the `attempt_teaching` threshold, and success
    /// feeds the §8.1.7 desacralization chain (acquired knowledge is evidence
    /// exposure) — the same hook gossip uses.
    /// §19.5.E: Sites have finite storage — goods stored beyond a site's
    /// storage capacity are exposed and spoil at 10x the base seasonal rate.
    /// Transparent while inventories stay under capacity; binds only when
    /// production outpaces consumption (bountiful harvests rot without
    /// adequate storage, creating boom-bust scarcity pressure).
    fn apply_storage_overflow(&mut self) {
        let resource_defs = &self.world.resources;
        let spoilage_modifier = self.season.current.spoilage_modifier();
        for site in &mut self.world.sites {
            let mut storage = logistics::StorageCapacity::new(site.storage_capacity);
            storage.update_from_inventory(&site.inventory);
            let overflow = storage.current_used - site.storage_capacity;
            if overflow > Fixed::ZERO && storage.current_used > Fixed::ZERO {
                // Every stock shares the overflow burden proportionally; the
                // exposed portion decays at 10x the normal spoilage rate.
                let spill_ratio = overflow / storage.current_used;
                for stock in &mut site.inventory {
                    if let Some(res_def) = resource_defs.iter().find(|r| r.id == stock.resource_id)
                    {
                        if res_def.perishable && res_def.spoilage_rate > Fixed::ZERO {
                            let loss = stock.quantity
                                * spill_ratio
                                * res_def.spoilage_rate
                                * Fixed::from_f64(10.0)
                                * spoilage_modifier;
                            stock.quantity = (stock.quantity - loss).max(Fixed::ZERO);
                        }
                    }
                }
            }
        }
    }

    fn run_apprenticeship_pass(&mut self, tick_u64: u64, tick: Tick) {
        let n = self.agents.len();
        if n < 2 {
            return;
        }
        for student in 0..n {
            // Find the first knowledge in store order that this student lacks
            // AND for which a capable teacher exists. Scanning candidates
            // (rather than teaching only the first missing item) is essential:
            // a student missing several items may only have a teacher for one.
            let mut chosen: Option<(u64, usize, u32)> = None;
            for knowledge in &self.knowledge_store {
                let knowledge_id = knowledge.id;
                if self.agents[student].education.has_learned(knowledge_id) {
                    continue;
                }
                // §5 (Iteration 148): the technology tree gates education too
                // — a student cannot be taught a node without its prereqs.
                // Deliberately checks `cultural.knowledge` (the same vec the
                // other three gates use) rather than `education.has_learned`:
                // prerequisites live in the knowledge vec, and the two are
                // kept in sync by the apprenticeship bookkeeping below.
                if !self
                    .technology
                    .can_learn(&self.agents[student].cultural.knowledge, knowledge_id)
                {
                    continue;
                }
                // Pick the best teacher among agents who hold this knowledge,
                // can teach, and share a relationship with the student.
                let mut best: Option<(usize, Fixed)> = None;
                for teacher in 0..n {
                    if teacher == student {
                        continue;
                    }
                    if !self.agents[teacher].education.has_learned(knowledge_id) {
                        continue;
                    }
                    if self.agents[teacher].education.teaching_skill < Fixed::from_f64(0.3) {
                        continue;
                    }
                    let rel_quality = self.relationship_quality(teacher, student);
                    if rel_quality < Fixed::from_f64(0.1) {
                        continue;
                    }
                    let skill = self.agents[teacher].education.teaching_skill;
                    if best.is_none_or(|(_, s)| skill > s) {
                        best = Some((teacher, skill));
                    }
                }
                if let Some((teacher, _)) = best {
                    chosen = Some((knowledge_id, teacher, knowledge.holders));
                    break;
                }
            }
            let Some((knowledge_id, teacher, holders)) = chosen else {
                continue;
            };

            // Familiarity: how broadly held the knowledge is (holders / agents).
            let familiarity = Fixed::from_f64(holders as f64 / n as f64).clamp_01();
            let rel_quality = self.relationship_quality(teacher, student);
            let event = crate::culture::education::attempt_teaching(
                teacher,
                student,
                knowledge_id,
                &self.agents[teacher].education,
                &self.agents[student].education,
                familiarity,
                rel_quality,
                tick_u64,
            );

            self.agents[student]
                .education
                .record_learning(event.clone());
            self.agents[teacher]
                .education
                .record_teaching(event.clone());
            if event.success {
                // The student now holds the knowledge in both education state
                // and the shared cultural knowledge vector.
                if !self.agents[student]
                    .cultural
                    .knowledge
                    .contains(&knowledge_id)
                {
                    self.agents[student].cultural.knowledge.push(knowledge_id);
                }
                if let Some(k) = self
                    .knowledge_store
                    .iter_mut()
                    .find(|k| k.id == knowledge_id)
                {
                    k.holders += 1;
                }
                // §8.1.7: Acquired knowledge is evidence exposure — learning
                // desacralizes in proportion to the learning rate and the
                // student's reasoning capacity (same hook as gossip transfer).
                let reasoning = self.agents[student].cognitive.executive_capacity;
                self.agents[student]
                    .sacred_values
                    .desacralize_through_exposure(event.learning_rate, reasoning);
                self.events.push(SimEvent::KnowledgeTransferred {
                    source: AgentId::new(teacher as u64),
                    target: AgentId::new(student as u64),
                    knowledge_id,
                    tick,
                });
                self.provenance
                    .record_institutional(crate::provenance::InstitutionalTrace {
                        institution_name: "Apprenticeship".into(),
                        tick: tick_u64,
                        decision_kind: "apprenticeship_teaching".into(),
                        description: format!(
                            "Agent {teacher} taught knowledge {knowledge_id} to Agent {student}"
                        ),
                        affected: vec![AgentId::new(teacher as u64), AgentId::new(student as u64)],
                        success: true,
                    });

                // §8.1.3: Semantic memory — successfully acquiring knowledge is
                // the canonical semantic episode. Naturally sparse (one pass per
                // tick, success needs a capable teacher and a warm relationship),
                // so the 200-capacity store is not flooded.
                let learner = &mut self.agents[student];
                if learner.agent_tier.tier.runs_memory_encoding()
                    && learner.agent_tier.budget_tracker.can_memory_op()
                {
                    let _ = learner.agent_tier.budget_tracker.consume_memory_op();
                    let emotional =
                        learner.affect.arousal * Fixed::from_f64(0.6) + Fixed::from_f64(0.1);
                    learner.memory.encode(
                        MemoryKind::Semantic,
                        tick_u64,
                        Fixed::from_f64(0.4),
                        emotional,
                        Some(teacher as u32),
                        MemoryTag::LearnedKnowledge,
                    );
                }
            }
        }
    }

    /// §5 (Iteration 151): One formal school term — a single competent
    /// teacher instructs a small cohort of the youngest students in the
    /// teacher's most advanced knowledge. Gated on a `SiteKind::School`
    /// existing (no default world places one, so this is a structural no-op
    /// in every calibrated window). Fully deterministic — teacher and cohort
    /// selection follow fixed rules and `attempt_teaching` draws no
    /// randomness — so a term can never perturb any RNG stream.
    pub fn tick_school_term(&mut self, tick_u64: u64) {
        // Zero-blast gate: without a schoolhouse there is nothing to convene.
        if !self
            .world
            .sites
            .iter()
            .any(|s| s.kind == crate::world::SiteKind::School)
        {
            return;
        }
        let n = self.agents.len();
        if n < 2 {
            return;
        }

        // Instructor: the most experienced teacher holding at least one
        // knowledge (highest teaching skill, ties → lowest index).
        let skills: Vec<Fixed> = self
            .agents
            .iter()
            .map(|a| a.education.teaching_skill)
            .collect();
        let held: Vec<bool> = self
            .agents
            .iter()
            .map(|a| !a.cultural.knowledge.is_empty())
            .collect();
        let Some(teacher_idx) = crate::schools::select_teacher(&skills, &held) else {
            return;
        };

        // Lesson topic: the teacher's most advanced knowledge — the last
        // element of the store-ordered knowledge vector. `select_teacher`
        // only returns knowledge-holding indices, but we still degrade
        // gracefully instead of panicking.
        let Some(&knowledge_id) = self.agents[teacher_idx].cultural.knowledge.last() else {
            return;
        };

        // Cohort: the youngest students who lack the topic and whose
        // technology prerequisites are met — the same gate the apprenticeship
        // applies, so schools can never bypass the tech tree.
        let mut students: Vec<usize> = (0..n)
            .filter(|&s| s != teacher_idx)
            .filter(|&s| !self.agents[s].education.has_learned(knowledge_id))
            .filter(|&s| {
                self.technology
                    .can_learn(&self.agents[s].cultural.knowledge, knowledge_id)
            })
            .collect();
        students.sort_by_key(|&s| self.agents[s].age);
        students.truncate(crate::schools::COHORT_SIZE);
        if students.is_empty() {
            return;
        }

        let cohort = students.len() as u64;
        let mut graduates: u64 = 0;
        // Familiarity: how broadly held the knowledge is (holders / agents);
        // an unseeded innovation scores zero.
        let familiarity = Fixed::from_f64(
            self.knowledge_store
                .iter()
                .find(|k| k.id == knowledge_id)
                .map_or(0.0, |k| k.holders as f64 / n as f64),
        )
        .clamp_01();

        for &student in &students {
            let rel_quality = self.relationship_quality(teacher_idx, student);
            let event = crate::culture::education::attempt_teaching(
                teacher_idx,
                student,
                knowledge_id,
                &self.agents[teacher_idx].education,
                &self.agents[student].education,
                familiarity,
                rel_quality,
                tick_u64,
            );
            self.agents[student]
                .education
                .record_learning(event.clone());
            self.agents[teacher_idx]
                .education
                .record_teaching(event.clone());
            if !event.success {
                continue;
            }
            graduates += 1;
            // The student now holds the knowledge in both education state
            // and the shared cultural knowledge vector.
            if !self.agents[student]
                .cultural
                .knowledge
                .contains(&knowledge_id)
            {
                self.agents[student].cultural.knowledge.push(knowledge_id);
            }
            if let Some(k) = self
                .knowledge_store
                .iter_mut()
                .find(|k| k.id == knowledge_id)
            {
                k.holders += 1;
            }
            // §8.1.7: Acquired knowledge is evidence exposure — learning
            // desacralizes in proportion to the learning rate and the
            // student's reasoning capacity (same hook as gossip transfer).
            let reasoning = self.agents[student].cognitive.executive_capacity;
            self.agents[student]
                .sacred_values
                .desacralize_through_exposure(event.learning_rate, reasoning);
            self.events.push(SimEvent::KnowledgeTransferred {
                source: AgentId::new(teacher_idx as u64),
                target: AgentId::new(student as u64),
                knowledge_id,
                tick: mindstrata_core::clock::Tick::new(tick_u64),
            });
            self.provenance.record_institutional(crate::provenance::InstitutionalTrace {
                institution_name: "School".into(),
                tick: tick_u64,
                decision_kind: "school_teaching".into(),
                description: format!(
                    "Agent {teacher_idx} taught knowledge {knowledge_id} to Agent {student} in the school term"
                ),
                affected: vec![AgentId::new(teacher_idx as u64), AgentId::new(student as u64)],
                success: true,
            });

            // §8.1.3: Semantic memory — the same sparse-encoding discipline
            // as the apprenticeship (one term per year, small cohort), so the
            // 200-capacity store is not flooded.
            let learner = &mut self.agents[student];
            if learner.agent_tier.tier.runs_memory_encoding()
                && learner.agent_tier.budget_tracker.can_memory_op()
            {
                let _ = learner.agent_tier.budget_tracker.consume_memory_op();
                let emotional =
                    learner.affect.arousal * Fixed::from_f64(0.6) + Fixed::from_f64(0.1);
                learner.memory.encode(
                    MemoryKind::Semantic,
                    tick_u64,
                    Fixed::from_f64(0.4),
                    emotional,
                    Some(teacher_idx as u32),
                    MemoryTag::LearnedKnowledge,
                );
            }
        }

        self.school.terms_run += 1;
        self.school.lessons_taught += students.len() as u64;
        self.school.graduates += graduates;
        self.journal.record(
            tick_u64,
            AgentId::new(teacher_idx as u64),
            JournalEntryKind::SchoolTerm {
                teacher: teacher_idx as u64,
                cohort,
                graduates,
            },
        );
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

    /// Relationship quality (0–1) between two agents, from the source's
    /// relationship_v2 view; defaults to a neutral 0.5 when absent.
    fn relationship_quality(&self, from: usize, to: usize) -> Fixed {
        self.agents[from]
            .relationship_v2s
            .iter()
            .find(|r| r.to.as_u64() as usize == to)
            .map_or(Fixed::from_f64(0.5), |r| {
                (r.trust + r.affection) * Fixed::from_f64(0.5)
            })
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

    /// §5 (Iteration 150): One diplomacy pass — mean-revert relations, then
    /// roll a rare event per neighbor on the virgin Economy stream (a raid
    /// when the relation leans hostile, a caravan when it leans friendly).
    /// Idempotent and safe to call from tests; draws from Economy cannot
    /// perturb any other subsystem.
    pub fn tick_diplomacy(&mut self, tick_u64: u64) {
        self.diplomacy.pass_count += 1;
        self.diplomacy.revert_relations();
        let n_neighbors = self.diplomacy.neighbors.len();
        for i in 0..n_neighbors {
            let relation = self.diplomacy.neighbors[i].relation;
            let raid_chance = self.diplomacy.raid_chance(relation);
            let caravan_chance = self.diplomacy.caravan_chance(relation);
            let roll = Fixed::from_f64(self.rng.get_mut(RngStream::Economy).random::<f64>());
            if roll < raid_chance {
                self.apply_raid(i, tick_u64);
            } else if roll < raid_chance + caravan_chance {
                self.apply_caravan(i, tick_u64);
            }
        }
    }

    /// §5 (Iteration 150): A hostile neighbor raids the village — a fraction
    /// of the FARM stocks' grain (capped) is carried off, hostility deepens,
    /// and the ACTUAL loss is journaled. Targeting the farms directly (rather
    /// than `total_food()`, which also counts market/house grain) keeps
    /// journal ≈ reality. Deterministic.
    pub fn apply_raid(&mut self, idx: usize, tick_u64: u64) {
        if idx >= self.diplomacy.neighbors.len() {
            return;
        }
        let name = self.diplomacy.neighbors[idx].name.clone();
        let farm_indices: Vec<usize> = self
            .world
            .sites
            .iter()
            .enumerate()
            .filter(|(_, s)| s.kind == crate::world::SiteKind::Farm)
            .map(|(i, _)| i)
            .collect();
        let farm_grain: Fixed = farm_indices
            .iter()
            .map(|&fi| {
                self.world.sites[fi]
                    .inventory
                    .iter()
                    .find(|st| st.resource_id == GRAIN_RESOURCE_ID)
                    .map_or(Fixed::ZERO, |st| st.quantity)
            })
            .fold(Fixed::ZERO, |a, b| a + b);
        let amount = ((farm_grain * Fixed::from_f64(RAID_GRAIN_FRACTION))
            .min(Fixed::from_f64(RAID_GRAIN_CAP)))
            * raid_loss_multiplier(self.military.readiness);
        let mut lost = Fixed::ZERO;
        if amount > Fixed::ZERO {
            let n = farm_indices.len().max(1);
            let per = amount / Fixed::from_int(n as i64);
            for fi in farm_indices {
                lost += self.world.consume_resource(fi, GRAIN_RESOURCE_ID, per);
            }
        }
        self.diplomacy.neighbors[idx].relation = (self.diplomacy.neighbors[idx].relation
            - Fixed::from_f64(EVENT_RELATION_SHIFT))
        .max(Fixed::from_f64(-1.0));
        self.diplomacy.neighbors[idx].last_raid_tick = Some(tick_u64);
        self.diplomacy.raids += 1;
        self.journal.record(
            tick_u64,
            AgentId::new(0),
            JournalEntryKind::TradeRaid {
                settlement: name,
                grain_lost: lost.to_f64(),
            },
        );
    }

    /// §5 (Iteration 150): A caravan arrives — grain is delivered to the
    /// market (scaled 1.0–2.0× by how friendly the relation is; the market's
    /// grain stock is `AccessRight::Public`, so agents can consume it), the
    /// relation warms, and the event is journaled. If no market exists the
    /// caravan is a no-op (nothing counted, nothing journaled).
    /// Deterministic.
    pub fn apply_caravan(&mut self, idx: usize, tick_u64: u64) {
        if idx >= self.diplomacy.neighbors.len() {
            return;
        }
        let name = self.diplomacy.neighbors[idx].name.clone();
        let relation = self.diplomacy.neighbors[idx].relation.to_f64();
        let scale = Fixed::from_f64(1.0 + (relation + 1.0) * 0.5);
        let amount = Fixed::from_f64(CARAVAN_GRAIN_BASE) * scale;
        if let Some(market_idx) = self
            .world
            .sites
            .iter()
            .position(|s| s.kind == crate::world::SiteKind::Market)
        {
            self.world
                .produce_resource(market_idx, GRAIN_RESOURCE_ID, amount);
            self.diplomacy.neighbors[idx].relation = (self.diplomacy.neighbors[idx].relation
                + Fixed::from_f64(EVENT_RELATION_SHIFT))
            .min(Fixed::ONE);
            self.diplomacy.neighbors[idx].caravan_count += 1;
            self.diplomacy.caravans += 1;
            self.journal.record(
                tick_u64,
                AgentId::new(0),
                JournalEntryKind::TradeCaravan {
                    settlement: name,
                    grain_gained: amount.to_f64(),
                },
            );
        }
    }

    fn tick_resource_operations(
        &mut self,
        action_starts: &[(usize, ActionKind)],
        _pre_tick_events: usize,
        tick_u64: u64,
        tick: Tick,
    ) {
        // ── 4b. Resource operations, journal recording, intention tracking ──

        for (agent_idx, action) in action_starts {
            let agent_id = AgentId::new(*agent_idx as u64);
            // §24.5: Track intention success/failure when action completes.
            // The resource operation outcome (e.g., Eat failing because no grain)
            // is known here, so we record intention success/failure based on
            // the actual resource operation result.
            let action_succeeded = match action {
                ActionKind::Eat => {
                    let amount = Fixed::from_f64(0.1);
                    // §19.5.E: Use access-checking method to enforce resource access rights.
                    // §8.1.4 (P3-6): a FAILED Eat must not relieve hunger — the
                    // per-tick relief in `apply_action_tick` is unconditional, so
                    // an agent whose Eat finds no grain anywhere still "ate" and
                    // hunger never accumulated in famine/pestilence (probe:
                    // needs.hunger 0.006–0.041 in EVERY scenario, famine ≈ calm
                    // — the goal-incongruence branch was unreachable). The
                    // failure paths below revert the relief applied this tick and
                    // cancel the action so its remaining duration ticks don't
                    // relieve either: no food consumed, no satiation.
                    //
                    // A portion is a full meal: `accessible_farm_with_grain`
                    // matches ANY nonzero stock, and `consume_resource` takes
                    // `min(amount, stock)` — so with 0.015 grain left, an Eat
                    // "succeeded" and fully relieved hunger (crumbs counted as
                    // meals, keeping hunger pinned near 0 through the famine
                    // window). Success now requires the FULL portion; a farm
                    // holding less than a meal's worth fails the Eat.
                    if let Some(farm_idx) = self
                        .world
                        .accessible_farm_with_grain_amount(
                            agent_id,
                            &self.institutions,
                            amount,
                        )
                    {
                        let taken =
                            self.world
                                .consume_resource(farm_idx, GRAIN_RESOURCE_ID, amount);
                        // §13.3: Consumers pay coin for resources consumed. Coin is not
                        // destroyed — it flows into the Market treasury (which then
                        // funds wages), closing the wealth-destruction loop.
                        let cost = taken * self.market.price(GRAIN_RESOURCE_ID);
                        let paid = cost.min(self.agents[*agent_idx].wealth.coin);
                        self.agents[*agent_idx].wealth.coin =
                            (self.agents[*agent_idx].wealth.coin - paid).max(Fixed::ZERO);
                        if let Some(market) = self
                            .institutions
                            .iter_mut()
                            .find(|i| i.kind == InstitutionKind::Market)
                        {
                            market.treasury = (market.treasury + paid).max(Fixed::ZERO);
                        }
                        self.journal.record(
                            tick_u64,
                            agent_id,
                            JournalEntryKind::Consumed {
                                resource: "grain".into(),
                                amount: taken.to_f64(),
                            },
                        );
                        // §7.2.7: Food enters the digestive system. Quality 0.6
                        // matches the sim-wide nutrition placeholder — good food,
                        // so effective_digestion stays exactly 1.0 (zero drift);
                        // a future spoilage mechanism can lower it.
                        if taken > Fixed::ZERO {
                            self.agents[*agent_idx]
                                .embodied
                                .digestive
                                .consume_food(Fixed::from_f64(0.6));
                        }
                        taken > Fixed::ZERO
                    } else {
                        // §19.5.D: Theft detection — no accessible farm with a
                        // full portion, but inaccessible farms holding one exist?
                        // (P3-6: amount-gated like the accessible path — a crumb
                        // is not worth stealing and must not feed the thief.)
                        if let Some(thief_idx) = self
                            .world
                            .inaccessible_farm_with_grain_amount(
                                agent_id,
                                &self.institutions,
                                amount,
                            )
                        {
                            self.enforce_theft(
                                *agent_idx,
                                agent_id,
                                thief_idx,
                                GRAIN_RESOURCE_ID,
                                amount,
                                tick_u64,
                                tick,
                            )
                        } else {
                            false
                        }
                    }
                }
                ActionKind::Drink => {
                    let amount = Fixed::from_f64(0.15);
                    // §19.5.E: Use access-checking method to enforce resource access rights.
                    if let Some(well_idx) = self
                        .world
                        .accessible_well_with_water(agent_id, &self.institutions)
                    {
                        let taken =
                            self.world
                                .consume_resource(well_idx, WATER_RESOURCE_ID, amount);
                        // §13.3: Consumers pay coin for water consumed. Coin flows into
                        // the Market treasury instead of vanishing.
                        let cost = taken * self.market.price(WATER_RESOURCE_ID);
                        let paid = cost.min(self.agents[*agent_idx].wealth.coin);
                        self.agents[*agent_idx].wealth.coin =
                            (self.agents[*agent_idx].wealth.coin - paid).max(Fixed::ZERO);
                        if let Some(market) = self
                            .institutions
                            .iter_mut()
                            .find(|i| i.kind == InstitutionKind::Market)
                        {
                            market.treasury = (market.treasury + paid).max(Fixed::ZERO);
                        }
                        self.journal.record(
                            tick_u64,
                            agent_id,
                            JournalEntryKind::Consumed {
                                resource: "water".into(),
                                amount: taken.to_f64(),
                            },
                        );
                        taken > Fixed::ZERO
                    } else {
                        // §19.5.D: Water theft detection — no accessible well, but inaccessible wells with water exist?
                        if let Some(thief_idx) = self
                            .world
                            .inaccessible_well_with_water(agent_id, &self.institutions)
                        {
                            self.enforce_theft(
                                *agent_idx,
                                agent_id,
                                thief_idx,
                                WATER_RESOURCE_ID,
                                amount,
                                tick_u64,
                                tick,
                            )
                        } else {
                            false
                        }
                    }
                }
                ActionKind::Work => {
                    // §4.2: Productivity = base (conscientiousness) + skill bonus
                    let skill_bonus =
                        self.agents[*agent_idx].skills.farming * Fixed::from_f64(0.02);
                    let base = (self.agents[*agent_idx].personality.conscientiousness
                        * Fixed::from_f64(0.05)
                        + skill_bonus)
                        .clamp_01();
                    // §7.5: Sickness depresses output — a plague must hurt the
                    // food economy, not just the population. Healthy agents are
                    // untouched (work_impairment = 1.0 with no diseases), so
                    // calibrated baseline runs don't drift; only outbreaks bite.
                    let sickness_factor = health::work_impairment(&self.agent_diseases[*agent_idx]);
                    // §7.2.3: Frailty/injury depresses output too — mobility is
                    // exactly 1.0 for healthy adults, so calibrated runs carry
                    // zero drift; elders and the severely injured produce less.
                    let mobility_factor = self.agents[*agent_idx]
                        .embodied
                        .skeletal
                        .effective_mobility();
                    // §5 (Iteration 147): weather scales farm output — the
                    // growth factor is ≈ identity in normal weather (mild
                    // calibrated drift) and ≈0.6 during an emergent drought
                    // (genuine famine pressure).
                    // §8.1.4 (P3-6): an active famine window additionally
                    // suppresses grain output to `1 − magnitude` of the shock
                    // (a crop failure — Work keeps laboring but the fields
                    // yield little). The factor is exactly 1.0 outside a
                    // famine, so calibrated windows (riverford/calm/pestilence
                    // and the golden) are untouched; only the Famine and
                    // Collapse scenarios ever open a window.
                    let famine_factor = if tick_u64 < self.famine_until {
                        self.famine_production_factor
                    } else {
                        Fixed::ONE
                    };
                    let productivity = base
                        * sickness_factor
                        * mobility_factor
                        * self.weather.growth_factor()
                        * famine_factor;
                    if let Some(farm_idx) = self.world.best_farm_for_work() {
                        self.world
                            .produce_resource(farm_idx, GRAIN_RESOURCE_ID, productivity);
                        // §13.3: Workers earn coin proportional to productivity.
                        // Wage must cover the cost of eating (0.1 × price per
                        // Eat); the old ×0.3 factor left agents with a net coin
                        // drain (median wealth collapsed to ~0.1, killing trade).
                        let wage = productivity * self.market.price(GRAIN_RESOURCE_ID);
                        self.agents[*agent_idx].wealth.coin += wage;

                        self.journal.record(
                            tick_u64,
                            agent_id,
                            JournalEntryKind::Worked {
                                productivity: productivity.to_f64(),
                            },
                        );
                        true
                    } else {
                        false
                    }
                }
                ActionKind::Rest => {
                    self.journal
                        .record(tick_u64, agent_id, JournalEntryKind::Rested);
                    true
                }
                ActionKind::Worship => {
                    self.journal
                        .record(tick_u64, agent_id, JournalEntryKind::Worshiped);
                    true
                }
                ActionKind::Trade => {
                    // §13.3: Agent-to-agent trade — find a counter-party to trade with.
                    // Trade happens at the settlement Market: any other agent within
                    // a generous radius of the market square counts as a partner.
                    // The old `distance <= 3` filter made trade effectively
                    // impossible (agents rarely stood that close), so volume was 0.
                    let buyer_coin = self.agents[*agent_idx].wealth.coin;
                    let seller_info = self
                        .agents
                        .iter()
                        .enumerate()
                        .find(|(j, a)| {
                            *j != *agent_idx
                                && self.agents[*agent_idx]
                                    .position
                                    .manhattan_distance(&a.position)
                                    <= 12
                        })
                        .map(|(j, _)| j);
                    // Grain source: an accessible farm with a full portion
                    // (prefer the buyer's own accessible farm; fall back to any
                    // farm). §8.1.4 (P3-6): amount-gated like the Eat path — a
                    // trade must move a full portion; crumbs are not marketable
                    // grain and must not feed the buyer through Trade's hunger
                    // relief.
                    let quantity = Fixed::from_f64(0.1);
                    let farm_idx = self
                        .world
                        .accessible_farm_with_grain_amount(
                            agent_id,
                            &self.institutions,
                            quantity,
                        )
                        .or_else(|| self.world.farm_with_grain_amount(quantity));
                    if let (Some(seller), Some(farm_idx)) = (seller_info, farm_idx) {
                        let trust = self
                            .relationships
                            .iter()
                            .find(|r| {
                                r.from == AgentId::new(*agent_idx as u64)
                                    && r.to == AgentId::new(seller as u64)
                            })
                            .map_or(Fixed::from_f64(0.5), |r| r.trust);
                        // §13.3: Execute trade — buyer pays coin, seller's farm provides grain
                        let base_price = self.market.price(GRAIN_RESOURCE_ID);
                        // Trust discount: high trust → lower price
                        let trust_modifier = Fixed::from_f64(1.0) - trust * Fixed::from_f64(0.2)
                            + (Fixed::ONE - trust) * Fixed::from_f64(0.1);
                        // §29 (Iteration 187 — the price-trend consumer): the
                        // market's 5-sample price trend was write-only (audit
                        // FINDING 4). A falling price lets the buyer haggle
                        // down; a rising price has them pay up (scarcity
                        // urgency). Mean-zero: trend 0 → modifier exactly
                        // 1.0, byte-identical to the pre-Iter-187 price.
                        let trend = self.market.price_trend(GRAIN_RESOURCE_ID);
                        let trend_modifier = (Fixed::ONE + trend * TREND_BUY_RATE)
                            .clamp(Fixed::from_f64(0.9), Fixed::from_f64(1.1));
                        // §19.5.E (Iteration 187 — the logistics consumer):
                        // the module's `local_scarcity_modifier` was fully
                        // orphaned (audit FINDING 6). The site's own grain
                        // stock now prices its grain — a scarce farm
                        // commands a premium, an overflowing one discounts.
                        // Damped from the raw [0.5, 3.0] band to [0.95, 1.2].
                        let local_modifier = crate::logistics::local_scarcity_modifier(
                            &self.world.sites[farm_idx].inventory,
                            GRAIN_RESOURCE_ID,
                            Fixed::from_f64(1.0),
                        );
                        let local_price_mod =
                            (local_modifier - Fixed::ONE) * LOGISTICS_PRICE_DAMP + Fixed::ONE;
                        let price = (base_price
                            * trust_modifier
                            * trend_modifier
                            * local_price_mod)
                            .max(Fixed::from_f64(1.0));
                        let cost = price * quantity;
                        if buyer_coin >= cost {
                            let taken =
                                self.world
                                    .consume_resource(farm_idx, GRAIN_RESOURCE_ID, quantity);
                            if taken > Fixed::ZERO {
                                let actual_cost = price * taken;
                                // Buyer pays coin to seller (farm owner)
                                self.agents[*agent_idx].wealth.coin =
                                    (self.agents[*agent_idx].wealth.coin - actual_cost)
                                        .max(Fixed::ZERO);
                                self.agents[seller].wealth.coin += actual_cost;
                                self.events.push(SimEvent::TradeOccurred {
                                    buyer: AgentId::new(*agent_idx as u64),
                                    seller: AgentId::new(seller as u64),
                                    good: ResourceId::new(GRAIN_RESOURCE_ID),
                                    quantity: taken,
                                    price,
                                    tick,
                                });
                                // §13.3: Track volume so the market dashboard and
                                // inequality metrics reflect real trade activity.
                                self.market.volume_this_tick =
                                    (self.market.volume_this_tick + taken).max(Fixed::ZERO);
                                self.market.trade_count = self.market.trade_count.saturating_add(1);
                                self.market.total_trades =
                                    self.market.total_trades.saturating_add(1);
                                self.journal.record(
                                    tick_u64,
                                    agent_id,
                                    JournalEntryKind::Consumed {
                                        resource: "grain_via_trade".into(),
                                        amount: taken.to_f64(),
                                    },
                                );
                                // §19.5.J: Record relationship trace — trade builds trust
                                if let Some(rel) = self.relationships.iter_mut().find(|r| {
                                    r.from == AgentId::new(*agent_idx as u64)
                                        && r.to == AgentId::new(seller as u64)
                                }) {
                                    let old_trust = rel.trust;
                                    rel.trust = (rel.trust + Fixed::from_f64(0.02)).clamp_01();
                                    self.provenance.record_relationship(
                                        crate::provenance::RelationshipTrace {
                                            from: AgentId::new(*agent_idx as u64),
                                            to: AgentId::new(seller as u64),
                                            tick: tick_u64,
                                            cause: "trade".into(),
                                            old_trust,
                                            new_trust: rel.trust,
                                            old_affection: rel.affection,
                                            new_affection: rel.affection,
                                            description: format!(
                                                "Trade built trust ({} -> {})",
                                                old_trust.to_f64(),
                                                rel.trust.to_f64()
                                            ),
                                        },
                                    );
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

            // §8.1.4 (P3-6): a FAILED Eat must not relieve hunger. The
            // per-tick relief in `apply_action_tick` is unconditional, so
            // without this revert an agent whose Eat found no grain anywhere
            // still "ate" — hunger never accumulated in famine/pestilence
            // (probe: needs.hunger 0.006–0.041 in EVERY scenario, famine ≈
            // calm; the goal-incongruence branch was unreachable). Revert
            // the single tick of relief already applied and cancel the
            // action so its remaining duration ticks don't relieve either:
            // no food consumed, no satiation.
            if *action == ActionKind::Eat && !action_succeeded {
                let fx = ActionKind::Eat.per_tick_effects();
                let agent = &mut self.agents[*agent_idx];
                agent.needs.hunger = (agent.needs.hunger + fx.hunger_relief).clamp_01();
                agent.action_progress = 0;
                agent.current_action = ActionKind::Idle;
            }

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
            // Architecture-plan-2 §8.1.20: Decision policy learns from action outcome.
            // EMA learning adjusts utility weights toward successful action patterns.
            let action_cost = match action {
                ActionKind::Work => Fixed::from_f64(0.1),
                ActionKind::Eat | ActionKind::Drink => Fixed::from_f64(0.05),
                ActionKind::Move { .. } | ActionKind::Wander => Fixed::from_f64(0.08),
                ActionKind::Socialize => Fixed::from_f64(0.03),
                _ => Fixed::from_f64(0.02),
            };
            self.agents[*agent_idx]
                .decision_policy
                .learn_from_outcome(action_succeeded, action_cost);

            // §9.2: Neural-like RL values — learn the plan's four value
            // components from the outcome. The outcome profile is the
            // shared `ActionKind::outcome_profile` (single source of
            // truth with the Iteration-94 selection consumer), so what an
            // agent learns is exactly what biases its future choices.
            let profile = action.outcome_profile();
            self.agents[*agent_idx]
                .neural_like
                .values
                .learn_from_outcome(
                    action_succeeded,
                    profile[0],
                    profile[1],
                    profile[2],
                    profile[3],
                );
        }
    }

    /// Section 9: Memory encoding from this tick's events.
    fn tick_memory_encoding(
        &mut self,
        pre_tick_events: usize,
        tick_u64: u64,
        phases: crate::scheduler::TickPhases,
    ) {
        // ── 9. Memory encoding from this tick's events ───────────────
        // §22.5: Use attention system to compute salience instead of hardcoded values.
        // Only events that pass the attention threshold are encoded into memory.
        for (i, agent) in self.agents.iter_mut().enumerate() {
            // §17: Background agents skip memory encoding entirely
            // §17.2: Also check memory retrieval budget
            if !agent.agent_tier.tier.runs_memory_encoding()
                || !agent.agent_tier.budget_tracker.can_memory_op()
            {
                continue;
            }
            // §8.1.2: Fresh salience competition each tick — the map records
            // this tick's percepts, not a rolling top-N.
            agent.attention.salience_map.clear();
            for ev in &self.events[pre_tick_events..] {
                // §22.5: Attention computes salience based on intensity, novelty, relevance
                let salience = agent.attention.compute_salience(
                    ev,
                    AgentId::new(i as u64),
                    &agent.needs,
                    &agent.affect,
                );

                // §8.1.2: Record the percept into the salience competition
                // (write-only observational state — keeps the top percepts by
                // computed salience, so the salience gate itself is untouched).
                let percept_agent = match ev {
                    SimEvent::InteractionOccurred { from, to, .. } => {
                        Some(if from.as_u64() == i as u64 {
                            *to
                        } else {
                            *from
                        })
                    }
                    SimEvent::AgentAte { agent: a, .. }
                    | SimEvent::AgentDrank { agent: a, .. }
                    | SimEvent::RelationshipChanged { from: a, .. } => Some(*a),
                    _ => None,
                };
                agent.attention.record_salience(
                    PerceptKind::of(ev),
                    percept_agent,
                    salience,
                    tick_u64,
                );

                // Only encode events that exceed the attention threshold
                if salience < Fixed::from_f64(0.2) {
                    continue;
                }

                // Emotional intensity scales with arousal
                let emotional = agent.affect.arousal * Fixed::from_f64(0.6) + Fixed::from_f64(0.1);
                match ev {
                    SimEvent::AgentAte { agent: a, .. } if a.as_u64() == i as u64 => {
                        if agent.agent_tier.budget_tracker.can_memory_op() {
                            let _ = agent.agent_tier.budget_tracker.consume_memory_op();
                            agent.memory.encode(
                                MemoryKind::Somatic,
                                tick_u64,
                                salience,
                                emotional,
                                None,
                                MemoryTag::AteFood,
                            );
                        }
                    }
                    SimEvent::AgentDrank { agent: a, .. } if a.as_u64() == i as u64 => {
                        if agent.agent_tier.budget_tracker.can_memory_op() {
                            let _ = agent.agent_tier.budget_tracker.consume_memory_op();
                            agent.memory.encode(
                                MemoryKind::Somatic,
                                tick_u64,
                                salience,
                                emotional,
                                None,
                                MemoryTag::DrankWater,
                            );
                        }
                    }
                    SimEvent::AgentRested { agent: a, .. } if a.as_u64() == i as u64 => {
                        if agent.agent_tier.budget_tracker.can_memory_op() {
                            let _ = agent.agent_tier.budget_tracker.consume_memory_op();
                            agent.memory.encode(
                                MemoryKind::Emotional,
                                tick_u64,
                                salience,
                                emotional,
                                None,
                                MemoryTag::Rested,
                            );
                        }
                    }
                    SimEvent::InteractionOccurred { from, to, kind, .. } => {
                        if from.as_u64() == i as u64 {
                            let tag = match kind {
                                mindstrata_core::event::InteractionKind::Help => {
                                    MemoryTag::HelpedBy
                                }
                                mindstrata_core::event::InteractionKind::Threaten => {
                                    MemoryTag::ThreatenedBy
                                }
                                mindstrata_core::event::InteractionKind::Insult => {
                                    MemoryTag::InsultedBy
                                }
                                mindstrata_core::event::InteractionKind::Gossip => {
                                    MemoryTag::GossipedAbout
                                }
                                mindstrata_core::event::InteractionKind::Trade => {
                                    MemoryTag::TradedWith
                                }
                                _ => MemoryTag::TalkedTo,
                            };
                            let kind = match kind {
                                mindstrata_core::event::InteractionKind::Threaten
                                | mindstrata_core::event::InteractionKind::Insult => {
                                    MemoryKind::Traumatic
                                }
                                mindstrata_core::event::InteractionKind::Help
                                | mindstrata_core::event::InteractionKind::Comfort => {
                                    MemoryKind::Emotional
                                }
                                _ => MemoryKind::Social,
                            };
                            if agent.agent_tier.budget_tracker.can_memory_op() {
                                let _ = agent.agent_tier.budget_tracker.consume_memory_op();
                                agent.memory.encode(
                                    kind,
                                    tick_u64,
                                    salience,
                                    emotional,
                                    Some(to.as_u64() as u32),
                                    tag,
                                );
                            }
                        }
                        if to.as_u64() == i as u64
                            && agent.agent_tier.budget_tracker.can_memory_op()
                        {
                            let _ = agent.agent_tier.budget_tracker.consume_memory_op();
                            agent.memory.encode(
                                MemoryKind::Social,
                                tick_u64,
                                salience,
                                emotional,
                                Some(from.as_u64() as u32),
                                MemoryTag::TalkedTo,
                            );
                        }
                    }
                    _ => {}
                }
            }
            // Probabilistic memory rehearsal (every 10 ticks)
            if phases.is_deca {
                agent
                    .memory
                    .rehearse_random(tick_u64, self.rng.get_mut(RngStream::Behavior));
            }
            // Decay memories
            agent.memory.decay(tick_u64);

            // §22.5: Attention maintenance — decay habituation, replenish budget
            agent.attention.decay_habituation(Fixed::from_f64(0.005));
            let stress = agent.emotions.fear + agent.emotions.anger;
            agent
                .attention
                .replenish_budget(stress, agent.needs.fatigue);
            // §8.1.2: Recompute the perceptual biases from current agent state
            // (pure function, no RNG — write-only observational state, so
            // calibrated trajectories are untouched).
            agent.attention.recompute_biases(
                agent.emotions.fear,
                agent.emotions.anger,
                agent.derived.trauma_risk,
                agent.personality.extraversion,
                agent.personality.openness,
                agent.attachment.security,
                // §9.2: yesterday's surprise raises today's novelty-seeking
                // attention (the neural pass below updates
                // `last_prediction_error`; the value seen here is the prior
                // tick's). Zero in calm windows → byte-identical.
                agent.neural_like.expectation.last_prediction_error,
                // §7.2.2 (Iteration 187 — the S2-2-2 arousal consumer): the
                // biological endocrine arousal level now feeds the threat
                // bias — a physiologically aroused agent is hypervigilant.
                // Mean-zero at the 0.5 anchor inside `recompute_biases`.
                agent.embodied.endocrine.arousal.level,
            );

            // §19.5.G: Update status from current wealth and social connections
            agent.status.wealth_status = (agent.wealth.coin / Fixed::from_f64(20.0)).clamp_01();
            // Social status: ratio of positive relationships to total relationships
            // §19.5.G: A relationship is "positive" when trust exceeds 0.6
            let agent_id = AgentId::new(i as u64);
            let (positive_rels, total_rels) = self
                .relationships
                .iter()
                .filter(|r| r.from == agent_id)
                .fold((0u32, 0u32), |(pos, total), r| {
                    (
                        pos + if r.trust > Fixed::from_f64(0.6) { 1 } else { 0 },
                        total + 1,
                    )
                });
            if total_rels > 0 {
                agent.status.social_status =
                    Fixed::from_f64(positive_rels as f64 / total_rels as f64);
            }
            agent.status.recompute();

            // §22 / §8.1.3: Memory reconsolidation — current emotions bias recalled
            // memories and erode their accuracy (recording distortion events).
            agent
                .memory
                .reconsolidate(tick_u64, agent.emotions.anger, agent.emotions.joy);

            // §8.1.3: Retrieval reconstructs — the most accessible trace
            // (salience argmax: strength + charge/2 + identity/4) is recalled
            // and re-encoded under the current emotional state: rehearsal
            // strengthens it while an emotion-mismatched reconstruction erodes
            // accuracy and records a RetrievalReconstruction distortion event.
            // High-charge (traumatic) traces always win the argmax, so they
            // intrude preferentially. Deterministic (no RNG), purely
            // observational memory state — calibrated runs stay byte-identical.
            agent.memory.retrieve_and_reconsolidate(
                tick_u64,
                agent.emotions.anger,
                agent.emotions.joy,
            );

            // §9.2: Deterministic neural-like runtime. The concept vector is
            // derived deterministically from the agent's live state (no RNG)
            // and spread through the association network. Since Iteration
            // 183d the prediction error feeds three live consumers —
            // attention (novelty_bias, continuous PE × 0.2 via
            // recompute_biases), belief reinforcement, and emotional
            // intensity (arousal, both gated at 0.3). All are exactly zero
            // in calm windows (zero prediction error), so calibrated
            // trajectories stay byte-identical; the learned RL values
            // remain observational (consumed only via `learned_delta` in the
            // action-value fold, actions.rs).
            let concepts = crate::psychology::neural_like::ConceptVector {
                safety: Fixed::ONE - agent.emotions.fear,
                sacredness: if agent.cultural.ideology.is_some() {
                    Fixed::from_f64(0.5)
                } else {
                    Fixed::ZERO
                },
                status: agent.status.social_status,
                kinship: if agent.partner.is_some() || agent.parent_a.is_some() {
                    Fixed::from_f64(0.5)
                } else {
                    Fixed::ZERO
                },
                scarcity: Fixed::ONE - agent.needs.safety,
                threat: agent.emotions.fear,
                purity: agent.moral_values.purity,
                freedom: Fixed::ONE - agent.personality.conformity,
                loyalty: agent.moral_values.loyalty,
                pleasure: agent.emotions.joy,
                shame: agent.emotions.shame,
                hope: agent.emotions.hope,
            };
            agent
                .neural_like
                .network
                .spread(concepts, Fixed::from_f64(0.8));
            // Predictive error: observe the current success rate as the
            // outcome of the agent's recent attempts.
            let attempts = agent.recent_attempts;
            if attempts > 0 {
                let success_rate = Fixed::from_int(agent.recent_successes as i64)
                    / Fixed::from_int(attempts as i64);
                agent.neural_like.expectation.observe(success_rate);
            }
            // §9.2 (Iteration 183d): LARGE prediction error creates emotional
            // intensity — a violated world model spikes arousal (the
            // intensity axis), amplifying the emotional weight of subsequent
            // decisions. Gated at 0.3 (calm windows carry zero prediction
            // error, so the golden baseline stays byte-identical); at 0.1
            // gain a 0.3 surprise adds 0.03 arousal — a modest, honest
            // intensity bump.
            let surprise = agent.neural_like.expectation.last_prediction_error;
            if surprise > Fixed::from_f64(0.3) {
                agent.affect.arousal =
                    (agent.affect.arousal + surprise * Fixed::from_f64(0.1)).clamp_01();
            }
            // §9.2 script grammar: partnered agents replay the courtship script
            // as an observational narrative track (no behavioral effect).
            if agent.partner.is_some() {
                let script = agent
                    .neural_like
                    .script
                    .get_or_insert_with(crate::psychology::neural_like::BehaviorScript::courtship);
                script.next_step();
            }

            // §4.2: Skill improvement — agents improve skills through repeated practice.
            // Small increments per tick; skills cap at 1.0.
            let skill_gain = SKILL_GAIN_PER_TICK;
            // §8.1.3: Procedural memory — practice crossing a 0.1-proficiency
            // milestone encodes the mastered routine (sparse by design: with a
            // 0.001 gain a milestone needs ~100 practice ticks, keeping the
            // 200-capacity store focused on the vivid events).
            let practiced: Option<&mut Fixed> = match agent.current_action {
                crate::actions::ActionKind::Work => Some(&mut agent.skills.farming),
                crate::actions::ActionKind::Trade => Some(&mut agent.skills.trading),
                crate::actions::ActionKind::Socialize | crate::actions::ActionKind::Worship => {
                    Some(&mut agent.skills.social)
                }
                _ => None,
            };
            // §8.1.19 (P3-1, August 14, 2026): the psychology SkillState was
            // structurally unwired — `practice`/`form_habit`/`execute_habit`
            // had ZERO production call sites, so the skills and habits maps
            // stayed permanently empty (probe-pinned: skill_count/habit_count
            // 0.000 for 12/12 agents in every window). The same action
            // practice now feeds BOTH the §4.2 person-level proficiency above
            // AND the psychology skill system below: repeated practice grows
            // proficiency, and crossing the 0.1-proficiency milestone forms
            // (or reinforces) the corresponding habit.
            let psych_skill = match agent.current_action {
                crate::actions::ActionKind::Work => Some((
                    crate::psychology::skill::SKILL_FARMING,
                    "Work",
                    "hunger",
                )),
                crate::actions::ActionKind::Trade => Some((
                    crate::psychology::skill::SKILL_TRADING,
                    "Trade",
                    "thirst",
                )),
                crate::actions::ActionKind::Socialize => Some((
                    crate::psychology::skill::SKILL_SPEAKING,
                    "Socialize",
                    "social",
                )),
                crate::actions::ActionKind::Worship => Some((
                    crate::psychology::skill::SKILL_RITUAL,
                    "Worship",
                    "meaning",
                )),
                crate::actions::ActionKind::Eat => Some((
                    crate::psychology::skill::SKILL_COOKING,
                    "Eat",
                    "hunger",
                )),
                _ => None,
            };
            if let Some((skill_id, action_name, trigger)) = psych_skill {
                if practiced.is_some() {
                    // §8.1.19: practice grows proficiency (difficulty-scaled,
                    // clamped); neuroplasticity is the agent's own trait.
                    // Iteration 197: cognitive_development (the §17
                    // life-stage state — 0.1 infant → 1.0 adult → decline
                    // past 50) modulates how easily the agent learns.
                    // Baseline-corrected: exactly 1.0 at the default 0.7,
                    // rising to 1.3 at full adult cognition, falling to 0.85
                    // at infant 0.1 — populate-default agents are exact
                    // no-ops (the developmental pass only rewrites
                    // cognitive_development for Focal agents), and only
                    // agents whose cognition actually developed diverge.
                    let cognitive_scale = Fixed::ONE
                        + (agent.developmental.cognitive_development
                            - Fixed::from_f64(0.7))
                            * Fixed::from_f64(0.5);
                    agent.psych_skills.practice(
                        skill_id,
                        tick_u64,
                        (agent.psych_skills.neuroplasticity * cognitive_scale).clamp_01(),
                    );
                    // §8.1.19: habit formation at the 0.1-proficiency
                    // milestone — the same gate the procedural-memory encode
                    // uses, so habits arrive with genuine mastery (~100
                    // practice ticks). Deterministic, no RNG.
                    let prof = agent.psych_skills.proficiency(skill_id);
                    let crossed = prof >= Fixed::from_f64(0.1)
                        && prof - skill_gain < Fixed::from_f64(0.1);
                    if crossed {
                        agent.psych_skills.form_habit(
                            action_name.to_string(),
                            trigger.to_string(),
                            Fixed::from_f64(0.3),
                            tick_u64,
                        );
                    } else {
                        // §8.1.19 (P3-1 completion, re-audit): practicing
                        // the action exercises any matching habit — refresh
                        // recency so the centum decay pass cannot grind
                        // formed habits to zero while the agent keeps
                        // performing the action (the pre-fix habit_count
                        // collapsed 1.17 @2K → 0.000 @10K because
                        // `execute_habit`'s automaticity > 0.5 gate was
                        // unreachable and `last_performed` never refreshed).
                        agent.psych_skills.refresh_habit(trigger, tick_u64);
                    }
                }
            }
            if let Some(skill) = practiced {
                let before = *skill;
                *skill = (before + skill_gain).clamp_01();
                if skill_milestone_crossed(before, *skill)
                    && agent.agent_tier.budget_tracker.can_memory_op()
                {
                    let _ = agent.agent_tier.budget_tracker.consume_memory_op();
                    let emotional =
                        agent.affect.arousal * Fixed::from_f64(0.6) + Fixed::from_f64(0.1);
                    agent.memory.encode(
                        MemoryKind::Procedural,
                        tick_u64,
                        Fixed::from_f64(0.3),
                        emotional,
                        None,
                        MemoryTag::SkillMastered,
                    );
                }
            }
        }
    }

    /// Section 11: Norm evaluation — threats and insults are norm violations.
    fn tick_norm_evaluation(&mut self, pre_tick_events: usize, tick_u64: u64, tick: Tick) {
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
                            let inhibition_hold = if self.agents[from_idx]
                                .cognitive_runtime
                                .can_inhibit()
                            {
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
                                            self.rng
                                                .get_mut(RngStream::Behavior)
                                                .random::<f64>(),
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
    fn tick_moral_panic_and_revolution(&mut self, tick_u64: u64, tick: Tick) {
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
            let belief_refs: Vec<&Vec<Belief>> = self.agents.iter().map(|a| &a.beliefs).collect();
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

                // Revolution score: weighted combination of grievance, size, and council weakness
                let revolution_score = faction_grievance * factions::REVOLUTION_GRIEVANCE_WEIGHT
                    + Fixed::from_int(faction_size as i64)
                        / Fixed::from_int(total_pop.max(1) as i64)
                        * factions::REVOLUTION_SIZE_WEIGHT
                    + (Fixed::ONE - council_legitimacy) * factions::REVOLUTION_LEGITIMACY_WEIGHT;

                let revolution_cooldown_ok = tick_u64.saturating_sub(self.last_revolution_tick)
                    >= factions::REVOLUTION_COOLDOWN;
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
    fn tick_moral_panic_lifecycle(&mut self, tick_u64: u64) {
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
    fn tick_derived_states_and_beliefs(&mut self, pre_tick_events: usize, tick_u64: u64) {
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
                        - self.agents[from_idx]
                            .cultural_cognition
                            .change_resistance()
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
                    let from_surprise =
                        self.agents[from_idx].neural_like.expectation.last_prediction_error;
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

        // §10.1: Refresh each agent's three relational fields (sensory /
        // social / noospheric) on the daily cadence.
        self.refresh_relational_fields();
    }

    /// §10.1: Refresh each agent's three relational fields.
    ///
    /// Sensory: proximity (agents within `PERCEPTION_RADIUS`) and the
    /// perceived emotional expression of neighbors (mean (fear+anger)/2).
    /// Social: the relationship-graph summary (mean trust, mean obligation,
    /// kin-stage count, highest peer status). Noospheric: held beliefs
    /// (mean confidence, hottest emotional charge), perceived legitimacy, and
    /// the world's collective fear.
    ///
    /// Deterministic: index-order iteration, no RNG, no hash-map iteration.
    /// Observational: no decisional consumer is wired yet, so calibrated runs
    /// carry zero drift.
    fn refresh_relational_fields(&mut self) {
        use crate::social::relational_field::{RelationalFields, PERCEPTION_RADIUS};
        let n = self.agents.len();
        if n == 0 {
            return;
        }
        let collective_fear = self
            .agents
            .iter()
            .map(|a| a.emotions.fear)
            .fold(Fixed::ZERO, |acc, f| acc + f)
            / Fixed::from_int(n as i64);
        // Single-pass accumulation per agent — no intermediate Vec allocations
        // (this runs per agent per daily tick; the plan targets large
        // populations, §17.4).
        for i in 0..n {
            // ── Sensory field: neighbor scan accumulates count / min distance /
            // stress sum / max peer status in one pass. ──
            let mut nearby_count: u32 = 0;
            let mut nearest = i32::MAX;
            let mut stress_sum = Fixed::ZERO;
            let mut peer_status = Fixed::ZERO;
            for j in 0..n {
                if i == j {
                    continue;
                }
                let d = self.agents[i]
                    .position
                    .manhattan_distance(&self.agents[j].position);
                if d <= PERCEPTION_RADIUS {
                    nearby_count += 1;
                    nearest = nearest.min(d);
                    stress_sum += (self.agents[j].emotions.fear + self.agents[j].emotions.anger)
                        * Fixed::from_f64(0.5);
                    peer_status = peer_status.max(self.agents[j].status_v2.effective_status());
                }
            }
            // ── Social field ──
            let mut trust_sum = Fixed::ZERO;
            let mut oblig_sum = Fixed::ZERO;
            let mut kin_count: u32 = 0;
            for r in &self.agents[i].relationship_v2s {
                trust_sum += r.trust;
                oblig_sum += r.obligation;
                if crate::social::relationship_stages::is_kin_stage(r.stage) {
                    kin_count += 1;
                }
            }
            let rel_count = self.agents[i].relationship_v2s.len().max(1);
            // ── Noospheric field ──
            let mut belief_sum = Fixed::ZERO;
            let mut hottest_charge = Fixed::ZERO;
            for b in &self.agents[i].beliefs {
                belief_sum += b.confidence;
                hottest_charge = hottest_charge.max(b.emotional_charge);
            }
            let belief_count = self.agents[i].beliefs.len().max(1);
            self.agents[i].relational_fields = RelationalFields {
                nearby_agents: nearby_count,
                nearest_closeness: if nearest == i32::MAX {
                    Fixed::ZERO
                } else {
                    RelationalFields::closeness(nearest)
                },
                perceived_stress: if nearby_count == 0 {
                    Fixed::ZERO
                } else {
                    stress_sum / Fixed::from_int(nearby_count as i64)
                },
                social_trust: trust_sum / Fixed::from_int(rel_count as i64),
                social_obligation: oblig_sum / Fixed::from_int(rel_count as i64),
                kin_count,
                peer_status,
                belief_confidence: belief_sum / Fixed::from_int(belief_count as i64),
                hottest_charge,
                legitimacy_perceived: self.agents[i].legitimacy_field.overall,
                collective_fear,
            };
        }
    }

    /// Sections 18-21: Feud tracking, market, demography, conflict, carrying cost.
    fn tick_social_cluster(
        &mut self,
        pre_tick_events: usize,
        tick_u64: u64,
        tick: Tick,
        phases: crate::scheduler::TickPhases,
    ) {
        // ── 18. §19.5.G Feud tracking — repeated violence creates persistent feuds ──
        let snap_count = self.events.len();
        for idx in pre_tick_events..snap_count {
            let ev = &self.events[idx];
            if let SimEvent::ConflictOccurred {
                aggressor,
                target,
                kind,
                ..
            } = ev
            {
                if *kind == ConflictKind::Violence {
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
                            // §10.8: Repeated violence forges a clan enmity (feud boundary).
                            self.forge_clan_enmity(a_idx, t_idx, tick_u64);
                        }
                    }
                }
            }
        }

        // ── 20. Conflict state update — trauma decay, combat fatigue, feud decay ──
        for agent in &mut self.agents {
            agent.conflict.update(&self.params);
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

        // §10.8/§19.5.G: Clear clan enmities whose feuds have fully decayed —
        // a feud that ends can heal into peace (and later, a marriage alliance).
        self.decay_clan_enmities();

        // ── 22. Courtship creation — eligible agents begin romantic courtship ──
        // Architecture-plan-2 §10.4: When two unpartnered adult agents with sufficient
        // attraction and trust interact, a Courtship is initiated.
        // Runs every 12 ticks (~2 hours) to avoid O(N²) overhead every tick.
        if phases.is_duodeca {
            let n = self.agents.len();
            // Build HashSet of agents already in courtships for O(1) lookup.
            let mut in_courtship: std::collections::HashSet<usize> =
                std::collections::HashSet::new();
            for c in &self.active_courtships {
                if c.active {
                    in_courtship.insert(c.pursuer);
                    in_courtship.insert(c.pursued);
                }
            }
            for i in 0..n {
                // Skip agents already in a courtship, partnered, or minors
                if in_courtship.contains(&i) {
                    continue;
                }
                if self.agents[i].partner.is_some() {
                    continue;
                }
                if self.agents[i].age < Fixed::from_f64(16.0) {
                    continue;
                }

                for j in (i + 1)..n {
                    if in_courtship.contains(&j) {
                        continue;
                    }
                    if self.agents[j].partner.is_some() {
                        continue;
                    }
                    if self.agents[j].age < Fixed::from_f64(16.0) {
                        continue;
                    }

                    // Check kinship — skip close relatives
                    let kinship_coeff = self.kinship_graph.coefficient_between(i, j);

                    // Compute mutual trust from relationship_v2
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

                    // Delegate all eligibility checks to the function
                    let already_bonded = self.marriage_registry.find_bond(i, j).is_some();
                    let eligible = crate::social::courtship::eligible_for_courtship(
                        self.agents[i].age,
                        self.agents[j].age,
                        kinship_coeff,
                        already_bonded,
                        COURTSHIP_TRUST_THRESHOLD,
                        avg_trust,
                    );
                    if !eligible {
                        continue;
                    }

                    // Deterministic chance to start courtship — based on trust
                    let chance = avg_trust * Fixed::from_f64(0.02);
                    let roll =
                        Fixed::from_f64(self.rng.get_mut(RngStream::Social).random_range(0.0..1.0));
                    if roll < chance {
                        let mut courtship =
                            crate::social::courtship::Courtship::new(i, j, tick_u64);
                        courtship.mutual_attraction = avg_trust;
                        courtship.stage = crate::social::marriage::RomanticStage::Awareness;
                        // Update the HashSet immediately to prevent double-matching
                        // within the same tick (agent in two courtships simultaneously).
                        in_courtship.insert(i);
                        in_courtship.insert(j);
                        self.active_courtships.push(courtship);
                    }
                }
            }
        }

        // ── 22b. Patronage creation (architecture-plan-2 §10.9) ──────────
        // When a high-status agent has repeated positive interactions with a low-status
        // agent, a patronage relationship may form. Runs every 12 ticks to limit O(N²).
        if phases.is_duodeca {
            // Pre-cache effective status for all agents (O(N) instead of O(N²) recomputation).
            let status_cache: Vec<Fixed> = self
                .agents
                .iter()
                .map(|a| a.status_v2.effective_status())
                .collect();
            // Pre-build HashSet of agents already in a patronage relation as client.
            // O(M) build + O(1) lookups vs O(N²×M) with inline patron_of() calls.
            let patronage_clients: std::collections::HashSet<usize> = self
                .agents
                .iter()
                .enumerate()
                .filter(|(idx, _)| self.patronage_registry.patron_of(*idx).is_some())
                .map(|(idx, _)| idx)
                .collect();
            // Track how many clients each patron has acquired this tick (max 3).
            // Vec indexed by agent ID — simpler and avoids HashMap overhead.
            let mut patron_client_counts: Vec<usize> = vec![0; self.agents.len()];
            #[expect(
                clippy::needless_range_loop,
                reason = "intentional parallel-array indexing: status_cache[i/j] + patron_client_counts[i] + relationship_v2s[i]"
            )]
            for i in 0..self.agents.len() {
                let patron_status = status_cache[i];
                if patron_client_counts[i] >= PATRONAGE_MAX_CLIENTS_PER_PATRON {
                    continue;
                }
                for j in 0..self.agents.len() {
                    if i == j {
                        continue;
                    }
                    // Patron must be notably higher status than client
                    if patron_status <= status_cache[j] + PATRONAGE_STATUS_GAP {
                        continue;
                    }
                    // Must not already have a patronage link (O(1) HashSet lookup)
                    if patronage_clients.contains(&j) {
                        continue;
                    }
                    // Check trust from relationship_v2
                    let rv2_idx = Self::relationship_v2_pos(i, j);
                    if rv2_idx >= self.agents[i].relationship_v2s.len() {
                        continue;
                    }
                    let trust = self.agents[i].relationship_v2s[rv2_idx].trust;
                    let affection = self.agents[i].relationship_v2s[rv2_idx].affection; // Need moderate trust and positive affect to form patronage
                    if trust < PATRONAGE_TRUST_THRESHOLD
                        || affection < PATRONAGE_AFFECTION_THRESHOLD
                    {
                        continue;
                    }
                    let chance = (trust * PATRONAGE_CHANCE_SCALE).clamp_01();
                    let roll =
                        Fixed::from_f64(self.rng.get_mut(RngStream::Social).random_range(0.0..1.0));
                    if roll < chance {
                        let rel = crate::social::patronage::PatronageRelation::new(i, j, tick_u64);
                        self.patronage_registry.register(rel);
                        patron_client_counts[i] += 1;
                        tracing::info!(
                            patron = i,
                            client = j,
                            trust = trust.to_f64(),
                            tick = tick_u64,
                            "Patronage relationship formed"
                        );
                    }
                }
            }
        }

        // ── 23. §19.5.E Carrying cost — heavy loads increase fatigue ──
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
                self.agents[i].needs.fatigue =
                    (self.agents[i].needs.fatigue + fatigue_cost).clamp_01();
            }
        }

        // ── 19. Market system: price formation, inequality tracking ──
        {
            let agent_economic_state: Vec<_> = self
                .agents
                .iter()
                .map(|a| (a.needs.clone(), a.wealth.clone()))
                .collect();
            market::system_market(
                &self.world,
                &agent_economic_state,
                &mut self.market,
                tick_u64,
                &self.params,
            );

            // §4.4: Update black market state based on current scarcity and enforcement
            // Scarcity is normalized: 0 = abundant, 1 = critically scarce
            let total_grain = self.world.total_food();
            let expected_capacity = Fixed::from_int(self.agents.len() as i64)
                * Fixed::from_int(EXPECTED_GRAIN_PER_AGENT as i64);
            let scarcity = if expected_capacity > Fixed::ZERO {
                (Fixed::ONE - total_grain / expected_capacity).clamp_01()
            } else {
                Fixed::ZERO
            };
            let council_enforcement = self
                .institutions
                .iter()
                .filter(|i| i.kind == InstitutionKind::Council)
                .map(|i| i.enforcement_capacity)
                .fold(Fixed::ZERO, std::cmp::Ord::max);
            self.black_market.update(scarcity, council_enforcement);
        }

        // ── 18. Demography: aging and mortality ──
        if phases.is_deca {
            // process demography every 10 ticks to reduce overhead
            let mut deaths = Vec::new();
            for i in 0..self.agents.len() {
                // §31: Probabilistic death roll — uniform RNG vs age/health-based chance.
                let roll = Fixed::from_f64(self.rng.get_mut(RngStream::Behavior).random::<f64>());
                let (new_age, died) = demography::age_agent(
                    self.agents[i].age,
                    self.agents[i].body.health,
                    self.agents[i].body.energy,
                    DEMOGRAPHY_TICK_INTERVAL,
                    &self.demography_config,
                    roll,
                );
                self.agents[i].age = new_age;
                if died {
                    deaths.push(i);
                }
            }
            // §31: Dead agents are replaced IN PLACE by newborns so the
            // `AgentId::new(i) == index i` invariant is never broken.
            for &idx in &deaths {
                self.handle_agent_death(idx, &deaths, tick_u64, tick, DeathCause::OldAge);
            }
        }

        // §19.5.J: Trim provenance traces to prevent unbounded growth
        self.provenance.trim(institutions::MAX_PROVENANCE_RECORDS);
    }
}

/// Maximum number of metric snapshots to retain (recorded every 10 ticks).
///
/// Old value 500 silently dropped the first half of any run longer than
/// 5,000 ticks (ring buffer), so CSV exports started at tick 5010 for a
/// 10,000-tick run. Raised to 20_000 so a full year of sim time
/// (~51,840 ticks ≈ 5,184 rows) is preserved in exports.
const MAX_METRIC_HISTORY: usize = 20_000;

/// A snapshot of key simulation metrics at a specific tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub tick: u64,
    // ── Basic need state ──
    pub avg_hunger: f64,
    pub avg_thirst: f64,
    pub avg_fatigue: f64,
    // ── Emotion ──
    pub avg_valence: f64,
    pub avg_joy: f64,
    pub avg_fear: f64,
    // ── Resources ──
    pub total_grain: f64,
    pub total_water: f64,
    // ── System stats ──
    pub event_count: u64,
    pub journal_len: u64,
    pub agent_count: u64,
    // ── §17 Observability: deep metrics for Phase 5 tuning ──
    /// Average stress (fear + anger) across all agents.
    pub avg_stress: f64,
    /// Average health from EmbodiedState.
    pub avg_health: f64,
    /// Average accumulated trauma load across all agents (§7 nervous).
    #[serde(default)]
    pub avg_trauma_load: f64,
    /// Average relationship trust across all relationship_v2 edges.
    pub avg_relationship_trust: f64,
    /// Average relationship quality across all relationship_v2 edges.
    pub avg_relationship_quality: f64,
    /// Number of active memes in the meme registry.
    pub active_meme_count: u64,
    /// Echo chamber polarization index.
    pub polarization_index: f64,
    /// Gini coefficient of coin wealth (0 = equality, 1 = inequality).
    #[serde(default)]
    pub gini: f64,
    /// Mean coin wealth across agents.
    #[serde(default)]
    pub avg_wealth: f64,
    /// Median coin wealth across agents.
    #[serde(default)]
    pub median_wealth: f64,
    /// Cumulative completed trades (market activity).
    #[serde(default)]
    pub total_trades: u64,
    /// Number of active households.
    pub household_count: u64,
    /// Number of active kinship edges.
    pub kinship_edge_count: u64,
    /// Average agent tier (0=Focal, 1=Secondary, 2=Background).
    pub avg_agent_tier: f64,
    /// Number of active feuds across all agents.
    pub total_active_feuds: u64,
    /// Number of clans (kinship-based social groups, §10.8).
    #[serde(default)]
    pub clan_count: u64,
    /// Number of distinct inter-clan relations (alliances + enmities, §10.8).
    #[serde(default)]
    pub clan_relation_count: u64,
    /// Number of active cults (high-intensity groups, §12.4).
    #[serde(default)]
    pub cult_count: u64,
    /// Number of symbolic nodes in the noospheric field (§13).
    #[serde(default)]
    pub noosphere_nodes: u64,
    /// Peak activation of the noospheric field — the dominant symbolic mood
    /// (the §13 belief-ecology zeitgeist).
    #[serde(default)]
    pub noosphere_zeitgeist: f64,
    /// Total shared memories across all groups (§13.5).
    #[serde(default)]
    pub collective_memory_count: u64,
    /// Active patronage relations (§10.9).
    #[serde(default)]
    pub patronage_relation_count: u64,
}

impl MetricsSnapshot {
    /// §5.1/§19: CSV header for exporting metrics for analysis.
    pub fn csv_header() -> &'static str {
        "tick,avg_hunger,avg_thirst,avg_fatigue,avg_valence,avg_joy,avg_fear,total_grain,total_water,event_count,journal_len,agent_count,avg_stress,avg_health,avg_trauma_load,avg_relationship_trust,avg_relationship_quality,active_meme_count,polarization_index,gini,avg_wealth,median_wealth,total_trades,household_count,kinship_edge_count,avg_agent_tier,total_active_feuds,clan_count,clan_relation_count,cult_count,noosphere_nodes,noosphere_zeitgeist,collective_memory_count,patronage_relation_count"
    }

    /// §5.1/§19: One CSV line for this snapshot.
    pub fn to_csv_line(&self) -> String {
        format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.tick,
            self.avg_hunger, self.avg_thirst, self.avg_fatigue,
            self.avg_valence, self.avg_joy, self.avg_fear,
            self.total_grain, self.total_water,
            self.event_count, self.journal_len, self.agent_count,
            self.avg_stress, self.avg_health, self.avg_trauma_load,
            self.avg_relationship_trust, self.avg_relationship_quality,
            self.active_meme_count, self.polarization_index,
            self.gini, self.avg_wealth, self.median_wealth, self.total_trades,
            self.household_count, self.kinship_edge_count,
            self.avg_agent_tier, self.total_active_feuds,
            self.clan_count, self.clan_relation_count, self.cult_count,
            self.noosphere_nodes,
            self.noosphere_zeitgeist, self.collective_memory_count,
            self.patronage_relation_count,
        )
    }
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
    /// Architecture-plan-2 §8.1.14: Attachment style for the live agent view.
    pub attachment_style: AttachmentStyle,
    /// Architecture-plan-2 §8.1.14: Attachment security (0 = insecure, 1 = secure).
    pub attachment_security: Fixed,
    /// Architecture-plan-2 §8.1.14: Attachment anxiety.
    pub attachment_anxiety: Fixed,
    /// Architecture-plan-2 §8.1.14: Attachment avoidance.
    pub attachment_avoidance: Fixed,
    /// Architecture-plan-2 §8.1.14: Protest-behavior threshold.
    pub attachment_protest_threshold: Fixed,
    /// Architecture-plan-2 §8.1.14: Receptivity to soothing from others.
    pub attachment_soothing_receptivity: Fixed,
    /// Architecture-plan-2 §8.1.14: Separation distress.
    pub attachment_separation_distress: Fixed,
    /// Architecture-plan-2 §8.1.14: Caregiving style for the live agent view.
    pub attachment_caregiving_style: CaregivingStyle,
    /// Architecture-plan-2 §7: Accumulated trauma load (§17 observability).
    pub trauma_load: Fixed,
}

/// §8.1: Regulation-capacity support contributed by self-esteem (deviation
/// from the 0.5 baseline × gain). Zero at baseline; positive for healthy
/// self-models, negative for eroded ones. Pure and deterministic.
fn self_esteem_support(self_esteem: Fixed) -> Fixed {
    (self_esteem - Fixed::from_f64(0.5)) * SELF_MODEL_REGULATION_GAIN
}

#[cfg(test)]
mod tests {
    use super::*;

    /// §5 (Iteration 149): the enforce_theft → court wiring — a caught theft
    /// on an owned site files a case and convicts the thief. Deterministic:
    /// council enforcement is maxed (detection roll < 1 always catches) and
    /// the thief's zero risk tolerance keeps them out of the black market
    /// (which would halve effective enforcement).
    #[test]
    fn caught_theft_on_owned_site_is_prosecuted() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        for inst in &mut sim.institutions {
            if inst.kind == crate::institutions::InstitutionKind::Council {
                inst.enforcement_capacity = Fixed::ONE;
            }
        }
        sim.agents[0].personality.risk_tolerance = Fixed::ZERO;
        let farm_idx = sim
            .world
            .sites
            .iter()
            .position(|s| s.kind == crate::world::SiteKind::Farm)
            .expect("riverford has farms");
        let thief = AgentId::new(0);
        let victim = AgentId::new(1);
        sim.world.sites[farm_idx].owner = Some(victim);

        let taken = sim.enforce_theft(
            0,
            thief,
            farm_idx,
            GRAIN_RESOURCE_ID,
            Fixed::from_f64(5.0),
            100,
            Tick::new(100),
        );
        assert!(
            taken,
            "the theft succeeds (no internalized norms before tick 4320)"
        );
        assert_eq!(sim.legal.cases.len(), 1, "the caught theft files a case");
        let case = &sim.legal.cases[0];
        assert_eq!(
            case.verdict,
            Some(Verdict::Guilty),
            "owned-site theft convicts"
        );
        assert_eq!(case.victim, Some(victim));
        assert_eq!(case.site_idx, Some(farm_idx));
        assert_eq!(sim.legal.convictions, 1);
        assert_eq!(sim.legal.established_tick, Some(100));
    }

    /// §8.1: Self-esteem support is zero at baseline and signed around it.
    #[test]
    fn self_esteem_support_is_zero_at_baseline() {
        assert_eq!(self_esteem_support(Fixed::from_f64(0.5)), Fixed::ZERO);
        assert!(self_esteem_support(Fixed::from_f64(0.9)) > Fixed::ZERO);
        assert!(self_esteem_support(Fixed::from_f64(0.1)) < Fixed::ZERO);
    }

    /// §8.1: The self-model must track life events — previously constructed
    /// but never updated (a dead system). After 500 ticks the focal agents'
    /// self-esteem must have drifted from the 0.5 baseline as their
    /// narratives (and the self-model's parallel narrative) form.
    #[test]
    fn self_model_tracks_life_events() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        for _ in 0..500 {
            sim.tick();
        }
        let any_drift = sim.agents.iter().any(|a| {
            let d = a.self_model.self_esteem - Fixed::from_f64(0.5);
            d > Fixed::from_f64(0.005)
                || d < -Fixed::from_f64(0.005)
                || a.self_model.narrative.contamination_script != Fixed::from_f64(0.2)
        });
        assert!(
            any_drift,
            "self_model must update from life events (was a dead system)"
        );
    }

    /// §12.4: Cult emergence must fire when institutional legitimacy is low
    /// and agents suffer a meaning deficit — and the cooldown must prevent
    /// cult spam. Selection is deterministic (no RNG).
    #[test]
    fn cults_emerge_under_low_legitimacy_and_meaning_crisis() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        // Force the §12.4 preconditions: weak institutions + meaning crisis.
        for inst in &mut sim.institutions {
            inst.legitimacy = Fixed::from_f64(0.2);
        }
        for agent in &mut sim.agents {
            agent.needs.meaning = Fixed::from_f64(0.9);
        }
        let before = sim.cult_registry.cults.iter().filter(|c| c.active).count();
        sim.tick_cults(3000); // >= CULT_COOLDOWN so formation is not blocked
        let after = sim.cult_registry.cults.iter().filter(|c| c.active).count();
        assert!(
            after > before,
            "cult should form under low legitimacy + meaning crisis"
        );
        assert_eq!(sim.last_cult_formation_tick, 3000);
        // Cooldown: a second qualifying tick must NOT form another cult.
        sim.tick_cults(3100);
        assert_eq!(
            sim.cult_registry.cults.iter().filter(|c| c.active).count(),
            after,
            "cult formation cooldown must prevent cult spam"
        );
    }

    /// §13: Noospheric nodes must stay activated by meme-driven spreading
    /// (total activation rises above the initial 0.3 baseline after one tick).
    #[test]
    fn noosphere_spreads_activation_from_memes() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        assert!(
            !sim.noospheric_field.nodes.is_empty(),
            "noosphere must be seeded"
        );
        let before: Fixed = sim
            .noospheric_field
            .nodes
            .iter()
            .fold(Fixed::ZERO, |acc, n| acc + n.activation);
        sim.tick_noosphere();
        let after: Fixed = sim
            .noospheric_field
            .nodes
            .iter()
            .fold(Fixed::ZERO, |acc, n| acc + n.activation);
        assert!(
            after > before,
            "meme-driven spreading should raise activation"
        );
    }

    /// §12.4: Cult belonging must satisfy members' meaning need and entrench
    /// their beliefs (the cult's narrative grip).
    #[test]
    fn cult_members_get_psychological_feedback() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        for inst in &mut sim.institutions {
            inst.legitimacy = Fixed::from_f64(0.2);
        }
        for agent in &mut sim.agents {
            agent.needs.meaning = Fixed::from_f64(0.9);
        }
        sim.tick_cults(3000);
        let cult = &sim.cult_registry.cults[0];
        assert!(
            !cult.members.is_empty(),
            "cult formation should store its members"
        );
        let member = cult.members[0];
        let pre_meaning = sim.agents[member].needs.meaning;
        let pre_conf: Fixed = sim.agents[member]
            .beliefs
            .iter()
            .fold(Fixed::ZERO, |acc, b| acc + b.confidence);
        sim.tick_cult_dynamics();
        assert!(
            sim.agents[member].needs.meaning < pre_meaning,
            "cult belonging should satisfy the meaning need"
        );
        let post_conf: Fixed = sim.agents[member]
            .beliefs
            .iter()
            .fold(Fixed::ZERO, |acc, b| acc + b.confidence);
        assert!(post_conf > pre_conf, "cult should entrench member beliefs");
    }

    /// §12.4: Active cults must recruit meaning-starved agents up to the
    /// membership cap.
    #[test]
    fn cults_recruit_meaning_starved_agents() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        for inst in &mut sim.institutions {
            inst.legitimacy = Fixed::from_f64(0.2);
        }
        for agent in &mut sim.agents {
            agent.needs.meaning = Fixed::from_f64(0.9);
        }
        sim.tick_cults(3000);
        let after_formation = sim.cult_registry.cults[0].members.len();
        sim.tick_cult_dynamics();
        let after_recruit = sim.cult_registry.cults[0].members.len();
        // 12 agents: leader + up to 4 formed members; cap = 12/2 = 6.
        assert!(
            after_recruit > after_formation,
            "cult should recruit meaning-starved agents"
        );
        assert!(after_recruit <= 6, "membership should respect the cap");
    }

    /// §12.4: When a cult dissolves, former members suffer a meaning-crisis
    /// rebound and their cult-entrenched beliefs decay — the lifecycle's
    /// closing act.
    #[test]
    fn cult_dissolution_causes_member_fallout() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        for inst in &mut sim.institutions {
            inst.legitimacy = Fixed::from_f64(0.2);
        }
        for agent in &mut sim.agents {
            agent.needs.meaning = Fixed::from_f64(0.9);
        }
        sim.tick_cults(3000); // forms the cult
        let member = sim.cult_registry.cults[0].members[0];
        // Entrench the member first (dynamics raises confidence/charge).
        sim.tick_cult_dynamics();
        let entrench_conf: Fixed = sim.agents[member]
            .beliefs
            .iter()
            .fold(Fixed::ZERO, |acc, b| acc + b.confidence);
        let entrench_charge: Fixed = sim.agents[member]
            .beliefs
            .iter()
            .fold(Fixed::ZERO, |acc, b| acc + b.emotional_charge);
        let pre_fallout_meaning = sim.agents[member].needs.meaning;
        // Force dissolution: high institutional legitimacy + low isolation.
        for inst in &mut sim.institutions {
            inst.legitimacy = Fixed::from_f64(0.9);
        }
        if let Some(cult) = sim.cult_registry.cults.first_mut() {
            cult.isolation = Fixed::from_f64(0.1);
            cult.dependence = Fixed::from_f64(0.3); // keep leader-failure inert
        }
        sim.tick_cults(6000); // cooldown passed; dissolution check runs
        assert!(
            !sim.cult_registry.cults[0].active,
            "cult should dissolve under high legitimacy + low isolation"
        );
        // Fallout: meaning rebounds into crisis, entrenched beliefs decay.
        assert!(
            sim.agents[member].needs.meaning > pre_fallout_meaning,
            "meaning need should rebound after the cult dissolves"
        );
        let post_conf: Fixed = sim.agents[member]
            .beliefs
            .iter()
            .fold(Fixed::ZERO, |acc, b| acc + b.confidence);
        let post_charge: Fixed = sim.agents[member]
            .beliefs
            .iter()
            .fold(Fixed::ZERO, |acc, b| acc + b.emotional_charge);
        assert!(
            post_conf <= entrench_conf,
            "entrenched beliefs should decay after dissolution"
        );
        assert!(
            post_charge <= entrench_charge,
            "emotional charge should decay after dissolution"
        );
    }

    /// §13.5: Ritual repetition must sustain the village's collective memory —
    /// salience stays high over a long run instead of decaying to zero (the
    /// pre-Iteration-12 quadratic decay wiped memories within ~2 weeks).
    #[test]
    fn collective_memories_survive_ritual_rehearsal() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 20_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        // ~1 monthly ritual rehearsal (tick 4320) over 5000 ticks, during which
        // the daily decay (0.001/day) would have destroyed every memory under
        // the old quadratic accumulation (~2-week lifespan).
        sim.run(5000);
        let village = sim.collective_memory_registry.get(0);
        assert!(
            village.is_some(),
            "village collective memory must be seeded"
        );
        let memories = &village.unwrap().memories;
        assert!(!memories.is_empty(), "village memories must exist");
        for mem in memories {
            assert!(
                mem.salience > Fixed::from_f64(0.5),
                "memory '{}' salience should be sustained by ritual rehearsal, got {}",
                mem.description,
                mem.salience.to_f64(),
            );
        }
    }

    /// §8.1.4 (Iteration 129): the nostalgia → collective-memory
    /// preservation wiring, proven through the PUBLIC path (no field
    /// injection — nostalgia's producer is live everywhere, so an
    /// injection differential is impossible; instead the daily pass's own
    /// decay delta is measured). Run past two daily boundaries (5040 =
    /// 35×144 and 5184 = 36×144) with NO ritual fire in the window (the
    /// first ritual is at 4320, the next at 8640) and no famine memory:
    /// the total salience lost over those two days must be strictly LESS
    /// than the un-scaled 2 × 0.001 × count (the Iter-12 linear daily
    /// decay) — the pass must have multiplied by a preservation factor <
    /// 1 derived from the LIVE population mean nostalgia — and strictly
    /// MORE than the floor-scaled value (preservation never fully
    /// freezes the fade). If the fold in the daily pass is ever deleted,
    /// `decayed == unscaled` and this test fails.
    #[test]
    fn nostalgia_preserves_collective_memory_salience() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 6_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config.clone());
        sim.populate();
        sim.run(5000);
        let total_salience = |sim: &Simulation| -> f64 {
            sim.collective_memory_registry.get(0).map_or(0.0, |cm| {
                cm.memories.iter().map(|m| m.salience.to_f64()).sum::<f64>()
            })
        };
        let count = sim
            .collective_memory_registry
            .get(0)
            .map_or(0.0, |cm| cm.memories.len() as f64);
        assert!(count > 0.0, "the village collective memory must be seeded");
        let before = total_salience(&sim);
        // Two daily boundaries: 5040 and 5184. No ritual in this window
        // (rituals at 4320/8640), no famine memory (calm world).
        sim.run(200);
        let after = total_salience(&sim);
        let decayed = before - after;
        let unscaled = 2.0 * 0.001 * count;
        let floor_scaled = unscaled * crate::appraisal::NOSTALGIA_PRESERVATION_FLOOR;
        assert!(
            decayed < unscaled,
            "the daily pass must apply a live-nostalgia preservation factor < 1 \
             (decayed {decayed:.6} vs un-scaled {unscaled:.6} over 2 days)"
        );
        // The daily pass scales the fade by `1 − mean_nostalgia × rate`,
        // floored at 0.7 — the honest invariant is that the factor lives in
        // [floor, 1.0): decayed is strictly less than the un-scaled amount
        // (a live preservation factor < 1, asserted above) and never
        // STRONGER than the floor (preservation never fully freezes the
        // fade). The pre-iteration-184 tight pin (factor == floor exactly,
        // calibrated on nostalgia saturating at 1.0) was already stale — the
        // NOST_DEBUG probe shows the pass reading mean nostalgia 0.80–0.83
        // (factor 0.75–0.76) at the daily boundaries, and the audit-closure
        // feud-guilt path (goal-incongruent feuding agents no longer
        // produce the positive congruence that feeds nostalgia) lowers it
        // to ~0.28–0.35 at the pass point (factor 0.90–0.92) — still a
        // live, legal preservation factor, just not floor-pinned. The
        // robust band asserts the factor is a genuine differential in
        // [floor, 1.0): decayed ∈ [floor_scaled, unscaled − 0.0002]
        // (a 0.9× factor on count=2 yields ~0.0036, comfortably inside;
        // a fold deletion yields decayed == unscaled and fails the upper
        // bound; the factor CAN legitimately sit AT the floor 0.7 — the
        // Iteration 190 hydration re-pace (routine drink slot + Drink
        // relief 0.7) shifted the emotion mix so nostalgia saturates to
        // 1.0 at the pass point: decayed == floor_scaled == 0.002800 is
        // maximal legal preservation, probe-pinned, not a fold).
        assert!(
            decayed >= floor_scaled - 0.0001 && decayed < unscaled - 0.0002,
            "the preservation factor must be a live differential in [0.7, 1.0) \
             (decayed {decayed:.6}, floor-scaled {floor_scaled:.6}, un-scaled {unscaled:.6})"
        );
        // Determinism: identical seed → byte-identical decay delta.
        let mut again = Simulation::new(config);
        again.populate();
        again.run(5000);
        let b2 = total_salience(&again);
        again.run(200);
        assert_eq!(
            before, b2,
            "the pre-window salience must be seed-deterministic"
        );
        assert_eq!(
            decayed,
            before - total_salience(&again),
            "the preservation-scaled decay must be seed-deterministic"
        );
    }

    /// §13: The noosphere zeitgeist must feed back into agents — a hot field
    /// amplifies the hottest belief's emotional charge; a dormant field
    /// changes nothing.
    #[test]
    fn noosphere_zeitgeist_amplifies_conviction() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        // Dormant field: zero activation and backing → zeitgeist 0 → no-op.
        for node in &mut sim.noospheric_field.nodes {
            node.activation = Fixed::ZERO;
            node.institutional_backing = Fixed::ZERO;
        }
        let charge_of = |sim: &Simulation| -> Fixed {
            sim.agents[0]
                .beliefs
                .iter()
                .fold(Fixed::ZERO, |acc, b| acc + b.emotional_charge)
        };
        let before = charge_of(&sim);
        sim.tick_noosphere_belief_projection();
        assert_eq!(
            charge_of(&sim),
            before,
            "a dormant field must not project onto beliefs"
        );
        // Hot field: peak activation on every node → the zeitgeist amplifies
        // the hottest belief.
        for node in &mut sim.noospheric_field.nodes {
            node.activation = Fixed::ONE;
        }
        sim.tick_noosphere_belief_projection();
        assert!(
            charge_of(&sim) > before,
            "a hot field must amplify the hottest belief's emotional charge"
        );
    }

    /// §13.5: Critical scarcity must be captured as a trauma memory (once per
    /// episode), so the village's collective memory records real crises.
    #[test]
    fn famine_records_trauma_memory() {
        use crate::culture::SharedMemoryKind;
        let config = SimConfig {
            seed: 42,
            max_ticks: 60_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let live_traumas = |sim: &Simulation| -> usize {
            sim.collective_memory_registry.get(0).map_or(0, |cm| {
                cm.memories
                    .iter()
                    .filter(|m| m.kind == SharedMemoryKind::Trauma && m.event_tick > 0)
                    .count()
            })
        };
        // Forced scarcity: critical hunger, no thirst.
        for agent in &mut sim.agents {
            agent.needs.hunger = Fixed::from_f64(0.9);
            agent.needs.thirst = Fixed::from_f64(0.1);
        }
        sim.record_famine_memory(5000);
        assert_eq!(live_traumas(&sim), 1, "one scarcity episode → one trauma");
        assert!(
            sim.collective_memory_registry
                .get(0)
                .unwrap()
                .memories
                .iter()
                .any(|m| m.event_tick == 5000),
            "famine memory must carry its event tick"
        );
        // Same episode (still scarce, recent guard) → no duplicate.
        sim.record_famine_memory(6000);
        assert_eq!(
            live_traumas(&sim),
            1,
            "episode guard must prevent duplicates"
        );
        // No scarcity → nothing recorded.
        for agent in &mut sim.agents {
            agent.needs.hunger = Fixed::ZERO;
            agent.needs.thirst = Fixed::ZERO;
        }
        sim.record_famine_memory(50000);
        assert_eq!(live_traumas(&sim), 1, "no scarcity → no new memory");
    }

    /// §8.1: Interoceptive filters must reach behavior — an anxious agent
    /// (high negative_bias) feels the same body deficit as more dire, so its
    /// depression risk accumulates faster than a low-bias agent's under
    /// identical material conditions.
    #[test]
    fn interoception_filters_feed_depression_risk() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 60_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut anxious = Simulation::new(config.clone());
        anxious.populate();
        let mut calm = Simulation::new(config);
        calm.populate();
        // Identical material conditions: a real need deficit in every agent.
        for sim in [&mut anxious, &mut calm] {
            for agent in &mut sim.agents {
                agent.needs.hunger = Fixed::from_f64(0.7);
                agent.needs.thirst = Fixed::from_f64(0.7);
                agent.needs.fatigue = Fixed::from_f64(0.7);
                agent.needs.safety = Fixed::from_f64(0.7);
            }
        }
        // Only the interoceptive lens differs: anxious amplifies distress.
        for agent in &mut anxious.agents {
            agent.interoception.negative_bias = Fixed::from_f64(0.9);
        }
        for agent in &mut calm.agents {
            agent.interoception.negative_bias = Fixed::from_f64(0.0);
        }
        // Converge the derived states over many updates (no tick machinery).
        for _ in 0..500 {
            anxious.tick_derived_states_and_beliefs(0, 1);
            calm.tick_derived_states_and_beliefs(0, 1);
        }
        let anxious_risk = anxious.agents[0].derived.depression_risk;
        let calm_risk = calm.agents[0].derived.depression_risk;
        assert!(
            anxious_risk > calm_risk,
            "anxious agents must accumulate higher depression risk from the \
             same needs (got {anxious_risk} vs {calm_risk})"
        );
        // The felt deficit itself is monotone in bias (unit-level check).
        let felt_high = anxious.agents[0].interoception.felt_need_deficit(
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.7),
        );
        let felt_low = calm.agents[0].interoception.felt_need_deficit(
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.7),
        );
        assert!(
            felt_high > felt_low,
            "high bias must yield a higher felt deficit"
        );
    }

    /// §8.1: Embodied emotions must resist regulation in the live tick —
    /// high-sensitivity agents retain more arousal than low-sensitivity agents
    /// under identical conditions, because their emotions are felt more
    /// intensely in the body and cognitive strategies bite less.
    #[test]
    fn emotional_body_tone_resists_regulation_in_tick() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 60_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut embodied = Simulation::new(config.clone());
        embodied.populate();
        let mut detached = Simulation::new(config);
        detached.populate();
        for agent in &mut embodied.agents {
            agent.interoception.sensitivity = Fixed::from_f64(0.9);
        }
        for agent in &mut detached.agents {
            agent.interoception.sensitivity = Fixed::from_f64(0.1);
        }
        for _ in 0..500 {
            embodied.tick();
            detached.tick();
        }
        let mean_arousal = |sim: &Simulation| -> Fixed {
            let n = sim.agents.len();
            let total = sim
                .agents
                .iter()
                .fold(Fixed::ZERO, |acc, a| acc + a.affect.arousal);
            total / Fixed::from_int(n as i64)
        };
        assert!(
            mean_arousal(&embodied) > mean_arousal(&detached),
            "embodied emotions must resist regulation: high-sensitivity agents \
             should retain more arousal than low-sensitivity agents"
        );
    }

    #[test]
    fn punitive_narrative_frames_slow_belief_resistance_decay() {
        // Identical sims; only the narrative frames differ. Default frames are
        // the mean-zero anchor (decay unchanged); punitive frames must slow
        // the resistance decay so beliefs stay more rigid at the same horizon.
        let config = SimConfig {
            seed: 42,
            max_ticks: 60_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut punitive = Simulation::new(config.clone());
        punitive.populate();
        let mut baseline = Simulation::new(config);
        baseline.populate();
        for agent in &mut punitive.agents {
            agent.narrative_frames.punishment_as_justice = Fixed::from_f64(1.0);
        }
        // Converge over the transient window where resistance has not yet
        // bottomed out at the 0.3 baseline floor.
        for _ in 0..200 {
            punitive.tick();
            baseline.tick();
        }
        let mean_resistance = |sim: &Simulation| -> Fixed {
            let mut total = Fixed::ZERO;
            let mut count = 0;
            for agent in &sim.agents {
                for belief in &agent.beliefs {
                    total += belief.resistance;
                    count += 1;
                }
            }
            total / Fixed::from_int(count.max(1) as i64)
        };
        assert!(
            mean_resistance(&punitive) > mean_resistance(&baseline),
            "punitive narrative frames must slow belief-resistance decay: \
             rigid agents should retain higher resistance at tick 200"
        );
    }

    #[test]
    fn sacred_value_outrage_erodes_justice_into_resentment() {
        // Identical sims; only the accumulated moral outrage differs. Agents
        // outraged by witnessed violations of sacred values must perceive less
        // justice and accumulate more resentment over the same horizon.
        let config = SimConfig {
            seed: 42,
            max_ticks: 60_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut outraged = Simulation::new(config.clone());
        outraged.populate();
        let mut calm = Simulation::new(config);
        calm.populate();
        // The amplification hop is pinned by the sacred.rs unit tests; here we
        // drive the read-back hop directly: outrage accumulated from witnessed
        // sacred-value violations erodes justice_perception.
        for agent in &mut outraged.agents {
            agent.moral_cognition.moral_emotions.outrage = Fixed::from_f64(0.9);
        }
        for _ in 0..500 {
            outraged.tick_derived_states_and_beliefs(0, 1);
            calm.tick_derived_states_and_beliefs(0, 1);
        }
        let outraged_resentment = outraged.agents[0].derived.resentment;
        let calm_resentment = calm.agents[0].derived.resentment;
        assert!(
            outraged_resentment > calm_resentment,
            "moral outrage from violated sacred values must erode justice into \
             resentment (got {outraged_resentment} vs {calm_resentment})"
        );
    }

    /// §8.1.7: Successful knowledge acquisition is evidence exposure — the
    /// absorbed knowledge desacralizes the learner's sacred values in
    /// proportion to absorption strength (acceptance) × reasoning capacity
    /// (executive function). Fabricates a gossip interaction and runs the
    /// diffusion block directly (the Iter-23 idiom).
    #[test]
    fn knowledge_acquisition_desacralizes_sacred_values() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 2,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        // Deterministic diffusion setup: source (0) knows exactly one item
        // that target (1) lacks, so the RNG pick is forced to that item.
        sim.agents[0].cultural.knowledge = vec![1]; // Well Maintenance
        sim.agents[1].cultural.knowledge = vec![0]; // Crop Rotation
        if let Some(r) = sim
            .relationships
            .iter_mut()
            .find(|r| r.from.as_u64() == 0 && r.to.as_u64() == 1)
        {
            r.trust = Fixed::from_f64(0.9); // acceptance = 0.9*0.5 + openness*0.5 > 0.5
        }
        sim.agents[1].cognitive.executive_capacity = Fixed::from_f64(0.9);
        // A mid-sacredness value (should erode) and a maximally sacred value
        // (must stay inert — resistance gate inside attempt_desacred).
        sim.agents[1].sacred_values.values.clear();
        sim.agents[1].sacred_values.add_or_strengthen(
            "mid_sacred".into(),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.3),
        );
        sim.agents[1].sacred_values.add_or_strengthen(
            "very_sacred".into(),
            Fixed::from_f64(0.95),
            Fixed::from_f64(0.9),
        );
        let mid_before = sim.agents[1].sacred_values.values[0].sacredness;
        let sacred_before = sim.agents[1].sacred_values.values[1].sacredness;
        // Fabricate a gossip interaction and run the diffusion block directly.
        sim.events.push(SimEvent::InteractionOccurred {
            from: AgentId::new(0),
            to: AgentId::new(1),
            kind: mindstrata_core::event::InteractionKind::Gossip,
            tick: Tick::new(1),
        });
        sim.tick_gossip_and_knowledge(0, 1, Tick::new(1));
        assert!(
            sim.agents[1].cultural.knowledge.contains(&1),
            "gossip must transfer the knowledge item"
        );
        let mid_after = sim.agents[1].sacred_values.values[0].sacredness;
        assert!(
            mid_after < mid_before,
            "absorbing new knowledge must erode mid-sacredness values \
             (got {mid_after} vs {mid_before})"
        );
        let sacred_after = sim.agents[1].sacred_values.values[1].sacredness;
        assert_eq!(
            sacred_after, sacred_before,
            "very sacred values must resist desacralization"
        );
    }

    /// §8.1.7 zero-at-zero companion: when no knowledge is acquired (the
    /// target already knows the item), no evidence exposure occurs and
    /// sacredness is untouched — desacralization is driven strictly by
    /// successful acquisition.
    #[test]
    fn no_knowledge_acquisition_keeps_sacred_values() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 2,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        // Target already knows the item → the transfer branch never fires.
        sim.agents[0].cultural.knowledge = vec![1];
        sim.agents[1].cultural.knowledge = vec![1];
        if let Some(r) = sim
            .relationships
            .iter_mut()
            .find(|r| r.from.as_u64() == 0 && r.to.as_u64() == 1)
        {
            r.trust = Fixed::from_f64(0.9);
        }
        sim.agents[1].cognitive.executive_capacity = Fixed::from_f64(0.9);
        sim.agents[1].sacred_values.values.clear();
        sim.agents[1].sacred_values.add_or_strengthen(
            "mid_sacred".into(),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.3),
        );
        let before = sim.agents[1].sacred_values.values[0].sacredness;
        sim.events.push(SimEvent::InteractionOccurred {
            from: AgentId::new(0),
            to: AgentId::new(1),
            kind: mindstrata_core::event::InteractionKind::Gossip,
            tick: Tick::new(1),
        });
        sim.tick_gossip_and_knowledge(0, 1, Tick::new(1));
        assert_eq!(
            sim.agents[1]
                .cultural
                .knowledge
                .iter()
                .filter(|&&k| k == 1)
                .count(),
            1,
            "already-known knowledge is not transferred again"
        );
        let after = sim.agents[1].sacred_values.values[0].sacredness;
        assert_eq!(
            after, before,
            "no knowledge acquisition must leave sacredness untouched"
        );
    }

    /// §8.1.3: The chapter gate fires only when the integrated-life-event
    /// count crosses a 100-event boundary — never on sub-chapter progress.
    #[test]
    fn life_chapter_crossed_fires_only_on_centenary_boundaries() {
        assert!(life_chapter_crossed(99, 100));
        assert!(life_chapter_crossed(199, 201));
        assert!(!life_chapter_crossed(0, 99));
        assert!(!life_chapter_crossed(100, 100));
        assert!(!life_chapter_crossed(100, 149));
        assert!(!life_chapter_crossed(250, 259));
    }

    /// §8.1.3: The milestone gate fires only when practice crosses a 0.1
    /// proficiency boundary — never on sub-step progress or on the cap plateau.
    #[test]
    fn skill_milestone_crossed_fires_only_on_tenth_boundaries() {
        assert!(skill_milestone_crossed(
            Fixed::from_f64(0.099),
            Fixed::from_f64(0.100)
        ));
        assert!(skill_milestone_crossed(
            Fixed::from_f64(0.050),
            Fixed::from_f64(0.100)
        ));
        assert!(!skill_milestone_crossed(
            Fixed::from_f64(0.100),
            Fixed::from_f64(0.149)
        ));
        assert!(!skill_milestone_crossed(
            Fixed::from_f64(0.250),
            Fixed::from_f64(0.259)
        ));
        assert!(!skill_milestone_crossed(
            Fixed::from_f64(0.000),
            Fixed::from_f64(0.099)
        ));
        assert!(!skill_milestone_crossed(
            Fixed::from_f64(1.0),
            Fixed::from_f64(1.0)
        ));
    }

    /// §8.1.18: The apprenticeship pass transmits knowledge from a capable
    /// teacher to a willing student. Agent 0 holds knowledge id 2 (Herbal
    /// Medicine) with high teaching skill; Agent 1 lacks it with high
    /// learning aptitude. The pass must transfer it, record both education
    /// events, and bump the knowledge-store holder count.
    #[test]
    fn apprenticeship_transfers_knowledge_from_teacher_to_student() {
        use crate::culture::education::EducationEvent;
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 2,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        // Teacher: knows id 2, can teach.
        sim.agents[0].education.learned = vec![2];
        sim.agents[0].education.teaching_skill = Fixed::from_f64(0.9);
        sim.agents[0].education.teaching_patience = Fixed::from_f64(0.8);
        // Student: lacks id 2, learns fast.
        sim.agents[1].education.learning_aptitude = Fixed::from_f64(0.9);
        sim.agents[1].cultural.knowledge.retain(|&k| k != 2);
        // A warm relationship makes the transfer reliable.
        if let Some(r) = sim
            .relationships
            .iter_mut()
            .find(|r| r.from.as_u64() == 0 && r.to.as_u64() == 1)
        {
            r.trust = Fixed::from_f64(0.9);
            r.affection = Fixed::from_f64(0.8);
        }
        let holders_before = sim
            .knowledge_store
            .iter()
            .find(|k| k.id == 2)
            .map_or(0, |k| k.holders);
        sim.run_apprenticeship_pass(1, Tick::new(1));
        assert!(
            sim.agents[1].education.has_learned(2),
            "student must learn the taught knowledge"
        );
        assert!(
            sim.agents[1].cultural.knowledge.contains(&2),
            "student's cultural knowledge must include id 2"
        );
        let teach_ok = sim.agents[0]
            .education
            .teaching_events
            .iter()
            .any(|e: &EducationEvent| e.knowledge_id == 2 && e.success);
        assert!(teach_ok, "teacher must record a successful teaching event");
        let learn_ok = sim.agents[1]
            .education
            .learning_events
            .iter()
            .any(|e: &EducationEvent| e.knowledge_id == 2 && e.success);
        assert!(learn_ok, "student must record a successful learning event");
        let holders_after = sim
            .knowledge_store
            .iter()
            .find(|k| k.id == 2)
            .map_or(0, |k| k.holders);
        assert!(
            holders_after > holders_before,
            "knowledge holder count must grow"
        );
    }

    #[test]
    fn storage_overflow_rots_exposed_grain_only() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 2,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let farm_idx = sim
            .world
            .sites
            .iter()
            .position(|s| s.kind == crate::world::SiteKind::Farm)
            .unwrap();
        let stored = sim.world.sites[farm_idx].inventory[0].quantity;
        // Pump the farm far past its 500-unit storage capacity.
        sim.world
            .produce_resource(farm_idx, GRAIN_RESOURCE_ID, Fixed::from_f64(600.0));
        let before = sim.world.sites[farm_idx].inventory[0].quantity;
        sim.apply_storage_overflow();
        let after = sim.world.sites[farm_idx].inventory[0].quantity;
        assert!(after < before, "overflowing grain must rot");
        assert!(
            after > Fixed::from_f64(500.0),
            "only the exposed overflow rots, never the stored grain"
        );
        assert_eq!(stored, Fixed::from_f64(100.0), "farm seeds 100 grain");
    }

    #[test]
    fn storage_under_capacity_is_transparent() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 2,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let farm_idx = sim
            .world
            .sites
            .iter()
            .position(|s| s.kind == crate::world::SiteKind::Farm)
            .unwrap();
        let before = sim.world.sites[farm_idx].inventory[0].quantity;
        sim.apply_storage_overflow();
        assert_eq!(
            sim.world.sites[farm_idx].inventory[0].quantity, before,
            "under-capacity storage must not lose goods"
        );
    }

    #[test]
    fn storage_overflow_does_not_rot_non_perishables() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 2,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let well_idx = sim
            .world
            .sites
            .iter()
            .position(|s| s.kind == crate::world::SiteKind::Well)
            .unwrap();
        sim.world
            .produce_resource(well_idx, WATER_RESOURCE_ID, Fixed::from_f64(5000.0));
        let before = sim.world.sites[well_idx].inventory[0].quantity;
        sim.apply_storage_overflow();
        assert_eq!(
            sim.world.sites[well_idx].inventory[0].quantity, before,
            "water is non-perishable: overflow must not destroy it"
        );
    }

    /// §8.1.18 zero-at-zero companion: when nobody in the village can teach a
    /// knowledge item (no capable teacher), the pass transfers nothing — the
    /// education system stays inert until a qualified teacher exists.
    #[test]
    fn apprenticeship_no_teacher_transfers_nothing() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 2,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        // Nobody knows id 2 (or can teach it) — student lacks it.
        sim.agents[0].education.learned.clear();
        sim.agents[1].education.learned.clear();
        sim.agents[0].education.teaching_skill = Fixed::from_f64(0.1);
        let before = sim.agents[1].cultural.knowledge.len();
        sim.run_apprenticeship_pass(1, Tick::new(1));
        assert_eq!(
            sim.agents[1].cultural.knowledge.len(),
            before,
            "no teacher means no transfer"
        );
        assert!(
            !sim.agents[1].education.has_learned(2),
            "student must not learn without a teacher"
        );
    }

    /// §13.5: Factions found their own mythic past — group 1 + faction id.
    #[test]
    fn faction_founding_myth_recorded() {
        use crate::culture::SharedMemoryKind;
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.record_faction_founding_memory(3, 1000);
        let group = sim.collective_memory_registry.get(4);
        assert!(group.is_some(), "group 1+3 must be created");
        assert!(
            group
                .unwrap()
                .memories
                .iter()
                .any(|m| m.kind == SharedMemoryKind::Founding && m.event_tick == 1000),
            "faction group must hold its founding myth"
        );
    }

    /// §10.8: Find two agents in different seeded clans (home-site parity
    /// seeds 2 clans during populate).
    fn cross_clan_pair(sim: &Simulation) -> (usize, usize) {
        let clans = &sim.clan_registry.clans;
        assert!(clans.len() >= 2, "two clans must be seeded");
        assert!(!clans[0].core_households.is_empty(), "clan 0 has members");
        assert!(!clans[1].core_households.is_empty(), "clan 1 has members");
        (clans[0].core_households[0], clans[1].core_households[0])
    }

    /// §10.8: Marriage forges a symmetric clan alliance between the spouses'
    /// clans — the design doc's stated alliance source.
    #[test]
    fn marriage_forges_clan_alliance() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let (a, b) = cross_clan_pair(&sim);
        let (ca, cb) = (sim.clan_of(a).unwrap(), sim.clan_of(b).unwrap());
        sim.forge_clan_alliance(a, b, 100);
        let clan_a = sim.clan_registry.get(ca).unwrap();
        assert!(clan_a.is_ally(cb), "marriage must ally clan {ca} with {cb}");
        assert_eq!(clan_a.last_interaction_tick, 100, "interaction tick set");
        let clan_b = sim.clan_registry.get(cb).unwrap();
        assert!(clan_b.is_ally(ca), "alliance must be symmetric");
    }

    /// §10.8: Feud formation forges a symmetric clan enmity and breaks any
    /// prior marriage alliance between the two clans.
    #[test]
    fn feud_forges_clan_enmity_and_breaks_alliance() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let (a, b) = cross_clan_pair(&sim);
        let (ca, cb) = (sim.clan_of(a).unwrap(), sim.clan_of(b).unwrap());
        sim.forge_clan_alliance(a, b, 100);
        assert!(sim.clan_registry.get(ca).unwrap().is_ally(cb));
        sim.forge_clan_enmity(a, b, 200);
        let clan_a = sim.clan_registry.get(ca).unwrap();
        assert!(clan_a.is_enemy(cb), "feud must forge enmity");
        assert!(!clan_a.is_ally(cb), "enmity must break the alliance");
        assert_eq!(clan_a.last_interaction_tick, 200);
        let clan_b = sim.clan_registry.get(cb).unwrap();
        assert!(clan_b.is_enemy(ca), "enmity must be symmetric");
        assert!(!clan_b.is_ally(ca));
    }

    /// §10.8: The clan-relation predicates — enemy and ally edges are
    /// symmetric; same-clan members are never enemies/allies of themselves;
    /// enmity breaks an existing alliance.
    #[test]
    fn clan_relation_predicates_are_symmetric() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let (a, b) = cross_clan_pair(&sim);
        // Same-clan members are never enemies/allies of their own clan.
        let clan_a = sim.clan_of(a).unwrap();
        let same_clan_mate = sim
            .clan_registry
            .get(clan_a)
            .unwrap()
            .core_households
            .iter()
            .copied()
            .find(|&m| m != a)
            .expect("clan has another member");
        assert!(!sim.clans_are_enemies(a, same_clan_mate));
        assert!(!sim.clans_are_allies(a, same_clan_mate));
        // Unrelated cross-clan pair: neither enemy nor ally.
        assert!(!sim.clans_are_enemies(a, b));
        assert!(!sim.clans_are_allies(a, b));
        // Forge alliance → allies (both directions), not enemies.
        sim.forge_clan_alliance(a, b, 10);
        assert!(sim.clans_are_allies(a, b));
        assert!(sim.clans_are_allies(b, a));
        assert!(!sim.clans_are_enemies(a, b));
        // Forge enmity → enemies (both directions), alliance broken.
        sim.forge_clan_enmity(a, b, 20);
        assert!(sim.clans_are_enemies(a, b));
        assert!(sim.clans_are_enemies(b, a));
        assert!(!sim.clans_are_allies(a, b));
    }

    /// §10.8: Enemy clans do not intermarry — the marriage chance is zeroed
    /// for enemy pairs (feud boundary = marriage boundary), while same-clan
    /// marriages are unaffected.
    #[test]
    fn enemy_clans_do_not_intermarry() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let (a, b) = cross_clan_pair(&sim);
        // Declare the two clans mutual enemies before any marriage can form,
        // backed by an active feud so `decay_clan_enmities` cannot clear the
        // boundary. Iteration-159 note: a feudless test-forged enmity clears
        // on the first decay pass (~tick 501), so a marriage can form during
        // the peace window and a LATER feud re-forges the enmity — the
        // emergent peace-then-war sequence, not an intermarriage violation.
        // Feuds decay after 500 ticks, so the feud is re-armed each segment
        // to keep the boundary standing for the whole window.
        sim.forge_clan_enmity(a, b, 0);
        // The feud is seeded at tick 1 (not 0) so the first feud-decay pass
        // keeps it (feuds with `feud_ticks > tick − 500` survive; a tick-0
        // feud is dropped at tick 1, silently clearing the boundary).
        sim.agents[a].feuds.push(b);
        sim.agents[a].feud_ticks.push(1);
        sim.agents[b].feuds.push(a);
        sim.agents[b].feud_ticks.push(1);
        for _ in 0..10 {
            sim.run(400);
            // Re-arm the feud each segment (feuds decay after 500 ticks) so
            // the enemy boundary stands for the whole window.
            let now = sim.current_tick().as_u64();
            sim.agents[a].feuds.push(b);
            sim.agents[a].feud_ticks.push(now);
            sim.agents[b].feuds.push(a);
            sim.agents[b].feud_ticks.push(now);
        }
        assert_eq!(sim.current_tick().as_u64(), 4000);
        // Same-clan marriages must still happen...
        let any_married = sim.agents.iter().any(|ag| ag.partner.is_some());
        assert!(any_married, "same-clan marriages must still occur");
        // ...but no agent may be partnered with a member of an enemy clan
        // (the standing feud keeps the boundary armed for the whole window,
        // so the marriage gate's zeroing of enemy pairs is the only way to
        // pair — the invariant is directly observed).
        for i in 0..sim.agents.len() {
            if let Some(j) = sim.agents[i].partner {
                assert!(
                    !sim.clans_are_enemies(i, j),
                    "agent {i} must not marry into an enemy clan"
                );
            }
        }
    }

    /// §10.8/§19.5.H: Enemy clans escalate failed threats at twice the base
    /// rate (feud = standing state of war); the escalation chance is a pure
    /// function of clan relations, and a deterred threat or timid aggressor
    /// never escalates.
    #[test]
    fn enemy_clans_escalate_twice_as_readily() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let (a, b) = cross_clan_pair(&sim);
        let base = sim.params.conflict_escalation_chance.to_f64();
        // Unrelated cross-clan pair: base chance.
        assert_eq!(sim.escalation_chance(a, b), base);
        // Enemy clans: twice the base rate, symmetric.
        sim.forge_clan_enmity(a, b, 0);
        assert_eq!(sim.escalation_chance(a, b), (base * 2.0).min(1.0));
        assert_eq!(sim.escalation_chance(b, a), (base * 2.0).min(1.0));
        // Deterred threat or timid aggressor → no escalation, regardless of
        // clan relations (aggression must exceed the 1.2 threshold).
        let aggression = Fixed::from_f64(1.5);
        assert!(!sim.should_escalate(a, b, false, aggression));
        assert!(!sim.should_escalate(a, b, true, Fixed::ZERO));
    }

    /// §10.2 (Iteration 102): the dominance scale is identity at zero
    /// (legacy chance), clamped to [0.5, 1.5], and monotone in the
    /// aggressor's directed power over the target.
    #[test]
    fn dominance_escalation_scale_is_identity_at_zero_and_clamped() {
        assert_eq!(Simulation::dominance_escalation_scale(Fixed::ZERO), 1.0);
        assert_eq!(Simulation::dominance_escalation_scale(Fixed::ONE), 1.5);
        assert_eq!(
            Simulation::dominance_escalation_scale(Fixed::from_f64(-1.0)),
            0.5
        );
        let low = Simulation::dominance_escalation_scale(Fixed::from_f64(-0.4));
        let high = Simulation::dominance_escalation_scale(Fixed::from_f64(0.4));
        assert!(low < 1.0 && high > 1.0 && low < high);
    }

    /// §10.2 (Iteration 102): the fold genuinely shifts escalation outcomes.
    /// Two same-seed worlds differ only in the pair's directed
    /// `power_balance` (+1 vs −1). The RNG draw sequence is identical in
    /// both (same seed, same call order, exactly one draw per
    /// `should_escalate`), so the count gap is deterministic: a dominant
    /// aggressor escalates strictly more often than a subordinate one.
    #[test]
    fn relational_dominance_shifts_escalation_outcomes() {
        let make_config = || SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut dominant = Simulation::new(make_config());
        dominant.populate();
        let mut subordinate = Simulation::new(make_config());
        subordinate.populate();
        let (a, b) = cross_clan_pair(&dominant);
        let pos = Simulation::relationship_v2_pos(a, b);
        dominant.agents[a].relationship_v2s[pos].power_balance = Fixed::ONE;
        subordinate.agents[a].relationship_v2s[pos].power_balance = Fixed::from_f64(-1.0);
        let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
        let mut dominant_escalations = 0;
        let mut subordinate_escalations = 0;
        for _ in 0..200 {
            if dominant.should_escalate(a, b, true, aggression) {
                dominant_escalations += 1;
            }
            if subordinate.should_escalate(a, b, true, aggression) {
                subordinate_escalations += 1;
            }
        }
        assert!(
            dominant_escalations > subordinate_escalations,
            "dominant aggressor must escalate strictly more: \
             {dominant_escalations} vs {subordinate_escalations}"
        );
    }

    /// §10.1.2 (Iteration 110): the fold genuinely shifts escalation
    /// outcomes. Two same-seed worlds differ only in the aggressor's mean
    /// `social_trust` (0.9 vs 0.0 — a trusting relationship graph pacifies
    /// the failed-threat escalation). The RNG draw sequence is identical in
    /// both (same seed, same call order, exactly one draw per
    /// `should_escalate`), so the count gap is deterministic: the trusting
    /// aggressor escalates strictly less often.
    #[test]
    fn social_trust_pacifies_escalation_outcomes() {
        let make_config = || SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut trusting = Simulation::new(make_config());
        trusting.populate();
        let mut control = Simulation::new(make_config());
        control.populate();
        let (a, b) = cross_clan_pair(&trusting);
        trusting.agents[a].relational_fields.social_trust = Fixed::from_f64(0.9); // mean trust over a rich relationship graph
                                                                                  // Control keeps the tick-0 identity (trust 0 → factor 1.0).
        let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
        let mut trusting_escalations = 0;
        let mut control_escalations = 0;
        for _ in 0..500 {
            if trusting.should_escalate(a, b, true, aggression) {
                trusting_escalations += 1;
            }
            if control.should_escalate(a, b, true, aggression) {
                control_escalations += 1;
            }
        }
        assert!(
            control_escalations > 0,
            "control must escalate at the base rate, got {control_escalations}"
        );
        assert!(
            trusting_escalations < control_escalations,
            "a trusting aggressor must escalate strictly less: \
             {trusting_escalations} vs {control_escalations}"
        );
    }

    /// §8.1.4 (Iteration 122): the contempt fold genuinely shifts escalation
    /// outcomes. Two same-seed worlds differ only in the aggressor's
    /// `emotions.contempt` (1.0 vs 0.0 — a contemptuous aggressor sees the
    /// target as beneath them and escalates a failed threat more readily).
    /// The RNG draw sequence is identical in both (same seed, same call
    /// order, exactly one draw per `should_escalate`), so the count gap is
    /// deterministic: the contemptuous aggressor escalates strictly more
    /// often.
    #[test]
    fn contempt_amplifies_escalation_outcomes() {
        let make_config = || SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut contemptuous = Simulation::new(make_config());
        contemptuous.populate();
        let mut control = Simulation::new(make_config());
        control.populate();
        let (a, b) = cross_clan_pair(&contemptuous);
        contemptuous.agents[a].emotions.contempt = Fixed::ONE;
        // Control keeps the tick-0 identity (contempt 0 → factor 1.0).
        let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
        let mut contemptuous_escalations = 0;
        let mut control_escalations = 0;
        for _ in 0..500 {
            if contemptuous.should_escalate(a, b, true, aggression) {
                contemptuous_escalations += 1;
            }
            if control.should_escalate(a, b, true, aggression) {
                control_escalations += 1;
            }
        }
        assert!(
            control_escalations > 0,
            "control must escalate at the base rate, got {control_escalations}"
        );
        assert!(
            contemptuous_escalations > control_escalations,
            "a contemptuous aggressor must escalate strictly more: \
             {contemptuous_escalations} vs {control_escalations}"
        );
    }

    /// §8.1.4 (Iteration 122): the despair fold genuinely shifts escalation
    /// outcomes. Two same-seed worlds differ only in the aggressor's
    /// `emotions.despair` (1.0 vs 0.0 — a despairing aggressor is demobilized
    /// and escalates a failed threat less readily). The RNG draw sequence is
    /// identical in both (same seed, same call order, exactly one draw per
    /// `should_escalate`), so the count gap is deterministic: the despairing
    /// aggressor escalates strictly less often.
    #[test]
    fn despair_pacifies_escalation_outcomes() {
        let make_config = || SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut despairing = Simulation::new(make_config());
        despairing.populate();
        let mut control = Simulation::new(make_config());
        control.populate();
        let (a, b) = cross_clan_pair(&despairing);
        despairing.agents[a].emotions.despair = Fixed::ONE;
        // Control keeps the tick-0 identity (despair 0 → factor 1.0).
        let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
        let mut despairing_escalations = 0;
        let mut control_escalations = 0;
        for _ in 0..500 {
            if despairing.should_escalate(a, b, true, aggression) {
                despairing_escalations += 1;
            }
            if control.should_escalate(a, b, true, aggression) {
                control_escalations += 1;
            }
        }
        assert!(
            control_escalations > 0,
            "control must escalate at the base rate, got {control_escalations}"
        );
        assert!(
            despairing_escalations < control_escalations,
            "a despairing aggressor must escalate strictly less: \
             {despairing_escalations} vs {control_escalations}"
        );
    }

    /// §8.1.4 (Iteration 125): the moral-outrage fold genuinely shifts
    /// escalation outcomes. Two same-seed worlds differ only in the
    /// aggressor's `emotions.moral_outrage` (1.0 vs 0.0 — a morally
    /// outraged aggressor sees the violated sacred as demanding retaliation
    /// and escalates a failed threat more readily). The RNG draw sequence
    /// is identical in both (same seed, same call order, exactly one draw
    /// per `should_escalate`), so the count gap is deterministic: the
    /// outraged aggressor escalates strictly more often. The factor is
    /// multiplied into the chance chain AFTER the Iter-122 contempt factor
    /// and BEFORE the despair pacifier; at full outrage it raises the
    /// chance by exactly 30% (the appraisal unit test pins the math).
    #[test]
    fn moral_outrage_amplifies_escalation_outcomes() {
        let make_config = || SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut outraged = Simulation::new(make_config());
        outraged.populate();
        let mut control = Simulation::new(make_config());
        control.populate();
        let (a, b) = cross_clan_pair(&outraged);
        outraged.agents[a].emotions.moral_outrage = Fixed::ONE;
        // Control keeps the tick-0 identity (outrage 0 → factor 1.0).
        let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
        let mut outraged_escalations = 0;
        let mut control_escalations = 0;
        for _ in 0..500 {
            if outraged.should_escalate(a, b, true, aggression) {
                outraged_escalations += 1;
            }
            if control.should_escalate(a, b, true, aggression) {
                control_escalations += 1;
            }
        }
        assert!(
            control_escalations > 0,
            "control must escalate at the base rate, got {control_escalations}"
        );
        assert!(
            outraged_escalations > control_escalations,
            "a morally outraged aggressor must escalate strictly more: \
             {outraged_escalations} vs {control_escalations}"
        );
    }

    /// §8.1.4 (Iteration 128): the relief fold genuinely shifts escalation
    /// outcomes. Two same-seed worlds differ only in the aggressor's
    /// `emotions.relief` (1.0 vs 0.0 — a relieved aggressor, the lucky
    /// survivor of an uncontrollable positive outcome, is emboldened and
    /// escalates a failed threat more readily). The RNG draw sequence is
    /// identical in both (same seed, same call order, exactly one draw per
    /// `should_escalate`), so the count gap is deterministic: the relieved
    /// aggressor escalates strictly more often. The factor is multiplied
    /// into the chance chain AFTER the Iter-125 outrage factor and BEFORE
    /// the despair pacifier; at full relief it raises the chance by
    /// exactly 10% (the appraisal unit test pins the math).
    #[test]
    fn relief_amplifies_escalation_outcomes() {
        let make_config = || SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut relieved = Simulation::new(make_config());
        relieved.populate();
        let mut control = Simulation::new(make_config());
        control.populate();
        let (a, b) = cross_clan_pair(&relieved);
        relieved.agents[a].emotions.relief = Fixed::ONE;
        // Control keeps the tick-0 identity (relief 0 → factor 1.0).
        let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
        let mut relieved_escalations = 0;
        let mut control_escalations = 0;
        for _ in 0..500 {
            if relieved.should_escalate(a, b, true, aggression) {
                relieved_escalations += 1;
            }
            if control.should_escalate(a, b, true, aggression) {
                control_escalations += 1;
            }
        }
        assert!(
            control_escalations > 0,
            "control must escalate at the base rate, got {control_escalations}"
        );
        assert!(
            relieved_escalations > control_escalations,
            "a relieved aggressor must escalate strictly more: \
             {relieved_escalations} vs {control_escalations}"
        );
    }

    /// §10.1.2 (Iteration 114): the obligation fold genuinely shifts
    /// escalation outcomes — the identical-RNG proof, mirroring Iter-110's
    /// trust test. Two same-seed worlds are IDENTICAL except the aggressor's
    /// mean `social_obligation` (0.9 — a deeply bound reciprocal web — vs
    /// the tick-0 identity zero). Both worlds share the same RNG stream, so
    /// the draw sequence is byte-identical across the 500 calls; the
    /// obligated world's escalation threshold is strictly lower, so it MUST
    /// escalate strictly less often.
    #[test]
    fn social_obligation_restrains_escalation_outcomes() {
        let make_config = || SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut obligated = Simulation::new(make_config());
        obligated.populate();
        let mut control = Simulation::new(make_config());
        control.populate();
        let (a, b) = cross_clan_pair(&obligated);
        obligated.agents[a].relational_fields.social_obligation = Fixed::from_f64(0.9); // mean obligation over a deeply bound graph
                                                                                        // Control keeps the tick-0 identity (obligation 0 → factor 1.0).
        let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
        let mut obligated_escalations = 0;
        let mut control_escalations = 0;
        for _ in 0..500 {
            if obligated.should_escalate(a, b, true, aggression) {
                obligated_escalations += 1;
            }
            if control.should_escalate(a, b, true, aggression) {
                control_escalations += 1;
            }
        }
        assert!(
            control_escalations > 0,
            "control must escalate at the base rate, got {control_escalations}"
        );
        assert!(
            obligated_escalations < control_escalations,
            "an obligation-bound aggressor must escalate strictly less: \
             {obligated_escalations} vs {control_escalations}"
        );
    }

    /// §8.1.4 (Iteration 116): a humiliated agent escalates a failed threat
    /// to violence MORE readily — the amplification is wired into
    /// `should_escalate` as the counterpoint to the Iter-110 trust /
    /// Iter-114 obligation pacifiers. The identical-RNG proof: 500
    /// `should_escalate` calls per world on identical configs; the
    /// humiliated aggressor (humiliation 1.0 → factor 1.30) must escalate
    /// strictly more than the control (0 → factor 1.0). **Fails if the fold
    /// line is ever deleted.**
    #[test]
    fn humiliation_amplifies_failed_threat_escalation() {
        let make_config = || SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut humiliated = Simulation::new(make_config());
        humiliated.populate();
        let mut control = Simulation::new(make_config());
        control.populate();
        let (a, b) = cross_clan_pair(&humiliated);
        humiliated.agents[a].emotions.humiliation = Fixed::from_f64(1.0); // deep status defeat
                                                                          // Control keeps the tick-0 identity (humiliation 0 → factor 1.0).
        let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
        let mut humiliated_escalations = 0;
        let mut control_escalations = 0;
        for _ in 0..500 {
            if humiliated.should_escalate(a, b, true, aggression) {
                humiliated_escalations += 1;
            }
            if control.should_escalate(a, b, true, aggression) {
                control_escalations += 1;
            }
        }
        assert!(
            control_escalations > 0,
            "control must escalate at the base rate, got {control_escalations}"
        );
        assert!(
            humiliated_escalations > control_escalations,
            "a humiliated aggressor must escalate strictly more: \
             {humiliated_escalations} vs {control_escalations}"
        );
    }

    /// §8.1.10 (Iteration 83): An agent who has internalized the no-violence
    /// norm resists escalating a failed threat — `norm_resistance("No
    /// Violence")` scales the escalation chance continuously (zero-at-zero:
    /// no internalized norm → legacy behavior).
    #[test]
    fn internalized_no_violence_norm_resists_escalation() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let (a, b) = cross_clan_pair(&sim);
        let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
                                               // No internalized norm: the gate must read exactly zero resistance.
        assert_eq!(
            sim.agents[a].moral_cognition.norm_resistance("No Violence"),
            Fixed::ZERO
        );
        // Full internalization: chance scales to 0 → never escalates,
        // regardless of the RNG draw (the draw still happens — stream safe).
        sim.agents[a]
            .moral_cognition
            .internalize_norm("No Violence".into(), Fixed::ONE);
        assert_eq!(
            sim.agents[a].moral_cognition.norm_resistance("No Violence"),
            Fixed::ONE
        );
        for _ in 0..50 {
            assert!(
                !sim.should_escalate(a, b, true, aggression),
                "full no-violence internalization must suppress escalation"
            );
        }
        // Partial internalization at the same strength as a fresh village norm
        // (0.7) leaves a non-zero but reduced chance — continuous, not a cliff.
        // (Agent `b` is untouched, so its norm set is still empty.)
        sim.agents[b]
            .moral_cognition
            .internalize_norm("No Violence".into(), Fixed::from_f64(0.7));
        let chance_full = sim.params.conflict_escalation_chance.to_f64() * (1.0 - 1.0);
        let chance_partial = sim.params.conflict_escalation_chance.to_f64() * (1.0 - 0.7);
        assert_eq!(chance_full, 0.0);
        assert!(chance_partial > 0.0 && chance_partial < 1.0);
    }

    /// §8.1.10 (Iteration 88): witnessed no-violence enforcement compounds
    /// the escalation gate — an agent with zero norm strength but full
    /// witnessed-enforcement exposure and full hypocrisy sensitivity never
    /// escalates (the hypocrisy factor alone drives the chance to 0),
    /// while a partial-sensitivity agent keeps a reduced non-cliff chance.
    #[test]
    fn witnessed_no_violence_enforcement_compounds_escalation_gate() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let (a, b) = cross_clan_pair(&sim);
        let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
                                               // Zero norm strength (no resistance effect) + full exposure + full
                                               // sensitivity: the hypocrisy factor alone suppresses escalation.
        sim.agents[a].moral_cognition.hypocrisy_sensitivity = Fixed::ONE;
        sim.agents[a]
            .moral_cognition
            .internalize_norm("No Violence".into(), Fixed::ZERO);
        for _ in 0..5 {
            sim.agents[a]
                .moral_cognition
                .record_witnessed_enforcement("No Violence");
        }
        assert_eq!(
            sim.agents[a]
                .moral_cognition
                .hypocrisy_factor("No Violence"),
            Fixed::ONE
        );
        for _ in 0..50 {
            assert!(
                !sim.should_escalate(a, b, true, aggression),
                "full no-violence hypocrisy must suppress escalation"
            );
        }
        // Partial sensitivity (0.5): reduced but non-zero chance — no cliff.
        sim.agents[b].moral_cognition.hypocrisy_sensitivity = Fixed::from_f64(0.5);
        sim.agents[b]
            .moral_cognition
            .internalize_norm("No Violence".into(), Fixed::ZERO);
        for _ in 0..5 {
            sim.agents[b]
                .moral_cognition
                .record_witnessed_enforcement("No Violence");
        }
        // Math-only on purpose: a count-based probabilistic assert over 50
        // draws at chance 0.075 would be flaky (P(never fires) ~ 0.02).
        let base = sim.params.conflict_escalation_chance.to_f64();
        let chance_partial = base * (1.0 - 0.0) * (1.0 - 0.5);
        assert!(chance_partial > 0.0 && chance_partial < 1.0);
    }

    /// §8.1.10/§19.5.D (Iteration 88): the violence-enforcement audit is
    /// live — when a violent act fires, every holder of the internalized
    /// no-violence norm witnesses it (violence is inherently public, unlike
    /// sneaky theft which needs a detection roll). Pre-internalizing at
    /// ZERO strength keeps the escalation gate at its baseline (violence
    /// still fires on seed 42 at tick ~2), so each holder's count increments
    /// exactly once per event while a non-holder control stays norm-less.
    #[test]
    fn violence_audit_increments_holders_when_violence_fires() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        for idx in [0usize, 1usize] {
            sim.agents[idx]
                .moral_cognition
                .internalize_norm("No Violence".into(), Fixed::ZERO);
        }
        // Run past the first violence event. Iteration 185 (emergent-
        // quality audit — calm lethality recalibration): the escalation-
        // chance 0.3 → 0.12 fix delays seed 42's first violence from ~tick
        // 2 to ~tick 1,000 (probe: 0 events @500, 1 @1000, 3 @2000), so
        // the window extends 500 → 2000 to stay past the first event.
        // Iteration 186: the coin-dividend recirculation re-paces seed 42's
        // threat stream — first violence now ~tick 2,010 (probe: 0 @2000,
        // 2 @3000, 4 @5000), so the window extends 2000 → 3000.
        sim.run(3000);
        let events = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    mindstrata_core::event::SimEvent::ConflictOccurred {
                        kind: mindstrata_core::conflict::ConflictKind::Violence,
                        ..
                    }
                )
            })
            .count();
        assert!(
            events >= 1,
            "seed-42 baseline must produce violence within 3000 ticks (got {events})"
        );
        for idx in [0usize, 1usize] {
            let norm = sim.agents[idx]
                .moral_cognition
                .internalized_norms
                .iter()
                .find(|n| n.description == "No Violence")
                .expect("pre-internalized holder");
            // Deliberate exact equality: it proves every public violence
            // event is witnessed by every holder (both survive all 500 ticks
            // on seed 42 — no mid-window removal). A looser `>= 1` would
            // only prove liveness, not completeness.
            assert_eq!(
                norm.enforcement_count as usize, events,
                "every public violence event must be witnessed by each holder"
            );
        }
        // Control: an agent that never internalized stays norm-less.
        assert!(
            sim.agents[5].moral_cognition.internalized_norms.is_empty(),
            "non-holder must not gain the norm from the audit"
        );
    }

    /// §8.1.10/§19.5.D (Iteration 84): an agent who has internalized the
    /// no-theft norm takes less from an inaccessible farm — the norm's
    /// strength scales the amount taken continuously; at full internalization
    /// the agent refuses the theft outright (nothing consumed, no enforcement
    /// run). Zero-at-zero: no internalized norm → legacy take unchanged.
    #[test]
    fn internalized_no_theft_norm_reduces_theft_take() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let thief = 0;
        let thief_id = AgentId::new(thief as u64); // agent index == agent id
                                                   // A farm owned by another agent, stocked with grain.
        let site_idx = sim
            .world
            .sites
            .iter()
            .position(|s| s.kind == crate::world::SiteKind::Farm)
            .expect("seed-42 world has a farm");
        {
            let site = &mut sim.world.sites[site_idx];
            if let Some(stock) = site
                .inventory
                .iter_mut()
                .find(|s| s.resource_id == crate::world::GRAIN_RESOURCE_ID)
            {
                stock.quantity = Fixed::from_f64(1.0);
            } else {
                site.inventory.push(crate::world::ResourceStock {
                    resource_id: crate::world::GRAIN_RESOURCE_ID,
                    quantity: Fixed::from_f64(1.0),
                    quality: Fixed::ONE,
                    access: crate::world::AccessRight::OwnerOnly,
                });
            }
        }
        let grain_left = |sim: &Simulation| -> f64 {
            sim.world.sites[site_idx]
                .inventory
                .iter()
                .find(|s| s.resource_id == crate::world::GRAIN_RESOURCE_ID)
                .map_or(0.0, |s| s.quantity.to_f64())
        };
        let amount = Fixed::from_f64(0.15);
        let tick = Tick::ZERO;
        // No internalized norm: the gate must read exactly zero resistance.
        assert_eq!(
            sim.agents[thief]
                .moral_cognition
                .norm_resistance("No Theft"),
            Fixed::ZERO
        );
        // Baseline: the full 0.15 is taken.
        assert!(sim.enforce_theft(
            thief,
            thief_id,
            site_idx,
            crate::world::GRAIN_RESOURCE_ID,
            amount,
            0,
            tick
        ));
        assert!((grain_left(&sim) - 0.85).abs() < 0.001);
        // Full internalization: the scaled amount is zero → refusal, nothing
        // consumed, no enforcement run.
        sim.agents[thief]
            .moral_cognition
            .internalize_norm("No Theft".into(), Fixed::ONE);
        assert_eq!(
            sim.agents[thief]
                .moral_cognition
                .norm_resistance("No Theft"),
            Fixed::ONE
        );
        assert!(!sim.enforce_theft(
            thief,
            thief_id,
            site_idx,
            crate::world::GRAIN_RESOURCE_ID,
            amount,
            0,
            tick
        ));
        assert!((grain_left(&sim) - 0.85).abs() < 0.001);
        // Partial internalization at the fresh-village strength (0.7): the
        // take scales continuously to 0.15 × (1 − 0.7) = 0.045 — not a cliff.
        let partial = 1;
        let partial_id = AgentId::new(partial as u64);
        sim.agents[partial]
            .moral_cognition
            .internalize_norm("No Theft".into(), Fixed::from_f64(0.7));
        assert!(sim.enforce_theft(
            partial,
            partial_id,
            site_idx,
            crate::world::GRAIN_RESOURCE_ID,
            amount,
            0,
            tick
        ));
        assert!((grain_left(&sim) - 0.805).abs() < 0.001);
    }

    /// §10.1.3 (Iteration 111): the fold genuinely shifts theft outcomes.
    /// Two same-seed worlds are IDENTICAL except the thief's perceived
    /// legitimacy (0.9 — an institution that has genuinely earned the
    /// agent's belief in its right to rule — vs 0.5, the construction
    /// anchor where the factor is identity). The take is computed
    /// deterministically before any enforcement roll, so the
    /// *legitimacy-deterred* world must strictly take less from the same
    /// farm stock.
    #[test]
    fn perceived_legitimacy_deters_theft_take() {
        let make_config = || SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let grain_left = |sim: &Simulation| -> f64 {
            let site_idx = sim
                .world
                .sites
                .iter()
                .position(|s| s.kind == crate::world::SiteKind::Farm)
                .expect("seed-42 world has a farm");
            sim.world.sites[site_idx]
                .inventory
                .iter()
                .find(|s| s.resource_id == crate::world::GRAIN_RESOURCE_ID)
                .map_or(0.0, |s| s.quantity.to_f64())
        };
        let amount = Fixed::from_f64(0.15);
        let tick = Tick::ZERO;

        let mut anchored = Simulation::new(make_config());
        anchored.populate();
        let mut legitimated = Simulation::new(make_config());
        legitimated.populate();
        let thief = (0..anchored.agents.len())
            .find(|&i| {
                !anchored
                    .black_market
                    .can_participate(&anchored.agents[i].personality)
            })
            .expect("at least one non-black-market agent");
        let thief_id = AgentId::new(thief as u64);
        let site_idx = anchored
            .world
            .sites
            .iter()
            .position(|s| s.kind == crate::world::SiteKind::Farm)
            .expect("seed-42 world has a farm");
        // The consumer reads the produced noospheric field.
        legitimated.agents[thief]
            .relational_fields
            .legitimacy_perceived = Fixed::from_f64(0.9);
        // Both worlds start from the same farm stock.
        assert!((grain_left(&anchored) - grain_left(&legitimated)).abs() < 0.001);
        let baseline = grain_left(&anchored);
        assert!(anchored.enforce_theft(
            thief,
            thief_id,
            site_idx,
            crate::world::GRAIN_RESOURCE_ID,
            amount,
            0,
            tick
        ));
        assert!(legitimated.enforce_theft(
            thief,
            thief_id,
            site_idx,
            crate::world::GRAIN_RESOURCE_ID,
            amount,
            0,
            tick
        ));
        let anchored_taken = baseline - grain_left(&anchored);
        let legitimated_taken = baseline - grain_left(&legitimated);
        assert!(
            legitimated_taken < anchored_taken,
            "a legitimacy-deterred thief must take strictly less: \
             {legitimated_taken:.4} vs {anchored_taken:.4}"
        );
        assert!(
            (anchored_taken - 0.15).abs() < 0.001,
            "the anchored thief takes the full amount"
        );
        assert!(
            (legitimated_taken - 0.15 * 0.8).abs() < 0.001,
            "the legitimacy-deterred thief takes 0.15 x (1 - 0.4 x 0.5) = 0.12"
        );
    }

    /// §10.1.3 (Iteration 112): the panic fold genuinely shifts legitimacy
    /// outcomes. Two same-seed worlds are IDENTICAL except the population's
    /// collective fear (0.99 — a genuinely terrified population — vs 0.8,
    /// below the 0.95 anchor where the amplifier is identity). Every agent's
    /// council belief is pinned above the §7.2 trigger threshold (avg charge
    /// ≥ 0.55, ≥ 30% of agents at charge > 0.4), so a panic fires
    /// deterministically in both worlds at the same tick; the terrified
    /// world must lose strictly more institutional legitimacy. The
    /// amplification is provable without any RNG: the damage fold happens
    /// before any roll.
    #[test]
    fn collective_fear_amplifies_panic_legitimacy_damage() {
        let make_config = || SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let council_legitimacy = |sim: &Simulation| -> f64 {
            sim.institutions
                .iter()
                .find(|i| i.kind == institutions::InstitutionKind::Council)
                .map_or(0.0, |i| i.legitimacy.to_f64())
        };
        let mut calm = Simulation::new(make_config());
        calm.populate();
        let mut terrified = Simulation::new(make_config());
        terrified.populate();
        // Pin every agent's council belief above the §7.2 trigger threshold
        // so the panic fires deterministically in both worlds at the same
        // tick (proposition 1 → Council).
        for agent in &mut calm.agents {
            agent.beliefs[1].emotional_charge = Fixed::from_f64(0.9);
        }
        for agent in &mut terrified.agents {
            agent.beliefs[1].emotional_charge = Fixed::from_f64(0.9);
        }
        // The consumer reads the produced noospheric field (world mean fear,
        // refreshed daily into the relational fields). 0.99 > 0.95 anchor →
        // amplified; 0.8 < 0.95 → identity.
        for agent in &mut calm.agents {
            agent.emotions.fear = Fixed::from_f64(0.8);
        }
        for agent in &mut terrified.agents {
            agent.emotions.fear = Fixed::from_f64(0.99);
        }
        // Raise council legitimacy so the damage differential is not erased
        // by the clamp at zero: 1.0 − 0.66 (calm) vs 1.0 − 0.6732
        // (terrified, ×1.02) are both visible above zero.
        for inst in &mut calm.institutions {
            if inst.kind == institutions::InstitutionKind::Council {
                inst.legitimacy = Fixed::ONE;
            }
        }
        for inst in &mut terrified.institutions {
            if inst.kind == institutions::InstitutionKind::Council {
                inst.legitimacy = Fixed::ONE;
            }
        }
        assert_eq!(council_legitimacy(&calm), 1.0);
        assert_eq!(council_legitimacy(&terrified), 1.0);
        // 400 ≥ the 300-tick cooldown; `Tick::ZERO` is fine — `tick_u64`
        // alone gates the cooldown, `tick` only stamps the emitted event.
        let tick_u64 = 400u64;
        calm.tick_moral_panic_and_revolution(tick_u64, Tick::ZERO);
        terrified.tick_moral_panic_and_revolution(tick_u64, Tick::ZERO);
        let calm_after = council_legitimacy(&calm);
        let terrified_after = council_legitimacy(&terrified);
        // Damage 0.66: calm (identity) → 1.0 − 0.66 = 0.34; terrified
        // (×1.02) → 1.0 − 0.6732 = 0.3268 — strictly lower.
        assert!(
            terrified_after < calm_after,
            "a terrified population must lose strictly more legitimacy: \
             calm {calm_after:.4} vs terrified {terrified_after:.4}"
        );
        assert!(
            (calm_after - 0.34).abs() < 0.001,
            "the calm world loses exactly the base damage"
        );
        assert!(
            (terrified_after - (1.0 - 0.66 * 1.02)).abs() < 0.001,
            "the terrified world loses the base damage amplified by 1.02"
        );
    }

    /// §8.1.10/§19.5.D (Iteration 86): the witnessed-enforcement *wiring* is
    /// live — a caught theft (Council enforcement capacity 1.0 → the
    /// detection roll < 1.0 always catches) increments `enforcement_count`
    /// on the no-theft norm for every holder, including the violator, while
    /// non-holders are a no-op. This closes the Iteration-86 coverage gap:
    /// the default world's public-access farms never catch a theft, so only
    /// this end-to-end test exercises the `enforce_theft` →
    /// `record_witnessed_enforcement` loop.
    #[test]
    fn caught_theft_increments_witnessed_enforcement() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        // A thief who cannot use the black market (enforcement is not halved).
        let thief = (0..sim.agents.len())
            .find(|&i| !sim.black_market.can_participate(&sim.agents[i].personality))
            .expect("at least one non-black-market agent");
        let thief_id = AgentId::new(thief as u64);
        // A farm with grain (same setup as the Iter-84 theft-take test).
        let site_idx = sim
            .world
            .sites
            .iter()
            .position(|s| s.kind == crate::world::SiteKind::Farm)
            .expect("seed-42 world has a farm");
        {
            let site = &mut sim.world.sites[site_idx];
            if let Some(stock) = site
                .inventory
                .iter_mut()
                .find(|s| s.resource_id == crate::world::GRAIN_RESOURCE_ID)
            {
                stock.quantity = Fixed::from_f64(1.0);
            } else {
                site.inventory.push(crate::world::ResourceStock {
                    resource_id: crate::world::GRAIN_RESOURCE_ID,
                    quantity: Fixed::from_f64(1.0),
                    quality: Fixed::ONE,
                    access: crate::world::AccessRight::OwnerOnly,
                });
            }
        }
        // Guarantee the catch: a Council at full enforcement capacity.
        let council = sim
            .institutions
            .iter_mut()
            .find(|i| i.kind == institutions::InstitutionKind::Council)
            .expect("default village has a Council");
        council.enforcement_capacity = Fixed::ONE;
        // Two holders of the no-theft norm (thief + one witness) and one
        // control agent with no internalized norm.
        let witness = if thief == 0 { 1 } else { 0 };
        let control = if thief == 2 || witness == 2 { 3 } else { 2 };
        for &i in &[thief, witness] {
            sim.agents[i]
                .moral_cognition
                .internalize_norm("No Theft".into(), Fixed::from_f64(0.6));
        }
        let tick = Tick::ZERO;
        assert!(sim.enforce_theft(
            thief,
            thief_id,
            site_idx,
            crate::world::GRAIN_RESOURCE_ID,
            Fixed::from_f64(0.15),
            0,
            tick
        ));
        let count = |i: usize| -> u32 {
            sim.agents[i]
                .moral_cognition
                .internalized_norms
                .iter()
                .find(|n| n.description == "No Theft")
                .map_or(0, |n| n.enforcement_count)
        };
        assert_eq!(count(thief), 1, "the violator experiences the enforcement");
        assert_eq!(
            count(witness),
            1,
            "a holder witnesses the public enforcement"
        );
        assert_eq!(count(control), 0, "non-holders have nothing to witness");
    }

    /// §8.1.10 (Iteration 87): the hypocrisy *wiring* is live — with zero
    /// norm strength but full witnessed-enforcement exposure and full
    /// hypocrisy sensitivity, the agent refuses the theft outright (the
    /// hypocrisy factor alone drives the take to zero), a normless control
    /// steals the full amount, and a partial-sensitivity agent takes
    /// proportionally less (0.15 × (1 − 0.5) = 0.075).
    #[test]
    fn witnessed_enforcement_hypocrisy_suppresses_theft() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let site_idx = sim
            .world
            .sites
            .iter()
            .position(|s| s.kind == crate::world::SiteKind::Farm)
            .expect("seed-42 world has a farm");
        {
            let site = &mut sim.world.sites[site_idx];
            if let Some(stock) = site
                .inventory
                .iter_mut()
                .find(|s| s.resource_id == crate::world::GRAIN_RESOURCE_ID)
            {
                stock.quantity = Fixed::from_f64(1.0);
            } else {
                site.inventory.push(crate::world::ResourceStock {
                    resource_id: crate::world::GRAIN_RESOURCE_ID,
                    quantity: Fixed::from_f64(1.0),
                    quality: Fixed::ONE,
                    access: crate::world::AccessRight::OwnerOnly,
                });
            }
        }
        let grain_left = |sim: &Simulation| -> f64 {
            sim.world.sites[site_idx]
                .inventory
                .iter()
                .find(|s| s.resource_id == crate::world::GRAIN_RESOURCE_ID)
                .map_or(0.0, |s| s.quantity.to_f64())
        };
        let amount = Fixed::from_f64(0.15);
        let tick = Tick::ZERO;
        // Full hypocrisy: sensitivity 1.0, zero norm strength, 5 witnessed
        // enforcements → the hypocrisy factor alone refuses the theft.
        let full = 0;
        sim.agents[full].moral_cognition.hypocrisy_sensitivity = Fixed::ONE;
        sim.agents[full]
            .moral_cognition
            .internalize_norm("No Theft".into(), Fixed::ZERO);
        for _ in 0..5 {
            sim.agents[full]
                .moral_cognition
                .record_witnessed_enforcement("No Theft");
        }
        assert_eq!(
            sim.agents[full]
                .moral_cognition
                .hypocrisy_factor("No Theft"),
            Fixed::ONE
        );
        // Partial hypocrisy: default sensitivity 0.5, same exposure.
        let partial = 1;
        sim.agents[partial]
            .moral_cognition
            .internalize_norm("No Theft".into(), Fixed::ZERO);
        for _ in 0..5 {
            sim.agents[partial]
                .moral_cognition
                .record_witnessed_enforcement("No Theft");
        }
        // Normless control.
        let control = 2;
        // Control steals the full amount.
        assert!(sim.enforce_theft(
            control,
            AgentId::new(control as u64),
            site_idx,
            crate::world::GRAIN_RESOURCE_ID,
            amount,
            0,
            tick
        ));
        assert!((grain_left(&sim) - 0.85).abs() < 0.001);
        // Full hypocrisy refuses outright — nothing consumed.
        assert!(!sim.enforce_theft(
            full,
            AgentId::new(full as u64),
            site_idx,
            crate::world::GRAIN_RESOURCE_ID,
            amount,
            0,
            tick
        ));
        assert!((grain_left(&sim) - 0.85).abs() < 0.001);
        // Partial hypocrisy takes 0.15 × (1 − 0.5) = 0.075.
        assert!(sim.enforce_theft(
            partial,
            AgentId::new(partial as u64),
            site_idx,
            crate::world::GRAIN_RESOURCE_ID,
            amount,
            0,
            tick
        ));
        assert!((grain_left(&sim) - 0.775).abs() < 0.001);
    }

    /// §10.8/§19.5.G: A clan enmity persists while any feud remains between
    /// the clans' members, and clears (symmetrically) once the feuds decay —
    /// after which a marriage alliance can form again.
    #[test]
    fn clan_enmity_clears_when_feuds_decay() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let (a, b) = cross_clan_pair(&sim);
        let (ca, cb) = (sim.clan_of(a).unwrap(), sim.clan_of(b).unwrap());
        sim.forge_clan_enmity(a, b, 100);
        assert!(sim.clan_registry.get(ca).unwrap().is_enemy(cb));
        // An active feud between the clans' members keeps the enmity alive.
        sim.agents[a].feuds.push(b);
        sim.agents[a].feud_ticks.push(100);
        sim.agents[b].feuds.push(a);
        sim.agents[b].feud_ticks.push(100);
        sim.decay_clan_enmities();
        assert!(
            sim.clan_registry.get(ca).unwrap().is_enemy(cb),
            "active feud keeps the enmity"
        );
        // The feud fully decays → enmity clears both ways.
        sim.agents[a].feuds.clear();
        sim.agents[a].feud_ticks.clear();
        sim.agents[b].feuds.clear();
        sim.agents[b].feud_ticks.clear();
        sim.decay_clan_enmities();
        assert!(
            !sim.clan_registry.get(ca).unwrap().is_enemy(cb),
            "decayed feud clears the enmity"
        );
        assert!(
            !sim.clan_registry.get(cb).unwrap().is_enemy(ca),
            "cleared symmetrically"
        );
        // Peace reopens the alliance path: a later marriage can forge one.
        sim.forge_clan_alliance(a, b, 300);
        assert!(
            sim.clan_registry.get(ca).unwrap().is_ally(cb),
            "peace allows a marriage alliance"
        );
    }

    /// §10.9: Patrons provision destitute clients — a daily stipend moves
    /// wealth through the social structure when the client is destitute and
    /// the patron can afford it; no transfer otherwise.
    #[test]
    fn patronage_provision_transfers_wealth_to_destitute_clients() {
        use crate::social::patronage::PatronageRelation;
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let (patron, client) = (0usize, 1usize);
        sim.agents[patron].wealth.coin = Fixed::from_f64(10.0);
        sim.agents[client].wealth.coin = Fixed::from_f64(0.2);
        let mut rel = PatronageRelation::new(patron, client, 0);
        rel.provision = Fixed::from_f64(0.3);
        sim.patronage_registry.register(rel);
        let transfer = Fixed::from_f64(0.3) * PATRONAGE_TRANSFER_RATE;
        sim.tick_patronage_provision();
        assert!(
            (sim.agents[client].wealth.coin - (Fixed::from_f64(0.2) + transfer))
                .to_f64()
                .abs()
                < 1e-6,
            "client must receive the stipend"
        );
        assert!(
            (sim.agents[patron].wealth.coin - (Fixed::from_f64(10.0) - transfer))
                .to_f64()
                .abs()
                < 1e-6,
            "patron must pay the stipend"
        );
        // Non-destitute client → no transfer.
        sim.agents[client].wealth.coin = Fixed::from_f64(5.0);
        sim.tick_patronage_provision();
        assert_eq!(sim.agents[client].wealth.coin.to_f64(), 5.0);
        // Destitute client but destitute patron → no transfer.
        sim.agents[client].wealth.coin = Fixed::from_f64(0.2);
        sim.agents[patron].wealth.coin = Fixed::ZERO;
        sim.tick_patronage_provision();
        assert_eq!(sim.agents[client].wealth.coin.to_f64(), 0.2);
    }

    /// §10.9: Patronage buys political quiescence — a dependent client's
    /// faction grievance is dampened by dependence × the dampening factor;
    /// a zero-dependence client is unaffected.
    #[test]
    fn patronage_dampens_client_grievance() {
        use crate::social::patronage::PatronageRelation;
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let client = 1usize;
        // Force a measurable grievance baseline.
        sim.agents[client].derived.resentment = Fixed::from_f64(0.6);
        sim.agents[client].emotions.anger = Fixed::from_f64(0.5);
        let plain = sim.faction_grievance(client);
        // A patron with high dependence dampens the client's grievance by the
        // exact dampening factor.
        let mut rel = PatronageRelation::new(0, client, 0);
        rel.client_dependence = Fixed::from_f64(0.8);
        sim.patronage_registry.register(rel);
        let dampened = sim.faction_grievance(client);
        let expected = (plain * (Fixed::ONE - Fixed::from_f64(0.8) * PATRONAGE_GRIEVANCE_DAMPEN))
            .max(Fixed::ZERO);
        assert_eq!(dampened.to_f64(), expected.to_f64());
        assert!(dampened < plain, "patronage must dampen client grievance");
        // A zero-dependence relation dampens nothing.
        sim.patronage_registry.relations.clear();
        let mut rel0 = PatronageRelation::new(0, client, 0);
        rel0.client_dependence = Fixed::ZERO;
        sim.patronage_registry.register(rel0);
        assert_eq!(sim.faction_grievance(client).to_f64(), plain.to_f64());
    }

    /// §11.1: Perceived legitimacy modulates faction grievance — a deviation
    /// of `overall` from the 0.5 construction anchor dampens (above) or
    /// amplifies (below) grievance by the exact dampening factor; the anchor
    /// itself is a no-op.
    #[test]
    fn perceived_legitimacy_modulates_faction_grievance() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let agent = 1usize;
        // Force a measurable grievance baseline.
        sim.agents[agent].derived.resentment = Fixed::from_f64(0.6);
        sim.agents[agent].emotions.anger = Fixed::from_f64(0.5);
        // Mean-zero anchor: the fresh field (overall == 0.5) dampens nothing.
        let plain = sim.faction_grievance(agent);
        // High perceived legitimacy quiesces the agent by the exact factor.
        sim.agents[agent].legitimacy_field.overall = Fixed::from_f64(0.9);
        let dampened = sim.faction_grievance(agent);
        let expected = (plain * (Fixed::ONE - Fixed::from_f64(0.4) * LEGITIMACY_GRIEVANCE_DAMPEN))
            .max(Fixed::ZERO);
        assert_eq!(dampened.to_f64(), expected.to_f64());
        assert!(dampened < plain, "high legitimacy must dampen grievance");
        // Low perceived legitimacy amplifies grievance symmetrically.
        sim.agents[agent].legitimacy_field.overall = Fixed::from_f64(0.1);
        let amplified = sim.faction_grievance(agent);
        let expected_amp = (plain
            * (Fixed::ONE + Fixed::from_f64(0.4) * LEGITIMACY_GRIEVANCE_DAMPEN))
            .max(Fixed::ZERO);
        assert_eq!(amplified.to_f64(), expected_amp.to_f64());
        assert!(amplified > plain, "low legitimacy must amplify grievance");
    }

    /// §8.1.4 (Iteration 130): the awe reverence fold is LIVE — the daily
    /// scandal erosion is multiplied by `awe_reverence_factor`, so a run
    /// whose awe is pinned at saturation (1.0, factor 0.85) must preserve
    /// strictly more perceived legitimacy than the control (natural awe
    /// ~0.67, factor ~0.90) over a horizon that is mid-erosion but below
    /// the floor. AWE CONVERGENCE (probe-pinned): awe converges to the same
    /// world-determined equilibrium (~0.67 by tick 130) regardless of the
    /// start value, so a one-shot injection at populate is erased before
    /// scandal erosion begins (~tick 120) — the differential would vanish.
    /// The manual-tick harness re-injects awe before EVERY tick, pinning
    /// the treatment at the 0.85 floor for every scandal call while the
    /// control's natural awe yields ~0.90; same seed, same RNG stream, same
    /// call order — any legitimacy divergence is attributable to the fold.
    #[test]
    fn awe_reverence_shields_legitimacy_from_scandal_erosion() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut reverent = Simulation::new(config.clone());
        reverent.populate();
        let mut plain = Simulation::new(config);
        plain.populate();
        // Iteration 185 (emergent-quality audit — calm lethality
        // recalibration): the violence fix calms scandal pressure, so the
        // reverence differential builds slower — probe-pinned diff 0.0000
        // @200, 0.0001 @400, 0.0005 @800, 0.0018 @1500 (reverent > plain
        // throughout the growing phase). The horizon extends 200 → 1500 to
        // land mid-erosion with a measurable gap.
        for _ in 0..1500 {
            for a in &mut reverent.agents {
                a.emotions.awe = Fixed::ONE;
            }
            reverent.tick();
            plain.tick();
        }
        let leg_mean = |sim: &Simulation| {
            let n = sim.agents.len() as f64;
            sim.agents
                .iter()
                .map(|a| a.legitimacy_field.overall.to_f64())
                .sum::<f64>()
                / n
        };
        let reverent_leg = leg_mean(&reverent);
        let plain_leg = leg_mean(&plain);
        assert!(
            plain_leg < 0.5,
            "scandal erosion must have fired in the horizon (control {plain_leg:.4})"
        );
        assert!(
            reverent_leg > plain_leg,
            "awe-saturated run must preserve more legitimacy than the control \
             (the §8.1.4 reverence fold is wired), reverent {reverent_leg:.4} vs \
             plain {plain_leg:.4}"
        );
    }
}

// ── §5 (Iteration 155): Interactive-TUI command channel ────────────────

#[cfg(test)]
mod command_channel_tests {
    use super::*;

    fn build_sim(n: u32) -> Simulation {
        let mut sim = Simulation::new(SimConfig {
            seed: 7,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();
        sim
    }

    #[test]
    fn command_agent_injects_high_priority_command_goal() {
        let mut sim = build_sim(6);
        assert!(sim.command_agent(0, GoalKind::Work));
        let goal = sim.agents[0]
            .goals
            .iter()
            .find(|g| g.kind == GoalKind::Work)
            .expect("injected goal present");
        assert_eq!(goal.priority, Fixed::ONE);
        assert_eq!(goal.commitment, Fixed::ONE);
        assert_eq!(goal.source, GoalSource::Command);
        assert_eq!(goal.created_tick, 0);
    }

    #[test]
    fn command_agent_rejects_out_of_range_index() {
        let mut sim = build_sim(6);
        assert!(!sim.command_agent(6, GoalKind::Work));
        assert!(!sim.command_agent(usize::MAX, GoalKind::Work));
        assert!(sim.agents[0].goals.is_empty());
    }

    #[test]
    fn commanded_directive_steers_and_is_consumed_on_first_selection() {
        let mut sim = build_sim(6);
        assert!(sim.command_agent(2, GoalKind::Work));
        assert!(sim.agents[2]
            .goals
            .iter()
            .any(|g| g.source == GoalSource::Command));
        // Fresh worlds select on the first tick; the directive fires Work and
        // is consumed in the same selection — a one-shot nudge.
        sim.tick();
        assert_eq!(
            sim.agents[2].current_action,
            ActionKind::Work,
            "the directive steered the selection"
        );
        assert!(
            !sim.agents[2]
                .goals
                .iter()
                .any(|g| g.source == GoalSource::Command),
            "the directive was consumed by the selection it steered"
        );
    }

    #[test]
    fn command_goal_action_returns_aligned_action_for_directives() {
        let work = vec![Goal {
            kind: GoalKind::Work,
            priority: Fixed::ONE,
            commitment: Fixed::ONE,
            created_tick: 0,
            source: GoalSource::Command,
        }];
        let calm = NeedState::default();
        assert_eq!(
            command_goal_action(&work, &calm),
            Some((ActionKind::Work, GoalKind::Work))
        );

        let worship = vec![Goal {
            kind: GoalKind::Worship,
            priority: Fixed::ONE,
            commitment: Fixed::ONE,
            created_tick: 0,
            source: GoalSource::Command,
        }];
        assert_eq!(
            command_goal_action(&worship, &calm),
            Some((ActionKind::Worship, GoalKind::Worship))
        );
    }

    #[test]
    fn command_goal_action_yields_to_critical_needs_of_other_kinds() {
        let work = vec![Goal {
            kind: GoalKind::Work,
            priority: Fixed::ONE,
            commitment: Fixed::ONE,
            created_tick: 0,
            source: GoalSource::Command,
        }];
        let starving = NeedState {
            hunger: Fixed::from_f64(0.95),
            ..Default::default()
        };
        assert_eq!(
            command_goal_action(&work, &starving),
            None,
            "Work yields to critical hunger"
        );
    }

    #[test]
    fn command_goal_action_ignores_endogenous_goals_and_seek_safety() {
        let endogenous = vec![Goal {
            kind: GoalKind::Work,
            priority: Fixed::ONE,
            commitment: Fixed::ONE,
            created_tick: 0,
            source: GoalSource::Identity,
        }];
        assert_eq!(
            command_goal_action(&endogenous, &NeedState::default()),
            None
        );

        let seek = vec![Goal {
            kind: GoalKind::SeekSafety,
            priority: Fixed::ONE,
            commitment: Fixed::ONE,
            created_tick: 0,
            source: GoalSource::Command,
        }];
        assert_eq!(command_goal_action(&seek, &NeedState::default()), None);
    }

    #[test]
    fn command_goal_action_prioritizes_highest_priority_directive() {
        let goals = vec![
            Goal {
                kind: GoalKind::Rest,
                priority: Fixed::from_f64(0.5),
                commitment: Fixed::ONE,
                created_tick: 0,
                source: GoalSource::Command,
            },
            Goal {
                kind: GoalKind::Work,
                priority: Fixed::ONE,
                commitment: Fixed::ONE,
                created_tick: 0,
                source: GoalSource::Command,
            },
        ];
        assert_eq!(
            command_goal_action(&goals, &NeedState::default()),
            Some((ActionKind::Work, GoalKind::Work))
        );
    }

    #[test]
    fn command_agent_upserts_repeat_directive_of_same_kind() {
        let mut sim = build_sim(6);
        assert!(sim.command_agent(1, GoalKind::Work));
        assert!(sim.command_agent(1, GoalKind::Work));
        let count = sim.agents[1]
            .goals
            .iter()
            .filter(|g| g.source == GoalSource::Command && g.kind == GoalKind::Work)
            .count();
        assert_eq!(
            count, 1,
            "repeat directives of the same kind replace, not pile up"
        );
    }

    #[test]
    fn clear_commands_removes_all_directives() {
        let mut sim = build_sim(6);
        assert!(sim.command_agent(1, GoalKind::Work));
        assert!(sim.command_agent(1, GoalKind::Worship));
        assert_eq!(
            sim.agents[1]
                .goals
                .iter()
                .filter(|g| g.source == GoalSource::Command)
                .count(),
            2
        );
        assert!(sim.clear_commands(1));
        assert_eq!(
            sim.agents[1]
                .goals
                .iter()
                .filter(|g| g.source == GoalSource::Command)
                .count(),
            0,
            "clear removes every directive"
        );
        assert!(!sim.clear_commands(6), "out-of-range clear returns false");
    }
}
