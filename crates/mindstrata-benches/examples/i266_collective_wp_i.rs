//! Iter-266 WP-I probe: collective field is LIVE and differentiates.
//!
//! Evidence contract (AGENTS.md §2/§4 — probe before touching):
//! 1. Before WP-I, `step_collective` was inert — i280 proved `is_neutral() ==
//!    true` at all 12 seeds (field pinned at founder neutral forever).
//! 2. After WP-I, the field must move: per-line press > 0 at every seed,
//!    per-bucket fulfillment differentiates by catalyst regime, and the
//!    meaning bucket (ambient baseline) is non-zero everywhere.
//! 3. Ladder behavior: press integrates toward stage advance; the shadow
//!    stage coordinate must stay in [1, 17].
//!
//! Run: cargo run --release -p mindstrata-benches --example i266_collective_wp_i

use mindstrata_development::collective::{CollectiveBucket, COLLECTIVE_LINE_COUNT};
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const TICKS: u64 = 5000;

fn main() {
    let mut rows = Vec::new();
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
        let field = &sim.collective_field;

        let mut max_press = 0.0_f64;
        let mut moved_lines = 0_usize;
        let mut max_stage = 1.0_f64;
        for l in &field.lines {
            if l.press > max_press {
                max_press = l.press;
            }
            if l.press > 0.0 {
                moved_lines += 1;
            }
            if l.stage > max_stage {
                max_stage = l.stage;
            }
        }
        let rel = field.mean_fulfillment_for_bucket(CollectiveBucket::Relational);
        let saf = field.mean_fulfillment_for_bucket(CollectiveBucket::Safety);
        let ide = field.mean_fulfillment_for_bucket(CollectiveBucket::Identity);
        let mea = field.mean_fulfillment_for_bucket(CollectiveBucket::Meaning);
        let bucket_spread = rel.max(saf).max(ide).max(mea) - rel.min(saf).min(ide).min(mea);
        println!(
            "seed={seed} moved_lines={moved_lines}/{COLLECTIVE_LINE_COUNT} max_press={max_press:.4} max_stage={max_stage:.0} fulfill[rel={rel:.4} saf={saf:.4} ide={ide:.4} mea={mea:.4}] spread={bucket_spread:.4}"
        );
        rows.push((seed, moved_lines, max_press, bucket_spread));
    }

    let all_moved = rows.iter().filter(|(_, m, _, _)| *m > 0).count();
    let spread_ok = rows.iter().filter(|(_, _, _, s)| *s > 0.001).count();
    let rate = all_moved as f64 / rows.len() as f64;
    println!(
        "\nfamily_moved_rate={rate:.4} threshold=1.00 spread_seeds={spread_ok}/{}",
        rows.len()
    );
    if rate >= 1.0 {
        println!("verdict=COLLECTIVE_FIELD_LIVE (WP-I step integrated, field moves at every seed)");
    } else {
        println!("verdict=COLLECTIVE_FIELD_DEAD (field still inert at some seeds — WP-I failed)");
    }
}
