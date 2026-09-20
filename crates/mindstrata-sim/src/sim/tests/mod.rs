//! Domain-grouped unit tests (split from the former monolithic tests.rs;
//! pure moves — fn bodies untouched). Child modules inherit compilation
//! from the cfg(test)-gated `mod tests;` declaration in mod.rs.

mod biology;
mod conflict;
mod culture;
mod development;
mod economy;
mod family;
mod governance;
mod legal;
mod psychology;

use super::Simulation;

// ── i329: O(1) relationship lookup ────────────────────────────────────

/// `rel_pos` must agree with a linear scan for every pair — including after
/// ticks that add/remove relationships, where the revalidation fallback is
/// what keeps it correct (a birth or death moves the matrix and invalidates
/// the stored positions).
#[test]
fn rel_pos_matches_linear_scan_including_after_population_change() {
    use mindstrata_core::id::AgentId;
    let mut sim = super::Simulation::new(super::SimConfig {
        seed: 42,
        max_ticks: 3_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();

    let check = |sim: &mut super::Simulation, when: &str| {
        sim.rebuild_rel_lookup();
        let n = sim.agents.len();
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                let linear = sim.relationships().iter().position(|r| {
                    r.from == AgentId::new(i as u64) && r.to == AgentId::new(j as u64)
                });
                assert_eq!(sim.rel_pos(i, j), linear, "{when}: pair ({i},{j})");
            }
        }
        // Self-edges and out-of-range must be None.
        assert_eq!(sim.rel_pos(0, 0), None, "{when}: self edge");
        assert_eq!(sim.rel_pos(n, 0), None, "{when}: out of range");
    };

    check(&mut sim, "at populate");
    // Long enough to cross demography/birth windows, so the matrix changes
    // under the lookup between rebuilds.
    sim.run(3_000);
    check(&mut sim, "after 3000 ticks");
}

/// The buffer is left alone below `2×MAX_EVENTS` (so every calibrated
/// horizon — well under the bound — is byte-identical), and one bulk drop
/// brings it to `MAX_EVENTS` once it exceeds it. The cumulative reading is
/// carried by `total_event_count`, not by buffer length, so the trim cannot
/// move a public number.
#[test]
fn event_buffer_is_bounded_by_an_amortized_bulk_drop() {
    use crate::sim::MAX_EVENTS;
    let ev = || mindstrata_core::event::SimEvent::AgentSpawned {
        agent: mindstrata_core::id::AgentId::new(0),
        tick: mindstrata_core::clock::Tick::new(0),
    };

    let mut small = vec![ev(); MAX_EVENTS];
    Simulation::trim_event_buffer(&mut small);
    assert_eq!(small.len(), MAX_EVENTS, "at the cap: untouched");

    let mut over = vec![ev(); 2 * MAX_EVENTS];
    Simulation::trim_event_buffer(&mut over);
    assert_eq!(over.len(), 2 * MAX_EVENTS, "at 2×: still untouched");

    let mut huge = vec![ev(); 2 * MAX_EVENTS + 1];
    Simulation::trim_event_buffer(&mut huge);
    assert_eq!(huge.len(), MAX_EVENTS, "past 2×: one drop to the cap");
}

// Shared test helper (used by family + conflict domains).
/// §10.8: Find two agents in different seeded clans (home-site parity
/// seeds 2 clans during populate).
pub(crate) fn cross_clan_pair(sim: &Simulation) -> (usize, usize) {
    let clans = &sim.clan_registry.clans;
    assert!(clans.len() >= 2, "two clans must be seeded");
    assert!(!clans[0].core_households.is_empty(), "clan 0 has members");
    assert!(!clans[1].core_households.is_empty(), "clan 1 has members");
    (clans[0].core_households[0], clans[1].core_households[0])
}
