//! Iter-280 liveness diagnosis — why was the WP-J A/B byte-identical?
//!
//! Probes three candidate dead points:
//! 1. Are there institution members at N=12 (membership live)?
//! 2. Is institution morale nonzero in vivo (channel input live)?
//! 3. Does `norm_pressure` ever reach the §12.3 branch (branch reachability)?
//! 4. Forced-stage: governance=eco=12.0 → mult=1.40. Does ANY metric move?

use mindstrata_sim::sim::{SimConfig, Simulation};

fn run(forced: bool) {
    let config = SimConfig {
        seed: 42,
        max_ticks: 20_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    if forced {
        for line in &mut sim.collective_field.lines {
            line.stage = 12.0;
        }
    }
    sim.run(20_000);
    let m = sim.metrics_snapshot();

    // 1+2: membership and morale census
    let mut members = 0usize;
    let mut morale_sum = 0.0f64;
    let mut morale_nonzero_insts = 0usize;
    for inst in &sim.institutions {
        let n = inst.members.len();
        members += n;
        let mo = inst.collective.morale.to_f64();
        if mo > 0.0 {
            morale_nonzero_insts += 1;
        }
        morale_sum += mo;
    }
    println!(
        "forced={forced}  insts={}  members={}  morale>0 insts={}  mean_morale={:.4}",
        sim.institutions.len(),
        members,
        morale_nonzero_insts,
        if sim.institutions.is_empty() {
            0.0
        } else {
            morale_sum / sim.institutions.len() as f64
        },
    );

    let violations = sim
        .recent_events(sim.event_count())
        .iter()
        .filter(|e| matches!(e, mindstrata_core::event::SimEvent::NormViolated { .. }))
        .count();
    println!(
        "  trades={} grain={:.1} feuds={} events={} violations={} memes={} memesgen={}",
        m.total_trades,
        m.total_grain,
        m.total_active_feuds,
        m.event_count,
        violations,
        m.active_meme_count,
        sim.collective_field
            .lines
            .iter()
            .filter(|l| l.stage > 1.0)
            .count(),
    );
}

fn main() {
    run(false);
    run(true);
}
