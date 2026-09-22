# Iteration 368 — the perf budget is a ceiling; test N is per-phenomenon

**Status:** CHARTER DECISION (no source changed) · **Item owned:** the long-open
"Phase-1 population-target" question (recorded as open in i332/i334's ledger rows).

## The decision

The DC-3 P0 budget (122 / 1088 / 5961 µs/tick @ N=12/48/96) was being read as a
*mandate* for the population the suite should run at. It is not — it bounds **cost**. This
iteration makes the policy explicit (AGENTS.md §3, ENGINE_STATUS §8):

- The budget is a **ceiling**, not a test-population mandate.
- Every test/probe runs **the smallest N that exhibits the phenomenon under test** — a
  gossip pin needs N=12; a settlement/trade pin needs N≥144.
- Default matrix **{12, 48, 144}** (village / large village / town edge); **N=256** is the
  **town-scale stress tier** (i359 measured the envelope fits it at ~5.8 ms/tick).
- Do not raise N for "more realism" when a smaller N demonstrates the same contract; do not
  lower it below the phenomenon's floor (that is the unreachable-gate hazard, §5).

## Why (the reasoning the user's question surfaced)

- The "population target" is a **feedback-loop cost** choice, not a simulation realism
  knob. The realism knobs are `MAX_POPULATION` (a real cap, 256) and the world law.
- Probes **already** choose N adaptively (i359 swept 48→256; i360/i362 used 192/256 for the
  settlement work). The policy simply names what good practice already does and stops the
  charter from implying a single mandated N.
- The perf gates keep the N=12/48/96 envelope unchanged — this is a testing-doctrine
  change, not a budget edit.

## Effect

Closes the "raise the Phase-1 target?" question by **reframing** it: there is no single
target to raise — the budget is a ceiling and each phenomenon picks its own N. Subsequent
behavioural iterations follow the same rule.

## Verification

Docs/doctrine only; **no source changed**. Golden byte-identical, `gate --full` GREEN.
