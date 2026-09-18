//! i292 — dose calibration A3+A4 (PLAN_DC3 §4 i292): the two remaining
//! first-estimate constants, one root cause (initial guesses never swept).
//!
//! A3 — mourning-rite Agape dose 0.6 (`household.rs`, CALIBRATION-PENDING
//! since i285). The Allergy consumption law is
//!     next = I + growth·0.1·headroom·(1−p) − decay·p·I
//! so a rite both suppresses absence-growth and consumes the quadrant in
//! proportion to the dose. Sweep the dose over the production Q4 params
//! (growth 0.03, decay 0.025, ceiling 0.75 — `systems/development.rs`) at
//! the quadrant levels i285 actually measured (I ≈ 0.2–0.7), reporting the
//! per-rite decay fraction. The metabolizer's designed role ("bind grief
//! referents", wave brief WP-H3) needs a measurable-but-bounded effect:
//! decay fraction 0.5–3% per rite. Below that the rite is invisible at
//! fixed-4 resolution; above ~5% a single rite starts overriding the
//! quadrant's own arc.
//!
//! A4 — norm-proposal strength cap 0.6 (`system_norm_proposal`, CALIBRATION-
//! PENDING since i286). In-vivo census at the i286 vivo shape (drought, 50K,
//! Safety gate forced 6.0): if proposed strengths pile AT the cap, the cap
//! is the binding constraint on every proposal (a hidden constant); if they
//! sit below, consensus breadth is the real scale and 0.6 is a harmless
//! guard. Natural-stage control: zero proposals below the band gate.

use mindstrata_development::dynamics::{Metabolism, OperatorParams, QuadrantState};
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

/// Production Q4 params (`systems/development.rs::params_q4`).
const Q4: OperatorParams = OperatorParams {
    growth: 0.03,
    decay: 0.025,
    ceiling: 0.75,
};

fn a3_agape_sweep() {
    println!("=== A3: agape dose sweep (Q4 Allergy, production params) ===");
    println!(
        "{:>6} {:>18} {:>18} {:>18} {:>18}",
        "dose", "dI @ I=0.2", "dI @ I=0.3", "dI @ I=0.5", "dI @ I=0.7"
    );
    for dose in [0.3_f64, 0.45, 0.6, 0.75, 0.9] {
        let cells: Vec<f64> = [0.2, 0.3, 0.5, 0.7]
            .iter()
            .map(|&i0| {
                let q = QuadrantState { intensity: i0 };
                let stepped = q.step(Metabolism::Allergy, dose, &Q4);
                stepped.intensity - i0
            })
            .collect();
        println!(
            "{dose:>6.2} {:>18.5} {:>18.5} {:>18.5} {:>18.5}",
            cells[0], cells[1], cells[2], cells[3]
        );
    }
    // Reference: 500 ticks of pure absence growth (no rite, no catalyst) —
    // the accumulation one rite-cycle band is measured against.
    let mut q = QuadrantState { intensity: 0.2 };
    for _ in 0..500 {
        q = q.step(Metabolism::Allergy, 0.0, &Q4);
    }
    println!(
        "reference: 500-tick no-rite absence growth from I=0.2 → {:.4}",
        q.intensity
    );
}

fn proposed_strengths(sim: &Simulation) -> Vec<f64> {
    sim.norms
        .norms()
        .iter()
        .filter(|n| n.name.starts_with("[proposed:"))
        .map(|n| n.strength.to_f64())
        .collect()
}

fn run_census(label: &str, force_stage: bool) {
    let mut sc = Scenario::drought();
    sc.ticks = 50_000;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    if force_stage {
        for line in sim.collective_field.lines.iter_mut() {
            line.stage = 6.0;
        }
    }
    sim.run(50_000);
    let strengths = proposed_strengths(&sim);
    let at_cap = strengths
        .iter()
        .filter(|&&s| (s - 0.6).abs() < 1e-9)
        .count();
    println!(
        "A4[{label:>12}]: proposals={} at_cap_0.6={at_cap} strengths={:?}",
        strengths.len(),
        strengths
    );
}

fn a4_norm_cap_census() {
    println!("=== A4: norm-proposal strength census (drought 50K) ===");
    run_census("natural", false);
    run_census("forced-6.0", true);
}

fn main() {
    a3_agape_sweep();
    a4_norm_cap_census();
}
