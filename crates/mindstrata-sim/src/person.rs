//! Person entity and its component bundles.

use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use rand::Rng;
use serde::{Deserialize, Serialize};

// ── Psychological substrate ─────────────────────────────────────────────

/// Biological / body state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyState {
    pub health: Fixed,
    pub energy: Fixed,
    pub hunger: Fixed,
    pub thirst: Fixed,
    pub fatigue: Fixed,
    pub sickness: Fixed,
    pub injury: Fixed,
    pub fertility: Option<Fixed>,
}

impl Default for BodyState {
    fn default() -> Self {
        Self {
            health: Fixed::ONE,
            energy: Fixed::ONE,
            hunger: Fixed::ZERO,
            thirst: Fixed::ZERO,
            fatigue: Fixed::ZERO,
            sickness: Fixed::ZERO,
            injury: Fixed::ZERO,
            fertility: None,
        }
    }
}

/// Need deficit values (0 = fully satisfied, 1 = critical).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeedState {
    pub hunger: Fixed,
    pub thirst: Fixed,
    pub fatigue: Fixed,
    pub safety: Fixed,
    pub social: Fixed,
    pub esteem: Fixed,
    pub autonomy: Fixed,
    pub meaning: Fixed,
}

impl Default for NeedState {
    fn default() -> Self {
        Self {
            hunger: Fixed::from_f64(0.2),
            thirst: Fixed::from_f64(0.1),
            fatigue: Fixed::from_f64(0.1),
            safety: Fixed::from_f64(0.1),
            social: Fixed::from_f64(0.3),
            esteem: Fixed::from_f64(0.2),
            autonomy: Fixed::from_f64(0.1),
            meaning: Fixed::from_f64(0.1),
        }
    }
}

/// Personality trait model (Big Five + extensions).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    pub openness: Fixed,
    pub conscientiousness: Fixed,
    pub extraversion: Fixed,
    pub agreeableness: Fixed,
    pub neuroticism: Fixed,
    pub risk_tolerance: Fixed,
    pub conformity: Fixed,
    pub ambition: Fixed,
    pub altruism: Fixed,
    pub traditionalism: Fixed,
    pub dominance: Fixed,
    pub impulsivity: Fixed,
    /// §8.1.6: Biologically-rooted temperament layer (the state-trait dynamics
    /// target). Derived deterministically from the traits at construction,
    /// then reshaped by the plasticity pass. Observational — no decision
    /// consumer reads it yet, so calibrated runs stay byte-identical.
    #[serde(default)]
    pub temperament: Temperament,
}

impl Personality {
    /// Generate a random personality using the given RNG.
    pub fn random(rng: &mut impl Rng) -> Self {
        let mut p = Self {
            openness: Fixed::from_f64(rng.random_range(0.0..1.0)),
            conscientiousness: Fixed::from_f64(rng.random_range(0.0..1.0)),
            extraversion: Fixed::from_f64(rng.random_range(0.0..1.0)),
            agreeableness: Fixed::from_f64(rng.random_range(0.0..1.0)),
            neuroticism: Fixed::from_f64(rng.random_range(0.0..1.0)),
            risk_tolerance: Fixed::from_f64(rng.random_range(0.0..1.0)),
            conformity: Fixed::from_f64(rng.random_range(0.0..1.0)),
            ambition: Fixed::from_f64(rng.random_range(0.0..1.0)),
            altruism: Fixed::from_f64(rng.random_range(0.0..1.0)),
            traditionalism: Fixed::from_f64(rng.random_range(0.0..1.0)),
            dominance: Fixed::from_f64(rng.random_range(0.0..1.0)),
            impulsivity: Fixed::from_f64(rng.random_range(0.0..1.0)),
            // §8.1.6: Temperament is derived deterministically from the 12
            // drawn traits — zero extra RNG draws keeps seeded runs byte-identical.
            temperament: Temperament::default(),
        };
        p.temperament = Temperament::from_traits(&p);
        p
    }
}

/// §8.1.6: Biologically-rooted temperament layer — the plan's seven
/// temperament dimensions. Derived deterministically from the 12 stable
/// traits at construction (zero extra RNG draws preserves seeded-run
/// byte-identity), then slowly reshaped by `plastic_update` as the agent
/// accumulates life experience. Observational: no decision system reads
/// temperament yet (consumers are a future behavioral iteration), so
/// calibrated runs remain byte-identical.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Temperament {
    /// Reactivity — how quickly the agent responds to stress.
    pub reactivity: Fixed,
    /// Soothability — how quickly the agent recovers after stress.
    pub soothability: Fixed,
    /// Sociability — drive to engage socially.
    pub sociability: Fixed,
    /// Persistence — goal adherence under difficulty.
    pub persistence: Fixed,
    /// Sensitivity — perceptual/emotional sensitivity.
    pub sensitivity: Fixed,
    /// Regularity — rhythm/regularity of daily patterns.
    pub regularity: Fixed,
    /// Approach/withdrawal — 0 = withdrawal, 1 = approach.
    pub approach_withdrawal: Fixed,
}

