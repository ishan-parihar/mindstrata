//! World generation — terrain, sites, resources, and agent placement.

use crate::world::{
    AccessRight, Region, ResourceDef, ResourceStock, Site, SiteKind, Terrain, Tile, World,
    COIN_RESOURCE_ID, GRAIN_RESOURCE_ID,
};
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::EntityId;
use mindstrata_core::rng::{RngStream, RngStreams};
use rand::Rng;

/// i360: find the first free (non-water, site-less) tile on a golden-angle ray
/// radiating from `(base_x, base_y)` — attempt 0 is the candidate itself, later
/// attempts widen by `step0 + attempt·width`. A bounded walk that never
/// consumes RNG so placement stays a pure function of the world stream.
fn free_tile_near(
    world: &World,
    base_x: i32,
    base_y: i32,
    golden_angle: f64,
    attempts: u32,
    width: f64,
    step0: f64,
    reserved: &[(i32, i32)],
) -> Option<(i32, i32)> {
    for attempt in 0..attempts {
        let (dx, dy) = if attempt == 0 {
            (0, 0)
        } else {
            let a = f64::from(attempt) * golden_angle;
            let step = step0 + f64::from(attempt) * width;
            (
                (a.cos() * step).round() as i32,
                (a.sin() * step).round() as i32,
            )
        };
        let (x, y) = (base_x + dx, base_y + dy);
        let free = world
            .tile(x, y)
            .is_some_and(|t| t.site.is_none() && !matches!(t.terrain, Terrain::Water))
            && !reserved.contains(&(x, y));
        if free {
            return Some((x, y));
        }
    }
    None
}

/// i360: the last-resort placement — the free tile nearest `(base_x, base_y)
/// (Manhattan; ties broken by scan order, y then x). Deterministic and total
/// (returns `None` only when the world has NO free tile). A golden-angle walk
/// samples a spiral and can miss sparse free tiles in a cramped world, so this
/// guarantees i344's invariant — one tile per house, always — while the local
/// walk keeps placement village-shaped in every normal case.
fn nearest_free_tile(
    world: &World,
    base_x: i32,
    base_y: i32,
    reserved: &[(i32, i32)],
) -> Option<(i32, i32)> {
    let mut best: Option<(i32, (i32, i32))> = None;
    for y in 0..world.height as i32 {
        for x in 0..world.width as i32 {
            let free = world
                .tile(x, y)
                .is_some_and(|t| t.site.is_none() && !matches!(t.terrain, Terrain::Water))
                && !reserved.contains(&(x, y));
            if !free {
                continue;
            }
            let d = (x - base_x).abs() + (y - base_y).abs();
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, (x, y)));
            }
        }
    }
    best.map(|(_, p)| p)
}

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

/// i344 (A9): the simulator's own calibrated spatial density — 16×16 for 12
/// agents and 32×32 for 48 agents **both** give 21.33 cells/agent, so the two
/// scales the engine was tuned at agree on one number.
pub const CELLS_PER_AGENT: f64 = 256.0 / 12.0;

/// i344 (A9): the smallest world side the density law will emit.
///
/// The 16×16 floor is the **N=12 calibrated point itself** (256 cells / 12
/// agents = 21.33), so the law reproduces the smallest calibrated world exactly
/// rather than approximating it — which is what makes every calibrated corpus
/// byte-identical under this policy.
pub const MIN_WORLD_SIDE: u32 = 16;

