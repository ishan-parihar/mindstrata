# Iteration 326 — sizing the scale levers: the event buffer is the wall, not the matrix

**Status:** LANDED (**measurement only, no behaviour change**) · **Levers owned:**
i316's ranked list — (1) sparse relationship store, then the event-buffer
VecDeque trigger.

## Why measure before refactoring

i294/i316 both name the Ω(N²) dense `relationships` matrix as *the* structural
cost floor and rank a sparse store as lever #1. Both the sparse-store refactor
and the event-buffer data-structure change are large (the latter "touches every
`self.events.push/get` site", per the debt note in `core::tick`). Neither should
be attempted on a ranking alone, so this iteration sizes both from live state
first (§2).

## What was measured (`i326_relationship_sparsity`, release, 4 seeds)

**Leg 1 — matrix sparsity.** `Relationship::interaction_count == 0` is the
store's own "never touched since populate" signal.

| N | horizon | R = N(N−1) | never touched | evolved kind | sparse reduction |
|---|---|---|---|---|---|
| 12 | 20 000 | 170 | **46.0%** | 54.0% | 46% |
| 24 | 2 000 | 552 | **53.3%** | 46.7% | 53% |
| 24 | 20 000 | 691 | **47.9%** | 52.1% | 48% |
| 48 | 2 000 | 2 304 | **52.3%** | 47.7% | 52% |
| 48 | 20 000 | 2 552 | **49.9%** | 50.1% | 50% |

The 50% split is **stable across N and horizon** — half the matrix is ballast,
half is live. So the best case for a sparse store is a **2× reduction**, not an
order of magnitude.

**And it does not address the bottleneck.** The three per-tick traversals of the
matrix (the pre-tick snapshot capture, the trust-delta build, the provenance
change-scan) are **< 1% of tick cost** (0.57–0.83% across the legs). The
superlinear tick cost i294 measured lives in the *interaction/product* passes,
not in traversing the matrix.

**Leg 2 — the event buffer.** `Simulation::events` is **never trimmed** (a
documented, deliberate debt: `core::tick` records that a `drain(..drop_n)`
ring-trim regressed N=48 10K from 742 → 524 tps on the O(n) shift, and names
`VecDeque<SimEvent>` as the proper fix).

| N | horizon | events buffered | size @56 B/event |
|---|---|---|---|
| 12 | 20 000 | 274 573 | 14.7 MiB |
| 24 | 20 000 | 544 965 | 29.1 MiB |
| 48 | 2 000 | 108 283 | 5.8 MiB |
| 48 | 20 000 | **1 049 162** | **56.0 MiB** |

Growth is **~O(N) events per tick** with no bound (≈52 events/tick at N=48 → 25.5
events/agent/tick). Extrapolating the envelope: N=96 @100K ticks is ~10–25M
events ≈ **0.6–1.5 GiB**.

## Verdict

`EVENT_BUFFER_IS_THE_SCALE_WALL`

The levers are **re-ranked on evidence**:

1. **Event buffer** — *demoted from "trigger at >250K ticks" to the top lever*,
   and reclassified: it is a **memory** wall (unbounded `Vec<SimEvent>`), not a
   time wall. The recorded fix (`VecDeque<SimEvent>` + a bounded ring) is the
   next iteration.
2. **Sparse relationship store** — *demoted*: at most 2× on a term that is
   < 1% of tick cost. Not worth its blast radius as a scaling fix; it stays
   worth doing only if a *semantic* need appears (e.g. per-pair lazy synthesis),
   which is a different motivation.

This also explains why i316 saw ≈N^1.8 tick scaling while the matrix traverse
was negligible: the time cost is the interaction passes, and the *unbounded*
term is memory.

## Verification

- Measurement only — **no source change**, golden byte-identical by construction.
- `cargo fmt` clean, clippy 0 warnings, `scripts/gate --full` **GATE GREEN**.

**Probe:** `cargo run -p mindstrata-benches --release --example i326_relationship_sparsity`