/// §8.1.6: Per-tick state-trait dynamics signals consumed by
    /// `Temperament::plastic_update`. Every field is a deterministic function of
/// existing agent state — no RNG, no writes to decision-read state — so the
/// plasticity pass cannot disturb calibrated runs.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlasticitySignals {
    /// Repeated expression of stress this tick (0..1) — builds reactivity/sensitivity.
    pub repeated_stress: Fixed,
    /// Recovery / positive engagement this tick (0..1) — builds soothability.
    pub recovery: Fixed,
    /// Social engagement this tick (0..1) — builds sociability.
    pub social_engagement: Fixed,
    /// Goal striving this tick (0..1) — builds persistence.
    pub goal_striving: Fixed,
    /// Identity integration (self-model coherence, 0..1).
    pub identity_integration: Fixed,
    /// Agent age in years — drives developmental plasticity (younger = more plastic).
    pub age_years: Fixed,
}

impl Temperament {
    /// §8.1.6: Deterministic mapping from the stable trait constitution.
    /// Each dimension is a weighted blend of existing traits with
    /// coefficients summing to 1 (no division, no RNG).
    pub fn from_traits(t: &Personality) -> Self {
        Self {
            reactivity: (Fixed::from_f64(0.6) * t.neuroticism
                + Fixed::from_f64(0.4) * t.impulsivity)
                .clamp_01(),
            soothability: (Fixed::from_f64(0.4) * t.agreeableness
                + Fixed::from_f64(0.3) * (Fixed::ONE - t.neuroticism)
                + Fixed::from_f64(0.3) * t.conformity)
                .clamp_01(),
            sociability: (Fixed::from_f64(0.6) * t.extraversion
                + Fixed::from_f64(0.4) * t.agreeableness)
                .clamp_01(),
            persistence: (Fixed::from_f64(0.6) * t.conscientiousness
                + Fixed::from_f64(0.4) * t.ambition)
                .clamp_01(),
            sensitivity: (Fixed::from_f64(0.5) * t.openness
                + Fixed::from_f64(0.5) * t.neuroticism)
                .clamp_01(),
            regularity: (Fixed::from_f64(0.5) * t.conscientiousness
                + Fixed::from_f64(0.5) * t.traditionalism)
                .clamp_01(),
            approach_withdrawal: (Fixed::from_f64(0.6) * t.extraversion
                + Fixed::from_f64(0.4) * t.risk_tolerance)
                .clamp_01(),
        }
    }

    /// §8.1.6 (Iteration 105): how much a temperament-reactivity deviation
    /// from the trait-derived baseline scales the fear/anger stress response.
    /// Identity at zero deviation (Δ = 0 → amplifier 1.0 → legacy
    /// byte-identical appraise); bounded to [0.5, 1.5] so even an extreme
    /// deviation cannot more than halve or one-and-a-half-fold the response.
    /// Pure (no RNG), deterministic. The deviation is zero at construction
    /// (temperament starts at `from_traits`), so the amplifier is inert until
    /// the plasticity pass accumulates repeated-stress experience — the
    /// state-trait dynamics loop is closed: stressed agents become more
    /// reactive, then feel fear/anger more intensely.
    #[inline]
    pub fn reactivity_amplifier(deviation: Fixed) -> Fixed {
        (Fixed::ONE + deviation * Fixed::from_f64(1.0))
            .clamp(Fixed::from_f64(0.5), Fixed::from_f64(1.5))
    }