/// i344 (A9, DECIDED i392): the world side for a population at the calibrated
/// density — the interim world-area policy for any run that does not pin its
/// own size.
///
/// **Why a law.** i340/i344/i345 closed the *housing* half of the crowding
/// story (one house per ~4 villagers at a declared `House.capacity` of 4), and
/// i344 measured what remained: in the charter's **fixed** 32×32 world the
/// contact graph **re-saturates past N≈144** — contacted α 2.654, i.e. the
/// best-connected villager has met everyone, while a world at constant density
/// measures α **0.866**. Space is decorative, so ties form by co-residency and
/// proximity rather than by encounter.
///
/// **This does not fix capacity — A11 did.** i345's area packing is what took
/// max co-location `19 → 4` at N=192 (the ring had been silently stack-piling
/// agents on collided tiles against `SiteType::House.capacity`). The density
/// law's own measured effect is **contact dilution**: near-pair share
/// 11.3% → 4.7%, contacted-row share 11.5% → 5.3%, mean partners/agent
/// 22.1 → 10.2, contacted α 1.276 → 0.866. Its cost is **+5.6% µs/tick at
/// N=192** (measured +9.1%, −2.3%, +5.6% at N=96/144/192) — i344's verdict is
/// that world area is a **fidelity** lever, never a throughput one, because the
/// interaction store is the complete row set `N(N−1)` and a bigger world leaves
/// it untouched.
///
/// **Anchored, so the calibrated corpus cannot move.** The law is derived from
/// the simulator's own two calibrated points rather than invented: N=12 in 16×16
/// and N=48 in 32×32 *both* give 21.33 cells/agent, so
/// `world_side_for_population(12) == 16` and `world_side_for_population(48) ==
/// 32` **exactly**, and every window below the N≈96 tier is byte-identical
/// under either policy.
///
/// **Interim, not organic — and this is the point to keep in view.** This is
/// still an *input* keyed on a chosen population: it says "the world is this big
/// for this many agents", when the emergent form is the inverse — a fixed
/// physical area whose **population** is what varies, bounded by the ecology's
/// carrying capacity, with crowding as a live pressure (stress, infection,
/// conflict over space) and settlement **fission** as the demographic response.
/// That end state retires this function rather than replacing its constant. The
/// increments toward it are recorded in `docs/PLAN_DC5_DEVELOPMENT.md` §4
/// (encounter-driven contact i397, travel cost, carrying capacity, fission).
pub fn world_side_for_population(agents: u32) -> u32 {
    let side = (CELLS_PER_AGENT * f64::from(agents)).sqrt().ceil() as u32;
    side.max(MIN_WORLD_SIDE)
}

/// i339/i340: the village's baseline house count. Historically a hardcoded
/// `for i in 0..8` — and i338 measured that this makes every N live on exactly
/// 8 cells, which is what pins the relationship store at Ω(N²).
pub const DEFAULT_HOUSE_COUNT: u32 = 8;

/// i340: houses needed to seat a population of `agents` at the declared
/// `SiteKind::House.capacity` of 4.
///
/// The governor is the *co-residency* the store has to represent (i338): with a
/// fixed 8 houses, `ceil(N/8)` villagers share each cell and the contact graph
/// grows with N. Scaling housing with the population is the measured fix — with
/// one house per ~4 villagers the touched-relationship exponent falls from 1.929
/// to 0.898 and mean partners/agent goes flat (i339).
///
/// Floored at [`DEFAULT_HOUSE_COUNT`] so small populations keep the historical
/// village exactly: at N ≤ 32 the generated world (and its RNG draw order) is
/// byte-identical to the pre-i340 path, which is why the calibrated N=12 windows
/// and the goldens are untouched by construction.
pub fn houses_for_population(agents: u32) -> u32 {
    agents.div_ceil(4).max(DEFAULT_HOUSE_COUNT)
}

/// i345 (A11): the house count above which the ring layout is replaced by an
/// area packing.
///
/// `ring_span = min(w,h)/2 − 2` is a function of **world size only**, so the
/// ring's angular stride shrinks as houses are added: at 48 houses in the
/// charter's 32×32 world it is `2π·14/48 ≈ 1.8` tiles and i344 measured a
/// minimum pairwise house gap of **1 tile**, 8.4% of house pairs inside the
/// radius-5 perception neighbourhood, and (with i340's housing) **19 agents
/// co-located on one cell against the declared `SiteKind::House.capacity = 4`**.
///
/// 24 is the largest count ever *measured or calibrated* — i339/i340 probed up
/// to N=96 ⇒ `ceil(96/4) = 24` houses — so the legacy ring path stays
/// byte-identical for every calibrated run and the packing governs only
/// unexplored territory (N ≥ 116).
pub const MAX_RING_HOUSE_COUNT: u32 = 24;

