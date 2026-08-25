//! Longitudinal chart components (DC-1 CLIENT library).
//!
//! Deterministic, allocation-light ASCII rendering over plain data — no
//! sim-crate types in this module's signatures (see docs/chart-component-api.md).

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
}
