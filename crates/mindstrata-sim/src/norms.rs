//! Norm system — institutional norms that agents internalize and evaluate.
//!
//! Norms define social rules with scope, strength, and punishment.
//! Agents evaluate compliance based on conformity, identity alignment,
//! and fear of punishment. Violations generate events and affect relationships.

use crate::person::IdentityKind;
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// A social norm that agents may internalize.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Norm {
    pub id: u64,
    pub name: String,
    /// How strongly the norm is enforced (0.0–1.0).
    pub strength: Fixed,
    /// How much agents tend to internalize it (0.0–1.0).
    pub internalization: Fixed,
    /// Punishment severity for violation (0.0–1.0).
    pub punishment: Fixed,
    /// Identity that reinforces this norm (optional).
    pub reinforcing_identity: Option<IdentityKind>,
}

/// A recorded norm violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormViolation {
    pub norm_id: u64,
    pub violator: AgentId,
    pub tick: u64,
}

/// The norm registry for the simulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NormRegistry {
    norms: Vec<Norm>,
    violations: Vec<NormViolation>,
}

impl NormRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            norms: Vec::new(),
            violations: Vec::new(),
        }
    }

    /// Register a norm.
    pub fn register(&mut self, norm: Norm) {
        self.norms.push(norm);
    }

    /// Get all norms.
    pub fn norms(&self) -> &[Norm] {
        &self.norms
    }

    /// Get all violations.
    pub fn violations(&self) -> &[NormViolation] {
        &self.violations
    }

    /// Record a violation.
    pub fn record_violation(&mut self, norm_id: u64, violator: AgentId, tick: u64) {
        self.violations.push(NormViolation {
            norm_id,
            violator,
            tick,
        });
    }

    /// Compute the norm pressure on an agent for a given action.
    /// Returns a value in [-1.0, 1.0]: negative = norm-compliant, positive = norm-violating.
    pub fn compute_pressure(
        &self,
        agent_conformity: Fixed,
        agent_identity_strength: Fixed,
        norm_id: u64,
    ) -> Fixed {
        if let Some(norm) = self.norms.iter().find(|n| n.id == norm_id) {
            // Pressure = violation tendency (positive = violating, negative = compliant)
            // conformity and internalization REDUCE violation → subtract
            // reinforcing identity REDUCES violation → subtract
            let base = norm.strength * (Fixed::ONE - agent_conformity);
            let identity_reduction = if norm.reinforcing_identity.is_some() {
                agent_identity_strength * Fixed::from_f64(0.3)
            } else {
                Fixed::ZERO
            };
            base - identity_reduction - norm.internalization
        } else {
            Fixed::ZERO
        }
    }

    /// Check if an action violates any norm and record violations.
    /// Returns the total punishment severity if violations occurred.
    pub fn check_violation(
        &mut self,
        norm_id: u64,
        violator: AgentId,
        tick: u64,
    ) -> Fixed {
        if let Some(norm) = self.norms.iter().find(|n| n.id == norm_id) {
            let punishment = norm.punishment;
            self.record_violation(norm_id, violator, tick);
            punishment
        } else {
            Fixed::ZERO
        }
    }
}

/// Create the default norms for a settlement.
pub fn default_norms() -> Vec<Norm> {
    vec![
        Norm {
            id: 0,
            name: "No Theft".into(),
            strength: Fixed::from_f64(0.8),
            internalization: Fixed::from_f64(0.6),
            punishment: Fixed::from_f64(0.5),
            reinforcing_identity: Some(IdentityKind::Farmer),
        },
        Norm {
            id: 1,
            name: "Help Neighbors".into(),
            strength: Fixed::from_f64(0.5),
            internalization: Fixed::from_f64(0.4),
            punishment: Fixed::from_f64(0.1),
            reinforcing_identity: Some(IdentityKind::Parent),
        },
        Norm {
            id: 2,
            name: "Respect Elders".into(),
            strength: Fixed::from_f64(0.6),
            internalization: Fixed::from_f64(0.5),
            punishment: Fixed::from_f64(0.2),
            reinforcing_identity: None,
        },
        Norm {
            id: 3,
            name: "Obey Ruler".into(),
            strength: Fixed::from_f64(0.7),
            internalization: Fixed::from_f64(0.4),
            punishment: Fixed::from_f64(0.6),
            reinforcing_identity: Some(IdentityKind::Believer),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norm_pressure_decreases_with_conformity() {
        let mut registry = NormRegistry::new();
        registry.register(Norm {
            id: 0,
            name: "No Theft".into(),
            strength: Fixed::from_f64(0.8),
            internalization: Fixed::from_f64(0.1),
            punishment: Fixed::from_f64(0.5),
            reinforcing_identity: None,
        });

        let low_conformity = Fixed::from_f64(0.2);
        let high_conformity = Fixed::from_f64(0.8);

        let p_low = registry.compute_pressure(low_conformity, Fixed::ZERO, 0);
        let p_high = registry.compute_pressure(high_conformity, Fixed::ZERO, 0);

        // High conformity means LESS violation pressure (more compliant)
        assert!(p_high < p_low, "High conformity should produce lower violation pressure");
    }

    #[test]
    fn identity_reduces_pressure() {
        let mut registry = NormRegistry::new();
        registry.register(Norm {
            id: 0,
            name: "Help Neighbors".into(),
            strength: Fixed::from_f64(0.6),
            internalization: Fixed::from_f64(0.3),
            punishment: Fixed::from_f64(0.2),
            reinforcing_identity: Some(IdentityKind::Parent),
        });

        let conformity = Fixed::from_f64(0.5);
        let no_identity = Fixed::ZERO;
        let has_identity = Fixed::from_f64(0.8);

        let p_no = registry.compute_pressure(conformity, no_identity, 0);
        let p_has = registry.compute_pressure(conformity, has_identity, 0);

        // Identity should reduce pressure (make it more negative / less positive)
        assert!(p_has < p_no, "Identity should reduce norm pressure");
    }

    #[test]
    fn violation_recorded() {
        let mut registry = NormRegistry::new();
        registry.register(Norm {
            id: 0,
            name: "No Theft".into(),
            strength: Fixed::from_f64(0.8),
            internalization: Fixed::from_f64(0.6),
            punishment: Fixed::from_f64(0.5),
            reinforcing_identity: None,
        });

        let punishment = registry.check_violation(0, AgentId::new(5), 100);
        assert!(punishment > Fixed::ZERO, "Punishment should be positive");
        assert_eq!(registry.violations().len(), 1);
        assert_eq!(registry.violations()[0].violator, AgentId::new(5));
    }
}
