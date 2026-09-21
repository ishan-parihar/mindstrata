//! i351 — panic-registration sweep: with the Wander driver live, which
//! pestilence seeds still register moral panics? The belief-charge gate is
//! marginal on several seeds (i343 already re-contracted this family once);
//! the driver's re-pacing of conflict/stress diets can flip marginal seeds.
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

fn main() {
    let seeds = [5u64, 7, 42, 1, 11, 43, 44, 46, 47, 123];
    println!("i351 panic sweep (pestilence @20K, post-driver):");
    for &seed in &seeds {
        let mut sc = Scenario::pestilence();
        sc.seed = seed;
        sc.ticks = 20000;
        let mut s = Simulation::from_scenario(sc);
        s.populate();
        s.run(20000);
        let n = s.moral_panic_registry.panics.len();
        let peak = s
            .moral_panic_registry
            .panics
            .iter()
            .map(|p| p.intensity.to_f64())
            .fold(0.0f64, f64::max);
        println!("seed {seed:>3}: panics {n:>3} · peak intensity {peak:.4}");
    }
}
