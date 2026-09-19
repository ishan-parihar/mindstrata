# Iteration 306 — the meaning reflex (meaning-channel dead producer)

**Status:** LANDED · **Root cause owned:** the i305 finding that the `meaning`
need saturates at the calibrated horizon (`p90 = 1.0000`, `Worship` goal duty
41.5% vs `Socialize` 0.43%). i305 recorded it as "a *producer*-side calibration
problem (need decay vs relief balance) … it belongs in its own behavioural
iteration". This is that iteration, and the root cause turned out to be neither
decay nor relief magnitude.

## The probe found a different bug than the one queued

Probe `i306_meaning_channel` measured the channel anatomy before any code was
touched — trajectory, action duty, action *starts*, landing relief, cult gate —
because the arithmetic does not add up on paper:

| quantity | value | implication |
|---|---|---|
| meaning accumulation | `1e-4`/tick | slow |
| Worship relief | `0.1`/tick × 4-tick action = **0.4 per action** | 1000× the accumulation rate |

A village with any real worship duty should hold meaning near zero. Measured
balance (leg 1, 20K × 12 seeds): relief `0.002234`/tick vs accumulation
`0.000100`/tick — a **22.3× surplus** — yet 27.2% of all agent-ticks sat at the
ceiling. Relief magnitude was never the problem.

Leg 3 (per-agent anatomy) found the actual defect:

| seed 42, id | meaning | worship duty | `Worship` goal present | dominant action |
|---|---|---|---|---|
| 1 | **1.0000** | 0.00120 | **84.9% of ticks** | Work 0.416 |
| 4 | **1.0000** | 0.00120 | **84.9%** | Work 0.415 |
| 9 | **1.0000** | 0.00120 | **84.9%** | Work 0.499 |

Agents pinned at the meaning ceiling, with a `Worship` goal live ~85% of ticks,
performed **zero** Worship ticks — their lives were Work. Root cause: the
§10.3 routine override (Iteration 255 order: physiological reflex → command →
**routine** → utility) resolves to `Work` with strength 0.70 against the 0.5
threshold, and **the daily routine template has no worship slot at all**. So for
exactly the agents who needed it, the meaning channel was dead by construction:
no reflex, no command, no routine slot, and the utility contest never reached.

This is the audit's E3 inversion class ("agent starving while working") one
channel over. The i305 notes queued a *calibration* fix; the measurement says the
missing piece is a *structural* one — a relief route, not a bigger dose.

## What landed

**`pass_action.rs` — the meaning reflex**, in the existing Iteration-255
survival-integrity reflex chain, below every physiological reflex (body still
outranks soul — the ratified order) and above routine and utility:

```rust
} else if needs[i].meaning > REFLEX_MEANING_THRESHOLD {   // 0.9000
    Some(ActionKind::Worship)
}
```

Deterministic, RNG-free. Placed last in the chain so a body at its limits keeps
priority; thresholded just under saturation (0.9) so calibrated short windows are
untouched by construction (meaning tops out ~0.30 at 2K ticks).

**2 sim pins** (`crates/mindstrata-sim/src/sim/tests/psychology.rs`):

- `meaning_reflex_forces_worship_over_a_strong_routine` — an agent at meaning
  0.95 whose routine would select Work selects `Worship` instead; the same agent
  at meaning 0.55 does not (the reflex is a last resort, not a worship bias).
- `worship_action_relieves_meaning_through_the_pipeline` — running the action
  through the real pipeline drops `meaning`, so the reflex's target is
  demonstrably in the relief path (the "selected but never applied" case).

## Exit evidence — leg 5 verdict, 12-seed family at 20K

Pre-fix baseline measured by the **same probe at the same horizon** on the
pre-i306 action pass (`5c7122b`-era), recorded before the fix existed:

| metric | pre-fix | post-fix | change |
|---|---|---|---|
| family mean meaning | 0.5280 | **0.4580** | −13% |
| share above the 0.7 gate | 0.3240 | **0.1837** | −43% |
| share at the ceiling (≥0.9) | 0.2720 | **0.0562** | **−79%** |
| worship ACTION duty | 0.02200 | **0.02252** | preserved (+2%) |

| clause | contract | measured | verdict |
|---|---|---|---|
| C1 | 12/12 seeds alive at 20K | 12/12 | PASS |
| C2 | every long ceiling excursion is explained (body reflex or actually worshipping) | 21 excursions, **0 unexplained** | PASS |
| C3 | ceiling share ≤ half the pre-fix baseline (0.136) | 0.0562 | PASS |
| C4 | worship action duty ≥ half the pre-fix baseline (0.011) | 0.02252 | PASS |

**Verdict: `MEANING_REFLEX_LIVE`.**

## Findings recorded, not smoothed

