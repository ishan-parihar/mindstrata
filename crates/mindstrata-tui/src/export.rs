//! Metrics export — CLIENT 19-22 polish (2026-08-31).
//!
//! Deterministic JSONL export of `MetricsSnapshot` slices. Pure, no sim
//! coupling, `cargo test` proves it. `golden 5/5` untouched — TUI only.

use mindstrata_sim::sim::MetricsSnapshot;

/// Export snapshots to JSONL lines deterministically.
///
/// Each line is `{"tick": N, "agents": M, "grain": G}` with fixed field
/// order. Empty input → empty output (no trailing newline).
///
/// ```
/// use mindstrata_sim::sim::MetricsSnapshot;
/// use mindstrata_tui::export::export_jsonl;
/// let mut m = MetricsSnapshot::default();
/// m.tick = 0; m.agent_count = 12; m.total_grain = 100.0;
/// let snaps = vec![m];
/// let out = export_jsonl(&snaps);
/// assert!(out.contains("\"tick\":0"));
/// assert_eq!(export_jsonl(&[]), "");
/// assert_eq!(export_jsonl(&snaps), export_jsonl(&snaps)); // deterministic
/// ```
pub fn export_jsonl(snaps: &[MetricsSnapshot]) -> String {
    if snaps.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for s in snaps {
        // Fixed field order, one decimal for grain (deterministic).
        out.push_str(&format!(
            "{{\"tick\":{},\"agents\":{},\"grain\":{:.1}}}\n",
            s.tick, s.agent_count, s.total_grain
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(tick: u64, agents: u64, grain: f64) -> MetricsSnapshot {
        let mut m = MetricsSnapshot::default();
        m.tick = tick;
        m.agent_count = agents;
        m.total_grain = grain;
        m
    }

    #[test]
    fn empty_is_empty_and_deterministic() {
        assert_eq!(export_jsonl(&[]), "");
        let a = vec![snap(0, 12, 100.0)];
        assert_eq!(export_jsonl(&a), export_jsonl(&a));
    }

    #[test]
    fn single_line_field_order() {
        let out = export_jsonl(&[snap(42, 12, 100.0)]);
        assert_eq!(out, "{\"tick\":42,\"agents\":12,\"grain\":100.0}\n");
    }

    #[test]
    fn multi_line_newline_terminated() {
        let out = export_jsonl(&[snap(0, 12, 100.0), snap(100, 12, 90.5)]);
        let lines: Vec<&str> = out.trim_end().split('\n').collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"tick\":0"));
        assert!(lines[1].contains("\"tick\":100"));
        assert!(out.ends_with('\n'));
    }
}
