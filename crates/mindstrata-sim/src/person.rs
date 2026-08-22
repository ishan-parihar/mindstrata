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

/// §8.1.6 (Iteration 179): per-tick rate for the observational Temperament
/// layer's plasticity (see `Temperament::plastic_update`).
pub const TEMPERAMENT_PLASTICITY_RATE: f64 = 0.0005;

/// §8.1.6 (Iteration 179): per-tick rate for the 12 core traits' plasticity
/// (see `Personality::plastic_update_traits`). Set EQUAL to the temperament
/// rate because the 4-decimal Fixed resolution floors anything smaller to
/// inertness: at 0.0002 the compound rate (development × integration ×
/// reinforcement ≈ 0.43–0.82 at typical signals) truncates to 1 raw, and
/// then `stress × rate` (0.8 × 0.0001) truncates to zero — the stress
/// pathway never moves. At 0.0005 the compound lands at 2–4 raw and the
/// stress/product terms stay representable (same proven floor as the
/// temperament layer). Effective core-trait movement is STILL slower than
/// the observational layer in practice because the constitution pull term
/// re-anchors each trait to its birth value, damping net drift.
pub const CORE_TRAIT_PLASTICITY_RATE: f64 = 0.0005;

/// §8.1.6 (Iteration 179): The agent's birth-trait constitution — the stable
/// anchor for core-trait plasticity. Captured at construction (`random()`)
/// before any life experience, so `plastic_update_traits` can push each trait
/// toward its repeatedly-expressed state and pull it back toward this
/// constitution when the signal fades (the plan's "traits slowly change
/// through repeated behavior, trauma, success, failure..." formula applied to
/// the 12 decision-read traits). `#[serde(default)]` on the `Personality`
/// field keeps old snapshots valid — a restored snapshot lazily anchors to
/// its current traits on the first plasticity tick.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TraitConstitution {
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
}

impl TraitConstitution {
    /// Capture the current trait values as the birth constitution.
    pub fn from_personality(p: &Personality) -> Self {
        Self {
            openness: p.openness,
            conscientiousness: p.conscientiousness,
            extraversion: p.extraversion,
            agreeableness: p.agreeableness,
            neuroticism: p.neuroticism,
            risk_tolerance: p.risk_tolerance,
            conformity: p.conformity,
            ambition: p.ambition,
            altruism: p.altruism,
            traditionalism: p.traditionalism,
            dominance: p.dominance,
            impulsivity: p.impulsivity,
        }
    }
}

/// Founder given names (Iteration 245): shared by `populate` and the
/// birth paths so every villager — founder or newborn — draws from one
/// name pool. Newborns append the family surname; founders are single-
/// token roots.
pub const FIRST_NAMES: [&str; 24] = [
    "Anna", "Bran", "Cara", "Dane", "Elise", "Finn", "Greta", "Hans", "Ines", "Jorik", "Kira",
    "Lars", "Mira", "Nils", "Opal", "Poul", "Quinn", "Rosa", "Sven", "Tova", "Ulf", "Vera", "Wulf",
    "Xena",
];

/// Iteration 245 (Arc A heredity): derive a child's full name from a
/// parent's name and a given name. The surname is the parent's family
/// token — everything after their first space — so second-generation
/// children keep the founding line's surname. A founder parent (single
/// token) lends their own given name as the founding family name, which
/// reads as a patronymic lineage from generation one. Replacement
/// newborns pass the deceased's name for household continuity.
///
/// ```
/// use mindstrata_sim::person::inherit_surname;
/// assert_eq!(inherit_surname("Mira Lars", "Tova"), "Tova Lars");
/// assert_eq!(inherit_surname("Anna", "Bran"), "Bran Anna");
/// assert_eq!(inherit_surname("Mira Lars", "Tova") , inherit_surname("Kira Lars", "Tova"));
/// ```
pub fn inherit_surname(parent_name: &str, given_name: &str) -> String {
    let surname = parent_name.split_once(' ').map_or(parent_name, |(_, s)| s);
    format!("{given_name} {surname}")
}

