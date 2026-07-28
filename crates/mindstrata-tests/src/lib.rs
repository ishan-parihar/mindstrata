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
        // Agents should have non-zero hunger (needs decay over time)
        let avg_hunger: f64 = summaries.iter().map(|s| s.hunger.to_f64()).sum::<f64>() / summaries.len() as f64;
        assert!(avg_hunger > 0.0, "Average hunger should be non-zero after 100 ticks, got {avg_hunger}");

        // Agents should be performing actions
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

        // Capture snapshots at key ticks
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

        // Agents survive — average hunger should stay below critical threshold
        assert!(s1000.avg_hunger < 0.95, "Agents should not all be starving: avg_hunger={}", s1000.avg_hunger);
        assert!(s1000.avg_thirst < 0.95, "Agents should not all be dehydrated: avg_thirst={}", s1000.avg_thirst);

        // Resource economy: grain should fluctuate (production via Work, consumption via Eat)
        assert!(s500.total_grain != s1000.total_grain || s100.total_grain != s1000.total_grain,
            "Grain should fluctuate: s100={}, s500={}, s1000={}",
            s100.total_grain, s500.total_grain, s1000.total_grain);

        // Events and journal should accumulate
        assert!(s1000.event_count > s500.event_count, "Events should accumulate");
        assert!(s1000.journal_len > s500.journal_len, "Journal should accumulate");

        // Determinism: replay should produce identical results
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
