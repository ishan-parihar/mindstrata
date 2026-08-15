//! Phase 4 (AP2 §9.2) — neural-like runtime probe.
//! Measures: learned_delta distribution + utility influence (RL liveness),
//! prediction-error magnitudes (for the mandated attention/belief/emotion
//! fold design), association-network activations, script progress, and the
//! three §9.2 predictive-error consumers (novelty_bias, arousal,
//! belief confidence).
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::{SimConfig, Simulation};

/// Sample the three §9.2 predictive-error fold consumers for an agent.
fn sample_folds(sim: &Simulation) -> (f64, f64, usize, f64, usize, f64) {
    // (mean novelty_bias, mean arousal, agents with novelty_bias>0.52,
    //  max arousal, agents with any belief, mean belief confidence)
    let n = sim.agents.len() as f64;
    let mut nb_sum = 0.0f64;
    let mut ar_sum = 0.0f64;
    let mut nb_raised = 0usize;
    let mut ar_max = 0.0f64;
    let mut any_belief = 0usize;
    let mut conf_sum = 0.0f64;
    for a in &sim.agents {
        let nb = a.attention.novelty_bias.to_f64();
        let ar = a.affect.arousal.to_f64();
        nb_sum += nb;
        ar_sum += ar;
        ar_max = ar_max.max(ar);
        if nb > 0.52 {
            nb_raised += 1;
        }
        if !a.beliefs.is_empty() {
            any_belief += 1;
            conf_sum += a.beliefs.iter().map(|b| b.confidence.to_f64()).sum::<f64>()
                / a.beliefs.len() as f64;
        }
    }
    (
        nb_sum / n,
        ar_sum / n,
        nb_raised,
        ar_max,
        any_belief,
        if any_belief > 0 { conf_sum / any_belief as f64 } else { 0.0 },
    )
}

fn run_sim(seed: u64, ticks: u64) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(ticks);
    sim
}

fn main() {
    for seed in [42u64, 7, 99] {
        for ticks in [1000u64, 10_000] {
            let sim = run_sim(seed, ticks);
            let n = sim.agents.len() as f64;
            let mut ld_nonzero = 0usize;
            let mut max_ld = 0.0f64;
            let mut sum_ld = 0.0f64;
            let mut max_err = 0.0f64;
            let mut sum_err = 0.0f64;
            let mut err_over_0_3 = 0usize;
            let mut max_activation = 0.0f64;
            let mut script_agents = 0usize;
            for a in &sim.agents {
                let nl = &a.neural_like;
                // Learned-delta magnitude across the 4 action outcome profiles.
                let profiles = [
                    [Fixed::from_f64(0.8), Fixed::from_f64(0.2), Fixed::from_f64(0.0), Fixed::from_f64(0.0)], // Eat
                    [Fixed::from_f64(0.7), Fixed::from_f64(0.1), Fixed::from_f64(0.1), Fixed::from_f64(0.1)], // Work
                    [Fixed::from_f64(0.2), Fixed::from_f64(0.4), Fixed::from_f64(0.3), Fixed::from_f64(0.1)], // Socialize
                    [Fixed::from_f64(0.0), Fixed::from_f64(0.2), Fixed::from_f64(0.2), Fixed::from_f64(0.6)], // Worship
                ];
                let mut max = 0.0f64;
                for p in profiles {
                    let d = nl.values.learned_delta(p).to_f64().abs();
                    max = max.max(d);
                    sum_ld += d;
                    if d > 1e-9 {
                        ld_nonzero += 1;
                    }
                }
                max_ld = max_ld.max(max);
                let err = nl.expectation.last_prediction_error.to_f64();
                max_err = max_err.max(err);
                sum_err += err;
                if err > 0.3 {
                    err_over_0_3 += 1;
                }
                let act = nl
                    .network
                    .activation
                    .components()
                    .iter()
                    .fold(0.0f64, |m, x| m.max(x.to_f64()));
                max_activation = max_activation.max(act);
                if nl.script.is_some() {
                    script_agents += 1;
                }
            }
            let (nb_mean, ar_mean, nb_raised, ar_max, any_belief, conf_mean) = sample_folds(&sim);
            println!(
                "seed {seed} @{ticks}: learned_delta_mean={:.6} max={:.6} nonzero_agents={}/{} | pred_err_mean={:.4} max={:.4} >0.3:{} | max_activation={:.3} | scripts={} | novelty_bias_mean={:.3} raised>0.52:{} | arousal_mean={:.3} max={:.3} | beliefs_agents={} conf_mean={:.3}",
                sum_ld / (n * 4.0),
                max_ld,
                ld_nonzero,
                sim.agents.len(),
                sum_err / n,
                max_err,
                err_over_0_3,
                max_activation,
                script_agents,
                nb_mean,
                nb_raised,
                ar_mean,
                ar_max,
                any_belief,
                conf_mean
            );
        }
    }

    // Utility-influence differential: abundant-grain vs scarcity worlds.
    // Abundant grain → Eat/Work succeed → learned need_relief rises toward
    // the success profile; scarcity → fewer successes → values stay near the
    // 0.5 prior.
    println!("== RL DIFFERENTIAL (abundant vs scarcity) ==");
    {
        let mut abundant = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 5000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        abundant.populate();
        // Fill every site's grain so Eat/Work always succeed.
        for site in &mut abundant.world.sites {
            for stock in &mut site.inventory {
                if stock.resource_id == 1 {
                    stock.quantity = Fixed::from_f64(500.0);
                }
            }
        }
        abundant.run(5000);
        let abundant_need_relief: f64 = abundant
            .agents
            .iter()
            .map(|a| a.neural_like.values.need_relief.to_f64())
            .sum::<f64>()
            / 12.0;

        let mut scarcity = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 5000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        scarcity.populate();
        // Empty every site's grain so Eat/Work fail.
        for site in &mut scarcity.world.sites {
            for stock in &mut site.inventory {
                if stock.resource_id == 1 {
                    stock.quantity = Fixed::ZERO;
                }
            }
        }
        scarcity.run(5000);
        let scarcity_need_relief: f64 = scarcity
            .agents
            .iter()
            .map(|a| a.neural_like.values.need_relief.to_f64())
            .sum::<f64>()
            / 12.0;
        let (ab_nb, ab_ar, ab_nb_r, ab_ar_max, ab_bel, ab_conf) = sample_folds(&abundant);
        let (sc_nb, sc_ar, sc_nb_r, sc_ar_max, sc_bel, sc_conf) = sample_folds(&scarcity);
        println!(
            "need_relief: abundant={abundant_need_relief:.4} scarcity={scarcity_need_relief:.4} (diverged={})",
            (abundant_need_relief - scarcity_need_relief).abs() > 0.01
        );
        println!(
            "folds abundant: novelty_bias_mean={ab_nb:.3} raised={ab_nb_r} arousal_mean={ab_ar:.3} max={ab_ar_max:.3} beliefs_agents={ab_bel} conf_mean={ab_conf:.3}"
        );
        println!(
            "folds scarcity: novelty_bias_mean={sc_nb:.3} raised={sc_nb_r} arousal_mean={sc_ar:.3} max={sc_ar_max:.3} beliefs_agents={sc_bel} conf_mean={sc_conf:.3}"
        );
        println!(
            "fold divergence: novelty_bias |Δ|={:.3} arousal |Δ|={:.3} belief_conf |Δ|={:.3}",
            (ab_nb - sc_nb).abs(),
            (ab_ar - sc_ar).abs(),
            (ab_conf - sc_conf).abs()
        );
    }
}
