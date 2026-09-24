//! i401 — the mechanism behind the attachment-liveness shift.
//!
//! `attachment_separation_distress_coupling_is_live_after_tuning` (seed 42,
//! N=48, tick 5000) failed at "the coupling must be live population-wide"
//! after the v1 trust/affection WRITE was deleted: 34–39/48 (the i185/i191/
//! i381 band) → 27/48. The count of agents that carry *nonzero* distress is a
//! ratio over `partnered`, so it moves for two very different reasons:
//!
//! * the **denominator** grows (more partnerships formed), which is a liveness
//!   gain, not a loss; or
//! * the **zeros** grow because more couples are *co-resident* — the i185 note
//!   already recorded that a fully co-resident couple never separates, so zero
//!   distress there is the CORRECT distance-driven behaviour, not a dead
//!   coupling.
//!
//! This probe prints the parts separately, plus the partner-distance
//! distribution, so the re-anchor decision has a mechanism rather than a band.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i401_attachment_mechanism`

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn main() {
    println!("i401 — attachment liveness: denominator, zeros, co-residency, distance\n");

    for seed in [42u64, 7, 11] {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: 5000,
            world_width: 24,
            world_height: 24,
            num_agents: 48,
            snapshot_interval: None,
        });
        sim.populate();
        sim.run(5000);

        let partnered: Vec<usize> = (0..sim.agents.len())
            .filter(|i| sim.agents[*i].partner.is_some())
            .collect();
        let n_partnered = partnered.len();
        let nonzero = partnered
            .iter()
            .filter(|i| sim.agents[**i].attachment.separation_distress > Fixed::ZERO)
            .count();
        let sum: f64 = partnered
            .iter()
            .map(|i| sim.agents[*i].attachment.separation_distress.to_f64())
            .sum();
        let max: f64 = partnered
            .iter()
            .map(|i| sim.agents[*i].attachment.separation_distress.to_f64())
            .fold(0.0f64, f64::max);

        // Co-residency: partners sharing a home site (the i185 "correct zero").
        let mut coresident = 0usize;
        let mut coresident_zero = 0usize;
        let mut separated = 0usize;
        let mut separated_nonzero = 0usize;
        let mut dist_sum = 0.0f64;
        for i in &partnered {
            let Some(p) = sim.agents[*i].partner else {
                continue;
            };
            let d = sim.agents[*i]
                .position
                .manhattan_distance(&sim.agents[p].position);
            dist_sum += f64::from(d);
            let zero = sim.agents[*i].attachment.separation_distress == Fixed::ZERO;
            let same_home = sim.agents[*i].home_site.is_some()
                && sim.agents[*i].home_site == sim.agents[p].home_site;
            if same_home {
                coresident += 1;
                if zero {
                    coresident_zero += 1;
                }
            } else {
                separated += 1;
                if !zero {
                    separated_nonzero += 1;
                }
            }
        }
        let mean_dist = if n_partnered == 0 {
            0.0
        } else {
            dist_sum / n_partnered as f64
        };

        println!("seed {seed}:");
        println!("  partnered            {n_partnered}/48");
        println!(
            "  nonzero distress     {nonzero}/{}  ({:.0}%)",
            n_partnered,
            if n_partnered == 0 {
                0.0
            } else {
                100.0 * nonzero as f64 / n_partnered as f64
            }
        );
        println!(
            "  mean {:.4}  max {:.4}",
            if n_partnered == 0 {
                0.0
            } else {
                sum / n_partnered as f64
            },
            max
        );
        println!("  co-resident couples  {coresident} (of which zero-distress {coresident_zero})");
        println!("  separated couples    {separated} (of which nonzero {separated_nonzero})");
        println!("  mean partner distance {mean_dist:.2}\n");
    }
}
