//! Longitudinal chart components (DC-1 CLIENT library).
//!
//! Deterministic, allocation-light ASCII rendering over plain data — no
//! sim-crate types in this module's signatures (see docs/chart-component-api.md).
//!
//! DC-1 tasks 2.10–2.12 add lineage, emotion, and village data plumbing:
//! each lane reads only `&[MetricsSnapshot]` (the session history fixture),
//! never `Simulation`, so the TUI stays decoupled from the tick loop.

use mindstrata_sim::sim::MetricsSnapshot;

/// Display band a series is scaled into.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Band {
    /// Natural [0, 1].
    UnitInterval,
    /// Pinned explicit band (e.g. trait variance on [0, 0.25]).
    Fixed(f64, f64),
    /// Scale to the observed maximum of the samples.
    ObservedMax,
}

impl Band {
    fn resolve(&self, samples: &[f64]) -> (f64, f64) {
        match *self {
            Band::UnitInterval => (0.0, 1.0),
            Band::Fixed(lo, hi) => (lo, hi),
            Band::ObservedMax => {
                let hi = samples.iter().copied().fold(0.0_f64, f64::max).max(1.0);
                (0.0, hi)
            }
        }
    }
}

/// A named series ready for rendering.
#[derive(Debug, Clone)]
pub struct Series {
    /// Display label (lane title / chart name).
    pub name: &'static str,
    /// Unit suffix appended to the last-value readout.
    pub unit: &'static str,
    /// Scaling band applied before rasterizing.
    pub band: Band,
    /// Chronological samples, oldest first.
    pub samples: Vec<f64>,
}

const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// One-row block-character sparkline over `width` most-recent samples
/// (tail window; dense series are downsampled by striding).
pub fn sparkline(series: &Series, width: usize) -> String {
    let tail = tail_samples(&series.samples, width);
    let (lo, hi) = series.band.resolve(tail);
    let span = (hi - lo).max(1e-9);
    tail.iter()
        .map(|v| {
            let t = ((v - lo) / span).clamp(0.0, 1.0);
            BARS[((t * 7.0).round() as usize).min(7)]
        })
        .collect()
}

fn tail_samples(samples: &[f64], width: usize) -> &[f64] {
    if width == 0 || samples.len() <= width {
        samples
    } else {
        &samples[samples.len() - width..]
    }
}

fn trend_arrow(first: f64, last: f64) -> char {
    if last - first > 1e-6 {
        '↑'
    } else if first - last > 1e-6 {
        '↓'
    } else {
        '→'
    }
}

/// Titled lane: label + sparkline + trend arrow + last/first readout.
pub fn lane(series: &Series, width: usize) -> String {
    if series.samples.is_empty() {
        return format!("{:<14} (no history)", series.name);
    }
    let first = series.samples[0];
    let last = series.samples[series.samples.len() - 1];
    format!(
        "{:<14} {} {} {:.3}{} (was {:.3})",
        series.name,
        sparkline(series, width),
        trend_arrow(first, last),
        last,
        series.unit,
        first
    )
}

/// Lineage lane — family-count proliferation over generational time
/// (DC-1 task 2.10). Reads only the metric history fixture; no `Simulation`
/// coupling. Band is `ObservedMax` so the spark scales to the village's
/// observed lineage diversity.
pub fn lineage_lane(history: &[MetricsSnapshot], width: usize) -> String {
    let samples: Vec<f64> = history.iter().map(|m| m.family_count as f64).collect();
    let s = Series {
        name: "families",
        unit: "",
        band: Band::ObservedMax,
        samples,
    };
    lane(&s, width)
}

/// Emotion lane — joy/fear valence proxy (DC-1 task 2.11). Single-series
/// `avg_joy` lane using the shared `Band::UnitInterval` interface; gaps
/// (empty history) render as the standard `(no history)` sentinel and the
/// same `lane` contract as the lineage lane.
pub fn emotion_lane(history: &[MetricsSnapshot], width: usize) -> String {
    let samples: Vec<f64> = history.iter().map(|m| m.avg_joy).collect();
    let s = Series {
        name: "joy",
        unit: "",
        band: Band::UnitInterval,
        samples,
    };
    lane(&s, width)
}

