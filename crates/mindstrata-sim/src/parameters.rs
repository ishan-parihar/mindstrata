//! §5.1 / Phase 5: Configurable simulation parameters.
//!
//! Centralizes all tuning constants that control simulation behavior.
//! Instead of scattering `Fixed::from_f64(0.3)` across dozens of files,
//! parameters are defined here and referenced by subsystems.
//!
//! ```text
//! Parameter categories:
//!   - Biological: decay rates, thresholds, ceilings
//!   - Psychological: appraisal weights, emotion regulation, cognitive load
//!   - Relational: trust decay, bonding rates, conflict escalation
//!   - Cultural: meme virality, propaganda effectiveness, ritual cohesion
//!   - Economic: price elasticity, trade friction, resource decay
//!   - Institutional: legitimacy decay, policy enforcement, corruption
//!   - Scheduler: tick intervals, phase boundaries
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Top-level simulation parameters — all tuning constants in one place.
///
/// Per Rust best practices (Chapter 1), small Copy types are passed by value.
/// This struct is Copy + Clone + Default for cheap embedding in Simulation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SimParameters {
    // ── Biological ─────────────────────────────────────────────
    /// Base hunger decay rate per tick (how fast hunger deficit grows).
    pub hunger_decay_rate: Fixed,
    /// Base thirst decay rate per tick.
    pub thirst_decay_rate: Fixed,
    /// Base fatigue decay rate per tick.
    pub fatigue_decay_rate: Fixed,
    /// Safety need decay rate per tick.
    pub safety_decay_rate: Fixed,
    /// Social need decay rate per tick.
    pub social_decay_rate: Fixed,
    /// Meaning need decay rate per tick.
    pub meaning_decay_rate: Fixed,
    /// Attachment separation distress decay rate (daily).
    pub attachment_decay_rate: Fixed,

    // ── Psychological ─────────────────────────────────────────
    /// Stress smoothing factor (lower = slower adaptation).
    pub stress_smoothing: Fixed,
    /// Fatigue smoothing factor.
    pub fatigue_smoothing: Fixed,
    /// Stress contribution to heuristic bias.
    pub stress_to_heuristic: Fixed,
    /// Fatigue contribution to heuristic bias.
    pub fatigue_to_heuristic: Fixed,
    /// Base heuristic bias (cognitive load floor).
    pub heuristic_bias_floor: Fixed,
    /// Trust sync convergence rate (how fast trust aligns with relationships).
    pub trust_sync_rate: Fixed,
    /// Meme novelty decay rate (daily).
    pub meme_novelty_decay: Fixed,
    /// Rumor prevalence decay rate (daily).
    pub rumor_prevalence_decay: Fixed,

    // ── Relational ────────────────────────────────────────────
    /// Trust threshold for friendship classification.
    pub friendship_trust_threshold: Fixed,
    /// Trust threshold for alliance classification.
    pub alliance_trust_threshold: Fixed,
    /// Relationship decay rate when dormant (daily).
    pub relationship_dormant_decay: Fixed,
    /// Emotional event contribution to relationship change.
    pub emotional_event_weight: Fixed,
    /// Bonding rate from positive interactions.
    pub bonding_rate: Fixed,
    /// Conflict escalation rate from negative interactions.
    pub conflict_escalation_rate: Fixed,

    // ── Cultural ──────────────────────────────────────────────
    /// Meme transmission base chance multiplier.
    pub meme_transmission_multiplier: Fixed,
    /// Propaganda effectiveness multiplier.
    pub propaganda_effectiveness: Fixed,
    /// Ritual cohesion boost per participation.
    pub ritual_cohesion_boost: Fixed,
    /// Echo chamber emotional charge threshold.
    pub echo_chamber_emotional_threshold: Fixed,

    // ── Economic ──────────────────────────────────────────────
    /// Grain price elasticity (how much price responds to scarcity).
    pub price_elasticity: Fixed,
    /// Water price elasticity.
    pub water_price_elasticity: Fixed,
    /// Trade friction (cost of trading).
    pub trade_friction: Fixed,

    // ── Institutional ─────────────────────────────────────────
    /// Legitimacy decay rate per tick.
    pub legitimacy_decay: Fixed,
    /// Policy enforcement effectiveness.
    pub enforcement_effectiveness: Fixed,
    /// Corruption accumulation rate.
    pub corruption_rate: Fixed,

    // ── Scheduler ─────────────────────────────────────────────
    /// Tick interval for hourly systems.
    pub hourly_interval: u64,
    /// Tick interval for daily systems.
    pub daily_interval: u64,
    /// Tick interval for weekly systems.
    pub weekly_interval: u64,
    /// Tick interval for seasonal systems.
    pub seasonal_interval: u64,
    /// Tick interval for yearly systems.
    pub yearly_interval: u64,
    /// Tier reclassification interval (ticks).
    pub tier_reclassify_interval: u64,
}

