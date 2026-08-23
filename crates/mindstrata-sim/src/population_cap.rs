//! Population ceilings.
//!
//! Two constants with distinct jobs:
//! - [`MAX_POPULATION`] is the hard demographic ceiling the birth gates
//!   enforce (Iteration 261 lifts it 48 -> 256; the agent_tier LOD keeps
//!   large settlements tractable by demoting non-focal agents).
//! - [`STRESS_POPULATION`] preserves the historical full-settlement size
//!   (48, the section-19.5.F designed village) that scale suites and
//!   benches calibrate against, so lifting the ceiling does not silently
//!   resize those workloads.

/// Hard demographic ceiling enforced at populate and by the birth gates.
pub const MAX_POPULATION: usize = 256;

/// Historical designed-settlement size used by stress tests and benches.
pub const STRESS_POPULATION: usize = 48;
