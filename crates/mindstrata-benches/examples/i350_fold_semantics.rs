//! i350 — per-edge pass fold semantics: what do the N−1 stranger rows actually
//! contribute to the mean-folds, and how much of the per-edge tick share could a
//! contacted-only iteration buy?
//!
//! i349 made the contacted predicate honest (witness rows no longer stamped
//! village-wide). Post-i349 a stranger v2 row is FROZEN at its prior
//! (trust 0.4 / affection 0.3, stage Unnoticed) — it never moves unless the
//! pair interacts. The per-edge passes still fold it:
//!   - appraisal (i341: largest pass, α 1.937): own_rel_trust_sum/min +
//!     own_rel_count over ALL rows → social_sadness (min_trust < 0.5),
//!     nostalgia/gratitude drains + trust emotion + moral_pride (mean trust)
//!   - cognitive ·cog row sweep: quality_sum / num_connections →
//!     network_centrality → effective_status (×0.15) → status_attraction,
//!     endocrine dominance, patronage
//!   - social_cluster: trust_sum/oblig_sum / rel_count → social_trust →
//!     clans trust_pacify_factor (violence restraint), social_obligation
//!   - trust_sync+reset: matrix rows → epistemic.trust_network entries
//!   - kinship_daily stage pass + marriage/patronage scans: pair eligibility
//!     (legitimate over strangers — a stranger CAN be courted)
//!
//! As N grows the stranger-row share of every mean grows linearly, so each
//! fold's output drifts toward the static prior — the §4.3
//! "reads-a-constant" shape, now in the means. This probe measures, per
//! fold, the all-rows vs contacted-only output delta (the re-contract's
//! blast radius), plus the contacted share's growth with horizon (the win's
//! decay), plus the per-pass cost sizing (the win's ceiling).
//!
//! Run: cargo run --release -p mindstrata-benches --example i350_fold_semantics -- [N...]

use mindstrata_sim::sim::{SimConfig, Simulation};
use mindstrata_social::social::relationship_stages::{is_authority_stage, is_kin_stage};

/// v2 rows that carry LIVE semantics: interacted at least once, or a
/// structurally-assigned kin/authority stage. Everything else is the frozen
/// stranger prior.
fn is_live(rv2: &mindstrata_social::social::relationship_v2::RelationshipV2) -> bool {
    rv2.interaction_count > 0 || is_kin_stage(rv2.stage) || is_authority_stage(rv2.stage)
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (((sorted.len() as f64) * p).ceil() as usize).saturating_sub(1);
    sorted[idx.min(sorted.len() - 1)]
}

struct FoldDelta {
    d_mean_trust: f64,
    d_min_trust: f64,
    d_centrality: f64,
    d_social_trust: f64,
    sadness_flip: bool,
    empty_contacted: bool,
}

fn fold_deltas(sim: &Simulation) -> Vec<FoldDelta> {
    let mut out = Vec::new();
    for agent in &sim.agents {
        let mut all_sum = 0.0f64;
        let mut all_min = 1.0f64;
        let mut all_quality = 0.0f64;
        let mut all_count = 0usize;
        let mut live_sum = 0.0f64;
        let mut live_min = 1.0f64;
        let mut live_quality = 0.0f64;
        let mut live_count = 0usize;
        let mut live_oblig = 0.0f64;
        for rv2 in &agent.relationship_v2s {
            let t = rv2.trust.to_f64();
            let q = rv2.quality().to_f64();
            all_sum += t;
            all_min = all_min.min(t);
            all_quality += q;
            all_count += 1;
            if is_live(rv2) {
                live_sum += t;
                live_min = live_min.min(t);
                live_quality += q;
                live_oblig += rv2.obligation.to_f64();
                live_count += 1;
            }
        }
        let all_mean = if all_count > 0 {
            all_sum / all_count as f64
        } else {
            0.4
        };
        let live_mean = if live_count > 0 {
            live_sum / live_count as f64
        } else {
            0.4
        };
        let all_cent = if all_count > 0 {
            all_quality / all_count as f64
        } else {
            0.0
        };
        let live_cent = if live_count > 0 {
            live_quality / live_count as f64
        } else {
            0.0
        };
        // social_trust: social_cluster divides by len().max(1); empty live set
        // falls back to the stranger prior 0.4 (what the all-rows fold reads
        // today when nothing has happened).
        let live_social = if live_count > 0 { live_mean } else { 0.4 };
        out.push(FoldDelta {
            d_mean_trust: (all_mean - live_mean).abs(),
            d_min_trust: (all_min - live_min).abs(),
            d_centrality: (all_cent - live_cent).abs(),
            d_social_trust: (all_mean - live_social).abs(),
            sadness_flip: (all_min < 0.5) != (if live_count > 0 { live_min } else { 0.4 } < 0.5),
            empty_contacted: live_count == 0,
        });
    }
    out
}

