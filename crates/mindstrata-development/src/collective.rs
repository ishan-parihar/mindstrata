//! Village `CollectiveField` (STORY 10-11 + WP-I activation, AP3 03-substrate §2).
//!
//! The collective holon mirrors the per-person `DevelopmentField` but over the
//! 29 collective lines (sourced from the vendored `LineId` registry per FR-020).
//! Reads are pure and deterministic; `step_collective` (WP-I, Iter-266)
//! integrates per-line press from the catalyst-derived pressure vector,
//! advances the shadow stage coordinate on press saturation
//! (transcend-and-include), and tracks a fulfillment EMA (stage-adequate
//! signal).
//!
//! Zero-at-zero identity law: a fully neutral field under an all-zero pressure
//! vector returns bit-identical — the sim's empty-window pin
//! (`collective_field_empty_window_is_identity`) stays green and goldens are
//! untouched (the field is not in any golden/snapshot metric projection).
//!
//! Calibration: growth/EMA constants are CALIBRATION-PENDING(AP3); the probe
//! `i266_collective_wp_i` (12-seed family, 5K ticks) measures the trajectory
//! and validates that the bucket mapping differentiates before any behavioral
//! consumer anchors on the field.

use crate::canon::STAGE_COUNT;
use crate::line::{all_lines, LineId, Scope};

/// The vendored collective line count (WP-A registry). The plan's "8 collective
/// lines" was a placeholder; the actual KosmOS registry carries N=29. The
/// holon is fixed-size once we know the count — change in WP-0A lockstep with
/// the registry bump.
pub const COLLECTIVE_LINE_COUNT: usize = 29;

/// Per-tick press growth fraction under full pressure (WP-I v1).
///
/// CALIBRATION-PENDING(AP3): probe `i266_collective_wp_i` measures the
/// 5000-tick press equilibrium across the 12-seed family at natural
/// per-capita catalyst rates; the value must keep a stage 1→2 advance
/// reachable within a 50K horizon (crisis worlds concentrate catalysts)
/// without saturating press at 1.0 in calm windows.
pub const COLLECTIVE_PRESS_GROWTH: f64 = 0.05;

/// Fulfillment EMA rate — how fast the stage-adequate signal tracks pressure.
///
/// CALIBRATION-PENDING(AP3): same probe; the EMA must track sustained
/// pressure with a ~50-tick lag, not per-tick catalyst noise.
pub const COLLECTIVE_FULFILLMENT_EMA: f64 = 0.02;

/// Catalyst-pressure bucket a collective line listens to (WP-I v1 mapping).
///
/// The sim-side derivation (`system_collective_field_step`) counts per-capita
/// catalysts into four buckets: Bond → Relational, Threat/Transgression →
/// Safety, Grief → Identity, plus a small baseline into Meaning. This mapping
/// assigns each of the 29 vendored collective lines exactly one bucket by
/// vault `kind`:
/// - `culture` lines carry relational/bonding content (aesthetic, mythos,
///   narrative, religion, worldview…)
/// - `system` lines carry order/security content (governance, justice,
///   health, security, economic-systems…)
/// - `collective-system` (knowledge-systems) carries shared-memory identity
/// - `consciousness` lines carry village-meaning content
///
/// CALIBRATION-PENDING(AP3): the bucket→line affinity awaits vendor coupling
/// data (same documented debt as cross-line resonance in `dynamics.rs`);
/// i266 measures whether the mapping differentiates (per-bucket press
/// spread > 0 across the seed family).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectiveBucket {
    /// Bond catalysts — bonding, shared practice, communal meaning-making.
    Relational,
    /// Threat/Transgression catalysts — order, security, justice strain.
    Safety,
    /// Grief catalysts — shared loss, collective memory, identity.
    Identity,
    /// Ambient baseline — the world exists, so it has meaning.
    Meaning,
}

