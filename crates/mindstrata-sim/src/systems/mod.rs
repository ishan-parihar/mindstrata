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
pub(crate) mod biology;
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
        // Esteem and autonomy decay at 2/3 the meaning rate
        need.esteem =
            (need.esteem + params.meaning_decay_rate * Fixed::from_f64(0.6667)).clamp_01();
        need.autonomy =
            (need.autonomy + params.meaning_decay_rate * Fixed::from_f64(0.6667)).clamp_01();
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
