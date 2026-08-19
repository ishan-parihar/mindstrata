//! Hormonal / Endocrine system — connects body to mind.
//!
//! Hormones modulate emotion, motivation, trust, aggression, bonding, appetite,
//! sleep, and reproduction. We simulate functional axes, not individual hormones.
//!
//! Each axis has:
//! - inputs (what raises/lowers it)
//! - current level (0–1)
//! - effects on psychology and behavior

use mindstrata_core::fixed::Fixed;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Basal parasympathetic floor for stress recovery (Iteration 172, Phase 5
/// "balance hormonal effects").
///
/// Recovery = `recovery_rate × tone`; the nervous `parasympathetic_tone`
/// input drains toward 0 under sustained threat (safety = 1 − fear − anger
/// ≈ 0 in calibrated windows), which zeroed recovery at ANY rate and pinned
/// the stress axis at 1.0 for the majority of agents (probe-pinned: 8/12
/// agents at 1.0 with chronic_load 0.75–1.0 across every seed × horizon).
/// The floor guarantees a basal recovery even at full sympathetic
/// activation — a floor of 0.3 with the 0.10 recovery rate restores a
/// producer-driven equilibrium (stress mean 0.42–0.58 with real per-agent
/// spread, stable to 10K ticks).
pub const STRESS_RECOVERY_TONE_FLOOR: f64 = 0.3;
/// [`STRESS_RECOVERY_TONE_FLOOR`] pre-converted to [`Fixed`] (hoisted out of
/// the per-tick per-agent `update` hot path; 0.3 × 10_000 = 3_000).
pub const STRESS_RECOVERY_TONE_FLOOR_FIXED: Fixed = Fixed::from_raw(3_000);

/// §7.2.2 (Iteration 189 — the graded stress equilibrium): the chronic-
/// stress feedback into the level. The pre-Iter-189 value 0.1 was a
/// self-accelerating ratchet: `chronic_delta = chronic_load × 0.1` reached
/// up to 0.1/tick — exactly the maximum recovery (0.10 × tone 1.0) — so
/// any agent whose level crossed 0.6 for ~200+ consecutive ticks
/// accumulated chronic_load until the delta overpowered recovery
/// permanently: the axis pinned at 1.0 (MS_TRACE_STRESS over 2000 ticks ×
/// 3 scenarios: 49,219 ticks at level 0.0, 21,383 at level 1.0, nothing
/// graded between — the Iter-172 "chronic_recovery 0.0005/tick" escape was
/// mechanically unreachable, since the level can never drop ≤ 0.6 while
/// delta > recovery). At 0.02 the memory term is bounded (max 0.02/tick)
/// BELOW the tone-floor recovery (0.03), so a high-chronic agent whose
/// acute subsides ALWAYS decays — chronic load modulates the level, it can
/// no longer pin it.
const CHRONIC_FEEDBACK: Fixed = Fixed::from_raw(200); // 0.02

/// Stress axis (cortisol-like).
/// Raised by: threat, pain, hunger, sleep debt, humiliation, uncertainty.
/// Effects: increases fear/anger, reduces planning, increases heuristic bias.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressAxis {
    /// Current level (0 = calm, 1 = panicked).
    pub level: Fixed,
    /// Baseline reactivity — how quickly this agent responds to stress.
    pub reactivity: Fixed,
    /// Chronic stress load — accumulates over time.
    pub chronic_load: Fixed,
}

impl Default for StressAxis {
    fn default() -> Self {
        Self {
            level: Fixed::from_f64(0.2),
            reactivity: Fixed::from_f64(0.5),
            chronic_load: Fixed::ZERO,
        }
    }
}

