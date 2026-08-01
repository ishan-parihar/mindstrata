//! Snapshot system — §16.1 of the architecture spec.
//!
//! Full world state at a tick for deterministic replay.
//! Uses postcard for compact binary serialization.
//!
//! RNG streams are NOT serialized — the master seed is stored and streams
//! are re-created on restore (per rng.rs design).
//!
//! ```text
//! /saves/world_seed_123_tick_10000.snapshot
//! ```

use crate::black_market::BlackMarketState;
use crate::culture::Knowledge;
use crate::demography::DemographyConfig;
use crate::ecology::{EcologyConfig, SeasonTracker};
use crate::health::{ActiveDisease, HealthConfig};
use crate::institutions::Institution;
use crate::journal::EventJournal;
use crate::market::MarketState;
use crate::norms::NormRegistry;
use crate::person::Relationship;
use crate::provenance::CausalProvenance;
use crate::sim::{AgentBundle, MetricsSnapshot, SimConfig};
use crate::world::World;
use mindstrata_core::clock::Tick;
use serde::{Deserialize, Serialize};

/// A deterministic snapshot of the entire simulation state at a specific tick.
/// §16.1: "Full world state at a tick."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Version of the snapshot format (for future compatibility).
    pub version: u32,
    /// The simulation config used to create this state.
    pub config: SimConfig,
    /// Current tick number.
    pub tick: u64,
    /// Current clock state (for exact restoration).
    pub clock_tick: Tick,
    /// Master RNG seed — streams are re-created from this on restore.
    pub master_seed: u64,
    /// The world state.
    pub world: World,
    /// All agent bundles.
    pub agents: Vec<AgentBundle>,
    /// All relationships.
    pub relationships: Vec<Relationship>,
    /// Events generated so far.
    pub events: Vec<mindstrata_core::event::SimEvent>,
    /// The event journal.
    pub journal: EventJournal,
    /// The norm registry.
    pub norms: NormRegistry,
    /// Institutions.
    pub institutions: Vec<Institution>,
    /// Causal provenance store.
    pub provenance: CausalProvenance,
    /// §28: Season tracker.
    pub season: SeasonTracker,
    /// §28: Ecology config.
    pub ecology_config: EcologyConfig,
    /// §9: Health config.
    pub health_config: HealthConfig,
    /// §31: Demography config.
    pub demography_config: DemographyConfig,
    /// §32: Active diseases per agent.
    pub agent_diseases: Vec<Vec<ActiveDisease>>,
    /// §13.3: Market state.
    pub market: MarketState,
    /// §19.5.I: Cultural knowledge store.
    pub knowledge_store: Vec<Knowledge>,
    /// §6.5: Metric history for observability.
    pub metric_history: Vec<MetricsSnapshot>,
    /// §7.3: Last revolution tick.
    pub last_revolution_tick: u64,
    /// §4.4: Black market state.
    pub black_market: BlackMarketState,
    /// §19.5.E: Site work ticks.
    pub site_work_ticks: Vec<u32>,
}

/// Version of the snapshot format.
/// Version 2 → 3: Added mind_models to AgentBundle.
/// Version 3 → 4: Added cultural_cognition to AgentBundle.
/// v3: mind_models, v4: cultural_cognition, v5: decision_policy
pub const SNAPSHOT_VERSION: u32 = 5;

/// Bundles all simulation state references needed to capture a snapshot.
/// Replaces the 21-parameter `capture()` signature with a single struct.
pub struct CaptureContext<'a> {
    pub config: &'a SimConfig,
    pub tick: Tick,
    pub master_seed: u64,
    pub world: &'a World,
    pub agents: &'a [AgentBundle],
    pub relationships: &'a [Relationship],
    pub events: &'a [mindstrata_core::event::SimEvent],
    pub journal: &'a EventJournal,
    pub norms: &'a NormRegistry,
    pub institutions: &'a [Institution],
    pub provenance: &'a CausalProvenance,
    pub season: &'a SeasonTracker,
    pub ecology_config: &'a EcologyConfig,
    pub health_config: &'a HealthConfig,
    pub demography_config: &'a DemographyConfig,
    pub agent_diseases: &'a [Vec<ActiveDisease>],
    pub market: &'a MarketState,
    pub knowledge_store: &'a [Knowledge],
    pub metric_history: &'a [MetricsSnapshot],
    pub last_revolution_tick: u64,
    pub black_market: &'a BlackMarketState,
    pub site_work_ticks: &'a [u32],
}

