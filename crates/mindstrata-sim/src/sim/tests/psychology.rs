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
    let conflict_count = |sim: &Simulation| -> usize {
        sim.recent_events(usize::MAX)
            .iter()
            .filter(|ev| {
                matches!(
                    ev,
                    mindstrata_core::event::SimEvent::ConflictOccurred { .. }
                )
            })
            .count()
    };
    // i275 trail (§4.2/§4.4 — full mechanism record):
    // 1. Under the v1 mono-subtle projection the tension channel was dead
    //    (probe: 1,292 claims, zero ActiveTension) and this pin read only
    //    the direct physiological channel.
    // 2. With severity-grounded Threat claims but reconciliation still dead
    //    (is_active_tension/reconcile_subtle domain gate mismatch), the
    //    tension pool grew unboundedly → the action bias inverted this pin
    //    (measured: arousal 0.204 vs 0.231 INVERTED, conflicts 117 vs 141).
    // 3. With the reconciliation loop CLOSED (i275 final), the pool drains
    //    into Integrated syntheses and the original equilibrium is RESTORED:
    //    measured (seed 42, 4000 ticks) arousal 0.206 vs 0.196 — the direct
    //    physiological channel dominates again. The original pin stands, now
    //    with the polarity graph live and bounded rather than dead.
    let (embodied_conflicts, detached_conflicts) =
        (conflict_count(&embodied), conflict_count(&detached));
    assert_eq!(
        embodied_conflicts, detached_conflicts,
        "with the polarity loop closed, identical world dynamics must produce \
             identical conflict exposure (the bias is bounded, not chaotic)"
    );
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

// ── i305: difficulty-lever row 2, threshold half (goal gates) ─────────────

/// Standard must resolve the canon fulfillment thresholds bit-for-bit — the
/// zero-blast contract for the threshold half (the golden replay proves it
/// end-to-end; this names the mechanism).
#[test]
fn goal_gates_in_the_standard_band_are_the_canon_thresholds() {
    use crate::systems::GoalGates;
    let gates = GoalGates::for_params(&crate::parameters::SimParameters::default());
    assert_eq!(gates, GoalGates::CANON);
    assert_eq!(gates.eat.to_raw(), 5_000);
    assert_eq!(gates.drink.to_raw(), 5_000);
    assert_eq!(gates.rest.to_raw(), 6_000);
    assert_eq!(gates.socialize.to_raw(), 7_000);
    assert_eq!(gates.worship.to_raw(), 7_000);
    assert_eq!(gates.retain.to_raw(), 3_000);
}

/// The band multiplier reaches every gate, in the coherent direction: a
/// Lenient village responds to smaller deficits (lower gates), a Harsh one
/// tolerates more (higher gates). Fixed-4 is exact for 0.6/1.4, so no gate
/// lands on a rounding edge.
#[test]
fn goal_gates_scale_with_the_difficulty_band() {
    use crate::parameters::{DifficultyProfile, SimParameters};
    use crate::systems::GoalGates;
    let lenient =
        GoalGates::for_params(&SimParameters::with_difficulty(DifficultyProfile::Lenient));
    let harsh = GoalGates::for_params(&SimParameters::with_difficulty(DifficultyProfile::Harsh));
    assert_eq!(lenient.eat.to_raw(), 3_000);
    assert_eq!(lenient.socialize.to_raw(), 4_200);
    assert_eq!(lenient.retain.to_raw(), 1_800);
    assert_eq!(harsh.eat.to_raw(), 7_000);
    assert_eq!(harsh.socialize.to_raw(), 9_800);
    assert_eq!(harsh.retain.to_raw(), 4_200);
    for (l, h) in [
        (lenient.eat, harsh.eat),
        (lenient.drink, harsh.drink),
        (lenient.rest, harsh.rest),
        (lenient.socialize, harsh.socialize),
        (lenient.worship, harsh.worship),
        (lenient.retain, harsh.retain),
    ] {
        assert!(l < h, "lenient gates must sit below harsh gates");
    }
}

