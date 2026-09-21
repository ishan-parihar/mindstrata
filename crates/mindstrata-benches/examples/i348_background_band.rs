//! i348 — the Background entry band, sized against the live importance
//! distribution.
//!
//! i316 measured `narrative_importance` floored at **0.3000** across real runs
//! (a `+0.15` base term plus non-negative bonuses, smoothed at 0.1/tick), while
//! the Secondary→Background entry gate sits at `< 0.1` — unreachable by
//! construction, so the aggregate tier holds **0 agent-ticks** and the §17
//! scaling strategy's cheapest rung is dead (§4.3).
//!
//! i316 deliberately deferred the fix as its own behavioural iteration. This
//! probe supplies what that decision needs: the actual distribution of
//! `narrative_importance` (and of the bonuses that compose its target) across
//! contexts, post-i347, so the re-contract chooses a threshold with a measured
//! population on both sides rather than widening until green (§4.1/§4.2).
//!
//! Run: cargo run --release -p mindstrata-benches --example i348_background_band

use mindstrata_sim::sim::{SimConfig, Simulation};

const SEEDS: [u64; 3] = [1, 5, 42];

#[derive(Default)]
struct ImportanceStats {
    samples: Vec<f64>,
    /// Agent-ticks that would fall below each candidate Background gate.
    below: [u64; 5],
    tiers: [u64; 3], // Focal, Secondary, Background
}

/// Candidate entry gates to evaluate (the shipped one is 0.1).
const GATES: [f64; 5] = [0.10, 0.20, 0.22, 0.25, 0.30];

impl ImportanceStats {
    fn observe(&mut self, sim: &Simulation) {
        for a in &sim.agents {
            let v = a.agent_tier.narrative_importance.to_f64();
            self.samples.push(v);
            for (k, g) in GATES.iter().enumerate() {
                if v < *g {
                    self.below[k] += 1;
                }
            }
            match a.agent_tier.tier {
                mindstrata_sim::agent_tier::AgentTier::Focal => self.tiers[0] += 1,
                mindstrata_sim::agent_tier::AgentTier::Secondary => self.tiers[1] += 1,
                mindstrata_sim::agent_tier::AgentTier::Background => self.tiers[2] += 1,
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
        let n = self.samples.len().max(1) as f64;
        println!(
            "  {label:<20} min {:.4} p01 {:.4} p05 {:.4} p50 {:.4} max {:.4} | tiers F/S/B {}/{}/{}",
            pct(0.0),
            pct(0.01),
            pct(0.05),
            pct(0.50),
            pct(1.0),
            self.tiers[0],
            self.tiers[1],
            self.tiers[2],
        );
        let shares: Vec<String> = GATES
            .iter()
            .enumerate()
            .map(|(k, g)| format!("<{g:.2}: {:.3}%", 100.0 * self.below[k] as f64 / n))
            .collect();
        println!("      share below gate: {}", shares.join("  "));
    }
}

fn run(n: u32, ticks: u64, st: &mut ImportanceStats) {
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

fn main() {
    println!(
        "i348 — narrative_importance distribution and Background-entry candidates (3 seeds each)\n"
    );

    for (n, ticks) in [(12u32, 10_000u64), (48, 10_000)] {
        let mut st = ImportanceStats::default();
        run(n, ticks, &mut st);
        st.report(&format!("calm N={n} @{ticks}"));
    }
}
