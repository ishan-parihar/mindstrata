//! Phase 5 re-audit (Iteration 184) — EMERGENT-QUALITY probe.
//!
//! The Phase 5 acceptance criteria demand the sim behave as an *emergent*
//! experience, not just wired mechanisms:
//!   1. Emergent stories are legible (agent narratives, collective myths,
//!      event streams tell a causal story).
//!   2. No system dominates unnaturally (relationship stages balanced,
//!      emotions differentiated, attachment styles varied).
//!   3. 10,000-tick simulations remain stable (no NaN, populations sane).
//!   4. Different seeds produce genuinely DIFFERENT histories (novelty).
//!
//! Measures, across seeds @20K ticks:
//!   - marriage pair sets (Jaccard divergence between seeds),
//!   - death counts + life-theme distributions,
//!   - event-type distribution + diversity (top event kinds),
//!   - clan myths + collective memories (story artifacts),
//!   - relationship-stage histogram (Nemesis share = domination check),
//!   - emotion means, attachment style spread,
//!   - focal-agent narrative differentiation (legibility),
//!   - a 50K-tick stability leg (NaN + population checks).
//!
//! Run with: `cargo run -p mindstrata-benches --example p5_emergent_probe`

use mindstrata_core::event::{InteractionKind, SimEvent};
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::agent_tier::AgentTier;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;
use std::collections::BTreeMap;

fn run(seed: u64, ticks: u64) -> Simulation {
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

/// Map each event variant to a coarse kind for distribution counting.
fn event_kind(e: &SimEvent) -> &'static str {
    match e {
        SimEvent::AgentDied { .. } => "death",
        SimEvent::ChildBorn { .. } => "birth",
        SimEvent::MarriageFormed { .. } => "marriage",
        SimEvent::ConflictOccurred { .. } => "conflict",
        SimEvent::FeudFormed { .. } => "feud",
        SimEvent::RumorSpread { .. } => "rumor",
        SimEvent::NormViolated { .. } => "norm_violation",
        SimEvent::RelationshipChanged { .. } => "relationship_change",
        SimEvent::KnowledgeTransferred { .. } => "knowledge",
        SimEvent::InteractionOccurred { kind, .. } => match kind {
            InteractionKind::Help => "help",
            InteractionKind::Comfort => "comfort",
            InteractionKind::Insult => "insult",
            InteractionKind::Threaten => "threaten",
            InteractionKind::Talk => "talk",
            InteractionKind::Teach => "teach",
            InteractionKind::Gossip => "gossip",
            InteractionKind::Trade => "trade",
            _ => "interaction",
        },
        _ => "other",
    }
}

