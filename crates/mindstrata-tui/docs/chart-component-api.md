# TUI Chart Component Library — API Design Note

Task: CLIENT chart scaffolding audit → component API for longitudinal charts.
Audited working tree 2026-08-25; zero code changes in this task.

## Existing scaffolding (crates/mindstrata-tui/src/render.rs)

| Item | Location | What it does |
|---|---|---|
| `sparkline(&[f64], lo, hi) -> String` | render.rs:356–370 | one-row block-character sparkline (`▁▂▃▄▅▆▇█`), clamped to [0,1] normalized band |
| `series_chart(name, values, lo, hi, unit) -> String` | render.rs:372–390 | label + sparkline + trend arrow (↑↓→) + last/first values |
| `render_metric_charts(&[mindstrata_sim::sim::MetricsSnapshot]) -> String` | render.rs:392–438 | trends view: fixed 60-sample tail window; bounded series pinned to natural ranges (stress/health/fear/joy/gini/skill on [0,1]; trait variance on [0,0.25]; kinship on [0,0.5]); families scaled to observed max |
| `DashboardConfig` | render.rs:333–347 | season/year/grain/water/institution/faction header inputs |
| Session plumbing | session.rs (`View`, `UiState`, key_to_command), main.rs (`run_loop`, `draw`, `render_view`) | view cycling + dispatch |

Assessment: primitives are deterministic, allocation-light, and dependency-free — good
seeds for a component library. The one structural wart: `render_metric_charts` reaches
directly into `mindstrata_sim::sim::MetricsSnapshot` with per-field closures, coupling
the view layout to sim metric names. The library must invert that.

## Component API (proposed)

All components take owned/borrowed plain data — no sim-crate types in signatures
(per FR-041 / IC-2 direction). New module: `crates/mindstrata-tui/src/charts/`.

```rust
// charts/series.rs
pub struct Series {
    pub name: &'static str,
    pub unit: &'static str,
    pub band: Band,            // Fixed(f64,f64) | ObservedMax | UnitInterval
    pub samples: Vec<f64>,     // caller-downsampled or raw
}
pub enum Band { UnitInterval, Fixed(f64, f64), ObservedMax }

// charts/render.rs
pub fn sparkline(s: &Series, width: usize) -> String;          // generalizes render.rs:356
pub fn line_chart(s: &Series, width: usize, height: usize) -> String;  // multi-row ASCII (braille or block raster)
pub fn lane(s: &Series, opts: LaneOpts) -> String;             // titled lane w/ legend row

// charts/source.rs
pub trait ChartDataSource {
    fn series(&self, key: SeriesKey) -> Option<Series>;        // key = semantic slot, not a sim field name
    fn window(&self) -> (u64, u64);                            // tick range covered
}
pub enum SeriesKey { Stress, Health, FearP90, JoyP90, Gini, BestSkill, Families,
                     TraitVariance, MeanKinship, /* extensible */ }
```

Migration map (existing → target):

- `sparkline` → `charts::render::sparkline` wrapper kept for `render_metric_charts`
  during transition, then retired.
- `series_chart` → `charts::render::lane` with `LaneOpts { show_trend_arrow: bool }`.
- `render_metric_charts` closures → a `ChartDataSource` impl backed by
  `&[MetricsSnapshot]` living NEXT TO the snapshot import (adapter layer), so views
  consume only `SeriesKey`s. When IC-2 annals-trace records arrive (STORY), a second
  adapter implements the same trait from trace records — views unchanged.

## Constraints carried forward

- Deterministic rendering only (pure f64 → chars; iteration in index order).
- No allocation in hot redraw paths beyond the output String (matches existing style);
  downsample BEFORE constructing a `Series` when history exceeds display width.
- Terminal grid discipline: width comes from the caller (`UiState` knows terminal size);
  components never query the terminal themselves.
- No sim-crate dependency inside `charts/` — enforced by keeping adapters in
  render.rs/session.rs side (FR-041 invariant).

## Follow-on work slots

Lane containers + stage-position rendering (line-stage lanes) land after IC-2 schema
exists; color/legend consolidation is tracked under the palette task; virtualization
(memory bound at 100K ticks) belongs to the chart-virtualization task and will use
`Band::ObservedMax` over downsampled windows rather than full-history scans.
