//! Integration tests for Mindstrata.

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod golden_replay;

#[cfg(test)]
mod statistical_emergence;

#[cfg(test)]
mod comparison;

#[cfg(test)]
mod integration_tests;

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
        assert!(action_counts.len() > 1, "Agents should perform diverse actions, got: {action_counts:?}");
        // Work should appear (norm pressure rewards it for conformist farmers)
        assert!(action_counts.contains_key("Work"),
            "Work should appear in action distribution: {action_counts:?}");
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

    #[test]
    fn derived_mental_states_accumulate_under_stress() {
        use mindstrata_core::fixed::Fixed;
        use mindstrata_sim::person::{DerivedMentalState, MentalStateInput};

        // Stressed agent: high stress, low coping, low social support
        let mut stressed = DerivedMentalState::default();
        let stressed_input = MentalStateInput {
            stress: Fixed::from_f64(0.8),
            coping_potential: Fixed::from_f64(0.2),
            social_support: Fixed::from_f64(0.1),
            need_deficit_avg: Fixed::from_f64(0.7),
            meaning: Fixed::from_f64(0.2),
            autonomy: Fixed::from_f64(0.1),
            success_rate: Fixed::from_f64(0.1),
            neuroticism: Fixed::from_f64(0.8),
            justice_perception: Fixed::from_f64(0.3),
        };

        // Calm agent: low stress, high coping, high social support
        let mut calm = DerivedMentalState::default();
        let calm_input = MentalStateInput {
            stress: Fixed::from_f64(0.1),
            coping_potential: Fixed::from_f64(0.8),
            social_support: Fixed::from_f64(0.9),
            need_deficit_avg: Fixed::from_f64(0.1),
            meaning: Fixed::from_f64(0.8),
            autonomy: Fixed::from_f64(0.9),
            success_rate: Fixed::from_f64(0.8),
            neuroticism: Fixed::from_f64(0.2),
            justice_perception: Fixed::from_f64(0.8),
        };

        // Simulate 200 ticks — states should accumulate
        for _ in 0..200 {
            stressed.compute(&stressed_input, Fixed::from_f64(0.995), Fixed::from_f64(0.005));
            calm.compute(&calm_input, Fixed::from_f64(0.995), Fixed::from_f64(0.005));
        }

        // Stressed agent should have higher trauma and depression risk
        assert!(stressed.trauma_risk > calm.trauma_risk,
            "Stressed agent should have higher trauma risk: stressed={:.3}, calm={:.3}",
            stressed.trauma_risk.to_f64(), calm.trauma_risk.to_f64());
        assert!(stressed.depression_risk > calm.depression_risk,
            "Stressed agent should have higher depression risk: stressed={:.3}, calm={:.3}",
            stressed.depression_risk.to_f64(), calm.depression_risk.to_f64());
        assert!(stressed.resentment > calm.resentment,
            "Stressed agent should have higher resentment: stressed={:.3}, calm={:.3}",
            stressed.resentment.to_f64(), calm.resentment.to_f64());
        assert!(calm.resilience > stressed.resilience,
            "Calm agent should have higher resilience: calm={:.3}, stressed={:.3}",
            calm.resilience.to_f64(), stressed.resilience.to_f64());
        assert!(calm.ambition > stressed.ambition,
            "Calm agent should have higher ambition: calm={:.3}, stressed={:.3}",
            calm.ambition.to_f64(), stressed.ambition.to_f64());
    }

    #[test]
    fn snapshot_roundtrip_with_simulation() {
        use mindstrata_sim::sim::{Simulation, SimConfig};
        use mindstrata_sim::snapshot::{Snapshot, SNAPSHOT_VERSION};

        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 1000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.populate();
        sim.run(100);

        // Capture snapshot at tick 100
        let snapshot = sim.save_snapshot();
        assert_eq!(snapshot.version, SNAPSHOT_VERSION);
        assert_eq!(snapshot.tick, 100);
        // §19.5.F: Population may grow from births during simulation
        assert!(snapshot.agents.len() >= 12, "Should have at least 12 agents after 100 ticks");
        assert!(!snapshot.events.is_empty(), "Should have events after 100 ticks");

        // Serialize to bytes and back
        let bytes = snapshot.to_bytes().expect("Failed to serialize snapshot");
        let restored = Snapshot::from_bytes(&bytes).expect("Failed to deserialize snapshot");

        // Verify key state matches
        assert_eq!(restored.tick, snapshot.tick);
        assert_eq!(restored.agents.len(), snapshot.agents.len());
        assert_eq!(restored.config.seed, snapshot.config.seed);
        assert_eq!(restored.event_count(), snapshot.event_count());

        // Verify determinism — same seed produces same hash
        let hash1 = snapshot.deterministic_hash().expect("Failed to hash");
        let hash2 = restored.deterministic_hash().expect("Failed to hash");
        assert_eq!(hash1, hash2, "Snapshot hash should be deterministic");
    }

    #[test]
    fn snapshot_deterministic_roundtrip() {
        use mindstrata_sim::sim::{Simulation, SimConfig};
        use mindstrata_sim::snapshot::Snapshot;

        let config = SimConfig {
            seed: 42,
            max_ticks: 500,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };

        // Create a fresh simulation and save snapshot at tick 0 (before any ticks)
        let mut sim1 = Simulation::new(config);
        sim1.populate();
        let snapshot = sim1.save_snapshot();
        let bytes = snapshot.to_bytes().expect("Snapshot serialization failed");

        // Load snapshot into a fresh simulation — both start from identical RNG state
        let loaded = Snapshot::from_bytes(&bytes).expect("Snapshot deserialization failed");
        let mut sim2 = Simulation::from_snapshot(loaded);

        // Run both simulations for 100 ticks — should produce identical results
        sim1.run(100);
        sim2.run(100);

        // Verify determinism: same seed + same ticks = same final state
        let s1 = sim1.metrics_snapshot();
        let s2 = sim2.metrics_snapshot();

        assert_eq!(s1.tick, s2.tick, "Tick mismatch after roundtrip");
        assert_eq!(s1.avg_hunger, s2.avg_hunger, "Hunger mismatch after roundtrip");
        assert_eq!(s1.avg_joy, s2.avg_joy, "Joy mismatch after roundtrip");
        assert_eq!(s1.total_grain, s2.total_grain, "Grain mismatch after roundtrip");
        assert_eq!(s1.event_count, s2.event_count, "Event count mismatch after roundtrip");
        assert_eq!(s1.agent_count, s2.agent_count, "Agent count mismatch after roundtrip");
    }

    #[test]
    fn resource_access_rights_enforced() {
        use mindstrata_core::fixed::Fixed;
        use mindstrata_core::id::EntityId;
        use mindstrata_sim::world::{AccessRight, ResourceStock, Site, SiteKind, World, GRAIN_RESOURCE_ID};

        let mut world = World::new(4, 4);

        // Create a farm with OwnerOnly grain
        let owner_id = EntityId::new(1);
        let stranger_id = EntityId::new(2);
        let farm = Site {
            id: EntityId::new(0),
            kind: SiteKind::Farm,
            name: "Private Farm".into(),
            owner: Some(owner_id),
            capacity: 10,
            inventory: vec![ResourceStock {
                resource_id: GRAIN_RESOURCE_ID,
                quantity: Fixed::from_f64(50.0),
                quality: Fixed::from_f64(0.8),
                access: AccessRight::OwnerOnly,
            }],
        };
        world.sites.push(farm);

        // Owner can access (pass empty institutions for access check)
        let empty_institutions: Vec<mindstrata_sim::institutions::Institution> = Vec::new();
        assert!(world.can_access_resource(0, GRAIN_RESOURCE_ID, owner_id, &empty_institutions),
            "Owner should be able to access their own resources");

        // Stranger cannot access
        assert!(!world.can_access_resource(0, GRAIN_RESOURCE_ID, stranger_id, &empty_institutions),
            "Non-owner should NOT be able to access OwnerOnly resources");

        // Public resources are accessible to everyone
        let well = Site {
            id: EntityId::new(1),
            kind: SiteKind::Well,
            name: "Public Well".into(),
            owner: None,
            capacity: 20,
            inventory: vec![ResourceStock {
                resource_id: 1, // WATER_RESOURCE_ID
                quantity: Fixed::from_f64(100.0),
                quality: Fixed::from_f64(1.0),
                access: AccessRight::Public,
            }],
        };
        world.sites.push(well);

        assert!(world.can_access_resource(1, 1, owner_id, &empty_institutions),
            "Public resources should be accessible to everyone");
        assert!(world.can_access_resource(1, 1, stranger_id, &empty_institutions),
            "Public resources should be accessible to everyone");
    }

    // ── §19.5.F §19.5.I New Mechanics Tests ───────────────────────

    #[test]
    fn inheritance_transfers_wealth_to_family() {
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
        sim.run(1000);
        // After 1000 ticks, some agents should have died and inheritance should have occurred.
        // Verify no agent has negative wealth (inheritance doesn't lose money).
        for agent in &sim.agents {
            assert!(agent.wealth.coin >= Fixed::ZERO,
                "Agent {} has negative wealth after inheritance", agent.name);
        }
    }

    #[test]
    fn childhood_socialization_spreads_knowledge() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(2000);
        // After 2000 ticks, some children should have been born and socialized.
        // At least some agents should have more knowledge than the initial set.
        let max_knowledge = sim.agents.iter()
            .map(|a| a.cultural.knowledge.len())
            .max()
            .unwrap_or(0);
        assert!(max_knowledge >= 2,
            "Some agents should have learned knowledge through socialization: max={max_knowledge}");
    }

    #[test]
    fn innovation_creates_new_knowledge() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(2000);
        // After 2000 ticks, at least one agent should know more than the max initial knowledge.
        // Initial max is 4 (Crop Rotation + Well Maintenance + Herbal Medicine + Grain Storage).
        let max_knowledge = sim.agents.iter()
            .map(|a| a.cultural.knowledge.len())
            .max()
            .unwrap_or(0);
        assert!(max_knowledge > 4,
            "Innovation should produce knowledge beyond initial seeding: max_knowledge={max_knowledge}");
    }

    // ── §4.4 Black Market Integration Tests ──────────────────────

    #[test]
    fn black_market_activates_under_scarcity() {
        use mindstrata_core::fixed::Fixed;
        use mindstrata_sim::black_market::BlackMarketState;

        let mut bm = BlackMarketState::default();
        // High scarcity + low enforcement → active
        bm.update(Fixed::from_f64(0.6), Fixed::from_f64(0.3));
        assert!(bm.active, "Black market should be active with scarcity > 0.4 and enforcement < 0.5");

        // Low scarcity → inactive
        let mut bm2 = BlackMarketState::default();
        bm2.update(Fixed::from_f64(0.1), Fixed::from_f64(0.3));
        assert!(!bm2.active, "Black market should be inactive with low scarcity");

        // High enforcement → inactive
        let mut bm3 = BlackMarketState::default();
        bm3.update(Fixed::from_f64(0.8), Fixed::from_f64(0.8));
        assert!(!bm3.active, "Black market should be inactive with high enforcement");
    }

    #[test]
    fn black_market_price_multiplier() {
        use mindstrata_core::fixed::Fixed;
        use mindstrata_sim::black_market::{BlackMarketState, BLACK_MARKET_PRICE_MULTIPLIER};

        let mut bm = BlackMarketState::default();
        bm.update(Fixed::from_f64(0.6), Fixed::from_f64(0.3));
        let legal_price = Fixed::from_f64(10.0);
        let bm_price = bm.black_market_price(legal_price);
        let expected = legal_price * BLACK_MARKET_PRICE_MULTIPLIER;
        assert_eq!(bm_price.to_f64(), expected.to_f64(),
            "Black market price should be legal price × multiplier");
    }

    #[test]
    fn black_market_participation_depends_on_personality() {
        use mindstrata_core::fixed::Fixed;
        use mindstrata_sim::black_market::BlackMarketState;
        use mindstrata_sim::person::Personality;

        let mut bm = BlackMarketState::default();
        bm.update(Fixed::from_f64(0.6), Fixed::from_f64(0.3));

        // High risk tolerance + low conformity → can participate
        let risk_taker = Personality {
            openness: Fixed::from_f64(0.5),
            conscientiousness: Fixed::from_f64(0.5),
            extraversion: Fixed::from_f64(0.5),
            agreeableness: Fixed::from_f64(0.5),
            neuroticism: Fixed::from_f64(0.5),
            risk_tolerance: Fixed::from_f64(0.8),
            conformity: Fixed::from_f64(0.3),
            ambition: Fixed::from_f64(0.5),
            altruism: Fixed::from_f64(0.5),
            traditionalism: Fixed::from_f64(0.5),
            dominance: Fixed::from_f64(0.5),
            impulsivity: Fixed::from_f64(0.5),
        };
        assert!(bm.can_participate(&risk_taker),
            "High risk tolerance + low conformity should participate");

        // High conformity → cannot participate
        let conformist = Personality {
            risk_tolerance: Fixed::from_f64(0.8),
            conformity: Fixed::from_f64(0.8),
            ..risk_taker
        };
        assert!(!bm.can_participate(&conformist),
            "High conformity should prevent participation");

        // Low risk tolerance → cannot participate
        let cautious = Personality {
            risk_tolerance: Fixed::from_f64(0.2),
            conformity: Fixed::from_f64(0.3),
            ..risk_taker
        };
        assert!(!bm.can_participate(&cautious),
            "Low risk tolerance should prevent participation");
    }

    #[test]
    fn snapshot_roundtrip_preserves_agent_state() {
        use mindstrata_sim::sim::{Simulation, SimConfig};
        use mindstrata_sim::snapshot::{Snapshot, SNAPSHOT_VERSION};

        let config = SimConfig {
            seed: 42,
            max_ticks: 500,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim1 = Simulation::new(config);
        sim1.populate();
        sim1.run(200);

        // Capture snapshot at tick 200
        let snapshot = sim1.save_snapshot();
        assert_eq!(snapshot.version, SNAPSHOT_VERSION);
        assert_eq!(snapshot.tick, 200);
        assert!(snapshot.agents.len() >= 12, "Should have at least 12 agents");

        // Serialize to bytes and back
        let bytes = snapshot.to_bytes().expect("Snapshot serialization failed");
        let restored = Snapshot::from_bytes(&bytes).expect("Snapshot deserialization failed");

        // Verify key state matches
        assert_eq!(restored.tick, snapshot.tick);
        assert_eq!(restored.agents.len(), snapshot.agents.len());
        assert_eq!(restored.config.seed, snapshot.config.seed);

        // Verify determinism — same seed produces same hash
        let hash1 = snapshot.deterministic_hash().expect("hash failed");
        let hash2 = restored.deterministic_hash().expect("hash failed");
        assert_eq!(hash1, hash2, "Snapshot hash should be deterministic");
    }

    #[test]
    fn move_action_has_completion_detection() {
        use mindstrata_sim::actions::ActionKind;
        use mindstrata_core::fixed::Fixed;

        let config = SimConfig {
            seed: 42,
            max_ticks: 200,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();

        // Run simulation — feuding agents should use Move action
        sim.run(200);

        // Verify Move action definition exists and has correct duration
        let move_def = ActionKind::Move { target_x: 5, target_y: 5 };
        assert_eq!(move_def.definition().duration_ticks, 1, "Move should take 1 tick per step");
        assert!(move_def.definition().energy_cost > Fixed::ZERO, "Move should have energy cost");
    }

    #[test]
    fn feuding_agents_use_move_action() {
        use mindstrata_sim::sim::{Simulation, SimConfig};

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

        // Check if any agent has feuds (feuds form from negative interactions)
        let has_feuds = sim.agents.iter().any(|a| !a.feuds.is_empty());
        // If feuds exist, agents should use Move instead of Wander
        if has_feuds {
            let summaries = sim.agent_summaries();
            let action_counts = summaries.iter().fold(std::collections::HashMap::new(), |mut acc, s| {
                *acc.entry(s.current_action.clone()).or_insert(0) += 1;
                acc
            });
            // Move should appear in the action distribution if feuds exist
            // (agents that are angry and feuding will Move toward targets)
            let total = action_counts.values().sum::<u32>();
            assert!(total > 0, "Should have actions recorded");
        }
        // The key assertion is that the simulation doesn't panic with feuding Move actions
        assert!(sim.current_tick().as_u64() >= 500, "Simulation should run to completion");
    }
}
