//! i322 — DC-4(c) graphical-shell spike: the i301 asset document → scene graph
//! → SVG, end to end.
//!
//! The ASSET-PIPELINE-v0 charter §4 says the document "is the seed of the scene
//! graph — sites → meshes, culture → narrative props". This probe exercises the
//! CLIENT spike (`mindstrata_tui::scene`) over a real 10K run: build, then
//! render to SVG, then assert the contract the charter demands:
//!
//! 1. **Coverage** — every site and every agent in the document reaches the
//!    scene (counts match the document).
//! 2. **Read-only / out-of-process** — the scene is built only from the
//!    document (no sim access), so the export-golden invariant holds.
//! 3. **Determinism** (charter rule 2) — same document → byte-identical SVG.
//! 4. **Observable output** — the SVG is written to `target/` so a human (or a
//!    browser) can look at the rendered village; the probe prints the header.
//!
//! Run: cargo run --release -p mindstrata-benches --example i322_scene_graph

use mindstrata_sim::sim::assets::export_world_assets;
use mindstrata_sim::sim::{SimConfig, Simulation};
use mindstrata_tui::scene::{build_scene, count_agents, count_sites, to_svg};

fn main() {
    println!("i322 DC-4(c) graphical-shell spike — asset document → scene graph → SVG");

    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    // Two polities so territory colouring carries real payload.
    sim.assign_polities(vec![
        (0..12).filter(|i| i % 2 == 0).collect(),
        (0..12).filter(|i| i % 2 == 1).collect(),
    ]);
    sim.run(10_000);

    // The ONLY input to the scene is the published document (charter §1:
    // clients never reach into sim internals).
    let doc = export_world_assets(&sim);
    let scene = build_scene(&doc);
    let svg = to_svg(&scene);

    let sites_ok = count_sites(&scene) == doc.world.sites.len();
    let agents_ok = count_agents(&scene) == doc.agents.len();
    let territories = doc.agents.iter().filter(|a| a.polity.is_some()).count();

    // Determinism: rebuild from the same document → identical SVG.
    let svg_again = to_svg(&build_scene(&doc));
    let deterministic = svg == svg_again;

    // A distinct world (different seed) must produce a different scene — proves
    // the output is actually driven by the document, not a constant template.
    let mut sim2 = Simulation::new(SimConfig {
        seed: 99,
        max_ticks: 200,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim2.populate();
    sim2.run(200);
    let svg_other = to_svg(&build_scene(&export_world_assets(&sim2)));
    let document_driven = svg != svg_other;

    let out_path = std::path::Path::new("target/i322_scene_graph.svg");
    let wrote = std::fs::write(out_path, &svg).is_ok();

    println!(
        "  document: schema v{} · {} sites · {} agents ({} in a polity) · {} memes",
        doc.schema_version,
        doc.world.sites.len(),
        doc.agents.len(),
        territories,
        doc.culture.len()
    );
    println!(
        "  scene:    {}×{:.0} units · {} primitives (sites {} · agents {})",
        scene.width as i64,
        scene.height,
        scene.primitives.len(),
        count_sites(&scene),
        count_agents(&scene)
    );
    println!("  svg:      {} bytes → {}", svg.len(), out_path.display());
    println!();
    let checks = vec![
        ("site coverage", sites_ok),
        ("agent coverage", agents_ok),
        ("territory payload", territories > 0),
        ("deterministic render", deterministic),
        ("document-driven output", document_driven),
        ("svg written", wrote),
        (
            "svg well-formed",
            svg.starts_with("<svg") && svg.trim_end().ends_with("</svg>"),
        ),
    ];
    for (name, ok) in &checks {
        println!("  {:<22} {}", name, if *ok { "PASS" } else { "FAIL" });
    }
    println!("\n--- svg header ---");
    for line in svg.lines().take(4) {
        println!("{line}");
    }

    let all_pass = checks.iter().all(|(_, ok)| *ok);
    println!(
        "\nverdict={}",
        if all_pass {
            "SCENE_GRAPH_SPIKE_LIVE"
        } else {
            "SCENE_GRAPH_SPIKE_PARTIAL"
        }
    );
}
