//! Asset-viewer panel — CLIENT consumer of the i301 asset document
//! (`ASSET-PIPELINE-v0` charter §4: "asset-viewer panels, cultural diffusion
//! maps, territory views").
//!
//! Pure render over the versioned [`WorldAssets`] document — no sim coupling,
//! no state, no allocation growth beyond the output string. The charter's rule
//! 5 ("no per-frame export in v0") is honored by the binary: it captures the
//! document ONCE (the `a` key) and this panel renders the cached value.

use std::fmt::Write as _;

use mindstrata_sim::sim::assets::{CultureAsset, PolityAsset, SiteAsset, WorldAssets};

/// Rows shown per list section before truncation (bounded render — the
/// document itself is bounded by the charter, but a 200-meme village would
/// still cost a frame's worth of lines).
pub const ASSET_VIEW_ROWS: usize = 40;

/// Render the asset viewer. Deterministic: document order in, document order
/// out; no maps, no iteration over unordered containers.
pub fn render_asset_viewer(doc: &WorldAssets) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "Asset Viewer — schema v{}  (i301 export; read-only over public state)",
        doc.schema_version
    );
    let _ = writeln!(
        out,
        "  seed {}  ·  tick {}  ·  agents {}  ·  polities {}",
        doc.meta.seed, doc.meta.ticks, doc.meta.agents, doc.meta.polities
    );

    render_world(&mut out, &doc.world.sites);
    render_polities(&mut out, &doc.polities);
    render_culture(&mut out, &doc.culture, doc.meta.agents);
    render_annals(&mut out, &doc.annals);
    out
}

fn render_world(out: &mut String, sites: &[SiteAsset]) {
    let _ = writeln!(out, "\nWORLD  {} sites", sites.len());
    for s in sites.iter().take(ASSET_VIEW_ROWS) {
        let _ = writeln!(
            out,
            "  #{:<3} {:<12} \"{}\"  @({}, {})  cap {}",
            s.id, s.kind, s.name, s.position.0, s.position.1, s.capacity
        );
    }
    truncated_note(out, sites.len());
}

fn render_polities(out: &mut String, polities: &[PolityAsset]) {
    let _ = writeln!(out, "\nPOLITIES  {}", polities.len());
    if polities.is_empty() {
        let _ = writeln!(out, "  (no polities assigned yet)");
        return;
    }
    for (i, p) in polities.iter().enumerate() {
        let _ = writeln!(
            out,
            "  p{i}  {} members · {} stage lines",
            p.members.len(),
            p.stage_lines.len()
        );
    }
}

/// Culture section — the diffusion map. `hosts` is the i300 per-agent hosting
/// ledger, so a host share over the population is the diffusion signal the
/// charter §4 asks CLIENT to render (a bar, no sim reads).
fn render_culture(out: &mut String, culture: &[CultureAsset], agents: usize) {
    let _ = writeln!(
        out,
        "\nCULTURE  {} memes  (hosts / {agents} agents)",
        culture.len()
    );
    if culture.is_empty() {
        let _ = writeln!(out, "  (no memes yet)");
        return;
    }
    for m in culture.iter().take(ASSET_VIEW_ROWS) {
        let _ = writeln!(
            out,
            "  [{:<3}] {:<10} t={:<6} {:>3} hosts  {}  \"{}\"",
            m.id,
            m.content_type,
            m.created_tick,
            m.hosts.len(),
            diffusion_bar(m.hosts.len(), agents),
            m.description
        );
    }
    truncated_note(out, culture.len());
}

/// A 12-cell host-share bar. Zero agents → empty bar (zero-at-zero).
fn diffusion_bar(hosts: usize, agents: usize) -> String {
    const CELLS: usize = 12;
    if agents == 0 {
        return "░".repeat(CELLS);
    }
    let filled = (hosts as f64 / agents as f64 * CELLS as f64).round() as usize;
    let filled = filled.min(CELLS);
    format!("{}{}", "▓".repeat(filled), "░".repeat(CELLS - filled))
}

fn render_annals(out: &mut String, annals: &str) {
    let lines = annals.lines().count();
    let _ = writeln!(
        out,
        "\nANNALS  ({lines} lines, showing first {ASSET_VIEW_ROWS})"
    );
    for line in annals.lines().take(ASSET_VIEW_ROWS) {
        let _ = writeln!(out, "  {line}");
    }
    if lines > ASSET_VIEW_ROWS {
        let _ = writeln!(out, "  … {} more lines", lines - ASSET_VIEW_ROWS);
    }
}

fn truncated_note(out: &mut String, total: usize) {
    if total > ASSET_VIEW_ROWS {
        let _ = writeln!(out, "  … {} more", total - ASSET_VIEW_ROWS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mindstrata_sim::sim::assets::{MetaAsset, WorldSection};

    fn doc() -> WorldAssets {
        WorldAssets {
            schema_version: 1,
            meta: MetaAsset {
                seed: 42,
                ticks: 10_000,
                agents: 12,
                polities: 1,
            },
            world: WorldSection {
                sites: vec![SiteAsset {
                    id: 0,
                    kind: "Farm".into(),
                    name: "North Field".into(),
                    position: (3, 4),
                    capacity: 200,
                }],
            },
            polities: vec![PolityAsset {
                members: (0..12).collect(),
                stage_lines: vec![],
            }],
            culture: vec![CultureAsset {
                id: 7,
                description: "Harvest song".into(),
                content_type: "Ritual".into(),
                created_tick: 1_200,
                hosts: (0..6).collect(),
            }],
            annals: "Annals of Riverford\nYear 1: founding\n".into(),
        }
    }

    #[test]
    fn viewer_renders_every_v0_section() {
        let out = render_asset_viewer(&doc());
        assert!(out.contains("schema v1"), "{out}");
        assert!(out.contains("seed 42"), "{out}");
        assert!(out.contains("WORLD  1 sites"), "{out}");
        assert!(out.contains("North Field"), "{out}");
        assert!(out.contains("POLITIES  1"), "{out}");
        assert!(out.contains("12 members"), "{out}");
        assert!(out.contains("CULTURE  1 memes"), "{out}");
        assert!(out.contains("Harvest song"), "{out}");
        assert!(out.contains("ANNALS"), "{out}");
        assert!(out.contains("Annals of Riverford"), "{out}");
    }

    #[test]
    fn diffusion_bar_reflects_host_share() {
        assert_eq!(diffusion_bar(12, 12), "▓▓▓▓▓▓▓▓▓▓▓▓");
        assert_eq!(diffusion_bar(0, 12), "░░░░░░░░░░░░");
        assert_eq!(diffusion_bar(6, 12), "▓▓▓▓▓▓░░░░░░");
        assert_eq!(diffusion_bar(0, 0), "░░░░░░░░░░░░", "zero-at-zero");
        // Never overflows the bar even if the ledger is inconsistent.
        assert_eq!(diffusion_bar(99, 12), "▓▓▓▓▓▓▓▓▓▓▓▓");
    }

    #[test]
    fn render_is_deterministic() {
        let d = doc();
        assert_eq!(render_asset_viewer(&d), render_asset_viewer(&d));
    }

    #[test]
    fn empty_document_renders_zero_at_zero() {
        let mut d = doc();
        d.world.sites.clear();
        d.polities.clear();
        d.culture.clear();
        d.annals.clear();
        let out = render_asset_viewer(&d);
        assert!(out.contains("WORLD  0 sites"), "{out}");
        assert!(out.contains("(no polities assigned yet)"), "{out}");
        assert!(out.contains("(no memes yet)"), "{out}");
    }
}
