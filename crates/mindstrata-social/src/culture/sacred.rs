//! §8.1.7: Sacred values — beliefs that resist cost-benefit tradeoff reasoning.
//!
//! Architecture §8.1.7: Sacred values are identity-linked beliefs that
//! resist change even when evidence contradicts them. They trigger strong
//! moral emotions when violated and create cognitive dissonance when
//! tradeoffs are attempted.
//!
//! ```text
//! Sacred value dynamics:
//!   - Violation triggers moral outrage proportional to sacredness
//!   - Tradeoff proposals trigger disgust (taboo tradeoff effect)
//!   - Identity-linked sacred values resist updating
//!   - Sacred values can be desacred through exposure and reasoning
//!   - Sacred values can be created through ritual and indoctrination
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A single sacred value held by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacredValue {
    /// Name of the sacred value (e.g., "family honor", "divine law").
    pub name: String,
    /// How sacred this value is (0 = profane, 1 = absolutely sacred).
    pub sacredness: Fixed,
    /// How linked to identity this value is (0 = peripheral, 1 = core identity).
    pub identity_linkage: Fixed,
    /// Emotional charge when violated (0–1).
    pub emotional_charge: Fixed,
    /// Resistance to cost-benefit reasoning (0 = negotiable, 1 = non-negotiable).
    pub tradeoff_resistance: Fixed,
}

impl SacredValue {
    /// Create a new sacred value.
    pub fn new(name: String, sacredness: Fixed, identity_linkage: Fixed) -> Self {
        let emotional_charge = sacredness * Fixed::from_f64(0.8);
        let tradeoff_resistance = (sacredness * Fixed::from_f64(0.9)
            + identity_linkage * Fixed::from_f64(0.1))
        .clamp_01();
        Self {
            name,
            sacredness,
            identity_linkage,
            emotional_charge,
            tradeoff_resistance,
        }
    }

    /// Compute moral outrage when this value is violated.
    pub fn violation_outrage(&self, violation_severity: Fixed) -> Fixed {
        (self.emotional_charge * violation_severity * Fixed::from_f64(0.5)
            + self.identity_linkage * violation_severity * Fixed::from_f64(0.3))
        .clamp_01()
    }

    /// Compute resistance to updating this value when presented with evidence.
    pub fn update_resistance(&self) -> Fixed {
        (self.sacredness * Fixed::from_f64(0.4)
            + self.identity_linkage * Fixed::from_f64(0.3)
            + self.tradeoff_resistance * Fixed::from_f64(0.3))
        .clamp_01()
    }

    /// Attempt to desacred this value through evidence or reasoning.
    ///
    /// Returns true if the value was desacred.
    pub fn attempt_desacred(
        &mut self,
        evidence_strength: Fixed,
        reasoning_capacity: Fixed,
    ) -> bool {
        let resistance = self.update_resistance();
        let desacred_pressure = evidence_strength * reasoning_capacity * (Fixed::ONE - resistance);
        if desacred_pressure > Fixed::from_f64(0.1) {
            self.sacredness =
                (self.sacredness - desacred_pressure * Fixed::from_f64(0.05)).max(Fixed::ZERO);
            self.emotional_charge = self.sacredness * Fixed::from_f64(0.8);
            self.tradeoff_resistance = (self.sacredness * Fixed::from_f64(0.9)
                + self.identity_linkage * Fixed::from_f64(0.1))
            .clamp_01();
            self.sacredness < Fixed::from_f64(0.1) // desacred if below threshold
        } else {
            false
        }
    }
}

/// A set of sacred values held by an agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SacredValues {
    /// Values. (doc added at S3 extraction)
    pub values: Vec<SacredValue>,
}

impl SacredValues {
    /// Total moral outrage from all sacred values given a set of violations.
    pub fn total_violation_outrage(&self, violation_severity: Fixed) -> Fixed {
        if self.values.is_empty() {
            return Fixed::ZERO;
        }
        let total: Fixed = self
            .values
            .iter()
            .map(|v| v.violation_outrage(violation_severity))
            .fold(Fixed::ZERO, |acc, o| acc + o);
        // Normalize by number of values
        (total / Fixed::from_int(self.values.len() as i64)).clamp_01()
    }

