//! Phase 5 (AP2 §10) — relational-systems probe.
//! Measures: relationship-stage histogram, deep-dimension means/spreads
//! (intimacy, jealousy, power_balance), active marriage/courtship counts,
//! household membership distribution, clan counts + myths.
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn run_sim(seed: u64, ticks: u64) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(ticks);
    sim
}

fn main() {
    for seed in [42u64, 7, 99] {
        for ticks in [5000u64, 20_000] {
            let sim = run_sim(seed, ticks);
            // ── Stage histogram + dimension stats over relationship_v2s ──
            let mut stage_counts: std::collections::BTreeMap<String, u32> =
                std::collections::BTreeMap::new();
            let mut n_edges = 0usize;
            let mut int_sum = 0.0f64;
            let mut int_nonzero = 0usize;
            let mut jel_sum = 0.0f64;
            let mut pb_neg = 0usize;
            let mut pb_pos = 0usize;
            for a in &sim.agents {
                for r in &a.relationship_v2s {
                    n_edges += 1;
                    *stage_counts
                        .entry(format!("{:?}", r.stage))
                        .or_insert(0u32) += 1;
                    let i = r.intimacy.to_f64();
                    int_sum += i;
                    if i > 0.01 {
                        int_nonzero += 1;
                    }
                    jel_sum += r.jealousy.to_f64();
                    if r.power_balance < Fixed::ZERO {
                        pb_neg += 1;
                    } else if r.power_balance > Fixed::ZERO {
                        pb_pos += 1;
                    }
                }
            }
            let populated = stage_counts.len();
            // ── Marriage / courtship / household / clan counts ──────────
            let active_marriages = sim
                .marriage_registry
                .marriages
                .iter()
                .filter(|m| m.active)
                .count();
            let active_courtships = sim.active_courtships.iter().filter(|c| c.active).count();
            let multi_households = sim.households.iter().filter(|h| h.members.len() >= 2).count();
            let mut hh_sizes: Vec<usize> = sim.households.iter().map(|h| h.members.len()).collect();
            hh_sizes.sort_unstable();
            let clans = sim.clan_registry.clans.len();
            let clans_with_myths = sim
                .clan_registry
                .clans
                .iter()
                .filter(|c| !c.myths.is_empty())
                .count();
            let clan_members: usize = sim
                .clan_registry
                .clans
                .iter()
                .map(|c| c.core_households.len())
                .sum();
            println!(
                "seed {seed} @{ticks}: edges={n_edges} stages_populated={populated} {:?} | \
                 intimacy_mean={:.3} nonzero={int_nonzero} | jealousy_mean={:.3} | \
                 power_balance pos={pb_pos} neg={pb_neg} | marriages={active_marriages} \
                 courtships={active_courtships} | households={} multi={multi_households} \
                 sizes={hh_sizes:?} | clans={clans} with_myths={clans_with_myths} members={clan_members}",
                stage_counts,
                int_sum / n_edges.max(1) as f64,
                jel_sum / n_edges.max(1) as f64,
                sim.households.len(),
            );
        }
    }
}
