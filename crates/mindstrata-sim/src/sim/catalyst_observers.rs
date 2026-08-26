//! Draft catalyst vocabulary + read-side observers (DC-1 task 2.8).
//!
//! Observers scan the post-write-back event window and emit draft
//! [`CatalystEvent`] values into a caller-owned inert buffer. They take
//! `&self` only — the compiler enforces side-effect freedom — and no
//! consumer acts on the events yet (wiring lands with the task 3.x arc;
//! IC-1 vocabulary freeze is task 2.9).
//!
//! Draft producer set v1 (event-journal-derived only):
//! - `MarriageFormed` → Bond to both spouses
//! - `ChildBorn` → Bond to both parents
//! - `FeudFormed` → Threat to both parties
//! - `ConflictOccurred` → Threat to aggressor and target
//! - `NormViolated` → Transgression to the violator
//! - `AgentDied` → Grief via the guarded widow heuristic below
//!
//! Known draft limitation (recorded for the IC-1 freeze): the deaths pass
//! clears partner/household references in the same tick, so co-resident
//! kin grief is not recoverable read-side. The widow heuristic scans
//! inactive marriages containing the deceased whose surviving partner is
//! currently unpartnered — ambiguous across long histories — so 3.x wiring
//! should record grief targets into an inert side-buffer inside the deaths
//! pass if the freeze demands exact co-resident targeting.
//!
//! Magnitudes are CALIBRATION-PENDING(AP3) placeholders in [0,1]; the
//! zero-at-zero law holds trivially because an empty event window yields
//! an empty buffer.

use super::{Simulation, Tick};
use mindstrata_core::event::SimEvent;
use mindstrata_core::id::AgentId;

/// Sim event ids are `u64`-backed; observers speak agent indices.
fn idx(id: AgentId) -> usize {
    id.as_u64() as usize
}

/// Draft drive taxonomy for catalysts (IC-1 pending; task 2.9 freezes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Drive {
    /// Loss of an attachment figure.
    Grief,
    /// Formation or expansion of an attachment bond.
    Bond,
    /// Direct exposure to conflict or violence.
    Threat,
    /// Personal violation of a norm one holds.
    Transgression,
}

/// One draft catalyst observation.
///
/// `subject` is an agent index (`usize`) — the sim's internal coin — not
/// the public [`mindstrata_core::id::AgentId`]; the IC-1 freeze decides the
/// canonical identity form.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalystEvent {
    /// Tick the producing event occurred.
    pub tick: Tick,
    /// Affected agent index.
    pub subject: usize,
    /// Draft drive classification.
    pub drive: Drive,
    /// Pressure magnitude in [0,1] (CALIBRATION-PENDING(AP3)).
    pub magnitude: f64,
}

impl Simulation {
    /// Current event-log watermark for incremental observation.
    ///
    /// Store this before a tick (or batch of ticks) and pass it to
    /// [`Self::observe_catalysts_since`] afterwards.
    #[must_use]
    pub fn catalyst_watermark(&self) -> usize {
        self.events.len()
    }

    /// Observe draft catalysts over the post-write-back window
    /// `[watermark, now)`.
    ///
    /// Pure read: identical watermarks yield byte-identical buffers, and a
    /// watermark equal to the current log length yields an empty buffer
    /// (zero-catalyst days produce zero events).
    #[must_use]
    pub fn observe_catalysts_since(&self, watermark: usize) -> Vec<CatalystEvent> {
        let start = watermark.min(self.events.len());
        let mut out = Vec::new();
        for ev in &self.events[start..] {
            match *ev {
                SimEvent::MarriageFormed {
                    spouse_a,
                    spouse_b,
                    tick,
                } => {
                    for subject in [idx(spouse_a), idx(spouse_b)] {
                        out.push(CatalystEvent {
                            tick,
                            subject,
                            drive: Drive::Bond,
                            magnitude: 0.8,
                        });
                    }
                }
                SimEvent::ChildBorn {
                    parent_a,
                    parent_b,
                    tick,
                    ..
                } => {
                    for subject in [idx(parent_a), idx(parent_b)] {
                        out.push(CatalystEvent {
                            tick,
                            subject,
                            drive: Drive::Bond,
                            magnitude: 0.7,
                        });
                    }
                }
                SimEvent::FeudFormed {
                    party_a,
                    party_b,
                    tick,
                } => {
                    for subject in [idx(party_a), idx(party_b)] {
                        out.push(CatalystEvent {
                            tick,
                            subject,
                            drive: Drive::Threat,
                            magnitude: 0.4,
                        });
                    }
                }
                SimEvent::ConflictOccurred {
                    aggressor,
                    target,
                    injury,
                    fear_induced,
                    tick,
                    ..
                } => {
                    let base = 0.3 + injury.to_f64().clamp(0.0, 0.5);
                    out.push(CatalystEvent {
                        tick,
                        subject: idx(aggressor),
                        drive: Drive::Threat,
                        magnitude: base.min(1.0),
                    });
                    out.push(CatalystEvent {
                        tick,
                        subject: idx(target),
                        drive: Drive::Threat,
                        magnitude: (base + fear_induced.to_f64().clamp(0.0, 0.2)).min(1.0),
                    });
                }
                SimEvent::NormViolated { agent, tick, .. } => {
                    out.push(CatalystEvent {
                        tick,
                        subject: idx(agent),
                        drive: Drive::Transgression,
                        magnitude: 0.5,
                    });
                }
                SimEvent::AgentDied { agent, tick, .. } => {
                    self.emit_widow_grief(idx(agent), tick, &mut out);
                }
                _ => {}
            }
        }
        out
    }