fn contacted_shares(sim: &Simulation) -> (f64, f64) {
    let total: usize = sim.agents.iter().map(|a| a.relationship_v2s.len()).sum();
    let live: usize = sim
        .agents
        .iter()
        .map(|a| a.relationship_v2s.iter().filter(|r| is_live(r)).count())
        .sum();
    (live as f64, total as f64)
}

fn run(n: u32, horizon: u64) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: horizon,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(horizon);
    sim
}

fn main() {
    let ns: Vec<u32> = {
        let args: Vec<u32> = std::env::args()
            .skip(1)
            .filter_map(|s| s.parse().ok())
            .collect();
        if args.is_empty() {
            vec![12, 48, 96]
        } else {
            args
        }
    };

    println!("i350 — fold semantics: all-rows vs contacted-only (seed 42, calm, 32x32)\n");

    // ── Leg A: contacted share vs HORIZON (the win's decay) ──
    // Horizons come from argv (comma-separated); default {2K, 10K, 40K}.
    let horizons: Vec<u64> = {
        let raw = std::env::args().nth(1);
        match raw {
            Some(s) if !s.is_empty() => s
                .split(',')
                .filter_map(|t| t.parse().ok())
                .collect::<Vec<u64>>(),
            _ => vec![2_000, 10_000, 40_000],
        }
    };
    println!("leg A — live-row share vs horizon (N=48):");
    for h in &horizons {
        let sim = run(48, *h);
        let (live, total) = contacted_shares(&sim);
        println!(
            "  @{:>6}: live {}/{} = {:.1}%",
            h,
            live,
            total,
            live / total.max(1.0) * 100.0
        );
    }

    // ── Leg B: fold output deltas at settlement (N sweep @10K) ──
    println!("\nleg B — fold deltas @10K (|all-rows − contacted-only|):");
    for n in &ns {
        let sim = run(*n, 10_000);
        let (live, total) = contacted_shares(&sim);
        let deltas = fold_deltas(&sim);
        let mut dm: Vec<f64> = deltas.iter().map(|d| d.d_mean_trust).collect();
        let mut dmn: Vec<f64> = deltas.iter().map(|d| d.d_min_trust).collect();
        let mut dc: Vec<f64> = deltas.iter().map(|d| d.d_centrality).collect();
        let mut ds: Vec<f64> = deltas.iter().map(|d| d.d_social_trust).collect();
        for v in [&mut dm, &mut dmn, &mut dc, &mut ds] {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        }
        let flips = deltas.iter().filter(|d| d.sadness_flip).count();
        let empties = deltas.iter().filter(|d| d.empty_contacted).count();
        println!(
            "  N={n}: live {live}/{total} ({:.1}%) · dMeanTrust p50 {:.4} p90 {:.4} · dMinTrust p50 {:.4} p90 {:.4}",
            live / total.max(1.0) * 100.0,
            percentile(&dm, 0.5),
            percentile(&dm, 0.9),
            percentile(&dmn, 0.5),
            percentile(&dmn, 0.9)
        );
        println!(
            "        dCentrality p50 {:.4} p90 {:.4} · dSocialTrust p50 {:.4} p90 {:.4} · sadness-threshold flips {flips}/{} · zero-live agents {empties}/{}",
            percentile(&dc, 0.5),
            percentile(&dc, 0.9),
            percentile(&ds, 0.5),
            percentile(&ds, 0.9),
            deltas.len(),
            deltas.len()
        );
    }

    // ── Leg C: per-pass cost sizing (i341 method, 250 warmup + 150 window) ──
    println!("\nleg C — per-edge pass cost (seed 42, 250+150):");
    const PER_EDGE: &[&str] = &[
        "cognitive",
        "appraisal",
        "social_cluster",
        "trust_sync+reset",
        "rel_traces",
        "kinship_daily",
    ];
    for n in [48u32, 96, 192] {
        Simulation::pass_profile_reset();
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 400,
            world_width: 32,
            world_height: 32,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();
        sim.run(250);
        Simulation::pass_profile_reset();
        let t0 = std::time::Instant::now();
        sim.run(150);
        let us = t0.elapsed().as_secs_f64() * 1e6 / 150.0;
        let mut parts: Vec<String> = Vec::new();
        for (name, ns_total, _) in Simulation::pass_profile_totals() {
            if PER_EDGE.contains(&name) {
                parts.push(format!("{name} {:.0}", ns_total as f64 / 150.0 / 1000.0));
            }
        }
        println!("  N={n}: tick {us:.0} µs · {}", parts.join(" · "));
    }

    println!(
        "\nreading: small fold deltas ⇒ the stranger rows are pure constant dilution\n\
         (re-contract onto contacted-only folds, fallback = stranger prior 0.4);\n\
         large deltas ⇒ the rows carry live semantics and the sparse iteration\n\
         needs the fallback semantics designed per pass."
    );
}
