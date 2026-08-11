//! Modding API — the formal content interface (AP2 Phase 4).
//!
//! A **content pack** is a directory containing a required `manifest.ron`
//! plus optional content files:
//!
//! ```text
//! my_mod/
//!   manifest.ron      (required: name, description, version)
//!   norms.ron         (optional: a RON `[Norm, ...]` — added to the
//!                      registry, so mod norms immediately participate in
//!                      per-tick norm-pressure computation)
//!   knowledge.ron     (optional: a RON `[Knowledge, ...]` — added to the
//!                      world knowledge store, so mod knowledge is
//!                      transmittable through the existing learning paths)
//!   scenario.ron      (optional: a RON `Scenario` — parse-checked on load
//!                      and runnable through the CLI `scenario` command,
//!                      which accepts a path to any `.ron` scenario file)
//! ```
//!
//! Packs are strictly **opt-in**: the CLI `sim --mod <dir>` flag or a
//! library call to [`Simulation::apply_content_pack`]. No default world
//! loads a pack, every loader/validator is pure, and `apply_content_pack`
//! is never called from the tick loop — so calibrated windows (golden
//! replays, snapshots) are structurally untouched.

use crate::culture::Knowledge;
use crate::norms::Norm;
use crate::scenario::Scenario;
use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::path::Path;

/// Metadata for a content pack, loaded from `manifest.ron`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    /// Human-readable pack name (required, non-empty).
    pub name: String,
    /// What the pack changes or adds.
    pub description: String,
    /// Pack version, e.g. "1.0.0".
    pub version: String,
}

/// A validated content pack: manifest plus whatever content files exist.
#[derive(Debug, Clone)]
pub struct ContentPack {
    /// Pack metadata.
    pub manifest: ModManifest,
    /// Optional scenario override (loaded from `scenario.ron`).
    pub scenario: Option<Scenario>,
    /// Knowledge additions (loaded from `knowledge.ron`).
    pub knowledge: Vec<Knowledge>,
    /// Norm additions (loaded from `norms.ron`).
    pub norms: Vec<Norm>,
}

/// Errors produced while loading or validating a content pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModError {
    /// A required file (the manifest) is missing.
    MissingFile { path: String },
    /// A file exists but could not be read.
    ReadFailed { path: String, reason: String },
    /// A file's contents are not valid RON for the expected type.
    ParseFailed { path: String, reason: String },
    /// Semantic validation failed (duplicate ids, out-of-range values, ...).
    Validation(String),
}

impl fmt::Display for ModError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModError::MissingFile { path } => write!(f, "missing required file: {path}"),
            ModError::ReadFailed { path, reason } => {
                write!(f, "failed to read {path}: {reason}")
            }
            ModError::ParseFailed { path, reason } => {
                write!(f, "failed to parse {path}: {reason}")
            }
            ModError::Validation(msg) => write!(f, "invalid content pack: {msg}"),
        }
    }
}

impl std::error::Error for ModError {}