1. **The residual ceiling time is the body reflex, by design.** Every one of the
   21 long excursions is fully explained: `at-ceiling ticks == body-reflex ticks`
   in each case (e.g. pestilence seed 99 agent 2: 14 900 ceiling ticks, 14 900
   with thirst 1.00 and fatigue 1.00, dominant action Trade). Those agents never
   worship because Drink/Eat/Rest outrank the meaning reflex — the ratified
   "body outranks soul" order. C2 was written as an instantaneous
   "no non-physiological agent at the ceiling" clause first, which is the **wrong
   contract**: an agent at 0.9001 has not yet been relieved (the reflex fires on
   the crossing, relief lands over the following ticks). Re-contracted to the
   invariant that actually matters — *no excursion is left unexplained*. The
   re-contract is stated here rather than a threshold widened.
2. **Post-fix ceiling share is fully accounted for by the body-reflex overlap**
   (~6% of agent-ticks ≈ the 21 excursions' ceiling ticks over 12 seeds) — i.e.
   the reflex removed exactly the non-physiological pins and nothing else.
3. **New finding: sustained physiological deficit late in the run.** Seed 42's
   worship duty falls from 0.077 (first 2K) to 0.004 (last 2K) partly because a
   growing share of the village sits at thirst 1.00 **and** fatigue 1.00 for
   12–15K consecutive ticks. This is pre-existing (the pre-fix run shows the same
   vitals for those agents) and is *not* caused by the reflex — but it is the same
   failure class one layer down: a relief route (water/rest supply) that the
   physiological reflex selects for and yet cannot satisfy at long horizons.
   Recorded as the next candidate, with this probe's leg-5 table as its baseline.
4. **The 41.5% Worship *goal* duty from i305 does not mean 41.5% worship
   *action* duty** — post-fix action duty is 2.25%. Goal presence is not action
   performance; i305's goal-duty table should be read with that caveat, which is
   now measured rather than assumed.

## Re-anchors (both required, both probe-evidenced)

1. **Snapshot `long_horizon_surface_10000_ticks`** regenerated. Mechanism: the
   reflex threshold is 0.9, and meaning crosses it around tick ~6000 under the
   i305 calibration, so any run ≥ ~6K ticks re-rolls its trajectories. The
   shorter golden baselines are unaffected and stayed byte-identical (the gate's
   golden replay passed unchanged), which is the expected shape: this is a
   *re-roll of a long-horizon surface*, not a calibration drift. Reviewed with
   `cargo insta test --release` before accepting per §3.
2. **Faction attachment test seed 5 → 1**
   (`integration_tests::psychology::attachment`). This is the timing-fragility
   case already recorded in Iteration 240: the test samples the FIRST instant a
   live faction exists, and the reflex re-rolls long-horizon trajectories. Under
   the new dynamics, seed 5's first live faction is at 7000 with a one-faction
   registry whose stored style is Avoidant while its members' modal style is
   Secure — the style clause fails at the sample instant (the pre-i306 comment
   claimed this seed always formed a Freshly-Secure faction; the reflex
   invalidated that). Sweep `i306_faction_reanchor` (12 seeds × 30K, all three
   clauses evaluated at the first live instant) → **seed 1** satisfies all three:
   first live 4000, registry 1, non-Secure 1, supplies 0.6960, cohesion 0.6071,
   style clause true. Valid re-anchor seeds at 30K: `[1, 2, 7, 21, 46, 77, 99,
   123]` — seed 1 chosen as the earliest first-live instant, and both
   observability clauses hold (stronger than the `or` the assertion allows).
   Seeds 13 (no faction within 30K), 42 and 55 (style clause) and 5 (fresh-faction
   clause) are recorded as ineligible, so the choice is evidence, not a lucky pin.

## Blast radius

`Standard` difficulty is **not** involved — the reflex is unbanded liveness, not
a difficulty knob. Nothing in the lever surface moved. The only drift is the
long-horizon re-roll above, which is the intended behavioural consequence: agents
whose meaning saturates now act instead of sitting at 1.0.

## Verification

- core lib **34/34**; sim lib **272/272** (2 new pins); tests-crate release suite
  **309/0/1** (one re-anchored seed, no pin deleted).
- fmt clean; clippy 0; full `scripts/gate --full` **GATE GREEN** with golden
  baselines byte-identical except the reviewed 10K surface above.

## Queue after this iteration

- **Sustained physiological deficit at long horizons** (finding 3): the next
  behavioural candidate — same failure class, one layer down.
- **Row 3 residuals**: ceiling band; Q2 dark-allergy saturation.
- **Row 1**: stays prohibited (AGENTS §5 H5).
- DC-4 continues with the CLIENT asset-viewer and the graphical shell spike.
