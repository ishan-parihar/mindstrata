# i400 — the `social_cluster` v1 trust readers: measured, and REVERTED

**Status:** REVERTED with measurements (no behavioural change landed; the code carries
the finding). Probe: `crates/mindstrata-benches/examples/i400_cluster_trust.rs`.

**Scope attempted:** the last two self-contained v1 trust readers in
`social_cluster.rs` — the §8.1.9 ToM pair-trust reads (`trust_from_to` /
`trust_to_from`) and the §19.5.I knowledge-diffusion `source_trust`. The comfort
path in the same file had already migrated (i369), so these were the file's only
remaining `rel_pos` reads.

---

## 1. What the probe measured (before touching anything)

| leg | N=12 | N=48 |
|---|---|---|
| A — \|v1−v2\| on all pairs (mean / max) | 0.0096 / 0.7029 | 0.0122 / 1.0000 |
| A — on contacted rows only (`is_contacted`) | 0.0096 / 0.6407 | 0.0195 / 1.0000 |
| B — pairs straddling the 0.5 verdict | 0.90% (206/25 781 contacted) | 1.18% (1.82% contacted) |
| C — fallback misses (v1 / v2) | 0 / 0 | 0 / 0 |

Sampling at a 72-tick cadence, deliberately, so both sides of the daily i376 sync
boundary (144) are covered rather than always landing on the synced side.

Three readings shaped the attempt:

* **The mean is small and the tail is not.** Mean 0.010–0.020 — the i376 sync is
  doing its job — but max 0.70/1.00, i.e. individual pairs sit at opposite ends of
  the scale across the two stores, unbounded in horizon.
* **The fallback is dead on both stores.** Zero misses means the `map_or(0.5, …)` is
  a guard, not a live path, so the migration would change *only* the store — the
  cleanest possible shape for a reader migration.
* **But the read feeds a live band edge.** `infer_intent` reports Friendly only
  above trust 0.5, so every straddling pair *flips its inferred intent* under the
  migration. That is the i392-row-4 test (band edge inside the divergence range),
  and 0.8–1.8% is small but not zero.

## 2. What the suite said — the reason for the revert

Built at the three sites with the file's own i369 convention (same 0.5 fallback,
only the store changes), then run:

**304 passed / 10 failed / 1 ignored in 211 s.** The same tree reverted measures
**314 / 0 / 1 in 165 s**, so both the failures and the slowdown attribute to this
change:

| failure | text |
|---|---|
| `pregnancy_state_refactor_keeps_lifecycle_dormant` | *"agent Lars became pregnant — the lifecycle must stay dormant within the 500-tick window (demography drives births)"* |
| `conception_pregnancy_birth_pipeline_runs_and_is_seed_deterministic` | *"no pregnancy may exist in the golden window"* (left 1, right 0) |
| `knowledge_acquisition_desacralizes_sacred_values` (sim unit) | *"gossip must transfer the knowledge item"* — the fixture pins v1 trust 0.9; on the dyadic read the source sat at the v2 stranger prior 0.4, so `acceptance = 0.4·0.5 + openness·0.5` fell under the 0.5 floor |
| both goldens + 6 snapshots | stream reshape via the acceptance-floor crossings |

The decisive pair is the **two dormancy contracts**. They are not chaos-sensitive
seed families — they assert a *design* state (in the calibrated village the
pregnancy pipeline is dormant and demography drives births) that this repo has
deliberately pinned more than once. Clearing them would mean widening a contract,
which §4.1 forbids; and the migration's own benefit is a ~0.01-mean accuracy change
on a **soft** consumer (ToM intent labels, a knowledge gate) with no new fidelity —
the trade is bad in both directions.

## 3. What is recorded, and where

* `social_cluster.rs` carries the finding at both sites (measured probe numbers, the
  four failed families, the slowdown) so the next attempt starts from evidence.
* `tests/culture.rs` records the fixture requirement for the retry: with the dyadic
  read, the v1-only pin is insufficient and the v2 pin is needed, because the
  acceptance floor makes 0.4 vs 0.9 a pass/fail difference.
* The probe stays in the tree as the instrument.

## 4. The retry's precondition (what would make it landable)

This is the third item in the sweep whose shape changed under measurement, and it
points the same way each time: **the v1 readers cannot be moved one at a time once a
band edge is involved.** The i393 §6 sequence already says the writer goes last — but
the evidence now says something sharper: the ToM/knowledge/diffusion trust path and
the **interaction-gain deletion** must move in **one commit**, so the dormancy
windows and the acceptance floor are re-derived once against the whole new trust
surface instead of being perturbed three separate times. Until then these reads stay
on v1, exactly as v1 stays the store the interaction gain writes.

## 5. Cumulative state of the sweep

| site | verdict |
|---|---|
| `NormsImpl` belief evidence (reader) | **landed, zero-blast** (i398) |
| `NormsImpl` violence/punishment writers | deferred — transient vs persistent magnitude (i398) |
| `Marriage` closed loop (gate + boost + jealousy) | **landed, behavioural** (i399) |
| `SocialCluster` comfort | landed earlier (i369) |
| `SocialCluster` ToM + knowledge diffusion | **measured, reverted** (i400) |
| `Economy` trade pair | landed (i384) |
| interaction gain + `RelationshipKind` ladder | ladder retired (i393); gain deletion queued with the `bonding_rate`/`conflict_escalation_rate` re-hosting |
| `MemoryOps` `trust > 0.6` status fold | queued |
