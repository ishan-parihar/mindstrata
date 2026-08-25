# IC-3 — Determinism Law

```yaml
provider: PLATFORM
consumer: ALL
frozen_at: PENDING-P0 (draft below freezes as-is unless amended at P0)
version: 1.0.0-draft
change_orders: []
```

## Purpose

Every run of the simulation, on any machine, at any optimization mode (release), with the
same seed and inputs, produces bit-identical state trajectories and artifacts. This is the
property golden replay certifies and every department depends on.

## Surface

1. **Numeric discipline**: per-tick incremental math uses f64 shadows; ONE quantization
   point into `Fixed` (`Fixed::from_f64`) where values enter persistent state or events.
   No raw float accumulation inside Fixed-domain state.
2. **RNG stream law**: draws are append-only in documented order; birth-path constructors
   consume in field order; new draw sites append at stream END with a comment naming count
   and order. Never insert draws mid-stream.
3. **Iteration order**: all passes iterate agents/structures in deterministic index order;
   no HashMap iteration may reach observable behavior.
4. **Pass purity**: tick passes are pure functions of (state snapshot, pre-drawn catalysts);
   no wall-clock, no thread-local entropy, no environment reads.
5. **Save schema**: versioned; migrations are pure old→new transforms; round-trip test is
   part of the gate.

## Guarantees

- PLATFORM maintains the budget table + benchmark harness; determinism regressions are
  release-blocking and bisected via pristine-archive recipe (AGENTS.md §3 / QA runbook).
- Consumers may rely on replay identity for any fixed seed family committed to the registry.

## Obligations

- Consumers NEVER introduce float nondeterminism (no parallel reduction without ordered
  combine), never reorder draws, never iterate unordered collections observably.
- Violations found in foreign territory are reported to PROD, not patched across territories.

## Tests guarding this contract

- Golden replay suite (`scripts/gate --full`); save/load round-trip harness (QA phase 5);
  seed-family determinism runner (QA phase 6).

## Changelog

| ver | change | approved |
|---|---|---|
| 1.0.0-draft | initial from AP3 doctrine §2 + AGENTS.md §5 | P0 pending |
