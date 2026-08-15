//! Musculoskeletal system — strength, endurance, fatigue, adaptation, and physical agency.
//!
//! Labor increases conditioning, overwork increases fatigue/injury, rest restores,
//! starvation causes atrophy, age reduces peak strength, training improves efficiency.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Muscular state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuscularState {
    /// Current strength (0 = feeble, 1 = peak human).
    pub strength: Fixed,
    /// Endurance capacity (0 = fragile, 1 = inexhaustible).
    pub endurance: Fixed,
    /// Current fatigue (0 = fresh, 1 = exhausted).
    pub fatigue: Fixed,
    /// Muscle soreness from recent exertion (0 = none, 1 = very sore).
    pub soreness: Fixed,
    /// Atrophy from disuse or starvation (0 = none, 1 = severe).
    pub atrophy: Fixed,
    /// Conditioning level — improves with consistent moderate labor.
    pub conditioning: Fixed,
    /// Current injury level (0 = uninjured, 1 = severely injured).
    pub injury: Fixed,
}

impl Default for MuscularState {
    fn default() -> Self {
        Self {
            strength: Fixed::from_f64(0.6),
            endurance: Fixed::from_f64(0.5),
            fatigue: Fixed::ZERO,
            soreness: Fixed::ZERO,
            atrophy: Fixed::ZERO,
            conditioning: Fixed::from_f64(0.4),
            injury: Fixed::ZERO,
        }
    }
}

impl MuscularState {
    /// Update musculoskeletal state each tick.
    pub fn tick_update(
        &mut self,
        activity_level: Fixed,
        nutrition_quality: Fixed,
        rest_quality: Fixed,
        age_modifier: Fixed,
    ) {
        // Fatigue accumulates with activity
        let fatigue_gain = activity_level * Fixed::from_f64(0.015);
        self.fatigue = (self.fatigue + fatigue_gain).clamp_01();

        // Rest reduces fatigue
        let fatigue_recovery = rest_quality * Fixed::from_f64(0.02);
        self.fatigue = (self.fatigue - fatigue_recovery).max(Fixed::ZERO);

        // Soreness from high activity
        if activity_level > Fixed::from_f64(0.7) {
            let soreness_gain = (activity_level - Fixed::from_f64(0.7)) * Fixed::from_f64(0.01);
            self.soreness = (self.soreness + soreness_gain).clamp_01();
        }
        // Soreness decays with rest
        self.soreness = (self.soreness - rest_quality * Fixed::from_f64(0.008)).max(Fixed::ZERO);

        // Conditioning improves with moderate-to-heavy consistent activity.
        // §7.2.4 (S2-2-3 fix): the gain band was `> 0.3 && < 0.7` — it
        // EXCLUDED Work (activity 0.8, the sim's primary action), so
        // conditioning sat frozen at its birth value 0.4 in every scenario
        // (probe-pinned). The band now includes Work (`<= 0.8`), the gain is
        // saturating (`× (1 − conditioning)`) to bound the axis below 1.0,
        // and a slow excess-proportional detrain decay keeps it
        // differentiated (hard workers converge to ≈0.60, idlers drift back
        // to the 0.4 floor).
        //
        // Computed in f64 for the Iteration-176 reason (same as the
        // cardiovascular fitness): the Fixed-only gain `0.8 × 0.6 × 0.0003 ×
        // 0.6 = 0.0000864` truncates to raw 0 at the 4-decimal scale, so a
        // Fixed-only implementation silently froze conditioning at 0.4001
        // (the probe-pinned trap). f64 arithmetic is deterministic and the
        // value is re-quantized to Fixed once per tick.
        let c = self.conditioning.to_f64();
        let activity = activity_level.to_f64();
        let nutrition = nutrition_quality.to_f64();
        if activity > 0.3 && activity <= 0.8 {
            let gain = activity * nutrition * 0.001 * (1.0 - c);
            self.conditioning = Fixed::from_f64((c + gain).clamp(0.0, 1.0));
        }
        // Detrain (excess-proportional, floor 0.4): zero at the birth floor
        // (idlers return to baseline, never below) and growing with the
        // surplus — prevents the ratchet to 1.0.
        let excess = (self.conditioning.to_f64() - 0.4).max(0.0);
        if excess > 0.0 {
            let detrain = excess * 0.001;
            self.conditioning = Fixed::from_f64((self.conditioning.to_f64() - detrain).max(0.4));
        }

        // Strength derived from conditioning, minus fatigue, minus atrophy
        self.strength = (self.conditioning * Fixed::from_f64(0.7) + Fixed::from_f64(0.3)
            - self.fatigue * Fixed::from_f64(0.2)
            - self.atrophy * Fixed::from_f64(0.3)
            - self.injury * Fixed::from_f64(0.2))
        .clamp_01();

        // Endurance from conditioning
        self.endurance =
            (self.conditioning * Fixed::from_f64(0.6) + Fixed::from_f64(0.3)).clamp_01();

        // Atrophy from disuse or poor nutrition
        if activity_level < Fixed::from_f64(0.1) || nutrition_quality < Fixed::from_f64(0.2) {
            self.atrophy = (self.atrophy + Fixed::from_f64(0.0002)).clamp_01();
        } else if nutrition_quality > Fixed::from_f64(0.5) {
            // Recovery from atrophy with nutrition and activity
            self.atrophy = (self.atrophy - Fixed::from_f64(0.0001)).max(Fixed::ZERO);
        }

        // Age reduces peak strength
        self.strength = (self.strength - age_modifier * Fixed::from_f64(0.0001)).max(Fixed::ZERO);
    }

