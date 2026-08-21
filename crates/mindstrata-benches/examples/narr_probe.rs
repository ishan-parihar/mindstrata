//! Iteration 181 probe v2 — NarrativeIdentity post-fix behavior.
//!
//! Iteration 181 fixed two things for the RWR row-8 residual (the six
//! narrative scripts had ZERO decision consumers and write-only saturation):
//!   1. `decay_scripts` — proportional pull toward the birth-narrative
//!      envelope, stopping the 0.5→0.99 / 0.2→0.84 / 0.5→1.00 saturation.
//!   2. `stress_resilience_factor` — the first script consumer: the plan's
//!      "narrative feeds ... resilience" wired into the per-tick stress input.
//!
//! This probe verifies: (a) the scripts now drift but stay bounded (no
//! saturation), (b) the resilience factor is live and directional (redemptive
//! stories buffer stress), and (c) focal-only containment — secondary/minor
//! agents never run the narrative block, so their factor must be exactly 1.0.
//!
//! Run with: `cargo run -p mindstrata-benches --example narr_probe --release`

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::agent_tier::AgentTier;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn run(seed: u64, ticks: u64) -> Simulation {
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
    let rate = Fixed::from_f64(0.15);
    // Defaults: redemption 0.5, contamination 0.2, victimhood 0.2,
    // heroism 0.5, chosenness 0.1, shame 0.1, coherence 0.6.
    for (seed, ticks) in [
        (42u64, 2000u64),
        (42, 5000),
        (42, 10000),
        (1, 2000),
        (7, 2000),
    ] {
        let sim = run(seed, ticks);
        let n = sim.agents.len() as f64;
        let (mut red, mut cont, mut vict, mut hero, mut chos, mut shame) =
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut factors: Vec<f64> = Vec::new();
        for a in &sim.agents {
            red += a.narrative.redemption_script.to_f64();
            cont += a.narrative.contamination_script.to_f64();
            vict += a.narrative.victimhood_script.to_f64();
            hero += a.narrative.heroism_script.to_f64();
            chos += a.narrative.chosenness_script.to_f64();
            shame += a.narrative.shame_script.to_f64();
            factors.push(
                a.narrative
                    .stress_resilience_factor(rate, a.embodied.endocrine.stress.chronic_load)
                    .to_f64(),
            );
        }
        let f_mean: f64 = factors.iter().sum::<f64>() / n;
        let f_min = factors.iter().copied().fold(f64::INFINITY, f64::min);
        let f_max = factors.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        println!(
            "seed{seed}/{ticks}: red={:.3} cont={:.3} vict={:.3} hero={:.3} chos={:.3} shame={:.3} | \
             factor mean={f_mean:.4} min={f_min:.4} max={f_max:.4}",
            red / n, cont / n, vict / n, hero / n, chos / n, shame / n
        );
    }

    // Focal-only containment: non-focal agents never run the narrative block,
    // so their scripts stay at the birth envelope → factor exactly 1.0.
    let sim = run(42, 10000);
    let mut off_nonfocal = 0;
    let mut off_focal = 0;
    let mut focal_factors: Vec<f64> = Vec::new();
    for a in &sim.agents {
        let f = a
            .narrative
            .stress_resilience_factor(rate, a.embodied.endocrine.stress.chronic_load);
        if a.agent_tier.tier == AgentTier::Focal {
            if f != Fixed::ONE {
                off_focal += 1;
            }
            focal_factors.push(f.to_f64());
        } else if f != Fixed::ONE {
            off_nonfocal += 1;
        }
    }
    let ff_mean: f64 = focal_factors.iter().sum::<f64>() / focal_factors.len() as f64;
    println!(
        "seed42/10000 containment: nonfocal_factor_off={off_nonfocal} focal_off={off_focal} focal_mean={ff_mean:.4}"
    );

    // Stress input liveness: mean cognitive.stress and the fear+anger sum
    // the resilience channel scales.
    let sim = run(42, 2000);
    let stress_mean: f64 = sim
        .agents
        .iter()
        .map(|a| a.cognitive.stress.to_f64())
        .sum::<f64>()
        / sim.agents.len() as f64;
    println!("seed42/2000: stress_mean={stress_mean:.3}");

    let _ = Fixed::ZERO;
}
