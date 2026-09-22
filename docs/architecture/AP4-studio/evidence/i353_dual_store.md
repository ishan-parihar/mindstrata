# Iteration 353 — dual-store migration, probed first: the planned target is refuted

**Status:** DONE (measurement + decision) · **Root cause owned:** the engine carries two
relationship stores — the legacy dense v1 `relationships` matrix and the per-agent v2
`relationship_v2s` — written independently, so consumers disagree depending on which they
read. The plan's i353 was to migrate a consumer read+write with a full sweep; doctrine §2.2
requires probing *each consumer's divergence* first.

## What the probe measured (`i353_dual_store`)

### Leg A — divergence grows with horizon, steeply

| N | ticks | pairs | Δtrust p50 | Δtrust p90 | Δaff p50 | Δaff p90 | pairs >0.01 |
|---|---|---|---|---|---|---|---|
| 12 | 2 000 | 132 | 0.0026 | 0.3420 | 0.0013 | 0.2860 | 51 (39%) |
| 12 | 20 000 | 132 | 0.0121 | 0.2424 | 0.0103 | 0.1586 | 79 (60%) |
| 48 | 2 000 | 2256 | 0.0026 | 0.0516 | 0.0013 | 0.0168 | 320 (14%) |
| 48 | 20 000 | 2256 | **0.0382** | **0.2424** | 0.0138 | 0.1662 | **2020 (90%)** |

So by a behavioural horizon the two stores disagree on almost every pair. Divergence is
**not** a legacy tail — it is the steady state.

### Leg B — the marriage gate's exposure is ~zero

The opened candidate was the marriage pass, because it is the one consumer that both *reads*
and *writes* trust/affection for the same pair (so read+write could move atomically).

| N | ticks | unpartnered adults | eligible pairs | mean chance v2/v1 | pairs crossing >10% |
|---|---|---|---|---|---|
| 12 | 0 | 12 | 47 | **1.000** | **0** |
| 12 | 50 | 10 | 30 | 0.600 | 2 |
| 12 | 150 | 2 | 1 | — | 0 |
| 12 | ≥300 | 0 | 0 | — | 0 |
| 48 | 0 | 48 | 736 | **1.000** | **0** |
| 48 | 50 | 16 | 63 | 0.875 | 1 |
| 48 | 150 | 6 | 7 | 1.000 | 0 |
| 48 | ≥300 | 0 | 0 | — | 0 |

Two facts kill the candidate:

1. **The stores are synced at `populate`** (ratio exactly 1.000, zero crossings at t=0), and
   only diverge afterwards.
2. **Marriage is an opening-window producer.** Every adult is partnered by `t≈300`
   (`unpartnered-adults 12→0` / `48→0` within 500 ticks), so the formation gate fires only
   in the window where v1 and v2 still agree. Worst observed exposure: **2 of 30** pairs at
   N=12, t=50.

**Verdict:** migrating the marriage pass would cost a full re-anchor sweep to change, at
most, a couple of pair verdicts in the first fifty ticks. Per AGENTS §4.1 and the i338/i350
precedent (do not pay a sweep for a demoted constant), the candidate is **REFUTED**. No code
changed in this iteration.

## The real target ranking (what the divergence actually feeds)

The 90%-divergent long-horizon pairs are read by the v1 consumers, ranked by exposure:

| Consumer | Store | Site | Exposure |
|---|---|---|---|
| `social_cluster` trust reads (cohesion / patronage gates) | **v1** (`rel_pos`) | `social_cluster.rs:540/573/576/850` | **high** — reads the steady-state-divergent pairs every daily boundary |
| `economy` trade trust (pricing / acceptance) | **v1** (`rel_pos`) | `economy.rs:334/408` | **high** |
| `norms_impl` trust read | **v1** (`rel_pos`) | `norms_impl.rs:969` | medium |
| daily mean-reversion **write** | **v1** | `systems/cognitive.rs:984` | high (it *is* a divergence source) |
| appraisal trust folds, cognitive centrality | v2 (`is_contacted`-gated, i350) | `systems/appraisal.rs`, `systems/cognitive.rs` | — (already v2) |
| marriage gate (read) | v1 | `sim/marriage.rs` | ~0 (refuted above) |

**The migration order for i354+ is therefore:** the daily mean-reversion *writer* first (it
is a divergence *source*; migrating it stops new divergence), then the `social_cluster` /
`economy` / `norms` *readers* (they disagree with the appraisal/cognitive v2 readers about
the same pairs). Each is a behavioural migration carrying its own sweep; none is mechanical.

## Ledger

1. **The dual-store item is re-scoped, not closed.** The marriage pass is removed from the
   target list; the writer-then-readers order above replaces it.
2. **New systemic-debt entry:** the two stores disagree on **90% of pairs at N=48/20K**, and
   subsystems are split across them (appraisal/cognitive read v2; cluster/economy/norms read
   v1). Until unified, any two subsystems reasoning about "the same" relationship are
   reasoning about different numbers.
3. **Method note recorded:** "stores are synced at populate" is the reason short-window
   probes under-report this class — the same horizon-scaling lesson as i338/i350, now a
   third instance. Probe at behavioural horizons.

## Verification

Probe + docs only; **no source changed**. Golden 5/5 byte-identical, `gate` GREEN.