impl Default for SimParameters {
    fn default() -> Self {
        Self {
            // Biological — base decay_rate = 0.001, matching original system_need_decay
            hunger_decay_rate: Fixed::from_f64(0.001),
            thirst_decay_rate: Fixed::from_f64(0.002),   // decay_rate * 2
            fatigue_decay_rate: Fixed::from_f64(0.0005),  // decay_rate * 0.5
            safety_decay_rate: Fixed::from_f64(0.0003),   // decay_rate * 0.3
            social_decay_rate: Fixed::from_f64(0.0002),   // decay_rate * 0.2
            meaning_decay_rate: Fixed::from_f64(0.00015), // decay_rate * 0.15
            attachment_decay_rate: Fixed::from_f64(0.05),

            // Psychological
            stress_smoothing: Fixed::from_f64(0.1),
            fatigue_smoothing: Fixed::from_f64(0.05),
            stress_to_heuristic: Fixed::from_f64(0.6),
            fatigue_to_heuristic: Fixed::from_f64(0.2),
            heuristic_bias_floor: Fixed::from_f64(0.2),
            trust_sync_rate: Fixed::from_f64(0.1),
            meme_novelty_decay: Fixed::from_f64(0.002),
            rumor_prevalence_decay: Fixed::from_f64(0.01),

            // Relational
            friendship_trust_threshold: Fixed::from_f64(0.5),
            alliance_trust_threshold: Fixed::from_f64(0.7),
            relationship_dormant_decay: Fixed::from_f64(0.001),
            emotional_event_weight: Fixed::from_f64(0.3),
            bonding_rate: Fixed::from_f64(0.05),
            conflict_escalation_rate: Fixed::from_f64(0.08),

            // Cultural
            meme_transmission_multiplier: Fixed::from_f64(1.0),
            propaganda_effectiveness: Fixed::from_f64(0.3),
            ritual_cohesion_boost: Fixed::from_f64(0.1),
            echo_chamber_emotional_threshold: Fixed::from_f64(0.6),

            // Economic
            price_elasticity: Fixed::from_f64(0.5),
            water_price_elasticity: Fixed::from_f64(0.4),
            trade_friction: Fixed::from_f64(0.05),

            // Institutional
            legitimacy_decay: Fixed::from_f64(0.001),
            enforcement_effectiveness: Fixed::from_f64(0.5),
            corruption_rate: Fixed::from_f64(0.002),

            // Scheduler — §6 tick intervals
            hourly_interval: 6,
            daily_interval: 144,
            weekly_interval: 1008,
            seasonal_interval: 4320,
            yearly_interval: 51840,
            tier_reclassify_interval: 100,
        }
    }
}

impl SimParameters {
    /// Create parameters tuned for faster simulation (compressed timescales).
    pub fn fast() -> Self {
        let mut p = Self::default();
        p.hunger_decay_rate = Fixed::from_f64(0.002);
        p.thirst_decay_rate = Fixed::from_f64(0.004);
        p.fatigue_decay_rate = Fixed::from_f64(0.001);
        p
    }

    /// Create parameters tuned for slow, stable simulation.
    pub fn stable() -> Self {
        let mut p = Self::default();
        p.hunger_decay_rate = Fixed::from_f64(0.0005);
        p.thirst_decay_rate = Fixed::from_f64(0.001);
        p.fatigue_decay_rate = Fixed::from_f64(0.00025);
        p.stress_smoothing = Fixed::from_f64(0.05);
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_parameters_are_sane() {
        let p = SimParameters::default();
        assert!(p.hunger_decay_rate > Fixed::ZERO);
        assert!(p.thirst_decay_rate > Fixed::ZERO);
        assert!(p.stress_smoothing > Fixed::ZERO && p.stress_smoothing < Fixed::ONE);
        assert!(p.trust_sync_rate > Fixed::ZERO && p.trust_sync_rate < Fixed::ONE);
        assert!(p.friendship_trust_threshold > Fixed::ZERO);
        assert!(p.friendship_trust_threshold < p.alliance_trust_threshold);
        assert_eq!(p.daily_interval, 144);
        assert_eq!(p.yearly_interval, 51840);
    }

    #[test]
    fn fast_parameters_have_higher_decay() {
        let default = SimParameters::default();
        let fast = SimParameters::fast();
        assert!(fast.hunger_decay_rate > default.hunger_decay_rate);
        assert!(fast.thirst_decay_rate > default.thirst_decay_rate);
    }

    #[test]
    fn stable_parameters_have_lower_decay() {
        let default = SimParameters::default();
        let stable = SimParameters::stable();
        assert!(stable.hunger_decay_rate < default.hunger_decay_rate);
        assert!(stable.stress_smoothing < default.stress_smoothing);
    }

    #[test]
    fn parameters_are_copy() {
        let p1 = SimParameters::default();
        let p2 = p1; // Copy, not move
        assert_eq!(p1.hunger_decay_rate, p2.hunger_decay_rate);
    }

    #[test]
    fn parameters_are_serializable() {
        let p = SimParameters::default();
        let json = serde_json::to_string(&p).unwrap();
        let deserialized: SimParameters = serde_json::from_str(&json).unwrap();
        assert_eq!(p.hunger_decay_rate, deserialized.hunger_decay_rate);
        assert_eq!(p.daily_interval, deserialized.daily_interval);
    }
}
