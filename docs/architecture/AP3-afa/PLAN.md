---
name: ap3-afa
description: "Attractor-Field Architecture (AP3): staged developmental lines for mindstrata agents and the village holon, directed by the KosmOS ontology. Entry point for a 50–100 iteration multi-agent arc. Load this file first; it routes everywhere else. Triggers: implementing AFA waves, 'continue AFA', planning an AFA iteration, resolving AFA scope conflicts."
type: Architecture-Plan
plan_id: AP3
supersedes: docs/PLAN_ATTRACTOR_FIELD_ARCHITECTURE.md (uncommitted v1, deleted)
status: Active
created: 2026-08-25
iteration_range: "266 – ~365"
ontology_source: /home/ishanp/Documents/knowledge-base/KosmOS/_Ontology (see refs/kosmos-index.md)
---

# AP3 — Attractor-Field Architecture

## Mission

Replace mindstrata's calibrated-constant psychology with **theory-derived attractor fields**:
every villager and the village itself carry **developmental line coordinates** on the KosmOS
17-stage unified ladder; four invariant mechanisms (stage gate, resonance, pathology,
Λ-gating) direct development; all cultural content becomes **generated**, composed from
line-configurations × live world referents under three-realm typing. Structure directs;
content emerges.

One-sentence theory anchor: *"The stage sequence is fixed and universal; the lines through
which it expresses are many; pathologies are metabolic distortions of transcend-and-include;
and content is whatever the field metabolizes."* — synthesized from KosmOS `_Ontology/stages.md`,
`pathologies.md`, `realms.md`.

## Navigation map (read in this order for your role)

| You are… | Read |
|---|---|
| Any agent starting on AP3 | this file → [01-doctrine.md](01-doctrine.md) §1–§4 → your WP brief |
| Implementing a work package | [waves/](waves/) index → your WP brief → [02-theory-map.md](02-theory-map.md) sections it cites → [03-substrate.md](03-substrate.md) for interfaces you consume |
| Planning / re-planning iterations | [04-waves.md](04-waves.md) → [01-doctrine.md](01-doctrine.md) §5 (conflict protocol) |
| Calibrating or re-anchoring pins | [01-doctrine.md](01-doctrine.md) §3 → AGENTS.md §4 → [03-substrate.md](03-substrate.md) §6 |
| Looking up ontology sources | [refs/kosmos-index.md](refs/kosmos-index.md) |
| Locating existing sim code | [refs/sim-inventory.md](refs/sim-inventory.md) |

## The five non-negotiables (details in doctrine)

1. **File-ownership ledger.** Parallel agents never share write scopes. Every WP brief
   lists OWNED paths and FORBIDDEN paths; the ledger in [04-waves.md](04-waves.md) §3 is
   authoritative. Check `git log --oneline -3 && git status` before every session — a
   parallel session shares this clone and lands commits mid-flight.
2. **Contract-freeze before fanout.** Interface types land and merge BEFORE dependent WPs
   spawn. Frozen contracts carry a `// FROZEN(AP3): <wave>` marker; changes need wave-owner
   sign-off recorded in the WP brief's changelog block.
3. **Golden replay referees merges.** Any structural change must be byte-identical on
   golden runs; any behavioral change owns its re-anchor evidence per AGENTS.md §4.
4. **Probe before touching; observable output is the only success signal.**
5. **Every behavioral claim cites its derivation** — probe output, ontology cell path, or
   both (KosmOS CONSTITUTION rule 12 adopted).

## Current state

- Era I not started. Next actionable: [waves/WP-0A](waves/WP-0A-vendor-codegen.md).
- Iterations 264–265 were consumed by the parallel session (TUI dossier, annals
  provenance). AP3 numbering starts at **266**.
