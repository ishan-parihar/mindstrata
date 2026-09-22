# Iteration 357 — the locomotion pace is NOT cadence-limited (a plan premise refuted)

**Status:** MEASUREMENT (probe + decision) · **Item owned:** the plan's B-item
"locomotion pace (one step/decision is slow — size decision cadence vs movement)",
following i347's recorded open question.

## What the code actually does

`ActionKind::Move`'s `duration_ticks` is **1**, so `action_progress` returns to 0 on
the same tick and the selection chain **re-decides every tick**. The §6 movement pass
(`core.rs`) then steps one Manhattan tile per tick while `current_action` is `Move`.
So i347's phrasing — "approach is one Manhattan step per *decision*" — is imprecise:
the pace is **one step per tick** whenever the branch wins, i.e. already maximal. There
is no cadence term to fix.

## Probe (`i357_locomotion_pace`, seeds 42/7/11, N=48, 46×46, 20 000 ticks)

| seed | Move duty (agent-ticks) | feud-open | of feud-open ticks, anger > 0.02 | episodes | with movement | start dist p50/p90 |
|---|---|---|---|---|---|---|
| 42 | 0.113% | 4.05% | **3.2%** | 71 | 10 (14%) | 0 / 5 |
| 7 | 0.048% | 6.03% | **0.9%** | 108 | 8 (7%) | 0 / 5 |
| 11 | 0.069% | 2.03% | **3.9%** | 37 | 5 (14%) | 0 / 5 |

When an episode *does* move it nets a mean of 4.3–17.9 steps (max 116–253) — consistent
with one step per tick over a multi-tick angry window, not with a throttled walk.

## Verdict

**`LOCOMOTION_PACE_IS_NOT_CADENCE_LIMITED`** — the cadence premise is refuted. The two
real limiters are:

1. **The anger gate's reachability.** Only **0.9–3.9%** of feud-open ticks pass
   `anger > FEUD_APPROACH_ANGER (0.02)`. Anger is an *acute* emotion, while a feud is a
   chronic relationship; the branch only fires in the acute window after a triggering
   event. This is the shape i347 deliberately calibrated (0.02 ≈ p96 of the calm
   distribution), so it is design-consistent — approaching is something an angry agent
   does, not something a feud makes an agent do continuously.
2. **Co-location.** Feud targets start at **Manhattan distance 0 (p50)** and ≤5 for 90%
   of episodes — a feud's parties usually already share a house. Approach is moot for
   most feuds, which is the i338/i340 housing/contact finding (and A9) surfacing in the
   action layer, not a locomotion defect.

**Disposition:** no code change. The plan's locomotion-pace item is **closed as
refuted**; the pace needs no calibration. Two design options are recorded, **not**
taken:

- *Chronic-feud proximity anger* — elevate anger while a feud party is within the
  perception radius, so an ongoing hostile relationship motivates approach in the
  absence of a fresh trigger. Same design-act shape as A8/i356, would carry a sweep.
- *Co-location* — A9 (charter): a fixed 32×32 world cannot spread 48 agents above
  ~4 per house, so "approach" is a small fraction of feud time by construction.

## Verification

Probe + docs only; **no source changed**. Golden replay byte-identical, no re-anchors.
(`i357_locomotion_pace` clippy-clean, bench law conformant.)
