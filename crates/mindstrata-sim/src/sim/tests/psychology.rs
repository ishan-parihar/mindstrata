//! Domain-grouped unit tests (split from tests.rs; pure moves).

use super::super::snapshot_metrics::self_esteem_support;
use super::super::*;

/// §8.1: Self-esteem support is zero at baseline and signed around it.
#[test]
fn self_esteem_support_is_zero_at_baseline() {
    assert_eq!(self_esteem_support(Fixed::from_f64(0.5)), Fixed::ZERO);
    assert!(self_esteem_support(Fixed::from_f64(0.9)) > Fixed::ZERO);
    assert!(self_esteem_support(Fixed::from_f64(0.1)) < Fixed::ZERO);
}

/// §8.1: The self-model must track life events — previously constructed
/// but never updated (a dead system). After 500 ticks the focal agents'
/// self-esteem must have drifted from the 0.5 baseline as their
/// narratives (and the self-model's parallel narrative) form.
#[test]
fn self_model_tracks_life_events() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    for _ in 0..500 {
        sim.tick();
    }
    let any_drift = sim.agents.iter().any(|a| {
        let d = a.self_model.self_esteem - Fixed::from_f64(0.5);
        d > Fixed::from_f64(0.005)
            || d < -Fixed::from_f64(0.005)
            || a.self_model.narrative.contamination_script != Fixed::from_f64(0.2)
    });
    assert!(
        any_drift,
        "self_model must update from life events (was a dead system)"
    );
}

/// §8.1: Interoceptive filters must reach behavior — an anxious agent
/// (high negative_bias) feels the same body deficit as more dire, so its
/// depression risk accumulates faster than a low-bias agent's under
/// identical material conditions.
#[test]
fn interoception_filters_feed_depression_risk() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 60_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut anxious = Simulation::new(config.clone());
    anxious.populate();
    let mut calm = Simulation::new(config);
    calm.populate();
    // Identical material conditions: a real need deficit in every agent.
    for sim in [&mut anxious, &mut calm] {
        for agent in &mut sim.agents {
            agent.needs.hunger = Fixed::from_f64(0.7);
            agent.needs.thirst = Fixed::from_f64(0.7);
            agent.needs.fatigue = Fixed::from_f64(0.7);
            agent.needs.safety = Fixed::from_f64(0.7);
        }
    }
    // Only the interoceptive lens differs: anxious amplifies distress.
    for agent in &mut anxious.agents {
        agent.interoception.negative_bias = Fixed::from_f64(0.9);
    }
    for agent in &mut calm.agents {
        agent.interoception.negative_bias = Fixed::from_f64(0.0);
    }
    // Converge the derived states over many updates (no tick machinery).
    for _ in 0..500 {
        anxious.tick_derived_states_and_beliefs(0, 1);
        calm.tick_derived_states_and_beliefs(0, 1);
    }
    let anxious_risk = anxious.agents[0].derived.depression_risk;
    let calm_risk = calm.agents[0].derived.depression_risk;
    assert!(
        anxious_risk > calm_risk,
        "anxious agents must accumulate higher depression risk from the \
             same needs (got {anxious_risk} vs {calm_risk})"
    );
    // The felt deficit itself is monotone in bias (unit-level check).
    let felt_high = anxious.agents[0].interoception.felt_need_deficit(
        Fixed::from_f64(0.7),
        Fixed::from_f64(0.7),
        Fixed::from_f64(0.7),
        Fixed::from_f64(0.7),
    );
    let felt_low = calm.agents[0].interoception.felt_need_deficit(
        Fixed::from_f64(0.7),
        Fixed::from_f64(0.7),
        Fixed::from_f64(0.7),
        Fixed::from_f64(0.7),
    );
    assert!(
        felt_high > felt_low,
        "high bias must yield a higher felt deficit"
    );
}

/// §8.1: Embodied emotions must resist regulation in the live tick —
/// high-sensitivity agents retain more arousal than low-sensitivity agents
/// under identical conditions, because their emotions are felt more
/// intensely in the body and cognitive strategies bite less.
#[test]
fn emotional_body_tone_resists_regulation_in_tick() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 60_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut embodied = Simulation::new(config.clone());
    embodied.populate();
    let mut detached = Simulation::new(config);
    detached.populate();
    for agent in &mut embodied.agents {
        agent.interoception.sensitivity = Fixed::from_f64(0.9);
    }
    for agent in &mut detached.agents {
        agent.interoception.sensitivity = Fixed::from_f64(0.1);
    }
    // Iteration 234: increased to 4000 ticks so the sensitivity-
    // based arousal difference overcomes all ambient producers +
    // seasonal modulation added in Iterations 223-233.
    for _ in 0..4000 {
        embodied.tick();
        detached.tick();
    }
    let mean_arousal = |sim: &Simulation| -> Fixed {
        let n = sim.agents.len();
        let total = sim
            .agents
            .iter()
            .fold(Fixed::ZERO, |acc, a| acc + a.affect.arousal);
        total / Fixed::from_int(n as i64)
    };
    assert!(
        mean_arousal(&embodied) > mean_arousal(&detached),
        "embodied emotions must resist regulation: high-sensitivity agents \
             should retain more arousal than low-sensitivity agents"
    );
}

/// §8.1.3: The chapter gate fires only when the integrated-life-event
/// count crosses a 100-event boundary — never on sub-chapter progress.
#[test]
fn life_chapter_crossed_fires_only_on_centenary_boundaries() {
    assert!(life_chapter_crossed(99, 100));
    assert!(life_chapter_crossed(199, 201));
    assert!(!life_chapter_crossed(0, 99));
    assert!(!life_chapter_crossed(100, 100));
    assert!(!life_chapter_crossed(100, 149));
    assert!(!life_chapter_crossed(250, 259));
}

/// §8.1.3: The milestone gate fires only when practice crosses a 0.1
/// proficiency boundary — never on sub-step progress or on the cap plateau.
#[test]
fn skill_milestone_crossed_fires_only_on_tenth_boundaries() {
    assert!(skill_milestone_crossed(
        Fixed::from_f64(0.099),
        Fixed::from_f64(0.100)
    ));
    assert!(skill_milestone_crossed(
        Fixed::from_f64(0.050),
        Fixed::from_f64(0.100)
    ));
    assert!(!skill_milestone_crossed(
        Fixed::from_f64(0.100),
        Fixed::from_f64(0.149)
    ));
    assert!(!skill_milestone_crossed(
        Fixed::from_f64(0.250),
        Fixed::from_f64(0.259)
    ));
    assert!(!skill_milestone_crossed(
        Fixed::from_f64(0.000),
        Fixed::from_f64(0.099)
    ));
    assert!(!skill_milestone_crossed(
        Fixed::from_f64(1.0),
        Fixed::from_f64(1.0)
    ));
}
