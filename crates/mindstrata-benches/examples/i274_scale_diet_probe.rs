//! Iter-274 probe — DC-3 entry: N≥48 scale survey (PLAN_DC2 next-horizon).
//!
//! Two questions before committing DC-3 engineering effort:
//!
//! 1. **Catalyst-diet asymmetry** (i273 debt): at N=12 only Safety-bucket
//!    genesis fires because Threat events dominate the natural diet. Does
//!    N=48 diversify the diet (more marriages/births → more Bond press →
//!    Relational genesis)?
//! 2. **Perf baseline**: wall-clock per 1K ticks at N=12/48/96 — is the
//!    VecDeque perf ponytail needed before DC-3's larger villages?
//!
//! Genesis is keyed on per-bucket max line stage; per-bucket stage reads use
//! the public `bucket_for_line` affinity (i273). Zero RNG in measurement.
//! World held at 32×32 across all N so comparisons are like-for-like.

use mindstrata_development::collective::{bucket_for_line, CollectiveBucket, CollectiveField};
use mindstrata_sim::sim::{SimConfig, Simulation};
use std::time::Instant;

const ALL_BUCKETS: [CollectiveBucket; 4] = [
    CollectiveBucket::Relational,
    CollectiveBucket::Safety,
    CollectiveBucket::Identity,
    CollectiveBucket::Meaning,
];

fn bucket_name(b: CollectiveBucket) -> &'static str {
    match b {
        CollectiveBucket::Relational => "Relational",
        CollectiveBucket::Safety => "Safety",
        CollectiveBucket::Identity => "Identity",
        CollectiveBucket::Meaning => "Meaning",
    }
}

fn per_bucket_max_stage(field: &CollectiveField) -> [f64; 4] {
    let slugs = CollectiveField::line_slugs();
    let mut out = [0.0_f64; 4];
    for (i, line) in field.lines.iter().enumerate() {
        if i >= slugs.len() {
            break;
        }
        let b = bucket_for_line(slugs[i]) as usize;
        if line.stage > out[b] {
            out[b] = line.stage;
        }
    }
    out
}

fn genesis_count(sim: &Simulation) -> usize {
    sim.meme_registry
        .memes
        .iter()
        .filter(|m| m.description.contains("[genesis:"))
        .count()
}

fn run(n: u32, horizon: u64, seed: u64) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(horizon);
    sim
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--relational") {
        // Iteration-274 post-fix verification: does the RitualPerformed → Bond
        // catalyst route unpin the Relational bucket at N=12 / 20K? (Pre-fix
        // probe: Relational max_stage 1.000 at every N.)
        let sim = run(12, 20_000, 42);
        let stages = per_bucket_max_stage(&sim.collective_field);
        // The event buffer grows unboundedly (ponytail in core.rs), so
        // recent_events(usize::MAX) is the whole journal.
        let rituals = sim
            .recent_events(usize::MAX)
            .iter()
            .filter(|ev| matches!(ev, mindstrata_core::event::SimEvent::RitualPerformed { .. }))
            .count();
        println!("RELATIONAL-POSTFIX  N=12 ticks=20000  ritual_events_in_window={rituals}");
        for (b, s) in ALL_BUCKETS.iter().zip(stages.iter()) {
            println!("    {:>11}: max_stage {s:.3}", bucket_name(*b));
        }
        let g: Vec<&str> = sim
            .meme_registry
            .memes
            .iter()
            .filter(|m| m.description.contains("[genesis:"))
            .map(|m| m.description.as_str())
            .collect();
        println!("    genesis_memes={}", g.len());
        for line in &g {
            println!("    · {line}");
        }
        return;
    }
    // Part 1 — perf baseline at the golden horizon.
    for n in [12_u32, 48, 96] {
        let t = Instant::now();
        let sim = run(n, 2000, 42);
        let dt = t.elapsed();
        println!(
            "PERF  N={n:>3}  2000 ticks  {dt:?}  ({:.1} µs/tick)",
            dt.as_micros() as f64 / 2000.0
        );
        assert_eq!(
            sim.agent_count() as u64,
            n as u64,
            "populate must honor num_agents"
        );
    }

    // Part 2 — genesis differentiation at 10K (genesis fires ~5–8K at N=12).
    for n in [12_u32, 48, 96] {
        let sim = run(n, 10_000, 42);
        let stages = per_bucket_max_stage(&sim.collective_field);
        println!(
            "GENESIS N={n:>3} ticks=10000  genesis_memes={}",
            genesis_count(&sim)
        );
        for (b, s) in ALL_BUCKETS.iter().zip(stages.iter()) {
            println!("    {:>11}: max_stage {s:.3}", bucket_name(*b));
        }
        let g: Vec<&str> = sim
            .meme_registry
            .memes
            .iter()
            .filter(|m| m.description.contains("[genesis:"))
            .map(|m| m.description.as_str())
            .collect();
        for line in &g {
            println!("    · {line}");
        }
    }
}
