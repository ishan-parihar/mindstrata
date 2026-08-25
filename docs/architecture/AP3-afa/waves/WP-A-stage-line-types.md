---
name: wp-a
description: "AP3 WP-A: stage.rs + line.rs public types, FROZEN contracts for the wave. Era I, first of three parallel substrate WPs."
type: Work-Package-Brief
plan_id: AP3
wave: I
owned_paths: ["crates/mindstrata-development/src/stage.rs", "crates/mindstrata-development/src/line.rs"]
forbidden_paths: ["crates/mindstrata-development/src/{dynamics,pathology,lambda,field}.rs", "other crates"]
depends_on: ["WP-0B"]
freeze_note: "Types in this brief are the wave's FIRST freeze point. B/C/D codegen against them."
---

# WP-A — Stage & Line Types

## Deliverables

`stage.rs`: `StageCoord(u8)` (TryFrom<u8> 1..=17), slug()/altitude()/band() via canon,
band predicates (is_red_band etc. per lens functions), Display as `NN-slug`.

`line.rs`: `LineId(u8)`, `Scope::{Individual, Collective}`, registry access
(`line_scope`, `line_quadrant`), `StageLinesMap` (array of `Option<StageCoord>` +
attested iteration; mirrors ontology frontmatter convention).

Unit pins: every coord round-trips; unattested default is visible not silent; band
boundaries match vendor ladder.csv exactly.

**Freeze ritual**: land → merge → announce contract hash in this file:

```
FROZEN(AP3/W-I): stage.rs@<sha>, line.rs@<sha> — sign-off WP-A owner <date>
```

## Acceptance gates

- [ ] clippy clean, deny(missing_docs) holds
- [ ] golden replay byte-identical (no sim consumers yet)
- [ ] contract hashes recorded here

## Changelog

| iter | note |
|---|---|
