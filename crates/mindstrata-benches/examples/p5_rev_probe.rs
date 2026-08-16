//! P5 revolution seed sweep — find seeds where a faction of >= 5 members
//! revolts and the council absorbs it (peak council >= 5) within 70K.
use mindstrata_core::conflict::ConflictKind;
use mindstrata_core::event::SimEvent;
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::institutions::InstitutionKind;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    for seed in [1u64, 2, 3, 4, 5, 7, 11, 13, 42, 44, 46, 55, 99] {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: 40000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.params.meme_mutation_rate_base = Fixed::ZERO;
        sim.populate();
        let mut peak_council = 0usize;
        let mut peak_at = 0u64;
        let mut peak_faction_size = 0usize;
        for chunk in 0..80 {
            sim.run(500);
            let tick = chunk * 500;
            let council_n = sim
                .institutions
                .iter()
                .find(|i| i.kind == InstitutionKind::Council)
                .map_or(0, |c| c.members.len());
            if council_n > peak_council {
                peak_council = council_n;
                peak_at = tick;
                peak_faction_size = sim
                    .institutions
                    .iter()
                    .filter(|i| i.kind == InstitutionKind::Faction)
                    .map(|i| i.members.len())
                    .max()
                    .unwrap_or(0);
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
        println!(
            "seed {seed:>2}: peak_council {peak_council} @{peak_at} (revolting faction size ~{peak_faction_size}), revolutions {rev_count}"
        );
    }
}
