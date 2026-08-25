---
name: ap4-spec-source
description: "AP4 DC-1 requirement catalog — the body of this file is loaded verbatim into .swarm/spec.md via spec_write at P0. FR-ids referenced by plan tasks."
type: Spec-Source
plan_id: AP4
---

# mindstrata DC-1 Requirements

Derived from AP4 interlock contracts (IC-1…IC-8), AP3 substrate doctrine, AGENTS.md
calibration rules, and UM-1. Each FR is mechanically checkable by a reviewer or test.

## Determinism & persistence

- FR-001: All per-tick incremental math computes in f64 shadows with exactly one
  quantization point into Fixed where values enter persistent state or events.
- FR-002: RNG draws are append-only in documented order; new draw sites append at stream
  end with an order comment; birth-path constructors consume in field order.
- FR-003: All passes iterate agents/structures in deterministic index order; unordered
  collection iteration never reaches observable behavior.
- FR-004: Tick passes are pure functions of state snapshot plus pre-drawn catalysts; no
  wall-clock, entropy, or environment reads.
- FR-005: Save artifacts carry a schema version; migrations are pure old→new transforms;
  a round-trip test guards the current format.
- FR-006: Performance budgets (tick rate, memory) are measured in release mode and
  enforced at gates; budget tables cite their measurement basis.

## Catalysts (IC-1)

- FR-010: Mechanical-to-developmental signal flows exclusively through typed
  CatalystEvent records (line weights, Drive, realm refs, f64 magnitude quantized once
  at uptake).
- FR-011: The producer set is exactly: motive pressure deltas, appraisal outputs,
  courtship/marriage/birth/death events, norm violations plus verdicts, gossip uptake,
  ritual participation.
- FR-012: Observers are side-effect-free reads of post-write-back snapshots; zero
  catalysts produce zero field deltas (zero-at-zero).
- FR-013: Skill/craft mastery grants no direct stage credit; witnessed-evaluation
  context only (scale firewall).

## Development fields (AP3 substrate)

- FR-020: Stage/line types derive from vendored ontology tables; band boundaries match
  vendor data exactly; unattested defaults are visible in types, not silent.
- FR-021: The field engine is pure deterministic functions over frozen types.
- FR-022: Contract freezes record hashes; downstream consumers quote freeze hashes.
- FR-023: Person development fields initialize neutral; consumers pass zero-at-zero
  gates (identity at neutral inputs keeps goldens byte-identical until values move).
- FR-024: Pathology follows the ratified 4-fold model (dark/golden × addiction/allergy)
  with Agape/Eros metabolism; identity holds at neutral inputs.
- FR-025: Resonance dynamics weight catalysts by line affinity deterministically.
- FR-026: Development profiles inherit vertically under one-draw-per-trait RNG
  discipline with child-neutrality pins.

## Needs & gating

- FR-027: Action selection consumes fulfillment thresholds from canon values; gating
  activates only after zero-at-zero verification.
- FR-028: Founder line profiles produce measurable behavioral cluster separation
  (differentiation matrix probe above its defined floor) as UM-1 evidence.

## Content grammar (Era III prep)

- FR-030: Content types enforce the three-realm grammar (causal domain × gross referent
  × subtle claim).
- FR-031: Templates embed theory citations per ontology cell; generation is
  deterministic under fixed seed.
- FR-032: Polarity dynamics observe belief claims read-only; reconciliation cases are
  unit-pinned.
- FR-033: Seeded cultures diverge measurably across seeds (culture-disjointness probe).

## Observability & client

- FR-040: Probes follow the naming law and template conventions of IC-4; enforcement
  reports violations on demand.
- FR-041: TUI development lanes render stage positions from IC-2 records through the
  chart component library; feature-flagged until IC-2 stabilizes.
- FR-042: Render hot paths meet the ratified IC-8 frame-time/memory budget; violations
  fail gates.
- FR-043: UI telemetry emits through the IC-7 channel without sim-crate coupling.
- FR-044: Annals trace schema v1 (IC-2) publishes development-stage observability
  records behind a stable flag.
- FR-045: Chronicle hooks respect territory boundaries — STORY never edits CLIENT files.

## Process & gates

- FR-050: Golden replay is the referee: structural changes prove byte-identical goldens;
  behavioral re-anchors register hashes through QA custody.
- FR-051: Calibration value changes land only via approved change-orders citing
  measured value, old band, and mechanism (AGENTS.md §4.2 form).
- FR-052: Lucky-pin detection runs across seed families before milestone close.
- FR-053: Dead producers are bugs even when tests pass; equilibrium probes accompany
  liveness assertions for coupled subsystems.
- FR-054: UM-1 composes six gate families — suite green, golden policy, perf budget,
  playability bar, content coherence, calibration audit — each mechanically checkable
  with an owner.
- FR-055: Territory exclusivity holds throughout: agents edit only ledger-owned files;
  cross-department needs route through contracts and change-orders.
