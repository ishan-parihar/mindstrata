//! Social layer — relationships, attachment, status, kinship, factions.
//!
//! Architecture §10-§12: Relationships are first-class systems with stages,
//! memories, obligations, attachment patterns, power dynamics, public labels,
//! private feelings, and social witnesses.
//!
//! ```text
//! Social systems:
//!   interaction          — agent-to-agent social interactions (talk, help, gossip, etc.)
//!   relationship_v2      — rich relationship model with stages and deep dimensions
//!   relationship_stages  — explicit stage definitions and transitions
//!   attraction           — multi-factor attraction model
//!   courtship            — courtship mechanics from attraction to pair-bond
//!   status_dims          — multi-dimensional status (dominance, prestige, authority, etc.)
//!   kinship              — kinship graph (biological, marriage, adoption, ritual links)
//!   household            — household as primary social unit
//!   clan                 — clan formation from kinship + narrative + alliance
//!   hierarchy            — hierarchy formation and destabilization
//!   marriage             — pair bonds and marriage as institution
//!   cult                 — cult formation and dissolution dynamics
//!   group_formation      — emergent group cohesion and dissociation
//!   patronage            — asymmetric patron/client power relations
//!   epistemic            — belief and epistemic style
//! ```

pub mod attraction;
pub mod clan;
pub mod courtship;
pub mod cult;
pub mod epistemic;
pub mod faction_v2;
pub mod group_formation;
pub mod hierarchy;
pub mod household;
pub mod interaction;
pub mod kinship;
pub mod marriage;
pub mod patronage;
pub mod relational_power;
pub mod relationship_stages;
pub mod relationship_v2;
pub mod speech_act;
pub mod status_dims;

// Re-export key types for convenient access
pub use attraction::AttractionModel;
pub use clan::Clan;
pub use courtship::Courtship;
pub use cult::{CultDynamics, CultRegistry};
pub use epistemic::EpistemicState;
pub use faction_v2::{FactionV2, FactionV2Registry};
pub use group_formation::{
    derive_group_attachment_style, GroupAttachmentStyle, GroupCandidate, GroupId, GroupRegistry,
    GroupType, PeerGroup,
};
pub mod relational_field;
pub use household::Household;
pub use interaction::{
    process_interaction, select_interaction_target, system_social_interactions, Interaction,
};
pub use kinship::KinshipGraph;
pub use marriage::{Marriage, MarriageRegistry, PairBond, RomanticStage};
pub use patronage::{PatronageRegistry, PatronageRelation};
pub use relational_field::{RelationalFields, PERCEPTION_RADIUS};
pub use relationship_stages::{try_advance_stage, try_regress_stage};
pub use relationship_v2::{RelationshipStage, RelationshipV2};
pub use speech_act::{
    ActDomain, RelationalIntent, SpeechAct, SpeechActKind, SpeechEffect, SPEECH_LOG_CAPACITY,
};
pub use status_dims::StatusDimensions;