    /// §8.1.6: Trait plasticity — the plan's formula
    /// `trait_change = repeated_state_expression × identity_integration ×
    /// social_reinforcement × developmental_plasticity`, applied to the
    /// mutable temperament layer: each dimension is pushed by its repeatedly
    /// expressed state signal and pulled back toward the stable trait-derived
    /// baseline (continuous equilibrium: `x → baseline + signal`), both at
    /// the same `rate`. At the Fixed 10 000 scale the pull term needs a
    /// deviation of roughly a third to move, so the true regime is a deadband
    /// around `baseline + signal` rather than a point; likewise the effective
    /// plasticity cutoff lands near age 56 (the rate's sub-resolution tail),
    /// where the developmental term still rounds above zero but the products
    /// truncate. Deviations below ~0.0001 are inert by design. Every dimension
    /// is clamped to 0..1. The 12 core traits are intentionally immutable
    /// here — temperament consumers must be wired (future behavioral
    /// iteration) before core traits can move without breaking byte-identical
    /// calibration.
    pub fn plastic_update(&mut self, baseline: &Temperament, s: &PlasticitySignals) {
        // Developmental plasticity decays with age — full at birth, zero at 70.
        let developmental = (Fixed::ONE
            - (s.age_years / Fixed::from_f64(70.0)).clamp_01())
            .clamp_01();
        let integration = s.identity_integration.clamp_01();
        if developmental <= Fixed::ZERO || integration <= Fixed::ZERO {
            return;
        }
        // Social reinforcement amplifies the absorption of experience.
        let reinforcement =
            Fixed::ONE + Fixed::HALF * s.social_engagement.clamp_01();
        let rate = developmental * integration * reinforcement * Fixed::from_f64(0.0005);
        if rate <= Fixed::ZERO {
            return;
        }
        let stress = s.repeated_stress.clamp_01();
        let recovery = s.recovery.clamp_01();
        let social = s.social_engagement.clamp_01();
        let striving = s.goal_striving.clamp_01();
        // Push from repeated state expression; pull toward the baseline.
        self.reactivity = (self.reactivity + stress * rate
            + (baseline.reactivity - self.reactivity) * rate)
            .clamp_01();
        self.soothability = (self.soothability + recovery * rate
            + (baseline.soothability - self.soothability) * rate)
            .clamp_01();
        self.sociability = (self.sociability + social * rate
            + (baseline.sociability - self.sociability) * rate)
            .clamp_01();
        self.persistence = (self.persistence + striving * rate
            + (baseline.persistence - self.persistence) * rate)
            .clamp_01();
        // Sensitivity is driven by total arousal exposure (stress + recovery).
        self.sensitivity = (self.sensitivity + (stress + recovery) * rate
            + (baseline.sensitivity - self.sensitivity) * rate)
            .clamp_01();
        // Routinized striving builds regularity; social engagement builds approach.
        self.regularity = (self.regularity + striving * rate
            + (baseline.regularity - self.regularity) * rate)
            .clamp_01();
        self.approach_withdrawal = (self.approach_withdrawal + social * rate
            + (baseline.approach_withdrawal - self.approach_withdrawal) * rate)
            .clamp_01();
    }
}

/// Dimensional affect (PAD model).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Affect {
    pub valence: Fixed,
    pub arousal: Fixed,
    pub control: Fixed,
}

/// Discrete emotion intensities (initial set: 8 core emotions).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscreteEmotions {
    pub fear: Fixed,
    pub anger: Fixed,
    pub joy: Fixed,
    pub sadness: Fixed,
    pub trust: Fixed,
    pub shame: Fixed,
    pub pride: Fixed,
    pub guilt: Fixed,
    /// §8.1.4 expanded emotion families — the plan's 22-family roster
    /// (write-only observational state; nothing reads them for decisions).
    #[serde(default)]
    pub disgust: Fixed,
    #[serde(default)]
    pub contempt: Fixed,
    #[serde(default)]
    pub awe: Fixed,
    #[serde(default)]
    pub gratitude: Fixed,
    #[serde(default)]
    pub jealousy: Fixed,
    #[serde(default)]
    pub envy: Fixed,
    #[serde(default)]
    pub loneliness: Fixed,
    #[serde(default)]
    pub tenderness: Fixed,
    #[serde(default)]
    pub humiliation: Fixed,
    #[serde(default)]
    pub relief: Fixed,
    #[serde(default)]
    pub hope: Fixed,
    #[serde(default)]
    pub despair: Fixed,
    #[serde(default)]
    pub nostalgia: Fixed,
    #[serde(default)]
    pub moral_outrage: Fixed,
}

/// §19.5.A: How information was acquired — determines source trust weighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceSource {
    /// Direct personal experience (highest trust).
    PersonalExperience,
    /// Heard from a trusted friend/family member.
    TrustedPeer,
    /// Heard from a known community member.
    CommunityMember,
    /// Heard via gossip from an unknown source.
    Hearsay,
    /// Official institutional record/announcement.
    InstitutionalRecord,
    /// Learned through education/teaching.
    Education,
}

impl EvidenceSource {
    /// Base trust value for this source type (0.0–1.0).
    pub fn base_trust(&self) -> Fixed {
        match self {
            EvidenceSource::PersonalExperience => Fixed::from_f64(0.95),
            EvidenceSource::TrustedPeer => Fixed::from_f64(0.8),
            EvidenceSource::CommunityMember => Fixed::from_f64(0.6),
            EvidenceSource::Hearsay => Fixed::from_f64(0.3),
            EvidenceSource::InstitutionalRecord => Fixed::from_f64(0.7),
            EvidenceSource::Education => Fixed::from_f64(0.85),
        }
    }
}

