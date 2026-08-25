---
name: wp-0b
description: "AP3 WP-0B: scaffold mindstrata-development crate, wire codegen outputs into canon.rs. Era I."
type: Work-Package-Brief
plan_id: AP3
wave: I
owned_paths: ["crates/mindstrata-development/**", "scripts/afa_codegen.py"]
forbidden_paths: ["crates/mindstrata-person/**", "crates/mindstrata-sim/**", "crates/mindstrata-psych/**", "crates/mindstrata-social/**"]
depends_on: ["WP-0A"]
---

# WP-0B — Crate Scaffold

## Goal

`crates/mindstrata-development` exists, compiles as a leaf on `mindstrata-core`, carries the
generated canon tables, and is otherwise empty. `deny(missing_docs)` from birth.

## Tasks

1. New crate in workspace `Cargo.toml`; deps: `mindstrata-core` only.
2. Module skeletons per 03-substrate §1 (stubs return unimplemented!() only where tests
   don't reach; no TODO comments — missing pieces stay unmerged).
3. Integrate codegen output as `src/canon_gen.rs` (`include!` or copied module — pick one,
   document choice) + hand-written `src/canon.rs` for theory constants that are NOT yet
   attested per-cell (fulfillment thresholds etc.) with placeholder values marked
   `// CALIBRATION-PENDING(AP3): <what the probe must measure>` and a single obvious value.
4. Unit tests: slug↔coord round-trip for all 17; registry lookups; resonance table shape.

## Acceptance gates

- [ ] `cargo clippy -p mindstrata-development` clean; docs complete.
- [ ] Golden replay byte-identical (crate unreferenced by sim yet).
- [ ] Suite green.

## Changelog

| iter | note |
|---|---|
