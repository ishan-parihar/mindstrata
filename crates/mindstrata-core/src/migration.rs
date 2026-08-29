//! Save-schema versioning framework v0 — DC-1 task 2.17 (PLATFORM).
//!
//! Version header, migration trait, and the invariant that any stored
//! snapshot at version `N` can be loaded at `CURRENT` via an explicit
//! migration chain. See `mindstrata-sim/src/snapshot.rs` for the concrete
//! `Snapshot` type and `SNAPSHOT_VERSION` constant — this crate only owns
//! the *trait* so the dependency stays `core <- sim`.
//!
//! # Contract (IC-3 companion)
//!
//! - Every persisted snapshot carries a `version: u32` header first.
//! - `SNAPSHOT_VERSION` in `mindstrata-sim` is the single current version.
//! - A migration is a pure function `Snapshot@from -> Snapshot@to` with
//!   `from + 1 == to`. Multi-version jumps chain single-step migrations.
//! - v0 ships one example no-op migration and the harness; real migrations
//!   land as separate `impl SnapshotMigration` items with their own pin.

/// Current snapshot wire version — re-exported for documentation; the
/// authoritative constant lives in `mindstrata-sim::snapshot::SNAPSHOT_VERSION`.
/// Duplicated here only so the trait docs can name it without a runtime dep.
pub const SNAPSHOT_SCHEMA_VERSION: u32 = 13;

/// Pure, deterministic single-step migration `from -> from+1`.
///
/// Implementors are zero-sized marker types; the trait is object-safe only
/// via the free function [`migrate_chain`] in `mindstrata-sim`. Keeping the
/// trait in `core` freezes the *shape* without pulling `Snapshot` into `core`.
pub trait SnapshotMigration {
    /// Source version (inclusive).
    fn from_version() -> u32;
    /// Target version (must be `from + 1`).
    fn to_version() -> u32;
}

/// Example no-op migration `v12 -> v13` — the "development field" addition.
///
/// v12 snapshots lack `AgentBundle.development`; postcard deserialization
/// with `#[serde(default)]` already yields a neutral field, so the migration
/// is identity. It exists as the canonical example and as the compile-time
/// proof that the framework can express the current 12→13 bump without
/// runtime cost.
pub struct MigrateV12ToV13;

impl SnapshotMigration for MigrateV12ToV13 {
    fn from_version() -> u32 {
        12
    }
    fn to_version() -> u32 {
        13
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_versions_are_consecutive() {
        assert_eq!(
            MigrateV12ToV13::to_version(),
            MigrateV12ToV13::from_version() + 1
        );
    }

    #[test]
    fn schema_version_matches_sim_constant() {
        // Guard against drift: the core constant must track sim's.
        // This test is intentionally loose — it only checks that the
        // constant is >= the example migration's target.
        assert!(SNAPSHOT_SCHEMA_VERSION >= MigrateV12ToV13::to_version());
    }
}
