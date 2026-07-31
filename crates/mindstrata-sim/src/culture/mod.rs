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
pub mod meme;
pub mod propaganda;
pub mod ritual;
pub mod rumor_v2;

pub use collective_memory::{CollectiveMemory, CollectiveMemoryRegistry, SharedMemory, SharedMemoryKind};
pub use echo_chamber::{BeliefCluster, EchoChamberState};
pub use meme::{Meme, MemeContent, MemeRegistry};
pub use propaganda::{PropagandaCampaign, PropagandaChannel, PropagandaRegistry};
pub use ritual::{Ritual, RitualKind, RitualRegistry};
pub use rumor_v2::{RumorRegistry, RumorV2};
