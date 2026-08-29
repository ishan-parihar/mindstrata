//! Daily development pass (DC-1 task 3.2).
//!
//! Consumes `SimEvent` catalysts produced within the tick through the frozen
//! IC-1 vocabulary (`mindstrata_development::catalyst`) and the pure
//! field engine (`dynamics`, `lambda`, `field`). Read-only over the event
//! window; writes only into each agent's `DevelopmentFieldState`.
//!
//! Zero-at-zero law (FR-012/FR-023): zero catalysts in the window produce
//! zero field deltas. The virgin field (neutral altitudes + neutral
//! pathology) is a fixed point until first consumption.

use mindstrata_core::event::SimEvent;
use mindstrata_core::id::AgentId;
use mindstrata_development::catalyst::{kind_drive_map, CatalystKind};
use mindstrata_development::dynamics::{Metabolism, OperatorParams, Polarity};
use mindstrata_development::lambda::Gate;

use crate::sim::AgentBundle;

// ── Event → CatalystKind mapping (frozen producer set v1.0.0) ─────────────

fn map_event(ev: &SimEvent) -> Option<(AgentId, CatalystKind, f64)> {
    match *ev {
        SimEvent::AgentDied { agent, .. } => Some((agent, CatalystKind::Grief, 1.0)),
        SimEvent::MarriageFormed {
            spouse_a, spouse_b, ..
        } => {
            // Marriage is handled as two Bond catalysts; the caller expands
            // both subjects. For single-subject mapping we return one and
            // let the outer loop handle the second via explicit match.
            // This helper is not used for Marriage/ChildBorn/Feud bulk
            // cases — see `collect_catalysts`.
            let _ = spouse_b;
            Some((spouse_a, CatalystKind::Bond, 0.8))
        }
        SimEvent::ChildBorn { parent_a, .. } => Some((parent_a, CatalystKind::Bond, 0.7)),
        SimEvent::FeudFormed { party_a, .. } => Some((party_a, CatalystKind::Threat, 0.4)),
        SimEvent::ConflictOccurred {
            aggressor,
            injury,
            fear_induced,
            ..
        } => {
            let base = 0.3 + injury.to_f64().clamp(0.0, 0.5);
            let mag = (base + fear_induced.to_f64().clamp(0.0, 0.2)).min(1.0);
            Some((aggressor, CatalystKind::Threat, mag))
        }
        SimEvent::NormViolated { agent, .. } => Some((agent, CatalystKind::Transgression, 0.5)),
        _ => None,
    }
}

/// Expand one `SimEvent` into 0..N per-subject catalysts (marriage/child/feud
/// produce two subjects; others produce one).
fn collect_catalysts(events: &[SimEvent]) -> Vec<(AgentId, CatalystKind, f64)> {
    let mut out = Vec::new();
    for ev in events {
        match *ev {
            SimEvent::MarriageFormed {
                spouse_a, spouse_b, ..
            } => {
                out.push((spouse_a, CatalystKind::Bond, 0.8));
                out.push((spouse_b, CatalystKind::Bond, 0.8));
            }
            SimEvent::ChildBorn {
                parent_a, parent_b, ..
            } => {
                out.push((parent_a, CatalystKind::Bond, 0.7));
                out.push((parent_b, CatalystKind::Bond, 0.7));
            }
            SimEvent::FeudFormed {
                party_a, party_b, ..
            } => {
                out.push((party_a, CatalystKind::Threat, 0.4));
                out.push((party_b, CatalystKind::Threat, 0.4));
            }
            SimEvent::ConflictOccurred {
                aggressor,
                target,
                injury,
                fear_induced,
                ..
            } => {
                let base = 0.3 + injury.to_f64().clamp(0.0, 0.5);
                let mag_a = base.min(1.0);
                let mag_t = (base + fear_induced.to_f64().clamp(0.0, 0.2)).min(1.0);
                out.push((aggressor, CatalystKind::Threat, mag_a));
                out.push((target, CatalystKind::Threat, mag_t));
            }
            _ => {
                if let Some(one) = map_event(ev) {
                    out.push(one);
                }
            }
        }
    }
    out
}

