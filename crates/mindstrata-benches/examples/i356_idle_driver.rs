//! i356 — Idle revival probe: what driver magnitude makes `Idle` live?
//!
//! `Idle` is the last dead action. i346/i351 measured it at **0 wins** in
//! 96 000 agent-ticks: it carries 0.05 fatigue/tick — the *same effective rate*
//! as `Rest` (0.4 over 8 ticks) but with no energy recovery, no motive relief
//! (`core.rs`'s completion match has no `Idle` arm), and a
//! behavioural-inhibition trait, so `Rest` strictly dominates it.
//!
//! The symmetric design act to i351's A8: `MotiveCategory::Play` (recreation)
//! grows a deficit (0.001/tick, cap 0.2, urgency 0.2, joy-amplified) and
//! competes in `update_dominant`, but **no action relieves it anywhere** — a
//! dead motive, the i351 `Novelty` class. `Idle` is its natural expression:
//! a satiated villager with unmet recreation need does nothing in particular.
//!
//! This probe measures, before the driver is sized:
//!   1. the in-vivo `Play` pressure distribution (driver headroom),
//!   2. the quiet-window `Idle` wall (pre-driver `winner − Idle` gap),
//!   3. the offline coefficient sweep over realized samples (§4.2 sizing).
//!
//! Run: `cargo run --release -p mindstrata-benches --example i356_idle_driver`

use mindstrata_sim::sim::decision_census;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn run(seed: u64, n: u32, horizon: u64, side: u32) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: horizon,
        world_width: side,
        world_height: side,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(horizon);
    sim
}

fn percentile(v: &mut [f64], p: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = (((v.len() as f64) * p).ceil() as usize).saturating_sub(1);
    v[idx.min(v.len() - 1)]
}

fn main() {
    println!("i356 — Idle recreation-driver sizing (32x32)\n");

    // ── Leg A: the Play pressure distribution at settlement ──
    println!("leg A — Play pressure (full formula, the number update_dominant compares):");
    for (n, horizon) in [(12u32, 10_000u64), (48, 10_000)] {
        let sim = run(42, n, horizon, 32);
        let mut ps: Vec<f64> = sim
            .agents
            .iter()
            .map(|a| {
                a.motivation
                    .pressure_full(mindstrata_psych::psychology::motivation::MotiveCategory::Play)
                    .to_f64()
            })
            .collect();
        let mean = ps.iter().sum::<f64>() / ps.len().max(1) as f64;
        println!(
            "  N={n:<3} mean {mean:.4}  p50 {:.4}  p90 {:.4}  max {:.4}",
            percentile(&mut ps, 0.5),
            percentile(&mut ps, 0.9),
            ps.last().copied().unwrap_or(0.0)
        );
    }

    // ── Leg B: the quiet-window Idle wall + coefficient sweep ──
    println!("\nleg B — quiet-window Idle gap + idles-at-coef sweep:");
    for (seed, n, horizon) in [
        (42u64, 12u32, 20_000u64),
        (7, 12, 20_000),
        (11, 12, 20_000),
        (46, 12, 20_000),
        (42, 48, 20_000),
    ] {
        decision_census::reset();
        decision_census::enable();
        let _sim = run(seed, n, horizon, 32);
        let rep = decision_census::report();
        let idle = rep.idle;
        println!(
            "  seed={seed} N={n}: decisions {}  idle wins {}  within-noise {}  mean-loss {:.3}  max {:.3}",
            rep.utility_samples,
            idle.wins,
            idle.within_noise,
            idle.mean_loss(rep.utility_samples),
            idle.max
        );
        let qw = rep.quiet_wander;
        println!(
            "    quiet windows {}  quiet-idle pairs {}  (quiet-Wander mean-loss {:.3})",
            rep.quiet_samples,
            decision_census::quiet_idle_pair_count(),
            qw.mean_loss(rep.quiet_samples)
        );
        let mut gaps = decision_census::quiet_idle_gaps();
        println!(
            "    idle-gap p10 {:.3}  p25 {:.3}  p50 {:.3}  p75 {:.3}  p90 {:.3}",
            percentile(&mut gaps, 0.10),
            percentile(&mut gaps, 0.25),
            percentile(&mut gaps, 0.50),
            percentile(&mut gaps, 0.75),
            percentile(&mut gaps, 0.90)
        );
        let coefs = [1.0_f64, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0];
        let sweep = decision_census::quiet_idle_sweep(&coefs);
        let denom = decision_census::quiet_idle_pair_count().max(1) as f64;
        let parts: Vec<String> = sweep
            .iter()
            .map(|(c, w)| format!("c={c}: {} ({:.2}%)", w, 100.0 * *w as f64 / denom))
            .collect();
        println!("    sweep: {}", parts.join("  "));
        decision_census::disable();
    }

    // ── Leg C: in-vivo post-driver share + the de-saturated play pressure ──
    println!("\nleg C — post-driver Idle share + play pressure (de-saturation check):");
    for (seed, n, horizon) in [
        (42u64, 12u32, 20_000u64),
        (7, 12, 20_000),
        (11, 12, 20_000),
        (46, 12, 20_000),
        (42, 48, 20_000),
        (5, 12, 20_000),
        (23, 12, 20_000),
        (99, 12, 20_000),
        (7, 48, 20_000),
    ] {
        decision_census::reset();
        decision_census::enable();
        let sim = run(seed, n, horizon, 32);
        let rep = decision_census::report();
        decision_census::disable();
        let total: u64 = rep.actions.iter().sum();
        let idle_share = if total > 0 {
            100.0
                * rep.actions
                    [decision_census::action_index(mindstrata_sim::actions::ActionKind::Idle)]
                    as f64
                / total as f64
        } else {
            0.0
        };
        let wander_share = if total > 0 {
            100.0
                * rep.actions
                    [decision_census::action_index(mindstrata_sim::actions::ActionKind::Wander)]
                    as f64
                / total as f64
        } else {
            0.0
        };
        let mut ps: Vec<f64> = sim
            .agents
            .iter()
            .map(|a| {
                a.motivation
                    .pressure_full(mindstrata_psych::psychology::motivation::MotiveCategory::Play)
                    .to_f64()
            })
            .collect();
        let pmean = ps.iter().sum::<f64>() / ps.len().max(1) as f64;
        println!(
            "  seed={seed} N={n}: decisions {total}  Idle {:.2}%  Wander {:.2}%  playµ {pmean:.4} p50 {:.4} max {:.4}",
            idle_share,
            wander_share,
            percentile(&mut ps, 0.5),
            ps.last().copied().unwrap_or(0.0)
        );
    }

    // ── Leg D: provisioning invariant — the driver displaces some Work, so
    // confirm the village is not starved at long horizons (the 10K snapshot's
    // total_grain moved; a stock is phase-sensitive, hunger is not). ──
    println!("\nleg D — provisioning (hunger/grain/health) at long horizons:");
    for (seed, n, horizon) in [(42u64, 12u32, 10_000u64), (42, 12, 50_000), (7, 12, 50_000)] {
        let sim = run(seed, n, horizon, 32);
        let ms = sim.metrics_snapshot();
        println!(
            "  seed={seed} N={n} t={horizon}: agents {}  hunger {:.4}  grain {:.4}  health {:.4}  stress {:.4}",
            ms.agent_count, ms.avg_hunger, ms.total_grain, ms.avg_health, ms.avg_stress
        );
    }
}
