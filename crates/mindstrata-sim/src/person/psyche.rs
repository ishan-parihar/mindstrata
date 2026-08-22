//! Trait constitution, personality, temperament plasticity, affect.

use mindstrata_core::fixed::Fixed;
use rand::Rng;
use serde::{Deserialize, Serialize};

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
