//! Executive function and metacognition — the "CEO" of the mind.
//!
//! Executive function governs:
//! - Working memory capacity (what can be held in mind simultaneously)
//! - Inhibition (suppressing impulsive or inappropriate responses)
//! - Cognitive flexibility (switching between strategies)
//! - Planning depth (how many steps ahead the agent can consider)
//! - Self-monitoring (detecting errors and adjusting behavior)
//!
//! Stress, fatigue, pain, and trauma reduce executive function.
//! High executive function enables:
//! - Long-term planning
//! - Deception, diplomacy
//! - Skill mastery
//! - Emotional regulation
//! - Institutional leadership
//!
//! ```text
//! executive_capacity =
//!     base_capacity
//!   × (1 - stress × stress_penalty)
//!   × (1 - fatigue × fatigue_penalty)
//!   × (1 - pain × pain_penalty)
//!   × (1 - trauma × trauma_penalty)
//!   × neuroplasticity_bonus
//! ```
//!
//! Emergent effects:
//! - Sleep-deprived agents make impulsive decisions
//! - Traumatized agents struggle with inhibition
//! - High-EQ agents can plan long-term social strategies
//! - Stressed leaders make worse institutional decisions

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Executive function state — the metacognitive control system.
///
/// Governs working memory, inhibition, flexibility, and planning capacity.
/// Degrades under stress, fatigue, pain, and trauma.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveRuntime {
    /// Base working memory capacity (0–1). How many items the agent can juggle.
    pub working_memory_capacity: Fixed,
    /// Base inhibition strength (0–1). Personality-derived, stable.
    pub inhibition: Fixed,
    /// Base cognitive flexibility (0–1). Personality-derived, stable.
    pub flexibility: Fixed,
    /// Planning depth — how many steps ahead the agent can reason.
    pub planning_depth: Fixed,
    /// Self-monitoring ability (0–1). Detecting errors and adjusting.
    pub self_monitoring: Fixed,
    /// Error sensitivity (0–1). How strongly errors trigger corrective action.
    pub error_sensitivity: Fixed,
    /// Current effective capacity after degradation factors applied.
    pub effective_capacity: Fixed,
    /// Base impulsivity (0–1). Personality-derived, stable baseline.
    pub impulsivity: Fixed,
    /// Current effective inhibition after stress degradation (0–1).
    pub effective_inhibition: Fixed,
    /// Current effective flexibility after degradation (0–1).
    pub effective_flexibility: Fixed,
}

impl Default for CognitiveRuntime {
    fn default() -> Self {
        let mut s = Self {
            working_memory_capacity: Fixed::from_f64(0.5),
            inhibition: Fixed::from_f64(0.5),
            flexibility: Fixed::from_f64(0.5),
            planning_depth: Fixed::from_f64(0.5),
            self_monitoring: Fixed::from_f64(0.4),
            error_sensitivity: Fixed::from_f64(0.5),
            effective_capacity: Fixed::ZERO, // computed below
            impulsivity: Fixed::from_f64(0.3),
            effective_inhibition: Fixed::ZERO, // computed below
            effective_flexibility: Fixed::ZERO, // computed below
        };
        s.recompute(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO, Fixed::ZERO);
        s
    }
}

impl CognitiveRuntime {
    /// Create from personality traits for use in AgentBundle population.
    pub fn from_personality(personality: &crate::person::Personality) -> Self {
        let base = (personality.conscientiousness + personality.openness) / Fixed::from_int(2);
        let inhibition = (personality.conscientiousness * Fixed::from_f64(0.6)
            + (Fixed::ONE - personality.impulsivity) * Fixed::from_f64(0.4))
            .clamp_01();
        let flexibility = (personality.openness * Fixed::from_f64(0.6)
            + personality.extraversion * Fixed::from_f64(0.4))
            .clamp_01();
        let planning = (personality.conscientiousness * Fixed::from_f64(0.5)
            + personality.openness * Fixed::from_f64(0.3)
            + (Fixed::ONE - personality.impulsivity) * Fixed::from_f64(0.2))
            .clamp_01();
        let mut s = Self {
            working_memory_capacity: base,
            inhibition,
            flexibility,
            planning_depth: planning,
            self_monitoring: (personality.conscientiousness * Fixed::from_f64(0.6)
                + personality.openness * Fixed::from_f64(0.4))
                .clamp_01(),
            error_sensitivity: (personality.neuroticism * Fixed::from_f64(0.4)
                + personality.conscientiousness * Fixed::from_f64(0.6))
                .clamp_01(),
            effective_capacity: Fixed::ZERO, // computed below
            impulsivity: personality.impulsivity,
            effective_inhibition: Fixed::ZERO, // computed below
            effective_flexibility: Fixed::ZERO, // computed below
        };
        s.recompute(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO, Fixed::ZERO);
        s
    }