/// Public read side of the vendored-`kind` bucket affinity (Era IV, i273):
/// which catalyst bucket a collective line listens to. The sim-side genesis
/// consumer uses this to key generated content on the *advanced* lines
/// without duplicating the vault-kind mapping.
#[must_use]
pub fn bucket_for_line(line: LineId) -> CollectiveBucket {
    match line.kind() {
        "culture" => CollectiveBucket::Relational,
        "system" => CollectiveBucket::Safety,
        "collective-system" => CollectiveBucket::Identity,
        // consciousness kinds (consciousness-state, framework-complexity).
        _ => CollectiveBucket::Meaning,
    }
}

fn collect_collective_slugs() -> Vec<LineId> {
    all_lines()
        .filter(|l| l.scope() == Scope::Collective)
        .collect()
}

/// Build the per-line pressure vector from the four catalyst buckets.
///
/// Index order is `all_lines()` filtered to `Scope::Collective`, in vendored
/// registry order — the same order as `CollectiveField::lines`. Every
/// collective line receives exactly its bucket's pressure, clamped to [0,1].
///
/// Replaces the DC-1 v1 cyclic `i % 4` distribution: the vendored `kind` data
/// now owns the affinity instead of index arithmetic (WP-I deliverable).
#[must_use]
pub fn pressure_vector(
    relational: f64,
    safety: f64,
    identity: f64,
    meaning: f64,
) -> [f64; COLLECTIVE_LINE_COUNT] {
    let mut v = [0.0_f64; COLLECTIVE_LINE_COUNT];
    for (i, line) in collect_collective_slugs().into_iter().enumerate() {
        if i >= COLLECTIVE_LINE_COUNT {
            break;
        }
        v[i] = match bucket_for_line(line) {
            CollectiveBucket::Relational => relational,
            CollectiveBucket::Safety => safety,
            CollectiveBucket::Identity => identity,
            CollectiveBucket::Meaning => meaning,
        }
        .clamp(0.0, 1.0);
    }
    v
}

/// Per-field step parameters (WP-I). Carries the CALIBRATION-PENDING(AP3)
/// constants so the IC-5 change-order path can specialize them without a
/// signature break — same pattern as `dynamics::OperatorParams`.
#[derive(Debug, Clone, Copy)]
pub struct CollectiveParams {
    /// Per-tick press growth fraction under full pressure.
    pub press_growth: f64,
    /// Fulfillment EMA tracking rate.
    pub fulfillment_ema: f64,
}

impl CollectiveParams {
    /// Pending canon defaults (see the constant docs for the probe plan).
    #[must_use]
    pub const fn pending() -> Self {
        Self {
            press_growth: COLLECTIVE_PRESS_GROWTH,
            fulfillment_ema: COLLECTIVE_FULFILLMENT_EMA,
        }
    }
}

/// A single collective line's state on the village holon.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CollectiveLineState {
    /// Shadow stage coordinate in [1..=17]; quantized view via `StageCoord::floor`.
    pub stage: f64,
    /// Transition accumulator toward the next stage.
    pub press: f64,
    /// EMA of stage-adequate signal — drives the transcend-and-include gate.
    pub fulfillment: f64,
}

impl CollectiveLineState {
    /// Neutral collective line (per AP3 03-substrate §2 — `lambda=1.0`, `press=0.0`).
    #[must_use]
    pub const fn neutral() -> Self {
        Self {
            stage: 1.0,
            press: 0.0,
            fulfillment: 0.0,
        }
    }

    /// Advance one tick under `pressure` in [0,1] (WP-I semantics).
    ///
    /// - press integrates `pressure × params.press_growth` toward the next
    ///   stage transition;
    /// - on press saturation (≥ 1.0) below the ladder ceiling the shadow
    ///   stage advances by one and the surplus press carries over
    ///   (transcend-and-include: the accumulated transition fires);
    /// - fulfillment EMA-tracks the pressure (stage-adequate signal).
    ///
    /// Identity-at-zero: zero pressure on a line with zero press returns the
    /// line unchanged bit-for-bit. (A line with accumulated press still steps
    /// its fulfillment EMA toward zero under absent pressure — press is
    /// accumulated transition energy, fulfillment is a signal EMA.)
    #[must_use]
    pub fn step(&self, pressure: f64, params: &CollectiveParams) -> Self {
        if pressure == 0.0 && self.press == 0.0 {
            return *self;
        }
        let p = pressure.clamp(0.0, 1.0);
        let raw_press = self.press + p * params.press_growth;
        let ceiling = f64::from(STAGE_COUNT);
        let (stage, press) = if raw_press >= 1.0 && self.stage < ceiling {
            (self.stage + 1.0, raw_press - 1.0)
        } else {
            // At the ladder ceiling the transition cannot fire; press pins.
            (self.stage, raw_press.min(1.0))
        };
        let fulfillment = self.fulfillment + (p - self.fulfillment) * params.fulfillment_ema;
        Self {
            stage,
            press,
            fulfillment,
        }
    }
}

