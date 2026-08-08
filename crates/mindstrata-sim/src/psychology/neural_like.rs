//! §9.2 (AP2): Deterministic "neural-like" implementation.
//!
//! Lightweight deterministic mechanisms that give agents associative,
//! predictive, and scripted cognition without any nondeterminism:
//!
//! - [`ConceptVector`] — the plan's 12-dimensional fixed-point concept
//!   embeddings with cosine similarity (`dot / (norm × norm)`);
//! - [`AssociationNetwork`] — spreading activation across a fixed 12×12
//!   association matrix (the plan's associative-thought formula);
//! - [`PredictiveExpectation`] — expectation tracking with prediction-error
//!   signals (the plan's predictive-error mechanic);
//! - [`ActionValues`] — the plan's reinforcement-learning value
//!   decomposition (need relief + emotional relief + social reward +
//!   identity congruence − cost − risk), EMA-learned from outcomes;
//! - [`BehaviorScript`] — the plan's script grammar (the CourtshipScript
//!   worked example) for LLM-like generativity without nondeterminism.
//!
//! Wired as **observational state**: activations, prediction errors, and
//! learned values accumulate but no decision system reads them yet (decisional
//! consumers are future behavioral iterations that will re-calibrate the
//! golden baseline), so calibrated runs stay byte-identical. All arithmetic is
//! Fixed-domain; cosine similarity uses a deterministic integer square root.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// The plan's 12 concept dimensions (fixed order — determinism).
pub const CONCEPT_DIMENSIONS: usize = 12;

pub const D_SAFETY: usize = 0;
pub const D_SACREDNESS: usize = 1;
pub const D_STATUS: usize = 2;
pub const D_KINSHIP: usize = 3;
pub const D_SCARCITY: usize = 4;
pub const D_THREAT: usize = 5;
pub const D_PURITY: usize = 6;
pub const D_FREEDOM: usize = 7;
pub const D_LOYALTY: usize = 8;
pub const D_PLEASURE: usize = 9;
pub const D_SHAME: usize = 10;
pub const D_HOPE: usize = 11;

/// §9.2: The plan's 12-dimensional concept embedding (fixed-point vector).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConceptVector {
    pub safety: Fixed,
    pub sacredness: Fixed,
    pub status: Fixed,
    pub kinship: Fixed,
    pub scarcity: Fixed,
    pub threat: Fixed,
    pub purity: Fixed,
    pub freedom: Fixed,
    pub loyalty: Fixed,
    pub pleasure: Fixed,
    pub shame: Fixed,
    pub hope: Fixed,
}

impl ConceptVector {
    /// The dimensions in the fixed [`CONCEPT_DIMENSIONS`] order.
    pub fn components(&self) -> [Fixed; CONCEPT_DIMENSIONS] {
        [
            self.safety,
            self.sacredness,
            self.status,
            self.kinship,
            self.scarcity,
            self.threat,
            self.purity,
            self.freedom,
            self.loyalty,
            self.pleasure,
            self.shame,
            self.hope,
        ]
    }

    /// Build a vector from the fixed [`CONCEPT_DIMENSIONS`] order.
    pub fn from_components(components: [Fixed; CONCEPT_DIMENSIONS]) -> Self {
        Self {
            safety: components[D_SAFETY],
            sacredness: components[D_SACREDNESS],
            status: components[D_STATUS],
            kinship: components[D_KINSHIP],
            scarcity: components[D_SCARCITY],
            threat: components[D_THREAT],
            purity: components[D_PURITY],
            freedom: components[D_FREEDOM],
            loyalty: components[D_LOYALTY],
            pleasure: components[D_PLEASURE],
            shame: components[D_SHAME],
            hope: components[D_HOPE],
        }
    }

    /// Dot product (Fixed-domain, deterministic).
    pub fn dot(&self, other: &Self) -> Fixed {
        self.components()
            .iter()
            .zip(other.components().iter())
            .fold(Fixed::ZERO, |acc, (a, b)| acc + *a * *b)
    }

    /// §9.2 cosine similarity: `dot(a, b) / (norm(a) × norm(b))`.
    /// `None` when either vector is the zero vector (undefined).
    pub fn cosine_similarity(&self, other: &Self) -> Option<Fixed> {
        let na = fixed_sqrt(self.dot(self));
        let nb = fixed_sqrt(other.dot(other));
        if na <= Fixed::ZERO || nb <= Fixed::ZERO {
            return None;
        }
        Some((self.dot(other) / (na * nb)).clamp(-Fixed::ONE, Fixed::ONE))
    }
}

