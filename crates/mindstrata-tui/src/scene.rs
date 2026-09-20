//! DC-4(c) graphical-shell spike — a deterministic scene graph over the i301
//! asset document.
//!
//! The `ASSET-PIPELINE-v0` charter §4 opens the AA graphical client: "the same
//! document is the seed of the scene graph — sites → meshes, culture →
//! narrative props, annals → story text". This module is that seed, built as a
//! **headless spike**: it converts a versioned [`WorldAssets`] document into a
//! flat, ordered draw list ([`Scene`]) and can serialize it to SVG. It has the
//! properties the charter demands of a client consumer:
//!
//! - **Read-only over published schemas** (IC-2/IC-8): the only input is the
//!   document, never sim state.
//! - **Deterministic** (charter rule 2): document order in, document order out;
//!   no maps, no unordered iteration — `to_svg` is a pure function of the
//!   document, so the same export renders byte-identical.
//! - **Bounded** (rule 5): one build per capture, not per frame; the draw list
//!   is O(sites + agents + culture), never history.
//!
//! The spike deliberately does **not** depend on a windowing/GPU stack — it
//! proves the geometry/ordering contract a real renderer will consume, and the
//! SVG form makes the result observable in CI and in a browser.

use std::fmt::Write as _;

use mindstrata_sim::sim::assets::{AgentAsset, SiteAsset, WorldAssets};

/// Tile size in scene units (SVG user units). One world tile → one cell.
pub const TILE: f64 = 16.0;
/// Margin around the drawn world, in scene units.
pub const MARGIN: f64 = 24.0;

/// A flat 2-D primitive. Ordering inside [`Scene::primitives`] is stable and
/// is the draw order (later primitives paint over earlier ones).
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    /// A world site: a filled square at its tile with its kind's colour and a
    /// label.
    Site {
        /// Left edge in scene units.
        x: f64,
        /// Top edge in scene units.
        y: f64,
        /// Square edge length in scene units.
        size: f64,
        /// Fill colour (kind-derived).
        color: &'static str,
        /// Single-character kind marker (the ASCII fallback glyph).
        glyph: char,
        /// Text label drawn inside the square.
        label: String,
    },
    /// An agent: a small disc, coloured by polity (`None` → neutral).
    Agent {
        /// Centre x in scene units.
        cx: f64,
        /// Centre y in scene units.
        cy: f64,
        /// Disc radius in scene units.
        r: f64,
        /// Fill colour (polity-derived).
        color: &'static str,
        /// Polity index (the ASCII marker); `None` → unassigned.
        polity: Option<usize>,
    },
    /// A caption line (meta / diffusion legend).
    Caption {
        /// Baseline x in scene units.
        x: f64,
        /// Baseline y in scene units.
        y: f64,
        /// Caption text.
        text: String,
    },
}

/// The built scene: extents + an ordered primitive list.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    /// Scene width in units.
    pub width: f64,
    /// Scene height in units (includes the caption band).
    pub height: f64,
    /// Ordered draw list (later paints over earlier).
    pub primitives: Vec<Primitive>,
}

/// Palette for polity territory. Index by `polity % len`; `None` → `NEUTRAL`.
const POLITY_COLORS: [&str; 6] = [
    "#3b82f6", "#ef4444", "#22c55e", "#eab308", "#a855f7", "#14b8a6",
];
/// Colour for an unassigned agent.
const NEUTRAL: &str = "#94a3b8";

/// Site colour by published `kind` string. Unknown kinds render neutral — the
/// mapping never panics on a schema addition (forward compatibility).
fn site_color(kind: &str) -> &'static str {
    match kind {
        "Farm" | "Field" => "#65a30d",
        "Well" | "Water" => "#0ea5e9",
        "Temple" | "Shrine" => "#e879f9",
        "Market" => "#f59e0b",
        "Home" | "House" => "#a16207",
        "Council" | "TownHall" => "#8b5cf6",
        "Barracks" | "Watchtower" => "#64748b",
        _ => "#475569",
    }
}

/// Colour for a polity index (registry order → stable colour).
fn polity_color(polity: Option<usize>) -> &'static str {
    match polity {
        Some(i) => POLITY_COLORS[i % POLITY_COLORS.len()],
        None => NEUTRAL,
    }
}

/// A single-character glyph for a site kind — the ASCII fallback the TUI map
/// uses, kept here so a future renderer can draw text tiles without the sim.
#[must_use]
pub fn site_glyph(kind: &str) -> char {
    match kind {
        "Farm" | "Field" => 'f',
        "Well" | "Water" => 'w',
        "Temple" | "Shrine" => 't',
        "Market" => 'm',
        "Home" | "House" => 'h',
        "Council" | "TownHall" => 'c',
        "Barracks" | "Watchtower" => 'b',
        _ => '?',
    }
}

