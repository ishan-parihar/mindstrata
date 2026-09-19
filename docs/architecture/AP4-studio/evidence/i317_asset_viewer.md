# Iteration 317 — DC-4a: CLIENT asset-viewer panel

**Status:** LANDED (CLIENT) · **Root cause owned:** `docs/PLAN_DC3_DEVELOPMENT.md`
DC-4 entry item **(a)** — "CLIENT asset-viewer panel consuming `--export-assets`
output" — the first DC-4 deliverable, opened by the i301 `ASSET-PIPELINE-v0`
charter §4 ("asset-viewer panels, cultural diffusion maps, territory views").

## What landed (TUI only — zero sim-behaviour blast)

1. **`crates/mindstrata-tui/src/assets_view.rs`** — `render_asset_viewer(&WorldAssets) -> String`,
   a pure render over the versioned i301 document. Sections: meta header, world
   sites (id/kind/name/position/capacity), polities (members + stage-line count),
   **culture diffusion map** (per-meme host share as a 12-cell bar, driven by the
   i300 hosting ledger), and the annals surface. Bounded render: 40 rows per list
   (`ASSET_VIEW_ROWS`) with an explicit truncation note.
2. **`View::Assets`** joins the tab cycle (Dossier → Assets → Dashboard) with the
   `a` key capturing the document and opening the panel. `UiState.assets` caches
   the capture.
3. **`WorldAssets` and its section structs** gain `Debug, Clone` (additive;
   serialization shape untouched).

### Charter compliance

- **Rule 1 (read-only)**: the panel consumes the published document; it never
  reads sim state. The `a` key calls the existing pure `export_world_assets`.
- **Rule 5 (no per-frame export)**: honored by construction — the export runs
  once on capture, and the panel renders `UiState.assets`. A fresh session holds
  `None` (pinned), so a frame with no capture does not touch the sim.
- **Determinism**: document-order in, document-order out; no map iteration.

## Probe verdict — `ASSET_VIEWER_PANEL_LIVE`

`i317_asset_viewer`: run a real 10K-tick seed-42 simulation, export, render.

```
export: schema v1 · 12 sites · 0 polities · 7 memes · annals 1638 chars
  schema_version / meta header / world sites / polities / culture /
  annals / site name / deterministic render   ...   all PASS
```

Rendered panel (excerpt):

```
Asset Viewer — schema v1  (i301 export; read-only over public state)
  seed 42  ·  tick 10000  ·  agents 12  ·  polities 0

WORLD  12 sites
  #0   House        "House 1"  @(11, 8)  cap 4
  #8   Farm         "Village Farm"  @(2, 8)  cap 10
  #9   Well         "Village Well"  @(8, 13)  cap 20

POLITIES  0
  (no polities assigned yet)

CULTURE  7 memes  (hosts / 12 agents)
  [0  ] Theological t=0       10 hosts  ▓▓▓▓▓▓▓▓▓▓░░  "The river feeds the village; honor it"
  [1  ] Moral      t=0        3 hosts  ▓▓▓░░░░░░░░░  "Hard work before the harvest brings plenty"
```

The diffusion bar makes the i300 hosting ledger legible without a sim read —
the charter's named CLIENT deliverable.

## Pins

- `mindstrata-tui::assets_view` — 4 tests (every section renders, diffusion bar
  incl. zero-at-zero and overflow clamp, deterministic render, empty-document
  zero-at-zero).
- `mindstrata-tui::tests::asset_view_state_starts_empty_and_labels` — the cache
  starts empty (rule 5) and the tab labels.
- `ui_state_cycles_through_all_views` extended for the new tab.

## Verification

`cargo fmt --all` clean · clippy **0** · tui **53/53 + 3 doc** · sim/tests
unchanged (**309/0/1**) · `scripts/gate --full` **GATE GREEN**, golden
byte-identical (TUI-only change, sim untouched).

## What this opens

DC-4 **(c)** graphical shell spike can now consume the same document through a
proven render path; the cultural diffusion map and territory/polity sections are
directly reusable.
