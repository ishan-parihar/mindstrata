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
                assert!((0.0..=1.0).contains(&f),
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

    let prev_ages: Vec<f64> = sim.agents.iter().map(|a| a.age.to_f64()).collect();
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

// ── §17.2 + §18: Budget gating property tests ──────────────────

use mindstrata_sim::agent_tier::{AgentTier, AgentTierState, CognitiveBudget, CognitiveBudgetTracker};

/// §17.2: Budget tracker respects tier limits — focal agents get higher
/// budgets than secondary, which get higher than background.
#[test]
fn budget_limits_decrease_with_tier() {
    let focal = CognitiveBudget::focal();
    let secondary = CognitiveBudget::secondary();
    let background = CognitiveBudget::background();

    // Appraisals: focal > secondary > background
    assert!(focal.max_appraisals > secondary.max_appraisals);
    assert!(secondary.max_appraisals > background.max_appraisals);

    // Memory operations: focal > secondary > background
    assert!(focal.max_memory_operations > secondary.max_memory_operations);
    assert!(secondary.max_memory_operations > background.max_memory_operations);

    // Prospections: focal > secondary = background = 0
    assert!(focal.max_prospections > secondary.max_prospections);
    assert_eq!(secondary.max_prospections, 0);
    assert_eq!(background.max_prospections, 0);

    // Social inferences: focal > secondary > background = 0
    assert!(focal.max_social_inferences > secondary.max_social_inferences);
    assert!(secondary.max_social_inferences > background.max_social_inferences);
    assert_eq!(background.max_social_inferences, 0);
}

/// §17.2: Budget exhaustion prevents further operations.
#[test]
fn budget_exhaustion_stops_operations() {
    let budget = CognitiveBudget::secondary();
    let mut tracker = CognitiveBudgetTracker::from_budget(&budget);

    // Exhaust appraisals
    for _ in 0..budget.max_appraisals {
        assert!(tracker.consume_appraisal(), "should succeed before exhaustion");
    }
    assert!(!tracker.can_appraise(), "should be exhausted");
    assert!(!tracker.consume_appraisal(), "should fail after exhaustion");
}

/// §17.2: Budget resets each tick — exhausted budget becomes full again.
#[test]
fn budget_resets_each_tick() {
    let mut state = AgentTierState::new(AgentTier::Focal, 0);

    // Exhaust all budget categories
    for _ in 0..20 {
        let _ = state.budget_tracker.consume_appraisal();
    }
    for _ in 0..10 {
        let _ = state.budget_tracker.consume_memory_op();
    }
    for _ in 0..5 {
        let _ = state.budget_tracker.consume_prospection();
    }
    for _ in 0..10 {
        let _ = state.budget_tracker.consume_social_inference();
    }

    assert!(!state.budget_tracker.can_appraise());
    assert!(!state.budget_tracker.can_memory_op());
    assert!(!state.budget_tracker.can_prospect());
    assert!(!state.budget_tracker.can_social_infer());

    // Reset
    state.reset_tick_budget();

    assert!(state.budget_tracker.can_appraise());
    assert!(state.budget_tracker.can_memory_op());
    assert!(state.budget_tracker.can_prospect());
    assert!(state.budget_tracker.can_social_infer());
}

/// §17.1: Tier reclassification always produces a valid tier.
#[test]
fn tier_reclassification_always_valid() {
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
            let tier = &agent.agent_tier.tier;
            // Tier must be one of the three valid variants
            assert!(matches!(tier, AgentTier::Focal | AgentTier::Secondary | AgentTier::Background),
                "Seed {seed}, agent {i}: invalid tier {tier:?}");
        }
    }
}

