---
name: ap3-theory-map
description: "AP3 theory mapping: KosmOS ontology concepts → mindstrata simulation mechanisms, with exact source citations. The canonical crosswalk for all WP briefs."
type: Architecture-Plan-Reference
plan_id: AP3
---

# 02 — Theory Map (KosmOS → mindstrata)

All paths under `KOSMOS = /home/ishanp/Documents/knowledge-base/KosmOS/_Ontology` unless
noted. Curated reading index: [refs/kosmos-index.md](refs/kosmos-index.md).

## §1 The master axis: 17-stage unified ladder

Source: `KOSMOS/stages.md` (ratified). Canonical coordinates:

| # | slug | altitude | identity band | MHC | Kegan |
|---|------|----------|---------------|-----|-------|
| 1 | prehension | infrared | pre-egocentric | 0 | — |
| 2 | irritability | infrared | pre-egocentric | 1 | — |
| 3 | basic-impulse | magenta | egocentric | 2 | 0→1 |
| 4 | emotion-complex | red | egocentric | 3 | 1 |
| 5 | sensory-motor | red | egocentric | 4 | 1→2 |
| 6 | nominal-rule | amber | ethnocentric | 5 | 2 |
| 7 | sentential-faith | amber | ethnocentric | 6 | 2 |
| 8 | preoperational | amber | ethnocentric | 7 | 2→3 |
| 9 | primary-modern | amber | ethnocentric | 8 | 3 |
| 10 | concrete-peak | amber | ethnocentric | 9 | 3 |
| 11 | abstract-emerge | orange | ethno→worldcentric | 10 | 3→4 |
| 12 | formal-landmark | orange | worldcentric emerging | 11 | 4 |
| 13 | systematic-green | green | worldcentric pluralistic | 12 | 4 |
| 14 | metasystematic-teal | teal | worldcentric mature | 13 | 4→5 |
| 15 | paradigmatic-turq | turquoise | worldcentric stabilized | 14 | 5 |
| 16 | cross-paradigmatic | indigo | kosmocentric | 15 | 5 |
| 17 | meta-cross-violet | violet | unity/nondual | 16 | 5+ |

Sim representation: `StageCoord(u8)` 1..=17 + slug table in `canon.rs`. Villager-relevant
band is 1..=13; 14+ exists for the village holon and long-horizon studies only.

**Correction vs v1 plan**: rays are NOT the axis. `KOSMOS/lenses/rays.md`: *"A lens, never
a place — diagnosis runs on the neutral altitude scale."* The ray overlay maps altitudes to
Law-of-One functions and is used only for chronicle flavor text (Era V).

## §2 Line registry → sim lines

Sources: `KOSMOS/lines/<line>.md` definitions; per-line×stage behavioral cells at
`KOSMOS/stages/by-line/<line>/<NN>-<slug>/_index.md` ("Expression on this line" section =
the behavioral signature vocabulary).

Two holon scopes, mirroring the ontology's own `holon_scope` field:

**Individual lines** (person holons; UL/UR quadrants):

| line slug | quadrant home | sim signal source today |
|---|---|---|
| needs | UL/UR | MotivationState motive pressures (19 categories) |
| cognitive | UL | belief_update maturity, journal complexity |
| emotional-interpersonal | UL | appraisal outputs, relationship affect |
| moral | UL/LL | norm internalization state, moral values inheritance |
| values | UL | inherited moral values vector |
| worldview | UL | belief graph coherence |
| meaning | UL | meaning/certainty/justice motives |
| intrapersonal | UL | interoception activation (Iter-247) |
| somatic / physical-vitality | UR | biology passes (read-only signals) |
| spiritual | UL | theology/sacred-value systems |
| willpower | UR→UL | motivation override events |

**Collective lines** (village holon; LL/LR quadrants):

| line slug | quadrant home | sim signal source today |
|---|---|---|
| culture | LL | meme pool composition |
| governance | LR | institutions legitimacy |
| social-systems | LR | faction/clan/household topology |
| economic-systems | LR | market/market-legitimacy state |
| justice | LR | legal institution caseload/outcomes |
| mythos / narrative / storytelling | LL | chronicle events, founding myths |
| civilization-axioms | LR | (new; Era IV) deepest institutional commitments |

A holon's position set = `stage_lines: { <line>: StageCoord | Unattested }` — exactly the
ontology's frontmatter convention (`stages.md` §Storage rules A). Sim snapshots can export
this map verbatim for cross-referencing.

## §3 The four invariant mechanisms

