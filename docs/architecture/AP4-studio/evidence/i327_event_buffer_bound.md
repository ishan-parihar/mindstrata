# Iteration 327 — the rolling event buffer is bounded (the i326 scale wall closed)

**Status:** LANDED · **Item owned:** i326's promoted lever #1 — the unbounded
`Simulation::events` buffer (`EVENT_BUFFER_IS_THE_SCALE_WALL`).

## The defect

`Simulation::events` grew without bound — ~O(N) events per tick, with no trim.
Probe i326 measured **1 049 162 events ≈ 56 MiB at N=48 @20K**, extrapolating to
0.6–1.5 GiB at N=96 @100K. It was a **memory** wall, not a time one (i316's
≈N^1.8 tick scaling has a different cause).

The debt was deliberate and documented in `core::tick`: a `Vec::drain(..drop_n)`
ring-trim had regressed N=48 10K from 742 → 524 tps because `Vec::drain` shifts
the entire tail, and the recorded fix was `VecDeque<SimEvent>`.

## What landed

**An amortized bulk drop.** The buffer is allowed to reach `2×MAX_EVENTS`, then
one `drain` brings it back to `MAX_EVENTS` (`MAX_EVENTS = 262 144`, the perf
charter §4 / ASSET-PIPELINE-v0 ring trigger rounded to a power of two):

```rust
if events.len() > 2 * MAX_EVENTS {
    let drop_n = events.len() - MAX_EVENTS;
    events.drain(..drop_n);
}
```

The memmove is O(MAX) but fires only once per (MAX / events-per-tick) ticks — at
the measured ~52 events/tick (N=48) that is one ~14 MiB move every ~5 000 ticks,
i.e. **amortized ≈O(1) per event**, with peak memory `2×MAX ≈ 28 MiB` instead of
unbounded. It is trimmed at **tick end**, after every pass has read its
`pre_tick_events..` window, so no in-tick index is disturbed; the next tick
re-reads `self.events.len()` as its own base. `total_event_count` carries the
cumulative reading, so the public `event_count()` and the golden `metric_hash`
are unmoved.

### Why not the recorded `VecDeque` fix

Attempted first and measured against the real churn: `VecDeque` has **no range
indexing** (every `&self.events[a..]` read site — ~15 of them — fails to compile)
and **no `push`** (≈40 sites need `push_back`). That refactor is far larger than
the win here, so it is **not attempted piecemeal**; it stays recorded as the
upgrade path for jitter-free ticks. `// ponytail:` at the call site names the
ceiling (the periodic bulk memmove) and the upgrade path.

## Result (`i326_relationship_sparsity`, release, 4 seeds)

| N | horizon | before | after | ms/tick before → after |
|---|---|---|---|---|
| 24 | 20 000 | 544 965 events (29.1 MiB) | **348 345 (18.6 MiB)** | 0.264 → 0.262 |
| 48 | 20 000 | 1 049 162 events (56.0 MiB) | **328 203 (17.5 MiB)** | 1.322 → 1.326 |

The buffer is now bounded regardless of horizon, and **tick cost is unchanged**
(within run-to-run noise) — the amortized drop is free at this cadence.

## Re-contracts (§4.4) — the bounded-journal contract invalidates whole-run scans

`recent_events(n)` is now a window over **recent** history, not the whole run
(documented at the call site; the unbounded reading is `event_count()`). Three
pins in the suite read a 175K/220K-tick run through `recent_events(10_000_000)`
and silently lost their early events (read **0** births). They are re-contracted
to **incremental observation** via a new shared helper
`test_helpers::collect_child_born_ticks` (step the run in 2 000-tick segments,
read each segment's window, filter by that segment's tick range — independent of
any trim). The real invariants are unchanged and still asserted:

- `conception_pregnancy_birth_pipeline_runs_and_is_seed_deterministic` — birth
  volume band (21 births measured, `>=8 && <=30`), every birth post-golden-window,
  mothers' counters account for most deliveries, identical timeline + population
  across two seed-1 runs. The old assertion's mechanism (a whole-run buffer scan)
  is what the charter legitimately invalidates, not its contract.
- `reproduction_conception_multiplier_parameter_is_live` — the doubled-rate FIRST
  birth is no later than baseline's.

This is a **re-contract, not a re-pin**: nothing was widened to accept a weaker
claim; the observation method moved to the bounded-journal contract.

## Verification

- `scripts/gate --full` **GATE GREEN** (309/0/1); **golden byte-identical**,
  zero snapshot drift (every calibrated horizon is far below the bound).
- New unit pin `event_buffer_is_bounded_by_an_amortized_bulk_drop` (at cap →
  untouched, at 2× → untouched, past 2× → one drop to the cap) — runnable without
  a 100K-tick run, via the extracted `trim_event_buffer`.
- `cargo fmt` clean, clippy 0 warnings.

**Probes:** `cargo run -p mindstrata-benches --release --example i326_relationship_sparsity`
