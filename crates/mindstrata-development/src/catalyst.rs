//! IC-1 catalyst vocabulary v1.0.0 — FROZEN (DC-1 task 2.9).
//!
//! The typed channel by which mechanical simulation state becomes
//! developmental signal: the ONLY route from SIM mechanics into attractor
//! fields. SIM observes and emits; it never mutates state to feed.
//!
//! Frozen surfaces (change via IC change-order only):
//! - [`Drive`] — the AP3 motivational axis `Agency | Communion | Eros | Agape`
//! - [`CatalystKind`] — the producer-side event classification
//! - [`CatalystEvent`] — the carrier, with per-line resonance weights
//! - [`kind_drive_map`] — the total kind→drive projection
//!
//! The SIM observer harness (`mindstrata-sim::sim::catalyst_observers`)
//! drafts these events post-write-back into inert buffers; STORY consumes
//! them through the development pass (task 3.x wiring).
//!
//! Skill-mastery firewall (binding): skill/craft mastery is NOT a producer.
//! Mastery context enters field dynamics only through witnessed-evaluation
//! channels; no direct stage credit flows from skill milestones.

use crate::line::LineId;
use mindstrata_core::clock::Tick;
use mindstrata_core::id::AgentId;

/// AP3 motivational drive axis (frozen).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Drive {
    /// Mastery, competence, effective power over self and world.
    Agency,
    /// Belonging, attachment, relational embeddedness.
    Communion,
    /// Desire, appetite, approach toward the valued object.
    Eros,
    /// Selfless care, release, donation beyond the self.
    Agape,
}

/// Producer-side event classification (frozen set; extension needs a
/// change-order naming the new producer pass).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalystKind {
    /// Loss of an attachment figure (deaths pass).
    Grief,
    /// Formation or expansion of an attachment bond
    /// (marriage/birth producers).
    Bond,
    /// Direct exposure to conflict or violence (conflict/feud producers).
    Threat,
    /// Personal violation of a norm one holds (norms/legal producers).
    Transgression,
}

/// One catalyst crossing the SIM→STORY boundary.
///
/// `magnitude` is an f64 shadow per IC-3 numeric discipline; downstream
/// uptake quantizes ONCE at the field boundary. `line_tags` carry per-line
/// resonance weights in [0,1]; cross-line weights are zero until vendor
/// coupling data lands (see `dynamics::resonance_weight`).
#[derive(Debug, Clone, PartialEq)]
pub struct CatalystEvent {
    /// Tick the producing event occurred.
    pub tick: Tick,
    /// Affected agent.
    pub subject: AgentId,
    /// Producer-side classification.
    pub kind: CatalystKind,
    /// Resolved motivational drive.
    pub drive: Drive,
    /// Per-line resonance weights (line, weight∈[0,1]).
    pub line_tags: Vec<(LineId, f64)>,
    /// Pressure magnitude in [0,1] (CALIBRATION-PENDING(AP3)).
    pub magnitude: f64,
}

/// The total kind→drive projection (frozen mapping).
///
/// - Grief → Agape: loss metabolized as release/care beyond the self.
/// - Bond → Communion: attachment formation is belonging itself.
/// - Threat → Agency: dominance struggle engages effective power.
/// - Transgression → Agency: norm breach is an assertion of self against
///   the collective; guilt/reparation then routes back through Agape in
///   the pathology operator's golden quadrant.
#[must_use]
pub const fn kind_drive_map(kind: CatalystKind) -> Drive {
    match kind {
        CatalystKind::Grief => Drive::Agape,
        CatalystKind::Bond => Drive::Communion,
        CatalystKind::Threat | CatalystKind::Transgression => Drive::Agency,
    }
}

impl CatalystEvent {
    /// Construct with the frozen drive resolution; validates magnitudes
    /// and line weights into [0,1] so no out-of-range value can enter the
    /// channel.
    #[must_use]
    pub fn new(
        tick: Tick,
        subject: AgentId,
        kind: CatalystKind,
        line_tags: Vec<(LineId, f64)>,
        magnitude: f64,
    ) -> Self {
        Self {
            tick,
            subject,
            kind,
            drive: kind_drive_map(kind),
            line_tags: line_tags
                .into_iter()
                .map(|(l, w)| (l, w.clamp(0.0, 1.0)))
                .collect(),
            magnitude: magnitude.clamp(0.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lid(slug: &'static str) -> LineId {
        LineId::new(slug).expect("registered")
    }

    #[test]
    fn drive_projection_is_total_and_frozen() {
        let kinds = [
            CatalystKind::Grief,
            CatalystKind::Bond,
            CatalystKind::Threat,
            CatalystKind::Transgression,
        ];
        for kind in kinds {
            let _ = kind_drive_map(kind);
        }
        assert_eq!(kind_drive_map(CatalystKind::Grief), Drive::Agape);
        assert_eq!(kind_drive_map(CatalystKind::Bond), Drive::Communion);
        assert_eq!(kind_drive_map(CatalystKind::Threat), Drive::Agency);
        assert_eq!(kind_drive_map(CatalystKind::Transgression), Drive::Agency);
    }

    #[test]
    fn constructor_clamps_magnitudes_and_weights() {
        let ev = CatalystEvent::new(
            Tick::new(7),
            AgentId::new(3),
            CatalystKind::Bond,
            vec![(lid("emotional-interpersonal"), 4.0)],
            -2.0,
        );
        assert_eq!(ev.magnitude, 0.0);
        assert_eq!(ev.line_tags[0].1, 1.0);
        assert_eq!(ev.drive, Drive::Communion);
    }

    #[test]
    fn zero_magnitude_event_is_well_formed() {
        // Zero-at-zero law: a zero-magnitude catalyst may exist but must be
        // constructible and carry no hidden pressure.
        let ev = CatalystEvent::new(
            Tick::new(0),
            AgentId::new(0),
            CatalystKind::Threat,
            Vec::new(),
            0.0,
        );
        assert_eq!(ev.magnitude, 0.0);
        assert!(ev.line_tags.is_empty());
    }
}
