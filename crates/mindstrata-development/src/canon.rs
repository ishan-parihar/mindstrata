//! Hand-written canon constants — the calibration surface.
//!
//! Every constant here is `CALIBRATION-PENDING(AP3)` until a probe measures it
//! and an IC-5 change-order lands the value (AGENTS.md §4.2 form).
//!
//! **These are READ** (i386 correction — the previous note claimed nothing in sim
//! consumed them): `FIELD_UPTAKE_QUANTUM_F64` gates field admission in
//! `lambda.rs:56` and `STAGE_COUNT` sets the collective-field ceiling in
//! `collective.rs:191`, both of which the sim drives every tick — so changing a
//! value here is a behavioural change that must be probed and golden-checked, not
//! a compile-time placeholder swap. `canon-inventory.md` tracks each constant's
//! landing status.

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
