# Iteration 276 — WP-G2 referent grounding (implementation evidence)

**Status:** LANDED · **Doctrine:** probe-first; zero-blast proven by full gate.

## Probe findings (`i276_referent_survey.rs`)

1. **Gross-referent inventory is rich enough to compose with**: 12 sites
   (8 Houses, Farm, Well, Market, Temple) + 3→4 institutions (Council,
   Temple, Market, + a Faction by 20K) at N=12.
2. **Culture cited nothing**: every seeded meme (i256-conditional roster) and
   every i273 genesis meme was abstract — no world entity ever appeared in
   generated content. The substrate's §5 type-check law ("every subtle item
   cites ≥1 gross entity within exactly one domain") had zero compliance.
3. Genesis output pre-fix: "The civilization-axioms keeps the peace our
   elders won" ×5 — pure line-slug template, no world grounding.

## Implementation

- `system_collective_genesis` gains `institutions: &[Institution]` and
  `sites: &[Site]` params (call site passes `&self.institutions`,
  `&self.world.sites` — read-only, no clone).
- `eligible_referents(bucket, ...)`: domain-scoped citation sources per the
  type-check law — Relational/Safety (institution buckets) cite
  INSTITUTIONS; Identity/Meaning cite communal SITES with Houses filtered
  (private dwellings are not the village's self-image).
- Referent binding: deterministic `epoch % len` rotation through the
  eligible set in registry order — no RNG, restore-safe, and the rotation
  naturally tracks the LIVE registry (a Faction forming mid-run changes
  later citations without any new state).
- Templates now carry both runtime variables: `{ref}` (gross citation) and
  `{line}` (individual line-signature). Empty-world fallback keeps the
  ungrounded text (never hit in practice; registries seed at populate).

## End-to-end (20K, seed 42)

```
The Village Market and the civilization-axioms keep the peace our elders won  [genesis:Safety:2]
The Village Council and the civilization-axioms keep the peace our elders won [genesis:Safety:3]
The Village Temple and the civilization-axioms keep the peace our elders won  [genesis:Safety:4]
...
```

Every generated item now type-checks: cites a gross entity inside its
bucket's domain. UM-2's "the village tells stories" now means stories about
*actual places and powers*, not abstract slugs.

## Zero-blast proof

`gate --full` GREEN (307/0/1) with **zero re-anchors**: the 10K snapshot's
metric projection pins counts, not meme text, and genesis still fires only
past stage 2.0 (i273's identity floor holds). Byte-identical calibration
horizons with richer long-horizon culture.

## New unit evidence (5 genesis tests)

- `genesis_cites_a_gross_referent_within_one_domain`: Safety cites an
  institution; Identity cites the communal Well and NOT "House 1".
- `genesis_referent_rotation_is_deterministic_per_epoch`: successive epochs
  rotate registry-order, no RNG.
- Pre-existing 3 (identity-below-gate, dedup, cite-first determinism) hold
  with fixtures.

## WP-G2 remaining (honest)

The full §5 generator is `domain × referent × stance × line-signature` —
this iteration landed the referent term for the GENESIS path. Stance
(polarity state of the claim) and uptake-gating ride on i277's
tetra-arising work. `seed_initial_memes` retirement stays gated on the
i278 disjointness probe (the roster is still the founding vocabulary;
retiring it before the probe would remove the control group).
