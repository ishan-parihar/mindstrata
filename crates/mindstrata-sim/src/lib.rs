//! Simulation crate for Mindstrata.
//!
//! Contains the world model, entity components, simulation systems,
//! world generation, scenario loading, and the tick-loop orchestrator.

#![allow(missing_docs)]

pub mod actions;
pub mod agent_tier;
pub mod appraisal;
pub mod attention;
pub mod belief_update;
pub mod biology {
    pub use mindstrata_person::biology::*;
}
pub mod black_market;
pub mod conflict;
pub mod culture;
pub mod demography;
pub mod diplomacy;
pub mod ecology;
pub mod factions;
pub mod gossip;
pub mod health;
pub mod institutions;
pub mod journal;
pub mod legal;
pub mod logistics;
pub mod market;
pub mod memory;
pub mod military;
pub mod mods;
pub mod noosphere;
pub mod norms;
pub mod parameters {
    pub use mindstrata_core::parameters::*;
}
pub mod person {
    pub use mindstrata_person::person::*;
}
pub mod population_cap;
pub mod provenance;
pub mod psychology;
pub mod routines;
pub mod scenario;
pub mod scheduler;
pub mod schools;
pub mod sim;
pub mod snapshot;
pub mod social;
pub mod spec_lint;
pub mod systems;
pub mod theology;
pub mod world;
pub mod world_gen;

pub use sim::Simulation;