### 3.1 Transcend-and-include gate
Stages contain their predecessors (`pathologies.md`: "a holon at altitude N contains
stages 1…N-1 as its foundation"). Sim rule: press toward stage N+1 accumulates only while
stage N's fulfillment integral exceeds threshold AND no active Dark pathology on the same
line blocks consolidation. No skipping. Implemented in `development/dynamics.rs`.

### 3.2 Resonance matrix (sparse)
Source data: `KOSMOS/method/line-coupling-map-v1.csv` — 2117 pairwise rows with strengths
(strong/moderate/weak/unknown) attested at teal/turquoise/indigo/violet. Lower-altitude
cells are mostly `unknown`. Sim v1 policy: use attested cells where they exist; treat
`unknown` as weak-with-flag (loggable), never silently strong. Matrix is canon-frozen;
extending it requires citing new ontology evidence.

### 3.3 Pathology operator (supersedes v1's simple "blockage")
Source: `KOSMOS/pathologies.md` (Ratified). Four kinds on two axes — pole
(Dark=submergent lower / Golden=emergent higher) × direction (Addiction=hyper-ingestion /
Allergy=hypo-ingestion):

| pathology | signature (theory) | sim behavioral translation (testable pin) |
|---|---|---|
| Dark-Addiction | clinging to lower-stage structures; repetition compulsion | obsolete-strategy persistence: hoarding when scarcity resolved; ritualized grief loops |
| Dark-Allergy | skipped foundation; fragile ego-state | volatile overreaction to norm friction; low resilience after violation shocks |
| Golden-Addiction | bypassing; premature transcendence | professing high-stage content while Red-band needs unmet (cult-leader pattern); credibility gap observable to peers |
| Golden-Allergy | Jonah complex; refusing emergence | threshold met but transition refused; competence suppression under witnessed evaluation |

Healing drives (same source): **Agape** descends to metabolize Dark (communal embrace:
mourning rites, care reception); **Eros** answers the Golden call *grounded* (call +
foundation, never maximized); **Agency**/Communion process horizontal polarity.

### 3.4 Λ focal gating
Per-holon frontier Λ = stage-weighted mode across its attested lines. Catalysts whose line
resonates outside window [Λ−1, Λ+1] register at reduced weight. Spiral dynamics
(forward→back→forward) emerge because active pathologies depress Λ, re-opening lower-window
catalysis. This implements the adept-gate intuition from `lenses/rays.md` §1st/2nd-tier
without importing its esoterics into mechanics.

## §4 Three-realm typing = the content grammar

Source: `KOSMOS/realms.md`, `emanation.md` §4, `polarity.md`.

| KosmOS realm | sim referent | examples |
|---|---|---|
| **Causal** (domain/lens, formless, stage-less) | reasoning domains villagers operate within | justice, religion-statistical, health, governance domains |
| **Subtle** (epistemic models — maps, never territory) | beliefs, memes, norms-as-claims, rituals-as-prescriptions | a meme claiming "the river spirit demands tribute" |
| **Gross** (ontic entities) | agents, sites, resources, institutions, events | the mill, the famine of year 34, Elder Mora |

Type firewall ⇒ generated content must type-check: a **subtle** item (meme/norm/belief)
cites ≥1 **gross** entity_evidence within exactly one **causal** domain, held via a
framework with line-signature. Bridges typed: observes / models / frames
(`holon-anatomy.md`). This is what makes two histories generate disjoint cultures from one
grammar: same types, different referent bindings.

Polarity reconciliation (`polarity.md`) drives subtle-layer evolution: claims carry
thesis⟷antithesis tension states {reconciled, active-tension, undiscovered}; gossip/belief
events propose reconciliations; refutation events re-open reconciled pairs. Moral panics =
active-tension cascades; norm crystallization = reconciliation synthesis. Maps onto
existing belief/journal/gossip machinery WITHOUT replacing its transport mechanics.

## §5 Tetra-arising

Individual-line movement (UL/UR) and collective-line movement (LL/LR) co-arise: village
collective stages gate which cultural CONTENT can exist; individual stage distributions
gate UPTAKE. Neither alone suffices — this is the mechanism behind Era III/IV coupling.
Quadrant observability hooks trace every transition to its quadrant(s) (CONSTITUTION R6).

## §6 What AFA does NOT adopt

- The Law-of-One esoteric layer beyond flavor text (rays.md stays a lens).
- Directory-as-stage storage (ontology stores stage ONLY in frontmatter coordinates —
  same discipline: stages live in state fields, never in module names).
- The 50-line registry wholesale: 11 individual + 8 collective lines above are the v1
  curated subset; adding lines is a canon change requiring cited cells for every stage
  the line will attest.
