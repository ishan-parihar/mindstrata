//! i351 — the trust input to the marriage gate: distribution of
//! `relationships.trust` (the v1 matrix the marriage block reads) at 0/5000
//! ticks, to size why the 0.0002 low-rate run no longer marries on seed 42.
use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    for seed in [42u64, 7, 43] {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: 5000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.populate();
        let mut vals: Vec<f64> = sim.relationships.iter().map(|r| r.trust.to_f64()).collect();
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p = |q: f64| -> f64 { vals[(q * (vals.len() - 1) as f64) as usize] };
        println!(
            "seed {seed} t=0: trust p10 {:.3} p50 {:.3} p90 {:.3} max {:.3}",
            p(0.10),
            p(0.50),
            p(0.90),
            vals[vals.len() - 1]
        );
        sim.run(5000);
        let mut vals: Vec<f64> = sim.relationships.iter().map(|r| r.trust.to_f64()).collect();
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p = |q: f64| -> f64 { vals[(q * (vals.len() - 1) as f64) as usize] };
        let contacted: usize = sim
            .relationships
            .iter()
            .filter(|r| r.interaction_count > 0)
            .count();
        let mean_contact_trust: f64 = {
            let v: Vec<f64> = sim
                .relationships
                .iter()
                .filter(|r| r.interaction_count > 0)
                .map(|r| r.trust.to_f64())
                .collect();
            if v.is_empty() {
                0.0
            } else {
                v.iter().sum::<f64>() / v.len() as f64
            }
        };
        println!(
            "seed {seed} t=5000: trust p10 {:.3} p50 {:.3} p90 {:.3} max {:.3} · contacted {contacted}/{} · mean contacted trust {mean_contact_trust:.3}",
            p(0.10),
            p(0.50),
            p(0.90),
            vals[vals.len() - 1],
            sim.relationships.len()
        );
    }
}
