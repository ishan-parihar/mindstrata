//! i335 — per-pass LOCAL EXPONENTS, measured on accumulations instead of a
//! single tick.
//!
//! i329 found the surviving ≈N^2.8 term by measuring *local* exponents of the
//! whole tick; i330 attributed it with i330's profiler, which printed ONE tick
//! per run. That single sample carries ~±15% run-to-run noise at N=192 —
//! enough to mis-attribute a term, which is exactly the failure mode i294's
//! single log-log fit paid for.
//!
//! This probe uses the i335 accumulating profiler to read each pass back as a
//! mean over the run, then computes the **local exponent per pass** between
//! consecutive N. That separates the passes that are genuinely pair-shaped
//! (α ≈ 2: per-relationship work, R ∝ N²) from accidental superlinear shapes
//! and from the linear ones (α ≈ 1).
//!
//! Run: cargo run --release -p mindstrata-benches --example i335_pass_exponents

use mindstrata_sim::sim::{SimConfig, Simulation};

const NS: [u32; 4] = [48, 96, 144, 192];
const WARMUP: u64 = 400;
const WINDOW: u64 = 200;

fn measure(n: u32) -> (f64, Vec<(&'static str, f64)>) {
    Simulation::pass_profile_reset();
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: WARMUP + WINDOW,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(WARMUP);
    // Measure only the window: reset after warmup so the numbers describe
    // steady-state ticks, not the populate-heavy start.
    Simulation::pass_profile_reset();
    let t0 = std::time::Instant::now();
    sim.run(WINDOW);
    let us_per_tick = t0.elapsed().as_secs_f64() * 1e6 / WINDOW as f64;
    let per_pass: Vec<(&'static str, f64)> = Simulation::pass_profile_totals()
        .into_iter()
        .map(|(name, ns, samples)| (name, ns as f64 / samples.max(1) as f64 / 1000.0))
        .collect();
    (us_per_tick, per_pass)
}

/// Steady-state structure at N: relationship edges and events per tick.
fn structure(n: u32) -> (usize, f64) {
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: WARMUP,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    let before = sim.event_count();
    sim.run(WARMUP);
    let events = (sim.event_count() - before) as f64 / WARMUP as f64;
    (sim.relationships().len(), events)
}

fn edges_of(n: u32) -> (u32, usize) {
    (n, structure(n).0)
}

/// Baseline density: the i295 charter's N=48 in a 32×32 world.
const BASELINE_DENSITY: f64 = 48.0 / (32.0 * 32.0);

fn structure_sized(n: u32, world: u32) -> (usize, f64, f64) {
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: WARMUP,
        world_width: world,
        world_height: world,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    let before = sim.event_count();
    let t0 = std::time::Instant::now();
    sim.run(WARMUP);
    let us = t0.elapsed().as_secs_f64() * 1e6 / WARMUP as f64;
    let events = (sim.event_count() - before) as f64 / WARMUP as f64;
    (sim.relationships().len(), events, us)
}

/// Fixed-density leg: the world grows with the population, so density (and
/// therefore how much of the village is co-located) is held constant.
fn density_leg() {
    println!("\n=== fixed-DENSITY leg (world scales with N; same density as N=48 in 32x32) ===\n");
    println!(
        "{:>5} {:>6} {:>10} {:>10} {:>12} {:>12}",
        "N", "world", "edges", "edges/N", "events/tick", "µs/tick"
    );
    let mut rows: Vec<(u32, f64, f64)> = Vec::new();
    for n in [48u32, 96, 192] {
        let w = ((n as f64 / BASELINE_DENSITY).sqrt()).round() as u32;
        let (edges, events, us) = structure_sized(n, w);
        println!(
            "{:>5} {:>6} {:>10} {:>10.1} {:>12.1} {:>12.1}",
            n,
            w,
            edges,
            edges as f64 / n as f64,
            events,
            us
        );
        rows.push((n, edges as f64, us));
    }
    let alpha = |a: f64, b: f64| (b / a).ln() / (rows[2].0 as f64 / rows[0].0 as f64).ln();
    println!(
        "\n  edges   alpha (48 -> 192): {:.3}   (fixed-world reading was 2.019)",
        alpha(rows[0].1, rows[2].1)
    );
    println!(
        "  µs/tick alpha (48 -> 192): {:.3}   (fixed-world reading was ~1.78)",
        alpha(rows[0].2, rows[2].2)
    );
    println!(
        "\n  verdict: {}",
        if alpha(rows[0].1, rows[2].1) < 1.4 {
            "DENSITY_IS_THE_GOVERNOR — at constant density the relationship graph is ~linear, so the N² floor is a fixed-world artifact"
        } else {
            "RELATIONSHIP_GRAPH_IS_QUADRATIC_EVEN_AT_CONSTANT_DENSITY"
        }
    );
}
fn main() {
    // Safety: set before ANY sim reads it. `pass_profile_tick` caches the parsed
    // value in a `OnceLock`, so it must be set before the first sim in the
    // process (the density leg below builds sims first).
    std::env::set_var("MINDSTRATA_PROFILE_TICK", "1");
    density_leg();
    println!("\n=== fixed-WORLD leg (the i295 charter: 32x32 for every N) ===\n");
    println!(
        "i335 — per-pass local exponents (seed 42, 32x32, warmup {WARMUP}, window {WINDOW} ticks)"
    );
    println!(
        "each pass value is the MEAN ns/tick over the window (accumulated, not a single sample)\n"
    );

    let mut rows: Vec<(u32, f64, Vec<(&'static str, f64)>)> = Vec::new();
    for n in NS {
        let (us, passes) = measure(n);
        let (edges, events_per_tick) = structure(n);
        println!(
            "N={n:<4} {us:>9.1} µs/tick | relationship edges {edges:>6} ({:.2}/agent, /N² {:.3}) | events/tick {events_per_tick:>6.1}",
            edges as f64 / n as f64,
            edges as f64 / (n as f64 * n as f64),
        );
        rows.push((n, us, passes));
    }

    let base = edges_of(NS[0]);
    let top = edges_of(NS[3]);
    println!(
        "\nrelationship-edge local exponent 48 -> 192: {:.3}  (2.0 = every pair related; the per-edge passes are then inherently N²)",
        (top.1 as f64 / base.1 as f64).ln() / (NS[3] as f64 / NS[0] as f64).ln()
    );

    // Union of pass names, preserving the largest N's ordering.
    let mut names: Vec<&'static str> = rows
        .last()
        .map(|r| r.2.iter().map(|(k, _)| *k).collect())
        .unwrap_or_default();
    for (_, _, passes) in &rows {
        for (k, _) in passes {
            if !names.contains(k) {
                names.push(k);
            }
        }
    }

    let value = |row_i: usize, name: &str| -> Option<f64> {
        rows[row_i]
            .2
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| *v)
    };

    println!(
        "\n{:<20} {:>10} {:>10} {:>10} {:>8} ver",
        "pass (mean µs/tick)", "N=48", "N=96", "N=192", "alpha"
    );
    let mut superlinear: Vec<(&'static str, f64)> = Vec::new();
    for name in &names {
        let v: Vec<Option<f64>> = (0..rows.len()).map(|i| value(i, name)).collect();
        if v.iter().any(Option::is_none) {
            continue;
        }
        let (a, b) = (v[0].unwrap(), v[3].unwrap());
        if b < 5.0 {
            continue; // below a microsecond at the top of the envelope
        }
        let alpha = (b / a.max(1e-9)).ln() / (NS[3] as f64 / NS[0] as f64).ln();
        println!(
            "{:<20} {:>10.1} {:>10.1} {:>10.1} {:>8.3} {}",
            name,
            a,
            v[1].unwrap(),
            b,
            alpha,
            match alpha {
                x if x >= 1.85 => "superlinear",
                x if x >= 1.4 => "between",
                _ => "linear",
            }
        );
        if alpha >= 1.4 {
            superlinear.push((name, alpha));
        }
    }

    let total_alpha = (rows[3].1 / rows[0].1).ln() / (NS[3] as f64 / NS[0] as f64).ln();
    println!("\nwhole-tick local exponent 48 -> 192: {total_alpha:.3}");

    println!("\nsuperlinear passes (alpha >= 1.4) at the top of the envelope:");
    for (name, alpha) in &superlinear {
        println!("  {name:<20} alpha {alpha:.3}");
    }
    let top = rows[3].2.iter().max_by(|a, b| a.1.total_cmp(&b.1)).unwrap();
    println!(
        "\nlargest single pass at N=192: {} at {:.1} µs/tick ({:.0}% of the tick)",
        top.0,
        top.1,
        top.1 / 1000.0 / rows[3].1 * 100.0
    );
}
