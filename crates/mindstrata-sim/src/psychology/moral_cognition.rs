//! Moral cognition system — moral foundations, internalized norms, moral emotions.
//!
//! Moral foundations: care/harm, fairness/cheating, loyalty/betrayal,
//! authority/subversion, purity/degradation, liberty/oppression.
//!
//! Emergent effects: moral outrage, scapegoating, martyrdom, reform movements,
//! religious schisms, honor feuds.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// §8.1.10 (P3-11): minimum first-exposure reinforcement to internalize a NEW
/// norm. The norm count was uniform (5.000 for 12/12 agents at 10K — zero
/// spread) because `internalize_norm` accepted any positive amount, so every
/// ritual participant internalized every community norm at the identical
/// rate; conformity had no seat at the ritual. Ritual reinforcement now
/// scales by per-agent conformity (sim-side, 0.5–1.0×), and this floor gates
/// first internalization: a weakly-reinforced exposure (low conformity ×
/// weakly-internalized community norm) never cements a norm. Sized at 0.05 —
/// below the seasonal-ritual base reinforcement for every default norm
/// (0.4×0.23 = 0.092 at minimum), so no default norm becomes unreachable,
/// but above the scaled floor of a low-conformity agent on a weakly-
/// internalized norm, so per-agent counts differentiate.
pub const NORM_FIRST_EXPOSURE_FLOOR: Fixed = Fixed::from_raw(500); // 0.05

/// Moral Foundations Theory — six moral dimensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoralFoundations {
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

impl Default for MoralFoundations {
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

/// An internalized norm — a moral rule the agent has adopted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalizedNorm {
    /// What the norm prohibits or requires.
    pub description: String,
    /// How strongly the agent holds this norm (0–1).
    pub strength: Fixed,
    /// Emotional charge when violated (0–1).
    pub emotional_charge: Fixed,
    /// Whether this norm is identity-linked (resists change).
    pub identity_linked: bool,
    /// Number of times the agent has witnessed enforcement.
    pub enforcement_count: u32,
}

/// Moral emotions — emotions specifically tied to moral evaluation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MoralEmotions {
    /// Moral outrage at witnessed injustice.
    pub outrage: Fixed,
    /// Shame from personal moral failure.
    pub shame: Fixed,
    /// Guilt from harming others.
    pub guilt: Fixed,
    /// Pride from moral achievement.
    pub pride: Fixed,
    /// Contempt for moral violators.
    pub contempt: Fixed,
    /// Gratitude for moral gifts.
    pub gratitude: Fixed,
}

/// Complete moral cognition state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoralCognition {
    /// Moral foundations profile.
    pub foundations: MoralFoundations,
    /// Internalized norms.
    pub internalized_norms: Vec<InternalizedNorm>,
    /// Current moral emotions.
    pub moral_emotions: MoralEmotions,
    /// How central morality is to identity (0–1).
    pub moral_identity: Fixed,
    /// Sensitivity to hypocrisy (own and others').
    pub hypocrisy_sensitivity: Fixed,
    /// Sensitivity to purity violations.
    pub purity_sensitivity: Fixed,
    /// Sensitivity to fairness violations.
    pub fairness_sensitivity: Fixed,
    /// Sensitivity to loyalty violations.
    pub loyalty_sensitivity: Fixed,
    /// Sensitivity to authority violations.
    pub authority_sensitivity: Fixed,
}

impl Default for MoralCognition {
    fn default() -> Self {
        Self {
            foundations: MoralFoundations::default(),
            internalized_norms: Vec::new(),
            moral_emotions: MoralEmotions::default(),
            moral_identity: Fixed::from_f64(0.5),
            hypocrisy_sensitivity: Fixed::from_f64(0.5),
            purity_sensitivity: Fixed::from_f64(0.3),
            fairness_sensitivity: Fixed::from_f64(0.5),
            loyalty_sensitivity: Fixed::from_f64(0.4),
            authority_sensitivity: Fixed::from_f64(0.4),
        }
    }
}

