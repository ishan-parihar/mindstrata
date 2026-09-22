# Iteration 354 — long horizons are already unlocked; the `VecDeque` conversion is not a prerequisite

**Status:** DONE (measurement + decision, no code change) · **Item owned:** the plan's
"convert the append-only event buffer to a bounded `VecDeque` so >250K-tick horizons become
runnable."

## The claim under test

The plan treated the buffer conversion as the gate on long-horizon (city-scale *time*)
runs. But **i327 already bounded the rolling buffer** with an amortized bulk drop —
`MAX_EVENTS = 262_144`, peak retained `2×MAX ≈ 28 MiB` — and explicitly recorded
`VecDeque<SimEvent>` as a **deferred** upgrade path in a `ponytail:` comment whose only
remaining motivation is jitter-free ticks (a sub-ms stall every ~5k ticks).

So the right move was to test the horizon rather than assume it (`§3`: observable output is
the only success signal).

## What the probe measured (`i354_long_horizon`)

A real 250 000-tick run, N=12, seed 42, release:

| Metric | Value |
|---|---|
| wall time | **55.3 s** (221 µs/tick at the grown population) |
| population at end | 12 → **36** (births; all 36 with positive health) |
| mean health | 0.805 |
| partnered | 12/36 |
| cumulative events | 6 243 115 |

The run completes in bounded memory and the village stays alive and reproducing across a
quarter-million ticks. **`VERDICT_LONG_HORIZON_RUNS`.**

## Verdict

**REFUTED as a prerequisite.** Long horizons are already runnable; the memory wall the
conversion existed to fix was closed by i327 five iterations ago. What remains of the item
is the jitter ceiling i327 already named — a sub-ms bulk-drop stall every ~5K ticks — which
has **not** been observed to matter for any consumer (no pass requires jitter-free ticks).

**Disposition:** keep `VecDeque<SimEvent>` **deferred**, with its trigger restated honestly:
*"revisit only if a pass ever needs jitter-free ticks, or a real-time/interactive horizon
exposes the bulk-drop stall."* Do **not** spend an iteration on it as a horizon prerequisite
— that work is already done.

## Ledger

1. Plan item "i354 VecDeque conversion" is **closed as refuted**; the deferred trigger is
   rewritten to the jitter-only condition.
2. New measured datapoint for the scale table: **250K ticks is a ~1-minute village run**
   (N=12→36), so long-horizon operator scenarios are available *now*.
3. Method note (third of the epoch, after i353, i338/i350): **plan items can be stale too.**
   The doc-alignment system now covers docs; this is the same failure in a plan line —
   probe the claim before executing it.

## Verification

Probe + docs only; **no source changed**. Golden 5/5 byte-identical, `gate` GREEN.
