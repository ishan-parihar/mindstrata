//! Diplomacy — §5 (AP2, Iteration 150).
//!
//! The world is a single village (`generate_village`). This module adds the
//! missing multi-settlement dimensions — *neighboring villages, trade
//! routes, diplomacy, conflict* — as OFF-MAP neighbors: abstract
//! settlements whose caravans and raids reach the village on a rare event
//! cadence, without regenerating the world (which would shatter every
//! calibrated index).
//!
//! - **Neighbors**: `Riverside`, `Millbrook`, `Stonegate` — each with a
//!   relation in [-1 (hostile), 0 (neutral), +1 (friendly)] that
//!   mean-reverts toward neutrality (grudges and friendships cool over
//!   time) and drifts through events.
//! - **Trade routes / diplomacy**: on the yearly pass (every 4320 ticks —
//!   the sim's established ritual cadence), each neighbor rolls a rare
//!   event: a friendly-leaning relation favors a CARAVAN (grain delivered
//!   to the market, relation warms), a hostile-leaning relation favors a
//!   RAID (a fraction of the village's grain carried off, relation
//!   chills). Relations shape the odds; events reshape the relations.
//! - **Conflict**: raids are the conflict vector — a hostile neighbor
//!   raids the granary, costing grain and deepening hostility.
//!
//! Blast contract: the pass runs ONLY on the 4320-tick cadence, so
//! calibrated windows (golden @2000, snapshots ≤2000) contain ZERO passes
//! — byte-identical by construction, and even the pass's draws land on the
//! VIRGIN Economy RNG stream (the only consumer), so they cannot perturb
//! any other subsystem's RNG. Events are rare (≈1% per neighbor per pass
//! at neutrality): the 5000-tick drought probe contains one pass, and any
//! event there hits both the drought and control worlds identically (same
//! seed), preserving its relative differentials. The 10000-tick long-
//! horizon snapshot contains two passes — deterministic regeneration
//! whether or not an event fires.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// The pass cadence — mirrors the technology discovery pass and the ritual.
pub const DIPLOMACY_CADENCE: u64 = 4320;
/// Per-pass mean-reversion step of a neighbor relation toward neutrality.
pub const RELATION_REVERSION: f64 = 0.02;
/// Fraction of the village's total grain a raid carries off (capped below).
pub const RAID_GRAIN_FRACTION: f64 = 0.02;
/// Absolute cap on grain lost to a single raid.
pub const RAID_GRAIN_CAP: f64 = 50.0;
/// Base grain a caravan delivers (scaled 1.0–2.0× by how friendly the
/// relation is).
pub const CARAVAN_GRAIN_BASE: f64 = 15.0;
/// Per-neighbor event base rate per pass at a neutral relation.
pub const EVENT_BASE_RATE: f64 = 0.01;
/// How strongly the relation tilts the event odds (× relation, ±).
pub const EVENT_RELATION_SLOPE: f64 = 0.05;
/// How much a raid/caravan moves the relation (chills/warms).
pub const EVENT_RELATION_SHIFT: f64 = 0.1;

/// A neighboring off-map settlement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborSettlement {
    pub name: String,
    /// -1 (hostile) .. 0 (neutral) .. +1 (friendly).
    pub relation: Fixed,
    pub last_raid_tick: Option<u64>,
    pub caravan_count: u64,
}

/// The off-map settlement network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomacyRegistry {
    pub neighbors: Vec<NeighborSettlement>,
    pub raids: u64,
    pub caravans: u64,
    pub pass_count: u64,
}

impl Default for DiplomacyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DiplomacyRegistry {
    #[must_use]
    pub fn new() -> Self {
        let names = ["Riverside", "Millbrook", "Stonegate"];
        Self {
            neighbors: names
                .iter()
                .map(|n| NeighborSettlement {
                    name: (*n).into(),
                    relation: Fixed::ZERO,
                    last_raid_tick: None,
                    caravan_count: 0,
                })
                .collect(),
            raids: 0,
            caravans: 0,
            pass_count: 0,
        }
    }

