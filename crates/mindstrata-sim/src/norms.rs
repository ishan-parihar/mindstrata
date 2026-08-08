//! Norm system — institutional norms that agents internalize and evaluate.
//!
//! Norms define social rules with scope, strength, and punishment.
//! Agents evaluate compliance based on conformity, identity alignment,
//! and fear of punishment. Violations generate events and affect relationships.
//!
//! §19.5.D Legal/Normative Layer:
//! - CrimeRecord tracks repeat offenders per agent
//! - Enforcement probability: not all violations are caught (based on enforcement capacity)
//! - Escalating punishment: repeat offenders face harsher consequences

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
    /// §19.5.D: Actual punishment applied after escalation.
    pub punishment_applied: Fixed,
}

/// §19.5.D: The default "No Violence" norm id (index 4 in `default_norms`).
/// Hostile interactions (Threaten/Insult) are direct confrontations checked
/// against this norm by the Iteration-78 enforcement wiring.
pub const NO_VIOLENCE_NORM_ID: u64 = 4;

/// §19.5.D: The default "No Theft" norm id (index 0 in `default_norms`).
/// Resource thefts are checked against this norm inside `enforce_theft`
/// (Iteration-78-era wiring) and, from Iteration 84, the theft *decision*
/// itself is gated by the agent's internalized strength of this norm
/// (`no_theft_resistance` in the sim).
pub const NO_THEFT_NORM_ID: u64 = 0;

/// §8.1.10 (Iteration 89): The prescriptive "Help Neighbors" norm id
/// (index 1 in `default_norms`). Unlike the prohibitive norms, an
/// internalized Help Neighbors norm *amplifies* the prosocial Help
/// decision in the interaction system rather than suppressing an act.
pub const HELP_NEIGHBORS_NORM_ID: u64 = 1;

/// §8.1.10 (Iteration 91): The prescriptive "Respect Elders" norm id
/// (index 2 in `default_norms`). An internalized norm suppresses
/// disrespectful acts (threat/insult) toward the community's designated
/// elder — the Council "Elder" role holder, the live elder anchor (age-based
/// elders > 60 never appear at the sim's timescale: 35040 ticks/year, so
/// test horizons stay below ~56 years).
pub const RESPECT_ELDERS_NORM_ID: u64 = 2;

/// §19.5.D (Iteration 78): Episode window for notoriety — at most one
/// notoriety increment per agent per window, so notoriety tracks distinct
/// conflict *episodes* rather than raw hostile-act counts. A conflict-prone
/// village produces hundreds of hostile acts per agent over 2000 ticks
/// (probe: 57–316), which saturated notoriety at 1.0 for everyone and
/// erased the differentiation the social-cost channel needs. `offense_count`
/// (the punishment multiplier driver) is deliberately NOT gated — keeping
/// every punishment trajectory byte-identical to the pre-existing baseline
/// (fines → wealth_rank, trust erosion → relationship metrics are all
/// snapshot-hashed, so gating them would drift the golden baselines).
pub const ENFORCEMENT_EPISODE_TICKS: u64 = 400;

/// §19.5.D: Crime record tracking repeat offenders.
/// Each agent accumulates a criminal history that escalates punishment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrimeRecord {
    /// Number of times this agent has committed crimes. Drives the
    /// punishment multiplier — grows on every offense, ungated, so the
    /// punishment trajectory stays byte-identical to the baseline.
    pub offense_count: u32,
    /// Last tick when a crime was committed. Pre-existing field kept for
    /// API/format stability; the Iteration-78 episode gating tracks
    /// `last_episode_tick` instead (no production readers remain).
    pub last_offense_tick: u64,
    /// Distinct conflict episodes (one per `ENFORCEMENT_EPISODE_TICKS`
    /// window). Drives notoriety so it stays a differentiating 0–0.9 signal.
    pub episode_count: u32,
    /// Last tick when a new episode began (for episode gating).
    pub last_episode_tick: u64,
    /// Accumulated notoriety (0.0–1.0). Higher = more feared/known criminal.
    pub notoriety: Fixed,
}

