# Iteration 309 — the health-critical reflex was a Rest mutex

**Status:** LANDED · **Root cause owned:** the i308 open finding (the Rest
plateau). i308 measured the plateau and excluded energy, sleep,
habit-substitution and the emotional modifiers by live-sim ablation, localizing
the driver to "a pass-level input the utility ledger does not carry". This
iteration names it.

## The trace named the branch

A temporary env-gated instrumentation of the action pass (removed before commit)
printed, for the plateau agent, every reselection:

```
[pass] agent 6 tick 20005: action Rest | reflex Some(Rest) | follow_routine false
       (strength 0.618 vs thresh 0.300) | needs h0.531 t0.766 f0.001 s1.000 m1.000 |
       fear 0.940 anger 0.051
```

**`reflex Some(Rest)`** — the i255 survival-integrity chain was forcing Rest,
via its last branch:

```rust
} else if agents[i].body.health < Fixed::from_f64(0.25) {
    Some(ActionKind::Rest)          // "health-critical: restrict to recovery actions"
}
```

every tick, for the whole run. Why the branch never released:

- `body.health` is the **derived** value. `derived_health()` = `(base ×
  immune_modifier − 0.2 × stress.level − 0.15 × chronic_load − pain − sickness −
  shock) × skeletal_factor`. The plateau agents sit at stress level 0.70–0.78 and
  chronic load 0.80–0.87, which is −0.27 by itself.
- **Rest reduces none of those penalties.** So the gate could never open, and
  while it was closed the agent was forbidden every other action — including
  Drink, which is why its thirst was allowed to climb to 0.9 before the thirst
  reflex (checked earlier in the same chain) could fire, and why its social and
  meaning needs were pinned at 1.000 because Socialize/Worship were literally
  illegal.

The i308 ablation results are consistent with this mechanism and are now
explained by it: clearing habits or emotional state cannot move a Rest duty
driven by a reflex branch, and "Zeroing fear/sadness raises Rest" is exactly what
you expect when fear is the *input* to the chronic-load penalty that keeps the
gate shut.

## What landed

The frame keeps the i255 intent ("a body at its limits does not exert itself")
and drops the mutex (one file, `pass_action.rs`):

1. The health branch leaves the reflex chain; the chain keeps the acute
   physiological reflexes and the i306 meaning reflex.
2. `let health_critical = agents[i].body.health < HEALTH_CRITICAL_THRESHOLD;`
3. After **all** selection paths — including the stress-habit fallback, so a
   habit cannot smuggle `Work` past it — an exerting action is downgraded:

```rust
if health_critical && is_exerting(action) { action = ActionKind::Rest; }
```

with `is_exerting` = `{Work, Wander}` (heavy labour, energy cost 0.05; roaming,
energy cost 0.02 and the classified risky action). Everything else — eat, drink,
rest, socialize, worship, trade, idle — stays available to a body in crisis.
Deterministic, RNG-free.

**3 sim pins**: the frame keeps agency (a health-critical agent with a 0.5 hunger
deficit selects `Eat` and its hunger falls — the old mutex forbade it), the frame
still bites (a routine-forced `Work` selection is downgraded to `Rest`), and the
classification is exact (`is_exerting` true for Work/Wander only).

## Exit evidence — the trap's signature, 50K

| agent | Rest duty | top actions | social | meaning |
|---|---|---|---|---|
| seed 123 / 6 | **0.813 → 0.167** | `Trade 0.15 → Trade 0.80, Work 0.03` | **1.000 → 0.009** | 1.000 → 0.451 |
| seed 2 / 2 | **0.893 → 0.562** | `Work 0.06 → Trade 0.27, Rest 0.56, Worship 0.09` | **0.793 → 0.005** | 0.866 → 0.377 |
| seed 1 / 0 | **0.833 → 0.183** | `Trade 0.10 → Trade 0.76, Work 0.05` | 0.777 → **0.000** | 0.867 → 0.528 |

Family level (12 seeds × 50K): Rest duty **0.3117 → 0.2924**; agents with Rest
duty > 0.5: **6 → 1**. Needs that were structurally unreachable are now met, and
the freed agents are economically active (Trade 0.76–0.80).

**Verdict: `HEALTH_FRAME_KEEPS_AGENCY`.**

## Findings recorded, not smoothed

1. **Derived health equilibrates at ~0.21–0.25 for the most chronically stressed
   agents** (tracked agents' 50K means: 0.208–0.245). So the 0.25 gate is not an
   acute crisis detector in practice — it is a *chronic frailty* state that some
   agents occupy for the whole run. Post-fix that state is a functioning one
   (non-exerting work: trading, socializing, worship) instead of a trapped one,
   but the gate is doing double duty for both acute and chronic conditions.
   Recorded as calibration debt: a future iteration should either split the gate
   (acute: pain/shock/injury/sickness) from chronic frailty, or make derived
   health's chronic-load penalty mean-reverting.
2. **The 6 → 1 residual.** One agent (seed 2 / 2) still rests 56% of its life —
   but with social 0.005 and meaning 0.377, i.e. it is *resting with its needs
   met*, which is the legitimate equilibrium i308 was looking for, not the trap.
3. **Interaction with i306 worth recording**: because `Rest` was the only legal
   action, the meaning reflex could never fire for these agents. Removing the
   health branch from the chain also un-shadowed the meaning reflex for the
   health-critical population.
4. **No re-anchors at all** — golden and every snapshot stayed byte-identical,
   because the fix only changes behaviour where derived health is below 0.25,
   which does not occur inside the pinned horizons (≤ 2000 ticks; the 10K surface
   and 4320-tick crisis reach it only rarely). This is consistent with i308's
   "minority, long-horizon" characterization of the plateau, and it is the
   *opposite* of i307's re-anchor shape — which tells the next reader that the
   long-horizon surface is where this class of defect lives.

## Verification

- core **34/34**; sim lib **277/277** (3 new pins); tests-crate **309/0/1**
  (no re-anchors); fmt clean; clippy 0; `scripts/gate --full` **GATE GREEN** with
  golden byte-identical; bench index `--strict` 0 violations.

## Queue after this iteration

1. **Gate split / chronic-frailty health** (finding 1) — the next calibration
   item on this axis.
2. **Re-measure the i306 meaning channel** in light of finding 3 (the reflex now
   reaches the frailty population).
3. Levers row-3 residuals (ceiling band, Q2 dark-allergy saturation); row 1 stays
   prohibited (AGENTS §5 H5); DC-4's CLIENT asset-viewer + graphical shell.
