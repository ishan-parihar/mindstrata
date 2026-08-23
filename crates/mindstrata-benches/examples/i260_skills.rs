//! Iteration 260 — E8 skill-saturation census: do person-level skills pin
//! population-wide at exactly 1.0, and does practice frequency differentiate
//! agents? Run before/after the power-law learning change.
fn main() {
    for seed in [42u64, 7, 13] {
        let config = mindstrata_sim::sim::SimConfig {
            seed,
            max_ticks: 20_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = mindstrata_sim::Simulation::new(config);
        sim.populate();
        sim.run(20_000);
        let mut farming = Vec::new();
        let mut trading = Vec::new();
        for a in &sim.agents {
            farming.push(a.skills.farming.to_f64());
            trading.push(a.skills.trading.to_f64());
        }
        let pinned_f = farming.iter().filter(|&&s| s >= 1.0).count();
        let pinned_t = trading.iter().filter(|&&s| s >= 1.0).count();
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        let spread = |v: &[f64]| {
            let m = mean(v);
            (v.iter().map(|s| (s - m) * (s - m)).sum::<f64>() / v.len() as f64).sqrt()
        };
        println!(
            "seed {seed}: farming mean={:.3} sd={:.3} pinned@1.0={}/{} | trading mean={:.3} sd={:.3} pinned={}",
            mean(&farming),
            spread(&farming),
            pinned_f,
            farming.len(),
            mean(&trading),
            spread(&trading),
            pinned_t,
        );
    }
}
