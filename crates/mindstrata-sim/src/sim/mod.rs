//! Simulation orchestrator — the fixed-tick loop.

use crate::actions::ActionKind;
use crate::appraisal::{Agency, Appraisal};
pub use crate::attention;
use crate::attention::{AttentionState, PerceptKind};
use crate::biology::EmbodiedState;
use crate::conflict::{ConflictKind, ConflictState};
use crate::culture::{CulturalState, Knowledge, KnowledgeCategory, TechnologyTree};
use crate::demography;
use crate::diplomacy::{
    DiplomacyRegistry, CARAVAN_GRAIN_BASE, EVENT_RELATION_SHIFT, RAID_GRAIN_CAP,
    RAID_GRAIN_FRACTION,
};
use crate::ecology;
use crate::health;
use crate::institutions::{Institution, InstitutionKind};
use crate::journal::{EventJournal, JournalEntryKind};
use crate::legal::{LegalCase, LegalRegistry, Verdict};
use crate::market::{MarketState, WealthState};
use crate::memory::{MemoryKind, MemoryStore, MemoryTag};
use crate::military::{MilitaryRegistry, MilitiaMember, MUSTER_CAP};
use crate::norms::NormRegistry;
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
use crate::social::attraction::AttractionModel;
use crate::social::relationship_v2::RelationshipV2;
use crate::social::status_dims::StatusDimensions;
use crate::systems::SystemContext;
use crate::theology::{Temperament, TheologicalBelief, TheologyRegistry, ELDER_AGE};
use crate::world::{World, GRAIN_RESOURCE_ID, WATER_RESOURCE_ID};
use mindstrata_core::clock::{Clock, Tick};
use mindstrata_core::event::{DeathCause, InteractionKind, SimEvent};
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::{AgentId, EntityId, ResourceId};
use mindstrata_core::rng::{RngStream, RngStreams};
use serde::{Deserialize, Serialize};

// ── Named constants for magic numbers ──────────────────────────────
/// Expected grain capacity per agent — used for scarcity normalization.
pub const EXPECTED_GRAIN_PER_AGENT: u32 = 10;
/// Water endowment assumed per agent when deriving §8.1.5 situational
/// affordance scarcity (world water stock / (this × population)).
/// Iteration 228: raised from 10 to 200 to match the larger well
/// capacity (2000 stock, 12 agents). This keeps the scarcity
/// proportionality honest: drought at 0.7 leaves 30% of 2000 = 600,
/// which is 600/2400 = 25% scarcity (realistic for a drought).
pub const EXPECTED_WATER_PER_AGENT: u32 = 200;
/// Skill improvement per tick of practice.
/// Iteration 227: increased from 0.001 to 0.002 so agents acquire
/// meaningful skill levels within 100K ticks. At 0.001 a milestone
/// needs ~100 practice ticks, but agents only practice a few times
/// per day, so skills barely moved. At 0.002 a milestone needs ~50
/// ticks, and agents reach 0.5 proficiency within ~5K practice ticks
/// (~14 days of daily practice).
/// Iteration 234: increased from 0.002 to 0.004 so skills differentiate
/// within ~10K ticks instead of ~20K. At 0.004 a milestone needs ~25
/// ticks, and agents reach 0.5 proficiency within ~2.5K practice ticks
/// (~7 days of daily practice).
pub const SKILL_GAIN_PER_TICK: Fixed = Fixed::from_raw(40); // 0.004

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
    /// Iteration 232: drought-pressure window. While active, aquifer
    /// recharge is suppressed so the drought shock's water drain persists.
    drought_until: u64,
    /// §29.2 (Iteration 240): Crisis-pressure hazard accumulator — sustained
    /// grievance/legitimacy-deficit builds toward faction formation; peace
    /// bleeds it off. See `factions::CRISIS_PRESSURE_BUILD_RATE` for the
    /// design rationale and the Fixed-quantization rationale for f64 storage.
    /// Not serialized: a restored world re-accumulates from zero (the crisis
    /// conditions that built it are re-observable in the restored state).
    faction_crisis_pressure: f64,
    /// §29.2 (Iteration 240): Tick of the most recent faction formation —
    /// enforces the pressure-path rearm refractory. Not serialized (same
    /// rationale as the accumulator).
    last_faction_formation_tick: u64,
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
    /// Iteration 218: Pre-allocated tick-scoped scratch buffers.
    /// These Vecs were previously created and dropped every tick, causing
    /// ~15.7M unnecessary heap allocations over a 100K run. Now allocated
    /// once in `populate()` and reused via `clear()` each tick.
    tick_trust_deltas: Vec<Vec<(u64, Fixed)>>,
    tick_agent_ages: Vec<Fixed>,
    tick_rel_snapshot: Vec<(AgentId, AgentId, Fixed, Fixed)>,
    tick_action_starts: Vec<(usize, ActionKind)>,
}

pub use snapshot_metrics::{AgentSummary, MetricsSnapshot};

mod api;
mod births_deaths;
mod clans;
mod core;
mod cults_noosphere;
mod diplomacy_impl;
mod economy;
mod education;
mod factions_impl;
mod household;
mod institutions_impl;
mod legal_impl;
mod marriage;
mod memory_ops;
mod norms_impl;
mod pass_action;
mod pass_appraisal;
mod pass_biology;
mod pass_cognitive;
mod pass_decay;
mod pass_social;
mod population;
mod snapshot_metrics;
mod social_cluster;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod command_channel_tests;