    /// Per-pass raid odds for a neighbor — hostile relations court raids.
    #[must_use]
    pub fn raid_chance(&self, relation: Fixed) -> Fixed {
        let c = EVENT_BASE_RATE - relation.to_f64() * EVENT_RELATION_SLOPE;
        Fixed::from_f64(c.clamp(0.0, 1.0))
    }

    /// Per-pass caravan odds — friendly relations attract caravans.
    #[must_use]
    pub fn caravan_chance(&self, relation: Fixed) -> Fixed {
        let c = EVENT_BASE_RATE + relation.to_f64() * EVENT_RELATION_SLOPE;
        Fixed::from_f64(c.clamp(0.0, 1.0))
    }

    /// Cool grudges and friendships alike — relations mean-revert toward
    /// neutrality so no neighbor permanently locks into hostility or
    /// friendship without fresh events.
    pub fn revert_relations(&mut self) {
        for n in &mut self.neighbors {
            let r = n.relation.to_f64();
            let next = if r > 0.0 {
                (r - RELATION_REVERSION).max(0.0)
            } else {
                (r + RELATION_REVERSION).min(0.0)
            };
            n.relation = Fixed::from_f64(next);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_neighbors_start_neutral() {
        let reg = DiplomacyRegistry::new();
        assert_eq!(reg.neighbors.len(), 3);
        assert!(reg.neighbors.iter().all(|n| n.relation == Fixed::ZERO));
        assert_eq!(reg.raids, 0);
        assert_eq!(reg.caravans, 0);
        assert_eq!(reg.pass_count, 0);
    }

    #[test]
    fn raid_chance_rises_with_hostility() {
        let reg = DiplomacyRegistry::new();
        let hostile = reg.raid_chance(Fixed::from_f64(-0.8));
        let neutral = reg.raid_chance(Fixed::ZERO);
        let friendly = reg.raid_chance(Fixed::from_f64(0.8));
        assert!(hostile > neutral, "hostile neighbors raid more");
        assert!(neutral > friendly, "friendly neighbors raid less");
        assert!(neutral.to_f64() > 0.0, "a neutral neighbor can still raid");
        // Fully friendly → no raids at all.
        assert_eq!(reg.raid_chance(Fixed::ONE), Fixed::ZERO);
    }

    #[test]
    fn caravan_chance_rises_with_friendship() {
        let reg = DiplomacyRegistry::new();
        let friendly = reg.caravan_chance(Fixed::from_f64(0.8));
        let neutral = reg.caravan_chance(Fixed::ZERO);
        let hostile = reg.caravan_chance(Fixed::from_f64(-0.8));
        assert!(friendly > neutral, "friendly neighbors attract caravans");
        assert!(
            neutral > hostile,
            "hostile neighbors attract fewer caravans"
        );
        assert_eq!(reg.caravan_chance(Fixed::from_f64(-1.0)), Fixed::ZERO);
    }

    #[test]
    fn relations_mean_revert_toward_neutrality() {
        let mut reg = DiplomacyRegistry::new();
        reg.neighbors[0].relation = Fixed::from_f64(0.8);
        reg.neighbors[1].relation = Fixed::from_f64(-0.6);
        reg.revert_relations();
        assert!(
            reg.neighbors[0].relation < Fixed::from_f64(0.8)
                && reg.neighbors[0].relation > Fixed::ZERO,
            "friendship cools toward neutral"
        );
        assert!(
            reg.neighbors[1].relation > Fixed::from_f64(-0.6)
                && reg.neighbors[1].relation < Fixed::ZERO,
            "hostility cools toward neutral"
        );
        assert_eq!(
            reg.neighbors[2].relation,
            Fixed::ZERO,
            "neutral stays neutral"
        );
        // Repeated reversion drives toward zero but never past it.
        for _ in 0..100 {
            reg.revert_relations();
        }
        assert_eq!(reg.neighbors[0].relation, Fixed::ZERO);
        assert_eq!(reg.neighbors[1].relation, Fixed::ZERO);
    }
}
