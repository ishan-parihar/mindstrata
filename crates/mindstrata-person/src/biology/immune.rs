//! Immune system — disease resistance, inflammation, infection, and recovery.
//!
//! Interacts with nutrition, stress, sleep, age, injury, hygiene, and crowding.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Quantize a strictly-positive per-tick rate to `Fixed` without annihilating
/// it (i311 §5).
///
/// `Fixed::from_f64` rounds to nearest, so a positive rate below half a step
/// (5e-5) becomes exactly ZERO. For infection kinetics that is not a rounding
/// detail but a freeze: with `resistance` at the floor, the clearance term is
/// `0.15 × 0.13 × 0.005 × I × mult`, i.e. `0.975·I` raw units — so any load
/// below `I ≈ 0.513` clears at exactly zero per tick. A calm agent (stress
/// ≤ 0.4, no exposure) that ends up with a residual load would then keep it
/// forever (probe `i311_immune_clearance` measured 3 calm / 10 collapse agents
/// in the frozen band at the 20K horizon).
///
/// The §5 rule prescribed computing in f64 and quantizing once; that alone is
/// insufficient when the *rate itself* is sub-resolution, which is the case
/// here — the second remedy on record is an f64 shadow accumulator, but for a
/// strictly-positive monotone rate an f64→Fixed quote that rounds UP to one
/// step is equivalent and avoids a new serialized field. Rates are positive by
/// construction at every call site below.
fn quantize_rate(v: f64) -> Fixed {
    let f = Fixed::from_f64(v);
    if v > 0.0 && f.to_f64() == 0.0 {
        Fixed::from_raw(1)
    } else {
        f
    }
}

/// Innate-immunity floor (i311).
///
/// Resistance used to be absorptive at zero: stress suppression is a flat
/// `stress × 0.001` per tick while the nutrition and sleep boosts are
/// `quality × 0.002/0.001 × (1 − R)`, so any agent whose per-tick suppression
/// exceeds its boosts (measured: stress 0.73–0.79 against nutrition at the
/// 0.1 floor + sleep) decays resistance to ~0 and never regains competence.
/// With `R ≈ 0.0007` the pathogen clearance term is ~1e-7 per tick — below the
/// Fixed-4 resolution of 1e-4 (AGENTS §5 sub-resolution class) — so infection
/// pinned at 1.0000 and sickness at 0.84: the measured signature of the calm
/// village's 3.5% frailty population (`i310_frailty_share`).
///
/// A body does not lose innate immunity entirely; 0.15 keeps competence
/// non-absorptive so the stress suppression no longer drives resistance to a
/// dead zero. (Clearance representability at low loads is handled separately
/// by `quantize_rate`, not by this floor.)
const INNATE_IMMUNITY_FLOOR: Fixed = Fixed::from_raw(1_500);

/// Immune state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmuneState {
    /// Iteration 252: hazard-accumulated epidemic immunity (0–1). Set to
    /// 1.0 when an Epidemic disease course completes, decays ~0.0005/day
    /// (generational scale). Multiplies exposure/transmission by
    /// (1 − immunity): survivor immunity turns the R0≈1 knife-edge into
    /// damped burnout against a stable immunity wall.
    pub epidemic_immunity: Fixed,
    /// Baseline resistance (0 = immunocompromised, 1 = robust).
    pub resistance: Fixed,
    /// Current inflammation level (0 = none, 1 = systemic).
    pub inflammation: Fixed,
    /// Active infection load (0 = healthy, 1 = severely infected).
    pub infection_load: Fixed,
    /// Recovery capacity — how fast the body fights infection.
    pub recovery_capacity: Fixed,
    /// Autoimmune risk — chronic inflammation from overactive immunity.
    pub autoimmune_risk: Fixed,
}

impl Default for ImmuneState {
    fn default() -> Self {
        Self {
            resistance: Fixed::from_f64(0.6),
            inflammation: Fixed::ZERO,
            infection_load: Fixed::ZERO,
            recovery_capacity: Fixed::from_f64(0.5),
            autoimmune_risk: Fixed::ZERO,
            epidemic_immunity: Fixed::ZERO,
        }
    }
}

