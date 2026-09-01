# Perf Investigation: Per-Tick Event Bus (2026-08-31)

Owner: PLATFORM. Status: **ROOT CAUSE DOCUMENTED, FIX DEFERRED** —
golden-coupled, needs a metrics_snapshot API change.

## Symptom

i270 perf probe at HEAD `96ea2c6`:
- N=12 2K: 7374 tps
- N=12 10K: 8357 tps
- **N=48 2K: 879 tps**
- **N=48 10K: 621 tps** (IC-8 floor 700, 11% short)

The N=48 10K degradation is the last IC-8 floor not met.

## Root Cause

`self.events: Vec<SimEvent>` in `Simulation` is never bounded. At
N=48 the per-tick event volume is ~80-200 (vs ~12-30 at N=12), so
after 10K ticks the buffer holds ~1-2 million SimEvent values
(~3.5 MB). Every tick:

1. `metrics_snapshot()` reads `self.events.len()` for `event_count`
2. The `recent_events(n)` API slices into the buffer
3. The catalyst observer walks `&self.events[start..]`
4. The buffer is the per-tick message bus: `pre_tick_events..self.events.len()`

At 10K ticks with N=48, the buffer accesses dominate the per-tick
cost (~50% of elapsed time per i270).

## Attempted Fixes

### Attempt 1: `self.events.clear()` at end of tick

Replaced with `clear()` (keeps capacity, resets length). i270 results:
- N=12 2K: 7323 → 11066 tps (+50%)
- N=12 10K: 5955 → 9952 tps (+67%)
- N=48 2K: 684 → 1054 tps (+54%)
- N=48 10K: 519 → 742 tps (+43%)

**Gate failed**: golden_replay `metric_hash` includes
`ms.event_count` (line 62 of golden_replay.rs). The clear makes the
post-tick event_count zero instead of the last-tick's small residue.
Hash changes: `df1bcdaed6747a26` → `98c867ba71583be5`. Reverted.

### Attempt 2: Ring-trim to EVENT_RING_CAP=4096

Same metrics_snapshot coupling — `event_count` reads the residual
length, which is now capped at 4096 instead of growing. Hash changes.
Reverted.

## The Real Fix (Deferred)

The proper fix is to make `metrics_snapshot.event_count` the
**cumulative** event count (new field on `Simulation`, incremented in
`core::tick` after the events pass), and let `self.events` be a
bounded ring buffer (cap ~4096). Then:

1. `metrics_snapshot.event_count` is read from the cumulative counter
   → identical to current
2. `recent_events(n)` slices the ring buffer (still works)
3. The per-tick message bus window is `pre_tick_events..self.events.len()`
   (still works as long as `pre_tick_events` is taken *after* the
   trim, not before)
4. Catalyst observer uses the watermark approach (already does)

This requires:
- Add `total_event_count: u64` to `Simulation`
- Increment it after the events pass
- Update `MetricsSnapshot` to read from this counter
- Add the ring trim (carefully placed so `pre_tick_events` accounting
  survives)
- Re-anchor golden: `event_count` value will be the same (cumulative),
  but the per-tick buffer state is different — the `metric_hash` for
  snapshots taken mid-run might differ slightly. A targeted re-anchor
  is needed.

Estimated effort: 2-4 hours + golden re-anchor + 1 commit.

## What was fixed (this iteration)

The **metric_history.remove(0)** O(n) shift at the end of tick was
replaced with `drain(..drop_n)` (O(drop_n), keep capacity). This
fixes a smaller N=48 10K regression without breaking the golden.

The **lore archetype backfill** hot-path fix (commit 96ea2c6) is the
bigger win — recovered N=12 10K from 5955 to 8357 tps (40%).

## Sign-off

* `bash scripts/gate`: GATE GREEN 308/0/1 (golden 5/5)
* `i270_perf_snapshot` N=12 10K: 8357 tps (PASS IC-8 floor 8000)
* `i270_perf_snapshot` N=48 10K: 621 tps (FAIL floor 700 — 11% short)
* `cargo fmt --all` clean
* `cargo clippy --workspace --quiet` clean

`evidence/perf-investigation-events-buffer.md:1` is the DC-2 pref
investigation for the N=48 10K floor gap.
