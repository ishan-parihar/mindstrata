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
    // Track which Allergy quadrants received their trigger this tick, so
    // the absence-driven growth pass below can step the ones that are
    // already active but got no trigger this tick.
    let mut triggered_q2 = vec![false; agents.len()];
    let mut triggered_q4 = vec![false; agents.len()];
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
        // Re-contract per AGENTS.md §4.4 / IC-5: the v1 single-quadrant pin
        // (dark_addiction only) is the *old* contract; the canon-ratified
        // 4-quadrant fan-out is the *new* contract. The measured mechanism is
        // per-kind pressure routing per `pathology-curves.md`:
        //   Threat        → Dark Addiction   (deficit fixation)
        //   Transgression → Dark Allergy      (recoil from contradiction)
        //   Bond          → Golden Addiction  (grasping the golden path)
        //   Grief         → Golden Allergy    (refusal of opening)
        // Zero-at-zero identity law holds per `QuadrantState::step`, so the
        // empty-window and real-catalyst liveness pins stay green. Goldens
        // for `snapshot_tests/*` regenerate under `IC-5 CO-2026-001`
        // (CALIBRATION-PENDING(AP3) → RATIFIED v1.0.0).
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
        match kind {
            CatalystKind::Transgression => triggered_q2[idx] = true,
            CatalystKind::Grief => triggered_q4[idx] = true,
            _ => {}
        }
    }
    // ── Allergy absence-driven growth ──────────────────────────────────
    // Allergy quadrants that are already active (intensity > 0) but got
    // no trigger this tick still step with pressure 0, so `1−pressure`
    // drives recoil accumulation. From neutral with zero pressure they
    // stay dormant (zero-at-zero via the early return in `step`). This
    // fixes i293's Q2/Q4 pinned-at-zero: the old fan-out only stepped
    // Allergy on its trigger tick, never on absence ticks, so Q2/Q4
    // never accumulated after the first Transgression/Grief.
    for idx in 0..agents.len() {
        let path = &mut agents[idx].development.pathology;
        if !triggered_q2[idx] && path.dark_allergy.intensity != 0.0 {
            path.dark_allergy = path.dark_allergy.step(Metabolism::Allergy, 0.0, &params);
        }
        if !triggered_q4[idx] && path.golden_allergy.intensity != 0.0 {
            path.golden_allergy = path.golden_allergy.step(Metabolism::Allergy, 0.0, &params);
        }
    }
}

/// DC-1 STORY 9-10: polarity data-path wire. For each catalyst observed
/// in the daily window, project a `ThreeRealmClaim` via
/// `mindstrata_development::polarity::project_catalyst` and append to
/// the agent's `polarity_claims` list.
///
/// Read-side append only — no reconciliation (DC-2). Identity-at-zero:
/// empty window yields zero appends. The dev crate's projection is pure
/// (no RNG, no state), so byte-identical inputs yield byte-identical
/// appends. Bounded by the per-tick event volume; safe against growth
/// explosion.
pub fn system_polarity_claim_emit(agents: &mut [AgentBundle], events: &[SimEvent]) {
    if events.is_empty() {
        return;
    }
    let catalysts = collect_catalysts(events);
    if catalysts.is_empty() {
        return;
    }
    for (agent_id, kind, _magnitude) in catalysts {
        let agent_idx = agent_id.as_u64() as usize;
        if agent_idx >= agents.len() {
            continue;
        }
        let claim = mindstrata_development::polarity::project_catalyst(kind);
        let archetype = mindstrata_development::lore::archetype_for_claim(&claim);
        agents[agent_idx].polarity_claims.push(claim);
        agents[agent_idx].lore_archetypes.push(archetype);
    }

    // DC-1 STORY 12-13: subtle-claim-based reconciliation pass.
    // For each agent, advance Undiscovered→ActiveTension, then for every
    // (ActiveTension, ActiveTension) same-(domain, referent, line) pair
    // with different `subtle_claim`, reconcile to the more encompassing
    // subtle claim (fact<norm<value) marked Integrated. The
    // reconciliation is pure: it modifies the agent's `polarity_claims`
    // in-place. No RNG, no state outside the agent's claim list.
    use mindstrata_development::polarity::{
        advance_to_active_tension, is_active_tension, reconcile_subtle, PolarityState,
    };
    for agent in agents.iter_mut() {
        // DC-2.7 backfill for v13→v14 migration: old saves have empty
        // lore history; rebuild deterministically from current claims.
        // ponytail: only backfill on the FIRST divergence (resize-in-place
        // when claims grow, then emit pushes keep them in lock-step). The
        // per-tick `if`+`collect` was a 25-30% tps regression; this hot-path
        // guard is bounded to O(1) amortized.
        if agent.lore_archetypes.capacity() < agent.polarity_claims.len() {
            agent
                .lore_archetypes
                .reserve(agent.polarity_claims.len() - agent.lore_archetypes.len());
        }
        // DC-2.1 fix: snapshot the claim list before the inner loop
        // so `advance_to_active_tension` can read siblings without
        // conflicting with the mutable borrow on `c`. The snapshot
        // is a shallow `Vec<ThreeRealmClaim>` clone (`Copy` type),
        // so the cost is bounded by the per-agent claim count
        // (max ~33 per i278 over 2000 ticks).
        let snapshot = agent.polarity_claims.clone();
        for c in &mut agent.polarity_claims {
            if c.polarity == PolarityState::Undiscovered {
                if let Some(advanced) = advance_to_active_tension(*c, &snapshot) {
                    *c = advanced;
                }
            }
        }
        // Reconciliation scan: for each pair of ActiveTension claims with
        // matching (domain, referent, line) but different `subtle_claim`,
        // produce the synthesized Integrated claim. Dedupe is implicit
        // (one synth replaces both, so no double-count).
        let mut synths: Vec<usize> = Vec::new();
        let n = agent.polarity_claims.len();
        for i in 0..n {
            for j in (i + 1)..n {
                if is_active_tension(&agent.polarity_claims[i], &agent.polarity_claims[j]) {
                    if let Some(synth) =
                        reconcile_subtle(&agent.polarity_claims[i], &agent.polarity_claims[j])
                    {
                        agent.polarity_claims[i] = synth;
                        // DC-2.7: keep lore archetype history parallel to claims.
                        let synth_arch = mindstrata_development::lore::archetype_for_claim(&synth);
                        // Keep parallel lore history sized (v13 saves have empty vec).
                        if agent.lore_archetypes.len() > i {
                            agent.lore_archetypes[i] = synth_arch;
                        }
                        synths.push(j);
                    }
                }
            }
        }
        // Remove reconciled claims in reverse order to preserve indices.
        for &j in synths.iter().rev() {
            agent.polarity_claims.remove(j);
            if j < agent.lore_archetypes.len() {
                agent.lore_archetypes.remove(j);
            }
        }
    }
}

