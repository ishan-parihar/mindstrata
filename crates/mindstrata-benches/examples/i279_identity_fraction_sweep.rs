//! Iter-279 probe — Identity-bucket feed fraction sweep (PLAN_DC2 §6.5).
//!
//! The collective step maps one catalyst to one bucket; major conflicts
//! (Violence/Combat/Revolution — the i275 MAJOR discriminator) reshaping the
//! village's self-image is the substrate's own multi-claim reading. This probe
//! sweeps the fractional Identity press from major Threat catalysts to find
//! whether any f lifts Identity max_stage past the genesis gate (2.0) by 20K
//! at N=12, and measures the cost to the Safety trajectory (which pins the
//! i273 genesis timing).
//!
//! FINDING (seed 42, 20K, N=12): 55 major conflicts occur in 20K ticks. Even
//! at f=1.0 the identity press sum is 9.167 → 0.458 accumulated
//! (press_growth 0.05) — HALF the 1.0 needed for stage 2. The fraction is
//! NOT the binding constraint; the diet is (i274 result repeated on a second
//! bucket). No fraction is ratifiable; the sweep records the impossibility
//! and the env affordance stays at f=0 (current behavior).

use mindstrata_core::conflict::ConflictKind;
use mindstrata_core::event::SimEvent;
use mindstrata_development::collective::{bucket_for_line, CollectiveField};
use mindstrata_sim::sim::{SimConfig, Simulation};

/// Genesis gate from WP-I band I: stage >= 2.0 unlocks foundational content.
const GENESIS_GATE: f64 = 2.0;

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

/// Reconstruct the Identity-bucket diet from the run journal: MAJOR conflicts
/// (2 catalysts each: aggressor + target), feuds (2), griefs (1). The press
/// sum at f=1.0 tells us whether ANY fraction can reach the gate by 20K.
fn identity_diet(sim: &Simulation) -> (usize, usize, usize, f64) {
    let mut majors = 0usize;
    let mut feuds = 0usize;
    let mut griefs = 0usize;
    for ev in sim.recent_events(usize::MAX) {
        match ev {
            SimEvent::ConflictOccurred { kind, .. } => {
                if !matches!(kind, ConflictKind::Threat | ConflictKind::Intimidation) {
                    majors += 1;
                }
            }
            SimEvent::FeudFormed { .. } => feuds += 1,
            SimEvent::GriefStruck { .. } => griefs += 1,
            _ => {}
        }
    }
    let n = 12.0_f64;
    // majors × 2 catalysts × f × 1/n each; griefs × 1 × 1/n (uncapped —
    // GriefStruck is Identity's canonical feed per i272).
    let sum = ((majors + feuds) as f64) * 2.0 / n + (griefs as f64) / n;
    (majors, feuds, griefs, sum)
}

fn main() {
    let mut any_genesis = false;
    for f in [0.0_f64, 0.25, 0.5, 1.0, 2.0] {
        std::env::set_var("MINDSTRATA_IDENTITY_PRESS_FRACTION", f.to_string());
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
        sim.run(20_000);
        let stages = per_bucket_max_stage(&sim.collective_field);
        let (majors, feuds, griefs, sum) = identity_diet(&sim);
        if stages[2] >= GENESIS_GATE {
            any_genesis = true;
        }
        println!(
            "f={f:.2}  Identity={:.2}  Safety={:.2}  Relational={:.2}  Meaning={:.2}  \
             | majors={majors} feuds={feuds} griefs={griefs} id_press@f*1.0={sum:.3} \
             acc@0.05={:.3}",
            stages[2],
            stages[1],
            stages[0],
            stages[3],
            sum * 0.05
        );
    }
    if !any_genesis {
        println!("VERDICT: no fraction reaches genesis gate {GENESIS_GATE} by 20K at N=12 — diet-blocked (matches i274 Relational result)");
    }
}
