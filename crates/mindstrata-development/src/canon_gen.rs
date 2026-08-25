//! Generated canon tables — INCLUDED verbatim from vendor/afa/gen/canon_tables.rs
//! (single source of truth; regenerate via `python3 scripts/afa_codegen.py`).
//!
//! Vendored at KosmOS sha 868b2239da3af06909ec10e82345b98d1082f04a; see
//! vendor/afa/PROVENANCE.md for extraction provenance and row counts.

/// Ladder stages (17), line registries (49) and observed couplings (851).
pub mod tables {
    /// Canonical 17-stage unified ladder entry.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LadderStage {
        /// 1-based stage index on the unified ladder.
        pub stage: u8,
        /// URL-ish slug (`prehension`, `sentential-faith`, …).
        pub slug: &'static str,
        /// Canonical display name.
        pub name: &'static str,
        /// Altitude color tag (`infrared`…`violet`).
        pub altitude: &'static str,
        /// Identity band from IST crosswalk.
        pub identity_band: &'static str,
        /// MHC order (0–16 as string).
        pub mhc_order: &'static str,
        /// MHC order name.
        pub mhc_name: &'static str,
        /// Kegan transition marker.
        pub kegan: &'static str,
    }

    /// A line registry entry from the vault.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LineEntry {
        /// Line slug (`cognitive`, `values`, …).
        pub slug: &'static str,
        /// Vault `kind` (e.g. `individual-intelligence`).
        pub kind: &'static str,
        /// Quadrant tag (`UL`, `UR`, `LL`, `LR`).
        pub quadrant: &'static str,
        /// Ratification status in the vault.
        pub status: &'static str,
        /// Required cell count per vault convention (17 for complete matrices).
        pub matrix_cells_required: u32,
        /// Vault depth assessment of the matrix.
        pub matrix_depth: &'static str,
    }

    /// An observed line×stage cell.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Coupling {
        /// Owning line slug.
        pub line: &'static str,
        /// Stage number (1–17).
        pub stage: u8,
        /// Stage slug at this rung.
        pub stage_slug: &'static str,
        /// Altitude recorded by the cell.
        pub altitude: &'static str,
        /// Vault depth status (`complete`, `draft`, …).
        pub depth_status: &'static str,
    }

    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../vendor/afa/gen/canon_tables.rs"
    ));

    /// Look up a ladder rung by 1-based stage number.
    #[must_use]
    pub fn ladder_stage(stage: u8) -> Option<&'static LadderStage> {
        LADDER.iter().find(|l| l.stage == stage)
    }

    /// Look up a ladder rung by slug.
    #[must_use]
    pub fn ladder_by_slug(slug: &str) -> Option<&'static LadderStage> {
        LADDER.iter().find(|l| l.slug == slug)
    }

    /// Look up a line registry entry by slug.
    #[must_use]
    pub fn line(slug: &str) -> Option<&'static LineEntry> {
        LINES.iter().find(|l| l.slug == slug)
    }

    /// All observed cells along one line, ordered by stage.
    #[must_use]
    pub fn line_couplings(line: &str) -> Vec<&'static Coupling> {
        let mut v: Vec<&Coupling> = COUPLINGS.iter().filter(|c| c.line == line).collect();
        v.sort_by_key(|c| c.stage);
        v
    }

    /// The observed coupling for one line×stage, if the vault has the cell.
    #[must_use]
    pub fn coupling(line: &str, stage: u8) -> Option<&'static Coupling> {
        COUPLINGS
            .iter()
            .find(|c| c.line == line && c.stage == stage)
    }
}

#[cfg(test)]
mod tests {
    use super::tables::*;

    #[test]
    fn ladder_has_exactly_seventeen_stages_in_order() {
        assert_eq!(LADDER.len(), 17);
        for (i, rung) in LADDER.iter().enumerate() {
            assert_eq!(rung.stage as usize, i + 1);
        }
    }

    #[test]
    fn slug_round_trip_through_index() {
        let seven = ladder_stage(7).expect("stage 7 exists");
        assert_eq!(seven.slug, "sentential-faith");
        assert_eq!(ladder_by_slug("sentential-faith").map(|l| l.stage), Some(7));
        assert!(ladder_by_slug("nonexistent-rung").is_none());
    }

    #[test]
    fn registry_lookups_resolve_curated_lines() {
        let cog = line("cognitive").expect("cognitive registered");
        assert_eq!(cog.quadrant, "UL");
        assert_eq!(cog.matrix_cells_required, 17);
        assert!(line("not-a-line").is_none());
    }

    #[test]
    fn cognitive_matrix_is_complete_to_stage_seventeen() {
        let cells = line_couplings("cognitive");
        assert_eq!(cells.len(), 17);
        assert_eq!(cells.first().map(|c| c.stage), Some(1));
        assert_eq!(cells.last().map(|c| c.stage), Some(17));
        assert!(coupling("cognitive", 12).is_some());
        // Documented vault quirk: religion-statistical carries BOTH a
        // legacy-altitude-named duplicate set and canonical-slug dirs; its
        // stage-12 cell resolves via 12-formal-mind (see PROVENANCE.md), so
        // the coupling EXISTS even though the 12-formal-landmark dir is empty.
        assert!(coupling("religion-statistical", 12).is_some());
        // Truly absent lookups must return None, not invent rows.
        assert!(coupling("not-a-line", 7).is_none());
    }
}
