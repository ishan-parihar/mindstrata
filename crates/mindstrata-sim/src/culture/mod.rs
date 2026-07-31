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

pub mod meme;
pub mod propaganda;
pub mod ritual;

pub use meme::{Meme, MemeContent, MemeRegistry};
pub use propaganda::{PropagandaCampaign, PropagandaChannel, PropagandaRegistry};
pub use ritual::{Ritual, RitualKind, RitualRegistry};
