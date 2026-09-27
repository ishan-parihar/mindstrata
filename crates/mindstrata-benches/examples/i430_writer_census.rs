//! i430 — the four-writer affection census (advisory follow-up to i429).
//!
//! i429 measured the affection store's trajectory (prior 0.3 → p50 0.945 by
//! day 10, band [0.5,0.95) drained 34%→1.9% by 18K) and verified the per-act
//! law in code (+0.015×vol/act, decay 0.0001/day cold + 0.0144/day dirty).
//! But the per-act law cannot explain a pair crossing 0.5→1.0 inside a day:
//! two chunky writers sit on top — household rituals (`affection +=
//! bonding×0.2` per participant pair, household.rs) and marriage (+0.3,
//! marriage.rs) — and the repair must target whichever writer owns the mass.
//!
//! Census method (no engine changes): daily-boundary diff of the ordered
//! dyadic rows on seed 42 calm N=12, attributing each row's daily Δ by
//! (a) Δsize — ≥0.05/day is unreachable for record_positive (+0.015×vol max)
//!     and is the chunky-writer signature;
//! (b) event overlap from `recent_events` — RitualPerformed participants,
//!     MarriageFormed spouses;
//! (c) per-act rows: 0<Δ<0.05 with interactions that day → measured per-act
//!     gain Δ/interactions;
//! (d) decay rows: Δ<0 with no events → the decay legs' letter.
//! Any chunky day with NO explaining event is reported as an UNKNOWN WRITER.
//!
//! Also emits the advisory's split columns per seed {42,7,11}: share <0.70,
//! band [0.5,0.95), share ≥0.95, contacted p50, and (seed 42) the p50 of the
//! contacted-and-active subset.
//!
//! Run: cargo run --release -p mindstrata-benches --example i430_writer_census
use mindstrata_core::event::SimEvent;
use mindstrata_sim::sim::{SimConfig, Simulation};

const N: usize = 12;
const DAYS: u64 = 139; // 139 × 144 ≈ 20K ticks

fn snapshot(sim: &Simulation) -> Vec<Vec<f64>> {
    let mut m = vec![vec![0.0; N]; N];
    for (i, row) in m.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            if i != j {
                if let Some(rv2) = sim.relationship_v2_between(i, j) {
                    *cell = rv2.affection.to_f64();
                }
            }
        }
    }
    m
}

fn counts(sim: &Simulation) -> Vec<Vec<u32>> {
    let mut m = vec![vec![0u32; N]; N];
    for (i, row) in m.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            if i != j {
                if let Some(rv2) = sim.relationship_v2_between(i, j) {
                    *cell = rv2.interaction_count;
                }
            }
        }
    }
    m
}

fn pct(rows: &[f64], lo: f64, hi: f64) -> f64 {
    rows.iter().filter(|&&v| (lo..hi).contains(&v)).count() as f64 / rows.len() as f64 * 100.0
}

fn splits(sim: &Simulation, label: &str, active_p50: Option<f64>) {
    let mut vals: Vec<f64> = Vec::new();
    for i in 0..N {
        for j in 0..N {
            if i != j {
                if let Some(rv2) = sim.relationship_v2_between(i, j) {
                    if rv2.interaction_count > 0 {
                        vals.push(rv2.affection.to_f64());
                    }
                }
            }
        }
    }
    if vals.is_empty() {
        return;
    }
    vals.sort_unstable_by(f64::total_cmp);
    let p50 = vals[(vals.len() - 1) / 2];
    let extra = active_p50.map_or(String::new(), |a| format!(" active_p50={a:.3}"));
    println!(
        "{label}: n={:>3} p50={:.3} <0.70={:.1}% band[0.5,0.95)={:.1}% >=0.95={:.1}%{extra}",
        vals.len(),
        p50,
        pct(&vals, f64::MIN, 0.7),
        pct(&vals, 0.5, 0.95),
        pct(&vals, 0.95, f64::INFINITY),
    );
}

