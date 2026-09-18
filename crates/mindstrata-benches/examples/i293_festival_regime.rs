//! i293 — UM-2 diet realism at operator horizons (PLAN_DC3 §4 i293, A5).
//!
//! The i278 20K gate measured PARTIAL (only Safety generates); i283 corrected
//! the pacing model (per-event press 1/n × 0.05) and PASSED UM-2 at ~100K —
//! Relational ~54K, Identity ~44K stage-2 arrival. The plan names the
//! accelerator: a festival-dense cultural regime (recurring Relational+Identity
//! press). This probe runs that regime and asks the plan's exact question:
//! does ≥3-bucket generation close by 20K WITHOUT a production magnitude knob?
//!
//! Method: probe-side forcing only (§4.4-compliant — same discipline as the
//! forced-stage probes). Every duodeca (12 ticks — the ritual boundary), each
//! Relational and Identity line takes ONE festival step through the production
//! `CollectiveLineState::step` at pressure = attendance fraction. No production
//! constant is touched. Verdict rule: ≥3 buckets generating by 20K in the
//! festival legs → the i278 PARTIAL closes as diet-pacing (mechanism was never
//! dead); N=12 natural-gait ~100K stands as the designed cultural horizon.

use mindstrata_development::collective::{
    bucket_for_line, CollectiveBucket, CollectiveField, CollectiveParams,
};
use mindstrata_sim::sim::{SimConfig, Simulation};

fn festival_step(field: &mut CollectiveField, attendance: f64) {
    let slugs = CollectiveField::line_slugs();
    let params = CollectiveParams::pending();
    for (i, line) in field.lines.iter_mut().enumerate() {
        let Some(slug) = slugs.get(i) else { break };
        if matches!(
            bucket_for_line(*slug),
            CollectiveBucket::Relational | CollectiveBucket::Identity
        ) {
            *line = line.step(attendance, &params);
        }
    }
}

fn census(sim: &Simulation) -> (usize, Vec<String>) {
    let mut generated = 0usize;
    let mut buckets = Vec::new();
    for m in &sim.meme_registry.memes {
        if m.description.contains("[genesis:") {
            generated += 1;
            let b = m
                .description
                .split("[genesis:")
                .nth(1)
                .map(|r| r.split(':').next().unwrap_or("").to_string())
                .unwrap_or_default();
            if !buckets.contains(&b) {
                buckets.push(b);
            }
        }
    }
    buckets.sort();
    (generated, buckets)
}

fn run(seed: u64, attendance: Option<f64>) {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: 20_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    for h in [5_000_u64, 10_000, 20_000] {
        while sim.current_tick().as_u64() < h {
            sim.tick();
            if let Some(a) = attendance {
                if sim.current_tick().as_u64() % 12 == 0 {
                    festival_step(&mut sim.collective_field, a);
                }
            }
        }
        let (n, buckets) = census(&sim);
        println!(
            "seed {seed} att={:?} @{h:>5}: generated={n:>2} buckets={buckets:?}",
            attendance
        );
    }
}

fn run_collect(seed: u64, attendance: Option<f64>) -> std::collections::HashSet<String> {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: 20_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    while sim.current_tick().as_u64() < 20_000 {
        sim.tick();
        if let Some(a) = attendance {
            if sim.current_tick().as_u64() % 12 == 0 {
                festival_step(&mut sim.collective_field, a);
            }
        }
    }
    sim.meme_registry
        .memes
        .iter()
        .filter(|m| m.description.contains("[genesis:"))
        .map(|m| m.description.clone())
        .collect()
}

fn jaccard(a: &std::collections::HashSet<String>, b: &std::collections::HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let inter = a.intersection(b).count();
    inter as f64 / (a.len() + b.len() - inter) as f64
}

/// Variable festival schedule: each village's festival seasons arrive on a
/// seed-derived subset of duodecas (40% active, attendance 0.5) — the honest
/// proxy for event-driven festivals (real regimes press through seed-specific
/// Bond/Grief catalyst streams, so press timing carries the seed signal).
fn run_varied(seed: u64) -> std::collections::HashSet<String> {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: 20_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    let mut lcg = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
    while sim.current_tick().as_u64() < 20_000 {
        sim.tick();
        if sim.current_tick().as_u64() % 12 == 0 {
            lcg = lcg
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            if (lcg >> 33) % 100 < 40 {
                festival_step(&mut sim.collective_field, 0.5);
            }
        }
    }
    let (n, buckets) = census(&sim);
    println!("seed {seed} varied-festival @20000: generated={n:>2} buckets={buckets:?}");
    sim.meme_registry
        .memes
        .iter()
        .filter(|m| m.description.contains("[genesis:"))
        .map(|m| m.description.clone())
        .collect()
}

fn main() {
    // Control: natural gait (i283 baseline — only Safety inside 20K).
    run(42, None);
    // Festival-dense: half-village attendance per duodeca.
    for s in [42_u64, 43, 44] {
        run(s, Some(0.5));
    }
    // Full-village festival season (upper bound of the regime family).
    run(42, Some(1.0));

    // Disjointness re-run (the i278/UM-2 gate's other half): festival-dense
    // rosters across seeds must stay seed-disjoint. Uniform press first
    // (the degenerate corner), then the variable-schedule regime.
    let rosters: Vec<(u64, _)> = [42_u64, 43, 44]
        .iter()
        .map(|&s| (s, run_collect(s, Some(0.5))))
        .collect();
    let mut jac = Vec::new();
    for i in 0..rosters.len() {
        for j in (i + 1)..rosters.len() {
            let v = jaccard(&rosters[i].1, &rosters[j].1);
            jac.push(v);
            println!(
                "uniform-festival jaccard {}-{}: {:.3} (|A|={} |B|={})",
                rosters[i].0,
                rosters[j].0,
                v,
                rosters[i].1.len(),
                rosters[j].1.len()
            );
        }
    }
    println!(
        "UNIFORM-FESTIVAL MEAN JACCARD: {:.3} (i283 natural-100K was 0.158; control 1.000)",
        jac.iter().sum::<f64>() / jac.len() as f64
    );

    let varied: Vec<(u64, _)> = [42_u64, 43, 44]
        .iter()
        .map(|&s| (s, run_varied(s)))
        .collect();
    let mut jac = Vec::new();
    for i in 0..varied.len() {
        for j in (i + 1)..varied.len() {
            let v = jaccard(&varied[i].1, &varied[j].1);
            jac.push(v);
            println!(
                "varied-festival jaccard {}-{}: {:.3} (|A|={} |B|={})",
                varied[i].0,
                varied[j].0,
                v,
                varied[i].1.len(),
                varied[j].1.len()
            );
        }
    }
    println!(
        "VARIED-FESTIVAL MEAN JACCARD: {:.3}",
        jac.iter().sum::<f64>() / jac.len() as f64
    );
}
