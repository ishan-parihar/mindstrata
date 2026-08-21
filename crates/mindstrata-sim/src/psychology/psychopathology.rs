//! Psychopathology / Mental health system — risk dynamics, not diagnostic labels.
//!
//! Risk states: depression, anxiety, PTSD, paranoia, addiction, dissociation,
//! grief pathology, resentment syndrome.
//!
//! Causal inputs: chronic stress, trauma, isolation, humiliation, loss,
//! moral injury, chronic pain, sleep deprivation, genetic vulnerability.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Mental health risk profile — tracks risk levels for various conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsychopathologyState {
    /// Depression risk (0 = none, 1 = severely depressed).
    pub depression_risk: Fixed,
    /// Anxiety risk (0 = none, 1 = severely anxious).
    pub anxiety_risk: Fixed,
    /// PTSD risk (0 = none, 1 = severely traumatized).
    pub ptsd_risk: Fixed,
    /// Paranoia risk (0 = none, 1 = severely paranoid).
    pub paranoia_risk: Fixed,
    /// Addiction vulnerability (0 = none, 1 = highly vulnerable).
    pub addiction_vulnerability: Fixed,
    /// Dissociation risk (0 = none, 1 = highly dissociative).
    pub dissociation_risk: Fixed,
    /// Grief pathology risk (0 = none, 1 = complicated grief).
    pub grief_pathology_risk: Fixed,
    /// Resentment syndrome risk (0 = none, 1 = deeply resentful).
    pub resentment_risk: Fixed,
    /// Overall mental health (0 = severely impaired, 1 = excellent).
    pub overall_health: Fixed,
}

impl Default for PsychopathologyState {
    fn default() -> Self {
        Self {
            depression_risk: Fixed::ZERO,
            anxiety_risk: Fixed::ZERO,
            ptsd_risk: Fixed::ZERO,
            paranoia_risk: Fixed::ZERO,
            addiction_vulnerability: Fixed::ZERO,
            dissociation_risk: Fixed::ZERO,
            grief_pathology_risk: Fixed::ZERO,
            resentment_risk: Fixed::ZERO,
            overall_health: Fixed::from_f64(0.8),
        }
    }
}

