//! §39.2 Property tests — verify invariants that must hold across all seeds.
//!
//! Uses property-based testing to check:
//! - Resources are never negative
//! - Belief confidence remains within [0, 1]
//! - Relationships decay without interaction
//! - Deterministic seed produces same output

use mindstrata_sim::{Simulation, sim::SimConfig};

/// Helper: run a simulation with given seed and return metrics.
fn run_sim(seed: u64, ticks: u64) -> mindstrata_sim::sim::MetricsSnapshot {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 8,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(ticks);
    sim.metrics_snapshot()
}

#[test]
fn resources_never_negative_across_seeds() {
    for seed in 0..20u64 {
        let config = SimConfig {
            seed,
            max_ticks: 500,
            world_width: 16,
            world_height: 16,
            num_agents: 8,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();

        for tick in 0..500 {
            sim.tick();
            let grain = sim.total_grain().to_f64();
            let water = sim.total_water().to_f64();
            assert!(grain >= 0.0,
                "Seed {seed}, tick {tick}: grain={grain} is negative");
            assert!(water >= 0.0,
                "Seed {seed}, tick {tick}: water={water} is negative");
        }
    }
}

#[test]
fn belief_confidence_bounded_across_seeds() {
    for seed in 0..15u64 {
        let config = SimConfig {
            seed,
            max_ticks: 500,
            world_width: 16,
            world_height: 16,
            num_agents: 8,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(500);

        for (i, agent) in sim.agents.iter().enumerate() {
            for (j, belief) in agent.beliefs.iter().enumerate() {
                let conf = belief.confidence.to_f64();
                assert!((0.0..=1.0).contains(&conf),
                    "Seed {seed}, agent {i}, belief {j}: confidence={conf} out of [0,1]");
                let ec = belief.emotional_charge.to_f64();
                assert!((-1.0..=1.0).contains(&ec),
                    "Seed {seed}, agent {i}, belief {j}: emotional_charge={ec} out of [-1,1]");
            }
        }
    }
}

#[test]
fn agent_health_never_exceeds_one() {
    for seed in 0..10u64 {
        let config = SimConfig {
            seed,
            max_ticks: 300,
            world_width: 16,
            world_height: 16,
            num_agents: 8,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(300);

        for (i, agent) in sim.agents.iter().enumerate() {
            let health = agent.body.health.to_f64();
            assert!((0.0..=1.0).contains(&health),
                "Seed {seed}, agent {i}: health={health} out of [0,1]");
            let energy = agent.body.energy.to_f64();
            assert!((0.0..=1.0).contains(&energy),
                "Seed {seed}, agent {i}: energy={energy} out of [0,1]");
        }
    }
}

#[test]
fn relationship_trust_bounded() {
    for seed in 0..10u64 {
        let config = SimConfig {
            seed,
            max_ticks: 500,
            world_width: 16,
            world_height: 16,
            num_agents: 8,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(500);

        for (i, rel) in sim.relationships.iter().enumerate() {
            let trust = rel.trust.to_f64();
            assert!((0.0..=1.0).contains(&trust),
                "Seed {seed}, rel {i}: trust={trust} out of [0,1]");
            let affection = rel.affection.to_f64();
            assert!((0.0..=1.0).contains(&affection),
                "Seed {seed}, rel {i}: affection={affection} out of [0,1]");
        }
    }
}

#[test]
fn emotions_bounded_across_seeds() {
    for seed in 0..10u64 {
        let config = SimConfig {
            seed,
            max_ticks: 500,
            world_width: 16,
            world_height: 16,
            num_agents: 8,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(500);

        for (i, agent) in sim.agents.iter().enumerate() {
            let emotions = [
                ("fear", agent.emotions.fear),
                ("anger", agent.emotions.anger),
                ("joy", agent.emotions.joy),
                ("sadness", agent.emotions.sadness),
                ("trust", agent.emotions.trust),
                ("shame", agent.emotions.shame),
                ("pride", agent.emotions.pride),
                ("guilt", agent.emotions.guilt),
            ];
            for (name, val) in &emotions {
                let v = val.to_f64();
                assert!((0.0..=1.0).contains(&v),
                    "Seed {seed}, agent {i}: {name}={v} out of [0,1]");
            }
        }
    }
}

#[test]
fn deterministic_replay_across_seeds() {
    for seed in [0, 7, 42, 99, 12345] {
        let m1 = run_sim(seed, 200);
        let m2 = run_sim(seed, 200);

        assert_eq!(m1.avg_hunger, m2.avg_hunger,
            "Seed {seed}: hunger determinism failed");
        assert_eq!(m1.avg_thirst, m2.avg_thirst,
            "Seed {seed}: thirst determinism failed");
        assert_eq!(m1.avg_fatigue, m2.avg_fatigue,
            "Seed {seed}: fatigue determinism failed");
        assert_eq!(m1.total_grain, m2.total_grain,
            "Seed {seed}: grain determinism failed");
        assert_eq!(m1.total_water, m2.total_water,
            "Seed {seed}: water determinism failed");
        assert_eq!(m1.event_count, m2.event_count,
            "Seed {seed}: event count determinism failed");
        assert_eq!(m1.agent_count, m2.agent_count,
            "Seed {seed}: agent count determinism failed");
    }
}

// ── §18.2: Additional property tests ──────────────────────────

#[test]
fn agent_age_never_negative() {
    for seed in 0..10u64 {
        let config = SimConfig {
            seed,
            max_ticks: 1000,
            world_width: 16,
            world_height: 16,
            num_agents: 8,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(1000);

        for (i, agent) in sim.agents.iter().enumerate() {
            let age = agent.age.to_f64();
            assert!(age >= 0.0,
                "Seed {seed}, agent {i}: age={age} is negative");
        }
    }
}

#[test]
fn fertility_non_negative() {
    for seed in 0..10u64 {
        let config = SimConfig {
            seed,
            max_ticks: 500,
            world_width: 16,
            world_height: 16,
            num_agents: 8,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(500);

        for (i, agent) in sim.agents.iter().enumerate() {
            if let Some(fertility) = agent.body.fertility {
                let f = fertility.to_f64();
                assert!(f >= 0.0 && f <= 1.0,
                    "Seed {seed}, agent {i}: fertility={f} out of [0,1]");
            }
        }
    }
}

#[test]
fn agent_ages_monotonically_non_decreasing() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 500,
        world_width: 16,
        world_height: 16,
        num_agents: 8,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();

    let mut prev_ages: Vec<f64> = sim.agents.iter().map(|a| a.age.to_f64()).collect();
    sim.run(500);

    // After running, all original agents should have age >= their initial age
    // (new agents may have been born with lower age, so we can't check all)
    let n_initial = prev_ages.len();
    for (i, agent) in sim.agents.iter().take(n_initial).enumerate() {
        assert!(agent.age.to_f64() >= prev_ages[i] - 0.01,
            "Agent {i}: age went from {} to {}", prev_ages[i], agent.age.to_f64());
    }
}

#[test]
fn stress_increases_heuristic_bias() {
    // §22.1: sleep debt increases heuristic bias — verified at unit level
    use mindstrata_core::fixed::Fixed;
    use mindstrata_sim::person::CognitiveState;

    let mut calm = CognitiveState::default();
    calm.update(Fixed::from_f64(0.1), Fixed::from_f64(0.1));
    let calm_bias = calm.heuristic_bias;

    let mut stressed = CognitiveState::default();
    stressed.update(Fixed::from_f64(0.9), Fixed::from_f64(0.8));
    let stressed_bias = stressed.heuristic_bias;

    assert!(stressed_bias > calm_bias,
        "Stress should increase heuristic bias: calm={calm_bias}, stressed={stressed_bias}");
}