    /// Update executive function based on degradation factors.
    ///
    /// Called each tick with current stress, fatigue, pain, and trauma levels.
    /// Recomputes effective capacity, inhibition, flexibility, and impulsivity.
    ///
    /// All effective fields are dampened signals that track current state,
    /// not accumulators — they recover when stress drops.
    pub fn update(
        &mut self,
        stress: Fixed,
        fatigue: Fixed,
        pain: Fixed,
        trauma: Fixed,
    ) {
        self.recompute(stress, fatigue, pain, trauma);
    }

    /// Recompute all effective fields from degradation factors.
    ///
    /// Effective inhibition tracks stress-driven degradation of the base:
    /// ```text
    /// stress_load = stress×0.3 + fatigue×0.2 + pain×0.15
    /// effective_inhibition = inhibition × (1 - stress_load)
    /// effective_flexibility = flexibility × (1 - fatigue×0.25 - pain×0.1)
    /// effective_capacity = wm × (1-stress×0.3) × (1-fatigue×0.25)
    ///                    × (1-pain×0.2) × (1-trauma×0.15)
    /// impulsivity = base_impulsivity × (1 - effective_inhibition)
    ///            + stress_load × 0.3  // stress-driven impulsivity
    /// ```
    fn recompute(
        &mut self,
        stress: Fixed,
        fatigue: Fixed,
        pain: Fixed,
        trauma: Fixed,
    ) {
        let stress_load = stress * Fixed::from_f64(0.3)
            + fatigue * Fixed::from_f64(0.2)
            + pain * Fixed::from_f64(0.15);

        // Effective inhibition: base inhibited by stress (recovers when stress drops)
        self.effective_inhibition = (self.inhibition * (Fixed::ONE - stress_load)).clamp_01();

        // Effective flexibility: base inhibited by fatigue and pain
        let flex_degradation = fatigue * Fixed::from_f64(0.25)
            + pain * Fixed::from_f64(0.1);
        self.effective_flexibility = (self.flexibility * (Fixed::ONE - flex_degradation)).clamp_01();

        // Effective capacity: multiplicative degradation from all factors
        self.effective_capacity = self.compute_effective_capacity(
            stress, fatigue, pain, trauma,
        );

        // Impulsivity: base trait amplified by stress, decays when calm.
        // The recovery rate ensures impulsivity returns toward baseline when stress drops.
        let stress_push = stress_load * Fixed::from_f64(0.15);
        let recovery = (Fixed::ONE - stress_load) * Fixed::from_f64(0.08);
        self.impulsivity = (self.impulsivity
            + stress_push - recovery)
            .clamp_01();
    }

    /// Compute effective capacity from degradation factors.
    ///
    /// ```text
    /// effective = base × (1 - stress×0.3) × (1 - fatigue×0.25)
    ///           × (1 - pain×0.2) × (1 - trauma×0.15)
    /// ```
    fn compute_effective_capacity(
        &self,
        stress: Fixed,
        fatigue: Fixed,
        pain: Fixed,
        trauma: Fixed,
    ) -> Fixed {
        let base = self.working_memory_capacity;
        let stress_factor = Fixed::ONE - stress * Fixed::from_f64(0.3);
        let fatigue_factor = Fixed::ONE - fatigue * Fixed::from_f64(0.25);
        let pain_factor = Fixed::ONE - pain * Fixed::from_f64(0.2);
        let trauma_factor = Fixed::ONE - trauma * Fixed::from_f64(0.15);
        (base * stress_factor * fatigue_factor * pain_factor * trauma_factor).clamp_01()
    }

    /// Can the agent engage in long-term planning right now?
    ///
    /// Returns true if effective capacity and planning depth are above threshold.
    pub fn can_plan_long_term(&self) -> bool {
        self.effective_capacity > Fixed::from_f64(0.4)
            && self.planning_depth > Fixed::from_f64(0.3)
    }

    /// Can the agent resist an impulse right now?
    pub fn can_inhibit(&self) -> bool {
        self.effective_inhibition > Fixed::from_f64(0.3)
    }

    /// Can the agent switch strategies flexibly right now?
    pub fn can_switch_strategy(&self) -> bool {
        self.effective_flexibility > Fixed::from_f64(0.3)
    }