impl PsychopathologyState {
    /// Update mental health risks based on current conditions.
    pub fn tick_update(
        &mut self,
        chronic_stress: Fixed,
        trauma_load: Fixed,
        social_isolation: Fixed,
        humiliation_recent: Fixed,
        loss_magnitude: Fixed,
        moral_injury: Fixed,
        chronic_pain: Fixed,
        sleep_deprivation: Fixed,
        genetic_depression_vulnerability: Fixed,
        genetic_addiction_risk: Fixed,
        social_support: Fixed,
        // Iteration 192 (famine-chain closure): the documented "sadness ->
        // despair -> depression-from-deprivation" chain (famine-grain-drain
        // comment in sim.rs) ended at despair with NO consumer into
        // depression_risk — the endpoint read only chronic_stress and loss
        // magnitude, so even a despairing starving agent's depression risk
        // never moved. Despair is now a dedicated causal input (the
        // emotional hopelessness of an unmet, uncontrollable situation),
        // weighted below chronic stress/loss so it modulates rather than
        // dominates the risk. Zero-at-zero: an agent with no despair adds
        // nothing, so calm windows are untouched.
        despair: Fixed,
    ) {
        // Depression: chronic stress + loss + isolation + genetic vulnerability
        // + emotional despair (Iter-192: the deprivation chain endpoint).
        let depression_input = chronic_stress * Fixed::from_f64(0.2)
            + loss_magnitude * Fixed::from_f64(0.3)
            + social_isolation * Fixed::from_f64(0.2)
            + genetic_depression_vulnerability * Fixed::from_f64(0.15)
            + despair * Fixed::from_f64(0.2)
            - social_support * Fixed::from_f64(0.1);
        self.depression_risk = (self.depression_risk * Fixed::from_f64(0.995)
            + depression_input * Fixed::from_f64(0.005))
        .clamp_01();

        // Anxiety: chronic stress + trauma + sleep deprivation + genetic
        let anxiety_input = chronic_stress * Fixed::from_f64(0.25)
            + trauma_load * Fixed::from_f64(0.2)
            + sleep_deprivation * Fixed::from_f64(0.15)
            - social_support * Fixed::from_f64(0.08);
        self.anxiety_risk = (self.anxiety_risk * Fixed::from_f64(0.995)
            + anxiety_input * Fixed::from_f64(0.005))
        .clamp_01();

        // PTSD: trauma load + low coping + low support
        let ptsd_input = trauma_load * Fixed::from_f64(0.3)
            + (Fixed::ONE - social_support) * Fixed::from_f64(0.1);
        self.ptsd_risk = (self.ptsd_risk * Fixed::from_f64(0.998)
            + ptsd_input * Fixed::from_f64(0.002))
        .clamp_01();

        // Paranoia: social isolation + humiliation + low trust
        let paranoia_input = social_isolation * Fixed::from_f64(0.2)
            + humiliation_recent * Fixed::from_f64(0.25)
            + chronic_stress * Fixed::from_f64(0.1);
        self.paranoia_risk = (self.paranoia_risk * Fixed::from_f64(0.997)
            + paranoia_input * Fixed::from_f64(0.003))
        .clamp_01();

        // Addiction: genetic vulnerability + chronic stress + pain
        let addiction_input = genetic_addiction_risk * Fixed::from_f64(0.2)
            + chronic_stress * Fixed::from_f64(0.15)
            + chronic_pain * Fixed::from_f64(0.15);
        self.addiction_vulnerability = (self.addiction_vulnerability * Fixed::from_f64(0.999)
            + addiction_input * Fixed::from_f64(0.001))
        .clamp_01();

        // Dissociation: extreme trauma + pain
        let dissociation_input =
            trauma_load * Fixed::from_f64(0.2) + chronic_pain * Fixed::from_f64(0.1);
        self.dissociation_risk = (self.dissociation_risk * Fixed::from_f64(0.999)
            + dissociation_input * Fixed::from_f64(0.001))
        .clamp_01();

        // Grief pathology: loss magnitude + low support
        let grief_input = loss_magnitude * Fixed::from_f64(0.3)
            + (Fixed::ONE - social_support) * Fixed::from_f64(0.15);
        self.grief_pathology_risk = (self.grief_pathology_risk * Fixed::from_f64(0.998)
            + grief_input * Fixed::from_f64(0.002))
        .clamp_01();

        // Resentment: moral injury + humiliation + low autonomy
        let resentment_input = moral_injury * Fixed::from_f64(0.25)
            + humiliation_recent * Fixed::from_f64(0.2)
            + chronic_stress * Fixed::from_f64(0.1);
        self.resentment_risk = (self.resentment_risk * Fixed::from_f64(0.997)
            + resentment_input * Fixed::from_f64(0.003))
        .clamp_01();

        // Overall mental health is inverse of worst risk
        let worst_risk = self
            .depression_risk
            .max(self.anxiety_risk)
            .max(self.ptsd_risk)
            .max(self.paranoia_risk)
            .max(self.dissociation_risk);
        self.overall_health = (Fixed::ONE - worst_risk).clamp_01();
    }

    /// Whether the agent is experiencing significant mental health impairment.
    pub fn is_impaired(&self) -> bool {
        self.overall_health < Fixed::from_f64(0.5)
    }

    /// Compute modifier for cognitive function from mental health state.
    /// Returns the agent's `overall_health` directly: fully healthy (1.0)
    /// gives full planning depth; severely impaired (0.0) gives near-zero.
    /// Replaces the binary `is_impaired()` gate (§8.1.16) with a smooth
    /// graduated modifier — no cliff at the 0.5 threshold.
    pub fn cognitive_modifier(&self) -> Fixed {
        self.overall_health
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depression_increases_with_stress_and_isolation() {
        let mut p = PsychopathologyState::default();
        p.tick_update(
            Fixed::from_f64(0.8), // chronic stress
            Fixed::from_f64(0.3), // trauma
            Fixed::from_f64(0.7), // isolation
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5), // genetic depression
            Fixed::ZERO,
            Fixed::from_f64(0.2), // low support
            Fixed::ZERO,          // despair
        );
        assert!(p.depression_risk > Fixed::ZERO);
    }

    /// Iteration 192 (famine-chain closure): despair is now a dedicated
    /// causal input into depression_risk — the documented
    /// "sadness -> despair -> depression-from-deprivation" chain's last
    /// link, which previously had NO consumer. A despairing agent (all
    /// other inputs zero) must accumulate depression risk; the same agent
    /// with zero despair stays at zero.
    #[test]
    fn despair_feeds_depression_risk() {
        let mut despairing = PsychopathologyState::default();
        let mut control = PsychopathologyState::default();
        // 500 calls at the same 0.005 accumulation rate the sim uses.
        for _ in 0..500 {
            despairing.tick_update(
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::from_f64(0.5), // despair
            );
            control.tick_update(
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO, // no despair
            );
        }
        assert!(
            despairing.depression_risk > Fixed::from_f64(0.01),
            "despair must accumulate depression risk, got {}",
            despairing.depression_risk
        );
        assert_eq!(
            control.depression_risk,
            Fixed::ZERO,
            "zero despair must leave depression risk at zero"
        );
        // The hopelessness of an unmet need is a real but secondary driver:
        // below chronic stress/loss weights so it modulates, never dominates.
        assert!(
            despairing.depression_risk < Fixed::from_f64(0.5),
            "despair must not dominate depression risk, got {}",
            despairing.depression_risk
        );
    }