impl MoralCognition {
    /// Compute moral outrage from a witnessed norm violation.
    pub fn compute_outrage(&self, violation_severity: Fixed, violator_is_in_group: bool) -> Fixed {
        let care_outrage = self.foundations.care * violation_severity * Fixed::from_f64(0.3);
        let fairness_outrage =
            self.foundations.fairness * violation_severity * Fixed::from_f64(0.25);
        let loyalty_outrage = if violator_is_in_group {
            self.foundations.loyalty * violation_severity * Fixed::from_f64(0.2)
        } else {
            Fixed::ZERO
        };
        let authority_outrage =
            self.foundations.authority * violation_severity * Fixed::from_f64(0.15);
        let purity_outrage = self.purity_sensitivity * violation_severity * Fixed::from_f64(0.1);

        (care_outrage + fairness_outrage + loyalty_outrage + authority_outrage + purity_outrage)
            .clamp_01()
    }

    /// Reinforce an internalized norm through ritual participation
    /// (AP2 §12.5: rituals \"reinforce norms\"). Strengthens an existing
    /// norm on repeated exposure, or internalizes it on first exposure.
    /// Deterministic — no RNG, pure function of input state.
    ///
    /// §8.1.10 (P3-11): first exposure is gated by `NORM_FIRST_EXPOSURE_FLOOR`
    /// — a ritual exposure weaker than the floor never cements a new norm, so
    /// per-agent conformity scaling at the call site differentiates the
    /// internalized-norm COUNT, not just the strengths.
    pub fn reinforce_norm(&mut self, description: &str, amount: Fixed) {
        if let Some(norm) = self
            .internalized_norms
            .iter_mut()
            .find(|n| n.description == description)
        {
            norm.strength = (norm.strength + amount).clamp_01();
            norm.emotional_charge = norm.strength * Fixed::from_f64(0.5);
            // Ritual exposure is NOT witnessed enforcement — `enforcement_count`
            // stays untouched so the field keeps its documented meaning
            // ("times the agent has witnessed enforcement") for any future
            // consumer such as a violation audit.
            norm.identity_linked = norm.identity_linked || norm.strength > Fixed::from_f64(0.7);
        } else if amount >= NORM_FIRST_EXPOSURE_FLOOR {
            self.internalize_norm(description.to_string(), amount);
        }
    }

    /// Internalize a new norm through repeated exposure and enforcement.
    pub fn internalize_norm(&mut self, description: String, strength: Fixed) {
        // Check if already internalized
        if self
            .internalized_norms
            .iter()
            .any(|n| n.description == description)
        {
            return;
        }
        self.internalized_norms.push(InternalizedNorm {
            description,
            strength,
            emotional_charge: strength * Fixed::from_f64(0.5),
            identity_linked: strength > Fixed::from_f64(0.7),
            enforcement_count: 0,
        });
    }

    /// Update moral emotions based on recent events.
    ///
    /// `moral_development` (Kohlberg stage 0 = preconventional, 1 =
    /// postconventional) scales the internalized shame/pride response: a
    /// more-developed agent feels its own violations and achievements more
    /// sharply. Baseline-corrected against the default stage (0.4) so the
    /// default is an EXACT no-op (scale 1.0) and only evolved development
    /// modulates — the interoception `felt_need_deficit` pattern, keeping
    /// populate-default populations byte-identical while the §17
    /// developmental pipeline genuinely feeds behavior as it advances.
    pub fn update_moral_emotions(
        &mut self,
        witnessed_violations: Fixed,
        personal_violations: Fixed,
        moral_achievements: Fixed,
        moral_development: Fixed,
    ) {
        // 1.0 at the default stage 0.4 → ±~1.5 at the extremes (0.0/1.0),
        // clamped to [0.5, 2.0] defensively.
        let development_scale = (Fixed::ONE
            + (moral_development - Fixed::from_f64(0.4)) * Fixed::from_f64(1.5))
        .clamp(Fixed::from_f64(0.5), Fixed::from_f64(2.0));
        // Outrage builds from witnessed violations
        self.moral_emotions.outrage = (self.moral_emotions.outrage
            + witnessed_violations * self.foundations.fairness * Fixed::from_f64(0.05))
        .clamp_01();
        // Shame from personal violations
        self.moral_emotions.shame = (self.moral_emotions.shame
            + personal_violations * self.moral_identity * development_scale
                * Fixed::from_f64(0.05))
        .clamp_01();
        // Pride from moral achievements
        self.moral_emotions.pride = (self.moral_emotions.pride
            + moral_achievements * self.moral_identity * development_scale
                * Fixed::from_f64(0.03))
        .clamp_01();

        // Decay all moral emotions
        let decay = Fixed::from_f64(0.003);
        self.moral_emotions.outrage = (self.moral_emotions.outrage - decay).max(Fixed::ZERO);
        self.moral_emotions.shame = (self.moral_emotions.shame - decay).max(Fixed::ZERO);
        self.moral_emotions.pride = (self.moral_emotions.pride - decay).max(Fixed::ZERO);
        self.moral_emotions.contempt = (self.moral_emotions.contempt - decay).max(Fixed::ZERO);
        self.moral_emotions.gratitude = (self.moral_emotions.gratitude - decay).max(Fixed::ZERO);
    }

