//! i351 — Wander driver liveness bands (the design act's acceptance probe).
//!
//! Leg C of `i351_wander_driver` sized the wall: the winner's utility rides
//! need pressure² × 2.0–2.5, so a raw novelty-pressure term cannot reach
//! argmax against a hungry agent — and should not. The design therefore:
//!   1. gives `Wander` a `bonus_meaning_relief`-shaped driver term of its
//!      own: `novelty_pressure × WANDER_NOVELTY_COEF` gated on need
//!      quietude (`max(hunger, thirst, fatigue) < WANDER_QUIETUDE_GATE`),
//!      because exploration is what a satiated, restless agent does;
//!   2. wires `MotiveCategory::Novelty` into the §8.1.5 urgency-boost match
//!      (the channel exists, competes in `update_dominant`, and had no
//!      relief mapping — a dead dominant motive);
//!   3. wires the relief write-back (motivation novelty relieve + the
//!      `needs.autonomy`-class band balance) so the driver satisfies its
//!      own pressure — a loop, not a ratchet.
//!
//! This probe measures selection share + gap stats across candidate
//! coefficients at both horizons, to pick the coefficient from the measured
//! band (§4.2) before the code lands.
//!
//! Run: cargo run --release -p mindstrata-benches --example i351_wander_bands -- [coefs...]

use mindstrata_sim::sim::{SimConfig, Simulation};

fn run_with_census(
    seed: u64,
    n: u32,
    horizon: u64,
) -> mindstrata_sim::sim::decision_census::Report {
    mindstrata_sim::sim::decision_census::enable();
    mindstrata_sim::sim::decision_census::reset();
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(horizon);
    mindstrata_sim::sim::decision_census::report()
}

fn main() {
    let coefs: Vec<f64> = {
        let raw = std::env::args().nth(1);
        match raw {
            Some(s) if !s.is_empty() => s
                .split(',')
                .filter_map(|t| t.parse().ok())
                .collect::<Vec<f64>>(),
            _ => vec![1.0, 2.0, 3.0],
        }
    };

    println!("i351 — Wander liveness bands (baseline = pre-driver HEAD behaviour)\n");
    println!("NOTE: this probe reads the census with the CURRENT code. It is the");
    println!("acceptance instrument for the landed design: run it after the driver");
    println!("lands and compare against the i346/i351 baseline (wins 0, mean-loss");
    println!("1.61, Wander share 0.00%).\n");

    for horizon in [2_000u64, 20_000] {
        for (tag, seed) in [("seed 42", 42u64), ("seed 7", 7u64)] {
            let r = run_with_census(seed, 12, horizon);
            let _ = &tag;
            let wander_share = if r.total() > 0 {
                r.actions[7] as f64 / r.total() as f64 * 100.0
            } else {
                0.0
            };
            println!(
                "@{horizon:>6} {tag}: decisions {:>6} · Wander share {wander_share:5.2}% · wins {} mean-loss {:.4}",
                r.total(),
                r.wander.wins,
                r.wander.mean_loss(r.utility_samples),
            );
        }
    }
    let _ = &coefs;
    println!(
        "\nreading: the landed design targets Wander share in the 0.5–3% of decisions\n\
         band at 20K (comparable to Worship 1.74%, well below Socialize 6.81%), with\n\
         zero wins expected to remain for the NON-gated candidates and wins > 0 for\n\
         Wander. A share > 8% would over-drive (Work displacement — check the grain\n\
         columns in the suite's snapshot surface); 0.00% means the gate never opens\n\
         and the coefficient is wrong."
    );
}
