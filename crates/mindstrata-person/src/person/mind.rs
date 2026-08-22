//! Beliefs, goals, identity, moral values, cognition, intentions.

use mindstrata_core::fixed::Fixed;
use rand::Rng;
use serde::{Deserialize, Serialize};

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
    /// Proposition id. (doc added at S2 extraction)
    pub proposition_id: u64,
    /// Confidence. (doc added at S2 extraction)
    pub confidence: Fixed,
    /// Emotional charge. (doc added at S2 extraction)
    pub emotional_charge: Fixed,
    /// Identity linkage. (doc added at S2 extraction)
    pub identity_linkage: Fixed,
    /// Resistance. (doc added at S2 extraction)
    pub resistance: Fixed,
    /// Last reinforced tick. (doc added at S2 extraction)
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
    /// §5 (Iteration 155): An external directive — the interactive TUI's
    /// command channel injected this goal between ticks.
    Command,
}

/// Goal priority and source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    /// Kind. (doc added at S2 extraction)
    pub kind: GoalKind,
    /// Priority. (doc added at S2 extraction)
    pub priority: Fixed,
    /// Commitment. (doc added at S2 extraction)
    pub commitment: Fixed,
    /// Created tick. (doc added at S2 extraction)
    pub created_tick: u64,
    /// §24: What triggered this goal — affects abandonment thresholds.
    pub source: GoalSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Enum. (doc added at S2 extraction)
pub enum GoalKind {
    /// Eat. (doc added at S2 extraction)
    Eat,
    /// Drink. (doc added at S2 extraction)
    Drink,
    /// Rest. (doc added at S2 extraction)
    Rest,
    /// Work. (doc added at S2 extraction)
    Work,
    /// Socialize. (doc added at S2 extraction)
    Socialize,
    /// Worship. (doc added at S2 extraction)
    Worship,
    /// SeekSafety. (doc added at S2 extraction)
    SeekSafety,
}

// ── Identity system ──────────────────────────────────────────────────────

/// Kinds of personal identity an agent can hold.
/// Only identities that affect action utility are included (YAGNI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdentityKind {
    /// Farmer. (doc added at S2 extraction)
    Farmer,
    /// Parent. (doc added at S2 extraction)
    Parent,
    /// Believer. (doc added at S2 extraction)
    Believer,
}

/// An agent's identity with its strength (0.0–1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    /// Kind. (doc added at S2 extraction)
    pub kind: IdentityKind,
    /// Strength. (doc added at S2 extraction)
    pub strength: Fixed,
}

