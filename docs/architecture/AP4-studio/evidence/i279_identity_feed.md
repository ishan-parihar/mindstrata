# Iteration 279 — Identity-bucket feed (sweep-ratified catalyst parity)

**Status:** LANDED · **Doctrine:** probe-first (§2), sweep evidence before any constant, zero re-anchors.

## The question

The Identity collective bucket was the last dead collective producer at N=12:
its only live feed was `GriefStruck` (i272), and griefs require mortality
horizons (~2.3M ticks). The §6.5 plan named two candidate feeds — fractional
ritual press vs grief-proxy events — and this iteration probed before
building.

## What was measured (i279_identity_fraction_sweep)

Design under probe: MAJOR conflicts (Violence/Combat/Revolution/Feud — the
i275 severity discriminator) press the Identity bucket with weight `f`, the
collective reading of the same event that produces the individual-level
Identity claim ("this violence reshapes who we are").

Fraction sweep (seed 42, 20K, N=12), with the env affordance removed before
landing:

| f | Identity stage | majors | feuds | griefs | id press sum @f=1.0 | accumulated @0.05 |
|---|---|---|---|---|---|---|
| 0.00 | 1.00 | 29 | 26 | 0 | — | 0.000 |
| 0.25 | 1.00 | 29 | 26 | 0 | 9.167 | 0.458 |
| 0.50 | 1.00 | 29 | 26 | 0 | 9.167 | 0.458 |
| 1.00 | 1.00 | 29 | 26 | 0 | 9.167 | 0.458 |
| 2.00 | 1.00 | 29 | 26 | 0 | 9.167 | 0.458 |

**Verdict: no fraction reaches the genesis gate (2.0) by 20K.** The binding
constraint is the diet, not the weight: 55 major events × 2 catalysts / 12
agents = 9.167 raw press → 0.458 accumulated (press_growth 0.05) — half the
1.0 needed for stage 2. Griefs = 0 (mortality-blocked, i272 debt). This is
the i274 Relational verdict reproduced on a second bucket: the collective
pacing constants are *correct*; the catalyst diets at N=12 are the constraint.

## What landed

- `IDENTITY_PRESS_FRACTION = 1.0` constant in
  `systems/development.rs::system_collective_field_step`, with the full sweep
  evidence in the doc comment (§4.2 rule: measured value, old band, mechanism).
  f=1.0 is chosen because it is the theoretically honest weight — catalyst
  parity with Grief, Identity's canonical feed (each major event genuinely
  reshapes the village's self-image) — and projects stage 2 at ~44K ticks,
  the same real-cultural-timescale class as Relational's ~80K (i274). No
  magnitude knob was pulled to pass a probe.
- **The env override affordance was removed before landing** — env-dependent
  simulation is a determinism hazard (§5 RNG/state discipline); the sweep
  affordance exists only in probe commits.
- `collect_catalysts` promoted to `pub(crate)` for in-crate test access.
- New pin `major_conflict_presses_identity_bucket_minor_does_not`: Violence
  flags major → identity-bucket lines receive press; verbal Threat stays
  Safety-only; minor conflicts leave the identity bucket at press 0.

## Zero-blast proof

Full gate GREEN (307/0/1) with **zero re-anchors**: goldens (2K/5K horizons)
and the 10K snapshot are byte-stable because 10K accumulates only ~0.23
identity press — below the 1.0 stage-2 threshold everywhere. clippy 0
warnings, bench naming law 0 violations (90 indexed), sim lib 227/227.

## Debt recorded

- **Identity genesis projects to ~44K ticks at N=12** — live-but-slow, same
  honest pacing class as Relational (~80K). Scale-invariant: majors ×
  attendance-equivalent / N.
- **The diet, not the weights, gates UM-2's full disjointness closure.** The
  named accelerators (§6.5 i283 options): festival-dense regimes (recurring
  Relational+Identity press), mortality-horizon grief feeds, or larger N.
  Attempting to compensate with a magnitude knob would violate §4.4 and the
  §5 hazard ledger (founder-variance lesson: don't reshape the diet piecemeal).
