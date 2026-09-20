# Iteration 336 — the appraisal pass's O(N²) population folds, hoisted

**Status:** LANDED · **Item owned:** i335's pre-registered #1 — "incremental
per-agent aggregates for the appraisal folds (five full-list scans per agent per
tick) — the i334-class repair, value-identical by construction, and the tick's
largest pass".

## What the profiler said

i335 attributed the quadratic floor and found the tick's largest pass to be
`appraisal` (**1 227.8 µs/tick at N=192, α 1.937**). Reading the pass end to end
found **three per-agent scans of the whole population** — and one list walked
four separate times:

| site | shape | per-tick bound |
|---|---|---|
| `avg_status` (shame from social comparison) | avg of `effective_status()` over **all j ≠ i**, per agent | **O(N²)** |
| `max_other_status` (envy) | max of `effective_status()` over all j ≠ i, per agent | **O(N²)** |
| `max_other_anger` (disgust / moral outrage) | max of `emotions[j].anger` over all j ≠ i, per agent | **O(N²)** — see below |
| `relationship_v2s` mean trust ×3, minimum trust ×1 | four folds of the agent's own list | O(N) each, ×4 |

`effective_status()` is not a field read — it is a weighted computation — so
these scans were expensive *and* redundant.

## The repair

Two of the three population scans and all four own-list folds are **pure
reductions over state this pass does not write**, so computing them once is
**value-identical**:

- **`avg_status`** → `(status_total − own) / (n − 1)`. `Fixed` addition is exact
  integer arithmetic at these magnitudes (n × 1.0 raw ≪ `i32::MAX`), so
  subtracting one's own term recovers the excluded sum exactly.
- **`max_other_status`** → a one-pass top-2 with the argmax index: the population
  max, except when `i` *is* the argmax, where it is the runner-up — which is also
  correct under ties.
- **The four own-list folds** → one `fold` yielding `(sum, min)`; the empty-list
  branch (`0.3` fallback) and the `.max(1)` divisor guards are preserved verbatim.

**`max_other_anger` is deliberately NOT hoisted.** The loop writes
`emotions[i].anger` at line 360 while the fold at line 602 reads every other
agent's anger — so it reads a *mixed* pre/post-update population by index. That is
order-dependent by construction; hoisting it would change values. It is the
pass's remaining quadratic term and is recorded as such.

## Measured result

Same probe (`i330_pass_profile`, seed 42, 32×32, warmup 400, window 200,
accumulated means), baseline measured in this session by swapping in `HEAD`'s
`appraisal.rs`:

| N | whole tick before | after | Δ | appraisal before | after | Δ |
|---|---|---|---|---|---|---|
| 96 | 1 488.7 | 1 317.3 | **−11.5%** | 313.9 | 70.7 | **−77.5%** |
| 192 | 6 126.1 | 4 936.7 | **−19.4%** | 1 274.4 | 274.0 | **−78.5%** |

Per-pass exponents (`i335_pass_exponents`, 48 → 192): appraisal **1.937 → 1.655**
and it drops from the tick's **largest** pass (1 227.8 µs) to ~5th (248.6 µs,
−79.8%). Whole-tick α 1.779 → **1.746**; fixed-world cost at N=192 5 923.3 →
4 724.7 µs/tick. `cognitive` is now the largest pass (1 138.1 µs, α 1.993).

## Blast radius — ZERO

The hoists are value-identical by construction, and the gates confirm it rather
than assume it: **golden 9/9 byte-identical**, **full suite 309 passed / 0 failed
/ 1 ignored** — including the 50K long-horizon and 100K birth-pipeline tests that
would expose any drift the short windows hide. **No re-anchors, no snapshot
regeneration.**

## Verdict

**`APPRAISAL_POPULATION_FOLDS_HOISTED`** — the tick's largest pass was 78% pure
redundancy; two of its three O(N²) scans are gone and the four repeated list walks
are one. The pass is still superlinear (it keeps `max_other_anger` and the
own-list walk over a complete graph), and the tick is still governed by the
complete relationship graph (i335).

## Recorded next (in order, evidence in hand)

1. **`cognitive` — now the largest pass (1 138.1 µs, α 1.993).** `cognitive.rs:908`
   walks every row of every agent's `relationship_v2s` each tick just to find the
   few that are `is_active_this_tick`; dormant rows already decay daily. Needs an
   exactness argument (or a touched-row index) before landing.
2. **`·derived+belief` (449.7, α 2.001) and `trust_sync+reset` (352.7, α 2.061)**
   — the same "is this actually necessary pair work, or an accident" question the
   last four iterations kept answering with "accident".
3. **`max_other_anger`** — order-dependent; only a re-contract (pre-pass snapshot,
   behavioral) can remove it. Not attempted.
4. The contact-driven sparse store (i335/#3) remains the structural fix.

**Probes:** `i335_pass_exponents`, `i330_pass_profile`.
