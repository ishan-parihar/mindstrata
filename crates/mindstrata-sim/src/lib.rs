//! Simulation crate for Mindstrata.
//!
//! Contains the world model, entity components, simulation systems,
//! world generation, scenario loading, and the tick-loop orchestrator.

#![allow(missing_docs)]

pub mod actions;
pub mod agent_tier;
pub mod appraisal {
    pub use mindstrata_psych::appraisal::*;
}
pub mod attention;
pub mod belief_update;
pub mod biology {
    pub use mindstrata_person::biology::*;
}
pub mod black_market {
    pub use mindstrata_world::black_market::*;
}
pub mod conflict;
pub mod culture {
    pub use mindstrata_social::culture::*;
}
pub mod demography {
    pub use mindstrata_world::demography::*;
}
pub mod diplomacy {
    pub use mindstrata_institutions::diplomacy::*;
}
pub mod ecology {
    pub use mindstrata_world::ecology::*;
}
pub mod factions {
    pub use mindstrata_institutions::factions::*;
}
pub mod gossip;
pub mod health;
pub mod institutions {
    pub use mindstrata_institutions::institutions::*;
}
pub mod journal;
pub mod legal {
    pub use mindstrata_institutions::legal::*;
}
pub mod logistics {
    pub use mindstrata_world::logistics::*;
}
pub mod market {
    pub use mindstrata_world::market::*;
}
pub mod memory;
pub mod military {
    pub use mindstrata_institutions::military::*;
}
pub mod mods;
pub mod noosphere {
    pub use mindstrata_social::noosphere::*;
}
pub mod norms {
    pub use mindstrata_institutions::norms::*;
}
pub mod parameters {
    pub use mindstrata_core::parameters::*;
}
pub mod person {
    pub use mindstrata_person::person::*;
}
pub mod population_cap;
pub mod provenance;
pub mod psychology {
    pub use mindstrata_psych::psychology::*;
}
pub mod routines;
pub mod scenario;
pub mod scheduler;
pub mod schools {
    pub use mindstrata_institutions::schools::*;
}
pub mod sim;
pub mod snapshot;
pub mod social {
    pub use mindstrata_social::social::*;
}
pub mod spec_lint;
pub mod systems;
pub mod theology {
    pub use mindstrata_institutions::theology::*;
}
pub mod world {
    pub use mindstrata_world::world::*;
}
pub mod world_gen {
    pub use mindstrata_world::world_gen::*;
}

pub use sim::Simulation;
