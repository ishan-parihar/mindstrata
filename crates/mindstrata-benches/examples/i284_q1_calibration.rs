//! Q1 dark-addiction calibration probe — measure growth/decay/ceiling
//! constants against i269 trajectory. Goal: pin the constants in
//! `dynamics.rs` so the field is calibrated, not just `CALIBRATION-PENDING(AP3)`.
//!
//! IC-4 key=value report for the CA-1..CA-8 calibration audit:
//! `ic4_q1_*` per (growth, decay, ceiling, cpt) combo, and a chosen
//! `ic4_q1_recommended_*` set that best matches i269's measured
//! trajectory (0.0 → 0.49 at 20K).
//!
//! The sim calls `slot.step(metabolism, admitted, &params)` once per
//! catalyst event, where `admitted = gate.admit_quantized(magnitude)`.
//! We model this as N per-tick catalysts at unit magnitude.

use mindstrata_development::dynamics::{Metabolism, OperatorParams, QuadrantState};

fn main() {
    println!("ic4_seed=42");
    println!("ic4_horizon=20000");
    println!("ic4_target_20k=0.490");

    let growths = [0.001_f64, 0.005, 0.01, 0.05, 0.10];
    let decays = [0.0005_f64, 0.001, 0.005, 0.01];
    let ceilings = [0.5_f64, 0.7, 1.0];
    let catalysts_per_ticks = [0.008_f64, 0.05, 0.5, 1.0, 2.0];

    let mut best_combo: Option<(f64, f64, f64, f64, f64)> = None;
    let mut best_err = f64::INFINITY;

    for &cpt in &catalysts_per_ticks {
        for &g in &growths {
            for &d in &decays {
                for &c in &ceilings {
                    let params = OperatorParams {
                        growth: g,
                        decay: d,
                        ceiling: c,
                    };
                    let mut q = QuadrantState::neutral();
                    let mut at_5k = 0.0;
                    let mut at_10k = 0.0;
                    let mut at_20k = 0.0;
                    for t in 0..20_000 {
                        // Fractional cpt: emit one catalyst of magnitude cpt
                        // every tick (deterministic, matches the one-step-per-
                        // catalyst contract).
                        q = q.step(Metabolism::Addiction, cpt, &params);
                        if t == 4_999 {
                            at_5k = q.intensity;
                        }
                        if t == 9_999 {
                            at_10k = q.intensity;
                        }
                        if t == 19_999 {
                            at_20k = q.intensity;
                        }
                    }
                    let err = (at_20k - 0.49).abs();
                    if err < best_err {
                        best_err = err;
                        best_combo = Some((cpt, g, d, c, at_20k));
                    }
                    if (g, d, c, cpt) == (0.05, 0.02, 1.0, 1.0)
                        || (g, d, c, cpt) == (0.10, 0.01, 1.0, 0.5)
                        || (g, d, c, cpt) == (0.20, 0.02, 1.0, 0.5)
                        || (g, d, c, cpt) == (0.10, 0.02, 1.0, 0.5)
                    {
                        println!(
                            "ic4_combo cpt={cpt:.1} g={g:.3} d={d:.3} c={c:.2} 5k={at_5k:.4} 10k={at_10k:.4} 20k={at_20k:.4} err={err:.4}"
                        );
                    }
                }
            }
        }
    }
    if let Some((cpt, g, d, c, v)) = best_combo {
        println!("ic4_q1_recommended_cpt={cpt:.4}");
        println!("ic4_q1_recommended_growth={g:.4}");
        println!("ic4_q1_recommended_decay={d:.4}");
        println!("ic4_q1_recommended_ceiling={c:.4}");
        println!("ic4_q1_recommended_20k={v:.4}");
        println!("ic4_q1_recommendation_err={best_err:.4}");
    }
    let (g, d, c) = (0.05, 0.02, 1.0);
    let params = OperatorParams {
        growth: g,
        decay: d,
        ceiling: c,
    };
    let mut q = QuadrantState::neutral();
    for t in 0..20_000 {
        q = q.step(Metabolism::Addiction, 1.0, &params);
        if t == 19_999 {
            println!("ic4_default_20k={:.4}", q.intensity);
        }
    }
    println!("verdict=Q1_CALIBRATION_DONE");
}