    /// Find a sacred value by name.
    pub fn find(&self, name: &str) -> Option<&SacredValue> {
        self.values.iter().find(|v| v.name == name)
    }

    /// Amplify a witnessed violation by how sacred the agent's values are.
    ///
    /// §8.1.7: "Violation triggers moral outrage proportional to sacredness."
    /// Returns the witnessed-violation magnitude fed into moral cognition —
    /// higher sacredness amplifies the same observed violation into more
    /// outrage. Zero-at-zero anchor: no violations -> no amplification, so
    /// typical calm agents are unaffected and only moral events diverge.
    pub fn amplify_witnessed_violations(&self, witnessed: Fixed) -> Fixed {
        if witnessed <= Fixed::ZERO {
            return Fixed::ZERO;
        }
        (witnessed + self.total_violation_outrage(witnessed) * Fixed::from_f64(0.5)).clamp_01()
    }

    /// Add a sacred value (or strengthen existing).
    pub fn add_or_strengthen(&mut self, name: String, sacredness: Fixed, identity_linkage: Fixed) {
        if let Some(existing) = self.values.iter_mut().find(|v| v.name == name) {
            existing.sacredness =
                (existing.sacredness + sacredness * Fixed::from_f64(0.1)).clamp_01();
            existing.identity_linkage =
                (existing.identity_linkage + identity_linkage * Fixed::from_f64(0.1)).clamp_01();
        } else {
            self.values
                .push(SacredValue::new(name, sacredness, identity_linkage));
        }
    }

    /// Desacralize through exposure and reasoning — the §8.1.7 lifecycle
    /// promise: "sacred values can be desacred through exposure and
    /// reasoning." Applies the same evidence × reasoning pressure to every
    /// held value; each value's internal `> 0.1` gate keeps very sacred
    /// values (high resistance) inert while mid-sacredness values erode a
    /// little per exposure — a slow secularization drift, not a flip.
    ///
    /// Returns the number of values that were fully desacred (sacredness
    /// dropped below the 0.1 threshold). Zero-at-zero anchor: zero evidence
    /// or zero reasoning yields zero pressure, so typical calm agents are
    /// untouched; only sustained high-absorption learning by high-reasoning
    /// agents secularizes them.
    pub fn desacralize_through_exposure(
        &mut self,
        evidence_strength: Fixed,
        reasoning_capacity: Fixed,
    ) -> usize {
        let mut fully_desacred = 0;
        for value in &mut self.values {
            if value.attempt_desacred(evidence_strength, reasoning_capacity) {
                fully_desacred += 1;
            }
        }
        fully_desacred
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sacred_value_outrage_on_violation() {
        let sv = SacredValue::new("honor".into(), Fixed::from_f64(0.9), Fixed::from_f64(0.8));
        let outrage = sv.violation_outrage(Fixed::from_f64(0.7));
        assert!(outrage > Fixed::from_f64(0.2));
    }

    #[test]
    fn sacred_value_resists_update() {
        let sv = SacredValue::new(
            "divine_law".into(),
            Fixed::from_f64(0.95),
            Fixed::from_f64(0.9),
        );
        assert!(sv.update_resistance() > Fixed::from_f64(0.5));
    }

    #[test]
    fn desacred_reduces_sacredness() {
        let mut sv = SacredValue::new("test".into(), Fixed::from_f64(0.5), Fixed::from_f64(0.3));
        let initial = sv.sacredness;
        sv.attempt_desacred(Fixed::from_f64(0.8), Fixed::from_f64(0.8));
        assert!(sv.sacredness < initial);
    }

    #[test]
    fn desacred_is_zero_at_zero_evidence() {
        let mut sv = SacredValue::new("test".into(), Fixed::from_f64(0.5), Fixed::from_f64(0.3));
        let initial = sv.sacredness;
        // No evidence (or no reasoning capacity) must not move sacredness.
        sv.attempt_desacred(Fixed::ZERO, Fixed::from_f64(0.8));
        assert_eq!(sv.sacredness, initial, "zero evidence must not desacred");
        sv.attempt_desacred(Fixed::from_f64(0.8), Fixed::ZERO);
        assert_eq!(sv.sacredness, initial, "zero reasoning must not desacred");
    }

    #[test]
    fn very_sacred_value_resists_desacralization() {
        // A maximally sacred, identity-linked value has resistance ~0.9,
        // so even strong evidence × reasoning stays under the gate.
        let mut sv = SacredValue::new(
            "divine_law".into(),
            Fixed::from_f64(0.95),
            Fixed::from_f64(0.9),
        );
        let initial = sv.sacredness;
        sv.attempt_desacred(Fixed::from_f64(1.0), Fixed::from_f64(1.0));
        assert_eq!(
            sv.sacredness, initial,
            "very sacred values resist secularization"
        );
    }

    #[test]
    fn desacralize_through_exposure_applies_to_all_values() {
        let mut sv = SacredValues::default();
        sv.add_or_strengthen(
            "mid_sacred".into(),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.3),
        );
        sv.add_or_strengthen(
            "very_sacred".into(),
            Fixed::from_f64(0.95),
            Fixed::from_f64(0.9),
        );
        let mid_before = sv.values[0].sacredness;
        let sacred_before = sv.values[1].sacredness;
        let fully = sv.desacralize_through_exposure(Fixed::from_f64(0.8), Fixed::from_f64(0.8));
        assert!(
            sv.values[0].sacredness < mid_before,
            "mid-sacredness value erodes"
        );
        assert_eq!(
            sv.values[1].sacredness, sacred_before,
            "very sacred value resists"
        );
        // Pressure is too small to fully desacred a 0.5 value in one shot.
        assert_eq!(fully, 0);
    }

