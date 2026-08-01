//! §13: Noospheric layer — the symbolic field where minds interact.
//!
//! The noosphere includes rumors, memes, ideologies, sacred symbols,
//! collective memories, propaganda, moral panics, and legitimacy narratives.
//!
//! Architecture §13: The noosphere is the symbolic layer above culture.
//! It emerges from agent communication, institutional broadcasting,
//! rituals, gossip, memory distortion, emotional amplification,
//! identity-protective cognition, and network topology.

pub mod field;
pub mod belief_ecology;
pub mod legitimacy;
pub mod moral_panic;

pub use field::{NoosphericField, SymbolicNode, SymbolicEdge};
pub use belief_ecology::{BeliefEcologyNoosphere, NarrativeCluster};
pub use legitimacy::{LegitimacyField, LegitimacySource};
pub use moral_panic::{MoralPanic, MoralPanicRegistry, PanicTrigger};
