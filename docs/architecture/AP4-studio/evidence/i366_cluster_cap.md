# Iteration 366 — the cluster cap never binds; the divisor is the knob

**Status:** MEASUREMENT + DECISION (premise refuted, no source changed) · **Item owned:**
the i360/i362 open item — "does raising `cluster_count_for` to 5–6 buy more polities
without crowding a density-law world?"

## The arithmetic, measured

`cluster_count_for(houses) = (houses/16).clamp(1, 4)`. The cap can only bind when
`houses/16 > 4`, i.e. `houses ≥ 80`. On the density world law that needs **N ≥ 320** —
**above `MAX_POPULATION = 256`**:

| N | side | houses | raw (houses/16) | K | cap binds? |
|---|---|---|---|---|---|
| 96 | 46 | 24 | 1 | 1 | no |
| 116 | 50 | 29 | 1 | 1 | no |
| 144 | 56 | 36 | 2 | 2 | no |
| 192 | 64 | 48 | 3 | 3 | no |
| **256** | 74 | 64 | **4** | **4** | **no** (raw == cap, not above) |
| 320 | 83 | 80 | 5 | 4 | yes — unreachable |

**The cap is inert across the entire reachable population.** Raising it to 5–6 changes
nothing at any N the engine can run. `verdict = CAP_NEVER_BINDS`.

## What the town actually is (settlements @ gap 8, 2K liveness)

| N | side | K | settlements | health | hunger | gini |
|---|---|---|---|---|---|---|
| 116 | 50 | 1 | 13 | 0.797 | 0.0199 | 0.317 |
| 144 | 56 | 2 | 2 | 0.782 | 0.0179 | 0.357 |
| 192 | 64 | 3 | 3 | 0.799 | 0.0185 | 0.374 |
| 256 | 74 | 4 | 4 | 0.772 | 0.0190 | 0.400 |

At the clustered sizes (K ≥ 2) settlements track K exactly. The **N=116 / K=1** row is an
observation, not a defect: K ≤ 1 routes through the unchanged i345 **uniform** spiral, and
an evenly-spread field partitions into ~13 near-singleton clusters at a tight gap (it
would merge to 1 at gap ≥ 12). So the uniform and clustered regimes answer the partition
question differently — which is fine, because only the clustered regime is the town.

## Decision

**The real knob is the divisor (16), not the cap.** To reach 5–6 settlements at N=256
(64 houses) the divisor would have to fall to ~10–12 — a *different* change that trades
village size for village count, and is behavioural/sweep-carrying. Since the clustered
town already makes the i296–i299 stack end-to-end live (i360/i362), a divisor sweep is
**queued as an experiment, not taken** — and the cap-raise item is **closed as refuted**.

## Verification

Probe + docs only; **no source changed**. Golden byte-identical, `gate --full` GREEN.
