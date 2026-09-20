//! i323 — does the per-tick clan-enmity decay pass have a cost profile worth
//! fixing?
//!
//! `Simulation::decay_clan_enmities` runs **every tick** (it is called
//! unconditionally from `tick_social_cluster`) and is O(C² · N · feuds): for
//! each clan *pair* it scans **every agent** to ask "does any member of clan i
//! hold an active feud with a member of clan j?". If the clan registry grows
//! with horizon (feuds forge enmities, enmities heal into alliances, later
//! feuds re-forge), that scan is the i294/i320 accident class — a per-agent
//! scan inside a pair loop — and would first bite once C is more than a handful.
//!
//! This probe measures the precondition before anything is touched (§2):
//! clan-registry size and enmity count over horizons, per seed, plus the share
//! of ticks that even have ≥2 clans (below that, the pair loop is empty and the
//! pass is free).
//!
//! Run: cargo run --release -p mindstrata-benches --example i323_clan_enmity_scan

use mindstrata_sim::sim::{SimConfig, Simulation};

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const HORIZONS: [u64; 4] = [2_000, 20_000, 50_000, 100_000];

fn main() {
    println!("i323 — clan-registry growth vs the per-tick enmity-decay scan (N=12)");

    let mut max_c_ever = 0usize;
    let mut any_enmity = false;
    for ticks in HORIZONS {
        let mut max_c = 0usize;
        let mut sum_c = 0usize;
        let mut samples = 0usize;
        let mut clanned_ticks = 0usize;
        let mut enmity_total = 0usize;
        for &seed in &SEEDS {
            let mut sim = Simulation::new(SimConfig {
                seed,
                max_ticks: ticks,
                num_agents: 12,
                snapshot_interval: None,
                ..SimConfig::default()
            });
            sim.populate();
            // Sample every 100 ticks so we see the trajectory, not just the end.
            let mut done = 0u64;
            while done < ticks {
                let step = 100.min(ticks - done);
                sim.run(step);
                done += step;
                let c = sim.clan_registry.clans.len();
                max_c = max_c.max(c);
                sum_c += c;
                samples += 1;
                if c >= 2 {
                    clanned_ticks += 1;
                }
                enmity_total += sim
                    .clan_registry
                    .clans
                    .iter()
                    .map(|cl| cl.enemies.len())
                    .sum::<usize>();
            }
        }
        let mean_c = sum_c as f64 / samples.max(1) as f64;
        let share = 100.0 * clanned_ticks as f64 / samples.max(1) as f64;
        println!(
            "  @{ticks:>6}: max clans={max_c}  mean clans={mean_c:.2}  \
             ticks with >=2 clans={share:.0}%  enmity observations={enmity_total}"
        );
        max_c_ever = max_c_ever.max(max_c);
        any_enmity |= enmity_total > 0;
    }

    println!();
    println!("  max clan count ever observed: {max_c_ever}");
    println!("  any enmity observed: {any_enmity}");
    println!(
        "verdict={}",
        if max_c_ever >= 4 {
            "CLAN_SCAN_MATERIAL"
        } else {
            "CLAN_SCAN_IMMATERIAL"
        }
    );
}
