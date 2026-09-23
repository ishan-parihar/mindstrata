//! i367 — is the LOD tier live at town scale, and where does it sit?
//!
//! i359 recorded an unmeasured follow-up: "the LOD tier carrying Background
//! agents was not exercised — i348 measured Background at 25–40% of *crisis*-world
//! agent-ticks, and the calm town runs sit below the importance gate, so the LOD
//! saving at town scale is unmeasured." This probe measures the tier distribution
//! as a share of agent-ticks across population sizes, on both a **calm** village
//! and the **collapse** crisis, so the LOD's reach at scale is known rather than
//! assumed.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i367_lod_town_scale`

use mindstrata_sim::agent_tier::AgentTier;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::{SimConfig, Simulation};

// i392: the i344 density world law is single-sourced in the world generator
// (`world_side_for_population`) — do not re-implement it.
use mindstrata_sim::world_gen::world_side_for_population as density_side;

/// Share of agent-ticks spent in each tier over `ticks`.
fn tier_shares(mut sim: Simulation, ticks: u64) -> (f64, f64, f64) {
    let mut focal = 0u64;
    let mut secondary = 0u64;
    let mut background = 0u64;
    for _ in 0..ticks {
        sim.tick();
        for a in &sim.agents {
            match a.agent_tier.tier {
                AgentTier::Focal => focal += 1,
                AgentTier::Secondary => secondary += 1,
                AgentTier::Background => background += 1,
            }
        }
    }
    let total = (focal + secondary + background).max(1) as f64;
    (
        100.0 * focal as f64 / total,
        100.0 * secondary as f64 / total,
        100.0 * background as f64 / total,
    )
}

fn calm(n: u32, seed: u64, ticks: u64) -> Simulation {
    let side = density_side(n);
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
    println!("i367 — LOD tier distribution (share of agent-ticks %)\n");

    println!("CALM village, 10K ticks, seeds 42/7:");
    println!(
        "{:>5} {:>5} {:>8} {:>8} {:>10}",
        "N", "seed", "Focal%", "Second%", "Background%"
    );
    for n in [12u32, 48, 96, 192, 256] {
        for seed in [42u64, 7] {
            let (f, s, b) = tier_shares(calm(n, seed, 10_000), 10_000);
            println!("{n:>5} {seed:>5} {f:>8.1} {s:>8.1} {b:>10.1}");
        }
    }

    println!("\nCOLLAPSE crisis (full 4320-tick cascade), density variants:");
    println!(
        "{:>5} {:>8} {:>8} {:>10}",
        "N", "Focal%", "Second%", "Background%"
    );
    for n in [12u32, 48, 96] {
        let mut sc = Scenario::collapse();
        let side = density_side(n);
        sc.num_agents = n;
        sc.world_width = side;
        sc.world_height = side;
        let ticks = sc.ticks;
        let mut sim = Simulation::from_scenario(sc);
        sim.populate();
        let (f, sec, b) = tier_shares(sim, ticks);
        println!("{n:>5} {f:>8.1} {sec:>8.1} {b:>10.1}");
    }
}