    /// Iteration 193: humiliation and moral injury are now dedicated causal
    /// inputs into paranoia and resentment — the two risk channels that were
    /// structurally dead at the sim call site (both hardcoded ZERO). A
    /// humiliated agent must accumulate paranoia + resentment; a morally
    /// injured agent (witnessing sacred violations) must accumulate
    /// resentment; control agents with neither stay at zero.
    #[test]
    fn humiliation_and_moral_injury_feed_paranoia_and_resentment() {
        let mut humiliated = PsychopathologyState::default();
        let mut injured = PsychopathologyState::default();
        let mut control = PsychopathologyState::default();
        for _ in 0..500 {
            humiliated.tick_update(
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::from_f64(0.5), // humiliation_recent
                Fixed::ZERO,
                Fixed::ZERO, // moral_injury
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
            );
            injured.tick_update(
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::from_f64(0.5), // moral_injury
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
            );
            control.tick_update(
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
            );
        }
        assert!(
            humiliated.paranoia_risk > Fixed::from_f64(0.01),
            "humiliation must feed paranoia, got {}",
            humiliated.paranoia_risk
        );
        assert!(
            humiliated.resentment_risk > Fixed::ZERO,
            "humiliation must feed resentment, got {}",
            humiliated.resentment_risk
        );
        assert!(
            injured.resentment_risk > Fixed::from_f64(0.01),
            "moral injury must feed resentment, got {}",
            injured.resentment_risk
        );
        assert_eq!(
            control.paranoia_risk,
            Fixed::ZERO,
            "control must stay at zero paranoia"
        );
        assert_eq!(
            control.resentment_risk,
            Fixed::ZERO,
            "control must stay at zero resentment"
        );
        // Both are secondary drivers: humiliation must not dominate the
        // paranoia channel beyond the documented weight.
        assert!(
            humiliated.paranoia_risk < Fixed::from_f64(0.5),
            "humiliation must not dominate paranoia, got {}",
            humiliated.paranoia_risk
        );
    }

    #[test]
    fn social_support_reduces_depression() {
        let mut p = PsychopathologyState::default();
        p.tick_update(
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::from_f64(0.9), // high support
            Fixed::ZERO,          // despair
        );
        // With high support, depression should be low
        assert!(p.depression_risk < Fixed::from_f64(0.1));
    }

    #[test]
    fn overall_health_reflects_worst_risk() {
        let mut p = PsychopathologyState {
            anxiety_risk: Fixed::from_f64(0.8),
            ..Default::default()
        };
        p.tick_update(
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ONE,
            Fixed::ZERO, // despair
        );
        assert!(p.overall_health < Fixed::from_f64(0.5));
    }

    /// Iteration 211: the graduated cognitive_modifier replaces the binary
    /// is_impaired() gate. Formula: returns overall_health directly.
    /// Fully healthy → 1.0, severely impaired → ~0.0.
    #[test]
    fn cognitive_modifier_equals_overall_health() {
        let mut p = PsychopathologyState::default();
        // Default: overall_health = 0.8, modifier should match
        assert_eq!(p.cognitive_modifier(), p.overall_health);
        // Impair the agent
        for _ in 0..500 {
            p.tick_update(
                Fixed::from_f64(0.8), // chronic stress
                Fixed::ZERO,
                Fixed::from_f64(0.8), // isolation
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::from_f64(0.5), // genetic depression
                Fixed::ZERO,
                Fixed::from_f64(0.2), // low support
                Fixed::ONE,           // despair
            );
        }
        // Modifier should now be < 0.8 and track overall_health
        assert!(p.cognitive_modifier() < Fixed::from_f64(0.8));
        assert_eq!(p.cognitive_modifier(), p.overall_health);
    }

    #[test]
    fn cognitive_modifier_is_graduated_not_binary() {
        // The modifier must be smooth: a partially impaired agent (health
        // = 0.7) gets a partial reduction, not the full 0.5× cliff.
        let p = PsychopathologyState {
            overall_health: Fixed::from_f64(0.7),
            ..Default::default()
        };
        assert_eq!(p.cognitive_modifier(), Fixed::from_f64(0.7));
        // The modifier should be between 0.5 and 1.0 (graduated, not binary)
        assert!(p.cognitive_modifier() > Fixed::from_f64(0.5));
        assert!(p.cognitive_modifier() < Fixed::ONE);
    }
}
