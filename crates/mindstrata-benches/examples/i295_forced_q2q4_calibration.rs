//! i295 — forced Q2/Q4 calibration: prove the Allergy infrastructure
//! works when Transgression/Grief are forced, and record the
//! trajectory for IC-5 calibration.

use mindstrata_development::dynamics::{Metabolism, OperatorParams, QuadrantState};

fn trajectory(steps: &[(Metabolism, f64)], label: &str) {
    let p = OperatorParams::pending();
    let mut q = QuadrantState::neutral();
    println!("--- {} (pending g=0.05 d=0.02 c=1.0) ---", label);
    for (i, (met, pressure)) in steps.iter().enumerate() {
        q = q.step(*met, *pressure, &p);
        if i < 5 || i % 500 == 0 || i == steps.len() - 1 {
            println!(
                "tick {:4}: pressure {:.2} intensity {:.4}",
                i + 1,
                pressure,
                q.intensity
            );
        }
    }
    println!("final {} intensity {:.4}", label, q.intensity);
}

fn main() {
    // Q2 dark_allergy: forced Transgression pressure 0.5 for 100 ticks,
    // then absence (0.0) for 900 ticks (recoil accumulation).
    let mut steps_q2 = Vec::new();
    for _ in 0..100 {
        steps_q2.push((Metabolism::Allergy, 0.5));
    }
    for _ in 0..900 {
        steps_q2.push((Metabolism::Allergy, 0.0));
    }
    trajectory(&steps_q2, "Q2 dark_allergy forced 0.5×100 then absence×900");

    // Q4 golden_allergy: same pattern with Grief 1.0
    let mut steps_q4 = Vec::new();
    for _ in 0..100 {
        steps_q4.push((Metabolism::Allergy, 1.0));
    }
    for _ in 0..900 {
        steps_q4.push((Metabolism::Allergy, 0.0));
    }
    trajectory(
        &steps_q4,
        "Q4 golden_allergy forced 1.0×100 then absence×900",
    );

    // Q2 with no forced trigger (pure absence from neutral) — should stay 0.
    let steps_q2_neutral: Vec<(Metabolism, f64)> =
        (0..1000).map(|_| (Metabolism::Allergy, 0.0)).collect();
    trajectory(
        &steps_q2_neutral,
        "Q2 neutral absence×1000 (should stay 0.0000)",
    );

    // For nudge calibration: dark_allergy 0.8 → Work suppression
    // Work nudge = -0.08 * pathology_dark, Rest = +0.02
    let high_allergy = 0.8;
    println!(
        "Work nudge at allergy 0.8: {:.4} (rest {:.4})",
        -0.08 * high_allergy,
        0.02 * high_allergy
    );
    println!("verdict=FORCED_Q2Q4_CALIBRATION_DONE");
}
