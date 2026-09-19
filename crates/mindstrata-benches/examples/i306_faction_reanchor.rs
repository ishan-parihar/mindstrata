//! i306 — faction re-anchor probe (the behavioral blast of the meaning reflex).
//!
//! The meaning reflex (i306) re-rolls long-horizon trajectories wherever
//! meaning saturates, so the pestilence seed the faction-attachment
//! integration test sampled no longer carries its style-aware-dynamics signal
//! (assertion: "style-aware dynamics should be observable across the faction
//! population" — a liveness contract, so the SCENARIO is re-anchored, never the
//! assertion). This probe sweeps the i268-style seed family under the new
//! dynamics and reports, per seed:
//!
//!   * the first tick a live faction exists (the test samples that instant),
//!   * whether ANY faction registered BY THEN is non-Secure (style modulation
//!     ran),
//!   * the minimum supply level at that instant (style-independent daily
//!     consumption ran).
//!
//! A seed is a valid re-anchor if a live faction exists at some 1K sample AND
//! (a non-Secure faction exists OR supplies fell below 0.7) — i.e. exactly the
//! integration test's contract.
//!
//! Run: cargo run --release -p mindstrata-benches --example i306_faction_reanchor

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;
use mindstrata_sim::social::group_formation::{
    derive_group_attachment_style, GroupAttachmentStyle,
};

const SEEDS: [u64; 12] = [1, 2, 5, 7, 13, 21, 42, 46, 55, 77, 99, 123];
const TICKS: u64 = 30_000;

fn main() {
    println!("i306 faction re-anchor sweep — pestilence, {TICKS} ticks/seed, sample every 1K");
    println!(
        "  {:<6} {:>12} {:>10} {:>12} {:>10} {:>10} {:>6} {:>8}",
        "seed", "first_live", "reg@first", "non_secure", "min_supply", "cohesion", "style", "valid"
    );
    let mut valid = Vec::new();
    for seed in SEEDS {
        let mut sc = Scenario::pestilence();
        sc.seed = seed;
        sc.ticks = TICKS;
        let mut sim = Simulation::from_scenario(sc);
        sim.populate();
        // The integration test samples the FIRST instant a live faction exists
        // and reads the contract from the registry AT THAT INSTANT (its loop
        // breaks there) — mirror that exactly, or the probe measures a
        // different question than the test asks.
        let mut first_live: Option<u64> = None;
        let mut style_ok = false;
        let mut at_first = (0usize, 0usize, f64::NAN, f64::NAN);
        for step in 1..=(TICKS / 1000) {
            sim.run(1000);
            if first_live.is_none() && sim.faction_v2_registry.factions.iter().any(|f| f.active) {
                first_live = Some(step * 1000);
            }
            if first_live == Some(step * 1000) {
                let reg = &sim.faction_v2_registry.factions;
                // Full integration-test contract at this instant: every live
                // faction's stored style must equal the modal style of its
                // live members.
                style_ok = reg.iter().filter(|f| f.active).all(|f| {
                    let styles: Vec<_> = f
                        .members
                        .iter()
                        .map(|&m| sim.agents[m].attachment.style)
                        .collect();
                    f.attachment_style == derive_group_attachment_style(&styles)
                });
                at_first = (
                    reg.len(),
                    reg.iter()
                        .filter(|f| f.attachment_style != GroupAttachmentStyle::Secure)
                        .count(),
                    reg.iter()
                        .map(|f| f.supplies)
                        .min()
                        .map(|s| s.to_f64())
                        .unwrap_or(f64::NAN),
                    reg.iter()
                        .map(|f| f.cohesion)
                        .min()
                        .map(|c| c.to_f64())
                        .unwrap_or(f64::NAN),
                );
                break; // the test breaks here
            }
        }
        let (at_first_len, non_secure, min_supply, min_cohesion) = at_first;
        // All three clauses the integration test asserts at the sample instant.
        let cohesion_ok = min_cohesion <= 0.9 && min_cohesion >= 0.0;
        let ok =
            first_live.is_some() && style_ok && cohesion_ok && (non_secure > 0 || min_supply < 0.7);
        if ok {
            valid.push(seed);
        }
        println!(
            "  {seed:<6} {:>12} {:>10} {:>12} {:>10.4} {:>10.4} {:>6} {:>8}",
            first_live
                .map(|t| t.to_string())
                .unwrap_or_else(|| "none".to_string()),
            at_first_len,
            non_secure,
            min_supply,
            min_cohesion,
            style_ok,
            ok
        );
    }
    println!("\n  valid re-anchor seeds: {valid:?}");
}
