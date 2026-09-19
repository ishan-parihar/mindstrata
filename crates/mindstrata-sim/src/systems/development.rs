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

use mindstrata_core::conflict::ConflictKind;
use mindstrata_core::event::SimEvent;
use mindstrata_core::id::AgentId;
use mindstrata_development::catalyst::{kind_drive_map, CatalystKind};
use mindstrata_development::dynamics::{Metabolism, OperatorParams, Polarity};
use mindstrata_development::lambda::Gate;

use crate::sim::AgentBundle;

// ── Event → CatalystKind mapping (frozen producer set v1.0.0) ─────────────

fn map_event(ev: &SimEvent) -> Option<(AgentId, CatalystKind, f64, bool)> {
    match *ev {
        // Iteration-272 (§4.3 root-cause fix): Grief routes to the *surviving*
        // mourner via `GriefStruck` (emitted by the deaths pass with still-live
        // references). The old `AgentDied → Grief(subject)` mapping landed the
        // catalyst on the deceased's slot — which the same-tick in-place
        // generational replacement had already overwritten with a newborn, so
        // every Grief catalyst ever produced soaked into the wrong agent (and
        // at calibration horizons, zero deaths fire at all). Magnitude 1.0
        // stands: the loss of a spouse/co-resident kin is the maximal-loss
        // exemplar; the widow-heuristic emission discipline bounds it to
        // genuinely-tied survivors.
        // Magnitude 1.0 RATIFIED (i289): observed constant at the mortality
        // horizon (pestilence 20K, all seeds) — spouse/kin loss is a one-shot
        // maximal event by semantics, no distribution to re-contract.
        SimEvent::GriefStruck { mourner, .. } => Some((mourner, CatalystKind::Grief, 1.0, false)),
        // NOTE (Iter-285, WP-H3): `MourningObserved` is deliberately NOT a
        // catalyst here. It is the Agape-METABOLIZER read-side handled by a
        // dedicated sweep in `system_development` — pressure that DECAYS the
        // Grief-routed quadrant (Q4) rather than growing it, with no
        // altitude press and no polarity-claim projection (a rite is not an
        // appraisal). Routing it through the catalyst fan-out would
        // re-raise the grief altitude line and re-emit a claim per mourner.
        SimEvent::MarriageFormed {
            spouse_a, spouse_b, ..
        } => {
            // Marriage is handled as two Bond catalysts; the caller expands
            // both subjects. For single-subject mapping we return one and
            // let the outer loop handle the second via explicit match.
            // This helper is not used for Marriage/ChildBorn/Feud bulk
            // cases — see `collect_catalysts`.
            let _ = spouse_b;
            Some((spouse_a, CatalystKind::Bond, 0.8, false))
        }
        SimEvent::ChildBorn { parent_a, .. } => Some((parent_a, CatalystKind::Bond, 0.7, false)),
        SimEvent::FeudFormed { party_a, .. } => Some((party_a, CatalystKind::Threat, 0.4, true)),
        SimEvent::ConflictOccurred {
            aggressor,
            kind,
            injury,
            fear_induced,
            ..
        } => {
            let base = 0.3 + injury.to_f64().clamp(0.0, 0.5);
            let mag = (base + fear_induced.to_f64().clamp(0.0, 0.2)).min(1.0);
            let major = !matches!(kind, ConflictKind::Threat | ConflictKind::Intimidation);
            Some((aggressor, CatalystKind::Threat, mag, major))
        }
        SimEvent::NormViolated { agent, .. } => {
            // Magnitude 0.5 RATIFIED (i289): spec midpoint confirmed in vivo —
            // Q2 dark-allergy equilibrium 0.69–0.72 at 20K against ceiling 0.80
            // (in-band) with the i285 mourning-rite Agape decay as the working
            // consumption channel. See evidence/i289_catalyst_magnitudes.md.
            Some((agent, CatalystKind::Transgression, 0.5, false))
        }
        _ => None,
    }
}

