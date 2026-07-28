//! §39.3 Golden Replay Tests — verify deterministic replay against stored baselines.
//!
//! These tests run a scenario for N ticks, compute metric hashes, and compare
//! against golden baselines stored in `golden/`. If a baseline changes, the test
//! fails and the developer must inspect whether the change was intended.

use mindstrata_sim::{Simulation, sim::SimConfig};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// Compute a deterministic hash of metric values.
fn hash_metrics(metrics: &[f64]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for m in metrics {
        m.to_bits().hash(&mut hasher);
    }
    hasher.finish()
}

/// Run a simulation and return golden baseline data.
struct GoldenBaseline {
    metric_hash: u64,
    event_hash: u64,
    agent_hash: u64,
    agent_count: u64,
    total_grain: f64,
    total_water: f64,
}

fn run_golden(seed: u64, ticks: u64) -> GoldenBaseline {
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

    let ms = sim.metrics_snapshot();
    let metrics = vec![
        ms.avg_hunger,
        ms.avg_thirst,
        ms.avg_fatigue,
        ms.avg_valence,
        ms.avg_joy,
        ms.avg_fear,
        ms.total_grain,
        ms.total_water,
        ms.event_count as f64,
        ms.journal_len as f64,
        ms.agent_count as f64,
    ];

    let summaries = sim.agent_summaries();
    let agent_data: Vec<(f64, f64, f64, f64)> = summaries
        .iter()
        .map(|s| (s.hunger.to_f64(), s.thirst.to_f64(), s.fatigue.to_f64(), s.valence.to_f64()))
        .collect();

    let event_count_hash = {
        let mut h = DefaultHasher::new();
        ms.event_count.hash(&mut h);
        h.finish()
    };

    GoldenBaseline {
        metric_hash: hash_metrics(&metrics),
        event_hash: event_count_hash,
        agent_hash: hash_metrics(&agent_data.iter().flat_map(|(a, b, c, d)| [*a, *b, *c, *d]).collect::<Vec<_>>()),
        agent_count: ms.agent_count,
        total_grain: ms.total_grain,
        total_water: ms.total_water,
    }
}

/// Golden baseline file path for a given scenario and seed.
fn golden_path(scenario: &str, seed: u64) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../golden")
        .join(scenario)
        .join(format!("seed_{}", seed))
        .join("baseline.json")
}

/// Baseline data stored on disk.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct StoredBaseline {
    scenario: String,
    seed: u64,
    ticks: u64,
    metric_hash: u64,
    event_hash: u64,
    agent_hash: u64,
    agent_count: u64,
    total_grain: f64,
    total_water: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SEED: u64 = 42;
    const TEST_TICKS: u64 = 1000;

    #[test]
    fn golden_replay_deterministic() {
        let b1 = run_golden(TEST_SEED, TEST_TICKS);
        let b2 = run_golden(TEST_SEED, TEST_TICKS);
        assert_eq!(b1.metric_hash, b2.metric_hash, "Metric hash not deterministic");
        assert_eq!(b1.event_hash, b2.event_hash, "Event hash not deterministic");
        assert_eq!(b1.agent_hash, b2.agent_hash, "Agent hash not deterministic");
        assert_eq!(b1.agent_count, b2.agent_count, "Agent count not deterministic");
    }

    #[test]
    fn golden_replay_vs_baseline() {
        let path = golden_path("riverford_minor", TEST_SEED);
        if !path.exists() {
            let b = run_golden(TEST_SEED, TEST_TICKS);
            let stored = StoredBaseline {
                scenario: "riverford_minor".into(),
                seed: TEST_SEED,
                ticks: TEST_TICKS,
                metric_hash: b.metric_hash,
                event_hash: b.event_hash,
                agent_hash: b.agent_hash,
                agent_count: b.agent_count,
                total_grain: b.total_grain,
                total_water: b.total_water,
            };
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let json = serde_json::to_string_pretty(&stored).unwrap();
            std::fs::write(&path, &json).unwrap();
            eprintln!("Golden baseline generated at {:?}", path);
            eprintln!("  metric_hash: {:016x}", b.metric_hash);
            return;
        }

        let json = std::fs::read_to_string(&path).unwrap();
        let baseline: StoredBaseline = serde_json::from_str(&json).unwrap();
        let b = run_golden(baseline.seed, baseline.ticks);

        assert_eq!(b.metric_hash, baseline.metric_hash,
            "\n⚠️  Golden baseline changed!\n  Scenario: {}\n  Seed: {}\n  Old: {:016x}\n  New: {:016x}\n  Update: {:?}",
            baseline.scenario, baseline.seed, baseline.metric_hash, b.metric_hash, path);
        assert_eq!(b.agent_count, baseline.agent_count,
            "Agent count changed! Old: {}, New: {}", baseline.agent_count, b.agent_count);
    }

    #[test]
    fn golden_replay_different_seeds_differ() {
        let b1 = run_golden(42, 500);
        let b2 = run_golden(99, 500);
        assert_ne!(b1.metric_hash, b2.metric_hash, "Different seeds should produce different metric hashes");
    }

    #[test]
    fn golden_replay_agent_count_stable() {
        let b = run_golden(TEST_SEED, TEST_TICKS);
        assert!(b.agent_count > 0, "Simulation should have agents");
        assert!(b.agent_count <= 12, "Should not have more agents than initialized");
    }
}
