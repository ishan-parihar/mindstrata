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
    /// Runtime note: 20 seeds × 2000 ticks ≈ 26s (acceptable for the gate;
    /// ~1.3s per seed).
    #[test]
    fn faction_counts_vary_across_worlds() {
        use mindstrata_sim::institutions::InstitutionKind;
        let mut counts = std::collections::BTreeSet::new();
        for seed in 0..20u64 {
            let config = SimConfig {
                seed,
                max_ticks: 2000,
                world_width: 16,
                world_height: 16,
                num_agents: 12,
                snapshot_interval: None,
            };
            let mut sim = Simulation::new(config);
            sim.populate();
            sim.run(2000);
            let faction_count = sim
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
}
