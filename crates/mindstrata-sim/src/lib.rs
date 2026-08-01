//! Simulation crate for Mindstrata.
//!
//! Contains the world model, entity components, simulation systems,
//! world generation, scenario loading, and the tick-loop orchestrator.

#![allow(missing_docs)]

pub mod actions;
pub mod appraisal;
pub mod attention;
pub mod belief_update;
pub mod biology;
pub mod demography;
pub mod conflict;
pub mod culture;
pub mod ecology;
pub mod factions;
pub mod hierarchy;
pub mod gossip;
pub mod health;
pub mod logistics;
pub mod institutions;
pub mod journal;
pub mod memory;
pub mod norms;
pub mod parameters;
pub mod person;
pub mod provenance;
pub mod psychology;
pub mod scheduler;
pub mod routines;
pub mod scenario;
pub mod sim;
pub mod snapshot;
pub mod population_cap;
pub mod clan;
pub mod social;
pub mod systems;
pub mod world;
pub mod world_gen;
pub mod market;
pub mod black_market;
pub mod household;
pub mod kinship;
pub mod spec_lint;
pub mod relationship_v2;
pub mod attraction;
pub mod status_dims;

pub use sim::Simulation;
pub mod epistemic;
pub mod agent_tier;