/// Belief about a proposition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    pub proposition_id: u64,
    pub confidence: Fixed,
    pub emotional_charge: Fixed,
    pub identity_linkage: Fixed,
    pub resistance: Fixed,
    pub last_reinforced_tick: u64,
    /// §19.5.A: How this belief was originally acquired.
    pub source: EvidenceSource,
    /// §19.5.A: Number of times confirmed by social contacts.
    pub social_reinforcement: u32,
    /// §19.5.A: Whether this belief is based on accurate information.
    /// If false, it is misinformation that persists due to identity linkage.
    pub is_accurate: bool,
}

impl Default for Belief {
    fn default() -> Self {
        Self {
            proposition_id: 0,
            confidence: Fixed::from_f64(0.5),
            emotional_charge: Fixed::ZERO,
            identity_linkage: Fixed::ZERO,
            resistance: Fixed::from_f64(0.5),
            last_reinforced_tick: 0,
            source: EvidenceSource::PersonalExperience,
            social_reinforcement: 0,
            is_accurate: true,
        }
    }
}

/// §24: Source of a goal — determines how it was triggered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalSource {
    /// Goal generated from a pressing need deficit.
    Need,
    /// Goal driven by an identity (e.g., Farmer → Work).
    Identity,
    /// Goal driven by emotional state (e.g., fear → SeekSafety).
    Emotion,
}

/// Goal priority and source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub kind: GoalKind,
    pub priority: Fixed,
    pub commitment: Fixed,
    pub created_tick: u64,
    /// §24: What triggered this goal — affects abandonment thresholds.
    pub source: GoalSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalKind {
    Eat,
    Drink,
    Rest,
    Work,
    Socialize,
    Worship,
    SeekSafety,
}

// ── Identity system ──────────────────────────────────────────────────────

/// Kinds of personal identity an agent can hold.
/// Only identities that affect action utility are included (YAGNI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdentityKind {
    Farmer,
    Parent,
    Believer,
}



/// An agent's identity with its strength (0.0–1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub kind: IdentityKind,
    pub strength: Fixed,
}

/// Identity state for an agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdentityState {
    pub identities: Vec<AgentIdentity>,
}

impl IdentityState {
    /// Get the strength of a specific identity, or 0.0 if not present.
    pub fn strength_of(&self, kind: IdentityKind) -> Fixed {
        self.identities
            .iter()
            .find(|i| i.kind == kind)
            .map_or(Fixed::ZERO, |i| i.strength)
    }
}

// ── Moral foundations (§22.1) ─────────────────────────────────────────

/// Moral values model (Moral Foundations Theory).
/// These affect norm internalization, emotional response to violation,
/// political alignment, and factional attraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoralValues {
    /// Sensitivity to harm and suffering.
    pub care: Fixed,
    /// Sensitivity to fairness and justice.
    pub fairness: Fixed,
    /// Sensitivity to loyalty and betrayal.
    pub loyalty: Fixed,
    /// Sensitivity to authority and subversion.
    pub authority: Fixed,
    /// Sensitivity to purity and taboo violation.
    pub purity: Fixed,
    /// Sensitivity to liberty and coercion.
    pub liberty: Fixed,
}

impl Default for MoralValues {
    fn default() -> Self {
        Self {
            care: Fixed::from_f64(0.5),
            fairness: Fixed::from_f64(0.5),
            loyalty: Fixed::from_f64(0.5),
            authority: Fixed::from_f64(0.5),
            purity: Fixed::from_f64(0.5),
            liberty: Fixed::from_f64(0.5),
        }
    }
}

impl MoralValues {
    /// Generate random moral values.
    pub fn random(rng: &mut impl Rng) -> Self {
        Self {
            care: Fixed::from_f64(rng.random_range(0.2..0.9)),
            fairness: Fixed::from_f64(rng.random_range(0.2..0.9)),
            loyalty: Fixed::from_f64(rng.random_range(0.1..0.8)),
            authority: Fixed::from_f64(rng.random_range(0.1..0.8)),
            purity: Fixed::from_f64(rng.random_range(0.1..0.7)),
            liberty: Fixed::from_f64(rng.random_range(0.2..0.9)),
        }
    }

    /// Compute how much moral outrage a norm violation causes.
    /// Higher values = more anger/shame when the agent witnesses a violation.
    pub fn moral_outrage(&self, violation_severity: Fixed) -> Fixed {
        // High care + high fairness → strong outrage at any violation
        // High authority → outrage at subversion
        // High liberty → less outrage (tolerance for rule-breaking)
        let base = (self.care + self.fairness) * Fixed::from_f64(0.3);
        let authority_boost = self.authority * Fixed::from_f64(0.2);
        let liberty_dampen = self.liberty * Fixed::from_f64(0.15);
        ((base + authority_boost - liberty_dampen) * violation_severity).clamp_01()
    }
}

