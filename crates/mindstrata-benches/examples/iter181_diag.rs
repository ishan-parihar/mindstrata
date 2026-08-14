//! Iteration 181 diagnostic: reproduce the two failing integration contracts
//! with instrumentation.
//!
//! (1) Revolution contract (seed 42, 12 agents, 16x16, 70K, mutation OFF):
//!     prints faction formation, council/faction membership over time, and
//!     the legitimacy + grievance trajectories so we can see whether the
//!     revolution is delayed or suppressed.
//! (2) Birth pipeline (seed 46, 16x16, 12 agents, 160K): prints the birth
//!     ticks plus marriage/courtship diagnostics to explain the 2-vs-6 birth
//!     count shift.
//!
//! Run: `cargo run -p mindstrata-benches --example iter181_diag --release`

use mindstrata_sim::institutions::InstitutionKind;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn base(seed: u64, ticks: u64) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    sim
}

fn main() {
    // ── (1) Revolution diagnostics ──────────────────────────────────────
    println!("=====REVOLUTION-CONTRACT=====");
    let mut sim = base(42, 70_000);
    sim.params.meme_mutation_rate_base = mindstrata_core::fixed::Fixed::ZERO;
    let mut faction_born: Option<u64> = None;
    for tick in (500..=70_000).step_by(500) {
        sim.run(500);
        if faction_born.is_none()
            && sim.institutions.iter().any(|i| i.kind == InstitutionKind::Faction)
        {
            faction_born = Some(tick as u64);
        }
    }
    let council = sim
        .institutions
        .iter()
        .find(|i| i.kind == InstitutionKind::Council);
    let faction = sim
        .institutions
        .iter()
        .find(|i| i.kind == InstitutionKind::Faction);
    println!(
        "faction_formed_at={:?} council_members={} faction_members={}",
        faction_born,
        council.map_or(0, |c| c.members.len()),
        faction.map_or(0, |f| f.members.len()),
    );
    let rev_events: Vec<u64> = sim
        .recent_events(10_000_000)
        .iter()
        .filter_map(|e| match e {
            mindstrata_core::event::SimEvent::ConflictOccurred {
                kind: mindstrata_core::conflict::ConflictKind::Revolution,
                tick,
                ..
            } => Some(tick.as_u64()),
            _ => None,
        })
        .collect();
    println!("revolution_events={rev_events:?}");
    let legitimacy = sim
        .institutions
        .iter()
        .find(|i| i.kind == InstitutionKind::Council)
        .map(|c| c.legitimacy.to_f64());
    println!("council_legitimacy_end={legitimacy:?}");

    // Council/faction membership trajectory over the full horizon.
    let mut sim3 = base(42, 70_000);
    sim3.params.meme_mutation_rate_base = mindstrata_core::fixed::Fixed::ZERO;
    for tick in (5000..=70_000).step_by(5000) {
        sim3.run(5000);
        let c = sim3
            .institutions
            .iter()
            .find(|i| i.kind == InstitutionKind::Council);
        let f = sim3
            .institutions
            .iter()
            .find(|i| i.kind == InstitutionKind::Faction);
        println!(
            "tick={} council={} faction={} pop={}",
            tick,
            c.map_or(0, |x| x.members.len()),
            f.map_or(0, |x| x.members.len()),
            sim3.agents.len(),
        );
    }

    // Fine-grained: watch the absorbed members vanish (15K-25K).
    let mut sim4 = base(42, 30_000);
    sim4.params.meme_mutation_rate_base = mindstrata_core::fixed::Fixed::ZERO;
    sim4.run(12_000);
    for tick in (12_000..=30_000).step_by(1000) {
        sim4.run(1000);
        let c = sim4
            .institutions
            .iter()
            .find(|i| i.kind == InstitutionKind::Council);
        let f = sim4
            .institutions
            .iter()
            .find(|i| i.kind == InstitutionKind::Faction);
        println!(
            "fine tick={} council_members={:?} faction_members={}",
            tick,
            c.map_or(vec![], |x| x.members.clone()),
            f.map_or(0, |x| x.members.len()),
        );
    }

    // ── (2) Birth pipeline diagnostics ──────────────────────────────────
    println!("=====BIRTH-PIPELINE=====");
    let mut sim2 = base(46, 160_000);
    sim2.run(160_000);
    let birth_ticks: Vec<u64> = sim2
        .recent_events(10_000_000)
        .iter()
        .filter_map(|e| match e {
            mindstrata_core::event::SimEvent::ChildBorn { tick, .. } => Some(tick.as_u64()),
            _ => None,
        })
        .collect();
    println!("birth_ticks={:?} (count={})", birth_ticks, birth_ticks.len());
    println!(
        "marriages={} marriage_children_sum={}",
        sim2.marriage_registry.marriages.len(),
        sim2
            .marriage_registry
            .marriages
            .iter()
            .map(|m| m.children.len())
            .sum::<usize>(),
    );
    println!(
        "live_children_with_parentage={} children_born_field_sum={} pregnancies_open={}",
        sim2.agents.iter().filter(|a| a.parent_a.is_some()).count(),
        sim2.agents
            .iter()
            .map(|a| a.embodied.reproductive.children_born)
            .sum::<u32>(),
        sim2.agents
            .iter()
            .filter(|a| a.embodied.reproductive.pregnancy.is_some())
            .count(),
    );
    let stress_mean: f64 = sim2
        .agents
        .iter()
        .map(|a| a.cognitive.stress.to_f64())
        .sum::<f64>()
        / sim2.agents.len().max(1) as f64;
    println!(
        "avg_cognitive_stress_end={:.4} population={}",
        stress_mean,
        sim2.agents.len()
    );
    // Script envelope at end (focal drift bounded?).
    let red_mean: f64 = sim2
        .agents
        .iter()
        .map(|a| a.narrative.redemption_script.to_f64())
        .sum::<f64>()
        / sim2.agents.len().max(1) as f64;
    let cont_mean: f64 = sim2
        .agents
        .iter()
        .map(|a| a.narrative.contamination_script.to_f64())
        .sum::<f64>()
        / sim2.agents.len().max(1) as f64;
    let hero_mean: f64 = sim2
        .agents
        .iter()
        .map(|a| a.narrative.heroism_script.to_f64())
        .sum::<f64>()
        / sim2.agents.len().max(1) as f64;
    println!(
        "script_envelope_end red={red_mean:.3} cont={cont_mean:.3} hero={hero_mean:.3}"
    );
}
