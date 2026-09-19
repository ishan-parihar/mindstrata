//! i316 — tier/scaling audit.
//!
//! §17 is the project's scalability strategy: Focal / Secondary / Background
//! tiers, each running less. `runs_full_biology()` and `runs_action_selection()`
//! have ZERO production call sites, and `Background` needs
//! `narrative_importance < 0.1` — but importance converges ~0.45–0.65 for
//! everyone. This probe measures the real tier census and the per-tick cost
//! curve, to see whether the strategy is actually scaling the sim.
//!
//! Run: cargo run --release -p mindstrata-benches --example i316_tier_scaling

use mindstrata_sim::agent_tier::AgentTier;
use mindstrata_sim::sim::{SimConfig, Simulation};
use std::time::Instant;

fn main() {
    println!("i316 tier/scaling audit");
    for n in [12u32, 24, 48, 96] {
        for ticks in [500u64] {
            let cfg = SimConfig {
                seed: 42,
                max_ticks: ticks,
                num_agents: n,
                snapshot_interval: None,
                ..SimConfig::default()
            };
            let mut sim = Simulation::new(cfg);
            sim.populate();
            let start = Instant::now();
            for _ in 0..ticks {
                sim.tick();
            }
            let elapsed = start.elapsed();
            let mut focal = 0u32;
            let mut secondary = 0u32;
            let mut background = 0u32;
            for a in sim.agents.iter() {
                match a.agent_tier.tier {
                    AgentTier::Focal => focal += 1,
                    AgentTier::Secondary => secondary += 1,
                    AgentTier::Background => background += 1,
                }
            }
            println!(
                "  N={n:<4} ticks={ticks:<6} agents_final={:<4} | Focal {focal:<3} Secondary {secondary:<3} Background {background:<3} | {:.2} ms/tick ({:.2}s total)",
                sim.agents.len(),
                elapsed.as_secs_f64() * 1000.0 / ticks as f64,
                elapsed.as_secs_f64(),
            );
        }
    }
    // Does ANY agent ever reach Background over a long run?
    println!("  -- ever-Background census (seed 42, 10K) --");
    for n in [12u32, 48] {
        let cfg = SimConfig {
            seed: 42,
            max_ticks: 10_000,
            num_agents: n,
            snapshot_interval: None,
            ..SimConfig::default()
        };
        let mut sim = Simulation::new(cfg);
        sim.populate();
        let mut ever_background = 0u64;
        let mut min_importance = f64::INFINITY;
        for _ in 0..10_000 {
            sim.tick();
            for a in sim.agents.iter() {
                if a.agent_tier.tier == AgentTier::Background {
                    ever_background += 1;
                }
                min_importance = min_importance.min(a.agent_tier.narrative_importance.to_f64());
            }
        }
        println!(
            "  N={n:<4} background agent-ticks {ever_background} | min narrative importance over run {min_importance:.4}"
        );
    }
}
