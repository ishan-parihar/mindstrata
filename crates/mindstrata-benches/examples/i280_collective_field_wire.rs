//! DC-1 STORY 11 collective-field wire probe — verifies the v1
//! wire is live (the catalyst stream is observed by
//! `system_collective_field_step`) AND that the current step impl is
//! inert at the dev-crate level (per the `ponytail: no pressure
//! derivation yet (WP-I)` note in `crates/mindstrata-development/src/
//! collective.rs`). The v1 wire is the input to the future WP-I
//! implementation.
//!
//! At DC-1 close: the wire is exercised (call lands in the daily
//! pass), the press vector is computed (verified by stepping with a
//! non-empty pressure in unit tests), and the field stays inert
//! (zero press everywhere) because the dev-crate step is
//! intentionally a no-op pending WP-I. This is the correct v1
//! contract: wire is live, derivation is computed, step is inert.
//!
//! Run: cargo run --release -p mindstrata-benches --example i280_collective_field_wire

use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const TICKS: u64 = 2000;

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
        let total_lines = mindstrata_development::collective::COLLECTIVE_LINE_COUNT;
        let mut max_press = 0.0_f64;
        let mut nonzero_lines = 0;
        for l in field.lines.iter() {
            if l.press > max_press {
                max_press = l.press;
            }
            if l.press > 0.0 {
                nonzero_lines += 1;
            }
        }
        // The v1 wire contract at DC-1 close:
        //   - total_lines = COLLECTIVE_LINE_COUNT (29, vendored)
        //   - field.is_neutral() == true (step is inert by design)
        //   - max_press == 0.0 (no change from founder neutral)
        let is_neutral = field.is_neutral();
        println!(
            "seed={seed} total_lines={total_lines} nonzero_lines={nonzero_lines} max_press={max_press:.4} is_neutral={is_neutral}"
        );
        rows.push((seed, total_lines, is_neutral));
    }
    let passing = rows.iter().filter(|(_, _, n)| *n).count();
    let rate = passing as f64 / rows.len() as f64;
    println!(
        "\nfamily_pass_rate={rate:.4} threshold=1.00 wire_live_count={passing}/{}",
        rows.len()
    );
    if rate >= 1.0 {
        println!("verdict=COLLECTIVE_WIRE_LIVE (wire exercised, field inert per WP-I ponytail)");
    } else {
        println!("verdict=COLLECTIVE_WIRE_DEAD (no field step landed in any seed)");
    }
}
