//! Cultural layer — memes, propaganda, rituals, knowledge, and meaning-making.
//!
//! Culture emerges from agent communication, institutional broadcasting,
//! rituals, gossip, memory distortion, emotional amplification,
//! identity-protective cognition, and network topology.
//!
//! Architecture §13:
//! ```text
//! Memes → Propagation → Mutation → Competition → Cultural Evolution
//!         ↓
//! Propaganda → Institutional Narratives → Legitimacy → Power
//!         ↓
//! Rituals → Synchronization → Cohesion → Norm Reinforcement → Stability
//!         ↓
//! Narrative Frames → Meaning-Making → Identity → Resilience / Radicalization
//!         ↓
//! Ideology → Clusters → Polarization → Echo Chambers
//!         ↓
//! Sacred Values → Moral Outrage → Tradeoff Resistance → Conflict
//!         ↓
//! Education → Knowledge Transmission → Cultural Continuity
//! ```

pub mod collective_memory;
pub mod education;
pub mod echo_chamber;
pub mod ideology;
pub mod knowledge;
pub mod meme;
pub mod meme_aggregator;
pub mod narrative_frame;
pub mod propaganda;
pub mod ritual;
pub mod rumor_v2;
pub mod sacred;
pub mod technology;

pub use collective_memory::{ACTIVE_TRAUMA_SALIENCE_THRESHOLD, CollectiveMemory, CollectiveMemoryRegistry, SharedMemory, SharedMemoryKind, SharedTrauma};
pub use echo_chamber::{BeliefCluster as EchoBeliefCluster, EchoChamberState};
pub use education::{EducationState, EducationEvent};
pub use ideology::{BeliefEcology, BeliefCluster, Ideology, IdeologyAxis};
pub use knowledge::{cultural_pressure, diffuse_knowledge, taboo_severity, CulturalState, DiffusionResult, EffectTarget, Ideology as KnowledgeIdeology, IdeologyValue, Knowledge, KnowledgeCategory, Practice, PracticeCategory, PracticeEffect};
pub use meme::{Meme, MemeContent, MemeRegistry};
pub use meme_aggregator::{AggregatedMemeMetrics, MemeAggregator};
pub use narrative_frame::{NarrativeFrame, NarrativeFrameSet};
pub use propaganda::{PropagandaCampaign, PropagandaChannel, PropagandaRegistry};
pub use ritual::{Ritual, RitualKind, RitualRegistry};
pub use rumor_v2::{RumorRegistry, RumorV2};
pub use sacred::{SacredValue, SacredValues};
pub use technology::TechnologyTree;
