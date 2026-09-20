# Iteration 320 — the daily boundary carried two O(N²)+ relationship accidents

**Status:** LANDED (**behaviour-identical**, golden 9/9 byte-identical) · **Root
cause owned:** the daily-boundary (`tick % 144`) relationship passes. i294's
phase leg measured the boundary at "+132% over fast ticks but 0.8% amortized" at
N=192 and recorded it as second-order. This iteration re-measures it, finds the
absolute capture is dominated by two algorithmic accidents of the i294 class, and
removes both without moving a single value.

## What was measured (`i320_daily_relationship_scan`, release, 2K ticks, seed 42)

Per-tick timing bucketed by cadence; boundary mean drops the first sample
(cache-cold, i294 method).

| N | fast µs/tick | daily-boundary µs/tick | excess |
|---|---|---|---|
| 96 | 4 570.6 | 15 782.8 | **+245.3%** |
| 192 | 38 757.6 | 93 923.0 | **+142.3%** |

The boundary tick costs **3.4× a fast tick** at N=96 — not a rounding term. The
amortized share is only 1.5% (13 boundary ticks per 2 000), so this was invisible
to the i294 headline; but it is the term that grows fastest and would dominate
first once the envelope moves past N≈96.

## Accident 1 — the mean-reversion scan was O(N·R)=O(N³)

`systems/cognitive.rs` ran the §5.1 legacy relationship mean reversion *inside*
the per-agent loop (`for i in 0..agents.len()`), and each agent re-scanned the
ENTIRE `relationships` matrix filtering `rel.from == i`.

Fix: hoist to one O(R) pass after the loop. Each row's drift reads only that
row's own trust, and the old `from == i` filter visits every row exactly once
across the i-loop, so the pass is value-for-value identical — no accumulation,
no order dependence, no RNG.

## Accident 2 — the kinship max-relatedness BFS allocated and re-walked per pair

The daily `kinship_penalty` fold (§10.4) called
`kinship_graph.transitive_coefficient(i, j)` for **every** adult `j` — i.e. O(N²)
calls per boundary tick, each one **re-walking the graph and allocating a fresh
`HashSet` + `VecDeque` + `Vec`**. The BFS itself is query-independent: it never
terminates early on `b` and expands each node at most once, so the BFS from `i`
computes the same `best[]` for every `j`.

Fix: new `KinshipGraph::transitive_coefficients(a, n_agents) -> Vec<Fixed>` (one
BFS, all targets); `transitive_coefficient(a, b)` now delegates to it. The caller
does **one** BFS from `i` and takes the max over adult `j`. Bit-identical: same
BFS, same per-`j` values, same max.

## Result

| N | boundary before | boundary after | Δ | excess before → after |
|---|---|---|---|---|
| 96 | 15 782.8 | 14 399.2 | **−8.8%** | +245.3% → **+204.7%** |
| 192 | 93 923.0 | 77 127.9 | **−17.9%** | +142.3% → **+97.2%** |

The absolute boundary-cost drop *grows with N* (−8.8% at 96, −17.9% at 192), the
signature of removing a superlinear term. Isolating the legs: the mean-reversion
hoist alone gave 15 782.8 → 14 932.3 at N=96 and 93 923.0 → 82 534.2 at N=192;
the BFS batching supplied the remainder. (Fast-tick means drift ±5% run-to-run on
this host, so the boundary **absolute** is the robust signal, not the excess %.)

The residual boundary excess (+205% at N=96) is the legitimate daily work: the
O(N²) §11.2 power-balance fold and the culture/faction/norms daily passes — those
are *supposed* to run daily, not accidents.

## Verification

- **Referee: golden 9/9 byte-identical** before and after (behaviour-identical
  refactor, per §7's split discipline).
- New unit test `transitive_coefficients_match_single_pair_calls` asserts, for
  every ordered pair in a small bidirectionally-linked family graph, that the
  batched single-BFS value equals the per-pair call — the exact bit-identity
  claim the daily fold relies on.
- `cargo fmt` clean, clippy 0 warnings, `scripts/gate --full` **GATE GREEN**
  (309/0/1). Zero re-anchors, zero snapshot drift.

**Probe:** `cargo run -p mindstrata-benches --release --example i320_daily_relationship_scan`
