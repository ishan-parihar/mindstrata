# Iteration 355 — the tier gate stops lying: stale name fixed, unwired predicates documented

**Status:** LANDED (naming + documentation, zero behaviour change) · **Item owned:** the
§17 tier-gate residual (queue item 4) — "cosmetic + a dead predicate."

## What was wrong

Two ways the `AgentTier` API misrepresented the system:

1. **A dishonest parameter name.** `update_narrative_importance(.., relationship_count, ..)`
   took the *count of all N−1 rows* before i348, then i348 re-wired the caller to the honest
   `contacted_degrees()` (users with real interaction state). The name kept describing the
   value it no longer received — exactly the i342 "social-count proxy" failure shape, now in
   an identifier.
2. **A false affordance.** `AgentTier::runs_full_biology()` and
   `AgentTier::runs_action_selection()` have **zero production call sites** (i316): the
   biology pass and action selection run for *every* tier. The predicates describe an
   intended LOD rung that was never wired, so any future agent gating on them would get a
   silent no-op — or, worse, freeze Background agents mid-action if they wired it without a
   sweep. The census (i346) shows Background agents still take decisions, so
   `runs_action_selection() == false` for Background is simply untrue.

## What landed

- Renamed the parameter `relationship_count` → `contacted_degree`, with the reason recorded
  in place (§6: comments explain WHY). Purely positional at the call site; no behaviour.
- Documented both unwired predicates as **RECORDED DEBT (i316, i355) — NOT CONSUMED**,
  stating what they describe, why they are not wired, that i328 measured the whole gate
  payoff at ≈5%, and that wiring them is its own behavioural iteration (a `runs_action_selection`
  gate would silently freeze Background agents and needs a full sweep).

The **cognitive** LOD rungs remain real and stay as they are — `runs_full_psychology`,
`runs_prospection`, `runs_theory_of_mind`, `runs_relationship_updates` are all consumed in
`systems/cognitive.rs` / `social_cluster.rs`, so the tier's actual cut is cognitive, which is
consistent with the i348 re-contract (Background keeps social presence, loses memory /
prospection / belief / ToM).

## Verification

Zero behaviour change by construction (rename + comments). `mindstrata-sim` **296/296**,
`gate` GREEN, golden replay **5/5 byte-identical**, clippy clean.

## Ledger

- Queue item 4 is **closed for the cosmetic half**; the remaining half is a *decision*, not
  a cleanup: wire the biology/action LOD rungs (behavioural, sweep-carrying, ≈5% ceiling) or
  delete the predicates. Recorded in AGENTS §8 item 4.
