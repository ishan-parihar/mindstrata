//! i360 — the clustered world generator: does a 256-agent town actually become
//! a town of villages, and does the already-built multi-settlement machinery go
//! live?
//!
//! i359 measured the fault: i345's single Vogel spiral spreads houses *evenly*
//! over the whole disc, so on the i344 density world law (`side(N) =
//! max(16, ceil(sqrt(21.333·N)))`) the mean inter-house spacing (~9 tiles at
//! N=192/64²) is below any partition threshold and `auto_partition_polities`
//! collapses the world into **one settlement** (`gap 12 -> 0`, `gap 20 -> 0`).
//!
//! This probe measures the clustered layout (`cluster_count_for`: one village
//! centre per ~16 houses, capped at 4) against four questions:
//!
//!   1. **Settlements** — does `auto_partition_polities` now find K villages?
//!   2. **Liveness** — do agents survive and stay healthy, i.e. is the layout
//!      habitable (site access is institution-based, so this is a real check)?
//!   3. **Polity genesis** — does each settlement get its own holon? Count the
//!      `[genesis:pN:...]` memes minted per namespace.
//!   4. **Calibrated-path guard** — every run below 25 houses must be untouched
//!      (the ring path) and the cluster rule must not fire there.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i360_clustered_world`

use mindstrata_sim::sim::{SimConfig, Simulation};

/// i344 density world law.
fn density_side(n: u32) -> u32 {
    let s = (21.333_f64 * n as f64).sqrt().ceil() as u32;
    s.max(16)
}

fn build(n: u32, seed: u64, side: u32, ticks: u64) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: side,
        world_height: side,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim
}

fn main() {
    println!("i360 — clustered world generator (seed 42)\n");

    // ── Leg A: settlement structure at town scale ──
    println!("leg A — settlements found by auto_partition_polities (i359 baseline:");
    println!("        N=192/256 gave 48/58 @ gap4, 20/21 @ gap8, 0 @ gap12, 0 @ gap20):");
    println!(
        "{:<6} {:<5} {:<6} {:>8} {:>8} {:>8} {:>8}",
        "N", "side", "houses", "gap4", "gap8", "gap12", "gap20"
    );
    for n in [192u32, 256] {
        let side = density_side(n);
        let probe = build(n, 42, side, 0);
        let houses = probe
            .world
            .sites
            .iter()
            .filter(|s| matches!(s.kind, mindstrata_sim::world::SiteKind::House))
            .count();
        let mut row = Vec::new();
        for g in [4i32, 8, 12, 20] {
            let mut p = build(n, 42, side, 0);
            p.auto_partition_polities(g);
            row.push(p.polity_members.len());
        }
        println!(
            "{:<6} {:<5} {:<6} {:>8} {:>8} {:>8} {:>8}",
            n, side, houses, row[0], row[1], row[2], row[3]
        );
    }

    // ── Leg B: liveness + polity genesis at town scale ──
    println!("\nleg B — liveness + per-polity genesis (gap 8, 10K ticks):");
    for n in [192u32, 256] {
        let side = density_side(n);
        let mut sim = build(n, 42, side, 10_000);
        sim.auto_partition_polities(8);
        let polities = sim.polity_members.len();
        let members: Vec<usize> = sim.polity_members.iter().map(|p| p.len()).collect();
        sim.run(10_000);
        let ms = sim.metrics_snapshot();
        let zero_coin = sim
            .agents
            .iter()
            .filter(|a| a.wealth.coin.to_f64() <= 1e-6)
            .count();
        // Per-polity genesis liveness: count memes minted into each `pN`
        // namespace by the Era IV genesis pass (i158 tag convention).
        let mut per_polity = vec![0usize; polities];
        for meme in &sim.meme_registry.memes {
            for k in 0..polities {
                if meme.description.contains(&format!("[genesis:p{k}:")) {
                    per_polity[k] += 1;
                }
            }
        }
        println!(
            "  N={n} side={side}: agents {}  health {:.3}  hunger {:.4}  stress {:.3}  grain {:.2}  gini {:.3}  zero-coin {zero_coin}",
            ms.agent_count, ms.avg_health, ms.avg_hunger, ms.avg_stress, ms.total_grain, ms.gini
        );
        println!(
            "    polities {polities} (members {members:?})  genesis-memes/polity {per_polity:?}"
        );
    }

    // ── Leg C: calibrated-path guard ──
    println!("\nleg C — calibrated-path guard (cluster rule must be inert <= 24 houses):");
    for (n, houses) in [(12u32, 8u32), (48, 12), (96, 24)] {
        let sim = build(n, 42, 32, 0);
        let got = sim
            .world
            .sites
            .iter()
            .filter(|s| matches!(s.kind, mindstrata_sim::world::SiteKind::House))
            .count() as u32;
        println!(
            "  N={n}: houses {got} (expected {houses})  cluster_count({houses}) = {}",
            mindstrata_sim::world_gen::cluster_count_for(houses)
        );
    }
}
