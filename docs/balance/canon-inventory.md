# Canon Inventory — CALIBRATION-PENDING Sweep

**Method**: `grep -rn "CALIBRATION-PENDING" --include=*.rs crates/` plus repo-wide sweep
over `*.{rs,md,toml}` (2026-08-25). Marker convention defined in
`docs/architecture/AP3-afa/waves/WP-0B-crate-scaffold.md` (form:
`// CALIBRATION-PENDING(AP3): <what the probe must measure>`) and consumed by IC-5
(`03-interlock-map.md`).

## Finding

**Zero markers exist in compiled code today.** The `mindstrata-development` crate that
will host `canon.rs` does not exist yet (WP-0B pending), so no constant currently carries
the marker. The five repo-wide hits are all documentation references to the convention
itself (`04-cycle-plan-DC1.md`, `03-interlock-map.md`, two AP3 wave docs,
`MINDSTRATA_CURRENT_STATE.md` historical note).

## Forward registry — constants that WILL carry markers at scaffold time

Sourced from the balance-doc skeleton plan and existing sim surfaces awaiting theory
grounding. Each becomes a `canon.rs` entry with an owner department and probe pointer
when WP-0B lands; owners per the AP4 ledger.

| # | Future canon entry | Subsystem | Owner | Theory cell | Probe plan |
|---|---|---|---|---|---|
| 1 | Needs-band fulfillment thresholds (per drive: safety/certainty/justice/play/belonging/romance/meaning) | Needs gating (WP-G1) | DESIGN→SIM | AP3-afa 03-substrate §drives | bench i<iter>_needs_gate_delta |
| 2 | Pathology intensity curves (onset/growth/ceiling per 4-fold quadrant) | Pathology operator (WP-C) | DESIGN→STORY | 03-substrate §pathology | bench i<iter>_pathology_signature |
| 3 | Stage band boundaries (somatic 3..=7; spiritual/values wide 3..=11 et al.) | Stage ladder mapping | STORY | WP-EFGH era2-person-wiring §bands | vendored-table equality pin |
| 4 | Resonance line-affinity weights | Dynamics (WP-C resonance) | STORY | 03-substrate §resonance | determinism pin + sensitivity probe |
| 5 | Catalyst uptake quantization point (f64→Fixed single-step) | Catalyst bus (IC-1) | SIM/PLATFORM | IC-3 numeric discipline | zero-at-zero pin suite |
| 6 | Pacing horizons (player-experienced milestones at 10K/50K/100K ticks) | Pacing model | DESIGN | UM-1 playability bar | long-horizon probes |
| 7 | Founder line-profile distribution parameters | Differentiation matrix (FR-028) | SIM | AGENTS.md §5 founder-variance debt | differentiation matrix probe |

## Coverage arithmetic

grep hits: 5 (all documentation-of-convention; 0 code sites). Table rows above: 7
forward entries. Sites unclassified: 0. Re-run command recorded in header reproduces
both counts.
