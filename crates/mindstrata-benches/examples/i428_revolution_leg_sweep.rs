//! i428 — revolution leg sweep: for every firing seed of the post-deletion
//! tree (the i399 sweep measured 6/10 firing), record BOTH legs of the
//! revolution pin's contract (revolution count + Elder office handover), so
//! the pin family is built only from members that satisfy both.
//!
//! The experiment is the revolution pin's own loop, extracted without edits:
//! pestilence @70K, meme mutation off, council sampled every 500 ticks, the
//! Elder holder captured at start and end. Firing-seed set from the trimmed
//! arc's own sweep run (/tmp/i428_rev.txt, measured after the social_cluster
//! and kind-schedule reverts).
//!
//! Run: cargo run --release -p mindstrata-benches --example i428_revolution_leg_sweep
use mindstrata_core::conflict::ConflictKind;
use mindstrata_core::event::SimEvent;
use mindstrata_core::EntityId;
use mindstrata_sim::institutions::InstitutionKind;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

fn main() {
    println!("i428 revolution leg sweep (both pin legs, all firing seeds):");
    println!("  seed | revs peak_council elder_start elder_end handover");
    for seed in [99u64, 23, 5, 42, 11, 1] {
        let mut sc = Scenario::pestilence();
        sc.seed = seed;
        sc.ticks = 70_000;
        let mut sim = Simulation::from_scenario(sc);
        sim.params.meme_mutation_rate_base = mindstrata_core::fixed::Fixed::ZERO;
        sim.populate();
        let mut peak_council = 0usize;
        let mut elder_at_start: Option<u64> = None;
        for _ in 0..140 {
            sim.run(500);
            let council = sim
                .institutions
                .iter()
                .find(|i| i.kind == InstitutionKind::Council)
                .expect("council should exist");
            if elder_at_start.is_none() {
                elder_at_start = council.get_role_holder("Elder").map(EntityId::as_u64);
            }
            peak_council = peak_council.max(council.members.len());
        }
        let elder_at_end = sim
            .institutions
            .iter()
            .find(|i| i.kind == InstitutionKind::Council)
            .and_then(|c| c.get_role_holder("Elder"))
            .map(EntityId::as_u64);
        let rev_count = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    SimEvent::ConflictOccurred {
                        kind: ConflictKind::Revolution,
                        ..
                    }
                )
            })
            .count();
        println!(
            "  {seed:>4} | {rev_count:>4} {peak_council:>3} {:?} {:?} {}",
            elder_at_start,
            elder_at_end,
            if elder_at_start != elder_at_end {
                "CHANGED"
            } else {
                "same"
            }
        );
    }
}
