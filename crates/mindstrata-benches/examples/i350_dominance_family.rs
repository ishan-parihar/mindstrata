//! i350 — dominance→violence family under the contacted-only trust fold.
//!
//! The i350 re-contract de-diluted `social_trust` (now a contacted-row mean,
//! fallback = stranger prior 0.4). That feeds `trust_pacify_factor` in the
//! clans escalation decision, so the dominance→violence channel's aggregate
//! flipped at N=12 (i347 family: dominant 103 vs subordinate 81 → post-fold
//! 72 vs 74). This probe re-measures the full 12-seed family plus a wider
//! seed pool, both totals and per-seed direction, to decide whether the
//! aggregate-direction contract survives on the measured pool (§4.1: fix the
//! family, never the seed).
//!
//! Run: cargo run --release -p mindstrata-benches --example i350_dominance_family

use mindstrata_core::conflict::ConflictKind;
use mindstrata_core::event::SimEvent;
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn violence_of(sim: &Simulation) -> usize {
    sim.recent_events(10_000_000)
        .iter()
        .filter(|e| {
            matches!(
                e,
                SimEvent::ConflictOccurred {
                    kind: ConflictKind::Violence,
                    ..
                }
            )
        })
        .count()
}

fn run_world(dependence: f64, seed: u64, horizon: u64) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    for a in &mut sim.agents {
        for r in &mut a.relationship_v2s {
            r.dependence = Fixed::from_f64(dependence);
        }
    }
    sim.run(horizon);
    sim
}

fn main() {
    let horizon: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(5000);
    let seeds = [1u64, 2, 3, 5, 7, 11, 13, 17, 21, 29, 33, 42, 44, 47, 55, 99];
    let mut dom_total = 0usize;
    let mut sub_total = 0usize;
    let mut higher = 0usize;
    let mut equal = 0usize;
    let mut lower = 0usize;
    let mut dom_trust_sum = 0.0f64;
    let mut sub_trust_sum = 0.0f64;
    let mut rows = Vec::new();
    for seed in seeds {
        let dominant = run_world(0.0, seed, horizon);
        let subordinate = run_world(1.0, seed, horizon);
        let d = violence_of(&dominant);
        let s = violence_of(&subordinate);
        dom_total += d;
        sub_total += s;
        match d.cmp(&s) {
            std::cmp::Ordering::Greater => higher += 1,
            std::cmp::Ordering::Equal => equal += 1,
            std::cmp::Ordering::Less => lower += 1,
        }
        // The feedback signature: escalation → interaction → contacted trust →
        // pacification. Mean CONTACTED trust per world should be higher where
        // more escalation happened.
        let contacted_trust = |sim: &Simulation| -> f64 {
            let (sum, n) = sim
                .agents
                .iter()
                .flat_map(|a| a.relationship_v2s.iter())
                .fold((0.0f64, 0usize), |(s, n), r| {
                    if r.is_contacted() {
                        (s + r.trust.to_f64(), n + 1)
                    } else {
                        (s, n)
                    }
                });
            if n == 0 {
                0.0
            } else {
                sum / n as f64
            }
        };
        dom_trust_sum += contacted_trust(&dominant);
        sub_trust_sum += contacted_trust(&subordinate);
        rows.push(format!(
            "  seed {seed:>3}: dominant {d:>3} vs subordinate {s:>3} (Δ {:+})",
            d as i64 - s as i64
        ));
    }
    for r in &rows {
        println!("{r}");
    }
    println!(
        "\naggregate @{horizon}: dominant {dom_total} vs subordinate {sub_total} (Δ {:+}) · higher {higher}/16, equal {equal}, lower {lower}",
        dom_total as i64 - sub_total as i64
    );
    println!(
        "mean contacted trust: dominant {dom_trust_sum:.4} vs subordinate {sub_trust_sum:.4} (feedback signature)"
    );
}
