//! Core kernel for Mindstrata.
//!
//! Provides the foundational building blocks used across all simulation layers:
//! - Fixed-point deterministic math
//! - Seeded RNG streams
//! - Entity and resource ID types
//! - Event system
//! - Clock and time types
//! - Error types

#![allow(missing_docs)]

pub mod clock;
pub mod error;
pub mod event;
pub mod fixed;
pub mod id;
pub mod rng;

pub use clock::{Clock, Tick};
pub use error::{Error, Result};
pub use fixed::Fixed;
pub use id::{AgentId, EntityId, ResourceId, SiteId};
pub use rng::RngStreams;