impl StressAxis {
    /// Update stress level based on acute input and recovery.
    pub fn update(
        &mut self,
        acute_input: Fixed,
        parasympathetic_tone: Fixed,
        recovery_rate: Fixed,
        chronic_rate: Fixed,
        chronic_recovery: Fixed,
    ) {
        // Iteration 172 (Phase 5 "balance hormonal effects"): the recovery
        // term is `rate × tone`, and under sustained threat the nervous
        // parasympathetic tone drains toward 0 (safety = 1 − fear − anger is
        // near 0 in calibrated windows). A zero tone zeroes recovery at ANY
        // rate, so the stress axis pinned at 1.0 for the majority of agents
        // (probe: 8/12 pinned with chronic_load 0.75–1.0, every window) — a
        // system dominating unnaturally, exactly what the Phase 5 acceptance
        // criteria forbid. The floor guarantees a basal recovery floor even
        // at full sympathetic activation, restoring producer-driven
        // equilibrium (calibrated 0.3 floor + 0.10 recovery: stress mean
        // 0.42–0.58 with real per-agent spread, stable to 10K ticks).
        //
        // Iteration 189 (the graded equilibrium — FINDING 10): the axis was
        // BINARY, not graded — the linear update `level += acute×reactivity −
        // recovery` has no interior fixed point, so the level only stopped at
        // the clamps: 0 (recovery dominates) or 1.0 (acute dominates, then
        // pinned permanently by the old `chronic_load × 0.1` ratchet — the
        // pre-Iter-189 escape, chronic_recovery 0.0005/tick, was unreachable
        // because the level can never drop ≤ 0.6 while delta > recovery).
        // MS_TRACE_STRESS over 2000 ticks × 3 scenarios: 49,219 ticks at
        // level 0.0, 21,383 at 1.0, nothing graded between. Three changes
        // make the axis GRADED (the repo's standard saturating-logistic
        // pattern, as on bonding/arousal/immune):
        //   (1) the ACUTE gain is saturating — `acute × reactivity ×
        //       (1 − level)` — giving an interior equilibrium `level* =
        //       1 − recovery/(acute×reactivity)`: calm (acute ≈ 0.11 from
        //       the live thirst channel) sits ~0.1–0.45 by tone, famine
        //       (acute ≈ 0.13) ~0.2–0.5, pain-heavy agents ~0.7–0.94;
        //       the level can only APPROACH 1.0 asymptotically, never
        //       clamp-pin;
        //   (2) the chronic feedback is 5× weaker (`CHRONIC_FEEDBACK` 0.02,
        //       max delta 0.02 < the 0.03 tone-floor recovery — a
        //       high-chronic agent whose acute subsides always decays);
        //   (3) the chronic load accumulates/decays with the saturating
        //       form — `chronic_rate × (1 − chronic_load)` stressed,
        //       `−= chronic_recovery × chronic_load` calm — bounded by
        //       construction instead of the linear accumulator.
        let tone = parasympathetic_tone.max(STRESS_RECOVERY_TONE_FLOOR_FIXED);
        let recovery = recovery_rate * tone;
        let acute_delta = acute_input * self.reactivity * (Fixed::ONE - self.level);
        let chronic_delta = self.chronic_load * CHRONIC_FEEDBACK;
        if std::env::var("MS_TRACE_STRESS").is_ok() {
            eprintln!(
                "STRESS level={} +acute={} +chronic={} -recov={} (rate={} tone={})",
                self.level.to_f64(),
                acute_delta.to_f64(),
                chronic_delta.to_f64(),
                recovery.to_f64(),
                recovery_rate.to_f64(),
                parasympathetic_tone.to_f64()
            );
        }
        self.level = (self.level + acute_delta + chronic_delta - recovery).clamp_01();
        // Chronic load accumulates slowly from high acute stress, saturating
        // (logistic) so it can never exceed 1.0 or ratchet linearly.
        if self.level > Fixed::from_f64(0.6) {
            self.chronic_load = (self.chronic_load
                + chronic_rate * (Fixed::ONE - self.chronic_load))
                .clamp_01();
        } else {
            // Chronic load recovers very slowly, proportionally.
            self.chronic_load =
                (self.chronic_load - chronic_recovery * self.chronic_load).max(Fixed::ZERO);
        }
    }
}

/// Bonding axis (oxytocin-like).
/// Raised by: positive touch, comforting, caregiving, shared meals, ritual.
/// Effects: increases trust, attachment, in-group favoritism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondingAxis {
    pub level: Fixed,
    /// Baseline receptivity to bonding.
    pub receptivity: Fixed,
}

impl Default for BondingAxis {
    fn default() -> Self {
        Self {
            level: Fixed::from_f64(0.4),
            receptivity: Fixed::from_f64(0.5),
        }
    }
}