/// Personality trait model (Big Five + extensions).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// §8.1.6 (Iteration 179): The birth-trait constitution — anchor for
    /// core-trait plasticity (`plastic_update_traits`). `None` only for
    /// snapshots saved before this field existed; lazily anchored on the
    /// first plasticity tick.
    #[serde(default)]
    pub constitution: Option<TraitConstitution>,
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
            // §8.1.6 (Iteration 179): capture the birth constitution BEFORE
            // any life experience reshapes the traits.
            constitution: None,
        };
        p.temperament = Temperament::from_traits(&p);
        // Capture the constitution after `from_traits` (zero extra RNG draws
        // — the constitution is a pure snapshot of the drawn traits).
        p.constitution = Some(TraitConstitution::from_personality(&p));
        p
    }

    /// Quantitative-genetics inheritance (Iteration 244): child trait =
    /// parental midpoint, shrunk 20% toward the population mean (0.5), plus
    /// small noise (±0.06). With `other == None` (single known ancestor —
    /// household continuity for replacement newborns) the child clusters
    /// around that ancestor. Temperament and birth constitution are re-derived
    /// exactly as in `random` (zero extra RNG draws beyond the per-trait
    /// noise, keeping seeded streams stable and short).
    pub fn inherit(parent: &Personality, other: Option<&Personality>, rng: &mut impl Rng) -> Self {
        let mut blend_trait = |a: Fixed, b: Fixed| -> Fixed {
            let mid = (a.to_f64() + b.to_f64()) * 0.5;
            let shrunk = mid * 0.8 + 0.5 * 0.2;
            let noise = rng.random_range(-0.06f64..0.06);
            Fixed::from_f64((shrunk + noise).clamp(0.0, 1.0))
        };
        let fallback = parent;
        let b = other.unwrap_or(fallback);
        let mut p = Self {
            openness: blend_trait(parent.openness, b.openness),
            conscientiousness: blend_trait(parent.conscientiousness, b.conscientiousness),
            extraversion: blend_trait(parent.extraversion, b.extraversion),
            agreeableness: blend_trait(parent.agreeableness, b.agreeableness),
            neuroticism: blend_trait(parent.neuroticism, b.neuroticism),
            risk_tolerance: blend_trait(parent.risk_tolerance, b.risk_tolerance),
            conformity: blend_trait(parent.conformity, b.conformity),
            ambition: blend_trait(parent.ambition, b.ambition),
            altruism: blend_trait(parent.altruism, b.altruism),
            traditionalism: blend_trait(parent.traditionalism, b.traditionalism),
            dominance: blend_trait(parent.dominance, b.dominance),
            impulsivity: blend_trait(parent.impulsivity, b.impulsivity),
            temperament: Temperament::default(),
            constitution: None,
        };
        p.temperament = Temperament::from_traits(&p);
        p.constitution = Some(TraitConstitution::from_personality(&p));
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

/// §8.1.6 (Iteration 179): Shared plasticity-rate computation for both the
/// observational temperament layer (`Temperament::plastic_update`) and the
/// 12 decision-read core traits (`Personality::plastic_update_traits`) — the
/// plan's formula `repeated_state_expression × identity_integration ×
/// social_reinforcement × developmental_plasticity` folded into a single
/// per-tick rate. Returns `Fixed::ZERO` (inert) when either developmental
/// plasticity (full age) or identity integration is zero, so callers can
/// early-return. The arithmetic order (developmental × integration ×
/// reinforcement × rate_const, left-associative Fixed multiplies) is shared
/// verbatim by both callers to preserve byte-identical drift.
fn plasticity_rate(s: &PlasticitySignals, rate_const: f64) -> Fixed {
    // Developmental plasticity decays with age — full at birth, zero at 70.
    let developmental = (Fixed::ONE - (s.age_years / Fixed::from_f64(70.0)).clamp_01()).clamp_01();
    let integration = s.identity_integration.clamp_01();
    if developmental <= Fixed::ZERO || integration <= Fixed::ZERO {
        return Fixed::ZERO;
    }
    // Social reinforcement amplifies the absorption of experience.
    let reinforcement = Fixed::ONE + Fixed::HALF * s.social_engagement.clamp_01();
    developmental * integration * reinforcement * Fixed::from_f64(rate_const)
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
            sensitivity: (Fixed::from_f64(0.5) * t.openness + Fixed::from_f64(0.5) * t.neuroticism)
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
        let rate = plasticity_rate(s, TEMPERAMENT_PLASTICITY_RATE);
        if rate <= Fixed::ZERO {
            return;
        }
        let stress = s.repeated_stress.clamp_01();
        let recovery = s.recovery.clamp_01();
        let social = s.social_engagement.clamp_01();
        let striving = s.goal_striving.clamp_01();
        // Push from repeated state expression; pull toward the baseline.
        self.reactivity =
            (self.reactivity + stress * rate + (baseline.reactivity - self.reactivity) * rate)
                .clamp_01();
        self.soothability = (self.soothability
            + recovery * rate
            + (baseline.soothability - self.soothability) * rate)
            .clamp_01();
        self.sociability =
            (self.sociability + social * rate + (baseline.sociability - self.sociability) * rate)
                .clamp_01();
        self.persistence =
            (self.persistence + striving * rate + (baseline.persistence - self.persistence) * rate)
                .clamp_01();
        // Sensitivity is driven by total arousal exposure (stress + recovery).
        self.sensitivity = (self.sensitivity
            + (stress + recovery) * rate
            + (baseline.sensitivity - self.sensitivity) * rate)
            .clamp_01();
        // Routinized striving builds regularity; social engagement builds approach.
        self.regularity =
            (self.regularity + striving * rate + (baseline.regularity - self.regularity) * rate)
                .clamp_01();
        self.approach_withdrawal = (self.approach_withdrawal
            + social * rate
            + (baseline.approach_withdrawal - self.approach_withdrawal) * rate)
            .clamp_01();
    }
}

