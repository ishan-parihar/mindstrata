//! Cardiovascular system — stamina, shock, blood loss, fitness, and acute mortality risk.
//!
//! Fitness improves with healthy labor, overexertion under starvation causes collapse,
//! combat injury causes blood loss, chronic stress raises long-term risk, aging reduces capacity.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Cardiovascular state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardiovascularState {
    /// Cardiac capacity (0 = failing, 1 = peak). Declines with age and illness.
    pub cardiac_capacity: Fixed,
    /// Blood volume (0.5 = hemorrhaging, 1.0 = normal).
    pub blood_volume: Fixed,
    /// Fitness level (0 = unfit, 1 = elite). Improves with healthy labor.
    pub fitness: Fixed,
    /// Blood pressure analog (0 = dangerously low, 0.5 = normal, 1 = hypertensive).
    pub blood_pressure: Fixed,
    /// Recovery rate — how fast the cardiovascular system recovers from stress.
    pub recovery_rate: Fixed,
    /// Shock risk (0 = none, 1 = imminent collapse).
    pub shock_risk: Fixed,
}

impl Default for CardiovascularState {
    fn default() -> Self {
        Self {
            cardiac_capacity: Fixed::from_f64(0.7),
            blood_volume: Fixed::ONE,
            fitness: Fixed::from_f64(0.5),
            blood_pressure: Fixed::from_f64(0.5),
            recovery_rate: Fixed::from_f64(0.5),
            shock_risk: Fixed::ZERO,
        }
    }
}

impl CardiovascularState {
    /// Update cardiovascular state each tick.
    pub fn tick_update(
        &mut self,
        activity_level: Fixed,
        injury_severity: Fixed,
        stress_level: Fixed,
        nutrition_quality: Fixed,
        age_modifier: Fixed,
    ) {
        // Blood loss from injury
        if injury_severity > Fixed::from_f64(0.3) {
            let blood_loss = injury_severity * Fixed::from_f64(0.005);
            self.blood_volume = (self.blood_volume - blood_loss).max(Fixed::from_f64(0.3));
        }
        // Blood volume recovers slowly with good nutrition
        if injury_severity < Fixed::from_f64(0.2) {
            let recovery = nutrition_quality * self.recovery_rate * Fixed::from_f64(0.002);
            self.blood_volume = (self.blood_volume + recovery).min(Fixed::ONE);
        }

        // Fitness improves with moderate-to-heavy activity + good nutrition.
        // §7.2.9 (S2-2-3 fix): the gain band was `> 0.3 && < 0.8` — a STRICT
        // upper bound that excluded Work (activity 0.8, the sim's primary
        // action), so fitness sat frozen at its birth value 0.5 in every
        // scenario (probe-pinned). The band now includes Work (`<= 0.8`),
        // and the gain is saturating (`× (1 − fitness)`) so the axis
        // approaches a stable equilibrium below 1.0 instead of ratcheting
        // there (the same logistic pattern as the Iteration-176 trauma and
        // S2-2-2 bonding fixes). A slow excess-proportional detrain decay
        // ("use it or lose it": zero at the 0.5 floor, growing with the
        // surplus) keeps fitness differentiated: hard workers converge to
        // ≈0.70, idlers drift back to 0.5.
        //
        // Computed in f64 for the Iteration-176 reason (the trauma-decay and
        // should_birth precedents): at the 4-decimal Fixed scale the
        // saturating factor `rate × (1 − fitness)` truncates to zero the
        // moment fitness leaves the floor (0.0002 × 0.4999 → raw 0), so a
        // Fixed-only implementation silently froze fitness at 0.5001 (the
        // probe-pinned trap). f64 arithmetic is deterministic (IEEE-754,
        // same result on every platform) and the value is re-quantized to
        // Fixed once per tick. (Gain constant 0.0015, not 0.0005: the
        // analytic equilibrium with the 0.001 detrain is 0.597, but 4-decimal
        // truncation pinched the plateau to exactly 0.55 — a per-tick net
        // ≈0.000058 rounds to 0. The larger constant lifts the equilibrium
        // to ≈0.70, comfortably clear of the 0.0001 quantization step.)
        // Iteration 225: widened fitness gain band and increased rate.
        // Before: activity > 0.3 && <= 0.8 with rate 0.0015, floor 0.5.
        // Result: fitness = 0.550 uniform (no spread across agents).
        // Now: activity >= 0.1 && <= 0.8 with rate 0.002, floor 0.4.
        // The wider band includes Wander (0.4), Socialize (0.2), and
        // Trade (0.3), so social agents gain fitness too. The higher
        // rate lifts the equilibrium from ~0.55 to ~0.65, and the lower
        // floor (0.4) creates real differentiation between active and
        // idle agents.
        let f = self.fitness.to_f64();
        let activity = activity_level.to_f64();
        let nutrition = nutrition_quality.to_f64();
        if (0.1..=0.8).contains(&activity) {
            let gain = activity * nutrition * 0.002 * (1.0 - f);
            self.fitness = Fixed::from_f64((f + gain).clamp(0.0, 1.0));
        }
        // Detrain (excess-proportional, floor 0.4): idle agents drift
        // back to baseline, active agents retain gains. The 0.4 floor
        // ensures even resting agents have some baseline fitness.
        let excess = (self.fitness.to_f64() - 0.4).max(0.0);
        if excess > 0.0 {
            let detrain = excess * 0.001;
            self.fitness = Fixed::from_f64((self.fitness.to_f64() - detrain).max(0.4));
        }
        // Overexertion reduces fitness
        if activity_level > Fixed::from_f64(0.85) {
            self.fitness = (self.fitness - Fixed::from_f64(0.0002)).max(Fixed::ZERO);
        }

        // Cardiac capacity declines with age and chronic stress
        self.cardiac_capacity = (Fixed::from_f64(0.8)
            - age_modifier * Fixed::from_f64(0.001)
            - stress_level * Fixed::from_f64(0.0003))
        .clamp_01();

        // Blood pressure responds to stress
        self.blood_pressure = (Fixed::from_f64(0.5) + stress_level * Fixed::from_f64(0.2)
            - self.fitness * Fixed::from_f64(0.1))
        .clamp_01();

        // Shock risk from severe blood loss + stress
        self.shock_risk = if self.blood_volume < Fixed::from_f64(0.6) {
            ((Fixed::from_f64(0.6) - self.blood_volume) * Fixed::from_f64(2.0)
                + stress_level * Fixed::from_f64(0.3))
            .clamp_01()
        } else {
            Fixed::ZERO
        };

        // Recovery rate influenced by fitness
        self.recovery_rate =
            (self.fitness * Fixed::from_f64(0.6) + Fixed::from_f64(0.2)).clamp_01();
    }

