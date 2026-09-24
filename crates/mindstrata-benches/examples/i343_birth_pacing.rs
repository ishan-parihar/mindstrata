//! i343 — is seed-1's birth volume delayed, or is fertility suppressed?
//!
//! The conception-pipeline pin bands seed 1 @175K at 8–30 births; i342 recorded
//! 18 births starting at 11 060 (i242), and the contacted-degree social rewire
//! brings that to 5. A drop of that size from a small, bounded channel change
//! has two possible readings, and they demand opposite responses (§2.3/§4.2):
//!
//!   * **pacing** — births simply land later, the producer is alive and the band
//!     should be re-pinned on measured evidence;
//!   * **suppression** — the change damaged the fertility chain, in which case
//!     the producer must be revived rather than the band widened.
//!
//! This probe distinguishes them: it runs past the pinned horizon and reports the
//! birth trajectory alongside the state the change touches (couples, open
//! pregnancies, anxiety, and the contacted-degree census).
//!
//! Run: cargo run --release -p mindstrata-benches --example i343_birth_pacing

use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    let horizon = 250_000u64;
    let mut sim = Simulation::new(SimConfig {
        seed: 1,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    let births = crate::collect(&mut sim, horizon);

    println!("i343 — seed 1, 16x16, N=12, {horizon} ticks (pin horizon is 175K)\n");
    let partnered = sim.agents.iter().filter(|a| a.partner.is_some()).count();
    let deg = sim.contacted_degrees();
    let mean_deg = deg.iter().sum::<u32>() as f64 / deg.len().max(1) as f64;
    println!(
        "final state: population {} | partnered {} | mean contacted degree {mean_deg:.1}",
        sim.agents.len(),
        partnered
    );
    println!("\nbirth ticks ({} total):", births.len());
    println!("  {births:?}");
    let by = |limit: u64| births.iter().filter(|t| **t <= limit).count();
    for h in [50_000u64, 100_000, 150_000, 175_000, 200_000, 250_000] {
        println!("  by {:>7}: {:>3} births", h, by(h));
    }
    println!(
        "\nreading: births continuing to accumulate past 175K at a similar rate ⇒ pacing (re-pin the\n\
         band on measured evidence); a flat tail with couples/pregnancies present but deliveries\n\
         absent ⇒ suppressed fertility (revive the producer, do not widen the band)."
    );
}

/// Segmented birth collection — the event buffer is bounded (i327), so the
/// whole-run scan the old pin used is no longer available (the same reason
/// `test_helpers::collect_child_born_ticks` exists).
fn collect(sim: &mut Simulation, horizon: u64) -> Vec<u64> {
    let mut out = Vec::new();
    let mut done = 0u64;
    while done < horizon {
        let seg = 2_000u64.min(horizon - done);
        let lo = done;
        sim.run(seg);
        done += seg;
        for e in sim.recent_events(usize::MAX) {
            if let mindstrata_core::event::SimEvent::ChildBorn { tick, .. } = e {
                let t = tick.as_u64();
                if t > lo && t <= done {
                    out.push(t);
                }
            }
        }
    }
    out.sort_unstable();
    out
}
