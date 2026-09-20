//! Cardiovascular system — stamina, shock, blood loss, fitness, and acute mortality risk.
//!
//! Fitness improves with healthy labor, overexertion under starvation causes collapse,
//! combat injury causes blood loss, chronic stress raises long-term risk, aging reduces capacity.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Wound severity above which a body bleeds.
///
/// Iteration 319. The rule was a bare `0.3`, calibrated when `Combat`-kind
/// conflicts (severity 0.5) and multi-wound stacking were the bleeding
/// pathway. The i314 exertion veto ended the stacking — an injured agent is
/// vetoed from exertion and therefore from further conflict — and `Combat`
/// fires **0 times** at every horizon (probe `i319_wound_reachability`), so
/// the reachable wound range became single `Violence` wounds 0.12–0.166 and
/// two-wound stacks up to 0.295. `0.3` is therefore crossed by **0
/// agent-ticks** in 12 seeds × 20K/50K and the whole cardiovascular chain
/// (`blood_volume` → `shock_risk` → the derived-health `shock_penalty`) went
/// dead — the same AGENTS §4.3 dead-producer class i311/i313 closed.
///
/// `0.15` makes a *single* serious beating bleed (just under the 0.166 maximum
/// reachable single wound) while leaving the common 0.12–0.13 scuffle dry;
/// measured share above it is 0.06–0.13% of agent-ticks. Chosen on the probe's
/// threshold table, not guessed: 0.20 requires a two-wound stack (0.02–0.04%
/// of agent-ticks) and 0.30 is unreachable.
pub const BLOOD_LOSS_INJURY_THRESHOLD: f64 = 0.15;

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
        // Blood loss from injury (threshold reconciled with the post-i314
        // reachable wound range — see BLOOD_LOSS_INJURY_THRESHOLD).
        if injury_severity > Fixed::from_f64(BLOOD_LOSS_INJURY_THRESHOLD) {
            let blood_loss = injury_severity * Fixed::from_f64(0.005);
            self.blood_volume = (self.blood_volume - blood_loss).max(Fixed::from_f64(0.3));
        }
        // Blood volume recovers slowly with good nutrition — but only while
        // the body is NOT bleeding. The recovery band used to be `< 0.2`
        // against a bleed threshold of `0.3`, leaving a dry gap; reconciling
        // the threshold to 0.15 (i319) made the bands OVERLAP in (0.15, 0.2),
        // where recovery (`0.5 × 0.002 = 0.001/tick`) exceeds the loss
        // (`0.16 × 0.005 = 0.0008/tick`) and a serious wound still nets zero
        // — caught by `blood_loss_threshold_admits_a_serious_wound_but_not_a_scuffle`.
        // Pinning recovery to the same threshold removes the overlap: a body
        // at or above it only bleeds, and recovers once the wound drops below.
        if injury_severity < Fixed::from_f64(BLOOD_LOSS_INJURY_THRESHOLD) {
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

    /// i319: the blood-loss threshold must be reachable by a real single
    /// `Violence` wound. A violence wound is `0.12 + aggression × 0.1`
    /// (`conflict.rs`), and the i319 probe measured a 12-seed maximum of
    /// **0.1656**; `Combat` (0.5) fires 0 times. The old threshold `0.3` sat
    /// above the entire reachable range, so the cardiovascular chain was dead
    /// state. Guard the boundary: a common scuffle stays dry, a serious
    /// beating bleeds.
    #[test]
    fn blood_loss_threshold_admits_a_serious_wound_but_not_a_scuffle() {
        assert!(
            BLOOD_LOSS_INJURY_THRESHOLD < 0.1656,
            "threshold {BLOOD_LOSS_INJURY_THRESHOLD} must be reachable by the measured max single wound 0.1656"
        );
        // A serious wound (just under the measured 0.1656 max) bleeds.
        let mut serious = CardiovascularState::default();
        serious.tick_update(
            Fixed::ZERO,
            Fixed::from_f64(0.16),
            Fixed::ZERO,
            Fixed::ONE,
            Fixed::from_f64(0.3),
        );
        assert!(
            serious.blood_volume < Fixed::ONE,
            "a serious wound must bleed (blood volume {:?})",
            serious.blood_volume
        );
        // A base-severity `Violence` scuffle (0.12) stays dry, so ordinary
        // fights do not slowly exsanguinate the village.
        let mut scuffle = CardiovascularState::default();
        scuffle.tick_update(
            Fixed::ZERO,
            Fixed::from_f64(0.12),
            Fixed::ZERO,
            Fixed::ONE,
            Fixed::from_f64(0.3),
        );
        assert_eq!(
            scuffle.blood_volume,
            Fixed::ONE,
            "a base-severity scuffle must not bleed"
        );
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