impl Snapshot {
    /// Create a snapshot from the current simulation state.
    /// This captures all deterministic state needed for replay.
    pub fn capture(ctx: &CaptureContext<'_>) -> Self {
        Self {
            version: SNAPSHOT_VERSION,
            config: ctx.config.clone(),
            tick: ctx.tick.as_u64(),
            clock_tick: ctx.tick,
            master_seed: ctx.master_seed,
            world: ctx.world.clone(),
            agents: ctx.agents.to_vec(),
            relationships: ctx.relationships.to_vec(),
            events: ctx.events.to_vec(),
            journal: ctx.journal.clone(),
            norms: ctx.norms.clone(),
            institutions: ctx.institutions.to_vec(),
            provenance: ctx.provenance.clone(),
            season: ctx.season.clone(),
            ecology_config: ctx.ecology_config.clone(),
            health_config: ctx.health_config.clone(),
            demography_config: ctx.demography_config.clone(),
            agent_diseases: ctx.agent_diseases.to_vec(),
            market: ctx.market.clone(),
            knowledge_store: ctx.knowledge_store.to_vec(),
            metric_history: ctx.metric_history.to_vec(),
            last_revolution_tick: ctx.last_revolution_tick,
            black_market: ctx.black_market.clone(),
            site_work_ticks: ctx.site_work_ticks.to_vec(),
        }
    }

    /// Serialize this snapshot to bytes using postcard.
    pub fn to_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    /// Deserialize a snapshot from bytes with version validation.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        let snapshot: Self = postcard::from_bytes(bytes)?;
        if snapshot.version != SNAPSHOT_VERSION {
            tracing::warn!(
                snapshot_version = snapshot.version,
                expected = SNAPSHOT_VERSION,
                "Snapshot version mismatch — data may be incompatible"
            );
        }
        Ok(snapshot)
    }

    /// Serialize this snapshot to a JSON string (for debugging).
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize a snapshot from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let snapshot: Self = serde_json::from_str(json)?;
        if snapshot.version != SNAPSHOT_VERSION {
            tracing::warn!(
                expected = SNAPSHOT_VERSION,
                got = snapshot.version,
                "Incompatible snapshot JSON version"
            );
        }
        Ok(snapshot)
    }

    /// Compute a deterministic hash of the snapshot for verification.
    /// Uses postcard serialization (deterministic) then a simple FNV-1a hash.
    pub fn deterministic_hash(&self) -> Result<u64, postcard::Error> {
        let bytes = self.to_bytes()?;
        Ok(fnv1a_hash(&bytes))
    }

    /// Verify that this snapshot matches the given hash.
    pub fn verify_hash(&self, expected: u64) -> Result<bool, postcard::Error> {
        let actual = self.deterministic_hash()?;
        Ok(actual == expected)
    }

    /// Total number of events in the snapshot.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Total number of journal entries in the snapshot.
    pub fn journal_len(&self) -> usize {
        self.journal.len()
    }

    /// Generate a summary of this snapshot for display.
    pub fn summary(&self) -> SnapshotSummary {
        let n = self.agents.len() as f64;
        SnapshotSummary {
            version: self.version,
            tick: self.tick,
            seed: self.config.seed,
            agent_count: self.agents.len(),
            event_count: self.events.len(),
            journal_len: self.journal.len(),
            institution_count: self.institutions.len(),
            total_grain: self.world.total_food().to_f64(),
            total_water: self.world.total_water().to_f64(),
            avg_hunger: if n > 0.0 {
                self.agents.iter().map(|a| a.needs.hunger.to_f64()).sum::<f64>() / n
            } else {
                0.0
            },
            avg_thirst: if n > 0.0 {
                self.agents.iter().map(|a| a.needs.thirst.to_f64()).sum::<f64>() / n
            } else {
                0.0
            },
            avg_fatigue: if n > 0.0 {
                self.agents.iter().map(|a| a.needs.fatigue.to_f64()).sum::<f64>() / n
            } else {
                0.0
            },
        }
    }

    /// Generate a filename for this snapshot.
    /// Format: world_seed_{seed}_tick_{tick}.snapshot
    pub fn filename(&self) -> String {
        format!("world_seed_{}_tick_{}.snapshot", self.config.seed, self.tick)
    }

    /// Save this snapshot to a file using postcard binary format.
    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        let bytes = self.to_bytes().map_err(|e| format!("Serialization failed: {e}"))?;
        std::fs::write(path, &bytes).map_err(|e| format!("File write failed: {e}"))?;
        tracing::info!(
            path = %path.display(),
            bytes = bytes.len(),
            tick = self.tick,
            "Snapshot saved"
        );
        Ok(())
    }

    /// Load a snapshot from a file using postcard binary format.
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("File read failed: {e}"))?;
        let snapshot = Self::from_bytes(&bytes).map_err(|e| format!("Deserialization failed: {e}"))?;
        tracing::info!(
            path = %path.display(),
            tick = snapshot.tick,
            agents = snapshot.agents.len(),
            "Snapshot loaded"
        );
        Ok(snapshot)
    }
}

