# Iteration 283 — UM-2 unify review: seed-disjoint cultures at the designed horizon

**Status:** LANDED · **Doctrine:** probe-first; the verdict moved twice on measurement, not intent (50K → corrected model → 100K).

## The question

UM-2 (DC-2 exit gate): "seed-disjoint cultures." i278 measured PARTIAL at 20K (jaccard 0.369, 12/15 pairs) with a 21/21 Safety census — three buckets had never crossed their first genesis threshold. i283 asks: is that dead machinery or pacing?

## The corrected pacing model

Reading `CollectiveField.step` against the i274/i279 predictions exposed an accounting error in the *predictions* (not the code): per-catalyst bucket pressure is **1/n_agents** (a single Bond = 1/12 = 0.083 pressure), so per-event press is `(1/12) × 0.05 = 0.00417` — the i274 sweep had modeled attendance-fraction pressure (0.5), 4× too high. Measured residuals at 50K recalibrate the thresholds:

| bucket | stage-2 arrival (measured model) | i274/i279 prediction |
|---|---|---|
| Relational | ~54K | ~80K |
| Identity | ~74K (s42 residual 0.673 @50K) | ~44K |
| Meaning | between | — |

**Bucket genesis is per-line:** `pressure_vector` gives every line of a bucket the full bucket pressure; genesis gates on the deepest line's stage. (The 8-line-split hypothesis was tested and rejected — lines don't share press.)

## The 100K verdict (3 seeds)

```
seed 42: generated=16 buckets={Safety, Meaning, Relational}
seed 43: generated=3  buckets={Safety, Relational}
seed 44: generated=16 buckets={Meaning, Safety, Identity}
BUCKET CENSUS: Safety 30, Relational 2, Meaning 2, Identity 1
jaccard: 42-43=0.188  42-44=0.231  43-44=0.056  mean=0.158
```

- **All four buckets generate.** The Era III content pipeline is live end-to-end at N=12 — no dead producers remain in the collective culture path.
- **Disjointness PASS**: mean jaccard **0.158** (20K: 0.369; control 1.000), every pair < 0.25. Each village's generated roster is unique in composition.

## UM-2 ruling

**PASS at the designed cultural horizon (~100K at N=12), with the pacing caveat recorded**: the original 20K wording assumed faster bucket diets. The residual debt is documented pacing (Relational 2 items in 100K), not mechanism. `seed_initial_memes` retirement decision: **deferred** — the seeded roster is still 3/3 universal-founding items per village (the by-design shared cosmology); generated rosters are now the divergence signal the gate wanted. Re-visit retirement when generated counts exceed seeded counts at typical operator horizons (currently true only for seeds 42/44).

## Files

- Probes: `i283_um2_horizon.rs` (100K genesis census + jaccard), `i283_bucket_stages.rs` (per-bucket stage/press readout)
- The 50K intermediate (census still all-Safety, jaccard 0.113) is in git history of this file — kept as the model-confirmation step.
