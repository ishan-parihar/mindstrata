//! §39.4 Statistical Emergence Tests — verify tendencies across many seeds.
//!
//! These tests run many seeds and check statistical properties rather than
//! exact outcomes. This makes them robust to minor behavioral changes while
//! still catching fundamental regressions.

#[cfg(test)]
mod tests {
    use mindstrata_sim::{sim::SimConfig, Simulation};

    fn run_sim(seed: u64, ticks: u64) -> mindstrata_sim::sim::MetricsSnapshot {
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
        sim.metrics_snapshot()
    }

    struct Stats {
        avg_hunger: Vec<f64>,
        avg_thirst: Vec<f64>,
        total_grain: Vec<f64>,
        total_water: Vec<f64>,
        event_count: Vec<u64>,
        agent_count: Vec<u64>,
    }

    fn collect_stats(seeds: u64, ticks: u64) -> Stats {
        let mut stats = Stats {
            avg_hunger: Vec::new(),
            avg_thirst: Vec::new(),
            total_grain: Vec::new(),
            total_water: Vec::new(),
            event_count: Vec::new(),
            agent_count: Vec::new(),
        };
        for seed in 0..seeds {
            let ms = run_sim(seed, ticks);
            stats.avg_hunger.push(ms.avg_hunger);
            stats.avg_thirst.push(ms.avg_thirst);
            stats.total_grain.push(ms.total_grain);
            stats.total_water.push(ms.total_water);
            stats.event_count.push(ms.event_count);
            stats.agent_count.push(ms.agent_count);
        }
        stats
    }

    fn mean(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f64>() / values.len() as f64
    }

