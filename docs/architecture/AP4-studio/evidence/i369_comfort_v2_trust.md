# i369 — the comfort/soothing path reads the dyadic v2 store

**Status:** LANDED (behavioural, first v1→v2 migration) · **Scope:** one read in
`social_cluster.rs` (`tick_gossip_and_knowledge`, the §8.1.14 Iteration-191 active
soothing path).

## What moved

The Comfort-event handler read the recipient's soothing context from the legacy v1
`relationships` matrix; it now reads `relationship_v2_between(from, to)` with the same
0.5 stranger fallback. This was an **intra-module inconsistency**: the same file already
read v2 at the cluster/pair folds (`relationship_v2_pos` call sites). The migration makes
the whole comfort path consistent with the store that carries the attachment fields
(`attachment_security`, Sternberg triangle) the soothing effect is grounded in.

Per the accepted charter decision: **v2 is the replacement model; the migration proceeds
subsystem by subsystem**, each with its own probe + sweep.

## Runnable check

`sim::tests::psychology::comfort_soothing_reads_the_v2_dyadic_store` — a three-way pin:
two sims differing **only** in v1 trust (0.1 vs 0.9) must soothe **identically** (v1 is
ignored by this path); a sim with higher **v2** trust must soothe **more** (v2 is the live
read store). Guards against regression on both directions (a silent v1 re-read and a dead
v2 read).

## Measured impact

Calm windows are nearly neutral (v1≈v2 at short horizons, per i353):

- Snapshots: `metrics_2000` and `long_horizon_surface_10000` shifted at 1e-5 scale
  (reviewed → accepted).
- **Collapse golden: one birth flipped (13 → 12).** The migration bites under stress —
  v2 diverges from v1 exactly when comfort/attachment matter most, which is the point of
  the migration. The stability pin contracts `0 < count ≤ 48`; in-contract.

## Sweep classification

No assertion contract broke. Two snapshot regenerations + one golden regeneration, all
mechanism-explained above. sim **299/299** (300 with the new pin), integration
**310/0/1**, gate GREEN.

## Remaining v1 readers (queue, per charter decision)

`economy`, `norms_impl`, `household` (51 refs), `marriage`, `births_deaths` — each
migration is behavioural and carries its own probe. **Do not migrate the marriage pass**
(i353 refuted: it fires only in the opening ~300 ticks where v1≈v2; a sweep there is the
§4.1 mistake). The daily mean-reversion *writer* (`systems/cognitive.rs:984`) is queued
first among writers because it is a divergence *source*.
