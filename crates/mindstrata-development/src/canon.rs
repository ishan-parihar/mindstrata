//! Hand-written canon constants — the calibration surface.
//!
//! Every constant here is `CALIBRATION-PENDING(AP3)` until a probe measures it
//! and an IC-5 change-order lands the value (AGENTS.md §4.2 form). The single
//! obvious placeholder value keeps consumers compiling inert; nothing in sim
//! reads these yet, so goldens are untouched by construction.

/// Development-field uptake quantum: f64 shadow accumulations quantize into
/// the field exactly once per tick at this resolution.
///
/// CALIBRATION-PENDING(AP3): probe must measure the smallest behaviorally
/// visible field delta vs golden noise floor (i<iter>_field_quantum).
pub const FIELD_UPTAKE_QUANTUM_F64: f64 = 1.0e-4;

/// Neutral initialization for every person-side development coordinate.
///
/// CALIBRATION-PENDING(AP3): neutral point must satisfy identity-at-neutral
/// pins (FR-023); probe plan i<iter>_zero_at_zero.
pub const NEUTRAL_FIELD_VALUE_F64: f64 = 0.0;

/// Number of stages on the unified ladder (vendor-pinned, not calibrated).
pub const STAGE_COUNT: u8 = 17;
