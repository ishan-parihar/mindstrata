# Iteration 335 — the anatomy of the quadratic floor: a complete relationship graph

**Status:** LANDED (measurement + instrumentation, **zero behavior change**) ·
**Item owned:** the ledger's #1 remaining scale item — "the structural Ω(N²) floor …
breaching it needs a spatial/index structure, not a scan removal".

## The instrument was the first problem

i330's profiler printed **one tick** per run. At N=192 one tick's per-pass sample
carries ~±15% run-to-run noise (memory at the same N/tick measured 988 102 ns, then
1 174 637 ns, then 1 024 093 ns across three runs) — enough to mis-attribute a term,
which is precisely the failure mode i294's single log-log fit already paid for.

i335 makes the opt-in profiler **accumulate**: `mark!` records `(name, ns)` into a
process-global sink (a diagnostic, never serialized, never read by a pass, cannot
touch RNG or ordering), read back as `Simulation::pass_profile_totals()` sorted by
cost, with `pass_profile_reset()` so a probe can measure several N in one process.
`MINDSTRATA_PROFILE_TICK` still enables it; the numeric value is retained only for
backward compatibility. Off by default at the same cost as before (one
`Instant::now()` per tick).

## Per-pass local exponents (α, 48 → 192, accumulated means)

`i335_pass_exponents`: seed 42, 32×32, 400-tick warmup, 200-tick window.

| pass (mean µs/tick) | N=48 | N=96 | N=192 | α |
|---|---|---|---|---|
| appraisal | 83.8 | 317.1 | 1 227.8 | **1.937** |
| cognitive | 69.9 | 247.4 | 1 151.5 | **2.021** |
| memory | 90.4 | 283.9 | 994.8 | 1.730 |
| ·derived+belief | 28.4 | 101.4 | 460.5 | 2.010 |
| trust_sync+reset | 20.8 | 77.9 | 378.1 | 2.093 |
| rel_traces | 37.9 | 103.5 | 346.6 | 1.596 |
| social_pass | 28.9 | 81.6 | 311.4 | 1.715 |
| kinship_daily | 12.0 | 37.7 | 171.9 | 1.923 |
| social_cluster | 17.6 | 48.7 | 155.7 | 1.572 |
| biology | 32.0 | 66.7 | 143.9 | 1.083 |
| +speech_acts | 7.8 | 26.7 | 119.7 | 1.968 |
| action | 16.9 | 40.1 | 72.8 | 1.053 |
| writeback | 14.9 | 30.6 | 67.5 | 1.088 |

Whole-tick α (48 → 192) = **1.779**. The tick is not one hot pass; it is ~8 passes
between 150 µs and 1.2 ms, nearly all superlinear.

## The decisive measurement: the graph is complete

The same probe measures structure at steady state:

| N | world | relationship edges | edges / N | edges / N² | events/tick |
|---|---|---|---|---|---|
| 48 | 32×32 | 2 256 | 47.0 | 0.979 | 57.0 |
| 96 | 32×32 | 9 506 | 99.0 | 1.031 | 113.8 |
| 192 | 32×32 | 37 056 | 193.0 | 1.005 | 224.3 |

`edges ≈ N(N−1)`: the relationship store is a **complete directed graph**, so
`edges/agent ≈ N − 1` exactly.

It is not a density artifact. Holding density constant (the world scaling with the
population — N=48/32×32 is the i295 charter's baseline), the probe measures edges
2 256 → 9 120 → 36 672 for N = 48 → 96 → 192, i.e. **edges α = 2.011 and tick-cost
α = 2.011**. Space is not the governor; the graph's completeness is.

Root cause located in one place: `population.rs` **creates a `Relationship` for every
ordered pair (i, j), i ≠ j, at populate** (random trust 0.3–0.7,
`interaction_count: 0`, `kind: Stranger`), mirrors it into every agent's
`relationship_v2s`, and the birth path adds an edge from every newborn to every
existing agent (`trust 0.4`). Locality was never applied to the store — the same
"village-wide awareness" assumption i334 removed from attention, one layer down.

## What this corrects (i326 re-contract, §4.4)

i326 **demoted** the sparse relationship store on two measurements: `interaction_count
== 0` is only 46–53% of edges (so the best case is ~2×), and "the three per-tick matrix
traversals are **<1% of tick cost**". Neither number was wrong; the scope was too
narrow. The per-edge passes are **not** three traversals — they are `trust_sync+reset`
(378) + `rel_traces` (347) + `·derived+belief` (461, mostly per-edge) + `kinship_daily`
(172) + `social_cluster` (156) ≈ **1.5 ms of 5.9 ms (26%)** at N=192, and the two
largest passes (appraisal + cognitive, **41%**) are quadratic because **each agent
folds its full N−1 `relationship_v2s` list every tick** — `appraisal.rs` does so at
five separate sites per agent per tick, and `cognitive.rs:908` walks every row each
tick just to find the few that are `is_active_this_tick`.

So the recorded lever ranking is corrected: the complete graph is **the** quadratic
driver, and the fix is the one the architecture already implies.

## Verdict

**`QUADRATIC_FLOOR_IS_A_COMPLETE_RELATIONSHIP_GRAPH`** — the remaining superlinear
cost is not a spatial-access pattern and cannot be fixed by a grid; it is an all-pairs
data structure being traversed by per-edge and per-agent-fold work.

## Recorded next iteration (pre-registered, evidence in hand)

1. **Incremental per-agent aggregates** for the appraisal folds (five full-list scans
   per agent per tick) — the i334-class repair, value-identical by construction, and
   the tick's largest pass. `avg_rel_quality`/`avg_trust`-style folds have an
   incremental form fed by the existing `tick_rel_snapshot` deltas.
2. **Per-tick active-row iteration** in `cognitive.rs:908` — the per-tick loop only
   needs rows touched this tick; dormant rows already decay daily.
3. **A contact-driven sparse store** (the i326 lever, now correctly ranked): the
   complete graph is created at populate, and 46–53% of its edges are never touched
   afterwards. This one is behavioral and needs its own probe + re-anchor sweep — it
   changes every system that reads initial trust.

## Verification

Zero behavior change: **golden 9/9 byte-identical**, full suite green, snapshot
untouched — the iteration touches instrumentation and probes only. The profiler
remainder (unmarked work) is printed so the pass table can never silently exceed the
tick.

**Probes:** `i335_pass_exponents` (new), `i330_pass_profile` (reworked to accumulated
means).
