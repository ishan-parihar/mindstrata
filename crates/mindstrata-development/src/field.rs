//! Attractor-field placement (WP-B): pure functions mapping continuous
//! developmental readings onto ladder coordinates over the frozen types.
//!
//! Deterministic contract: same reading ⇒ same placement, no RNG, no clock.
//! Readings are per-line altitudes on the continuous [0, 17] span where the
/// integer part selects the rung and the fractional part is headroom toward
/// the next rung. Neutral readings (zero) place at nothing — an empty
/// reading set yields an empty placement, preserving zero-at-zero.
use crate::canon_gen::tables::ladder_stage;
use crate::line::LineId;
use crate::stage::StageCoord;
use core::fmt;

/// One line's continuous developmental reading.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineReading {
    /// Owning line.
    pub line: LineId,
    /// Continuous altitude in [0, 17]; values outside clamp at the ends.
    pub altitude: f64,
}

impl LineReading {
    /// Construct, normalizing non-finite inputs to the untouched-field
    /// reading (deterministic poison-control: NaN must not silently corrupt
    /// placement math downstream) and clamping into the ladder span.
    #[must_use]
    pub fn new(line: LineId, altitude: f64) -> Self {
        let a = if altitude.is_finite() { altitude } else { 0.0 };
        Self {
            line,
            altitude: a.clamp(0.0, 17.0),
        }
    }

    /// Dominant rung: ceil of the altitude bounded into 1..=17, so any
    /// positive progress lands on rung 1 first and exactly 17.0 tops out.
    #[must_use]
    pub fn dominant_rung(&self) -> StageCoord {
        let n = self.altitude.ceil().clamp(1.0, 17.0);
        // Clamp guarantees the span; representable exactly as u8.
        StageCoord::new_clamped(if n >= 17.0 { 17 } else { n as u8 })
    }

    /// Fractional headroom toward the next rung within the current band,
    /// in [0, 1). At exact integer boundaries this reads 0 — the agent is
    /// standing ON that rung, not progressing into it.
    #[must_use]
    pub fn band_progress(&self) -> f64 {
        self.altitude.fract()
    }

    /// Distance (in rungs) to the next rung; 0 when at the top.
    #[must_use]
    pub fn distance_to_next(&self) -> f64 {
        if self.altitude >= 17.0 {
            0.0
        } else {
            let next = self.altitude.floor() + 1.0;
            next - self.altitude
        }
    }

    /// Vendor canonical name of the dominant rung.
    #[must_use]
    pub fn rung_name(&self) -> &'static str {
        ladder_stage(self.dominant_rung().get()).map_or("unknown", |l| l.name)
    }
}

/// A placed coordinate pair — the atom of field state downstream (WP-C
/// resonance keys off these).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldCoordinate {
    /// Line axis.
    pub line: LineId,
    /// Stage axis.
    pub stage: StageCoord,
}

impl fmt::Display for FieldCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.line, self.stage)
    }
}

/// Place a set of readings into field coordinates (one per reading, in the
/// same order — index-stable so downstream consumers iterate
/// deterministically).
#[must_use]
pub fn place(readings: &[LineReading]) -> Vec<FieldCoordinate> {
    readings
        .iter()
        .map(|r| FieldCoordinate {
            line: r.line,
            stage: r.dominant_rung(),
        })
        .collect()
}

/// Ladder-wide population of a line: how many of the 17 rungs sit at or
/// below the reading (the "occupied depth" used by lambda gating later).
#[must_use]
pub fn occupied_depth(reading: &LineReading) -> u8 {
    reading.dominant_rung().get()
}

/// Total ladder coverage across all lines — a cheap whole-field scalar for
/// observability; sum of occupied depths divided by (lines × 17).
#[must_use]
pub fn field_coverage(readings: &[LineReading]) -> f64 {
    if readings.is_empty() {
        return 0.0;
    }
    let total: f64 = readings.iter().map(|r| f64::from(occupied_depth(r))).sum();
    total / (readings.len() as f64 * 17.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cog(v: f64) -> LineReading {
        LineReading::new(LineId::new("cognitive").expect("registered"), v)
    }

    #[test]
    fn neutral_reading_places_at_first_rung_not_zero() {
        // Zero altitude = untouched field: still a valid S1 standing, because
        // the ladder has no rung 0 — identity-at-neutral means "at the base".
        assert_eq!(cog(0.0).dominant_rung(), StageCoord::new(1).unwrap());
        assert_eq!(cog(0.0).band_progress(), 0.0);
    }

    #[test]
    fn clamping_holds_at_both_ends() {
        assert_eq!(cog(-5.0).dominant_rung(), StageCoord::new(1).unwrap());
        assert_eq!(cog(99.0).dominant_rung(), StageCoord::new(17).unwrap());
        assert_eq!(cog(99.0).distance_to_next(), 0.0);
    }

    #[test]
    fn boundary_values_stand_on_their_rung() {
        // Exactly 7.0 stands ON S7 (progress 0), not inside S7's band.
        let r = cog(7.0);
        assert_eq!(r.dominant_rung().get(), 7);
        assert_eq!(r.band_progress(), 0.0);
        assert!((r.distance_to_next() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn fractional_headroom_reads_band_progress() {
        let r = cog(7.25);
        assert_eq!(r.dominant_rung().get(), 8); // ceil: progressing into 8
        assert!((r.band_progress() - 0.25).abs() < 1e-12);
        assert!((r.distance_to_next() - 0.75).abs() < 1e-12);
    }

    #[test]
    fn placement_is_index_stable_and_deterministic() {
        let readings = vec![cog(3.2), cog(11.9), cog(0.4)];
        let a = place(&readings);
        let b = place(&readings);
        assert_eq!(a, b);
        assert_eq!(a.len(), 3);
        assert_eq!(a[1].stage.get(), 12); // ceil(11.9)
        assert_eq!(a[2].stage.get(), 1);
    }

    #[test]
    fn empty_readings_give_empty_placement_and_zero_coverage() {
        assert!(place(&[]).is_empty());
        assert_eq!(field_coverage(&[]), 0.0);
    }

    #[test]
    fn coverage_scales_with_occupied_depth() {
        let readings = vec![cog(17.0), cog(8.5)];
        // (17 + 9) / (2 × 17)
        assert!((field_coverage(&readings) - 26.0 / 34.0).abs() < 1e-12);
    }

    #[test]
    fn monotone_readings_never_place_lower() {
        let mut prev = 0;
        for step in [0.5, 1.0, 1.9, 3.3, 6.6, 9.9, 13.4, 16.9, 17.0] {
            let cur = cog(step).dominant_rung().get();
            assert!(cur >= prev, "placement regressed at {step}");
            prev = cur;
        }
    }
}
