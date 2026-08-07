//! Psychological layer — structured mind for each agent.
//!
//! The agent should not merely have "traits + needs + emotions."
//! It should have a structured mind with:
//! - Interoception (body → felt experience)
//! - Self-model (identity, roles, values, narrative)
//! - Theory of mind (modeling other agents' minds)
//! - Emotion regulation (strategies for managing emotions)
//! - Attachment (relational security patterns)
//! - Moral cognition (moral foundations and norm internalization)
//! - Imagination/prospection (mental simulation of futures)
//! - Narrative (life story and meaning-making)
//! - Developmental psychology (lifespan development)
//! - Psychopathology (mental health risk dynamics)
//! - Skills and habits (learned behaviors)
//!
//! Architecture:
//! ```text
//! Biology → Interoception → Affect → Appraisal → Emotion
//!                                        ↓
//!                              Emotion Regulation → Modulated Affect
//!                                        ↓
//!                              Self-Model → Identity Defense
//!                                        ↓
//!                              Theory of Mind → Social Inference
//!                                        ↓
//!                              Moral Cognition → Norm Compliance
//!                                        ↓
//!                              Imagination → Prospective Simulation
//!                                        ↓
//!                              Narrative → Meaning-Making
//! ```

pub mod attachment;
pub mod cognitive_runtime;
pub mod cultural_cognition;
pub mod decision_policy;
pub mod developmental;
pub mod emotion_regulation;
pub mod imagination;
pub mod interoception;
pub mod moral_cognition;
pub mod motivation;
pub mod neural_like;
pub mod narrative;
pub mod psychopathology;
pub mod self_model;
pub mod skill;
pub mod theory_of_mind;

pub use attachment::AttachmentSystem;
pub use cognitive_runtime::CognitiveRuntime;
pub use cultural_cognition::CulturalCognition;
pub use decision_policy::DecisionPolicy;
pub use developmental::DevelopmentalPsychState;
pub use emotion_regulation::{EmotionRegulationState, RegulationStrategy};
pub use imagination::ProspectionState;
pub use interoception::InteroceptiveState;
pub use moral_cognition::MoralCognition;
pub use motivation::{MotivationState, MotiveCategory};
pub use neural_like::NeuralLikeState;
pub use narrative::NarrativeIdentity;
pub use psychopathology::PsychopathologyState;
pub use self_model::SelfModel;
pub use skill::SkillState;
pub use theory_of_mind::OtherMindModel;
