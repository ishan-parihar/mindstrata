//! §18.5 Criterion benchmarks — tick-loop and subsystem performance regression detection.
//!
//! Run with: `cargo bench -p mindstrata-benches`
//!
//! These benchmarks establish baseline performance for the simulation tick loop
//! at various agent counts and tick depths, catching regressions before they ship.

#![allow(missing_docs)] // criterion_group!/criterion_main! generate the public harness fns

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mindstrata_sim::{sim::SimConfig, Simulation};

/// Build a Simulation with given parameters, populated but not yet ticked.
fn make_sim(seed: u64, num_agents: u32, ticks: u64) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim
}

/// Benchmark: single tick at various agent counts. 48 is the §19.5.F
/// MAX_POPULATION cap (Iteration 141 adds it — the designed full-settlement
/// size — to every sweep so the scaling curve reaches the cap).
fn bench_single_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_tick");
    for num_agents in [6, 12, 24, 48] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{num_agents}_agents")),
            &num_agents,
            |b, &n| {
                b.iter_batched(
                    || make_sim(42, n, 1000),
                    |mut sim| {
                        let _: () = sim.tick();
                        black_box(());
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }
    group.finish();
}

/// Benchmark: 100-tick burst (short simulation).
fn bench_100_tick_burst(c: &mut Criterion) {
    let mut group = c.benchmark_group("burst_100_ticks");
    for num_agents in [6, 12, 24, 48] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{num_agents}_agents")),
            &num_agents,
            |b, &n| {
                b.iter_batched(
                    || make_sim(42, n, 500),
                    |mut sim| {
                        let _: () = sim.run(100);
                        black_box(());
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }
    group.finish();
}

/// Benchmark: 1000-tick run (standard simulation).
fn bench_1000_tick_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("run_1000_ticks");
    for num_agents in [6, 12, 24, 48] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{num_agents}_agents")),
            &num_agents,
            |b, &n| {
                b.iter_batched(
                    || make_sim(42, n, 2000),
                    |mut sim| {
                        let _: () = sim.run(1000);
                        black_box(());
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }
    group.finish();
}

/// Benchmark: metrics snapshot collection (hot path in logging/observation).
fn bench_metrics_snapshot(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics_snapshot");
    for num_agents in [12, 24, 48] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{num_agents}_agents")),
            &num_agents,
            |b, &n| {
                let mut sim = make_sim(42, n, 1000);
                sim.run(500); // warm up
                b.iter(|| {
                    black_box(sim.metrics_snapshot());
                });
            },
        );
    }
    group.finish();
}

/// Benchmark: agent summary generation (used by TUI and CLI).
fn bench_agent_summaries(c: &mut Criterion) {
    let mut group = c.benchmark_group("agent_summaries");
    for num_agents in [12, 24, 48] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{num_agents}_agents")),
            &num_agents,
            |b, &n| {
                let mut sim = make_sim(42, n, 1000);
                sim.run(500); // warm up
                b.iter(|| {
                    black_box(sim.agent_summaries());
                });
            },
        );
    }
    group.finish();
}

/// Benchmark: 10,000-tick run (long horizon, past the first ritual at 4320
/// and into structural time — Iteration 141). Measures the deep-horizon cost
/// that the 50K determinism tests (Iter-135) and the 10K surface snapshot
/// (Iter-136) pay, so horizon scaling is regression-tracked.
fn bench_10k_tick_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("run_10k_ticks");
    for num_agents in [12, 48] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{num_agents}_agents")),
            &num_agents,
            |b, &n| {
                b.iter_batched(
                    || make_sim(42, n, 10_000),
                    |mut sim| {
                        let _: () = sim.run(10_000);
                        black_box(());
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_single_tick,
    bench_100_tick_burst,
    bench_1000_tick_run,
    bench_10k_tick_run,
    bench_metrics_snapshot,
    bench_agent_summaries,
);
criterion_main!(benches);
