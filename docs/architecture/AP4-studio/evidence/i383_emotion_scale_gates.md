# i383 — the acute-emotion arms read their own scale (and a metric-scope correction)

**Status:** LANDED (behavioural; crisis golden re-anchored with mortality unchanged) ·
**Root cause owned:** i379 flagged `emotions.anger > 0.50` as near-dead at town scale and
queued it "re-anchor or delete, explicitly". The bar appears at **two** sites with different
jobs, and the probe's first reading was **wrong about which one was broken**.

## The metric-scope correction (my own error, recorded first — doctrine §4.12)

The first version of the probe measured each arm's open rate per *tick* ("did ANY agent clear
it"), which reported `fear > 0.5` at **100.00% in all 10 worlds** and nearly produced the
verdict "the fear arm is decoration, the whole shock gate is unconditional". Re-measured per
**agent** — the scope the predicate actually runs at — fear's arm opens **15.2–38.8%** of
agent-ticks. A live, discriminating gate. The tick-level figure was an artifact of the wrong
scope, and acting on it would have "fixed" the one arm that was working.

## Measured (probe `i383_anger_channel`, 10 worlds = 5 scenarios × 2–3 seeds)

| quantity (per AGENT-tick) | calm ×3 | towns ×2 | pestilence ×2 | collapse ×2 | famine ×2 |
|---|---|---|---|---|---|
| `anger > 0.5` | 0.00 / 0.00 / 0.29% | 0.00 / 0.00% | 0.00 / 0.00% | 0.00 / 0.19% | 0.00 / 0.01% |
| `fear > 0.5` | 15.1–40.8% | 24.9–31.2% | 23.5–29.6% | 26.0–37.5% | 17.5–32.9% |
| anger per agent | p50 **0.000** in every world; p99 0.011–0.281; MAX 0.178–0.940 | | | | |

So: **anger is the dead arm** (0.00% in 7 of 10 worlds, ≤0.30% in the rest) at *both* sites —
the intention-abandonment shock (`pass_action.rs`, ORed with the live fear arm, hence a
false affordance) and the "high anger → Work (aggressive productivity)" goal emitter
(`systems/mod.rs`, where anger is the only gate, hence a **dead producer** that no test could
see). Fear's arm was measured discriminating and **left alone** — a threshold is a defect only
when it stops discriminating (§4.10).

## Candidate references, sized on the same corpus (the §4.11 rule)

| candidate reference | open rate (per agent-tick) | verdict |
|---|---|---|
| `anger > 1.25 × derived.resentment` | 0.00–3.85% | **refuted** — resentment is an injustice index sitting an order of magnitude ABOVE the acute signal (pooled 0.177–0.283 vs 0.000–0.012), so this bar is as dark as the one it would replace |
| `anger > 1.25 × population mean anger` | **3.1–19.6%** in every world | **shipped** — self-normalizing: 1.25× a near-zero mean in a calm crowd (a genuine spike), rising with the crowd in an enraged one |
| `fear > 1.25 × derived.trauma_risk` | 23.3–40.0% | measured, not needed (fear's absolute bar already discriminates) — recorded for the next audit |

## The law

`EMOTION_SHOCK_RATIO = 1.25` (the same anomaly multiple as i381's panic leg) applied to the
population's own mean anger (`emotion_regulation::mean_anger`, one hoisted per-tick fold —
never re-folded per agent, §6/i336). Both anger arms read it; the Work emitter's priority is
the **excess** over that bar (`(anger − bar) × 0.3`), so the goal scales with the anomaly
instead of pinning to a value below the arbitration bar (the i380 lesson).

## Live result (`SHIPPED i383` rows, same probe re-run)

| | before (absolute) | after (relative) |
|---|---|---|
| anger-sourced Work goals | **0 in 7 of 10 worlds**, ≤1 sample tick in the others | **0.020–0.184 per agent-tick** (2.0–18.4% of agent-ticks) in **all 10** |
| fear-sourced SeekSafety (control) | 0.151–0.408 | 0.151–0.408 (untouched) |
| shock gate's anger arm | 0.00–0.30% | 3.1–19.6% |

## Blast radius

`collapse` golden re-anchored: `metric_hash` 69367314cf77147f → 2f80d7a95d3e5ecd with
**`agent_count` 12 → 12** (mortality identical) and the resource envelope essentially held
(grain 0.3340 → 0.3292, water 1141.5062 → 1141.6559). The shift is goal/abandonment timing in
the collapse cascade — exactly where the arms now decide. The **calm golden is
byte-identical**, **no snapshot drift**, sim **300/300**, integration **312/0/1** (the new pin
is the 312th). `gate --full` GREEN.

New runnable pin `anger_driven_work_is_relative_to_the_population` (three-way): a uniformly
angry population (0.30 everywhere) emits **nothing** (the bar rises with the crowd); one
0.90 spike in a 0.05 crowd emits **for the spiking agent alone**; and a collapse cascade
carries anger-Work goals at all (the dead-producer check).

## Audit refresh (appended to i373)

The anger arms move **Class 4 → Class A**; the fear arms stay **Class C** with the new
measurement (they were flagged by the same census and are *not* defects — the first time the
audit's class label needed splitting between two arms of one predicate). Method corrections
recorded: (a) **match the metric's scope to the predicate's scope** (§4.12) — a per-agent gate
measured per tick will read 100% or 0% for reasons that have nothing to do with the gate;
(b) a dead *tick-level* rate is not a dead *agent-level* rate, and vice versa.

**Still queued from i379's rulings:** `needs.social` (Class 3b — needs the
winner-decomposition census first).