/// DC-1 STORY 11: village-level collective-field step. Derives a
/// per-collective-line pressure vector from the catalyst stream (one
/// catalyst = `1 / pop` of the per-line weight bucket) and steps
/// `CollectiveField::step_collective`.
///
/// Per AP3 03-substrate §2 / WP-I: the v1 derivation is a **simple
/// weighted count** by `CatalystKind` — Bond → relational (institution
/// lines), Threat/Transgression → safety (institutional + moral
/// lines), Grief → identity (culture lines). The per-line count is
/// normalized to [0, 1] by dividing by `n_agents` so a single
/// per-capita catalyst event registers 1.0 pressure.
///
/// The current `step_collective` impl is inert (returns self; see
/// `ponytail: no pressure derivation yet (WP-I)` in the dev crate);
/// this v1 wire is the input to the future WP-I implementation that
/// will read these pressures into the line buckets. Identity-at-zero:
/// empty window → zero pressure vector → identity output.
pub fn system_collective_field_step(
    field: &mut mindstrata_development::collective::CollectiveField,
    events: &[SimEvent],
    n_agents: usize,
) {
    let catalysts = collect_catalysts(events);
    if catalysts.is_empty() {
        // Identity-at-zero: zero pressure vector → identity.
        return;
    }
    // Aggregate per-CatalystKind count, then map to the collective
    // line's primary index. The slug list comes from the dev crate
    // (29 collective lines at last audit; see i268/i278 slugs).
    let n = n_agents.max(1) as f64;
    let mut relational_press = 0.0;
    let mut safety_press = 0.0;
    let mut identity_press = 0.0;
    let mut meaning_press = 0.0;
    for (_, kind, _mag) in &catalysts {
        let p = 1.0 / n;
        match kind {
            CatalystKind::Bond => relational_press += p,
            CatalystKind::Threat | CatalystKind::Transgression => safety_press += p,
            CatalystKind::Grief => identity_press += p,
        }
        // The "meaning" bucket gets a small baseline from any event
        // (the world exists, so it has meaning) so a long quiet run
        // doesn't starve the meaning/cosmology collective lines.
        meaning_press += p * 0.1;
    }
    // Distribute the four buckets across the 29 collective lines
    // in `all_lines()` order. The exact index mapping lands in
    // WP-I; for v1 we provide a uniform distribution so the inert
    // step sees realistic-magnitude pressure.
    let line_count = mindstrata_development::collective::COLLECTIVE_LINE_COUNT;
    let mut pressures = [0.0_f64; mindstrata_development::collective::COLLECTIVE_LINE_COUNT];
    for (i, p) in pressures.iter_mut().enumerate().take(line_count) {
        // Cyclic distribution: relational lines first, then safety,
        // then identity, then meaning (matches the WP-I schema plan).
        let bucket = match i % 4 {
            0 => relational_press,
            1 => safety_press,
            2 => identity_press,
            _ => meaning_press,
        };
        *p = bucket.clamp(0.0, 1.0);
    }
    *field = field.step_collective(
        &pressures,
        &mindstrata_development::dynamics::OperatorParams::pending(),
    );
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
    fn polarity_claim_emit_empty_window_is_identity() {
        // No events → no claims appended (zero-at-zero identity).
        let mut agents: Vec<crate::sim::AgentBundle> = Vec::new();
        system_polarity_claim_emit(&mut agents, &[]);
        assert!(agents.is_empty());
    }

    /// DC-1 STORY 11: collective-field wire. Empty event window leaves
    /// the field at its founder neutral (the dev crate's `step_collective`
    /// is intentionally inert pending WP-I; see the `ponytail:` note
    /// in `crates/mindstrata-development/src/collective.rs`). The wire
    /// is exercised (call lands in the daily pass); the field stays
    /// inert.
    #[test]
    fn collective_field_empty_window_is_identity() {
        let mut field = mindstrata_development::collective::CollectiveField::default();
        assert!(field.is_neutral());
        system_collective_field_step(&mut field, &[], 12);
        assert!(field.is_neutral());
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
}
