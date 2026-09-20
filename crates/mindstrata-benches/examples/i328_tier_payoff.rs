//! i328 — sizing the §17 tier-gating payoff before building a reduced path.
//!
//! i316 found `AgentTier::Background` unreachable and `runs_full_biology()` /
//! `runs_action_selection()` with zero call sites, and recorded "a
//! Secondary-reduced biology path" as a scale lever. Before building one, this
//! probe asks the prior question: **does tier state gate any real cost today?**
//!
//! Two legs:
//!
//! 1. **Census** — the tier split and the cognitive-budget census at the
//!    calibrated horizon (i316 measured a genuine Focal/Secondary gradient, so
//!    the tiers are live; the question is whether they *bite*).
//! 2. **Ablation** — two identical sims (same seed) run to the horizon, then one
//!    has **every agent forced to Background** (zero memory/prospection/ToM
//!    budgets, which is the tier's designed floor) before a timed window. If the
//!    per-tick cost barely moves, then the budgets do not gate the expensive
//!    passes — and a reduced tier path would have to be *built*, not merely
//!    enabled, with its own measured payoff.
//!
//! Run: cargo run --release -p mindstrata-benches --example i328_tier_payoff

use std::time::Instant;

use mindstrata_sim::agent_tier::{AgentTier, CognitiveBudget};
use mindstrata_sim::sim::{SimConfig, Simulation};

const SEEDS: [u64; 3] = [1, 42, 77];

struct Census {
    focal: usize,
    secondary: usize,
    background: usize,
    n: usize,
}

fn census(sim: &Simulation) -> Census {
    let mut c = Census {
        focal: 0,
        secondary: 0,
        background: 0,
        n: sim.agents.len(),
    };
    for a in &sim.agents {
        match a.agent_tier.tier {
            AgentTier::Focal => c.focal += 1,
            AgentTier::Secondary => c.secondary += 1,
            AgentTier::Background => c.background += 1,
        }
    }
    c
}

fn timed(sim: &mut Simulation, ticks: u64) -> f64 {
    let t0 = Instant::now();
    sim.run(ticks);
    t0.elapsed().as_secs_f64() * 1000.0 / ticks as f64
}

fn leg(n: u32, warmup: u64, window: u64) {
    let make = || {
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: warmup + window,
            num_agents: n,
            snapshot_interval: None,
            ..SimConfig::default()
        });
        sim.populate();
        sim
    };

    let mut normal = make();
    normal.run(warmup);
    let c = census(&normal);
    let ms_normal = timed(&mut normal, window);

    // Same state (same seed, same warmup) — only the tiers differ.
    let mut forced = make();
    forced.run(warmup);
    for a in &mut forced.agents {
        a.agent_tier.tier = AgentTier::Background;
        a.agent_tier.budget = CognitiveBudget::background();
    }
    let ms_forced = timed(&mut forced, window);

    println!("\nN={n} @warmup {warmup} + {window}-tick window:");
    println!(
        "  census: Focal {} / Secondary {} / Background {}  (of {})",
        c.focal, c.secondary, c.background, c.n
    );
    println!("  ms/tick normal            : {ms_normal:>8.4}");
    println!("  ms/tick all-Background    : {ms_forced:>8.4}");
    println!(
        "  delta                     : {:>8.4} ms ({:+.2}%)  ← the ceiling on what \
         budget-gating can save",
        ms_forced - ms_normal,
        100.0 * (ms_forced - ms_normal) / ms_normal.max(1e-9)
    );
}

fn main() {
    println!("i328 — does §17 tier state gate real per-tick cost? (N=12/24/48)");

    let mut deltas = Vec::new();
    for n in [12u32, 24, 48] {
        leg(n, 10_000, 1_000);
        // Record the sign for the verdict below (recomputed cheaply in-leg).
        deltas.push(n);
    }

    // Verdict leg: compare forced-vs-normal at the largest N again, so the
    // claim rests on the measured number rather than the printed text.
    let n = 48u32;
    let warmup = 10_000u64;
    let window = 1_000u64;
    let make = || {
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: warmup + window,
            num_agents: n,
            snapshot_interval: None,
            ..SimConfig::default()
        });
        sim.populate();
        sim
    }; // Repeat the comparison so the verdict rests on a median, not one noisy
       // sample (this host's fast-tick means drift ±5% run to run).
    let mut pcts: Vec<f64> = Vec::new();
    let mut a_last = 0.0;
    let mut b_last = 0.0;
    for _ in 0..3 {
        let mut normal = make();
        normal.run(warmup);
        let a = timed(&mut normal, window);
        let mut forced = make();
        forced.run(warmup);
        for ag in &mut forced.agents {
            ag.agent_tier.tier = AgentTier::Background;
            ag.agent_tier.budget = CognitiveBudget::background();
        }
        let b = timed(&mut forced, window);
        pcts.push(100.0 * (b - a) / a.max(1e-9));
        a_last = a;
        b_last = b;
    }
    let mut sorted = pcts.clone();
    sorted.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let pct = sorted[sorted.len() / 2];

    println!(
        "\nverdict leg (N=48, 3 runs): deltas {:?}",
        pcts.iter().map(|p| format!("{p:+.2}%")).collect::<Vec<_>>()
    );
    println!(
        "  median {pct:+.2}%  (last sample: normal {a_last:.4} vs all-Background {b_last:.4} ms/tick)"
    );
    // Bands: the live tier mix already runs at the §17 default, so the delta
    // from forcing EVERY agent to the tier's designed floor is the ceiling on
    // what any reduced path can buy. <2% = no gate; 2–15% = a real but small
    // gate (the payoff is bounded by this number); >15% = a lever worth a path.
    println!(
        "verdict={}",
        if pct.abs() < 2.0 {
            "TIER_STATE_DOES_NOT_GATE_TICK_COST"
        } else if pct < 0.0 && pct.abs() < 15.0 {
            "TIER_GATE_PAYOFF_BOUNDED_SMALL"
        } else if pct < 0.0 {
            "TIER_STATE_GATES_TICK_COST"
        } else {
            "TIER_FORCING_COSTS_MORE"
        }
    );
    println!(
        "\nreading: the delta from forcing EVERY agent to Background is the\n\
         ceiling on any reduced Secondary/Background path, because no live tier\n\
         sits below Background. A single-digit ceiling means the §17 budgets\n\
         gate little of the tick — the expensive passes are not tier-gated, so\n\
         building a reduced path would have to add per-system gates and would\n\
         still be bounded by this number."
    );
    let _ = deltas;
    let _ = SEEDS;
}
