# Iteration 367 — the LOD tier is a crisis rung; it is dark in calm towns

**Status:** MEASUREMENT (probe only, no source changed) · **Item owned:** the i359
recorded follow-up — "the LOD tier carrying Background agents was not exercised at town
scale; i348 measured Background at 25–40% of *crisis*-world agent-ticks, and calm town
runs sit below the importance gate, so the LOD saving at town scale is unmeasured".

## Measurement — tier share of agent-ticks

`i367_lod_town_scale` counts each agent's `agent_tier.tier` every tick and reports the
share of agent-ticks per tier, on a calm village (10K) and the full collapse cascade
(4320 ticks, density variants):

**Calm village (10K ticks):**

| N | seed 42 Focal% / Second% / **Background%** | seed 7 Focal% / Second% / **Background%** |
|---|---|---|
| 12 | 46.6 / 53.4 / **0.0** | 41.3 / 55.6 / **3.2** |
| 48 | 40.0 / 60.0 / **0.0** | 32.8 / 65.6 / **1.7** |
| 96 | 33.0 / 67.0 / **0.0** | 31.5 / 68.5 / **0.0** |
| 192 | 26.5 / 73.5 / **0.0** | 47.7 / 52.3 / **0.0** |
| 256 | 27.0 / 73.0 / **0.0** | 34.5 / 65.5 / **0.0** |

**Collapse crisis (full cascade):**

| N | Focal% | Second% | **Background%** |
|---|---|---|---|
| 12 | 54.5 | 38.0 | **7.5** |
| 48 | 42.9 | 43.2 | **13.9** |
| 96 | 36.4 | 54.7 | **8.9** |

## Verdict — the tier is a crisis modulator, not a scale lever

- **Background is dark in every calm town at N ≥ 96 (0.0 %)** and negligible below it
  (≤ 3.2 %): calm villages never push importance under the 0.22 entry gate (i348), so the
  aggregate rung simply does not engage. This **confirms i359's suspicion with numbers**.
- **Background is live only under crisis** (7.5–13.9 % of agent-ticks in the collapse
  cascade) — the rung is entered when stress collapses low-status agents' importance.
- The Focal→Secondary gradient *does* scale (Focal 46.6 % → 27.0 % as N grows) — that is
  the tier system working — but the *aggregate* rung the scaling strategy was built
  around is a wartime/epidemic phenomenon, not a steady-state one.

**Consequence:** the LOD tier provides **≈0 scale saving in the common (calm) regime** and
~9–14 % occupancy in crisis. Combined with i355 (the biology/action LOD predicates are
documented debt, not wired) and i328 (the wired cognitive gate's measured payoff ≈5 %),
the honest reading is: **the tier system is a fidelity/cost modulator, not the engine's
scaling mechanism** — scaling rides the housing/contact and pass-cost work (i331–i352,
i359), not the tiers.

## Recorded discrepancy

i348 measured Background at **25–40 %** of crisis-world agent-ticks; this probe measures
**7.5–13.9 %** on the collapse cascade. The gap is scenario/horizon/window dependent
(i348 spanned *pestilence* runs and different N); both agree qualitatively — Background is
a crisis-only rung. No pin depends on the exact figure.

## Verification

Probe + docs only; **no source changed**. Golden byte-identical, `gate --full` GREEN.
