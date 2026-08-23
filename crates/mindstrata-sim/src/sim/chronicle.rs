//! Iteration 259 (audit Phase 6): the legibility layer - village chronicle
//! annals and agent dossiers, generated purely from live simulation state.
//!
//! Both renderers are READ-ONLY pure functions over `Simulation` public
//! accessors: no new sim state, no RNG, no determinism surface. The
//! chronicle groups notable events into year-by-year annals; the dossier
//! renders one agent's lineage, inherited-vs-expressed traits, and life
//! timeline.

use crate::sim::Simulation;
use mindstrata_core::event::SimEvent;
use mindstrata_core::id::AgentId;
use mindstrata_core::ConflictKind;
use mindstrata_psych::journal::JournalEntryKind;

fn event_year(e: &SimEvent, ticks_per_year: u64) -> Option<u64> {
    let t = match e {
        SimEvent::AgentDied { tick, .. }
        | SimEvent::ChildBorn { tick, .. }
        | SimEvent::MarriageFormed { tick, .. }
        | SimEvent::FeudFormed { tick, .. }
        | SimEvent::ConflictOccurred { tick, .. } => tick.as_u64(),
        _ => return None,
    };
    Some(t / ticks_per_year.max(1))
}

/// Village annals: the emergent history made human-readable. One block per
/// simulated year carrying that year's notable events (deaths, births,
/// marriages, revolutions, feuds), plus any collective-memory entry the
/// village recorded in it.
pub fn render_chronicle(sim: &Simulation) -> String {
    let tpy = sim.demography_config.ticks_per_year.max(1);
    let total = sim.current_tick().as_u64();
    let years = total / tpy + 1;

    let name_of = |id: AgentId| -> String {
        sim.agents
            .get(id.as_u64() as usize)
            .map_or_else(|| String::from("someone"), |a| a.name.clone())
    };

    // Collective-memory entries by year.
    let mut memory_by_year: std::collections::BTreeMap<u64, Vec<String>> =
        std::collections::BTreeMap::new();
    for cm in &sim.collective_memory_registry.entries {
        for m in &cm.memories {
            memory_by_year
                .entry(m.event_tick / tpy)
                .or_default()
                .push(m.description.clone());
        }
    }

    let mut out = String::from("-- Village Chronicle --\n");
    for year in 0..years {
        let mut lines: Vec<String> = Vec::new();
        for e in sim.recent_events(10_000_000) {
            if event_year(e, tpy) != Some(year) {
                continue;
            }
            match e {
                SimEvent::AgentDied { agent, cause, .. } => lines.push(format!(
                    "{} died ({})",
                    name_of(*agent),
                    format!("{cause:?}").to_lowercase()
                )),
                SimEvent::ChildBorn { child, .. } => {
                    lines.push(format!("{} was born", name_of(*child)));
                }
                SimEvent::MarriageFormed {
                    spouse_a, spouse_b, ..
                } => lines.push(format!(
                    "{} and {} wed",
                    name_of(*spouse_a),
                    name_of(*spouse_b)
                )),
                SimEvent::ConflictOccurred {
                    kind: ConflictKind::Revolution,
                    ..
                } => lines.push("the council was overthrown".into()),
                SimEvent::FeudFormed {
                    party_a, party_b, ..
                } => lines.push(format!(
                    "a feud ignited between {} and {}",
                    name_of(*party_a),
                    name_of(*party_b)
                )),
                _ => {}
            }
        }
        if let Some(memories) = memory_by_year.get(&year) {
            for m in memories {
                lines.push(format!("the village remembers: {m}"));
            }
        }
        if !lines.is_empty() {
            out.push_str(&format!("\nYear {}\n", year + 1));
            for line in lines {
                out.push_str(&format!("  - {line}\n"));
            }
        }
    }
    if out == "-- Village Chronicle --\n" {
        out.push_str("\n(an unremarkable stretch of years)\n");
    }
    out
}

