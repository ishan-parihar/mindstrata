//! i370 — probe: council membership dynamics across seeds and horizons.
//!
//! The charter decision: the council is an OFFICE (Elder + Guard Captain +
//! Councilor = 3 members, the `default_institutions` shape), not a taxed
//! class. i361/i363 recorded membership inconsistency (31 vs 3) and the
//! root-cause reading says the source is `norms_impl.rs`'s revolution path,
//! which does `members.clone_from(&faction_members)` — the whole faction
//! roster becomes the council, and `collect_taxes` taxes *members*, so the
//! hoard equilibrium T* = reserve + inflow/share scales with the roster.
//!
//! This probe measures, per seed:
//! 1. council membership size at populate (expect 3),
//! 2. max membership over the run (expect inflation only if a revolution fired),
//! 3. revolutions fired (counted from the rendered Chronicle's revolution line),
//! 4. final treasury vs membership (the T* = 20 + 4·I law predicts the pair),
//! 5. whether membership ever exceeds the office count while NO revolution
//!    has fired (that would falsify the root cause).
//!
//! Run: `cargo run --release -p mindstrata-benches --example i370_council_membership`
//! Knobs: `I370_SEEDS` (comma list), `I370_HORIZON`.

use mindstrata_sim::institutions::InstitutionKind;
use mindstrata_sim::sim::{chronicle, SimConfig, Simulation};

fn main() {
    let horizon: u64 = std::env::var("I370_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20_000);
    let seeds: Vec<u64> = std::env::var("I370_SEEDS").ok().map_or_else(
        || vec![7, 23, 42, 55, 99, 123],
        |s| s.split(',').filter_map(|t| t.trim().parse().ok()).collect(),
    );

    println!("== i370: council membership probe (N=48, 46x46, horizon {horizon}) ==");
    for seed in seeds {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: horizon,
            world_width: 46,
            world_height: 46,
            num_agents: 48,
            snapshot_interval: None,
        });
        sim.populate();

        let council_at_populate = sim
            .institutions
            .iter()
            .find(|i| i.kind == InstitutionKind::Council)
            .map_or(0, |c| c.members.len());

        let mut max_members = council_at_populate;
        let mut final_members = council_at_populate;
        let mut inflation_without_revolution = false;

        for _t in 0..horizon {
            sim.tick();
            if let Some(c) = sim
                .institutions
                .iter()
                .find(|i| i.kind == InstitutionKind::Council)
            {
                max_members = max_members.max(c.members.len());
                if c.members.len() > 3 && !inflation_without_revolution {
                    // Cheap conservative flag: any inflation is interesting;
                    // the chronicle line tells us whether a revolution caused it.
                    inflation_without_revolution = true;
                }
                final_members = c.members.len();
            }
        }

        let revolutions = chronicle::render_chronicle(&sim)
            .lines()
            .filter(|l| l.contains("overthrown"))
            .count() as u64;

        let treasury = sim
            .institutions
            .iter()
            .find(|i| i.kind == InstitutionKind::Council)
            .map_or(0.0, |c| c.treasury.to_f64());

        println!(
            "seed {seed:>3}: populate={council_at_populate} max={max_members} final={final_members} revolutions={revolutions} treasury={treasury:.1}{}",
            if max_members > 3 && revolutions == 0 {
                "  <-- INFLATION WITHOUT REVOLUTION"
            } else {
                ""
            }
        );
    }
}
