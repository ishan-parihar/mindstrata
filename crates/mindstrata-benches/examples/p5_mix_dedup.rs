//! Iteration 186 — full 4-kind dedup stress (Cold + Fever + WoundInfection +
//! Epidemic circulating simultaneously).
//!
//! The `p5_epidemic_dedup` probe verified the same-kind cap under a
//! single-kind epidemic (Epidemic only — that's all a real pestilence run
//! ever contains, because Cold/Fever are never seeded by any scenario and
//! `apply_injury` (the only WoundInfection source) is never called in the
//! sim). This probe injects the secondary vectors directly — the same way
//! the pestilence shock seeds Epidemic (`already`-guarded push, sim.rs
//! §27 shock) — so the dedup is exercised with FOUR kinds circulating
//! through the contagion pass simultaneously:
//!   - Epidemic: seeded by the pestilence shock at tick ~500 (rate 0.15)
//!   - Cold:     injected at tick 1000 (rate 0.05, duration 200)
//!   - Fever:    injected at tick 1000 (rate 0.03, duration 400)
//!   - WoundInfection: injected at tick 1000 (rate 0 — non-contagious,
//!     exercises the already-infected guard on the non-contagious path)
//!
//! Invariants (same as `p5_epidemic_dedup`, now PER-KIND):
//!   1. No agent ever holds 2+ entries of the SAME kind — the per-tick
//!      pending-dedup + cross-tick already_infected cap must hold for every
//!      kind independently while all four circulate.
//!   2. Per-agent vec length stays within the 8-entry pathological guard
//!      (MAX_CONCURRENT_DISEASES) — a pile of ANY kind would blow past it.
//!   3. Determinism: same seed → byte-identical disease fingerprint.
//!   4. Realism: the mixed endemic state persists (kinds cycle via
//!      recovery → re-infection) without collapse.
//!
//! Run: `cargo run -p mindstrata-benches --example p5_mix_dedup --release`
//! Knobs: `P5_HORIZON` (default 100_000), `P5_SEED` (default 42),
//! `P5_AGENTS` (default 48).

use mindstrata_sim::health::{ActiveDisease, DiseaseKind};
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;
use std::collections::BTreeMap;

/// Tick at which the secondary vectors are injected — right after the
/// pestilence shock (tick 500) seeds Epidemic, so all four kinds
/// circulate together for the rest of the horizon.
const INJECT_TICK: u64 = 1000;

fn inject(sim: &mut Simulation, kind: DiseaseKind, tick: u64) {
    // Mirror the pestilence shock's `already`-guard (sim.rs §27 shock): push
    // into every (i mod stride)-th agent that doesn't already carry the kind.
    // Deterministic (no RNG — `rng` is private), so the determinism twin
    // re-runs the identical injection sequence. The stride spreads each
    // vector across the village so the contagion pass has dense sources.
    let stride = match kind {
        DiseaseKind::Cold => 3,             // ~1/3 of agents
        DiseaseKind::Fever => 4,            // ~1/4 of agents
        DiseaseKind::WoundInfection => 2,   // ~1/2 of agents (non-contagious)
        _ => 1,
    };
    let n = sim.agents.len();
    let mut count = 0;
    for i in 0..n {
        if i % stride != 0 {
            continue;
        }
        let already = sim.agent_diseases[i].iter().any(|d| d.kind == kind);
        if !already {
            sim.agent_diseases[i].push(ActiveDisease::new(kind));
            count += 1;
        }
    }
    println!("  [t={tick}] injected {kind:?} into {count} agents");
}

