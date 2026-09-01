use mindstrata_development::dynamics::{Metabolism, OperatorParams, QuadrantState};
fn main() {
    let pressures = [0.008_f64, 0.01, 0.05, 0.5, 1.0];
    let growths = [0.01_f64, 0.05, 0.1, 0.2, 0.5, 1.0];
    let decays = [0.0001_f64, 0.0005, 0.001, 0.005, 0.01, 0.05];
    let ceilings = [0.5_f64, 1.0];
    let mut best: Option<(f64, f64, f64, f64, f64, f64)> = None;
    let mut best_err = f64::INFINITY;
    for &p in &pressures {
        for &g in &growths {
            for &d in &decays {
                for &c in &ceilings {
                    let params = OperatorParams {
                        growth: g,
                        decay: d,
                        ceiling: c,
                    };
                    let mut q = QuadrantState::neutral();
                    for _ in 0..20_000 {
                        q = q.step(Metabolism::Addiction, p, &params);
                    }
                    let err = (q.intensity - 0.49).abs();
                    if err < best_err {
                        best_err = err;
                        best = Some((p, g, d, c, q.intensity, err));
                    }
                }
            }
        }
    }
    if let Some((p, g, d, c, v, e)) = best {
        println!("best_pressure={p} g={g} d={d} c={c} v={v:.4} err={e:.4}");
    }
}
