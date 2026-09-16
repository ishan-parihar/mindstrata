//! i275 — needs-band calibration sweep (CO-2026-003, FR-027).
//!
//! Evidence contract (AGENTS.md §2/§4 — probe before touching):
//! 1. The 6 band families in `docs/balance/needs-bands.md` are
//!    CALIBRATION-PENDING(AP3); this probe measures all 12 derived bands
//!    across the versioned 12-seed family (i268 harness) with per-band
//!    cross-seed CV so CO-2026-003 can land on measured values.
//! 2. The work↔rest pacing band (CO-2026-003 draft: [0.40,0.60] →
//!    [0.38,0.58], measured 0.43 via i268 stress corridor) is measured
//!    DIRECTLY here as Work:Rest action ratio — the first direct
//!    measurement of the band the CO names.
//! 3. Ratification gate: a band is ratifiable when CV = σ/μ ≤ 0.35 across
//!    the 12-seed family (needs-bands.md promotion rule). 11/12 ratifiable
//!    promotes the CO per design-final-co.md.
//!
//! Run: cargo run --release -p mindstrata-benches --example i275_needs_band_calibration

use mindstrata_sim::actions::ActionKind;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;
use std::collections::BTreeMap;

/// Versioned seed family — changing this is a runbook change.
const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const TICKS: u64 = 5000;
/// Equilibration window skipped before sampling (founder transients).
const WARMUP: u64 = 1000;

fn main() {
    // band_name -> per-seed mean values (BTreeMap for deterministic order)
    let mut band_series: BTreeMap<&'static str, Vec<f64>> = BTreeMap::new();
    let mut work_rest_ratios: Vec<f64> = Vec::new();

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

        // Sampling accumulators
        let mut ticks_sampled = 0_u64;
        let mut band_sum: BTreeMap<&'static str, f64> = BTreeMap::new();
        let mut action_sum: BTreeMap<&'static str, f64> = BTreeMap::new();
        let mut hunger_gate_hits = 0_u64;
        let mut agent_tick_samples = 0_u64;

        for t in 0..TICKS {
            sim.tick();
            if t < WARMUP {
                continue;
            }
            ticks_sampled += 1;
            let n = sim.agents.len() as f64;
            let mut work = 0.0_f64;
            let mut rest = 0.0_f64;
            let mut socialize = 0.0_f64;
            let mut worship = 0.0_f64;
            for a in &sim.agents {
                // Per-band need pressure (deficit 0..1)
                *band_sum.entry("survival_hunger").or_default() += a.needs.hunger.to_f64() / n;
                *band_sum.entry("survival_thirst").or_default() += a.needs.thirst.to_f64() / n;
                *band_sum.entry("safety_gate").or_default() += a.needs.safety.to_f64() / n;
                *band_sum.entry("belonging_social").or_default() += a.needs.social.to_f64() / n;
                *band_sum.entry("esteem_esteem").or_default() += a.needs.esteem.to_f64() / n;
                *band_sum.entry("transcendence_autonomy").or_default() +=
                    a.needs.autonomy.to_f64() / n;
                *band_sum.entry("meaning_gate").or_default() += a.needs.meaning.to_f64() / n;
                *band_sum.entry("fatigue_gate").or_default() += a.needs.fatigue.to_f64() / n;
                // FR-027 damage-gate liveness: fraction of agent-ticks past the
                // 0.9 starvation damage threshold (0 = dead producer, 1 = saturated).
                if a.needs.hunger.to_f64() > 0.9 {
                    hunger_gate_hits += 1;
                }
                agent_tick_samples += 1;
                match a.current_action {
                    ActionKind::Work => work += 1.0,
                    ActionKind::Rest => rest += 1.0,
                    ActionKind::Socialize => socialize += 1.0,
                    ActionKind::Worship => worship += 1.0,
                    _ => {}
                }
            }
            let total = n.max(1.0);
            *action_sum.entry("work").or_default() += work / total;
            *action_sum.entry("rest").or_default() += rest / total;
            *action_sum.entry("socialize").or_default() += socialize / total;
            *action_sum.entry("worship").or_default() += worship / total;
        }

        // Per-seed means for every band
        let mut row: BTreeMap<&'static str, f64> = BTreeMap::new();
        for (k, v) in &band_sum {
            row.insert(k, v / ticks_sampled as f64);
        }
        for (k, v) in &action_sum {
            row.insert(k, v / ticks_sampled as f64);
        }
        row.insert(
            "hunger_gate_exceedance",
            hunger_gate_hits as f64 / agent_tick_samples.max(1) as f64,
        );
        let work_share = row["work"];
        let rest_share = row["rest"];
        // The CO band: Work:Rest pacing ratio (0 = all rest, 1 = balanced, →∞ all work).
        // Measured as rest/(work+rest) — the rest-share of upkeep behavior.
        let wr = if work_share + rest_share > 0.0 {
            rest_share / (work_share + rest_share)
        } else {
            0.0
        };
        row.insert("work_rest_ratio", wr);
        work_rest_ratios.push(wr);

        print!("seed={seed}");
        for (k, v) in &row {
            print!(" {k}={v:.4}");
            band_series.entry(k).or_default().push(*v);
        }
        println!();
    }

    // ── Cross-seed CV per band ──
    println!("\n# band cross-seed statistics (CV = σ/μ; ratifiable ≤ 0.35)");
    let mut ratifiable = 0_usize;
    let mut total_bands = 0_usize;
    for (band, series) in &band_series {
        let n = series.len() as f64;
        let mean = series.iter().sum::<f64>() / n;
        let var = series.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let sigma = var.sqrt();
        let cv = if mean > 1e-9 { sigma / mean } else { 0.0 };
        let ok = cv <= 0.35;
        total_bands += 1;
        if ok {
            ratifiable += 1;
        }
        println!(
            "band_summary band={band} mean={mean:.4} sigma={sigma:.4} cv={cv:.4} ratifiable={}",
            u8::from(ok)
        );
    }

    // ── CO-2026-003 direct measurement ──
    let wr_mean = work_rest_ratios.iter().sum::<f64>() / work_rest_ratios.len() as f64;
    let wr_var = work_rest_ratios
        .iter()
        .map(|v| (v - wr_mean).powi(2))
        .sum::<f64>()
        / work_rest_ratios.len() as f64;
    let wr_sigma = wr_var.sqrt();
    let wr_cv = if wr_mean > 1e-9 {
        wr_sigma / wr_mean
    } else {
        0.0
    };
    println!("\n# CO-2026-003 work↔rest band: draft [0.40,0.60]→[0.38,0.58], draft measured 0.43");
    println!(
        "co_2026_003 measured={wr_mean:.4} sigma={wr_sigma:.4} cv={wr_cv:.4} in_new_band={}",
        u8::from((0.38..=0.58).contains(&wr_mean))
    );

    let rate = ratifiable as f64 / total_bands.max(1) as f64;
    println!(
        "\nratifiable_rate={rate:.4} bands={ratifiable}/{total_bands} threshold=11/12 (0.917)"
    );
    if ratifiable as f64 / total_bands.max(1) as f64 >= 11.0 / 12.0 {
        println!("verdict=NEEDS_BANDS_RATIFIABLE (CO-2026-003 may land on measured values)");
    } else {
        println!("verdict=NEEDS_BANDS_PENDING (CV gate not met — bands stay CALIBRATION-PENDING)");
    }
}
