# Iteration 349 — the witness channel was non-local (and it pinned village-wide trust at 1.000)

**Status:** LANDED (behavioural) · **Root cause owned:** the i349 sparse-store design
probe was blocked by a §4.3 hazard one layer down: `update_witnesses` had **no locality
test** — every agent in the village witnessed every interaction.

## What the probe measured first (`i349_sparse_design`)

The pre-registered i341 question: how much fold output depends on never-contacted
relationship rows? Three legs answered it (seed 42, 5K ticks):

| leg | N=12 | N=48 | N=96 |
|---|---|---|---|
| contacted rows / all rows | 40.9% | 12.1% | 11.7% |
| [1] top-3 social-support slots held by never-contacted rows | 11.1% | 9.7% | 4.8% |
| [3] TrustNetwork entries fed by strangers (mass) | 59% (44%) | 88% (79%) | 88% (80%) |

Stranger rows carry *semantic* mass at scale — but leg [4/5] exposed the real finding:
**stranger-row trust p50 = 1.000** with `interaction_count == 0`. The seed band is
0.3–0.7 and dormant decay pulls toward 0.5, so something else was writing those rows.

## The writer hunt → the fix

`update_witnesses` (social/interaction.rs:424) lifted `witness → helper` trust
(+0.02 × bonding) and dropped `witness → perpetrator` trust (−0.03 × escalation) for
**all N agents per act**, stamping `last_interaction_tick` on every row. With
~1–2 interactions/tick the ratchet saturated 43–77% of never-interacted rows at
exactly 1.000 within ~5K ticks — every downstream fold (top-3 social support,
appraisal trust means, patronage/peer-group gates) read a constant. §4.3's
"fear pinned at 0.99" hazard class, in trust.

**Fix:** witnesses must be able to *perceive* the act. The loop now applies the same
Manhattan-distance perception model `select_interaction_target` already uses
(`DEFAULT_PERCEPTION_RADIUS`); positions come from the single production caller
(`system_social_interactions`). Post-fix probe leg 5: witness-stamped rows
43–77% → **0.2–2.3%**; stranger-row trust p50 **1.000 → 0.500–0.549** (the dormant
baseline). The contacted predicate is now honest — the sparse store is green-lit.

## The revival (probe `i349_kinship_seed44`, A/B against HEAD `a592510`)

Stashing the fix and re-running seed 44 @2000: HEAD has **0 births, zero kinship
activity**; post-fix, **1 birth, 16 kinship edges, kinship_penalty 0.500**. The
trust de-saturation revived organic courtship — the saturated constant had been the
trust input to the attraction channel. Producers revived, not killed.

## Classified sweep (nothing waved through)

- **`kinship_penalty_rises_when_families_form`** — its seed-44 "clean" premise is
  refuted by the revival. Fifth re-anchor of a documented treadmill
  (42/43/44 → … → 43/44); per §4.4 the surviving invariant is "some founding seeds
  stay clean" (situational taboo). Seed 44 → forms-family set, seed 43 the clean
  witness (probe: 0 births post-fix).
- **Golden (riverford_minor + collapse)** — re-contract: hashes moved with the
  de-saturation; determinism/agent-count/seeds-differ legs unmodified;
  **agent_count 12 preserved on both** (mortality check green). Evidence above.
- **7 snapshots** — reviewed before accept: endocrine shifts are the courtship
  channel live (arousal 0.37→0.72, fertility 0.88→0.63, stress mid-band, no
  saturation); the 10K surface gains **agent_count 12→13 — a birth in the standard
  scenario**; trust de-pins 0.720→0.674; memory mix shifts (Emotional/Social up,
  Cultural out) coherently with revived interaction; legitimacy/morale move <0.01.
- **Stale ledger rows fixed** (separate commit): A1→i288, A2→i289, A3+A4→i292,
  A5→i293, A6→i287 were already closed by evidence docs.

## Result

`cargo test -p mindstrata-tests --lib --release`: **309 passed / 0 failed / 1
ignored.** Sparse-store design contract (i341): the store keys on *contacted* rows;
witness influence is now radius-scoped, so per-edge passes can drop non-contacted
rows without semantic loss beyond the measured legs [1]/[3] mass — the next
iteration's implementation target.
