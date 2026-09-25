//! i426 — formation-time vs steady-state: the i403 §6 precondition probe.
//!
//! The i403 rejection measured the honest (dyadic v2) surface starving the
//! crisis engine — belief-ecology conviction halved (0.486 → 0.215), panics
//! dark on 9-of-10 pestilence seeds — but did NOT decompose WHEN the deficit
//! forms. Two candidate shapes need opposite repairs:
//!
//! * **FORMATION-TIME**: the founding belief ecology is built while v2 trust
//!   climbs from its ≈0.4 stranger prior at the ratchet's ~10×-slower pace,
//!   so early channel traffic under-clears the trust gates (knowledge-
//!   diffusion acceptance `trust×0.5 + openness×0.5 ≥ 0.5`; ToM Friendly
//!   band `trust > 0.5`) and the scar persists forever.
//! * **STEADY-STATE**: v2's steady-state trust sits below the channel bars
//!   permanently, so the deficit is not a ramp artifact at all.
//!
//! This probe runs the crisis corpus (pestilence, the i388 revolution family
//! {5, 42, 12345}) on the CURRENT tree — where the channels read v1 (the
//! ratcheted store) — and measures, per time bucket, the gate-clearing rates
//! for the SAME contacted pairs on BOTH stores (both exist simultaneously;
//! the channels' gate formulas are mirrored exactly, from their sites:
//! social_cluster.rs §19.5.I for knowledge-diffusion, §8.1.9 for ToM), plus
//! the belief-ecology stock (counts, mean confidence, propositions 0/1
//! holder counts and mean emotional charge — the panic route's inputs, the
//! charge law being distress-driven but applying only to HELD propositions),
//! plus each distinct (agent, proposition) belief's first-seen tick.
//!
//! Verdict logic: if v2's gate shares start far below v1's and converge by
//! tick T, the formation window is [0, T) and the formation-time shape holds
//! (the re-derivation then targets the early-world ramp); if the v2 shares
//! stay depressed at 20K, the deficit is steady-state (the charge/gate laws
//! need re-derivation against the honest surface, i381 gap method).
//!
//! Run: cargo run --release -p mindstrata-benches --example i426_belief_formation_surface
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;
use std::collections::HashMap;

const TICKS: u64 = 20_000;
const SAMPLE_EVERY: u64 = 1_440;

