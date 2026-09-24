//! i392 row 5 — `aggression_threshold` against the escalation gate.
//!
//! i391's census found the gene drawn `U(0.2, 0.9)`, defaulted (0.5), blended at
//! `inherit`, and read by nothing outside `genome.rs`. Meanwhile the escalation
//! gate (`should_escalate`, `clans.rs`) compares a **composed** aggressor
//! score against the population-wide parameter
//! `conflict_escalation_aggression_threshold = 1.2` — a constant standing where
//! a per-agent disposition should be:
//!
//! ```text
//! aggressor_aggression = dominance + risk_tolerance + endo_dominance×0.3 + inhibition_hold
//! escalate ⇔ threat_failed AND aggression > threshold(1.2, village-wide)
//! ```
//!
//! **The harness problem that shaped this probe:** pinning the gene and
//! re-running the world does nothing — the gene is not yet wired, so a pinned
//! world IS the natural world (B/C in the first draft measured zero effect for
//! exactly that reason, and dominance is the *score*, not the *threshold*).
//! The gene's consumer would be the threshold side of the comparison, so the
//! manipulable dial at the same locus is the **score**, not the gene: for each
//! agent we raise `aggressor_aggression` by δ and find the ceiling δ* where
//! the gate first opens. The gene maps onto that ceiling.
//!
//! * **A — occupancy:** how much headroom does the composed score have below
//!   the gate in the corpus (is the band open at all)?
//! * **B — per-agent ceilings δ\***: the manipulable measurement the gene
//!   would ride. A gene-scaled threshold moves the ceiling by (δ* × relative
//!   shift); the spread of δ\* across agents tells us whether a gene spread
//!   produces a gradient or a lottery.
//! * **C — the mapping check:** the candidate law
//!   `threshold(gene) = 1.2 × (1 + (gene − 0.55) × 1.5)` over the realized
//!   ceilings — how many agents cross, gene by gene.
//!
//! **OUTCOME (i392 row 5): MEASURED AND REJECTED.** Legs A–C found the locus correct —
//! a live, load-bearing gate with a *smooth* response surface (pool share
//! 41 → 37 → 31 → 11 → 6 across the gene range), the gradient shape rows 2
//! and 4 lacked. The law was then wired (leg D's harness measures it) and the
//! pre-registered checklist failed on **four independent families plus both
//! goldens**: revolution liveness `[(5,0,1,0),(42,0,2,0),(12345,1,3,1)]`
//! (seed 5 fires zero), fear-contagion presence 10/12 (band 11–12),
//! prediction-error 3/6 (needs a majority), founding `kinship_penalty`
//! 0.5 ≠ 0 at seed 43. The structural reason: every coupling that survives
//! the golden gate here is *dormant in the calibrated window*, and this gene
//! is drawn at founder time — every agent deviates at tick 0, so the two-sided
//! shift is live in the goldens by construction. Reverted; the trait stays
//! deliberately inert. Full record + the state-gated upgrade path:
//! `docs/architecture/AP4-studio/evidence/i395_aggression_threshold.md`.
//!
//! Leg D below is retained as the reproducible rejection harness.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i395_aggression_threshold`
//!

use mindstrata_core::event::SimEvent;
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

const GENE_MIDPOINT: f64 = 0.55; // draw U(0.2, 0.9) mean
const GENE_SPAN: f64 = 1.5;
const BASE_THRESHOLD: f64 = 1.2;

fn threshold_for(gene: f64) -> f64 {
    BASE_THRESHOLD * (1.0 + (gene - GENE_MIDPOINT) * GENE_SPAN)
}

fn village(n: u32, seed: u64, ticks: u64) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim
}

fn violence_count(sim: &Simulation) -> usize {
    sim.recent_events(10_000_000)
        .iter()
        .filter(|e| {
            matches!(
                e,
                SimEvent::ConflictOccurred {
                    kind: mindstrata_core::conflict::ConflictKind::Violence,
                    ..
                }
            )
        })
        .count()
}

fn threat_count(sim: &Simulation) -> usize {
    sim.recent_events(10_000_000)
        .iter()
        .filter(|e| {
            matches!(
                e,
                SimEvent::ConflictOccurred {
                    kind: mindstrata_core::conflict::ConflictKind::Threat,
                    ..
                }
            )
        })
        .count()
}

