//! Command-channel unit tests.

use super::{command_goal_action, SimConfig, Simulation};
use crate::actions::ActionKind;
use crate::person::{Goal, GoalKind, GoalSource, NeedState};
use mindstrata_core::fixed::Fixed;

fn build_sim(n: u32) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed: 7,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim
}

#[test]
fn command_agent_injects_high_priority_command_goal() {
    let mut sim = build_sim(6);
    assert!(sim.command_agent(0, GoalKind::Work));
    let goal = sim.agents[0]
        .goals
        .iter()
        .find(|g| g.kind == GoalKind::Work)
        .expect("injected goal present");
    assert_eq!(goal.priority, Fixed::ONE);
    assert_eq!(goal.commitment, Fixed::ONE);
    assert_eq!(goal.source, GoalSource::Command);
    assert_eq!(goal.created_tick, 0);
}

#[test]
fn command_agent_rejects_out_of_range_index() {
    let mut sim = build_sim(6);
    assert!(!sim.command_agent(6, GoalKind::Work));
    assert!(!sim.command_agent(usize::MAX, GoalKind::Work));
    assert!(sim.agents[0].goals.is_empty());
}

#[test]
fn commanded_directive_steers_and_is_consumed_on_first_selection() {
    let mut sim = build_sim(6);
    assert!(sim.command_agent(2, GoalKind::Work));
    assert!(sim.agents[2]
        .goals
        .iter()
        .any(|g| g.source == GoalSource::Command));
    // Fresh worlds select on the first tick; the directive fires Work and
    // is consumed in the same selection — a one-shot nudge.
    sim.tick();
    assert_eq!(
        sim.agents[2].current_action,
        ActionKind::Work,
        "the directive steered the selection"
    );
    assert!(
        !sim.agents[2]
            .goals
            .iter()
            .any(|g| g.source == GoalSource::Command),
        "the directive was consumed by the selection it steered"
    );
}

#[test]
fn command_goal_action_returns_aligned_action_for_directives() {
    let work = vec![Goal {
        kind: GoalKind::Work,
        priority: Fixed::ONE,
        commitment: Fixed::ONE,
        created_tick: 0,
        source: GoalSource::Command,
    }];
    let calm = NeedState::default();
    assert_eq!(
        command_goal_action(&work, &calm),
        Some((ActionKind::Work, GoalKind::Work))
    );

    let worship = vec![Goal {
        kind: GoalKind::Worship,
        priority: Fixed::ONE,
        commitment: Fixed::ONE,
        created_tick: 0,
        source: GoalSource::Command,
    }];
    assert_eq!(
        command_goal_action(&worship, &calm),
        Some((ActionKind::Worship, GoalKind::Worship))
    );
}

#[test]
fn command_goal_action_yields_to_critical_needs_of_other_kinds() {
    let work = vec![Goal {
        kind: GoalKind::Work,
        priority: Fixed::ONE,
        commitment: Fixed::ONE,
        created_tick: 0,
        source: GoalSource::Command,
    }];
    let starving = NeedState {
        hunger: Fixed::from_f64(0.95),
        ..Default::default()
    };
    assert_eq!(
        command_goal_action(&work, &starving),
        None,
        "Work yields to critical hunger"
    );
}

#[test]
fn command_goal_action_ignores_endogenous_goals_and_seek_safety() {
    let endogenous = vec![Goal {
        kind: GoalKind::Work,
        priority: Fixed::ONE,
        commitment: Fixed::ONE,
        created_tick: 0,
        source: GoalSource::Identity,
    }];
    assert_eq!(
        command_goal_action(&endogenous, &NeedState::default()),
        None
    );

    let seek = vec![Goal {
        kind: GoalKind::SeekSafety,
        priority: Fixed::ONE,
        commitment: Fixed::ONE,
        created_tick: 0,
        source: GoalSource::Command,
    }];
    assert_eq!(command_goal_action(&seek, &NeedState::default()), None);
}

#[test]
fn command_goal_action_prioritizes_highest_priority_directive() {
    let goals = vec![
        Goal {
            kind: GoalKind::Rest,
            priority: Fixed::from_f64(0.5),
            commitment: Fixed::ONE,
            created_tick: 0,
            source: GoalSource::Command,
        },
        Goal {
            kind: GoalKind::Work,
            priority: Fixed::ONE,
            commitment: Fixed::ONE,
            created_tick: 0,
            source: GoalSource::Command,
        },
    ];
    assert_eq!(
        command_goal_action(&goals, &NeedState::default()),
        Some((ActionKind::Work, GoalKind::Work))
    );
}

