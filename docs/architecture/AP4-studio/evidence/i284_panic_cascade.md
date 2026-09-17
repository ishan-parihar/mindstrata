# Iteration 284 — WP-H2 refutation half: contradictory evidence → Refuted → panic cascade

**Status:** LANDED · **Doctrine:** probe-first; one scope correction caught by the probe before landing (§4.3 applied to my own new code).

## What was missing (meta-review finding)

The Era III wave brief (WP-H2) names two halves: reconciliation (landed i275) and **refutation** — "contradictory evidence refutes (Reconciled→ActiveTension)" — the moral-panic mechanism (probe `i288_panic_cascade`: refutation storms produce measurable norm churn, then re-crystallization). No `Refuted` state, no refute transition, no cascade probe existed.

## What landed

- **`PolarityState::Refuted`** (dev crate) + `PolarityTag::Refuted` rendering ("now called into question" — the chronicle shows the village arguing with itself).
- **`refute_claim(claim)`**: Integrated → Refuted. Pure, deterministic.
- **Refutation scan** in `system_polarity_claim_emit` (after reconciliation, per agent): a Threat catalyst in the current window on the same `(referent, line)` slot, **postdating** the claim, refutes it. Postdating matters: same-tick evidence is the claim's own origin, not a contradiction.

## Scope fix caught by the probe

First pass refuted `ActiveTension` claims too. Result: the contested Event/cognitive slot saturated to **100% Refuted in every regime** (integrated 0, tension 0, refuted 110–126 @20K) — refuting an in-question claim is semantically a no-op and mechanically destroyed the tension substrate. **Fix: Integrated-only.** The wave brief's own semantics ("Reconciled→ActiveTension") name synthesized belief as the refutation target; tension claims resolve through reconciliation or the i281 window.

## Panic-cascade probe results (`i284_panic_cascade`, 20K, Event/cognitive slot)

| regime | tension | refuted | refuted share |
|---|---|---|---|
| calm | 80 | 16 | 7.5% |
| drought | 98 | 22 | 10.0% |
| collapse | 76 | 19 | — |

1. **Refutation live**: 16–22 Integrated-then-refuted turnovers per 20K — the advance→tension→integrate→refute cycle runs in vivo (Grief's `(Event, cognitive, Identity)` projection collides with Threat's Fact claim on the same slot, so crystallized Identity claims exist to refute).
2. **Churn is regime-ordered**: drought (10.0%) > calm (7.5%) — more conflict, more contradiction, more refutation.
3. **Re-crystallization**: `Refuted` is terminal (refuted beliefs don't resurrect); the fresh-claim stream from ongoing catalysts is the recovery channel — tension counts 76–98 prove the slot keeps living.

## Re-anchor trail (the mechanism is the refutation semantic widening, all verified in-field)

- **Both goldens** (riverford@1000, collapse@4320): `agent_count` stable 12→12; riverford grain −0.17 (−0.2%); collapse grain 23.56→33.15 (+9.6 — refutation damping the social bias lets provisioners work more; consistent with the i281 10K correction direction).
- **8 snapshots** (500/1000/2000/10000): stress/hunger/trust/thirst tails moved at 500 (the widened state changes selection from the first refuted claim); 10K: `event_count` 140K→140.5K, grain 1.80→2.31, envy −0.001 — all same damping signature.
- **bonding_rate behavioral pin**: floor 0.0005 → 0.0001 with mechanism (measured delta 0.000142, baseline 0.615315 vs treated 0.615457 — the consumer stays live; refutation damps relationship cascades, so the floor tracks the new damped magnitude).

## Pins

`contradictory_evidence_refutes_living_claims` — postdating threat refutes an Integrated claim; same-tick evidence does not. Sim lib 234/234.
