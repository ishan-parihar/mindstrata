//! DC-1 STORY 12-13 polarity reconciliation probe — measures the
//! natural distribution of `(referent, line)` collisions per agent
//! and counts how many would be eligible for reconciliation under
//! the v1 passive-advance policy.
//!
//! Run: cargo run --release -p mindstrata-benches --example i281_polarity_reconcile_probe

use mindstrata_development::polarity::{
    is_active_tension, reconcile_claims, PolarityState, ThreeRealmClaim,
};
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;
use std::collections::HashMap;

const SEEDS: [u64; 6] = [1, 7, 42, 99, 123, 12345];
const TICKS: u64 = 2000;

fn main() {
    let mut total_collisions = 0usize;
    let mut total_reconcile_eligible = 0usize;
    let mut total_active_tension = 0usize;
    let mut total_agents = 0usize;

    for &seed in &SEEDS {
        let config = SimConfig {
            seed,
            max_ticks: TICKS,
            num_agents: 12,
            snapshot_interval: None,
            ..SimConfig::default()
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(TICKS);
        let mut seed_collisions = 0usize;
        let mut seed_reconcile = 0usize;
        for agent in &sim.agents {
            total_agents += 1;
            // Group claims by (referent, line).
            let mut groups: HashMap<(&str, &str), Vec<&ThreeRealmClaim>> = HashMap::new();
            for claim in &agent.polarity_claims {
                let key = (referent_label(claim.referent), claim.line.slug());
                groups.entry(key).or_default().push(claim);
            }
            // For each group with 2+ claims, advance to ActiveTension
            // (mimicking the v1 policy) and try reconciliation.
            for claims in groups.values() {
                if claims.len() < 2 {
                    continue;
                }
                seed_collisions += claims.len() - 1;
                for window in claims.windows(2) {
                    let mut a = *window[0];
                    let mut b = *window[1];
                    a.polarity = PolarityState::ActiveTension;
                    b.polarity = PolarityState::ActiveTension;
                    if is_active_tension(&a, &b) {
                        total_active_tension += 1;
                    }
                    if let Some(_synth) = reconcile_claims(&a, &b) {
                        seed_reconcile += 1;
                    }
                }
            }
        }
        total_collisions += seed_collisions;
        total_reconcile_eligible += seed_reconcile;
        println!("seed={seed} collisions={seed_collisions} reconcile_eligible={seed_reconcile}");
    }
    println!(
        "\ntotal_collisions={total_collisions} reconcile_eligible={total_reconcile_eligible} active_tension_seen={total_active_tension} agents_sampled={total_agents}"
    );
    let rate = if total_collisions > 0 {
        total_reconcile_eligible as f64 / total_collisions as f64
    } else {
        0.0
    };
    println!("reconcile_rate_per_collision={rate:.3}");
    if total_reconcile_eligible > 0 {
        println!("verdict=POLARITY_RECONCILE_LIVE");
    } else {
        println!("verdict=POLARITY_RECONCILE_INERT (no opposite-domain collisions)");
    }

    // DC-1 STORY 12-13: also measure the subtle-claim reconciliation path
    // (same-domain, different `subtle_claim` on same (referent, line)).
    // Per the i281 v1 finding the opposite-domain path is projection-bound
    // (every catalyst projects to a fixed domain); the subtle-claim path
    // is the lowest-impact alternative.
    let seeds2 = [1u64, 7, 42, 99, 123, 12345];
    let mut total_subtle = 0usize;
    for &seed in &seeds2 {
        let config = SimConfig {
            seed,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(2000);
        let mut seed_subtle = 0usize;
        for agent in &sim.agents {
            let n = agent.polarity_claims.len();
            for i in 0..n {
                for j in (i + 1)..n {
                    if mindstrata_development::polarity::is_active_tension(
                        &agent.polarity_claims[i],
                        &agent.polarity_claims[j],
                    ) {
                        if mindstrata_development::polarity::reconcile_subtle(
                            &agent.polarity_claims[i],
                            &agent.polarity_claims[j],
                        )
                        .is_some()
                        {
                            seed_subtle += 1;
                        }
                    }
                }
            }
        }
        total_subtle += seed_subtle;
        println!("subtle_seed={seed} reconcile_eligible={seed_subtle}");
    }
    println!("subtle_reconcile_total={total_subtle}");
    if total_subtle > 0 {
        println!("subtle_verdict=POLARITY_RECONCILE_SUBTLE_LIVE");
    } else {
        println!("subtle_verdict=POLARITY_RECONCILE_SUBTLE_INERT (no (ActiveTension, ActiveTension) same-quartet with different subtle_claim)");
    }
}

fn referent_label(r: mindstrata_development::polarity::GrossReferent) -> &'static str {
    use mindstrata_development::polarity::GrossReferent;
    match r {
        GrossReferent::Event => "Event",
        GrossReferent::Institution => "Institution",
        GrossReferent::Site => "Site",
        GrossReferent::Resource => "Resource",
    }
}
