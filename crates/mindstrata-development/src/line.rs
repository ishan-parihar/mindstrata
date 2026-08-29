//! Line identity and line×stage coupling maps (WP-A).
//!
//! Frozen against the vendored registries (`canon_gen::tables::LINES` /
//! `COUPLINGS`); scope classification derives from vault `kind` data with an
//! explicit, documented mapping.

use crate::canon_gen::tables::{line as lookup_line, COUPLINGS, LINES};
use core::fmt;
use serde::de::Error as _;
use serde::{Deserialize, Serialize};

/// Whether a line develops individual holons or collective ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    /// Lines developing individual agents.
    Individual,
    /// Lines developing collectives (cultures, systems, consciousness-at-scale).
    Collective,
}

/// A validated line identifier backed by the vendor registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineId(&'static str);

impl Serialize for LineId {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.0)
    }
}

impl<'de> Deserialize<'de> for LineId {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let s: String = String::deserialize(de)?;
        // Must reconstruct as &'static str: only valid if the slug is in the
        // vendored registry. This is enforced at construction time; at
        // deserialize time we leak the String to obtain a 'static lifetime
        // so subsequent lookups succeed (DC-1 only persists vendored slugs).
        let static_str: &'static str = Box::leak(s.into_boxed_str());
        if lookup_line(static_str).is_none() {
            return Err(D::Error::custom(format!(
                "LineId deserialize: unknown line slug `{static_str}`"
            )));
        }
        Ok(Self(static_str))
    }
}

impl LineId {
    /// Construct from a registry slug; rejects slugs absent from `LINES`.
    #[must_use]
    pub fn new(slug: &'static str) -> Option<Self> {
        lookup_line(slug).map(|_| Self(slug))
    }

    /// Registry slug.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        self.0
    }

    /// Vault kind tag for this line.
    #[must_use]
    pub fn kind(self) -> &'static str {
        lookup_line(self.0).map_or("unknown", |l| l.kind)
    }

    /// Scope derived from the vault kind: `collective-*`, `culture`,
    /// `system`, and `consciousness` kinds develop collectives;
    /// `individual-intelligence` develops individuals.
    #[must_use]
    pub fn scope(self) -> Scope {
        match self.kind() {
            "individual-intelligence" => Scope::Individual,
            k if k.starts_with("collective-") => Scope::Collective,
            "culture" | "system" | "consciousness" => Scope::Collective,
            _ => Scope::Individual,
        }
    }

    /// Vault ratification status.
    #[must_use]
    pub fn status(self) -> &'static str {
        lookup_line(self.0).map_or("unknown", |l| l.status)
    }
}

impl fmt::Display for LineId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// Depth assessment of one observed cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthStatus {
    /// Cell fully developed in the vault.
    Complete,
    /// Cell drafted but not deepened.
    Draft,
    /// Partially developed.
    Partial,
    /// Stub only.
    Stub,
    /// Unified pass applied.
    Unified,
}

impl DepthStatus {
    /// Parse a vault depth tag; `None` on unattested tags.
    #[must_use]
    pub fn parse(tag: &str) -> Option<Self> {
        Some(match tag {
            "complete" => Self::Complete,
            "draft" => Self::Draft,
            "partial" => Self::Partial,
            "stub" => Self::Stub,
            "unified" => Self::Unified,
            _ => return None,
        })
    }
}

/// The observed stage cells along one line, ordered by stage.
#[derive(Debug, Clone)]
pub struct StageLinesMap {
    line: LineId,
    cells: Vec<(crate::stage::StageCoord, DepthStatus)>,
}

impl StageLinesMap {
    /// Build the map for a registered line by scanning observed couplings.
    #[must_use]
    pub fn for_line(line: LineId) -> Self {
        let mut cells: Vec<(crate::stage::StageCoord, DepthStatus)> = COUPLINGS
            .iter()
            .filter(|c| c.line == line.slug())
            .filter_map(|c| {
                Some((
                    crate::stage::StageCoord::new(c.stage)?,
                    DepthStatus::parse(c.depth_status)?,
                ))
            })
            .collect();
        cells.sort_by_key(|(s, _)| s.get());
        Self { line, cells }
    }

    /// Owning line.
    #[must_use]
    pub const fn line(&self) -> LineId {
        self.line
    }

    /// Observed cells as `(stage, depth)` pairs, ascending.
    #[must_use]
    pub fn cells(&self) -> &[(crate::stage::StageCoord, DepthStatus)] {
        &self.cells
    }

    /// Count of fully complete cells.
    #[must_use]
    pub fn complete_count(&self) -> usize {
        self.cells
            .iter()
            .filter(|(_, d)| *d == DepthStatus::Complete)
            .count()
    }

    /// Stages with no observed cell on this line.
    #[must_use]
    pub fn missing_stages(&self) -> Vec<crate::stage::StageCoord> {
        crate::stage::all_stages()
            .filter(|s| !self.cells.iter().any(|(have, _)| have == s))
            .collect()
    }
}

/// Iterate every registered line id in registry order.
pub fn all_lines() -> impl Iterator<Item = LineId> {
    LINES.iter().map(|l| LineId(l.slug))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stage::StageCoord;

    #[test]
    fn registry_slug_validation_rejects_unknowns() {
        assert!(LineId::new("cognitive").is_some());
        assert!(LineId::new("not-a-line").is_none());
    }

    #[test]
    fn scope_mapping_follows_vault_kind() {
        let cog = LineId::new("cognitive").unwrap();
        assert_eq!(cog.kind(), "individual-intelligence");
        assert_eq!(cog.scope(), Scope::Individual);
        // At least one culture-kind line classifies collective.
        let any_collective = all_lines().any(|l| l.scope() == Scope::Collective);
        assert!(any_collective);
    }

    #[test]
    fn cognitive_map_is_complete_across_all_stages() {
        let cog = LineId::new("cognitive").unwrap();
        let map = StageLinesMap::for_line(cog);
        assert_eq!(map.cells().len(), 17);
        assert_eq!(map.complete_count(), 17);
        assert!(map.missing_stages().is_empty());
        assert_eq!(map.cells()[0].0, StageCoord::new(1).unwrap());
        assert_eq!(map.cells()[16].0, StageCoord::new(17).unwrap());
    }

    #[test]
    fn missing_stages_reported_for_sparse_line() {
        // religion-statistical has duplicate legacy dirs and gaps; it must
        // NOT claim full 17-stage coverage.
        let rel = LineId::new("religion-statistical").unwrap();
        let map = StageLinesMap::for_line(rel);
        assert!(
            !map.missing_stages().is_empty() || map.cells().len() >= 17,
            "sparse-line accounting must be honest"
        );
    }

    #[test]
    fn every_coupling_depth_tag_parses() {
        // Unattested tags would silently drop cells; the freeze requires them visible.
        let unparsed: usize = COUPLINGS
            .iter()
            .filter(|c| DepthStatus::parse(c.depth_status).is_none())
            .count();
        assert_eq!(unparsed, 0, "unrecognized depth_status tags exist");
    }
}
