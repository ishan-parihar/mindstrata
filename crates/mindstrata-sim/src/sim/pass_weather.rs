//! Weather pass — extracted verbatim from sim/core.rs block 4a.
//!
//! Pure structural move (Arc-D): the weather advance, regime-gated well
//! drain/recharge, and famine grain drain, unchanged. Golden replay is the
//! referee.

use super::{
    ecology, Fixed, RngStream, Simulation, FAMINE_GRAIN_DRAIN, GRAIN_RESOURCE_ID, WATER_RESOURCE_ID,
};

impl Simulation {
    pub(super) fn weather_pass(&mut self, tick_u64: u64) {
        // ── 4a. Weather: continuous temperature/rainfall + emergent regimes ──
        // The Ecology RNG stream has no other consumer (virgin), so weather's
        // two draws per tick cannot shift any existing stream's position —
        // replays stay byte-identical. Weather is computed before production
        // (growth factor) and spoilage (temperature factor) read it. NB: the
        // season baseline used here rolled at the PREVIOUS tick's block 16
        // (season advances after weather), so weather lags a season boundary
        // by one tick — a deliberate, negligible ordering artifact.
        let weather_event = self
            .weather
            .advance(self.season.current, self.rng.get_mut(RngStream::Ecology));
        if let Some(ev) = weather_event {
            tracing::info!(
                event = ?ev,
                rainfall = self.weather.rainfall.to_f64(),
                temperature = self.weather.temperature.to_f64(),
                "Weather regime change"
            );
        }
        // Regime-gated well water: an emergent drought drains each well's
        // water every tick; an emergent flood recharges it. Normal weather
        // touches nothing — wells only move through consumption and these
        // regimes, so calibrated windows are unaffected by this block.
        match self.weather.regime {
            ecology::WeatherRegime::Drought => {
                let drain = self.weather.config.drought_water_drain;
                for site in &mut self.world.sites {
                    for stock in &mut site.inventory {
                        if stock.resource_id == WATER_RESOURCE_ID && stock.quantity > Fixed::ZERO {
                            stock.quantity =
                                (stock.quantity * (Fixed::ONE - drain)).max(Fixed::ZERO);
                        }
                    }
                }
            }
            ecology::WeatherRegime::Flood => {
                // Iteration 238: suppress flood recharge during the drought
                // shock window — a drought desiccates the aquifer, so even
                // flood weather cannot restore water until the drought
                // pressure window expires. Without this, the weather tracker
                // can enter Flood mode during a drought scenario, recharging
                // water past the shock drain (water goes from 120 back to
                // 643 in 500 ticks, defeating the drought test).
                if tick_u64 >= self.drought_until {
                    let recharge = self.weather.config.flood_water_recharge;
                    for site in &mut self.world.sites {
                        // Cap recharged wells at the site's storage capacity so a
                        // sustained flood cannot balloon water past the §19.5.E
                        // storage contract (the cap only binds during floods,
                        // which never fire in calibrated windows).
                        let cap = site.storage_capacity;
                        for stock in &mut site.inventory {
                            if stock.resource_id == WATER_RESOURCE_ID
                                && stock.quantity > Fixed::ZERO
                            {
                                stock.quantity =
                                    (stock.quantity * (Fixed::ONE + recharge)).min(cap);
                            }
                        }
                    }
                }
            }
            ecology::WeatherRegime::Normal => {
                // Iteration 222: natural aquifer recharge — wells slowly
                // refill from groundwater during normal weather. Without
                // this, wells drain to 0 within ~2K ticks (12 agents ×
                // daily consumption) and never recover (only floods
                // recharged them). The recharge rate is slow (0.05%
                // per tick = ~50% recovery over 1K ticks), so the well
                // acts as a buffer rather than an infinite source.
                // Deterministic, no RNG.
                // Iteration 228: drought-pressure suppresses recharge so
                // a Drought shock's water drain persists for the pressure
                // window (3000 ticks). Without this, aquifer recharge
                // restores water by tick 3000, making drought and vanilla
                // baselines identical.
                // Iteration 232: suppress recharge during drought window
                if tick_u64 >= self.drought_until {
                    let recharge_rate = Fixed::from_f64(0.0005);
                    for site in &mut self.world.sites {
                        // Iteration 237: only Wells receive aquifer recharge.
                        // Non-Well sites (Farms, generic) must not gain water
                        // from groundwater — the isolation-chamber test (and
                        // realism) demands that water at a non-Well site stays
                        // exactly stable absent explicit production.
                        if site.kind != crate::world::SiteKind::Well {
                            continue;
                        }
                        let cap = site.storage_capacity;
                        for stock in &mut site.inventory {
                            if stock.resource_id == WATER_RESOURCE_ID && stock.quantity < cap {
                                let deficit = cap - stock.quantity;
                                let recharge = deficit * recharge_rate;
                                stock.quantity = (stock.quantity + recharge).min(cap);
                            }
                        }
                    }
                }
            }
        }
        // §8.1.4 (P3-6): famine grain drain — while a Famine shock's
        // production-suppression window is open, stored grain additionally
        // decays per tick (a famine consumes the granary: eating outpaces the
        // failed harvest). Mirrors the drought regime's per-tick water drain
        // above. With only the one-shot 70% store drain + production
        // suppression, grain plateaued at ~2.1 (production at 0.3× still
        // matched consumption) and body hunger never rose; the drain drives
        // stored grain to zero mid-window so Eat fails, hunger accumulates
        // toward the 0.5 goal-congruence gate, and the goal-incongruence
        // branch (sadness → despair → depression-from-deprivation) becomes
        // reachable. Outside an open window the factor is 1.0 — calibrated
        // windows (riverford/calm/pestilence/golden) are untouched.
        if tick_u64 < self.famine_until {
            let drain = FAMINE_GRAIN_DRAIN;
            for site in &mut self.world.sites {
                for stock in &mut site.inventory {
                    if stock.resource_id == GRAIN_RESOURCE_ID && stock.quantity > Fixed::ZERO {
                        stock.quantity = (stock.quantity * (Fixed::ONE - drain)).max(Fixed::ZERO);
                    }
                }
            }
        }
    }
}