// ── Cognitive state (§22.1) ─────────────────────────────────────────────

/// Bounded rationality parameters.
/// §22.1: "When stressed, agents should rely on habit, imitate peers,
/// obey authority, become impulsive, become aggressive or fearful, simplify beliefs."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveState {
    /// Base attention capacity (0..1).
    pub attention_capacity: Fixed,
    /// Base executive function capacity (0..1).
    pub executive_capacity: Fixed,
    /// Current cognitive fatigue (0 = fresh, 1 = exhausted).
    pub fatigue: Fixed,
    /// Current stress level (0 = calm, 1 = panicked).
    pub stress: Fixed,

    /// Effective planning horizon in ticks.
    pub planning_horizon: u32,
    /// Heuristic bias: 0 = systematic reasoning, 1 = pure heuristics.
    pub heuristic_bias: Fixed,
}

impl Default for CognitiveState {
    fn default() -> Self {
        Self {
            attention_capacity: Fixed::from_f64(0.7),
            executive_capacity: Fixed::from_f64(0.7),
            fatigue: Fixed::ZERO,
            stress: Fixed::ZERO,
            planning_horizon: 20,
            heuristic_bias: Fixed::from_f64(0.3),
        }
    }
}

impl CognitiveState {
    /// Update cognitive state based on current emotions and needs.
    /// §22.1: Stress reduces planning horizon and increases heuristic bias.
    pub fn update(&mut self, stress: Fixed, fatigue: Fixed) {
        self.stress = (self.stress * Fixed::from_f64(0.9) + stress * Fixed::from_f64(0.1)).clamp_01();
        self.fatigue = (self.fatigue * Fixed::from_f64(0.95) + fatigue * Fixed::from_f64(0.05)).clamp_01();

        // Stress reduces effective planning horizon
        let stress_factor = Fixed::ONE - self.stress;
        let fatigue_factor = Fixed::ONE - self.fatigue * Fixed::from_f64(0.3);
        self.planning_horizon = (20.0 * stress_factor.to_f64() * fatigue_factor.to_f64()) as u32;

        // Stress increases heuristic reliance
        self.heuristic_bias = (self.stress * Fixed::from_f64(0.6) + self.fatigue * Fixed::from_f64(0.2) + Fixed::from_f64(0.2)).clamp_01();
    }
}

// ── Derived mental states (§22) ──────────────────────────────────────

/// Derived mental states computed from lower-level variables.
/// §22.1: "Do not simulate everything as direct variables.
/// Some states should be derived: trauma, depression, ambition, resilience."
/// These influence traits over time — psychology is plastic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedMentalState {
    /// Trauma risk from repeated high-stress events, low coping, low support.
    pub trauma_risk: Fixed,
    /// Depression risk from chronic need deficit, low meaning, social isolation.
    pub depression_risk: Fixed,
    /// Ambition from repeated success, high energy, competitive environment.
    pub ambition: Fixed,
    /// Resilience from social support, high coping, past recovery.
    pub resilience: Fixed,
    /// Resentment from perceived injustice, repeated norm violations witnessed.
    pub resentment: Fixed,
}

impl Default for DerivedMentalState {
    fn default() -> Self {
        Self {
            trauma_risk: Fixed::ZERO,
            depression_risk: Fixed::ZERO,
            ambition: Fixed::from_f64(0.3),
            resilience: Fixed::from_f64(0.5),
            resentment: Fixed::ZERO,
        }
    }
}

/// Inputs for derived mental state computation.
/// Grouped to avoid too-many-arguments clippy lint. Ephemeral per-tick.
pub struct MentalStateInput {
    pub stress: Fixed,
    pub coping_potential: Fixed,
    pub social_support: Fixed,
    pub need_deficit_avg: Fixed,
    pub meaning: Fixed,
    pub autonomy: Fixed,
    pub success_rate: Fixed,
    pub neuroticism: Fixed,
    pub justice_perception: Fixed,
}