    #[test]
    fn desacralize_through_exposure_counts_fully_desacred() {
        let mut sv = SacredValues::default();
        sv.add_or_strengthen(
            "nearly_profane".into(),
            Fixed::from_f64(0.12),
            Fixed::from_f64(0.1),
        );
        let fully = sv.desacralize_through_exposure(Fixed::from_f64(1.0), Fixed::from_f64(1.0));
        assert_eq!(
            fully, 1,
            "a value pushed below 0.1 sacredness counts as desacred"
        );
    }

    #[test]
    fn sacred_values_total_outrage() {
        let mut sv = SacredValues::default();
        sv.add_or_strengthen("honor".into(), Fixed::from_f64(0.8), Fixed::from_f64(0.7));
        sv.add_or_strengthen("family".into(), Fixed::from_f64(0.9), Fixed::from_f64(0.9));
        let outrage = sv.total_violation_outrage(Fixed::from_f64(0.5));
        assert!(outrage > Fixed::ZERO);
    }

    #[test]
    fn amplification_is_zero_at_zero_violations() {
        let mut sv = SacredValues::default();
        sv.add_or_strengthen("honor".into(), Fixed::from_f64(0.9), Fixed::from_f64(0.9));
        assert_eq!(sv.amplify_witnessed_violations(Fixed::ZERO), Fixed::ZERO);
    }

    #[test]
    fn amplification_is_monotone_in_sacredness() {
        let mut low = SacredValues::default();
        low.add_or_strengthen("honor".into(), Fixed::from_f64(0.1), Fixed::from_f64(0.1));
        let mut high = SacredValues::default();
        high.add_or_strengthen("honor".into(), Fixed::from_f64(0.9), Fixed::from_f64(0.9));
        let witnessed = Fixed::from_f64(0.4);
        assert!(
            high.amplify_witnessed_violations(witnessed)
                > low.amplify_witnessed_violations(witnessed)
        );
        // Amplification never shrinks the witnessed magnitude.
        assert!(low.amplify_witnessed_violations(witnessed) >= witnessed);
    }
}
