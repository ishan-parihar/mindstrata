//! Needs-bands calibration sweep (DC-1 DESIGN 8-9).
//!
//! Per `docs/balance/needs-bands.md` (RATIFIED via IC-5), 19 MotiveCategory
//! needs are banded by salience (deficit → urgency → activation) across
//! the 12-seed family at 2000 ticks. Prints the equilibrium band for
//! each need — this is the CO-2026-001 baseline measurement DESIGN 8-9
//! requires. `i268` already measures headline aggregates; this probe
//! measures the per-need decomposition.
//!
//! Run: cargo run --release -p mindstrata-benches --example i277_needs_calibration_sweep

use mindstrata_sim::psychology::MotiveCategory;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const TICKS: u64 = 2000;

fn avg_deficit(agents: &[mindstrata_sim::sim::AgentBundle], cat: MotiveCategory) -> f64 {
    let n = agents.len() as f64;
    agents
        .iter()
        .map(|a| a.motivation.need(cat).deficit.to_f64())
        .sum::<f64>()
        / n
}

fn main() {
    for &seed in &SEEDS {
        let config = SimConfig {
            seed,
            max_ticks: TICKS,
            num_agents: 12,
            snapshot_interval: None,
            ..SimConfig::default()
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(TICKS);

        for cat in [
            MotiveCategory::Hunger,
            MotiveCategory::Thirst,
            MotiveCategory::Sleep,
            MotiveCategory::Warmth,
            MotiveCategory::Health,
            MotiveCategory::Safety,
            MotiveCategory::Attachment,
            MotiveCategory::Belonging,
            MotiveCategory::Esteem,
            MotiveCategory::Autonomy,
            MotiveCategory::Competence,
            MotiveCategory::Meaning,
            MotiveCategory::Certainty,
            MotiveCategory::Novelty,
            MotiveCategory::Play,
            MotiveCategory::Care,
            MotiveCategory::Romance,
            MotiveCategory::Justice,
            MotiveCategory::Recognition,
        ] {
            let v = avg_deficit(&sim.agents, cat);
            let band = if v < 0.2 {
                "low"
            } else if v < 0.5 {
                "mid"
            } else if v < 0.8 {
                "high"
            } else {
                "sat"
            };
            println!("seed={seed} need={:?} value={v:.4} band={band}", cat);
        }
    }
}
