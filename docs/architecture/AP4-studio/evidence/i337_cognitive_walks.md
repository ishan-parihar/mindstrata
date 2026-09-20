# Iteration 337 — the cognitive pass sub-profiled: one traversal instead of two

**Status:** LANDED (small perf win, zero behavior change) · **Item owned:** the
requested "sub-profile the cognitive pass and remove its remaining redundant
per-agent work".

## The sub-profile

i335 named `cognitive` the tick's largest pass after i336 (1 138–1 358 µs/tick at
N=192, α ≈ 1.99). Five sub-marks were added inside its second per-agent loop
(which fires once per agent, so the probe reads each share from the accumulated
**total**, not `ns/samples` — `i330_pass_profile` now prints both):

| mark | µs/tick @N=192 | share of tick | per call |
|---|---|---|---|
| `·cog quality fold` | 464.3 | 8.7% | 2.406 µs |
| `·cog decay walk` | 447.3 | 8.4% | 2.317 µs |
| `·cog pre-decay` (gap) | 18.6 | 0.3% | 0.097 µs |
| `·cog inst-rank` | 5.6 | 0.1% | 0.029 µs |
| `·cog head` (loop overhead) | 4.1 | 0.1% | 0.021 µs |

**The two per-agent walks over `relationship_v2s` are 911.6 µs of the pass's
1 357.9 µs — 67% of the pass and ~17% of the whole tick.** Everything else in the
loop is small; the remaining ~418 µs is the genuinely per-agent psychology work
(motivation, attachment, regulation, prospection gating, moral cognition) in the
*first* loop, which the marks do not cover and which is not redundant.

## A plan assumption corrected (i335, §4.4)

i335's recorded plan item #2 said "`cognitive.rs:908` walks every row each tick
just to find the few that are `is_active_this_tick`; dormant rows already decay
daily". Probe `i337_cognitive_walks` measures the actual dirty share:

| N | total rows | dirty rows | share |
|---|---|---|---|
| 48 | 2 256 | 801 | 35.5% |
| 96 | 9 506 | 3 044 | 32.0% |
| 192 | 37 056 | 9 721 | **26.2%** |

So ~74% of rows *are* dormant per tick — the filter is selective after all (my own
working estimate before measuring was ~98%, i.e. wrong; the measurement is the
point). But **selectivity is not where the cost is**: the walk's price is the
*pure memory traversal* (26% of rows visited to do work, 100% visited to look).
An explicit touched-row index would therefore save the traversal — but it is a
positional index over `relationship_v2s`, and positions are re-derived on every
birth/death (`rebuild_relationship_v2s_after_death`, the append path in
`births_deaths.rs`), so the index would have to be invalidated on any population
change — which is every tick in a growing world. Recorded as **debt with its
disqualifying hazard**, not built.

## What was removed

The quality fold and the dirty-decay walk both walked the same list once per
agent per tick, and **nothing between the two sites reads or writes those rows**
(verified statement-by-statement across the whole range). They now fuse into
**one traversal**: each row's `quality()` is read before that row's own decay
(the old fold read every row before *any* decay, and one row's decay cannot
affect another row's quality), the summation order is unchanged, and `Fixed`
addition is order-exact at these magnitudes. The daily `clear_dirty` rides along
on the same tick. `·cog row sweep` = **759.7 µs/tick**, vs 911.6 µs for the two
walks — **−152 µs/tick (−17% of the two walks)**.

## Blast radius — ZERO

**golden 9/9 byte-identical; full suite 309/0/1**, including the 50K and 100K
horizon tests. No re-anchors, no snapshot regeneration.

## Measurement discipline (a caveat that matters for the next iteration)

Whole-tick `µs/tick` at N=192 on this machine swings **±5–10% between processes**
(three consecutive runs of the *same binary*: 4 881 / 5 385 / 5 418; four runs of
the fused binary: 4 794 / 4 809 / 5 113 / 5 343). A single-shot whole-tick
comparison is therefore **not** a valid A/B — my own first read of this change
said "slower by 7.1%" and the repeats refuted it.

The durable lesson: **compare in-run pass marks (same process, adjacent in time),
not whole-tick totals.** On that signal the fusion is a clear −17% on the two
walks; on the whole tick it is inside the noise, which is why it is recorded as a
small win rather than a headline one. i336's whole-tick figures carry the same
caveat — its *appraisal* figure (−78.5%, a pass mark) is the solid one.

## Verdict

**`COGNITIVE_WALKS_FUSED`** — the pass's 67% is inherent O(R) arithmetic over a
complete graph, not redundant computation; the removable part was one of the two
traversals, and it is gone. The remaining lever is the graph itself (i335) — the
contact-driven sparse store.

## Recorded next

1. **Contact-driven sparse store** — the structural fix (now the top item).
2. **Positional dirty-row index** — debt, disqualified by birth/death position
   reuse until the store is sparse (a sparse store makes the index stable, so
   these two are coupled and should land together).
3. **Per-row arithmetic** (`decay()` recomputes `decay_rate × Fixed::from_int(1)`
   and two derived products per row) — a precomputed-constants variant would be
   value-identical; measured headroom is inside this iteration's noise, so it
   needs the interleaved methodology above before it is worth landing.

**Probes:** `i337_cognitive_walks` (new), `i330_pass_profile` (now reporting
per-call means from accumulated totals).
