//! Virtual window — CLIENT 4.9-4.12 panel virtualization (2026-08-31).
//!
//! Deterministic slicing for village panel virtualization. Pure, no sim
//! coupling, `golden 5/5` untouched — TUI only.

use mindstrata_sim::sim::MetricsSnapshot;

/// Slice `history` to `offset..offset+limit` deterministically.
///
/// Clamps `offset` and `limit` to history bounds. Empty history → empty.
/// `limit == 0` → empty. Pure, no allocation beyond slice.
///
/// ```
/// use mindstrata_sim::sim::MetricsSnapshot;
/// use mindstrata_tui::panel_virtual::virtual_window;
/// let hist: Vec<MetricsSnapshot> = (0..5).map(|i| { let mut m = MetricsSnapshot::default(); m.tick = i*100; m }).collect();
/// assert_eq!(virtual_window(&hist, 0, 2).len(), 2);
/// assert_eq!(virtual_window(&hist, 10, 2).len(), 0); // offset beyond end clamps to empty
/// assert_eq!(virtual_window(&hist, 1, 10).len(), 4); // limit clamps to remainder
/// assert_eq!(virtual_window(&[], 0, 10).len(), 0);
/// assert_eq!(virtual_window(&hist, 1, 2), virtual_window(&hist, 1, 2)); // deterministic
/// ```
pub fn virtual_window(
    history: &[MetricsSnapshot],
    offset: usize,
    limit: usize,
) -> &[MetricsSnapshot] {
    if history.is_empty() || limit == 0 || offset >= history.len() {
        return &[];
    }
    let end = (offset + limit).min(history.len());
    &history[offset..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hist(n: usize) -> Vec<MetricsSnapshot> {
        (0..n)
            .map(|i| {
                let mut m = MetricsSnapshot::default();
                m.tick = (i * 100) as u64;
                m
            })
            .collect()
    }

    #[test]
    fn empty_and_zero_limit_are_empty() {
        assert_eq!(virtual_window(&[], 0, 10).len(), 0);
        let h = hist(5);
        assert_eq!(virtual_window(&h, 0, 0).len(), 0);
        assert_eq!(virtual_window(&h, 10, 2).len(), 0);
    }

    #[test]
    fn window_clamps_to_remainder() {
        let h = hist(5);
        assert_eq!(virtual_window(&h, 1, 10).len(), 4);
        assert_eq!(virtual_window(&h, 0, 2).len(), 2);
        assert_eq!(virtual_window(&h, 4, 1).len(), 1);
    }

    #[test]
    fn deterministic_and_exact_slice() {
        let h = hist(5);
        let a = virtual_window(&h, 1, 2);
        let b = virtual_window(&h, 1, 2);
        assert_eq!(a.as_ptr(), b.as_ptr());
        assert_eq!(a[0].tick, 100);
        assert_eq!(a[1].tick, 200);
    }
}
