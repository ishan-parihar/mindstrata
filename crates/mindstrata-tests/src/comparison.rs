//! §19.5.J Scenario Comparison & Replay Diffing — observability tools for debugging emergence.
//!
//! - **Scenario Comparison**: Run two configurations and compare metric trajectories.
//! - **Replay Diffing**: Compare two golden baselines and report divergences.

use mindstrata_sim::{Simulation, sim::SimConfig};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A metrics snapshot at a specific tick.
#[derive(Debug, Clone)]
pub struct MetricsPoint {
    pub tick: u64,
    pub avg_hunger: f64,
    pub avg_thirst: f64,
    pub avg_fatigue: f64,
    pub avg_valence: f64,
    pub avg_joy: f64,
    pub avg_fear: f64,
    pub total_grain: f64,
    pub total_water: f64,
    pub agent_count: u64,
    pub event_count: u64,
}

/// Run a simulation and collect metric snapshots at regular intervals.
fn collect_trajectory(config: &SimConfig, sample_interval: u64) -> Vec<MetricsPoint> {
    let mut sim = Simulation::new(config.clone());
    sim.populate();
    let max_ticks = config.max_ticks;
    let mut points = Vec::new();
    for tick in 1..=max_ticks {
        sim.tick();
        if tick % sample_interval == 0 || tick == max_ticks {
            let ms = sim.metrics_snapshot();
            points.push(MetricsPoint {
                tick,
                avg_hunger: ms.avg_hunger,
                avg_thirst: ms.avg_thirst,
                avg_fatigue: ms.avg_fatigue,
                avg_valence: ms.avg_valence,
                avg_joy: ms.avg_joy,
                avg_fear: ms.avg_fear,
                total_grain: ms.total_grain,
                total_water: ms.total_water,
                agent_count: ms.agent_count,
                event_count: ms.event_count,
            });
        }
    }
    points
}

/// A single divergence between two metric trajectories.
#[derive(Debug, Clone)]
pub struct MetricDivergence {
    pub tick: u64,
    pub metric: String,
    pub value_a: f64,
    pub value_b: f64,
    pub abs_diff: f64,
}

/// Result of comparing two simulation runs.
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    pub divergences: Vec<MetricDivergence>,
    pub max_divergence: f64,
    pub divergent_metrics: Vec<String>,
}

/// Compare two simulation configurations and report metric divergences.
///
/// Runs both configs for the same number of ticks and compares metric snapshots
/// at regular intervals. Reports any divergences above the threshold.
pub fn compare_scenarios(
    config_a: &SimConfig,
    config_b: &SimConfig,
    sample_interval: u64,
    threshold: f64,
) -> ComparisonResult {
    let traj_a = collect_trajectory(config_a, sample_interval);
    let traj_b = collect_trajectory(config_b, sample_interval);

    let mut divergences = Vec::new();
    let mut max_div = 0.0_f64;
    let mut divergent_metrics = std::collections::HashSet::new();

    for (pa, pb) in traj_a.iter().zip(traj_b.iter()) {
        debug_assert_eq!(pa.tick, pb.tick, "Trajectory points should be at same tick");

        let pairs = [
            ("avg_hunger", pa.avg_hunger, pb.avg_hunger),
            ("avg_thirst", pa.avg_thirst, pb.avg_thirst),
            ("avg_fatigue", pa.avg_fatigue, pb.avg_fatigue),
            ("avg_valence", pa.avg_valence, pb.avg_valence),
            ("avg_joy", pa.avg_joy, pb.avg_joy),
            ("avg_fear", pa.avg_fear, pb.avg_fear),
            ("total_grain", pa.total_grain, pb.total_grain),
            ("total_water", pa.total_water, pb.total_water),
            ("agent_count", pa.agent_count as f64, pb.agent_count as f64),
        ];

        for (name, va, vb) in &pairs {
            let diff = (va - vb).abs();
            if diff > threshold {
                divergences.push(MetricDivergence {
                    tick: pa.tick,
                    metric: name.to_string(),
                    value_a: *va,
                    value_b: *vb,
                    abs_diff: diff,
                });
                if diff > max_div {
                    max_div = diff;
                }
                divergent_metrics.insert(name.to_string());
            }
        }
    }

    ComparisonResult {
        divergences,
        max_divergence: max_div,
        divergent_metrics: divergent_metrics.into_iter().collect(),
    }
}

/// A baseline data point for replay diffing.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BaselinePoint {
    pub tick: u64,
    pub metric_hash: u64,
    pub agent_count: u64,
    pub total_grain: f64,
    pub total_water: f64,
    pub event_count: u64,
}

/// A single difference between two baselines.
#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub tick: u64,
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}

/// Result of diffing two golden baselines.
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub entries: Vec<DiffEntry>,
    pub identical: bool,
}

