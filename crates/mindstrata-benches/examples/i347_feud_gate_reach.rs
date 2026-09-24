//! i347 — the anger channel's reachable band, and what a reachable §19.5.G gate costs.
//!
//! i346 measured the §19.5.G feud-approach branch (the sim's **only** `Move`
//! producer) firing **0** times in 96 000 agent-ticks: it needs
//! `feuds non-empty && anger > 0.4 && hunger < 0.85 && thirst < 0.85`, and the
//! anger channel's reachable maximum is 0.2786 (N=12) / 0.4095 (N=48) — the gate
//! sits above the band at small N and exactly on its edge at N=48.
//!
//! Re-contracting that gate needs two numbers, not a guess: the **band** (so the
//! threshold is inside it) and the **firing share** at each candidate (so the
//! branch becomes live without becoming a movement governor). This probe
//! measures both, across calm and crisis contexts and several seeds.
//!
//! Run: cargo run --release -p mindstrata-benches --example i347_feud_gate_reach

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::{SimConfig, Simulation};

const SEEDS: [u64; 3] = [1, 5, 42];
/// Thresholds to evaluate, spanning the reachable band below the old 0.4.
const THRESHOLDS: [f64; 8] = [0.02, 0.05, 0.075, 0.10, 0.15, 0.20, 0.30, 0.40];

#[derive(Default)]
struct AngerStats {
    samples: Vec<f64>,
    anger_above: [u64; THRESHOLDS.len()],
    branch_above: [u64; THRESHOLDS.len()],
    /// The §19.5.G branch in full, including its `hunger < 0.85` and
    /// `thirst < 0.85` clauses.
    full_above: [u64; THRESHOLDS.len()],
    feud_ticks: u64,
    agent_ticks: u64,
}

impl AngerStats {
    fn observe(&mut self, sim: &Simulation) {
        for a in &sim.agents {
            let anger = a.emotions.anger.to_f64();
            self.samples.push(anger);
            self.agent_ticks += 1;
            let feuding = !a.feuds.is_empty();
            if feuding {
                self.feud_ticks += 1;
            }
            for (k, t) in THRESHOLDS.iter().enumerate() {
                if anger > *t {
                    self.anger_above[k] += 1;
                }
                if feuding && anger > *t {
                    // The §19.5.G branch, minus the hunger/thirst clauses (which
                    // the census reports in vivo after the fix).
                    self.branch_above[k] += 1;
                    if a.needs.hunger.to_f64() < 0.85 && a.needs.thirst.to_f64() < 0.85 {
                        self.full_above[k] += 1;
                    }
                }
            }
        }
    }

    fn report(&mut self, label: &str) {
        self.samples.sort_by(f64::total_cmp);
        let pct = |q: f64| -> f64 {
            if self.samples.is_empty() {
                return f64::NAN;
            }
            self.samples[((self.samples.len() - 1) as f64 * q).round() as usize]
        };
        let n = self.agent_ticks.max(1) as f64;
        println!(
            "  {label:<22} p50 {:.4} p90 {:.4} p99 {:.4} p99.9 {:.4} max {:.4} | feuds {:.2}%",
            pct(0.50),
            pct(0.90),
            pct(0.99),
            pct(0.999),
            pct(1.0),
            100.0 * self.feud_ticks as f64 / n,
        );
        let shares: Vec<String> = THRESHOLDS
            .iter()
            .enumerate()
            .map(|(k, t)| {
                format!(
                    ">{t:.3}: {:.3}% / {:.3}% / {:.4}%",
                    100.0 * self.anger_above[k] as f64 / n,
                    100.0 * self.branch_above[k] as f64 / n,
                    100.0 * self.full_above[k] as f64 / n
                )
            })
            .collect();
        println!(
            "      anger-above / feud∧anger / FULL-§19.5.G shares: {}",
            shares.join("  ")
        );
    }
}

fn calm(n: u32, ticks: u64, st: &mut AngerStats) {
    for seed in SEEDS {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: ticks,
            world_width: 32,
            world_height: 32,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();
        for _ in 0..ticks {
            sim.tick();
            st.observe(&sim);
        }
    }
}

/// Per-seed version: a threshold whose liveness lives in one seed's tail is a
/// lucky-seed pin (§4.1), so the candidate has to fire across the family.
fn calm_per_seed(n: u32, ticks: u64) {
    for seed in SEEDS {
        let mut st = AngerStats::default();
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: ticks,
            world_width: 32,
            world_height: 32,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();
        for _ in 0..ticks {
            sim.tick();
            st.observe(&sim);
        }
        st.report(&format!("  seed {seed} calm N={n}"));
    }
}

fn crisis(label: &str, scenario: &Scenario, ticks: u64, st: &mut AngerStats) {
    for seed in SEEDS {
        let mut sc = scenario.clone();
        sc.seed = seed;
        sc.ticks = ticks;
        let mut sim = Simulation::from_scenario(sc);
        sim.populate();
        for _ in 0..ticks {
            sim.tick();
            st.observe(&sim);
        }
    }
    let _ = label;
}

fn main() {
    println!("i347 — anger reachable band and §19.5.G gate candidates (3 seeds each)\n");

    for (n, ticks) in [(12u32, 20_000u64), (48, 20_000)] {
        let mut st = AngerStats::default();
        calm(n, ticks, &mut st);
        st.report(&format!("calm N={n} @{ticks}"));
        calm_per_seed(n, ticks);
    }

    for (label, sc) in [
        ("pestilence", Scenario::pestilence()),
        ("collapse", Scenario::collapse()),
        ("drought", Scenario::drought()),
    ] {
        let mut st = AngerStats::default();
        crisis(label, &sc, 4_320, &mut st);
        st.report(&format!("{label} N=12 @4320"));
    }
}
