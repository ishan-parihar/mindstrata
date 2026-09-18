# Iteration 290 — Era V WP-K: observability (PLAN_DC3 §4 i290)

**Status:** LANDED · **Doctrine:** probe-first, observability-only (no behavior change
→ zero re-anchors by construction; golden 5/5 must stay byte-identical — verified).

## Scope (from the Era V wave brief)

WP-K "observability" names three deliverables:

1. **`stage_lines` KosmOS-frontmatter export** — R7 ("altitude claims cite the scale")
   at the snapshot boundary.
2. **Per-quadrant pathology means on `MetricsSnapshot`** — R6 transition-trace inputs.
3. **TUI longitudinal pathology panel** — per-quadrant lanes + holon stage lane.

## Implementation

- `crates/mindstrata-sim/src/snapshot.rs`: `export_stage_lines(&CollectiveField) ->
  Vec<StageLineEntry>` — registry order, each entry resolves its canon slug →
  `coupling(slug, ceil(stage))` and emits `stage_slug`/`altitude`/`depth_status`
  alongside the live `press`/`fulfillment` (the scale citation R7 demands).
  Unit pin: `stage_lines_export_cites_canon_scale` (stage 1.0 resolves to the rung-1
  canon cell; unknown slugs degrade to `kind: "unknown"` without panicking).
- `crates/mindstrata-sim/src/sim/snapshot_metrics.rs`: `q1_dark_addiction`,
  `q2_dark_allergy`, `q3_golden_addiction`, `q4_golden_allergy`, `collective_stage_max`
  — population means of each agent's `development.pathology.{quadrant}.intensity`
  plus the deepest collective line. `serde(default)` on all five (wire-compatible);
  CSV header extended in the same commit (new columns appended last).
- `crates/mindstrata-tui/src/charts.rs`: `pathology_panel` — four f64 lanes + the
  holon-stage lane, fn-pointer getter table (no closure-array type blowup); panel
  tests pinned.

## In-vivo evidence (`i290_observability_harness`, golden seed 42, N=12)

| Horizon | Q1 dark-add | Q2 dark-all | Q3 gold-add | Q4 gold-all | stage_max | deepest line |
|---|---|---|---|---|---|---|
| 1K | 0.037 | 0.056 | 0.047 | 0.035 | 1.00 | worldview (culture LL) |
| 20K | 0.411 | **0.732** | 0.110 | 0.610 | **6.00** | technology (system LR) |

- All 29 canon lines moved (stage > 0) by 1K — the field is live from the start, and
  the export enumerates the full scale-cited map.
- The quadrant divergence at 20K (Q2 dominant 0.732, Q3 quiet 0.110) is exactly the
  per-pathology-class signature R6 traces require; flat panels would have hidden it.
- `collective_stage_max` 6.00 at 20K matches i287's natural-trajectory WP-J gate
  (governance/economic-systems at 6.0) — two independent surfaces agree.

## Verification

- `cargo fmt --all` clean; `cargo clippy --workspace --quiet` 0 warnings.
- sim lib: 241/241; tui: 3/3 (incl. new panel tests); full release suite **307/0/1**
  (226.7s); bench index 107 ok / 0 violations.
- Golden 5/5 byte-identical (metrics snapshots print selected fields, not the new
  struct members; the full-struct debug print is not a snapshot surface).

## WP-K close

All three deliverables landed with in-vivo non-degeneracy evidence. **WP-K CLOSED.**
