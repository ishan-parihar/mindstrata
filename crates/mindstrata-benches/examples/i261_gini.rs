//! Iteration 261 — E6 wealth-concentration census: does Gini still climb
//! monotonically, and how many agents sit below the destitution floor?
fn gini(coins: &[f64]) -> f64 {
    let n = coins.len() as f64;
    if n == 0.0 {
        return 0.0;
    }
    let mut sorted = coins.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let total: f64 = sorted.iter().sum();
    if total <= 0.0 {
        return 0.0;
    }
    let cum: f64 = sorted
        .iter()
        .enumerate()
        .map(|(i, c)| (i as f64 + 1.0) * c)
        .sum();
    (2.0 * cum) / (n * total) - (n + 1.0) / n
}
fn census(seed: u64, ticks: u64) -> (f64, usize, f64) {
    let config = mindstrata_sim::sim::SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = mindstrata_sim::Simulation::new(config);
    sim.populate();
    sim.run(ticks);
    let coins: Vec<f64> = sim.agents.iter().map(|a| a.wealth.coin.to_f64()).collect();
    let mut sc2 = coins.clone();
    sc2.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med2 = sc2[sc2.len() / 2];
    let destitute = coins.iter().filter(|&&c| c < 0.10 * med2).count();
    let mean = coins.iter().sum::<f64>() / coins.len() as f64;
    (gini(&coins), destitute, mean)
}
fn main() {
    council_debug(7);
    for seed in [42u64, 7, 13] {
        let (g5k, _, _) = census(seed, 5000);
        let (g10k, _, _) = census(seed, 10_000);
        let (g20k, d20k, m20k) = census(seed, 20_000);
        let (g50k, d50k, m50k) = census(seed, 50_000);
        println!(
            "seed {seed}: gini 5K={g5k:.3} 10K={g10k:.3} 20K={g20k:.3} 50K={g50k:.3} | \
             relief-recipients 20K={d20k} 50K={d50k} | mean 50K={m50k:.1}"
        );
    }
}
// appended debug census
#[allow(dead_code)]
fn council_debug(seed: u64) {
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
    for chunk in 0..4 {
        sim.run(5000);
        let coins: Vec<f64> = sim.agents.iter().map(|a| a.wealth.coin.to_f64()).collect();
        let mut sc = coins.clone();
        sc.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = sc[sc.len() / 2];
        for inst in &sim.institutions {
            if matches!(
                inst.kind,
                mindstrata_sim::institutions::InstitutionKind::Council
            ) {
                println!(
                    "  @{:>5}: council members={} treasury={:.1} median={med:.1}",
                    (chunk + 1) * 5000,
                    inst.members.len(),
                    inst.treasury.to_f64()
                );
            }
        }
    }
}
