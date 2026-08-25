---
name: ap3-kosmos-index
description: "Curated pointer index into the KosmOS ontology vault for AP3 agents: what each doc governs and when an AFA agent must read it."
type: Reference-Index
plan_id: AP3
vault_root: /home/ishanp/Documents/knowledge-base/KosmOS
---

# refs — KosmOS Ontology Index

Paths relative to `_Ontology/`. The vault is LIVE (a parallel project, actively edited).
Rule: vendor/ snapshots are the build-time source of truth; read the vault directly only
for research/deepening, and re-run WP-0A codegen after meaningful upstream changes.

## Core doctrine (read once at arc start)

| File | Governs | AP3 use |
|---|---|---|
| `CONSTITUTION.md` | invariants of the vault | translated governance → 01-doctrine §6 |
| `stages.md` | THE 17-stage ladder + storage rules | axis definition; frontmatter convention |
| `lenses/rays.md` | altitude→ray lens ("lens never place") | chronicle flavor only; corrects v1 ray-axis error |
| `pathologies.md` | 4-fold pathology model (Ratified) | pathology operator spec — cite per kind |
| `dynamics.md` | Agape/Eros twin dynamics | metabolize() operators |
| `realms.md` | causal/subtle/gross type-firewall | content grammar typing |
| `polarity.md` | reconciliation engine | belief tension states |
| `scale.md` | intra-holonic vertical ≠ altitude | honesty rule §3.4 |
| `emanation.md` | derivation-order of domains | background; domain selection heuristics Era III |
| `holarchy.md` / `holon-anatomy.md` | holon levels/anatomy | background; village-as-holon framing |

## Line definitions & stage cells (per-mechanism citations)

- `lines/<slug>.md` — line definition: measures, quadrant, scope. 19 curated slugs listed
  in 02-theory-map §2.
- `stages/by-line/<line>/<NN>-<stage-slug>/_index.md` — behavioral signature vocabulary.
  Frontmatter carries depth_status; "Expression on this line" is generator/pin source.

## Method artifacts

- `method/line-coupling-map-v1.csv` — 2117 pairwise coupling rows (teal/turquoise/indigo/
  violet attestations; lower altitudes mostly unknown) → resonance defaults policy.
- `method/stage-ladder-17*.csv|json` — machine ladder exports (cross-check vendor/ladder.csv).

## Reading recipes

| Task | Read |
|---|---|
| Implementing a pathology kind | pathologies.md whole + that kind's row; then needs/moral cells near onset band |
| Wiring a line's fulfillment signal | lines/<slug>.md + 2–3 cells spanning the villager band (6–10) |
| Extending resonance matrix | coupling CSV rows for candidate pair; require non-unknown attestation |
| Content template authoring | realms.md §bridges + 5 sample cells of target line for voice/vocabulary |
| Verifying a canon change | CONSTITUTION R4/R7 + the exact cells cited |
