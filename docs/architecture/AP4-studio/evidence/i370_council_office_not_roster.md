# i370 — the council is an office: revolution installs seats, not a roster

**Status:** LANDED (behavioural, sweep-carrying, root-cause fix) · **Root cause
owned:** the "membership inconsistency (31 vs 3)" recorded since i361 and the
"residual hoard" recorded since i365 are **one causal chain**, not two debts.

## The probe (`i370_council_membership`)

Measured council membership + treasury across seeds at 20K and 50K:

| Seed | Populate | Max | Final | Revolutions | Treasury @50K |
|---|---|---|---|---|---|
| 7 (pre-fix) | 3 | **34** | **34** | **1** | **28 697.7** |
| 42 | 3 | 3 | 3 | 0 | 257.6 |
| 23 | 3 | 3 | 3 | 0 | 345.7 |
| 55 | 3 | 3 | 3 | 0 | 185.8 |
| 99 | 3 | 3 | 3 | 0 | 270.9 |

**The mechanism:** `collect_taxes` taxes `institution.members`. The revolution
path in `norms_impl.rs` did `council.members.clone_from(&faction_members)` — the
entire faction roster became the council membership. One coup on seed 7 turned a
3-office council into a 34-member taxed class, and the i363 dividend's hoard
equilibrium `T* = reserve + inflow/share` is linear in inflow, which scales with
the taxed roster: hence 28 697 vs ~300. This also retro-explains i361's
"31 members / 163 602 coins" observation (a post-revolution council) and the
i365 seed-23 residual anomaly is now known to be a non-revolution world whose
inflow is simply higher (345.7 post-fix, in band).

## The fix

A coup installs **offices, not a roster** (the `default_institutions` shape):
the faction leader takes the **Elder** seat, the top
dominance+conscientiousness faction members fill **Guard Captain** and the
second seat, the rest return to villager status. Faction roles are still
recorded on the council so the history stays legible. Deterministic (score sort
with index tiebreak).

**Post-fix probe:** seed 7 at 50K — membership **3 / 3 / 3**, treasury **182.0**
(below every no-revolution seed's pre-fix value; the revolution no longer
inflates the tax base at all).

## Sweep classification

- `revolution_is_regime_change_not_repeat_loop` — **RE-CONTRACTED (§4.4).** The
  old pin asserted *peak council membership ≥ 5* after a coup, i.e. it pinned
  the roster-clone bug itself as the contract. The real invariant (regime
  change = new office holders from the faction) is now asserted directly: on
  every seed that fires, the **Elder office changes hands**. Membership is
  capped at the 3-office shape by design; the ≥5-membership assertion is
  heredity-dead.
- No other pin moved. Goldens byte-identical (9/9 golden-scoped tests), sim
  **300/300**, integration **310/0/1**, clippy clean, gate GREEN.

## Consequences

- The hoard equilibrium is now bounded by the 3-member tax base on every
  trajectory, revolution or not (~180–350 at 50K across the swept seeds).
- The i365 "treasury ceiling" queued item is **closed as superseded**: with the
  tax base fixed, the organic dividend drains the treasury to ~200–350 on all
  measured seeds; no ceiling constant is needed.
- Council-membership inconsistency (31 vs 3) — **closed** (was the same bug).