/// Read a file that may be absent (`None`) or present but unreadable/parseable.
fn read_optional_ron<T: for<'de> Deserialize<'de>>(
    dir: &Path,
    file: &str,
) -> Result<Option<T>, ModError> {
    let path = dir.join(file);
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path).map_err(|e| ModError::ReadFailed {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;
    ron::from_str(&content)
        .map(Some)
        .map_err(|e| ModError::ParseFailed {
            path: path.display().to_string(),
            reason: e.to_string(),
        })
}

impl ContentPack {
    /// Load and validate a content pack from a directory.
    ///
    /// `manifest.ron` is required; `norms.ron`, `knowledge.ron`, and
    /// `scenario.ron` are optional. A pack with invalid content fails fast
    /// with a descriptive [`ModError`].
    pub fn load(dir: impl AsRef<Path>) -> Result<Self, ModError> {
        let dir = dir.as_ref();
        let manifest_path = dir.join("manifest.ron");
        let manifest_content =
            std::fs::read_to_string(&manifest_path).map_err(|_| ModError::MissingFile {
                path: manifest_path.display().to_string(),
            })?;
        let manifest: ModManifest =
            ron::from_str(&manifest_content).map_err(|e| ModError::ParseFailed {
                path: manifest_path.display().to_string(),
                reason: e.to_string(),
            })?;

        let scenario: Option<Scenario> = read_optional_ron(dir, "scenario.ron")?;
        let knowledge: Vec<Knowledge> =
            read_optional_ron(dir, "knowledge.ron")?.unwrap_or_default();
        let norms: Vec<Norm> = read_optional_ron(dir, "norms.ron")?.unwrap_or_default();

        let pack = Self {
            manifest,
            scenario,
            knowledge,
            norms,
        };
        pack.validate()?;
        Ok(pack)
    }

    /// Validate the pack's semantic invariants:
    /// - a non-empty manifest name,
    /// - unique knowledge ids and norm ids within the pack,
    /// - all probability-like fields within `0.0..=1.0`.
    pub fn validate(&self) -> Result<(), ModError> {
        if self.manifest.name.trim().is_empty() {
            return Err(ModError::Validation(
                "manifest.name must not be empty".into(),
            ));
        }

        let in_01 = |v: Fixed| v >= Fixed::ZERO && v <= Fixed::ONE;

        let mut knowledge_ids = HashSet::new();
        for k in &self.knowledge {
            if !knowledge_ids.insert(k.id) {
                return Err(ModError::Validation(format!(
                    "duplicate knowledge id {} ('{}')",
                    k.id, k.name
                )));
            }
            if !in_01(k.difficulty) || !in_01(k.utility) {
                return Err(ModError::Validation(format!(
                    "knowledge {} ('{}') has difficulty/utility outside 0..=1",
                    k.id, k.name
                )));
            }
        }

        let mut norm_ids = HashSet::new();
        for n in &self.norms {
            if !norm_ids.insert(n.id) {
                return Err(ModError::Validation(format!(
                    "duplicate norm id {} ('{}')",
                    n.id, n.name
                )));
            }
            if !in_01(n.strength) || !in_01(n.internalization) || !in_01(n.punishment) {
                return Err(ModError::Validation(format!(
                    "norm {} ('{}') has strength/internalization/punishment outside 0..=1",
                    n.id, n.name
                )));
            }
        }

        Ok(())
    }

    /// One-line summary for CLI output.
    pub fn describe(&self) -> String {
        let parts = [
            format!("{} v{}", self.manifest.name, self.manifest.version),
            self.manifest.description.clone(),
            format!(
                "{} norms, {} knowledge items{}",
                self.norms.len(),
                self.knowledge.len(),
                self.scenario
                    .as_ref()
                    .map(|s| format!(", scenario '{}'", s.name))
                    .unwrap_or_default(),
            ),
        ];
        parts.join(" — ")
    }
}

/// Result of [`Simulation::apply_content_pack`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppliedContent {
    /// Norms added to the registry.
    pub norms_added: usize,
    /// Knowledge items added to the world store.
    pub knowledge_added: usize,
}

