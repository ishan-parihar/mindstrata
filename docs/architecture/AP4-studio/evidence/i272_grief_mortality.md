# Iteration 272 — Grief producer root-cause fix (§4.3 dead producer)

**Status:** LANDED · **Probe:** `crates/mindstrata-benches/examples/i272_grief_mortality_probe.rs`

## Probe evidence (5 seeds × 20K ticks, N=12)

| seed | deaths | first death | norm_viol | first violation |
|---|---|---|---|---|
| 42/7/99/13/46 | **0** | None | **0** | None |

- Natural mortality at default demography (35040 tpy, Gompertz senescence,
  agents age ≈0.57y/1000 ticks → ~55y at 20K): cumulative death chance per
  2000-tick window ≈ **0.23%/agent**. First natural deaths land at ~2.3M
  ticks. **Grief and Transgression are structurally unobservable at any
  feasible calibration horizon** — the i270 "event-rate-limited" note was
  generous; they are horizon-blocked by construction.

## The real bug found (worse than unobservability)

`map_event` routed `SimEvent::AgentDied { agent } → Grief(1.0)` to the
**deceased's AgentId**. But dead agents are replaced IN PLACE by newborns in
the same tick (the `AgentId == index` invariant), and `handle_agent_death`
clears partner/parent references before returning. So any Grief catalyst ever
produced would soak into the **replacement newborn**, not a mourner — a
doubly-dead producer: unreachable at horizon AND mis-targeted if reached.
The `catalyst_observers` module docs had documented this exact hazard and
prescribed the fix ("record grief targets into an inert side-buffer inside
the deaths pass").

## The fix

1. **`SimEvent::GriefStruck { mourner, deceased, tick }`** (new core variant).
2. **Capture-then-emit**: `handle_agent_death` collects grief targets
   (surviving spouse first, then co-resident kin — children and parents) into
   `pending_grief_targets` BEFORE the in-place replacement clears references.
   The social-cluster pass (which owns the demography deaths loop) drains the
   buffer into `GriefStruck` events in the same tick, in death order
   (deterministic).
3. **`map_event` re-routed**: `GriefStruck { mourner } → Grief(1.0)` on the
   surviving subject; the old `AgentDied` route is removed. Magnitude 1.0
   stands (maximal-loss exemplar, bounded by the capture discipline).
4. Pestilence-shock deaths (the collapse scenario) flow through the same
   `handle_agent_death` path, so scenario mortality now produces real grief.

## Golden re-anchor (§4.2 form)

Only the **collapse** golden moved — the one scenario whose horizon contains
deaths (Pestilence immediate-mortality wave at tick 1100). All other goldens
(death-free horizons) are byte-identical, confirming the change is inert
where no deaths occur.

- `metric_hash`: `3105625988242210193` → `6919834816934608471` (regenerated)
- `event_hash` moved (GriefStruck events now in the window), `agent_hash`
  moved (survivors' Q4/identity fields now respond to loss)
- `total_grain`: 61.01 → 30.13 — **mechanism**: mourning survivors with
  elevated Golden-Allergy tension reduce Work; the village under compound
  crisis now grieves, and provisioning visibly drops. This is the emergent
  cost-of-grief signal the system was designed to produce and never could.

## Liveness pin

`grief_routes_to_surviving_mourner_not_replacement`: spouse dies → mourner's
Grief-indexed altitude line advances; replacement newborn receives nothing.
Also corrects the test-side expectation that Q4 intensity grows on grief:
Allergy metabolism *resolves* under pressure (Δ = −0.025·I at pressure 1.0) —
the mourning-resolution law — so altitude line 0 is the arrival observable.

## Verification

- sim lib **217/217** (new pin included) · dev 80/80 · clippy 0 warnings · fmt clean
- `gate --full` **GATE GREEN — 307/0/1**, golden re-anchored with mechanism evidence
- bench law: 0 violations (i272 registered)