/// Behavioural pin (no RNG, no seed dependence): the band actually changes
/// which goals exist. An agent carrying a hunger deficit of 0.55 sits between
/// the canon gate (0.5) and the Harsh gate (0.7), so the same state generates
/// an `Eat` goal under Lenient/Standard and none under Harsh.
#[test]
fn harsh_gate_scale_withholds_a_goal_the_standard_band_generates() {
    use crate::parameters::{DifficultyProfile, SimParameters};
    use crate::person::GoalKind;
    use crate::systems::{system_goal_generation, SystemContext};

    let generated = |profile: DifficultyProfile| -> bool {
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 100,
            world_width: 16,
            world_height: 16,
            num_agents: 6,
            snapshot_interval: None,
        });
        sim.populate();
        for a in &mut sim.agents {
            a.needs.hunger = Fixed::from_f64(0.55);
        }
        let personalities: Vec<_> = sim.agents.iter().map(|a| a.personality.clone()).collect();
        let needs: Vec<_> = sim.agents.iter().map(|a| a.needs.clone()).collect();
        let emotions: Vec<_> = sim.agents.iter().map(|a| a.emotions.clone()).collect();
        let mut goals: Vec<Vec<crate::person::Goal>> = vec![Vec::new(); sim.agents.len()];
        let params = SimParameters::with_difficulty(profile);
        let mut events = Vec::new();
        let mut ctx = SystemContext {
            tick: 1,
            rng: &mut sim.rng,
            world: &mut sim.world,
            events: &mut events,
        };
        system_goal_generation(
            &mut ctx,
            &personalities,
            &needs,
            &mut goals,
            &emotions,
            &params,
        );
        goals[0].iter().any(|g| g.kind == GoalKind::Eat)
    };

    assert!(
        generated(DifficultyProfile::Lenient),
        "a hunger deficit of 0.55 must generate an Eat goal in the Lenient band"
    );
    assert!(
        generated(DifficultyProfile::Standard),
        "a hunger deficit of 0.55 must generate an Eat goal in the Standard band"
    );
    assert!(
        !generated(DifficultyProfile::Harsh),
        "the Brittle/1.4x band must tolerate 0.55 hunger before acting"
    );
}

/// i306 regression pin: the MEANING reflex. An agent whose meaning need is
/// saturated performs Worship on the next selection even when a strong daily
/// routine (Work, strength 0.70 > the 0.5 follow-threshold) would otherwise
/// win — the measured deadlock that pinned 27% of agent-ticks at the meaning
/// ceiling (`i306_meaning_channel`). Vital signs are set clear of every
/// physiological reflex so the ordering under test is the meaning one.
#[test]
fn meaning_reflex_forces_worship_over_a_strong_routine() {
    use crate::actions::ActionKind;
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: 100,
        world_width: 16,
        world_height: 16,
        num_agents: 6,
        snapshot_interval: None,
    });
    sim.populate();
    let a = &mut sim.agents[0];
    a.needs.meaning = Fixed::from_f64(0.95);
    a.needs.hunger = Fixed::from_f64(0.05);
    a.needs.thirst = Fixed::from_f64(0.05);
    a.needs.fatigue = Fixed::from_f64(0.05);
    a.body.health = Fixed::ONE;
    a.current_action = ActionKind::Work;
    a.action_progress = 0; // force a fresh selection this tick
    sim.tick();
    assert_eq!(
        sim.agents[0].current_action,
        ActionKind::Worship,
        "a saturated meaning need must outrank the routine"
    );
    // And the same agent below the reflex threshold keeps its routine.
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: 100,
        world_width: 16,
        world_height: 16,
        num_agents: 6,
        snapshot_interval: None,
    });
    sim.populate();
    let a = &mut sim.agents[0];
    a.needs.meaning = Fixed::from_f64(0.85);
    a.needs.hunger = Fixed::from_f64(0.05);
    a.needs.thirst = Fixed::from_f64(0.05);
    a.needs.fatigue = Fixed::from_f64(0.05);
    a.body.health = Fixed::ONE;
    a.action_progress = 0;
    sim.tick();
    assert_ne!(
        sim.agents[0].current_action,
        ActionKind::Worship,
        "below the reflex threshold the routine decision stands"
    );
}

/// i306 diagnostic (kept as a regression pin): does the Worship action's
/// meaning relief actually land through the real tick pipeline?
#[test]
fn worship_action_relieves_meaning_through_the_pipeline() {
    use crate::actions::ActionKind;
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: 100,
        world_width: 16,
        world_height: 16,
        num_agents: 6,
        snapshot_interval: None,
    });
    sim.populate();
    sim.agents[0].needs.meaning = Fixed::from_f64(0.8);
    sim.agents[0].current_action = ActionKind::Worship;
    sim.agents[0].action_progress = 4;
    let before = sim.agents[0].needs.meaning.to_f64();
    sim.tick();
    let after = sim.agents[0].needs.meaning.to_f64();
    println!(
        "i306 diag: meaning {before} -> {after} (action {:?})",
        sim.agents[0].current_action
    );
    assert!(
        after < before - 0.05,
        "Worship must relieve meaning through the pipeline: {before} -> {after}"
    );
}