    /// Effective stamina for physical tasks (0–1).
    pub fn effective_stamina(&self) -> Fixed {
        (self.fitness * Fixed::from_f64(0.4)
            + self.blood_volume * Fixed::from_f64(0.3)
            + self.cardiac_capacity * Fixed::from_f64(0.3))
        .clamp_01()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fitness_improves_with_moderate_activity() {
        let mut cv = CardiovascularState::default();
        let initial = cv.fitness;
        for _ in 0..100 {
            cv.tick_update(
                Fixed::from_f64(0.5),
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.3),
            );
        }
        assert!(cv.fitness > initial);
    }

    #[test]
    fn work_activity_builds_fitness() {
        // S2-2-3 regression: Work maps to activity 0.8, which the old strict
        // `activity < 0.8` band EXCLUDED — so the primary labor action never
        // built fitness (frozen at 0.5 in every scenario). The widened band
        // (`<= 0.8`) plus saturating gain must move fitness for a working
        // agent.
        let mut cv = CardiovascularState::default();
        let initial = cv.fitness;
        for _ in 0..500 {
            cv.tick_update(
                Fixed::from_f64(0.8), // Work
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.3),
            );
        }
        assert!(cv.fitness > initial, "Work must build fitness");
        assert!(cv.fitness < Fixed::from_f64(0.95), "must not saturate");
    }

    #[test]
    fn fitness_detrains_without_exertion() {
        // S2-2-3: the detrain term prevents fitness from ratcheting to 1.0
        // under sustained Work — an idling agent must drift back down toward
        // the 0.5 floor.
        let mut cv = CardiovascularState {
            fitness: Fixed::from_f64(0.85),
            ..Default::default()
        };
        for _ in 0..2000 {
            cv.tick_update(
                Fixed::from_f64(0.1), // idle
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.3),
            );
        }
        assert!(
            cv.fitness < Fixed::from_f64(0.85),
            "fitness must decay when idle"
        );
        assert!(cv.fitness >= Fixed::from_f64(0.5), "detrain floor holds");
    }

    #[test]
    fn fitness_work_equilibrium_is_below_one() {
        // S2-2-3: sustained Work must push fitness UP but the saturating gain
        // plus excess-proportional detrain must hold it at a stable
        // equilibrium well below 1.0 (no ratchet).
        let mut cv = CardiovascularState::default();
        for _ in 0..20000 {
            cv.tick_update(
                Fixed::from_f64(0.8), // Work
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::from_f64(0.6),
                Fixed::from_f64(0.3),
            );
        }
        let after = cv.fitness.to_f64();
        assert!(
            after > 0.55,
            "sustained Work must lift fitness (got {after})"
        );
        assert!(
            after < 0.95,
            "equilibrium must stay below 1.0 (got {after})"
        );
    }

    #[test]
    fn injury_causes_blood_loss() {
        let mut cv = CardiovascularState::default();
        cv.tick_update(
            Fixed::ZERO,
            Fixed::from_f64(0.8),
            Fixed::ZERO,
            Fixed::ONE,
            Fixed::from_f64(0.3),
        );
        assert!(cv.blood_volume < Fixed::ONE);
    }

    #[test]
    fn shock_risk_from_severe_blood_loss() {
        let mut cv = CardiovascularState {
            blood_volume: Fixed::from_f64(0.4),
            ..Default::default()
        };
        cv.tick_update(
            Fixed::ZERO,
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.7),
            Fixed::ONE,
            Fixed::from_f64(0.3),
        );
        assert!(cv.shock_risk > Fixed::ZERO);
    }
}
