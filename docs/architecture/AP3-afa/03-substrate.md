---
name: ap3-substrate
description: "AP3 engineering design: crate layout, data structures, determinism contracts, pass placement, codegen pipeline. Interfaces here are contract-freeze candidates."
type: Architecture-Plan-Reference
plan_id: AP3
---

# 03 — Substrate Design

## §1 Crate topology

New leaf crate `crates/mindstrata-development` — born with `#![deny(missing_docs)]`,
depends ONLY on `mindstrata-core`. DAG after landing:

```
core ← development ← {person, psych, social, institutions, world} ← sim
```

```
crates/mindstrata-development/src/
├── lib.rs              # crate root, re-exports
├── canon.rs            # FROZEN(AP3) theory constants: ladder table, resonance defaults,
│                       #   pathology signature parameters, Λ window width. Canon changes
│                       #   require user ratification (doctrine §6).
├── stage.rs            # StageCoord, StageSlug, altitude mapping, band predicates
├── line.rs             # LineId (11 individual + 8 collective), LineSet, stage_lines map
├── field.rs            # DevelopmentField: per-line press/blockage/pathology state + math
├── dynamics.rs         # transcend-and-include gate, press accumulation, resonance leakage
├── pathology.rs        # 4-fold pathology onset/metabolism (Agape/Eros operators)
├── lambda.rs           # frontier computation, catalyst gating window
├── content/
│   ├── mod.rs          # three-realm types: Domain(Causal), Framework(Subtle), Entity(Gross)
│   ├── grammar.rs      # compositional template engine (domain × referent × stance × line-sig)
│   └── polarity.rs     # reconciliation graph states + refute/reconcile transitions
├── snapshot.rs         # KosmOS-frontmatter export of stage_lines maps
└── tests/              # unit pins per module (gradient monotonicity, no-skip, etc.)
```

## §2 Core types (contract sketch)

```rust
pub struct StageCoord(u8);            // 1..=17; TryFrom, slug lookup via canon
pub struct LineId(u8);                // index into canon LINE_REGISTRY

pub struct LineState {
    pub stage: f64,        // shadow coordinate in [1..=17]; quantized view = StageCoord(floor)
    pub press: f64,        // transition accumulator toward next stage
    pub fulfillment: f64,  // EMA of stage-adequate signal, drives gate
    pub pathology: PathologyState,
}

pub struct PathologyState {
    pub kind: PathologyKind,   // None | DarkAddiction | DarkAllergy | GoldenAddiction | GoldenAllergy
    pub intensity: f64,        // 0..1
}

pub struct DevelopmentField {
    pub lines: [LineState; N_INDIVIDUAL_LINES],
    pub lambda: f64,
}

// Village holon:
pub struct CollectiveField {
    pub lines: [LineState; N_COLLECTIVE_LINES],
    pub lambda: f64,
}
```

Determinism contracts:
- All arithmetic f64; ONE quantization point per tick where state crosses into Fixed or
  event output (`Fixed::from_f64`).
- Field update is a PURE function of (field snapshot, catalyst vector, canon constants) —
  zero RNG inside the pass. All stochasticity enters as pre-drawn catalysts.
- Founder initialization draws at END of populate stream, uniform within per-line bands;
  draw order documented in WP-E and frozen thereafter.

## §3 Catalysts: the input vocabulary

`CatalystEvent` variants produced by existing passes (read-only observers), each carrying
`(line_tags: SmallVec[(LineId, f64)], drive: Drive, realm_refs)`:

| source system today | becomes |
|---|---|
| motive pressure deltas (needs) | Red-band fulfillment signals for needs line |
| appraisal outputs | Agency/Communion/Eros/Agape drive-weighted catalysts |
| courtship/marriage/birth/death | Green-band relational catalysts |
| norm violations, legal verdicts | justice-line catalysts; refutation events |
| gossip meme uptake | belief-graph reconciliation proposals (subtle layer) |
| ritual/festival participation | Agape metabolizers; culture-line collective catalysts |
| apprenticeship/craft mastery | NOT stage credit (scale firewall) — provides witnessed-evaluation contexts that can trigger Golden-Allergy |

## §4 Pass placement

One daily-tick pass `sim/pass_development.rs`, appended AFTER all six existing passes:
reads day's catalyst buffer (built by observers), updates person `DevelopmentField`s and
the village `CollectiveField`, writes events to the annals channel. Follows the verbatim
pass pattern; consumes snapshots taken after biology write-back loop (mem::take hazard).

Person state home: `mindstrata-person` gains `development: DevelopmentField` on the psyche
side (WP-E owns placement); village field lives on Simulation root (WP-I).

## §5 Content generation pipeline (Era III)

```
world referents (gross: sites/resources/events/institutions)
  × causal domain selection (weighted by collective line stages)
  × stance/polarity state (reconciled / active-tension / undiscovered)
  × individual line-signature targeting (which lines the item resonates with)
  ⇒ generated Subtle items: memes / norms-proposals / ritual forms / narrative beats
```

Replaces `seed_initial_memes` fixed roster (sim/population.rs:107). Grammar templates are
canon-frozen STRUCTURE; referent bindings are runtime VARIABLE. Every generated item
type-checks against three-realm rules (meme cites gross evidence within one domain).

Belief evolution rides polarity graph: reconcile/refute transitions emitted by existing
belief_update/gossip passes become graph mutations; moral panics emerge as active-tension
cascades — observable, testable.

## §6 Calibration surface (what Era II+ probes measure)

- Fulfillment thresholds per band (needs-line cells give qualitative ordering:
  survival→belonging→achievement→self-actualization→self-transcendence).
- Pathology onset intensities + Agape/Eros metabolic rates.
- Resonance leak fractions from attested coupling strengths.
- Λ window width (starts 1.0; widen only with probe evidence of starvation, Q1 below).

Each lands with probe evidence naming measured value, old behavior, mechanism — AGENTS.md
§4.2 form. Open research questions carried from planning: Q1 Λ-window starvation vs event
density; Q2 minimum viable resonance set; Q3 grammar size before seed-disjoint cultures
(20K ticks); Q4 collision of pathology signatures with Iter-248 Whitehall equilibria.

## §7 Observability & export

- Per-quadrant trace of every stage transition + pathology onset/heal (annals events).
- `snapshot.rs` exports holon `stage_lines` maps as KosmOS-style YAML frontmatter so
  chronicles/TUI render ontology-compatible coordinates (altitude slugs resolve via canon —
  R7 compliance).
- TUI longitudinal charts gain a line-stage lane per selected agent + village panel (Era V,
  coordinated with the parallel session's chart scaffolding — file ownership in wave ledger).
