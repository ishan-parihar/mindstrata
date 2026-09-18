//! i290 — Era V WP-K observability in-vivo (PLAN_DC3 §4 i290).
//!
//! WP-K landed three surfaces: (1) `export_stage_lines` KosmOS-frontmatter
//! export in `snapshot.rs` (R7: altitude claims cite the canon scale), (2)
//! per-quadrant pathology means on `MetricsSnapshot` (R6: per-quadrant
//! transition-trace inputs), (3) the TUI `pathology_panel` longitudinal view.
//! This harness proves the surfaces produce real, non-degenerate data on the
//! golden trajectory at the two horizons where collective stages are known to
//! move (1K pre-gate; 20K past the WP-J governance/economic gate — i287
//! evidence: stage 6.0 natural, 8 genesis memes).

use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    // Horizon A: 1K — pre-coupling baseline (golden snapshot horizon).
    // Horizon B: 20K — past the WP-J gate; metrics sampled every 100 ticks.
    for horizon in [1_000u64, 20_000u64] {
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: horizon,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.populate();
        for t in 0..horizon {
            sim.tick();
            if (t + 1) % 100 == 0 || t + 1 == horizon {
                let m = sim.metrics_snapshot();
                println!(
                    "h={} t={} q(1/2/3/4)=({:.3} {:.3} {:.3} {:.3}) stage_max={:.2}",
                    horizon,
                    t + 1,
                    m.q1_dark_addiction,
                    m.q2_dark_allergy,
                    m.q3_golden_addiction,
                    m.q4_golden_allergy,
                    m.collective_stage_max,
                );
            }
        }
        let lines = mindstrata_sim::snapshot::export_stage_lines(&sim.collective_field);
        let deepest = lines
            .iter()
            .max_by(|a, b| a.stage.total_cmp(&b.stage))
            .expect("canon lines");
        let moved = lines.iter().filter(|l| l.stage > 0.0).count();
        println!(
            "h={} deepest: {} ({} {}) stage {:.2} | moved lines: {}/{}",
            horizon,
            deepest.line,
            deepest.kind,
            deepest.quadrant,
            deepest.stage,
            moved,
            lines.len(),
        );
    }
}
