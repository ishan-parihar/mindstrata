//! Season/cadence + migration passes — extracted verbatim from sim/core.rs
//! blocks 16/16a and 17c.
//!
//! Pure structural move (Arc-D): season advance, yearly cadence gates
//! (technology/diplomacy/school/theology/military), work-tick bookkeeping,
//! tile fertility dynamics, and scarcity migration pressure, unchanged.

use super::{ecology, ActionKind, Fixed, Simulation};

impl Simulation {
    pub(super) fn ecology_season_pass(&mut self, tick_u64: u64) {
        // ── 16. Ecology: season advance and fertility dynamics ──
        let new_season = self.season.advance();
        if new_season {
            tracing::info!(
                season = self.season.current.name(),
                year = self.season.year,
                "New season"
            );
        }
        // ── 16a. §19.5.I (Iteration 148): Technology discovery — the yearly
        // innovation pass, on the same 4320-tick cadence as the monthly
        // ritual. Calibrated windows (golden @2000, snapshots ≤2000) contain
        // no pass at all; draws come from the virgin Narrative stream, so a
        // pass cannot perturb any other subsystem.
        if tick_u64 > 0 && tick_u64.is_multiple_of(4320) {
            self.tick_technology_discovery(tick_u64);
        }
        // §5 (Iteration 150): the yearly diplomacy pass — off-map neighbors
        // act (caravans / raids) on the same cadence; draws come from the
        // virgin Economy stream, so no other subsystem's RNG can shift.
        if tick_u64 > 0 && tick_u64.is_multiple_of(crate::diplomacy::DIPLOMACY_CADENCE) {
            self.tick_diplomacy(tick_u64);
        }
        // §5 (Iteration 151): the yearly school term — formal cohort
        // instruction, gated on a School site existing. No default world
        // places one, so the pass is a structural no-op in every calibrated
        // window, and it draws no randomness even when a school exists.
        // Both this pass and the per-tick apprenticeship guard idempotently
        // (a `has_learned` check and a `contains` before any push), so
        // whichever runs first in a tick wins and the other skips the
        // student — the same-tick interplay is self-consistent either way.
        if tick_u64 > 0 && tick_u64.is_multiple_of(crate::schools::SCHOOL_CADENCE) {
            self.tick_school_term(tick_u64);
        }
        // §5 (Iteration 152): the half-year religious pass — conversions and
        // conviction drift on the yearly mark, festivals on the mid-year
        // mark. Gated on a seeded Religion (no default world seeds one, so
        // the pass is a structural no-op in every calibrated window) and
        // fully deterministic (no RNG drawn even when a religion exists).
        if tick_u64 > 0 && tick_u64.is_multiple_of(crate::theology::RELIGIOUS_CADENCE) {
            self.tick_theology(tick_u64);
        }
        // §5 (Iteration 153): the yearly military pass — conscription and
        // drills, gated on a Barracks site existing. No default world places
        // one, so the pass is a structural no-op in every calibrated window,
        // and it draws no randomness even when a barracks exists.
        if tick_u64 > 0 && tick_u64.is_multiple_of(crate::military::MILITARY_CADENCE) {
            self.tick_military(tick_u64);
        }
        // Reset work ticks tracker for this tick
        for tick in &mut self.site_work_ticks {
            *tick = 0;
        }
        // Count work actions per site this tick
        for (_agent_idx, action) in &self.tick_action_starts {
            if let ActionKind::Work = action {
                if let Some(farm_idx) = self.world.best_farm_for_work() {
                    if farm_idx < self.site_work_ticks.len() {
                        self.site_work_ticks[farm_idx] += 1;
                    }
                }
            }
        }
        // Apply ecology system to tile fertility
        let mut fertilities: Vec<Fixed> = self.world.tiles.iter().map(|t| t.fertility).collect();
        ecology::system_ecology(
            &self.season,
            &mut fertilities,
            &self.site_work_ticks,
            &self.ecology_config,
        );
        for (i, tile) in self.world.tiles.iter_mut().enumerate() {
            if i < fertilities.len() {
                tile.fertility = fertilities[i];
            }
        }
    }

    /// §Phase 1.5: Ecology migration pressure.
    pub(super) fn ecology_migration_pass(&mut self) {
        // ── 17c. §Phase 1.5: Ecology migration pressure ──
        // Agents under high resource scarcity may migrate to a different site.
        let total_grain = self.world.total_food();
        let local_scarcity = (Fixed::ONE - total_grain).clamp_01();
        let season = self.season.current;
        for i in 0..self.agents.len() {
            let has_shelter = self.agents[i].home_site.is_some();
            let pressure = ecology::migration_pressure(local_scarcity, season, has_shelter);
            if pressure > Fixed::from_f64(0.7) {
                // Agent migrates — teleport to a random site (simplified)
                if let Some(farm_idx) = self.world.best_farm_for_work() {
                    if farm_idx < self.world.sites.len() {
                        let pos = self
                            .world
                            .site_position(farm_idx)
                            .map_or(crate::sim::Position::new(8, 8), |(x, y)| {
                                crate::sim::Position::new(x, y)
                            });
                        self.agents[i].position = pos;
                    }
                }
            }
        }
    }
}
