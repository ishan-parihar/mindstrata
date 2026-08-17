//! Iteration 185 — 100K-tick long-horizon stability check (Phase 5
//! acceptance: "10,000-tick simulations remain stable" + "no system
//! dominates unnaturally"), across calm / famine / pestilence.
//!
//! Run with: `cargo run -p mindstrata-benches --example p5_100k_probe --release`
//!
//! Measures per run @100K:
//!   - deaths by cause (post Iteration-185 the Violence share must be tiny —
//!     the pre-fix calm world collapsed via 100% Violence deaths),
//!   - births / population turnover,
//!   - marriages / households / clans+myths / collective memories,
//!   - relationship-stage histogram (Nemesis share — was 0.52–0.54 in
//!     collapsed seeds),
//!   - life-theme + attachment distributions, emotion means, stress,
//!   - grain/water, top event kinds, revolutions.

use mindstrata_core::event::{InteractionKind, SimEvent};
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;
use std::collections::BTreeMap;

const HORIZON: u64 = 100_000;

fn run_scenario(name: &str, seed: u64) -> Simulation {
    let mut sc = match name {
        "calm" => Scenario::calm(),
        "famine" => Scenario::famine(),
        "pestilence" => Scenario::pestilence(),
        _ => unreachable!(),
    };
    sc.seed = seed;
    // Pestilence carries the per-tick epidemic machinery (proximity
    // contagion + per-agent disease state), so 100K is computationally
    // heavy; P5_HORIZON lets the check stay within a CI budget while
    // still clearing the 10K acceptance bar by 4×.
    let ticks = std::env::var("P5_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(HORIZON);
    sc.ticks = ticks;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    let start = std::time::Instant::now();
    sim.run(ticks);
    eprintln!("  [{name} seed {seed}] ran {ticks} ticks in {:?}", start.elapsed());
    sim
}

fn event_kind(e: &SimEvent) -> &'static str {
    match e {
        SimEvent::AgentDied { .. } => "death",
        SimEvent::ChildBorn { .. } => "birth",
        SimEvent::MarriageFormed { .. } => "marriage",
        SimEvent::FeudFormed { .. } => "feud",
        SimEvent::RumorSpread { .. } => "rumor",
        SimEvent::NormViolated { .. } => "norm_violation",
        SimEvent::ConflictOccurred { .. } => "conflict",
        SimEvent::InteractionOccurred { kind, .. } => match kind {
            InteractionKind::Help => "help",
            InteractionKind::Comfort => "comfort",
            InteractionKind::Threaten => "threaten",
            InteractionKind::Insult => "insult",
            InteractionKind::Gossip => "gossip",
            _ => "interaction",
        },
        _ => "other",
    }
}