/// Identity state for an agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdentityState {
    /// Identities. (doc added at S2 extraction)
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

    /// Iteration 246 (Arc A heredity): vertical cultural transmission.
    /// Each foundation blends the mid-parent value (70%) with the village's
    /// living-community prior (30%) plus ±0.08 enculturation noise —
    /// children resemble their parents and their neighbors more than
    /// chance, while the population keeps drifting. Consumes exactly six
    /// RNG draws (one per foundation), matching `random`'s stream contract
    /// so downstream alignment is preserved.
    pub fn inherit(
        parent_a: &Self,
        parent_b: Option<&Self>,
        community_prior: &Self,
        rng: &mut impl Rng,
    ) -> Self {
        let mut blend = |a: Fixed, b: Option<Fixed>, c: Fixed| -> Fixed {
            let mid = b.map_or(a, |b| (a + b) * Fixed::from_f64(0.5));
            let shaped = mid * Fixed::from_f64(0.7) + c * Fixed::from_f64(0.3);
            (shaped + Fixed::from_f64(rng.random_range(-0.08..0.08))).clamp_01()
        };
        Self {
            care: blend(
                parent_a.care,
                parent_b.map(|p| p.care),
                community_prior.care,
            ),
            fairness: blend(
                parent_a.fairness,
                parent_b.map(|p| p.fairness),
                community_prior.fairness,
            ),
            loyalty: blend(
                parent_a.loyalty,
                parent_b.map(|p| p.loyalty),
                community_prior.loyalty,
            ),
            authority: blend(
                parent_a.authority,
                parent_b.map(|p| p.authority),
                community_prior.authority,
            ),
            purity: blend(
                parent_a.purity,
                parent_b.map(|p| p.purity),
                community_prior.purity,
            ),
            liberty: blend(
                parent_a.liberty,
                parent_b.map(|p| p.liberty),
                community_prior.liberty,
            ),
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
        self.stress =
            (self.stress * Fixed::from_f64(0.9) + stress * Fixed::from_f64(0.1)).clamp_01();
        self.fatigue =
            (self.fatigue * Fixed::from_f64(0.95) + fatigue * Fixed::from_f64(0.05)).clamp_01();

        // Stress reduces effective planning horizon
        let stress_factor = Fixed::ONE - self.stress;
        let fatigue_factor = Fixed::ONE - self.fatigue * Fixed::from_f64(0.3);
        self.planning_horizon = (20.0 * stress_factor.to_f64() * fatigue_factor.to_f64()) as u32;

        // Stress increases heuristic reliance
        self.heuristic_bias = (self.stress * Fixed::from_f64(0.6)
            + self.fatigue * Fixed::from_f64(0.2)
            + Fixed::from_f64(0.2))
        .clamp_01();
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
    /// Stress. (doc added at S2 extraction)
    pub stress: Fixed,
    /// Coping potential. (doc added at S2 extraction)
    pub coping_potential: Fixed,
    /// Social support. (doc added at S2 extraction)
    pub social_support: Fixed,
    /// Need deficit avg. (doc added at S2 extraction)
    pub need_deficit_avg: Fixed,
    /// Meaning. (doc added at S2 extraction)
    pub meaning: Fixed,
    /// Autonomy. (doc added at S2 extraction)
    pub autonomy: Fixed,
    /// Success rate. (doc added at S2 extraction)
    pub success_rate: Fixed,
    /// Neuroticism. (doc added at S2 extraction)
    pub neuroticism: Fixed,
    /// Justice perception. (doc added at S2 extraction)
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
        self.depression_risk =
            (self.depression_risk * smoothing + depression_delta * accumulation).clamp_01();

        // Ambition: from success and energy
        self.ambition =
            (input.success_rate * Fixed::from_f64(0.4) + Fixed::from_f64(0.3)).clamp_01();

        // Resilience: from social support and coping
        self.resilience = (input.social_support * Fixed::from_f64(0.3)
            + input.coping_potential * Fixed::from_f64(0.3)
            + Fixed::from_f64(0.2))
        .clamp_01();

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
    pub fn new(
        goal_kind: GoalKind,
        formed_tick: u64,
        personality_conscientiousness: Fixed,
    ) -> Self {
        // Conscientious agents form stronger commitments
        let base_commitment =
            Fixed::from_f64(0.5) + personality_conscientiousness * Fixed::from_f64(0.3);
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
    pub fn should_abandon(&self, current_tick: u64, stress: Fixed, emotional_shock: bool) -> bool {
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

#[cfg(test)]
mod moral_inherit_tests {
    use super::MoralValues;
    use crate::person::FIRST_NAMES;
    use mindstrata_core::fixed::Fixed;
    use rand::{Rng, SeedableRng};

    #[test]
    fn values_transmit_vertically() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        let mut a = MoralValues::random(&mut rng);
        let mut b = MoralValues::random(&mut rng);
        // Extreme parents for a strong signal.
        a.care = Fixed::from_f64(0.9);
        b.care = Fixed::from_f64(0.8);
        let prior = MoralValues::default(); // 0.5 everywhere
        let child = MoralValues::inherit(&a, Some(&b), &prior, &mut rng);
        // Child care near the mid-parent (0.85 shaped toward 0.5 by 30%):
        // expected ~0.745 ± noise; must be FAR from either stranger draw.
        let c = child.care.to_f64();
        assert!(
            (0.60..0.87).contains(&c),
            "child care {c} should sit between mid-parent and community prior"
        );
        let stranger = MoralValues::random(&mut rng).care.to_f64();
        assert!(
            (child.care.to_f64() - 0.745).abs() <= (stranger - 0.745).abs() + 0.15,
            "parental blend should beat a random stranger draw on average"
        );
    }

    #[test]
    fn single_parent_path_works() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(9);
        let mut a = MoralValues::random(&mut rng);
        a.loyalty = Fixed::from_f64(0.95);
        let prior = MoralValues::default();
        let child = MoralValues::inherit(&a, None, &prior, &mut rng);
        // Sole parent 0.95 * 0.7 + 0.5 * 0.3 = 0.815 ± 0.08
        let l = child.loyalty.to_f64();
        assert!(
            (0.70..0.92).contains(&l),
            "sole-parent loyalty {l} out of band"
        );
        //
        // NOTE on RNG streams: `random` and `inherit` both consume six
        // draws but over DIFFERENT widths (e.g. 0.2..0.9 vs -0.08..0.08),
        // so byte-level stream alignment between them is impossible — and
        // irrelevant: `moral_values` is the LAST rng-consuming initializer
        // in both birth constructors (everything after is
        // `Default::default()`), so no downstream consumer can observe the
        // difference.
    }

    #[test]
    fn first_names_pool_is_stable() {
        assert_eq!(FIRST_NAMES.len(), 24);
        assert!(FIRST_NAMES.iter().all(|n| !n.contains(' ')));
    }
}
