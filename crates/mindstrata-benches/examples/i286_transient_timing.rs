//! i286 transient-synthesis timing: first tick an Integrated claim exists
//! per scenario (the proposal channel's trigger transient). Per-500-tick
//! census instead of per-tick (600s timeout lesson: 20K sims inside a
//! 20K-iteration loop is 40M tick-pipelines).
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

fn synthesis_census(sc: Scenario, horizon: u64) -> (Option<u64>, usize, u64) {
    let mut sc = sc;
    sc.ticks = horizon;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    let mut first = None;
    let mut max_seen = 0usize;
    let mut last_seen = 0u64;
    let step = 500u64;
    let mut t = step;
    while t <= horizon {
        sim.run(step); // run() advances BY n, not TO t — stepped totals stay 20K
        let now: usize = sim
            .agents
            .iter()
            .map(|a| {
                a.polarity_claims
                    .iter()
                    .filter(|c| {
                        c.polarity == mindstrata_development::polarity::PolarityState::Integrated
                    })
                    .count()
            })
            .sum();
        if now > 0 {
            if first.is_none() {
                first = Some(t);
            }
            max_seen = max_seen.max(now);
            last_seen = t;
        }
        t += step;
    }
    (first, max_seen, last_seen)
}

fn main() {
    for (name, sc) in [
        ("calm", Scenario::calm()),
        ("drought", Scenario::drought()),
        ("collapse", Scenario::collapse()),
    ] {
        let (first, peak, last) = synthesis_census(sc, 20_000);
        println!("{name:>8}: first_integrated@={first:?} peak_concurrent={peak} last_seen@={last}");
    }
}
