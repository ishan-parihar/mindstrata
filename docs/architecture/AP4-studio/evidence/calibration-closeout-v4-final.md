# Calibration + Optimization Closeout v4 — FINAL (2026-08-31, HEAD d76aaef)

Owner: QA + DESIGN + PLATFORM. Status: **FINAL — system operational,
all IC-8 floors PASS, no further changes pending**.

## What this session landed (commits this stretch)

| SHA | Change | Perf impact |
|---|---|---|
| `96ea2c6` | lore archetype hot-path fix (capacity check vs collect) | N=12 10K 5955→8357 (+40%) |
| `b0c80dd` | cumulative event_count re-contract | N=12 10K 8357→10092, N=48 10K 621→786 (+27%) |
| `b897d9c` | pre-allocated per-tick buffers (socialization, innovations) | ~0.8 MB allocations saved, pattern uniform |
| `d76aaef` | v3 closeout documentation | — |

**No functional/operational regressions** across this stretch: 308/0/1
suite GREEN, golden 5/5 byte-identical, fmt+clippy clean, bench_index
27 ok / 36 legacy / 0 violations.

## What was tried and reverted (ponytail, not in the tree)

1. **Amortized event-buffer ring-trim** (start of tick, if buffer >
   8192 drop first 4096) — at N=48 the buffer fills every ~80 ticks
   so the trim fires often; the O(n) drain shifts ate the cumulative-
   counter gains. N=48 10K dropped 753→700. Reverted.
2. **VecDeque<SimEvent> refactor** — proper O(1) front pop, but the
   slice syntax `&self.events[start..]` doesn't translate cleanly
   (VecDeque returns `*mut T` not `&[T]`); would touch ~15 push sites
   and 5 read sites. Reverted; recorded as DC-2 work.
3. **Inlined metrics_snapshot aggregates** (replacing `summaries`
   Vec allocation with 7 inline f64 sums) — at N=48 10K snapshots
   were slightly faster but N=12 10K regressed from 9644→9042
   tps. Per-tick cost vs per-snapshot cost didn't pay off.
   Reverted.
4. **Direct `events.clear()` at end of tick** — recovered N=12 10K
   5955→9952 tps (+67%) and N=48 10K 519→742 tps (+43%) but broke
   the golden because `metric_hash` includes `ms.event_count`
   (which used to read the residual length; the cumulative-counter
   re-contract is the durable fix).

## Final state at HEAD d76aaef

### Calibration
| Layer | Verdict |
|---|---|
| Q1 dark_addiction | **CALIBRATED** (pending() correct per i269+i286+i287+i288) |
| Q2/Q3/Q4 quadrants | **CALIBRATION-PENDING** (no events at seed 42) |
| 0.08/0.02 nudges | **CALIBRATED** for subtle regime (6-15% of social driver) |
| 0.10 polarity bias | **STRUCTURALLY ZERO** (projection-bound; mono-subtle v1) |

### Optimization (i270)
| Horizon | tps | IC-8 floor | Status |
|---|---|---|---|
| N=12 2K | 10623 | — | — |
| N=12 10K | 9644 | 8000 | **PASS** (21% headroom) |
| N=48 2K | 1061 | — | — |
| N=48 10K | 753 | 700 | **PASS** (8% headroom) |

### Suite & gates
* `bash scripts/gate`: **GATE GREEN 308/0/1** (golden 5/5)
* `cargo fmt --all`: clean
* `cargo clippy --workspace --quiet`: clean
* `cargo test -p mindstrata-development --lib`: 69/69
* `cargo test -p mindstrata-sim --lib`: 208/208
* `cargo test -p mindstrata-tui --lib`: 48/48
* `python3 scripts/bench_index.py --strict`: 27 ok / 36 legacy / 0 violations

## Why we stop here

The DC-1 system is **operationally calibrated and optimized**:

1. **Field engine live** — i269 measures dark_addiction moving
   0.0→0.49 over 20K ticks, monotonically. The 0.08/0.02 nudges
   in `crates/mindstrata-sim/src/actions/mod.rs:914-920` are
   active in compute.
2. **Polarity wired** — `system_polarity_claim_emit` and the
   orchestrator pass at `crates/mindstrata-sim/src/systems/
   development.rs` project claims and reconcile. The 0.10 bias
   at `actions/mod.rs:804-811` is structurally present but
   zero-effective due to projection-bound mono-subtle v1.
3. **All IC-8 floors pass** — N=12 10K at 9644 (vs floor 8000) and
   N=48 10K at 753 (vs floor 700).
4. **Calibration evidenced** — Q1 dark_addiction moves
   measurably, nudge coefficients are within 6-15% of social
   driver, polarity projection is correctly documented as the
   bottleneck for Era III.
5. **Suite green at every commit** — no functional regression
   introduced by any of the perf or calibration work.

**Ponytail debt** (recorded, not blocking):
* VecDeque refactor for events buffer (secondary perf gain
  available, would touch ~20 sites).
* N=48 10K additional 8% headroom (snapshot capture dominates).
* Q2/Q3/Q4 calibration at a different seed (norm violations +
  deaths firing).
* Polarity projection enrichment (DC-2 Era III proper).

These are scoped and prioritized for the next cycle, not the
current operational target.

## Sign-off

* `bash scripts/gate` at HEAD d76aaef: **GATE GREEN 308/0/1**
* `i270_perf_snapshot` N=12 10K: 9644 tps (PASS 8000)
* `i270_perf_snapshot` N=48 10K: 753 tps (PASS 700)
* `i269_pathology_signature`: dark_addiction mean 0.30 (5K), 0.49 (20K) monotone
* `i288_q2q3q4_calibration`: Q1 monotone 0.30→0.41 5K→20K
* `i290_polarity_4k_baseline`: avg_active_tension=0.0000 (projection-bound)
* `i291_event_count_diff`: cumulative counts (riverford 15633, collapse 175372)
* `i292_metric_hash`: riverford c958712fed59b62a, collapse a8b5ed42609feea7

`evidence/calibration-closeout-v4-final.md:1` is the final
calibration + optimization sign-off for the DC-1 system. This
document supersedes v1, v2, and v3 closeouts.
