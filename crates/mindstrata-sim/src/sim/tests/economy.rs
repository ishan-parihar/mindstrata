//! Domain-grouped unit tests (split from tests.rs; pure moves).

use super::super::*;

/// §8.1.18: The apprenticeship pass transmits knowledge from a capable
/// teacher to a willing student. Agent 0 holds knowledge id 2 (Herbal
/// Medicine) with high teaching skill; Agent 1 lacks it with high
/// learning aptitude. The pass must transfer it, record both education
/// events, and bump the knowledge-store holder count.
#[test]
fn apprenticeship_transfers_knowledge_from_teacher_to_student() {
    use crate::culture::education::EducationEvent;
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 2,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    // Teacher: knows id 2, can teach.
    sim.agents[0].education.learned = vec![2];
    sim.agents[0].education.teaching_skill = Fixed::from_f64(0.9);
    sim.agents[0].education.teaching_patience = Fixed::from_f64(0.8);
    // Student: lacks id 2, learns fast.
    sim.agents[1].education.learning_aptitude = Fixed::from_f64(0.9);
    sim.agents[1].cultural.knowledge.retain(|&k| k != 2);
    // A warm relationship makes the transfer reliable.
    if let Some(r) = sim
        .relationships
        .iter_mut()
        .find(|r| r.from.as_u64() == 0 && r.to.as_u64() == 1)
    {
        r.trust = Fixed::from_f64(0.9);
        r.affection = Fixed::from_f64(0.8);
    }
    let holders_before = sim
        .knowledge_store
        .iter()
        .find(|k| k.id == 2)
        .map_or(0, |k| k.holders);
    sim.run_apprenticeship_pass(1, Tick::new(1));
    assert!(
        sim.agents[1].education.has_learned(2),
        "student must learn the taught knowledge"
    );
    assert!(
        sim.agents[1].cultural.knowledge.contains(&2),
        "student's cultural knowledge must include id 2"
    );
    let teach_ok = sim.agents[0]
        .education
        .teaching_events
        .iter()
        .any(|e: &EducationEvent| e.knowledge_id == 2 && e.success);
    assert!(teach_ok, "teacher must record a successful teaching event");
    let learn_ok = sim.agents[1]
        .education
        .learning_events
        .iter()
        .any(|e: &EducationEvent| e.knowledge_id == 2 && e.success);
    assert!(learn_ok, "student must record a successful learning event");
    let holders_after = sim
        .knowledge_store
        .iter()
        .find(|k| k.id == 2)
        .map_or(0, |k| k.holders);
    assert!(
        holders_after > holders_before,
        "knowledge holder count must grow"
    );
}

#[test]
fn storage_overflow_rots_exposed_grain_only() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 2,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let farm_idx = sim
        .world
        .sites
        .iter()
        .position(|s| s.kind == crate::world::SiteKind::Farm)
        .unwrap();
    let stored = sim.world.sites[farm_idx].inventory[0].quantity;
    // Pump the farm far past its 500-unit storage capacity.
    sim.world
        .produce_resource(farm_idx, GRAIN_RESOURCE_ID, Fixed::from_f64(600.0));
    let before = sim.world.sites[farm_idx].inventory[0].quantity;
    sim.apply_storage_overflow();
    let after = sim.world.sites[farm_idx].inventory[0].quantity;
    assert!(after < before, "overflowing grain must rot");
    assert!(
        after > Fixed::from_f64(500.0),
        "only the exposed overflow rots, never the stored grain"
    );
    assert_eq!(stored, Fixed::from_f64(100.0), "farm seeds 100 grain");
}

#[test]
fn storage_under_capacity_is_transparent() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 2,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let farm_idx = sim
        .world
        .sites
        .iter()
        .position(|s| s.kind == crate::world::SiteKind::Farm)
        .unwrap();
    let before = sim.world.sites[farm_idx].inventory[0].quantity;
    sim.apply_storage_overflow();
    assert_eq!(
        sim.world.sites[farm_idx].inventory[0].quantity, before,
        "under-capacity storage must not lose goods"
    );
}

#[test]
fn storage_overflow_does_not_rot_non_perishables() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 2,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let well_idx = sim
        .world
        .sites
        .iter()
        .position(|s| s.kind == crate::world::SiteKind::Well)
        .unwrap();
    sim.world
        .produce_resource(well_idx, WATER_RESOURCE_ID, Fixed::from_f64(5000.0));
    let before = sim.world.sites[well_idx].inventory[0].quantity;
    sim.apply_storage_overflow();
    assert_eq!(
        sim.world.sites[well_idx].inventory[0].quantity, before,
        "water is non-perishable: overflow must not destroy it"
    );
}

/// §8.1.18 zero-at-zero companion: when nobody in the village can teach a
/// knowledge item (no capable teacher), the pass transfers nothing — the
/// education system stays inert until a qualified teacher exists.
#[test]
fn apprenticeship_no_teacher_transfers_nothing() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 2,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    // Nobody knows id 2 (or can teach it) — student lacks it.
    sim.agents[0].education.learned.clear();
    sim.agents[1].education.learned.clear();
    sim.agents[0].education.teaching_skill = Fixed::from_f64(0.1);
    let before = sim.agents[1].cultural.knowledge.len();
    sim.run_apprenticeship_pass(1, Tick::new(1));
    assert_eq!(
        sim.agents[1].cultural.knowledge.len(),
        before,
        "no teacher means no transfer"
    );
    assert!(
        !sim.agents[1].education.has_learned(2),
        "student must not learn without a teacher"
    );
}