    /// Check if a proposed action would violate any internalized norm.
    /// Returns the strength of norm resistance (0 = no resistance, 1 = strong resistance).
    pub fn norm_resistance(&self, action_description: &str) -> Fixed {
        let mut max_resistance = Fixed::ZERO;
        for norm in &self.internalized_norms {
            if norm.description == action_description {
                max_resistance = max_resistance.max(norm.strength);
            }
        }
        max_resistance
    }

    /// §8.1.10/§19.5.D (Iteration 86): an agent who witnesses the community
    /// enforce an internalized norm records it. `enforcement_count` (doc:
    /// "Number of times the agent has witnessed enforcement") was created at
    /// 0 by `internalize_norm` and never incremented in production — a
    /// ritual could internalize the norm, but a caught theft fined by the
    /// Council strengthened nothing in anyone's moral cognition. Public
    /// enforcement is a community event: every agent holding the norm
    /// witnesses it, so the audit increments their count. Observational (no
    /// production consumer reads `enforcement_count` yet), saturating, and a
    /// no-op for norms the agent has not internalized.
    pub fn record_witnessed_enforcement(&mut self, description: &str) {
        if let Some(norm) = self
            .internalized_norms
            .iter_mut()
            .find(|n| n.description == description)
        {
            norm.enforcement_count = norm.enforcement_count.saturating_add(1);
        }
    }

    /// §8.1.10 (Iteration 87): hypocrisy — the behavioral consumer of the
    /// Iteration-86 witnessed-enforcement audit. An agent who has seen the
    /// community punish a norm violation (high `enforcement_count`) and is
    /// sensitive to hypocrisy resists committing that same violation: "I
    /// have seen people punished for this; doing it myself would make me a
    /// hypocrite." Returns `hypocrisy_sensitivity × exposure`, where
    /// exposure is the witnessed-enforcement count capped at
    /// [`HYPOCRISY_ENFORCEMENT_CAP`] (5 public enforcements fully activate
    /// the trait's weight). Zero-at-zero: no witnessed enforcement (the
    /// default world's public-access farms never catch a theft) → factor 0 →
    /// legacy behavior. Absent norms and zero sensitivity are both 0.
    pub fn hypocrisy_factor(&self, description: &str) -> Fixed {
        let count = self
            .internalized_norms
            .iter()
            .find(|n| n.description == description)
            .map_or(0, |n| n.enforcement_count);
        if count == 0 {
            return Fixed::ZERO;
        }
        let exposure =
            (count.min(HYPOCRISY_ENFORCEMENT_CAP) as f64) / (HYPOCRISY_ENFORCEMENT_CAP as f64);
        // Clamped: `hypocrisy_sensitivity` is a pub field that a scenario
        // config could deserialize out of range — without the clamp an
        // over-1.0 factor would drive the theft gate's (1 − hypocrisy)
        // negative and silently refuse every theft.
        Fixed::from_f64(self.hypocrisy_sensitivity.to_f64() * exposure).clamp_01()
    }
}

