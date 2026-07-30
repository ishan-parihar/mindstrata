//! Psychological layer — structured mind for each agent.
//!
//! The agent should not merely have "traits + needs + emotions."
//! It should have a structured mind with:
//! - Interoception (body → felt experience)
//! - Self-model (identity, roles, values, narrative)
//! - Theory of mind (modeling other agents' minds)
//!
//! Architecture:
//! ```text
//! Biology → Interoception → Affect → Appraisal → Emotion
//!                                        ↓
//!                              Self-Model → Identity Defense
//!                                        ↓
//!                              Theory of Mind → Social Inference
//! ```

pub mod interoception;
pub mod self_model;
pub mod theory_of_mind;

pub use interoception::InteroceptiveState;
pub use self_model::SelfModel;
pub use theory_of_mind::OtherMindModel;