/// Village holon state (29 collective lines + Λ frontier).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CollectiveField {
    /// One `CollectiveLineState` per collective line, in vendored order.
    pub lines: [CollectiveLineState; COLLECTIVE_LINE_COUNT],
    /// Λ window width (AP3 03-substrate §6 — starts `1.0`, widen only with probe).
    pub lambda: f64,
}

impl Default for CollectiveField {
    fn default() -> Self {
        Self::neutral()
    }
}

impl CollectiveField {
    /// Fully neutral village holon — the founder default.
    #[must_use]
    pub const fn neutral() -> Self {
        Self {
            lines: [CollectiveLineState::neutral(); COLLECTIVE_LINE_COUNT],
            lambda: 1.0,
        }
    }

    /// All-N collective line slugs the vendored registry references (FR-020).
    #[must_use]
    pub fn line_slugs() -> Vec<LineId> {
        collect_collective_slugs()
    }

    /// True when every collective line is at the founder neutral (used by
    /// the sim's empty-window pin and restore-path diagnostics).
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.lambda == 1.0
            && self
                .lines
                .iter()
                .all(|l| l.stage == 1.0 && l.press == 0.0 && l.fulfillment == 0.0)
    }

    /// Mean fulfillment across the lines listening to `bucket`.
    ///
    /// The behavioral-consumer read side: an anchor compares this against a
    /// probe-measured peak (Iter-112 pattern) rather than a raw threshold.
    #[must_use]
    pub fn mean_fulfillment_for_bucket(&self, bucket: CollectiveBucket) -> f64 {
        let slugs = collect_collective_slugs();
        let mut sum = 0.0_f64;
        let mut n = 0_usize;
        for (i, line) in slugs.into_iter().enumerate() {
            if i >= COLLECTIVE_LINE_COUNT {
                break;
            }
            if bucket_for_line(line) == bucket {
                sum += self.lines[i].fulfillment;
                n += 1;
            }
        }
        if n == 0 {
            0.0
        } else {
            sum / n as f64
        }
    }

    /// WP-I step: integrate per-line pressures and advance every line.
    ///
    /// Pure function of (field snapshot, pressure vector, params). Zero RNG.
    /// Identity-at-zero: an all-neutral field under an all-zero pressure
    /// vector returns `self` unchanged bit-for-bit.
    #[must_use]
    pub fn step_collective(
        &self,
        pressures: &[f64; COLLECTIVE_LINE_COUNT],
        params: &CollectiveParams,
    ) -> Self {
        let mut any = false;
        let mut lines = self.lines;
        for (i, line) in lines.iter_mut().enumerate() {
            let p = pressures[i];
            if p != 0.0 || line.press != 0.0 {
                *line = line.step(p, params);
                any = true;
            }
        }
        if !any {
            return *self;
        }
        Self {
            lines,
            lambda: self.lambda,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_collective_is_identity() {
        let cf = CollectiveField::neutral();
        assert!(cf.is_neutral());
        let p = CollectiveParams::pending();
        let stepped = cf.step_collective(&[0.0; COLLECTIVE_LINE_COUNT], &p);
        assert_eq!(stepped, cf);
    }

    #[test]
    fn line_slugs_match_vendored_registry() {
        let slugs = CollectiveField::line_slugs();
        assert_eq!(slugs.len(), COLLECTIVE_LINE_COUNT);
        for s in slugs {
            assert!(!s.slug().is_empty());
        }
    }

    #[test]
    fn single_line_step_is_identity_at_zero() {
        let l = CollectiveLineState::neutral();
        let stepped = l.step(0.0, &CollectiveParams::pending());
        assert_eq!(stepped, l);
    }

    #[test]
    fn single_line_step_accumulates_press() {
        let l = CollectiveLineState::neutral();
        let stepped = l.step(0.5, &CollectiveParams::pending());
        assert!(stepped.press > 0.0);
        assert!(stepped.press < 1.0);
    }

    /// WP-I: full pressure integrates press to the transition threshold and
    /// the shadow stage advances (transcend-and-include fires).
    #[test]
    fn sustained_full_pressure_advances_stage() {
        let params = CollectiveParams::pending();
        let mut l = CollectiveLineState::neutral();
        // 0.05 growth → press 1.0 exactly at tick 20.
        for tick in 1..=40 {
            l = l.step(1.0, &params);
            if tick < 20 {
                assert!(l.stage == 1.0, "no advance before saturation (tick {tick})");
            } else {
                assert!(l.stage >= 2.0, "advance fired by tick {tick}");
                break;
            }
        }
    }

    /// WP-I: the bucket mapping assigns every collective line exactly one
    /// bucket by vault kind, with the documented counts (12 culture /
    /// 14 system / 1 collective-system / 2 consciousness).
    #[test]
    fn pressure_vector_maps_every_line_by_kind() {
        let slugs = CollectiveField::line_slugs();
        let (mut culture, mut system, mut ksys, mut consci) = (0, 0, 0, 0);
        for l in &slugs {
            match l.kind() {
                "culture" => culture += 1,
                "system" => system += 1,
                "collective-system" => ksys += 1,
                "consciousness" => consci += 1,
                other => panic!("unexpected collective line kind: {other}"),
            }
        }
        assert_eq!((culture, system, ksys, consci), (12, 14, 1, 2));

        // Relational pressure lands only on culture lines; a system line
        // under relational-only pressure stays at neutral.
        let v = pressure_vector(1.0, 0.0, 0.0, 0.0);
        let params = CollectiveParams::pending();
        let f = CollectiveField::neutral().step_collective(&v, &params);
        for (i, l) in slugs.iter().enumerate() {
            if l.kind() == "culture" {
                assert!(
                    f.lines[i].press > 0.0,
                    "culture line {} must move",
                    l.slug()
                );
            } else {
                assert_eq!(
                    f.lines[i],
                    CollectiveLineState::neutral(),
                    "non-culture line {} must stay neutral under relational-only pressure",
                    l.slug()
                );
            }
        }
    }

    /// WP-I: fulfillment EMA tracks sustained pressure without per-tick
    /// noise coupling.
    #[test]
    fn fulfillment_ema_tracks_pressure() {
        let params = CollectiveParams::pending();
        let mut l = CollectiveLineState::neutral();
        for _ in 0..100 {
            l = l.step(0.5, &params);
        }
        // EMA toward 0.5 at rate 0.02: after 100 ticks ≈ 0.5·(1−0.98^100) ≈ 0.43.
        assert!(l.fulfillment > 0.35 && l.fulfillment < 0.5);
        // Absent pressure decays the EMA toward zero (press persists).
        for _ in 0..500 {
            l = l.step(0.0, &params);
        }
        assert!(
            l.fulfillment < 0.01,
            "EMA must decay toward 0, got {}",
            l.fulfillment
        );
        assert!(
            l.press > 0.0,
            "accumulated press persists (transition energy)"
        );
    }

    /// WP-I: the consumer read side aggregates per-bucket fulfillment.
    #[test]
    fn mean_fulfillment_for_bucket_aggregates_by_kind() {
        let params = CollectiveParams::pending();
        // Safety-only pressure: system lines accumulate fulfillment,
        // culture lines stay at 0.
        let v = pressure_vector(0.0, 1.0, 0.0, 0.0);
        let mut f = CollectiveField::neutral();
        for _ in 0..100 {
            f = f.step_collective(&v, &params);
        }
        assert!(f.mean_fulfillment_for_bucket(CollectiveBucket::Safety) > 0.3);
        assert_eq!(
            f.mean_fulfillment_for_bucket(CollectiveBucket::Relational),
            0.0
        );
    }
}
