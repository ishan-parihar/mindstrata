//! Iter-283 probe #2 — per-bucket collective stages at 50K.
//!
//! Identity predicted stage-2 at ~44K (i279: 0.458 press by 20K at growth
//! 0.05, needing 1.0). Genesis census at 50K shows 0 Identity memes — this
//! probe reads the actual stage per bucket to see whether the threshold
//! was crossed and genesis missed it, or the stage never arrived.

use mindstrata_development::collective::{bucket_for_line, CollectiveBucket, CollectiveField};
use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    let horizon = 50_000u64;
    for seed in [42_u64, 43, 44] {
        let config = SimConfig {
            seed,
            max_ticks: horizon,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(horizon);
        let slugs = CollectiveField::line_slugs();
        let mut stage = [0.0f64; 4];
        let mut press = [0.0f64; 4];
        for (i, line) in sim.collective_field.lines.iter().enumerate() {
            if i >= slugs.len() {
                break;
            }
            let b = bucket_for_line(slugs[i]) as usize;
            if line.stage > stage[b] {
                stage[b] = line.stage;
            }
            press[b] += line.press;
        }
        println!(
            "seed {seed}: Safety stage={:.2} Identity stage={:.2} Relational stage={:.2} Meaning stage={:.2} | press S={:.3} I={:.3} R={:.3} M={:.3}",
            stage[CollectiveBucket::Safety as usize],
            stage[CollectiveBucket::Identity as usize],
            stage[CollectiveBucket::Relational as usize],
            stage[CollectiveBucket::Meaning as usize],
            press[CollectiveBucket::Safety as usize],
            press[CollectiveBucket::Identity as usize],
            press[CollectiveBucket::Relational as usize],
            press[CollectiveBucket::Meaning as usize],
        );
    }
}
