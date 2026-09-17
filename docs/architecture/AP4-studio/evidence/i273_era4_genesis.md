# Iteration 273 — Era IV collective activation: stage-gated meme genesis (i273)

**Status:** LANDED · **Doctrine:** probe-first (i273 stage probe), causal control
(stash-run), §4.2 re-anchor with named mechanism.

## What was missing

Substrate §5 names the Era IV consumer: content generation weighted by collective
line stages, "replacing the `seed_initial_memes` fixed roster". Until i273 the
village `CollectiveField` stepped live (i266) and had exactly one consumer
(Safety-fulfillment panic-pacify) — no collective line had ever *generated*
content. The village's culture began and ended with 3–5 hardcoded seed memes.

## Probe evidence (`i273_collective_stage_probe`, seed 42, N=12)

| Horizon | mean stage | σ | max stage (line) | genesis memes |
|---|---|---|---|---|
| 2 000 ticks | 1.0000 | 0.000 | 1.0 (aesthetic) | **0** |
| 20 000 ticks | 2.9310 | 1.999 | 5.0 (civilization-axioms) | **4** |

Findings:
1. **Stage signal is real but long-horizon**: every line sits at exactly 1.0
   through the golden horizon (CV 0.000); differentiation begins ~5–8K ticks.
   A genesis gate keyed on "any line ≥ 2.0" is therefore provably inert at all
   calibration horizons and live beyond — the zero-blast contract.
2. The deepest line at 20K is `civilization-axioms` — the theory map's Era IV
   line (LR, "deepest institutional commitments"). The engine found the canon's
   own axis without being told.
3. **Catalyst-diet asymmetry (recorded debt)**: only Safety-bucket genesis fires
   at N=12 because Threat/Transgression catalysts dominate the natural event
   diet (conflicts frequent; marriages/births rare). Relational bucket read is
   0.0025 at 20K. Not a bug — a small-village diet artifact; revisit at N≥48
   (DC-3) or with forced-Bond shock scenarios (i268 pattern).

## Implementation

- `mindstrata-development::collective::bucket_for_line` made `pub` — the
  vendored-`kind` affinity is the single source of bucket membership.
- `systems/genesis.rs`: `system_collective_genesis` — when a bucket's deepest
  line crosses an integer stage ≥ `GENESIS_STAGE_GATE` (2.0), register a meme
  keyed to that bucket's domain, tagged `[genesis:{Bucket}:{epoch}]` in the
  description. Dedup by tag scan = no new serialized state, restore-safe under
  `from_snapshot`. Deterministic templates (STRUCTURE) + line slug (VARIABLE),
  cite-first per the Era III render law. Genesis memes mutate slowly (0.03):
  institutional memory, not gossip.
- Wired in `core.rs` immediately after `system_collective_field_step`, sharing
  the same tick pass (no extra event walk).

## Unit pins (3/3)

- `genesis_is_identity_below_stage_gate` — founding-stage field births nothing.
- `genesis_fires_on_advanced_line_and_dedups` — one crossing → one meme;
  re-run idempotent; next crossing → next epoch.
- `genesis_descriptions_are_deterministic_and_cite_first` — byte-identical
  across registries; text cites its line slug.

## Zero-blast proof + causal control

- 2K-gate suite: fully green unchanged (all snapshots/goldens at ≤2K horizons).
- **Causal control (stash-run)**: with the core.rs wiring stashed, the 10K
  snapshot passes on the old baseline (grain 0.9127, memes 3). Live: memes 5,
  grain 1.4565. Genesis is the *only* cause of the drift.
- Probe (post-wiring, release): 0 genesis memes at 2K, 4 at 20K — matches the
  stage probe exactly.

## §4.2 Re-anchors (mechanism named)

1. **10K surface snapshot** (`long_horizon_surface_10000_ticks`):
   memes 3→5 (the genesis memes), events 147783→147935 (+152 meme
   transmissions), memory traces 1021→1073 (Emotional 549→612, Social
   441→420), stress 0.3498→0.3467, grain 0.9127→1.4565 (effort
   reallocation under new identity-relevant content), water −0.18.
   Old band anchored a world with only hardcoded memes; the contract
   (surface-level observability of the memetic ecology) is unchanged.
2. **Collapse golden** (`golden/collapse/seed_42`): crisis world concentrates
   collective press faster, so genesis fires within its horizon. Same
   mechanism; hash re-pinned after regeneration.

## Debt recorded (honest)

- Safety-only genesis at small N (catalyst-diet asymmetry) — revisit at DC-3
  N≥48 or with relational shock scenarios.
- Genesis templates are v1 single-template-per-bucket; grammar growth
  (multiple templates × template selection hashing, i269 FNV pattern) is the
  natural Era IV continuation once more lines differentiate.
- `seed_initial_memes` remains as founding culture (correct per substrate —
  genesis *grows* from, not replaces, the founding roster).
