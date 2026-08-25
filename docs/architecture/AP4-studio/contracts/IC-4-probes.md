# IC-4 — Probe Conventions

```yaml
provider: TOOLS
consumer: ALL
owners: [TOOLS]
frozen_at: PENDING-P0
version: 1.0.0-draft
status: DRAFT
change_orders: []
```

## Purpose

Probes are the project's evidence instrument (AGENTS.md §2 step 2: probe before touching).
This contract makes them findable, reproducible, and auditable.

## Naming law

New probe examples in `crates/mindstrata-benches/examples/` MUST be named
`i<iter>_<slug>.rs` — lowercase slug, underscore-separated, no dates:

- `i266_needs_gate_delta.rs` ✓
- `p5_violence_probe.rs`, `probe_fixes.rs`, `iter182_diag.rs` ✗ (legacy; see below)

**Legacy policy**: at audit time 36 of 45 example files pre-date the convention
(`p<N>_*`, `*_probe.rs`, `*_diag.rs`, `*_sweep.rs`, `*_gate.rs` shapes). They are
indexed as-is by tooling and reported as non-compliant — never bulk-renamed (rename
churn breaks probe references inside historical calibration comments, which are
citation-stable evidence per AGENTS.md §4.2).

## Probe template structure

```rust
//! Probe: <what behavior is measured, one sentence>
//!
//! Evidence context: <iteration/audit question this answers>
//! Horizon/seed: <ticks, seeds> — MUST match the assertion being calibrated.

use mindstrata_sim::Simulation;

fn main() {
    // 1. Construct at the REAL horizon/seed of the question.
    // 2. Run ticks; sample ONLY observable output (means/p90s/events).
    // 3. Print stable, machine-greppable lines: `key=value` rows.
}
```

Requirements per AGENTS.md §2/§3: release-mode runs only (`cargo run --release -p
mindstrata-benches --example <name>`); output is the canonical success signal; a probe
cited in a calibration comment keeps its exact name forever.

## Evidence comment requirements

Any test re-anchor or calibration change cites its probe as:
`measured <value> via <probe-name> at <horizon>, seed(s) <seeds>; old band <x>; mechanism <why it moved>`
(AGENTS.md §4.2 form). Probes themselves carry a header comment naming the question,
horizon, and seeds so the citation resolves without archaeology.

## Enforcement

The naming/index automation lands with the bench-index tooling task; until then
violations are reported by manual sweep on demand. Once landed: new non-compliant names
fail, legacy names stay grandfathered.
