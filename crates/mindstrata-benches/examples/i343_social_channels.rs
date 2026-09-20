//! i343 — the two rewired social-connection channels, measured pre/post.
//!
//! i342 surveyed the bug (both channels read `relationship_v2s.len()`, the
//! complete graph's N−1 for every agent, so `min(len,4)==4` for everyone and the
//! channels were constants). This probe produces the re-anchor evidence for
//! wiring the contacted degree in: the distribution of the two derived
//! quantities, the share of agents that actually move, and the anxiety channel's
//! new isolation input.
//!
//! The pre-fix values are computed inline from the list length (exactly what the
//! sim used to do), so one run yields both columns — no HEAD swap needed.
//!
//! Run: cargo run --release -p mindstrata-benches --example i343_social_channels -- [N...]

use mindstrata_sim::sim::{SimConfig, Simulation};

fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

/// Exactly the two post-fix expressions, in f64 (the sim's `Fixed` at these
/// magnitudes quantizes below the printed precision).
fn visibility(partnered: bool, degree: usize) -> f64 {
    let term = (degree as f64).min(4.0) * 0.1;
    if partnered {
        clamp01(0.5 + term)
    } else {
        clamp01(term)
    }
}

fn anxiety_social_factor(degree: usize) -> f64 {
    (4.0 - (degree as f64).min(4.0)) * 0.002
}

fn main() {
    let months: Vec<u32> = {
        let args: Vec<u32> = std::env::args()
            .skip(1)
            .filter_map(|s| s.parse().ok())
            .collect();
        if args.is_empty() {
            vec![12, 48, 96]
        } else {
            args
        }
    };
    for ticks in [2_000u64, 20_000] {
        println!("=== {ticks} ticks (seed 42, 32x32) ===\n");
        println!(
            "{:>5} {:>10} {:>10} {:>12} {:>18} {:>18} {:>16} {:>16}",
            "N",
            "deg mean",
            "deg max",
            "isolates(<4)",
            "vis PRE min/mean",
            "vis POST min/mean",
            "anx PRE max",
            "anx POST max"
        );
        for n in &months {
            let n = *n;
            let mut sim = Simulation::new(SimConfig {
                seed: 42,
                max_ticks: ticks,
                world_width: 32,
                world_height: 32,
                num_agents: n,
                snapshot_interval: None,
            });
            sim.populate();
            sim.run(ticks);

            let lengths: Vec<usize> = sim
                .agents
                .iter()
                .map(|a| a.relationship_v2s.len())
                .collect();
            let degrees: Vec<usize> = sim
                .contacted_degrees()
                .iter()
                .map(|d| *d as usize)
                .collect();
            let partnered: Vec<bool> = sim.agents.iter().map(|a| a.partner.is_some()).collect();

            let pre: Vec<f64> = (0..lengths.len())
                .map(|i| visibility(partnered[i], lengths[i]))
                .collect();
            let post: Vec<f64> = (0..lengths.len())
                .map(|i| visibility(partnered[i], degrees[i]))
                .collect();
            let anx_pre = lengths
                .iter()
                .map(|l| anxiety_social_factor(*l))
                .fold(0.0f64, f64::max);
            let anx_post = degrees
                .iter()
                .map(|d| anxiety_social_factor(*d))
                .fold(0.0f64, f64::max);

            let mean = |v: &Vec<f64>| v.iter().sum::<f64>() / v.len().max(1) as f64;
            let min = |v: &Vec<f64>| v.iter().copied().fold(f64::INFINITY, f64::min);
            let mean_deg = degrees.iter().sum::<usize>() as f64 / degrees.len().max(1) as f64;
            let max_deg = degrees.iter().copied().max().unwrap_or(0);
            let isolates = degrees.iter().filter(|d| **d < 4).count();

            println!(
                "{:>5} {:>10.1} {:>10} {:>12} {:>8.3}/{:<7.3} {:>8.3}/{:<7.3} {:>16.3} {:>16.3}",
                n,
                mean_deg,
                max_deg,
                format!("{isolates}/{}", degrees.len()),
                min(&pre),
                mean(&pre),
                min(&post),
                mean(&post),
                anx_pre,
                anx_post
            );
        }
        println!();
    }
    println!(
        "reading: PRE the visibility column cannot leave its partner-status constant and the anxiety\n\
         term is exactly 0.000 for everyone. POST both move only for agents below 4 contacts —\n\
         isolates gain a real isolation signal (visibility down, anxiety up) while connected agents\n\
         keep the calibrated value, which is why the golden/nightly budget shifts at all."
    );
}