/// A summary of a snapshot for display purposes.
#[derive(Debug, Clone)]
pub struct SnapshotSummary {
    pub version: u32,
    pub tick: u64,
    pub seed: u64,
    pub agent_count: usize,
    pub event_count: usize,
    pub journal_len: usize,
    pub institution_count: usize,
    pub total_grain: f64,
    pub total_water: f64,
    pub avg_hunger: f64,
    pub avg_thirst: f64,
    pub avg_fatigue: f64,
}

/// Simple FNV-1a hash function for deterministic hashing.
fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::SimConfig;

    fn make_test_snapshot() -> Snapshot {
        let config = SimConfig {
            seed: 42,
            max_ticks: 100,
            world_width: 4,
            world_height: 4,
            num_agents: 3,
            snapshot_interval: None,
        };
        Snapshot {
            version: SNAPSHOT_VERSION,
            config: config,
            tick: 100,
            clock_tick: Tick::new(100),
            master_seed: 42,
            world: World::new(4, 4),
            agents: vec![],
            relationships: vec![],
            events: vec![],
            journal: EventJournal::new(),
            norms: NormRegistry::new(),
            institutions: vec![],
            provenance: CausalProvenance::new(),
            season: crate::ecology::SeasonTracker::new(8760),
            ecology_config: crate::ecology::EcologyConfig::default(),
            health_config: crate::health::HealthConfig::default(),
            demography_config: crate::demography::DemographyConfig::default(),
            agent_diseases: vec![],
            market: crate::market::MarketState::new(&crate::parameters::SimParameters::default()),
            knowledge_store: vec![],
            metric_history: vec![],
            last_revolution_tick: 0,
            black_market: crate::black_market::BlackMarketState::default(),
            site_work_ticks: vec![],
        }
    }

    #[test]
    fn snapshot_postcard_roundtrip() {
        let original = make_test_snapshot();
        let bytes = original.to_bytes().expect("Failed to serialize");
        let restored = Snapshot::from_bytes(&bytes).expect("Failed to deserialize");
        assert_eq!(original.tick, restored.tick);
        assert_eq!(original.config.seed, restored.config.seed);
        assert_eq!(original.agents.len(), restored.agents.len());
    }

    #[test]
    fn snapshot_json_roundtrip() {
        let original = make_test_snapshot();
        let json = original.to_json().expect("Failed to serialize to JSON");
        let restored = Snapshot::from_json(&json).expect("Failed to deserialize from JSON");
        assert_eq!(original.tick, restored.tick);
        assert_eq!(original.config.seed, restored.config.seed);
    }

    #[test]
    fn snapshot_deterministic_hash() {
        let s1 = make_test_snapshot();
        let s2 = make_test_snapshot();
        let h1 = s1.deterministic_hash().expect("Failed to hash");
        let h2 = s2.deterministic_hash().expect("Failed to hash");
        assert_eq!(h1, h2, "Identical snapshots should have identical hashes");
    }

    #[test]
    fn snapshot_filename() {
        let snapshot = make_test_snapshot();
        let filename = snapshot.filename();
        assert!(filename.contains("seed_42"));
        assert!(filename.contains("tick_100"));
        assert!(filename.ends_with(".snapshot"));
    }

    #[test]
    fn snapshot_summary() {
        let snapshot = make_test_snapshot();
        let summary = snapshot.summary();
        assert_eq!(summary.version, SNAPSHOT_VERSION);
        assert_eq!(summary.tick, 100);
        assert_eq!(summary.seed, 42);
    }

    #[test]
    fn snapshot_event_count() {
        let snapshot = make_test_snapshot();
        assert_eq!(snapshot.event_count(), 0);
    }
}