impl DerivedMentalState {
    /// Compute derived mental states from lower-level variables.
    ///
    /// §22.1: These should influence traits over time.
    /// §5.1: `smoothing` and `accumulation` control the blending rate:
    ///   - smoothing=0.995, accumulation=0.005 → original slow build
    ///   - smoothing=0.99, accumulation=0.01 → 2× faster adaptation
    pub fn compute(&mut self, input: &MentalStateInput, smoothing: Fixed, accumulation: Fixed) {
        // Trauma risk: repeated stress + low coping + low support
        let stress_factor = input.stress * Fixed::from_f64(0.3);
        let coping_factor = (Fixed::ONE - input.coping_potential) * Fixed::from_f64(0.3);
        let support_factor = (Fixed::ONE - input.social_support) * Fixed::from_f64(0.2);
        let neuroticism_boost = input.neuroticism * Fixed::from_f64(0.2);
        let trauma_delta = stress_factor + coping_factor + support_factor + neuroticism_boost;
        // §5.1: Configurable accumulation — trauma builds over many ticks
        self.trauma_risk = (self.trauma_risk * smoothing + trauma_delta * accumulation).clamp_01();

        // Depression risk: chronic need deficit + low meaning + low autonomy
        let deficit_factor = input.need_deficit_avg * Fixed::from_f64(0.3);
        let meaning_factor = (Fixed::ONE - input.meaning) * Fixed::from_f64(0.25);
        let autonomy_factor = (Fixed::ONE - input.autonomy) * Fixed::from_f64(0.2);
        let depression_delta = deficit_factor + meaning_factor + autonomy_factor;
        // §5.1: Configurable accumulation
        self.depression_risk = (self.depression_risk * smoothing + depression_delta * accumulation).clamp_01();

        // Ambition: from success and energy
        self.ambition = (input.success_rate * Fixed::from_f64(0.4) + Fixed::from_f64(0.3)).clamp_01();

        // Resilience: from social support and coping
        self.resilience = (input.social_support * Fixed::from_f64(0.3) + input.coping_potential * Fixed::from_f64(0.3) + Fixed::from_f64(0.2)).clamp_01();

        // Resentment: from perceived injustice
        let injustice = (Fixed::ONE - input.justice_perception) * Fixed::from_f64(0.4);
        // §5.1: Configurable accumulation
        self.resentment = (self.resentment * smoothing + injustice * accumulation).clamp_01();
    }
}

// ── Intention system ───────────────────────────────────────────────────

/// An intention is a committed plan to achieve a goal.
/// §24.5: agents should not abandon intentions too easily.
/// Abandonment depends on failure, cost escalation, emotional shock,
/// higher-priority interruption, belief change, and social pressure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intention {
    /// The goal this intention is pursuing.
    pub goal_kind: GoalKind,
    /// How committed the agent is to this intention (0..1).
    /// Higher commitment = harder to abandon.
    pub commitment: Fixed,
    /// Tick when this intention was formed.
    pub formed_tick: u64,
    /// How many ticks this intention has been active.
    pub duration_ticks: u32,
    /// Number of consecutive failures (increases abandonment likelihood).
    pub consecutive_failures: u32,
    /// Whether this intention has been successfully completed.
    pub completed: bool,
}

impl Intention {
    /// Create a new intention for a goal.
    pub fn new(goal_kind: GoalKind, formed_tick: u64, personality_conscientiousness: Fixed) -> Self {
        // Conscientious agents form stronger commitments
        let base_commitment = Fixed::from_f64(0.5) + personality_conscientiousness * Fixed::from_f64(0.3);
        Self {
            goal_kind,
            commitment: base_commitment,
            formed_tick,
            duration_ticks: 0,
            consecutive_failures: 0,
            completed: false,
        }
    }

    /// Should this intention be abandoned?
    /// Returns true if the agent should give up and select a new action.
    pub fn should_abandon(
        &self,
        current_tick: u64,
        stress: Fixed,
        emotional_shock: bool,
    ) -> bool {
        // Never abandon completed intentions
        if self.completed {
            return false;
        }

        let duration = current_tick.saturating_sub(self.formed_tick);

        // Abandon if too many consecutive failures (commitment threshold)
        if self.consecutive_failures >= 3 {
            let fail_threshold = Fixed::ONE - self.commitment;
            if fail_threshold < Fixed::from_f64(0.3) {
                return false; // very committed agents persist through failures
            }
            return true;
        }

        // Abandon if intention has lasted too long without success
        // (commitment determines patience: high commitment = longer patience)
        let max_duration = (self.commitment * Fixed::from_f64(100.0)).to_raw() as u64;
        if duration > max_duration && max_duration > 0 {
            return true;
        }

        // Abandon on high emotional shock if commitment is low
        if emotional_shock && self.commitment < Fixed::from_f64(0.4) {
            return true;
        }

        // Abandon under extreme stress if commitment is moderate
        if stress > Fixed::from_f64(0.8) && self.commitment < Fixed::from_f64(0.6) {
            return true;
        }

        false
    }

    /// Record a failure (action didn't achieve the goal).
    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        // Failures weaken commitment slightly
        self.commitment = (self.commitment - Fixed::from_f64(0.05)).max(Fixed::ZERO);
    }

    /// Record a success (action achieved the goal).
    pub fn record_success(&mut self) {
        self.completed = true;
        // Success strengthens commitment for future intentions
        self.commitment = (self.commitment + Fixed::from_f64(0.1)).clamp_01();
    }

    /// Tick the intention (increment duration).
    pub fn tick(&mut self) {
        self.duration_ticks += 1;
    }
}