/// Deterministic integer square root (Newton's method, i64 domain).
fn isqrt(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }
    let mut x = n;
    // Overflow-safe midpoint of (x, 1): (x + 1) / 2 without the addition.
    let mut y = x / 2 + (x % 2 + 1) / 2;
    while y < x {
        x = y;
        // x and n/x are both ≤ n ≤ Fixed raw-domain bounds (~10^8 for the
        // squared scale), so x + n/x cannot overflow i64 here.
        y = (x + n / x) / 2;
    }
    x
}

/// Deterministic fixed-point square root.
///
/// `sqrt(raw / SCALE) = sqrt(raw × SCALE) / SCALE`, so the integer root is
/// taken on the re-scaled value. Converges exactly for perfect squares and is
/// deterministic (fixed Newton iterations) for all others.
fn fixed_sqrt(v: Fixed) -> Fixed {
    let scale = Fixed::ONE.to_raw();
    Fixed::from_raw(isqrt(v.to_raw().saturating_mul(scale)))
}

/// §9.2: Spreading activation across the 12 concept dimensions.
///
/// Each `spread` step (the plan's formula, plus a natural retention decay so
/// associative memory fades between perceptions):
/// ```text
/// activation(d) = activation(d) × retention
/// activation(d) += input(d)
/// activation(t) += activation(s) × edge_weight(s,t) × decay   ∀ edges
/// ```
/// Deterministic, no RNG; every dimension is clamped to 0..1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationNetwork {
    /// Fixed 12×12 symmetric association weights.
    weights: [[Fixed; CONCEPT_DIMENSIONS]; CONCEPT_DIMENSIONS],
    /// Current activation per dimension (observational).
    pub activation: ConceptVector,
}

impl Default for AssociationNetwork {
    fn default() -> Self {
        // Seeded deterministic associations (symmetric): threat↔shame,
        // threat↔safety, scarcity↔threat, sacredness↔purity, kinship↔loyalty,
        // status↔freedom, pleasure↔hope.
        let mut weights = [[Fixed::ZERO; CONCEPT_DIMENSIONS]; CONCEPT_DIMENSIONS];
        let mut link = |a: usize, b: usize, v: f64| {
            let f = Fixed::from_f64(v);
            weights[a][b] = f;
            weights[b][a] = f;
        };
        link(D_THREAT, D_SHAME, 0.3);
        link(D_THREAT, D_SAFETY, 0.25);
        link(D_SCARCITY, D_THREAT, 0.3);
        link(D_SACREDNESS, D_PURITY, 0.35);
        link(D_KINSHIP, D_LOYALTY, 0.3);
        link(D_STATUS, D_FREEDOM, 0.2);
        link(D_PLEASURE, D_HOPE, 0.25);
        Self {
            weights,
            activation: ConceptVector::default(),
        }
    }
}

impl AssociationNetwork {
    /// §9.2 spreading activation — one deterministic propagation step.
    pub fn spread(&mut self, input: ConceptVector, decay: Fixed) {
        let input_components = input.components();
        let mut act = self.activation.components();
        // Retention: associative memory fades 10%/step toward baseline.
        let retention = Fixed::from_f64(0.9);
        for a in &mut act {
            *a = (*a * retention).clamp_01();
        }
        // Perception adds input strength to each node.
        for d in 0..CONCEPT_DIMENSIONS {
            act[d] = (act[d] + input_components[d]).clamp_01();
        }
        // Associations propagate: activation(target) += activation(source) × edge × decay.
        let src = act;
        for (s, row) in self.weights.iter().enumerate() {
            for (t, w) in row.iter().enumerate() {
                if *w > Fixed::ZERO {
                    act[t] = (act[t] + src[s] * *w * decay).clamp_01();
                }
            }
        }
        self.activation = ConceptVector::from_components(act);
    }
}

/// §9.2: Predictive error — agents maintain an expectation and register the
/// magnitude of surprise. Large errors are the plan's hook for attention,
/// belief updates, and emotional intensity (decisional consumers deferred).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PredictiveExpectation {
    /// EMA of observed outcomes (0..1).
    pub expectation: Fixed,
    /// EMA learning rate.
    pub learning_rate: Fixed,
    /// |observed − expected| from the last observation.
    pub last_prediction_error: Fixed,
}

