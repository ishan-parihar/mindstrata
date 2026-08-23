//! World generation — terrain, sites, resources, and agent placement.

use crate::world::{
    AccessRight, Region, ResourceDef, ResourceStock, Site, SiteKind, Terrain, Tile, World,
    COIN_RESOURCE_ID, GRAIN_RESOURCE_ID,
};
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::EntityId;
use mindstrata_core::rng::{RngStream, RngStreams};
use rand::Rng;

/// Place a site at (x, y) if the coordinates are in bounds.
fn place_site(world: &mut World, x: i32, y: i32, site: Site) -> bool {
    if let Some(tile) = world.tile_mut(x, y) {
        tile.site = Some(site.id);
        world.sites.push(site);
        true
    } else {
        false
    }
}

/// Generate a small village world.
pub fn generate_village(world: &mut World, rng: &mut RngStreams) {
    let world_rng = rng.get_mut(RngStream::World);
    let w = world.width as i32;
    let h = world.height as i32;

    // Place terrain features
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            let roll: f64 = world_rng.random_range(0.0..1.0);

            world.tiles[idx] = if y < 2 || y >= h - 2 {
                Tile::new(Terrain::Forest)
            } else if x < 2 || x >= w - 2 {
                Tile::new(Terrain::Hill)
            } else if roll < 0.05 {
                Tile::new(Terrain::Water)
            } else {
                Tile::new(Terrain::Grassland)
            };
        }
    }

    // Iteration 257 (audit Phase 5 - world variance): the river MEANDERS -
    // a momentum random walk instead of a ruler-straight line - and its
    // banks carry a distance-falloff fertility gradient (rich bottomland
    // near the water, thin soil far from it). Same seed -> same river.
    let mut river_x = w / 2;
    let mut drift: i32 = 0;
    for y in 2..h - 2 {
        if world_rng.random_bool(0.35) {
            drift += world_rng.random_range(-1..=1);
        }
        drift = drift.clamp(-2, 2);
        river_x = (river_x + drift).clamp(2, w - 3);
        let idx = (y * w + river_x) as usize;
        world.tiles[idx] = Tile::new(Terrain::Water);
        for bank in [river_x - 1, river_x + 1] {
            if bank > 0 && bank < w {
                let bank_idx = (y * w + bank) as usize;
                world.tiles[bank_idx].fertility = Fixed::from_f64(0.95);
                world.tiles[bank_idx].moisture = Fixed::from_f64(0.9);
            }
        }
    }
    // Distance-to-water fertility field: every non-water tile's fertility
    // grades from 0.9 adjacent to the river toward 0.4 at the map edge,
    // plus small per-tile noise. Deterministic from the World stream.
    let water_cols: Vec<i32> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|(x, y)| matches!(world.tiles[(y * w + x) as usize].terrain, Terrain::Water))
        .map(|(x, _)| x)
        .collect();
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if matches!(world.tiles[idx].terrain, Terrain::Water) {
                continue;
            }
            let nearest = water_cols
                .iter()
                .map(|cx| (cx - x).abs())
                .min()
                .unwrap_or(w);
            let base = 0.9 - (nearest as f64 * 0.08);
            let noise: f64 = world_rng.random_range(-0.05..0.05);
            let fert = (base + noise).clamp(0.3, 0.95);
            world.tiles[idx].fertility = Fixed::from_f64(fert);
        }
    }

    let mut site_id = 0u64;
    let center_x = w / 2;
    let center_y = h / 2;

    // Iteration 257: houses on a jittered ring - seeded per-house radius
    // (3-5) and angle wobble break the perfect-circle grammar. A candidate
    // that lands on water falls back to the unjittered position.
    for i in 0..8 {
        let angle = (i as f64) * std::f64::consts::PI / 4.0 + world_rng.random_range(-0.15..0.15);
        let radius = 4.0 + world_rng.random_range(-1.0..1.0);
        let mut hx = center_x + (angle.cos() * radius) as i32;
        let mut hy = center_y + (angle.sin() * radius) as i32;
        let on_water = world
            .tile(hx, hy)
            .is_some_and(|t| matches!(t.terrain, Terrain::Water));
        if on_water {
            hx = center_x + (angle.cos() * 4.0) as i32;
            hy = center_y + (angle.sin() * 4.0) as i32;
        }
        let site = Site {
            id: EntityId::new(site_id),
            kind: SiteKind::House,
            name: format!("House {}", i + 1),
            owner: None,
            capacity: 4,
            storage_capacity: Fixed::from_f64(200.0),
            inventory: vec![],
        };
        if place_site(world, hx, hy, site) {
            site_id += 1;
        }
    }

    // Place farm (west of center)
    let farm = Site {
        id: EntityId::new(site_id),
        kind: SiteKind::Farm,
        name: "Village Farm".into(),
        owner: None,
        capacity: 10,
        storage_capacity: Fixed::from_f64(500.0),
        inventory: vec![ResourceStock {
            resource_id: GRAIN_RESOURCE_ID,
            // Iteration 257: soil-quality multiplier from the local
            // fertility field - richer land founds with bigger granaries.
            quantity: Fixed::from_f64(
                100.0
                    * world
                        .tile(center_x.saturating_sub(6), center_y)
                        .map_or(1.4, |t| (0.6 + t.fertility.to_f64()).clamp(0.8, 1.5)),
            ),
            quality: Fixed::from_f64(0.8),
            access: AccessRight::Public,
        }],
    };
    if place_site(world, center_x.saturating_sub(6), center_y, farm) {
        site_id += 1;
    }

    // Place well (south of center)
    let well = Site {
        id: EntityId::new(site_id),
        kind: SiteKind::Well,
        name: "Village Well".into(),
        owner: None,
        capacity: 20,
        // Iteration 228: well capacity raised from 1000→2000 and initial
        // stock from 200→2000 so the Drought shock's 70% proportional drain
        // leaves 600 water — enough to create ~500 ticks of additional
        // scarcity beyond normal consumption. Previously, agent consumption
        // drained the 200-stock well faster than the shock could matter.
        storage_capacity: Fixed::from_f64(2000.0),
        inventory: vec![ResourceStock {
            resource_id: 1, // WATER_RESOURCE_ID
            quantity: Fixed::from_f64(2000.0),
            quality: Fixed::from_f64(1.0),
            access: AccessRight::Public,
        }],
    };
    if place_site(world, center_x, (center_y + 5).min(h - 1), well) {
        site_id += 1;
    }

    // Place market (east of center)
    let market = Site {
        id: EntityId::new(site_id),
        kind: SiteKind::Market,
        name: "Village Market".into(),
        owner: None,
        capacity: 30,
        storage_capacity: Fixed::from_f64(1500.0),
        inventory: vec![
            ResourceStock {
                resource_id: GRAIN_RESOURCE_ID,
                quantity: Fixed::from_f64(50.0),
                quality: Fixed::from_f64(0.9),
                access: AccessRight::Public,
            },
            ResourceStock {
                resource_id: 1,
                quantity: Fixed::from_f64(200.0),
                quality: Fixed::from_f64(1.0),
                access: AccessRight::Public,
            },
            ResourceStock {
                resource_id: COIN_RESOURCE_ID,
                quantity: Fixed::from_f64(500.0),
                quality: Fixed::ONE,
                access: AccessRight::Public,
            },
        ],
    };
    if place_site(world, (center_x + 5).min(w - 1), center_y, market) {
        site_id += 1;
    }

    // Place temple (north of center)
    let temple = Site {
        id: EntityId::new(site_id),
        kind: SiteKind::Temple,
        name: "Village Temple".into(),
        owner: None,
        capacity: 50,
        storage_capacity: Fixed::from_f64(300.0),
        inventory: vec![],
    };
    // Place temple (north of center) — always call place_site, assert in debug only
    let temple_placed = place_site(world, center_x, center_y.saturating_sub(5), temple);
    debug_assert!(
        temple_placed,
        "Temple placement failed — out of bounds at ({}, {})",
        center_x,
        center_y.saturating_sub(5)
    );

    // Add resource definitions
    world.resources.push(ResourceDef {
        id: GRAIN_RESOURCE_ID,
        name: "Grain".into(),
        perishable: true,
        spoilage_rate: Fixed::from_f64(0.001),
    });
    world.resources.push(ResourceDef {
        id: 1,
        name: "Water".into(),
        perishable: false,
        spoilage_rate: Fixed::ZERO,
    });
    world.resources.push(ResourceDef {
        id: COIN_RESOURCE_ID,
        name: "Coin".into(),
        perishable: false,
        spoilage_rate: Fixed::ZERO,
    });

    // Add region
    world.regions.push(Region {
        name: "Riverford".into(),
        origin_x: 0,
        origin_y: 0,
        width: world.width,
        height: world.height,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn village_has_sites() {
        let mut rng = RngStreams::new(42);
        let mut world = World::new(16, 16);
        generate_village(&mut world, &mut rng);

        assert!(!world.sites.is_empty());
        assert!(!world.regions.is_empty());
        assert!(!world.resources.is_empty());

        let site_kinds: Vec<_> = world.sites.iter().map(|s| s.kind).collect();
        assert!(site_kinds.contains(&SiteKind::Farm));
        assert!(site_kinds.contains(&SiteKind::Well));
        assert!(site_kinds.contains(&SiteKind::Market));
        assert!(site_kinds.contains(&SiteKind::Temple));
    }

    #[test]
    fn village_has_food() {
        let mut rng = RngStreams::new(42);
        let mut world = World::new(16, 16);
        generate_village(&mut world, &mut rng);

        let total_food = world.total_food();
        assert!(total_food > Fixed::ZERO);
    }
}
