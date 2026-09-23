# i392 row 5 — `aggression_threshold` reaches the escalation gate: MEASURED AND REJECTED

**Status:** REJECTED (wiring built, measured, reverted) — the fifth and last inert gene.
Tree restored: integration **314/0/1**, both goldens byte-identical, no re-anchors.

## The consumer the gene always named

The gene's doc comment says "low = easier to anger", and the engine already contains exactly
that gate: `should_escalate` (`clans.rs`) holds a failed threat back unless

```text
aggressor_aggression > conflict_escalation_aggression_threshold   (parameter 1.2, village-wide)
```

while the aggressor score is composed per-agent from live state
(`dominance + risk_tolerance + endo_dominance×0.3 + inhibition_hold`). The gene is the missing
per-agent disposition on the **threshold** side, so the candidate law was

```text
threshold(gene) = 1.2 × (1 + (gene − 0.55) × 1.5)      clamped to [0.6, 1.8]
```

anchored at the draw midpoint 0.55 (`U(0.2, 0.9)`), so a midpoint carrier reproduces the
shipped parameter exactly and the population-mean threshold is unchanged (§4.6).

## Why the harness had to be different from rows 1–4

Pinning the gene and re-running the world does **nothing** until the wiring exists — the gene
is not yet read, so a pinned world IS the natural world. And dominance, the row the census
guessed first, is the *score*, not the *threshold*. The gene's consumer is the threshold side of
the comparison, so the manipulable dial at that locus is the **score**: for each agent, raise
`aggressor_aggression` by δ and find the ceiling δ\* where the gate first opens.

## Measurements (legs A–C, pre-wiring)

**A — occupancy: the band is WIDE open (unlike row 3).** i273's 12-seed violence family @4K:
4–7 of 12 agents per seed already sit **above** the 1.2 gate on `dominance + risk_tolerance`
alone (mean score 1.04–1.22, max 1.80). Threats 76–140 per seed, violence 1–8. This is a live,
load-bearing decision point.

**B — the δ\* distribution.** Per seed (48 agents total): mean δ\* ≈ 0.18–0.24, max 0.67–1.18,
with **4–7 agents at δ\* = 0** per seed. Bimodal at the band edge: a block of carriers who
clear the gate on their score alone, and a tail needing 0.3–0.7 more.

**C — the candidate law over those ceilings.** Monotone and smooth in the pool share:

| gene | threshold | agents the shifted gate admits (of 48) |
|---|---|---|
| 0.20 | 0.57 | 41 |
| 0.35 | 0.84 | 37 |
| 0.55 | 1.20 | 31 |
| 0.75 | 1.56 | 11 |
| 0.90 | 1.83 | 6 |

No occupancy explosion and no threshold lottery — the gate's *chance chain* beneath the
comparison (trust, obligation, taboo, dominance scale) already smooths per-decision outcomes.
**This is the gradient shape rows 2 and 4 lacked, so the locus is right.** The probe therefore
proceeded to the wiring on the strength of its own pre-registered checklist.

## The pre-registered checklist, and its result

Written into this document *before* the wiring ran, per §4.15:

| # | criterion | result |
|---|---|---|
| 1 | fmt / clippy / sim suite | PASS |
| 2 | integration suite green | **FAIL — 13 tests** |
| 3 | `violence_taboo_aversion_suppresses_escalation_differentially` | PASS |
| 4 | `relational_dominance_feeds_violence_escalation` (12-seed direction) | PASS |
| 5 | `revolution_is_regime_change_not_repeat_loop` (≥2 of 3 seeds fire) | **FAIL — 1 of 3** |
| 6 | `gate --full` goldens byte-identical | **FAIL — both goldens** |
| 7 | law pinned by a unit test | built (`escalation_threshold_is_gene_modulated_around_the_draw_midpoint`), passed |

## The measured failures (wiring live)

| family | measured | contract |
|---|---|---|
| `golden_replay_vs_baseline`, `golden_replay_crisis_vs_baseline` | diverged | byte-identical |
| 7 × `snapshot_tests` | drifted | pinned snapshots |
| `revolution_is_regime_change_not_repeat_loop` | `[(5, 0, 1, 0), (42, 0, 2, 0), (12345, 1, 3, 1)]` — seed 5 fires **0** | ≥2 of 3 seeds fire |
| `sensory_field_fear_contagion_is_live_and_sustains_fear` | presence **10/12** | i351 band 11–12/12 |
| `neural_like_prediction_error_folds_are_live_and_directional` | wins **3/6**, best delta 0.0857 | majority of seeds + real magnitude |
| `kinship_penalty_rises_when_families_form` | seed 43 founding `kinship_penalty` **0.5 ≠ 0** | founding village has zero |

**Four independent families plus both goldens.** Re-running the same set on the reverted tree
passes all of them (9/9 identical run), so the attribution is exact and not a pre-existing red.

## Why this is a rejection and not a re-anchor

**§4.1 / §4.4.** The failures are not magnitude drift in one family with a stated mechanism —
they are the *violence channel* re-timing four unrelated subsystems (fear contagion presence,
prediction-error majority, founding kinship, the revolution family). That is not a re-anchor
sweep; it is a mis-shaped coupling. Rows 2 and 4 were rejected on exactly this signature (the
revolution family collapsing to 1/3), and consistency matters more than salvaging the row.

**The structural reason, and it is the useful finding.** Every calibration in this engine that
survives the golden gate does so by being **dormant in the calibrated window** — the repeated
`ONE-SIDED: identity at zero` pattern (norm resistance zero before tick 4320, obligation 1.0 at
the 0.5 anchor, humiliation/contempt/despair never produced in calm worlds). A gene on the
threshold side cannot use that device: `aggression_threshold` is drawn at founder time, so
**every agent deviates at tick 0** and the two-sided shift is live in the goldens by
construction. Rows 1 and 3 cleared the gate because their consumers were *inside a dormant
state* (the puberty ramp opens at ~385K ticks; severe injury never occurs in any corpus). Row 5
has no such state.

## Recorded upgrade path (not taken here)

The locus is established as live and correct; what a future row needs is a **state-gated
expression**, so the coupling is dormant through the calibrated windows and opens only in
long-horizon worlds. The concrete candidate, in dependency order:

1. Route the gate's threshold modulation through a state that is zero at tick 0 and nonzero
   only after the norm machinery exists — e.g. express it only for agents with
   `norm_resistance("no-violence") > 0` (zero before the first monthly ritual at tick 4320).
   That makes the golden/snapshot windows byte-identical *by mechanism* rather than by luck,
   which is the property rows 1 and 3 had and row 5 lacked.
2. Re-run the checklist above with the same four families; the probe (legs A–C) is already
   built and stays valid, since the locus and the gradient shape do not change.

Recorded as a **design-act candidate**, not debt: the trait stays deliberately inert, exactly
as `novelty_seeking` and `sensory_acuity` do. **Five inert genes went in; three are now wired
(`puberty_age`, `chronic_pain_risk`) or rejected (`novelty_seeking`, `sensory_acuity`,
`aggression_threshold`) with records, and no inert gene is left unwired-by-accident.**

## Artifacts

* Probe: `crates/mindstrata-benches/examples/i395_aggression_threshold.rs` (legs A–E; the
  wiring leg D is retained as the reproducible rejection harness).
* Reverted: `clans.rs` (the `escalation_threshold_for_gene` law + its four constants in
  `mod.rs`), `sim/tests/conflict.rs` (the unit pin + the nine escalation-fixture gene pins).