impl Default for PredictiveExpectation {
    fn default() -> Self {
        Self {
            expectation: Fixed::from_f64(0.5),
            learning_rate: Fixed::from_f64(0.05),
            last_prediction_error: Fixed::ZERO,
        }
    }
}

impl PredictiveExpectation {
    /// Observe an outcome (0..1): returns the signed prediction error and
    /// EMA-updates the expectation toward the outcome.
    pub fn observe(&mut self, outcome: Fixed) -> Fixed {
        let error = (outcome - self.expectation).abs();
        self.expectation = (self.expectation
            + (outcome - self.expectation) * self.learning_rate)
            .clamp_01();
        self.last_prediction_error = error.clamp_01();
        error
    }
}

/// §9.2: Reinforcement-learning action values.
///
/// The plan decomposes
/// ```text
/// action_value = expected need relief + emotional relief + social reward
///              + identity congruence − cost − risk
/// ```
/// The four learned components are EMA-updated from successful outcomes;
/// cost/risk are evaluated per context by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ActionValues {
    pub need_relief: Fixed,
    pub emotional_relief: Fixed,
    pub social_reward: Fixed,
    pub identity_congruence: Fixed,
    pub learning_rate: Fixed,
}

impl Default for ActionValues {
    fn default() -> Self {
        Self {
            need_relief: Fixed::from_f64(0.5),
            emotional_relief: Fixed::from_f64(0.5),
            social_reward: Fixed::from_f64(0.5),
            identity_congruence: Fixed::from_f64(0.5),
            learning_rate: Fixed::from_f64(0.05),
        }
    }
}

impl ActionValues {
    /// Evaluate the action value for a context (cost/risk provided by caller).
    pub fn value(&self, cost: Fixed, risk: Fixed) -> Fixed {
        (self.need_relief
            + self.emotional_relief
            + self.social_reward
            + self.identity_congruence
            - cost
            - risk)
            .clamp(-Fixed::ONE, Fixed::ONE)
    }

    /// §9.2 (Iteration 94): the *learned* valuation delta for an action's
    /// outcome profile `(need, emotional, social, identity)` —
    /// `dot(learned_components, profile) − dot(neutral_prior, profile)`.
    ///
    /// The four components start at the 0.5 neutral prior, so this is
    /// **exactly zero at tick 0**: the consumer is inert until outcomes have
    /// actually been experienced, which bounds the golden-baseline
    /// perturbation to the learned signal only (no constant offset). As
    /// learning differentiates the components (EMA toward the relief
    /// profile of successful actions), the delta becomes a *relative*
    /// preference signal — actions whose relief profile matches what the
    /// agent has learned to value are favored over mismatched ones.
    /// Deterministic, no RNG.
    pub fn learned_delta(&self, profile: [Fixed; 4]) -> Fixed {
        let prior = Fixed::from_f64(0.5);
        let sum = profile[0] + profile[1] + profile[2] + profile[3];
        if sum == Fixed::ZERO {
            return Fixed::ZERO;
        }
        // The profile is normalized by its own sum so every candidate's
        // baseline is the uniform 0.5 prior. Without this, actions with
        // larger profile sums are systematically penalized (every learned
        // component EMA-converges *downward* from the 0.5 prior toward its
        // ≤0.5 profile component, so all deltas are negative and the raw
        // 0.5·sum baseline rewards low-sum default actions like Rest
        // regardless of what the agent actually learned). With
        // normalization the signal is per-component —
        // `dot(learned − prior, profile/sum)` — so an agent that learned
        // need-relief favors need-heavy profiles (Eat/Work) over idle and
        // mismatched ones.
        let learned = self.need_relief * profile[0]
            + self.emotional_relief * profile[1]
            + self.social_reward * profile[2]
            + self.identity_congruence * profile[3];
        let baseline = prior * sum;
        (learned - baseline) / sum
    }

    /// EMA-update the learned components toward the experienced relief
    /// components of a successful outcome (failures leave values unchanged —
    /// "values update from outcomes", deterministic).
    pub fn learn_from_outcome(
        &mut self,
        success: bool,
        need: Fixed,
        emotional: Fixed,
        social: Fixed,
        identity: Fixed,
    ) {
        if !success {
            return;
        }
        let alpha = self.learning_rate;
        self.need_relief = (self.need_relief + (need - self.need_relief) * alpha).clamp_01();
        self.emotional_relief =
            (self.emotional_relief + (emotional - self.emotional_relief) * alpha).clamp_01();
        self.social_reward = (self.social_reward + (social - self.social_reward) * alpha).clamp_01();
        self.identity_congruence =
            (self.identity_congruence + (identity - self.identity_congruence) * alpha).clamp_01();
    }
}