// ── Relational substrate ─────────────────────────────────────────────────

/// §19.5.G: Type of relationship — determines interaction patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipKind {
    Stranger,
    Neighbor,
    Colleague,
    Friend,
    Rival,
    Kin,
}

/// §19.5.G: Per-agent status — derived from wealth, institutional role, social connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusState {
    /// Base status from wealth (0..1).
    pub wealth_status: Fixed,
    /// Status from institutional role (0..1).
    pub role_status: Fixed,
    /// Status from social connections (0..1).
    pub social_status: Fixed,
    /// Combined effective status.
    pub effective: Fixed,
}

impl Default for StatusState {
    fn default() -> Self {
        Self {
            wealth_status: Fixed::from_f64(0.3),
            role_status: Fixed::ZERO,
            social_status: Fixed::from_f64(0.3),
            effective: Fixed::from_f64(0.3),
        }
    }
}

impl StatusState {
    /// Recompute effective status from components.
    pub fn recompute(&mut self) {
        self.effective = (self.wealth_status * Fixed::from_f64(0.4)
            + self.role_status * Fixed::from_f64(0.35)
            + self.social_status * Fixed::from_f64(0.25)).clamp_01();
    }
}

/// A directed relationship between two agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from: AgentId,
    pub to: AgentId,
    pub trust: Fixed,
    pub affection: Fixed,
    pub respect: Fixed,
    pub fear: Fixed,
    pub obligation: Fixed,
    pub last_interaction_tick: u64,
    /// §19.5.G: Relationship type — evolves based on interaction history.
    pub kind: RelationshipKind,
    /// §19.5.G: Interaction count — drives relationship evolution.
    pub interaction_count: u32,
    /// §19.5.G: Last positive interaction tick (for friendship formation).
    pub last_positive_tick: u64,
    /// §19.5.G: Last negative interaction tick (for rivalry formation).
    pub last_negative_tick: u64,
}

#[cfg(test)]
mod temperament_tests {
    use super::*;

    fn base_personality() -> Personality {
        Personality {
            openness: Fixed::from_f64(0.5),
            conscientiousness: Fixed::from_f64(0.5),
            extraversion: Fixed::from_f64(0.5),
            agreeableness: Fixed::from_f64(0.5),
            neuroticism: Fixed::from_f64(0.5),
            risk_tolerance: Fixed::from_f64(0.5),
            conformity: Fixed::from_f64(0.5),
            ambition: Fixed::from_f64(0.5),
            altruism: Fixed::from_f64(0.5),
            traditionalism: Fixed::from_f64(0.5),
            dominance: Fixed::from_f64(0.5),
            impulsivity: Fixed::from_f64(0.5),
            temperament: Temperament::default(),
        }
    }

    fn signals(identity_integration: Fixed, age_years: Fixed) -> PlasticitySignals {
        PlasticitySignals {
            repeated_stress: Fixed::from_f64(0.8),
            recovery: Fixed::from_f64(0.2),
            social_engagement: Fixed::from_f64(0.5),
            goal_striving: Fixed::from_f64(0.5),
            identity_integration,
            age_years,
        }
    }

    #[test]
    fn temperament_derivation_is_deterministic_and_bounded() {
        let a = base_personality();
        let b = base_personality();
        let ta = Temperament::from_traits(&a);
        let tb = Temperament::from_traits(&b);
        assert_eq!(ta, tb, "same traits must derive the same temperament");
        // All seven dimensions strictly inside (0, 1) for mid traits.
        assert!(ta.reactivity > Fixed::ZERO && ta.reactivity < Fixed::ONE);
        assert!(ta.soothability > Fixed::ZERO && ta.soothability < Fixed::ONE);
        assert!(ta.sociability > Fixed::ZERO && ta.sociability < Fixed::ONE);
        assert!(ta.persistence > Fixed::ZERO && ta.persistence < Fixed::ONE);
        assert!(ta.sensitivity > Fixed::ZERO && ta.sensitivity < Fixed::ONE);
        assert!(ta.regularity > Fixed::ZERO && ta.regularity < Fixed::ONE);
        assert!(ta.approach_withdrawal > Fixed::ZERO && ta.approach_withdrawal < Fixed::ONE);
    }

    #[test]
    fn temperament_reactivity_follows_neuroticism_and_impulsivity() {
        let low = base_personality();
        let mut high = base_personality();
        high.neuroticism = Fixed::ONE;
        high.impulsivity = Fixed::ONE;
        let tl = Temperament::from_traits(&low);
        let th = Temperament::from_traits(&high);
        assert!(
            th.reactivity > tl.reactivity,
            "reactivity must rise with neuroticism/impulsivity"
        );
    }

