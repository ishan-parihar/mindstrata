//! i314 — pain-veto threshold probe.
//!
//! i312 established that the health-critical veto (`health < 0.25`) is dormant:
//! the derived-health distributions overlap across contexts, so no absolute
//! health threshold can separate crisis from calm. i313 made the pain channel
//! live, which gives a signal that is *structurally* crisis-only (a wound).
//! This probe measures the effective-pain distribution per context and the
//! firing rate of candidate thresholds, to pick a reachable veto band.
//!
//! Run: cargo run --release -p mindstrata-benches --example i314_pain_veto

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::{SimConfig, Simulation};

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];

#[derive(Default)]
struct PainStats {
    agent_ticks: u64,
    ab: [u64; 3], // firing counts for thresholds 0.5 / 0.7 / 0.9
    samples: Vec<f64>,
}

impl PainStats {
    fn observe(&mut self, sim: &Simulation, tick: u64) {
        for a in sim.agents.iter() {
            let p = a.embodied.nervous.pain.effective_pain().to_f64();
            self.agent_ticks += 1;
            if tick % 20 == 0 {
                self.samples.push(p);
            }
            for (i, t) in [0.5, 0.7, 0.9].iter().enumerate() {
                if p >= *t {
                    self.ab[i] += 1;
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
            "  {label:<16} ticks {:<8} p50 {:.4} p90 {:.4} p99 {:.4} max {:.4} | \
             >=0.5 {:.4}% | >=0.7 {:.4}% | >=0.9 {:.4}%",
            self.agent_ticks,
            pct(0.50),
            pct(0.90),
            pct(0.99),
            pct(1.0),
            100.0 * self.ab[0] as f64 / n,
            100.0 * self.ab[1] as f64 / n,
            100.0 * self.ab[2] as f64 / n,
        );
    }
}

fn main() {
    println!("i314 pain-veto threshold probe");
    let scenarios: [(&str, Scenario); 3] = [
        ("pestilence", Scenario::pestilence()),
        ("collapse", Scenario::collapse()),
        ("drought", Scenario::drought()),
    ];
    for (label, sc) in scenarios {
        for ticks in [4_320u64, 20_000] {
            let mut st = PainStats::default();
            for &seed in &SEEDS {
                let mut s = sc.clone();
                s.seed = seed;
                s.ticks = ticks;
                let mut sim = Simulation::from_scenario(s);
                sim.populate();
                for tick in 0..ticks {
                    sim.tick();
                    st.observe(&sim, tick);
                }
            }
            st.report(&format!("{label} @{ticks}"));
        }
    }
    for ticks in [20_000u64, 50_000] {
        let mut st = PainStats::default();
        for &seed in &SEEDS {
            let mut sim = Simulation::new(SimConfig {
                seed,
                max_ticks: ticks,
                num_agents: 12,
                snapshot_interval: None,
                ..SimConfig::default()
            });
            sim.populate();
            for tick in 0..ticks {
                sim.tick();
                st.observe(&sim, tick);
            }
        }
        st.report(&format!("calm family @{ticks}"));
    }
}
