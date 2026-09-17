//! Iter-281 probe — polarity-claim bias horizon-integral (session-2 debt).
//!
//! The social bias is `0.01 × ActiveTension_count × social_value` where the
//! count is an integral over the ENTIRE run: claims are pushed per catalyst
//! and only removed on reconciliation, with no timestamp. The audited bias
//! band (≤0.03 × social_value, i275 re-derivation) was measured at 1–2K
//! counts (mean 1.25–2.75/agent). If the count grows with horizon, the bias
//! leaves the audited band — unbounded accretion on a horizon-integral.
//!
//! Measured here: per-agent ActiveTension count + implied bias at
//! 1K/2K/5K/10K/20K (seed 42, N=12) plus multi-seed 20K.

use mindstrata_development::polarity::PolarityState;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn tension_recent(sim: &Simulation, now: u64) -> usize {
    let cutoff = now.saturating_sub(1000);
    sim.agents
        .iter()
        .flat_map(|a| a.polarity_claims.iter())
        .filter(|c| c.polarity == PolarityState::ActiveTension && c.created_tick >= cutoff)
        .count()
}

fn main() {
    for horizon in [1_000_i64, 2_000, 5_000, 10_000, 20_000] {
        let config = SimConfig {
            seed: 42,
            max_ticks: horizon as u64,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(horizon as u64);
        let mut total = 0usize;
        let mut max = 0usize;
        let mut tension = 0usize;
        for a in &sim.agents {
            let n = a.polarity_claims.len();
            total += n;
            max = max.max(n);
            tension += a
                .polarity_claims
                .iter()
                .filter(|c| c.polarity == PolarityState::ActiveTension)
                .count();
        }
        // Post-fix: the bias counts only claims inside the 1000-tick window.
        let recent = tension_recent(&sim, horizon as u64);
        println!(
            "t={horizon:>6}  claims_total={total:>5}  tension={tension:>4}  recent_tension={recent:>3}  bias_all={:.4}  bias_recent={:.4}",
            0.01 * tension as f64 / 12.0,
            0.01 * recent as f64 / 12.0,
        );
    }
}
