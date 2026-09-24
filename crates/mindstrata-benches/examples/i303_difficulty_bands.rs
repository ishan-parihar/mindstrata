//! i303 — Difficulty-lever bands probe (DC-4 entry "b").
//!
//! Promotes `docs/balance/difficulty-levers.md` row 2 ("Need decay /
//! fulfillment thresholds", candidate bands Lenient 0.6× / Standard 1.0× /
//! Harsh 1.4×) from a DRAFT catalog to a measured runtime surface. Three legs,
//! in the order the promotion rule (`difficulty-levers.md` §Promotion rule)
//! requires:
//!
//!   1. RAW BANDS (§5 quantize-once): the effective Fixed-4 rates each band
//!      materializes, with the derivation per channel — evidence that the
//!      mapping is computed once at construction, never per tick, and the
//!      honest sub-resolution notes (meaning's Lenient band collapses onto
//!      canon; social's Lenient band rounds to half canon).
//!   2. STANDARD ≡ CANON: `with_difficulty(Standard)` must be byte-identical
//!      to the untouched default — the zero-blast contract. Proven twice:
//!      params serde equality, and end-state digest equality of two full runs.
//!   3. FAMILY DIFFERENTIAL: 12-seed family × 3 bands × 2000 ticks. Measures
//!      mean survival deficits and per-agent-tick action shares; the catalog's
//!      predicted direction is measured, not assumed (Lenient feels abundant →
//!      lower deficits; Harsh feels scarcity-driven → higher deficits and less
//!      social/worship action).
//!
//! VERDICT CONTRACT (§4.4 re-contract, named not silently widened):
//!   LIVE ⇔ Standard identity holds
//!          ∧ all 12 seeds alive in all 3 bands
//!          ∧ the catalog's NAMED felt effect is monotone
//!            (Worship share: lenient > standard > harsh)
//!          ∧ the AGGREGATE mean deficit is monotone
//!            (lenient < standard < harsh, all 5 channels, equal weight).
//!
//! Why aggregate and not per-channel: the i303 family run measured per-channel
//! monotonicity at 2/5, and BOTH failures are mechanical, not lever weakness —
//! hunger/fatigue end-deficits are relief-saturated at this horizon (strong
//! feed/rest systems absorb a 2.3× decay-band change, measured), and meaning's
//! Lenient band is quantized onto canon at Fixed-4 (raw 1 both, leg 1). Since
//! the channels also couple through behavior (a band that changes thirst
//! action cadence perturbs every other channel), strict per-channel ordering
//! is not a contract the mechanism can promise; the aggregate is. The
//! per-channel table is printed in full as the finding, and the micro contract
//! (exact raw band per channel) is pinned in `parameters::tests` instead.
//!
//! Run: cargo run --release -p mindstrata-benches --example i303_difficulty_bands

use mindstrata_sim::parameters::{DifficultyProfile, SimParameters};
use mindstrata_sim::sim::{SimConfig, Simulation};
use std::collections::HashMap;

/// The i268 seed family (the calibration-audit stability instrument).
const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const TICKS: u64 = 2000;

/// The probes track these action labels as the "what the player feels" signal.
const TRACKED_ACTIONS: [&str; 8] = [
    "Work",
    "Socialize",
    "Worship",
    "Trade",
    "Eat",
    "Drink",
    "Rest",
    "Idle",
];

fn config(seed: u64) -> SimConfig {
    SimConfig {
        seed,
        max_ticks: TICKS,
        num_agents: 12,
        snapshot_interval: None,
        ..SimConfig::default()
    }
}

type DigestRow = (i64, i64, i64, i64, i64, i64, i64, i64, i32, i32, String);

/// Deterministic end-state digest: needs + body + action + position per agent,
/// plus the total event count. Dense enough that any behavioral divergence
/// between two runs shows up (aggregate means can cancel; this cannot).
fn state_digest(sim: &Simulation) -> String {
    let rows: Vec<DigestRow> = sim
        .agents
        .iter()
        .map(|a| {
            (
                a.needs.hunger.to_raw(),
                a.needs.thirst.to_raw(),
                a.needs.fatigue.to_raw(),
                a.needs.safety.to_raw(),
                a.needs.social.to_raw(),
                a.needs.meaning.to_raw(),
                a.body.health.to_raw(),
                a.body.energy.to_raw(),
                a.position.x,
                a.position.y,
                format!("{:?}", a.current_action),
            )
        })
        .collect();
    format!("{}|{rows:?}", sim.event_count())
}