impl BondingAxis {
    /// §7.2.2 (S2-2-2 fix): update the axis with *saturating* (logistic)
    /// gain. The old linear form (`level + input×receptivity − recovery`)
    /// was a step-function accumulator when called from the ritual loop
    /// (fires only at monthly intervals, no decay between fires): every
    /// fire added a fixed positive delta, so the axis ratcheted to 1.0 at
    /// some horizon regardless of recovery — probe-pinned at ~30K ticks
    /// for max-receptivity seasonal participants. The (1 − level) factor
    /// makes gain vanish as the axis approaches 1.0, giving a stable
    /// equilibrium `L* = 1 − recovery/(input×receptivity)` (≈ 0.78 for
    /// the seasonal ritual at max receptivity) that the axis approaches
    /// monotonically but never exceeds — bounded by construction, the
    /// same class of fix as the Iter-172 tone floor. Zero change before
    /// the first ritual (all inputs are zero), so calibrated golden and
    /// snapshot horizons (≤2000 ticks, no ritual until 4320) stay
    /// byte-identical.
    pub fn update(&mut self, social_input: Fixed, recovery_rate: Fixed) {
        // Saturating gain: proximity to the ceiling damps further growth.
        let delta = social_input * self.receptivity * (Fixed::ONE - self.level);
        self.level = (self.level + delta - recovery_rate).clamp_01();
    }
}

/// Dominance axis (testosterone-like).
/// Raised by: status victory, competition, public respect.
/// Effects: modulates confidence, aggression, risk tolerance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DominanceAxis {
    pub level: Fixed,
}

impl Default for DominanceAxis {
    fn default() -> Self {
        Self {
            level: Fixed::from_f64(0.4),
        }
    }
}

impl DominanceAxis {
    /// §7.2.2 (S2-2-2 fix): mean-revert the axis toward a status-derived
    /// target (the §11.1 `effective_status` composite). Rises with status
    /// gains, falls with status losses — the plan's "status victory raises
    /// dominance" channel, wired daily from the status pass.
    ///
    /// The old implementation (`level + change·rate − 0.01`) had zero call
    /// sites (the axis was write-once at birth) and its hardcoded per-call
    /// decay would drain the axis to 0 when called on the daily cadence.
    /// Mean-reversion to the agent's actual standing cannot saturate: the
    /// axis converges to whatever status the world actually grants.
    pub fn update(&mut self, status_target: Fixed, response_rate: Fixed) {
        let delta = (status_target - self.level) * response_rate;
        self.level = (self.level + delta).clamp_01();
    }
}

/// Metabolic axis — energy processing, hunger, satiety.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolicAxis {
    /// Energy availability.
    pub energy: Fixed,
    /// Appetite pressure (high = hungry).
    pub appetite: Fixed,
    /// Satiety (high = full).
    pub satiety: Fixed,
}

impl Default for MetabolicAxis {
    fn default() -> Self {
        Self {
            energy: Fixed::from_f64(0.7),
            appetite: Fixed::from_f64(0.3),
            satiety: Fixed::from_f64(0.5),
        }
    }
}

/// Arousal axis — sympathetic activation for fight/flight/freeze.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArousalAxis {
    /// Current arousal level.
    pub level: Fixed,
    /// Baseline arousal.
    pub baseline: Fixed,
}

impl Default for ArousalAxis {
    fn default() -> Self {
        Self {
            level: Fixed::from_f64(0.3),
            baseline: Fixed::from_f64(0.3),
        }
    }
}

impl ArousalAxis {
    /// §7.2.2 (S2-2-4b fix): update the axis with *saturating* (logistic)
    /// rise. The old linear form (`acute × rise`) vs decay
    /// `(level − baseline) × decay` was net-positive at every calibrated
    /// stress level: famine/pestilence stress 0.667 × rise 0.3 = +0.2/tick
    /// vs decay ≈0.07 at level 1.0, so 8–9/12 agents pinned at 1.0 in
    /// every stress window (probe-pinned) — the same saturation family as
    /// the S2-2-1/2-2-2/2-2-3 fixes. The (1 − level) factor bounds the
    /// equilibrium at `L* = (s·r + b·d)/(s·r + d)` — ≈0.61 calm (stress
    /// 0.26), ≈0.77 famine (stress 0.667) with baseline 0.3 — elevated
    /// under chronic stress but never pinned, and the axis still decays
    /// to baseline when inputs fade.
    pub fn update(&mut self, acute_input: Fixed, rise_factor: Fixed, decay_factor: Fixed) {
        // Saturating rise: proximity to the ceiling damps further activation.
        let rise = acute_input * rise_factor * (Fixed::ONE - self.level);
        let decay = (self.level - self.baseline) * decay_factor;
        self.level = (self.level + rise - decay).clamp_01();
    }
}

