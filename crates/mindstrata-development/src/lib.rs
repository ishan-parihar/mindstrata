//! mindstrata-development — AP3 attractor-field substrate (leaf crate).
//!
//! Hosts the vendored ontology canon (generated tables under [`canon_gen`],
//! hand-written calibration constants under [`canon`]) and, in later waves,
//! the pure field engine: stage/line placement (`WP-B`), resonance and the
//! ratified 4-fold pathology operator (`WP-C`), gating (`WP-D`).
//!
//! Contract discipline: types derive from vendor data at pinned sha
//! (vendor/afa/PROVENANCE.md); freezes record hashes (IC-3/IC-5 consumers
//! quote them). Zero sim-crate dependencies — this crate is a leaf on core.

#![deny(missing_docs)]

pub mod canon;
pub mod canon_gen;
pub mod dynamics;
pub mod field;
pub mod lambda;
pub mod line;
pub mod stage;
