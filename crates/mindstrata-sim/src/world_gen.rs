//! World generation — terrain, sites, resources, and agent placement.

use crate::world::{Region, ResourceDef, ResourceStock, Site, SiteKind, Terrain, Tile, World, GRAIN_RESOURCE_ID};
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::EntityId;
use mindstrata_core::rng::{RngStreams, RngStream};
use rand::Rng;

/// Place a site at (x, y) if the coordinates are in bounds.
fn place_site(
    world: &mut World,
    x: i32,
    y: i32,
    site: Site,
) -> bool {
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

    // Add a river running through the middle
    let river_x = w / 2;
    for y in 2..h - 2 {
        let idx = (y * w + river_x) as usize;
        world.tiles[idx] = Tile::new(Terrain::Water);
        if river_x > 0 {
            let bank_idx = (y * w + (river_x - 1)) as usize;
            world.tiles[bank_idx].fertility = Fixed::from_f64(0.95);
            world.tiles[bank_idx].moisture = Fixed::from_f64(0.9);
        }
    }

    let mut site_id = 0u64;
    let center_x = w / 2;
    let center_y = h / 2;

    // Place houses around center (8 houses)
    for i in 0..8 {
        let angle = (i as f64) * std::f64::consts::PI / 4.0;
        let hx = center_x + (angle.cos() * 4.0) as i32;
        let hy = center_y + (angle.sin() * 4.0) as i32;
        let site = Site {
            id: EntityId::new(site_id),
            kind: SiteKind::House,
            name: format!("House {}", i + 1),
            owner: None,
            capacity: 4,
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
        inventory: vec![ResourceStock {
            resource_id: GRAIN_RESOURCE_ID,
            quantity: Fixed::from_f64(100.0),
            quality: Fixed::from_f64(0.8),
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
        inventory: vec![ResourceStock {
            resource_id: 1, // WATER_RESOURCE_ID
            quantity: Fixed::from_f64(200.0),
            quality: Fixed::from_f64(1.0),
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
        inventory: vec![
            ResourceStock {
                resource_id: GRAIN_RESOURCE_ID,
                quantity: Fixed::from_f64(50.0),
                quality: Fixed::from_f64(0.9),
            },
            ResourceStock {
                resource_id: 1,
                quantity: Fixed::from_f64(200.0),
                quality: Fixed::from_f64(1.0),
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
        inventory: vec![],
    };
    // temple is the last site — no need to increment site_id after
    let _ = &mut site_id;
    // NOTE: debug_assert only fires in debug builds; release silently ignores failure.
    debug_assert!(
        place_site(world, center_x, center_y.saturating_sub(5), temple),
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
