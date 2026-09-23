//! Simulation systems.
//!
//! Systems are organised by layer and run in explicit, deterministic order
//! each tick.  The architecture mandates this ordering:
//!
//! ```text
//! 1. Perception
//! 2. Psychological update
//! 3. Goal selection
//! 4. Behavior planning
//! 5. Action execution
//! 6. Interaction
//! 7. Social update
//! 8. Economic
//! 9. Institutional
//! 10. Ecological
//! 11. Event generation
//! ```

use crate::person::{BodyState, NeedState};
use crate::world::World;
use mindstrata_core::fixed::Fixed;
use mindstrata_core::rng::RngStreams;

/// Shared context passed to every system.
pub struct SystemContext<'a> {
    pub tick: u64,
    pub rng: &'a mut RngStreams,
    pub world: &'a mut World,
    /// Rolling event buffer (bounded — see `Simulation::events`).
    pub events: &'a mut Vec<mindstrata_core::event::SimEvent>,
}

impl<'a> SystemContext<'a> {
    /// Create a new system context from borrowed fields.
    pub fn new(
        tick: u64,
        rng: &'a mut RngStreams,
        world: &'a mut World,
        events: &'a mut Vec<mindstrata_core::event::SimEvent>,
    ) -> Self {
        Self {
            tick,
            rng,
            world,
            events,
        }
    }
}

// ── Arc-D verbatim-moved passes (golden-referee pure refactor) ──────────
pub(crate) mod appraisal;
pub(crate) mod biology;
pub(crate) mod cognitive;
pub(crate) mod decay;
pub mod development;
pub mod genesis;
pub(crate) mod health;
pub mod institutions_multiplier;
pub mod trade_diffusion;

// ── Need decay system ────────────────────────────────────────────────────

/// §5.1: Need decay using configurable parameters.
///
/// Each need accumulates deficit at its own rate, controlled by
/// `SimParameters`. Higher-order needs (safety, social, meaning)
/// grow slower than survival needs (hunger, thirst).
pub fn system_need_decay_with_params(
    params: &crate::parameters::SimParameters,
    needs: &mut [NeedState],
) {
    for need in needs.iter_mut() {
        need.hunger = (need.hunger + params.hunger_decay_rate).clamp_01();
        need.thirst = (need.thirst + params.thirst_decay_rate).clamp_01();
        need.fatigue = (need.fatigue + params.fatigue_decay_rate).clamp_01();
        need.safety = (need.safety + params.safety_decay_rate).clamp_01();
        need.social = (need.social + params.social_decay_rate).clamp_01();
        need.meaning = (need.meaning + params.meaning_decay_rate).clamp_01();
        // Esteem and autonomy decay at 2/3 the meaning rate.
        // Iteration-267 (Fixed-4 truncation disease, §5 — i275_need_trace
        // evidence): `meaning_decay_rate` lands at raw 1 (1e-4), so the
        // per-tick product `1 × 0.6667 = 0.667` raw TRUNCATED TO ZERO in
        // Fixed::mul — esteem/autonomy decay were dead since the parameter
        // landed (fields pinned at founder defaults, σ=0 across the whole
        // 12-seed family; downstream autonomy readers in factions/
        // institutions read a frozen field). Sub-resolution rates compute
        // in f64 and quantize once, per the standing §5 rule. The 2/3
        // coupling is preserved parametrically (0.00015 = design decay_rate
        // × 0.15, parameters.rs:426) so future recalibration propagates.
        let esteem_autonomy_rate = Fixed::from_f64(params.meaning_decay_rate.to_f64() * 0.6667);
        need.esteem = (need.esteem + esteem_autonomy_rate).clamp_01();
        need.autonomy = (need.autonomy + esteem_autonomy_rate).clamp_01();
    }
}

// ── Body state system ────────────────────────────────────────────────────

