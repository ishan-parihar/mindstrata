//! §18.5 Per-subsystem regression gate (Iteration 170) — the RWR row-67
//! "per-system regression thresholds wired into CI" gap.
//!
//! The full-loop gates (`tick_throughput_regression_gate` in debug +
//! `regression_gate` in release) catch regressions in the AGGREGATE tick
//! loop, but a slowdown confined to ONE subsystem can hide inside the loop's
//! total budget (a 10x memory-encode regression moves the 48-agent tick by
//! only ~2% — far below the 1.7x tick floor's resolution). This gate
//! isolates the hottest subsystem entry points — the same paths the §17.4
//! criterion benches measure — and enforces hard ns/op ceilings in the
//! shipped release profile.
//!
//! Run with: `cargo run -p mindstrata-benches --example subsystem_gate --release`
//!
//! Exits non-zero (failing CI) when any measured path exceeds its ceiling.
//! Ceilings are calibrated from release-profile criterion pins (16.3ns /
//! 265ns / 41.2ns) with ~12x headroom for shared CI runners — the gate
//! trips on order-of-magnitude regressions (accidental O(n²) loops,
//! per-op allocation explosions), not micro-variance.
//!
//! The gate prints per-path timings and a final PASS/FAIL verdict.

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::gossip::{process_gossip, Rumor};
use mindstrata_sim::memory::{MemoryKind, MemoryStore, MemoryTag};
use mindstrata_sim::person::Belief;
use mindstrata_sim::{sim::SimConfig, Simulation};
use std::hint::black_box;
use std::time::Instant;

/// ns/op ceilings (release profile, best-of-N). Probe-pinned at Iteration
/// 170 on the same machine that pinned the §17.4 criterion numbers: the
/// criterion bench measures 16.3ns fresh-encode / 265ns near-capacity /
/// 41.2ns gossip, while THIS gate's harness shapes land at 16.1ns /
/// ~518ns / 38.2ns — the near-capacity path deliberately measures
/// prefill+encode (the 64-trace store is rebuilt inside each sample so the
/// eviction branch is exercised from a genuine steady state), so its pin
/// is higher than criterion's setup-separated 265ns. Headroom: fresh ~12x,
/// gossip ~13x, near-capacity ~5.6x — the tighter near-capacity margin is
/// deliberate: it is the path MOST likely to regress (eviction scan +
/// allocation churn), so it gets the closest watch, while the ceiling is
/// still well above any transient CI stall on a ~500ns op. A shared CI
/// runner can be transiently stalled and per-op timings are far noisier
/// than aggregate tick rates — the gate trips on real slowdowns, not
/// micro-variance.
const FRESH_ENCODE_MAX_NS: f64 = 200.0;
const NEAR_CAPACITY_ENCODE_MAX_NS: f64 = 3000.0;
const GOSSIP_HOP_MAX_NS: f64 = 500.0;

