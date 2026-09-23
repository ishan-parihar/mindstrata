//! i384 — the scarcity→belief-reinforcement directionality pin, re-swept.
//!
//! `neural_like_prediction_error_folds_are_live_and_directional` has been
//! re-contracted three times (i243/i256/…) because the quantity it pins — the
//! 5000-tick mean belief-confidence delta between a grain-starved world and a
//! grain-abundant one — is a knife-edge (`wins 2/6, best delta 0.0113` at the
//! last re-pin). i384's economy migration re-rolled every trajectory (prices
//! +3…10%, trade counts +1.4…2.8%, Gini +0.02…0.05), so the pin needs the same
//! question asked before it is touched: is the channel still LIVE (real
//! magnitude somewhere) and still DIRECTIONAL more often than chance, or has it
//! gone inert?
//!
//! Reports per seed the two arms' confidences and the signed delta, then the
//! family statistics for the 6 pinned seeds and for a 12-seed family.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i384_pe_fold_sweep`

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::{SimConfig, Simulation};

/// The pinned seed set, verbatim.
const PINNED: [u64; 6] = [22, 20, 46, 99, 42, 13];
/// Widened family (the i268-style set) for a win-rate estimate.
const WIDE: [u64; 12] = [22, 20, 46, 99, 42, 13, 1, 2, 7, 5, 21, 55];

/// One arm: returns the population-mean confidence plus the per-agent
/// (hunger, confidence) pairs, so the same run answers both the population pin
/// and the per-agent coupling question.
fn arm(seed: u64, resource1: f64) -> (f64, Vec<(f64, f64)>) {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: 5000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    for site in &mut sim.world.sites {
        for stock in &mut site.inventory {
            if stock.resource_id == 1 {
                stock.quantity = Fixed::from_f64(resource1);
            }
        }
    }
    sim.run(5000);
    let per_agent: Vec<(f64, f64)> = sim
        .agents
        .iter()
        .map(|a| {
            let conf = if a.beliefs.is_empty() {
                0.0
            } else {
                a.beliefs.iter().map(|b| b.confidence.to_f64()).sum::<f64>()
                    / a.beliefs.len() as f64
            };
            (a.needs.hunger.to_f64(), conf)
        })
        .collect();
    let with_beliefs = sim.agents.iter().filter(|a| !a.beliefs.is_empty()).count();
    let mean = sim
        .agents
        .iter()
        .filter(|a| !a.beliefs.is_empty())
        .map(|a| {
            a.beliefs.iter().map(|b| b.confidence.to_f64()).sum::<f64>() / a.beliefs.len() as f64
        })
        .sum::<f64>()
        / with_beliefs.max(1) as f64;
    (mean, per_agent)
}

/// Leg 3: the same manipulation, but HELD for the whole window (grain re-set
/// every tick) — is the manipulation inert because the horizon outlives it?
/// Returns (mean hunger, mean belief confidence).
fn arm_hold(seed: u64, resource1: f64) -> (f64, f64) {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: 5000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    let set = |sim: &mut Simulation, q: f64| {
        for site in &mut sim.world.sites {
            for stock in &mut site.inventory {
                if stock.resource_id == 1 {
                    stock.quantity = Fixed::from_f64(q);
                }
            }
        }
    };
    set(&mut sim, resource1);
    for _ in 0..5000 {
        sim.run(1);
        set(&mut sim, resource1);
    }
    let mean_hunger = sim
        .agents
        .iter()
        .map(|a| a.needs.hunger.to_f64())
        .sum::<f64>()
        / sim.agents.len() as f64;
    let with_beliefs = sim.agents.iter().filter(|a| !a.beliefs.is_empty()).count();
    let conf = sim
        .agents
        .iter()
        .filter(|a| !a.beliefs.is_empty())
        .map(|a| {
            a.beliefs.iter().map(|b| b.confidence.to_f64()).sum::<f64>() / a.beliefs.len() as f64
        })
        .sum::<f64>()
        / with_beliefs.max(1) as f64;
    (mean_hunger, conf)
}

