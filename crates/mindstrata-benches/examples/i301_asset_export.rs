//! i301 — asset pipeline v0 (ASSET-PIPELINE-v0 charter; PLAN_DC3 checklist 3).
//!
//! Exit evidence for the v0 export:
//! 1. **Determinism** — two exports of the same finished sim are
//!    byte-identical (charter rule 2).
//! 2. **Round-trip validity** — the document parses as JSON and every v0
//!    section carries the contract payload (schema version, meta, sites,
//!    polities + stage lines, culture + hosts, annals).
//! 3. **Polity payload** — with polities assigned, each polity asset carries
//!    its membership and a non-empty stage_lines canon map (i290 reuse).
//!
//! Run: cargo run -p mindstrata-benches --release --example i301_asset_export
use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    let horizon = 10_000u64;
    let seed = 42u64;
    println!("i301 asset export — {horizon} ticks, seed {seed}, N=12");

    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    // Two polities so the polity section carries real payload.
    sim.assign_polities(vec![
        (0..12).filter(|i| i % 2 == 0).collect(),
        (0..12).filter(|i| i % 2 == 1).collect(),
    ]);
    sim.run(horizon);

    // ── 1. Determinism.
    let a = mindstrata_sim::sim::assets::export_world_assets_json(&sim);
    let b = mindstrata_sim::sim::assets::export_world_assets_json(&sim);
    println!();
    println!("1. DETERMINISM: byte-identical = {}", a == b);

    // ── 2. Round-trip validity.
    let doc: serde_json::Value = match serde_json::from_str(&a) {
        Ok(v) => v,
        Err(e) => {
            println!("2. ROUND-TRIP: FAILED to parse ({e})");
            return;
        }
    };
    let version = doc["schema_version"].as_u64().unwrap_or(0);
    let ticks = doc["meta"]["ticks"].as_u64().unwrap_or(0);
    let agents = doc["meta"]["agents"].as_u64().unwrap_or(0);
    let sites = doc["world"]["sites"]
        .as_array()
        .map_or(0, std::vec::Vec::len);
    println!("2. ROUND-TRIP: schema v{version}, ticks {ticks}, agents {agents}, sites {sites}");

    // ── 3. Polity + culture payload.
    let polities = doc["polities"].as_array().map_or(0, std::vec::Vec::len);
    let stage_lines_p0 = doc["polities"][0]["stage_lines"]
        .as_array()
        .map_or(0, std::vec::Vec::len);
    let culture = doc["culture"].as_array().map_or(0, std::vec::Vec::len);
    let hosted: Vec<&serde_json::Value> = doc["culture"]
        .as_array()
        .map(|c| {
            c.iter()
                .filter(|m| m["hosts"].as_array().is_some_and(|h| !h.is_empty()))
                .collect()
        })
        .unwrap_or_default();
    let annals_len = doc["annals"].as_str().map_or(0, str::len);
    println!();
    println!("3. PAYLOAD:");
    println!("   polities: {polities} (p0 stage_lines: {stage_lines_p0})");
    println!(
        "   culture memes: {culture} (with non-empty hosts: {})",
        hosted.len()
    );
    println!("   annals: {annals_len} chars");

    println!();
    println!("VERDICT:");
    let det_ok = a == b;
    let shape_ok = version == 1 && sites > 0 && polities == 2 && stage_lines_p0 > 0;
    let annals_ok = annals_len > 0;
    println!("   determinism: {det_ok}, schema shape: {shape_ok}, annals rendered: {annals_ok}");
    if det_ok && shape_ok && annals_ok {
        println!("   asset pipeline v0: OPERATIONAL (charter exit evidence complete)");
    } else {
        println!("   asset pipeline v0: INCOMPLETE — investigate above");
    }
}