/// World extents from the document: the max site/agent tile plus one, floored
/// at 1 so an empty document still yields a valid (zero-content) scene.
fn world_extents(doc: &WorldAssets) -> (i32, i32) {
    let mut wx = 1;
    let mut wy = 1;
    for s in &doc.world.sites {
        wx = wx.max(s.position.0 + 1);
        wy = wy.max(s.position.1 + 1);
    }
    for a in &doc.agents {
        wx = wx.max(a.position.0 + 1);
        wy = wy.max(a.position.1 + 1);
    }
    (wx, wy)
}

/// Build the scene from a document. Pure and deterministic: the primitive
/// order is sites (document order) → agents (index order) → captions.
#[must_use]
pub fn build_scene(doc: &WorldAssets) -> Scene {
    let (wx, wy) = world_extents(doc);
    let width = wx as f64 * TILE + MARGIN * 2.0;
    let height = wy as f64 * TILE + MARGIN * 2.0 + 3.0 * 18.0; // caption band

    let mut primitives = Vec::with_capacity(doc.world.sites.len() + doc.agents.len() + 3);

    // Sites first (they are the ground layer), in document order.
    for s in &doc.world.sites {
        primitives.push(site_primitive(s));
    }
    // Agents paint over sites, in index order.
    for a in &doc.agents {
        primitives.push(agent_primitive(a));
    }

    // Caption band: meta + the culture diffusion legend.
    let cx = MARGIN;
    let mut cy = height - 3.0 * 18.0 + 18.0;
    primitives.push(Primitive::Caption {
        x: cx,
        y: cy,
        text: format!(
            "seed {} · tick {} · {} agents · {} polities",
            doc.meta.seed, doc.meta.ticks, doc.meta.agents, doc.meta.polities
        ),
    });
    cy += 18.0;
    let first = doc.culture.first();
    primitives.push(Primitive::Caption {
        x: cx,
        y: cy,
        text: match first {
            Some(m) => format!(
                "culture: {} memes · top #{} {:>3}/{:<3} hosts",
                doc.culture.len(),
                m.id,
                m.hosts.len(),
                doc.meta.agents
            ),
            None => "culture: 0 memes".to_string(),
        },
    });
    cy += 18.0;
    primitives.push(Primitive::Caption {
        x: cx,
        y: cy,
        text: "spike: SVG scene graph over the i301 document (read-only)".to_string(),
    });

    Scene {
        width,
        height,
        primitives,
    }
}

fn site_primitive(s: &SiteAsset) -> Primitive {
    Primitive::Site {
        x: MARGIN + s.position.0 as f64 * TILE,
        y: MARGIN + s.position.1 as f64 * TILE,
        size: TILE,
        color: site_color(&s.kind),
        glyph: site_glyph(&s.kind),
        label: format!("{} {}", site_glyph(&s.kind), s.name),
    }
}

fn agent_primitive(a: &AgentAsset) -> Primitive {
    Primitive::Agent {
        cx: MARGIN + (a.position.0 as f64 + 0.5) * TILE,
        cy: MARGIN + (a.position.1 as f64 + 0.5) * TILE,
        r: TILE * 0.28,
        color: polity_color(a.polity),
        polity: a.polity,
    }
}

/// Tile cell for an unassigned agent (no polity).
const AGENT_NEUTRAL_CELL: char = '☐';

