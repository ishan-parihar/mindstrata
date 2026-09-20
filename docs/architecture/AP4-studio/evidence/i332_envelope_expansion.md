# Iteration 332 — the scan removals bought 1.5× the scale envelope

**Status:** LANDED (measurement) · **Item owned:** none open — the post-i331
re-baseline of the charter perf envelope (i295's warn-only budget gate).

## The question

i295 re-baselined the charter budgets after i294: N=96 at 4577.7 µs/tick
against the 6500 budget = **27% headroom**. i330 (four accidental relationship
scans in the social pass) and i331 (the last three, in the memory / derived-belief
/ resource-ops passes) then cut tick cost ~30% at N=96 and ~49% at N=192. Since
those iterations were pure hot-path removals with **byte-identical goldens**, the
correct thing to do with the freed budget is measure whether a **larger
population** now fits the same charter target — not to edit the budget.

## Method

New probe `i332_envelope_expansion`, the i295 **charter method** exactly (world
32×32, 2000 ticks, seed 42, timer spans new+populate+run, min-of-3), at
N = 12/48/96/144/192. The N=96 budget (6500 µs/tick) is the fixed yardstick.

## Result (release, this host)

| N | µs/tick | vs N96 budget (6500) |
|---|---|---|
| 12 | 96.7 | −6403 |
| 48 | 531.4 | −5969 |
| 96 | 1853.0 | −4653 (**71% headroom**, was 27%) |
| 144 | 4157.9 | **−2197 (fits)** |
| 192 | 7700.8 | +1243 (breaches by 16%) |

`verdict=ENVELOPE_EXPANDED_1_5X`

The N=96 charter target now covers **N=144** — a 1.5× population at the same
per-tick budget — and N=192 is within 16% of fitting (its own overshoot is
dominated by the structural Ω(N²) interaction matrix, which i331 left as the
floor). The N=12 golden budget headroom is unchanged (96.7 vs 150 = 36%), as
expected: the removed terms only grow with N.

## Reading

- This is a **measurement**, not a new budget. The charter budgets are unchanged;
  what changed is that the envelope they define now has room. Any decision to
  raise the DC-3 Phase-1 population target is a charter edit that should follow
  from this number, recorded here rather than taken unilaterally.
- The gain tracks the removed terms' N-dependence exactly: −0% at N=12,
  −43% at N=96, −46% at N=192 versus the pre-i330 cost surface.
- i295's trigger review is unchanged by this: the VecDeque/claims triggers are
  horizon-based (>250K ticks), not population-based, and are still not hit.

## Verification

- Probe is warn-only (exit 0), matching i295; hard floors stay with i270/i271.
- No sim code changed this iteration → zero re-anchors, goldens untouched.
- New bench obeys the IC-4 naming law (`i332_envelope_expansion`).

**Probes:** `i332_envelope_expansion`, `i295_perf_budget_gate`, `i329_local_exponent`.
