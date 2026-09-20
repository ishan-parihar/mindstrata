# Iteration 321 — the i307 fatigue residual is a village-wide phase cycle, not drift

**Status:** LANDED (**measurement only, no behaviour change**) · **Root
question owned:** i307's finding-2 left "fatigue p90 still 0.359 @50K" as an
unexplained residual. The open question was which failure class it is:

- a **horizon attractor** — the share at the 1.0 ceiling climbs with horizon
  because relief cannot keep pace with accrual (the i318 Allergy class, where
  the long-horizon equilibrium is the horizon and differentiation collapses); or
- a **live equilibrium tail** — a workload-differentiated distribution that is
  stable across horizons.

There is a third class, and the probe shows this is it: **population-wide phase
synchronization**. The village as a whole oscillates between a
synchronized-rested state and a synchronized-fatigued state, and the fatigue
reading at any sampled tick is dominated by that shared phase rather than by the
individual agent.

## What was measured (`i321_fatigue_pace`, release, N=12, 12-seed family)

Final-tick distributions per horizon, plus an in-run trajectory at seed 42
(sampled every 10K ticks) to separate in-run oscillation from horizon drift.

| horizon | mean | sd | p10 | p50 | p90 | ≥0.90 | sd/mean |
|---|---|---|---|---|---|---|---|
| 2 000 | 0.3550 | 0.1217 | 0.1957 | 0.3747 | 0.4614 | **0/144** | 0.34 |
| 20 000 | 0.0841 | 0.1572 | 0.0110 | 0.0125 | 0.2977 | **0/156** | 1.87 |
| 50 000 | 0.3598 | 0.1260 | 0.2219 | 0.3752 | 0.4500 | **0/187** | 0.35 |
| 100 000 | 0.2650 | 0.1044 | 0.2406 | 0.2659 | 0.3101 | **0/237** | 0.39 |

**No ceiling is ever approached** — 0/… agents at ≥0.90 and 0/… at ≥0.99 at
every horizon, with max 0.75–0.81. The i318 attractor signature (share within 5%
of the ceiling growing monotonically with horizon) is absent.

### The in-run trajectory is the decisive leg (seed 42, every 10K)

| t | mean | sd | p10 | p50 | p90 |
|---|---|---|---|---|---|
| 10 000 | 0.1356 | 0.2564 | **0.0000** | **0.0000** | 0.6105 |
| 20 000 | 0.1332 | 0.2108 | 0.0030 | 0.0125 | 0.3463 |
| 30 000 | 0.1306 | 0.0850 | 0.0427 | 0.1436 | 0.1441 |
| 40 000 | 0.2426 | 0.0752 | 0.1768 | 0.2659 | 0.3073 |
| 50 000 | 0.4075 | 0.1454 | 0.3719 | 0.3847 | 0.5466 |
| 60 000 | 0.1207 | 0.2454 | **0.0000** | **0.0000** | 0.7218 |
| 70 000 | 0.0915 | 0.2016 | **0.0000** | **0.0000** | 0.3959 |
| 80 000 | 0.0569 | 0.1217 | 0.0010 | 0.0125 | 0.0600 |
| 90 000 | 0.1479 | 0.0547 | 0.1357 | 0.1441 | 0.1671 |
| 100 000 | 0.2824 | 0.1242 | 0.2532 | 0.2642 | 0.3787 |

The mean swings **0.05 ↔ 0.41** *within one run*, and the shape alternates
between two extremes:

- **synchronized-rested**: `p10 == p50 == 0.0` with `p90 ≈ 0.4–0.7` (t=10K/60K/70K)
  — half the village at exactly zero while a tail carries fatigue;
- **synchronized-fatigued**: a tightly-clustered band (`p10 0.136 / p90 0.167`
  at t=90K; `p10 0.0427 / p90 0.1441` at t=30K) — the whole village at one level.

So the horizon-to-horizon swings in the family table (0.355 → 0.084 → 0.360)
are **sampling phase**, not drift: the measurement lands on a different part of
the same in-run cycle. The between-seed p90 spread at 20K (0.021 → 0.553) is the
same fact seen across seeds.

## Verdict

`FATIGUE_PHASE_SYNCHRONIZED_NOT_SATURATING`

The i307 residual is **dispositioned, not smoothed away**: the channel is
bounded and never saturates, no re-anchor or parameter move is warranted, and
the remaining calibration question it exposes is a design one — whether the
shared circadian/work cycle should carry the fatigue channel *less* than
per-agent workload does, so that individual differentiation survives the
village's phase. That is recorded as debt for a future behavioural iteration
(§4.5: a knife-edge-adjacent equilibrium is recorded, not flip-flopped).

## Verification

- Measurement probe only — **no source change**, so `golden` is byte-identical
  by construction and no snapshot is touched.
- `cargo fmt` clean, clippy 0 warnings, `scripts/gate --full` **GATE GREEN**.

**Probe:** `cargo run -p mindstrata-benches --release --example i321_fatigue_pace`