/// §9.2: Script grammar — LLM-like behavioral generativity without
/// nondeterminism. The plan's CourtshipScript is the worked example.
///
/// `String` steps (not `&'static str`) so the struct stays serde-friendly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehaviorScript {
    pub name: String,
    pub steps: Vec<String>,
    pub current_step: usize,
}

impl BehaviorScript {
    /// The plan's §9.2 CourtshipScript, verbatim step order.
    pub fn courtship() -> Self {
        Self {
            name: "courtship".to_string(),
            steps: vec![
                "notice",
                "appraise",
                "signal_interest",
                "seek_proximity",
                "test_reciprocity",
                "gossip_validation",
                "propose_bond",
                "seek_social_approval",
                "formalize",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            current_step: 0,
        }
    }

    /// Return the next script step, advancing the pointer (`None` when done).
    pub fn next_step(&mut self) -> Option<String> {
        if self.current_step < self.steps.len() {
            let step = self.steps[self.current_step].clone();
            self.current_step += 1;
            Some(step)
        } else {
            None
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current_step >= self.steps.len()
    }
}

/// §9.2: The agent's neural-like runtime — associative network, predictive
/// expectation, learned action values, and an active behavioral script.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NeuralLikeState {
    pub network: AssociationNetwork,
    pub expectation: PredictiveExpectation,
    pub values: ActionValues,
    pub script: Option<BehaviorScript>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_sqrt_matches_known_values() {
        assert_eq!(fixed_sqrt(Fixed::ONE), Fixed::ONE, "sqrt(1.0) == 1.0");
        assert_eq!(fixed_sqrt(Fixed::from_f64(0.25)), Fixed::from_f64(0.5), "sqrt(0.25) == 0.5");
        assert_eq!(fixed_sqrt(Fixed::from_f64(4.0)), Fixed::from_f64(2.0), "sqrt(4.0) == 2.0");
        assert_eq!(fixed_sqrt(Fixed::ZERO), Fixed::ZERO, "sqrt(0.0) == 0.0");
    }

    #[test]
    fn concept_cosine_similarity_is_deterministic_and_correct() {
        let a = ConceptVector {
            safety: Fixed::ONE,
            ..ConceptVector::default()
        };
        let b = ConceptVector {
            safety: Fixed::ONE,
            ..ConceptVector::default()
        };
        let s1 = a.cosine_similarity(&b).expect("identical nonzero vectors");
        assert_eq!(s1, Fixed::ONE, "identical vectors are maximally similar");
        assert_eq!(a.cosine_similarity(&b), a.cosine_similarity(&b), "deterministic");

        let c = ConceptVector {
            threat: Fixed::ONE,
            ..ConceptVector::default()
        };
        assert_eq!(
            a.cosine_similarity(&c).expect("nonzero"),
            Fixed::ZERO,
            "orthogonal dimensions are dissimilar"
        );
        assert_eq!(ConceptVector::default().cosine_similarity(&a), None, "zero vector → None");
    }

    #[test]
    fn spreading_activation_propagates_through_associations() {
        let mut net = AssociationNetwork::default();
        let mut net2 = net.clone();
        let perceived = ConceptVector {
            threat: Fixed::ONE,
            ..ConceptVector::default()
        };
        net.spread(perceived, Fixed::from_f64(0.8));
        net2.spread(perceived, Fixed::from_f64(0.8));
        // Deterministic — identical inputs, identical activation states.
        assert_eq!(net.activation, net2.activation, "spread is deterministic");
        // threat → shame (edge 0.3), threat → safety (edge 0.25), and
        // threat → scarcity (edge 0.3) propagate on the first step.
        assert!(net.activation.shame > Fixed::ZERO, "threat associates to shame");
        assert!(net.activation.safety > Fixed::ZERO, "threat associates to safety");
        assert!(net.activation.scarcity > Fixed::ZERO, "threat associates to scarcity");
        assert!(net.activation.shame <= Fixed::ONE && net.activation.threat <= Fixed::ONE);
    }

    #[test]
    fn prediction_error_tracks_surprise_and_learns() {
        let mut p = PredictiveExpectation::default();
        assert_eq!(p.expectation, Fixed::from_f64(0.5));
        let e1 = p.observe(Fixed::from_f64(0.9));
        assert!(e1 > Fixed::from_f64(0.3), "first surprise is large, got {e1:?}");
        assert!(p.expectation > Fixed::from_f64(0.5), "expectation rises toward the outcome");
        // Observing the same outcome again shrinks the error (learning).
        let e2 = p.observe(Fixed::from_f64(0.9));
        assert!(e2 < e1, "repeated outcomes shrink prediction error");
        assert_eq!(p.last_prediction_error, e2);
    }

    #[test]
    fn action_values_learn_only_from_success() {
        let mut v = ActionValues::default();
        // Cost/risk 0.8 keep the value inside (−1, 1) — high default components
        // (4 × 0.5) would otherwise saturate both sides at the 1.0 clamp.
        let before = v.value(Fixed::from_f64(0.8), Fixed::from_f64(0.8));
        v.learn_from_outcome(false, Fixed::ONE, Fixed::ONE, Fixed::ONE, Fixed::ONE);
        assert_eq!(v.value(Fixed::from_f64(0.8), Fixed::from_f64(0.8)), before, "failure: no learning");
        v.learn_from_outcome(true, Fixed::ONE, Fixed::ONE, Fixed::ONE, Fixed::ONE);
        let after = v.value(Fixed::from_f64(0.8), Fixed::from_f64(0.8));
        assert!(after > before, "successful outcomes raise learned value");
    }

    #[test]
    fn learned_delta_is_zero_at_prior_and_tracks_learning() {
        // §9.2 (Iteration 94): the selection bias is exactly zero at the
        // neutral prior (tick-0 inertness — no constant offset), and becomes
        // a relative preference signal as components learn toward the
        // outcome profile of successful actions.
        let v = ActionValues::default();
        // Work profile: (need 0.4, emotional 0, social 0, identity 0.1)
        let work = [Fixed::from_f64(0.4), Fixed::ZERO, Fixed::ZERO, Fixed::from_f64(0.1)];
        assert_eq!(
            v.learned_delta(work),
            Fixed::ZERO,
            "at the neutral prior the delta must be exactly zero"
        );

        // An agent that repeatedly succeeds at work converges its components
        // toward the work profile, so the work delta rises relative to a
        // mismatched (social-heavy) profile — the relative signal is what
        // shifts the selection argmax (both profiles share the same baseline
        // 0.5 sum, so the baseline terms cancel in the comparison).
        let mut learned = ActionValues::default();
        for _ in 0..60 {
            learned.learn_from_outcome(true, Fixed::from_f64(0.4), Fixed::ZERO, Fixed::ZERO, Fixed::from_f64(0.1));
        }
        let work_delta = learned.learned_delta(work);
        // Socialize profile: (need 0, emotional 0.1, social 0.4, identity 0)
        let socialize =
            [Fixed::ZERO, Fixed::from_f64(0.1), Fixed::from_f64(0.4), Fixed::ZERO];
        let socialize_delta = learned.learned_delta(socialize);
        assert!(
            work_delta > socialize_delta,
            "work-learner must favor the work profile over the mismatched one: work={work_delta:?} socialize={socialize_delta:?}"
        );

        // And a symmetric control: a socialize-learner flips the preference.
        let mut social_learner = ActionValues::default();
        for _ in 0..60 {
            social_learner.learn_from_outcome(true, Fixed::ZERO, Fixed::from_f64(0.1), Fixed::from_f64(0.4), Fixed::ZERO);
        }
        assert!(
            social_learner.learned_delta(socialize) > social_learner.learned_delta(work),
            "socialize-learner must favor the socialize profile"
        );
    }

    #[test]
    fn courtship_script_advances_in_order_then_completes() {
        let mut s = BehaviorScript::courtship();
        assert!(!s.is_complete());
        assert_eq!(s.next_step(), Some("notice".to_string()));
        assert_eq!(s.next_step(), Some("appraise".to_string()));
        // Skip to the end deterministically.
        while s.next_step().is_some() {}
        assert!(s.is_complete(), "script completes after the last step");
        assert_eq!(s.next_step(), None, "no steps after completion");
    }
}
