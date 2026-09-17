//! Iter-280 probe — WP-J: institutional altitude coupling (PLAN_DC2 §6.5).
//!
//! WP-J: "Institution behavior parameters become functions of
//! governance/economic-systems line stages (read-side multipliers,
//! midpoint-neutral)." The groundwork type `InstitutionsMultiplier` (SIM
//! 4.25, inert) gets its live constructor. Questions for the probe:
//!
//! 1. **Zero-blast**: what are the governance/economic-systems stages at the
//!    golden horizons (2K/5K)? If < 2.0 (band I), a band-gated coupling is
//!    provably zero-blast on goldens.
//! 2. **Trajectory**: when does governance cross 2.0 (band II)? That is the
//!    coupling's activation tick.
//! 3. **Behavioral delta**: with the coupling live (env affordance
//!    MINDSTRATA_INSTITUTIONS_COUPLE=1), how do Work-action counts and
//!    grain equilibria shift at 20K vs the uncoupled run? The nudge must
//!    stay in the established 0.02–0.08 class (dread/hope scale), not a
//!    reordering lever.

use mindstrata_development::collective::CollectiveField;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn line_stage(field: &CollectiveField, slug: &str) -> f64 {
    let slugs = CollectiveField::line_slugs();
    for (i, s) in slugs.iter().enumerate() {
        if s.slug() == slug {
            return field.lines[i].stage;
        }
    }
    0.0
}

fn main() {
    let coupled = std::env::var("MINDSTRATA_INSTITUTIONS_COUPLE").is_ok();
    for horizon in [2_000_i64, 5_000, 10_000, 20_000] {
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
        let m = sim.metrics_snapshot();
        println!(
            "t={horizon:>6}  governance={:.3}  economic-systems={:.3}  trades={}  grain={:.1}  feuds={}  events={}",
            line_stage(&sim.collective_field, "governance"),
            line_stage(&sim.collective_field, "economic-systems"),
            m.total_trades,
            m.total_grain,
            m.total_active_feuds,
            m.event_count,
        );
        let _ = coupled;
    }
}