impl ImmuneState {
    /// Update immune state each tick.
    pub fn tick_update(
        &mut self,
        nutrition_quality: Fixed,
        stress_level: Fixed,
        sleep_quality: Fixed,
        injury_severity: Fixed,
        crowding: Fixed,
        hygiene: Fixed,
        age_modifier: Fixed,
        recovery_rate: Fixed,
        disease_susceptibility: Fixed,
    ) {
        // §7.2.6 (S2-2-4 fix): *saturating* boosts. The old linear form
        // (nutrition +0.002/tick, sleep +0.001/tick vs stress −0.001/tick,
        // NO decay term) was net-positive at every calibrated input, so
        // resistance ratcheted to 1.000 in EVERY window including
        // famine/pestilence (probe-pinned). The (1 − resistance) factor
        // makes each boost vanish as the axis approaches 1.0, giving a
        // stress-dependent equilibrium
        // `R* = 1 − (stress·0.001 + age·0.0001)/(nutrition·0.002 + sleep·0.001)`
        // ≈ 0.83 calm / ≈ 0.60 famine at the formula's assumed inputs
        // (sleep_quality 0.47); measured probe values are 0.764 calm /
        // 0.525 famine (real sim sleep averages lower) — robust under good
        // conditions, suppressed under chronic stress, never pinned. Same
        // logistic pattern as the S2-2-1/2-2-2/2-2-3 fixes.
        let nutrition_boost =
            nutrition_quality * Fixed::from_f64(0.002) * (Fixed::ONE - self.resistance);
        self.resistance = (self.resistance + nutrition_boost).clamp_01();

        // Stress suppresses immune function
        let stress_suppression = stress_level * Fixed::from_f64(0.001);
        self.resistance = (self.resistance - stress_suppression).max(Fixed::ZERO);

        // Sleep is critical for immune recovery (saturating, same rationale)
        let sleep_boost = sleep_quality * Fixed::from_f64(0.001) * (Fixed::ONE - self.resistance);
        self.resistance = (self.resistance + sleep_boost).clamp_01();

        // Age reduces immune function
        self.resistance =
            (self.resistance - age_modifier * Fixed::from_f64(0.0001)).max(Fixed::ZERO);

        // i311: competence is not absorptive — see `INNATE_IMMUNITY_FLOOR`.
        self.resistance = self.resistance.max(INNATE_IMMUNITY_FLOOR);

        // Infection triggers immune response
        if self.infection_load > Fixed::ZERO {
            // Inflammation rises to fight infection
            self.inflammation = (self.infection_load * Fixed::from_f64(0.6)).clamp_01();
            // Immune system fights infection. §7.2.6 (S2-2-4 fix): the fight
            // term is now PROPORTIONAL to the infection load (immune
            // response ∝ pathogen load — the natural form). The old flat
            // fight (`R×C×0.005` per tick) dominated any per-tick exposure
            // at realistic rates, so infection could never accumulate above
            // ~0 — the reachable stress channel below would have hovered at
            // 0↔E forever. Proportional fight gives a stable chronic-
            // infection equilibrium `I* = E/(R·C·0.005)` ≈ 0.10 at famine
            // (stress 0.667, hygiene 0.6, crowding 0.3) and ≈ 0.46 for a
            // severe open wound (injury 1.0) — a real, self-limiting
            // infected state instead of a flicker. (The wound path's
            // equilibrium shifted from ~0 under the old flat fight to
            // ~0.46; injury stays ~0 in calibrated windows, so no pinned
            // behavior changes.)
            // i311 (§5 sub-resolution rule): the clearance chain is computed
            // in f64 and quantized ONCE. The previous all-Fixed chain
            // truncated at every multiply — `resistance × recovery_capacity`
            // alone (0.0007 × 0.04 = 2.8e-5) quantized to ZERO — so an
            // immunocompromised body cleared no pathogen whatsoever and its
            // infection could only ratchet upward. Same defect family as the
            // Iteration-239/240/242/244 truncation kills (AGENTS §5).
            //
            // Iteration 244: genome recovery_rate scales pathogen clearance
            // (±30% around 1.0) — previously a dead gene.
            let fight_f = self.resistance.to_f64()
                * self.recovery_capacity.to_f64()
                * 0.005
                * self.infection_load.to_f64()
                * (0.7 + recovery_rate.to_f64() * 0.6);
            let fight = quantize_rate(fight_f);
            self.infection_load = (self.infection_load - fight).max(Fixed::ZERO);
            // Chronic inflammation damages resistance
            if self.inflammation > Fixed::from_f64(0.5) {
                self.autoimmune_risk = (self.autoimmune_risk + Fixed::from_f64(0.0001)).clamp_01();
            }
        } else {
            // No infection → inflammation decays
            self.inflammation = (self.inflammation - Fixed::from_f64(0.01)).max(Fixed::ZERO);
        }

        // §7.2.6 (S2-2-4 fix → Iter-213 derived): the old crowding/hygiene
        // exposure gate was UNREACHABLE — the sim used hardcoded 0.3/0.6
        // so `(crowding − 0.5)` was negative forever and infection_load
        // never left 0 in ANY scenario
        // (probe-pinned; the entire infection→inflammation→fight path was
        // dead state). Replaced with a stress-driven exposure channel:
        // chronic stress suppresses immune surveillance and opens infection
        // channels. Stress varies 0.25 (calm) → 0.67 (famine/pestilence),
        // so exposure is strictly scenario-directional; the hygiene
        // shortfall multiplies it, and crowding amplifies it (the sim
        // wires crowding at 0.3 → a mild ≈0.8× multiplier — low but live).
        // The 0.4 threshold keeps the calm MAJORITY (stress ≈ 0.26 < 0.4)
        // at zero exposure — golden-safe (the 1000-tick golden window
        // never crosses 0.4); only the documented chronic-stress
        // sub-population (stress pinned near 1.0, Iter-172 residual) opens
        // low-grade infection channels, which is precisely the mechanism
        // this fix is meant to expose.
        if stress_level > Fixed::from_f64(0.4) {
            let crowding_amplifier = Fixed::from_f64(0.5) + crowding;
            // Iteration 244: genome disease_susceptibility scales exposure
            // intake (±30% around 1.0) — previously a dead gene.
            // Computed in f64 and quantized ONCE: a Fixed-only chain would
            // truncate the ~0.00017 exposure to zero at the 4-decimal scale
            // (the Iteration-239/240/242 truncation-disease class).
            //
            // i311: the channel is now *saturating* — `(1 − infection_load)`,
            // the same logistic form the resistance boosts use (S2-2-4). The
            // old constant intake against a load-proportional clearance has
            // equilibrium `I* = E/(R·C·0.005·mult)`, which for the chronic-
            // stress sub-population (stress 0.75–0.81) means E ≈ 2.5e-4 against
            // a clearance coefficient of only R·C·0.005 ≈ 9.8e-5 at the
            // resistance floor — `I* ≈ 2.6`, i.e. clamped to 1.0000 forever
            // (probe anatomy: every pinned agent at R=0.15, C=0.13, stress≈0.78).
            // A body already fully infected cannot be infected further, so
            // exposure must vanish as the load approaches saturation: the
            // equilibrium becomes `I* = E/(E + k)`, strictly below 1 for any
            // positive clearance.
            let suscept_mult = 0.7 + disease_susceptibility.to_f64() * 0.6;
            let exposure = Fixed::from_f64(
                (stress_level.to_f64() - 0.4)
                    * (1.0 - hygiene.to_f64())
                    * crowding_amplifier.to_f64()
                    * 0.002
                    * suscept_mult
                    * (1.0 - self.infection_load.to_f64()),
            );
            self.infection_load = (self.infection_load + exposure).clamp_01();
        }

        // Injury increases infection vulnerability
        if injury_severity > Fixed::from_f64(0.5) {
            let wound_infection = (injury_severity - Fixed::from_f64(0.5)) * Fixed::from_f64(0.002);
            self.infection_load = (self.infection_load + wound_infection).clamp_01();
        }

        // Recovery capacity derived from resistance and nutrition
        self.recovery_capacity = (self.resistance * Fixed::from_f64(0.6)
            + nutrition_quality * Fixed::from_f64(0.4))
        .clamp_01();
    }

