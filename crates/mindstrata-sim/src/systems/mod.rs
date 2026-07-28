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

// ── Need decay system ────────────────────────────────────────────────────

/// Decay needs each tick: deficits grow over time.
pub fn system_need_decay(
    _ctx: &mut SystemContext,
    _bodies: &[BodyState],
    needs: &mut [NeedState],
) {
    let decay_rate = Fixed::from_f64(0.001); // per-tick deficit growth

    for need in needs.iter_mut() {
        need.hunger = (need.hunger + decay_rate).clamp_01();
        need.thirst = (need.thirst + decay_rate * Fixed::from_int(2)).clamp_01();
        need.fatigue = (need.fatigue + decay_rate * Fixed::from_f64(0.5)).clamp_01();
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

/// Generate goals based on need pressure.
pub fn system_goal_generation(
    _ctx: &mut SystemContext,
    _personalities: &[crate::person::Personality],
    needs: &[NeedState],
    goals: &mut [Vec<crate::person::Goal>],
) {
    for (need, agent_goals) in needs.iter().zip(goals.iter_mut()) {
        // Hunger generates eat goal
        if need.hunger > Fixed::from_f64(0.5)
            && !agent_goals
                .iter()
                .any(|g| g.kind == crate::person::GoalKind::Eat)
        {
            agent_goals.push(crate::person::Goal {
                kind: crate::person::GoalKind::Eat,
                priority: need.hunger,
                commitment: Fixed::from_f64(0.5),
                created_tick: 0,
            });
        }
        // Thirst generates drink goal
        if need.thirst > Fixed::from_f64(0.5)
            && !agent_goals
                .iter()
                .any(|g| g.kind == crate::person::GoalKind::Drink)
        {
            agent_goals.push(crate::person::Goal {
                kind: crate::person::GoalKind::Drink,
                priority: need.thirst,
                commitment: Fixed::from_f64(0.5),
                created_tick: 0,
            });
        }
        // Fatigue generates rest goal
        if need.fatigue > Fixed::from_f64(0.6)
            && !agent_goals
                .iter()
                .any(|g| g.kind == crate::person::GoalKind::Rest)
        {
            agent_goals.push(crate::person::Goal {
                kind: crate::person::GoalKind::Rest,
                priority: need.fatigue,
                commitment: Fixed::from_f64(0.4),
                created_tick: 0,
            });
        }

        // Social generates socialize goal
        if need.social > Fixed::from_f64(0.7)
            && !agent_goals
                .iter()
                .any(|g| g.kind == crate::person::GoalKind::Socialize)
        {
            agent_goals.push(crate::person::Goal {
                kind: crate::person::GoalKind::Socialize,
                priority: need.social,
                commitment: Fixed::from_f64(0.3),
                created_tick: 0,
            });
        }

        // Meaning generates worship goal
        if need.meaning > Fixed::from_f64(0.7)
            && !agent_goals
                .iter()
                .any(|g| g.kind == crate::person::GoalKind::Worship)
        {
            agent_goals.push(crate::person::Goal {
                kind: crate::person::GoalKind::Worship,
                priority: need.meaning,
                commitment: Fixed::from_f64(0.3),
                created_tick: 0,
            });
        }

        // ── Goal completion: remove goals whose need has dropped below threshold ──
        agent_goals.retain(|g| {
            let threshold = Fixed::from_f64(0.3);
            match g.kind {
                crate::person::GoalKind::Eat => need.hunger > threshold,
                crate::person::GoalKind::Drink => need.thirst > threshold,
                crate::person::GoalKind::Rest => need.fatigue > threshold,
                crate::person::GoalKind::Work => true, // work goals don't auto-complete
                crate::person::GoalKind::Socialize => need.social > threshold,
                crate::person::GoalKind::Worship => need.meaning > threshold,
                _ => true,
            }
        });
    }
}
