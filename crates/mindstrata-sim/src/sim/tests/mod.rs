//! Domain-grouped unit tests (split from the former monolithic tests.rs;
//! pure moves — fn bodies untouched). Child modules inherit compilation
//! from the cfg(test)-gated `mod tests;` declaration in mod.rs.

mod biology;
mod census;
mod conflict;
mod culture;
mod development;
mod economy;
mod family;
mod governance;
mod legal;
mod psychology;

use super::{SimConfig, Simulation};

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

/// i342: the contacted-degree census must count rows carrying interaction
/// state, per `from` agent, and nothing else.
///
/// This is the quantity a queued §4.3 fix wires into two appraisal channels that
/// currently read the complete graph's length (N−1 for everyone, so they
/// discriminate nothing). Pinned here so the census cannot drift silently while
/// that fix waits for its re-anchor sweep.
#[test]
fn contacted_degrees_counts_only_rows_with_interaction_state() {
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: 1,
        world_width: 16,
        world_height: 16,
        num_agents: 8,
        snapshot_interval: None,
    });
    sim.populate();
    // At populate every row is a stranger: degree 0 everywhere, while the
    // list length is N−1 — the discrepancy the survey exists to name.
    let degrees = sim.contacted_degrees();
    assert_eq!(degrees.len(), sim.agents.len());
    assert!(
        degrees.iter().all(|d| *d == 0),
        "populate seeds stranger rows, so no agent has contacts yet"
    );
    assert_eq!(
        sim.agents[0].relationship_v2s.len(),
        sim.agents.len() - 1,
        "…while the list length is the whole population"
    );

    // After a run some agents have contacts, some may not, and the sum over
    // agents equals the number of touched rows exactly.
    sim.run(500);
    let degrees = sim.contacted_degrees();
    let touched = sim
        .relationships()
        .iter()
        .filter(|r| r.interaction_count > 0)
        .count();
    assert_eq!(
        degrees.iter().map(|d| *d as usize).sum::<usize>(),
        touched,
        "every touched row contributes to exactly its own `from` agent"
    );
}

/// i335: the relationship store is a **complete directed graph** at populate.
///
/// This is a *structural fact*, not an emergent outcome, and it is the reason
/// the whole tick is quadratic: i335 measured edges = N(N−1) exactly at
/// N=48/96/192 (`edges/agent = N−1`), at the fixed 32×32 charter size *and*
/// at constant density, and located the construction in `population.rs`
/// (every ordered pair gets a `Relationship` with random trust 0.3–0.7 and
/// `interaction_count: 0`; the birth path similarly links every newborn to
/// everyone). Every per-edge pass and every per-agent fold over
/// `relationship_v2s` therefore carries an O(N²) term even though interactions
/// themselves are locality-gated at the §2.4 perception radius.
///
/// The pin exists so that sparsifying the store (i335's recorded next lever)
/// is an explicit re-contract, not a silent change: if this assertion starts
/// failing because edges were created on contact, that is the intended repair
/// — re-anchor it with probe evidence rather than widening it.
#[test]
fn relationship_store_is_complete_at_populate() {
    use crate::sim::SimConfig;
    for n in [12usize, 24, 48] {
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 1,
            world_width: 32,
            world_height: 32,
            num_agents: n as u32,
            snapshot_interval: None,
        });
        sim.populate();
        assert_eq!(
            sim.relationships().len(),
            n * (n - 1),
            "N={n}: every ordered pair must hold a relationship row"
        );
        for (i, agent) in sim.agents.iter().enumerate() {
            assert_eq!(
                agent.relationship_v2s.len(),
                n - 1,
                "N={n} agent {i}: every agent must hold a row to every other"
            );
        }
        // The rows start untouched: a stranger edge that has never been
        // interacted with still occupies matrix and per-agent list space
        // (i326 measured 46–53% of them never touched for a whole run).
        assert!(
            sim.relationships().iter().all(|r| r.interaction_count == 0),
            "N={n}: populate seeds stranger rows with zero interactions"
        );
    }
}