    /// Sickness severity for derived body state (0–1).
    pub fn sickness_level(&self) -> Fixed {
        (self.infection_load * Fixed::from_f64(0.6) + self.inflammation * Fixed::from_f64(0.4))
            .clamp_01()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nutrition_boosts_resistance() {
        let mut immune = ImmuneState::default();
        let initial = immune.resistance;
        for _ in 0..50 {
            immune.tick_update(
                Fixed::from_f64(0.9),
                Fixed::ZERO,
                Fixed::from_f64(0.8),
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.4),
            );
        }
        assert!(immune.resistance > initial);
    }

    #[test]
    fn infection_triggers_inflammation() {
        let mut immune = ImmuneState {
            infection_load: Fixed::from_f64(0.5),
            ..Default::default()
        };
        immune.tick_update(
            Fixed::ONE,
            Fixed::ZERO,
            Fixed::ONE,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ONE,
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.4),
        );
        assert!(immune.inflammation > Fixed::ZERO);
    }

    #[test]
    fn stress_suppresses_immunity() {
        let mut immune = ImmuneState {
            resistance: Fixed::from_f64(0.7),
            ..Default::default()
        };
        for _ in 0..100 {
            immune.tick_update(
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.9),
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ONE,
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.4),
            );
        }
        assert!(immune.resistance < Fixed::from_f64(0.7));
    }

    #[test]
    fn resistance_does_not_saturate_with_saturating_boosts() {
        // S2-2-4 regression: nutrition +0.002 + sleep +0.001 dwarfed stress
        // suppression −0.001 with no decay term, so resistance ratcheted to
        // 1.000 in every window including famine (probe-pinned). The
        // saturating boosts give a stress-dependent equilibrium ≈0.83 calm
        // (stress 0.26), ≈0.60 famine (stress 0.667) — differentiated and
        // never pinned.
        let mut calm = ImmuneState::default();
        for _ in 0..10_000 {
            calm.tick_update(
                Fixed::from_f64(0.6),
                Fixed::from_f64(0.26),
                Fixed::from_f64(0.47),
                Fixed::ZERO,
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.6),
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.4),
            );
        }
        let calm_res = calm.resistance.to_f64();
        assert!(
            calm_res < 0.95,
            "calm resistance must not saturate at 1.0 (got {calm_res})"
        );
        let mut famine = ImmuneState::default();
        for _ in 0..10_000 {
            famine.tick_update(
                Fixed::from_f64(0.6),
                Fixed::from_f64(0.667),
                Fixed::from_f64(0.47),
                Fixed::ZERO,
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.6),
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.4),
            );
        }
        let famine_res = famine.resistance.to_f64();
        assert!(
            famine_res < calm_res,
            "famine resistance must sit below calm ({famine_res} vs {calm_res})"
        );
        assert!(
            famine_res > 0.3,
            "famine resistance should not collapse (got {famine_res})"
        );
    }

    #[test]
    fn stress_driven_exposure_gate_is_reachable_and_directional() {
        // S2-2-4 regression: the old gate required crowding > 0.5 AND
        // hygiene < 0.3, but the sim used hardcoded 0.3/0.6, so
        // `(crowding − 0.5)` was negative forever and infection_load never
        // left 0 in ANY scenario. Now crowding/hygiene derive from world
        // state (Iter-213), but the stress-driven channel remains primary. The stress-driven channel fires only
        // above stress 0.4 — calm (0.26) stays untouched, famine (0.667)
        // accumulates a visible, self-limiting chronic infection.
        let mut calm = ImmuneState::default();
        for _ in 0..5_000 {
            calm.tick_update(
                Fixed::from_f64(0.6),
                Fixed::from_f64(0.26),
                Fixed::from_f64(0.47),
                Fixed::ZERO,
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.6),
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.4),
            );
        }
        assert_eq!(
            calm.infection_load,
            Fixed::ZERO,
            "calm stress must not open infection channels"
        );
        let mut famine = ImmuneState::default();
        for _ in 0..5_000 {
            famine.tick_update(
                Fixed::from_f64(0.6),
                Fixed::from_f64(0.667),
                Fixed::from_f64(0.47),
                Fixed::ZERO,
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.6),
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.4),
            );
        }
        let load = famine.infection_load.to_f64();
        assert!(
            load > 0.0,
            "famine stress must open infection channels (got {load})"
        );
        assert!(load < 1.0, "infection must not pin at 1.0 (got {load})");
    }
}

