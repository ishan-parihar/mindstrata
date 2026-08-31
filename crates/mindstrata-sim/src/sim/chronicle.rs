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
/// marriages, revolutions, feuds), any collective-memory entry the village
/// recorded in it, and its curated institutional decisions (policy
/// enactments, poor-relief grants) surfaced from the provenance ledger
/// (Iteration 265). Routine bookkeeping traces (taxes, wages) are
/// deliberately not annal-worthy.
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
    // Iteration 265: curated decision-trace provenance by year. The trace
    // vec is insertion-ordered (= tick order), so per-year phrase order is
    // deterministic; identical phrases dedup so a repeated grant reads once.
    let mut decisions_by_year: std::collections::BTreeMap<u64, Vec<String>> =
        std::collections::BTreeMap::new();
    for tr in sim.provenance.institutional_traces() {
        let inst = tr.institution_name.to_lowercase();
        let phrase = match tr.decision_kind.as_str() {
            "policy_enacted" => Some(format!("the {inst} enacted a new policy")),
            "poor_relief" => Some(format!("the {inst} granted relief to the destitute")),
            _ => None,
        };
        if let Some(p) = phrase {
            decisions_by_year.entry(tr.tick / tpy).or_default().push(p);
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
        if let Some(decisions) = decisions_by_year.get(&year) {
            let mut phrases: Vec<String> = decisions.clone();
            phrases.sort();
            phrases.dedup();
            lines.append(&mut phrases);
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

    // DC-1 CLIENT 19-22 polish: surface the polarity wire.
    // `i278` probe shows 13/13 agents accumulate claims in 2000 ticks; the
    // dossier now lists the per-line claim count and the top-3 dominant
    // polarity-state distribution. Read-only summary — no reconciliation
    // counter (that requires a contract freeze on integration policy).
    if !a.polarity_claims.is_empty() {
        use std::collections::HashMap;
        let mut by_line: HashMap<&'static str, usize> = HashMap::new();
        let mut by_state: HashMap<mindstrata_development::polarity::PolarityState, usize> =
            HashMap::new();
        for claim in &a.polarity_claims {
            *by_line.entry(claim.line.slug()).or_insert(0) += 1;
            *by_state.entry(claim.polarity).or_insert(0) += 1;
        }
        out.push_str(&format!(
            "\npolarity claims: {} total across {} lines\n",
            a.polarity_claims.len(),
            by_line.len()
        ));
        let mut line_vec: Vec<(&&'static str, &usize)> = by_line.iter().collect();
        line_vec.sort_by(|x, y| y.1.cmp(x.1));
        for (line, count) in line_vec.iter().take(3) {
            out.push_str(&format!("  {line} × {count}\n"));
        }
        // Polarity-state tally (Undiscovered/ActiveTension/Integrated).
        let states: [&str; 3] = ["Undiscovered", "ActiveTension", "Integrated"];
        let summary: Vec<String> = states
            .iter()
            .filter_map(|s| {
                let key = match *s {
                    "Undiscovered" => mindstrata_development::polarity::PolarityState::Undiscovered,
                    "ActiveTension" => {
                        mindstrata_development::polarity::PolarityState::ActiveTension
                    }
                    "Integrated" => mindstrata_development::polarity::PolarityState::Integrated,
                    _ => return None,
                };
                by_state.get(&key).map(|c| format!("{s}={c}"))
            })
            .collect();
        if !summary.is_empty() {
            out.push_str(&format!("  state: {}\n", summary.join(" · ")));
        }
    }

    // DC-2.7 lore dossier surface: parallel to polarity_claims.
    // `system_polarity_claim_emit` keeps `lore_archetypes` in lock-step
    // with `polarity_claims`, so any agent with claims now has a lore
    // framing history. Show total, distinct frames, and top-3 archetype
    // tally (deterministic, read-only, no CLIENT coupling).
    if !a.lore_archetypes.is_empty() {
        use std::collections::HashMap;
        let mut by_arch: HashMap<&'static str, usize> = HashMap::new();
        for arch in &a.lore_archetypes {
            *by_arch.entry(arch.slug()).or_insert(0) += 1;
        }
        out.push_str(&format!(
            "\nlore archetypes: {} total across {} frames\n",
            a.lore_archetypes.len(),
            by_arch.len()
        ));
        let mut arch_vec: Vec<(&&'static str, &usize)> = by_arch.iter().collect();
        arch_vec.sort_by(|x, y| y.1.cmp(x.1));
        for (slug, count) in arch_vec.iter().take(3) {
            out.push_str(&format!("  {slug} × {count}\n"));
        }
    }

    out
}

/// Iteration 264: shared index-or-name resolution for every dossier surface
/// (CLI `--dossier NAME`, TUI `/` search). A numeric spec selects by index
/// when in range; otherwise exact name match first, then a UNIQUE prefix.
/// Ambiguous prefixes return `None` — callers surface their own message.
pub fn resolve_agent_spec(sim: &Simulation, spec: &str) -> Option<usize> {
    if let Ok(idx) = spec.parse::<usize>() {
        return (idx < sim.agents.len()).then_some(idx);
    }
    let exact = sim.agents.iter().position(|a| a.name == spec);
    exact.or_else(|| {
        let prefix_hits: Vec<usize> = sim
            .agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.name.starts_with(spec))
            .map(|(i, _)| i)
            .collect();
        if prefix_hits.len() == 1 {
            Some(prefix_hits[0])
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{render_chronicle, render_dossier, resolve_agent_spec};
    use crate::sim::SimConfig;
    use crate::Simulation;

    fn seeded_sim() -> Simulation {
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
        sim
    }

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
    fn chronicle_annals_surface_institutional_decisions() {
        // Iteration 265: policy enactments from the provenance ledger become
        // year lines; routine tax/wage traces stay out of the narrative.
        let config = SimConfig {
            seed: 42,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(2000);
        let a = render_chronicle(&sim);
        assert!(a.contains("enacted a new policy"), "{a}");
        assert!(!a.contains("wage payment"), "wages are not annal-worthy");
        assert_eq!(a, render_chronicle(&sim), "annals stay deterministic");
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

    #[test]
    fn agent_spec_resolves_index_exact_and_unique_prefix() {
        // Iteration 264: the shared resolver backs CLI `--dossier` and the TUI
        // `/` search — one contract: numeric index in range, exact name, then
        // unique prefix; ambiguous/missing specs resolve to None.
        let sim = seeded_sim();
        let name0 = sim.agents[0].name.clone();
        assert_eq!(resolve_agent_spec(&sim, "0"), Some(0));
        assert_eq!(resolve_agent_spec(&sim, &name0), Some(0));
        let prefix = name0[..1].to_string();
        let hits: Vec<usize> = sim
            .agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.name.starts_with(prefix.as_str()))
            .map(|(i, _)| i)
            .collect();
        if hits.len() == 1 {
            assert_eq!(resolve_agent_spec(&sim, &prefix), Some(hits[0]));
        } else {
            assert_eq!(resolve_agent_spec(&sim, &prefix), None);
        }
        // Ambiguous prefix: every founder pool has at least one shared first
        // letter (24 names over 12 agents) — assert the contract, not luck.
        for letter in 'a'..='z' {
            let count = sim
                .agents
                .iter()
                .filter(|a| a.name.starts_with(letter))
                .count();
            if count > 1 {
                assert_eq!(
                    resolve_agent_spec(&sim, &letter.to_string()),
                    None,
                    "{count} agents share prefix {letter}"
                );
                break;
            }
        }
        // Out-of-range index and unknown names miss cleanly.
        assert_eq!(resolve_agent_spec(&sim, "9999"), None);
        assert_eq!(resolve_agent_spec(&sim, "Zaphod"), None);
    }

    /// DC-1 CLIENT 19-22 polish: dossier surfaces the polarity wire
    /// (i278: 13/13 agents accumulate claims in 2000 ticks). The
    /// dossier prints `polarity claims: N total across M lines` plus
    /// the top-3 line tally and the polarity-state distribution. This
    /// test runs the simulation long enough to accumulate claims and
    /// asserts the section appears.
    #[test]
    fn dossier_polarity_section_appears_after_run() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(2000);
        let mut found = false;
        for i in 0..sim.agents.len() {
            let d = render_dossier(&sim, i);
            if d.contains("polarity claims:") {
                found = true;
                assert!(d.contains("total across"), "{d}");
                assert!(d.contains("state:"), "{d}");
                break;
            }
        }
        assert!(found, "no agent had polarity claims in dossier");
    }

    /// DC-2.7 lore dossier surface: `lore_archetypes` is kept in
    /// lock-step with `polarity_claims`, so the same 2000-tick run
    /// that surfaces polarity must also surface lore frames.
    #[test]
    fn dossier_lore_section_appears_after_run() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(2000);
        let mut found = false;
        for i in 0..sim.agents.len() {
            let d = render_dossier(&sim, i);
            if d.contains("lore archetypes:") {
                found = true;
                assert!(d.contains("total across"), "{d}");
                assert!(d.contains("frames"), "{d}");
                break;
            }
        }
        assert!(found, "no agent had lore archetypes in dossier");
    }
}
