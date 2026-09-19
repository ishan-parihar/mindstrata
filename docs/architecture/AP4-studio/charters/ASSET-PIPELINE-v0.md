# ASSET PIPELINE v0 — Deterministic Export Charter

**Status:** RATIFIED (i301) · **Owner:** TOOLS (pipeline) + CLIENT (consumers)
· **Trigger:** ROADMAP DC-3 → UM-3 "asset pipeline v0 lands → art/audio
sub-charters open under CLIENT"; perf-charter §5 item 3.

## 1. What "asset" means here

The simulation IS the content generator. An **asset** is any deterministic,
pure function of live simulation state that a client (TUI today, graphical at
AA scale) can consume without reaching into sim internals (IC-8). v0 ships
the **world + culture export**: one JSON document, stable schema, produced
from a finished run (or snapshot restore) with zero behavioral surface.

## 2. Binding rules

1. **Read-only by construction**: export is a pure function over public
   accessors. No sim state mutation, no RNG draws, no event emission. Golden
   replay is byte-identical by construction (nothing reads the export).
2. **Determinism**: same snapshot → byte-identical export (schema ordering
   is registry order; no HashMap iteration in output paths).
3. **Schema stability**: the document is versioned (`schema_version` u32,
   bumped only on breaking shape changes; additive fields do not bump).
   CLIENT consumes only published versions (IC-2/IC-3).
4. **Bounded journals**: export renders from the rolling event buffer and
   persistent registries only — the annals/chronicle surfaces — never
   requesting unbounded history. If a future export needs >250K-tick
   journals, that is the VecDeque trigger (perf charter §4), decided THEN.
5. **Perf envelope**: export cost is O(world + registry sizes), one-shot at
   run end — outside the per-tick budgets (charter §1). No per-frame export
   in v0.

## 3. v0 document shape

```
{
  "schema_version": 1,
  "meta":     { seed, ticks, agents, polities },
  "world":    { sites: [ {id, kind, name, position, capacity} ] },
  "polities": [ { members: [agent ids], stage_lines: [StageLineEntry…] } ],
  "culture":  [ { id, description, content_type, created_tick, hosts } ],
  "annals":   { per-year rendered blocks (the chronicle surface, verbatim) }
}
```

- `polities[].stage_lines` reuses `export_stage_lines` (i290 canon export).
- `culture[].hosts` is the i300 per-agent hosting ledger (the diffusion
  signal) — CLIENT renders cultural diffusion maps from it without sim reads.
- `annals` embeds `render_chronicle` verbatim (i259 read-only surface).

## 4. Consumers (opened by this charter)

- **CLIENT**: asset-viewer panels, cultural diffusion maps, territory views.
- **AA graphical client (UM-4)**: the same document is the seed of the
  scene graph — sites → meshes, culture → narrative props, annals → story
  text. Art/audio sub-charters may now be drafted against THIS schema
  (versioned; additive-only until v1).

## 5. Exit evidence (i301)

- `export_world_assets(sim) -> String` (serde_json, pretty) + CLI flag
  `--export-assets <path>` on `sim`/`scenario`.
- Pins: schema-shape (version, sections present), determinism (same sim →
  byte-identical), zero-blast (export does not move golden/snapshot state —
  verified by the standing suites, which are export-blind).
- Probe `i301_asset_export`: export a 10K run, re-import-validate the JSON
  round-trip, and verify schema sections.
