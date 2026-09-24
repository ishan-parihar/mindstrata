//! Iter-276 probe — WP-G2: compositional content generation (substrate §5).
//!
//! The Era III content pipeline is `domain × gross-referent × stance ×
//! line-signature → generated items`. What exists today (i273 genesis):
//! domain = collective bucket, but the "referent" variable is only the LINE
//! SLUG (abstract), never a WORLD entity (site/institution). The substrate's
//! type-check law: every subtle item cites ≥1 gross entity within exactly
//! one domain.
//!
//! Questions this probe answers before implementation:
//! 1. What gross referents exist at calibration horizons (site/institution
//!    counts and names) — is there enough variety to compose with?
//! 2. Do seeded memes cite any gross entity today, or are they all abstract?
//! 3. At a 20K horizon, how many (bucket, stage-epoch) genesis events exist,
//!    and which institutions/sites were live then — i.e., what would
//!    referent-grounded generation have produced?

use mindstrata_development::collective::{bucket_for_line, CollectiveField};
use mindstrata_sim::sim::{SimConfig, Simulation};

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

fn main() {
    // Part 1 — gross referent inventory at populate time.
    let config = SimConfig {
        seed: 42,
        max_ticks: 20_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let site_kinds: Vec<String> = sim
        .world
        .sites
        .iter()
        .map(|s| format!("{:?}/{}", s.kind, s.name))
        .collect();
    println!("SITES ({}):", site_kinds.len());
    for s in site_kinds.iter().take(12) {
        println!("  {s}");
    }
    let inst: Vec<String> = sim
        .institutions
        .iter()
        .map(|i| format!("{:?}/{}", i.kind, i.name))
        .collect();
    println!("INSTITUTIONS ({}):", inst.len());
    for i in &inst {
        println!("  {i}");
    }

    // Part 2 — do seeded memes cite gross entities?
    println!("SEEDED MEMES:");
    for m in &sim.meme_registry.memes {
        println!("  [{:?}]: {}", m.content_type, m.description);
    }

    // Part 3 — 20K horizon: genesis events + final referent inventory.
    sim.run(20_000);
    let stages = per_bucket_max_stage(&sim.collective_field);
    println!(
        "20K STAGES: Relational={:.2} Safety={:.2} Identity={:.2} Meaning={:.2}",
        stages[0], stages[1], stages[2], stages[3]
    );
    let genesis: Vec<&String> = sim
        .meme_registry
        .memes
        .iter()
        .filter(|m| m.description.contains("[genesis:"))
        .map(|m| &m.description)
        .collect();
    println!("GENESIS MEMES ({}):", genesis.len());
    for g in &genesis {
        println!("  {g}");
    }
    let inst_now = sim.institutions.len();
    let site_now = sim.world.sites.len();
    println!("FINAL REFERENTS: sites={site_now} institutions={inst_now}");
}
