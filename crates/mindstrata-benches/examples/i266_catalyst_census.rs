//! Iter-266 evidence addendum: catalyst census via `recent_events` diffs —
//! explains the field's press/fulfillment magnitudes and validates the
//! consumer anchor site (which per-tick pressure will the moral-panic gate
//! read?).
//!
//! Run: cargo run --release -p mindstrata-benches --example i266_catalyst_census

use mindstrata_core::event::SimEvent;
use mindstrata_development::catalyst::CatalystKind;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

/// Mirror of the sim's event→catalyst expansion (development.rs) for census
/// purposes: 1 event → 0..2 catalysts. Exact magnitudes are not needed for
/// the per-tick COUNT, only the per-capita bucket math matters downstream.
fn count_catalysts(events: &[SimEvent]) -> [usize; 4] {
    let mut counts = [0_usize; 4]; // [Grief, Bond, Threat, Transgression]
    for ev in events {
        match *ev {
            SimEvent::AgentDied { .. } => counts[0] += 1,
            SimEvent::MarriageFormed { .. } => {
                counts[1] += 2;
            }
            SimEvent::ChildBorn { .. } => {
                counts[1] += 2;
            }
            SimEvent::FeudFormed { .. } => {
                counts[2] += 2;
            }
            SimEvent::ConflictOccurred { .. } => {
                counts[2] += 2;
            }
            SimEvent::NormViolated { .. } => counts[3] += 1,
            _ => {}
        }
    }
    counts
}

fn main() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 2000,
        num_agents: 12,
        snapshot_interval: None,
        ..SimConfig::default()
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let mut total = [0_usize; 4];
    let mut ticks_with_catalyst = 0_usize;
    let mut max_per_tick_safety = 0_usize;
    for _ in 0..2000_u64 {
        let pre_len = sim.recent_events(usize::MAX).len();
        sim.tick();
        let new_events = &sim.recent_events(usize::MAX)[pre_len..];
        let counts = count_catalysts(new_events);
        if counts.iter().sum::<usize>() > 0 {
            ticks_with_catalyst += 1;
        }
        for (t, c) in total.iter_mut().zip(counts.iter()) {
            *t += c;
        }
        let safety = counts[2] + counts[3];
        max_per_tick_safety = max_per_tick_safety.max(safety);
    }
    let names = ["Grief", "Bond", "Threat", "Transgression"];
    for (n, c) in names.iter().zip(total.iter()) {
        println!("catalyst_kind={n} total={c}");
    }
    println!(
        "ticks_with_catalyst={ticks_with_catalyst}/2000 max_safety_per_tick={max_per_tick_safety}"
    );
    // Per-capita safety pressure on the single worst tick (12 agents):
    let worst = max_per_tick_safety as f64 / 12.0;
    println!("worst_tick_safety_pressure_per_line={worst:.4}");
    let f = &sim.collective_field;
    println!(
        "field_after: max_press={:.4} max_fulfillment={:.4}",
        f.lines
            .iter()
            .map(|l| l.press)
            .fold(0.0_f64, |a, b| a.max(b)),
        f.lines
            .iter()
            .map(|l| l.fulfillment)
            .fold(0.0_f64, |a, b| a.max(b))
    );
    let _ = CatalystKind::Bond; // silence unused if mapping changes
}
