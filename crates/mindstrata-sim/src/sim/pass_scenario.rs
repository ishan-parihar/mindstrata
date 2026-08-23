//! Scenario-shock pass — extracted verbatim from sim/core.rs.
//!
//! Pure structural move (Arc-D): the per-tick scenario shock dispatch
//! (Drought proportional water drain + drought-pressure window, Famine
//! store drain + suppression window, Pestilence epidemic seeding,
//! Festival joy infusion), unchanged. Golden replay is the referee.

use super::{
    health, DeathCause, Fixed, RngStream, ShockKind, Simulation, FAMINE_WINDOW_TICKS,
    GRAIN_RESOURCE_ID, WATER_RESOURCE_ID,
};
use rand::Rng;

impl Simulation {
    pub(super) fn apply_scenario_shocks(
        &mut self,
        tick_u64: u64,
        tick: mindstrata_core::clock::Tick,
    ) {
        // Check for scenario shocks
        if let Some(ref scenario) = self.scenario {
            if let Some(shock) = scenario.shock_at(tick_u64) {
                tracing::warn!(
                    shock = ?shock.kind,
                    magnitude = shock.magnitude.to_f64(),
                    tick = tick_u64,
                    "Scenario shock applied"
                );
                match shock.kind {
                    ShockKind::Drought => {
                        // A drought dries the water supply (WATER_RESOURCE_ID = 1).
                        // It previously drained grain (resource_id == 0) — a drought
                        // that destroys food instead of water.
                        //
                        // The drain is proportional to the stocked quantity, not a
                        // fixed magnitude × 10: the village well starts with 200
                        // water, so an absolute drain of 3.0 vs 7.0 was a 1.5% vs
                        // 3.5% difference — both drowned out by agent consumption,
                        // making riverford (0.3) and drought (0.7) indistinguishable.
                        // A proportional drain leaves 70% vs 30% of remaining water,
                        // so the shock magnitude now genuinely differentiates.
                        //
                        // NB: no `.clamp_01()` here — a well holding 200 water is
                        // legitimately above 1.0, and clamping to [0,1] would erase
                        // the magnitude difference (60 and 140 both → 1.0). We only
                        // floor at zero to keep quantities non-negative.
                        for site in &mut self.world.sites {
                            for stock in &mut site.inventory {
                                if stock.resource_id == WATER_RESOURCE_ID {
                                    stock.quantity = (stock.quantity
                                        * (Fixed::ONE - shock.magnitude))
                                        .max(Fixed::ZERO);
                                }
                            }
                        }
                        // Iteration 228: drought-pressure suppresses aquifer
                        // recharge for 3000 ticks (~1 year). Without this, the
                        // shock drains 70% of water at tick 500, but aquifer
                        // recharge during Normal weather restores it by tick 3000
                        // — making drought and vanilla baselines identical
                        // (both 38,226 events at 3K).
                        // Iteration 232: set drought window to suppress
                        // aquifer recharge for 3000 ticks.
                        self.drought_until = tick_u64 + 3000;
                    }
                    ShockKind::Famine => {
                        // A famine destroys the grain supply (GRAIN_RESOURCE_ID).
                        // Same proportional-drain semantics as Drought: a fixed
                        // absolute drain would be drowned out by the farm's large
                        // stock, so the magnitude must scale with the stocked
                        // quantity to genuinely differentiate scenarios.
                        for site in &mut self.world.sites {
                            for stock in &mut site.inventory {
                                if stock.resource_id == GRAIN_RESOURCE_ID {
                                    stock.quantity = (stock.quantity
                                        * (Fixed::ONE - shock.magnitude))
                                        .max(Fixed::ZERO);
                                }
                            }
                        }
                        // §8.1.4 (P3-6): a famine is a CROP FAILURE, not a
                        // one-shot store drain. Without production suppression,
                        // Work refilled the destroyed grain within ~1.5K ticks
                        // and body hunger never rose (probe: needs.hunger
                        // 0.006–0.041 in every scenario, famine ≈ calm — the
                        // goal-incongruence branch was unreachable). Open a
                        // production-suppression window (FAMINE_WINDOW_TICKS)
                        // during which Work grain output is scaled by
                        // `1 − magnitude` (0.3 for the standard 0.7 famine):
                        // stored grain genuinely depletes, Eat fails, and
                        // hunger accumulates toward the 0.5 goal-congruence
                        // gate.
                        self.famine_until = tick_u64 + FAMINE_WINDOW_TICKS;
                        // Near-total crop failure: production collapses to
                        // 0.05 × magnitude (3.5% for the standard 0.7 famine).
                        // `1 − magnitude` = 0.3 was NOT enough — the surviving
                        // trickle still yielded ~2 full portions/day and the
                        // village scraped by (probe: top hunger 0.488, never
                        // crossing the 0.5 gate). A famine starves.
                        self.famine_production_factor =
                            shock.magnitude * Fixed::from_f64(0.05).max(Fixed::from_f64(0.01));
                    }
                    ShockKind::Pestilence => {
                        // A pestilence strikes in two composed waves, both routed
                        // through EXISTING machinery rather than fabricated:
                        //  1. IMMEDIATE MORTALITY — a seeded roll with
                        //     p = magnitude × (1 − health × 0.5) is deadly at
                        //     every health level yet deadlier for the weak
                        //     (health 1.0 → 0.35 vs health 0.5 → 0.525 at
                        //     mag 0.7). Deaths flow
                        //     through handle_agent_death — the same path the §31
                        //     demography pass uses — so inheritance, journaling,
                        //     event recording, social cleanup, and in-place
                        //     newborn replacement all stay consistent. This is
                        //     safe here: the shock block runs before the
                        //     SystemContext borrow (self is fully mutable), and
                        //     the dead are replaced IN PLACE so the
                        //     AgentId == index invariant survives.
                        //  2. SURVIVING EPIDEMIC — survivors (and the newborns
                        //     who replaced the dead) carry a virulent Epidemic
                        //     that block 17b spreads by proximity contagion and
                        //     block 17 drains health from over its 800-tick course.
                        // Copy the magnitude first: handle_agent_death borrows
                        // self mutably, so `shock` (borrowing self.scenario) must
                        // not be live across those calls.
                        let magnitude = shock.magnitude;
                        let n = self.agents.len();
                        let mut deaths: Vec<usize> = Vec::new();
                        for i in 0..n {
                            let health = self.agents[i].body.health;
                            let p = magnitude * (Fixed::ONE - health * Fixed::from_f64(0.5));
                            let roll = Fixed::from_f64(
                                self.rng.get_mut(RngStream::Behavior).random::<f64>(),
                            );
                            if roll < p {
                                deaths.push(i);
                            }
                        }
                        tracing::warn!(
                            deaths = deaths.len(),
                            "Pestilence: immediate mortality wave"
                        );
                        // Note: the newborns that replace the dead run the
                        // remainder of this tick's systems (the demography path
                        // kills at tick end instead) — an accepted divergence,
                        // invisible to calibrated runs since no existing
                        // scenario carries a Pestilence shock.
                        for &idx in &deaths {
                            self.handle_agent_death(
                                idx,
                                &deaths,
                                tick_u64,
                                tick,
                                DeathCause::Disease,
                            );
                        }
                        // Iteration 256 (audit Phase 4 — EARNED traumas):
                        // the de-scripting removed the fabricated
                        // pre-seeded drought trauma; this is the earning
                        // path that replaces it. A same-day mass death
                        // writes itself into collective memory as a trauma
                        // — the shared grief that later feeds belief
                        // charging and moral-panic grievance. (Famine
                        // starvation earns its own through the demography
                        // death path — future wiring.)
                        if deaths.len() >= 2 {
                            let village = self.collective_memory_registry.get_or_create(0);
                            village.add_memory(
                                format!(
                                    "The great sickness took {} of ours in a single day",
                                    deaths.len()
                                ),
                                crate::culture::SharedMemoryKind::Trauma,
                                tick_u64,
                                Fixed::from_f64(0.6),
                            );
                        }
                        // Seed the virulent epidemic: every agent rolls an
                        // independent infection check (p = magnitude), so the
                        // selection is index-fair — a plague spares no one.
                        let mut infected = 0usize;
                        for i in 0..n {
                            // Iteration 252: prior-epidemic survivors resist
                            // reinfection by their accumulated immunity.
                            let susceptibility = magnitude
                                * (Fixed::ONE - self.agents[i].embodied.immune.epidemic_immunity);
                            let roll = Fixed::from_f64(
                                self.rng.get_mut(RngStream::Behavior).random::<f64>(),
                            );
                            if roll < susceptibility {
                                let already = self.agent_diseases[i]
                                    .iter()
                                    .any(|d| d.kind == health::DiseaseKind::Epidemic);
                                if !already {
                                    self.agent_diseases[i].push(health::ActiveDisease {
                                        kind: health::DiseaseKind::Epidemic,
                                        ticks_infected: 0,
                                        severity_modifier: Fixed::ONE, // virulent strain
                                    });
                                    infected += 1;
                                }
                            }
                        }
                        tracing::debug!(
                            infected = infected,
                            "Pestilence: epidemic seeded into the population"
                        );
                    }
                    ShockKind::Festival => {
                        for agent in &mut self.agents {
                            agent.emotions.joy = (agent.emotions.joy
                                + shock.magnitude * Fixed::from_f64(0.3))
                            .clamp_01();
                        }
                    }
                }
            }
        }
    }
}