fn main() {
    let horizon: u64 = std::env::var("P5_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);
    let seed: u64 = std::env::var("P5_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);
    let agents: u32 = std::env::var("P5_AGENTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(48);
    let sample: u64 = std::env::var("P5_SAMPLE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);

    let mut sc = Scenario::pestilence();
    sc.seed = seed;
    sc.num_agents = agents.min(mindstrata_sim::population_cap::MAX_POPULATION as u32);
    sc.ticks = horizon;

    let start = std::time::Instant::now();
    let mut sim = Simulation::from_scenario(sc.clone());
    sim.populate();
    println!(
        "=== pestilence seed {seed} | {agents} agents | {horizon} ticks | 4-kind mix (sample every {sample}) ==="
    );

    let mut all_ok = true;
    let mut t = 0u64;
    // Phase 1: run to the injection tick (1000 — right after the
    // Epidemic shock at tick 500 has established) so the secondary
    // vectors enter alongside an active epidemic.
    sim.run(INJECT_TICK);
    t = INJECT_TICK;
    inject(&mut sim, DiseaseKind::Cold, t);
    inject(&mut sim, DiseaseKind::Fever, t);
    inject(&mut sim, DiseaseKind::WoundInfection, t);
    // Phase 2: run the remaining horizon in sample-sized chunks, checking
    // the per-kind cap at every sample.
    while t < horizon {
        let chunk = sample.min(horizon - t);
        sim.run(chunk);
        t += chunk;
        let ok = check_sample(&sim, t);
        all_ok &= ok;
        eprintln!("  [t={t}] {:.1}s elapsed", start.elapsed().as_secs_f64());
    }
    let elapsed = start.elapsed().as_secs_f64();
    let fp = fingerprint(&sim);
    println!("  final fingerprint: {fp}");
    println!(
        "  ran {horizon} ticks @{agents} agents in {elapsed:.1}s ({:.0} ticks/s)",
        horizon as f64 / elapsed
    );

    let n = sim.agents.len();
    let mut deaths = 0u32;
    for e in sim.recent_events(10_000_000) {
        if let mindstrata_core::event::SimEvent::AgentDied { .. } = e {
            deaths += 1;
        }
    }
    let pop_ok = n >= agents as usize / 2;
    let deaths_ok = deaths < 60;
    println!(
        "  end state: pop={n} deaths={deaths} (pop_ok={pop_ok} deaths_ok={deaths_ok})"
    );
    all_ok &= pop_ok && deaths_ok;

    // Determinism twin: re-run the same config INCLUDING the injection
    // ticks, and compare the final fingerprint.
    let mut sim2 = Simulation::from_scenario(sc);
    sim2.populate();
    sim2.run(INJECT_TICK);
    inject(&mut sim2, DiseaseKind::Cold, INJECT_TICK);
    inject(&mut sim2, DiseaseKind::Fever, INJECT_TICK);
    inject(&mut sim2, DiseaseKind::WoundInfection, INJECT_TICK);
    let mut t2 = INJECT_TICK;
    while t2 < horizon {
        let chunk = sample.min(horizon - t2);
        sim2.run(chunk);
        t2 += chunk;
    }
    let fp2 = fingerprint(&sim2);
    let det_ok = fp == fp2;
    all_ok &= det_ok;
    println!(
        "  determinism: {}",
        if det_ok { "IDENTICAL" } else { "MISMATCH" }
    );

    println!(
        "\n4-KIND MIX DEDUP @{agents} AGENTS: {}",
        if all_ok { "PASS" } else { "FAIL — see above" }
    );
    if !all_ok {
        std::process::exit(1);
    }
}

/// Check the PER-KIND same-kind cap across the full mix.
fn check_sample(sim: &Simulation, t: u64) -> bool {
    let mut total = 0usize;
    let mut max_vec = 0usize;
    let mut max_same_kind = 0usize;
    let mut dupes = 0usize;
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    let mut max_vec_agent = 0usize;
    for (ai, ds) in sim.agent_diseases.iter().enumerate() {
        total += ds.len();
        if ds.len() > max_vec {
            max_vec = ds.len();
            max_vec_agent = ai;
        }
        let mut per_kind: BTreeMap<String, usize> = BTreeMap::new();
        for d in ds {
            let k = format!("{:?}", d.kind);
            *kinds.entry(k.clone()).or_insert(0) += 1;
            let c = per_kind.entry(k).or_insert(0);
            *c += 1;
            if *c > 1 {
                dupes += 1;
            }
        }
        max_same_kind = max_same_kind.max(per_kind.values().copied().max().unwrap_or(0));
    }

    let cap_ok = max_same_kind <= 1;
    let bounded_ok = max_vec <= 8; // MAX_CONCURRENT_DISEASES pathological guard
    let linear_ok = total <= sim.agents.len() * 8;

    println!(
        "  t={t:>7}: total_entries={total} max_vec={max_vec}(a{max_vec_agent}) max_same_kind={max_same_kind} \
         dupes={dupes} kinds={kinds:?} | cap={cap_ok} bounded={bounded_ok} linear={linear_ok}"
    );
    cap_ok && bounded_ok && linear_ok && dupes == 0
}

/// Deterministic fingerprint of the full disease state.
fn fingerprint(sim: &Simulation) -> String {
    let mut items: Vec<String> = Vec::new();
    for (i, ds) in sim.agent_diseases.iter().enumerate() {
        for d in ds {
            items.push(format!("{i}:{:?}:{}", d.kind, d.ticks_infected));
        }
    }
    items.sort();
    items.join(",")
}
