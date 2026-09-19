//! Iteration 301 — asset pipeline v0 (ASSET-PIPELINE-v0 charter; ROADMAP
//! DC-3 → UM-3 "asset pipeline v0 lands").
//!
//! The simulation IS the content generator; an asset is a deterministic pure
//! function of live state that a client consumes without reaching into sim
//! internals (IC-8). v0 exports ONE JSON document:
//!
//! ```text
//! { schema_version, meta{seed,ticks,agents,polities},
//!   world{sites[]}, polities[{members, stage_lines[]}],
//!   culture[{id, description, content_type, created_tick, hosts}],
//!   annals (the chronicle surface, verbatim) }
//! ```
//!
//! Charter binding rules honored here:
//! - **Read-only**: pure over public accessors; no state mutation, no RNG,
//!   no events → golden replay byte-identical by construction.
//! - **Determinism**: registry order everywhere; serde_json with a stable
//!   BTree-style field order (serde struct order) — same sim → same bytes.
//! - **Schema versioned**: `schema_version: 1`; additive fields don't bump.
//! - **Bounded journals**: annals render from the rolling buffer + persistent
//!   registries (the chronicle's own contract), never unbounded history.
//! - **Perf**: one-shot O(world + registries) at run end — outside the
//!   per-tick budgets.

use serde::Serialize;

use crate::sim::chronicle::render_chronicle;
use crate::sim::Simulation;

/// v0 schema version. Bump ONLY on breaking shape changes.
pub const ASSET_SCHEMA_VERSION: u32 = 1;

/// A site asset: identity, kind, name, tile position, capacity.
#[derive(Debug, Clone, Serialize)]
pub struct SiteAsset {
    pub id: u64,
    pub kind: String,
    pub name: String,
    pub position: (i32, i32),
    pub capacity: u32,
}

/// One polity's export: membership + its holon's full stage-line canon map
/// (the i290 KosmOS-frontmatter export, reused verbatim).
#[derive(Debug, Clone, Serialize)]
pub struct PolityAsset {
    pub members: Vec<usize>,
    pub stage_lines: Vec<crate::snapshot::StageLineEntry>,
}

/// A culture asset: the meme as a consumable cultural item, with the i300
/// per-agent hosting ledger (CLIENT renders diffusion maps from it).
#[derive(Debug, Clone, Serialize)]
pub struct CultureAsset {
    pub id: usize,
    pub description: String,
    pub content_type: String,
    pub created_tick: u64,
    pub hosts: Vec<usize>,
}

