//! §17.4 Criterion per-subsystem benchmarks — the queue's "no per-system
//! benches" gap (Iteration 141). These isolate the hottest subsystem entry
//! points so a regression in one system is caught independently of the full
//! tick loop:
//!
//! - `memory_encode`: the §13.5 memory-store hot path (encoding a trace).
//! - `gossip_process`: the §18.4 rumor-transmission hot path (mutating a
//!   rumor across a hop, with emotional distortion + acceptance gating).
//!
//! Inputs are extracted from a real populated simulation so the measured work
//! matches production shapes (not hand-crafted trivial values). Per Apollo
//! Ch. 3, benchmarks run against the release profile by default (`cargo bench`).
//!
//! Run with: `cargo bench -p mindstrata-benches --bench subsystems`

#![allow(missing_docs)] // criterion_group!/criterion_main! generate the public harness fns

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::gossip::process_gossip;
use mindstrata_sim::memory::{MemoryKind, MemoryStore, MemoryTag};
use mindstrata_sim::person::Belief;
use mindstrata_sim::{Simulation, sim::SimConfig};

/// Build a populated simulation and run it briefly so agents carry realistic
/// emotion/personality/belief state for the subsystem benchmarks.
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

/// §13.5: Memory-trace encoding — the per-event write path. Called on every
/// salient event an agent notices, so its cost is paid constantly.
fn bench_memory_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_encode");
    let salience = Fixed::from_f64(0.8);
    let charge = Fixed::from_f64(0.6);
    group.bench_function("fresh_store", |b| {
        b.iter_batched(
            MemoryStore::default,
            |mut store| {
                // `encode` returns () so the work is the call itself; nothing
                // to observe via black_box.
                store.encode(
                    MemoryKind::Episodic,
                    100,
                    salience,
                    charge,
                    None,
                    MemoryTag::AteFood,
                );
            },
            BatchSize::SmallInput,
        );
    });
    // A store already near capacity exercises the eviction branch of encode.
    group.bench_function("near_capacity", |b| {
        b.iter_batched(
            || {
                let mut store = MemoryStore::default();
                for i in 0..64u64 {
                    store.encode(
                        MemoryKind::Episodic,
                        i,
                        salience,
                        charge,
                        None,
                        MemoryTag::AteFood,
                    );
                }
                store
            },
            |mut store| {
                // `encode` returns () so the work is the call itself; nothing
                // to observe via black_box.
                store.encode(
                    MemoryKind::Emotional,
                    1000,
                    salience,
                    charge,
                    None,
                    MemoryTag::HelpedBy,
                );
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

/// §18.4: Rumor transmission across one hop — the gossip hot path. Inputs are
/// cloned from a real populated agent (emotions, personality, beliefs).
fn bench_gossip_process(c: &mut Criterion) {
    let mut group = c.benchmark_group("gossip_process");
    let sim = make_runtime_sim();
    let agent = &sim.agents[0];
    let emotions = agent.emotions.clone();
    let personality = agent.personality.clone();
    let beliefs: Vec<Belief> = agent.beliefs.clone();
    let rumor = mindstrata_sim::gossip::Rumor {
        proposition_id: 0,
        confidence: Fixed::from_f64(0.7),
        hops: 0,
        origin_tick: 0,
        last_heard_tick: 50,
        emotional_charge: Fixed::from_f64(0.3),
        identity_linkage: Fixed::from_f64(0.1),
        original_resistance: Fixed::from_f64(0.2),
    };
    let source_trust = Fixed::from_f64(0.5);
    let base_fidelity = Fixed::from_f64(0.8);
    let emotional_distortion = Fixed::from_f64(0.2);
    let acceptance_threshold = Fixed::from_f64(0.5);
    group.bench_function("single_hop", |b| {
        b.iter(|| {
            black_box(process_gossip(
                &rumor,
                source_trust,
                &emotions,
                &personality,
                &personality,
                &beliefs,
                51,
                base_fidelity,
                emotional_distortion,
                acceptance_threshold,
            ));
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_memory_encode,
    bench_gossip_process,
);
criterion_main!(benches);