    /// Effective strength for tasks (accounting for fatigue and injury).
    pub fn effective_strength(&self) -> Fixed {
        self.strength
    }

    /// Whether the agent is too exhausted for physical labor.
    pub fn exhausted(&self) -> bool {
        self.fatigue > Fixed::from_f64(0.85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditioning_improves_with_moderate_activity() {
        let mut m = MuscularState::default();
        let initial = m.conditioning;
        for _ in 0..200 {
            m.tick_update(
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.3),
            );
        }
        assert!(m.conditioning > initial);
    }

    #[test]
    fn work_activity_builds_conditioning() {
        // S2-2-3 regression: Work maps to activity 0.8, which the old strict
        // `activity < 0.7` band EXCLUDED — the primary labor action never
        // built conditioning (frozen at 0.4 in every scenario). The widened
        // band (`<= 0.8`) plus saturating gain must move conditioning for a
        // working agent.
        let mut m = MuscularState::default();
        let initial = m.conditioning;
        for _ in 0..800 {
            m.tick_update(
                Fixed::from_f64(0.8), // Work
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.3),
            );
        }
        assert!(m.conditioning > initial, "Work must build conditioning");
        assert!(m.conditioning < Fixed::from_f64(0.95), "must not saturate");
    }

    #[test]
    fn conditioning_detrains_without_exertion() {
        // S2-2-3: the detrain term prevents conditioning from ratcheting to
        // 1.0 under sustained Work — an idling agent must drift back down
        // toward the 0.4 floor.
        let mut m = MuscularState {
            conditioning: Fixed::from_f64(0.8),
            ..Default::default()
        };
        for _ in 0..3000 {
            m.tick_update(
                Fixed::from_f64(0.1), // idle
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.3),
            );
        }
        assert!(m.conditioning < Fixed::from_f64(0.8), "conditioning must decay when idle");
        assert!(m.conditioning >= Fixed::from_f64(0.4), "detrain floor holds");
    }

    #[test]
    fn conditioning_work_equilibrium_is_below_one() {
        // S2-2-3: sustained Work must push conditioning UP but the saturating
        // gain plus excess-proportional detrain must hold it at a stable
        // equilibrium well below 1.0 (no ratchet).
        let mut m = MuscularState::default();
        for _ in 0..20000 {
            m.tick_update(
                Fixed::from_f64(0.8), // Work
                Fixed::from_f64(0.6),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.3),
            );
        }
        let after = m.conditioning.to_f64();
        assert!(after > 0.5, "sustained Work must lift conditioning (got {after})");
        assert!(after < 0.95, "equilibrium must stay below 1.0 (got {after})");
    }

    #[test]
    fn rest_recovers_fatigue() {
        let mut m = MuscularState {
            fatigue: Fixed::from_f64(0.8),
            ..Default::default()
        };
        m.tick_update(Fixed::ZERO, Fixed::ONE, Fixed::ONE, Fixed::from_f64(0.3));
        assert!(m.fatigue < Fixed::from_f64(0.8));
    }

    #[test]
    fn starvation_causes_atrophy() {
        let mut m = MuscularState::default();
        for _ in 0..100 {
            m.tick_update(Fixed::ZERO, Fixed::ZERO, Fixed::ONE, Fixed::from_f64(0.3));
        }
        assert!(m.atrophy > Fixed::ZERO);
    }
}
