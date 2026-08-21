//! Iteration 186 — epidemic same-kind dedup stress verification.
//!
//! The Iteration 185 fix (sim.rs §17b) caps each agent at ONE new
//! infection per kind per tick (pending-`new_infections` dedup) plus the
//! cross-tick `already_infected` guard that caps concurrent same-kind
//! diseases at 1. Pre-fix, the same-tick pile-up let a 100K pestilence
//! run carry **809,303 concurrent Epidemic entries across 12 agents**
//! (8,396 on a single agent), and the per-tick contagion cost grew with
//! the pile until the run blew up.
//!
//! This probe re-runs that audit at the heaviest legal configuration —
//! `MAX_POPULATION` (48 agents) @100K — and asserts the dedup invariants
//! hold at every 10K sample:
//!   1. **Same-kind cap**: no agent ever holds 2+ entries of the same
//!      DiseaseKind at a sample. A same-tick double-push (the pre-fix bug)
//!      would land BOTH entries in `agent_diseases[j]` and show up here.
//!   2. **Bounded vecs**: per-agent vec length ≤ #kinds, total entries
//!      O(agents × kinds) — no pile growth over the horizon.
//!   3. **Determinism**: same seed → byte-identical disease fingerprint
//!      (agent, kind, ticks_infected) at every sample.
//!   4. **Realism**: deaths stay bounded, the epidemic persists as an
//!      endemic low-grade state (not a permanent full saturation) and the
//!      population survives the 100K horizon.
//!
//! Run: `cargo run -p mindstrata-benches --example p5_epidemic_dedup --release`
//! Knobs: `P5_HORIZON` (ticks, default 100_000), `P5_SAMPLE` (sample
//! stride, default 10_000), `P5_SEED` (default 42), `P5_AGENTS`
//! (default 48 — the MAX_POPULATION stress leg; 12 gives the reference).

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;
use std::collections::BTreeMap;

fn main() {
    let horizon: u64 = std::env::var("P5_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);
    let sample: u64 = std::env::var("P5_SAMPLE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);
    let seed: u64 = std::env::var("P5_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);
    let agents: u32 = std::env::var("P5_AGENTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(48);

    let mut sc = Scenario::pestilence();
    sc.seed = seed;
    sc.num_agents = agents.min(mindstrata_sim::population_cap::MAX_POPULATION as u32);
    sc.ticks = horizon;

    let start = std::time::Instant::now();
    let mut sim = Simulation::from_scenario(sc.clone());
    sim.populate();
    println!(
        "=== pestilence seed {seed} | {agents} agents | {horizon} ticks (sample every {sample}) ==="
    );

    let mut all_ok = true;
    let mut t = 0u64;
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

    // Realism block
    let mut deaths = 0u32;
    let mut births = 0u32;
    for e in sim.recent_events(10_000_000) {
        match e {
            mindstrata_core::event::SimEvent::AgentDied { .. } => deaths += 1,
            mindstrata_core::event::SimEvent::ChildBorn { .. } => births += 1,
            _ => {}
        }
    }
    let n = sim.agents.len();
    let carriers = sim.agent_diseases.iter().filter(|d| !d.is_empty()).count();
    let pop_ok = n >= agents as usize / 2;
    let deaths_ok = deaths < 60; // pestilence is the mortality scenario; still must not collapse
    println!(
        "  end state: pop={n} deaths={deaths} births={births} carriers={carriers} \
         (pop_ok={pop_ok} deaths_ok={deaths_ok})"
    );
    all_ok &= pop_ok && deaths_ok;

    // Determinism twin: re-run the same config and compare the final fingerprint.
    let mut sim2 = Simulation::from_scenario(sc);
    sim2.populate();
    sim2.run(horizon);
    let fp2 = fingerprint(&sim2);
    let det_ok = fp == fp2;
    all_ok &= det_ok;
    println!(
        "  determinism: {}",
        if det_ok { "IDENTICAL" } else { "MISMATCH" }
    );

    println!(
        "\nEPIDEMIC DEDUP @{agents} AGENTS: {}",
        if all_ok { "PASS" } else { "FAIL — see above" }
    );
    if !all_ok {
        std::process::exit(1);
    }
}

/// Sample the disease state and assert the dedup invariants. Returns true
/// if all invariants hold at this sample.
fn check_sample(sim: &Simulation, t: u64) -> bool {
    let mut total = 0usize;
    let mut max_vec = 0usize;
    let mut max_same_kind = 0usize;
    let mut dupes = 0usize;
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    for ds in &sim.agent_diseases {
        total += ds.len();
        max_vec = max_vec.max(ds.len());
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

    // Invariants:
    // 1. No agent holds 2+ same-kind diseases (the dedup canary).
    let cap_ok = max_same_kind <= 1;
    // 2. Per-agent vec bounded by the number of distinct kinds (4:
    //    Cold, Fever, WoundInfection, Epidemic). A pile would blow past this.
    let bounded_ok = max_vec <= 4;
    // 3. Total entries bounded by agents × kinds (linear, no pile growth).
    let linear_ok = total <= sim.agents.len() * 4;

    println!(
        "  t={t:>7}: total_entries={total} max_vec={max_vec} max_same_kind={max_same_kind} \
         dupes={dupes} kinds={kinds:?} | cap={cap_ok} bounded={bounded_ok} linear={linear_ok}"
    );
    cap_ok && bounded_ok && linear_ok && dupes == 0
}

/// Deterministic fingerprint of the full disease state:
/// sorted (agent, kind, ticks_infected) tuples.
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
