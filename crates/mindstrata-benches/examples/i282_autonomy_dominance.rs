//! Iter-282 probe #2 — autonomy pressure dominance (the band's behavioral
//! expression). needs-bands.md self-transcendence row: target 0.55–0.65
//! (derived), theory "needs rung 7 collective field, CollectiveField
//! lambda". The other rows' semantics are GOAL-GATE thresholds (0.5
//! Eat/Drink, 0.7 Socialize/Worship generate) — pressure levels at which
//! the need drives behavior. So the honest sweep measures: where does
//! autonomy PRESSURE sit, and does it ever reach the [0.55, 0.65]
//! dominance window (i.e., become the argmax dominant_need)?

use mindstrata_sim::psychology::MotiveCategory;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn pressure(a: &mindstrata_sim::sim::AgentBundle, n: MotiveCategory) -> f64 {
    a.motivation.pressure_full(n).to_f64()
}

fn main() {
    for horizon in [1_000_i64, 5_000, 20_000] {
        let mut all = Vec::new();
        let mut dominant_count = 0usize;
        let mut in_band = 0usize;
        let mut total = 0usize;
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
            for a in &sim.agents {
                let p = pressure(a, MotiveCategory::Autonomy);
                all.push(p);
                total += 1;
                if (0.55..=0.65).contains(&p) {
                    in_band += 1;
                }
                if a.motivation.dominant_need == MotiveCategory::Autonomy {
                    dominant_count += 1;
                }
            }
        }
        all.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = all.len();
        println!(
            "t={horizon:>6}  autonomy_pressure p10={:.3} median={:.3} p90={:.3} max={:.3}  in_band[0.55,0.65]={in_band}/{total}  dominant_argmax={dominant_count}/{total}",
            all[n / 10],
            all[n / 2],
            all[(9 * n) / 10],
            all[n - 1],
        );
    }
}