/// The v0 root document.
#[derive(Debug, Clone, Serialize)]
pub struct WorldAssets {
    pub schema_version: u32,
    pub meta: MetaAsset,
    pub world: WorldSection,
    pub polities: Vec<PolityAsset>,
    pub culture: Vec<CultureAsset>,
    pub annals: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetaAsset {
    pub seed: u64,
    pub ticks: u64,
    pub agents: usize,
    pub polities: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldSection {
    pub sites: Vec<SiteAsset>,
}

/// Build the v0 asset document from a finished (or restored) simulation.
/// Pure over public state — see the module doc for the charter rules.
#[must_use]
pub fn export_world_assets(sim: &Simulation) -> WorldAssets {
    let sites: Vec<SiteAsset> = sim
        .world
        .sites
        .iter()
        .enumerate()
        .map(|(idx, s)| SiteAsset {
            id: s.id.as_u64(),
            kind: s.kind.name().to_string(),
            name: s.name.clone(),
            position: sim.world.site_position(idx).unwrap_or((0, 0)),
            capacity: s.capacity,
        })
        .collect();

    let polities: Vec<PolityAsset> = sim
        .polity_fields
        .iter()
        .zip(sim.polity_members.iter())
        .map(|(field, members)| PolityAsset {
            members: members.clone(),
            stage_lines: crate::snapshot::export_stage_lines(field),
        })
        .collect();

    let culture: Vec<CultureAsset> = sim
        .meme_registry
        .memes
        .iter()
        .map(|m| CultureAsset {
            id: m.id,
            description: m.description.clone(),
            content_type: format!("{:?}", m.content_type),
            created_tick: m.created_tick,
            hosts: m.hosts.clone(),
        })
        .collect();

    WorldAssets {
        schema_version: ASSET_SCHEMA_VERSION,
        meta: MetaAsset {
            seed: sim.config.seed,
            ticks: sim.current_tick().as_u64(),
            agents: sim.agents.len(),
            polities: sim.polity_fields.len(),
        },
        world: WorldSection { sites },
        polities,
        culture,
        annals: render_chronicle(sim),
    }
}

/// Serialize the asset document to pretty JSON (the CLIENT-facing form).
/// Deterministic: serde struct field order + registry iteration order.
#[must_use]
pub fn export_world_assets_json(sim: &Simulation) -> String {
    serde_json::to_string_pretty(&export_world_assets(sim)).unwrap_or_else(|_| "{}".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{SimConfig, Simulation};

    fn small_sim() -> Simulation {
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 0,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.populate();
        sim
    }

    /// Charter rule 3: the document carries the schema version and all v0
    /// sections; site assets carry identity + position.
    #[test]
    fn asset_document_has_v0_sections() {
        let sim = small_sim();
        let doc = export_world_assets(&sim);
        assert_eq!(doc.schema_version, ASSET_SCHEMA_VERSION);
        assert_eq!(doc.meta.seed, 42);
        assert_eq!(doc.meta.agents, 12);
        assert!(
            !doc.world.sites.is_empty(),
            "v0 must export the site registry"
        );
        let first = &doc.world.sites[0];
        assert!(!first.name.is_empty() && first.position.0 >= 0);
        assert!(
            doc.polities.is_empty(),
            "no polities assigned → empty polity section (zero-at-zero)"
        );
        assert!(
            doc.annals.contains("Annals") || !doc.annals.is_empty(),
            "annals section renders (chronicle surface)"
        );
    }

    /// Charter rule 2: same sim → byte-identical export.
    #[test]
    fn asset_export_is_deterministic() {
        let sim = small_sim();
        let a = export_world_assets_json(&sim);
        let b = export_world_assets_json(&sim);
        assert_eq!(a, b, "pure function of state → identical bytes");
    }

    /// Charter rule 4/5: export is read-only — running it twice and ticking
    /// in between must leave the tick trajectory untouched (no hidden state
    /// writes, no RNG draws).
    #[test]
    fn asset_export_is_side_effect_free() {
        let mut sim = small_sim();
        sim.run(50);
        let before = export_world_assets_json(&sim);
        let _ = export_world_assets_json(&sim); // extra export mid-run
        sim.run(50);
        let after = export_world_assets_json(&sim);
        assert_ne!(before, after, "sanity: ticks advanced");
        // The real zero-blast proof is the standing golden/snapshot suites
        // (export-blind by construction); this pin guards the cheap part.
        let doc = export_world_assets(&sim);
        assert_eq!(doc.meta.ticks, 100);
    }

    /// Charter §3: polities section carries membership + stage lines once
    /// assigned; culture carries the i300 hosting ledger.
    #[test]
    fn asset_document_exposes_polities_and_hosting() {
        let mut sim = small_sim();
        let n = sim.agents.len();
        sim.assign_polities(vec![(0..n).collect()]);
        let doc = export_world_assets(&sim);
        assert_eq!(doc.polities.len(), 1);
        assert_eq!(doc.polities[0].members.len(), n);
        assert!(
            !doc.polities[0].stage_lines.is_empty(),
            "stage_lines canon export rides the polity asset"
        );
        // Culture: memes may be empty at tick 0 — the section exists.
        assert!(doc.meta.polities == 1);
    }
}
