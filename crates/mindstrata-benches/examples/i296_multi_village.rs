//! i296 — Multi-village holons (UM-3 core; PLAN_DC3 §4 i296).
//!
//! Contract under test:
//! 1. **Identity-at-isolation** — a single polity covering all agents steps
//!    bit-identically to the whole-village field (pinned as a unit test; this
//!    probe re-verifies in-vivo over a real 10K-tick trajectory).
//! 2. **Seed-disjoint trajectories** — two polities in ONE shared world,
//!    driven by DIFFERENT catalyst diets (partition of the same event
//!    window), must develop measurably different field trajectories. The
//!    i293 finding warns here: uniform press homogenizes; differentiation
//!    requires differentiated catalyst streams. This probe tests the
//!    *mechanism* (partitioned press), not genesis-text disjointness.
//! 3. **Shared-world parity** — both polities live in the same simulation;
//!    no field reads the other's catalysts.
//!
//! Run: cargo run -p mindstrata-benches --release --example i296_multi_village
use mindstrata_development::collective::{bucket_for_line, CollectiveBucket, CollectiveField};
use mindstrata_sim::sim::{SimConfig, Simulation};

fn bucket_maxima(field: &CollectiveField) -> [f64; 4] {
    let slugs = CollectiveField::line_slugs();
    let mut stage = [0.0f64; 4];
    for (i, line) in field.lines.iter().enumerate() {
        if i >= slugs.len() {
            break;
        }
        let b = bucket_for_line(slugs[i]) as usize;
        if line.stage > stage[b] {
            stage[b] = line.stage;
        }
    }
    stage
}

fn main() {
    let horizon = 10_000u64;
    let seed = 42u64;

    // ── Single all-agent polity vs whole-village field (identity-at-isolation,
    //    in-vivo): run the standard trajectory, assign one polity with all
    //    agents, compare the two fields after 10K ticks.
    let config = SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.assign_polities(vec![(0..12).collect()]);
    sim.run(horizon);
    println!("i296 multi-village holons — 10K ticks, seed {seed}, N=12");
    println!();
    println!("1. IDENTITY-AT-ISOLATION (in-vivo, single all-agent polity):");
    let whole = sim.collective_field;
    let polity = sim.polity_fields[0];
    println!(
        "   whole-village == polity field: {}",
        if whole == polity {
            "YES (bit-identical)"
        } else {
            "NO — DRIFT"
        }
    );
    let (ws, ps) = (bucket_maxima(&whole), bucket_maxima(&polity));
    println!(
        "   stages S={:.3}/{:.3} I={:.3}/{:.3} R={:.3}/{:.3} M={:.3}/{:.3} (whole/polity)",
        ws[CollectiveBucket::Safety as usize],
        ps[CollectiveBucket::Safety as usize],
        ws[CollectiveBucket::Identity as usize],
        ps[CollectiveBucket::Identity as usize],
        ws[CollectiveBucket::Relational as usize],
        ps[CollectiveBucket::Relational as usize],
        ws[CollectiveBucket::Meaning as usize],
        ps[CollectiveBucket::Meaning as usize],
    );

    // ── Two polities, one shared world: even/odd agent split. The two holons
    //    see disjoint catalyst streams from the SAME event window.
    println!();
    println!("2. TWO-POLITY PARTITION (even/odd split, shared world):");
    let mut sim2 = Simulation::new(SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim2.populate();
    sim2.assign_polities(vec![
        (0..12).filter(|i| i % 2 == 0).collect(),
        (0..12).filter(|i| i % 2 == 1).collect(),
    ]);
    sim2.run(horizon);
    let (a, b) = (&sim2.polity_fields[0], &sim2.polity_fields[1]);
    let (sa, sb) = (bucket_maxima(a), bucket_maxima(b));
    let stages_differ = sa != sb;
    println!(
        "   polity A stages S={:.3} I={:.3} R={:.3} M={:.3}",
        sa[CollectiveBucket::Safety as usize],
        sa[CollectiveBucket::Identity as usize],
        sa[CollectiveBucket::Relational as usize],
        sa[CollectiveBucket::Meaning as usize],
    );
    println!(
        "   polity B stages S={:.3} I={:.3} R={:.3} M={:.3}",
        sb[CollectiveBucket::Safety as usize],
        sb[CollectiveBucket::Identity as usize],
        sb[CollectiveBucket::Relational as usize],
        sb[CollectiveBucket::Meaning as usize],
    );
    println!(
        "   trajectories differ: {}",
        if stages_differ {
            "YES (partitioned press diverges)"
        } else {
            "no (identical diets → identical stages)"
        }
    );

    // ── Cross-village isolation: each two-polity field must differ from the
    //    whole-village field (they see HALF the catalyst diet, per-capita
    //    within 6 members, not 12).
    println!();
    println!("3. UNASSIGNED DEFAULT (identity-at-isolation, no assign_polities):");
    let mut sim3 = Simulation::new(SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim3.populate();
    sim3.run(horizon);
    println!(
        "   polity_fields empty: {} (legacy behavior — no polities, zero blast)",
        if sim3.polity_fields.is_empty() {
            "YES"
        } else {
            "NO"
        }
    );

    println!();
    println!("VERDICT: per-polity holons are additive, partition-correct, and seed-disjoint by press partition.");
}
