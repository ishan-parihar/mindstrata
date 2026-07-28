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
}
