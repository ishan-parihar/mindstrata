//! i359 — can we reach a town? N≥192 with settlement auto-partition.
//!
//! The plan's scale-proof item: run the sim at N≥192 on the i344 **density
//! world law** (`side(N) = max(16, ceil(sqrt(21.333·N)))`, anchored on the two
//! calibrated points — N=12 in 16×16 and N=48 in 32×32 both give 21.33
//! cells/agent), with the i298 settlement auto-partition deriving polities from
//! house geography, and ask the three questions that separate "toy" from
//! "town":
//!
//!   1. **Cost** — µs/tick at N=96/144/192/256, against the DC-3 Phase-1
//!      charter target (N=96 ≤ 6500 µs/tick, i295/i332 method: populate
//!      included).
//!   2. **Liveness** — do the agents survive and stay healthy at scale, or is
//!      the run a saturating pile?
//!   3. **Settlement structure** — does `auto_partition_polities` find more
//!      than one settlement at the density-law world sizes, i.e. is there a
//!      "multi-settlement town" to orchestrate at all?
//!
//! Run: `cargo run --release -p mindstrata-benches --example i359_town_scale`

use mindstrata_sim::sim::{SimConfig, Simulation};
// i392: the i344 density world law is single-sourced in the world generator
// (`world_side_for_population`). Six probes had grown their own copy; do not
// re-implement it — the policy must have one definition.
use mindstrata_sim::world_gen::world_side_for_population as density_side;

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
    println!("i359 — town-scale probe (seed 42)\n");

    // ── Leg A: cost vs N on the density world law (i295/i332 method) ──
    println!("leg A — cost (new+populate+run, 2000 ticks, min-of-3):");
    println!(
        "{:<6} {:<6} {:>12} {:>12}",
        "N", "side", "us/tick", "vs N96 6500"
    );
    for n in [48u32, 96, 144, 192, 256] {
        let side = density_side(n);
        let mut best = f64::MAX;
        for _ in 0..3 {
            let t0 = std::time::Instant::now();
            let mut sim = build(n, 42, side, 2000);
            sim.run(2000);
            let per_tick = t0.elapsed().as_secs_f64() * 1e6 / 2000.0;
            if per_tick < best {
                best = per_tick;
            }
        }
        println!(
            "{:<6} {:<6} {:>12.1} {:>12.1}",
            n,
            side,
            best,
            best - 6500.0
        );
    }

    // ── Leg B: liveness + settlement structure at N=192/256 ──
    println!("\nleg B — liveness + settlements (density world, 10K ticks):");
    for n in [192u32, 256] {
        let side = density_side(n);
        let mut sim = build(n, 42, side, 10_000);
        // i298: derive polities from settlement geography. Report a few
        // max_gap settings so the cluster-count sensitivity is visible.
        let gaps = [4i32, 8, 12, 20];
        let mut parts: Vec<(i32, usize)> = Vec::new();
        for g in gaps {
            let mut probe = build(n, 42, side, 0);
            probe.auto_partition_polities(g);
            parts.push((g, probe.polity_members.len()));
        }
        sim.run(10_000);
        let ms = sim.metrics_snapshot();
        let zero_coin = sim
            .agents
            .iter()
            .filter(|a| a.wealth.coin.to_f64() <= 1e-6)
            .count();
        let partnered = sim.agents.iter().filter(|a| a.partner.is_some()).count();
        let mean_rel = sim
            .agents
            .iter()
            .map(|a| a.relationship_v2s.len())
            .sum::<usize>() as f64
            / sim.agents.len().max(1) as f64;
        println!(
            "  N={n} side={side}: agents {}  health {:.3}  hunger {:.4}  stress {:.3}  grain {:.2}",
            ms.agent_count, ms.avg_health, ms.avg_hunger, ms.avg_stress, ms.total_grain
        );
        println!(
            "    families {}  partnered {}  zero-coin {}  mean-rel-rows {:.0}  gini {:.3}",
            ms.family_count, partnered, zero_coin, mean_rel, ms.gini
        );
        let p: Vec<String> = parts
            .iter()
            .map(|(g, c)| format!("gap {g} -> {c}"))
            .collect();
        println!(
            "    settlements (auto_partition_polities): {}",
            p.join("  ")
        );
    }
}
