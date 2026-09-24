//! i376 leg C — did the v1→v2 store convergence DEGRADE the moral-panic
//! producer, or merely re-pace which seeds fire?
//!
//! The two panic integration tests (`beliefs_memory`, `governance`) re-anchored
//! their seed family three times as pacing shifted (i343, i351), each time via a
//! sweep probe. This repeats that measurement under i376 so the verdict is
//! evidence, not assertion: pestilence crisis @20K over a widened seed set,
//! reporting panic registrations, peak intensity, and the belief-charge driver.
//!
//! Pre-i376 reference counts (i351, same harness): {5: 0, 7: 11, 42: 0, 11: 5,
//! 46: 2} — 3 of 5 register, so the "majority of the family" contract held.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i376_panic_sweep`

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

fn main() {
    println!("i376 leg C — pestilence crisis @20K, panic registration by seed");
    println!("      pre-i376 (i351): {{5: 0, 7: 11, 42: 0, 11: 5, 46: 2}} — 3/5 register\n");
    println!(
        "{:>6} {:>8} {:>10} {:>10} {:>12}",
        "seed", "panics", "peak_int", "avg_charge", "registered"
    );
    let mut firing = 0;
    let mut n = 0;
    for seed in [5u64, 7, 42, 11, 46, 1, 23, 13, 99, 3] {
        let mut sc = Scenario::pestilence();
        sc.seed = seed;
        sc.ticks = 20_000;
        let mut s = Simulation::from_scenario(sc);
        s.populate();
        s.run(20_000);
        let panics = &s.moral_panic_registry.panics;
        let count = panics.len();
        let peak = panics
            .iter()
            .map(|p| p.intensity.to_f64())
            .fold(0.0f64, f64::max);
        // The belief-charge driver: mean charge over the two panic propositions.
        let charge: f64 = if s.agents.is_empty() {
            f64::NAN
        } else {
            let mut sum = 0.0;
            let mut k = 0;
            for a in &s.agents {
                for b in a.beliefs.iter().take(2) {
                    sum += b.emotional_charge.to_f64();
                    k += 1;
                }
            }
            if k == 0 {
                f64::NAN
            } else {
                sum / k as f64
            }
        };
        if count > 0 {
            firing += 1;
        }
        n += 1;
        println!(
            "{seed:>6} {count:>8} {peak:>10.4} {charge:>10.4} {:>12}",
            if count > 0 { "YES" } else { "no" }
        );
    }
    println!("\nregistering seeds: {firing}/{n} (contract: a MAJORITY of the family)");
}
