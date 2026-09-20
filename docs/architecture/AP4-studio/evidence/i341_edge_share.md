# i341 — Post-i340 re-ranking: the store is dense in rows, sparse in state

**Question.** i340 removed the contact *governor* (population-scaled housing ⇒ ~10% of rows
are ever contacted) but not the contact *storage*: `populate` still seeds a stranger row for
every ordered pair — the invariant `relationship_store_is_complete_at_populate` pinned at
i336. So every per-edge pass still walks N(N−1) rows per tick. How much of the tick is that,
now that contact is sparse?

**Probe.** `crates/mindstrata-benches/examples/i341_edge_share.rs` — runs the opt-in pass
profiler (`MINDSTRATA_PROFILE_TICK=1`) and groups the marks whose cost is a traversal of the
relationship store (`cognitive`, `rel_traces`, `trust_sync+reset`, `·derived+belief`,
`social_cluster`, `kinship_daily`, `appraisal`) into a "per-edge" total, reporting the
contacted-row census alongside. Seed 42, 32×32, shipped housing default, 250 warmup + 150
measured ticks.

| N | µs/tick | per-edge µs | per-edge share | store rows | contacted rows | contacted share |
|---|---------|-------------|----------------|-----------|----------------|-----------------|
| 48 | 447.3 | 210.7 | 47.1% | 2 256 | 216 | 9.6% |
| 96 | 1 119.8 | 626.2 | **55.9%** | 9 120 | 760 | **8.3%** |
| 192 | 4 172.7 | 2 431.5 | **58.3%** | 36 672 | 4 172 | **11.4%** |

Per-pass detail at N=192 (`i330_pass_profile`): cognitive 1 059.0 (of which `·cog row sweep`
662.0), memory 518.2, trust_sync+reset 357.5, ·derived+belief 323.6, rel_traces 300.8,
appraisal 267.2, social_pass 267.0, kinship_daily 149.1.

## Reading

- **The per-edge share grows with N** (47.1% → 55.9% → 58.3%) and is now the tick's single
  dominant cost class. It is *still* quadratic, because the row count is N(N−1) regardless
  of contact.
- **The store is dense in rows but sparse in state**: at the top of the envelope only
  **8–11% of rows carry any state at all**. Every per-tick traversal pays for the other
  ~90%.
- **This re-scopes i338's refutation correctly.** i338 refuted the contact-driven store as a
  fix for *contact saturation* — at that time contact was ~45% of the matrix because 8 fixed
  houses made the village a set of co-residency buckets, so a contact-only store was a ≤2×
  constant whose win shrank with horizon. With i340 the measurement moved: an order of
  magnitude of rows now carry no state, so removing them is worth ~50% of the tick, not a
  demotion-tier constant.
- **What it costs is unchanged and is the whole reason it is not done here**: those rows are
  not dead weight, they are *semantics*. Passes fold means/degrees over "every other agent"
  (mean trust, kinship degree, belief coupling), so a store that materialises only contacted
  rows changes what those folds compute — a behavioural change with a full re-anchor sweep,
  and an explicit re-contract of the i336 completeness pin (which is exactly what that pin
  was written to force).

## Pre-registration for i342

**Target: a sparse store with stable identity.** Iterate rows that carry state instead of all
N(N−1). Constraints already known from the earlier probes:

- **i337 disqualified a positional index**: row positions are reused by births/deaths, so
  "position N is touched" is not stable across ticks. The index must key on stable identity
  (agent id pair), not position.
- **i338's finding still bounds the win**: at a fixed world the *contacted* graph itself
  re-saturates past N≈144 (A9), so at the very top of the envelope the saving shrinks back
  toward the density of contact — this lever and A9 interact, and A9 (world area at constant
  density) is the one that keeps the contacted share low.
- **Re-contract explicitly**: `relationship_store_is_complete_at_populate` must be replaced
  by a liveness/positivity contract (every contacted pair has a row; folds are defined over
  materialised rows), per §4.1/§4.4 — not silently deleted.

## Reproduce

```
MINDSTRATA_PROFILE_TICK=1 cargo run --release -p mindstrata-benches --example i341_edge_share -- 48 96 192
```