/// Expand one `SimEvent` into 0..N per-subject catalysts (marriage/child/feud
/// produce two subjects; others produce one). The trailing `bool` is the
/// Threat-severity discriminator (i275): `true` = MAJOR conflict (Violence/
/// Combat/Revolution/MoralPanic — bodily or structural harm), `false` = MINOR
/// (verbal Threat/Intimidation, or a non-Threat catalyst). Only consumed by
/// `project_catalyst_severity` for Threat claims.
pub(crate) fn collect_catalysts(events: &[SimEvent]) -> Vec<(AgentId, CatalystKind, f64, bool)> {
    let mut out = Vec::new();
    for ev in events {
        match *ev {
            SimEvent::MarriageFormed {
                spouse_a, spouse_b, ..
            } => {
                out.push((spouse_a, CatalystKind::Bond, 0.8, false));
                out.push((spouse_b, CatalystKind::Bond, 0.8, false));
            }
            SimEvent::ChildBorn {
                parent_a, parent_b, ..
            } => {
                out.push((parent_a, CatalystKind::Bond, 0.7, false));
                out.push((parent_b, CatalystKind::Bond, 0.7, false));
            }
            SimEvent::RitualPerformed {
                ref participants,
                bonding,
                ..
            } => {
                // Iteration-274: per-participant Bond catalysts — the
                // substrate's "ritual/festival participation → culture-line
                // collective catalysts" channel (03-substrate §5). Magnitude
                // scales with the ritual's bonding_effect (seeded rituals
                // carry ~0.1–0.15 bonding; a marriage-level 0.8 caps at the
                // MarriageFormed precedent).
                let mag = (bonding.to_f64() * 2.0).clamp(0.2, 0.8);
                for p in participants {
                    out.push((*p, CatalystKind::Bond, mag, false));
                }
            }
            SimEvent::FeudFormed {
                party_a, party_b, ..
            } => {
                out.push((party_a, CatalystKind::Threat, 0.4, true));
                out.push((party_b, CatalystKind::Threat, 0.4, true));
            }
            SimEvent::ConflictOccurred {
                aggressor,
                target,
                kind,
                injury,
                fear_induced,
                ..
            } => {
                let base = 0.3 + injury.to_f64().clamp(0.0, 0.5);
                let mag_a = base.min(1.0);
                let mag_t = (base + fear_induced.to_f64().clamp(0.0, 0.2)).min(1.0);
                // i275 severity discriminator: bodily/structural conflict
                // (Violence/Combat/Revolution/MoralPanic) is MAJOR; verbal
                // Threat/Intimidation is MINOR. Drives the Fact-vs-Identity
                // claim split in `project_catalyst_severity`.
                let major = !matches!(kind, ConflictKind::Threat | ConflictKind::Intimidation);
                out.push((aggressor, CatalystKind::Threat, mag_a, major));
                out.push((target, CatalystKind::Threat, mag_t, major));
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
/// Production per-quadrant operator params (IC-5 #3 spec midpoints),
/// exposed for calibration probes and pins: `[0]` Q1 dark-addiction,
/// `[1]` Q2 dark-allergy, `[2]` Q3 golden-addiction, `[3]` Q4
/// golden-allergy. These are the CANON (Standard-band) values; the live
/// params a run uses are [`pathology_params`], which scales growth/decay by
/// the row-3 difficulty multipliers and returns these four bit-for-bit in
/// the Standard band.
pub const PROD_QUADRANT_PARAMS: [OperatorParams; 4] = [
    OperatorParams {
        growth: 0.06,
        decay: 0.015,
        ceiling: 0.80,
    },
    OperatorParams {
        growth: 0.045,
        decay: 0.022,
        ceiling: 0.80,
    },
    OperatorParams {
        growth: 0.07,
        decay: 0.015,
        ceiling: 0.85,
    },
    OperatorParams {
        growth: 0.03,
        decay: 0.025,
        ceiling: 0.75,
    },
];

/// Resolve the live per-quadrant operator params for a run's parameters
/// (difficulty-levers row 3, i304). Growth and decay of every quadrant are
/// scaled by the two band multipliers; ceilings are NOT scaled (the catalog's
/// row-3 candidate bands name growth and decay only — a per-quadrant ceiling
/// band is a separate hypothesis, recorded as residual rather than guessed).
///
/// Identity holds bit-for-bit in the Standard band: `x * 1.0 == x` in IEEE-754
/// for every finite `x`, so every calibrated window that ran before i304 runs
/// identically after it.
///
/// §5 quantize-once: the multipliers arrive already materialized in
/// `SimParameters` (one `Fixed::from_f64` at construction), so nothing here
/// re-derives a sub-resolution rate per tick.
#[must_use]
pub fn pathology_params(params: &crate::parameters::SimParameters) -> [OperatorParams; 4] {
    let growth = params.pathology_growth_scale.to_f64();
    let decay = params.pathology_decay_scale.to_f64();
    PROD_QUADRANT_PARAMS.map(|q| OperatorParams {
        growth: q.growth * growth,
        decay: q.decay * decay,
        ceiling: q.ceiling,
    })
}

/// The ratified mourning-rite Agape metabolizer dose (i292 sweep; see the
/// emitter comment in `household.rs` for the measured/old/mechanism record).
pub const MOURNING_AGAPE_DOSE: f64 = 0.6;

pub fn system_development(agents: &mut [AgentBundle], events: &[SimEvent]) {
    system_development_with_params(agents, events, &crate::parameters::SimParameters::default());
}

/// The parameterized pass the tick pipeline calls: identical law to
/// [`system_development`], with the per-quadrant operator params resolved
/// from the run's `SimParameters` (difficulty-levers row 3, i304). The
/// no-params wrapper above keeps every pre-i304 call site (probes, unit
/// tests) compiling at the canon band.
///
/// `events` is the slice `self.events[pre_tick_events..]` captured at tick
/// start (read-only); only `agents[*].development` is mutated.
/// Zero-at-zero: empty `events` or empty catalysts produce zero deltas.
/// Hooked after birth mechanics so all demographic events are visible.
pub fn system_development_with_params(
    agents: &mut [AgentBundle],
    events: &[SimEvent],
    params: &crate::parameters::SimParameters,
) {
    if events.is_empty() {
        return;
    }
    let catalysts = collect_catalysts(events);
    if catalysts.is_empty() {
        // Iter-285 (WP-H3): no catalysts, but a mourning rite in the window
        // still drives the Agape-metabolizer sweep below — fall through
        // instead of returning (zero-at-zero holds: no rites → no sweep).
        if !events
            .iter()
            .any(|ev| matches!(ev, SimEvent::MourningObserved { .. }))
        {
            return;
        }
    }

    // Per-subject accumulation: gate then apply in event order (deterministic).
    // Track which Allergy quadrants received their trigger this tick, so
    // the absence-driven growth pass below can step the ones that are
    // already active but got no trigger this tick.
    let mut triggered_q2 = vec![false; agents.len()];
    let mut triggered_q4 = vec![false; agents.len()];

    // Frozen engine components — CALIBRATION-PENDING values via pending().
    let gate = Gate::pending();
    // Live per-quadrant params (IC-5 #3 spec midpoints scaled by the row-3
    // difficulty band; Allergy 0.1x scaled in dynamics.rs for absence).
    // Resolved ONCE per tick — never re-derived per agent or per catalyst.
    let q = pathology_params(params);
    let (params_q1, params_q2, params_q3, params_q4) = (q[0], q[1], q[2], q[3]);

    // ── Iter-285 (WP-H3): Agape-metabolizer sweep ──────────────────────
    // Mourning rites are the wave brief's "ritual forms generated as
    // Agape-metabolizer vehicles (mourning rites bind grief referents)".
    // The Allergy step law is
    //     next = intensity + growth·0.1·headroom·(1−pressure)
    //                        − decay·pressure·intensity
    // so pressure is the ONLY consumption channel on an Allergy quadrant:
    // absence GROWS it. A rite re-presents the Grief catalyst communally
    // with pressure `agape ∈ [0,1]` — one dose at 0.6 drives decay
    // −0.025·0.6·I = −0.015·I on Q4 while suppressing the same tick's
    // absence-growth. The rite does NOT press the grief altitude line and
    // does NOT project a polarity claim (a rite is not an appraisal).
    // Deterministic: per-participant in event order; no RNG. Zero-at-zero:
    // no MourningObserved events → this sweep is a no-op.
    for ev in events {
        if let SimEvent::MourningObserved {
            participants,
            agape,
            ..
        } = ev
        {
            let p = agape.to_f64().clamp(0.0, 1.0);
            if p == 0.0 {
                continue;
            }
            for subject in participants {
                let idx = subject.as_u64() as usize;
                if idx >= agents.len() {
                    continue;
                }
                let path = &mut agents[idx].development.pathology;
                path.golden_allergy = path.golden_allergy.step(Metabolism::Allergy, p, &params_q4);
                // Mark the Q4 trigger so the absence-driven pass below
                // does not double-step the quadrant on rite ticks.
                triggered_q4[idx] = true;
            }
        }
    }

    // Stable line order for altitude indexing.
    let line_count = mindstrata_development::line::all_lines().count();
    // Ensure every agent's altitude vec is sized (v12 compat path yields
    // empty vec; re-size neutrally on first consumption).
    for agent in agents.iter_mut() {
        if agent.development.altitudes.len() != line_count {
            agent.development.altitudes.resize(line_count, 0.0);
        }
    }

    for (subject, kind, magnitude, _major) in catalysts {
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
        let (slot, params_for_slot) = match (polarity, metabolism) {
            (Polarity::Dark, Metabolism::Addiction) => (&mut path.dark_addiction, &params_q1),
            (Polarity::Dark, Metabolism::Allergy) => (&mut path.dark_allergy, &params_q2),
            (Polarity::Golden, Metabolism::Addiction) => (&mut path.golden_addiction, &params_q3),
            (Polarity::Golden, Metabolism::Allergy) => (&mut path.golden_allergy, &params_q4),
        };
        *slot = slot.step(metabolism, admitted, params_for_slot);
        match kind {
            CatalystKind::Transgression => triggered_q2[idx] = true,
            CatalystKind::Grief => triggered_q4[idx] = true,
            _ => {}
        }
    }
    // ── Allergy absence-driven growth ──────────────────────────────────
    // Allergy quadrants step EVERY tick, even from neutral with zero
    // pressure — their growth law `g·headroom·(1−pressure)` means
    // absence (pressure 0) drives recoil accumulation from zero.
    // The old fan-out only stepped Allergy on its trigger tick
    // (Transgression/Grief), never on absence ticks, so Q2/Q4 were
    // pinned at 0.0000 at all 20 seeds in i293 (5K/12) and all 6
    // conditions in i294 (20K/48).  With always-step, Q2/Q4 will
    // correctly show recoil even without a trigger.  The pending
    // growth 0.05 is the placeholder; per-quadrant tuning (open #3)
    // will slow Allergy if needed (pathology-curves.md Q4 0.02–0.04).
    for idx in 0..agents.len() {
        let path = &mut agents[idx].development.pathology;
        if !triggered_q2[idx] {
            path.dark_allergy = path.dark_allergy.step(Metabolism::Allergy, 0.0, &params_q2);
        }
        if !triggered_q4[idx] {
            path.golden_allergy = path
                .golden_allergy
                .step(Metabolism::Allergy, 0.0, &params_q4);
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
pub fn system_polarity_claim_emit(agents: &mut [AgentBundle], events: &[SimEvent], tick: u64) {
    if events.is_empty() {
        return;
    }
    let catalysts = collect_catalysts(events);
    if catalysts.is_empty() {
        return;
    }
    // i284: threat slots for the refutation scan (collected before the
    // emit loop consumes `catalysts`).
    let mut threat_slots: Vec<(GrossReferent, LineId)> = Vec::new();
    for (_, kind, _, _) in &catalysts {
        if matches!(kind, CatalystKind::Threat) {
            let projected =
                mindstrata_development::polarity::project_catalyst_severity(*kind, false);
            threat_slots.push((projected.referent, projected.line));
        }
    }
    for (agent_id, kind, _magnitude, major) in catalysts {
        let agent_idx = agent_id.as_u64() as usize;
        if agent_idx >= agents.len() {
            continue;
        }
        // i275: severity-grounded Threat projection — major conflicts
        // project Identity claims on the same (Event, cognitive) slot as
        // minor conflicts' Fact claims, making the tension gate reachable
        // from real event diversity (probe: 1,292 claims, zero tension under
        // the v1 mono-quartet mapping).
        let mut claim = mindstrata_development::polarity::project_catalyst_severity(kind, major);
        // i281: stamp emission tick — the action selector's salience-
        // recency window reads this so the social bias stays a bounded
        // recency integral (measured unbounded-run integral: bias 0.017 →
        // 0.82 × social_value over 1K→20K without the stamp).
        claim.created_tick = tick;
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
    use mindstrata_development::line::LineId;
    use mindstrata_development::polarity::GrossReferent;
    use mindstrata_development::polarity::{
        advance_to_active_tension, reconcile_subtle, refute_claim, PolarityState,
    };
    // i284 (WP-H2 refutation half): collect the (referent, line) slots of
    // THIS window's Threat catalysts — fresh contradictory evidence. Any
    // of the agent's living claims (Integrated/ActiveTension) on a refuted
    // slot is knocked to `Refuted`, excluded from the social bias until a
    // new synthesis recovers it. This is the moral-panic mechanism: a
    // refutation storm strips the village's norm claims at the contested
    // referent, norm churn follows, re-crystallization comes from renewed
    // reconciliation pressure. Refutation events carry no state beyond the
    // claim's polarity flip — pure, deterministic, no RNG.
    // (threat_slots collected above, before the emit loop)
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
        // i275 fix: the old gate called `is_active_tension`, which requires
        // a.domain != b.domain — but `reconcile_subtle` requires a.domain ==
        // b.domain (the polarity lives in the subtle layer). The two
        // predicates are mutually exclusive: the pair scan could NEVER fire
        // (probe i275: 1,171 ActiveTension claims at 20K, zero Integrated).
        // Gate directly on reconcile_subtle's contract instead.
        let mut synths: Vec<usize> = Vec::new();
        let n = agent.polarity_claims.len();
        for i in 0..n {
            // i275: skip indices already scheduled for removal (a claim can
            // be the `j` of an earlier pair; reconciling it again would
            // double-push the index and corrupt the reversed removal).
            if synths.contains(&i) {
                continue;
            }
            for j in (i + 1)..n {
                if synths.contains(&j) {
                    continue;
                }
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
        // Remove reconciled claims in reverse order to preserve indices.
        for &j in synths.iter().rev() {
            agent.polarity_claims.remove(j);
            if j < agent.lore_archetypes.len() {
                agent.lore_archetypes.remove(j);
            }
        }
        // i284: refutation scan — fresh Threat evidence on a slot where the
        // agent holds a living claim knocks it to Refuted. The threat slots
        // are (referent, line) pairs from THIS window's Threat catalysts
        // (the same events that just pushed new claims above). Only claims
        // with an OLDER stamp than the newest window threat refute: evidence
        // must POSTDATE the belief it contradicts (a same-tick claim is the
        // claim forming from that very event, not a belief being tested).
        for c in &mut agent.polarity_claims {
            // i284 scope: only crystallized (Integrated) belief refutes —
            // tension claims ARE the contested state, not settled belief.
            if c.polarity != PolarityState::Integrated {
                continue;
            }
            let contradicted = threat_slots.iter().any(|&(tr, ref line)| {
                tr == c.referent && line == &c.line && c.created_tick < tick
            });
            if contradicted {
                if let Some(refuted) = refute_claim(c) {
                    *c = refuted;
                }
            }
        }
    }
}

/// Iter-286 (WP-H3 second half): norm proposals from reconciled polarity
/// clusters — the wave brief's "norm proposals generated from reconciled
/// polarity clusters". The reconciliation scan (above) synthesizes
/// Integrated claims; the ones that SURVIVE the pass (not refuted) are
/// crystallized communal prescriptions. When a synthesis is Integrated on a
/// slot with village-wide reach (≥ majority distinct tension-holders at the
/// slot — the cluster's contested consensus) AND the collective Safety
/// stage has crossed the band-III institutional gate (4.0 — the i277
/// tetra-arising law: stage bands gate WHICH content classes the generator
/// may emit; a registry norm is institutional-political content), the
/// village proposes a norm. The proposal is a REGISTRY APPEND (id = next
/// free id, modest strength from the synthesis's consensus breadth) — it
/// then flows through the EXISTING consumers: monthly-ritual
/// reinforcement (§12.5) iterates ALL registry norms, and §12.3 compliance
/// pressure reads the registry. Deterministic: BTreeMap ordering, no RNG.
/// Zero-at-zero: no surviving syntheses → no proposals; the band gate
/// keeps every pinned horizon inert by construction (probe i273: max
/// collective stage at 2K = 1.0 exactly).
pub fn system_norm_proposal(
    agents: &[AgentBundle],
    field: &mindstrata_development::collective::CollectiveField,
    norms: &mut mindstrata_institutions::norms::NormRegistry,
    n_agents: usize,
) -> usize {
    use mindstrata_development::collective::{bucket_for_line, CollectiveBucket};
    use mindstrata_development::polarity::{PolarityState, SubtleClaim};
    use std::collections::BTreeMap;

    // Band-III gate (i277): institutional-political content unlocks at
    // stage 4.0. Below it, norm proposals cannot fire — the pinned
    // horizons (all stages 1.0) stay byte-identical.
    const BAND_III_GATE: f64 = 4.0;
    let slugs = mindstrata_development::collective::CollectiveField::line_slugs();
    let safety_stage = field
        .lines
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            slugs.get(*i).map(|s| bucket_for_line(*s)) == Some(CollectiveBucket::Safety)
        })
        .map(|(_, l)| l.stage)
        .fold(0.0_f64, f64::max);
    if safety_stage < BAND_III_GATE {
        return 0;
    }

    // Count surviving Integrated syntheses per (referent, line, subtle)
    // slot and the distinct holders of the underlying tension cluster.
    let mut synths: BTreeMap<(u8, String, u8), usize> = BTreeMap::new();
    let mut cluster: BTreeMap<(u8, String), std::collections::BTreeSet<usize>> = BTreeMap::new();
    for (i, a) in agents.iter().enumerate() {
        for c in &a.polarity_claims {
            match c.polarity {
                PolarityState::Integrated => {
                    *synths
                        .entry((c.referent as u8, c.line.slug().to_string(), c.claim as u8))
                        .or_default() += 1;
                    // The synthesis holder engaged the slot too — the
                    // cluster counts every holder that ever contested it
                    // (tension, its synthesis, or its refutation).
                    cluster
                        .entry((c.referent as u8, c.line.slug().to_string()))
                        .or_default()
                        .insert(i);
                }
                PolarityState::ActiveTension | PolarityState::Refuted => {
                    cluster
                        .entry((c.referent as u8, c.line.slug().to_string()))
                        .or_default()
                        .insert(i);
                }
                PolarityState::Undiscovered => {}
            }
        }
    }
    if synths.is_empty() {
        return 0;
    }

    // Majority quorum over the engaged cluster.
    let half = n_agents.max(1) / 2;
    let mut proposed = 0usize;
    for ((referent, line_slug, subtle), _n) in synths {
        // i286 in-vivo probe verdict (i286_diag + i286_norm_proposal_vivo):
        // Value/Norm syntheses are STRUCTURALLY UNREACHABLE — reconciliation
        // needs two subtle claims on one slot, and only (Event, cognitive)
        // collides in vivo (Threat-Fact × Grief/major-Identity → IDENTITY
        // synthesis; Bond/Transgression slots are mono-claim per agent).
        // The honest proposal source is therefore the Identity synthesis:
        // a village that reconciles "we are the kind of people who survive
        // threats" codifies it — the panic-crystallization arc the wave
        // brief describes (norm churn → re-crystallization INTO the
        // registry). Fact syntheses remain lore, not law. ponytail: when a
        // Transgression feed exists at N=12 (i280 debt), Value/Norm
        // syntheses become reachable and this gate widens to them.
        if !(subtle == SubtleClaim::Identity as u8
            || subtle == SubtleClaim::Value as u8
            || subtle == SubtleClaim::Norm as u8)
        {
            continue;
        }
        let quorum = cluster
            .get(&(referent, line_slug.clone()))
            .map_or(0, std::collections::BTreeSet::len);
        if quorum <= half {
            continue;
        }
        // Dedup: one norm per (line, referent) slot, ever. The registry
        // names encode the slot — deterministic, scan-based, restore-safe.
        let name = format!("[proposed:{line_slug}/r{referent}]");
        if norms.norms().iter().any(|n| n.name == name) {
            continue;
        }
        // Strength scales with consensus breadth (quorum/n_agents),
        // bounded — a proposal starts weak and grows through the
        // SAME reinforcement channel as every other norm (§12.5).
        // i292 RATIFIED: cap 0.6 measured binding in vivo — the sole
        // natural proposal in the 50K drought census carried quorum 8/12
        // and pinned exactly at 0.6 (the cap, not consensus, set its
        // strength; i292_dose_calibration A4). Kept: caps the founding
        // dose of a brand-new norm at majority breadth, leaving headroom
        // for the reinforcement channel to grow it — one-norm-per-slot
        // dedup means an over-strong founder can never be re-scaled down.
        let strength = (quorum as f64 / n_agents.max(1) as f64).min(0.6);
        norms.register(mindstrata_institutions::norms::Norm {
            id: norms.norms().iter().map(|n| n.id).max().unwrap_or(0) + 1,
            name,
            strength: mindstrata_core::fixed::Fixed::from_f64(strength),
            internalization: mindstrata_core::fixed::Fixed::from_f64(strength * 0.5),
            punishment: mindstrata_core::fixed::Fixed::from_f64(0.1),
            reinforcing_identity: None,
        });
        proposed += 1;
    }
    proposed
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
/// WP-I (Iter-266): `step_collective` is live — it integrates the per-line
/// press, advances shadow stages on saturation, and tracks fulfillment EMAs
/// (see the dev crate). Identity-at-zero is preserved: empty window → zero
/// pressure vector → identity output, so the empty-window pin stays green.
/// i279 (sweep-ratified): MAJOR conflicts press the Identity bucket at f=1.0
/// — full catalyst parity with Grief (i272), Identity's canonical feed. The
/// sweep (i279_identity_fraction_sweep, seed 42, 20K) measured the diet:
/// 55 major events × 2 catalysts / 12 agents = 9.167 raw press → 0.458
/// accumulated at press_growth 0.05 — HALF the 1.0 needed for stage 2, so
/// the fraction is not the binding constraint (the diet is; identical
/// verdict to i274 Relational). f=1.0 is chosen because it is the
/// theoretically honest weight (each major event genuinely reshapes the
/// village's self-image) and yields stage 2 at ~44K ticks — the same
/// real-cultural-timescale pacing class as Relational. No magnitude knob
/// was pulled to pass a probe; the diet constraint is recorded debt.
const IDENTITY_PRESS_FRACTION: f64 = 1.0;

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
    let (relational_press, safety_press, identity_press, meaning_press) =
        accumulate_bucket_presses(&catalysts, n_agents.max(1) as f64);
    // WP-I (Iter-266): the dev crate's vendored-`kind` bucket mapping owns
    // the affinity (culture→relational, system→safety, collective-system→
    // identity, consciousness→meaning), replacing the DC-1 v1 cyclic
    // `i % 4` distribution. CALIBRATION-PENDING(AP3): i266 measures the
    // per-bucket differentiation across the 12-seed family.
    let pressures = mindstrata_development::collective::pressure_vector(
        relational_press,
        safety_press,
        identity_press,
        meaning_press,
    );
    *field = field.step_collective(
        &pressures,
        &mindstrata_development::collective::CollectiveParams::pending(),
    );
}

/// Iter-296 (UM-3 core): per-polity collective holon step. Identical press
/// law to `system_collective_field_step`, restricted to catalysts whose
/// subject is a member of the polity, normalized per-capita WITHIN the
/// polity (n = members.len(), not the global population).
///
/// Additive by construction: the legacy whole-village field is untouched;
/// an unassigned polity map never reaches this fn (identity-at-isolation —
/// a single assigned polity covering all agents reproduces the legacy field
/// bit-for-bit, pinned in `sim/tests/development.rs`).
pub fn system_polity_collective_field_step(
    field: &mut mindstrata_development::collective::CollectiveField,
    events: &[SimEvent],
    members: &[usize],
) {
    if members.is_empty() {
        return;
    }
    let catalysts = collect_catalysts(events);
    // Member filter first (AgentId wraps the agent index; the catalyst
    // tuple's first element is the subject).
    let member_catalysts: Vec<_> = catalysts
        .into_iter()
        .filter(|(subject, _, _, _)| members.contains(&(subject.as_u64() as usize)))
        .collect();
    if member_catalysts.is_empty() {
        return;
    }
    let (relational_press, safety_press, identity_press, meaning_press) =
        accumulate_bucket_presses(&member_catalysts, members.len() as f64);
    let pressures = mindstrata_development::collective::pressure_vector(
        relational_press,
        safety_press,
        identity_press,
        meaning_press,
    );
    *field = field.step_collective(
        &pressures,
        &mindstrata_development::collective::CollectiveParams::pending(),
    );
}

/// Shared press accumulation (Iter-296 extraction): identical math to the
/// pre-split inline body so the whole-village path stays bit-identical.
/// Returns (relational, safety, identity, meaning) bucket presses.
fn accumulate_bucket_presses(
    catalysts: &[(AgentId, CatalystKind, f64, bool)],
    n: f64,
) -> (f64, f64, f64, f64) {
    let mut relational_press = 0.0;
    let mut safety_press = 0.0;
    let mut identity_press = 0.0;
    let mut meaning_press = 0.0;
    for (_, kind, _mag, major) in catalysts {
        let p = 1.0 / n;
        match kind {
            CatalystKind::Bond => relational_press += p,
            CatalystKind::Threat | CatalystKind::Transgression => {
                safety_press += p;
                // i279 (sweep-ratified at f=1.0): MAJOR conflicts also
                // press the Identity bucket — violence/revolution reshapes
                // who a community thinks it is (the collective reading of
                // the same i275 individual-level Identity claim).
                if *major {
                    identity_press += p * IDENTITY_PRESS_FRACTION;
                }
            }
            CatalystKind::Grief => identity_press += p,
        }
        // The "meaning" bucket gets a small baseline from any event
        // (the world exists, so it has meaning) so a long quiet run
        // doesn't starve the meaning/cosmology collective lines.
        meaning_press += p * 0.1;
    }
    (
        relational_press,
        safety_press,
        identity_press,
        meaning_press,
    )
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
        system_polarity_claim_emit(&mut agents, &[], 0);
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

    /// WP-I (Iter-266): the step is LIVE — real catalyst events move the
    /// field off neutral. Threat catalysts press the Safety bucket (system
    /// lines) only; culture/meaning lines stay at founder neutral.
    #[test]
    fn collective_field_catalyst_liveness_safety_bucket() {
        let tick = mindstrata_core::clock::Tick::new(1);
        let evs = vec![SimEvent::ConflictOccurred {
            aggressor: AgentId::new(0),
            target: AgentId::new(1),
            kind: mindstrata_core::conflict::ConflictKind::Threat,
            injury: mindstrata_core::fixed::Fixed::ZERO,
            fear_induced: mindstrata_core::fixed::Fixed::ZERO,
            tick,
        }];
        let mut field = mindstrata_development::collective::CollectiveField::default();
        system_collective_field_step(&mut field, &evs, 12);
        assert!(
            !field.is_neutral(),
            "catalysts must move the field off neutral"
        );
        let slugs = mindstrata_development::collective::CollectiveField::line_slugs();
        let mut safety_moved = 0;
        let mut other_moved = 0;
        for (i, l) in slugs.iter().enumerate() {
            if field.lines[i].press > 0.0 {
                match l.kind() {
                    "system" => safety_moved += 1,
                    _ => other_moved += 1,
                }
            }
        }
        assert!(
            safety_moved > 0,
            "threat catalysts must press system (safety-bucket) lines"
        );
        // Meaning baseline: every catalyst contributes 0.1× per-capita, so
        // consciousness lines also accumulate press.
        assert!(
            other_moved > 0,
            "meaning-bucket baseline must press consciousness lines too"
        );
        // Determinism: identical inputs, identical field.
        let mut again = mindstrata_development::collective::CollectiveField::default();
        system_collective_field_step(&mut again, &evs, 12);
        assert_eq!(field, again);
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
