# Iteration 305 — fulfillment thresholds (difficulty-levers row 2, threshold half)

**Status:** LANDED · **Root cause owned:** `docs/balance/difficulty-levers.md`
row 2 is "Need decay / fulfillment thresholds". i303 promoted the decay half;
the threshold half — the five goal-generation gates (`retain 0.3`,
`Eat/Drink 0.5`, `Rest 0.6`, `Socialize/Worship 0.7`) — stayed as inline
consts in `systems::system_goal_generation` with no executable surface, so the
band could not reach them and `docs/balance/needs-bands.md`'s threshold rows
(which name these exact consts as "empirical, NOT theory-cited …
CALIBRATION-PENDING(AP3)") had no runtime contract.

## What landed

1. **`SimParameters.goal_gate_scale`** (`mindstrata-core/src/parameters.rs`) —
   serde default = identity via the `scale_identity` helper; `with_difficulty`
   writes 0.6 (Lenient) / 1.0 (Standard) / 1.4 (Harsh), the same multipliers as
   the row's decay half.
2. **`systems::GoalGates`** — the five fulfillment thresholds + the retain
   bound, resolved ONCE per tick from the run's params (`GoalGates::CANON` holds
   the pre-i305 values as consts). `system_goal_generation` gained a `params`
   argument and reads the resolved gates instead of inline literals.
3. **3 sim pins, 1 core pin** (canon identity, band scaling incl. every
   scaled gate's Fixed-4 exactness, and a behavioural pin: an agent at 0.55
   hunger generates an `Eat` goal under Lenient/Standard and none under Harsh).

Standard is the canon gates bit-for-bit (`x * 1.0` in Fixed and IEEE-754 both),
so the promotion carries **no golden/snapshot re-anchor** — proven by the gate's
byte-identical golden replay.

## The horizon trap this iteration caught (methodology finding)

The first probe run measured at 2000 ticks only and read three need-driven
producers — `Eat`, `Socialize`, `Worship` — as **dead** (rate 0.0000), with a
distribution table showing each channel's max *below* its gate. That reading was
an artifact: need decay runs at 1e-4–2e-4 per tick, so a need starting near 0.1
only crosses a 0.7 gate around tick 6000. At 20K all seven goal kinds are live:

| goal kind | 2000 ticks | 20000 ticks |
|---|---|---|
| Eat | 0.00000 | 0.00888 |
| Drink | 0.00489 | 0.08258 |
| Rest | 0.05223 | 0.12518 |
| Work | 0.29678 | 0.26839 |
| Socialize | 0.00157 | 0.00426 |
| Worship | 0.00000 | **0.41517** |
| SeekSafety | 0.27025 | 0.26748 |

No producer is dead at every measured horizon. Reporting a single horizon would
have produced a false dead-producer finding and a wrong fix (re-anchoring gates
to a 2K distribution). Every producer claim in this iteration is therefore made
at both horizons, and the liveness criterion is evaluated per horizon.

## Exit evidence — probe, 12-seed family, 2K + 20K

### Leg 1 — configuration surface

| Check | Before i305 | After |
|---|---|---|
| `SimParameters` goal-gate keys (lenient↔standard / standard↔harsh) | 0 / 0 | 1 / 1 |
| resolved gates, lenient | — | eat 3000, drink 3000, rest 3600, socialize 4200, worship 4200, retain 1800 |
| resolved gates, standard | — | 5000 / 5000 / 6000 / 7000 / 7000 / 3000 (canon) |
| resolved gates, harsh | — | 7000 / 7000 / 8400 / 9800 / 9800 / 4200 |

### Leg 2 — canon distribution (raw units: 1 raw = 1e-4)

| channel | p50 | p75 | p90 | p99 | max | canon gate | above_gate (20K) |
|---|---|---|---|---|---|---|---|
| hunger | 0.0250 | 0.0580 | 0.1370 | 0.4130 | 1.0000 | 0.50 | 0.0088 |
| thirst | 0.1160 | 0.1780 | 0.3520 | 1.0000 | 1.0000 | 0.50 | 0.0826 |
| fatigue | 0.1446 | 0.3727 | 0.6196 | 1.0000 | 1.0000 | 0.60 | 0.1078 |
| social | 0.0048 | 0.0102 | 0.0172 | 0.1956 | 1.0000 | 0.70 | 0.0036 |
| meaning | 0.4711 | 0.9757 | **1.0000** | 1.0000 | 1.0000 | 0.70 | 0.3242 |