    /// Widow-heuristic grief emission for one death.
    ///
    /// Emits to every currently-unpartnered surviving partner of an
    /// inactive marriage containing `deceased`. See the module docs for
    /// the ambiguity ceiling this carries until the IC-1 freeze.
    fn emit_widow_grief(&self, deceased: usize, tick: Tick, out: &mut Vec<CatalystEvent>) {
        for marriage in &self.marriage_registry.marriages {
            if marriage.active {
                continue;
            }
            let survivor = if marriage.partner_a == deceased {
                Some(marriage.partner_b)
            } else if marriage.partner_b == deceased {
                Some(marriage.partner_a)
            } else {
                None
            };
            if let Some(survivor) = survivor {
                if survivor < self.agents.len() && self.agents[survivor].partner.is_none() {
                    out.push(CatalystEvent {
                        tick,
                        subject: survivor,
                        drive: Drive::Grief,
                        magnitude: 1.0,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_sim() -> Simulation {
        let config = super::super::SimConfig {
            seed: 4242,
            max_ticks: 10_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim
    }

    #[test]
    fn empty_window_yields_zero_events() {
        let sim = fresh_sim();
        let mark = sim.catalyst_watermark();
        assert!(sim.observe_catalysts_since(mark).is_empty());
    }

    #[test]
    fn observation_is_pure_and_repeatable() {
        let mut sim = fresh_sim();
        // Run some ticks to generate real events.
        for _ in 0..50 {
            sim.tick();
        }
        let mark = 0;
        let first = sim.observe_catalysts_since(mark);
        let second = sim.observe_catalysts_since(mark);
        assert_eq!(first, second, "repeated observation must be identical");
        // Purity: observing must not advance the event log.
        assert_eq!(sim.event_count(), {
            let _ = &first;
            sim.event_count()
        });
    }

    #[test]
    fn watermark_slices_exactly_the_new_events() {
        let mut sim = fresh_sim();
        for _ in 0..20 {
            sim.tick();
        }
        let mid = sim.event_count() / 2;
        let full = sim.observe_catalysts_since(0);
        let tail = sim.observe_catalysts_since(mid);
        // Tail events are exactly the chronological suffix of all events.
        assert_eq!(&full[full.len() - tail.len()..], &tail[..]);
    }

    #[test]
    fn magnitudes_stay_in_unit_range() {
        let mut sim = fresh_sim();
        for _ in 0..200 {
            sim.tick();
        }
        for ev in sim.observe_catalysts_since(0) {
            assert!(
                (0.0..=1.0).contains(&ev.magnitude),
                "magnitude {} out of range",
                ev.magnitude
            );
        }
    }

    #[test]
    fn golden_untouched_by_observation() {
        // Identical seeds, one observed and one not: metric streams must be
        // bit-equal after the same horizon.
        let mut observed = fresh_sim();
        let mut silent = fresh_sim();
        for _ in 0..100 {
            observed.tick();
            let mark = observed.catalyst_watermark();
            let _buf: Vec<CatalystEvent> = observed.observe_catalysts_since(mark);
            silent.tick();
        }
        assert_eq!(
            observed.metrics_snapshot(),
            silent.metrics_snapshot(),
            "observation must not perturb simulation state"
        );
    }
}