impl Personality {
    /// §8.1.6 (Iteration 179): Core-trait plasticity — the plan's "traits
    /// slowly change through repeated behavior, trauma, success, failure..."
    /// formula applied to the 12 decision-read core traits.
    ///
    /// Each trait is pushed toward its repeatedly-expressed state signal and
    /// pulled back toward the birth constitution (the stable anchor), both at
    /// the same `rate` derived from `repeated_state_expression ×
    /// identity_integration × social_reinforcement × developmental_plasticity`
    /// — the exact plan formula. The push term moves a trait toward the
    /// *live* expressed state (so a persistently stressed agent's neuroticism
    /// drifts up while the signal is present); the pull term returns it to
    /// the constitution when the signal fades (so a recovered agent's
    /// neuroticism settles back). Deterministic (no RNG), clamped to 0..1.
    ///
    /// Lazy anchor: with `constitution == None` (old snapshots) the method
    /// anchors to the current traits on the first tick, so the PULL term
    /// starts at zero (no anchor-induced jump); the PUSH terms still apply
    /// normally from the first tick (stress raises neuroticism immediately).
    /// With zero identity integration or full age the rate is zero and
    /// nothing moves.
    ///
    /// Rate constant: `CORE_TRAIT_PLASTICITY_RATE` 0.0005 — equal to the
    /// temperament rate because the 4-decimal Fixed resolution floors
    /// anything smaller to inertness; the constitution pull term still
    /// damps net drift below the observational layer's.
    pub fn plastic_update_traits(&mut self, s: &PlasticitySignals) {
        // Lazy anchor: old snapshots restore with `constitution: None` —
        // capture the current traits as the anchor so the PULL term starts
        // at zero (no anchor-induced jump). `TraitConstitution` is `Copy`,
        // so this avoids any borrow-holding reference.
        if self.constitution.is_none() {
            self.constitution = Some(TraitConstitution::from_personality(self));
        }
        // `TraitConstitution` is `Copy`; `is_none()` was just checked above,
        // so this read cannot fail (avoids a borrow-held reference).
        let constitution = self
            .constitution
            .unwrap_or_else(|| TraitConstitution::from_personality(self));
        let rate = plasticity_rate(s, CORE_TRAIT_PLASTICITY_RATE);
        if rate <= Fixed::ZERO {
            return;
        }
        // Identity integration drives the conformity/traditionalism push
        // terms below (the shared helper consumed it for the rate).
        let integration = s.identity_integration.clamp_01();
        let stress = s.repeated_stress.clamp_01();
        let recovery = s.recovery.clamp_01();
        let social = s.social_engagement.clamp_01();
        let striving = s.goal_striving.clamp_01();
        // Neuroticism: pushed by repeated stress (trauma/anxiety pathway),
        // pulled back toward the constitution in calm.
        self.neuroticism = (self.neuroticism
            + stress * rate
            + (constitution.neuroticism - self.neuroticism) * rate)
            .clamp_01();
        // Impulsivity: pushed by stress (impulse-control erosion under load).
        self.impulsivity = (self.impulsivity
            + stress * rate
            + (constitution.impulsivity - self.impulsivity) * rate)
            .clamp_01();
        // Openness: pushed by recovery (novel positive engagement opens minds).
        self.openness =
            (self.openness + recovery * rate + (constitution.openness - self.openness) * rate)
                .clamp_01();
        // Extraversion: pushed by social engagement.
        self.extraversion = (self.extraversion
            + social * rate
            + (constitution.extraversion - self.extraversion) * rate)
            .clamp_01();
        // Conscientiousness: pushed by goal striving (discipline through
        // repeated commitment).
        self.conscientiousness = (self.conscientiousness
            + striving * rate
            + (constitution.conscientiousness - self.conscientiousness) * rate)
            .clamp_01();
        // Agreeableness: pushed by recovery (positive interactions build
        // warmth) and social engagement.
        self.agreeableness = (self.agreeableness
            + (recovery + social) * rate
            + (constitution.agreeableness - self.agreeableness) * rate)
            .clamp_01();
        // Risk tolerance: pushed by recovery (successful outcomes build
        // risk appetite) and pulled back by repeated stress.
        self.risk_tolerance = (self.risk_tolerance
            + (recovery - stress) * rate
            + (constitution.risk_tolerance - self.risk_tolerance) * rate)
            .clamp_01();
        // Conformity: pushed by identity integration (internalized norms).
        self.conformity = (self.conformity
            + integration * rate
            + (constitution.conformity - self.conformity) * rate)
            .clamp_01();
        // Ambition: pushed by goal striving.
        self.ambition =
            (self.ambition + striving * rate + (constitution.ambition - self.ambition) * rate)
                .clamp_01();
        // Altruism: pushed by social engagement (prosocial practice).
        self.altruism =
            (self.altruism + social * rate + (constitution.altruism - self.altruism) * rate)
                .clamp_01();
        // Traditionalism: pushed by identity integration (stability-seeking).
        self.traditionalism = (self.traditionalism
            + integration * rate
            + (constitution.traditionalism - self.traditionalism) * rate)
            .clamp_01();
        // Dominance: pushed by goal striving (assertion through striving).
        self.dominance =
            (self.dominance + striving * rate + (constitution.dominance - self.dominance) * rate)
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
    /// §5 (Iteration 155): An external directive — the interactive TUI's
    /// command channel injected this goal between ticks.
    Command,
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
            + self.social_status * Fixed::from_f64(0.25))
        .clamp_01();
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
        let mut p = Personality {
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
            constitution: None,
        };
        p.constitution = Some(TraitConstitution::from_personality(&p));
        p
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
        assert!(
            up > Fixed::ONE,
            "stressed reactivity must amplify the response"
        );
        assert!(
            down < Fixed::ONE,
            "below-baseline reactivity must damp the response"
        );
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

