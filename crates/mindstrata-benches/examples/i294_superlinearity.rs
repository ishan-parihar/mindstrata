//! i294 — Superlinearity probe at N=192 (PLAN_DC3 §4 i294 / charter DC3-P0 §5.2).
//!
//! The i283 budget table measured ≈N^1.4 growth (122 → 1088 → 5961 µs/tick at
//! N=12/48/96) and *attributed* it to interaction-driven event volume. That
//! attribution was inferred from totals, never measured per-unit. This probe
//! separates the two candidate causes without touching sim code:
//!
//! 1. **Interaction volume** (healthy): events/tick grows with N because more
//!    agents emit more happenings; cost PER EVENT stays ~constant.
//! 2. **Algorithmic accident** (hot-path): cost per event rises with N — some
//!    pass does O(N)-or-worse work per event (or an O(N²)+ pass grows its
//!    per-unit cost), i.e. the scaling exponent of µs/event > 0.
//!
//! Discriminators measured here:
//!   α_total  — log-log slope of µs/tick vs N (the headline).
//!   α_volume — slope of events/tick vs N (volume growth).
//!   α_cost   — slope of µs/event vs N. α_cost ≈ 0 ⇒ superlinearity is pure
//!              volume; α_cost ≥ 0.15 ⇒ accident suspect, named via counters.
//!   Density leg — N=192 at world 32×32 vs 64×64: a large delta means
//!              density-dependent scanning (perception/pathing over crowded
//!              tiles), distinct from relationship-volume costs.
//!
//! Reference rows from charter §1 (i283, i274 method, populate included):
//!   N=12: 122.4 · N=48: 1087.6 · N=96: 5960.6 µs/tick.
//!
//! Run: cargo run -p mindstrata-benches --release --example i294_superlinearity

use mindstrata_sim::sim::{SimConfig, Simulation};
use std::time::Instant;

#[derive(Default)]
struct Volume {
    agents: usize,
    events: usize,
    claims: usize,
    rels: usize,
    memes: usize,
}

fn measure(n: u32, world: u32, ticks: u64, seed: u64, reps: u32) -> (f64, Volume) {
    // min-of-N: timing noise is strictly additive, so the minimum across
    // repeats is the robust estimator (mean ingests scheduler/thermal spikes;
    // the N=192 single-run spread measured ±30% on identical work).
    let mut best: Option<(f64, Volume)> = None;
    for _ in 0..reps {
        let (us, v) = measure_once(n, world, ticks, seed);
        if best.as_ref().is_none_or(|b| us < b.0) {
            best = Some((us, v));
        }
    }
    best.expect("reps >= 1")
}

fn measure_once(n: u32, world: u32, ticks: u64, seed: u64) -> (f64, Volume) {
    // Timer spans new+populate+run, matching the i274/charter-i283 method
    // (populate amortized over the horizon) for apples-to-apples rows.
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: world,
        world_height: world,
        num_agents: n,
        snapshot_interval: None,
    };
    let t = Instant::now();
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(ticks);
    let us_per_tick = t.elapsed().as_micros() as f64 / ticks as f64;

    // The event buffer is append-only (charter §3), so the whole journal is
    // the per-run event volume.
    let events = sim.recent_events(usize::MAX).len();
    let claims: usize = sim.agents.iter().map(|a| a.polarity_claims.len()).sum();
    let rels: usize = sim.agents.iter().map(|a| a.relationship_v2s.len()).sum();
    let v = Volume {
        agents: sim.agents.len(),
        events,
        claims,
        rels,
        memes: sim.meme_registry.memes.len(),
    };
    (us_per_tick, v)
}

/// Least-squares slope of y vs x in log-log space.
fn loglog_slope(pairs: &[(f64, f64)]) -> f64 {
    let n = pairs.len() as f64;
    let (mut sx, mut sy, mut sxx, mut sxy) = (0.0, 0.0, 0.0, 0.0);
    for (x, y) in pairs {
        let (lx, ly) = (x.ln(), y.ln());
        sx += lx;
        sy += ly;
        sxx += lx * lx;
        sxy += lx * ly;
    }
    (n * sxy - sx * sy) / (n * sxx - sx * sx)
}

