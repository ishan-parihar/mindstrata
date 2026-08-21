//! Iteration 186 — runtime-profile isolation probe.
//!
//! Answers: is the 200K/48 runtime linear-in-horizon because the cost is
//! contagion-bound (O(sick × n) proximity scan per tick) rather than
//! event-history-bound (per-tick iteration over the GROWING events vec)?
//!
//! Evidence strategy:
//!   1. **Segment flatness** — per-10K-chunk wall time must be FLAT from
//!      t=10K to t=200K while `events.len()` grows linearly. If any pass
//!      scanned the whole history per tick, later chunks would cost more
//!      (quadratic-in-horizon), not flat.
//!   2. **Density scaling** — per-tick cost at 12/24/48 agents. If the
//!      contagion pass dominates, cost should scale ~n² (4× per 2× agents)
//!      while the sick count scales ~n. Event-history cost would scale ~n
//!      (2× per 2× agents), since events-per-tick is proportional to agent
//!      count.
//!   3. **Scenario contrast** — calm (no epidemic seeded → contagion pass
//!      short-circuits on empty disease vecs) vs pestilence (endemic →
//!      full O(sick × n) scan) at the same density: the delta isolates the
//!      contagion cost.
//!
//! Run: `cargo run -p mindstrata-benches --example p5_profile_probe --release`
//! Knobs: `P5_HORIZON` (default 100_000), `P5_AGENTS` (default 48),
//! `P5_SEED` (default 42), `P5_SCEN` (default pestilence).

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;
use std::time::Instant;

fn main() {
    let horizon: u64 = std::env::var("P5_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);
    let agents: u32 = std::env::var("P5_AGENTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(48);
    let seed: u64 = std::env::var("P5_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);
    let scen = std::env::var("P5_SCEN").unwrap_or_else(|_| "pestilence".into());

    let mut sc = match scen.as_str() {
        "pestilence" => Scenario::pestilence(),
        "famine" => Scenario::famine(),
        "calm" => Scenario::calm(),
        _ => unreachable!(),
    };
    sc.seed = seed;
    sc.num_agents = agents;
    sc.ticks = horizon;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();

    let sample = 10_000u64;
    let mut t = 0u64;
    let mut seg_start = Instant::now();
    println!(
        "=== {scen} seed {seed} | {agents} agents | {horizon} ticks | per-{sample}-tick segments ==="
    );
    while t < horizon {
        let chunk = sample.min(horizon - t);
        sim.run(chunk);
        t += chunk;
        let seg = seg_start.elapsed().as_secs_f64();
        seg_start = Instant::now();
        let sick = sim.agent_diseases.iter().filter(|d| !d.is_empty()).count();
        let total_entries: usize = sim.agent_diseases.iter().map(std::vec::Vec::len).sum();
        println!(
            "  t={t:>7}: chunk={seg:5.2}s events={} sick={sick}/{} entries={total_entries}",
            sim.event_count(),
            sim.agents.len()
        );
    }
    let total = sim.event_count();
    let n = sim.agents.len();
    // Events-per-tick at the end: the history-growth rate. If the vec were
    // scanned per tick, the LAST chunk would be the slowest — compare chunk
    // times above.
    println!(
        "  summary: {horizon} ticks, {total} events total ({:.1} events/tick), \
         final pop={n}",
        total as f64 / horizon as f64
    );
}