#[test]
fn command_agent_upserts_repeat_directive_of_same_kind() {
    let mut sim = build_sim(6);
    assert!(sim.command_agent(1, GoalKind::Work));
    assert!(sim.command_agent(1, GoalKind::Work));
    let count = sim.agents[1]
        .goals
        .iter()
        .filter(|g| g.source == GoalSource::Command && g.kind == GoalKind::Work)
        .count();
    assert_eq!(
        count, 1,
        "repeat directives of the same kind replace, not pile up"
    );
}

#[test]
fn clear_commands_removes_all_directives() {
    let mut sim = build_sim(6);
    assert!(sim.command_agent(1, GoalKind::Work));
    assert!(sim.command_agent(1, GoalKind::Worship));
    assert_eq!(
        sim.agents[1]
            .goals
            .iter()
            .filter(|g| g.source == GoalSource::Command)
            .count(),
        2
    );
    assert!(sim.clear_commands(1));
    assert_eq!(
        sim.agents[1]
            .goals
            .iter()
            .filter(|g| g.source == GoalSource::Command)
            .count(),
        0,
        "clear removes every directive"
    );
    assert!(!sim.clear_commands(6), "out-of-range clear returns false");
}

/// i389: the settlement's own authority speaks through the SAME consumption path
/// as the operator's — `command_goal_action` accepts a `Decree` directive, and
/// `system_goal_generation` does NOT exempt it from decay (that is the whole
/// distinction: an operator's order stands until acted on, a decree fades).
#[test]
fn decree_is_a_directive_that_decays() {
    let needs = NeedState::default();
    let decree = |priority: Fixed| Goal {
        kind: GoalKind::Work,
        priority,
        commitment: priority,
        created_tick: 0,
        source: GoalSource::Decree,
    };
    assert_eq!(
        command_goal_action(&[decree(Fixed::from_f64(0.6))], &needs),
        Some((ActionKind::Work, GoalKind::Work)),
        "a decree steers selection exactly as an operator directive does"
    );
    // The behaviour at the decayed end is the point: a stale decree still steers
    // only while its priority survives the goal pass's retain arm (priority ≤ 0
    // removes it before selection), so authority is a repeated ask rather than a
    // standing order.
    let stale = decree(Fixed::ZERO);
    assert!(
        command_goal_action(&[stale], &needs).is_some(),
        "consumption is priority-agnostic; the RETAIN arm is what expires a decree"
    );
}

/// i389: the producer's conditional liveness, isolated from the office vacancy
/// the pestilence leg exposed (the Elder seat was EMPTY for the whole window
/// there — `i389_command_channel` — so the pestilence census measures a vacancy,
/// not a dark producer). Here the office is filled and the crisis is singular:
/// a frightened population draws a decree within one cadence; a calm one never
/// draws one.
#[test]
fn council_decrees_are_issued_in_crisis_and_never_in_calm() {
    use crate::systems::decree::DECREE_CADENCE_TICKS;

    let run_with_fear = |fear: f64| -> usize {
        let mut sim = build_sim(6);
        // The office must be filled for authority to exist at all; a fresh
        // village appoints its council seats at populate, and the determinism
        // here does not depend on which agent holds it.
        let council = sim
            .institutions
            .iter_mut()
            .find(|i| i.kind == crate::institutions::InstitutionKind::Council)
            .expect("council exists");
        if council.get_role_holder("Elder").is_none() {
            council.assign_role("Elder", mindstrata_core::AgentId::new(0));
        }
        let mut decrees = 0usize;
        for _ in 0..=DECREE_CADENCE_TICKS {
            for a in &mut sim.agents {
                a.emotions.fear = Fixed::from_f64(fear);
            }
            sim.run(1);
            decrees += sim
                .agents
                .iter()
                .map(|a| {
                    a.goals
                        .iter()
                        .filter(|g| g.source == GoalSource::Decree)
                        .count()
                })
                .sum::<usize>();
        }
        decrees
    };

    let crisis = run_with_fear(0.9);
    assert!(
        crisis > 0,
        "a frightened village must draw a decree from its council (got {crisis} decree agent-ticks)"
    );
    let calm = run_with_fear(0.05);
    assert_eq!(
        calm, 0,
        "authority must not speak in a calm village (got {calm} decree agent-ticks)"
    );
}
