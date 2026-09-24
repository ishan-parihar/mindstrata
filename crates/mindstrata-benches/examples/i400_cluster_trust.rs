//! i400 — the last self-contained v1 trust readers in `social_cluster.rs`,
//! sized before touching: the §8.1.9 ToM pair-trust reads (L595/L598) and the
//! §19.5.I knowledge-diffusion `source_trust` (L872).
//!
//! Three questions decide whether this migration is landable in one commit:
//! * **A — divergence.** How far apart are v1 `trust` and v2 `trust` on the
//!   pairs these consumers see? i376 syncs v1 ← v2 daily, so the expectation
//!   is small — but "expected small" was also true of the marriage gate before
//!   its probe found a 0.59 max. Measured on all pairs, and separately on
//!   rows the engine considers live (`is_contacted`), which is the population
//!   these consumers actually range over.
//! * **B — the 0.5 threshold exposure.** `infer_intent` says Friendly only at
//!   trust > 0.5, so a pair whose two stores straddle 0.5 *flips its inferred
//!   intent* under the migration. That is the i392-row-4 "threshold lottery"
//!   test, not a magnitude question: a band edge inside the divergence range
//!   cannot be re-anchored, it has to be understood.
//! * **C — the fallback.** The v1 read uses `rel_pos(..).map_or(0.5, ..)`. If
//!   v1 rows exist for every pair, the fallback is dead and the only change is
//!   the store; if v1 misses, the migration silently replaces 0.5 with the v2
//!   stranger prior (0.4) — a semantics change worth a number.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i400_cluster_trust`

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn village(n: u32, seed: u64) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: 0,
        world_width: 16,
        world_height: 16,
        num_agents: n,
        snapshot_interval: None,
        ..SimConfig::default()
    });
    sim.populate();
    sim
}

struct Acc {
    n: u64,
    sum: f64,
    max: f64,
    flips: u64,
}

impl Acc {
    fn new() -> Self {
        Self {
            n: 0,
            sum: 0.0,
            max: 0.0,
            flips: 0,
        }
    }

    fn push(&mut self, a: Fixed, b: Fixed, threshold: f64) {
        let (a, b) = (a.to_f64(), b.to_f64());
        let d = (a - b).abs();
        self.n += 1;
        self.sum += d;
        self.max = self.max.max(d);
        let (la, lb) = (a > threshold, b > threshold);
        if la != lb {
            self.flips += 1;
        }
    }

    fn mean(&self) -> f64 {
        if self.n == 0 {
            0.0
        } else {
            self.sum / self.n as f64
        }
    }
}

fn main() {
    println!("i400 — social_cluster v1 trust readers: divergence, 0.5 exposure, fallback\n");

    for (n, horizon) in [(12u32, 20_000u64), (48, 20_000)] {
        let mut sim = village(n, 42);
        let mut all = Acc::new();
        let mut live = Acc::new();
        let mut v1_miss = 0u64;
        let mut v2_miss = 0u64;

        // Sample every 72 ticks: the i376 sync is DAILY (144), so a 72-tick
        // cadence alternates boundary/mid-day samples instead of always
        // landing on the synced side of the day.
        for _ in 0..(horizon / 72) {
            sim.run(72);
            let k = sim.agents.len();
            for i in 0..k {
                for j in 0..k {
                    if i == j {
                        continue;
                    }
                    let v1 = sim
                        .relationships()
                        .iter()
                        .find(|r| r.from.as_u64() as usize == i && r.to.as_u64() as usize == j);
                    if v1.is_none() {
                        v1_miss += 1;
                    }
                    let v2 = sim.relationship_v2_between(i, j);
                    if v2.is_none() {
                        v2_miss += 1;
                    }
                    let (Some(v1), Some(v2)) = (v1, v2) else {
                        continue;
                    };
                    all.push(v1.trust, v2.trust, 0.5);
                    if v2.is_contacted() {
                        live.push(v1.trust, v2.trust, 0.5);
                    }
                }
            }
        }

        println!("N={n}  ({horizon} ticks, 72-tick sampling)");
        println!(
            "  A  all pairs      mean {:.4}  max {:.4}  n={}",
            all.mean(),
            all.max,
            all.n
        );
        println!(
            "  A  contacted rows mean {:.4}  max {:.4}  n={}",
            live.mean(),
            live.max,
            live.n
        );
        println!(
            "  B  0.5-threshold straddles: all {} ({:.3}%)  contacted {} ({:.3}%)",
            all.flips,
            if all.n > 0 {
                100.0 * all.flips as f64 / all.n as f64
            } else {
                0.0
            },
            live.flips,
            if live.n > 0 {
                100.0 * live.flips as f64 / live.n as f64
            } else {
                0.0
            }
        );
        println!("  C  v1 misses {v1_miss}   v2 misses {v2_miss}\n");
    }
}
