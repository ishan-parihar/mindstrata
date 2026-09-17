# Iteration 277 — WP-I tetra-arising gate (implementation evidence)

**Status:** LANDED · **Doctrine:** midpoint-neutral by construction; zero-blast proven by full gate.

## What landed

The substrate's tetra-arising rule ("collective stage bands gate WHICH content
classes the generator may emit", WP-I) is now enforced in genesis:

| Band | Stage | Unlocked content classes |
|---|---|---|
| I | < 2.0 | none (i273 identity floor) |
| II | 2.0–3.9 | foundational: Historical (Relational, Identity), Moral (Safety), Theological (Meaning) — the pre-277 behavior |
| III | 4.0–5.9 | + Political (Safety: "the {ref} must answer to the {line} of this village"), Song (Identity: communal celebration) |
| IV | ≥ 6.0 | + Prophecy (Meaning: reflective) |

Each template row carries a `min_band`; the emission loop precomputes each
bucket's deepest line stage once and gates every row on it. **Midpoint-neutral
by construction:** below stage 2 nothing emits (unchanged), and band II is
exactly the pre-277 class set — calibrated horizons see identical behavior.

## Dedup-tag correction (found by the new band tests)

The genesis tag was `[genesis:{bucket}:{epoch}]` — a band-II Moral meme at
stage epoch 4 would permanently block the band-III Political unlock at the
same epoch (same bucket, same epoch). Tags now carry the content class:
`[genesis:{bucket}:{class}:{epoch}]` — classes dedup independently.

## Zero-blast proof

`gate --full` GREEN (307/0/1), zero re-anchors: genesis still fires only past
stage 2.0, band II ≡ old classes, and the snapshot metric projection pins
counts not text.

## New unit evidence (7 genesis tests total)

- `tetra_arising_band_gate_blocks_and_unlocks_classes`: Safety at 3.5 emits
  zero Political; at 4.1 Political emits.
- `tetra_arising_band_iv_reflective_class_unlocks_at_six`: Meaning Prophecy
  gated below 6.0, unlocked at 6.1.

## Long-horizon implication (measured earlier, now actionable)

The i273/i276 probes measured Safety reaching stage 6 by 20K (seed 42) —
under the ladder this village now has Political-class culture live by ~mid-run
and its full founding class set. The i278 disjointness probe will be the first
test of whether band-diversified generated cultures actually diverge
cross-seed.
