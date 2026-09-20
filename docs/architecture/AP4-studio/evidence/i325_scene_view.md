# Iteration 325 — the scene graph becomes a TUI view (spike operationalized)

**Status:** LANDED · **Item owned:** the i322 follow-through — the DC-4(c)
spike was reachable only from a probe; this iteration gives it an operator-facing
surface so the "graphical shell" work is *usable*, not just provable.

## What landed

**1. The scene's terminal face** — `mindstrata_tui::scene::render_ascii(&Scene)`.
SVG needs a browser; a TUI needs text, so the spike gains a tile-grid renderer:
sites as their kind glyph (`f/w/t/m/h/c/b`), agents as their polity digit
(`☐` when unassigned), plus a legend with per-polity counts. It is a pure
function of the scene → of the document, so it is **deterministic**, and the
grid is derived from the scene extents — **bounded by the world**, never by
history (charter rule 5).

**2. `View::Scene`** — a new tab (Dashboard → … → Dossier → Assets → Scene →
wrap). Rendered from the **cached** document captured by the existing `a` key,
so the charter's "capture once, never per-frame" rule still holds: the view
builds a scene from the cached document each frame at O(sites + agents), with
**no export**.

**3. `Primitive` made self-describing** — `Site` carries its `glyph`, `Agent`
carries its `polity` index. The scene previously encoded kind/polity only in a
colour string, so a text renderer would have had to re-derive them from the
document; carrying them keeps the primitive list the single source a renderer
consumes (and is what makes `render_ascii` a pure function of the scene).

## Probe evidence (`i322_scene_graph`, release, 10K run, seed 42)

Two new checks join the i322 contract:

```text
  ascii deterministic    PASS
  ascii site coverage    PASS
verdict=SCENE_GRAPH_SPIKE_LIVE

--- ascii scene (the TUI face) ---
Scene preview — 14×14 tiles (i301 document)
          t
        1 0 1
    f 0   1  0 m
            1
       1
          0
          w
  legend: f/w/t/m/h/c/b = sites (12 placed)
  agents: [0]×6 [1]×6
```

The probe prints the rendered grid, so the deliverable is **observable output**
(§3), not a source diff.

## Verification

- **TUI 61/61** (+2 `render_ascii` tests: placement/legend, zero-at-zero) and
  the `ui_state_cycles_through_all_views` pin updated for the new tab.
- Clippy caught two `uninlined_format_args` pedantic warnings in the new code —
  **fixed, not silenced** (§6 linting discipline); clippy 0 warnings.
- TUI-only change: golden byte-identical, `scripts/gate --full` **GATE GREEN**
  (309/0/1), zero re-anchors.

**Probe:** `cargo run -p mindstrata-benches --release --example i322_scene_graph`
