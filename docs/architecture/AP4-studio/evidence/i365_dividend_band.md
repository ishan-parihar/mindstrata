# Iteration 365 — the dividend's improvement is structural across the seed family

**Status:** MEASUREMENT (probe only, no source changed) · **Item owned:** the i363
open item — "the dividend share is a first calibration (two-seed improvement); a
multi-seed band would confirm".

## Why this matters

i363 landed the council surplus dividend and measured Gini **0.647 → 0.611** (seed 42)
and **0.652 → 0.619** (seed 7) — below the pre-fix plateau on both. But two seeds is
exactly the lucky-seed risk doctrine §4.1 forbids relying on. This probe widens the
family to **six seeds** on the identical harness (46×46, N=48, 50K), reporting the final
Gini, the bottom-half share, and the **council treasury** — the hoard the fix was aimed
at draining (i361 measured 163 602 unspent at seed 42 pre-fix).

## Result — every seed is below the pre-fix plateau

| seed | gini | bottom-half | council treasury | health | hunger | grain |
|---|---|---|---|---|---|---|
| 42 | 0.6108 | 11.6% | 240.7 | 0.745 | 0.0178 | 2.25 |
| 7 | 0.6189 | 11.5% | 262.9 | 0.734 | 0.0219 | 5.75 |
| 1 | 0.4952 | 18.0% | 454.1 | 0.799 | 0.0189 | 3.17 |
| 99 | 0.5894 | 10.2% | 298.2 | 0.777 | 0.0102 | 3.96 |
| 13 | 0.5512 | 16.4% | 366.5 | 0.782 | 0.0160 | 5.09 |
| 23 | 0.6062 | 13.2% | **25 273.9** | 0.768 | 0.0203 | 6.79 |

**Band: mean Gini 0.5786, max 0.6189 — against the pre-fix plateau ~0.647–0.652.**
Every seed sits below the old plateau, the bottom-half share is 10.2–18.0 % (was 8.8–10.3 %),
and provisioning is healthy throughout (health 0.734–0.799, low hunger, grain present).
`verdict = IMPROVEMENT_STRUCTURAL`.

## Recorded observation (debt)

**Seed 23 retains a 25 274-coin council hoard** — ~100× the other seeds (241–454). The
drain is structural on 5 of 6 seeds, so this is not a failure of the mechanism, but it
shows the surplus can still accumulate when the council's inflow outpaces a single
centum's 25 % payout (likely a membership/legitimacy regime where a few wealthy members
are taxed hard). Recorded as a systemic-debt entry: the dividend is a *proportional*
share of the surplus per collection cycle, so a very large single inflow between cycles
can leave a residual. A treasury **ceiling** (rather than a per-cycle share) would bound
it deterministically — queued, not taken, because it is another behavioural change.

## Verification

Probe + docs only; **no source changed**. Golden byte-identical, `gate --full` GREEN.