fn main() {
    println!("i392 row 5 — aggression_threshold → the escalation gate\n");

    // ── A — occupancy: how often the gate even opens in the corpus ────
    println!("══ A — gate occupancy (i273 violence family @4K) ══");
    for seed in [5u64, 42, 7, 12345] {
        let sim = village(12, seed, 4000);
        let mut sim = sim;
        sim.run(4000);
        let v = violence_count(&sim);
        let t = threat_count(&sim);
        let scores: Vec<f64> = sim
            .agents
            .iter()
            .map(|a| a.personality.dominance.to_f64() + a.personality.risk_tolerance.to_f64())
            .collect();
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let max = scores.iter().copied().fold(0.0, f64::max);
        let above = scores.iter().filter(|s| **s > BASE_THRESHOLD).count();
        println!(
            "  s{seed:<6} threats {t:>4}  violence {v:>3}  score mean {mean:.3} max {max:.3}  agents already > 1.2: {above}/12"
        );
    }
    println!();

    // ── B — per-agent escalation ceilings (the gene's dial) ───────────
    // For each agent: find the smallest δ ≥ 0 with (score + δ) > 1.2 —
    // i.e. how much more "provocation score" the agent needs before the
    // gate could open. δ* = 0 means the agent is already above the gate;
    // the distribution of δ* is what a gene-scaled threshold would reshape.
    println!("══ B — per-agent escalation ceilings δ* = (1.2 − score)⁺ (s42 @4K) ══");
    let mut all_ceilings: Vec<f64> = Vec::new();
    for seed in [5u64, 42, 7, 12345] {
        let sim = village(12, seed, 4000);
        let mut ceilings: Vec<(f64, f64)> = sim
            .agents
            .iter()
            .map(|a| {
                let score =
                    a.personality.dominance.to_f64() + a.personality.risk_tolerance.to_f64();
                let gene = a
                    .embodied
                    .genome
                    .trait_predispositions
                    .aggression_threshold
                    .to_f64();
                (score, gene)
            })
            .map(|(score, gene)| ((BASE_THRESHOLD - score).max(0.0), gene))
            .collect();
        ceilings.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal));
        let zero = ceilings.iter().filter(|(d, _)| *d <= 0.0).count();
        let (mean_d, max_d) = ceilings
            .iter()
            .fold((0.0f64, 0.0f64), |(m, mx), (d, _)| (m + d, mx.max(*d)));
        let n = ceilings.len() as f64;
        println!(
            "  s{seed:<6} already-over-gate: {zero:>2}/12  δ* mean {:.3} max {:.3}",
            mean_d / n,
            max_d
        );
        for (d, g) in &ceilings {
            all_ceilings.push(*d);
            let _ = g;
        }
    }
    println!();

    // ── C — the candidate law over the realized ceilings ───────────────
    println!("══ C — candidate threshold law over realized ceilings ══");
    println!(
        "gene   threshold   agents-clearing (of {}) across the 48-agent pool",
        all_ceilings.len()
    );
    for g in [0.2f64, 0.35, 0.55, 0.75, 0.9] {
        let thr = threshold_for(g);
        let clearing = all_ceilings
            .iter()
            .filter(|d| **d < thr - BASE_THRESHOLD)
            .count();
        println!("{g:<6} {thr:>9.3}   {clearing:>3}");
    }
    println!();
    println!(
        "reading: the gene's consumer is the THRESHOLD side; δ* is the manipulable dial. If the \
         δ* distribution is bimodal (many 0s + a long tail), a gene-scaled threshold is a \
         membership lottery at the band edge (the row-4 signature) — wire only with a \
         same-iteration liveness re-sweep."
    );
    println!();

    // ── D — the wiring check (post-wiring): violence response, pinned genes ──
    println!("══ D — wired response (pinned gene, i273 family @4K) ══");
    println!("seed   gene 0.2   gene 0.55   gene 0.9   (violence count)");
    for seed in [5u64, 42, 7, 12345] {
        let mut cells = Vec::new();
        for g in [0.2f64, 0.55, 0.9] {
            let mut sim = village(12, seed, 4000);
            for a in &mut sim.agents {
                a.embodied.genome.trait_predispositions.aggression_threshold = Fixed::from_f64(g);
            }
            sim.run(4000);
            cells.push(violence_count(&sim));
        }
        println!(
            "s{seed:<5} {:>8} {:>11} {:>10}",
            cells[0], cells[1], cells[2]
        );
    }
    // ── E — the pair the unit tests use: which side of the gate? ──────
    println!("══ E — test-pair agents' genes and scores (s42, clan-founders) ══");
    {
        let sim = village(12, 42, 0);
        let c0 = sim.clan_registry.clans[0].core_households[0];
        let c1 = sim.clan_registry.clans[1].core_households[0];
        for i in [c0, c1] {
            let a = &sim.agents[i];
            let score = a.personality.dominance.to_f64() + a.personality.risk_tolerance.to_f64();
            let gene = a
                .embodied
                .genome
                .trait_predispositions
                .aggression_threshold
                .to_f64();
            println!("  agent {i:>2}  dom+risk {score:.3}  gene {gene:.3}");
        }
        println!(
            "  (factors at populate: trust {:?} obligation {:?} taboo_cost {:?} enemies {})",
            sim.agents[c0].relational_fields.social_trust,
            sim.agents[c0].relational_fields.social_obligation,
            sim.agents[c0]
                .cultural_cognition
                .taboo_violation_cost_sum("violence"),
            {
                // clan_of equivalent inline (the method is pub(super)).
                let clan_of = |idx: usize| {
                    sim.clan_registry
                        .clans
                        .iter()
                        .find(|c| c.core_households.contains(&idx))
                        .map(|c| c.id)
                };
                match (clan_of(c0), clan_of(c1)) {
                    (Some(x), Some(y)) => sim
                        .clan_registry
                        .clans
                        .iter()
                        .any(|c| c.id == x && c.is_enemy(y)),
                    _ => false,
                }
            }
        );
    }
}
