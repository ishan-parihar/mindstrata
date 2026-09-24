//! i348 — in-vivo liveness leg: does any agent ever enter the Background tier
//! with the re-contracted gate, and does it re-promote (non-trapping)?
//!
//! Run: cargo run --release -p mindstrata-benches --example i348_background_liveness

use mindstrata_sim::agent_tier::AgentTier;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    println!("i348 — Background liveness (3 seeds, calm, post-fix)\n");
    for n in [12u32, 48] {
        for seed in [1u64, 5, 42] {
            let ticks = 10_000u64;
            let mut sim = Simulation::new(SimConfig {
                seed,
                max_ticks: ticks,
                world_width: 32,
                world_height: 32,
                num_agents: n,
                snapshot_interval: None,
            });
            sim.populate();
            // Births grow the population beyond N; index defensively.
            let mut ever_background: Vec<bool> = vec![false; sim.agents.len()];
            let mut bg_agent_ticks = 0u64;
            let mut promotions = 0u64;
            let mut demotions = 0u64;
            let mut last: Vec<AgentTier> = sim.agents.iter().map(|a| a.agent_tier.tier).collect();
            for _ in 0..ticks {
                sim.tick();
                if sim.agents.len() > ever_background.len() {
                    let added = sim.agents.len() - ever_background.len();
                    ever_background.extend(std::iter::repeat_n(false, added));
                    last.extend(std::iter::repeat_n(AgentTier::Secondary, added));
                }
                for (i, a) in sim.agents.iter().enumerate() {
                    // Diagnostic: agents below the shipped entry gate (0.22) —
                    // what blocks entry? (i348: the first-draft 0.32 threshold
                    // was a superset; the degree<2 refutation is a fortiori.)
                    if a.agent_tier.narrative_importance.to_f64() < 0.22 {
                        static ONCE: std::sync::atomic::AtomicU64 =
                            std::sync::atomic::AtomicU64::new(0);
                        if ONCE.load(std::sync::atomic::Ordering::Relaxed) < 10 {
                            ONCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            eprintln!(
                                "  below-gate: importance={:.4} tier={:?} fear={:.3} anger={:.3} bonds_noticed={}",
                                a.agent_tier.narrative_importance.to_f64(),
                                a.agent_tier.tier,
                                a.emotions.fear.to_f64(),
                                a.emotions.anger.to_f64(),
                                a
                                    .relationship_v2s
                                    .iter()
                                    .filter(|r| {
                                        !matches!(
                                            r.stage,
                                            mindstrata_sim::social::relationship_v2::RelationshipStage::Unnoticed
                                        )
                                    })
                                    .count()
                            );
                        }
                    }
                    if a.agent_tier.tier == AgentTier::Background {
                        ever_background[i] = true;
                        bg_agent_ticks += 1;
                    }
                    if last[i] != AgentTier::Background
                        && a.agent_tier.tier == AgentTier::Background
                    {
                        demotions += 1;
                    }
                    if last[i] == AgentTier::Background
                        && a.agent_tier.tier != AgentTier::Background
                    {
                        promotions += 1;
                    }
                    last[i] = a.agent_tier.tier;
                }
            }
            let count = ever_background.iter().filter(|b| **b).count();
            println!(
                "  N={n} seed={seed}: ever-Background {count}/{} agents · {bg_agent_ticks} Background agent-ticks · {demotions} demotions / {promotions} re-promotions",
                sim.agents.len()
            );
            // i348 conjunct survey: the `degree < 2` clause blocks every
            // candidate (everyone has Noticed+ rows). What does the entry
            // population look like under a stage-weighted degree — rows at
            // Familiar+ (≥3 interactions, trust ≥0.3) instead of any non-
            // Unnoticed row?
            let degrees = sim.contacted_degrees();
            let mut candidates_raw = 0u64;
            let mut candidates_deg0 = 0u64;
            let mut candidates_deg_lt2 = 0u64;
            let mut candidates_deg_lt4 = 0u64;
            let mut contacted_degrees: Vec<u32> = Vec::new();
            for (i, a) in sim.agents.iter().enumerate() {
                let deg = degrees.get(i).copied().unwrap_or(0);
                contacted_degrees.push(deg);
                if a.agent_tier.narrative_importance.to_f64() < 0.22 {
                    candidates_raw += 1;
                    if deg == 0 {
                        candidates_deg0 += 1;
                    }
                    if deg < 2 {
                        candidates_deg_lt2 += 1;
                    }
                    if deg < 4 {
                        candidates_deg_lt4 += 1;
                    }
                }
            }
            contacted_degrees.sort_unstable();
            let pct = |p: usize| -> u32 {
                *contacted_degrees
                    .get(p * contacted_degrees.len() / 100)
                    .unwrap_or(contacted_degrees.last().unwrap_or(&0))
            };
            println!(
                "    below-gate agents: {candidates_raw} · contacted-deg==0: {candidates_deg0} · <2: {candidates_deg_lt2} · <4: {candidates_deg_lt4} · contacted degree p10/p50/p90 = {}/{}/{}",
                pct(10),
                pct(50),
                pct(90)
            );
        }
    }
}
