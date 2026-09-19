# Iteration 308 — the Rest plateau: measurement, and the mechanisms it rules out

**Status:** LANDED (measurement iteration, **no behaviour change**) · **Root cause
owned:** the i307 finding 1 — agents whose habit stopped pinning them settle into
`Rest` (seed 123 agent 6: `Rest 0.813` with fatigue mean 0.040, 88% of ticks
below 0.05, pre-i307 census `Trade 0.986`).

An agent sleeping with nothing to sleep off is a stuck producer of the same
family as i306 (meaning) and i307 (physiological reflex), so the iteration was
scoped as *probe first*: the doctrine's §2 order. **No code changed in this
iteration** — the deliverable is the anatomy plus a ruled-out list, which is what
makes the next iteration's fix targetable rather than guessed.

## What the plateau is

Probe `i308_rest_dominance`, 12-seed family at 50K (calm worlds):

| quantity | family | plateau agents (6 of ~168) |
|---|---|---|
| Rest action duty | 0.3117 | **0.78–0.89** |
| fatigue | 0.1754 | 0.02–0.08 |
| energy | 0.9016 | **0.94–0.99** |
| sleep pressure | 0.1252 | 0.04–0.11 |
| sleep debt | 0.0114 | 0.00 |
| Sleep as motivation argmax | — | 1.6–11% of ticks |
| fear / sadness / valence | 0.271 / 0.035 / −0.139 | **0.91–0.94 / 0.69–0.82 / −0.91…−0.96** |
| trauma / depression risk | 0.322 / 0.372 | 0.65–0.84 / 0.20–0.24 |
| **social need** | — | **0.71–0.84 (unmet)** |
| **meaning need** | — | **0.81–0.89 (unmet)** |

The plateau agents are not tired, not energyless, not sleepy — they are
**frightened and isolated**, and their Rest is not repaying the needs that are
actually unmet (social, meaning). Family-level affect is stable across horizons
(fear 0.244 @2K → 0.271 @50K: no family-wide ratchet), so this is a
**minority equilibrium**, not a global drift.

## Mechanisms ruled out (each by its own measurement)

| candidate | test | result |
|---|---|---|
| A. ENERGY — depleted, only Rest recovers | plateau energy 0.94–0.99, and Rest duty 0.89 with energy 0.99 | **ruled out** |
| B. SLEEP — pressure/debt stay high, Rest can't repay | sleep pressure 0.04–0.11, debt 0.00, Sleep-dominant 1.6–11% | **ruled out** |
| C. HABIT — the i307-exposed habit layer | live-sim ablation clearing `habits` + `automaticity` every tick: Rest duty 0.2995 → **0.2672** | **ruled out** |
| D. AFFECT — fear/sadness withdrawal modifiers | live-sim ablation zeroing `fear`+`sadness` every tick: Rest duty 0.2995 → **0.3341** (Rest *rises*) | **ruled out, inverted** |
| E. NEEDS — the landscape itself | plateau survives with social 0.71–0.84 and meaning 0.81–0.89 unmet | **surviving candidate** |

The D result is worth keeping: the emotional modifiers are *not* what pins these
agents (zeroing them makes Rest slightly more attractive, because the fear bias
applies to every non-risky action and is clamped at +0.3 — it is a near-uniform
offset, not a withdrawal lever).

## Where the remaining uncertainty is, exactly

A utility ledger (leg 4) reproduces the pass's inputs — `compute_utility` +
goal-alignment + the clamped emotional modifier — and **fails to reproduce the
sim's own winner**: over 50 sampled ticks of the plateau agent the ledger picks
`Trade 33 | Drink 16 | Rest 1`, while the sim's census is `Rest 0.81`. That is a
useful negative result: the missing term is a **pass-level input the ledger
omits** (habit modifier, institution work bonus, norm pressure as adjusted by the
pass, polarity bias, somatic marker, the routine override, or the interruption
path) — not something in `compute_utility`'s needs/personality/identity math.

Two further observations from the same leg:

- The flattened regime is real: in states where every need is satisfied the
  candidate utilities are all near zero, and Rest is the only action carrying a
  residual term (its fatigue-relief product) — a 0.04-fatigue agent still scores
  Rest above alternatives whose relief products are exactly 0.
- Pinning social+meaning high every tick collapses Rest to **0.004** and moves
  the village to `Idle 0.75 | Worship 0.24` — i.e. the plateau is not immune to
  need pressure on the *inputs*; it is a *selection* equilibrium.

## Verdict

**`REST_PLATEAU_MEASURED_DRIVER_LOCALIZED_TO_UTILITY_LANDSCAPE`** — the plateau
is measured, its candidates A–D are excluded by ablation, and the residual
driver is localized to the pass-level inputs that the utility ledger does not
carry.

## RESOLVED — see i309

The driver was named the next iteration by tracing the pass directly: the
**health-critical reflex** (`body.health < 0.25 → Some(Rest)`), a Rest mutex over
a *chronic* derived-health state. Fix and numbers:
`docs/architecture/AP4-studio/evidence/i309_health_frame.md`. The utility-ledger
route proposed below was superseded by that trace (the ledger could not have
reproduced the sim for the simplest of reasons: selection never ran — the reflex
path returns before the utility layer is consulted).

## Queue after this iteration (kept for the record)

1. **Utility-landscape iteration**: extend the
   ledger to a *pass-faithful* one (routine override + adjusted norm pressure +
   habit modifier + institution bonus + polarity + somatic marker), then re-run
   the ablation against it. Only after that should a fix be attempted — the
   i308 evidence says a "fear/withdrawal" or "habit" fix would be aimed at the
   wrong mechanism.
2. **Meaning residual re-measure**: post-i307 the i306 verdict strengthened
   (at-ceiling 0.2720 → 0.0562 → **0.0164**, long excursions 21 → 6, all still
   body-reflex-explained); the i306 doc's queue line is superseded by that
   number.
3. **Fatigue pace at 50K** (i307 finding 2) — memory recorded only; not
   re-measured here.
4. Levers row-3 residuals (ceiling band, Q2 dark-allergy saturation); row 1 stays
   prohibited (AGENTS §5 H5); DC-4's CLIENT asset-viewer + graphical shell.

## Verification

- Nothing in the workspace changed except a new bench example and docs: core
  **34/34**, sim lib **274/274**, tests-crate **309/0/1**, golden byte-identical,
  snapshot baselines untouched, `scripts/gate --full` **GATE GREEN**, bench index
  `--strict` 0 violations.