/// Pearson correlation over the paired samples; 0 when either side is constant.
fn pearson(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    if n == 0.0 {
        return 0.0;
    }
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (x, y) in xs.iter().zip(ys) {
        sxy += (x - mx) * (y - my);
        sxx += (x - mx) * (x - mx);
        syy += (y - my) * (y - my);
    }
    if sxx <= f64::EPSILON || syy <= f64::EPSILON {
        return 0.0;
    }
    sxy / (sxx.sqrt() * syy.sqrt())
}

fn main() {
    println!("scarcity (grain 0) − abundance (grain 500) belief-confidence delta @5000");
    println!(
        "  {:<6} {:>10} {:>10} {:>10}  {:<8}",
        "seed", "scarce", "abundant", "delta", "in pin"
    );
    let mut pinned_wins = 0usize;
    let mut pinned_best = 0.0f64;
    let mut wide_wins = 0usize;
    let mut wide_best = 0.0f64;
    // Leg 2 — the per-agent coupling: does the fold reinforce harder for the
    // agents that got hungrier? (The population mean dilutes that signal over
    // 12 agents; the mechanism's own claim is agent-local.)
    let mut coupling_seeds = 0usize;
    let mut coupling_sum = 0.0f64;
    let mut hungry_more_seeds = 0usize;
    for seed in WIDE {
        let (scarce, scarce_agents) = arm(seed, 0.0);
        let (abundant, abundant_agents) = arm(seed, 500.0);
        let delta = scarce - abundant;
        let is_pinned = PINNED.contains(&seed);
        if is_pinned {
            if delta > 0.0 {
                pinned_wins += 1;
            }
            pinned_best = pinned_best.max(delta);
        }
        if delta > 0.0 {
            wide_wins += 1;
        }
        wide_best = wide_best.max(delta);

        // Pair agents by index (both arms share the founder draw), then relate
        // the scarcity-induced hunger shift to the confidence shift.
        let dh: Vec<f64> = scarce_agents
            .iter()
            .zip(&abundant_agents)
            .map(|(s, a)| s.0 - a.0)
            .collect();
        let dc: Vec<f64> = scarce_agents
            .iter()
            .zip(&abundant_agents)
            .map(|(s, a)| s.1 - a.1)
            .collect();
        let hungry_more = dh.iter().filter(|d| **d > 0.0).count();
        if hungry_more > dh.len() / 2 {
            hungry_more_seeds += 1;
        }
        let r = pearson(&dh, &dc);
        coupling_sum += r;
        if r > 0.0 {
            coupling_seeds += 1;
        }
        println!(
            "  {seed:<6} {scarce:>10.4} {abundant:>10.4} {delta:>+10.4}  {:<8} 
         \
         | hungrier {hungry_more}/12  r(Δhunger,Δconf) {r:+.3}",
            if is_pinned { "yes" } else { "no" }
        );
    }
    println!(
        "\n  pinned 6-seed set: wins {pinned_wins}/6  best delta {pinned_best:+.4}\n  \
         per-agent coupling: r > 0 on {coupling_seeds}/12 seeds (mean r {:.3}), \
         starvation raised mean hunger on {hungry_more_seeds}/12 seeds",
        coupling_sum / WIDE.len() as f64
    );

    println!("\n=== leg 3: manipulation HELD every tick ===");
    println!(
        "  {:<6} {:>10} {:>10} {:>10}  {:>10} {:>10} {:>10}",
        "seed", "hunger_starve", "hunger_feed", "Δhunger", "conf_starve", "conf_feed", "Δconf"
    );
    let (mut held_hunger, mut held_wins, mut held_best) = (0usize, 0usize, 0.0f64);
    for seed in PINNED {
        let (hs, cs) = arm_hold(seed, 0.0);
        let (ha, ca) = arm_hold(seed, 500.0);
        let dconf = cs - ca;
        if hs > ha {
            held_hunger += 1;
        }
        if dconf > 0.0 {
            held_wins += 1;
        }
        held_best = held_best.max(dconf);
        println!(
            "  {seed:<6} {hs:>10.4} {ha:>10.4} {:>+10.4}  {cs:>10.4} {ca:>10.4} {dconf:>+10.4}",
            hs - ha
        );
    }
    println!(
        "  held: hungrier on {held_hunger}/6 seeds; scarcity wins {held_wins}/6, best Δconf {held_best:+.4}"
    );
}
