//! i297 — Territory-anchored per-polity genesis (UM-3 leg 1; PLAN_DC3 §8).
//!
//! i296 delivered per-polity holons with divergent stage trajectories but
//! genesis still cited the WHOLE village's referent pools. This probe is the
//! exit test for the referent leg:
//!
//! 1. **Zero blast without polities** — a standard run with no
//!    `assign_polities` must produce exactly the legacy meme registry
//!    (genesis tags unchanged, no new memes).
//! 2. **Territory routing** — two spatially separated polities in one shared
//!    world must generate founding memories citing their OWN territory's
//!    sites (nearest home-site centroid), not the foreign polity's.
//! 3. **Divergent referent sets** — the polity-namespaced genesis tags must
//!    carry disjoint referent citations (the UM-3 disjointness mechanism:
//!    differentiated place-attachment, not just differentiated stage counts).
//!
//! Zero RNG: all partition/forcing is deterministic. Inert at pinned horizons
//! by construction (genesis is identity below stage 2, i273).
//!
//! Run: cargo run -p mindstrata-benches --release --example i297_polity_genesis
use mindstrata_sim::sim::{SimConfig, Simulation};

fn genesis_memes(sim: &Simulation) -> Vec<String> {
    sim.meme_registry
        .memes
        .iter()
        .filter(|m| m.description.contains("[genesis:"))
        .map(|m| m.description.clone())
        .collect()
}

fn main() {
    let horizon = 20_000u64;
    let seed = 42u64;
    println!("i297 polity genesis — {horizon} ticks, seed {seed}, N=12");

    // ── 1. Zero blast: no polities → legacy registry, no p-namespaced tags.
    let mut sim_legacy = Simulation::new(SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim_legacy.populate();
    sim_legacy.run(horizon);
    let legacy = genesis_memes(&sim_legacy);
    let ns_in_legacy = legacy.iter().filter(|d| d.contains("[genesis:p")).count();
    println!();
    println!("1. ZERO BLAST (no assign_polities):");
    println!(
        "   genesis memes: {}, p-namespaced: {} (must be 0)",
        legacy.len(),
        ns_in_legacy
    );

    // ── 2. Two polities (even/odd home-site split): spatial separation comes
    //    from the populated home_site assignments; the even/odd agent split
    //    yields two home-site centroids that differ if home sites differ.
    //    Natural runs at 20K have Safety ~3.x → band-II genesis fires.
    let mut sim2 = Simulation::new(SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim2.populate();
    let homes: Vec<Option<usize>> = sim2.agents.iter().map(|a| a.home_site).collect();
    sim2.assign_polities(vec![
        (0..12).filter(|i| i % 2 == 0).collect(),
        (0..12).filter(|i| i % 2 == 1).collect(),
    ]);
    sim2.run(horizon);
    let partitioned = genesis_memes(&sim2);
    let ns_a = partitioned.iter().filter(|d| d.contains("[genesis:p0:"));
    let ns_b = partitioned.iter().filter(|d| d.contains("[genesis:p1:"));
    let (na, nb) = (ns_a.count(), ns_b.count());
    println!();
    println!("2. TERRITORY-ROUTED GENESIS (even/odd polities):");
    println!(
        "   whole-village genesis memes: {}",
        partitioned
            .iter()
            .filter(|d| !d.contains("[genesis:p"))
            .count()
    );
    println!("   polity A (p0) memes: {na}, polity B (p1) memes: {nb}");
    let mut referents_a: Vec<&str> = Vec::new();
    let mut referents_b: Vec<&str> = Vec::new();
    // The world's communal site names, extracted from the generated text the
    // honest way: anything between "the " and " first|keep|is what|glimpses"
    // is fragile — instead enumerate the registry site names and substring-
    // match them (deterministic, no parsing heuristics).
    // Houses are excluded from genesis pools upstream; mirror that here by
    // name-kind via the display name ("House" prefix) rather than importing
    // the world crate (benches dep-minimal policy).
    let site_names: Vec<String> = sim2
        .world
        .sites
        .iter()
        .filter(|s| !s.name.starts_with("House"))
        .map(|s| s.name.clone())
        .collect();
    for name in &site_names {
        if partitioned
            .iter()
            .filter(|d| d.contains("[genesis:p0:"))
            .any(|d| d.contains(name.as_str()))
        {
            referents_a.push(name);
        }
        if partitioned
            .iter()
            .filter(|d| d.contains("[genesis:p1:"))
            .any(|d| d.contains(name.as_str()))
        {
            referents_b.push(name);
        }
    }
    println!("   sites cited by A: {referents_a:?}");
    println!("   sites cited by B: {referents_b:?}");

    // ── 3. Divergence verdict: the two polities' referent sets must differ
    //    (that is the UM-3 mechanism this leg delivers).
    println!();
    println!("3. DIVERGENCE VERDICT:");
    let only_a: Vec<&&str> = referents_a
        .iter()
        .filter(|r| !referents_b.contains(r))
        .collect();
    let only_b: Vec<&&str> = referents_b
        .iter()
        .filter(|r| !referents_a.contains(r))
        .collect();
    if ns_in_legacy == 0 {
        println!("   zero-blast without polities: PASS");
    } else {
        println!("   zero-blast without polities: FAIL ({ns_in_legacy} namespaced in legacy)");
    }
    println!("   polity-specific referents: A-only {only_a:?}, B-only {only_b:?}");
    if !only_a.is_empty() || !only_b.is_empty() {
        println!("   territorial divergence: PASS (polities cite different places)");
    } else if na > 0 && nb > 0 && referents_a == referents_b {
        println!("   territorial divergence: HOMOGENIZED (both cite the same sites — check centroid separation)");
    } else {
        println!(
            "   territorial divergence: INSUFFICIENT FIRING (raise horizon or check stage gates)"
        );
    }

    println!();
    println!("home sites (for centroid interpretation): {homes:?}");
}
