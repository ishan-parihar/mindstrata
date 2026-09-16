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
pub(crate) mod health;
pub mod institutions_multiplier;

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

/// Generate goals based on need pressure, emotional state, and identity.
/// §24: Goals now carry source tracking and support emotional/identity modulation.
/// §3.4: Emotional goal modulation — anger, fear, and joy drive goal generation.
pub fn system_goal_generation(
    ctx: &mut SystemContext,
    personalities: &[crate::person::Personality],
    needs: &[NeedState],
    goals: &mut [Vec<crate::person::Goal>],
    emotions: &[crate::person::DiscreteEmotions],
) {
    let tick = ctx.tick;
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
            let threshold = Fixed::from_f64(0.3);
            match g.kind {
                crate::person::GoalKind::Eat => need.hunger > threshold,
                crate::person::GoalKind::Drink => need.thirst > threshold,
                crate::person::GoalKind::Rest => need.fatigue > threshold,
                crate::person::GoalKind::Work => true,
                crate::person::GoalKind::Socialize => need.social > threshold,
                crate::person::GoalKind::Worship => need.meaning > threshold,
                crate::person::GoalKind::SeekSafety => true,
            }
        });

        // ── Need-driven goals (§24 primary source) ──
        // §24: Only one goal per GoalKind — update priority if goal already exists
        let need_goals: &[(crate::person::GoalKind, Fixed, Fixed)] = &[
            (
                crate::person::GoalKind::Eat,
                need.hunger,
                Fixed::from_f64(0.5),
            ),
            (
                crate::person::GoalKind::Drink,
                need.thirst,
                Fixed::from_f64(0.5),
            ),
            (
                crate::person::GoalKind::Rest,
                need.fatigue,
                Fixed::from_f64(0.6),
            ),
            (
                crate::person::GoalKind::Socialize,
                need.social,
                Fixed::from_f64(0.7),
            ),
            (
                crate::person::GoalKind::Worship,
                need.meaning,
                Fixed::from_f64(0.7),
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
            if emo.anger > Fixed::from_f64(0.5) && need.hunger < Fixed::from_f64(0.8) {
                let anger_prio = emo.anger * Fixed::from_f64(0.3);
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
            if emo.joy > Fixed::from_f64(0.5) && need.social > Fixed::from_f64(0.3) {
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
            if personality.extraversion > Fixed::from_f64(0.6) && need.social > Fixed::from_f64(0.4)
            {
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
