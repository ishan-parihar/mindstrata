//! i287 — Era IV exit probe: WP-J institutional coupling, end-to-end forced-stage
//! A/B (`i300_institution_shift` shape, PLAN_DC3 §4 i287).
//!
//! i280 landed two live WP-J channels (§12.3 compliance multiplier + member Work
//! bonus) and proved unit-level liveness, but rejected the end-to-end A/B as
//! CONFOUNDED: it forced all 29 collective lines, shifting every stage-gated
//! system at once. This probe fixes the confound: force ONLY the two WP-J input
//! lines (governance, economic-systems) to the amber→green crossing band each
//! tick, hold everything else natural, and measure:
//!
//!   1. Ramp liveness — the compliance multiplier at/below gate vs above it.
//!   2. Channel #2 (Work axis) — provisioning surface: grain, trades.
//!   3. Channel #1 (§12.3 compliance) — NormViolated count (dead at N=12 per
//!      the i280 sweep; the probe confirms or falsifies at the forced horizon).
//!   4. Coupled-surface attribution — genesis class counts and norm-proposal
//!      count (the band-III gates on Safety system lines open as a side effect;
//!      reported so no delta is misattributed to WP-J itself).
//!
//! Zero-blast leg: forced lines at their natural stage ⇒ byte-identical control.
//! Doctrine §4.3: measure equilibria, not just assertions.

use mindstrata_core::event::SimEvent;
use mindstrata_sim::sim::{SimConfig, Simulation};

/// The two WP-J input-line slugs (vendored registry, `canon_tables.rs`).
const WPJ_LINES: [&str; 2] = ["governance", "economic-systems"];

fn config(seed: u64, ticks: u64) -> SimConfig {
    SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    }
}

/// Indexes of the two WP-J lines in the collective field's vendored order.
fn wpj_indexes(sim: &Simulation) -> Vec<usize> {
    let _ = sim; // caller has populated; signature keeps the call site self-documenting
    let slugs = mindstrata_development::collective::CollectiveField::line_slugs();
    slugs
        .iter()
        .enumerate()
        .filter(|(_, s)| WPJ_LINES.contains(&s.slug()))
        .map(|(i, _)| i)
        .collect()
}

/// A/B harness: run `ticks` with the WP-J lines held at `forced_stage` every
/// tick (f64 write directly into the field snapshot the passes read), or left
/// natural when `forced_stage` is `None`.
fn run_ab(seed: u64, ticks: u64, forced_stage: Option<f64>) -> AbResult {
    let mut sim = Simulation::new(config(seed, ticks));
    sim.populate();
    let idx = wpj_indexes(&sim);
    assert_eq!(idx.len(), 2, "both WP-J input lines must resolve");

    let mut last_metrics = (0.0_f64, 0_u64, 0_u64);
    for _ in 0..ticks {
        if let Some(stage) = forced_stage {
            let field = sim.collective_field;
            let mut lines = field.lines;
            for &i in &idx {
                lines[i].stage = stage;
            }
            sim.collective_field = mindstrata_development::collective::CollectiveField {
                lines,
                lambda: field.lambda,
            };
        }
        sim.tick();
    }
    // Event census over the full run window (pub read-side accessor).
    let norm_violated = sim
        .recent_events(sim.event_count())
        .iter()
        .filter(|e| matches!(e, SimEvent::NormViolated { .. }))
        .count() as u64;
    let genesis = sim
        .meme_registry
        .memes
        .iter()
        .filter(|m| m.description.contains("[genesis:"))
        .count() as u64;
    let mean_stage: f64 = idx
        .iter()
        .map(|&i| sim.collective_field.lines[i].stage)
        .sum::<f64>()
        / 2.0;
    // Morale lives in the derived collective psychology, not on Institution.
    let mean_morale: f64 = if sim.institutions.is_empty() {
        0.0
    } else {
        sim.institutions
            .iter()
            .map(|i| i.collective.morale.to_f64())
            .sum::<f64>()
            / sim.institutions.len() as f64
    };
    if let Some(m) = sim.metric_history.last() {
        last_metrics = (m.total_grain, m.total_trades, m.agent_count);
    }
    let (grain, trades, population) = last_metrics;
    AbResult {
        mean_stage,
        mean_morale,
        grain,
        trades,
        norm_violated,
        genesis,
        proposals: sim.norms.norms().len() as u64,
        population,
    }
}

#[allow(dead_code)]
struct AbResult {
    mean_stage: f64,
    mean_morale: f64,
    grain: f64,
    trades: u64,
    norm_violated: u64,
    genesis: u64,
    proposals: u64,
    population: u64,
}

fn cmp(label: &str, control: &AbResult, treated: &AbResult) {
    println!("--- {label} ---");
    println!(
        "  stage {:.3}  morale {:.4}  grain {:.2} (Δ {:+.2})  trades {} (Δ {:+})",
        treated.mean_stage,
        treated.mean_morale,
        treated.grain,
        treated.grain - control.grain,
        treated.trades,
        treated.trades as i64 - control.trades as i64,
    );
    println!(
        "  NormViolated {} (Δ {:+})  genesis {} (Δ {:+})  norms {} (Δ {:+})  pop {}",
        control.norm_violated,
        treated.norm_violated as i64 - control.norm_violated as i64,
        control.genesis,
        treated.genesis as i64 - control.genesis as i64,
        control.proposals,
        treated.proposals as i64 - control.proposals as i64,
        treated.population,
    );
}

fn main() {
    println!("== i287 institution_shift: WP-J forced-stage A/B (N=12, seed 42) ==");

    // Horizon 1: 2K, forced BELOW the band-III gate (stage 3.5) — the ramp
    // must stay identity; every surface byte-identical to natural.
    let c = run_ab(42, 2_000, None);
    let b = run_ab(42, 2_000, Some(3.5));
    println!("\n[h1] 2K, forced 3.5 (< gate 4.0) — expect all-zero deltas");
    cmp("natural vs forced-3.5", &c, &b);

    // Horizon 2: 2K, forced to the crossing band (stage 6.0 = the i280-measured
    // 20K natural value) — the ramp opens at 1.10; Work-axis surface must move.
    let b6 = run_ab(42, 2_000, Some(6.0));
    println!("\n[h2] 2K, forced 6.0 (crossing band, mult 1.10)");
    cmp("natural vs forced-6.0", &c, &b6);

    // Horizon 3: 20K natural — does the coupling open WITHOUT forcing, at the
    // i280-measured natural stage? (The Era IV exit question: is the village's
    // own trajectory enough, or is the coupling live only under forcing?)
    let c20 = run_ab(42, 20_000, None);
    println!("\n[h3] 20K natural — WP-J lines at their own trajectory");
    println!(
        "  stage {:.3}  morale {:.4}  grain {:.2}  trades {}  NormViolated {}  genesis {}  norms {}",
        c20.mean_stage, c20.mean_morale, c20.grain, c20.trades, c20.norm_violated, c20.genesis, c20.proposals,
    );
}