/// The ASCII face of the scene: a tile grid + a legend. This is the spike's
/// terminal-observable form (SVG needs a browser; a TUI needs text), and it is
/// a pure function of the scene → of the document, so it is deterministic.
///
/// Layer order mirrors the SVG: sites first, agents paint over them. The grid
/// is derived from the scene extents, so it is bounded by the world, never by
/// history.
#[must_use]
pub fn render_ascii(scene: &Scene) -> String {
    // Grid extents from the primitive coordinates (sites are top-left anchored
    // at MARGIN + tile·TILE; agents at the tile centre).
    let mut cols = 0usize;
    let mut rows = 0usize;
    for p in &scene.primitives {
        match p {
            Primitive::Site { x, y, .. } => {
                cols = cols.max((((x - MARGIN) / TILE).round() as usize) + 1);
                rows = rows.max((((y - MARGIN) / TILE).round() as usize) + 1);
            }
            Primitive::Agent { cx, cy, .. } => {
                cols = cols.max((((cx - MARGIN) / TILE - 0.5).round() as usize) + 1);
                rows = rows.max((((cy - MARGIN) / TILE - 0.5).round() as usize) + 1);
            }
            Primitive::Caption { .. } => {}
        }
    }

    let mut grid = vec![vec![' '; cols]; rows];
    let mut site_count = 0usize;
    let mut agent_by_polity: std::collections::BTreeMap<usize, usize> =
        std::collections::BTreeMap::new();
    let mut neutral_agents = 0usize;
    for p in &scene.primitives {
        match p {
            Primitive::Site { x, y, glyph, .. } => {
                let c = ((x - MARGIN) / TILE).round() as usize;
                let r = ((y - MARGIN) / TILE).round() as usize;
                if let Some(row) = grid.get_mut(r) {
                    if let Some(cell) = row.get_mut(c) {
                        *cell = *glyph;
                        site_count += 1;
                    }
                }
            }
            Primitive::Agent { cx, cy, polity, .. } => {
                let c = ((cx - MARGIN) / TILE - 0.5).round() as usize;
                let r = ((cy - MARGIN) / TILE - 0.5).round() as usize;
                let mark = match polity {
                    Some(i) => char::from_digit((i % 10) as u32, 10).unwrap_or('?'),
                    None => AGENT_NEUTRAL_CELL,
                };
                if let Some(row) = grid.get_mut(r) {
                    if let Some(cell) = row.get_mut(c) {
                        *cell = mark;
                    }
                }
                match polity {
                    Some(i) => *agent_by_polity.entry(*i % 10).or_insert(0) += 1,
                    None => neutral_agents += 1,
                }
            }
            Primitive::Caption { .. } => {}
        }
    }

    let mut out = String::new();
    let _ = writeln!(out, "Scene preview — {cols}×{rows} tiles (i301 document)");
    for row in &grid {
        let line: String = row.iter().collect();
        let _ = writeln!(out, "  {line}");
    }
    let _ = writeln!(
        out,
        "\n  legend: f/w/t/m/h/c/b = sites ({site_count} placed)"
    );
    let mut legend = String::new();
    for (polity, count) in &agent_by_polity {
        let _ = write!(legend, " [{polity}]×{count}");
    }
    if neutral_agents > 0 {
        let _ = write!(legend, " [{AGENT_NEUTRAL_CELL}]×{neutral_agents}");
    }
    let _ = writeln!(out, "  agents:{legend}");
    out
}

/// Serialize the scene to SVG. Pure function of the scene → of the document,
/// so the same export renders byte-identical (charter rule 2).
#[must_use]
pub fn to_svg(scene: &Scene) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{:.0}\" height=\"{:.0}\" \
         viewBox=\"0 0 {:.0} {:.0}\">",
        scene.width, scene.height, scene.width, scene.height
    );
    let _ = writeln!(
        out,
        "<rect width=\"100%\" height=\"100%\" fill=\"#0f172a\"/>"
    );
    for p in &scene.primitives {
        match p {
            Primitive::Site {
                x,
                y,
                size,
                color,
                label,
                ..
            } => {
                let _ = writeln!(
                    out,
                    "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{size:.1}\" height=\"{size:.1}\" \
                     fill=\"{color}\"/>"
                );
                let _ = writeln!(
                    out,
                    "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"#e2e8f0\" font-size=\"9\" \
                     font-family=\"monospace\">{}</text>",
                    x + 1.0,
                    y + size - 2.0,
                    escape(label)
                );
            }
            Primitive::Agent {
                cx, cy, r, color, ..
            } => {
                let _ = writeln!(
                    out,
                    "<circle cx=\"{cx:.1}\" cy=\"{cy:.1}\" r=\"{r:.1}\" fill=\"{color}\" \
                     stroke=\"#0f172a\" stroke-width=\"1\"/>"
                );
            }
            Primitive::Caption { x, y, text } => {
                let _ = writeln!(
                    out,
                    "<text x=\"{x:.1}\" y=\"{y:.1}\" fill=\"#cbd5e1\" font-size=\"12\" \
                     font-family=\"monospace\">{}</text>",
                    escape(text)
                );
            }
        }
    }
    let _ = writeln!(out, "</svg>");
    out
}

/// Minimal XML text escaping for labels drawn from sim-authored strings.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Count primitives by kind — used by the probe/pins as a cheap structural
/// assertion without string matching the whole SVG.
#[must_use]
pub fn count_sites(scene: &Scene) -> usize {
    scene
        .primitives
        .iter()
        .filter(|p| matches!(p, Primitive::Site { .. }))
        .count()
}

