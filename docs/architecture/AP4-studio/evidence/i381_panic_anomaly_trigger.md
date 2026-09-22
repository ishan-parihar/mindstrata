# i381 — the moral-panic trigger is now relative to the population's own charge

**Status:** LANDED (behavioural) · **Root cause owned:** i378 measured the §7.2
trigger's absolute bar (`avg_charge ≥ 0.55`) sitting *inside* the body of its own
input distribution — every firing run cleared it by only 2.5–9.4%, and one swept
crisis seed missed by 0.5%. i379 classified the class (`threshold on a moving
distribution`) and the audit's Class-4 ruling prescribed the form: **measure the
population against its own history, not against a constant.** i380's lesson said
to probe the *repair* before shipping it — which is what found the two things this
iteration actually turns on.

## What the probe measured first (`i381_panic_anomaly_trigger`)

13 worlds × 20 000 ticks, sampling the trigger's two raw inputs **every tick**
(crisis family `{7,1,23,11,46,5,42,13,99,3}` pestilence, plus calm `{42,7,1}`), then
evaluating candidate laws **offline** on the same series.

**Finding 1 — the corpus is not ordered the way i378 assumed.** The calm worlds are
*warm*: `calm/42` holds a mean charge of 0.373 with **85 % of its ticks** above the
ratio leg (0.30), while `crisis/7` holds 0.265 and meets the leg 50 % of the time. The
"panic_ratio is the clean discriminator" reading of i378 was never measured where it
mattered — the ratio leg is *saturated* in the calm world. Both legs are levels, and
the calm level is higher than most crisis levels.

**Finding 2 — what separates the worlds is spikiness, not level.**

| series | p50 avg | p95/p50 | ratio-leg share |
|---|---|---|---|
| `crisis/7#1` | 0.2650 | **1.94** | 50.2 % |
| `crisis/1#0` | 0.3185 | 1.45 | 35.6 % |
| `crisis/11#1` | 0.2704 | 1.35 | 8.2 % |
| `calm/42#0` | 0.3727 | **1.08** | 85.0 % |
| `calm/7#0` | 0.2173 | 1.23 | 3.5 % |

