//! Pestilence seed sweep for revolution liveness (DC-1 CO-2026-001).
//!
//! Runs the 12-seed family under pestilence scenario at 70K horizon,
//! counts Revolution ConflictOccurred events + peak council membership.
//! Used to re-anchor `revolution_is_regime_change_not_repeat_loop`
//! after the 4-quadrant pathology fan-out.

use mindstrata_core::conflict::ConflictKind;
use mindstrata_core::event::SimEvent;
use mindstrata_sim::institutions::InstitutionKind;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];

fn main() {
    for &seed in &SEEDS {
        let mut sc = Scenario::pestilence();
        sc.seed = seed;
        sc.ticks = 70_000;
        let mut sim = Simulation::from_scenario(sc);
        sim.params.meme_mutation_rate_base = mindstrata_core::fixed::Fixed::ZERO;
        sim.populate();
        let mut peak_council = 0usize;
        for _ in 0..140 {
            sim.run(500);
            if let Some(council) = sim
                .institutions
                .iter()
                .find(|i| i.kind == InstitutionKind::Council)
            {
                peak_council = peak_council.max(council.members.len());
            }
        }
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
        println!("seed={seed} revolutions={rev_count} peak_council={peak_council}");
    }
}