    #[test]
    fn plastic_update_preserves_core_traits_and_builds_reactivity_under_stress() {
        let p = base_personality();
        let before = p.clone();
        let baseline = Temperament::from_traits(&p);
        let mut t = baseline;
        let r0 = t.reactivity;
        let s = signals(Fixed::ONE, Fixed::from_f64(20.0));
        for _ in 0..500 {
            t.plastic_update(&baseline, &s);
        }
        assert!(
            t.reactivity > r0,
            "repeated stress must build reactivity over time"
        );
        // The 12 core traits are untouched by construction — assert the invariant.
        assert_eq!(p.openness, before.openness);
        assert_eq!(p.conscientiousness, before.conscientiousness);
        assert_eq!(p.extraversion, before.extraversion);
        assert_eq!(p.agreeableness, before.agreeableness);
        assert_eq!(p.neuroticism, before.neuroticism);
        assert_eq!(p.risk_tolerance, before.risk_tolerance);
        assert_eq!(p.conformity, before.conformity);
        assert_eq!(p.ambition, before.ambition);
        assert_eq!(p.altruism, before.altruism);
        assert_eq!(p.traditionalism, before.traditionalism);
        assert_eq!(p.dominance, before.dominance);
        assert_eq!(p.impulsivity, before.impulsivity);
        // Temperament stays bounded.
        assert!(t.reactivity <= Fixed::ONE && t.sociability <= Fixed::ONE);
    }

    #[test]
    fn plastic_update_is_inert_without_identity_integration_or_in_old_age() {
        let p = base_personality();
        let baseline = Temperament::from_traits(&p);
        let mut t = baseline;
        let snapshot = t;
        // Zero identity integration → zero rate, no matter the stress.
        let quiet = signals(Fixed::ZERO, Fixed::from_f64(20.0));
        t.plastic_update(&baseline, &quiet);
        assert_eq!(t, snapshot, "no identity integration → no plasticity");
        // Maximum age → developmental plasticity = 0.
        let old = signals(Fixed::ONE, Fixed::from_f64(70.0));
        t.plastic_update(&baseline, &old);
        assert_eq!(t, snapshot, "old age → no plasticity");
    }

    #[test]
    fn plastic_update_delta_scales_with_developmental_plasticity() {
        let p = base_personality();
        let baseline = Temperament::from_traits(&p);
        let mut young = baseline;
        let mut elderly = baseline;
        let s_young = signals(Fixed::ONE, Fixed::from_f64(20.0));
        // Age 55 (dev = 0.2143) keeps the elderly rate above the Fixed
        // sub-resolution tail — both sides must move for the comparison to be
        // meaningful (at 60+ the elderly rate truncates to zero entirely).
        let s_old = signals(Fixed::ONE, Fixed::from_f64(55.0));
        for _ in 0..100 {
            young.plastic_update(&baseline, &s_young);
            elderly.plastic_update(&baseline, &s_old);
        }
        assert!(
            young.reactivity > elderly.reactivity,
            "younger agents must show more trait plasticity"
        );
    }

    #[test]
    fn reactivity_amplifier_is_identity_at_zero_and_bounded() {
        // §8.1.6 (Iteration 105): zero deviation → amplifier 1.0 exactly —
        // the appraise fold is byte-identical when the plasticity pass has
        // not yet drifted the agent (the zero-at-zero contract).
        assert_eq!(Temperament::reactivity_amplifier(Fixed::ZERO), Fixed::ONE);
        // Positive deviation amplifies; negative damps — monotone.
        let up = Temperament::reactivity_amplifier(Fixed::from_f64(0.2));
        let down = Temperament::reactivity_amplifier(Fixed::from_f64(-0.2));
        assert!(up > Fixed::ONE, "stressed reactivity must amplify the response");
        assert!(down < Fixed::ONE, "below-baseline reactivity must damp the response");
        assert!(up > down, "the amplifier must be monotone in the deviation");
        // Bounded: a 0.5 deviation saturates at 1.5×; −1.0 at 0.5×.
        assert_eq!(
            Temperament::reactivity_amplifier(Fixed::from_f64(0.5)),
            Fixed::from_f64(1.5),
            "amplifier must clamp at the 1.5 upper bound"
        );
        assert_eq!(
            Temperament::reactivity_amplifier(Fixed::from_f64(-2.0)),
            Fixed::from_f64(0.5),
            "amplifier must clamp at the 0.5 lower bound"
        );
        // Deterministic: same deviation reproduces the same amplifier.
        assert_eq!(
            Temperament::reactivity_amplifier(Fixed::from_f64(0.2)),
            up,
            "the amplifier must be deterministic"
        );
    }
}