    fn mean_u64(values: &[u64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().map(|v| *v as f64).sum::<f64>() / values.len() as f64
    }

    const NUM_SEEDS: u64 = 20;
    const TICKS: u64 = 1000;

    #[test]
    fn survival_rate_above_threshold() {
        // §13.6: Agents should survive — average survival rate > 80%
        let stats = collect_stats(NUM_SEEDS, TICKS);
        let avg_survival = mean_u64(&stats.agent_count);
        let survival_rate = avg_survival / 12.0;
        assert!(
            survival_rate > 0.8,
            "Average survival rate should be > 80%, got {:.1}%",
            survival_rate * 100.0
        );
    }

    #[test]
    fn needs_managed_over_time() {
        // §9.1: Average hunger < 0.7 after 1000 ticks
        let stats = collect_stats(NUM_SEEDS, TICKS);
        let avg_hunger = mean(&stats.avg_hunger);
        let avg_thirst = mean(&stats.avg_thirst);
        assert!(
            avg_hunger < 0.7,
            "Average hunger should be < 0.7 after {TICKS} ticks, got {avg_hunger:.3}"
        );
        assert!(
            avg_thirst < 0.7,
            "Average thirst should be < 0.7 after {TICKS} ticks, got {avg_thirst:.3}"
        );
    }

    #[test]
    fn events_accumulate_proportionally() {
        // §8: Events should accumulate roughly proportionally to ticks
        let stats = collect_stats(NUM_SEEDS, TICKS);
        let avg_events = mean_u64(&stats.event_count);
        let events_per_tick = avg_events / TICKS as f64;
        assert!(
            events_per_tick > 0.05,
            "Events per tick should be > 0.05, got {events_per_tick:.4}"
        );
    }

    #[test]
    fn resources_never_deplete_completely() {
        // §28: Resources should never all deplete
        let stats = collect_stats(NUM_SEEDS, TICKS);
        let avg_grain = mean(&stats.total_grain);
        let avg_water = mean(&stats.total_water);
        assert!(
            avg_grain > 0.0,
            "Average grain should be > 0, got {avg_grain:.3}"
        );
        assert!(
            avg_water > 0.0,
            "Average water should be > 0, got {avg_water:.3}"
        );
    }

    #[test]
    fn social_interactions_occur() {
        // §11: Social interactions should occur
        let stats = collect_stats(NUM_SEEDS, TICKS);
        let avg_events = mean_u64(&stats.event_count);
        assert!(
            avg_events > 12.0,
            "Average event count should exceed agent count, got {avg_events:.0}"
        );
    }

    #[test]
    fn emergent_behavior_varies_by_seed() {
        // Different seeds should produce meaningfully different outcomes
        let stats = collect_stats(10, TICKS);
        let min_h = stats
            .avg_hunger
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        let max_h = stats
            .avg_hunger
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        // §9.1: Goal deduplication and routine bias reduce cross-seed variance.
        // Allow a lower threshold — the important thing is non-zero variance.
        assert!(
            max_h - min_h > 0.005,
            "Hunger should vary across seeds: range={:.4}",
            max_h - min_h
        );
    }

    // ── §18.4 Statistical emergence audits (Iteration 137) ────────────────

    /// §18.4 (Iteration 137): The D4 courtship gate (`total_attraction > 0.4`)
    /// must be structurally reachable in at least some worlds. In the calm
    /// seed-42 village at 10K, the max total_attraction is ~0.333 (below the
    /// gate) — but across seeds and agents, the gate should open somewhere.
    ///
    /// Runtime note: 10 seeds × 5000 ticks ≈ 33s (documented gate increase,
    /// matching Iter-135's 72s transparency). A reachability proof needs only
    /// a representative sample, not exhaustive coverage.
    #[test]
    fn courtship_d4_reachable_across_worlds() {
        let mut seeds_with_max_above_gate = 0u32;
        let mut global_max = 0.0f64;
        for seed in 0..10u64 {
            let config = SimConfig {
                seed,
                max_ticks: 5000,
                world_width: 16,
                world_height: 16,
                num_agents: 12,
                snapshot_interval: None,
            };
            let mut sim = Simulation::new(config);
            sim.populate();
            sim.run(5000);
            let seed_max = sim
                .agents
                .iter()
                .map(|a| a.attraction.total_attraction().to_f64())
                .fold(f64::NEG_INFINITY, f64::max);
            if seed_max > 0.4 {
                seeds_with_max_above_gate += 1;
            }
            global_max = global_max.max(seed_max);
        }
        assert!(
            seeds_with_max_above_gate > 0,
            "D4 gate should be reachable in at least one seed \
             (global max total_attraction={global_max:.4})"
        );
        assert!(
            global_max > 0.4,
            "global max total_attraction should exceed 0.4, got {global_max:.4}"
        );
    }

    /// §18.4 (Iteration 137): Faction formation must vary across seeds — if
    /// every seed produces the same number of factions, the formation system
    /// is degenerate (non-deterministic emergence is the expected contract).
    ///
    /// Iteration 186: re-anchored from the base world to the grievance-crisis
    /// scenario (pestilence) — the calm-world coup clock is closed and the
    /// formation system's cross-seed variance is exercised where the trigger
    /// arms.
    /// Iteration 240 re-anchor (crisis-pressure accumulator): formation no
    /// longer requires the single-tick legitimacy cliff (which produced {0}
    /// across ALL seeds after Iterations 228–236 re-paced legitimacy), so
    /// the horizon moves to the accumulator's arming scale — sustained
    /// epidemic discontent crosses the 0.5 pressure threshold at ~5–13K
    /// (probe: pestilence-5 arms ~8K, calm-42 ~13K), so 12K gives
    /// high-grievance seeds formations while stable ones stay clean — real
    /// cross-seed variance instead of a shared zero.
    ///
    /// Runtime note: 20 seeds × 12K pestilence ticks ≈ 3–4 min release-mode
    /// (the epidemic machinery is heavier; acceptable for the gate).
    #[test]
    fn faction_counts_vary_across_worlds() {
        use mindstrata_sim::institutions::InstitutionKind;
        use mindstrata_sim::scenario::Scenario;
        let mut counts = std::collections::BTreeSet::new();
        for seed in 0..20u64 {
            let mut sc = Scenario::pestilence();
            sc.seed = seed;
            sc.ticks = 12_000;
            let mut sim = Simulation::from_scenario(sc);
            sim.populate();
            sim.run(12_000);
            let faction_count = sim
                .faction_v2_registry
                .factions
                .iter()
                .filter(|f| f.active)
                .count()
                + sim
                    .institutions
                    .iter()
                    .filter(|i| i.kind == InstitutionKind::Faction)
                    .count();
            counts.insert(faction_count);
        }
        assert!(
            counts.len() > 1,
            "faction counts should vary across seeds (got {counts:?})"
        );
        assert!(
            counts.iter().any(|&c| c > 0),
            "at least one seed should form factions (got {counts:?})"
        );
    }

    /// i392b: the exploration driver's contract, **re-contracted from a measured
    /// family** — the retired claim was "Wander share in the 0.5–3% of decisions
    /// band at 20K" (i351's acceptance reading), and it had quietly stopped being
    /// true: `ENGINE_STATUS.md` §5's own i384 table records Wander at **4.4% /
    /// 5.4%** while the code comment still asserted the band, and *nothing
    /// asserted either*. Seven probes interpolate the law; no test did.
    ///
    /// Why the magnitude is **not** what gets pinned (§4.10): a band tight enough
    /// to discriminate the share re-creates the knife-edge pin that made the old
    /// claim unenforceable. The census measures **3–76% of Wander's wins decided
    /// within the decision-noise amplitude** (`i392b_wander_band` leg D), so the
    /// magnitude is recorded — 1.63–9.20% across the family, mean ≈4.5% — while
    /// the *invariants* are asserted. Rationale for the re-contract over a
    /// re-calibration: the band's own stated purpose is a Work-displacement
    /// guard, and Work does not move — seed 42 at i384's exact config reads
    /// **33.88% vs the recorded 32.70%** (+1.18 pt) at N=12 and **29.34 vs 30.10**
    /// (−0.76 pt) at N=48. Nothing is being displaced, so the coefficient is
    /// still at a defensible operating point and only the claim was stale.
    ///
    /// The three pinned invariants, each measured on all six seeds:
    ///   1. **liveness** — the driver reaches the deliberative layer on every
    ///      seed (a driver that stops winning is the dead-producer class, §2.3);
    ///   2. **gate exclusivity** — ≥99% of wins lie inside the need-quietude
    ///      window, i.e. exploration never outbids provisioning (measured 100%,
    ///      with the 1-point slack absorbing the two counters' ±1 instrumentation
    ///      skew);
    ///   3. **boundedness** — no seed runs away. The guard is **12%**, sized from
    ///      the sweep rather than picked: the calibrated family maxes at **9.20%**
    ///      (coefficient 2.0) and the over-drive coefficient i351 flagged
    ///      (**3.0**) maxes at **13.30%** — 12% leaves the calibrated spread ~30%
    ///      of headroom *and* trips at the over-drive regime. Both halves of this
    ///      gate were proven to trip (coefficient 0 → the liveness assertion with
    ///      "seed 1 produced 0 Wander selections"; coefficient 5.0 → the bound at
    ///      15.31%), per the i170 subsystem-gate rule.
    ///
    /// The family spans the measured extremes (seed 44 min, seed 99 max) plus the
    /// calibrated seed — a bound asserted on a friendly subset is a lucky-seed
    /// pin (§4.1). One test case on purpose: the census sink is process-global
    /// (see `sim::tests::census`), and this crate has no other census user.
    #[test]
    fn exploration_driver_is_live_gated_and_bounded_across_the_seed_family() {
        use mindstrata_sim::sim::decision_census;
        const FAMILY: [u64; 6] = [1, 7, 11, 42, 44, 99];
        const TICKS: u64 = 10_000;
        /// `ACTION_NAMES` index of `Wander`.
        const WANDER: usize = 7;
        /// `SOURCE_NAMES` index of `utility` — the only deliberating source.
        const UTILITY: usize = 4;
        const MAX_SHARE_PCT: f64 = 12.0;

        let mut measured = Vec::new();
        for seed in FAMILY {
            let mut sim = Simulation::new(SimConfig {
                seed,
                max_ticks: TICKS,
                world_width: 32,
                world_height: 32,
                num_agents: 12,
                snapshot_interval: None,
            });
            sim.populate();
            decision_census::reset();
            decision_census::enable();
            sim.run(TICKS);
            decision_census::disable();
            let r = decision_census::report();

            let deliberated = r.cross[UTILITY][WANDER];
            assert!(
                deliberated > 0,
                "i392b: the exploration driver must reach the deliberative layer on \
                 every seed; seed {seed} produced 0 Wander selections in {TICKS} ticks"
            );

            // Gate exclusivity: a win outside the quiet window would mean
            // exploration outbidding provisioning, which the driver's design
            // forbids (it is gated on `max(hunger, thirst, fatigue) < 0.5`).
            assert!(
                r.quiet_wander.wins * 100 >= r.wander.wins * 99,
                "i392b: exploration must stay inside the quiet window; seed {seed} won \
                 {} arbitrations of which only {} were quiet-gated",
                r.wander.wins,
                r.quiet_wander.wins
            );

            let share = deliberate_share(r.total(), deliberated);
            assert!(
                share <= MAX_SHARE_PCT,
                "i392b: Wander must not run away; seed {seed} spent {share:.2}% of \
                 decisions wandering (guard {MAX_SHARE_PCT}%)"
            );
            measured.push(share);
        }
        // Recorded, not asserted: the family's spread is the measured band the
        // retired claim used to guess at (i392b leg D).
        let mean = measured.iter().sum::<f64>() / measured.len() as f64;
        assert!(
            (1.0..=12.0).contains(&mean),
            "i392b: the measured mean share should track the probe's ≈4.5% \
             (family {measured:?}, mean {mean:.2}%)"
        );
    }

    /// Share of recorded decisions that were `Wander`, in percent.
    fn deliberate_share(total: u64, wander_decisions: u64) -> f64 {
        wander_decisions as f64 / total.max(1) as f64 * 100.0
    }
}