fn main() {
    println!(
        "i426 belief-formation surfaces (pestilence, {TICKS} ticks, sample every {SAMPLE_EVERY}):"
    );
    println!(
        "  gates per contacted ordered pair: KD = trust×0.5+openness×0.5 ≥ 0.5; ToM = trust > 0.5"
    );
    for seed in [5u64, 42, 12345] {
        let mut sc = Scenario::pestilence();
        sc.seed = seed;
        sc.ticks = TICKS;
        let mut sim = Simulation::from_scenario(sc);
        sim.populate();

        // v1 pair index: rows are (from,to)-keyed; rebuilt each sample (the
        // store only grows, and or_insert keeps the first row per pair).
        let mut rel_idx: HashMap<(u64, u64), usize> = HashMap::new();
        // Distinct-belief first-seen: (agent, proposition) -> tick of first
        // observation. Entries first seen at the initial sample include
        // everything formed in [0, SAMPLE_EVERY) — the genesis window.
        let mut first_seen: HashMap<(usize, u64), u64> = HashMap::new();

        println!("seed {seed}:");
        println!("  tick pairs v1mean v2mean | KDv1 KDv2 ToMv1 ToMv2 | beliefs conf p0 p1 charge");
        for t in 0..TICKS {
            if t > 0 && t % SAMPLE_EVERY == 0 {
                for (p, r) in sim.relationships().iter().enumerate() {
                    rel_idx.entry((r.from.as_u64(), r.to.as_u64())).or_insert(p);
                }
                let rels = sim.relationships();
                let n = sim.agents.len();
                let (mut pairs, mut v1_sum, mut v2_sum) = (0u64, 0.0f64, 0.0f64);
                let (mut kd1, mut kd2, mut tom1, mut tom2) = (0u64, 0u64, 0u64, 0u64);
                let (mut v1_gt_v2, mut open_sum) = (0u64, 0.0f64);
                let mut open_min = f64::INFINITY;
                let mut open_max = f64::NEG_INFINITY;
                for i in 0..n {
                    for j in 0..n {
                        if i == j {
                            continue;
                        }
                        let Some(v2) = sim.relationship_v2_between(i, j) else {
                            continue;
                        };
                        if v2.interaction_count == 0 {
                            continue;
                        }
                        let v1_t = rel_idx
                            .get(&(i as u64, j as u64))
                            .map_or(0.5, |&p| rels[p].trust.to_f64());
                        let v2_t = v2.trust.to_f64();
                        // §19.5.I reads the RECIPIENT's cultural openness.
                        let open_j = sim.agents[j].cultural.openness.to_f64();
                        pairs += 1;
                        v1_sum += v1_t;
                        v2_sum += v2_t;
                        open_sum += open_j;
                        open_min = open_min.min(open_j);
                        open_max = open_max.max(open_j);
                        if v1_t > v2_t {
                            v1_gt_v2 += 1;
                        }
                        if t == SAMPLE_EVERY && pairs <= 5 {
                            println!("    raw i={i} j={j} v1={v1_t:.4} v2={v2_t:.4} open_j={open_j:.4} v2from={} v2to={}", v2.from.as_u64(), v2.to.as_u64());
                        }
                        if v1_t * 0.5 + open_j * 0.5 >= 0.5 {
                            kd1 += 1;
                        }
                        if v2_t * 0.5 + open_j * 0.5 >= 0.5 {
                            kd2 += 1;
                        }
                        if v1_t > 0.5 {
                            tom1 += 1;
                        }
                        if v2_t > 0.5 {
                            tom2 += 1;
                        }
                    }
                }
                let (mut beliefs, mut conf_sum) = (0u64, 0.0f64);
                let (mut p0, mut p1, mut charge_sum) = (0u64, 0u64, 0.0f64);
                for (ai, a) in sim.agents.iter().enumerate() {
                    for b in &a.beliefs {
                        beliefs += 1;
                        conf_sum += b.confidence.to_f64();
                        first_seen.entry((ai, b.proposition_id)).or_insert(t);
                        match b.proposition_id {
                            0 => p0 += 1,
                            1 => {
                                p1 += 1;
                                charge_sum += b.emotional_charge.to_f64();
                            }
                            _ => {}
                        }
                    }
                }
                let pct = |c: u64, d: u64| {
                    if d > 0 {
                        c as f64 / d as f64 * 100.0
                    } else {
                        100.0
                    }
                };
                println!(
                    "  {t:>6} {pairs:>4} {:.4} {:.4} | {:>5.1} {:>5.1} {:>5.1} {:>5.1} | {beliefs:>4} {:.4} {p0:>3} {p1:>3} {:.4} | v1>v2 {:>4.1}% open[{:.3},{:.3}] {:.3}",
                    v1_sum / pairs as f64,
                    v2_sum / pairs as f64,
                    pct(kd1, pairs),
                    pct(kd2, pairs),
                    pct(tom1, pairs),
                    pct(tom2, pairs),
                    conf_sum / beliefs as f64,
                    if p1 > 0 { charge_sum / p1 as f64 } else { 0.0 },
                    pct(v1_gt_v2, pairs),
                    open_min,
                    open_max,
                    open_sum / pairs as f64,
                );
            }
            sim.tick();
        }

        // Formation exposure: how many distinct beliefs were first seen in
        // each quarter of the horizon.
        let mut windows = [0u64; 4];
        let quarter = TICKS / 4;
        for &tick in first_seen.values() {
            let idx = ((tick / quarter) as usize).min(3);
            windows[idx] += 1;
        }
        println!(
            "  distinct (agent,prop) beliefs first seen: [0,{}) x{} | [{},{}) x{} | [{},{}) x{} | [{},{}) x{}",
            quarter, windows[0],
            quarter, 2 * quarter, windows[1],
            2 * quarter, 3 * quarter, windows[2],
            3 * quarter, TICKS, windows[3],
        );
    }
}
