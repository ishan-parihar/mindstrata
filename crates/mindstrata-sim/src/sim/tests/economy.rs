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
    // Iteration 257 (Phase-5 world variance): the founding granary scales
    // with local soil fertility, so the seeded stock is no longer a fixed
    // 100 — the invariant is that `stored` captured the pre-pump level.
    assert!(
        stored >= Fixed::from_f64(80.0) && stored <= Fixed::from_f64(150.0),
        "farm seeds within the fertility-multiplied band: {stored}"
    );
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

/// i384: the trade price's trust read AND the trade's trust write both live on
/// the dyadic store now — this site was the last self-contained read+write pair
/// on the legacy v1 `relationships` matrix.
///
/// Probe (`i384_economy_trade_store`, 3 worlds × 20K): the two stores were
/// **+0.09…+0.13 apart on exactly the pairs that trade** (village 0.9174 v1 vs
/// 0.8242 v2; town 0.8068 vs 0.7036) because this site was itself a v1 writer
/// at a flat +0.02/act — 2× the dyadic gain — so the buyer paid 2.8…4.0% below
/// the price its own trust warranted. After the migration the same traded pairs
/// agree to −0.006…−0.009 and the read-source delta is 0.34…0.52%.
#[test]
fn trade_price_reads_the_dyadic_store_and_writes_it() {
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
    sim.rebuild_rel_lookup();

    // A pair whose v1 and v2 trust deliberately disagree (the divergence the
    // migration removed at this site: v1 used to be the HIGHER one).
    let p = sim.rel_pos(0, 1).expect("pair (0,1) row exists");
    let v1_trust = sim.relationships[p].trust;
    let q = Simulation::relationship_v2_pos(0, 1);
    sim.agents[0].relationship_v2s[q].trust = Fixed::from_f64(0.1);

    // The trade price's read expression, verbatim (`economy.rs`).
    let price_trust = sim
        .relationship_v2_between(0, 1)
        .map_or(Fixed::from_f64(0.5), |r| r.trust);
    assert_eq!(
        price_trust,
        Fixed::from_f64(0.1),
        "the trade price must read the dyadic store (0.1), not the v1 matrix"
    );
    assert_ne!(
        v1_trust, price_trust,
        "v1 must be a distinct store — the gap this migration closed"
    );

    // The write expression, verbatim: one completed trade is one positive act.
    let before_trust = sim.agents[0].relationship_v2s[q].trust;
    let before_affection = sim.agents[0].relationship_v2s[q].affection;
    let before_count = sim.agents[0].relationship_v2s[q].interaction_count;
    sim.agents[0].relationship_v2s[q].record_positive(1, Fixed::ONE);
    let after = &sim.agents[0].relationship_v2s[q];
    // Default volatility is 0.5, so trust gains 0.5 × 0.02 = 0.0100 and the
    // full dyadic state moves (affection/commitment/intimacy), unlike the
    // scalar +0.02 the legacy write carried.
    assert_eq!(after.trust - before_trust, Fixed::from_f64(0.01));
    assert!(after.affection > before_affection);
    assert_eq!(after.interaction_count, before_count + 1);
    assert_eq!(
        sim.relationships[p].trust, v1_trust,
        "the trade write must no longer touch the v1 matrix"
    );
}