/// i340: housing scales with the population, and the historical village is kept
/// exactly at small N.
///
/// This is the fix for i338's finding — with a fixed 8 houses every N lived on
/// exactly 8 cells, which pinned the relationship store at Ω(N²). The floor at
/// `DEFAULT_HOUSE_COUNT` is what makes the change a no-op for the calibrated
/// N=12 windows (and therefore for the goldens), so it is pinned here rather
/// than left to the probe.
#[test]
fn housing_scales_with_population_and_keeps_small_villages_historical() {
    // The pure rule: floor at 8, one house per ~4 villagers above it.
    for (n, expected) in [(1u32, 8u32), (12, 8), (32, 8), (33, 9), (48, 12), (96, 24)] {
        assert_eq!(
            crate::world_gen::houses_for_population(n),
            expected,
            "houses_for_population({n})"
        );
    }

    // End to end: the generated world carries that many House sites, and the
    // round-robin assignment respects the declared capacity of 4 (i340 measured
    // max co-location 12 → 4 at N=96).
    for (n, expected) in [(12usize, 8usize), (48, 12)] {
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 1,
            world_width: 32,
            world_height: 32,
            num_agents: n as u32,
            snapshot_interval: None,
        });
        sim.populate();
        let houses = sim
            .world
            .sites
            .iter()
            .filter(|s| matches!(s.kind, crate::world::SiteKind::House))
            .count();
        assert_eq!(houses, expected, "N={n}: house sites generated");
        let mut per_site = std::collections::BTreeMap::new();
        for a in sim.agents.iter() {
            if let Some(site) = a.home_site {
                *per_site.entry(site).or_insert(0usize) += 1;
            }
        }
        let worst = per_site.values().copied().max().unwrap_or(0);
        assert!(
            worst <= 4,
            "N={n}: {worst} agents share a house (declared capacity is 4)"
        );
    }
}

/// i345 (A11): above the calibrated house count the ring degenerates, so the
/// layout must place every house on its **own** tile.
///
/// i344 measured the defect this pins: `ring_span = min(w,h)/2 − 2` is a function
/// of world size only, so above ~29 houses the ring's angular stride collapses and
/// `place_site` silently overwrites an occupied tile — at N=192 the ring placed 48
/// houses onto **43 tiles**, and 19 agents ended up co-located on one cell against
/// the declared `SiteKind::House.capacity` of 4. The area packing (Vogel spiral,
/// applied above `MAX_RING_HOUSE_COUNT = 24`) places each house on a free tile, so
/// house sites and house tiles are one-to-one. The count check is what keeps the
/// round-robin in `population.rs` honest: fewer tiles than houses is the failure
/// mode, not a rounding detail.
#[test]
fn large_villages_place_one_house_per_tile() {
    let n = 192u32;
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: 1,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();

    let houses: Vec<_> = sim
        .world
        .sites
        .iter()
        .filter(|s| matches!(s.kind, crate::world::SiteKind::House))
        .map(|s| s.id)
        .collect();
    assert_eq!(
        houses.len(),
        crate::world_gen::houses_for_population(n) as usize,
        "house sites generated for N={n}"
    );

    let mut tiles_with_a_house = 0usize;
    for y in 0..32 {
        for x in 0..32 {
            if sim
                .world
                .tile(x, y)
                .and_then(|t| t.site)
                .is_some_and(|id| houses.contains(&id))
            {
                tiles_with_a_house += 1;
            }
        }
    }
    assert_eq!(
        tiles_with_a_house,
        houses.len(),
        "N={n}: every house must occupy its own tile (i344 measured the ring collapsing \
         48 houses onto 43 tiles)"
    );
}

/// i360: at town scale the world must be a **town of villages**, not one
/// uniform blob.
///
/// i359 measured the fault: i345's single Vogel spiral spreads houses evenly
/// over the disc, so `auto_partition_polities` collapses the density-law world
/// into ONE settlement (`gap 12 -> 0` at both N=192 and N=256). The clustered
/// layout (`cluster_count_for`) places one centre per ~16 houses; this pins both
/// the i344 one-tile-per-house invariant and the resulting multi-settlement
/// partition at the density-law sizes.
#[test]
fn clustered_world_forms_multiple_settlements_at_town_scale() {
    // Multi-seed: the separation must be structural, not a lucky-seed artifact
    // (doctrine §4.1 — the i338/i350 lesson).
    for seed in [42u64, 7, 1, 99] {
        for n in [192u32, 256] {
            let side = crate::world_gen::world_side_for_population(n);
            let cfg = || SimConfig {
                seed,
                max_ticks: 1,
                world_width: side,
                world_height: side,
                num_agents: n,
                snapshot_interval: None,
            };
            let mut sim = Simulation::new(cfg());
            sim.populate();
            let houses: Vec<_> = sim
                .world
                .sites
                .iter()
                .filter(|s| matches!(s.kind, crate::world::SiteKind::House))
                .map(|s| s.id)
                .collect();
            assert_eq!(
                houses.len(),
                crate::world_gen::houses_for_population(n) as usize,
                "seed {seed} N={n}: house sites generated"
            );
            let mut tiles = 0usize;
            for y in 0..side as i32 {
                for x in 0..side as i32 {
                    if sim
                        .world
                        .tile(x, y)
                        .and_then(|t| t.site)
                        .is_some_and(|id| houses.contains(&id))
                    {
                        tiles += 1;
                    }
                }
            }
            assert_eq!(tiles, houses.len(), "seed {seed} N={n}: one tile per house");

            let mut probe = Simulation::new(cfg());
            probe.populate();
            probe.auto_partition_polities(8);
            assert!(
                probe.polity_members.len() >= 2,
                "seed {seed} N={n}: expected a town of villages, found {} settlement(s) \
                 — the i359 one-settlement collapse",
                probe.polity_members.len()
            );
        }
    }
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
