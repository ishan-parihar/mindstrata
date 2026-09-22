# Iteration 364 — the collapse scenario's bite is intact; i363 added a birth, not a reprieve

**Status:** MEASUREMENT (probe only, no source changed) · **Item owned:** the open
consequence recorded by i363 — "collapse-scenario length-of-horizon lethality softening
unmeasured".

## The concern

i363's council surplus dividend moved **`agent_count` 12 → 13 on both goldens**,
including the `collapse` golden. That scenario exists precisely to stress *compound
emergence when crises stack on a weakened population* (drought @500 → famine @800 →
pestilence @1100 over 4320 ticks), so a demographic lift there could mean redistribution
had **neutralized crisis mortality** — a real side-effect, not a benefit.

## The measurement (A/B on the scenario's own 4320-tick run, seed 42)

Probe `i364_collapse_mortality` counts `AgentDied`/`ChildBorn` incrementally across the
cascade, run once on HEAD and once on the pre-i363 parent (`8923e8a`):

| build | final agents | **deaths** | births | health | hunger | gini |
|---|---|---|---|---|---|---|
| pre-i363 (`8923e8a`) | 12 | **2** | 0 | 0.774 | 0.0376 | 0.475 |
| i363 (`8199e64`) | 13 | **2** | **1** | 0.792 | 0.0274 | 0.453 |

The mortality trajectory is **identical**: both runs kill exactly **2 agents**, both at
the pestilence window (~tick 1100–1200), and both keep 12 live agents through the
cascade. The `12 → 13` shift is entirely **one extra birth** at ~tick 3500 — the
redistribution improved provisioning enough for a marriage to deliver a child, which is
the intended (benign) direction. `verdict = MORTALITY_PRESENT` on both builds.

## What this resolves and records

- **Resolved:** i363 did **not** soften the collapse scenario. The stress-test target —
  a population that dies under a staggered cascade — still fires 2 deaths, and the
  scenario keeps its bite. The `agent_count 12→13` golden shift is a birth, not a
  reprieve.
- **Recorded improvement:** even the collapse run is healthier under redistribution
  (health 0.774→0.792, hunger 0.0376→0.0274, gini 0.475→0.453) — the same benign
  redistribution signature seen in i363's snapshots.
- **Method:** A/B on the exact scenario run (checkout the parent, rebuild, re-run) is the
  cheap, decisive way to attribute a golden shift; recording it here closes the item
  without a guess.

## Verification

Probe + docs only; **no source changed**. Golden byte-identical, `gate --full` GREEN.
