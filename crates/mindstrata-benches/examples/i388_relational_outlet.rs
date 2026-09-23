//! i388 — the relational outlet: the drive is dominant, the channel is missing.
//!
//! i387's decomposition measured two things that together name this root cause:
//!
//!   1. **The dominant drive is relational.** `Attachment` dominates 34.6% and
//!      `Belonging` 5.5% of arbitrations (calm N=12: `Safety` 25.3%, `Sleep`
//!      21.1%, `Thirst` 13.4%). Two fifths of the utility leg is asked to serve
//!      a relational drive.
//!   2. **Nothing deliberative serves them.** The §8.1.5 urgency boost maps only
//!      `Hunger/Thirst/Sleep/Meaning/Novelty/Play` — so a relational dominance
//!      receives **no boost at all**, exactly the dead-producer class i351
//!      (`Novelty`) and i356 (`Play`) closed. `Socialize`'s own relief term is
//!      ~0.001 because the daily routine relieves `needs.social` before it can
//!      grow (p50 0.005), and the `GoalKind::Socialize` producer's gate (0.7)
//!      sits above the channel's own ceiling (p99 0.196) — dark by construction.
//!
//! This probe sizes the repair before it lands:
//!   * **leg A** — the `needs.social` distribution and the open-rate of candidate
//!     anomaly ratios for a *relative* goal producer (the i381/i382/i383 form:
//!     an agent anomalously deprived **for this population** gets the goal, since
//!     the absolute band gate is above the channel's reach);
//!   * **leg B** — after the change: the census's Socialize selections, the share
//!     of agent-ticks carrying a live Socialize goal, and the social-need
//!     equilibrium (the outlet must not collapse the need to zero).
//!
//! Run: `cargo run --release -p mindstrata-benches --example i388_relational_outlet`

use mindstrata_sim::sim::decision_census;
use mindstrata_sim::sim::{SimConfig, Simulation};

const WARMUP: u64 = 500;
const WINDOW: u64 = 20_000;

fn calm(n: u32, seed: u64) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: WARMUP + WINDOW,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim
}

fn pct(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[idx]
}

/// Candidate anomaly ratios for the relative producer (the i381/i383 form).
const RATIOS: [f64; 6] = [1.25, 1.5, 2.0, 3.0, 5.0, 8.0];

/// Leg A — the channel's reach and the ratio sweep.
fn leg_a(label: &str, n: u32, seed: u64) {
    let mut sim = calm(n, seed);
    sim.run(WARMUP);
    let mut social: Vec<f64> = Vec::new();
    let mut opened = vec![0u64; RATIOS.len()];
    let mut samples = 0u64;
    let mut dominance_social = 0u64;
    let step = 25u64;
    let mut done = 0u64;
    while done < WINDOW {
        sim.run(step);
        done += step;
        let mean_social: f64 = sim
            .agents
            .iter()
            .map(|a| a.needs.social.to_f64())
            .sum::<f64>()
            / sim.agents.len().max(1) as f64;
        for a in &sim.agents {
            let s = a.needs.social.to_f64();
            social.push(s);
            samples += 1;
            for (i, r) in RATIOS.iter().enumerate() {
                if s > mean_social * *r {
                    opened[i] += 1;
                }
            }
            // Which drive the motivation layer makes dominant, same instrument
            // as i387 (the goal layer's outlet must serve the dominant drive).
            if matches!(
                a.motivation.dominant_need,
                mindstrata_sim::psychology::MotiveCategory::Attachment
                    | mindstrata_sim::psychology::MotiveCategory::Belonging
            ) {
                dominance_social += 1;
            }
        }
    }
    social.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    println!("══ {label} (N={n}, seed {seed}) ══");
    println!(
        "needs.social: p50 {:.4} p90 {:.4} p99 {:.4} max {:.4} · mean {:.4}",
        pct(&social, 0.50),
        pct(&social, 0.90),
        pct(&social, 0.99),
        social.last().copied().unwrap_or(0.0),
        social.iter().sum::<f64>() / social.len().max(1) as f64
    );
    print!("open-rate by anomaly ratio (share of agent-ticks):");
    for (i, r) in RATIOS.iter().enumerate() {
        print!(
            "  ×{r} = {:.2}%",
            opened[i] as f64 / samples.max(1) as f64 * 100.0
        );
    }
    println!();
    println!(
        "relational dominance (Attachment|Belonging): {:.2}% of samples",
        dominance_social as f64 / samples.max(1) as f64 * 100.0
    );
    println!();
}

/// Leg B — the post-change census.
fn leg_b(label: &str, n: u32, seed: u64) {
    let mut sim = calm(n, seed);
    sim.run(WARMUP);
    decision_census::reset();
    decision_census::enable();
    sim.run(WINDOW);
    decision_census::disable();
    let r = decision_census::report();
    let total = r.total().max(1) as f64;
    let socialize = r.cross.iter().map(|row| row[4]).sum::<u64>();
    let util_socialize = r.cross[4][4];
    println!("══ {label} — post-change census (N={n}, seed {seed}) ══");
    println!(
        "decisions {} · Socialize {} ({:.2}% of decisions), of which utility {}",
        r.total(),
        socialize,
        socialize as f64 / total * 100.0,
        util_socialize
    );
    println!(
        "utility arbitrations {} · Socialize gap: wins {} (within noise {}), mean loss {:.4}",
        r.utility_samples,
        r.socialize_gap.wins,
        r.socialize_gap.within_noise,
        r.socialize_gap.mean_loss(r.terms_samples)
    );
    let mut goal_social = 0u64;
    let mut n_agents = 0u64;
    for a in &sim.agents {
        n_agents += 1;
        if a.goals
            .iter()
            .any(|g| matches!(g.kind, mindstrata_sim::person::GoalKind::Socialize))
        {
            goal_social += 1;
        }
    }
    println!(
        "live Socialize goals at window end: {goal_social}/{n_agents} agents · social need mean {:.4}",
        sim.agents.iter().map(|a| a.needs.social.to_f64()).sum::<f64>() / n_agents.max(1) as f64
    );
    println!();
}

fn main() {
    println!("i388 — relational outlet (warmup {WARMUP}, window {WINDOW})\n");
    leg_a("reach + ratio sweep", 12, 42);
    leg_a("reach + ratio sweep", 48, 42);
    leg_b("post-change", 12, 42);
    leg_b("post-change", 48, 42);
}
