//! i429 — affection write-side saturation probe.
//!
//! The i402/i403/i427 arc identified the kind schedule's `affection > 0.70`
//! branch as the §4.10 defect (fires 98–99% of interactions on both stores),
//! and §4.16(a) rules out re-anchoring the threshold against the saturated
//! input: the input itself must be measured. Code-verified write-side law
//! (`relationship_v2.rs`): prior 0.3; gain `+0.015×vol/act`; decay
//! `decay_rate × ticks × 0.5` (0.0002 → ~0.0144/day) on dirty rows only.
//!
//! Pre-probe arithmetic: a pair touching 1 act/day nets +0.0006/day (creeps
//! to the ceiling in years); a pair touching 3+ acts/day nets ~+0.03/day and
//! reaches the ceiling in ~23 days (≈3 312 ticks). The question the probe
//! answers: in which living worlds does the store lose discrimination, and
//! when — the repair must preserve the unsaturated band.
//!
//! Output per (scenario, seed): at each 1440-tick bucket, over CONTACTED
//! dyadic pairs (interaction_count > 0): count, p10/p50/p90, the share at
//! the ceiling (≥ 0.999), and the share in the discriminating band
//! (0.5–0.95). Instruments the v2 store ONLY (it is the kind schedule's
//! future read target; v1 was retired by i428's writer deletion).
//!
//! Run: cargo run --release -p mindstrata-benches --example i429_affection_saturation
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::{SimConfig, Simulation};

const TICKS: u64 = 20_000;
const SAMPLE_EVERY: u64 = 1_440;

fn sample(sim: &Simulation, label: &str, t: u64) {
    let mut vals: Vec<f64> = Vec::new();
    for i in 0..sim.agents.len() {
        for j in 0..sim.agents.len() {
            if i == j {
                continue;
            }
            if let Some(rv2) = sim.relationship_v2_between(i, j) {
                if rv2.interaction_count > 0 {
                    vals.push(rv2.affection.to_f64());
                }
            }
        }
    }
    if vals.is_empty() {
        println!("{label} t={t:>6}: no contacted pairs");
        return;
    }
    vals.sort_unstable_by(f64::total_cmp);
    let pct = |q: f64| vals[((vals.len() - 1) as f64 * q) as usize];
    let ceiling = vals.iter().filter(|&&v| v >= 0.999).count() as f64 / vals.len() as f64;
    let band =
        vals.iter().filter(|&&v| (0.5..0.95).contains(&v)).count() as f64 / vals.len() as f64;
    println!(
        "{label} t={t:>6}: n={:>3} p10={:.3} p50={:.3} p90={:.3} ceiling={:.1}% band[0.5,0.95)={:.1}%",
        vals.len(),
        pct(0.1),
        pct(0.5),
        pct(0.9),
        ceiling * 100.0,
        band * 100.0,
    );
}

fn main() {
    println!("i429 affection write-side saturation (sampled every {SAMPLE_EVERY} ticks):");
    for &seed in &[42u64, 7, 11] {
        let config = SimConfig {
            seed,
            max_ticks: TICKS,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        for t in 1..=TICKS {
            if t % SAMPLE_EVERY == 0 {
                sample(&sim, &format!(" calm/{seed:>5}"), t);
            }
            sim.tick();
        }
    }
    // Crisis arm: pestilence seed 42 (the revolution/panic corpus).
    let mut sc = Scenario::pestilence();
    sc.seed = 42;
    sc.ticks = TICKS;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    for t in 1..=TICKS {
        if t % SAMPLE_EVERY == 0 {
            sample(&sim, " crisis/   42", t);
        }
        sim.tick();
    }
}