impl crate::sim::Simulation {
    /// Apply a content pack's norms and knowledge to this simulation.
    ///
    /// Norms are registered into the live registry (they immediately
    /// participate in per-tick norm pressure); knowledge items are appended
    /// to the world knowledge store (transmittable through the existing
    /// learning paths). The pack's scenario is NOT auto-applied — scenarios
    /// are construction-time configuration, surfaced through the CLI
    /// `scenario` command (which accepts a path to any `.ron` file).
    ///
    /// Strictly opt-in: called only via the CLI `--mod-dir` flag or a library
    /// caller, never from the tick loop or any default world — calibrated
    /// windows are untouched. The pack is re-validated here (defense in
    /// depth: a programmatically constructed `ContentPack` skips `load`'s
    /// validation), and id collisions with existing registries are rejected
    /// with an error (never silently overwritten).
    pub fn apply_content_pack(&mut self, pack: &ContentPack) -> Result<AppliedContent, ModError> {
        // Re-validate even if the pack came from a struct literal rather
        // than `ContentPack::load` — no invalid content may enter a live
        // registry through any path.
        pack.validate()?;

        let existing_norm_ids: HashSet<u64> = self.norms.norms().iter().map(|n| n.id).collect();
        for norm in &pack.norms {
            if existing_norm_ids.contains(&norm.id) {
                return Err(ModError::Validation(format!(
                    "norm id {} ('{}') collides with an existing norm",
                    norm.id, norm.name
                )));
            }
        }
        let existing_knowledge_ids: HashSet<u64> =
            self.knowledge_store.iter().map(|k| k.id).collect();
        for knowledge in &pack.knowledge {
            if existing_knowledge_ids.contains(&knowledge.id) {
                return Err(ModError::Validation(format!(
                    "knowledge id {} ('{}') collides with existing knowledge",
                    knowledge.id, knowledge.name
                )));
            }
        }

        for norm in &pack.norms {
            self.norms.register(norm.clone());
        }
        self.knowledge_store.extend(pack.knowledge.clone());

        Ok(AppliedContent {
            norms_added: pack.norms.len(),
            knowledge_added: pack.knowledge.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{SimConfig, Simulation};
    use mindstrata_core::fixed::Fixed;
    use std::io::Write;

    fn write(dir: &Path, file: &str, content: &str) {
        let path = dir.join(file);
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    fn fixture_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("ms_mods_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    // `Fixed` serializes as its tuple struct `Fixed(raw i64)` (SCALE = 10 000),
    // so fixture values are written as `Fixed(6000)` for 0.6 etc.
    const VALID_MANIFEST: &str = r#"
        ModManifest(
            name: "River Trade Pact",
            description: "Adds the River-Trade norms and two craft techniques.",
            version: "1.0.0",
        )
    "#;

    const VALID_NORMS: &str = r#"
        [
            Norm(
                id: 100,
                name: "Honor River Trade",
                strength: Fixed(6000),
                internalization: Fixed(5000),
                punishment: Fixed(4000),
                reinforcing_identity: Some(Farmer),
            ),
        ]
    "#;

    const VALID_KNOWLEDGE: &str = r#"
        [
            Knowledge(
                id: 100,
                name: "River Ferry",
                category: Craft,
                difficulty: Fixed(4000),
                utility: Fixed(7000),
                holders: 0,
                discovered_tick: 0,
            ),
        ]
    "#;

    #[test]
    fn load_accepts_manifest_only_pack() {
        let dir = fixture_dir("manifest_only");
        write(&dir, "manifest.ron", VALID_MANIFEST);
        let pack = ContentPack::load(&dir).unwrap();
        assert_eq!(pack.manifest.name, "River Trade Pact");
        assert!(pack.norms.is_empty());
        assert!(pack.knowledge.is_empty());
        assert!(pack.scenario.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_accepts_full_pack_with_all_content_files() {
        let dir = fixture_dir("full");
        write(&dir, "manifest.ron", VALID_MANIFEST);
        write(&dir, "norms.ron", VALID_NORMS);
        write(&dir, "knowledge.ron", VALID_KNOWLEDGE);
        let pack = ContentPack::load(&dir).unwrap();
        assert_eq!(pack.norms.len(), 1);
        assert_eq!(pack.norms[0].id, 100);
        assert_eq!(pack.knowledge.len(), 1);
        assert_eq!(pack.knowledge[0].name, "River Ferry");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_rejects_missing_manifest() {
        let dir = fixture_dir("no_manifest");
        let err = ContentPack::load(&dir).unwrap_err();
        assert!(matches!(&err, ModError::MissingFile { .. }), "{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_rejects_malformed_norm_file() {
        let dir = fixture_dir("bad_ron");
        write(&dir, "manifest.ron", VALID_MANIFEST);
        write(&dir, "norms.ron", "this is not ron");
        let err = ContentPack::load(&dir).unwrap_err();
        assert!(matches!(&err, ModError::ParseFailed { .. }), "{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_rejects_duplicate_norm_ids() {
        let dir = fixture_dir("dup_norms");
        write(&dir, "manifest.ron", VALID_MANIFEST);
        write(
            &dir,
            "norms.ron",
            r#"
            [
                Norm(id: 100, name: "A", strength: Fixed(5000), internalization: Fixed(5000), punishment: Fixed(5000), reinforcing_identity: None),
                Norm(id: 100, name: "B", strength: Fixed(5000), internalization: Fixed(5000), punishment: Fixed(5000), reinforcing_identity: None),
            ]
            "#,
        );
        let err = ContentPack::load(&dir).unwrap_err();
        assert!(
            matches!(&err, ModError::Validation(msg) if msg.contains("duplicate norm id")),
            "{err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_rejects_out_of_range_values() {
        let dir = fixture_dir("range");
        write(&dir, "manifest.ron", VALID_MANIFEST);
        write(
            &dir,
            "knowledge.ron",
            r#"
            [
                Knowledge(id: 100, name: "Broken", category: Craft, difficulty: Fixed(15000), utility: Fixed(7000), holders: 0, discovered_tick: 0),
            ]
            "#,
        );
        let err = ContentPack::load(&dir).unwrap_err();
        assert!(
            matches!(&err, ModError::Validation(msg) if msg.contains("outside 0..=1")),
            "{err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_registers_norms_and_knowledge() {
        let dir = fixture_dir("apply_ok");
        write(&dir, "manifest.ron", VALID_MANIFEST);
        write(&dir, "norms.ron", VALID_NORMS);
        write(&dir, "knowledge.ron", VALID_KNOWLEDGE);
        let pack = ContentPack::load(&dir).unwrap();

        let mut sim = Simulation::new(SimConfig {
            seed: 7,
            max_ticks: 1000,
            world_width: 16,
            world_height: 16,
            num_agents: 6,
            snapshot_interval: None,
        });
        sim.populate();
        let norms_before = sim.norms.norms().len();
        let knowledge_before = sim.knowledge_store.len();

        let applied = sim.apply_content_pack(&pack).unwrap();
        assert_eq!(applied.norms_added, 1);
        assert_eq!(applied.knowledge_added, 1);
        assert_eq!(sim.norms.norms().len(), norms_before + 1);
        assert_eq!(sim.knowledge_store.len(), knowledge_before + 1);
        assert!(sim
            .norms
            .norms()
            .iter()
            .any(|n| n.id == 100 && n.name == "Honor River Trade"));
        assert!(sim
            .knowledge_store
            .iter()
            .any(|k| k.id == 100 && k.name == "River Ferry"));

        // The new norm participates in pressure computation without panicking.
        sim.tick();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_rejects_id_collision_with_existing_registries() {
        let dir = fixture_dir("apply_collision");
        write(&dir, "manifest.ron", VALID_MANIFEST);
        write(
            &dir,
            "norms.ron",
            r#"
            [
                Norm(id: 0, name: "No Theft Clash", strength: Fixed(5000), internalization: Fixed(5000), punishment: Fixed(5000), reinforcing_identity: None),
            ]
            "#,
        );
        let pack = ContentPack::load(&dir).unwrap();

        let mut sim = Simulation::new(SimConfig {
            seed: 7,
            max_ticks: 1000,
            world_width: 16,
            world_height: 16,
            num_agents: 6,
            snapshot_interval: None,
        });
        sim.populate();
        let err = sim.apply_content_pack(&pack).unwrap_err();
        assert!(
            err.to_string().contains("collides with an existing norm"),
            "{err}"
        );
        // Nothing was partially applied.
        assert_eq!(
            sim.norms
                .norms()
                .iter()
                .filter(|n| n.name == "No Theft Clash")
                .count(),
            0
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_revalidates_programmatic_pack() {
        // A pack constructed as a struct literal (skipping `ContentPack::load`)
        // must still be rejected by `apply_content_pack` — defense in depth.
        let pack = ContentPack {
            manifest: ModManifest {
                name: "Bad Range".into(),
                description: "strength out of range".into(),
                version: "0.1.0".into(),
            },
            scenario: None,
            knowledge: vec![],
            norms: vec![Norm {
                id: 500,
                name: "Overpowered".into(),
                strength: Fixed::from_f64(1.5),
                internalization: Fixed::from_f64(0.5),
                punishment: Fixed::from_f64(0.5),
                reinforcing_identity: None,
            }],
        };
        let mut sim = Simulation::new(SimConfig {
            seed: 7,
            max_ticks: 1000,
            world_width: 16,
            world_height: 16,
            num_agents: 6,
            snapshot_interval: None,
        });
        sim.populate();
        let err = sim.apply_content_pack(&pack).unwrap_err();
        assert!(
            err.to_string().contains("outside 0..=1"),
            "out-of-range content must be rejected at apply time: {err}"
        );
        assert!(
            sim.norms.norms().iter().all(|n| n.id != 500),
            "nothing must be registered when validation fails"
        );
    }

    #[test]
    fn range_check_accepts_boundary_and_rejects_outside() {
        let in_01 = |v: Fixed| v >= Fixed::ZERO && v <= Fixed::ONE;
        assert!(in_01(Fixed::from_f64(0.5)));
        assert!(in_01(Fixed::from_f64(1.0)));
        assert!(!in_01(Fixed::from_f64(1.1)));
        assert!(!in_01(Fixed::from_f64(-0.1)));
    }
}
