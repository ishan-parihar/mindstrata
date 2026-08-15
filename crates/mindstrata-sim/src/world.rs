//! World model — tiles, regions, sites, resources.

use crate::institutions::Institution;
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::EntityId;
use serde::{Deserialize, Serialize};

/// Resource ID for grain — the primary food resource.
pub const GRAIN_RESOURCE_ID: u64 = 0;

/// Resource ID for water — the primary hydration resource.
pub const WATER_RESOURCE_ID: u64 = 1;

/// Resource ID for coin — the medium of exchange.
pub const COIN_RESOURCE_ID: u64 = 2;

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

/// Access rights for a resource stock.
/// §19.5.E: Resources need access rights and ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessRight {
    /// Anyone can access this resource (e.g., public well).
    Public,
    /// Only the site owner can access.
    OwnerOnly,
    /// Members of an institution can access (e.g., temple grain for priests).
    InstitutionMembers,
}

/// A stock of a resource at a location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStock {
    pub resource_id: u64,
    pub quantity: Fixed,
    pub quality: Fixed,
    /// Who can access this resource.
    pub access: AccessRight,
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
    /// Maximum total resource units this site can store before overflow
    /// spoilage kicks in (§19.5.E "Storage").
    pub storage_capacity: Fixed,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Total water stock across all well sites.
    pub fn total_water(&self) -> Fixed {
        self.sites
            .iter()
            .filter(|s| s.kind == SiteKind::Well)
            .flat_map(|s| &s.inventory)
            .filter(|r| r.resource_id == WATER_RESOURCE_ID)
            .fold(Fixed::ZERO, |acc, r| acc + r.quantity)
    }

    /// Consume a quantity of a resource from a site. Returns amount actually taken.
    pub fn consume_resource(&mut self, site_idx: usize, resource_id: u64, amount: Fixed) -> Fixed {
        if let Some(site) = self.sites.get_mut(site_idx) {
            if let Some(stock) = site
                .inventory
                .iter_mut()
                .find(|s| s.resource_id == resource_id)
            {
                let taken = amount.min(stock.quantity);
                stock.quantity = (stock.quantity - taken).max(Fixed::ZERO);
                return taken;
            }
        }
        Fixed::ZERO
    }

    /// Produce a quantity of a resource at a site (e.g. farming generates grain).
    pub fn produce_resource(&mut self, site_idx: usize, resource_id: u64, amount: Fixed) {
        if let Some(site) = self.sites.get_mut(site_idx) {
            if let Some(stock) = site
                .inventory
                .iter_mut()
                .find(|s| s.resource_id == resource_id)
            {
                stock.quantity += amount;
            } else {
                site.inventory.push(ResourceStock {
                    resource_id,
                    quantity: amount,
                    quality: Fixed::from_f64(0.8),
                    access: AccessRight::Public,
                });
            }
        }
    }

    /// §6: Get the (x, y) coordinates of a site by index.
    /// Sites are stored in a flat list; we reverse-lookup from the tile grid.
    pub fn site_position(&self, site_idx: usize) -> Option<(i32, i32)> {
        let site = self.sites.get(site_idx)?;
        let site_id = site.id;
        for (tile_idx, tile) in self.tiles.iter().enumerate() {
            if tile.site == Some(site_id) {
                let x = (tile_idx as u32 % self.width) as i32;
                let y = (tile_idx as u32 / self.width) as i32;
                return Some((x, y));
            }
        }
        None
    }

    /// Find the nearest site of a given kind to an optional home site.
    pub fn nearest_site_of_kind(&self, kind: SiteKind, _from_site: Option<usize>) -> Option<usize> {
        self.sites.iter().position(|s| s.kind == kind)
    }

    /// §6: Find the nearest site of a given kind to a position (by Manhattan distance).
    pub fn nearest_site_of_kind_to_pos(
        &self,
        kind: SiteKind,
        pos_x: i32,
        pos_y: i32,
    ) -> Option<usize> {
        let mut best: Option<(usize, i32)> = None;
        for (i, site) in self.sites.iter().enumerate() {
            if site.kind == kind {
                if let Some((sx, sy)) = self.site_position(i) {
                    let dist = (sx - pos_x).abs() + (sy - pos_y).abs();
                    match best {
                        None => best = Some((i, dist)),
                        Some((_, best_dist)) if dist < best_dist => best = Some((i, dist)),
                        _ => {}
                    }
                }
            }
        }
        best.map(|(i, _)| i)
    }

    /// §6: Find the nearest site of any of the given kinds to a position.
    pub fn nearest_site_of_kinds_to_pos(
        &self,
        kinds: &[SiteKind],
        pos_x: i32,
        pos_y: i32,
    ) -> Option<usize> {
        let mut best: Option<(usize, i32)> = None;
        for (i, site) in self.sites.iter().enumerate() {
            if kinds.contains(&site.kind) {
                if let Some((sx, sy)) = self.site_position(i) {
                    let dist = (sx - pos_x).abs() + (sy - pos_y).abs();
                    match best {
                        None => best = Some((i, dist)),
                        Some((_, best_dist)) if dist < best_dist => best = Some((i, dist)),
                        _ => {}
                    }
                }
            }
        }
        best.map(|(i, _)| i)
    }

    /// §6: Manhattan distance between two grid positions.
    pub fn manhattan_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
        (x1 - x2).abs() + (y1 - y2).abs()
    }

    /// Find a farm site with available grain.
    pub fn farm_with_grain(&self) -> Option<usize> {
        self.sites
            .iter()
            .enumerate()
            .find(|(_, s)| {
                s.kind == SiteKind::Farm
                    && s.inventory
                        .iter()
                        .any(|r| r.resource_id == GRAIN_RESOURCE_ID && r.quantity > Fixed::ZERO)
            })
            .map(|(i, _)| i)
    }

    /// §8.1.4 (P3-6): Find a farm holding at least `amount` of grain — the
    /// Trade-path counterpart of `accessible_farm_with_grain_amount`. A trade
    /// must move a full portion; crumbs are not marketable grain and must not
    /// feed a buyer through the Trade action's hunger relief.
    pub fn farm_with_grain_amount(&self, amount: Fixed) -> Option<usize> {
        self.sites
            .iter()
            .enumerate()
            .find(|(_, s)| {
                s.kind == SiteKind::Farm
                    && s.inventory.iter().any(|r| {
                        r.resource_id == GRAIN_RESOURCE_ID && r.quantity >= amount
                    })
            })
            .map(|(i, _)| i)
    }

    /// Find a well site with available water.
    pub fn well_with_water(&self) -> Option<usize> {
        self.sites
            .iter()
            .enumerate()
            .find(|(_, s)| {
                s.kind == SiteKind::Well
                    && s.inventory
                        .iter()
                        .any(|r| r.resource_id == WATER_RESOURCE_ID && r.quantity > Fixed::ZERO)
            })
            .map(|(i, _)| i)
    }

    /// Find the most productive farm site (highest grain stock) for work.
    pub fn best_farm_for_work(&self) -> Option<usize> {
        self.sites
            .iter()
            .enumerate()
            .filter(|(_, s)| s.kind == SiteKind::Farm)
            .max_by(|(_, a), (_, b)| {
                let fa = a
                    .inventory
                    .iter()
                    .find(|r| r.resource_id == GRAIN_RESOURCE_ID)
                    .map_or(Fixed::ZERO, |r| r.quantity);
                let fb = b
                    .inventory
                    .iter()
                    .find(|r| r.resource_id == GRAIN_RESOURCE_ID)
                    .map_or(Fixed::ZERO, |r| r.quantity);
                fa.cmp(&fb)
            })
            .map(|(i, _)| i)
    }

    /// §19.5.E: Check if an agent can access a resource at a site.
    /// Returns true if the agent has access rights.
    pub fn can_access_resource(
        &self,
        site_idx: usize,
        resource_id: u64,
        agent_id: EntityId,
        institutions: &[Institution],
    ) -> bool {
        if let Some(site) = self.sites.get(site_idx) {
            if let Some(stock) = site.inventory.iter().find(|s| s.resource_id == resource_id) {
                match stock.access {
                    AccessRight::Public => true,
                    AccessRight::OwnerOnly => site.owner == Some(agent_id),
                    AccessRight::InstitutionMembers => {
                        // §19.5.E: Check if agent is a member of any institution that controls this site.
                        institutions.iter().any(|inst| inst.has_member(agent_id))
                    }
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    /// §19.5.E: Find a site with accessible grain for an agent.
    pub fn accessible_farm_with_grain(
        &self,
        agent_id: EntityId,
        institutions: &[Institution],
    ) -> Option<usize> {
        self.sites
            .iter()
            .enumerate()
            .find(|(i, s)| {
                s.kind == SiteKind::Farm
                    && s.inventory
                        .iter()
                        .any(|r| r.resource_id == GRAIN_RESOURCE_ID && r.quantity > Fixed::ZERO)
                    && self.can_access_resource(*i, GRAIN_RESOURCE_ID, agent_id, institutions)
            })
            .map(|(i, _)| i)
    }

    /// §8.1.4 (P3-6): Find an accessible farm holding AT LEAST `amount` of
    /// grain — a full meal's worth. `accessible_farm_with_grain` matches any
    /// nonzero stock, and `consume_resource` takes `min(amount, stock)`, so a
    /// 0.015-grain crumb "fed" an agent and fully relieved hunger through a
    /// famine (hunger stayed pinned near 0 while the granary bottomed out).
    /// The Eat path now requires a full portion: a farm holding less than a
    /// meal's worth fails the Eat, hunger accumulates, and the famine actually
    /// starves its victims.
    pub fn accessible_farm_with_grain_amount(
        &self,
        agent_id: EntityId,
        institutions: &[Institution],
        amount: Fixed,
    ) -> Option<usize> {
        self.sites
            .iter()
            .enumerate()
            .find(|(i, s)| {
                s.kind == SiteKind::Farm
                    && s.inventory.iter().any(|r| {
                        r.resource_id == GRAIN_RESOURCE_ID && r.quantity >= amount
                    })
                    && self.can_access_resource(*i, GRAIN_RESOURCE_ID, agent_id, institutions)
            })
            .map(|(i, _)| i)
    }

    /// §19.5.E: Find a site with accessible water for an agent.
    pub fn accessible_well_with_water(
        &self,
        agent_id: EntityId,
        institutions: &[Institution],
    ) -> Option<usize> {
        self.sites
            .iter()
            .enumerate()
            .find(|(i, s)| {
                s.kind == SiteKind::Well
                    && s.inventory
                        .iter()
                        .any(|r| r.resource_id == WATER_RESOURCE_ID && r.quantity > Fixed::ZERO)
                    && self.can_access_resource(*i, WATER_RESOURCE_ID, agent_id, institutions)
            })
            .map(|(i, _)| i)
    }

    /// §19.5.D: Find a site with grain that an agent cannot access (for theft tracking).
    pub fn inaccessible_farm_with_grain(
        &self,
        agent_id: EntityId,
        institutions: &[Institution],
    ) -> Option<usize> {
        self.sites
            .iter()
            .enumerate()
            .find(|(i, s)| {
                s.kind == SiteKind::Farm
                    && s.inventory
                        .iter()
                        .any(|r| r.resource_id == GRAIN_RESOURCE_ID && r.quantity > Fixed::ZERO)
                    && !self.can_access_resource(*i, GRAIN_RESOURCE_ID, agent_id, institutions)
            })
            .map(|(i, _)| i)
    }

    /// §8.1.4 (P3-6): Find an INACCESSIBLE farm holding at least `amount` of
    /// grain (the theft-path counterpart of
    /// `accessible_farm_with_grain_amount` — crumbs are not worth stealing
    /// and must not feed a thief).
    pub fn inaccessible_farm_with_grain_amount(
        &self,
        agent_id: EntityId,
        institutions: &[Institution],
        amount: Fixed,
    ) -> Option<usize> {
        self.sites
            .iter()
            .enumerate()
            .find(|(i, s)| {
                s.kind == SiteKind::Farm
                    && s.inventory.iter().any(|r| {
                        r.resource_id == GRAIN_RESOURCE_ID && r.quantity >= amount
                    })
                    && !self.can_access_resource(*i, GRAIN_RESOURCE_ID, agent_id, institutions)
            })
            .map(|(i, _)| i)
    }

    /// §19.5.D: Find a site with water that an agent cannot access (for theft tracking).
    pub fn inaccessible_well_with_water(
        &self,
        agent_id: EntityId,
        institutions: &[Institution],
    ) -> Option<usize> {
        self.sites
            .iter()
            .enumerate()
            .find(|(i, s)| {
                s.kind == SiteKind::Well
                    && s.inventory
                        .iter()
                        .any(|r| r.resource_id == WATER_RESOURCE_ID && r.quantity > Fixed::ZERO)
                    && !self.can_access_resource(*i, WATER_RESOURCE_ID, agent_id, institutions)
            })
            .map(|(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn farm_world(quantities: &[f64]) -> World {
        let mut w = World::new(4, 4);
        w.sites.clear();
        for (i, q) in quantities.iter().enumerate() {
            let s = Site {
                id: EntityId::new(i as u64),
                kind: SiteKind::Farm,
                name: format!("farm-{i}"),
                owner: None,
                capacity: 100,
                storage_capacity: Fixed::from_f64(100.0),
                inventory: if *q > 0.0 {
                    vec![ResourceStock {
                        resource_id: GRAIN_RESOURCE_ID,
                        quantity: Fixed::from_f64(*q),
                        quality: Fixed::from_f64(0.8),
                        access: AccessRight::Public,
                    }]
                } else {
                    Vec::new()
                },
            };
            w.sites.push(s);
        }
        w
    }

    #[test]
    fn accessible_farm_with_grain_amount_requires_full_portion() {
        // P3-6: a farm holding a crumb (0.05 < 0.1 portion) must NOT qualify;
        // the un-gated helper still matches it (the bug that fed agents
        // through a famine).
        let w = farm_world(&[0.05, 0.2]);
        let agent = EntityId::new(0);
        let inst = &[];
        assert_eq!(w.accessible_farm_with_grain(agent, inst), Some(0));
        assert_eq!(
            w.accessible_farm_with_grain_amount(agent, inst, Fixed::from_f64(0.1)),
            Some(1),
            "only the full-portion farm qualifies"
        );
        assert_eq!(
            w.accessible_farm_with_grain_amount(agent, inst, Fixed::from_f64(0.5)),
            None,
            "no farm holds a 0.5 portion"
        );
    }

    #[test]
    fn inaccessible_farm_with_grain_amount_requires_full_portion() {
        // Build a farm that is NOT accessible to the agent (restricted access)
        // holding only a crumb, plus one holding a full portion.
        let mut w = World::new(4, 4);
        w.sites.clear();
        let mut crumb = Site {
            id: EntityId::new(0),
            kind: SiteKind::Farm,
            name: "crumb".into(),
            owner: Some(EntityId::new(99)),
            capacity: 100,
            storage_capacity: Fixed::from_f64(100.0),
            inventory: vec![ResourceStock {
                resource_id: GRAIN_RESOURCE_ID,
                quantity: Fixed::from_f64(0.05),
                quality: Fixed::from_f64(0.8),
                access: AccessRight::OwnerOnly,
            }],
        };
        crumb.owner = Some(EntityId::new(99));
        w.sites.push(crumb);
        let agent = EntityId::new(0);
        let inst = &[];
        assert_eq!(w.inaccessible_farm_with_grain(agent, inst), Some(0));
        assert_eq!(
            w.inaccessible_farm_with_grain_amount(agent, inst, Fixed::from_f64(0.1)),
            None,
            "the crumb must not qualify as a full portion"
        );
    }

    #[test]
    fn farm_with_grain_amount_requires_full_portion() {
        let w = farm_world(&[0.05, 0.2]);
        assert_eq!(w.farm_with_grain(), Some(0));
        assert_eq!(
            w.farm_with_grain_amount(Fixed::from_f64(0.1)),
            Some(1),
            "only the full-portion farm qualifies"
        );
    }
}
