//! i392b — the exploration driver's acceptance band is stale. Re-measure it
//! properly, and decide re-contract vs re-calibrate from the data.
//!
//! **The finding that opened this.** i392 row 2 was rejected partly because the
//! *consumer's own calibration* turned out to be unenforced: `i351_wander_bands`
//! (i351's acceptance instrument, its own config) reads **5.44% / 3.53%** of
//! decisions at 20K on seeds 42 / 7 against the documented **0.5–3%** target,
//! and no test pins the claim — the instrument is a probe, so no gate ever
//! re-measured it. The action layer moved under it after i351 landed (i356 Idle
//! driver, i387 utility decomposition, i388 relational urgency family, i389
//! decree, i390 succession).
//!
//! **Why a family, not two seeds.** The claim is a *population* property, so two
//! seeds cannot distinguish "the band is stale" from "seed 42 is unlucky"
//! (§4.1). Leg A runs the repo's standard 12-seed family at the claimed N and
//! horizon.
//!
//! **The decision the probe has to inform.** A share above the target band is
//! only a defect if it is *displacing* something — the band's own reading says
//! ">8% would over-drive (Work displacement)". So the probe reports the **whole
//! action distribution** beside the share: if Work holds at its calibrated
//! ~32% the band was an uncalibrated guess and gets re-contracted to the
//! measured invariant (live + bounded + gated); if Work has been eaten, the
//! coefficient itself is uncalibrated and gets re-derived.
//!
//! Leg D closes the other half: the *shape* of the share with horizon, because
//! a statistic that drifts with the window cannot be a contract at all.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i392b_wander_band`

use mindstrata_sim::sim::decision_census;
use mindstrata_sim::sim::{SimConfig, Simulation};

/// The repo's standard 12-seed family (`escalation_trust`, `faction` and the
/// revolution pins all use it) — reused here so the band is measured on the
/// same population the rest of the suite treats as representative.
const SEEDS: [u64; 12] = [1, 2, 3, 5, 7, 11, 17, 21, 33, 42, 44, 99];

/// i351 measured its band at 20K after the driver landed; the charter's perf
/// harness uses 32×32 at the N=48 tier, so 12 agents in 32×32 is the claimed
/// configuration (note: 16×16 would be the density-law size for N=12 — the i351
/// claim predates that policy, and keeping the claimed config is the point).
const TICKS: u64 = 20_000;
const WORLD: u32 = 32;

/// Decision-census action order (`ACTION_NAMES`).
const ACTIONS: [&str; 10] = [
    "Eat",
    "Drink",
    "Rest",
    "Work",
    "Socialize",
    "Worship",
    "Trade",
    "Wander",
    "Move",
    "Idle",
];
const WANDER: usize = 7;

struct Row {
    seed: u64,
    share: f64,
    wins: u64,
    within_noise: u64,
    mean_loss: f64,
    quiet_share: f64,
    /// Every win happened inside the need-quietude window: the driver never
    /// outbids provisioning. Reported per seed because it is the invariant the
    /// re-contract is built on (the band never carried it).
    gate_exclusive: bool,
    actions: [f64; 10],
}

fn measure_with_warmup(seed: u64, n: u32, ticks: u64, warmup: u64) -> Row {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: warmup + ticks,
        world_width: WORLD,
        world_height: WORLD,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(warmup);
    decision_census::reset();
    decision_census::enable();
    sim.run(ticks);
    decision_census::disable();
    let r = decision_census::report();
    let total = r.total().max(1) as f64;
    let mut actions = [0.0f64; 10];
    for (i, slot) in actions.iter_mut().enumerate() {
        *slot = r.actions[i] as f64 / total * 100.0;
    }
    Row {
        seed,
        share: r.actions[WANDER] as f64 / total * 100.0,
        wins: r.wander.wins,
        within_noise: r.wander.within_noise,
        mean_loss: r.wander.mean_loss(r.utility_samples),
        quiet_share: r.quiet_wander.wins as f64 / r.quiet_samples.max(1) as f64 * 100.0,
        gate_exclusive: r.wander.wins == r.quiet_wander.wins,
        actions,
    }
}

fn measure(seed: u64, n: u32, ticks: u64) -> Row {
    measure_with_warmup(seed, n, ticks, 0)
}

fn stats(values: &[f64]) -> (f64, f64, f64) {
    let n = values.len().max(1) as f64;
    let mean = values.iter().sum::<f64>() / n;
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    (min, mean, max)
}

/// Leg A — the family at the claimed N and horizon.
fn leg_a() {
    println!("══ A — 12-seed family, N=12, {WORLD}×{WORLD}, {TICKS} ticks ══");
    println!("seed        share%   wins  ≤noise  mean-loss  quiet-win%  gated");
    let mut shares = Vec::new();
    let mut all_wins = true;
    let mut all_gated = true;
    let mut noise_fracs = Vec::new();
    for seed in SEEDS {
        let r = measure(seed, 12, TICKS);
        let noise = r.within_noise as f64 / r.wins.max(1) as f64 * 100.0;
        println!(
            "{:>5}      {:>6.2}  {:>5}  {:>6}  {:>9.4}  {:>10.2}  {}",
            r.seed,
            r.share,
            r.wins,
            r.within_noise,
            r.mean_loss,
            r.quiet_share,
            if r.gate_exclusive { "yes" } else { "NO" }
        );
        if r.wins == 0 {
            all_wins = false;
        }
        if !r.gate_exclusive {
            all_gated = false;
        }
        noise_fracs.push(noise);
        shares.push(r.share);
    }
    let (min, mean, max) = stats(&shares);
    let (nmin, nmean, nmax) = stats(&noise_fracs);
    println!(
        "→ share: min {min:.2}% · mean {mean:.2}% · max {max:.2}%   (documented target band 0.5–3%, over-drive guard 8%)"
    );
    println!("→ liveness: driver wins on every seed? {all_wins}");
    println!("→ gate exclusivity: every win inside the quiet window? {all_gated}");
    println!(
        "→ knife-edge signature — share of wins decided within the noise amplitude: min {nmin:.0}% · mean {nmean:.0}% · max {nmax:.0}%"
    );
    println!();
}

