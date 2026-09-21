# Iteration 347 — A12 closed: the feud-approach producer is live (and its ordering was the real fault)

**Status:** LANDED (behavioural) · **Root cause owned:** i346 found `Move` (the only
§6 movement action) selected **0 times in 96 000 agent-ticks** and named two candidate
causes: the `anger > 0.4` gate above the channel's reachable band (A12), or the branch
never running. The sweep needed both measured before touching anything.

## What the probe measured first (`i347_feud_gate_reach`, 3 seeds × {calm N=12/N=48 @20K, pestilence/collapse/drought @4320})

**The band.** `emotions.anger` is an *acute* emotion with fast decay, not a slow state:
calm `p50 / p90 / p99 / p99.9 = 0.0000 / 0.0089 / 0.1113 / 0.3005` (N=12), so any
absolute gate on it is a tail gate by construction and `0.4` sat past p99.9. On the
`feuds ∧ anger > t` conjunction (share of agent-ticks):

| t | calm N=12 | calm N=48 | collapse | drought | pestilence |
|---|---|---|---|---|---|
| 0.02 | 0.226% | 0.088% | 0.032% | 0.042% | 0.027% |
| 0.05 | 0.110% | 0.038% | 0.020% | 0.020% | 0.000% |
| 0.10 | 0.044% | 0.011% | 0.013% | 0.013% | 0.000% |
| 0.40 (old) | 0.000% | 0.000% | 0.000% | 0.000% | 0.000% |

**The ordering (the actual root cause, found by an experiment the gate re-pin alone
could not explain).** Re-pinning the gate to a reachable value with the branch in its
historical position (below the §10.3 routine ladder) moved the census `feud` row to
**still 0.000%** at gate 0.05/0.02 and only 0.40% at gate 0.005. Temporary counters at
the branch measured the mechanism: at gate 0.02 (N=12, seed 42), **45 of 45 candidate
decisions were swallowed by `follow_routine`** (314 of 505 at gate 0.005). The branch
sits above the "not when critical needs demand attention" needs-guard but *below* the
daily schedule — an angry agent mid-routine never approaches, the opposite of what the
§19.5.G clause says. **The dead producer was two faults stacked: an unreachable gate AND
a shadowed ordering.** Fixing only the gate would have re-pinned A12 and left the
producer dead.

## What landed

1. **`FEUD_APPROACH_ANGER = 0.02`** (`pass_action.rs`), from 0.4 — a §4.4 re-contract:
   the old number was never anchored on the channel it gated (no canon for anger's
   magnitude exists), and the invariant the branch must guard is *liveness*. 0.02 is
   ~p96 of the calm N=12 anger distribution — elevated, not the noise floor — and the
   only candidate live in every measured context (pestilence included, 0.027%).
2. **The branch moved ABOVE the routine ladder** (still below external commands §5 and
   the i255/i306 physiological reflexes; the `hunger < 0.85 && thirst < 0.85` guard
   retained as the "not when critical needs" carve-out).
3. **Acceptance pinned in the census test**: `SRC_FEUD > 0` and every feud-sourced
   decision is a `Move` (its only output).
4. **`i347_feud_move_delta`**: locomotion anatomy + the escalation statistic re-measured
   over a 12-seed family.

## Exit evidence

**Census (i346 probe, in vivo, seed 42):** `feud` source **0.55% of decisions at N=12
(242) / 0.37% at N=48 (710)**, all producing `Move` — the row moved off zero exactly as
the acceptance criterion required. `Move` is now a live action; `Wander`/`Idle` remain
structurally dominated (A8 unchanged, gap 1.68/1.41 mean).

**Locomotion anatomy (`i347_feud_move_delta`, 5 000 ticks, seed 42, 32×32):**

| | N=12 | N=48 |
|---|---|---|
| distinct cells / agent | **1.0** | **2.9** |
| near-pair share (d ≤ 5) | 40.9% | 10.0% |
| touched rows / mean partners | 54 / 4.5 | 280 / 4.7 |
| conflicts / violence | 223 / 19 | 524 / 32 |

Honest limit: approach takes **one Manhattan step per *decision***, and decisions are
sparse (18.3% of agent-ticks), so displacement stays small — feuding agents close
distance over hundreds of ticks, not tens. The producer is live; the *pace* of
locomotion is a separate calibration question, recorded not hidden.

**The sweep (exactly three expected failures, all classified):**

1. `relational_dominance_feeds_violence_escalation` — **re-contracted (§4.4), and the
   re-contract fixed a latent §4.1 violation in the old pin**: i200 had chosen seeds
   [1, 2, 3] by "healthiest aggregate margin" out of a 16-seed sweep — a lucky-seed
   family. The fix re-rolled that tail (27 vs 29, margin −2). The 12-seed family probe
   (`i347_feud_move_delta`) measures **dominant 103 vs subordinate 81 (margin +22),
   strictly higher on 7/12 seeds**; the pin now guards the aggregate direction over
   [1,2,3,5,7,11,17,21,33,42,44,99] instead of one seed's tail, and the per-seed
   crafted-asymmetry reach assertion (structural) is retained.
2. `golden_replay_crisis_vs_baseline` — the collapse golden regenerated: `agent_count
   12` preserved, `total_grain 0.7533 → 0.0589`, `total_water 1137.4 → 1140.6`
   (approach walks now cost ticks that used to go to work/provisioning in a
   collapse-strressed village).
3. `snapshot_long_horizon_surface_10000_ticks` — reviewed then accepted: `agent_count
   12` stable; `avg_relationship_trust 0.7065 → 0.7351`, `avg_relationship_quality
   0.5738 → 0.6138`, `avg_stress 0.3658 → 0.3660` (flat), `polarization 0.0665 → 0.0606`,
   memory traces 775 → 729, stage redistribution toward Ally. Coherent with feuding
   pairs meeting more; nothing saturating.

`scripts/gate --full` **GATE GREEN** (309/0/1), `mindstrata-sim` **294/294** ×5
consecutive (the census pin's upper-bound assertion was removed with the parallel-harness
reason documented in-place: the process-global sink counts every concurrent test's
simulations, so an upper bound cannot hold under `cargo test`'s parallelism).

## Ledger effect

- **A12 CLOSED** — the producer is live and pinned.
- **A8 re-scoped again**: `Move` is now live; `Wander`/`Idle` remain structurally
  dominated (zero relief term, gap 25–30× jitter). A8's remaining question is *design*
  (should a villager roam?), plus the new pace question above.
- Locomotion is no longer decorative in the feud dimension — the first live
  position-driven contact channel beyond the §10.4 courtship walk.
