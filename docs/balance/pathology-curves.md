# Pathology Intensity Curves (skeleton)

Owner: DESIGN → STORY/SIM implementation (FR-024, WP-C operator). Companion to
canon-inventory row 2. Theory grounding: ratified 4-fold model (dark/golden ×
addiction/allergy) with Agape/Eros metabolism — `docs/architecture/AP3-afa/
03-substrate.md` §pathology; vault root `_Ontology/pathologies.md` is the source-of-record
reading (not vendored; cite by sha 868b2239).

## Evidence

- Operator exists as pure-function spec in AP3 substrate; engine lands in
  mindstrata-development dynamics.rs (WP-C). No sim-side expression yet.
- Identity-at-neutral pin required before any behavioral fold (FR-023/FR-012).

## Curve shapes to specify per quadrant

| Quadrant | Onset trigger (catalyst pattern) | Growth law candidate | Ceiling candidate |
|---|---|---|---|
| dark-addiction | repeated drive-satisfying catalysts w/o line progress | saturating exponential | hard clamp pre-calibration |
| dark-allergy | belief-contradicting exposure | threshold+linear | clamp |
| golden-addiction | golden-path overindulgence | logistic | clamp |
| golden-allergy | avoidance of golden claims | slow ramp | clamp |

All four marked CALIBRATION-PENDING(AP3): parameters land only via IC-5 change-orders
with signature-probe evidence (i<iter>_pathology_signature).

## Open questions

1. Metabolism coupling: does Agape/Eros intake rate modulate growth law or ceiling?
2. Interaction with existing psych state (fear/anger saturation hazards — AGENTS.md §5
   dead-producer rule applies: equilibrium probes accompany liveness pins).