fn main() {
    println!("i430 — the four-writer affection census (calm 42, 16x16, N={N})");
    let config = SimConfig {
        seed: 42,
        max_ticks: DAYS * 144,
        world_width: 16,
        world_height: 16,
        num_agents: N as u32,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();

    // Per-writer mass accumulators.
    let mut mass_ritual = 0.0;
    let mut mass_marriage = 0.0;
    let mut mass_per_act = 0.0;
    let mut mass_chunky_unknown = 0.0;
    let mut unknown_rows: Vec<(u64, usize, usize, f64, u64)> = Vec::new();
    let mut mass_decay = 0.0;
    let mut days_ritual_rows = 0u64;
    let mut days_marriage_rows = 0u64;
    let mut per_act_gains: Vec<f64> = Vec::new();
    let mut decay_days: Vec<f64> = Vec::new();
    let mut dc_hist: Vec<u64> = Vec::new();
    let mut rituals = 0u64;
    let mut marriages = 0u64;
    let mut bonding_sum = 0.0;
    // Band transit per ordered row: (day crossed 0.5, day crossed 0.95).
    let mut crossed = vec![(0u64, 0u64); N * N];

    let mut prev_aff = snapshot(&sim);
    let mut prev_cnt = counts(&sim);
    let mut prev_tick = 0u64;

    for day in 1..=DAYS {
        for _ in 0..144 {
            sim.tick();
        }
        let now_tick = sim.current_tick().as_u64();
        let aff = snapshot(&sim);
        let cnt = counts(&sim);

        // Events of this window (recent_events is bounded; a day is ~1-3K).
        let mut ritual_pairs: Vec<(usize, usize)> = Vec::new();
        let mut marriage_pairs: Vec<(usize, usize)> = Vec::new();
        for ev in sim.recent_events(8192).iter().rev() {
            match ev {
                SimEvent::RitualPerformed {
                    participants,
                    bonding,
                    tick,
                    ..
                } => {
                    if tick.as_u64() > prev_tick {
                        rituals += 1;
                        bonding_sum += bonding.to_f64();
                        for a in 0..participants.len() {
                            for b in (a + 1)..participants.len() {
                                ritual_pairs.push((
                                    participants[a].as_u64() as usize,
                                    participants[b].as_u64() as usize,
                                ));
                            }
                        }
                    }
                }
                SimEvent::MarriageFormed {
                    spouse_a,
                    spouse_b,
                    tick,
                } if tick.as_u64() > prev_tick => {
                    marriages += 1;
                    marriage_pairs.push((spouse_a.as_u64() as usize, spouse_b.as_u64() as usize));
                }
                _ => {}
            }
        }

        for i in 0..N {
            for j in 0..N {
                if i == j {
                    continue;
                }
                let d = aff[i][j] - prev_aff[i][j];
                let dc = (cnt[i][j] - prev_cnt[i][j]) as f64;
                if dc > 0.0 {
                    dc_hist.push(dc as u64);
                }
                let in_ritual = ritual_pairs
                    .iter()
                    .any(|&(a, b)| (a == i && b == j) || (a == j && b == i));
                let in_marriage = marriage_pairs
                    .iter()
                    .any(|&(a, b)| (a == i && b == j) || (a == j && b == i));
                let slot = i * N + j;
                if d >= 0.05 {
                    if in_marriage {
                        mass_marriage += d;
                        days_marriage_rows += 1;
                    } else if in_ritual {
                        mass_ritual += d;
                        days_ritual_rows += 1;
                    } else {
                        mass_chunky_unknown += d;
                        unknown_rows.push((day, i, j, d, dc as u64));
                    }
                } else if d > 0.0 {
                    mass_per_act += d;
                    if dc > 0.0 {
                        per_act_gains.push(d / dc);
                    }
                } else if d < 0.0 {
                    mass_decay += d;
                    decay_days.push(d);
                }
                if crossed[slot].0 == 0 && aff[i][j] >= 0.5 {
                    crossed[slot].0 = day;
                }
                if crossed[slot].1 == 0 && aff[i][j] >= 0.95 {
                    crossed[slot].1 = day;
                }
            }
        }
        prev_aff = aff;
        prev_cnt = cnt;
        prev_tick = now_tick;
    }

    println!("\n-- writer mass over {DAYS} days (sum of ordered-row deltas) --");
    println!(
        "  ritual   : {mass_ritual:+.3} over {days_ritual_rows} row-days ({:.4}/row-day), events={rituals}, mean bonding={:.3}",
        if days_ritual_rows > 0 { mass_ritual / days_ritual_rows as f64 } else { 0.0 },
        if rituals > 0 { bonding_sum / rituals as f64 } else { 0.0 },
    );
    println!(
        "  marriage : {mass_marriage:+.3} over {days_marriage_rows} row-days, events={marriages}",
    );
    println!(
        "  per-act  : {mass_per_act:+.3} (mean gain {:+.4}/act over {} acts, min {:+.4} max {:+.4})",
        per_act_gains.iter().sum::<f64>() / per_act_gains.len().max(1) as f64,
        per_act_gains.len(),
        per_act_gains.iter().copied().fold(f64::INFINITY, f64::min),
        per_act_gains.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    );
    println!(
        "  unknown  : {mass_chunky_unknown:+.3} over {} row-days — resolution below",
        unknown_rows.len(),
    );
    let zero_act = unknown_rows.iter().filter(|r| r.4 == 0).count();
    let dcs: Vec<u64> = unknown_rows.iter().map(|r| r.4).collect();
    println!(
        "    interaction-delta distribution: dc=0: {zero_act}, dc>0: {}, mean dc={:.2}, max dc={} — dc=0 rows are the hidden-writer candidates",
        dcs.len() - zero_act,
        dcs.iter().sum::<u64>() as f64 / dcs.len().max(1) as f64,
        dcs.iter().copied().max().unwrap_or(0),
    );
    for r in unknown_rows.iter().filter(|r| r.4 == 0).take(12) {
        println!(
            "    HIDDEN day={} ({},{}) d={:+.4} dc=0",
            r.0, r.1, r.2, r.3
        );
    }
    let day1 = unknown_rows.iter().filter(|r| r.0 == 1).count();
    println!(
        "    day-1 rows: {day1}/{} (founding-day writes)",
        unknown_rows.len()
    );
    println!(
        "  decay    : {mass_decay:+.3} over {} row-days (mean {:+.5}, worst {:+.4})",
        decay_days.len(),
        decay_days.iter().sum::<f64>() / decay_days.len().max(1) as f64,
        decay_days.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    );
    dc_hist.sort_unstable();
    if !dc_hist.is_empty() {
        let q = |p: f64| dc_hist[((dc_hist.len() - 1) as f64 * p) as usize];
        println!(
            "  interactions/day over contacted row-days: p10={} p25={} p50={} p75={} p90={} max={} — the taper's sizing input",
            q(0.10),
            q(0.25),
            q(0.50),
            q(0.75),
            q(0.90),
            dc_hist[dc_hist.len() - 1]
        );
    }

    println!("\n-- band transit (ordered rows that ever crossed 0.5) --");
    let mut never = 0u64;
    let mut still_band = 0u64;
    for i in 0..N {
        for j in 0..N {
            if i == j {
                continue;
            }
            let (d5, d95) = crossed[i * N + j];
            if d5 == 0 {
                never += 1;
            } else if d95 == 0 {
                still_band += 1;
            }
        }
    }
    let crossed5 = (N * (N - 1)) as u64 - never;
    println!(
        "  crossed 0.5: {crossed5}/{} rows; of those, still under 0.95 by day {DAYS}: {still_band}",
        N * (N - 1)
    );

    println!("\n-- per-seed end-state splits --");
    splits(&sim, "  42 @20K", None);
    for &seed in &[7u64, 11] {
        let config = SimConfig {
            seed,
            max_ticks: 20_000,
            world_width: 16,
            world_height: 16,
            num_agents: N as u32,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        for t in 1..=20_000u64 {
            sim.tick();
            if t == 1440 || t == 10_080 {
                splits(&sim, &format!("  {seed} @t={t}"), None);
            }
        }
        splits(&sim, &format!("  {seed} @20K"), None);
    }
}
