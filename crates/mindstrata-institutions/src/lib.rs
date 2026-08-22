//! Institutional domain: governance, law, diplomacy, military, religion,
//! education, norms, factions.
//!
//! Extracted from `mindstrata-sim` (S3 of `docs/PLAN_SCALING_FOUNDATION.md`).
//! Pure domain types and free functions; Simulation glue stays in
//! `mindstrata-sim::sim::{institutions,legal,...}_impl`.

#![deny(missing_docs)]

pub mod diplomacy;
pub mod factions;
pub mod institutions;
pub mod legal;
pub mod military;
pub mod norms;
pub mod schools;
pub mod theology;
