//! §19.5.J Scenario Comparison & Replay Diffing — observability tools for debugging emergence.
//!
//! - **Scenario Comparison**: Run two configurations and compare metric trajectories.
//! - **Replay Diffing**: Compare two golden baselines and report divergences.

use mindstrata_sim::{sim::SimConfig, Simulation};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A metrics snapshot at a specific tick.
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct MetricDivergence {
    pub tick: u64,
    pub metric: String,
    pub value_a: f64,
    pub value_b: f64,
    pub abs_diff: f64,
}

/// Result of comparing two simulation runs.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ComparisonResult {
    pub divergences: Vec<MetricDivergence>,
    pub divergent_metrics: Vec<String>,
}

/// Compare two simulation configs and report metric divergences.
#[allow(dead_code)]
pub fn compare_configs(
    config_a: &SimConfig,
    config_b: &SimConfig,
    sample_interval: u64,
    threshold: f64,
) -> ComparisonResult {
    let points_a = collect_trajectory(config_a, sample_interval);
    let points_b = collect_trajectory(config_b, sample_interval);

    let mut divergences = Vec::new();
    let mut divergent_metrics = std::collections::HashSet::new();

    for (a, b) in points_a.iter().zip(points_b.iter()) {
        let metrics = [
            ("avg_hunger", a.avg_hunger, b.avg_hunger),
            ("avg_thirst", a.avg_thirst, b.avg_thirst),
            ("avg_fatigue", a.avg_fatigue, b.avg_fatigue),
            ("avg_valence", a.avg_valence, b.avg_valence),
            ("avg_joy", a.avg_joy, b.avg_joy),
            ("avg_fear", a.avg_fear, b.avg_fear),
            ("total_grain", a.total_grain, b.total_grain),
            ("total_water", a.total_water, b.total_water),
        ];

        for (name, va, vb) in &metrics {
            let diff = (va - vb).abs();
            if diff > threshold {
                divergences.push(MetricDivergence {
                    tick: a.tick,
                    metric: name.to_string(),
                    value_a: *va,
                    value_b: *vb,
                    abs_diff: diff,
                });
                divergent_metrics.insert(name.to_string());
            }
        }
    }

    ComparisonResult {
        divergences,
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
#[allow(dead_code)]
pub struct DiffEntry {
    pub tick: u64,
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}

/// Result of diffing two golden baselines.
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
            // Hash the full metrics state for deterministic comparison
            let mut hasher = DefaultHasher::new();
            ms.avg_hunger.to_bits().hash(&mut hasher);
            ms.avg_thirst.to_bits().hash(&mut hasher);
            ms.avg_fatigue.to_bits().hash(&mut hasher);
            ms.avg_valence.to_bits().hash(&mut hasher);
            ms.avg_joy.to_bits().hash(&mut hasher);
            ms.avg_fear.to_bits().hash(&mut hasher);
            ms.total_grain.to_bits().hash(&mut hasher);
            ms.total_water.to_bits().hash(&mut hasher);

            points.push(BaselinePoint {
                tick,
                metric_hash: hasher.finish(),
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
pub fn diff_baselines(baseline_a: &[BaselinePoint], baseline_b: &[BaselinePoint]) -> DiffResult {
    let mut entries = Vec::new();
    let mut identical = true;

    for (a, b) in baseline_a.iter().zip(baseline_b.iter()) {
        if a.metric_hash != b.metric_hash {
            identical = false;
            // Report which metrics diverged
            if (a.total_grain - b.total_grain).abs() > f64::EPSILON {
                entries.push(DiffEntry {
                    tick: a.tick,
                    field: "total_grain".to_string(),
                    old_value: format!("{:.3}", a.total_grain),
                    new_value: format!("{:.3}", b.total_grain),
                });
            }
            if (a.total_water - b.total_water).abs() > f64::EPSILON {
                entries.push(DiffEntry {
                    tick: a.tick,
                    field: "total_water".to_string(),
                    old_value: format!("{:.3}", a.total_water),
                    new_value: format!("{:.3}", b.total_water),
                });
            }
            if a.agent_count != b.agent_count {
                entries.push(DiffEntry {
                    tick: a.tick,
                    field: "agent_count".to_string(),
                    old_value: a.agent_count.to_string(),
                    new_value: b.agent_count.to_string(),
                });
            }
        }
    }

    DiffResult { entries, identical }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_configs_produce_identical_trajectories() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 50,
            world_width: 16,
            world_height: 16,
            num_agents: 6,
            snapshot_interval: None,
        };
        let points_a = collect_trajectory(&config, 10);
        let points_b = collect_trajectory(&config, 10);
        assert_eq!(points_a.len(), points_b.len());
        for (a, b) in points_a.iter().zip(points_b.iter()) {
            assert_eq!(a.avg_hunger, b.avg_hunger);
            assert_eq!(a.avg_thirst, b.avg_thirst);
            assert_eq!(a.total_grain, b.total_grain);
        }
    }

    #[test]
    fn different_seeds_produce_different_trajectories() {
        let config_a = SimConfig {
            seed: 42,
            max_ticks: 50,
            world_width: 16,
            world_height: 16,
            num_agents: 6,
            snapshot_interval: None,
        };
        let config_b = SimConfig {
            seed: 99,
            max_ticks: 50,
            world_width: 16,
            world_height: 16,
            num_agents: 6,
            snapshot_interval: None,
        };
        let points_a = collect_trajectory(&config_a, 10);
        let points_b = collect_trajectory(&config_b, 10);
        // Different seeds should produce at least some divergence
        let mut has_divergence = false;
        for (a, b) in points_a.iter().zip(points_b.iter()) {
            if (a.avg_hunger - b.avg_hunger).abs() > 0.01 {
                has_divergence = true;
                break;
            }
        }
        assert!(
            has_divergence,
            "Different seeds should produce different trajectories"
        );
    }

    #[test]
    fn baseline_roundtrip_is_deterministic() {
        let config = SimConfig {
            seed: 42,
            max_ticks: 30,
            world_width: 16,
            world_height: 16,
            num_agents: 6,
            snapshot_interval: None,
        };
        let baseline_a = produce_baseline_trajectory(&config, 10);
        let baseline_b = produce_baseline_trajectory(&config, 10);
        let diff = diff_baselines(&baseline_a, &baseline_b);
        assert!(
            diff.identical,
            "Same config should produce identical baselines"
        );
    }
}