    #[test]
    fn trait_plasticity_pushes_stressed_traits_up_and_returns_to_constitution() {
        // §8.1.6 (Iteration 179): under sustained stress the decision-read
        // core traits move — neuroticism/impulsivity rise (stress pathway);
        // when the stress fades they settle back toward the constitution.
        let mut p = base_personality();
        let constitution = p.constitution.unwrap();
        let n0 = p.neuroticism;
        let i0 = p.impulsivity;
        let stressed = PlasticitySignals {
            repeated_stress: Fixed::from_f64(0.8),
            recovery: Fixed::from_f64(0.1),
            social_engagement: Fixed::from_f64(0.3),
            goal_striving: Fixed::from_f64(0.3),
            identity_integration: Fixed::ONE,
            age_years: Fixed::from_f64(20.0),
        };
        for _ in 0..3000 {
            p.plastic_update_traits(&stressed);
        }
        assert!(
            p.neuroticism > n0,
            "repeated stress must raise neuroticism (was {n0}, now {})",
            p.neuroticism.to_f64()
        );
        assert!(
            p.impulsivity > i0,
            "repeated stress must raise impulsivity (was {i0}, now {})",
            p.impulsivity.to_f64()
        );
        // Stress also suppresses risk tolerance (recovery − stress < 0).
        assert!(
            p.risk_tolerance < constitution.risk_tolerance,
            "stress must suppress risk tolerance (constitution {}, now {})",
            constitution.risk_tolerance.to_f64(),
            p.risk_tolerance.to_f64()
        );
        // Now the stress fades: calm recovery returns the traits toward the
        // constitution (not instantly — the pull term is proportional).
        let calm = PlasticitySignals {
            repeated_stress: Fixed::ZERO,
            recovery: Fixed::from_f64(0.8),
            social_engagement: Fixed::from_f64(0.3),
            goal_striving: Fixed::from_f64(0.3),
            identity_integration: Fixed::ONE,
            age_years: Fixed::from_f64(20.0),
        };
        let raised = p.neuroticism;
        for _ in 0..3000 {
            p.plastic_update_traits(&calm);
        }
        assert!(
            p.neuroticism < raised,
            "calm must pull neuroticism back down (was {raised}, now {})",
            p.neuroticism.to_f64()
        );
    }

