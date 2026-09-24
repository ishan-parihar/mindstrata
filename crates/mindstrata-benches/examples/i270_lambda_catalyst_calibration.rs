//! Iteration-270 probe — lambda admission threshold + catalyst magnitude
//! evidence (PLAN_DC2 §Iter-270 ponytail batch).
//!
//! Two questions, measured at the live horizon with real event streams:
//!
//! 1. **Lambda threshold**: `Gate::pending()` uses 0.05, but the minimum
//!    catalyst magnitude in the live mapping is 0.3 (Conflict base) — the
//!    dead-zone has never actually gated anything. Sweep the threshold
//!    across the *observed* magnitude distribution and report the admission
//!    rate per threshold; the ratification decision is which threshold
//!    keeps the observed dynamics (all-events-admitted) while making the
//!    dead-zone meaningful for sub-noise pressure.
//! 2. **Catalyst magnitudes**: census the live magnitude distribution by
//!    kind (Grief/Bond/Threat/Transgression) over N seeds × 2000 ticks —
//!    evidence for whether the hand-set values (1.0/0.8/0.7/0.4/0.3+
//!    injury) produce differentiated pressure or collapse to one lump.
//!
//! No production code changes here — evidence first per doctrine §2.

use mindstrata_core::event::SimEvent;
use mindstrata_development::lambda::Gate;
use mindstrata_sim::sim::{SimConfig, Simulation};
use mindstrata_sim::systems::development::system_development;

fn main() {
    println!("=== i270: lambda threshold + catalyst magnitude evidence ===\n");

    // ── Part 1: catalyst magnitude census over the live event stream ────
    let seeds = [42u64, 7, 1234, 99, 2026];
    let mut by_kind: std::collections::BTreeMap<&'static str, Vec<f64>> =
        std::collections::BTreeMap::new();
    let mut total = 0usize;

    for seed in seeds {
        let config = SimConfig {
            seed,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(2000);
        // Replay the same event window the dev pass consumes, via
        // the public pure pass, to census what the field actually sees.
        let events: Vec<SimEvent> = sim.recent_events(10_000_000).to_vec();
        // Census via recent_events mapping replicating collect_catalysts'
        // magnitude mapping (the pass itself is mutative).
        for ev in &events {
            let mags: Vec<(&str, f64)> = match ev {
                SimEvent::AgentDied { .. } => vec![("Grief", 1.0)],
                SimEvent::MarriageFormed { .. } => vec![("Bond", 0.8), ("Bond", 0.8)],
                SimEvent::ChildBorn { .. } => vec![("Bond", 0.7), ("Bond", 0.7)],
                SimEvent::FeudFormed { .. } => vec![("Threat", 0.4), ("Threat", 0.4)],
                SimEvent::ConflictOccurred {
                    injury,
                    fear_induced,
                    ..
                } => {
                    let base = 0.3 + injury.to_f64().clamp(0.0, 0.5);
                    vec![
                        ("Threat", base.min(1.0)),
                        (
                            "Threat",
                            (base + fear_induced.to_f64().clamp(0.0, 0.2)).min(1.0),
                        ),
                    ]
                }
                SimEvent::NormViolated { .. } => vec![("Transgression", 0.5)],
                _ => vec![],
            };
            for (k, m) in mags {
                by_kind.entry(k).or_default().push(m);
                total += 1;
            }
        }
    }

    println!("catalyst census over {seeds:?} × 2000 ticks (12 agents):");
    for (k, mags) in &by_kind {
        let mean = mags.iter().sum::<f64>() / mags.len().max(1) as f64;
        let min = mags.iter().copied().fold(f64::INFINITY, f64::min);
        let max = mags.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        println!(
            "  {k:>14}: n={:>4}  mean={mean:.3}  min={min:.3}  max={max:.3}",
            mags.len()
        );
    }
    println!("  total catalysts: {total}\n");

    // ── Part 2: threshold sweep over the observed distribution ──────────
    println!("threshold sweep (admission rate over observed magnitudes):");
    let all_mags: Vec<f64> = by_kind.values().flatten().copied().collect();
    for t in [0.02, 0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.35] {
        let gate = Gate {
            engagement_threshold: t,
        };
        let admitted = all_mags.iter().filter(|&&m| gate.admit(m) > 0.0).count();
        let rate = admitted as f64 / all_mags.len().max(1) as f64;
        println!(
            "  t={t:.2}  admitted {admitted}/{}/{} = {rate:.3}",
            all_mags.len(),
            all_mags.len()
        );
    }

    // ── Part 3: does the gate shape matter downstream? Pressure at gate ──
    println!("\ngate output transform at observed magnitudes (t=0.05 vs t=0.30):");
    let g_lo = Gate {
        engagement_threshold: 0.05,
    };
    let g_hi = Gate {
        engagement_threshold: 0.30,
    };
    for &m in &[0.3f64, 0.4, 0.5, 0.7, 0.8, 1.0] {
        println!(
            "  m={m:.1}  admit(0.05)={:.3}  admit(0.30)={:.3}",
            g_lo.admit(m),
            g_hi.admit(m)
        );
    }

    // ── Part 4: live development-pass throughput sanity (events→admitted) ─
    println!("\nlive pass admission (t=0.05, seed 42, 2000 ticks):");
    let config = SimConfig {
        seed: 42,
        max_ticks: 2000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(2000);
    let events = sim.recent_events(10_000_000).to_vec();
    let mut agents = sim.agents.clone();
    let before: Vec<f64> = agents
        .iter()
        .map(|a| a.development.pathology.golden_addiction.intensity)
        .collect();
    system_development(&mut agents, &events);
    let moved = before
        .iter()
        .zip(agents.iter())
        .filter(|(b, a)| (a.development.pathology.golden_addiction.intensity - **b).abs() > 1e-9)
        .count();
    println!(
        "  agents whose Q3 moved on one full-window replay: {moved}/{}",
        agents.len()
    );

    println!("\nprobe complete.");
}
