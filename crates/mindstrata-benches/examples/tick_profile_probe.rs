//! Iteration 218 — 100K-tick profiling probe.
//! Measures per-phase timing and allocation patterns to identify optimization
//! hotspots. Does NOT modify the sim — purely observational.
//!
//! Run with: `cargo run -p mindstrata-benches --example tick_profile_probe --release`

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;
use std::time::Instant;

fn profile_scenario(name: &str, seed: u64, ticks: u64) {
    let mut sc = match name {
        "calm" => Scenario::calm(),
        "famine" => Scenario::famine(),
        "pestilence" => Scenario::pestilence(),
        _ => unreachable!(),
    };
    sc.seed = seed;
    sc.ticks = ticks;

    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    let n = sim.agents.len();

    eprintln!("=== {name} seed {seed} @{ticks} ({n} agents) ===");

    // Phase 1: Warmup — run 1000 ticks to fill caches / reach steady state
    let warmup_start = Instant::now();
    sim.run(1000);
    let warmup_elapsed = warmup_start.elapsed();
    eprintln!("  warmup (1K):   {warmup_elapsed:.3?}");

    // Phase 2: Profiled run — time individual ticks and measure aggregate
    let profile_start = Instant::now();
    let profile_ticks = ticks.saturating_sub(1000);

    // Collect per-tick wall-clock times for a sample window
    let sample_ticks: usize = profile_ticks.min(1000) as usize;
    let mut tick_times: Vec<std::time::Duration> = Vec::with_capacity(sample_ticks);

    for t in 0..profile_ticks {
        let tick_start = Instant::now();
        sim.tick();
        let tick_dur = tick_start.elapsed();

        if t < sample_ticks as u64 {
            tick_times.push(tick_dur);
        }
    }

    let profile_elapsed = profile_start.elapsed();
    let avg_tick = profile_elapsed / profile_ticks as u32;

    // Sort tick times for percentile analysis
    tick_times.sort();
    let p50 = tick_times[tick_times.len() / 2];
    let p90 = tick_times[(tick_times.len() as f64 * 0.9) as usize];
    let p99 = tick_times[(tick_times.len() as f64 * 0.99) as usize];
    let max_tick = tick_times.last().unwrap();
    let min_tick = tick_times.first().unwrap();

    eprintln!("  profiled ({profile_ticks} ticks):");
    eprintln!("    total:       {profile_elapsed:.3?}");
    eprintln!("    avg/tick:    {avg_tick:.3?}");
    eprintln!("    min/tick:    {min_tick:.3?}");
    eprintln!("    p50/tick:    {p50:.3?}");
    eprintln!("    p90/tick:    {p90:.3?}");
    eprintln!("    p99/tick:    {p99:.3?}");
    eprintln!("    max/tick:    {max_tick:.3?}");

    // Phase 3: Allocation estimate
    // Count key data structures that grow/shrink per tick
    let n_relationships: usize = sim.agents.iter().map(|a| a.relationship_v2s.len()).sum();
    let n_institutions: usize = sim.institutions.len();
    let n_institution_roles: usize = sim.institutions.iter().map(|i| i.roles.len()).sum();
    let n_diseases: usize = sim.agent_diseases.iter().map(std::vec::Vec::len).sum();
    let n_events = sim.recent_events(10_000_000).len();
    let n_marriages = sim.marriage_registry.marriages.len();
    let n_households = sim.households.len();
    let n_clans = sim.clan_registry.clans.len();
    let n_collective = sim.collective_memory_registry.entries.len();
    let n_factions = sim.faction_v2_registry.factions.len();

    eprintln!("  data structure sizes:");
    eprintln!("    agents:            {n}");
    eprintln!("    relationships:     {n_relationships} (O(N²) potential)");
    eprintln!("    institutions:      {n_institutions} ({n_institution_roles} roles)");
    eprintln!("    diseases:          {n_diseases}");
    eprintln!("    events:            {n_events}");
    eprintln!("    marriages:         {n_marriages}");
    eprintln!("    households:        {n_households}");
    eprintln!("    clans:             {n_clans}");
    eprintln!("    collective_memory: {n_collective}");
    eprintln!("    factions:          {n_factions}");

    // Estimate per-tick allocation cost
    // Key O(N²) paths to check:
    //   1. Trust sync: iterates relationships for each agent → O(N×R)
    //   2. Emotion regulation social_support: iterates ALL relationships per agent → O(N×R)
    //   3. Kinship BFS: O(N²) per agent daily → O(N³) daily
    //   4. Power balance: O(N²) daily
    //   5. Relationship decay: O(R) per agent → O(N×R)
    eprintln!("  complexity estimates:");
    eprintln!("    trust_sync per tick:     O(N×R) = {} ops", n * n_relationships);
    eprintln!("    social_support per tick: O(N×R) = {} ops", n * n_relationships);
    eprintln!("    kinship BFS per day:     O(N³) = {} ops", n * n * n);
    eprintln!("    power_balance per day:   O(N²×R) = {} ops", n * n * n_relationships);

    // Phase 4: Memory layout analysis
    // Check agent struct size for cache-line friendliness
    let agent_size = std::mem::size_of::<mindstrata_sim::person::BodyState>()
        + std::mem::size_of::<mindstrata_sim::person::DiscreteEmotions>()
        + std::mem::size_of::<mindstrata_sim::person::Personality>()
        + std::mem::size_of::<mindstrata_sim::person::NeedState>()
        + std::mem::size_of::<mindstrata_sim::person::Affect>()
        + std::mem::size_of::<mindstrata_sim::person::CognitiveState>()
        + std::mem::size_of::<mindstrata_sim::biology::EmbodiedState>();
    let cache_line = 64; // typical x86_64
    let agents_per_line = cache_line as f64 / agent_size as f64;
    eprintln!("  memory layout:");
    eprintln!("    agent struct size:  {agent_size} bytes");
    eprintln!("    agents per cache line: {agents_per_line:.2}");
    eprintln!("    full agent array:   {:.1} KB", (agent_size * n) as f64 / 1024.0);

    println!();
}

fn main() {
    let ticks: u64 = std::env::var("PROFILE_TICKS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);

    let seed = std::env::var("PROFILE_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    // Profile all three scenarios
    profile_scenario("calm", seed, ticks);
    profile_scenario("famine", seed, ticks);
    profile_scenario("pestilence", seed, ticks);

    println!("=== PROFILING COMPLETE ===");
}