fn main() {
    let combos = [
        ("calm", 42u64),
        ("calm", 7),
        ("calm", 99),
        ("famine", 42),
        ("famine", 13),
        ("pestilence", 42),
        ("pestilence", 13),
    ];
    let filter = std::env::var("P5_ONLY").unwrap_or_default();
    let mut all_ok = true;

    for (scenario, seed) in combos {
        if !filter.is_empty() && scenario != filter {
            continue;
        }
        let sim = run_scenario(scenario, seed);
        let n = sim.agents.len() as f64;

        // Growth diagnostics (P5_HORIZON < 40000 only)
        if std::env::var("P5_DIAG").is_ok() {
            let n_diseases: usize = sim.agent_diseases.iter().map(std::vec::Vec::len).sum();
            let n_events = sim.recent_events(10_000_000).len();
            let per_agent: Vec<String> = sim
                .agent_diseases
                .iter()
                .enumerate()
                .map(|(i, ds)| {
                    let mut kinds: BTreeMap<String, u32> = BTreeMap::new();
                    for d in ds {
                        *kinds.entry(format!("{:?}", d.kind)).or_insert(0) += 1;
                    }
                    format!("a{i}:{}[{kinds:?}]", ds.len())
                })
                .collect();
            eprintln!(
                "  DIAG {scenario} seed {seed}: events={n_events} diseases_total={n_diseases} journal={} marriages={} | {}",
                sim.journal_len(),
                sim.marriage_registry.marriages.len(),
                per_agent.join(" ")
            );
        }

        // Deaths by cause
        let mut causes: BTreeMap<String, u32> = BTreeMap::new();
        let mut births = 0u32;
        let mut revs = 0u32;
        let mut kinds: BTreeMap<&'static str, u32> = BTreeMap::new();
        for e in sim.recent_events(10_000_000) {
            match e {
                SimEvent::AgentDied { cause, .. } => {
                    *causes.entry(format!("{cause:?}")).or_insert(0) += 1;
                }
                SimEvent::ChildBorn { .. } => births += 1,
                SimEvent::ConflictOccurred {
                    kind: mindstrata_core::conflict::ConflictKind::Revolution,
                    ..
                } => revs += 1,
                _ => {}
            }
            *kinds.entry(event_kind(e)).or_insert(0) += 1;
        }
        let total_deaths: u32 = causes.values().sum();
        let violence = *causes.get("Violence").unwrap_or(&0);
        let violence_share = if total_deaths == 0 {
            0.0
        } else {
            violence as f64 / total_deaths as f64
        };

        // Marriages / households / clans / memories
        let active_marriages = sim
            .marriage_registry
            .marriages
            .iter()
            .filter(|m| m.active)
            .count();
        let multi_hh = sim.households.iter().filter(|h| h.members.len() > 1).count();
        let clans = sim.clan_registry.clans.len();
        let myths: usize = sim.clan_registry.clans.iter().map(|c| c.myths.len()).sum();
        let collective: usize = sim
            .collective_memory_registry
            .entries
            .iter()
            .map(|e| e.memories.len())
            .sum();

        // Stage histogram
        let mut stages: BTreeMap<String, u32> = BTreeMap::new();
        for a in &sim.agents {
            for r in &a.relationship_v2s {
                *stages.entry(format!("{:?}", r.stage)).or_insert(0) += 1;
            }
        }
        let total_edges: u32 = stages.values().sum();
        let nemesis = *stages.get("Nemesis").unwrap_or(&0);
        let nemesis_share = if total_edges == 0 {
            0.0
        } else {
            nemesis as f64 / total_edges as f64
        };

        // Life themes / attachments / emotions / stress
        let mut themes: BTreeMap<String, u32> = BTreeMap::new();
        let mut att: BTreeMap<String, u32> = BTreeMap::new();
        let (mut joy, mut anger, mut fear, mut stress, mut health) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
        for a in &sim.agents {
            *themes.entry(format!("{:?}", a.narrative.life_theme)).or_insert(0) += 1;
            *att.entry(format!("{:?}", a.attachment.style)).or_insert(0) += 1;
            joy += a.emotions.joy.to_f64();
            anger += a.emotions.anger.to_f64();
            fear += a.emotions.fear.to_f64();
            stress += a.embodied.endocrine.stress.level.to_f64();
            health += a.embodied.derived_health().to_f64();
        }

        let grain = sim.total_grain().to_f64();
        let water = sim.total_water().to_f64();

        // Verdicts
        let stable = total_deaths < 30; // ~1 death per 2 sim-years max in calm/famine; pestilence is harder
        let violence_dom = violence_share > 0.5;
        let nemesis_dom = nemesis_share > 0.35;
        let theme_dom = themes.len() <= 1;
        let run_ok = stable && !violence_dom && !nemesis_dom && !theme_dom;
        all_ok &= run_ok;

        let (joy_m, anger_m, fear_m, stress_m, health_m) =
            (joy / n, anger / n, fear / n, stress / n, health / n);
        println!(
            "=== {scenario} seed {seed} @{HORIZON} ===\ndeaths={total_deaths} (violence {violence} = {violence_share:.2}) births={births} revs={revs} | \
             marriages={active_marriages} multi_hh={multi_hh} clans={clans} myths={myths} collective={collective}\n\
             stages={stages:?} nemesis_share={nemesis_share:.2} | themes={themes:?} att={att:?}\n\
             joy={joy_m:.2} anger={anger_m:.2} fear={fear_m:.2} stress={stress_m:.2} health={health_m:.2} | grain={grain:.0} water={water:.0}\n\
             top_events={kinds:?}\n\
             → {}",
            if run_ok { "STABLE" } else { "CHECK" }
        );
    }

    println!("\n100K STABILITY: {}", if all_ok { "PASS — no system dominates" } else { "ISSUES — see above" });
}
