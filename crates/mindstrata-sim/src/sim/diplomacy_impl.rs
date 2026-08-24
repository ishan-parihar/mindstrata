//! Diplomacy ticks, raids and caravans.

use super::{
    AgentId, Fixed, JournalEntryKind, RngStream, Simulation, CARAVAN_GRAIN_BASE,
    EVENT_RELATION_SHIFT, GRAIN_RESOURCE_ID, RAID_GRAIN_CAP, RAID_GRAIN_FRACTION,
};
use crate::military::raid_loss_multiplier;

use rand::Rng;
impl Simulation {
    /// §5 (Iteration 150): One diplomacy pass — mean-revert relations, then
    /// roll a rare event per neighbor on the virgin Economy stream (a raid
    /// when the relation leans hostile, a caravan when it leans friendly).
    /// Idempotent and safe to call from tests; draws from Economy cannot
    /// perturb any other subsystem.
    pub fn tick_diplomacy(&mut self, tick_u64: u64) {
        self.diplomacy.pass_count += 1;
        self.diplomacy.revert_relations();
        let n_neighbors = self.diplomacy.neighbors.len();
        for i in 0..n_neighbors {
            let relation = self.diplomacy.neighbors[i].relation;
            let raid_chance = self.diplomacy.raid_chance(relation);
            let caravan_chance = self.diplomacy.caravan_chance(relation);
            let roll = Fixed::from_f64(self.rng.get_mut(RngStream::Economy).random::<f64>());
            if roll < raid_chance {
                self.apply_raid(i, tick_u64);
            } else if roll < raid_chance + caravan_chance {
                self.apply_caravan(i, tick_u64);
            }
        }
    }

    /// §5 (Iteration 150): A hostile neighbor raids the village — a fraction
    /// of the FARM stocks' grain (capped) is carried off, hostility deepens,
    /// and the ACTUAL loss is journaled. Targeting the farms directly (rather
    /// than `total_food()`, which also counts market/house grain) keeps
    /// journal ≈ reality. Deterministic.
    pub fn apply_raid(&mut self, idx: usize, tick_u64: u64) {
        if idx >= self.diplomacy.neighbors.len() {
            return;
        }
        let name = self.diplomacy.neighbors[idx].name.clone();
        let farm_indices: Vec<usize> = self
            .world
            .sites
            .iter()
            .enumerate()
            .filter(|(_, s)| s.kind == crate::world::SiteKind::Farm)
            .map(|(i, _)| i)
            .collect();
        let farm_grain: Fixed = farm_indices
            .iter()
            .map(|&fi| {
                self.world.sites[fi]
                    .inventory
                    .iter()
                    .find(|st| st.resource_id == GRAIN_RESOURCE_ID)
                    .map_or(Fixed::ZERO, |st| st.quantity)
            })
            .fold(Fixed::ZERO, |a, b| a + b);
        let amount = ((farm_grain * Fixed::from_f64(RAID_GRAIN_FRACTION))
            .min(Fixed::from_f64(RAID_GRAIN_CAP)))
            * raid_loss_multiplier(self.military.readiness);
        let mut lost = Fixed::ZERO;
        if amount > Fixed::ZERO {
            let n = farm_indices.len().max(1);
            let per = amount / Fixed::from_int(n as i64);
            for fi in farm_indices {
                lost += self.world.consume_resource(fi, GRAIN_RESOURCE_ID, per);
            }
        }
        self.diplomacy.neighbors[idx].relation = (self.diplomacy.neighbors[idx].relation
            - Fixed::from_f64(EVENT_RELATION_SHIFT))
        .max(Fixed::from_f64(-1.0));
        self.diplomacy.neighbors[idx].last_raid_tick = Some(tick_u64);
        self.diplomacy.raids += 1;
        self.journal.record(
            tick_u64,
            AgentId::new(0),
            JournalEntryKind::TradeRaid {
                settlement: name,
                grain_lost: lost.to_f64(),
            },
        );
    }

    /// §5 (Iteration 150): A caravan arrives — grain is delivered to the
    /// market (scaled 1.0–2.0× by how friendly the relation is; the market's
    /// grain stock is `AccessRight::Public`, so agents can consume it), the
    /// relation warms, and the event is journaled. If no market exists the
    /// caravan is a no-op (nothing counted, nothing journaled).
    /// Deterministic.
    pub fn apply_caravan(&mut self, idx: usize, tick_u64: u64) {
        if idx >= self.diplomacy.neighbors.len() {
            return;
        }
        let name = self.diplomacy.neighbors[idx].name.clone();
        let relation = self.diplomacy.neighbors[idx].relation.to_f64();
        let scale = Fixed::from_f64(1.0 + f64::midpoint(relation, 1.0));
        let amount = Fixed::from_f64(CARAVAN_GRAIN_BASE) * scale;
        if let Some(market_idx) = self
            .world
            .sites
            .iter()
            .position(|s| s.kind == crate::world::SiteKind::Market)
        {
            self.world
                .produce_resource(market_idx, GRAIN_RESOURCE_ID, amount);
            self.diplomacy.neighbors[idx].relation = (self.diplomacy.neighbors[idx].relation
                + Fixed::from_f64(EVENT_RELATION_SHIFT))
            .min(Fixed::ONE);
            self.diplomacy.neighbors[idx].caravan_count += 1;
            self.diplomacy.caravans += 1;
            self.journal.record(
                tick_u64,
                AgentId::new(0),
                JournalEntryKind::TradeCaravan {
                    settlement: name,
                    grain_gained: amount.to_f64(),
                },
            );
        }
    }
}