/// §8.1.10 (Iteration 87): witnessed-enforcement count at which the
/// hypocrisy effect saturates — 5 public enforcements fully activate the
/// agent's hypocrisy-sensitivity weight.
pub const HYPOCRISY_ENFORCEMENT_CAP: u32 = 5;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outrage_from_fairness_violation() {
        let mc = MoralCognition {
            foundations: MoralFoundations {
                fairness: Fixed::from_f64(0.9),
                ..MoralFoundations::default()
            },
            ..MoralCognition::default()
        };
        let outrage = mc.compute_outrage(Fixed::from_f64(0.8), false);
        assert!(outrage > Fixed::ZERO);
    }

    #[test]
    fn internalize_norm_prevents_duplicates() {
        let mut mc = MoralCognition::default();
        mc.internalize_norm("no_theft".into(), Fixed::from_f64(0.7));
        mc.internalize_norm("no_theft".into(), Fixed::from_f64(0.9));
        assert_eq!(mc.internalized_norms.len(), 1);
    }

    #[test]
    fn norm_resistance_returns_strength() {
        let mut mc = MoralCognition::default();
        mc.internalize_norm("no_theft".into(), Fixed::from_f64(0.8));
        let resistance = mc.norm_resistance("no_theft");
        assert_eq!(resistance, Fixed::from_f64(0.8));
    }

    #[test]
    fn reinforce_norm_strengthens_existing() {
        let mut mc = MoralCognition::default();
        mc.internalize_norm("no_violence".into(), Fixed::from_f64(0.3));
        mc.reinforce_norm("no_violence", Fixed::from_f64(0.2));
        assert_eq!(mc.internalized_norms.len(), 1);
        assert_eq!(
            mc.internalized_norms[0].strength,
            Fixed::from_f64(0.5),
            "repeated ritual participation should strengthen the norm"
        );
        // Ritual exposure is not witnessed enforcement — the counter stays 0.
        assert_eq!(mc.internalized_norms[0].enforcement_count, 0);
    }

    #[test]
    fn reinforce_norm_internalizes_on_first_exposure() {
        let mut mc = MoralCognition::default();
        mc.reinforce_norm("respect_elders", Fixed::from_f64(0.4));
        assert_eq!(mc.internalized_norms.len(), 1);
        assert_eq!(mc.internalized_norms[0].strength, Fixed::from_f64(0.4));
        // Sub-threshold first exposure: not identity-linked, zero enforcement.
        assert!(!mc.internalized_norms[0].identity_linked);
        assert_eq!(mc.internalized_norms[0].enforcement_count, 0);
    }

    #[test]
    fn reinforce_norm_clamps_at_one() {
        let mut mc = MoralCognition::default();
        mc.internalize_norm("no_theft".into(), Fixed::from_f64(0.8));
        mc.reinforce_norm("no_theft", Fixed::from_f64(0.5));
        assert_eq!(mc.internalized_norms[0].strength, Fixed::ONE);
        // Once strength crosses 0.7 the norm becomes identity-linked.
        assert!(mc.internalized_norms[0].identity_linked);
    }

    /// §8.1.10/§19.5.D (Iteration 86): witnessing public enforcement
    /// increments the internalized norm's `enforcement_count` — the audit
    /// channel created at 0 by `internalize_norm` and previously never
    /// written in production. Un-internalized norms are a silent no-op, the
    /// count is saturating, and the audit never touches strength or
    /// identity-linking.
    #[test]
    fn record_witnessed_enforcement_increments_count() {
        let mut mc = MoralCognition::default();
        // Un-internalized norm: no-op, no panic, nothing added.
        mc.record_witnessed_enforcement("no_theft");
        assert!(mc.internalized_norms.is_empty());
        // Internalize two norms; only the matching one records enforcement.
        mc.internalize_norm("no_theft".into(), Fixed::from_f64(0.6));
        mc.internalize_norm("no_violence".into(), Fixed::from_f64(0.6));
        mc.record_witnessed_enforcement("no_theft");
        mc.record_witnessed_enforcement("no_theft");
        mc.record_witnessed_enforcement("no_violence");
        let theft = mc
            .internalized_norms
            .iter()
            .find(|n| n.description == "no_theft")
            .unwrap();
        let violence = mc
            .internalized_norms
            .iter()
            .find(|n| n.description == "no_violence")
            .unwrap();
        assert_eq!(theft.enforcement_count, 2);
        assert_eq!(violence.enforcement_count, 1);
        // The audit does not touch strength or identity.
        assert_eq!(theft.strength, Fixed::from_f64(0.6));
    }

    /// §8.1.10 (Iteration 87): `hypocrisy_factor` scales the agent's
    /// hypocrisy sensitivity by witnessed-enforcement exposure — zero count
    /// or zero sensitivity → 0, cap saturation → the full trait weight, and
    /// the absent-norm case is a silent 0.
    #[test]
    fn hypocrisy_factor_scales_with_witnessed_enforcement() {
        let mut mc = MoralCognition::default();
        // No internalized norm → 0.
        assert_eq!(mc.hypocrisy_factor("no_theft"), Fixed::ZERO);
        mc.internalize_norm("no_theft".into(), Fixed::from_f64(0.3));
        // Internalized but no witnessed enforcement → 0 (zero-at-zero).
        assert_eq!(mc.hypocrisy_factor("no_theft"), Fixed::ZERO);
        // One witnessed enforcement: 0.5 sensitivity × 1/5 = 0.1.
        mc.record_witnessed_enforcement("no_theft");
        assert_eq!(mc.hypocrisy_factor("no_theft"), Fixed::from_f64(0.1));
        // Saturation: 5 witnessed enforcements → full sensitivity (0.5).
        for _ in 0..4 {
            mc.record_witnessed_enforcement("no_theft");
        }
        assert_eq!(mc.hypocrisy_factor("no_theft"), Fixed::from_f64(0.5));
        // Beyond the cap the factor stays at the full sensitivity.
        mc.record_witnessed_enforcement("no_theft");
        assert_eq!(mc.hypocrisy_factor("no_theft"), Fixed::from_f64(0.5));
        // A different norm's count does not bleed over.
        assert_eq!(mc.hypocrisy_factor("no_violence"), Fixed::ZERO);
        // Zero sensitivity → 0 regardless of exposure.
        let mut low = MoralCognition {
            hypocrisy_sensitivity: Fixed::ZERO,
            ..Default::default()
        };
        low.internalize_norm("no_theft".into(), Fixed::from_f64(0.3));
        low.record_witnessed_enforcement("no_theft");
        assert_eq!(low.hypocrisy_factor("no_theft"), Fixed::ZERO);
    }

    /// §17 (Iteration 195): `moral_development` (Kohlberg stage) modulates
    /// the internalized shame/pride response — a more-developed agent feels
    /// its own violations and achievements more sharply. The scale is
    /// 0.5 + development×0.5: 0.7 at the default 0.4, 1.0 at full
    /// postconventional development.
    #[test]
    fn moral_development_amplifies_shame_and_pride() {
        let mut low = MoralCognition::default();
        let mut high = MoralCognition::default();
        low.update_moral_emotions(
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.4), // default developmental stage
        );
        high.update_moral_emotions(
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::ONE, // fully postconventional
        );
        assert!(
            high.moral_emotions.shame > low.moral_emotions.shame,
            "higher moral development must amplify shame ({} vs {})",
            high.moral_emotions.shame,
            low.moral_emotions.shame
        );
        assert!(
            high.moral_emotions.pride > low.moral_emotions.pride,
            "higher moral development must amplify pride ({} vs {})",
            high.moral_emotions.pride,
            low.moral_emotions.pride
        );
        // Outrage is witnessed-side (fairness foundation only) — development
        // must NOT touch it (both zero here).
        assert_eq!(low.moral_emotions.outrage, Fixed::ZERO);
        assert_eq!(high.moral_emotions.outrage, Fixed::ZERO);
    }
}
