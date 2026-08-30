# IC-7 — Modding Surface Contract (PLATFORM 8-10, DC-1)

Status: **DRAFT v0.5.0** (2026-08-29, awaiting joint PLATFORM+DESIGN ratification).
Owner: PLATFORM. Implements DC-1 PLATFORM 8-10.

## Purpose

Documents the public modding surface (which registry entries a content
pack may write) and the read-only invariant (no field outside the
explicit write-list can be touched from a content pack). This is the
audit surface for the DC-1 modding close.

## Write-list (a content pack MAY write)

| Registry | Path / Field | Source |
|---|---|---|
| Norms | `self.norms.register(Norm)` | `crates/mindstrata-sim/src/mods.rs:262-264` |
| Knowledge | `self.knowledge_store.extend(Knowledge)` | `crates/mindstrata-sim/src/mods.rs:265` |
| Scenario | (CLI-side, construction-time) | `crates/mindstrata-cli/src/main.rs:624` |

The scenario is **not** a live write — it's loaded via `from_scenario`
before the simulation runs, never through `apply_content_pack`.

## Read-only invariant

A content pack MUST NOT write to:

* `self.agents` (vector of `AgentBundle`) — agent fields are
  sim-internal
* `self.relationships` — relationships are computed, not modded
* `self.events`, `self.journal` — observability surfaces
* `self.world` — terrain, sites, ecology
* `self.clock`, `self.rng` — determinism-critical
* `self.metric_history` — observability
* `self.collective_field` (STORY 11) — dev-crate owned, per-tick
  emergent
* `self.polarity_claims` (per-agent) — per-tick emergent
* `self.development` (per-agent `DevelopmentFieldState`) — per-tick
  emergent
* `self.echo_chamber`, `self.noospheric_field`, `self.diplomacy`,
  `self.theology`, `self.military` — per-tick emergent

A content pack is **never** called from the tick loop (per the
existing modding invariant at `crates/mindstrata-sim/src/mods.rs:24-25`).
Calibrated windows (golden replays, snapshots) are structurally
untouched because content packs are opt-in via `--mod-dir` or a
library call.

## Content pack shape

```text
my_mod/
  manifest.ron      (required: name, description, version)
  norms.ron         (optional: a RON `[Norm, ...]`)
  knowledge.ron     (optional: a RON `[Knowledge, ...]`)
  scenario.ron      (optional: a RON `Scenario` — loaded via `from_scenario`)
```

Validated by `ContentPack::load` at `crates/mindstrata-sim/src/mods.rs:1-209`.
Re-validated by `apply_content_pack` (`crates/mindstrata-sim/src/mods.rs:236-271`)
defense in depth.

## Validation rules

* Manifest `name` non-empty
* Norm ids MUST NOT collide with existing registered norms
* Knowledge ids MUST NOT collide with existing knowledge store
* RON parse errors return `ModError::ParseFailed`
* Missing manifest returns `ModError::MissingFile`

## What is NOT moddable (ponytail — deferred to DC-2+)

* **Founder generation** (genome draws, personality, predisposition
  bias) — requires core RNG stream contract change; H5 history
  cautions against this without coordinated re-anchor sweep
* **Calibration parameters** (`SimParameters::default()` is not
  moddable) — Design/QA owns canon via IC-5 change-orders
* **Tick pipeline structure** — pass order is canon, modding cannot
  inject passes
* **Catalogs** (vendored line slugs, catalyst kinds, scenarios in
  `crates/mindstrata-world/`) — these are upstream canon; mods
  reference them by slug, never replace them
* **Snapshot format** (v13 binary, `SNAPSHOT_VERSION`) — owned by
  PLATFORM, mods load/save through the existing public API

## Versioning

* **IC-7 v1.0.0** (planned for DC-2 joint ratification): add
  tick-pipeline-aware modding hooks (after ActionKind add tests
  land — the v1 surface is too narrow to add hooks without
  breaking the write-list)
* **IC-7 v0.5.0** (this draft): documents the v1 surface as
  closed. No code change required — the write-list is already
  enforced by the modding module's structural invariants.

## Tests

* `crates/mindstrata-sim/src/mods.rs:tests` — pack validation,
  id-collision rejection, scenario-without-content cases
* `crates/mindstrata-cli/src/main.rs:tests` (if present) — CLI
  `--mod-dir` happy path
* The `i268`/`i270`/`i271` probes exclude modded content by
  construction (no `--mod-dir` flag) so calibrated windows stay
  byte-identical.
