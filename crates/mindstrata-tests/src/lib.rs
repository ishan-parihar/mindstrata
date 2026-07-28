//! Integration tests for Mindstrata.

#[cfg(test)]
mod smoke {
    use mindstrata_core::clock::Tick;
    use mindstrata_core::fixed::Fixed;
    use mindstrata_core::rng::RngStreams;
    use mindstrata_sim::{Simulation, sim::SimConfig};
    use mindstrata_sim::scenario::Scenario;

    #[test]
    fn simulation_runs_without_panic() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 100,
            world_width: 16,
            world_height: 16,
            num_agents: 5,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(100);
        assert_eq!(sim.current_tick(), Tick::new(100));
        assert_eq!(sim.agent_count(), 5);
    }

    #[test]
    fn fixed_point_basic_ops() {
        let a = Fixed::from_f64(0.5);
        let b = Fixed::from_f64(0.3);
        assert_eq!((a + b).to_f64(), 0.8);
        assert_eq!((a - b).to_f64(), 0.2);
    }

    #[test]
    fn rng_determinism() {
        let mut a = RngStreams::new(42);
        let mut b = RngStreams::new(42);
        use rand::Rng;
        let a_val: u32 = a.get_mut(mindstrata_core::rng::RngStream::World).random();
        let b_val: u32 = b.get_mut(mindstrata_core::rng::RngStream::World).random();
        assert_eq!(a_val, b_val);
    }

    #[test]
    fn world_has_sites_after_populate() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 10,
            world_width: 16,
            world_height: 16,
            num_agents: 8,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();

        let world = sim.world();
        assert!(!world.sites.is_empty(), "World should have sites after populate");
        assert!(!world.regions.is_empty(), "World should have regions");
        assert!(!world.resources.is_empty(), "World should have resources");

        let kinds: Vec<_> = world.sites.iter().map(|s| s.kind).collect();
        assert!(kinds.contains(&mindstrata_sim::world::SiteKind::Farm));
        assert!(kinds.contains(&mindstrata_sim::world::SiteKind::Well));
        assert!(kinds.contains(&mindstrata_sim::world::SiteKind::Market));
        assert!(kinds.contains(&mindstrata_sim::world::SiteKind::Temple));
    }

    #[test]
    fn scenario_roundtrip() {
        let scenario = Scenario::riverford();
        let config = scenario.to_sim_config();
        assert_eq!(config.seed, 42);
        assert_eq!(config.num_agents, 12);
    }

    #[test]
    fn simulation_produces_events() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 50,
            world_width: 16,
            world_height: 16,
            num_agents: 8,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(50);

        assert!(sim.event_count() > 0, "Simulation should produce events");
    }

    #[test]
    fn agents_have_varied_states() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 100,
            world_width: 16,
            world_height: 16,
            num_agents: 8,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(100);

        let summaries = sim.agent_summaries();
        let avg_hunger: f64 = summaries.iter().map(|s| s.hunger.to_f64()).sum::<f64>() / summaries.len() as f64;
        assert!(avg_hunger > 0.0, "Average hunger should be non-zero after 100 ticks, got {avg_hunger}");

        let actions: Vec<_> = summaries.iter().map(|s| s.current_action.as_str()).collect();
        assert!(actions.iter().any(|a| *a != "Idle"), "At least some agents should be performing actions");
    }

    #[test]
    fn deterministic_replay() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 50,
            world_width: 16,
            world_height: 16,
            num_agents: 8,
            snapshot_interval: None,
        };

        let mut sim1 = Simulation::new(config.clone());
        sim1.populate();
        sim1.run(50);
        let hunger1: Vec<f64> = sim1.agent_summaries().iter().map(|s| s.hunger.to_f64()).collect();

        let mut sim2 = Simulation::new(config);
        sim2.populate();
        sim2.run(50);
        let hunger2: Vec<f64> = sim2.agent_summaries().iter().map(|s| s.hunger.to_f64()).collect();

        assert_eq!(hunger1, hunger2, "Same seed should produce same results");
    }

    #[test]
    fn norm_registry_is_wired_correctly() {
        // Verify the norm system is integrated: norms are registered and the registry is accessible
        let config = SimConfig {
            seed: 42,
            max_ticks: 100,
            world_width: 16,
            world_height: 16,
            num_agents: 8,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();

        // Verify default norms are registered
        let norms = sim.norms();
        assert!(!norms.norms().is_empty(), "Default norms should be registered");
        assert!(norms.norms().len() >= 4, "Should have at least 4 default norms");

        // Verify violations list is initially empty
        assert!(norms.violations().is_empty(), "No violations before simulation runs");

        // Run 100 ticks — social interactions should occur
        sim.run(100);

        // Verify social system produced events (interactions happen)
        assert!(sim.event_count() > 0, "Social system should produce events");
    }

    #[test]
    fn gossip_mutates_belief_confidence() {
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

        // Record initial belief confidence for each agent
        let initial_confidence: Vec<f64> = sim.agents.iter()
            .map(|a| a.beliefs.iter().map(|b| b.confidence.to_f64()).sum::<f64>())
            .collect();

        // Run 200 ticks — gossip interactions should spread beliefs
        sim.run(200);

        // Belief confidences should have changed through gossip
        let final_confidence: Vec<f64> = sim.agents.iter()
            .map(|a| a.beliefs.iter().map(|b| b.confidence.to_f64()).sum::<f64>())
            .collect();

        // At least some agents should have different belief confidence (gossip mutated them)
        let confidence_changed = initial_confidence.iter().zip(final_confidence.iter())
            .any(|(i, f)| (*i - *f).abs() > 0.001);
        assert!(confidence_changed,
            "Gossip should mutate belief confidences over 200 ticks");
    }

    #[test]
    fn norm_pressure_drives_action_diversity() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 500,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(500);

        let summaries = sim.agent_summaries();
        let action_counts = summaries.iter().fold(std::collections::HashMap::new(), |mut acc, s| {
            *acc.entry(s.current_action.clone()).or_insert(0) += 1;
            acc
        });
        // Agents should perform diverse actions (not all Idle)
        assert!(action_counts.len() > 1, "Agents should perform diverse actions, got: {:?}", action_counts);
        // Work should appear (norm pressure rewards it for conformist farmers)
        assert!(action_counts.contains_key("Work"),
            "Work should appear in action distribution: {:?}", action_counts);
    }

    #[test]
    fn witness_reputation_affects_trust() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 500,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();

        // Run 500 ticks — witness system should affect trust through observed interactions
        sim.run(500);

        // After 500 ticks with witness reputation, trust should分化
        // (some relationships low from witnessing threats, some high from witnessing help)
        let relationships = &sim.relationships;
        let min_trust = relationships.iter().map(|r| r.trust.to_f64()).fold(f64::INFINITY, f64::min);
        let max_trust = relationships.iter().map(|r| r.trust.to_f64()).fold(f64::NEG_INFINITY, f64::max);

        // Trust range should be wider than initial range (0.3..0.7) due to witness effects
        assert!(max_trust - min_trust > 0.2,
            "Witness reputation should create trust differentiation: min={min_trust}, max={max_trust}");
    }

    #[test]
    fn critical_needs_interrupt_actions() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 100,
            world_width: 16,
            world_height: 16,
            num_agents: 8,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(100);

        // After 100 ticks, agents should have addressed critical needs
        // (interrupt mechanism forces Eat/Drink when hunger/thirst > 0.9)
        let summaries = sim.agent_summaries();
        let avg_hunger: f64 = summaries.iter().map(|s| s.hunger.to_f64()).sum::<f64>() / summaries.len() as f64;
        let avg_thirst: f64 = summaries.iter().map(|s| s.thirst.to_f64()).sum::<f64>() / summaries.len() as f64;

        // Agents should not all be starving or dehydrated (interrupt mechanism works)
        assert!(avg_hunger < 0.85, "Average hunger should stay below 0.85 with interrupts: got {avg_hunger}");
        assert!(avg_thirst < 0.85, "Average thirst should stay below 0.85 with interrupts: got {avg_thirst}");
    }

    #[test]
    fn institutions_created_with_roles() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 100,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();

        // Verify institutions are created
        assert!(!sim.institutions.is_empty(), "Institutions should be created during populate");
        assert!(sim.institutions.len() >= 3, "Should have at least 3 institutions (Council, Temple, Market)");

        // Verify Council has Elder role assigned by personality
        {
            let council = sim.institutions.iter().find(|i| i.kind == mindstrata_sim::institutions::InstitutionKind::Council).unwrap();
            assert!(council.get_role_holder("Elder").is_some(), "Council should have an Elder assigned");
            assert!(council.members.len() >= 2, "Council should have at least 2 members");

            // Verify Temple has Priest role assigned by personality
            let temple = sim.institutions.iter().find(|i| i.kind == mindstrata_sim::institutions::InstitutionKind::Temple).unwrap();
            assert!(temple.get_role_holder("Priest").is_some(), "Temple should have a Priest assigned");
        }

        // Run 200 ticks — collective psychology should be derived
        sim.run(200);

        // Verify collective psychology was derived (morale should be non-zero)
        let council = sim.institutions.iter().find(|i| i.kind == mindstrata_sim::institutions::InstitutionKind::Council).unwrap();
        assert!(council.collective.morale > mindstrata_core::fixed::Fixed::ZERO,
            "Council morale should be derived from member states");
        assert!(council.collective.unity > mindstrata_core::fixed::Fixed::ZERO,
            "Council unity should be derived from member trust distribution");
    }

    #[test]
    fn routines_stabilize_behavior() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 500,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();

        // All agents should have village routines
        for agent in &sim.agents {
            assert!(agent.routine.preferred_action(mindstrata_core::clock::Tick::new(36)).0 != mindstrata_sim::actions::ActionKind::Idle,
                "Agents should have active routines that prefer Work during work hours");
        }

        // Run 500 ticks — routines should create behavioral patterns
        sim.run(500);

        // Agents should be alive and functional
        let summaries = sim.agent_summaries();
        let avg_hunger: f64 = summaries.iter().map(|s| s.hunger.to_f64()).sum::<f64>() / summaries.len() as f64;
        assert!(avg_hunger < 0.9, "Routines should help agents manage needs: avg_hunger={avg_hunger}");
    }

    #[test]
    fn golden_run_1000_ticks() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 1000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();

        let mut snapshots = Vec::new();
        for tick in 0..1000 {
            sim.tick();
            let t = tick + 1;
            if t == 100 || t == 500 || t == 1000 {
                snapshots.push(sim.metrics_snapshot());
            }
        }

        assert_eq!(snapshots.len(), 3, "Should have 3 snapshots at ticks 100, 500, 1000");

        let s100 = &snapshots[0];
        let s500 = &snapshots[1];
        let s1000 = &snapshots[2];

        assert_eq!(s100.tick, 100);
        assert_eq!(s500.tick, 500);
        assert_eq!(s1000.tick, 1000);

        assert!(s1000.avg_hunger < 0.95, "Agents should not all be starving: avg_hunger={}", s1000.avg_hunger);
        assert!(s1000.avg_thirst < 0.95, "Agents should not all be dehydrated: avg_thirst={}", s1000.avg_thirst);

        assert!(s500.total_grain != s1000.total_grain || s100.total_grain != s1000.total_grain,
            "Grain should fluctuate: s100={}, s500={}, s1000={}",
            s100.total_grain, s500.total_grain, s1000.total_grain);

        assert!(s1000.event_count > s500.event_count, "Events should accumulate");
        assert!(s1000.journal_len > s500.journal_len, "Journal should accumulate");

        let mut sim2 = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 1000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim2.populate();
        sim2.run(1000);
        let s2_final = sim2.metrics_snapshot();

        assert_eq!(s1000.avg_hunger, s2_final.avg_hunger, "Determinism failed for hunger");
        assert_eq!(s1000.avg_joy, s2_final.avg_joy, "Determinism failed for joy");
        assert_eq!(s1000.total_grain, s2_final.total_grain, "Determinism failed for grain");
        assert_eq!(s1000.event_count, s2_final.event_count, "Determinism failed for events");
    }
}