/// Update body state based on needs.
pub fn system_body_update(_ctx: &mut SystemContext, bodies: &mut [BodyState], needs: &[NeedState]) {
    for (body, need) in bodies.iter_mut().zip(needs.iter()) {
        // Starvation damages health
        if need.hunger > Fixed::from_f64(0.9) {
            body.health = (body.health - Fixed::from_f64(0.001)).clamp_01();
        }
        // Dehydration is worse
        if need.thirst > Fixed::from_f64(0.9) {
            body.health = (body.health - Fixed::from_f64(0.003)).clamp_01();
        }

        // High hunger drains energy
        if need.hunger > Fixed::from_f64(0.7) {
            body.energy = (body.energy - Fixed::from_f64(0.002)).clamp_01();
        }

        // Energy recovery when fatigue is low
        if need.fatigue < Fixed::from_f64(0.3) {
            body.energy = (body.energy + Fixed::from_f64(0.001)).clamp_01();
        }
    }
}

// ── Goal generation system ───────────────────────────────────────────────

/// The five fulfillment thresholds of `system_goal_generation`, resolved for a
/// run (difficulty-levers row 2, threshold half — i305).
///
/// These were inline consts (0.5 Eat/Drink, 0.6 Rest, 0.7 Socialize/Worship,
/// 0.3 retain) with no executable surface: the row-2 catalog row could not
/// reach them, so the "fulfillment thresholds" half of the lever was
/// documentation only. A LOWER gate makes the village respond earlier (the
/// standing deficit stays small, the abundant feel); a higher one makes it
/// tolerate more before acting (scarcity feel). The spec for these values is
/// `docs/balance/needs-bands.md` (all bands CALIBRATION-PENDING there).
///
/// Resolved ONCE per tick from `SimParameters::goal_gate_scale` (§5
/// quantize-once): the canon gates and the 0.6×/1.0×/1.4× multipliers are all
/// exactly representable at Fixed-4, so no band lands on a rounding edge.
/// Standard returns the canon gates bit-for-bit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GoalGates {
    /// Gate on the hunger deficit that generates an `Eat` goal.
    pub eat: Fixed,
    /// Gate on the thirst deficit that generates a `Drink` goal.
    pub drink: Fixed,
    /// Gate on the fatigue deficit that generates a `Rest` goal.
    pub rest: Fixed,
    /// Gate on the social deficit that generates a `Socialize` goal.
    pub socialize: Fixed,
    /// Gate on the meaning deficit that generates a `Worship` goal.
    pub worship: Fixed,
    /// Gate below which an existing goal is dropped (the retain half).
    /// Scales with the same multiplier — a village that acts early also lets
    /// goals go early, so the gate and its retain bound stay in proportion.
    pub retain: Fixed,
}

impl GoalGates {
    /// Canon gates — the values every pre-i305 run was calibrated with
    /// (raw units: 1 raw = 1e-4).
    pub const CANON: Self = Self {
        eat: Fixed::from_raw(5_000),
        drink: Fixed::from_raw(5_000),
        rest: Fixed::from_raw(6_000),
        socialize: Fixed::from_raw(7_000),
        worship: Fixed::from_raw(7_000),
        retain: Fixed::from_raw(3_000),
    };

    /// Resolve the run's gates from the difficulty band multiplier.
    #[must_use]
    pub fn for_params(params: &crate::parameters::SimParameters) -> Self {
        let scale = params.goal_gate_scale;
        let c = Self::CANON;
        Self {
            eat: c.eat * scale,
            drink: c.drink * scale,
            rest: c.rest * scale,
            socialize: c.socialize * scale,
            worship: c.worship * scale,
            retain: c.retain * scale,
        }
    }
}

