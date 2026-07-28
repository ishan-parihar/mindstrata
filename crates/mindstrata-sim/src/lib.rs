//! Simulation crate for Mindstrata.
//!
//! Contains the world model, entity components, simulation systems,
//! world generation, scenario loading, and the tick-loop orchestrator.

#![allow(missing_docs)]

pub mod actions;
pub mod appraisal;
pub mod belief_update;
pub mod memory;
pub mod person;
pub mod scenario;
pub mod sim;
pub mod social;
pub mod systems;
pub mod world;
pub mod world_gen;

pub use sim::Simulation;