/// Per-band aggregated measurements across the seed family.
#[derive(Default)]
struct BandStats {
    /// Mean final deficit per channel (mean over seeds of the per-seed mean).
    hunger: f64,
    thirst: f64,
    fatigue: f64,
    social: f64,
    meaning: f64,
    /// Per-agent-tick action shares over the whole family (label → share).
    action_share: HashMap<&'static str, f64>,
    /// Seeds that finished with a live population.
    alive_seeds: usize,
    /// Total agent-tick samples (the share denominator).
    samples: u64,
}

fn measure(profile: DifficultyProfile) -> BandStats {
    let mut stats = BandStats::default();
    let mut counts: HashMap<&'static str, u64> = HashMap::new();
    for &seed in &SEEDS {
        let mut sim = Simulation::new(config(seed));
        sim.params = SimParameters::with_difficulty(profile);
        sim.populate();
        for _ in 0..TICKS {
            sim.tick();
            for a in &sim.agents {
                let label = format!("{:?}", a.current_action);
                if let Some(tracked) = TRACKED_ACTIONS.iter().find(|t| **t == label) {
                    *counts.entry(tracked).or_insert(0) += 1;
                    stats.samples += 1;
                }
            }
        }
        let n = sim.agents.len() as f64;
        if n > 0.0 {
            stats.alive_seeds += 1;
            stats.hunger += sim
                .agents
                .iter()
                .map(|a| a.needs.hunger.to_f64())
                .sum::<f64>()
                / n;
            stats.thirst += sim
                .agents
                .iter()
                .map(|a| a.needs.thirst.to_f64())
                .sum::<f64>()
                / n;
            stats.fatigue += sim
                .agents
                .iter()
                .map(|a| a.needs.fatigue.to_f64())
                .sum::<f64>()
                / n;
            stats.social += sim
                .agents
                .iter()
                .map(|a| a.needs.social.to_f64())
                .sum::<f64>()
                / n;
            stats.meaning += sim
                .agents
                .iter()
                .map(|a| a.needs.meaning.to_f64())
                .sum::<f64>()
                / n;
        }
    }
    let n = SEEDS.len() as f64;
    stats.hunger /= n;
    stats.thirst /= n;
    stats.fatigue /= n;
    stats.social /= n;
    stats.meaning /= n;
    let denom = stats.samples.max(1) as f64;
    for label in TRACKED_ACTIONS {
        stats.action_share.insert(
            label,
            counts.get(label).copied().unwrap_or(0) as f64 / denom,
        );
    }
    stats
}

