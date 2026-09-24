//! i380 — is the need-driven SOCIAL drive inert, or is a near-zero deficit correct?
//!
//! The i379 gate census measured `needs.social` occupying 0–0.03 in every world
//! (p95 0.03) and flagged the action-utility term
//! `utility += needs.social × social_value × extraversion` (`actions/mod.rs`) as a
//! *gain/scale mismatch*: the term multiplies by ~0.006, so it contributes ~0.5%
//! of typical utility. But that flag has two readings that want opposite fixes:
//!
//!   * **INERT** — nobody socializes through the utility leg; the drive rounds to
//!     nothing in the common case, so the term is decoration and the fix is a
//!     deficit-independent baseline (an extravert socializes because it is
//!     rewarding, not only when lonely).
//!   * **CORRECT-BY-DESIGN** — villagers are socially satisfied, so a
//!     deficit-proportional drive *should* be near zero, and sociality is carried
//!     by the daily routine. Then the audit's flag is too strong and the item is
//!     refuted (as i353/i356/i357 refuted their premises).
//!
//! The decision census (`sim::decision_census`, i346) separates them: it reports
//! decisions per selection source and per final action, plus the cross table.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i380_socialize_reachability`

use mindstrata_sim::sim::decision_census;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn leg(label: &str, width: u32, height: u32, agent_count: u32, ticks: u64, social_decay: f64) {
    decision_census::enable();
    decision_census::reset();
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: ticks,
        world_width: width,
        world_height: height,
        num_agents: agent_count,
        snapshot_interval: None,
    });
    sim.params.social_decay_rate = mindstrata_core::fixed::Fixed::from_f64(social_decay);
    sim.populate();
    // Track the need's band concurrently with the decision stream.
    let mut social_samples: Vec<f64> = Vec::new();
    let step = 100u64;
    let mut done = 0u64;
    while done < ticks {
        sim.run(step);
        done += step;
        for agent in &sim.agents {
            social_samples.push(agent.needs.social.to_f64());
        }
    }
    let report = decision_census::report();
    decision_census::disable();

    social_samples
        .sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let quantile =
        |frac: f64| social_samples[(((social_samples.len() - 1) as f64) * frac).round() as usize];

    let total = report.total();
    println!("══ {label} ══  ({total} decisions)");
    println!(
        "  needs.social band: p50 {:.4}  p95 {:.4}  max {:.4}",
        quantile(0.50),
        quantile(0.95),
        social_samples[social_samples.len() - 1]
    );
    let _ = social_decay;
    println!("  decisions by source:");
    for (src_idx, name) in decision_census::SOURCE_NAMES.iter().enumerate() {
        let count = report.sources[src_idx];
        if count > 0 {
            println!(
                "    {name:<9} {count:>8}  {:>6.2}%",
                100.0 * count as f64 / total as f64
            );
        }
    }
    println!("  decisions by action:");
    let mut order: Vec<usize> = (0..decision_census::ACTION_COUNT).collect();
    order.sort_by_key(|act| std::cmp::Reverse(report.actions[*act]));
    for act_idx in order {
        let count = report.actions[act_idx];
        if count == 0 {
            continue;
        }
        println!(
            "    {:<10} {count:>8}  {:>6.2}%",
            decision_census::ACTION_NAMES[act_idx],
            100.0 * count as f64 / total as f64
        );
    }
    // The Socialize row, decomposed by source — the whole question.
    let socialize_idx =
        decision_census::action_index(mindstrata_sim::actions::ActionKind::Socialize);
    println!("  Socialize by source:");
    for (src, name) in decision_census::SOURCE_NAMES.iter().enumerate() {
        let count = report.cross[src][socialize_idx];
        if count > 0 {
            println!("    {name:<9} {count:>8}");
        }
    }
    println!(
        "  utility arbitrations sampled: {}; quiet-window arbitrations: {}",
        report.utility_samples, report.quiet_samples
    );
    // Sizing: what does the WINNER's utility look like? The social term is
    // `needs.social x social_value(0.3) x extraversion(~0.5)`; comparing it with
    // the winner's magnitude is what turns "the gain is too small" into a number.
    if report.quiet_samples > 0 {
        println!(
            "  quiet-window winner utility: mean {:.5}  max {:.5}   (social term at p50 {:.5}, p95 {:.5})",
            report.quiet_winner_sum / report.quiet_samples as f64,
            report.quiet_winner_max,
            quantile(0.50) * 0.3 * 0.5,
            quantile(0.95) * 0.3 * 0.5
        );
    }
    println!();
}

fn main() {
    println!("i380 — Socialize reachability and the needs.social scale (seed 42)");
    println!("      social_decay_rate sweep: the need's band vs its own 0.30/0.40 gates");
    println!("      (hunger accrues 0.001, thirst 0.002, fatigue 0.0005; social 0.0002)\n");
    for rate in [0.0002f64, 0.001, 0.002, 0.004] {
        leg(
            &format!("village 16x16 N=12 @20K, social_decay_rate {rate}"),
            16,
            16,
            12,
            20_000,
            rate,
        );
    }
    for rate in [0.0002f64, 0.002] {
        leg(
            &format!("town 46x46 N=48 @20K, social_decay_rate {rate}"),
            46,
            46,
            48,
            20_000,
            rate,
        );
    }
    println!(
        "reading: if Socialize's decisions are ~all `routine` and ~0 `utility`, the\n\
         need-term is inert in the common case (INERT). If the utility leg does pick\n\
         Socialize a meaningful share of the time, the other utility components carry\n\
         it and the near-zero need is the deficit working correctly (CORRECT-BY-DESIGN)."
    );
}
