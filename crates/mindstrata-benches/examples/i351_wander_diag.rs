//! i351 — why does the Wander driver never fire in vivo? (part 2)
//! Dumps the realized winner-minus-Wander gap distribution when needs are
//! quiet, and the components of the winner's utility at those decisions.
use mindstrata_sim::sim::decision_census;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    decision_census::enable();
    decision_census::reset();
    // i351: horizon from args (default 2000) — the 20K leg is the
    // displacement check against the i346 baseline (Work 39.70% @2K).
    let horizon: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(2000);
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: horizon,
        world_width: 32,
        world_height: 32,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(horizon);
    let r = decision_census::report();
    println!(
        "decisions {} · utility samples {} · Wander wins {} mean-loss {:.4} max {:.4}",
        r.total(),
        r.utility_samples,
        r.wander.wins,
        r.wander.mean_loss(r.utility_samples),
        r.wander.max
    );
    // share of decisions by action
    for (i, name) in decision_census::ACTION_NAMES.iter().enumerate() {
        if r.actions[i] > 0 {
            println!(
                "  {name:>10}: {:>6} ({:.2}%)",
                r.actions[i],
                r.actions[i] as f64 / r.total() as f64 * 100.0
            );
        }
    }
    // i351 part 3: the quiet-window arena — where the driver actually competes.
    println!(
        "\nquiet windows: {} of {} samples ({:.1}%)",
        r.quiet_samples,
        r.utility_samples,
        r.quiet_samples as f64 / r.utility_samples.max(1) as f64 * 100.0
    );
    println!(
        "quiet Wander gap: wins {} mean-loss {:.4} max {:.4}",
        r.quiet_wander.wins,
        r.quiet_wander.mean_loss(r.quiet_samples),
        r.quiet_wander.max
    );
    if r.quiet_samples > 0 {
        println!(
            "quiet winner utility: mean {:.4} max {:.4}",
            r.quiet_winner_sum / r.quiet_samples as f64,
            r.quiet_winner_max
        );
        println!("quiet winners by action:");
        for (i, name) in decision_census::ACTION_NAMES.iter().enumerate() {
            if r.quiet_winner_actions[i] > 0 {
                println!(
                    "  {name:>10}: {:>6} ({:.1}%)",
                    r.quiet_winner_actions[i],
                    r.quiet_winner_actions[i] as f64 / r.quiet_samples as f64 * 100.0
                );
            }
        }
        // §4.2 sizing: sweep the driver coefficient OFFLINE over the same
        // realized quiet samples — wins(c) = #{pre-gap ≤ pressure × c}.
        let coefs = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 15.0, 20.0];
        println!("\noffline coefficient sweep (share of quiet windows where Wander wins):");
        for (c, wins) in decision_census::quiet_sweep(&coefs) {
            println!(
                "  c={c:>5.1}: {:>6} wins ({:.2}%)",
                wins,
                wins as f64 / r.quiet_samples as f64 * 100.0
            );
        }
    }
}
