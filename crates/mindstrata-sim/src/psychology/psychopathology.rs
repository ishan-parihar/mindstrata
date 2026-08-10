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
    ) {
        // Depression: chronic stress + loss + isolation + genetic vulnerability
        let depression_input = chronic_stress * Fixed::from_f64(0.2)
            + loss_magnitude * Fixed::from_f64(0.3)
            + social_isolation * Fixed::from_f64(0.2)
            + genetic_depression_vulnerability * Fixed::from_f64(0.15)
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
        let dissociation_input = trauma_load * Fixed::from_f64(0.2)
            + chronic_pain * Fixed::from_f64(0.1);
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
        let worst_risk = self.depression_risk
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
    pub fn cognitive_modifier(&self) -> Fixed {
        // Mental health impairment reduces cognitive function
        (Fixed::ONE - self.overall_health * Fixed::from_f64(0.3)).clamp_01()
    }

    /// Compute modifier for social behavior from mental health state.
    pub fn social_modifier(&self) -> Fixed {
        // Depression and anxiety reduce social engagement
        let social_impact = self.depression_risk * Fixed::from_f64(0.3)
            + self.anxiety_risk * Fixed::from_f64(0.2)
            + self.paranoia_risk * Fixed::from_f64(0.2);
        (Fixed::ONE - social_impact).clamp_01()
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
        );
        assert!(p.depression_risk > Fixed::ZERO);
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
            Fixed::ZERO, Fixed::ZERO, Fixed::ZERO, Fixed::ZERO, Fixed::ZERO,
            Fixed::ZERO, Fixed::ZERO, Fixed::ZERO, Fixed::ZERO, Fixed::ZERO, Fixed::ONE,
        );
        assert!(p.overall_health < Fixed::from_f64(0.5));
    }
}