impl CrimeRecord {
    /// Record a new offense and update notoriety.
    /// `offense_count` grows every call (punishment trajectory unchanged);
    /// notoriety grows once per distinct episode window.
    pub fn record_offense(&mut self, tick: u64) {
        self.offense_count += 1;
        self.last_offense_tick = tick;
        let new_episode = self.episode_count == 0
            || tick.saturating_sub(self.last_episode_tick)
                >= ENFORCEMENT_EPISODE_TICKS;
        if new_episode {
            self.episode_count += 1;
            self.last_episode_tick = tick;
        }
        // Notoriety grows with distinct episodes, diminishing returns.
        self.notoriety = Fixed::from_f64(
            (1.0 - (-0.5 * self.episode_count as f64).exp()).min(1.0),
        );
    }

    /// Compute punishment multiplier based on criminal history.
    /// First offense = 1.0x, repeat offenders get escalating punishment.
    /// Maximum 3x multiplier for habitual criminals.
    pub fn punishment_multiplier(&self) -> Fixed {
        // 0.5 + 0.5 * offense_count → first=1.0, second=1.5, third=2.0, etc.
        // Capped at 3.0
        let mult = 0.5 + 0.5 * self.offense_count as f64;
        Fixed::from_f64(mult.min(3.0))
    }
}

/// The norm registry for the simulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NormRegistry {
    norms: Vec<Norm>,
    violations: Vec<NormViolation>,
    /// §19.5.D: Per-agent crime records for escalating punishment.
    crime_records: Vec<(AgentId, CrimeRecord)>,
}

