---
name: wp-0a
description: "AP3 WP-0A: vendor ontology extracts into repo + build codegen pipeline. Era I, serial-first."
type: Work-Package-Brief
plan_id: AP3
wave: I
owned_paths: ["docs/architecture/AP3-afa/vendor/**", "scripts/afa_codegen.py"]
forbidden_paths: ["crates/**"]
depends_on: []
---

# WP-0A — Vendor & Codegen

## Goal

Make the ontology machine-consumable INSIDE this repo, with provenance, so no sim code
ever reads `/home/ishanp/Documents/knowledge-base/KosmOS` at runtime or build time.

## Tasks

1. `git -C /home/ishanp/Documents/knowledge-base/KosmOS rev-parse HEAD` → record sha.
2. Vendor into `docs/architecture/AP3-afa/vendor/`:
   - `ladder.csv` — 17 rows from `_Ontology/stages.md` table (num, slug, altitude, band, mhc, kegan)
   - `lines.csv` — curated 19 lines (11 individual + 8 collective) with quadrant + scope from `lines/<slug>.md`
   - `couplings.csv` — copy of `_Ontology/method/line-coupling-map-v1.csv` (verbatim, provenance header)
   - `cells/<line>/<NN>-<slug>.md` — copy ONLY the "Expression on this line" body sections for the 19 lines × 17 stages that exist (`depth_status: complete` preferred; mark partials)
   - `PROVENANCE.md` — KosmOS git sha, copy date, per-file source path table
3. Write `scripts/afa_codegen.py`: reads vendor CSVs → emits Rust tables:
   - `ladder.rs` (StageCoord slugs, altitudes, bands)
   - `registry.rs` (LineId registry)
   - `resonance_default.rs` (attested coupling cells only; unknowns omitted, counted)
   Deterministic byte output; run twice = identical files (self-test).
4. Emit a coverage report: per line, how many of 17 stage cells are complete/partial/missing.

## Acceptance gates

- [ ] Codegen idempotent (byte-identical rerun).
- [ ] Coverage report committed under `vendor/COVERAGE.md`.
- [ ] No crate code touched. Suite untouched-green.

## Changelog

| iter | note |
|---|---|