/// §17.2: After a full simulation tick, budget tracker should have been
/// reset (not exhausted from previous tick).
#[test]
fn budget_not_stale_after_tick() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 200,
        world_width: 16,
        world_height: 16,
        num_agents: 8,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(200);

    // After running, agents should have had their budgets reset each tick.
    // The budget tracker state at the end should reflect the last reset.
    for (i, agent) in sim.agents.iter().enumerate() {
        let tier = &agent.agent_tier.tier;
        let budget = &agent.agent_tier.budget;
        let tracker = &agent.agent_tier.budget_tracker;

        // Budget limits should match the tier
        match tier {
            AgentTier::Focal => {
                assert_eq!(budget.max_appraisals, 20);
                assert_eq!(budget.max_prospections, 5);
            }
            AgentTier::Secondary => {
                assert_eq!(budget.max_appraisals, 5);
                assert_eq!(budget.max_prospections, 0);
            }
            AgentTier::Background => {
                assert_eq!(budget.max_appraisals, 1);
                assert_eq!(budget.max_prospections, 0);
            }
        }

        // Tracker should not exceed budget limits
        assert!(tracker.remaining_appraisals() <= budget.max_appraisals,
            "Seed 42, agent {i}: remaining appraisals {} exceeds max {}",
            tracker.remaining_appraisals(), budget.max_appraisals);
    }
}

/// §17.2: Background agents always have zero prospection budget.
#[test]
fn background_agents_never_prospect() {
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
            if matches!(agent.agent_tier.tier, AgentTier::Background) {
                assert_eq!(agent.agent_tier.budget.max_prospections, 0,
                    "Seed {seed}, agent {i}: background agent has prospection budget");
                assert_eq!(agent.agent_tier.budget.max_social_inferences, 0,
                    "Seed {seed}, agent {i}: background agent has social inference budget");
            }
        }
    }
}

/// §17.2: Narrative importance stays within [0, 1] across simulation.
#[test]
fn narrative_importance_bounded() {
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
            let ni = agent.agent_tier.narrative_importance.to_f64();
            assert!((0.0..=1.0).contains(&ni),
                "Seed {seed}, agent {i}: narrative_importance={ni} out of [0,1]");
        }
    }
}

// ── Structural invariant tests (Iteration 139) ───────────────────────
// These tests extend the property-test suite beyond simple boundedness
// into deeper structural invariants that the sim must never violate.

#[test]
fn norm_strength_bounded_across_seeds() {
    // Internalized norm strength must stay within [0, 1].
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
            for (j, norm) in agent.moral_cognition.internalized_norms.iter().enumerate() {
                let s = norm.strength.to_f64();
                assert!((0.0..=1.0).contains(&s),
                    "Seed {seed}, agent {i}, norm {j}: strength={s} out of [0,1]");
            }
        }
    }
}

#[test]
fn attachment_dimensions_bounded_across_seeds() {
    // Attachment security, anxiety, and avoidance must each stay in [0, 1].
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
            let sec = agent.attachment.security.to_f64();
            let anx = agent.attachment.anxiety.to_f64();
            let avo = agent.attachment.avoidance.to_f64();
            assert!((0.0..=1.0).contains(&sec),
                "Seed {seed}, agent {i}: attachment.security={sec} out of [0,1]");
            assert!((0.0..=1.0).contains(&anx),
                "Seed {seed}, agent {i}: attachment.anxiety={anx} out of [0,1]");
            assert!((0.0..=1.0).contains(&avo),
                "Seed {seed}, agent {i}: attachment.avoidance={avo} out of [0,1]");
        }
    }
}

#[test]
fn moral_emotions_bounded_across_seeds() {
    // All six moral emotion channels must stay in [0, 1].
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
            let me = &agent.moral_cognition.moral_emotions;
            let pairs = [
                ("outrage", me.outrage),
                ("guilt", me.guilt),
                ("contempt", me.contempt),
                ("gratitude", me.gratitude),
                ("pride", me.pride),
                ("shame", me.shame),
            ];
            for (name, val) in &pairs {
                let v = val.to_f64();
                assert!((0.0..=1.0).contains(&v),
                    "Seed {seed}, agent {i}: moral_emotions.{name}={v} out of [0,1]");
            }
        }
    }
}

#[test]
fn institution_legitimacy_bounded_across_seeds() {
    // Every institution's legitimacy must stay in [0, 1].
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

        for (j, inst) in sim.institutions.iter().enumerate() {
            let leg = inst.legitimacy.to_f64();
            assert!((0.0..=1.0).contains(&leg),
                "Seed {seed}, institution {j}: legitimacy={leg} out of [0,1]");
        }
    }
}