#[cfg(test)]
mod immunity_tests {
    use super::*;

    #[test]
    fn default_population_has_no_immunity() {
        let d = ImmuneState::default();
        assert_eq!(d.epidemic_immunity, Fixed::ZERO);
    }

    #[test]
    fn immunity_is_bounded_and_representable() {
        // The daily decay decrement (0.0005) must survive Fixed-4
        // quantization — the truncation disease would silently disable
        // the decay and freeze the immunity wall permanently.
        let step = Fixed::from_f64(0.0005);
        assert!(step > Fixed::ZERO, "decay step quantized to zero");
        let full = Fixed::ONE;
        assert_eq!(
            Fixed::from_f64(full.to_f64() - step.to_f64()) < full,
            true,
            "one decay day must strictly reduce full immunity"
        );
    }
}

#[cfg(test)]
mod clearance_tests {
    use super::*;

    /// Helper mirroring the sim's wiring: nutrition at its floor (0.1),
    /// hygiene 0.6, crowding 0.3 (systems/biology.rs), age_modifier 0.3.
    fn tick(immune: &mut ImmuneState, stress: f64, nutrition: f64) {
        immune.tick_update(
            Fixed::from_f64(nutrition),
            Fixed::from_f64(stress),
            Fixed::from_f64(0.47),
            Fixed::ZERO,
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.4),
        );
    }

    #[test]
    fn chronic_stress_infection_self_limits_below_saturation() {
        // i310 measured the calm village's frailty population as an infection
        // load pinned at 1.0000 over a resistance collapsed to ~0.000. With
        // the resistance floor and saturating exposure the chronic-stress
        // equilibrium must be a real, bounded state — not the clamp.
        let mut immune = ImmuneState::default();
        for _ in 0..50_000 {
            tick(&mut immune, 0.78, 0.1);
        }
        let load = immune.infection_load.to_f64();
        assert!(
            load > 0.0,
            "chronic stress must still open an infection channel (got {load})"
        );
        assert!(
            load < 0.95,
            "chronic-stress infection must not pin at saturation (got {load})"
        );
    }

    #[test]
    fn resistance_floor_is_non_absorptive_under_max_stress() {
        // Stress suppression (flat 0.001/tick) used to outrun the nutrition and
        // sleep boosts (which carry a `1 − resistance` factor), decaying
        // resistance to ~0 and making immunocompetence unrecoverable.
        let mut immune = ImmuneState::default();
        for _ in 0..50_000 {
            tick(&mut immune, 1.0, 0.1);
        }
        assert_eq!(
            immune.resistance,
            Fixed::from_raw(1_500),
            "maximal chronic stress must pin resistance at the innate floor, not zero"
        );
    }

    #[test]
    fn residual_infection_clears_at_the_resistance_floor() {
        // §5 freeze regression: at the floor (R 0.15, C 0.13) the clearance
        // term is `0.975·I` raw units, so any load below I ≈ 0.513 rounds to
        // ZERO per tick. Without `quantize_rate` this load would be frozen
        // forever once stress subsides and exposure stops.
        let mut immune = ImmuneState {
            resistance: Fixed::from_raw(1_500),
            recovery_capacity: Fixed::from_f64(0.13),
            infection_load: Fixed::from_f64(0.3),
            ..Default::default()
        };
        let before = immune.infection_load;
        // One step/tick (1e-4) takes ~3 000 ticks to clear 0.3 — the point is
        // that it MOVES at all (without `quantize_rate` it is exactly zero).
        for _ in 0..200 {
            // stress below the 0.4 exposure gate — no intake at all.
            tick(&mut immune, 0.2, 0.1);
        }
        assert!(
            immune.infection_load < before,
            "a positive clearance rate must never quantize to zero \
             (before {before:?}, after {:?})",
            immune.infection_load
        );
        for _ in 0..20_000 {
            tick(&mut immune, 0.2, 0.1);
        }
        assert_eq!(
            immune.infection_load,
            Fixed::ZERO,
            "a residual load must fully clear once exposure stops"
        );
    }
}