/// Leg B — the displacement check, which is what makes a band breach a defect
/// or not: the full action distribution at N=12 and N=48.
const CALIBRATED_WORK: f64 = 32.7;

/// `ENGINE_STATUS.md` §5's recorded distribution (i384, seed 42, 500-tick
/// warm-up, 20K, 32×32) — the only like-for-like comparator there is, because
/// it is the same seed, same config and same instrument. A family mean would
/// confound seed spread with displacement, so both are printed: the family for
/// the *shape*, this leg for the *delta*.
const RECORDED_N12: [f64; 10] = [10.4, 6.6, 17.9, 32.7, 4.9, 1.1, 20.5, 4.4, 0.39, 1.2];
const RECORDED_N48: [f64; 10] = [10.6, 6.4, 16.9, 30.1, 5.1, 0.3, 24.3, 5.4, 0.45, 0.4];

fn leg_b() {
    println!("══ B — displacement check ══");
    println!("— family mean (shape) —");
    for n in [12u32, 48] {
        let seeds: &[u64] = if n == 12 { &SEEDS } else { &SEEDS[..6] };
        let mut acc = [0.0f64; 10];
        for seed in seeds {
            let r = measure(*seed, n, TICKS);
            for (a, v) in acc.iter_mut().zip(r.actions.iter()) {
                *a += v / seeds.len() as f64;
            }
        }
        print!("  N={n:<3}");
        for (i, name) in ACTIONS.iter().enumerate() {
            print!("  {name} {:.1}", acc[i]);
        }
        println!();
    }
    println!("— like-for-like: seed 42, 500-tick warm-up, 20K (i384's exact config) —");
    println!("  action      i384 recorded   now     Δ");
    for n in [12u32, 48] {
        let recorded = if n == 12 { RECORDED_N12 } else { RECORDED_N48 };
        let r = measure_with_warmup(42, n, TICKS, 500);
        println!("  N={n}");
        for (i, name) in ACTIONS.iter().enumerate() {
            println!(
                "  {name:<10}  {:>10.2}   {:>6.2}  {:>+7.2}",
                recorded[i],
                r.actions[i],
                r.actions[i] - recorded[i]
            );
        }
    }
    println!("  (Work's calibrated N=12 value is {CALIBRATED_WORK}%)");
    println!();
}

/// Leg C — is the *statistic* stable enough to be a contract? Two seeds whose
/// measured shares sit at opposite ends of the family, three horizons.
fn leg_c() {
    println!("══ C — horizon sensitivity (N=12, {WORLD}×{WORLD}) ══");
    println!("seed   @2K     @10K    @20K");
    for seed in [7u64, 42] {
        let mut row = format!("{seed:>4} ");
        for ticks in [2_000u64, 10_000, TICKS] {
            let r = measure(seed, 12, ticks);
            row.push_str(&format!("  {:>5.2}%", r.share));
        }
        println!("{row}");
    }
    println!();
}

/// Leg D — **the test's exact configuration.** The contract that lands replaces
/// the band, so the numbers the pin asserts must be probe-evidenced at the pin's
/// own N, horizon and seeds (§4.2) — not extrapolated from leg A's 20K run.
///
/// The family spans the measured extremes (seed 44 min, seed 99 max) plus the
/// calibrated seed, because a bound asserted on a friendly subset is a lucky-seed
/// pin (§4.1).
fn leg_d() {
    const FAMILY: [u64; 6] = [1, 7, 11, 42, 44, 99];
    const TEST_TICKS: u64 = 10_000;
    println!("══ D — the pin's own configuration (N=12, {WORLD}×{WORLD}, {TEST_TICKS} ticks) ══");
    println!("seed    decisions  wander-wins  share%   quiet-wins  exclusive  ≤noise/wins");
    let mut max_share: f64 = 0.0;
    let mut all_live = true;
    let mut all_exclusive = true;
    for seed in FAMILY {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: TEST_TICKS,
            world_width: WORLD,
            world_height: WORLD,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.populate();
        decision_census::reset();
        decision_census::enable();
        sim.run(TEST_TICKS);
        decision_census::disable();
        let r = decision_census::report();
        let total = r.total().max(1) as f64;
        let share = r.cross[4][WANDER] as f64 / total * 100.0;
        let exclusive = r.wander.wins == r.quiet_wander.wins;
        max_share = max_share.max(share);
        all_live &= r.cross[4][WANDER] > 0;
        all_exclusive &= exclusive;
        println!(
            "{:>4}    {:>9}  {:>11}  {:>6.2}  {:>10}  {:<9}  {:.0}%",
            seed,
            r.total(),
            r.cross[4][WANDER],
            share,
            r.quiet_wander.wins,
            if exclusive { "yes" } else { "NO" },
            r.wander.within_noise as f64 / r.wander.wins.max(1) as f64 * 100.0
        );
    }
    println!(
        "→ max share {max_share:.2}% (guard 15%) · liveness {all_live} · exclusivity {all_exclusive}"
    );
    println!();
}

fn main() {
    println!("i392b — the exploration driver's band, re-measured on a family\n");
    leg_a();
    leg_b();
    leg_c();
    leg_d();
}
