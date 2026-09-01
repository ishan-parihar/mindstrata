use mindstrata_development::dynamics::{Metabolism, OperatorParams, QuadrantState};
fn main() {
    // pending() = g=0.05, d=0.02, c=1.0. Find pressure p such that
    // 20K iterations of `step(Addiction, p, &pending())` yields 0.49.
    let pending = OperatorParams::pending();
    let pressures = [
        0.05_f64, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.5, 2.0,
    ];
    for &p in &pressures {
        let mut q = QuadrantState::neutral();
        for _ in 0..20_000 {
            q = q.step(Metabolism::Addiction, p, &pending);
        }
        println!("pressure={p:5.2} intensity={:.4}", q.intensity);
    }
}
