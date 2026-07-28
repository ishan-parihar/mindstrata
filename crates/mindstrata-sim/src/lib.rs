//! Simulation crate for Mindstrata.
//!
//! Contains the world model, entity components, simulation systems,
//! world generation, scenario loading, and the tick-loop orchestrator.

#![allow(missing_docs)]

pub mod actions;
pub mod appraisal;
pub mod attention;
pub mod belief_update;
pub mod institutions;
pub mod journal;
pub mod memory;
pub mod norms;
pub mod person;
pub mod provenance;
pub mod routines;
pub mod scenario;
pub mod sim;
pub mod snapshot;
pub mod social;
pub mod systems;
pub mod world;
pub mod world_gen;

pub use sim::Simulation;
