# Iteration 322 — DC-4(c) graphical-shell spike: asset document → scene graph

**Status:** LANDED · **Queue item owned:** DC-4 remainder **(c)** — "graphical
shell spike over the same i301 document" (`PLAN_DC3_DEVELOPMENT.md`, i322+
row; `ASSET-PIPELINE-v0` charter §4: "the same document is the seed of the
scene graph — sites → meshes, culture → narrative props, annals → story
text").

## What landed

**1. An additive `agents` section in the v0 document** (`sim/assets.rs`).
The v0 shape (`charter §3`) carried sites, polities, culture and annals — but
**no agent placement**, so a scene graph could draw the world and nothing on
it (a territory/diffusion view needs positions; per the CLIENT charter, "schema
gaps = change-order, not hacks"). Added:

```text
agents: [ { id, position: (x, y), polity: |null } ]
```

- **Additive → no `schema_version` bump** (charter rule 3: "additive fields do
  not bump"; consumers ignore unknown fields by construction).
- **Deterministic**: agent index order; polity membership resolved by
  first-match scan over the polity registry (registry order, deterministic);
  unassigned agent → `null` (zero-at-zero).
- **Read-only**: built from the public `AgentBundle::position` and the polity
  registry — no state mutation, no RNG, no events, so the export stays
  golden-blind (charter rule 1).

**2. The CLIENT spike** — `crates/mindstrata-tui/src/scene.rs`:

- `build_scene(&WorldAssets) -> Scene`: a flat, ordered primitive list —
  sites (document order, kind → colour) → agents (index order, polity →
  colour) → captions (meta + culture diffusion legend). Bounded: O(sites +
  agents + culture), one build per capture, never per frame (charter rule 5).
- `to_svg(&Scene) -> String`: a **headless** renderer. Deliberately no
  windowing/GPU stack — the spike proves the geometry/ordering contract a real
  renderer will consume, and the SVG form makes the result observable in CI
  and in a browser.
- Forward-compatible: an unknown site `kind` renders neutral rather than
  panicking (a schema addition must not break a client). Labels drawn from
  sim-authored strings are XML-escaped.

## Probe evidence (`i322_scene_graph`, release, 10K run, seed 42)

```text
document: schema v1 · 12 sites · 12 agents (12 in a polity) · 11 memes
scene:    272×326 units · 27 primitives (sites 12 · agents 12)
svg:      3629 bytes → target/i322_scene_graph.svg

  site coverage          PASS      (count == document sites)
  agent coverage         PASS      (count == document agents)
  territory payload      PASS      (12/12 agents carry a polity)
  deterministic render   PASS      (same doc → byte-identical SVG)
  document-driven output PASS      (seed 99 → different SVG)
  svg written            PASS      (observable artifact)
  svg well-formed        PASS
verdict=SCENE_GRAPH_SPIKE_LIVE
```

The **document-driven** leg is the one that matters: it rules out a constant
template by showing a different seed's document produces a different scene.

## Verification

- **Golden byte-identical**: the export is not read by the sim, and the change
  is additive to a client-only document — `scripts/gate --full` **GATE GREEN**
  (309/0/1), zero re-anchors, zero snapshot drift.
- 6 new `scene` unit tests (coverage, polity/neutral colouring, determinism +
  escaping, zero-at-zero, unknown-kind fallback) and 1 new `assets` pin for the
  additive placement section; **TUI 59/59**.
- `cargo fmt` clean, clippy 0 warnings.

**Probe:** `cargo run -p mindstrata-benches --release --example i322_scene_graph`