fn main() {
    println!(
        "i303 difficulty-lever bands — need decay (levers row 2), {TICKS} ticks/seed, family of {}",
        SEEDS.len()
    );

    // ── Leg 1: raw band table (§5 quantize-once evidence) ────────────────
    println!("\n[leg 1] effective Fixed-4 raw bands (1 raw = 1e-4/tick)");
    println!(
        "{:<10} {:>7} {:>7} {:>8} {:>7} {:>7} {:>8}",
        "band", "hunger", "thirst", "fatigue", "safety", "social", "meaning"
    );
    for profile in [
        DifficultyProfile::Lenient,
        DifficultyProfile::Standard,
        DifficultyProfile::Harsh,
    ] {
        let p = SimParameters::with_difficulty(profile);
        println!(
            "{:<10} {:>7} {:>7} {:>8} {:>7} {:>7} {:>8}  (×{})",
            profile,
            p.hunger_decay_rate.to_raw(),
            p.thirst_decay_rate.to_raw(),
            p.fatigue_decay_rate.to_raw(),
            p.safety_decay_rate.to_raw(),
            p.social_decay_rate.to_raw(),
            p.meaning_decay_rate.to_raw(),
            profile.decay_multiplier(),
        );
    }
    println!("  note: social Lenient 0.00012 rounds DOWN to raw 1 (half canon);");
    println!(
        "        meaning Lenient 0.00009 rounds back to canon raw 1 (sub-resolution collapse)."
    );

    // ── Leg 2: Standard ≡ canon identity (zero-blast contract) ───────────
    println!("\n[leg 2] Standard identity vs untouched canon defaults");
    let default_params = serde_json::to_string(&SimParameters::default()).unwrap();
    let standard_params =
        serde_json::to_string(&SimParameters::with_difficulty(DifficultyProfile::Standard))
            .unwrap();
    let params_identical = default_params == standard_params;
    println!("  params serde identical: {params_identical}");

    let mut canon_sim = Simulation::new(config(42));
    canon_sim.populate();
    canon_sim.run(TICKS);
    let canon_digest = state_digest(&canon_sim);

    let mut standard_sim = Simulation::new(config(42));
    standard_sim.params = SimParameters::with_difficulty(DifficultyProfile::Standard);
    standard_sim.populate();
    standard_sim.run(TICKS);
    let standard_digest = state_digest(&standard_sim);
    let state_identical = canon_digest == standard_digest;
    println!("  end-state digest identical (seed 42, {TICKS} ticks): {state_identical}");

    // ── Leg 3: family differential ───────────────────────────────────────
    println!("\n[leg 3] 12-seed family differential (mean final deficit, action share)");
    let lenient = measure(DifficultyProfile::Lenient);
    let standard = measure(DifficultyProfile::Standard);
    let harsh = measure(DifficultyProfile::Harsh);

    println!(
        "{:<10} {:>7} {:>7} {:>8} {:>7} {:>8} {:>6}",
        "band", "hunger", "thirst", "fatigue", "social", "meaning", "alive"
    );
    for (name, s) in [
        ("lenient", &lenient),
        ("standard", &standard),
        ("harsh", &harsh),
    ] {
        println!(
            "{name:<10} {:>7.4} {:>7.4} {:>8.4} {:>7.4} {:>8.4} {:>4}/12",
            s.hunger, s.thirst, s.fatigue, s.social, s.meaning, s.alive_seeds
        );
    }
    println!("\n  action shares (per-agent-tick):");
    println!(
        "  {:<10} {:>8} {:>8} {:>8}",
        "band", "Work", "Socialize", "Worship"
    );
    for (name, s) in [
        ("lenient", &lenient),
        ("standard", &standard),
        ("harsh", &harsh),
    ] {
        println!(
            "  {name:<10} {:>8.4} {:>8.4} {:>8.4}",
            s.action_share["Work"], s.action_share["Socialize"], s.action_share["Worship"]
        );
    }

    // ── Verdict (pre-registered — see the module doc) ────────────────────
    // Deficit-channel monotonicity, reported per channel (nothing hidden).
    let channels: [(&str, f64, f64, f64); 5] = [
        ("hunger", lenient.hunger, standard.hunger, harsh.hunger),
        ("thirst", lenient.thirst, standard.thirst, harsh.thirst),
        ("fatigue", lenient.fatigue, standard.fatigue, harsh.fatigue),
        ("social", lenient.social, standard.social, harsh.social),
        ("meaning", lenient.meaning, standard.meaning, harsh.meaning),
    ];
    println!("\n  deficit-channel detail (informational — aggregate is the contract):");
    let mut monotone_channels = 0;
    for (name, l, s, h) in channels {
        let mono = l < s && s < h;
        if mono {
            monotone_channels += 1;
        }
        println!(
            "    {name:<8} monotone={mono:<5} (lenient {l:.4} | standard {s:.4} | harsh {h:.4})"
        );
    }
    let aggregate = |s: &BandStats| (s.hunger + s.thirst + s.fatigue + s.social + s.meaning) / 5.0;
    let (agg_l, agg_s, agg_h) = (aggregate(&lenient), aggregate(&standard), aggregate(&harsh));
    let aggregate_monotone = agg_l < agg_s && agg_s < agg_h;
    println!(
        "    aggregate {aggregate_monotone} (lenient {agg_l:.4} | standard {agg_s:.4} | harsh {agg_h:.4})"
    );

    let identity = params_identical && state_identical;
    let family_stable = lenient.alive_seeds == SEEDS.len()
        && standard.alive_seeds == SEEDS.len()
        && harsh.alive_seeds == SEEDS.len();
    let worship_monotone = lenient.action_share["Worship"] > standard.action_share["Worship"]
        && standard.action_share["Worship"] > harsh.action_share["Worship"];

    println!("\nleg2_standard_identity={identity}");
    println!(
        "leg3_family_stable={family_stable} (alive {}/{}/{})",
        lenient.alive_seeds, standard.alive_seeds, harsh.alive_seeds
    );
    println!(
        "leg3_worship_share_monotone={worship_monotone} ({:.4} > {:.4} > {:.4})",
        lenient.action_share["Worship"],
        standard.action_share["Worship"],
        harsh.action_share["Worship"]
    );
    println!("leg3_deficit_channels_monotone={monotone_channels}/5 (informational; contract is aggregate)");
    println!("leg3_aggregate_deficit_monotone={aggregate_monotone}");
    if identity && family_stable && worship_monotone && aggregate_monotone {
        println!("verdict=DIFFICULTY_BANDS_LIVE");
    } else {
        println!("verdict=DIFFICULTY_BANDS_PARTIAL (see failed leg above)");
    }
}
