# Iteration 328 — the §17 tier-gating ceiling is single-digit (lever sized)

**Status:** LANDED (**measurement only, no behaviour change**) · **Item owned:**
i316's surviving lever "a Secondary-reduced biology path", and the dead tier
gates `runs_full_biology()` / `runs_action_selection()`.

## The question

i316 found `AgentTier::Background` unreachable and two `AgentTier` gates with
zero production call sites, and recorded building a reduced Secondary/Background
path as a scale lever. Before building one, the prior question is whether tier
state gates **any** real per-tick cost today — if the budgets do not bite, a
reduced path has to *add* per-system gates and its payoff must be bounded by
whatever the existing tier floor already buys.

## Confirmed: Background is unreachable by construction

Probe `i328_tier_payoff`, census at 10K ticks, Background is **0 at every N**:

| N | Focal | Secondary | Background |
|---|---|---|---|
| 12 | 6 | 6 | **0** |
| 24 | 18 | 6 | **0** |
| 48 | 19 | 32 | **0** |

Two independent blockers on the Secondary → Background edge, both structural:

1. `narrative_importance < 0.1` — but `update_narrative_importance`'s target
   carries a `+0.15` base, so the smoothed floor is ≥ 0.15 (i316 measured 0.30).
2. `relationship_count < 2` — every agent in a village builds a relationship to
   every other, so the live value is N−1 (≥ 11 at N=12).

The code comment already says it is "structurally unreachable at village scale …
reserved for sparse multi-settlement runs" — this probe confirms that is exact.

## The ceiling on any reduced path

Two identical sims (same seed) are run to the warmup horizon, then one has
**every agent forced to `Background`** (zero memory/prospection/ToM budgets —
the tier's designed floor, and nothing sits below it) before a timed window. The
delta is therefore an **upper bound** on what any tier-reduction can buy.

| N | normal ms/tick | all-Background | delta |
|---|---|---|---|
| 12 | 0.1082 | 0.1043 | −3.6% |
| 24 | 0.2592 | 0.2459 | −5.1% |
| 48 | 1.5323 | 1.4895 | −2.8% |

Verdict leg (N=48, three repeats, median taken — this host's fast-tick mean
drifts ±5% run to run): deltas **−6.15% / −5.28% / −3.68%, median −5.28%**.

## Verdict

`TIER_GATE_PAYOFF_BOUNDED_SMALL`

Forcing **every** agent to the lowest tier saves single digits — roughly
**2–8%, median ≈5%**. So the expensive passes are **not tier-gated** today; the
§17 cognitive budgets gate a small slice of the tick. Consequences, recorded:

- **"Secondary-reduced biology path" is demoted.** Its payoff is bounded by this
  ceiling: even a path that put *every* agent at the Background floor — more
  aggressive than any real LOD policy — would buy ≈5%. Building per-system tier
  gates is a large change for a single-digit term, so it is **not** the next
  scale investment (same disposition as i326's sparse-store demotion).
- **Making Background reachable is not a perf lever.** Even if the two
  structural blockers were relaxed, the tier it promotes into is worth ≈5% at
  the extreme, and `runs_full_biology()`/`runs_action_selection()` would still be
  inert unless per-system gates are written.
- The true scale costs remain what i326 identified (memory: now bounded) and
  i294's interaction-pass superlinearity (time).

## Verification

- Measurement only — **no source change**, golden byte-identical by construction.
- `cargo fmt` clean, clippy 0 warnings, `scripts/gate --full` **GATE GREEN**.

**Probe:** `cargo run -p mindstrata-benches --release --example i328_tier_payoff`
