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

// ── i331: hoisted social-status fold ──────────────────────────────────

/// The hoisted O(R) `social_status_counts` pass must yield exactly the
/// per-agent `(positive, total)` counts the old per-agent
/// `relationships.iter().filter(r.from == i)` fold produced — for real sim
/// data (including duplicate pairs) and for a synthetic list carrying an
/// out-of-range `from` (a stale id after a death), which the old fold could
/// never match and must not be counted here either.
#[test]
fn social_status_counts_matches_per_agent_fold() {
    use super::Fixed;
    use crate::person::{Relationship, RelationshipKind};
    use mindstrata_core::id::AgentId;

    let mk = |from: u64, to: u64, trust: f64| Relationship {
        from: AgentId::new(from),
        to: AgentId::new(to),
        trust: Fixed::from_f64(trust),
        affection: Fixed::from_f64(0.5),
        respect: Fixed::ZERO,
        fear: Fixed::ZERO,
        obligation: Fixed::ZERO,
        last_interaction_tick: 0,
        kind: RelationshipKind::Stranger,
        interaction_count: 0,
        last_positive_tick: 0,
        last_negative_tick: 0,
    };

    let naive = |rels: &[Relationship], i: usize| -> (u32, u32) {
        let id = AgentId::new(i as u64);
        rels.iter()
            .filter(|r| r.from == id)
            .fold((0u32, 0u32), |(pos, total), r| {
                (pos + u32::from(r.trust > Fixed::from_f64(0.6)), total + 1)
            })
    };

    // Real sim data after enough ticks to have accumulated duplicate/edge rows.
    let mut sim = Simulation::new(super::SimConfig {
        seed: 42,
        max_ticks: 500,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(500);
    let n = sim.agents.len();
    let counts = super::memory_ops::social_status_counts(sim.relationships(), n);
    for i in 0..n {
        assert_eq!(counts[i], naive(sim.relationships(), i), "agent {i}");
    }

    // Synthetic: duplicate pairs keep both rows in the total; an out-of-range
    // `from` contributes nothing (idents > n are not an indexable agent).
    let rels = vec![
        mk(0, 1, 0.9),
        mk(0, 2, 0.2),
        mk(0, 1, 0.7),
        mk(1, 0, 0.65),
        mk(99, 0, 1.0),
    ];
    let counts = super::memory_ops::social_status_counts(&rels, 2);
    assert_eq!(counts[0], naive(&rels[..4], 0), "duplicates counted");
    assert_eq!(counts[1], naive(&rels[..4], 1), "single pair");
    assert_eq!(counts.len(), 2, "range clamped to the population");
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
