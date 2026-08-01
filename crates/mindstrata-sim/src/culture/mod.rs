//! Cultural layer — memes, propaganda, and rituals.
//!
//! Culture emerges from agent communication, institutional broadcasting,
//! rituals, gossip, memory distortion, emotional amplification,
//! identity-protective cognition, and network topology.
//!
//! Architecture:
//! ```text
//! Memes → Propagation → Mutation → Competition → Cultural Evolution
//!         ↓
//! Propaganda → Institutional Narratives → Legitimacy → Power
//!         ↓
//! Rituals → Synchronization → Cohesion → Norm Reinforcement → Stability
//! ```

pub mod collective_memory;
pub mod echo_chamber;
pub mod knowledge;
pub mod meme;
pub mod meme_aggregator;
pub mod propaganda;
pub mod ritual;
pub mod rumor_v2;

pub use collective_memory::{CollectiveMemory, CollectiveMemoryRegistry, SharedMemory, SharedMemoryKind};
pub use echo_chamber::{BeliefCluster, EchoChamberState};
pub use knowledge::{cultural_pressure, diffuse_knowledge, taboo_severity, CulturalState, DiffusionResult, EffectTarget, Ideology, IdeologyValue, Knowledge, KnowledgeCategory, Practice, PracticeCategory, PracticeEffect};
pub use meme::{Meme, MemeContent, MemeRegistry};
pub use meme_aggregator::{AggregatedMemeMetrics, MemeAggregator};
pub use propaganda::{PropagandaCampaign, PropagandaChannel, PropagandaRegistry};
pub use ritual::{Ritual, RitualKind, RitualRegistry};
pub use rumor_v2::{RumorRegistry, RumorV2};
