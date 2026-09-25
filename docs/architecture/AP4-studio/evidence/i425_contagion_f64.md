# i425 — the fear-contagion f64 quantization fix (landed)

**Iteration:** i425 · **Date:** 2026-09-25 · **Status:** LANDED (with custody re-baseline)
**Probe:** `crates/mindstrata-benches/examples/i425_contagion_calm_band.rs`
**Root cause class:** §5 Fixed-4 truncation disease (the i393 migration exposed it; the i403 arc preserved the fix in `stash@{0}`, exonerated it on the crisis corpora, and deferred it pending exactly this calm-band measurement)

## 1. Root cause

`RelationalFields::contagion_delta` computed `(perceived_stress * rate)` with
`Fixed::mul`, which **floors** at `SCALE = 10_000` (one output quantum = 1e-4).
With `FEAR_CONTAGION_RATE = 0.05`, every product from `perceived_stress < 0.002`
(= 1e-4 ÷ 0.05) truncated to **exactly zero**: the entire low-stress range
contributed nothing to the daily fear fold (`systems/decay.rs`, pass 7, daily
cadence `tick % 144 == 0`). The §5-prescribed repair — compute in f64, quantize
once — is the one-line fix:

```rust
Fixed::from_f64(perceived_stress.to_f64() * rate.to_f64()).clamp_01()
```

The contract (identity-at-zero, monotone in both arguments, clamp, no RNG) is
unchanged; the existing unit test
(`contagion_delta_is_identity_at_zero_and_monotone`) passes unmodified.

## 2. Measured decomposition (the probe)

The probe defines the repaired band **empirically** — `old == 0 && new > 0`,
never by the edge constant — and runs both code paths per agent-day on the
recorded fold-boundary stresses of the calm corpora (the riverford golden world
seed 42/1000 ticks/16×16/N=12, plus calm seeds 7 and 11).

**Synthetic header (the discriminators):** stress 0.0005 → old 0, new 0 (below
`from_f64`'s 5e-5 rounding threshold); stress 0.0010–0.0019 → old 0, new 1
quantum (the band); stress 0.0020 → old 1, new 1. So the repaired band is
**[0.001, 0.002)** — the arithmetic edges are `Fixed::mul`'s floor (products
< 1e-4 → 0, i.e. stress < 0.002) and `from_f64`'s round-to-nearest (products
< 5e-5 → 0, i.e. stress < 0.001).

**Calm occupancy (72 agent-days per seed = 6 folds × 12 agents):**

| seed | zero-stress | band days | round-up days | equal days | first-order gain/agent (mean) |
|-----|-----|-----|-----|-----|-----|
| 42 (golden corpus) | 0.00% | 0 | 52 | 20 | 0.000433 |
| 7 | 0.00% | 0 | 30 | 42 | 0.000267 |
| 11 | 0.00% | 1 | 45 | 26 | 0.000375 |

(Arm-to-arm the counters differ by ≤ 1 day — the recorded fold-input
distributions are effectively identical; the counters are measurements of the
fold inputs, not of the arm.)

**Two difference classes, not one.** The repaired band fires rarely (0–1 of 72
agent-days). The dominant per-fold class is **floor → round-to-nearest**: any
product with fractional residue ≥ 0.5 quantum gains +1 under the fix, which
fires on 30–53 of 72 agent-days per seed. Per-fold old-vs-new difference is
therefore ≤ 1 quantum per agent, every fold.

**A/B fold window (riverford seed 42, the tick-144 fold):** pre-fix mean
population fear at t=144 is 0.287800, fixed 0.287900 (**+1 quantum**). The
difference persists through the post-fold decay (still +1q at t=150) and
re-aligns by t=151 — the increment is **not washed** (the 500-tick snapshot's
+1q avg_stress, 68 ticks past the last fold, confirms persistence), but
quantization re-absorbs it within ~7 ticks of decay.

**End-state A/B (the decisive leg):**

| corpus | pre-fix end avg_fear | fixed end avg_fear | verdict |
|-----|-----|-----|-----|
| riverford 1000 ticks (seed 42) | 0.299775 | 0.299775 | **identical** |
| calm seed 7, 1000 ticks | 0.149342 | 0.149342 | **identical** |
| calm seed 11, 1000 ticks | 0.211758 | 0.211758 | **identical** |
| collapse 4320 ticks | 0.294131 | 0.294154 | **+2.3 quanta** |

