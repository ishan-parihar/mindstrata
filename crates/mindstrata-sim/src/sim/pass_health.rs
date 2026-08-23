//! Health/disease pass — extracted verbatim from sim/core.rs blocks
//! 17 / 17b / 17b-2.
//!
//! Pure structural move (Arc-D): disease effects + Iter-252 immunity wall,
//! proximity contagion with same-kind dedup, and the seasonal Cold/Fever
//! vector, unchanged.

use super::{
    health, Fixed, RngStream, Simulation, COLD_SEASON_RATE, COLD_SEASON_TEMP, FEVER_ESCALATION_RATE,
};
use crate::scheduler::TickPhases;
use rand::Rng;

impl Simulation {
    pub(super) fn health_disease_pass(&mut self, tick_u64: u64, phases: TickPhases) {
        // ── 17. Health: disease effects on agents ──
        {
            let n = self.agents.len();
            let decay_day = tick_u64.is_multiple_of(24); // daily cadence
            for i in 0..n {
                let hunger = self.agents[i].needs.hunger;
                let fatigue = self.agents[i].needs.fatigue;
                let body = &mut self.agents[i].body;
                // Iteration 252: hazard-accumulated immunity — a completed
                // Epidemic course grants lasting resistance that decays on
                // a generational scale (~0.0005/day ≈ 48K-tick half-life).
                let had_epidemic = self.agent_diseases[i]
                    .iter()
                    .any(|d| d.kind == health::DiseaseKind::Epidemic);
                health::system_health(
                    &mut body.health,
                    &mut body.energy,
                    &mut self.agent_diseases[i],
                    hunger,
                    fatigue,
                    &self.health_config,
                );
                let cleared_epidemic = had_epidemic
                    && !self.agent_diseases[i]
                        .iter()
                        .any(|d| d.kind == health::DiseaseKind::Epidemic);
                if cleared_epidemic {
                    self.agents[i].embodied.immune.epidemic_immunity = Fixed::ONE;
                } else if decay_day {
                    // Sub-resolution per-day decrement computed in f64 and
                    // quantized once (Fixed-4 truncation disease).
                    let imm = self.agents[i].embodied.immune.epidemic_immunity.to_f64();
                    self.agents[i].embodied.immune.epidemic_immunity =
                        Fixed::from_f64((imm - 0.0005).max(0.0));
                }
            }
        }

        // ── 17b. §7.5: Epidemic disease spread — proximity-based contagion ──
        // Contagious diseases (Cold, Fever, Epidemic) spread to nearby agents.
        // Uses Manhattan distance ≤ 2 as "close contact" radius.
        //
        // Iteration 185 (P5 100K audit): same-kind dedup. The old code
        // checked `already_infected` only against the PRE-TICK disease vecs;
        // `new_infections` is applied after the whole scan, so within ONE
        // tick every source's every disease could push the same (j, kind)
        // repeatedly — a freshly-recovered agent absorbed pushes from all
        // sources' all diseases at once, and the nested per-disease loops
        // multiplied the pile (probe: 100K pestilence seed 42 carried
        // 809,303 concurrent Epidemic entries across 12 agents — 8,396 on
        // one agent alone — and per-tick cost grew with the pile, blowing
        // up the run). Dedup against the pending infections caps each
        // agent at ONE new infection per kind per tick; combined with the
        // already_infected cross-tick guard that caps concurrent
        // same-kind diseases at 1, so the vecs stay bounded and the
        // contagion cost linear. Deterministic (Vec preserves push order;
        // the dedup check runs before the RNG draw, so the draw sequence
        // is a deterministic function of the new state).
        {
            let n = self.agents.len();
            let mut new_infections: Vec<(usize, health::DiseaseKind)> = Vec::new();
            for i in 0..n {
                if self.agent_diseases[i].is_empty() {
                    continue;
                }
                for disease in &self.agent_diseases[i] {
                    let rate = disease.kind.transmission_rate();
                    if rate <= Fixed::ZERO {
                        continue; // not contagious
                    }
                    // Check proximity to all other agents
                    for j in 0..n {
                        if i == j || j >= self.agents.len() {
                            continue;
                        }
                        let dist = self.agents[i]
                            .position
                            .manhattan_distance(&self.agents[j].position);
                        if dist > 2 {
                            continue; // too far
                        }
                        // Already infected with this kind (pre-tick state)?
                        let already_infected = self.agent_diseases[j]
                            .iter()
                            .any(|d| d.kind == disease.kind);
                        if already_infected {
                            continue;
                        }
                        // Iteration 185: also skip if a same-kind infection is
                        // already pending for j THIS tick (see the block
                        // comment above).
                        if new_infections
                            .iter()
                            .any(|(aj, ak)| *aj == j && *ak == disease.kind)
                        {
                            continue;
                        }
                        // Probability check — reduced by distance, target
                        // health, and (Iteration 252) the target's accumulated
                        // epidemic immunity: immune contacts are dead ends.
                        let distance_factor = Fixed::from_f64(1.0)
                            - Fixed::from_f64(dist as f64) * Fixed::from_f64(0.25);
                        let health_factor = self.agents[j].body.health;
                        let immunity = self.agents[j].embodied.immune.epidemic_immunity;
                        let transmission_prob = rate
                            * distance_factor
                            * (Fixed::ONE - health_factor * Fixed::from_f64(0.5))
                            * (Fixed::ONE - immunity);
                        let roll =
                            Fixed::from_f64(self.rng.get_mut(RngStream::Social).random::<f64>());
                        if roll < transmission_prob {
                            new_infections.push((j, disease.kind));
                        }
                    }
                }
            }
            // Apply new infections
            for (agent_idx, kind) in new_infections {
                if agent_idx < self.agent_diseases.len() {
                    self.agent_diseases[agent_idx].push(health::ActiveDisease::new(kind));
                    tracing::debug!(
                        agent = agent_idx,
                        disease = ?kind,
                        "Epidemic: disease transmitted"
                    );
                }
            }
        }

        // ── 17b-2. §32 seasonal Cold/Fever vector (Iteration 187) ──
        // The disease system's Cold/Fever kinds were unreachable in real
        // runs (defined + unit-tested, never seeded — audit FINDING 3). This
        // daily pass makes them seasonal: when the weather is cold
        // (temperature < 0.4, the Winter band), each susceptible agent rolls
        // a small chance to catch a Cold, and a Cold can escalate into a
        // Fever (an untreated cold's secondary infection). Deterministic
        // (the seeded Social stream), sparse (daily gate + tiny rates: at
        // Winter temperature 0.2, ~0.2%/day per agent → ~17%/agent/season),
        // and same-kind dedup-guarded like the epidemic pass. Cold/Fever
        // resolve through the normal `system_health` duration path, so the
        // per-kind cap (one of each kind) holds.
        if phases.is_daily {
            let n = self.agents.len();
            let cold_season =
                (Fixed::from_f64(COLD_SEASON_TEMP) - self.weather.temperature).clamp_01();
            if cold_season > Fixed::ZERO {
                for i in 0..n {
                    if self.agent_diseases[i].len() >= health::MAX_CONCURRENT_DISEASES {
                        continue;
                    }
                    let has_cold = self.agent_diseases[i]
                        .iter()
                        .any(|d| d.kind == health::DiseaseKind::Cold);
                    let has_fever = self.agent_diseases[i]
                        .iter()
                        .any(|d| d.kind == health::DiseaseKind::Fever);
                    if !has_cold {
                        let cold_roll =
                            Fixed::from_f64(self.rng.get_mut(RngStream::Social).random::<f64>());
                        if cold_roll < COLD_SEASON_RATE * cold_season {
                            self.agent_diseases[i]
                                .push(health::ActiveDisease::new(health::DiseaseKind::Cold));
                            tracing::debug!(agent = i, "Seasonal cold contracted");
                        }
                    } else if !has_fever {
                        let fever_roll =
                            Fixed::from_f64(self.rng.get_mut(RngStream::Social).random::<f64>());
                        if fever_roll < FEVER_ESCALATION_RATE {
                            self.agent_diseases[i]
                                .push(health::ActiveDisease::new(health::DiseaseKind::Fever));
                            tracing::debug!(agent = i, "Cold escalated into fever");
                        }
                    }
                }
            }
        }
    }
}