**Finding 3 — the decision gap a level bar has to live in is only 1.31× wide.**
Ranking every world by the highest charge it ever reached: the highest **non-firing**
plateau is `crisis/42` at **0.4188**, the lowest **must-fire** spike is `crisis/11`
(i378's knife-edge seed) at **0.5475**. Its geometric midpoint is **0.471** — the
shipped floor.

**Finding 4 — the cold-start rule matters more than the ratio.** With a baseline
that starts at 0, a stably-warm calm world fires 3–5 spurious panics in its first few
thousand ticks (its history is unmeasured, so the floor governs). Hence
`anomaly_baseline_fold`'s first rule: **a cold baseline adopts its first observation**,
and a zero observation (nobody holds a charged belief) carries **no information** and
is not folded at all.

**Validated against the reference:** the offline evaluator at `floor = 0.55`,
`mult = 1.0` reproduces the shipped law exactly (`crisis 7:11, 1:4, 23:5, 11:0`,
`calm/1: 2`) — so the sweep is measuring the real trigger, not a model of it.

## The law

```
fires ⟺ avg_charge ≥ max(baseline × 1.25, 0.47)   AND   panic_ratio ≥ 0.30

baseline: per proposition, seeded from the first observed mean, then
          EWMA with tau = MORAL_PANIC_COOLDOWN × 10 = 3 000 ticks
```

| constant | value | job | evidence |
|---|---|---|---|
| `MORAL_PANIC_CHARGE_FLOOR` | 0.47 | level arm: cool populations and unmeasured ones | geometric midpoint of [0.4188, 0.5475] → ×1.12 / ×1.165 equal margin |
| `MORAL_PANIC_ANOMALY_RATIO` | 1.25 | relative arm: populations that warm past the crossover | crossover `0.47/1.25` = 0.376, inside the measured warm band |
| `MORAL_PANIC_BASELINE_COOLDOWNS` | 10 | the baseline's memory, **derived** from the panic cadence rather than a fresh magic number | tau 3 000 and 10 000 rows are identical in the grid — the law is insensitive over a 3× band |
| `MORAL_PANIC_COOLDOWN` | 300 | promoted from a function-local const so the two timescales cannot drift apart | — |

**What the new law does to the corpus** (crisis, 20K):

| | old law (0.55) | new law (0.47 × 1.25) |
|---|---|---|
| firing seeds | `{7, 1, 23}` | `{7, 1, 23, **11**}` |
| `crisis/11` (the knife-edge seed) margin | **0.9955** | **1.165** |
| best firing margin | 1.094 | 1.280 |
| calm worlds firing | `calm/1: 2` | `calm/1: 2` (unchanged — a pre-existing spike to 0.55) |
| `crisis/42` (closest must-not-fire) | 0.752 | 0.891 |

The knife-edge seed that started this iteration now clears the bar by 16 % instead of
missing it by 0.5 %, `crisis/42` has *more* room below the bar, and no calm world that
was dark becomes bright.

## What the floor cannot do — the relative arm's own job

The floor is the operating point in **every** world of the corpus (the warmest *stable*
population, `calm/42`, sits at 0.373 — just under the crossover 0.376). So the arm is
sized on **synthetic shapes**, which is the only honest way to state its job:

| shape | old law (0.55) | new law |
|---|---|---|
| flat 0.20 (quiet) | 0 | 0 |
| flat 0.40 (warm, below floor) | 0 | 0 |
| flat 0.65 (sustained, above floor) | **67** | **0** |
| 0.20 → 0.65 step at 5K | **50** | **13** |
| 0.20 with a 300-tick 0.65 spike at 5K | 1 | 1 |

Read the last three rows carefully, because they are a **realism judgement call**:

* A spike on a quiet population still fires exactly once — the crisis semantics are
  preserved.
* A population that **warms into** a sustained high level fires a burst and then goes
  quiet (50 → 13): the baseline rises to meet it. That is what makes a panic an
  *event* rather than a new equilibrium.
* A population that was **never** at any other level fires **zero** times here and 67
  times before. A chronically charged population has no *sudden* collapse to have —
  which is §7.2's actual wording ("sudden collapse in trust"), and is the semantics
  the absolute bar could never express. It is also the saturated-state pathology the
  300-tick cooldown was introduced to blunt. Recorded as a **deliberate design
  consequence**, not a side effect.

## Blast radius (probe `i381_blast_radius`)

The trigger feeds legitimacy damage → faction grievance → coups, so its equilibrium
shift is visible downstream. Two pinned contracts moved; both were attributed before
being touched:

1. **`biology::attachment_separation_distress_coupling_is_live_after_tuning`** — the
   probe names the exact cause: this village fires its first panic at **tick 3014,
   prop 1, avg 0.4713, ratio 0.5417** — inside the new floor's band, below the old
   0.55 bar. One panic re-paces the calm micro-trajectory: non-zero partnered distress
   39/46 (85 %) → **34/48 (71 %)**, while the mean floor and the non-pinning ceiling
   both still hold. **Re-contracted** (not re-pinned): the pinned 3/4 was itself a
   calibration of this village's distress distribution; the invariant is that the
   coupling is live *population-wide*, and 2/3 survives the whole measured 34–39 band.
2. **`governance::revolution_is_regime_change_not_repeat_loop`** — the family
   `{5, 11, 42}` went 3/3/3 → **0/0/3**. A 10-seed sweep (pestilence @70K, meme
   mutation isolated) discovered the firing members: `42→3, 7→1, 23→1`, everything
   else 0. **Re-anchored** onto `{42, 7, 23}` with the liveness bar kept at **≥2 of 3**
   (§4.8: hold the family fixed, discover which members carry it). The three members
   also span the producer's two routes to a coup — seed 7 fires 1 revolution with *0*
   panics, seed 23 fires 1 with 6 — so the family is not simply re-naming a
   panic-driven set.

**Goldens, snapshots, long horizons: byte-identical and drift-free.** `gate --full`
reported zero snapshot changes and all four golden legs green (`snapshot_long_horizon_
surface_10000_ticks` included) — as expected, since the trigger's first in-corpus fire
is at tick 3014.

## Design notes

* **The accumulator is f64, deliberately** (`anomaly_baseline_fold`). A recursive
  `Fixed` update quantizes at 1e-4 *per tick* and accumulates a standing bias of up to
  `1e-4 × tau` = 0.3 — the size of the signal. A unit test pins the truncation
  (`Fixed::from_f64(0.001 * (1/3000)) == ZERO`) so the reason cannot be silently
  lost. §5, applied to state rather than to a one-off increment.
* **`SNAPSHOT_VERSION` 16 → 17** with `#[serde(default)]`: pre-i381 saves restore a
  cold baseline (the trigger's own cold-start assumption); pre-v17 postcard bytes fail
  loudly rather than drifting.
* **`Simulation::moral_charge_baseline()`** is now observable — the i379 lesson is that
  a gate whose inputs cannot be seen cannot be classified.
* The i378 probe was re-pointed at `MORAL_PANIC_CHARGE_FLOOR` and now measures
  headroom against *whatever bar ships*, so it stays a reusable instrument instead of
  a snapshot of one law. The `MORAL_PANIC_CHARGE_THRESHOLD` const is **deleted** — a
  superseded constant is exactly the debt this iteration exists to clear.

## Verification

`cargo fmt --all` clean, `cargo clippy --workspace --quiet` clean, sim **300/300**,
integration **310/0/1**, `cargo insta` no snapshots to review, all golden legs
byte-identical, `scripts/gate --full` **GREEN**. Probes (runnable, cited above):
`i381_panic_anomaly_trigger`, `i381_blast_radius`, `i378_panic_threshold_headroom`.
