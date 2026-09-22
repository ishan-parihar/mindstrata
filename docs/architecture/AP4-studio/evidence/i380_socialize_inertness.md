# i380 — the social drive is inert; and the two obvious repairs do NOT fix it

**Status:** LANDED as **measurement + refutation** (comment-only source change; no
simulator behaviour altered) · **Scope:** the i379 audit's queue item #1.

i379 flagged the action-utility term
`utility += needs.social × action.social_value × extraversion` (`actions/mod.rs`) as a
**gain/scale mismatch**: it multiplies by ~0.006, so the common case rounds to nothing.
That flag had two readings that want opposite work — *the term is inert* versus *a
near-zero deficit is the deficit working correctly* — so the first move was to measure
which one it is.

## Measurement 1 — the decision census (`i380_socialize_reachability`, seed 42)

| world | decisions | Socialize share | selected by routine | selected by utility | arbitrations |
|---|---|---|---|---|---|
| village 16×16 N=12 @20K | 42 519 | 5.86% | **2 489** | **0** | 18 958 |
| town 46×46 N=48 @20K | 197 110 | 5.15% | **10 156** | **1** | 93 711 |

**The flag is real, not conservative.** Across **112 669 utility arbitrations** the utility
leg chose `Socialize` *once*. Every social decision in the sim comes from the daily routine
slot. The i379 reading of *inert* is confirmed and the *correct-by-design* reading is
refuted: the routine is not "carrying" a satisfied need, it is the only route.

## Measurement 2 — why, and the two repairs that FAIL

Sizing first (the same probe): the **quiet-window winner utility is 0.692 mean / 1.702
max**, while the social term contributes **0.001 (p50) to 0.004 (p95)** — two to three
orders below any competitor.

Two single-knob repairs were then measured, each principled on its own:

**Repair A — a gain in family with the siblings.** Hunger/thirst/fatigue carry 2.0/2.5/1.5;
this term carried no coefficient at all. At `×6`: utility-selected `Socialize` moved
**0 → 0** (village) and **1 → 2** (town). Insufficient.

**Repair B — the need's operating range.** A sweep of `social_decay_rate` (the accrual),
because the need's band sits ~10× below its siblings' and every consumer — the 0.30 / 0.40
gates, the utility term, the dominant-need urgency boost — is calibrated for a
sibling-scale range:

| `social_decay_rate` | need p50 | need p95 | need max | utility-selected Socialize |
|---|---|---|---|---|
| 0.0002 (shipped) | 0.006 | 0.021 | 0.156 | **0** |
| 0.001 (hunger's rate) | 0.025 | 0.088 | 0.210 | **2** |
| 0.002 | 0.050 | 0.162 | 0.252 | **0** |
| 0.004 (20×) | 0.096 | 0.292 | 0.578 | **9** |

Even at 20× the accrual the utility leg selects `Socialize` 9 times in ~19 359
arbitrations (0.05%), and the total `Socialize` share barely moves (5.31–5.51%) because the
routine's share is fixed. Insufficient — and the honest conclusion is stronger: **both
single-knob repairs fail**, so neither is the root cause.

## What the failure establishes

The blocking quantity is the **arbitration bar**, not this term's scale. Even an
implausibly lonely agent (`needs.social = 0.29`, the 20×-accrual p95) scores
0.29 × 0.3 × 0.5 × 6 = **0.26** against a 0.692 mean winner. So the social candidate loses
on the terms it does **not** have — the goal-alignment bonus (`priority × 0.5`, which only
exists when the loneliness gate has already created a `Socialize` goal), the dominant-need
urgency boost, identity affinity, the norm bonus — rather than on the one term it does.

That is a categorically different defect from the two repairs tested, and it is exactly the
shape the project already found once for `Novelty`/`Play` (i351/i356: a motive whose
*relief channel* is structurally unreachable). The parallel is close enough to be the
leading hypothesis: **`Socialize`'s utility candidate may be missing a relief channel that
its siblings all have.**

## Deliberately NOT landed

No coefficient was shipped. A `×6` gain would have moved the metric by 0–2 selections while
its comment claimed the drive "becomes live in the lonely tail" — a symptom fix whose
documentation overstates its effect, which is the failure mode §4.2 exists to prevent. The
source change in this iteration is the **comment recording the measurement and the failed
repairs**, so the next agent does not re-run them.

## Next step (queued, with the sizing already done)

Extend `sim::decision_census` to **decompose the winner's utility per candidate** (the
census already carries the per-candidate gap machinery for `Wander`/`Idle`), then size the
repair against the real bar: specifically, test whether `Socialize` is missing the
goal-alignment / urgency channels that make `Eat`/`Rest`/`Work` win, and whether the 0.30
loneliness gate that creates the `Socialize` goal is ever satisfied by a *joint* condition
(it currently requires `joy > 0.5 ∧ social > 0.3`, measured at ~0%).

## Verification

Simulator behaviour is **unchanged** (comment-only diff). Suite state carried: sim
**300/300**, integration **310/0/1**, `scripts/gate --full` GREEN. Probe:
`i380_socialize_reachability` (runnable; legs for the census, the winner-utility sizing,
and the `social_decay_rate` sweep).