All three calm 1000-tick end states are **identical across arms** — the
horizon ends 136 ticks past its last fold (864), and the ≤1q offsets have
re-aligned. Collapse ends **on** a fold tick (4320 = 30 × 144): the final
fold's quantum difference is in the end state. `agent_count` (13) and
`event_count` (40 170) are identical across arms in collapse — mortality and
the event stream did not move.

**Crisis exoneration — the outcome, measured (the input story was
REFUTED by the census):** the 10-seed pestilence revolution sweep read
**identical per-seed revolution counts** in both arms (total 2, seed 11
only). The inherited mechanism claim ("crisis stress 0.2–0.8 → products
100–400 quanta, so the class is ≤1%") is **wrong as a distribution claim**:
the probe's collapse fold-boundary census (374 agent-days, 29 folds) reads
**mean stress 0.1770, min 0.0000, with 103/374 (27.5%) of products below
20 quanta** — the ≤1-quantum difference classes fire in crisis worlds too,
including zero-stress days (where both paths contribute exactly 0). The
measured fact is OUTCOME-insensitivity: the crisis engine's decisions do
not sit on sub-quantum fear margins, so identical revolution counts follow
— not input-absence.

## 3. Custody (§4.2: named mechanism, measured drift)

The suite on the fix-only tree read **307/4** — exactly the four artifacts the
mechanism predicts:

1. **`golden_replay_crisis_vs_baseline` (collapse)** — `metric_hash`
   8993835882449929447 → 11698195916283842933. **Only this field moved**:
   `event_hash`, `agent_hash`, `agent_count`, `total_grain`, `total_water` are
   all identical. Recomputed via `i276_golden_recompute` (which covers BOTH
   scenarios — riverford's recomputed block reproduces its stored baseline
   byte-for-byte, the custody no-op check).
2. **`metrics_500_ticks` / `metrics_2000_ticks`** — `avg_stress` +1 quantum
   population-wide each (0.2848083 → 0.2848167 = exactly 1e-4 ÷ 12): the
   persistent fold difference, measured.
3. **`long_horizon_surface_10000_ticks`** — `total_memory_traces` **constant
   at 789**; the kind stock redistributed: Emotional 447 → 443, Episodic
   15 → 14, Social 309 → 314; every other field (avg_stress, event_count,
   trust, relationship counts) identical. **Named mechanism** — survivorship,
   not selection: the ≤1q fear difference at each of the 69 folds propagates
   through the fully-in-code chain appraisal `arousal = (fear + anger + joy) ×
   0.5` (`systems/appraisal.rs:836`) → encode `emotional = arousal × 0.6 +
   0.1` (`sim/memory_ops.rs:319`) → decay `emotional_bonus = charge × 0.1` and
   `evict_weakest` strength ordering (`psych memory.rs`) → quantum-margin
   eviction tie-breaks swap which 5 of 789 traces survive. A constant total
   with redistributed kinds and identical end-state aggregates is that
   signature; an upstream interaction-kind change would have moved event
   composition and totals.

Post-custody suite: **311/0/1** (goldens 5/5, snapshots green).

## 4. Corrections to the i403 record

- The i403 record said the fix "moves **both** goldens" — measured now: **only
  the collapse golden moves**; riverford reproduces its stored baseline
  bit-for-bit (phase alignment, §2). The i403 evidence doc and the plan row are
  corrected in this commit.
- The i403 stash entry `stash@{0}` is fully landed here; `stash@{1}` (the
  rejected arc) remains the re-attempt's preserved code.

## 5. Instruments

- `i425_contagion_calm_band` — synthetic band discriminators; 3-seed calm
  occupancy with band/round-up/equal class counters; riverford fold-window
  series (t=138..152); collapse final-window series (t=4308..4319) + end
  state. One binary runs on both arms (the class counters read the recorded
  fold inputs; the end-state prints read the arm's trajectory).
- `i276_golden_recompute` — both golden baselines (custody tool; riverford
  block doubles as the no-op check).
- Suite: `cargo test -p mindstrata-tests --lib --release` — 307/4 pre-custody,
  311/0/1 post.

## 6. What this does NOT change

- No behavioural pin moved; no calibration constant touched; the fold's
  contract is identical. The crisis engine is untouched (identical per-seed
  revolution counts).
- The i403 re-attempt preconditions (belief-charge formation-time probe,
  per-channel trust-gate re-derivation) are unaffected — this landing removes
  the deferred companion, nothing more.