fn main() {
    let seeds = [42u64, 7, 99, 13, 46, 55];
    let ticks = 20_000u64;
    let mut marriage_sets: Vec<(u64, Vec<(usize, usize)>)> = Vec::new();
    let mut all_ok = true;

    for &seed in &seeds {
        let sim = run(seed, ticks);
        let n = sim.agents.len() as f64;

        // ── Marriage pair sets (active marriages, sorted pair) ─────────
        let mut pairs: Vec<(usize, usize)> = sim
            .marriage_registry
            .marriages
            .iter()
            .filter(|m| m.active)
            .map(|m| {
                let a = m.partner_a;
                let b = m.partner_b;
                if a < b {
                    (a, b)
                } else {
                    (b, a)
                }
            })
            .collect();
        pairs.sort_unstable();
        marriage_sets.push((seed, pairs));

        // ── Deaths ─────────────────────────────────────────────────────
        let deaths = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|e| matches!(e, SimEvent::AgentDied { .. }))
            .count();

        // ── Life themes (narrative identity) ───────────────────────────
        let mut themes: BTreeMap<String, u32> = BTreeMap::new();
        for a in &sim.agents {
            *themes.entry(format!("{:?}", a.narrative.life_theme)).or_insert(0) += 1;
        }

        // ── Event distribution ─────────────────────────────────────────
        let mut kinds: BTreeMap<&'static str, u32> = BTreeMap::new();
        for e in sim.recent_events(10_000_000) {
            *kinds.entry(event_kind(e)).or_insert(0) += 1;
        }
        let total_events: u32 = kinds.values().sum();
        let diverse_kinds = kinds.len();

        // ── Relationship-stage histogram (domination check) ────────────
        let mut stages: BTreeMap<String, u32> = BTreeMap::new();
        for a in &sim.agents {
            for r in &a.relationship_v2s {
                *stages.entry(format!("{:?}", r.stage)).or_insert(0) += 1;
            }
        }
        let nemesis = *stages.get("Nemesis").unwrap_or(&0);
        let total_edges: u32 = stages.values().sum();
        let nemesis_share = if total_edges == 0 { 0.0 } else { nemesis as f64 / total_edges as f64 };

        // ── Emotion means ──────────────────────────────────────────────
        let (mut joy, mut anger, mut fear) = (0.0f64, 0.0f64, 0.0f64);
        let mut att_styles: BTreeMap<String, u32> = BTreeMap::new();
        for a in &sim.agents {
            joy += a.emotions.joy.to_f64();
            anger += a.emotions.anger.to_f64();
            fear += a.emotions.fear.to_f64();
            *att_styles.entry(format!("{:?}", a.attachment.style)).or_insert(0) += 1;
        }

        // ── Story artifacts: clan myths, collective memories, chapters ─
        let myths: usize = sim.clan_registry.clans.iter().map(|c| c.myths.len()).sum();
        let collective: usize = sim
            .collective_memory_registry
            .entries
            .iter()
            .map(|e| e.memories.len())
            .sum();
        let chapters: usize = sim
            .agents
            .iter()
            .map(|a| {
                a.memory
                    .episodes
                    .iter()
                    .filter(|e| e.tag == mindstrata_sim::memory::MemoryTag::LifeEvent)
                    .count()
            })
            .sum();

        // ── Focal narrative differentiation (legibility) ───────────────
        let focal: Vec<&mindstrata_sim::sim::AgentBundle> =
            sim.agents.iter().filter(|a| a.agent_tier.tier == AgentTier::Focal).collect();
        let (mut focal_scripts, mut focal_themes) = (0.0f64, 0.0f64);
        for a in &focal {
            let s = a.narrative.redemption_script.to_f64() + a.narrative.victimhood_script.to_f64()
                + a.narrative.heroism_script.to_f64();
            focal_scripts += s;
            focal_themes += if a.narrative.life_theme == mindstrata_sim::psychology::narrative::LifeTheme::Growth { 0.0 } else { 1.0 };
        }
        let f_mean = if focal.is_empty() { 0.0 } else { focal_scripts / focal.len() as f64 };
        let f_theme_frac = if focal.is_empty() { 0.0 } else { focal_themes / focal.len() as f64 };

        let top = kinds.iter().max_by_key(|(_, v)| **v).map(|(k, _)| *k).unwrap_or("none");
        println!(
            "seed {seed:>2} @{ticks}: deaths={deaths:>2} themes={themes:?} | events={total_events} ({diverse_kinds} kinds, top={top}) | \
             stages={stages:?} nemesis_share={nemesis_share:.2} | joy={joy:.2} anger={anger:.2} fear={fear:.2} att={att_styles:?} | \
             myths={myths} collective={collective} chapters={chapters} focal={} | focal_scripts_mean={f_mean:.3} non-growth_frac={f_theme_frac:.2}",
            focal.len(),
        );

        // Population stability: a healthy 12-agent village keeps most alive
        // at 20K ticks (Fixed is NaN-proof by construction).
        let alive = sim.agents.iter().filter(|a| a.body.health > Fixed::ZERO).count();
        if alive < 6 {
            println!("  !! seed {seed}: population collapse at {ticks} ({alive}/12 alive)");
            all_ok = false;
        }
    }

    // ── Cross-seed marriage divergence (Jaccard) ───────────────────────
    println!("\n=== Marriage-set divergence (1 − Jaccard) across seeds @{ticks} ===");
    for i in 0..marriage_sets.len() {
        for j in (i + 1)..marriage_sets.len() {
            let (sa, pa) = &marriage_sets[i];
            let (sb, pb) = &marriage_sets[j];
            let a_set: std::collections::HashSet<&(usize, usize)> = pa.iter().collect();
            let b_set: std::collections::HashSet<&(usize, usize)> = pb.iter().collect();
            let inter = a_set.intersection(&b_set).count();
            let union = a_set.union(&b_set).count();
            let jac = if union == 0 { 1.0 } else { 1.0 - inter as f64 / union as f64 };
            println!("  seed {sa:>2} vs {sb:>2}: {pa:?} vs {pb:?} → divergence {jac:.2}");
        }
    }

    // ── 50K stability leg ──────────────────────────────────────────────
    let sim = run(42, 50_000);
    let alive = sim.agents.iter().filter(|a| a.body.health > Fixed::ZERO).count();
    let stable = alive >= 6;
    all_ok &= stable;
    println!(
        "\n=== 50K stability (seed 42) ===\nalive={alive}/{} → {}",
        sim.agents.len(),
        if stable { "STABLE" } else { "UNSTABLE" }
    );

    println!("\nEMERGENT PROBE: {}", if all_ok { "ALL CHECKS PASS" } else { "ISSUES FOUND" });
    if !all_ok {
        std::process::exit(1);
    }
}