/// Build a populated simulation and run it briefly so agents carry realistic
/// emotion/personality/belief state (mirrors the §17.4 bench inputs).
fn make_runtime_sim() -> Simulation {
    let config = SimConfig {
        seed: 42,
        max_ticks: 1000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(50);
    sim
}

/// Measure ns/op for a closure that performs `ops` operations per sample,
/// taking the BEST of `samples` — transient CI stalls must not trip the
/// gate, only real slowdowns of the code path (same philosophy as the
/// full-loop gate). The accumulator is black_box'd so the work cannot be
/// optimized away.
///
/// The caller must black_box any loop-invariant INPUTS inside the closure
/// (the gossip path does; the encode paths build their store inside).
/// Without that, an LTO-enabled release build could prove the call pure,
/// hoist it out of the loop, and the gate would silently measure ~0ns and
/// always pass — a regression gate that can't fail. Black-boxing the
/// inputs makes the harness robust regardless of build flags.
fn measure_ns_per_op<F: FnMut() -> u64>(ops: u64, samples: u32, mut f: F) -> f64 {
    // Warmup: one un-timed call so branch predictors/caches are settled for
    // the timed samples (the near-capacity eviction scan is data-dependent).
    black_box(f());
    let mut best = f64::INFINITY;
    for _ in 0..samples {
        let start = Instant::now();
        let mut acc = 0u64;
        for _ in 0..ops {
            acc = acc.wrapping_add(f());
        }
        black_box(acc);
        let elapsed = start.elapsed().as_secs_f64();
        let per_op = elapsed * 1e9 / ops as f64;
        if per_op < best {
            best = per_op;
        }
    }
    best
}

fn main() {
    let mut all_pass = true;

    // ── Path 1: fresh memory-store encode (§13.5 write path) ──────────
    let ops = 50_000u64;
    let fresh_ns = measure_ns_per_op(ops, 5, || {
        let mut store = MemoryStore::default();
        store.encode(
            MemoryKind::Episodic,
            100,
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.6),
            None,
            MemoryTag::AteFood,
        );
        // Return a store observation so the encode cannot be elided.
        store.episodes.len() as u64
    });
    let fresh_pass = fresh_ns <= FRESH_ENCODE_MAX_NS;
    all_pass &= fresh_pass;
    println!(
        "{}: memory_encode fresh_store {fresh_ns:.1} ns/op (ceiling {FRESH_ENCODE_MAX_NS}, best of 5×{ops})",
        if fresh_pass { "PASS" } else { "FAIL" }
    );

    // ── Path 2: near-capacity encode (eviction branch) ────────────────
    // Pre-fill 64 traces then keep encoding — the evict_weakest scan is the
    // hot cost at steady state.
    let near_ns = measure_ns_per_op(ops, 5, || {
        let mut store = MemoryStore::default();
        for i in 0..64u64 {
            store.encode(
                MemoryKind::Episodic,
                i,
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.6),
                None,
                MemoryTag::AteFood,
            );
        }
        store.encode(
            MemoryKind::Emotional,
            1000,
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.6),
            None,
            MemoryTag::HelpedBy,
        );
        store.episodes.len() as u64
    });
    let near_pass = near_ns <= NEAR_CAPACITY_ENCODE_MAX_NS;
    all_pass &= near_pass;
    println!(
        "{}: memory_encode near_capacity {near_ns:.1} ns/op (ceiling {NEAR_CAPACITY_ENCODE_MAX_NS}, best of 5×{ops})",
        if near_pass { "PASS" } else { "FAIL" }
    );

    // ── Path 3: gossip single-hop (§18.4 transmission hot path) ───────
    let sim = make_runtime_sim();
    let agent = &sim.agents[0];
    let emotions = agent.emotions.clone();
    let personality = agent.personality.clone();
    let beliefs: Vec<Belief> = agent.beliefs.clone();
    let rumor = Rumor {
        proposition_id: 0,
        confidence: Fixed::from_f64(0.7),
        hops: 0,
        origin_tick: 0,
        last_heard_tick: 50,
        emotional_charge: Fixed::from_f64(0.3),
        identity_linkage: Fixed::from_f64(0.1),
        original_resistance: Fixed::from_f64(0.2),
    };
    let gossip_ns = measure_ns_per_op(ops, 5, || {
        // All inputs are loop-invariant — black_box each so an LTO-enabled
        // release build cannot prove the call pure and hoist it out of the
        // timed loop (which would silently pass every run).
        let r = process_gossip(
            black_box(&rumor),
            black_box(Fixed::from_f64(0.5)),
            black_box(&emotions),
            black_box(&personality),
            black_box(&personality),
            black_box(&beliefs),
            black_box(51),
            black_box(Fixed::from_f64(0.8)),
            black_box(Fixed::from_f64(0.2)),
            black_box(Fixed::from_f64(0.5)),
        );
        // GossipResult is not Copy; observe the mutated confidence only.
        r.mutated_confidence.to_raw() as u64
    });
    let gossip_pass = gossip_ns <= GOSSIP_HOP_MAX_NS;
    all_pass &= gossip_pass;
    println!(
        "{}: gossip single_hop {gossip_ns:.1} ns/op (ceiling {GOSSIP_HOP_MAX_NS}, best of 5×{ops})",
        if gossip_pass { "PASS" } else { "FAIL" }
    );

    if all_pass {
        println!("SUBSYSTEM GATE: PASS");
    } else {
        println!("SUBSYSTEM GATE: FAIL — a subsystem path exceeded its release-mode ns/op ceiling");
        std::process::exit(1);
    }
}