/// Growth axis — development, repair, aging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthAxis {
    /// Current growth/repair capacity.
    pub capacity: Fixed,
    /// Developmental stage modifier (child=high, adult=moderate, elder=low).
    pub developmental_modifier: Fixed,
}

impl Default for GrowthAxis {
    fn default() -> Self {
        Self {
            capacity: Fixed::from_f64(0.6),
            developmental_modifier: Fixed::from_f64(1.0),
        }
    }
}

impl GrowthAxis {
    /// §7.2.2 (S2-2-2 fix): mean-revert `capacity` toward the life-stage
    /// growth target (child=high, adult=moderate, elder=low — see
    /// [`crate::biology::development::LifeStage::growth_modifier`]) and
    /// mirror the target into `developmental_modifier` (the "Growth →
    /// Developmental Stage → Trait Expression" arrow). Previously the axis
    /// had no update method at all — both fields were write-once at birth.
    pub fn update(&mut self, growth_target: Fixed, recovery_rate: Fixed) {
        self.developmental_modifier = growth_target;
        let delta = (growth_target - self.capacity) * recovery_rate;
        self.capacity = (self.capacity + delta).clamp_01();
    }
}

/// Complete endocrine state — all hormonal axes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EndocrineState {
    pub stress: StressAxis,
    pub bonding: BondingAxis,
    pub dominance: DominanceAxis,
    pub metabolic: MetabolicAxis,
    pub arousal: ArousalAxis,
    pub growth: GrowthAxis,
}

impl EndocrineState {
    /// Generate a random endocrine state.
    pub fn random(rng: &mut impl Rng) -> Self {
        Self {
            stress: StressAxis {
                level: Fixed::from_f64(0.2),
                reactivity: Fixed::from_f64(rng.random_range(0.2..0.8)),
                chronic_load: Fixed::ZERO,
            },
            bonding: BondingAxis {
                level: Fixed::from_f64(rng.random_range(0.3..0.6)),
                receptivity: Fixed::from_f64(rng.random_range(0.2..0.8)),
            },
            dominance: DominanceAxis {
                level: Fixed::from_f64(rng.random_range(0.2..0.6)),
            },
            metabolic: MetabolicAxis {
                energy: Fixed::from_f64(rng.random_range(0.5..0.8)),
                appetite: Fixed::from_f64(rng.random_range(0.2..0.5)),
                satiety: Fixed::from_f64(rng.random_range(0.3..0.6)),
            },
            arousal: ArousalAxis {
                level: Fixed::from_f64(rng.random_range(0.2..0.4)),
                baseline: Fixed::from_f64(rng.random_range(0.2..0.4)),
            },
            growth: GrowthAxis {
                capacity: Fixed::from_f64(0.6),
                developmental_modifier: Fixed::from_f64(1.0),
            },
        }
    }

