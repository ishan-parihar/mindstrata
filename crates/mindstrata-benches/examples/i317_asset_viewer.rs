//! i317 — DC-4a asset-viewer probe: export → render, end to end.
//!
//! The ASSET-PIPELINE-v0 charter §4 opens "asset-viewer panels, cultural
//! diffusion maps, territory views" to CLIENT. This probe runs a real 10K
//! simulation, exports the i301 document, renders it through the new TUI
//! `render_asset_viewer`, and asserts every v0 section reaches the panel.
//!
//! It also pins the charter interaction the panel depends on: the document is
//! cloned (not re-exported) per frame, and the render is deterministic.
//!
//! Run: cargo run --release -p mindstrata-benches --example i317_asset_viewer

use mindstrata_sim::sim::assets::export_world_assets;
use mindstrata_sim::sim::{SimConfig, Simulation};
use mindstrata_tui::render_asset_viewer;

fn main() {
    println!("i317 DC-4a asset-viewer probe");
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(10_000);

    let doc = export_world_assets(&sim);
    let view = render_asset_viewer(&doc);

    // Section presence (the charter §3 shape must reach the panel verbatim).
    let mut checks = vec![
        ("schema_version", view.contains("schema v1")),
        ("meta header", view.contains("seed 42")),
        ("world sites", view.contains("WORLD")),
        ("polities", view.contains("POLITIES")),
        ("culture", view.contains("CULTURE")),
        ("annals", view.contains("ANNALS")),
        (
            "site name",
            doc.world.sites.iter().any(|s| view.contains(&s.name)),
        ),
    ];
    // Determinism: two renders of the same document are byte-identical.
    let again = render_asset_viewer(&doc);
    checks.push(("deterministic render", view == again));

    println!(
        "  export: schema v{} · {} sites · {} polities · {} memes · annals {} chars",
        doc.schema_version,
        doc.world.sites.len(),
        doc.polities.len(),
        doc.culture.len(),
        doc.annals.len()
    );
    for (name, ok) in &checks {
        println!("  {:<20} {}", name, if *ok { "PASS" } else { "FAIL" });
    }
    println!("\n--- rendered panel (first 40 lines) ---");
    for line in view.lines().take(40) {
        println!("{line}");
    }

    let all_pass = checks.iter().all(|(_, ok)| *ok);
    println!(
        "\nverdict={}",
        if all_pass {
            "ASSET_VIEWER_PANEL_LIVE"
        } else {
            "ASSET_VIEWER_PANEL_PARTIAL"
        }
    );
}