/// Count agent discs in the scene.
#[must_use]
pub fn count_agents(scene: &Scene) -> usize {
    scene
        .primitives
        .iter()
        .filter(|p| matches!(p, Primitive::Agent { .. }))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mindstrata_sim::sim::assets::{CultureAsset, MetaAsset, WorldSection};

    fn doc() -> WorldAssets {
        WorldAssets {
            schema_version: 1,
            meta: MetaAsset {
                seed: 42,
                ticks: 10_000,
                agents: 2,
                polities: 1,
            },
            world: WorldSection {
                sites: vec![
                    SiteAsset {
                        id: 0,
                        kind: "Farm".into(),
                        name: "North Field".into(),
                        position: (3, 4),
                        capacity: 200,
                    },
                    SiteAsset {
                        id: 1,
                        kind: "Well".into(),
                        name: "Old Well & Co".into(),
                        position: (0, 0),
                        capacity: 50,
                    },
                ],
            },
            agents: vec![
                AgentAsset {
                    id: 0,
                    position: (3, 4),
                    polity: Some(0),
                },
                AgentAsset {
                    id: 1,
                    position: (1, 1),
                    polity: None,
                },
            ],
            polities: vec![],
            culture: vec![CultureAsset {
                id: 7,
                description: "Harvest song".into(),
                content_type: "Ritual".into(),
                created_tick: 1_200,
                hosts: vec![0, 1],
            }],
            annals: String::new(),
        }
    }

    #[test]
    fn scene_places_every_site_and_agent() {
        let scene = build_scene(&doc());
        assert_eq!(count_sites(&scene), 2);
        assert_eq!(count_agents(&scene), 2);
        // Extents cover the max tile (3, 4) → 4×5 tiles + margins.
        assert!(scene.width >= 4.0 * TILE + MARGIN * 2.0);
        assert!(scene.height > 5.0 * TILE + MARGIN * 2.0, "caption band");
    }

    #[test]
    fn polity_colours_agents_and_neutral_stays_neutral() {
        let scene = build_scene(&doc());
        let colors: Vec<&str> = scene
            .primitives
            .iter()
            .filter_map(|p| match p {
                Primitive::Agent { color, .. } => Some(*color),
                _ => None,
            })
            .collect();
        assert_eq!(colors, vec![POLITY_COLORS[0], NEUTRAL]);
    }

    #[test]
    fn svg_is_deterministic_and_escapes_labels() {
        let d = doc();
        let a = to_svg(&build_scene(&d));
        let b = to_svg(&build_scene(&d));
        assert_eq!(a, b, "same document → byte-identical SVG");
        assert!(a.contains("Old Well &amp; Co"), "amorphous & escaped: {a}");
        assert!(!a.contains("Old Well & Co"), "raw & must not survive");
        assert!(a.starts_with("<svg") && a.trim_end().ends_with("</svg>"));
    }

    #[test]
    fn empty_document_is_zero_at_zero() {
        let mut d = doc();
        d.world.sites.clear();
        d.agents.clear();
        d.culture.clear();
        let scene = build_scene(&d);
        assert_eq!(count_sites(&scene), 0);
        assert_eq!(count_agents(&scene), 0);
        // Extents floor at 1 tile so the SVG remains valid.
        assert!(scene.width >= TILE + MARGIN * 2.0);
        let svg = to_svg(&scene);
        assert!(svg.contains("culture: 0 memes"), "{svg}");
    }

    #[test]
    fn unknown_site_kind_falls_back_neutral() {
        assert_eq!(site_glyph("Spaceport"), '?');
        assert_eq!(site_color("Spaceport"), "#475569");
    }

    #[test]
    fn ascii_render_places_sites_and_polity_markers() {
        let scene = build_scene(&doc());
        let grid = render_ascii(&scene);
        assert!(
            grid.contains("2 sites (2 placed)") || grid.contains("(2 placed)"),
            "{grid}"
        );
        // Site glyph at (3,4) and (0,0): Farm glyph and Well glyph appear.
        assert!(grid.contains('f'), "{grid}");
        assert!(grid.contains('w'), "{grid}");
        // Agent 0 is polity 0 → digit '0'; agent 1 unassigned → neutral cell.
        assert!(grid.contains('0'), "{grid}");
        assert!(grid.contains(AGENT_NEUTRAL_CELL), "{grid}");
        assert!(grid.contains("[0]×1"), "{grid}");
        assert!(render_ascii(&scene) == grid, "deterministic");
    }

    #[test]
    fn ascii_render_is_zero_at_zero() {
        let mut d = doc();
        d.world.sites.clear();
        d.agents.clear();
        let scene = build_scene(&d);
        let grid = render_ascii(&scene);
        assert!(grid.contains("(0 placed)"), "{grid}");
        // No agent markers, and an empty agent legend roster.
        assert!(!grid.contains("[0]×"), "{grid}");
        assert!(!grid.contains(AGENT_NEUTRAL_CELL), "{grid}");
    }
}