/// Run a simulation and produce a baseline trajectory for diffing.
pub fn produce_baseline_trajectory(config: &SimConfig, sample_interval: u64) -> Vec<BaselinePoint> {
    let mut sim = Simulation::new(config.clone());
    sim.populate();
    let max_ticks = config.max_ticks;
    let mut points = Vec::new();
    for tick in 1..=max_ticks {
        sim.tick();
        if tick % sample_interval == 0 || tick == max_ticks {
            let ms = sim.metrics_snapshot();
            let summaries = sim.agent_summaries();
            let agent_data: Vec<(f64, f64, f64, f64)> = summaries
                .iter()
                .map(|s| (s.hunger.to_f64(), s.thirst.to_f64(), s.fatigue.to_f64(), s.valence.to_f64()))
                .collect();
            let metrics = vec![
                ms.avg_hunger, ms.avg_thirst, ms.avg_fatigue, ms.avg_valence,
                ms.avg_joy, ms.avg_fear, ms.total_grain, ms.total_water,
                ms.event_count as f64, ms.journal_len as f64, ms.agent_count as f64,
            ];
            let metric_hash = {
                let mut hasher = DefaultHasher::new();
                for m in &metrics {
                    m.to_bits().hash(&mut hasher);
                }
                hasher.finish()
            };
            let agent_hash = {
                let mut hasher = DefaultHasher::new();
                for (a, b, c, d) in &agent_data {
                    a.to_bits().hash(&mut hasher);
                    b.to_bits().hash(&mut hasher);
                    c.to_bits().hash(&mut hasher);
                    d.to_bits().hash(&mut hasher);
                }
                hasher.finish()
            };
            // Combine metric and agent hashes for a single point hash
            let combined_hash = {
                let mut hasher = DefaultHasher::new();
                metric_hash.hash(&mut hasher);
                agent_hash.hash(&mut hasher);
                hasher.finish()
            };

            points.push(BaselinePoint {
                tick,
                metric_hash: combined_hash,
                agent_count: ms.agent_count,
                total_grain: ms.total_grain,
                total_water: ms.total_water,
                event_count: ms.event_count,
            });
        }
    }
    points
}

/// Diff two baseline trajectories and report divergences.
pub fn diff_baselines(old: &[BaselinePoint], new: &[BaselinePoint]) -> DiffResult {
    let mut entries = Vec::new();

    for (po, pn) in old.iter().zip(new.iter()) {
        debug_assert_eq!(po.tick, pn.tick, "Baseline points should be at same tick");

        if po.metric_hash != pn.metric_hash {
            entries.push(DiffEntry {
                tick: po.tick,
                field: "metric_hash".into(),
                old_value: format!("{:016x}", po.metric_hash),
                new_value: format!("{:016x}", pn.metric_hash),
            });
        }
        if po.agent_count != pn.agent_count {
            entries.push(DiffEntry {
                tick: po.tick,
                field: "agent_count".into(),
                old_value: po.agent_count.to_string(),
                new_value: pn.agent_count.to_string(),
            });
        }
        if (po.total_grain - pn.total_grain).abs() > 0.01 {
            entries.push(DiffEntry {
                tick: po.tick,
                field: "total_grain".into(),
                old_value: format!("{:.4}", po.total_grain),
                new_value: format!("{:.4}", pn.total_grain),
            });
        }
        if (po.total_water - pn.total_water).abs() > 0.01 {
            entries.push(DiffEntry {
                tick: po.tick,
                field: "total_water".into(),
                old_value: format!("{:.4}", po.total_water),
                new_value: format!("{:.4}", pn.total_water),
            });
        }
        if po.event_count != pn.event_count {
            entries.push(DiffEntry {
                tick: po.tick,
                field: "event_count".into(),
                old_value: po.event_count.to_string(),
                new_value: pn.event_count.to_string(),
            });
        }
    }

    DiffResult {
        identical: entries.is_empty(),
        entries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(seed: u64, ticks: u64) -> SimConfig {
        SimConfig {
            seed,
            max_ticks: ticks,
            world_width: 16,
            world_height: 16,
            num_agents: 8,
            snapshot_interval: None,
        }
    }

    #[test]
    fn same_config_no_divergence() {
        let config = test_config(42, 200);
        let result = compare_scenarios(&config, &config, 50, 0.001);
        assert!(result.divergences.is_empty(),
            "Same config should have no divergences, found: {:?}", result.divergences);
        assert_eq!(result.max_divergence, 0.0);
    }

    #[test]
    fn different_seeds_diverge() {
        let a = test_config(42, 200);
        let b = test_config(99, 200);
        let result = compare_scenarios(&a, &b, 50, 0.01);
        // Different seeds should produce different trajectories
        assert!(!result.divergent_metrics.is_empty() || result.max_divergence > 0.01,
            "Different seeds should diverge on at least some metrics");
    }

    #[test]
    fn baseline_determinism() {
        let config = test_config(42, 200);
        let t1 = produce_baseline_trajectory(&config, 50);
        let t2 = produce_baseline_trajectory(&config, 50);
        assert_eq!(t1.len(), t2.len());
        for (p1, p2) in t1.iter().zip(t2.iter()) {
            assert_eq!(p1.metric_hash, p2.metric_hash,
                "Same config should produce identical baseline at tick {}", p1.tick);
            assert_eq!(p1.agent_count, p2.agent_count);
        }
    }

    #[test]
    fn diff_identical_baselines() {
        let config = test_config(42, 200);
        let t1 = produce_baseline_trajectory(&config, 50);
        let t2 = produce_baseline_trajectory(&config, 50);
        let diff = diff_baselines(&t1, &t2);
        assert!(diff.identical, "Identical baselines should have no diffs");
        assert!(diff.entries.is_empty());
    }

    #[test]
    fn diff_different_baselines() {
        let t1 = produce_baseline_trajectory(&test_config(42, 200), 50);
        let t2 = produce_baseline_trajectory(&test_config(99, 200), 50);
        let diff = diff_baselines(&t1, &t2);
        // At least some ticks should differ
        assert!(!diff.identical, "Different seeds should produce different baselines");
    }

    #[test]
    fn divergence_reports_metric_names() {
        let a = test_config(42, 200);
        let b = test_config(99, 200);
        let result = compare_scenarios(&a, &b, 50, 0.001);
        // Each divergence should have a valid metric name
        for d in &result.divergences {
            assert!(!d.metric.is_empty(), "Divergence metric name should not be empty");
            assert!(d.abs_diff > 0.0, "Absolute difference should be positive");
        }
    }
}
