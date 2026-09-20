//! i342 — is `relationship_v2s.len()` a valid "social connections" proxy?
//!
//! Found while building the i341 coupling map. Two appraisal channels read the
//! agent's relationship-list LENGTH as its number of social connections:
//!
//!   * `social_visibility` (attachment/isolation term): `0.5 | 0.0 + 0.1·min(len,4)`
//!   * normal-life anxiety's `social_factor`: `(4 − min(len,4))·0.002` — "few
//!     connections ⇒ worry"
//!
//! But i335 pinned that list at exactly N−1 rows for every agent: it is a
//! complete graph, so the length measures the POPULATION, not the agent's
//! sociality. `min(len, 4)` is therefore 4 for every agent at every N ≥ 5 — the
//! clamp fires for everyone and both channels become the same constant for the
//! isolate and the village's best-connected agent. That is the §4.3 dead-producer
//! shape: live code, zero discrimination.
//!
//! This probe measures it directly: list length vs *contacted* rows
//! (`interaction_count > 0`), and the two derived quantities that are supposed
//! to differentiate agents.
//!
//! Run: cargo run --release -p mindstrata-benches --example i342_social_count_proxy -- [N...]

use mindstrata_sim::sim::{SimConfig, Simulation};

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
    println!("i342 — is the relationship-count proxy a constant? (seed 42, 32x32, 5K ticks)\n");
    println!(
        "{:>5} {:>12} {:>14} {:>14} {:>16} {:>18}",
        "N", "list len", "contacted(mean)", "contacted(max)", "min(len,4)==4", "social_visibility"
    );
    for n in ns {
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 5_000,
            world_width: 32,
            world_height: 32,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();
        sim.run(5_000);

        let len: Vec<usize> = sim
            .agents
            .iter()
            .map(|a| a.relationship_v2s.len())
            .collect();
        // Contacted rows per agent — the production accessor (i342), so the probe
        // reads the same count the queued fix would wire in.
        let contacted: Vec<usize> = sim
            .contacted_degrees()
            .iter()
            .map(|d| *d as usize)
            .collect();
        let clamped_all = len.iter().all(|l| (*l).min(4) == 4);
        let mean_len = len.iter().sum::<usize>() as f64 / len.len().max(1) as f64;
        let mean_contacted = contacted.iter().sum::<usize>() as f64 / contacted.len().max(1) as f64;
        let max_contacted = contacted.iter().copied().max().unwrap_or(0);
        // The two derived quantities, computed from the raw list length where the
        // sim computes them (partnered case for the upper bound).
        let vis_lo = FixedLike::visibility(false, len[0]);
        let vis_hi = FixedLike::visibility(true, len[0]);
        let anx_factor = FixedLike::anxiety_social_factor(len[0]);
        println!(
            "{:>5} {:>12.1} {:>14.1} {:>14} {:>16} {:>10.3}/{:<.3} anx {:>5.3}",
            n,
            mean_len,
            mean_contacted,
            max_contacted,
            if clamped_all {
                "yes (all agents)"
            } else {
                "no"
            },
            vis_hi,
            vis_lo,
            anx_factor
        );
    }
    println!(
        "\nreading: if every agent shows `min(len,4)==4` and one constant social_visibility, the\n\
         channel cannot distinguish an isolate from the best-connected agent — the proxy measures\n\
         the population, not sociality, and the fix is a contacted-row count (which is also the\n\
         quantity i341 says a sparse store must define)."
    );
}

/// The two appraisal expressions, transcribed so the probe reports exactly what
/// the sim computes (f64 here; the sim uses `Fixed`, whose quantization at these
/// magnitudes is below the printed precision).
struct FixedLike;

impl FixedLike {
    fn visibility(partnered: bool, len: usize) -> f64 {
        let term = (len as f64).min(4.0) * 0.1;
        if partnered {
            (0.5 + term).clamp(0.0, 1.0)
        } else {
            term.clamp(0.0, 1.0)
        }
    }
    fn anxiety_social_factor(len: usize) -> f64 {
        (4.0 - (len as f64).min(4.0)) * 0.002
    }
}