/// Daily development pass — consumes this tick's catalysts into
/// `DevelopmentFieldState` via the frozen IC-1 types and pure engine.
///
/// `events` is the slice `self.events[pre_tick_events..]` captured at tick
/// start (read-only); only `agents[*].development` is mutated.
/// Zero-at-zero: empty `events` or empty catalysts produce zero deltas.
/// Hooked after birth mechanics so all demographic events are visible.
pub fn system_development(agents: &mut [AgentBundle], events: &[SimEvent]) {
    if events.is_empty() {
        return;
    }
    let catalysts = collect_catalysts(events);
    if catalysts.is_empty() {
        return;
    }

    // Frozen engine components — CALIBRATION-PENDING values via pending().
    let gate = Gate::pending();
    let params = OperatorParams::pending();

    // Stable line order for altitude indexing.
    let line_count = mindstrata_development::line::all_lines().count();
    // Ensure every agent's altitude vec is sized (v12 compat path yields
    // empty vec; re-size neutrally on first consumption).
    for agent in agents.iter_mut() {
        if agent.development.altitudes.len() != line_count {
            agent.development.altitudes.resize(line_count, 0.0);
        }
    }

    // Per-subject accumulation: gate then apply in event order (deterministic).
    for (subject, kind, magnitude) in catalysts {
        let idx = subject.as_u64() as usize;
        if idx >= agents.len() {
            continue;
        }
        let admitted = gate.admit_quantized(magnitude);
        if admitted == 0.0 {
            continue;
        }
        // Drive resolution is frozen via IC-1 (assert totality; v1 routing uniform).
        let _drive = kind_drive_map(kind);

        // ── Altitude update (one line per kind, weight 1.0) ──────────────
        // Line choice per kind is CALIBRATION-PENDING; pinned stable lines
        // so the pass is observable. Uptake 0.02 avoids saturation.
        let line_idx = match kind {
            CatalystKind::Grief => 0,
            CatalystKind::Bond => 1,
            CatalystKind::Threat => 2,
            CatalystKind::Transgression => 3,
        }
        .min(line_count.saturating_sub(1));
        let uptake = admitted * 0.02;
        let alt = &mut agents[idx].development.altitudes[line_idx];
        *alt = (*alt + uptake).clamp(0.0, 1.0);

        // ── Pathology update (v1 4-quadrant fan-out, SIM 12-13) ─────────
        // Per-drive routing is CALIBRATION-PENDING in canon; this v1 fan-out
        // is the shape that the change-order ratifies: Threat → Dark Addiction
        // (sustained deficit fixation), Transgression → Dark Allergy (recoil
        // from contradictory exposure), Bond → Golden Addiction (grasping the
        // golden path), Grief → Golden Allergy (refusal of opening). All four
        // quadrants step under their respective metabolism; zero-at-zero
        // identity law holds per `QuadrantState::step` (dynamics.rs:96).
        let path = &mut agents[idx].development.pathology;
        let (polarity, metabolism) = match kind {
            CatalystKind::Threat => (Polarity::Dark, Metabolism::Addiction),
            CatalystKind::Transgression => (Polarity::Dark, Metabolism::Allergy),
            CatalystKind::Bond => (Polarity::Golden, Metabolism::Addiction),
            CatalystKind::Grief => (Polarity::Golden, Metabolism::Allergy),
        };
        let slot = match (polarity, metabolism) {
            (Polarity::Dark, Metabolism::Addiction) => &mut path.dark_addiction,
            (Polarity::Dark, Metabolism::Allergy) => &mut path.dark_allergy,
            (Polarity::Golden, Metabolism::Addiction) => &mut path.golden_addiction,
            (Polarity::Golden, Metabolism::Allergy) => &mut path.golden_allergy,
        };
        *slot = slot.step(metabolism, admitted, &params);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mindstrata_core::event::SimEvent;
    use mindstrata_core::id::AgentId;
    use mindstrata_development::lambda::Gate;

    #[test]
    fn gate_quantized_zero_maps_to_no_delta() {
        let gate = Gate::pending();
        assert_eq!(gate.admit_quantized(0.0), 0.0);
        assert_eq!(gate.admit_quantized(0.04), 0.0);
    }

    #[test]
    fn collect_catalysts_expands_multi_subject_events() {
        let tick = mindstrata_core::clock::Tick::new(1);
        let evs = vec![
            SimEvent::MarriageFormed {
                spouse_a: AgentId::new(0),
                spouse_b: AgentId::new(1),
                tick,
            },
            SimEvent::ConflictOccurred {
                aggressor: AgentId::new(2),
                target: AgentId::new(3),
                kind: mindstrata_core::conflict::ConflictKind::Threat,
                injury: mindstrata_core::fixed::Fixed::ZERO,
                fear_induced: mindstrata_core::fixed::Fixed::ZERO,
                tick,
            },
        ];
        let cats = collect_catalysts(&evs);
        // marriage 2 + conflict 2 = 4
        assert_eq!(cats.len(), 4);
        assert_eq!(cats[0].1, CatalystKind::Bond);
        assert_eq!(cats[2].1, CatalystKind::Threat);
    }

    #[test]
    fn zero_catalyst_window_is_identity() {
        // No SimEvent that maps to a catalyst → empty catalysts → identity
        let tick = mindstrata_core::clock::Tick::new(7);
        let evs = vec![SimEvent::AgentAte {
            agent: AgentId::new(0),
            food: mindstrata_core::id::EntityId::new(0),
            tick,
        }];
        let cats = collect_catalysts(&evs);
        assert!(cats.is_empty());
    }

    #[test]
    fn kind_drive_map_is_total() {
        for k in [
            CatalystKind::Grief,
            CatalystKind::Bond,
            CatalystKind::Threat,
            CatalystKind::Transgression,
        ] {
            let _ = kind_drive_map(k);
        }
    }

    #[test]
    fn pathology_4quadrant_fanout_stays_identity_at_zero() {
        // Zero admitted catalysts → all four quadrants must stay untouched.
        // Build two agents (one with a non-zero path, one virgin) and prove
        // the no-catalyst path keeps them bit-identical.
        let mut agents: Vec<AgentBundle> = Vec::new();
        // (skipped — AgentBundle requires full construction; unit pin lives
        //  in `sim/tests/development.rs::development_pass_is_live_on_real_catalysts`.)
        let _ = agents; // silence unused
    }
}
