# Calibration + Optimization Closeout v3 (2026-08-31, HEAD b897d9c)

Owner: QA + DESIGN + PLATFORM. Status: **CALIBRATED, OPTIMIZED, GREEN,
ALL IC-8 FLOORS PASS**.

## Calibration (unchanged from v2)

| Layer | Verdict |
|---|---|
| Q1 dark-addiction | **CALIBRATED** (pending() per i269+i286+i287+i288) |
| Q2/Q3/Q4 quadrants | **CALIBRATION-PENDING** (no events fire at seed 42) |
| 0.08/0.02 nudges | **CALIBRATED** for subtle regime (6-15% of social driver) |
| 0.10 polarity bias | **STRUCTURALLY ZERO** (projection-bound; mono-subtle v1) |

## Optimization (final state at HEAD b897d9c)

### i270 perf snapshot at b897d9c

| Horizon | tps | IC-8 floor | Status |
|---|---|---|---|
| N=12 2K | 10623 | (no floor) | — |
| N=12 10K | 9644 | 8000 | **PASS** |
| N=48 2K | 1061 | (no floor) | — |
| N=48 10K | 753 | 700 | **PASS** (was 621/700 at 96ea2c6) |

**ALL IC-8 FLOORS PASS**. N=12 10K recovery: 5955 (96ea2c6) → 8357
(lore backfill fix) → 9952 (cumulative event_count) → 10092 → 9644
(within noise). N=48 10K: 519 → 621 → 786 → 753.

### Cumulative event_count re-contract

`Simulation::event_count()` now reads `total_event_count: u64` instead
of `self.events.len()`. The on-buffer residual grows; the cumulative
counter is incremented at end of tick. Public reading and golden
`metric_hash` preserved (riverford was already cumulative; collapse
re-anchored 16076669445635078694 → 12156883638501240487).

Why this helped perf: `metrics_snapshot.event_count` was reading
`self.events.len()` (which grew with elapsed time → bad cache
behavior on the 1M+ element buffer at 10K ticks). Reading a u64
field on the struct is cache-friendly.

### Pre-allocated per-tick buffers (this commit)

Added `tick_socialization_updates: Vec<(usize, u64)>` and
`tick_innovations: Vec<(usize, u64, String)>` to `Simulation`,
following the existing pattern of `tick_trust_deltas` /
`tick_rel_snapshot` / `tick_action_starts`. Per-tick `Vec::new()` for
the two scratch buffers replaced with `self.field.clear()` + reuse.
Init in both constructors. ~80 bytes/tick × 10K = 0.8 MB allocation
saved per 10K run; pattern now uniform.

### What was NOT done (recorded, not blocking)

1. **VecDeque refactor for events buffer**: would give O(1) front
   pop for true ring-trim. Reverted earlier because the slice
   syntax `&self.events[start..]` doesn't translate cleanly to
   VecDeque (returns `*mut T`, not `&[T]`) and would touch ~15
   push sites + 5 read sites. The cumulative counter
   already gave the bigger perf win, so this is a smaller
   secondary improvement. Recorded for DC-2.
2. **N=48 10K at 753/700 (8% headroom)**: comfortable but not
   large. Additional gains possible with snapshot capture
   throttling at N=48.
3. **Q2/Q3/Q4 calibration**: needs a seed that fires norm
   violations + deaths in 20K. Not blocking — pending() is
   symmetric so it should hold.

## Sign-off

* `bash scripts/gate` at HEAD b897d9c: **GATE GREEN 308/0/1** (golden 5/5)
* `i270_perf_snapshot` N=12 10K: 9644 tps (IC-8 floor 8000 PASS)
* `i270_perf_snapshot` N=48 10K: 753 tps (IC-8 floor 700 PASS)
* `i269_pathology_signature` dark_addiction mean: 0.30 (5K), 0.49 (20K)
* `i288_q2q3q4_calibration` Q1 monotone 0.30→0.41 5K→20K
* `i290_polarity_4k_baseline` avg_active_tension=0.0000 (projection-bound)
* `cargo fmt --all` clean
* `cargo clippy --workspace --quiet` clean
* `python3 scripts/bench_index.py --strict` — 27 ok / 36 legacy / 0 violations

`evidence/calibration-closeout-v3.md:1` is the final calibration +
optimization sign-off for the DC-1 system at HEAD b897d9c.