    /// Effective planning depth after degradation.
    pub fn effective_planning_depth(&self) -> Fixed {
        (self.planning_depth * self.effective_capacity).clamp_01()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_sane_values() {
        let cr = CognitiveRuntime::default();
        assert!(cr.effective_capacity > Fixed::ZERO);
        assert!(cr.inhibition > Fixed::ZERO);
        assert!(cr.working_memory_capacity > Fixed::ZERO);
    }

    #[test]
    fn stress_reduces_effective_capacity() {
        let mut cr = CognitiveRuntime::default();
        let baseline = cr.effective_capacity;
        cr.update(Fixed::from_f64(0.8), Fixed::ZERO, Fixed::ZERO, Fixed::ZERO);
        assert!(cr.effective_capacity < baseline);
    }

    #[test]
    fn fatigue_reduces_effective_capacity() {
        let mut cr = CognitiveRuntime::default();
        let baseline = cr.effective_capacity;
        cr.update(Fixed::ZERO, Fixed::from_f64(0.9), Fixed::ZERO, Fixed::ZERO);
        assert!(cr.effective_capacity < baseline);
    }

    #[test]
    fn stress_increases_impulsivity() {
        let mut cr = CognitiveRuntime::default();
        let baseline = cr.impulsivity;
        cr.update(
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
        );
        assert!(cr.impulsivity > baseline, "stress should increase impulsivity");
    }

    #[test]
    fn impulsivity_recovers_when_stress_drops() {
        let mut cr = CognitiveRuntime::default();
        // Stress for multiple ticks to push impulsivity up
        for _ in 0..10 {
            cr.update(
                Fixed::from_f64(0.9),
                Fixed::from_f64(0.9),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.5),
            );
        }
        let stressed = cr.impulsivity;
        // Recover: stress drops to zero for several ticks
        for _ in 0..20 {
            cr.update(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO, Fixed::ZERO);
        }
        let recovered = cr.impulsivity;
        assert!(recovered < stressed, "impulsivity should decrease when stress drops");
    }

    #[test]
    fn effective_inhibition_tracks_stress() {
        let mut cr = CognitiveRuntime::default();
        let full = cr.effective_inhibition;
        cr.update(Fixed::from_f64(0.8), Fixed::ZERO, Fixed::ZERO, Fixed::ZERO);
        assert!(cr.effective_inhibition < full);
        // Recover: inhibition should bounce back toward base
        cr.update(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO, Fixed::ZERO);
        assert!(cr.effective_inhibition > Fixed::from_f64(0.3));
    }

    #[test]
    fn effective_flexibility_used_by_can_switch() {
        // Use low base flexibility so degradation crosses the threshold
        let mut cr = CognitiveRuntime::default();
        cr.flexibility = Fixed::from_f64(0.4);
        cr.update(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO, Fixed::ZERO);
        assert!(cr.can_switch_strategy());
        // fatigue=1.0, pain=1.0 → degradation = 1.0*0.25 + 1.0*0.1 = 0.35
        // effective_flex = 0.4 * (1 - 0.35) = 0.26 < 0.3 → cannot switch
        cr.update(Fixed::ZERO, Fixed::ONE, Fixed::ONE, Fixed::ZERO);
        assert!(!cr.can_switch_strategy(),
            "effective_flexibility={:?}", cr.effective_flexibility);
    }

    #[test]
    fn planning_blocked_when_capacity_low() {
        let mut cr = CognitiveRuntime::default();
        assert!(cr.can_plan_long_term());
        cr.update(Fixed::from_f64(0.9), Fixed::from_f64(0.9), Fixed::ZERO, Fixed::ZERO);
        assert!(!cr.can_plan_long_term());
    }

    #[test]
    fn from_personality_conscientiousness_boosts_inhibition() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut p = crate::person::Personality::random(&mut rng);
        p.conscientiousness = Fixed::from_f64(0.9);
        p.impulsivity = Fixed::from_f64(0.1);
        let cr = CognitiveRuntime::from_personality(&p);
        assert!(cr.inhibition > Fixed::from_f64(0.5));
    }

    #[test]
    fn effective_planning_depth_scales_with_capacity() {
        let mut cr = CognitiveRuntime::default();
        let full_depth = cr.effective_planning_depth();
        cr.update(Fixed::from_f64(0.8), Fixed::from_f64(0.8), Fixed::ZERO, Fixed::ZERO);
        assert!(cr.effective_planning_depth() < full_depth);
    }

    #[test]
    fn values_stay_bounded() {
        let mut cr = CognitiveRuntime::default();
        for _ in 0..100 {
            cr.update(
                Fixed::from_f64(0.9),
                Fixed::from_f64(0.9),
                Fixed::from_f64(0.9),
                Fixed::from_f64(0.9),
            );
        }
        assert!(cr.effective_capacity <= Fixed::ONE);
        assert!(cr.impulsivity <= Fixed::ONE);
    }
}
