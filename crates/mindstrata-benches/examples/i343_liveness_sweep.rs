//! i343 — are the panic and revolution producers dead, or re-paced?
//!
//! Wiring the contacted-degree channels into `social_visibility` and the
//! normal-life anxiety term moved two liveness anchors:
//!
//!   * `collective_fear_amplifies_panic_legitimacy_damage_end_to_end`
//!     — "the §7.2 trigger must fire in the crisis window (pestilence seed 5 @20K)"
//!   * `revolution_is_regime_change_not_repeat_loop`
//!     — "a revolution must fire in the 70K horizon"
//!
//! Both are *producer-liveness* contracts, and AGENTS §2.3 forbids re-pinning them
//! to accept zero: if the producer went dead the fix is to revive it, not to move
//! the bar. So the first question is whether each producer is alive across a seed
//! family (in which case the single-seed anchors are lucky-seed pins to be
//! re-contracted per §4.1/§4.4) or dead everywhere.
//!
//! Run: cargo run --release -p mindstrata-benches --example i343_liveness_sweep
//! (≈3–6 min wall: 6 seeds × 20K pestilence + 4 seeds × 70K revolution.)

use mindstrata_core::event::SimEvent;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

fn run(scenario: &Scenario, seed: u64, ticks: u64) -> Simulation {
    let mut sc = scenario.clone();
    sc.seed = seed;
    sc.ticks = ticks;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    sim.run(ticks);
    sim
}

fn is_panic(e: &SimEvent) -> bool {
    format!("{e:?}").contains("Panic")
}

fn is_revolution(e: &SimEvent) -> bool {
    matches!(
        e,
        SimEvent::ConflictOccurred {
            kind: mindstrata_core::conflict::ConflictKind::Revolution,
            ..
        }
    )
}

fn main() {
    println!("i343 — liveness sweep, post-rewire (32x32 charter world)\n");

    println!("=== panic liveness · pestilence @ 20K (the anchor is seed 5) ===");
    let mut panic_seeds_firing = 0;
    for seed in [1u64, 3, 5, 7, 11, 42] {
        let sim = run(&Scenario::pestilence(), seed, 20_000);
        let panics = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|e| is_panic(e))
            .count();
        if panics >= 1 {
            panic_seeds_firing += 1;
        }
        println!("  seed {seed:>3}: {panics:>3} panics");
    }
    println!("  → {panic_seeds_firing}/6 seeds fire");

    // The revolution leg is pestilence @70K with meme mutation isolated (§7.3
    // from §13.2), exactly as the test configures it — and its anchor has been
    // re-seeded six times in the ledger (42 → 7 → 1 → 12345), the flip-flop
    // pattern §4.5 names as debt. A family sweep is what decides whether to
    // re-contract it onto a family instead of a seventh single seed.
    println!("\n=== revolution liveness · pestilence @ 70K (anchor: seed 12345) ===");
    let mut rev_seeds_firing = 0;
    for seed in [12345u64, 1, 5, 7, 11, 42] {
        let mut sc = Scenario::pestilence();
        sc.seed = seed;
        sc.ticks = 70_000;
        let mut sim = Simulation::from_scenario(sc);
        sim.params.meme_mutation_rate_base = mindstrata_core::fixed::Fixed::ZERO;
        sim.populate();
        sim.run(70_000);
        let revs = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|e| is_revolution(e))
            .count();
        if revs > 0 {
            rev_seeds_firing += 1;
        }
        println!("  seed {seed:>5}: {revs:>3} revolutions");
    }
    println!("  → {rev_seeds_firing}/6 seeds fire");

    println!(
        "\nreading: a producer firing on SOME seeds is re-paced/knife-edge, not dead — the\n\
         single-seed anchor is then a lucky-seed pin (§4.1) to be re-contracted onto the seed\n\
         family with the mechanism guarded. A producer firing on NONE is a true regression and\n\
         must be revived (§2.3), not re-pinned."
    );
}
