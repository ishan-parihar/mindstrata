# Iteration 301 — asset pipeline v0 (last DC-3 checklist item)

**Status:** LANDED · **Root cause owned:** the perf charter §5 item 3 and
ROADMAP DC-3 named "asset pipeline v0" as the last unbuilt DC-3 deliverable —
the gate that opens the CLIENT art/audio sub-charters toward UM-4.

## What landed

1. **`ASSET-PIPELINE-v0.md` charter** (`charters/`): defines "asset" as a
   deterministic pure function of live state consumable by clients (IC-8);
   five binding rules — read-only, determinism, versioned schema, bounded
   journals, one-shot perf outside per-tick budgets. v0 document shape:
   `schema_version, meta, world.sites, polities[{members, stage_lines}],
   culture[{id, description, content_type, created_tick, hosts}], annals`.
2. **`sim/assets.rs`** (`mindstrata-sim`): `export_world_assets(_json)` —
   pure over public accessors; reuses `export_stage_lines` (i290 canon
   export) per polity and `render_chronicle` (i259) for annals; culture
   carries the i300 per-agent hosting ledger.
3. **CLI**: `--export-assets <path>` on the `sim` command.
4. **Probe** `i301_asset_export` + **4 unit pins** (schema sections,
   determinism, side-effect-freeness, polity/hosting payload).

## Exit evidence (10K ticks, seed 42, N=12, 2 polities)

| Charter contract | Measured |
|---|---|
| Determinism (rule 2) | two exports byte-identical |
| Schema v1 round-trip | parses; schema_version 1, ticks 10000, agents 12, 12 sites |
| Polity payload | 2 polities, p0 stage_lines: 29 canon entries |
| Culture payload | 13 memes, 9 with non-empty `hosts` (the i300 ledger) |
| Annals | rendered verbatim (1555 chars) |
| Zero blast | export is read-only over public state; standing golden/snapshot suites are export-blind (full gate green, zero re-anchors) |

**Verdict: asset pipeline v0 OPERATIONAL — charter exit evidence complete.**

## What this opens (per the charter §4)

- CLIENT: asset-viewer panels, cultural diffusion maps (from `hosts`),
  territory views (from polity membership + stage lines).
- AA graphical client (UM-4): sites → meshes, culture → narrative props,
  annals → story text — all against the versioned schema.
- Art/audio sub-charters may now be drafted under CLIENT.

## Verification

- sim lib **262/262** (4 new pins); fmt clean; clippy 0; bench law
  0 violations; full gate green; zero re-anchors.