/// i360: the number of distinct village centres the large-village layout
/// places houses around.
///
/// i359 measured the fault this rule fixes: i345's single Vogel spiral spreads
/// houses *evenly* over the whole disc, so at town scale the mean inter-house
/// spacing (~9 tiles at N=192/64²) is below any partition threshold and
/// `auto_partition_polities` collapses the whole world into **one settlement** —
/// the i296/i297/i298/i299 multi-setlement machinery has nothing to orchestrate.
///
/// One centre per ~16 houses keeps each settlement's diameter well under the
/// usual partition gap while separating centres by `~2·ring_span/K`, so a large
/// village becomes a **town of K villages**. Capped at 4 so the centres fit on a
/// ring with a generous local radius even in the charter's 32×32 world.
/// Floored at 1: counts ≤ 16 route through the unchanged i345 spiral (byte-
/// identical), so every calibrated run (< 25 houses) is untouched.
pub fn cluster_count_for(houses: u32) -> u32 {
    (houses / 16).clamp(1, 4)
}

/// Generate a small village world with the historical 8 houses.
pub fn generate_village(world: &mut World, rng: &mut RngStreams) {
    generate_village_with_houses(world, rng, DEFAULT_HOUSE_COUNT);
}

/// Generate a small village world with `houses` house sites.
///
/// `houses == DEFAULT_HOUSE_COUNT` is the byte-identical legacy path (same draw
/// order). Other values shift the world RNG stream downstream of the house loop
/// — a larger village, not a re-housed village — which is the honest reading for
/// a spread experiment (see evidence/i339_housing_spread.md).
pub fn generate_village_with_houses(world: &mut World, rng: &mut RngStreams, houses: u32) {
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
    //
    // i339: the ring's RADIUS now spans the map when the count grows (the i338
    // finding is that a fixed radius-4 ring inside a fixed disc is what makes
    // contact saturate); the legacy 8-house case keeps radius ~4 exactly.
    let house_count = houses.max(1);
    let ring_span = (w.min(h) as f64 / 2.0 - 2.0).max(4.0);
    let house_site = |i: u32, id: u64| Site {
        id: EntityId::new(id),
        kind: SiteKind::House,
        name: format!("House {}", i + 1),
        owner: None,
        capacity: 4,
        storage_capacity: Fixed::from_f64(200.0),
        inventory: vec![],
    };
    if house_count <= MAX_RING_HOUSE_COUNT {
        // The historical ring — byte-identical for every calibrated run (same
        // draw order, same radius rule). See MAX_RING_HOUSE_COUNT.
        let ring_base = if house_count <= DEFAULT_HOUSE_COUNT {
            4.0
        } else {
            ring_span
        };
        for i in 0..house_count {
            let angle = (i as f64) * 2.0 * std::f64::consts::PI / house_count as f64
                + world_rng.random_range(-0.15..0.15);
            let radius = ring_base + world_rng.random_range(-1.0..1.0);
            let mut hx = center_x + (angle.cos() * radius) as i32;
            let mut hy = center_y + (angle.sin() * radius) as i32;
            let on_water = world
                .tile(hx, hy)
                .is_some_and(|t| matches!(t.terrain, Terrain::Water));
            if on_water {
                hx = center_x + (angle.cos() * ring_base) as i32;
                hy = center_y + (angle.sin() * ring_base) as i32;
            }
            if place_site(world, hx, hy, house_site(i, site_id)) {
                site_id += 1;
            }
        }
    } else {
        // i345 (A11): a spacing-aware AREA packing for house counts the ring
        // cannot hold. A Vogel/sunflower spiral gives every house the same share
        // of the disc's area (`r ∝ √i`) instead of crowding the rim, so the mean
        // nearest-neighbour spacing is `≈1.7·√(area/n)` — about 6 tiles at 48
        // houses in a 32×32 world, against the ring's measured 1-tile minimum.
        //
        // This branch consumes a different number of World-stream draws than the
        // ring (one jitter per house either way, but the same count) — it is new
        // territory above the calibrated count, not a re-housed historical
        // village, which is the honest reading for a layout the ring cannot do.
        // Placement is deterministic; a candidate that lands on water or on an
        // existing site is nudged along a fixed golden-angle spiral until a free
        // tile is found, so the house count (and therefore `population.rs`'s
        // round-robin) is honoured at every N.
        const GOLDEN_ANGLE: f64 = 2.399_963_229_728_653;
        let clusters = cluster_count_for(house_count);
        if clusters <= 1 {
            for i in 0..house_count {
                let frac = (f64::from(i) + 0.5) / f64::from(house_count);
                let angle = f64::from(i) * GOLDEN_ANGLE + world_rng.random_range(-0.2..0.2);
                let radius = ring_span * frac.sqrt();
                let base_x = center_x + (angle.cos() * radius).round() as i32;
                let base_y = center_y + (angle.sin() * radius).round() as i32;
                if let Some((x, y)) =
                    free_tile_near(world, base_x, base_y, GOLDEN_ANGLE, 96, 0.35, 1.0, &[])
                {
                    if place_site(world, x, y, house_site(i, site_id)) {
                        site_id += 1;
                    }
                }
                // If no free tile was found the house is dropped, as the ring path
                // already does for out-of-bounds candidates; the spiral's coverage
                // makes that unreachable in practice, and `population.rs`
                // round-robins over whatever was placed. The new
                // `large_villages_place_one_house_per_tile` pin guards the count.
            }
        } else {
            // i360: clustered multi-village layout. `clusters` village centres
            // sit evenly on a ring of radius `centre_ring` (analytic — consumes
            // no World draws), and each centre's house share packs onto a LOCAL
            // Vogel spiral of radius `local_r`, so intra-village spacing is
            // small while the villages stand `~2·centre_ring·sin(π/K)` apart.
            // The nudge search is bounded to the local disc so a house can never
            // drift into a neighbouring village and blur the partition.
            let k = clusters as usize;
            let centre_ring = (ring_span * 0.5).max(5.0);
            let local_r = (ring_span / (clusters as f64 * 1.6)).clamp(3.0, 9.0);
            // Reserve the four civic tiles (placed after the houses) so a house
            // can never occupy one and be overwritten — the i344 invariant is one
            // tile per house, and a civic site must not steal a house tile.
            let reserved = [
                (center_x - 6, center_y),
                (center_x, (center_y + 5).min(h - 1)),
                ((center_x + 5).min(w - 1), center_y),
                (center_x, center_y.saturating_sub(5)),
            ];
            for i in 0..house_count {
                let c = (i as usize * k / house_count as usize).min(k - 1);
                let start = c as u32 * house_count / clusters;
                let end = (c as u32 + 1) * house_count / clusters;
                let n_c = (end - start).max(1);
                let idx = i - start;
                let frac = (f64::from(idx) + 0.5) / f64::from(n_c);
                let centre_angle = c as f64 * 2.0 * std::f64::consts::PI / clusters as f64;
                let cx = center_x + (centre_angle.cos() * centre_ring).round() as i32;
                let cy = center_y + (centre_angle.sin() * centre_ring).round() as i32;
                let angle = f64::from(idx) * GOLDEN_ANGLE + world_rng.random_range(-0.2..0.2);
                let radius = local_r * frac.sqrt();
                let base_x = cx + (angle.cos() * radius).round() as i32;
                let base_y = cy + (angle.sin() * radius).round() as i32;
                // Primary: a LOCAL walk that keeps the house inside its village
                // (a wider walk would blur the settlement partition). Fallback:
                // a wide walk so the count is always honoured — i344's invariant
                // (fewer house tiles than houses breaks `population.rs`'s
                // round-robin). The fallback fires only in cramped synthetic
                // worlds (e.g. 48 houses in a 32×32 bound); at density-law
                // sizes the local walk always lands.
                let found = free_tile_near(
                    world,
                    base_x,
                    base_y,
                    GOLDEN_ANGLE,
                    24,
                    0.30,
                    0.6,
                    &reserved,
                )
                .or_else(|| {
                    free_tile_near(
                        world,
                        base_x,
                        base_y,
                        GOLDEN_ANGLE,
                        160,
                        0.6,
                        1.0,
                        &reserved,
                    )
                })
                .or_else(|| nearest_free_tile(world, base_x, base_y, &reserved));
                if let Some((x, y)) = found {
                    if place_site(world, x, y, house_site(i, site_id)) {
                        site_id += 1;
                    }
                }
            }
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

    /// i360: the cluster rule must be inert through the entire calibrated
    /// range (≤ 24 houses → 1 centre) and scale one centre per ~16 houses above
    /// it, capped at 4 so the centres fit even in the charter's 32×32 world.
    #[test]
    fn cluster_count_matches_house_bands() {
        for (houses, expected) in [
            (8u32, 1u32),
            (12, 1),
            (24, 1),
            (31, 1),
            (32, 2),
            (47, 2),
            (48, 3),
            (63, 3),
            (64, 4),
            (200, 4),
        ] {
            assert_eq!(
                cluster_count_for(houses),
                expected,
                "cluster_count_for({houses})"
            );
        }
    }

    #[test]
    fn village_has_food() {
        let mut rng = RngStreams::new(42);
        let mut world = World::new(16, 16);
        generate_village(&mut world, &mut rng);

        let total_food = world.total_food();
        assert!(total_food > Fixed::ZERO);
    }

    /// i344 (A9): the density law reproduces the simulator's own two calibrated
    /// points **exactly** — that anchoring is the whole reason the policy can be
    /// adopted without touching a calibrated corpus, so it is pinned rather than
    /// assumed.
    #[test]
    fn world_side_law_reproduces_both_calibrated_points() {
        assert_eq!(world_side_for_population(12), 16, "N=12 village is 16x16");
        assert_eq!(world_side_for_population(48), 32, "N=48 town tier is 32x32");
        // Both are 21.33 cells/agent: the law's premise, checked at the source.
        let cells = |n: u32| f64::from(world_side_for_population(n)).powi(2) / f64::from(n);
        assert!((cells(12) - CELLS_PER_AGENT).abs() < 1.0);
        assert!((cells(48) - CELLS_PER_AGENT).abs() < 1.0);
    }

    /// The law is monotone in the population and never dips below the smallest
    /// calibrated world — a floor that would otherwise silently shrink a
    /// populated map.
    #[test]
    fn world_side_law_is_monotone_and_floored() {
        let mut previous = 0;
        for n in 1..=512u32 {
            let side = world_side_for_population(n);
            assert!(
                side >= MIN_WORLD_SIDE,
                "side {side} below the floor at N={n}"
            );
            assert!(
                side >= previous,
                "side must not shrink: N={n} gave {side} after {previous}"
            );
            previous = side;
        }
        // The 256-agent cap sits inside the law's range, and the density stays
        // near the calibrated 21.33 rather than drifting (a 74-side world is
        // 5 476 cells / 256 agents = 21.4).
        let capped = world_side_for_population(256);
        assert_eq!(capped, 74);
        let density = f64::from(capped).powi(2) / 256.0;
        assert!(
            (density - CELLS_PER_AGENT).abs() < 1.0,
            "density {density} should track the calibrated {CELLS_PER_AGENT}"
        );
    }
}