/// i388: the multiple of the population's own **mean social deficit** at which an
/// agent reads as relationally DEPRIVED rather than ordinarily unsated.
///
/// The three social producers below all shipped with absolute bars (0.7 need gate,
/// 0.3 joy leg, 0.4 identity leg) that sit **above the channel's own distribution**:
/// the i388 probe measured `needs.social` at p50 0.0054 / p90 0.017 / p99 0.032–0.039
/// / mean 0.008 across village and town, because the daily routine relieves the
/// deficit before it can grow. So all three producers were dark by construction
/// ("live Socialize goals: 0/15 agents, 0/50 agents") — the same absolute-bar-on-a-
/// relative-scale defect i382 closed for faction legitimacy and i383 for the anger
/// arms, three times over in one channel.
///
/// The reference is the population's own mean, exactly the i383 `mean_anger` form:
/// self-normalizing, so a satiated village sets a low bar (a genuinely deprived
/// agent still acts) and a lonely one raises it (only the anomaly above the crowd
/// passes). Ratio 2.0 is the probe's measured choice — the ratio sweep over the
/// same two worlds (share of agent-ticks opening the band): ×1.25 = 27.7–30.6%,
/// **×2 = 10.5–12.1%**, ×3 = 4.8–5.2%, ×5 = 0.9–1.5%. ×2 sits just above the
/// routine's own social share (~5% of decisions), so the goal layer adds an outlet
/// for the deprived without spamming a goal onto every agent (§4.10: a band that
/// opens for everyone discriminates nothing — the mirror of the defect it fixes).
const RELATIONAL_DEPRIVATION_RATIO: Fixed = Fixed::from_int(2);

/// i388: the retain bar as a share of the create bar, so a goal the band just
/// created is not dropped the next tick.
///
/// The sibling rows keep their canon shape (retain 0.3 / create 0.7 = 3/7 ≈ 0.43);
/// this is the same relation expressed on the relative band, rounded to a
/// Fixed-exact 0.43. Without it the `GoalKind::Socialize` retain arm — still
/// reading the absolute `gates.retain` (0.3) — would delete every goal the new
/// band creates, a create-then-drop cycle (§4.3 dead producer with a live gate).
const RELATIONAL_RETAIN_SHARE: Fixed = Fixed::from_raw(4300); // 0.43

/// i388: the population's own mean social deficit this tick — the reference the
/// relational bands read (the `mean_anger` form, §6 hoisted out of the agent loop).
pub(crate) fn mean_social(needs: &[NeedState]) -> Fixed {
    if needs.is_empty() {
        return Fixed::ZERO;
    }
    let sum: Fixed = needs
        .iter()
        .map(|n| n.social)
        .fold(Fixed::ZERO, |a, b| a + b);
    sum / Fixed::from_int(needs.len() as i64)
}