/// Agent dossier (Phase 6.2): identity, lineage, inherited-vs-expressed
/// trait drift (birth constitution vs current personality), genome
/// highlights, and a life timeline from the journal.
pub fn render_dossier(sim: &Simulation, idx: usize) -> String {
    let Some(a) = sim.agents.get(idx) else {
        return format!("No agent at index {idx}.");
    };
    let name_of = |id: u64| -> String {
        sim.agents
            .get(id as usize)
            .map_or_else(|| String::from("unknown"), |x| x.name.clone())
    };

    let mut out = format!("-- Dossier: {} --\n", a.name);
    out.push_str(&format!(
        "age {:.1} · {:?} · health {:.2}\n",
        a.age.to_f64(),
        a.embodied.reproductive.sex,
        a.body.health.to_f64()
    ));

    let parents: Vec<String> = [a.parent_a, a.parent_b]
        .iter()
        .flatten()
        .map(|p| name_of(*p as u64))
        .collect();
    if parents.is_empty() {
        out.push_str("lineage: founder\n");
    } else {
        out.push_str(&format!("lineage: child of {}\n", parents.join(" & ")));
    }
    if let Some(partner) = a.partner {
        out.push_str(&format!("partner: {}\n", name_of(partner as u64)));
    }
    out.push_str(&format!(
        "children born: {}\n",
        a.embodied.reproductive.children_born
    ));

    // Inherited -> expressed drift: birth constitution vs current traits.
    if let Some(c) = &a.personality.constitution {
        out.push_str("\ninherited -> expressed (top drift):\n");
        let mut drifts: Vec<(&str, f64)> = vec![
            ("openness", (a.personality.openness - c.openness).to_f64()),
            (
                "conscientiousness",
                (a.personality.conscientiousness - c.conscientiousness).to_f64(),
            ),
            (
                "extraversion",
                (a.personality.extraversion - c.extraversion).to_f64(),
            ),
            (
                "agreeableness",
                (a.personality.agreeableness - c.agreeableness).to_f64(),
            ),
            (
                "neuroticism",
                (a.personality.neuroticism - c.neuroticism).to_f64(),
            ),
            ("ambition", (a.personality.ambition - c.ambition).to_f64()),
        ];
        drifts.sort_by(|x, y| {
            y.1.abs()
                .partial_cmp(&x.1.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (trait_name, d) in drifts.iter().take(3) {
            if d.abs() < 0.005 {
                continue;
            }
            let arrow = if *d > 0.0 { "up" } else { "down" };
            out.push_str(&format!("  {trait_name} {arrow} {d:+.3}\n"));
        }
    }

    out.push_str(&format!(
        "genome: stress-reactivity {:.2}, immune {:.2}, fertility {:.2}\n",
        a.embodied
            .genome
            .trait_predispositions
            .stress_reactivity
            .to_f64(),
        a.embodied
            .genome
            .health_predispositions
            .immune_strength
            .to_f64(),
        a.embodied
            .genome
            .fertility_predispositions
            .base_fertility
            .to_f64()
    ));

    // Life timeline: notable journal entries under this slot.
    let tpy = sim.demography_config.ticks_per_year.max(1);
    let mut timeline: Vec<String> = Vec::new();
    for e in sim.journal.entries_in_range(0, u64::MAX) {
        if e.agent.as_u64() as usize != idx {
            continue;
        }
        let year = e.tick / tpy + 1;
        match &e.kind {
            JournalEntryKind::Died { cause, .. } => {
                timeline.push(format!("year {year}: perished ({cause})"));
            }
            JournalEntryKind::Inheritance { amount, .. } => {
                timeline.push(format!("year {year}: inherited {amount:.1} coin"));
            }
            JournalEntryKind::KnowledgeDiscovered { .. } => {
                timeline.push(format!("year {year}: discovered new knowledge"));
            }
            JournalEntryKind::CommittedViolence { target, .. } => timeline.push(format!(
                "year {year}: committed violence against {}",
                name_of(*target)
            )),
            _ => {}
        }
    }
    if !timeline.is_empty() {
        out.push_str("\nlife timeline:\n");
        for line in timeline {
            out.push_str(&format!("  - {line}\n"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{render_chronicle, render_dossier};
    use crate::sim::SimConfig;
    use crate::Simulation;

    #[test]
    fn chronicle_renders_founding_year_and_determinism() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 500,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(500);
        let a = render_chronicle(&sim);
        let b = render_chronicle(&sim);
        assert_eq!(a, b, "chronicle must be deterministic");
        assert!(a.contains("Village Chronicle"));
    }

    #[test]
    fn dossier_shows_founder_lineage_and_genome() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 100,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(100);
        let d = render_dossier(&sim, 0);
        assert!(d.contains("Dossier:"), "{d}");
        assert!(d.contains("lineage: founder"), "{d}");
        assert!(d.contains("genome:"), "{d}");
        let oob = render_dossier(&sim, 9999);
        assert!(oob.contains("No agent"));
    }
}