    /// Dominance modifier for aggression (Iteration 191: now live — the
    /// escalation decision at the violence site folds this into
    /// `aggressor_aggression`, closing the S2-2-2 write-only axis).
    pub fn dominance_aggression_modifier(&self) -> Fixed {
        self.dominance.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn stress_axis_recovers() {
        let mut axis = StressAxis {
            level: Fixed::from_f64(0.8),
            ..StressAxis::default()
        };
        let tone = Fixed::from_f64(0.5);
        // Acute input 0, so only recovery happens
        axis.update(
            Fixed::ZERO,
            tone,
            Fixed::from_f64(0.05),
            Fixed::from_f64(0.001),
            Fixed::from_f64(0.0005),
        );
        assert!(axis.level < Fixed::from_f64(0.8));
    }

    #[test]
    fn stress_axis_increases_with_input() {
        let mut axis = StressAxis {
            level: Fixed::from_f64(0.2),
            ..StressAxis::default()
        };
        let tone = Fixed::from_f64(0.5);
        axis.update(
            Fixed::from_f64(0.8),
            tone,
            Fixed::from_f64(0.05),
            Fixed::from_f64(0.001),
            Fixed::from_f64(0.0005),
        );
        assert!(axis.level > Fixed::from_f64(0.2));
    }

    #[test]
    fn stress_recovers_even_when_tone_is_zero_via_floor() {
        // Iteration 172 regression: before the tone floor, a zero tone
        // zeroed recovery at ANY rate — recovery = rate × 0 = 0 — so a
        // saturated stress axis was permanently pinned at 1.0 (the nervous
        // tone drains to 0 under sustained threat in calibrated windows).
        // With the floor, even a zero tone yields a basal recovery of
        // `rate × STRESS_RECOVERY_TONE_FLOOR`, so a clean axis (no chronic
        // load) converges to a stable equilibrium BELOW 1.0 instead of
        // saturating. (The saturated sub-population — chronic_load ~1.0 —
        // stays pinned until its chronic load decays; chronic_recovery
        // 0.0005/tick is the pacing bottleneck there, and the residual
        // bimodality is documented in the integration test and changelog.)
        //
        // Equilibrium with chronic_load = 0: level' = level + 0.02×0.5 −
        // 0.10×0.3 = level − 0.01/tick → drains to 0. Without the floor it
        // is +0.01/tick → climbs to 1.0 and pins.
        let mut axis = StressAxis {
            level: Fixed::from_f64(0.5),
            reactivity: Fixed::from_f64(0.5),
            chronic_load: Fixed::ZERO,
        };
        let acute = Fixed::from_f64(0.02);
        let zero_tone = Fixed::ZERO;
        let recovery_rate = Fixed::from_f64(0.10);
        for _ in 0..60 {
            axis.update(
                acute,
                zero_tone,
                recovery_rate,
                Fixed::from_f64(0.001),
                Fixed::from_f64(0.0005),
            );
        }
        let after = axis.level.to_f64();
        assert!(
            after < 0.2,
            "the tone floor must provide basal recovery at zero tone (got {after})"
        );
    }

    #[test]
    fn high_chronic_agent_recovers_when_acute_subsides() {
        // Iteration 189 (FINDING 10 — ratchet escape): before the
        // CHRONIC_FEEDBACK 0.02 fix, a saturated agent (chronic_load ~1.0 →
        // delta 0.1/tick) could NEVER drop below 0.6 because the chronic
        // delta exactly matched/exceeded the max recovery — the level stayed
        // pinned at 1.0 even after the acute stressor ended (the Iter-172
        // "chronic_recovery 0.0005/tick" escape was unreachable). With the
        // 0.02 feedback (max delta 0.02 < tone-floor recovery 0.03), the
        // same agent whose acute subsides must decay back below the 0.6
        // threshold, after which chronic_load itself begins to unwind.
        let mut axis = StressAxis {
            level: Fixed::from_f64(1.0),
            reactivity: Fixed::from_f64(0.5),
            chronic_load: Fixed::from_f64(1.0),
        };
        // Calm conditions: near-zero acute, full parasympathetic tone.
        for _ in 0..400 {
            axis.update(
                Fixed::from_f64(0.02), // residual acute only
                Fixed::ONE,            // full parasympathetic tone
                Fixed::from_f64(0.10), // recovery rate
                Fixed::from_f64(0.001),
                Fixed::from_f64(0.0005),
            );
        }
        assert!(
            axis.level < Fixed::from_f64(0.6),
            "a saturated agent must escape the pin when acute subsides (level {:.3})",
            axis.level.to_f64()
        );
        assert!(
            axis.chronic_load < Fixed::from_f64(0.9),
            "chronic load must begin unwinding after escape (chronic {:.3})",
            axis.chronic_load.to_f64()
        );
    }

    #[test]
    fn chronic_load_saturates_logistically_not_linearly() {
        // Iteration 189: the chronic accumulator is now saturating
        // (logistic) — `rate × (1 − chronic)` — so it approaches 1.0
        // asymptotically and can never be driven past it, unlike the old
        // linear `+ rate` accumulator.
        let mut axis = StressAxis {
            level: Fixed::from_f64(0.8), // above the 0.6 accumulation gate
            chronic_load: Fixed::ZERO,
            ..StressAxis::default()
        };
        for _ in 0..1_000_000 {
            axis.update(
                Fixed::from_f64(0.4), // sustained acute keeps level high
                Fixed::ZERO,          // zero tone (floor recovery only)
                Fixed::from_f64(0.10),
                Fixed::from_f64(0.001),
                Fixed::from_f64(0.0005),
            );
        }
        assert!(
            axis.chronic_load <= Fixed::ONE,
            "chronic_load must stay bounded (got {})",
            axis.chronic_load
        );
        assert!(
            axis.chronic_load > Fixed::from_f64(0.5),
            "sustained stress must accumulate meaningful chronic load (got {:.3})",
            axis.chronic_load.to_f64()
        );
    }

    #[test]
    fn arousal_decays_to_baseline() {
        let mut axis = ArousalAxis {
            level: Fixed::from_f64(0.9),
            baseline: Fixed::from_f64(0.3),
        };
        // No acute input, should decay
        axis.update(Fixed::ZERO, Fixed::from_f64(0.3), Fixed::from_f64(0.1));
        assert!(axis.level < Fixed::from_f64(0.9));
        assert!(axis.level > axis.baseline);
    }

    #[test]
    fn arousal_saturating_gain_cannot_pin_at_one() {
        // S2-2-4b regression: the linear rise form pinned 8–9/12 agents at
        // 1.0 in famine/pestilence (probe: arousal mean 0.78–0.81 with the
        // majority at exactly 1.0). The logistic (1 − level) factor bounds
        // the equilibrium at L* = (s·r + b·d)/(s·r + d): ≈0.77 at famine
        // stress 0.667, ≈0.61 at calm stress 0.26 — elevated but never 1.0.
        let mut famine = ArousalAxis {
            level: Fixed::from_f64(0.3),
            baseline: Fixed::from_f64(0.3),
        };
        // 20K ticks of sustained famine-scale stress (rise 0.3, decay 0.1).
        for _ in 0..20_000 {
            famine.update(Fixed::from_f64(0.667), Fixed::from_f64(0.3), Fixed::from_f64(0.1));
        }
        let famine_level = famine.level.to_f64();
        assert!(
            famine_level < 0.95,
            "saturating gain must keep arousal below 1.0 at famine stress (got {famine_level})"
        );
        assert!(
            famine_level > 0.7,
            "famine stress must still elevate arousal above baseline (got {famine_level})"
        );
        // Calm-scale stress settles strictly lower — directionally distinct.
        let mut calm = ArousalAxis {
            level: Fixed::from_f64(0.3),
            baseline: Fixed::from_f64(0.3),
        };
        for _ in 0..20_000 {
            calm.update(Fixed::from_f64(0.26), Fixed::from_f64(0.3), Fixed::from_f64(0.1));
        }
        let calm_level = calm.level.to_f64();
        assert!(
            calm_level < famine_level,
            "calm arousal must sit below famine arousal ({calm_level} vs {famine_level})"
        );
        assert!(calm_level > 0.4, "calm arousal should stay mildly elevated (got {calm_level})");
    }

    #[test]
    fn endocrine_state_deterministic() {
        let mut rng1 = rand::rngs::StdRng::seed_from_u64(42);
        let mut rng2 = rand::rngs::StdRng::seed_from_u64(42);
        let e1 = EndocrineState::random(&mut rng1);
        let e2 = EndocrineState::random(&mut rng2);
        assert_eq!(e1.stress.reactivity.to_raw(), e2.stress.reactivity.to_raw());
    }

    // ── §7.2.2 axis-liveness tests (S2-2-2 fix) ────────────────────
    // The bonding/dominance/growth axes were write-once (their update
    // methods had zero call sites); these pin the update contracts the sim
    // wiring now exercises.

    #[test]
    fn bonding_axis_rises_with_social_input_and_recovers() {
        let mut axis = BondingAxis {
            level: Fixed::from_f64(0.4),
            receptivity: Fixed::from_f64(0.5),
        };
        // Ritual-scale input (bonding_effect ~0.1-0.225) with the shipped
        // recovery 0.02: net positive, far from saturation.
        for _ in 0..5 {
            axis.update(Fixed::from_f64(0.2), Fixed::from_f64(0.02));
        }
        assert!(axis.level > Fixed::from_f64(0.4), "bonding must rise with ritual input");
        assert!(axis.level < Fixed::from_f64(0.9), "bonding must not saturate");
        // No input: recovery drains the axis back down.
        let before = axis.level;
        for _ in 0..3 {
            axis.update(Fixed::ZERO, Fixed::from_f64(0.02));
        }
        assert!(axis.level < before, "bonding must recover when input stops");
    }

    #[test]
    fn bonding_axis_saturating_gain_cannot_pin_at_one() {
        // S2-2-2 regression (reviewer-caught): the linear gain form ratcheted
        // to 1.0 at ~30K ticks of monthly ritual fires for max-receptivity
        // agents (step-function accumulation with no decay between fires).
        // The logistic (1 − level) factor bounds the axis at a stable
        // equilibrium below 1.0 no matter how many fires accumulate.
        let mut axis = BondingAxis {
            level: Fixed::from_f64(0.6), // max birth value
            receptivity: Fixed::from_f64(0.8), // max receptivity
        };
        // 11 monthly fires ≈ the 50K-tick long-horizon window.
        for _ in 0..11 {
            axis.update(Fixed::from_f64(0.1125), Fixed::from_f64(0.02));
        }
        let after = axis.level.to_f64();
        assert!(
            after < 0.95,
            "saturating gain must keep the axis below 1.0 at the 50K horizon (got {after})"
        );
        // And the equilibrium is monotone-increasing from below: more fires
        // keep approaching it without overshooting.
        let before = axis.level;
        axis.update(Fixed::from_f64(0.1125), Fixed::from_f64(0.02));
        assert!(axis.level > before);
        assert!(axis.level < Fixed::from_f64(0.95));
    }

    #[test]
    fn dominance_axis_tracks_status_target() {
        let mut axis = DominanceAxis {
            level: Fixed::from_f64(0.4),
        };
        // High-status agent (elite, effective_status ~0.8) converges up.
        for _ in 0..30 {
            axis.update(Fixed::from_f64(0.8), Fixed::from_f64(0.1));
        }
        assert!(axis.level > Fixed::from_f64(0.6), "dominance must rise toward high status");
        // Low-status agent (effective_status ~0.2) converges down.
        let mut low = DominanceAxis {
            level: Fixed::from_f64(0.4),
        };
        for _ in 0..30 {
            low.update(Fixed::from_f64(0.2), Fixed::from_f64(0.1));
        }
        assert!(low.level < Fixed::from_f64(0.3), "dominance must fall with low status");
    }

    #[test]
    fn dominance_aggression_modifier_exposes_live_level() {
        // Iteration 191: `dominance_aggression_modifier` is now wired at the
        // escalation decision (the S2-2-2 write-only axis finally feeds the
        // violence gate). The accessor must track the live axis level so the
        // escalation fold is a real status signal, not a constant.
        let mut elite = EndocrineState {
            dominance: DominanceAxis {
                level: Fixed::from_f64(0.4),
            },
            ..EndocrineState::default()
        };
        for _ in 0..30 {
            elite.dominance.update(Fixed::from_f64(0.9), Fixed::from_f64(0.1));
        }
        let mut drained = EndocrineState {
            dominance: DominanceAxis {
                level: Fixed::from_f64(0.4),
            },
            ..EndocrineState::default()
        };
        for _ in 0..30 {
            drained.dominance.update(Fixed::from_f64(0.1), Fixed::from_f64(0.1));
        }
        assert!(
            elite.dominance_aggression_modifier()
                > drained.dominance_aggression_modifier(),
            "the aggression modifier must differentiate status: {} vs {}",
            elite.dominance_aggression_modifier().to_f64(),
            drained.dominance_aggression_modifier().to_f64()
        );
    }

    #[test]
    fn growth_axis_converges_to_life_stage_target() {
        use crate::biology::development::LifeStage;
        let mut axis = GrowthAxis::default(); // capacity 0.6
        // Adolescent target 1.0: capacity rises.
        axis.update(LifeStage::Adolescent.growth_modifier(), Fixed::from_f64(0.01));
        assert!(axis.capacity > Fixed::from_f64(0.6));
        assert_eq!(
            axis.developmental_modifier,
            LifeStage::Adolescent.growth_modifier()
        );
        // Elder target 0.2: capacity falls and developmental_modifier mirrors it.
        let mut elder = GrowthAxis {
            capacity: Fixed::from_f64(0.6),
            developmental_modifier: Fixed::ONE,
        };
        elder.update(LifeStage::Elder.growth_modifier(), Fixed::from_f64(0.01));
        assert!(elder.capacity < Fixed::from_f64(0.6));
        assert_eq!(elder.developmental_modifier, LifeStage::Elder.growth_modifier());
    }
}
