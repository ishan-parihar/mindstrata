//! i392 — the puberty gene reaches the reproductive clock (and the band nobody occupies).
//!
//! i391's census found `FertilityPredispositions.puberty_age` drawn at founder creation
//! (U(11.0..15.0)), defaulted, blended at `inherit` — and read by nothing outside
//! `genome.rs`, because the reproductive clock hardcoded the constant that value was
//! supposed to replace:
//!
//! ```text
//! reproductive.rs:168   let puberty_age = Fixed::from_f64(13.0);   // gene midpoint
//! ```
//!
//! This probe measures four things:
//!   1. the founder distribution of the gene (variation really exists),
//!   2. **the age band the gene governs** — agent-ticks with age in [11, 15) over a
//!    20 000-tick corpus, which is what decides whether wiring it can move a
//!    calibrated window at all,
//!   3. the live per-agent manipulation check: three agents aged 12.0 with the gene set
//!    to the early tail (11.0), the late tail (15.0) and the retired constant (13.0),
//!    ticked, and their maturity compared; the 13.0 agent is the pre-fix control, so
//!    midpoint neutrality is *measured* rather than asserted,
//!   4. the population's age distribution, for context on (2).
//!
//! Run: `cargo run --release -p mindstrata-benches --example i392_puberty_clock`

use mindstrata_core::Fixed;
use mindstrata_sim::sim::{SimConfig, Simulation};

const WINDOW: u64 = 20_000;
/// The band the gene governs: [`puberty_age` .. maturity_age) is where the maturity ramp
/// runs, and the gene's draw spans 11.0–15.0 inside it.
const BAND_LO: f64 = 10.0;
const BAND_HI: f64 = 16.0;

fn build(seed: u64, num_agents: u32) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: WINDOW,
        world_width: 32,
        world_height: 32,
        num_agents,
        snapshot_interval: None,
    });
    sim.populate();
    sim
}

fn gene_of(sim: &Simulation, i: usize) -> f64 {
    sim.agents[i]
        .embodied
        .genome
        .fertility_predispositions
        .puberty_age
        .to_f64()
}

fn distribution(label: &str, seed: u64, num_agents: u32) {
    let sim = build(seed, num_agents);
    let genes: Vec<f64> = (0..sim.agents.len()).map(|i| gene_of(&sim, i)).collect();
    let n = genes.len() as f64;
    let mean = genes.iter().sum::<f64>() / n;
    let (min, max) = genes
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), g| (lo.min(*g), hi.max(*g)));
    let spread = max - min;
    println!(
        "  {label:<14} n={:<4} puberty_age gene: mean {mean:.3} · min {min:.3} · max {max:.3} · \
         spread {spread:.3} yr",
        sim.agents.len()
    );
}

/// Agent-ticks whose age sits inside the band the gene governs, plus the age histogram.
fn band_occupancy(label: &str, seed: u64, num_agents: u32) {
    let mut sim = build(seed, num_agents);
    let mut in_band = 0u64;
    let mut total = 0u64;
    let mut min_age = f64::MAX;
    let mut max_age = f64::MIN;
    for _ in 0..WINDOW {
        sim.run(1);
        for a in &sim.agents {
            let age = a.embodied.age.to_f64();
            total += 1;
            min_age = min_age.min(age);
            max_age = max_age.max(age);
            if (BAND_LO..BAND_HI).contains(&age) {
                in_band += 1;
            }
        }
    }
    println!(
        "  {label:<14} agent-ticks in [{BAND_LO:.0},{BAND_HI:.0}) yr: **{in_band}** / {total} \
         ({:.4}%) · age range {min_age:.1}–{max_age:.1} yr",
        in_band as f64 / total.max(1) as f64 * 100.0
    );
}

/// The manipulation check: three 12-year-olds, three genes, one tick.
fn manipulation_check() {
    let mut sim = build(42, 12);
    // 11.0 = early tail, 15.0 = late tail, 13.0 = the retired constant (pre-fix control).
    let arms = [
        ("early gene", 11.0_f64),
        ("late gene", 15.0),
        ("control 13.0", 13.0),
    ];
    for (i, (_, gene)) in arms.iter().enumerate() {
        sim.agents[i]
            .embodied
            .genome
            .fertility_predispositions
            .puberty_age = Fixed::from_f64(*gene);
        sim.agents[i].embodied.age = Fixed::from_f64(12.0);
    }
    // Let the biology pass reach them (it is cadenced), then read the clock's verdict.
    sim.run(4);
    println!("  the manipulation check — all three agents aged 12.0:");
    for (i, (label, gene)) in arms.iter().enumerate() {
        let r = &sim.agents[i].embodied.reproductive;
        println!(
            "    {label:<14} gene {gene:.1} → stage {:?} · sexual_maturity {:.4}",
            r.puberty_stage,
            r.sexual_maturity.to_f64()
        );
    }
}

fn main() {
    println!("i392 — puberty gene → reproductive clock (window {WINDOW} ticks)\n");
    println!("founder gene distribution:");
    distribution("village s42", 42, 12);
    distribution("village s7", 7, 12);
    distribution("town s42", 42, 48);
    println!();

    println!("the band the gene governs:");
    band_occupancy("village s42", 42, 12);
    band_occupancy("town s42", 42, 48);
    println!();

    manipulation_check();
    println!();
    println!(
        "note: TICKS_PER_YEAR = 35 040, so a child born at t=0 enters the band at\n\
         ~385 000–525 000 ticks (11–15 yr) — beyond every calibrated corpus (the goldens\n\
         are 1 000 ticks, the long-horizon suite 50 000). The wiring is therefore\n\
         invisible in all of them by construction, and its effect is a\n\
         *long-horizon* one: the next generation's reproductive onset follows its own\n\
         inherited gene instead of a village-wide constant."
    );
}
