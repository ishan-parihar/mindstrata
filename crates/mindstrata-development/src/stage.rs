//! Stage coordinates over the canonical 17-rung unified ladder (WP-A).
//!
//! Types are frozen against the vendored ladder (`canon_gen::tables::LADDER`,
//! sha-pinned in [`crate::canon_gen`]); band predicates derive from vendor
//! altitude data, never hand-maintained lists.

use crate::canon_gen::tables::{ladder_stage, LADDER};
use core::fmt;

/// A 1-based position on the unified ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StageCoord(u8);

impl StageCoord {
    /// Smallest valid stage.
    pub const MIN: StageCoord = StageCoord(1);
    /// Largest valid stage.
    pub const MAX: StageCoord = StageCoord(17);

    /// Construct, rejecting out-of-ladder indices.
    #[must_use]
    pub fn new(stage: u8) -> Option<Self> {
        (1..=17).contains(&stage).then_some(Self(stage))
    }

    /// Raw index (1-based).
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Previous rung, if any.
    #[must_use]
    pub fn prev(self) -> Option<Self> {
        Self::new(self.0 - 1)
    }

    /// Next rung, if any.
    #[must_use]
    pub fn next(self) -> Option<Self> {
        Self::new(self.0 + 1)
    }

    /// Vendor altitude tag for this rung (e.g. `amber`, `teal`).
    #[must_use]
    pub fn altitude(self) -> &'static str {
        ladder_stage(self.0).map_or("unknown", |l| l.altitude)
    }

    /// Parsed altitude class for this rung.
    #[must_use]
    pub fn altitude_class(self) -> Option<Altitude> {
        Altitude::parse(self.altitude())
    }
}

impl TryFrom<u8> for StageCoord {
    type Error = &'static str;

    fn try_from(stage: u8) -> Result<Self, Self::Error> {
        Self::new(stage).ok_or("stage outside the 1..=17 unified ladder")
    }
}

impl fmt::Display for StageCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "S{}", self.0)
    }
}

/// Altitude color classes from the vendor ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Altitude {
    /// Infrared (stages 1–2).
    Infrared,
    /// Magenta (stage 3).
    Magenta,
    /// Red (stages 4–5).
    Red,
    /// Amber (stages 6–10).
    Amber,
    /// Orange (stages 11–12).
    Orange,
    /// Green (stage 13).
    Green,
    /// Teal (stage 14).
    Teal,
    /// Turquoise (stage 15).
    Turquoise,
    /// Indigo (stage 16).
    Indigo,
    /// Violet (stage 17).
    Violet,
}

impl Altitude {
    /// Parse a vendor altitude tag; `None` on unattested tags so unknown
    /// data stays visible instead of silently mapping anywhere.
    #[must_use]
    pub fn parse(tag: &str) -> Option<Self> {
        Some(match tag {
            "infrared" => Self::Infrared,
            "magenta" => Self::Magenta,
            "red" => Self::Red,
            "amber" => Self::Amber,
            "orange" => Self::Orange,
            "green" => Self::Green,
            "teal" => Self::Teal,
            "turquoise" => Self::Turquoise,
            "indigo" => Self::Indigo,
            "violet" => Self::Violet,
            _ => return None,
        })
    }

    /// Conventional-band predicate used by placement gating (draft): the
    /// ethnocentric span stages 6..=10 per the vendor identity bands.
    #[must_use]
    pub fn is_conventional_band(self) -> bool {
        matches!(self, Self::Amber)
    }
}

/// Iterate every rung in ladder order.
pub fn all_stages() -> impl Iterator<Item = StageCoord> {
    LADDER.iter().map(|l| StageCoord(l.stage))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_reject_out_of_ladder_indices() {
        assert!(StageCoord::new(0).is_none());
        assert!(StageCoord::new(18).is_none());
        assert_eq!(StageCoord::new(1).map(StageCoord::get), Some(1));
        assert!(StageCoord::try_from(18).is_err());
    }

    #[test]
    fn prev_next_walk_the_ladder() {
        assert_eq!(StageCoord::MIN.prev(), None);
        assert_eq!(StageCoord::MAX.next(), None);
        let mid = StageCoord::new(7).unwrap();
        assert_eq!(mid.prev().map(StageCoord::get), Some(6));
        assert_eq!(mid.next().map(StageCoord::get), Some(8));
    }

    #[test]
    fn altitudes_derive_from_vendor_table() {
        assert_eq!(
            StageCoord::new(1).unwrap().altitude_class(),
            Some(Altitude::Infrared)
        );
        assert_eq!(
            StageCoord::new(7).unwrap().altitude_class(),
            Some(Altitude::Amber)
        );
        assert_eq!(
            StageCoord::new(17).unwrap().altitude_class(),
            Some(Altitude::Violet)
        );
        // Every rung must parse — unattested tags would be a vendor drift signal.
        for s in all_stages() {
            assert!(s.altitude_class().is_some(), "unparsed altitude at {s}");
        }
    }

    #[test]
    fn display_uses_s_prefix() {
        assert_eq!(StageCoord::new(12).unwrap().to_string(), "S12");
    }
}