/// Village panel — aggregated metrics view (DC-1 task 2.12). Composes the
/// two lanes plus headline aggregates (agent_count, grain) as a deterministic
/// multi-line widget. Data source is the same `&[MetricsSnapshot]` fixture;
/// refresh path is caller-owned: pass the latest `history` slice each frame.
pub fn village_panel(history: &[MetricsSnapshot], width: usize) -> String {
    let lineage = lineage_lane(history, width);
    let emotion = emotion_lane(history, width);
    let (agents, grain) = history
        .last()
        .map_or((0, 0.0), |m| (m.agent_count, m.total_grain));
    format!("{lineage}\n{emotion}\nagents {agents}  grain {grain:.1}")
}

/// Multi-row line chart: `height` rows × `width` cols raster of the series
/// shape (tail window, column-strided when dense). Deterministic pure
/// f64 → chars; one marker per column (pure line semantics).
pub fn line_chart(series: &Series, width: usize, height: usize) -> String {
    let tail = tail_samples(&series.samples, width);
    if tail.is_empty() || height == 0 || width == 0 {
        return String::new();
    }
    let (lo, hi) = series.band.resolve(tail);
    let span = (hi - lo).max(1e-9);

    let n = tail.len();
    let stride = if n >= width {
        n as f64 / width as f64
    } else {
        1.0
    };
    let mut grid = vec![vec![' '; width]; height];
    let cols = width.min(n);
    for (col, col_row) in grid.iter_mut().enumerate().take(cols) {
        let idx = ((col as f64) * stride) as usize;
        let idx = idx.min(n - 1);
        let t = ((tail[idx] - lo) / span).clamp(0.0, 1.0);
        let row = ((t * (height as f64 - 1.0)).round()) as usize;
        let row = row.min(height - 1);
        col_row[height - 1 - row] = '•';
    }
    grid.into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(name: &'static str, band: Band, samples: Vec<f64>) -> Series {
        Series {
            name,
            unit: "",
            band,
            samples,
        }
    }

    #[test]
    fn sparkline_empty_is_empty() {
        let s = series("x", Band::UnitInterval, vec![]);
        assert_eq!(sparkline(&s, 10), "");
    }

    #[test]
    fn sparkline_single_point_renders_one_bar() {
        let s = series("x", Band::UnitInterval, vec![0.5]);
        assert_eq!(sparkline(&s, 10).chars().count(), 1);
    }

    #[test]
    fn sparkline_dense_series_downsamples_to_width() {
        let s = series(
            "x",
            Band::UnitInterval,
            (0..1000).map(|i| i as f64 / 999.0).collect(),
        );
        assert_eq!(sparkline(&s, 20).chars().count(), 20);
    }

    #[test]
    fn sparkline_monotonic_rise_hits_top_bar() {
        let s = series("x", Band::UnitInterval, vec![0.0, 0.5, 1.0]);
        let out = sparkline(&s, 3);
        assert_eq!(out.chars().last(), Some('█'));
    }

    #[test]
    fn observed_max_band_scales_to_data() {
        let s = series("f", Band::ObservedMax, vec![0.0, 5.0, 10.0]);
        // Max maps to top bar regardless of absolute magnitude.
        let out = sparkline(&s, 3);
        assert_eq!(out.chars().last(), Some('█'));
    }

    #[test]
    fn fixed_band_clamps_out_of_range_values() {
        let s = series("tv", Band::Fixed(0.0, 0.25), vec![0.9]);
        let out = sparkline(&s, 1);
        assert_eq!(out, "█"); // clamped to ceiling, not panicking
    }

    #[test]
    fn lane_empty_shows_no_history() {
        let s = series("stress", Band::UnitInterval, vec![]);
        assert!(lane(&s, 10).contains("(no history)"));
    }

    #[test]
    fn lane_rising_series_uses_up_arrow() {
        let s = series("stress", Band::UnitInterval, vec![0.1, 0.9]);
        let out = lane(&s, 10);
        assert!(out.contains('↑'), "lane output: {out}");
    }

    #[test]
    fn line_chart_empty_is_empty_and_single_point_places_marker() {
        let e = series("x", Band::UnitInterval, vec![]);
        assert_eq!(line_chart(&e, 10, 4), "");
        let one = series("x", Band::UnitInterval, vec![1.0]);
        let grid = line_chart(&one, 10, 4);
        assert_eq!(grid.lines().count(), 4);
        assert!(grid.contains('•'));
    }

    #[test]
    fn line_chart_dense_series_bounded_by_width() {
        let s = series(
            "x",
            Band::UnitInterval,
            (0..500).map(|i| i as f64 / 499.0).collect(),
        );
        let grid = line_chart(&s, 30, 6);
        assert_eq!(grid.lines().next().unwrap().chars().count(), 30);
    }

    #[test]
    fn rendering_is_deterministic_across_calls() {
        let s = series("k", Band::UnitInterval, vec![0.2, 0.8, 0.4, 0.6]);
        assert_eq!(lane(&s, 8), lane(&s, 8));
        assert_eq!(line_chart(&s, 12, 5), line_chart(&s, 12, 5));
    }

    fn fixture_history() -> Vec<MetricsSnapshot> {
        (0..5)
            .map(|i| {
                let mut m = MetricsSnapshot::default();
                m.family_count = i + 1;
                m.avg_joy = 0.2 + i as f64 * 0.1;
                m.agent_count = 12 + i;
                m.total_grain = 10.0 * i as f64;
                m.tick = i * 100;
                m
            })
            .collect()
    }

    #[test]
    fn lineage_lane_renders_from_fixture() {
        // 2.10 acceptance: lane renders from session metrics fixture; no direct sim-state reads.
        let hist = fixture_history();
        let out = lineage_lane(&hist, 10);
        assert!(out.contains("families"), "{out}");
        assert!(out.contains('↑'), "rising families should show ↑: {out}");
        // Shared interface unchanged — deterministic across calls.
        assert_eq!(out, lineage_lane(&hist, 10));
        // Empty fixture renders the sentinel, not a crash.
        assert!(lineage_lane(&[], 10).contains("(no history)"));
    }

    #[test]
    fn emotion_lane_renders_correctly_on_fixtures_incl_gaps() {
        // 2.11 acceptance: emotion series render correctly on fixtures incl. gaps; shared interface unchanged.
        let hist = fixture_history();
        let out = emotion_lane(&hist, 10);
        assert!(out.contains("joy"), "{out}");
        assert!(out.contains('↑'), "rising joy should show ↑: {out}");
        // Gaps: empty history still renders sentinel via the same lane contract.
        assert!(emotion_lane(&[], 10).contains("(no history)"));
        // Single-point fixture — no panic, deterministic.
        let mut one = MetricsSnapshot::default();
        one.avg_joy = 0.5;
        let out_one = emotion_lane(&[one.clone()], 10);
        assert!(out_one.contains("joy"));
        assert_eq!(out_one, emotion_lane(&[one], 10));
    }

    #[test]
    fn village_panel_renders_aggregate_fixture_data() {
        // 2.12 acceptance: panel renders aggregate fixture data; refresh path documented.
        let hist = fixture_history();
        let panel = village_panel(&hist, 10);
        assert!(panel.contains("families"), "{panel}");
        assert!(panel.contains("joy"), "{panel}");
        assert!(
            panel.contains("agents 16"),
            "last fixture has 16 agents: {panel}"
        );
        assert!(
            panel.contains("grain 40.0"),
            "last fixture grain 40: {panel}"
        );
        // Empty panel still renders lanes’ sentinels + zero aggregates.
        let empty = village_panel(&[], 10);
        assert!(empty.contains("(no history)"));
        assert!(empty.contains("agents 0"));
        // Deterministic across calls — refresh path is pure.
        assert_eq!(panel, village_panel(&hist, 10));
    }
}