fn main() {
    let ticks: u64 = 2_000;
    let seed: u64 = 42;
    let ns = [12_u32, 48, 96, 192];

    println!("i294 superlinearity — release, {ticks} ticks, seed {seed}, world 32×32");
    println!("charter-i283 reference (populate included): N12=122.4 N48=1087.6 N96=5960.6 µs/tick");
    println!();

    let mut rows: Vec<(u32, f64, Volume)> = Vec::new();
    let mut prev: Option<(u32, f64)> = None;
    for &n in &ns {
        // Fewer reps at large N to bound wall time (see the table note).
        let reps = if n >= 96 { 2 } else { 3 };
        let (us, v) = measure(n, 32, ticks, seed, reps);
        let ev_per_tick = v.events as f64 / ticks as f64;
        let us_per_event = if v.events > 0 {
            us / ev_per_tick
        } else {
            f64::NAN
        };
        print!(
            "N={n:>3}  {us:>9.1} µs/tick  agents={:<3} events/tick={:>7.1}  µs/event={:>6.2}  rels={:>6} claims={:>6} memes={}",
            v.agents, ev_per_tick, us_per_event, v.rels, v.claims, v.memes
        );
        if let Some((pn, pu)) = prev {
            let n_ratio = n as f64 / pn as f64;
            let c_ratio = us / pu;
            print!("  | vs N={pn}: cost ×{c_ratio:.2} for ×{n_ratio:.1} N");
        }
        println!();
        rows.push((n, us, v));
        prev = Some((n, us));
    }

    // ── Attribution ────────────────────────────────────────────────────────
    let cost: Vec<(f64, f64)> = rows.iter().map(|(n, us, _)| (*n as f64, *us)).collect();
    let volume: Vec<(f64, f64)> = rows
        .iter()
        .map(|(n, _, v)| (*n as f64, v.events as f64 / ticks as f64))
        .collect();
    let per_event: Vec<(f64, f64)> = rows
        .iter()
        .map(|(n, us, v)| {
            let ept = v.events as f64 / ticks as f64;
            (*n as f64, if v.events > 0 { us / ept } else { f64::NAN })
        })
        .collect();
    let rels_vol: Vec<(f64, f64)> = rows
        .iter()
        .map(|(n, _, v)| (*n as f64, v.rels as f64))
        .collect();

    let a_total = loglog_slope(&cost);
    let a_volume = loglog_slope(&volume);
    let a_cost = loglog_slope(&per_event);
    let a_rels = loglog_slope(&rels_vol);

    println!();
    println!(
        "α_total(µs/tick)      = {a_total:.3}   (interaction-healthy band ≈ 1.0–2.0; charter-i283 fit ≈ 1.4)"
    );
    println!(
        "α_volume(events/tick) = {a_volume:.3}   (pure interaction volume ⇒ this is the engine)"
    );
    println!(
        "α_cost(µs/event)      = {a_cost:.3}   (≈0 volume-only; ≥0.15 algorithmic-accident suspect)"
    );
    println!("α_rels (R vs N)       = {a_rels:.3}   (quadratic structural floor R=N(N−1) ⇒ 2.0)");

    if a_cost < 0.15 {
        println!(
            "VERDICT: superlinearity is INTERACTION-VOLUME — cost per event flat; no hot-path addition warranted."
        );
    } else {
        println!(
            "VERDICT: ALGORITHMIC-ACCIDENT SUSPECT (α_cost={a_cost:.3}) — cost per event grows with N."
        );
        println!(
            "  Suspect naming via counters: claims/agent and rels/agent growth mark the O(claims)² salience filter and O(R) decay passes (charter §3)."
        );
    }

    // ── Density leg: same N, half the crowding (min-of-2 each) ────────────
    let (us32, _) = measure(192, 32, 1_000, seed, 2);
    let (us64, _) = measure(192, 64, 1_000, seed, 2);
    let delta = (us32 - us64) / us32 * 100.0;
    println!();
    println!(
        "DENSITY leg (N=192, 1K ticks): world 32×32 = {us32:.1} µs/tick, 64×64 = {us64:.1} µs/tick, delta {delta:+.1}%"
    );
    if delta.abs() > 10.0 {
        println!(
            "  Density-sensitive cost present (>10%): crowding-dependent scanning is a real term — note in hot-path list."
        );
    } else {
        println!("  Density-insensitive (≤10%): tile-crowding is not a scaling term.");
    }

    // ── Phase attribution: which scheduler cadence carries the residual? ───
    // Per-tick timing bucketed by the phase-classes the tick satisfies
    // (scheduler.rs). A pass that only runs on Daily+ boundaries shows up as
    // a large daily mean with a small fast mean. Per-tick samples are noisy,
    // so compare population means across thousands of samples.
    println!();
    println!("PHASE attribution at N=192 (2K ticks, per-tick timing bucketed by cadence):");
    let n = 192_u32;
    let config = SimConfig {
        seed,
        max_ticks: 2_000,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let mut daily: Vec<f64> = Vec::new();
    let mut weekly: Vec<f64> = Vec::new();
    let mut quincent: Vec<f64> = Vec::new();
    let mut fast: Vec<f64> = Vec::new();
    for t in 1..=2_000_u64 {
        let s = Instant::now();
        sim.tick();
        let us = s.elapsed().as_micros() as f64;
        if t % 144 == 0 {
            daily.push(us);
        } else if t % 1008 == 0 || t % 500 == 0 || t % 4320 == 0 {
            weekly.push(us);
        } else if t % 100 == 0 || t % 12 == 0 || t % 10 == 0 {
            quincent.push(us);
        } else {
            fast.push(us);
        }
    }
    // Warmup-biased bucket: tick 144 is still cache-cold relative to steady
    // state — drop the first daily sample from the comparison.
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    let (d_mean, f_mean) = (mean(&daily[1.min(daily.len())..]), mean(&fast));
    println!(
        "  fast-only ticks:      n={:>5}  mean {:>9.1} µs/tick",
        fast.len(),
        f_mean
    );
    println!(
        "  centum/duodeca/deca:  n={:>5}  mean {:>9.1} µs/tick",
        quincent.len(),
        mean(&quincent)
    );
    println!(
        "  weekly/quincent:      n={:>5}  mean {:>9.1} µs/tick",
        weekly.len(),
        mean(&weekly)
    );
    println!(
        "  daily boundary:       n={:>5}  mean {:>9.1} µs/tick  (first sample dropped)",
        daily.len().saturating_sub(1),
        d_mean
    );
    let daily_excess_pct = if f_mean > 0.0 {
        (d_mean - f_mean) / f_mean * 100.0
    } else {
        f64::NAN
    };
    println!(
        "  daily excess over fast: {daily_excess_pct:+.1}% per boundary tick; amortized share of total ≈ {:.1}%",
        (d_mean - f_mean) * (daily.len().saturating_sub(1)) as f64 / (f_mean * 2_000.0) * 100.0
    );
}
