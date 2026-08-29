//! Probe: TUI render hot-path frame-time for IC-8 (CLIENT 3.9).
//!
//! Measures the wall-clock cost of the per-frame render pipeline
//! (`render_metric_charts`, `render_dashboard`, `render_agent_list`,
//! `render_world_map`) on representative fixtures: 2K and 10K metric
//! histories, 12 and 48 agents. Prints `key=value` rows for the budget.
//!
//! Run: cargo run --release -p mindstrata-benches --example i271_render_perf

use std::time::Instant;

use mindstrata_sim::sim::{MetricsSnapshot, SimConfig, Simulation};
use mindstrata_tui::{
    render_agent_list, render_dashboard, render_metric_charts, render_world_map, AgentMarker,
    DashboardConfig,
};

fn fixture_history(n: usize) -> Vec<MetricsSnapshot> {
    (0..n)
        .map(|t| {
            let mut m = MetricsSnapshot::default();
            m.tick = (t * 10) as u64;
            m.avg_stress = (t as f64 / n as f64).sin().abs();
            m.avg_health = 1.0 - t as f64 / n as f64 * 0.2;
            m.family_count = (t / 20) as u64;
            m
        })
        .collect()
}

fn time_render<F: Fn() -> String>(label: &str, f: F, iters: usize) {
    // Warmup
    let _ = f();
    let start = Instant::now();
    let mut total_bytes = 0usize;
    for _ in 0..iters {
        let s = f();
        total_bytes += s.len();
    }
    let elapsed = start.elapsed().as_secs_f64();
    let per_frame_us = elapsed * 1e6 / iters as f64;
    println!(
        "{} iters={} per_frame_us={:.1} total_bytes={}",
        label, iters, per_frame_us, total_bytes
    );
}

fn main() {
    let quick = std::env::args().any(|a| a == "--quick");
    let iters = if quick { 200 } else { 1000 };
    let heavy_iters = if quick { 100 } else { 500 };
    // Build a representative sim for dashboard/agent/world fixtures.
    let config = SimConfig {
        seed: 42,
        max_ticks: 2000,
        num_agents: 12,
        ..SimConfig::default()
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(1000);
    let summaries = sim.agent_summaries();
    let history_2k = fixture_history(200);
    let history_10k = fixture_history(1000);
    // Also a 10K-heavy history (60-window vs 1000) to stress the lane cap.
    let history_heavy = fixture_history(10_000);

    let dashboard_cfg = DashboardConfig {
        season: "Spring".into(),
        year: 1,
        grain: sim.total_grain().to_f64(),
        water: sim.total_water().to_f64(),
        institution_count: sim.institutions.len(),
        faction_count: 0,
    };

    println!(
        "render_hot_path perf (release, iters={} unless noted)",
        iters
    );
    time_render(
        "metric_charts_2k",
        || render_metric_charts(&history_2k),
        iters,
    );
    time_render(
        "metric_charts_10k",
        || render_metric_charts(&history_10k),
        iters,
    );
    time_render(
        "metric_charts_10k_heavy",
        || render_metric_charts(&history_heavy),
        heavy_iters,
    );
    time_render(
        "dashboard_12",
        || render_dashboard(&summaries, 42, sim.current_tick().as_u64(), &dashboard_cfg),
        iters,
    );
    time_render("agent_list_12", || render_agent_list(&summaries), iters);

    // 48-agent dashboard/list
    let config48 = SimConfig {
        seed: 42,
        max_ticks: 2000,
        num_agents: 48,
        ..SimConfig::default()
    };
    let mut sim48 = Simulation::new(config48);
    sim48.populate();
    sim48.run(200);
    let summaries48 = sim48.agent_summaries();
    let cfg48 = DashboardConfig {
        season: "Spring".into(),
        year: 1,
        grain: sim48.total_grain().to_f64(),
        water: sim48.total_water().to_f64(),
        institution_count: sim48.institutions.len(),
        faction_count: 0,
    };
    time_render(
        "dashboard_48",
        || render_dashboard(&summaries48, 42, sim48.current_tick().as_u64(), &cfg48),
        iters,
    );
    time_render("agent_list_48", || render_agent_list(&summaries48), iters);

    // World map (fixed 16x16, markers scale with agents)
    let markers12: Vec<AgentMarker> = sim
        .agents
        .iter()
        .enumerate()
        .map(|(i, a)| AgentMarker {
            index: i,
            x: a.position.x,
            y: a.position.y,
            name: a.name.chars().next().unwrap_or('?'),
        })
        .collect();
    let markers48: Vec<AgentMarker> = sim48
        .agents
        .iter()
        .enumerate()
        .map(|(i, a)| AgentMarker {
            index: i,
            x: a.position.x,
            y: a.position.y,
            name: a.name.chars().next().unwrap_or('?'),
        })
        .collect();
    time_render(
        "world_map_12",
        || render_world_map(sim.world(), &markers12),
        iters,
    );
    time_render(
        "world_map_48",
        || render_world_map(sim48.world(), &markers48),
        iters,
    );
    if quick {
        // Gate threshold: Trends heavy must stay ≤1 ms (1000 µs) — IC-8.
        let heavy_us = {
            let start = Instant::now();
            for _ in 0..heavy_iters {
                let _ = render_metric_charts(&history_heavy);
            }
            start.elapsed().as_secs_f64() * 1e6 / heavy_iters as f64
        };
        if heavy_us > 1000.0 {
            eprintln!(
                "perf_budget_violation: metric_charts_10k_heavy {:.1}us > 1000us (IC-8)",
                heavy_us
            );
            std::process::exit(1);
        }
    }
}