### Leg 3 — the row-2 band with BOTH halves moving (reported, not claimed)

| horizon | lenient duty | standard | harsh | monotone |
|---|---|---|---|---|
| 2000 | 0.2304 | 0.0587 | 0.0943 | **no** |
| 20000 | 0.7045 | 0.6361 | 0.6410 | **no** |

The combined band is non-monotone in goal duty because row 2's two halves push
the same way: Harsh both accumulates deficit faster (1.4× decay) and tolerates
more of it (1.4× gates), so harsh's duty lands marginally above standard's. The
row's felt effect was already promoted in i303 on the standing deficit
(aggregate +27%, monotone). This iteration therefore promotes the threshold half
on an observable where only the threshold half moves.

### Leg 3b — threshold half IN ISOLATION (decay rates pinned at canon)

| gate scale | Eat | Drink | Rest | Socialize | Worship | need-driven duty | alive |
|---|---|---|---|---|---|---|---|
| 0.6 (lenient) | 0.0050 | 0.0641 | 0.2512 | 0.0035 | 0.4402 | **0.7639** | 12/12 |
| 1.0 (standard) | 0.0089 | 0.0826 | 0.1252 | 0.0043 | 0.4152 | **0.6361** | 12/12 |
| 1.4 (harsh) | 0.0048 | 0.0556 | 0.0797 | 0.0025 | 0.3703 | **0.5130** | 12/12 |

At 2000 ticks: 0.2457 > 0.0587 > 0.0190. **Monotone at both horizons**, 12/12
alive in every band. A low-gate village responds to smaller deficits, so its
need-driven goals run more often — the catalog's "abundant vs scarcity-driven"
axis, measured on the half that owns it.

| Leg | Verdict |
|---|---|
| Surface band-sensitive | PASS |
| Standard ≡ canon (params serde + byte-identical golden replay) | PASS |
| No producer killed by a band (per-horizon canon-material criterion) | PASS |
| Isolated threshold scaling monotone, both horizons | PASS |
| 12/12 seeds alive × 3 bands × 2 horizons | PASS |

**Verdict: `GOAL_GATE_BANDS_LIVE`.**

## Findings recorded, not smoothed

1. **The meaning channel saturates at 20K (p90 = 1.0000, above_gate 0.32).**
   Its decay (1e-4/tick after §5 Fixed-4 truncation of 0.00015) outruns its only
   relief (0.1 per Worship action), so by ~9K ticks most agents sit pinned at
   1.0 and the `Worship` goal runs in **41.5%** of agent-ticks — with a priority
   equal to that pinned pressure, the largest single goal-alignment bonus in the
   utility function. Meanwhile `Socialize` runs at 0.43% and `Eat` at 0.89%. The
   goal layer's duty cycles are therefore wildly uneven at the calibrated
   horizon. This is a *producer*-side calibration problem (need decay vs relief
   balance), not a gate problem: it will re-anchor the meaning equilibrium pins,
   so it belongs in its own behavioural iteration — recorded here as the next
   candidate, with this table as its baseline.
2. **At 2000 ticks the Harsh band drops the `Drink` producer to ~0** (thirst
   only reaches the 0.7 gate in the tail at that horizon). That is the harsh
   semantics working, and the producer is live again at 20K (0.0556); it is
   recorded rather than smoothed into a wider band.
3. **The row-2 band's own duty cycle is not monotone** (leg 3) — recorded above
   with its mechanism, so no future iteration re-derives the two halves as if
   they were independent.
4. `docs/balance/needs-bands.md`'s threshold rows are updated to cite the
   measured distributions and the new surface; the ranges it proposed for
   survival/belonging/meaning gates are **above the ranges those channels
   occupy at the calibrated horizon**, which is why the gate values were left at
   canon rather than moved to the spec's proposed bands.

## Verification

- core lib **34/34** (1 new pin); sim lib **270/270** (3 new pins); tests-crate
  release suite **309/0/1** (unchanged — no existing pin moved).
- fmt clean; clippy 0; full `scripts/gate --full` **GATE GREEN** with golden
  baselines **byte-identical** (zero re-anchors — Standard is the canon gates
  bit-for-bit).

## Queue after this iteration

- **Meaning-channel balance** (finding 1): the next behavioural candidate; needs
  its own probe + honest re-pins.
- **Row 3 residuals**: ceiling band; Q2 dark-allergy saturation.
- **Row 1**: stays prohibited (AGENTS §5 H5).
- DC-4 continues with the CLIENT asset-viewer and the graphical shell spike.
