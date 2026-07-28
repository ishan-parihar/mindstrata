//! World model — tiles, regions, sites, resources.

use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::EntityId;
use serde::{Deserialize, Serialize};

/// Resource ID for grain — the primary food resource.
pub const GRAIN_RESOURCE_ID: u64 = 0;

// ── Terrain ──────────────────────────────────────────────────────────────

/// Basic terrain types for the initial 2D grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Terrain {
    Grassland,
    Forest,
    Hill,
    Mountain,
    Water,
    Desert,
    Swamp,
}

impl Terrain {
    /// Base fertility for this terrain type.
    pub fn base_fertility(self) -> Fixed {
        match self {
            Terrain::Grassland => Fixed::from_f64(0.8),
            Terrain::Forest => Fixed::from_f64(0.5),
            Terrain::Hill => Fixed::from_f64(0.4),
            Terrain::Mountain => Fixed::from_f64(0.1),
            Terrain::Water => Fixed::ZERO,
            Terrain::Desert => Fixed::from_f64(0.15),
            Terrain::Swamp => Fixed::from_f64(0.3),
        }
    }
}

// ── Tile ─────────────────────────────────────────────────────────────────

/// A single cell in the 2D world grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub terrain: Terrain,
    pub fertility: Fixed,
    pub moisture: Fixed,
    pub temperature: Fixed,
    pub depletion: Fixed,
    pub disease_pressure: Fixed,
    pub owner: Option<EntityId>,
    pub site: Option<EntityId>,
}

impl Tile {
    pub fn new(terrain: Terrain) -> Self {
        let fertility = terrain.base_fertility();
        Self {
            terrain,
            fertility,
            moisture: Fixed::from_f64(0.5),
            temperature: Fixed::from_f64(20.0),
            depletion: Fixed::ZERO,
            disease_pressure: Fixed::ZERO,
            owner: None,
            site: None,
        }
    }
}

// ── Resource ─────────────────────────────────────────────────────────────

/// A type of resource that can exist in the world.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceDef {
    pub id: u64,
    pub name: String,
    pub perishable: bool,
    pub spoilage_rate: Fixed,
}

/// A stock of a resource at a location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStock {
    pub resource_id: u64,
    pub quantity: Fixed,
    pub quality: Fixed,
}

// ── Site ─────────────────────────────────────────────────────────────────

/// A meaningful location in the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub id: EntityId,
    pub kind: SiteKind,
    pub name: String,
    pub owner: Option<EntityId>,
    pub capacity: u32,
    pub inventory: Vec<ResourceStock>,
}

/// Types of sites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SiteKind {
    House,
    Farm,
    Well,
    Market,
    Temple,
    Barracks,
    Workshop,
    Square,
    Prison,
    School,
}

impl SiteKind {
    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            SiteKind::House => "House",
            SiteKind::Farm => "Farm",
            SiteKind::Well => "Well",
            SiteKind::Market => "Market",
            SiteKind::Temple => "Temple",
            SiteKind::Barracks => "Barracks",
            SiteKind::Workshop => "Workshop",
            SiteKind::Square => "Square",
            SiteKind::Prison => "Prison",
            SiteKind::School => "School",
        }
    }
}

// ── Region ───────────────────────────────────────────────────────────────

/// A named region containing a rectangular area of tiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub name: String,
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: u32,
    pub height: u32,
}

// ── World ────────────────────────────────────────────────────────────────

/// The complete world state.
#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>,
    pub sites: Vec<Site>,
    pub regions: Vec<Region>,
    pub resources: Vec<ResourceDef>,
}

impl World {
    /// Create a new empty world of the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        let tiles = (0..width * height)
            .map(|_| Tile::new(Terrain::Grassland))
            .collect();
        Self {
            width,
            height,
            tiles,
            sites: Vec::new(),
            regions: Vec::new(),
            resources: Vec::new(),
        }
    }

    /// Get a tile by grid coordinates.
    pub fn tile(&self, x: i32, y: i32) -> Option<&Tile> {
        if x < 0 || y < 0 {
            return None;
        }
        let idx = y as u32 * self.width + x as u32;
        self.tiles.get(idx as usize)
    }

    /// Get a mutable tile by grid coordinates.
    pub fn tile_mut(&mut self, x: i32, y: i32) -> Option<&mut Tile> {
        if x < 0 || y < 0 {
            return None;
        }
        let idx = y as u32 * self.width + x as u32;
        self.tiles.get_mut(idx as usize)
    }

    /// Total food stock across all sites.
    pub fn total_food(&self) -> Fixed {
        self.sites
            .iter()
            .flat_map(|s| &s.inventory)
            .filter(|r| r.resource_id == GRAIN_RESOURCE_ID)
            .fold(Fixed::ZERO, |acc, r| acc + r.quantity)
    }
}
