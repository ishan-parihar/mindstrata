//! i357 — walk before you run: what actually limits the locomotion pace?
//!
//! i347 made the §19.5.G feud-approach producer live (`Move` had been 0 in
//! 96 000 agent-ticks) but left its **pace** as an open calibration question,
//! recording that "approach is one Manhattan step per *decision*". That claim
//! is worth checking before any fix: `Move`'s `duration_ticks` is 1, so
//! `action_progress` returns to 0 every tick and the selection chain re-decides
//! **every tick** — the movement pass then steps once per tick. If the anger
//! gate (i347: `FEUD_APPROACH_ANGER = 0.02`) holds across ticks, the pace is
//! one step per *tick*, not per decision, and the limiter is anger decay.
//!
//! Legs:
//!   A — the pace: `Move` duty cycle and, of the ticks a feud is open, what
//!       fraction passes the anger gate (the branch's precondition).
//!   B — feud episodes: how many produce movement, mean/max steps per episode.
//!   C — start distances (is the target even reachable in a few steps?).
//!
//! Run: `cargo run --release -p mindstrata-benches --example i357_locomotion_pace`

use mindstrata_sim::actions::ActionKind;
use mindstrata_sim::sim::{SimConfig, Simulation};

const FEUD_APPROACH_ANGER: f64 = 0.02; // i347

fn main() {
    println!("i357 — locomotion pace (seeds 42/7/11, N=48, 46x46)\n");

    for seed in [42u64, 7, 11] {
        let ticks = 20_000u64;
        let n = 48usize;
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: ticks,
            world_width: 46,
            world_height: 46,
            num_agents: n as u32,
            snapshot_interval: None,
        });
        sim.populate();

        let mut move_ticks = 0u64;
        let mut feud_open_ticks = 0u64;
        let mut feud_anger_passing_ticks = 0u64;
        let mut agent_ticks = 0u64;
        let mut in_episode = vec![false; n];
        let mut episode_steps = vec![0u64; n];
        let mut episode_start_dist = vec![0i64; n];
        let mut episodes: Vec<u64> = Vec::new();
        let mut start_dists: Vec<i64> = Vec::new();

        for _ in 0..ticks {
            sim.tick();
            for (i, a) in sim.agents.iter().enumerate() {
                if i >= n {
                    break;
                }
                agent_ticks += 1;
                let has_feud = !a.feuds.is_empty();
                if matches!(a.current_action, ActionKind::Move { .. }) {
                    move_ticks += 1;
                }
                if has_feud {
                    feud_open_ticks += 1;
                    if a.emotions.anger.to_f64() > FEUD_APPROACH_ANGER {
                        feud_anger_passing_ticks += 1;
                    }
                }
                match (in_episode[i], has_feud) {
                    (false, true) => {
                        in_episode[i] = true;
                        episode_steps[i] = 0;
                        let t = a.feuds[0];
                        if t < sim.agents.len() {
                            let tp = sim.agents[t].position;
                            let d = (a.position.x - tp.x).abs() as i64
                                + (a.position.y - tp.y).abs() as i64;
                            episode_start_dist[i] = d;
                            start_dists.push(d);
                        }
                    }
                    (true, true) => {
                        if matches!(a.current_action, ActionKind::Move { .. }) {
                            episode_steps[i] += 1;
                        }
                    }
                    (true, false) => {
                        episodes.push(episode_steps[i]);
                        in_episode[i] = false;
                    }
                    (false, false) => {}
                }
            }
        }

        let with_steps = episodes.iter().filter(|s| **s > 0).count();
        let mean_steps = if episodes.is_empty() {
            0.0
        } else {
            episodes.iter().sum::<u64>() as f64 / episodes.len() as f64
        };
        let max_steps = episodes.iter().copied().max().unwrap_or(0);
        println!("seed {seed} (N={n}, {ticks} ticks):");
        println!(
            "  A: Move duty {:.3}% of agent-ticks; feud-open {:.2}%; of feud-open ticks {:.1}% pass the anger gate",
            100.0 * move_ticks as f64 / agent_ticks.max(1) as f64,
            100.0 * feud_open_ticks as f64 / agent_ticks.max(1) as f64,
            100.0 * feud_anger_passing_ticks as f64 / feud_open_ticks.max(1) as f64,
        );
        println!(
            "  B: episodes {}  with-movement {} ({:.0}%)  mean steps {:.1}  max {}",
            episodes.len(),
            with_steps,
            if episodes.is_empty() {
                0.0
            } else {
                100.0 * with_steps as f64 / episodes.len() as f64
            },
            mean_steps,
            max_steps
        );
        start_dists.sort_unstable();
        if !start_dists.is_empty() {
            let p = |q: f64| start_dists[(q * (start_dists.len() - 1) as f64) as usize];
            println!(
                "  C: start distance p10 {} p50 {} p90 {}  (Manhattan tiles)",
                p(0.10),
                p(0.50),
                p(0.90)
            );
        }
    }
}
