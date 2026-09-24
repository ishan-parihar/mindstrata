//! i399 — the marriage v1→v2 migration, sized before touching.
//!
//! `marriage.rs` is a CLOSED v1 LOOP: the formation gate reads v1
//! affection (L70) + trust (L101), the jealousy charge reads v1 trust
//! (L448), and the bond-boost write (+0.2 trust, +0.3 affection per
//! MarriageFormed, both directions) lands in v1 — whose affection has NO
//! other production reader and NO decay. The dyadic store never sees a
//! marriage bond boost at all. Per the i384 rule the read+write loop
//! migrates wholesale; this probe sizes what that moves.
//!
//! MEASURED (N=12/48, seed 42, 20K ticks; evidence/i399_marriage_store.md):
//! * A — affection divergence at the read sites: mean 0.043/0.081,
//!   max 0.59/0.88, sign-flips 8186/142623 samples. Trust is held tight by
//!   the i376 daily sync (mean 0.008/0.019).
//! * B — the write sink: married-pair end affection v1 0.937 vs v2 0.878 —
//!   v1 pins the bond (no decay); the migration makes the boost visible to
//!   the dyadic decay and differentiation machinery.
//! * C — partner selection: 9 argmax flips (N=12); N=48 formed 0 marriages
//!   in 20K (separate pacing fact, recorded) with 45 preference flips.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i399_marriage_store`

use mindstrata_core::event::SimEvent;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn v2_pos(a: usize, b: usize) -> usize {
    if b > a {
        b - 1
    } else {
        b
    }
}

fn village(n: u32, seed: u64) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: 0,
        world_width: 16,
        world_height: 16,
        num_agents: n,
        snapshot_interval: None,
        ..SimConfig::default()
    });
    sim.populate();
    sim
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

fn main() {
    println!("i399 — the marriage v1→v2 migration, sized\n");

    for (n, horizon) in [(12u32, 20_000u64), (48, 20_000)] {
        let mut sim = village(n, 42);

        // ── A — divergence at the read sites, sampled every deca tick ──
        let mut aff_div = Vec::new();
        let mut trust_div = Vec::new();
        let mut aff_flips = 0usize;
        let mut samples = 0usize;
        let mut done = 0u64;
        while done < horizon {
            let target = (done / 10 + 1) * 10;
            sim.run(target - done);
            done = target;
            let agents_n = sim.agents.len();
            let eighteen = mindstrata_core::fixed::Fixed::from_f64(18.0);
            let fifteen = mindstrata_core::fixed::Fixed::from_f64(15.0);
            for i in 0..agents_n {
                for j in (i + 1)..agents_n {
                    let (ai, aj) = (&sim.agents[i], &sim.agents[j]);
                    if ai.age < eighteen || aj.age < eighteen || (ai.age - aj.age).abs() > fifteen {
                        continue;
                    }
                    let a1 = sim
                        .relationships
                        .iter()
                        .find(|r| r.from.as_u64() as usize == i && r.to.as_u64() as usize == j)
                        .map_or(0.0, |r| r.affection.to_f64());
                    let a2 = sim.agents[i].relationship_v2s[v2_pos(i, j)]
                        .affection
                        .to_f64();
                    let t1 = sim
                        .relationships
                        .iter()
                        .find(|r| r.from.as_u64() as usize == i && r.to.as_u64() as usize == j)
                        .map_or(0.0, |r| r.trust.to_f64());
                    let t2 = sim.agents[i].relationship_v2s[v2_pos(i, j)].trust.to_f64();
                    if (a1 - 0.5).signum() != (a2 - 0.5).signum() {
                        aff_flips += 1;
                    }
                    aff_div.push((a1 - a2).abs());
                    trust_div.push((t1 - t2).abs());
                    samples += 1;
                }
            }
        }

        println!("══ N={n}, seed 42, horizon {horizon} ══");
        println!(
            "  A  eligible-pair samples {samples}  |v1−v2| affection mean {:.4} max {:.4}  trust mean {:.4} max {:.4}  affection sign-flips {aff_flips}",
            mean(&aff_div),
            aff_div.iter().cloned().fold(0.0_f64, f64::max),
            mean(&trust_div),
            trust_div.iter().cloned().fold(0.0_f64, f64::max),
        );

        // ── B — the write sink: marriages formed + end-state bond levels ──
        let marriages: Vec<(usize, usize, u64)> = sim
            .recent_events(10_000_000)
            .iter()
            .filter_map(|ev| match ev {
                SimEvent::MarriageFormed {
                    spouse_a,
                    spouse_b,
                    tick,
                } => Some((
                    spouse_a.as_u64() as usize,
                    spouse_b.as_u64() as usize,
                    tick.as_u64(),
                )),
                _ => None,
            })
            .collect();
        let mut m_v1 = Vec::new();
        let mut m_v2 = Vec::new();
        for (i, j, _) in &marriages {
            let a1 = sim
                .relationships
                .iter()
                .find(|r| r.from.as_u64() as usize == *i && r.to.as_u64() as usize == *j)
                .map_or(0.0, |r| r.affection.to_f64());
            let a2 = sim.agents[*i].relationship_v2s[v2_pos(*i, *j)]
                .affection
                .to_f64();
            m_v1.push(a1);
            m_v2.push(a2);
        }
        println!(
            "  B  marriages formed {}  first-formation ticks {:?}  married-pair end affection v1 mean {:.3}  v2 mean {:.3}",
            marriages.len(),
            &marriages
                .iter()
                .map(|(_, _, t)| *t)
                .take(12)
                .collect::<Vec<_>>(),
            mean(&m_v1),
            mean(&m_v2),
        );

        // ── C — partner-selection argmax flips at end state ──
        let agents_n = sim.agents.len();
        let eighteen = mindstrata_core::fixed::Fixed::from_f64(18.0);
        let fifteen = mindstrata_core::fixed::Fixed::from_f64(15.0);
        let mut flips = 0usize;
        for i in 0..agents_n {
            let mut has_eligible = false;
            let mut best1: Option<usize> = None;
            let mut best2: Option<usize> = None;
            let mut s1 = -1.0_f64;
            let mut s2 = -1.0_f64;
            for j in 0..agents_n {
                if i == j {
                    continue;
                }
                let (ai, aj) = (&sim.agents[i], &sim.agents[j]);
                if ai.age < eighteen || aj.age < eighteen || (ai.age - aj.age).abs() > fifteen {
                    continue;
                }
                has_eligible = true;
                // v1 affection for i→j lives in the i→j row regardless of
                // order; v2 affection is per-perspective.
                let (a1, a2) = if j > i {
                    (
                        sim.relationships
                            .iter()
                            .find(|r| r.from.as_u64() as usize == i && r.to.as_u64() as usize == j)
                            .map_or(0.0, |r| r.affection.to_f64()),
                        sim.agents[i].relationship_v2s[v2_pos(i, j)]
                            .affection
                            .to_f64(),
                    )
                } else {
                    (
                        sim.relationships
                            .iter()
                            .find(|r| r.from.as_u64() as usize == j && r.to.as_u64() as usize == i)
                            .map_or(0.0, |r| r.affection.to_f64()),
                        sim.agents[j].relationship_v2s[v2_pos(j, i)]
                            .affection
                            .to_f64(),
                    )
                };
                if a1 > s1 {
                    s1 = a1;
                    best1 = Some(j);
                }
                if a2 > s2 {
                    s2 = a2;
                    best2 = Some(j);
                }
            }
            if has_eligible && best1 != best2 {
                flips += 1;
            }
        }
        println!("  C  argmax partner flips under v2 affection: {flips}");
    }
}
