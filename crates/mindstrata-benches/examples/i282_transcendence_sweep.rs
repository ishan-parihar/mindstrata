//! Iter-282 probe — needs-band transcendence sweep (needs-bands.md v1,
//! the last CALIBRATION-PENDING row; deferred "until CollectiveField
//! lands" — it landed at i266).
//!
//! The self-transcendence band is the autonomy need: decay 2/3 the
//! meaning rate (i267 revive), Work/Trade relief 0.0002/tick (i267),
//! theory citation "needs rung 7 collective field (Era II village holon),
//! CollectiveField lambda", target range 0.55–0.65 (derived).
//!
//! Questions:
//! 1. Where does the natural autonomy-deficit equilibrium sit vs the
//!    [0.55, 0.65] target band? (median + IQR across the population)
//! 2. Is the band LIVE (variance across agents/seeds, not σ=0 like the
//!    pre-i267 dead decay)?
//! 3. WP-I dependence: the theory says the CollectiveField modulates this
//!    band — nothing does today. Does the natural equilibrium NEED the
//!    collective channel to sit in-band, or is work-paced relief already
//!    there? (If in-band: promote; if out: sweep the collective channel.)

use mindstrata_sim::sim::{SimConfig, Simulation};

fn stats(v: &mut [f64]) -> (f64, f64, f64) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    let median = v[n / 2];
    let q1 = v[n / 4];
    let q3 = v[(3 * n) / 4];
    (median, q1, q3)
}

fn main() {
    for horizon in [1_000_i64, 5_000, 20_000] {
        let mut med_all = Vec::new();
        let mut gov_stage = 0.0f64;
        for seed in 42..45u64 {
            let config = SimConfig {
                seed,
                max_ticks: horizon as u64,
                world_width: 16,
                world_height: 16,
                num_agents: 12,
                snapshot_interval: None,
            };
            let mut sim = Simulation::new(config);
            sim.populate();
            sim.run(horizon as u64);
            let mut defs = Vec::new();
            for a in &sim.agents {
                defs.push(a.needs.autonomy.to_f64());
            }
            let (med, q1, q3) = stats(&mut defs);
            println!(
                "t={horizon:>6} seed={seed}  autonomy_deficit median={med:.4} IQR=[{q1:.4},{q3:.4}] spread={:.4}",
                q3 - q1
            );
            med_all.push(med);
            gov_stage = sim
                .collective_field
                .lines
                .iter()
                .zip(mindstrata_development::collective::CollectiveField::line_slugs())
                .find(|(_, s)| s.slug() == "governance")
                .map_or(0.0, |(l, _)| l.stage);
        }
        let mean_med = med_all.iter().sum::<f64>() / med_all.len() as f64;
        println!(
            "t={horizon:>6} MEAN median={mean_med:.4}  governance_stage={gov_stage:.1}  target band [0.55,0.65]  verdict={}",
            if (0.55..=0.65).contains(&mean_med) { "IN BAND" } else { "OUT OF BAND" }
        );
    }
}
