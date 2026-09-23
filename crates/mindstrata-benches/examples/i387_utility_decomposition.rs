//! i387 — the utility decomposition: which family of terms actually decides an arbitration?
//!
//! i380 proved `Socialize` loses the argmax (**0 utility selections in 112 669
//! arbitrations** at N=12, 1 at N=48) and that both obvious single-knob repairs
//! fail (a ×6 gain: 0→0; a ×20 need accrual: 1→9 of ~19 000). What it could not
//! say is **which family of terms** decides the winner, and therefore which term
//! the social candidate is missing rather than merely under-weighting.
//!
//! This probe reads the i387 per-bucket ledger
//! (`mindstrata_sim::sim::decision_census::record_utility_terms`): for every
//! arbitration it accumulates the bucket vector of the winner, of the
//! `Socialize` candidate, and of the **runner-up** (the best non-winning
//! candidate — the bar a loser actually had to clear). Two questions fall out:
//!
//!   * **The winner's composition** — which family carries the arbitration?
//!     If `need` dominates, the winners are pressure-driven and the social
//!     candidate's isolation is structural; if `urgency`/`goal`/`policy` carry
//!     it, the wall is arithmetic the social candidate simply lacks a term for.
//!   * **The Socialize gap's composition** — bucket by bucket, where the
//!     deficit lives. A deficit concentrated in buckets the candidate has no
//!     term for is a **missing-channel** finding (fix = give it the channel);
//!     one concentrated in `need` is a **calibration** finding (fix = re-derive
//!     the coefficient).
//!
//! Run: `cargo run --release -p mindstrata-benches --example i387_utility_decomposition`

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::decision_census::{
    self, MOTIVE_COUNT, MOTIVE_NAMES, UTILITY_TERM_COUNT, UTILITY_TERM_NAMES,
};
use mindstrata_sim::sim::{SimConfig, Simulation};

const WARMUP: u64 = 500;
const WINDOW: u64 = 20_000;

enum World {
    Calm,
    Collapse,
    Pestilence,
}

fn build(w: u32, h: u32, n: u32, seed: u64, ticks: u64, world: World) -> Simulation {
    match world {
        World::Calm => Simulation::new(SimConfig {
            seed,
            max_ticks: ticks,
            world_width: w,
            world_height: h,
            num_agents: n,
            snapshot_interval: None,
        }),
        World::Collapse | World::Pestilence => {
            let mut sc = match world {
                World::Calm => unreachable!(),
                World::Collapse => Scenario::collapse(),
                World::Pestilence => Scenario::pestilence(),
            };
            sc.seed = seed;
            sc.ticks = ticks;
            Simulation::from_scenario(sc)
        }
    }
}

fn leg(label: &str, w: u32, h: u32, n: u32, seed: u64, world: World) {
    let mut sim = build(w, h, n, seed, WARMUP + WINDOW, world);
    sim.populate();
    sim.run(WARMUP);

    decision_census::reset();
    decision_census::enable();
    sim.run(WINDOW);
    decision_census::disable();
    let r = decision_census::report();

    let samples = r.terms_samples.max(1) as f64;
    let arbitrations = r.utility_samples.max(1) as f64;
    println!("══ {label} (N={n}, seed {seed}) ══");
    println!(
        "arbitrations {} · decomposed {} · Socialize gap: wins {} (within noise {}), \
mean loss {:.4}, max {:.4}",
        r.utility_samples,
        r.terms_samples,
        r.socialize_gap.wins,
        r.socialize_gap.within_noise,
        r.socialize_gap.mean_loss(r.terms_samples),
        r.socialize_gap.max,
    );
    println!(
        "{:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "bucket", "winner", "runner", "socialize", "w−soc", "w share"
    );
    // Winner shares are computed over the positive buckets only — a share of a
    // signed sum is meaningless when penalties offset bonuses.
    let winner_pos: f64 = (0..UTILITY_TERM_COUNT)
        .map(|i| r.terms_winner[i].max(0.0))
        .sum::<f64>()
        .max(1e-9);
    for i in 0..UTILITY_TERM_COUNT {
        let w = r.terms_winner[i] / samples;
        let run = r.terms_runner[i] / samples;
        let s = r.terms_socialize[i] / samples;
        let share = if r.terms_winner[i] > 0.0 {
            r.terms_winner[i] / winner_pos
        } else {
            0.0
        };
        println!(
            "{:>12} {:>10.4} {:>10.4} {:>10.4} {:>10.4} {:>9.1}%",
            UTILITY_TERM_NAMES[i],
            w,
            run,
            s,
            w - s,
            share * 100.0
        );
    }
    let _ = arbitrations;

    // i388 input: which drive was dominant. A social-family category that
    // dominates often with no mapped action is a dead producer.
    if r.terms_samples > 0 {
        let total: u64 = r.dominant_counts.iter().sum();
        let mut rows: Vec<(usize, u64)> = (0..MOTIVE_COUNT)
            .map(|i| (i, r.dominant_counts[i]))
            .filter(|&(_, c)| c > 0)
            .collect();
        rows.sort_by_key(|&(_, c)| std::cmp::Reverse(c));
        println!("dominant motive ({} sampled):", total);
        for (i, c) in rows.iter().take(10) {
            println!(
                "    {:>14} {:>9} {:>6.2}%",
                MOTIVE_NAMES[*i],
                c,
                *c as f64 / total as f64 * 100.0
            );
        }
        println!(
            "    dominant pressure: mean {:.4}, max {:.4}",
            r.dominant_pressure_sum / total as f64,
            r.dominant_pressure_max
        );
    }
    println!();
}

fn main() {
    println!("i387 — utility decomposition (buckets as defined in decision_census)");
    println!("warmup {WARMUP}, window {WINDOW}\n");
    leg("calm village", 32, 32, 12, 42, World::Calm);
    leg("calm village", 32, 32, 48, 42, World::Calm);
    leg("calm village", 32, 32, 48, 7, World::Calm);
    leg("collapse town", 46, 46, 48, 42, World::Collapse);
    leg("pestilence town", 46, 46, 48, 7, World::Pestilence);
}
