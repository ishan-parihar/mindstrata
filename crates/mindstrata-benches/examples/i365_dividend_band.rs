//! i365 — is the council surplus dividend's improvement structural, or a
//! two-seed artifact?
//!
//! i363 measured the dividend on seeds 42 and 7 only (Gini 0.647→0.611,
//! 0.652→0.619) and i364 confirmed it does not soften crisis mortality. Doctrine
//! §4.1 forbids pinning a change on a lucky seed, so this probe widens the family:
//! several seeds on the identical 46×46 / N=48 / 50K harness, reporting the final
//! Gini, the bottom-half share, and the **council treasury** (the hoard the fix
//! was aimed at draining).
//!
//! Reference points: the pre-fix plateau was ~0.647–0.652 (i358), and the
//! unspent council hoard reached 163 602 coins (i361).
//!
//! Run: `cargo run --release -p mindstrata-benches --example i365_dividend_band`

use mindstrata_sim::sim::{SimConfig, Simulation};

fn gini(xs: &mut [f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = xs.len() as f64;
    let sum: f64 = xs.iter().sum();
    if sum <= 0.0 {
        return 0.0;
    }
    let weighted: f64 = xs
        .iter()
        .enumerate()
        .map(|(i, x)| (i as f64 + 1.0) * x)
        .sum();
    (2.0 * weighted) / (n * sum) - (n + 1.0) / n
}

fn main() {
    println!("i365 — council surplus dividend, multi-seed band (46x46, N=48, 50K)");
    println!("      pre-fix plateau ~0.647/0.652; unspent council hoard reached 163 602 (i361)\n");
    println!(
        "{:>5} {:>8} {:>10} {:>10} {:>9} {:>8} {:>8}",
        "seed", "gini", "bottom-half", "council$", "health", "hunger", "grain"
    );
    let mut ginis = Vec::new();
    let mut hoards = Vec::new();
    for seed in [42u64, 7, 1, 99, 13, 23] {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: 50_000,
            world_width: 46,
            world_height: 46,
            num_agents: 48,
            snapshot_interval: None,
        });
        sim.populate();
        sim.run(50_000);
        let mut w: Vec<f64> = sim.agents.iter().map(|a| a.wealth.coin.to_f64()).collect();
        let total: f64 = w.iter().sum();
        let g = gini(&mut w);
        let half = w.len() / 2;
        let bottom: f64 = w[..half].iter().sum();
        let council = sim
            .institutions
            .iter()
            .find(|i| i.kind == mindstrata_sim::institutions::InstitutionKind::Council)
            .map_or(0.0, |i| i.treasury.to_f64());
        let ms = sim.metrics_snapshot();
        ginis.push(g);
        hoards.push(council);
        println!(
            "{seed:>5} {g:>8.4} {:>9.1}% {council:>10.1} {:>9.3} {:>8.4} {:>8.2}",
            if total > 0.0 {
                100.0 * bottom / total
            } else {
                0.0
            },
            ms.avg_health,
            ms.avg_hunger,
            ms.total_grain
        );
    }
    let mean = ginis.iter().sum::<f64>() / ginis.len() as f64;
    let max = ginis.iter().cloned().fold(f64::MIN, f64::max);
    let max_hoard = hoards.iter().cloned().fold(f64::MIN, f64::max);
    println!(
        "\nband: gini mean {mean:.4}  max {max:.4}  (pre-fix plateau ~0.647–0.652)  max council hoard {max_hoard:.0}"
    );
    println!(
        "verdict: {}",
        if max < 0.647 {
            "IMPROVEMENT_STRUCTURAL (every seed below the pre-fix plateau)"
        } else {
            "MIXED (at least one seed at/above the pre-fix plateau)"
        }
    );
}
