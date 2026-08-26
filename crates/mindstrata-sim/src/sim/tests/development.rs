//! Development-field state pins (DC-1 task 3.1).
//!
//! The attractor field is INERT until task 3.2 wires its first consumer:
//! these pins hold the pre-consumption invariants — presence on every
//! agent, founder endowment drawn deterministically at stream end,
//! newborn neutrality — so any drift here is caught before behavior ever
//! depends on the field.

use super::super::*;

fn make_sim(seed: u64) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim
}

/// Every agent carries a fully-formed field: one altitude per registered
/// line in [0,1], pathology quadrants at exact zero.
#[test]
fn field_present_on_every_founder_with_neutral_pathology() {
    let sim = make_sim(42);
    let line_count = mindstrata_development::line::all_lines().count();
    assert!(!sim.agents.is_empty());
    for (i, a) in sim.agents.iter().enumerate() {
        assert_eq!(
            a.development.altitudes.len(),
            line_count,
            "agent {i}: altitude vec length mismatch"
        );
        assert!(
            a.development
                .altitudes
                .iter()
                .all(|x| (0.0..=1.0).contains(x)),
            "agent {i}: altitude out of range"
        );
        assert!(
            a.development.pathology.is_neutral(),
            "agent {i}: pathology must be neutral pre-consumption"
        );
    }
}

/// Founder endowments are deterministic per seed (draw order contract):
/// same seed ⇒ identical altitude vectors across agents.
#[test]
fn founder_endowment_is_seed_deterministic() {
    let a = make_sim(4242);
    let b = make_sim(4242);
    for i in 0..a.agents.len() {
        assert_eq!(
            a.agents[i].development.altitudes, b.agents[i].development.altitudes,
            "agent {i}: endowment diverged between same-seed runs"
        );
    }
    // Different seeds must produce different endowments somewhere.
    let c = make_sim(777);
    let differs = (0..a.agents.len())
        .any(|i| a.agents[i].development.altitudes != c.agents[i].development.altitudes);
    assert!(differs, "different seeds produced identical endowments");
}

/// Newborns enter the world fully neutral (pathology + altitudes), per the
/// heredity-neutral control required before 3.x wiring.
#[test]
fn replacement_newborn_field_starts_fully_neutral() {
    let mut sim = make_sim(99);
    let line_count = mindstrata_development::line::all_lines().count();
    assert!(
        !sim.agents[0].development.is_fully_neutral(),
        "founder should carry an endowment"
    );

    sim.handle_agent_death(
        0,
        &[],
        sim.current_tick().as_u64(),
        sim.current_tick(),
        DeathCause::OldAge,
    );

    // The replacement newborn occupies the freed slot with a fully
    // neutral field — no inherited altitudes until heredity lands.
    assert_eq!(sim.agents[0].development.altitudes.len(), line_count);
    assert!(
        sim.agents[0].development.is_fully_neutral(),
        "replacement newborn must start fully neutral"
    );
}