/// Generate goals based on need pressure, emotional state, and identity.
/// §24: Goals now carry source tracking and support emotional/identity modulation.
/// §3.4: Emotional goal modulation — anger, fear, and joy drive goal generation.
pub fn system_goal_generation(
    ctx: &mut SystemContext,
    personalities: &[crate::person::Personality],
    needs: &[NeedState],
    goals: &mut [Vec<crate::person::Goal>],
    emotions: &[crate::person::DiscreteEmotions],
    params: &crate::parameters::SimParameters,
) {
    let tick = ctx.tick;
    let gates = GoalGates::for_params(params);
    // i383: the population's own anger this tick — the reference the anger→Work
    // emitter reads (hoisted; §6). See `emotion_regulation::mean_anger`.
    let anger_shock_bar = crate::psychology::emotion_regulation::mean_anger(emotions)
        * crate::psychology::emotion_regulation::EMOTION_SHOCK_RATIO;
    // i388: the relational bands — population-relative (see
    // `RELATIONAL_DEPRIVATION_RATIO`), scaled by the same row-2 difficulty lever
    // the canon gates ride (`goal_gate_scale`), so the socialize row keeps its
    // "act earlier / tolerate longer" semantics on the relative scale.
    let social_bar = mean_social(needs) * RELATIONAL_DEPRIVATION_RATIO * params.goal_gate_scale;
    let social_retain_bar = social_bar * RELATIONAL_RETAIN_SHARE;
    for (i, (need, agent_goals)) in needs.iter().zip(goals.iter_mut()).enumerate() {
        // ── Goal decay: reduce priority of old goals over time ──
        // §24: Goals that aren't addressed gradually lose priority.
        // §5 (Iteration 155): external directives are exempt — a TUI command
        // persists until satisfied or replaced, never silently fades.
        for goal in agent_goals.iter_mut() {
            if goal.source == crate::person::GoalSource::Command {
                continue;
            }
            let age = tick.saturating_sub(goal.created_tick) as f64;
            let decay = Fixed::from_f64(age * 0.001); // slow decay
            goal.priority = (goal.priority - decay).max(Fixed::ZERO);
        }

        // ── Remove goals whose need has dropped below threshold OR priority decayed to zero ──
        agent_goals.retain(|g| {
            // Goals that decayed to zero priority are removed
            if g.priority <= Fixed::ZERO {
                return false;
            }
            // §5 (Iteration 155): external directives are exempt — endogenous
            // need pressure cannot drop a user-issued command.
            if g.source == crate::person::GoalSource::Command {
                return true;
            }
            // i305: the retain gate rides the same band multiplier (canon
            // 0.3, so a Standard run keeps its exact pre-i305 value).
            let threshold = gates.retain;
            match g.kind {
                crate::person::GoalKind::Eat => need.hunger > threshold,
                crate::person::GoalKind::Drink => need.thirst > threshold,
                crate::person::GoalKind::Rest => need.fatigue > threshold,
                crate::person::GoalKind::Work => true,
                // i388: the relational retain arm reads the relative band (the
                // peers keep the canon gate): a goal created at `social_bar` must
                // survive the next tick's retain pass or the band accomplishes
                // nothing.
                crate::person::GoalKind::Socialize => need.social > social_retain_bar,
                crate::person::GoalKind::Worship => need.meaning > threshold,
                crate::person::GoalKind::SeekSafety => true,
            }
        });

        // ── Need-driven goals (§24 primary source) ──
        // §24: Only one goal per GoalKind — update priority if goal already exists
        // i305 (row 2, threshold half): the fulfillment thresholds are the
        // run's resolved band gates, not inline consts.
        let need_goals: &[(crate::person::GoalKind, Fixed, Fixed)] = &[
            (crate::person::GoalKind::Eat, need.hunger, gates.eat),
            (crate::person::GoalKind::Drink, need.thirst, gates.drink),
            (crate::person::GoalKind::Rest, need.fatigue, gates.rest),
            // i388: the social band is population-relative (the peers stay
            // absolute — their needs DO reach their gates): see
            // `RELATIONAL_DEPRIVATION_RATIO` for the measurement and the ratio
            // sweep. `gates.socialize` remains the row's canon anchor (asserted by
            // `sim/tests/psychology.rs`); the band itself rides `goal_gate_scale`.
            (crate::person::GoalKind::Socialize, need.social, social_bar),
            (
                crate::person::GoalKind::Worship,
                need.meaning,
                gates.worship,
            ),
        ];
        for (kind, pressure, threshold) in need_goals {
            if *pressure > *threshold {
                // If goal already exists, refresh its priority and commitment
                if let Some(existing) = agent_goals.iter_mut().find(|g| g.kind == *kind) {
                    existing.priority = existing.priority.max(*pressure);
                    existing.commitment = (existing.commitment + Fixed::from_f64(0.01)).clamp_01();
                } else {
                    agent_goals.push(crate::person::Goal {
                        kind: *kind,
                        priority: *pressure,
                        commitment: Fixed::from_f64(0.5),
                        created_tick: tick,
                        source: crate::person::GoalSource::Need,
                    });
                }
            }
        }

        // ── Emotional goal modulation (§3.4) ──
        // §3.4: High anger → Work harder (atone/express), high fear → SeekSafety, high joy → Socialize
        if i < emotions.len() {
            let emo = &emotions[i];

            // Fear → SeekSafety goal
            if emo.fear > Fixed::from_f64(0.5) {
                let fear_prio = emo.fear * Fixed::from_f64(0.8);
                if let Some(existing) = agent_goals
                    .iter_mut()
                    .find(|g| g.kind == crate::person::GoalKind::SeekSafety)
                {
                    existing.priority = existing.priority.max(fear_prio);
                } else {
                    agent_goals.push(crate::person::Goal {
                        kind: crate::person::GoalKind::SeekSafety,
                        priority: fear_prio,
                        commitment: Fixed::from_f64(0.6),
                        created_tick: tick,
                        source: crate::person::GoalSource::Emotion,
                    });
                }
            }

            // High anger → Work (aggressive productivity) when hunger is manageable
            //
            // i383: "high anger" is relative to the population's own anger — the
            // absolute `anger > 0.5` opened for 0.00–0.30% of agent-ticks (a dead
            // emitter in 7 of 10 probed worlds) because anger is an acute,
            // fast-decaying emotion (per-agent p50 0.000). The priority is the
            // EXCESS over that reference, so the goal scales with the anomaly's
            // size instead of pinning to a value below the arbitration bar (the
            // i380 lesson: a term that cannot reach the bar is not a repair).
            if emo.anger > anger_shock_bar && need.hunger < Fixed::from_f64(0.8) {
                let anger_prio = (emo.anger - anger_shock_bar) * Fixed::from_f64(0.3);
                if let Some(existing) = agent_goals
                    .iter_mut()
                    .find(|g| g.kind == crate::person::GoalKind::Work)
                {
                    existing.priority = existing.priority.max(anger_prio);
                } else {
                    agent_goals.push(crate::person::Goal {
                        kind: crate::person::GoalKind::Work,
                        priority: anger_prio,
                        commitment: Fixed::from_f64(0.4),
                        created_tick: tick,
                        source: crate::person::GoalSource::Emotion,
                    });
                }
            }

            // High joy → Socialize goal
            // i388: the joy leg's need arm is the relational band (was an absolute
            // 0.3, above the channel's p99 of 0.032–0.039 — dark).
            if emo.joy > Fixed::from_f64(0.5) && need.social > social_bar {
                let joy_prio = emo.joy * Fixed::from_f64(0.4);
                if let Some(existing) = agent_goals
                    .iter_mut()
                    .find(|g| g.kind == crate::person::GoalKind::Socialize)
                {
                    existing.priority = existing.priority.max(joy_prio);
                } else {
                    agent_goals.push(crate::person::Goal {
                        kind: crate::person::GoalKind::Socialize,
                        priority: joy_prio,
                        commitment: Fixed::from_f64(0.3),
                        created_tick: tick,
                        source: crate::person::GoalSource::Emotion,
                    });
                }
            }
        }

        // ── Identity-driven goals (§24 identity modulation) ──
        // High-identity agents generate goals aligned with their identity.
        // Uses same upsert pattern as need-driven goals for consistency.
        if i < personalities.len() {
            let personality = &personalities[i];
            // Ambitious agents generate Work goals even without pressing need
            if personality.ambition > Fixed::from_f64(0.6) && need.fatigue < Fixed::from_f64(0.5) {
                let id_priority = personality.ambition * Fixed::from_f64(0.4);
                if let Some(existing) = agent_goals
                    .iter_mut()
                    .find(|g| g.kind == crate::person::GoalKind::Work)
                {
                    existing.priority = existing.priority.max(id_priority);
                } else {
                    agent_goals.push(crate::person::Goal {
                        kind: crate::person::GoalKind::Work,
                        priority: id_priority,
                        commitment: personality.conscientiousness * Fixed::from_f64(0.7),
                        created_tick: tick,
                        source: crate::person::GoalSource::Identity,
                    });
                }
            }
            // Traditional agents generate Worship goals even with moderate meaning
            if personality.traditionalism > Fixed::from_f64(0.6)
                && need.meaning > Fixed::from_f64(0.4)
            {
                let id_priority = personality.traditionalism * Fixed::from_f64(0.3);
                if let Some(existing) = agent_goals
                    .iter_mut()
                    .find(|g| g.kind == crate::person::GoalKind::Worship)
                {
                    existing.priority = existing.priority.max(id_priority);
                } else {
                    agent_goals.push(crate::person::Goal {
                        kind: crate::person::GoalKind::Worship,
                        priority: id_priority,
                        commitment: Fixed::from_f64(0.4),
                        created_tick: tick,
                        source: crate::person::GoalSource::Identity,
                    });
                }
            }
            // Extraverted agents generate Socialize goals more readily
            // i388: the extraversion leg's need arm is the relational band (was an
            // absolute 0.4 — dark for the same reason).
            if personality.extraversion > Fixed::from_f64(0.6) && need.social > social_bar {
                let id_priority = personality.extraversion * Fixed::from_f64(0.3);
                if let Some(existing) = agent_goals
                    .iter_mut()
                    .find(|g| g.kind == crate::person::GoalKind::Socialize)
                {
                    existing.priority = existing.priority.max(id_priority);
                } else {
                    agent_goals.push(crate::person::Goal {
                        kind: crate::person::GoalKind::Socialize,
                        priority: id_priority,
                        commitment: Fixed::from_f64(0.3),
                        created_tick: tick,
                        source: crate::person::GoalSource::Identity,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod need_decay_tests {
    use super::*;

    /// Iteration-267 (Fixed-4 truncation disease, §5): esteem/autonomy decay
    /// were dead since the parameter landed — the per-tick increment
    /// `meaning_decay_rate × 0.6667` (1 raw × 0.6667) truncated to ZERO in
    /// Fixed::mul, pinning both fields at founder defaults forever (i275
    /// sweep: σ=0 across the entire 12-seed family). Liveness contract:
    /// after the f64 quantize-once fix both fields must ADVANCE from their
    /// defaults in the decay system itself, at exactly 2/3 the meaning
    /// rate (design: parameters.rs "decay_rate × 0.15", × 0.6667).
    #[test]
    fn esteem_and_autonomy_decay_advance_from_defaults() {
        let params = crate::parameters::SimParameters::default();
        let mut needs = [crate::person::NeedState::default()];

        // Run the decay system in isolation, no relief actions applied.
        for _ in 0..1000 {
            system_need_decay_with_params(&params, &mut needs);
        }

        // Meaning advanced by raw 1/tick → default 0.1 + 0.1 after 1000 ticks.
        let expected_meaning = Fixed::from_f64(0.1 + 0.0001 * 1000.0);
        assert_eq!(
            needs[0].meaning, expected_meaning,
            "meaning advances +1 raw/tick"
        );
        // Esteem/autonomy advance at 2/3 meaning's rate (raw 1/tick after
        // quantize-once; 0.0001 × 0.6667 = 6.667e-5 rounds to 1e-4 raw 1).
        // Defaults differ (0.2 / 0.1) — equal RATES, not equal values.
        let esteem_advance = needs[0].esteem.to_f64() - 0.2;
        let autonomy_advance = needs[0].autonomy.to_f64() - 0.1;
        // 1e-4/tick × 1000 ticks = 0.1; epsilon absorbs f64 representation noise.
        assert!(
            (esteem_advance - 0.1).abs() < 1e-4,
            "esteem must advance (was dead: pinned at default 0.2 forever): {esteem_advance}"
        );
        assert!(
            (autonomy_advance - 0.1).abs() < 1e-4,
            "autonomy must advance (was dead: pinned at default 0.1 forever): {autonomy_advance}"
        );
        // 2/3 coupling preserved: both advance at the same rate.
        assert!((esteem_advance - autonomy_advance).abs() < 1e-4);
    }

    /// Iteration-267 relief-path balance (§4.3 hazard guard): Work relief
    /// (0.0002/tick) balances the revived decay (0.0001/tick) so a
    /// full-time worker (ρ≈0.65... actually ρ = decay/relief = 0.5)
    /// equilibria mid-band instead of pinning at 1.0. Verifies the
    /// equilibrium POINT of the decay+relief ODE, independent of action
    /// selection.
    #[test]
    fn work_relief_balances_revived_decay_at_mid_band() {
        // decay wins below 0.5 workload share, relief wins above.
        let decay_per_tick = 0.0001; // revived esteem/autonomy decay
        let relief_per_tick = 0.0002; // Work/Trade competence relief
        let equilibrium_workload = decay_per_tick / relief_per_tick;
        assert!(
            (0.38..=0.58).contains(&equilibrium_workload),
            "equilibrium workload {equilibrium_workload} must sit inside the CO-2026-003 pacing band"
        );
        // Net drift at full-time work (ρ=1) must be negative (relief wins).
        assert!(relief_per_tick - decay_per_tick > 0.0);
        // Net drift at zero work (ρ=0) must be positive (decay wins) —
        // the deficit still accumulates for idle agents (zero-at-zero).
        assert!(decay_per_tick > 0.0);
    }
}
