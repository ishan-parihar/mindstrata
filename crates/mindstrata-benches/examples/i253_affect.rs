//! Iteration 253 — affect distribution census.
fn pct(vals: &mut Vec<f64>, q: f64) -> f64 {
    if vals.is_empty() {
        return 0.0;
    }
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    vals[((vals.len() as f64 * q).ceil() as usize)
        .saturating_sub(1)
        .min(vals.len() - 1)]
}
fn main() {
    for seed in [42u64, 7, 13] {
        let config = mindstrata_sim::sim::SimConfig {
            seed,
            max_ticks: 5000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = mindstrata_sim::Simulation::new(config);
        sim.populate();
        sim.run(5000);
        let mut valence = Vec::new();
        let mut fear = Vec::new();
        let mut joy = Vec::new();
        for a in &sim.agents {
            valence.push(a.affect.valence.to_f64());
            fear.push(a.emotions.fear.to_f64());
            joy.push(a.emotions.joy.to_f64());
        }
        println!(
            "seed {seed}: valence p10={:+.3} med={:+.3} p90={:+.3} | fear p50={:.3} p90={:.3} | joy p50={:.3}",
            pct(&mut valence.clone(), 0.1),
            pct(&mut valence.clone(), 0.5),
            pct(&mut valence.clone(), 0.9),
            pct(&mut fear.clone(), 0.5),
            pct(&mut fear.clone(), 0.9),
            pct(&mut joy, 0.5),
        );
    }
}