impl NormRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            norms: Vec::new(),
            violations: Vec::new(),
            crime_records: Vec::new(),
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

    /// Get a reference to the crime records.
    pub fn crime_records(&self) -> &[(AgentId, CrimeRecord)] {
        &self.crime_records
    }

    /// Ensure a crime record exists for an agent, returning its index.
    fn ensure_record_index(&mut self, agent: AgentId) -> usize {
        if let Some((idx, _)) = self.crime_records.iter().enumerate().find(|(_, (a, _))| *a == agent) {
            return idx;
        }
        self.crime_records.push((agent, CrimeRecord::default()));
        self.crime_records.len() - 1
    }

    /// Get a read-only reference to an agent's crime record.
    pub fn crime_record(&self, agent: AgentId) -> Option<&CrimeRecord> {
        self.crime_records
            .iter()
            .find(|(a, _)| *a == agent)
            .map(|(_, r)| r)
    }

    /// Record a violation with known punishment amount.
    pub fn record_violation_with_punishment(&mut self, norm_id: u64, violator: AgentId, tick: u64, punishment_applied: Fixed) {
        self.violations.push(NormViolation {
            norm_id,
            violator,
            tick,
            punishment_applied,
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

    /// §19.5.D: Check if an action violates a norm, applying enforcement probability
    /// and escalating punishment based on criminal history.
    ///
    /// - `enforcement_capacity`: how likely the violation is to be detected (0.0–1.0)
    /// - `detection_roll`: random value [0,1) — if < enforcement_capacity, caught
    /// - Returns (punishment_amount, was_caught): punishment only applied if caught
    pub fn check_violation_with_enforcement(
        &mut self,
        norm_id: u64,
        violator: AgentId,
        tick: u64,
        enforcement_capacity: Fixed,
        detection_roll: Fixed,
    ) -> (Fixed, bool) {
        // §19.5.D: Enforcement probability — not all violations are caught
        let detected = detection_roll < enforcement_capacity;

        if let Some(norm) = self.norms.iter().find(|n| n.id == norm_id) {
            if detected {
                // §19.5.D: Escalating punishment based on criminal history
                let base_punishment = norm.punishment;
                let idx = self.ensure_record_index(violator);
                self.crime_records[idx].1.record_offense(tick);
                let multiplier = self.crime_records[idx].1.punishment_multiplier();
                let actual_punishment = (base_punishment * multiplier).min(Fixed::ONE);

                // Record the violation
                self.violations.push(NormViolation {
                    norm_id,
                    violator,
                    tick,
                    punishment_applied: actual_punishment,
                });

                (actual_punishment, true)
            } else {
                // Violation occurred but was not detected — no punishment, no record
                (Fixed::ZERO, false)
            }
        } else {
            (Fixed::ZERO, false)
        }
    }

    /// Check if an action violates any norm and record violations.
    /// Always records the violation (no enforcement probability check).
    /// Use for violations that are always witnessed (e.g., direct confrontation).
    /// Returns the total punishment severity after escalation.
    pub fn check_violation(
        &mut self,
        norm_id: u64,
        violator: AgentId,
        tick: u64,
    ) -> Fixed {
        if let Some(norm) = self.norms.iter().find(|n| n.id == norm_id) {
            let base_punishment = norm.punishment;
            let idx = self.ensure_record_index(violator);
            self.crime_records[idx].1.record_offense(tick);
            let multiplier = self.crime_records[idx].1.punishment_multiplier();
            let actual_punishment = (base_punishment * multiplier).min(Fixed::ONE);

            self.violations.push(NormViolation {
                norm_id,
                violator,
                tick,
                punishment_applied: actual_punishment,
            });

            actual_punishment
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
        Norm {
            id: 4,
            name: "No Violence".into(),
            strength: Fixed::from_f64(0.9),
            internalization: Fixed::from_f64(0.7),
            punishment: Fixed::from_f64(0.8),
            reinforcing_identity: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_violation_escalates_notoriety_and_punishment() {
        let mut registry = NormRegistry::new();
        registry.register(Norm {
            id: NO_VIOLENCE_NORM_ID,
            name: "No Violence".into(),
            strength: Fixed::from_f64(0.9),
            internalization: Fixed::from_f64(0.7),
            punishment: Fixed::from_f64(0.8),
            reinforcing_identity: None,
        });
        let violator = AgentId::new(3);
        // First offense: punishment 0.8 × 1.0 multiplier, notoriety 1−e^−0.5.
        let p1 = registry.check_violation(NO_VIOLENCE_NORM_ID, violator, 100);
        assert_eq!(p1, Fixed::from_f64(0.8));
        // Third offense across three distinct episodes (spaced beyond the
        // 400-tick episode window): multiplier 2.0 → 0.8 × 2.0 = 1.6, capped
        // at 1.0 (the punishment cap); notoriety 1−e^−1.5 ≈ 0.78. The
        // escalation is observable in offense_count/multiplier/notoriety.
        registry.check_violation(NO_VIOLENCE_NORM_ID, violator, 600);
        let p3 = registry.check_violation(NO_VIOLENCE_NORM_ID, violator, 1100);
        assert_eq!(p3, Fixed::ONE);
        let record = registry.crime_record(violator).unwrap();
        assert_eq!(record.offense_count, 3);
        assert!(record.notoriety > Fixed::from_f64(0.7));
        assert_eq!(registry.violations().len(), 3);
    }

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

    #[test]
    fn notoriety_gates_per_episode_but_offense_count_grows() {
        let mut record = CrimeRecord::default();

        // First offense at tick 100 opens an episode window.
        record.record_offense(100);
        assert_eq!(record.offense_count, 1);
        assert_eq!(record.episode_count, 1);

        // Rapid repeat (tick 150, within the 400-tick window): offense_count
        // grows (punishment trajectory unchanged) but notoriety does not
        // (a new episode has not begun).
        let notoriety_after_first = record.notoriety;
        record.record_offense(150);
        assert_eq!(record.offense_count, 2);
        assert_eq!(record.episode_count, 1);
        assert_eq!(record.notoriety, notoriety_after_first);

        // A fresh episode after the window elapses increments both.
        record.record_offense(510);
        assert_eq!(record.offense_count, 3);
        assert_eq!(record.episode_count, 2);
        assert!(record.notoriety > notoriety_after_first);
    }

    #[test]
    fn notoriety_differentiates_across_episode_counts() {
        // A one-episode offender stays well below a chronic one, keeping
        // notoriety a discriminating signal for the social-cost channel
        // (Iteration 78) rather than saturating at 1.0 for everyone.
        let mut occasional = CrimeRecord::default();
        occasional.record_offense(100);
        occasional.record_offense(600);

        let mut chronic = CrimeRecord::default();
        for tick in [100u64, 600, 1100, 1600] {
            chronic.record_offense(tick);
        }

        assert!(chronic.notoriety > occasional.notoriety);
        assert!(occasional.notoriety < Fixed::from_f64(0.9));
        assert!(chronic.notoriety < Fixed::ONE, "notoriety stays < 1.0");
    }

    // ── §19.5.D: Crime Record Tests ────────────────────────────────

    #[test]
    fn crime_record_escalates_punishment() {
        let mut record = CrimeRecord::default();

        // First offense: multiplier = 1.0
        record.record_offense(100);
        assert_eq!(record.offense_count, 1);
        let mult1 = record.punishment_multiplier();
        assert!((mult1.to_f64() - 1.0).abs() < 0.01, "First offense multiplier should be ~1.0");

        // Second offense: multiplier = 1.5
        record.record_offense(200);
        assert_eq!(record.offense_count, 2);
        let mult2 = record.punishment_multiplier();
        assert!((mult2.to_f64() - 1.5).abs() < 0.01, "Second offense multiplier should be ~1.5");

        // Third offense: multiplier = 2.0
        record.record_offense(300);
        assert_eq!(record.offense_count, 3);
        let mult3 = record.punishment_multiplier();
        assert!((mult3.to_f64() - 2.0).abs() < 0.01, "Third offense multiplier should be ~2.0");
    }

    #[test]
    fn crime_record_notoriety_grows() {
        let mut record = CrimeRecord::default();
        record.record_offense(100);
        let n1 = record.notoriety.to_f64();
        assert!(n1 > 0.0 && n1 < 1.0, "First offense notoriety should be between 0 and 1");

        // New episodes (spaced beyond the 400-tick window) grow notoriety.
        record.record_offense(600);
        let n2 = record.notoriety.to_f64();
        assert!(n2 > n1, "Notoriety should grow with more distinct episodes");

        // Cap at 1.0 (diminishing returns on episode count).
        for i in 0..20u64 {
            record.record_offense(600 + i * ENFORCEMENT_EPISODE_TICKS);
        }
        assert!(record.notoriety.to_f64() <= 1.0, "Notoriety should cap at 1.0");
    }

    #[test]
    fn crime_record_punishment_multiplier_caps_at_3() {
        let mut record = CrimeRecord::default();
        for i in 0..10 {
            record.record_offense(i * 100);
        }
        let mult = record.punishment_multiplier();
        assert!((mult.to_f64() - 3.0).abs() < 0.01, "Punishment multiplier should cap at 3.0");
    }

    #[test]
    fn enforcement_probability_catches_violation() {
        let mut registry = NormRegistry::new();
        registry.register(Norm {
            id: 0,
            name: "No Theft".into(),
            strength: Fixed::from_f64(0.8),
            internalization: Fixed::from_f64(0.6),
            punishment: Fixed::from_f64(0.5),
            reinforcing_identity: None,
        });

        let agent = AgentId::new(3);
        let enforcement = Fixed::from_f64(0.8); // 80% detection rate
        let rng_detected = Fixed::from_f64(0.5); // rng < 0.8 → caught
        let rng_missed = Fixed::from_f64(0.9);   // rng > 0.8 → not caught

        // Caught
        let (pun1, caught1) = registry.check_violation_with_enforcement(0, agent, 100, enforcement, rng_detected);
        assert!(caught1, "Violation should be detected");
        assert!(pun1 > Fixed::ZERO, "Punishment should be applied when caught");
        assert_eq!(registry.violations().len(), 1);

        // Not caught
        let (pun2, caught2) = registry.check_violation_with_enforcement(0, agent, 200, enforcement, rng_missed);
        assert!(!caught2, "Violation should NOT be detected");
        assert_eq!(pun2, Fixed::ZERO, "No punishment when not caught");
        assert_eq!(registry.violations().len(), 1, "Violation count should not increase");
    }

    #[test]
    fn enforcement_always_catches_at_100_percent() {
        let mut registry = NormRegistry::new();
        registry.register(Norm {
            id: 0,
            name: "No Theft".into(),
            strength: Fixed::from_f64(0.8),
            internalization: Fixed::from_f64(0.6),
            punishment: Fixed::from_f64(0.5),
            reinforcing_identity: None,
        });

        let agent = AgentId::new(1);
        let enforcement = Fixed::ONE; // 100% detection

        // Any rng_value < 1.0 → caught
        let (_, caught) = registry.check_violation_with_enforcement(0, agent, 100, enforcement, Fixed::from_f64(0.99));
        assert!(caught, "100% enforcement should always catch");
    }

    #[test]
    fn enforcement_never_catches_at_0_percent() {
        let mut registry = NormRegistry::new();
        registry.register(Norm {
            id: 0,
            name: "No Theft".into(),
            strength: Fixed::from_f64(0.8),
            internalization: Fixed::from_f64(0.6),
            punishment: Fixed::from_f64(0.5),
            reinforcing_identity: None,
        });

        let agent = AgentId::new(1);
        let enforcement = Fixed::ZERO; // 0% detection

        // rng_value > 0.0 → not caught
        let (_, caught) = registry.check_violation_with_enforcement(0, agent, 100, enforcement, Fixed::from_f64(0.5));
        assert!(!caught, "0% enforcement should never catch");
    }

    #[test]
    fn escalating_punishment_through_registry() {
        let mut registry = NormRegistry::new();
        registry.register(Norm {
            id: 0,
            name: "No Theft".into(),
            strength: Fixed::from_f64(0.8),
            internalization: Fixed::from_f64(0.6),
            punishment: Fixed::from_f64(0.4),
            reinforcing_identity: None,
        });

        let agent = AgentId::new(7);
        let enforcement = Fixed::ONE; // always caught

        // First offense: 0.4 * 1.0 = 0.4
        let (pun1, _) = registry.check_violation_with_enforcement(0, agent, 100, enforcement, Fixed::from_f64(0.1));
        assert!((pun1.to_f64() - 0.4).abs() < 0.01, "First offense: base punishment");

        // Second offense: 0.4 * 1.5 = 0.6
        let (pun2, _) = registry.check_violation_with_enforcement(0, agent, 200, enforcement, Fixed::from_f64(0.1));
        assert!((pun2.to_f64() - 0.6).abs() < 0.01, "Second offense: 1.5x multiplier");

        // Third offense: 0.4 * 2.0 = 0.8
        let (pun3, _) = registry.check_violation_with_enforcement(0, agent, 300, enforcement, Fixed::from_f64(0.1));
        assert!((pun3.to_f64() - 0.8).abs() < 0.01, "Third offense: 2.0x multiplier");

        // Verify criminal history
        let record = registry.crime_record(agent).unwrap();
        assert_eq!(record.offense_count, 3);
        assert!(record.notoriety.to_f64() > 0.0);
    }

    #[test]
    fn crime_record_accessor() {
        let mut registry = NormRegistry::new();
        registry.register(Norm {
            id: 0,
            name: "No Theft".into(),
            strength: Fixed::from_f64(0.8),
            internalization: Fixed::from_f64(0.6),
            punishment: Fixed::from_f64(0.5),
            reinforcing_identity: None,
        });

        let agent = AgentId::new(4);
        assert!(registry.crime_record(agent).is_none(), "No record before any violations");

        registry.check_violation(0, agent, 100);
        assert!(registry.crime_record(agent).is_some(), "Record exists after violation");
        assert_eq!(registry.crime_record(agent).unwrap().offense_count, 1);
    }
}
