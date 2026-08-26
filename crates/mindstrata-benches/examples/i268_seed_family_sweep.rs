//! Seed-family sweep runner v1 — the shared instrument behind calibration
//! audit rules CA-1/CA-2/CA-4 (see runbooks/calibration-audit-v2.md).
//!
//! Runs the calm scenario across the versioned 12-seed family and prints
//! machine-greppable `key=value` rows per IC-4 law, then a threshold
//! verdict block (family pass-rate ≥ 11/12, saturation flags).
//!
//! Run: cargo run --release -p mindstrata-benches --example i268_seed_family_sweep

use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

/// Versioned seed family — changing this is a runbook change.
const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const TICKS: u64 = 4000;

fn main() {
    let mut rows = Vec::new();
    for &seed in &SEEDS {
        let config = SimConfig {
            seed,
            max_ticks: TICKS,
            num_agents: 12,
            snapshot_interval: None,
            ..SimConfig::default()
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(TICKS);
        let m = sim.metrics_snapshot();

        // CA-2 saturation probes on headline aggregates: a bounded series
        // pinned against its bound at p90 signals a dead producer even when
        // assertions pass on the saturated state.
        let fear_ok = m.fear_p90 < 0.95;
        let stress_ok = m.avg_stress < 0.95;
        // Health must retain non-degenerate population spread (not clamped
        // to zero by a dead restoring force — Iteration-242 class).
        let health_ok = m.avg_health > 0.05;
        // Skill machinery alive (nonzero practice effect).
        let skill_ok = m.avg_best_skill > 0.01;

        for (pin, ok, value) in [
            ("fear_p90_unsat", fear_ok, m.fear_p90),
            ("stress_unsat", stress_ok, m.avg_stress),
            ("health_alive", health_ok, m.avg_health),
            ("skill_alive", skill_ok, m.avg_best_skill),
        ] {
            println!(
                "seed={seed} pin={pin} passed={} value={value:.6} class=scalar",
                u8::from(ok)
            );
        }
        let pins_ok = fear_ok && stress_ok && health_ok && skill_ok;
        rows.push((seed, pins_ok));
    }

    let passing = rows.iter().filter(|(_, ok)| *ok).count();
    let rate = passing as f64 / rows.len() as f64;
    println!(
        "family_pass_rate={rate:.4} threshold=0.92 seeds_passing={passing}/{}",
        rows.len()
    );
    if rate >= 0.92 {
        println!("verdict=FAMILY_PASS");
    } else {
        println!("verdict=FAMILY_FAIL");
        for (seed, _) in rows.iter().filter(|(_, ok)| !ok) {
            println!("failing_seed={seed}");
        }
    }
}