    #[test]
    fn trait_plasticity_stress_product_survives_fixed_resolution() {
        // §8.1.6 (Iteration 179): the rate must be sized so the compound
        // (development × integration × reinforcement × rate) lands above the
        // Fixed 4-decimal floor — otherwise `stress × rate` truncates to zero
        // and the stress pathway is inert (the 0.0002 probe: 0.8 × 0.0001 → 0).
        let mut p = base_personality();
        let stressed = PlasticitySignals {
            repeated_stress: Fixed::from_f64(0.8),
            recovery: Fixed::from_f64(0.1),
            social_engagement: Fixed::from_f64(0.3),
            goal_striving: Fixed::from_f64(0.3),
            identity_integration: Fixed::ONE,
            age_years: Fixed::from_f64(20.0),
        };
        let n0 = p.neuroticism;
        // A SINGLE tick must move the trait at the Fixed scale (≥ 1 raw).
        p.plastic_update_traits(&stressed);
        assert!(
            p.neuroticism != n0,
            "one stress tick must move neuroticism (0.8 stress × compound rate must survive Fixed truncation)"
        );
    }

    #[test]
    fn trait_plasticity_is_inert_without_identity_integration_or_in_old_age() {
        // Zero identity integration → zero rate, no matter the stress.
        let mut p = base_personality();
        let snapshot = p.clone();
        let quiet = signals(Fixed::ZERO, Fixed::from_f64(20.0));
        for _ in 0..100 {
            p.plastic_update_traits(&quiet);
        }
        assert_eq!(p, snapshot, "no identity integration → no trait plasticity");
        // Maximum age → developmental plasticity = 0.
        let old = signals(Fixed::ONE, Fixed::from_f64(70.0));
        for _ in 0..100 {
            p.plastic_update_traits(&old);
        }
        assert_eq!(p, snapshot, "old age → no trait plasticity");
    }

    #[test]
    fn trait_plasticity_lazily_anchors_constitution_when_missing() {
        // Old snapshots restore with `constitution: None` — the first
        // plasticity tick must anchor to the current traits. The PULL term
        // starts at zero on the anchor tick (no anchor-induced jump); the
        // PUSH terms still apply normally from tick 1.
        let mut p = base_personality();
        p.constitution = None;
        let pre_tick_traits = p.clone();
        let stressed = PlasticitySignals {
            repeated_stress: Fixed::from_f64(0.8),
            recovery: Fixed::from_f64(0.1),
            social_engagement: Fixed::from_f64(0.3),
            goal_striving: Fixed::from_f64(0.3),
            identity_integration: Fixed::ONE,
            age_years: Fixed::from_f64(20.0),
        };
        p.plastic_update_traits(&stressed);
        // The anchor must be the pre-tick traits (captured before any push).
        let anchored = p
            .constitution
            .expect("lazy anchor must set the constitution");
        let expected = TraitConstitution::from_personality(&pre_tick_traits);
        assert_eq!(
            anchored, expected,
            "lazy anchor must capture the pre-tick traits exactly (no jump)"
        );
        // Subsequent ticks drift normally (stress pushes neuroticism up).
        let n0 = p.neuroticism;
        for _ in 0..2000 {
            p.plastic_update_traits(&stressed);
        }
        assert!(p.neuroticism > n0, "anchored traits must then drift");
    }
}

#[cfg(test)]
mod surname_tests {
    use super::inherit_surname;

    #[test]
    fn surnames_form_lineages() {
        assert_eq!(inherit_surname("Mira Lars", "Tova"), "Tova Lars");
        assert_eq!(inherit_surname("Anna", "Bran"), "Bran Anna");
        // Any Lars-parent child carries the Lars line.
        assert_eq!(inherit_surname("Kira Lars", "Ulf"), "Ulf Lars");
        assert_eq!(
            inherit_surname("Mira Lars", "Tova"),
            inherit_surname("Kira Lars", "Tova")
        );
        // Third generation keeps the founding surname through the space rule.
        let gen2 = inherit_surname("Bran Anna", "Cara");
        assert_eq!(gen2, "Cara Anna");
    }
}

#[cfg(test)]
mod moral_inherit_tests {
    use super::{MoralValues, FIRST_NAMES};
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
