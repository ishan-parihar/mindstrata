//! §18.5 Criterion benchmarks — tick-loop and subsystem performance regression detection.
//!
//! Run with: `cargo bench -p mindstrata-benches`
//!
//! These benchmarks establish baseline performance for the simulation tick loop
//! at various agent counts and tick depths, catching regressions before they ship.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use mindstrata_sim::{Simulation, sim::SimConfig};

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

/// Benchmark: single tick at various agent counts.
fn bench_single_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_tick");
    for num_agents in [6, 12, 24] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{num_agents}_agents")),
            &num_agents,
            |b, &n| {
                b.iter_batched(
                    || make_sim(42, n, 1000),
                    |mut sim| {
                        black_box(sim.tick());
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

/// Benchmark: 100-tick burst (short simulation).
fn bench_100_tick_burst(c: &mut Criterion) {
    let mut group = c.benchmark_group("burst_100_ticks");
    for num_agents in [6, 12, 24] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{num_agents}_agents")),
            &num_agents,
            |b, &n| {
                b.iter_batched(
                    || make_sim(42, n, 500),
                    |mut sim| {
                        black_box(sim.run(100));
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

/// Benchmark: 1000-tick run (standard simulation).
fn bench_1000_tick_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("run_1000_ticks");
    for num_agents in [6, 12, 24] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{num_agents}_agents")),
            &num_agents,
            |b, &n| {
                b.iter_batched(
                    || make_sim(42, n, 2000),
                    |mut sim| {
                        black_box(sim.run(1000));
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

/// Benchmark: metrics snapshot collection (hot path in logging/observation).
fn bench_metrics_snapshot(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics_snapshot");
    for num_agents in [12, 24] {
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
    for num_agents in [12, 24] {
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

criterion_group!(
    benches,
    bench_single_tick,
    bench_100_tick_burst,
    bench_1000_tick_run,
    bench_metrics_snapshot,
    bench_agent_summaries,
);
criterion_main!(benches);
