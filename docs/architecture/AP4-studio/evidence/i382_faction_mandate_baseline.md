# i382 — the faction legitimacy reference is the council's own mandate (the audit's Class-4 ruling, second landing)

**Status:** LANDED (behavioural, zero calibrated-window blast radius) · **Root cause owned:**
i379's gate census read `council legitimacy < 0.50` at **0.0% open** in all four sampled
worlds and queued it as an unreachable arm ("live-or-delete"). The probe says the census was
right about the *defect* and wrong about the *shape*: the arm is not unreachable, it is
**knife-edge**, and it appears **twice in the same function**.

## What the probe measured first (`i382_legitimacy_channel`, 15 worlds = 5 scenarios × 3 seeds)

`legit < 0.5` opens 0.00% of ticks in **8 of 15** worlds and 0.07–4.26% in the other 7 —
i379's "0.0%" was a seed-42 artifact. Against the measured mandate equilibrium **0.54–0.58**
(floor-law `max(morale, 0.6) × (1 − 0.25·g)`), the bar sits a hair below the equilibrium, so
*whether a world can ever organize through legitimacy* is decided by its seed. And the arm is
**load-bearing**: of 8 formations in the corpus, **5 were cliff-only**, all in quiet worlds
(calm-42, calm-23, calm-town-23, collapse-23, famine-23).

**The second site is the same number.** The i240 accumulator's deficit term is
`(0.5 − avg_council_legitimacy).max(0)` — so the legitimacy channel contributed **exactly
0.0000 pressure units in all five pestilence/collapse/famine-town worlds** (its build-term
share of the tank: 0.00%), while in the quiet worlds it was the *only* live term (36–100%).
One absolute reference, two sites, and the channel's liveness decided by a bar at the
distribution's extreme.

## The measurement refuted my own first plan

I intended to **delete** the arm (the i372 precedent) and let the accumulator carry the
channel. The corpus says no: the accumulator *integrates*, so a brief deep collapse — the
calm-village dip to **0.144**, which produced a shipped cliff-only formation — adds almost no
pressure (max 0.413 at τ=60 in collapse-23, no crossing) and would have been **lost**. A
producer must be revived, not killed (§2.3). The instant arm and the deficit are *different
signals* (a step vs an integral) and both are needed.

## The law

```
baseline: f64 EWMA of the mean council legitimacy, τ = LEGITIMACY_DEBT_TAU_TICKS = 100,
          cold-adopting its first observation (0.0 = never observed)
deficit  = max(0, baseline − legit)          ← the accumulator's legitimacy term
collapse = legit < baseline − LEGITIMACY_COLLAPSE_MARGIN (0.05)   ← the instant arm
```

Both read the baseline **before** the fold, so a tick is judged against its past and never
against itself (the i381 rule). τ is derived, not guessed: the council converges legitimacy at
0.01/tick (`institutions_impl`, "≈70-tick half-life"), so the baseline is the mandate's own
memory one convergence time constant deep. The margin is the offset the *deleted* bar measured
empirically (0.5 against a 0.54–0.58 equilibrium) — expressed as a difference, it travels with
each world's mandate instead of pinning every world to one equilibrium's neighbourhood.

**Cold start is safe by construction**: the arm compares against a *smaller* baseline (0.0 →
false) and the deficit is `max(0, negative)` = 0, so a fresh or restored run cannot arm on a
placeholder. A world with no council never folds (no mandate to learn).

## Band checks (both sides of the corpus, not just the firing side — the i381 method correction)

| τ | quiet worlds crossing | crisis worlds crossing | deficit live in |
|---|---|---|---|
| 60 | 3/6 | 4/4 | 15/15 |
| **100 (shipped)** | **3/6** | **4/4** | **15/15** |
| 1080 | 3/6 | 4/4 | 15/15 |
| 3240 | 6/6 (faction factory) | 4/4 | 15/15 |

The must-not-fire property holds across **60–1080 (a 16× band)**; at 3240+ every quiet world
crosses. The instant arm's margin: 0.02 opens quiet worlds spuriously (4–22 episodes), 0.05
reproduces the shipped opening set, 0.10 is too strict (kills all but the deepest collapses).
τ=100 was chosen inside the admissible region, not at its edge.

## The shipped law, live (the probe re-run after landing)

| | before (absolute) | after (relative) |
|---|---|---|
| deficit pressure units | **0.0000 in 8/15 worlds** | **0.0209–0.2333 in 15/15** |
| formations | 8 | 12 |
| cliff/collapse-only formations | 5 | 5 (all preserved) |
| new formations | — | calm-v7, calm-town-7, collapse-7, famine-7 (one each) |

Every previously-cliff-only formation is preserved with collapse-arm attribution; the four
additions are quiet worlds reaching their first organization *once* (the documented i240
cadence, "a stable village forms ~1 faction per 30–50K") and two crisis worlds that the
absolute bar left dark.

## Blast radius: **zero** on the calibrated surface

sim **300/300**, integration **310/0/1**, goldens **9/9 byte-identical**, **no snapshot drift**
(no `.snap.new`), clippy clean, `gate --full` GREEN. Nothing needed re-anchoring — and that is
*measured*, not assumed: in every calibrated world the first formation (under either law) falls
outside the pinned windows, so the corpus shift is real but invisible to them. The new
`faction_trigger_reads_the_councils_own_mandate` pin is the runnable check (§9): at seed 7 /
N=48 / tick 3 000 the learned baseline is **0.609** (the deleted bar sat 0.109 below this
world's mandate) and 600 ticks of holding the mandate **0.03 below** it — level **0.579, above
the deleted bar** — gains **0.0147** of tank against **0.00097** held at its own reference
(15.2× drift).

## Audit refresh (appended to i373)

The legitimacy reference moves **Class 4 → Class A** at both sites. One method correction is
recorded: **a gate census is a snapshot of one seed** — i379's four-world 0.0% was true for its
seeds and false for the class, so an audit verdict needs the seed dimension before it becomes
"unreachable" (i376's store probe and i381's corpus lessons, third instance). And the "reverse"
lesson from i380's Class 3b: **verifying a defect does not verify the repair** — the delete fix
this iteration's plan opened with was refuted by the same corpus that proved the defect.

**Queued from i379's rulings, unchanged:** `needs.social` (Class 3b — needs the
winner-decomposition census), and `emotions.anger > 0.50` (near-dead at town scale —
re-anchor or delete, explicitly).
