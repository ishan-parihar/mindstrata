//! Tick pass 0: biological substrate update.
//! Arc-D verbatim move from sim/pass_biology.rs (golden-referee pure refactor).

use crate::actions::ActionKind;
use crate::sim::{AgentBundle, Simulation, HORMONAL_TRACE_THRESHOLD};
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;

impl Simulation {
    pub(crate) fn tick_biology_pass(
        agents: &mut [AgentBundle],
        emotions: &[crate::person::DiscreteEmotions],
        params: &crate::parameters::SimParameters,
        weather: &crate::ecology::WeatherTracker,
        provenance: &mut crate::provenance::CausalProvenance,
        tick_u64: u64,
        world_food_total: Fixed,
        world_water_total: Fixed,
    ) {
        // ── 0. Biological update (architecture-plan-2 §7) ──────────
        // Tick the rich biological substrate before cognitive processing.
        // Uses previous tick's emotions — biology reacts to current felt state.
        // EmbodiedState feeds endocrine/nervous signals into the legacy BodyState.

        // Iteration 248 (Arc B — Whitehall gradient, G3): the village-mean
        // effective status anchors the hierarchy→biology loop. Computed
        // once per pass (deterministic, RNG-free); agents below the mean
        // accumulate chronic stress faster, agents above recover faster —
        // applied after `EmbodiedState::tick_update` so it composes with
        // (not replaces) the graded endocrine dynamics from Iter-189/223.
        let mean_status_f = if agents.is_empty() {
            0.5
        } else {
            agents
                .iter()
                .map(|a| a.status_v2.effective_status().to_f64())
                .sum::<f64>()
                / agents.len() as f64
        };

        for i in 0..agents.len() {
            let threat_level = emotions[i].fear + emotions[i].anger;
            let social_safety = Fixed::ONE - threat_level;
            let is_sleeping = matches!(agents[i].current_action, ActionKind::Rest);
            // Compute activity level from current action
            let raw_activity = match agents[i].current_action {
                ActionKind::Work => Fixed::from_f64(0.8),
                ActionKind::Wander | ActionKind::Move { .. } => Fixed::from_f64(0.4),
                ActionKind::Trade => Fixed::from_f64(0.3),
                ActionKind::Socialize | ActionKind::Worship => Fixed::from_f64(0.2),
                _ => Fixed::from_f64(0.1),
            };
            // §7.2.6 (Iteration 215): respiratory endurance_modifier
            // from the PREVIOUS tick dampens current activity — an agent
            // with poor lung capacity (smoke/cold/disease) cannot sustain
            // high activity. At endurance 0.8 (healthy default) the
            // dampening is mild (×0.9); at endurance 0.3 (chronic
            // respiratory disease) it halves activity (×0.65). Floor at
            // 0.3 prevents total paralysis. Deterministic, no RNG.
            let activity_level = (raw_activity
                * (Fixed::from_f64(0.3)
                    + agents[i].embodied.respiratory.endurance_modifier * Fixed::from_f64(0.7)))
            .clamp_01();
            // §5 (S3-2-1 fix): the biology pass used a HARDCODED
            // "temperate default" 0.5 here, which froze the thermal
            // system at thermoneutral in every scenario (probe: body_temp
            // 0.500, cold/heat stress 0, no spread — the weather layer
            // was live but never reached the body). The WeatherTracker
            // temperature (0..1; seasonal baselines Spring 0.6 / Summer
            // 0.9 / Autumn 0.5 / Winter 0.2, mean-reverting + seeded
            // noise) is now the ambient input, so body temperature tracks
            // the seasons — mild heat in Spring, real cold in Winter —
            // the plan's "winter hardship" (§7.3.3) becomes live.
            // NB: this block (biology pass) runs before weather.advance
            // in the same tick, so it reads the PREVIOUS tick's weather
            // state — the same one-tick lag the weather site documents
            // for season boundaries; deterministic, seeded, RNG-free
            // here. Golden-safe: nothing golden-hashed reads thermal;
            // the respiratory consumer receives cold_stress (≈0 in
            // Spring/Summer/Autumn) via irritation.
            let ambient_temperature = weather.temperature;
            // §7.2.6 (Iteration 213): crowding and hygiene now derive
            // from world state instead of hardcoded 0.3/0.6.
            // Crowding: more agents → higher crowding (disease spreads
            // faster). Anchored at 0.3 for 24 agents, scales linearly.
            let agent_ratio = Fixed::from_int(agents.len() as i64) / Fixed::from_int(24);
            let crowding = (Fixed::from_f64(0.3) * agent_ratio)
                .clamp(Fixed::from_f64(0.15), Fixed::from_f64(0.6));
            // Hygiene: water scarcity degrades hygiene (less water →
            // poorer sanitation → disease spreads faster). Anchored
            // at 0.6 for abundant water, drops toward 0.3 in drought.
            let expected_water = crate::sim::EXPECTED_WATER_PER_AGENT as i64 * agents.len() as i64;
            let water_ratio = if expected_water > 0 {
                (world_water_total / Fixed::from_int(expected_water))
                    .clamp(Fixed::from_f64(0.5), Fixed::ONE)
            } else {
                Fixed::ONE
            };
            let hygiene = Fixed::from_f64(0.3) + water_ratio * Fixed::from_f64(0.4);
            // §7.2.2 (Iteration 188 — the S2-2-2 hunger/thirst channel):
            // the endocrine acute-stress term now reads the LIVE needs
            // (the same values the Eat/Drink decisions use) instead of
            // the frozen embodied facade fields — famine starvation and
            // chronic dehydration genuinely elevate cortisol.
            let live_hunger = agents[i].needs.hunger;
            let live_thirst = agents[i].needs.thirst;
            // §7.2.9 (Iteration 210): nutrition quality derives from
            // world food abundance instead of a hardcoded 0.6 placeholder.
            // Ratio: world_food / expected_food, clamped [0.1, 1.0].
            // Floor 0.1 prevents total starvation from disabling all
            // biology gains (agents still degrade, just slower). Ceiling
            // 1.0 is natural from the ratio. Deterministic, no RNG.
            let expected_food = crate::sim::EXPECTED_GRAIN_PER_AGENT as i64 * agents.len() as i64;
            let nutrition_quality = if expected_food > 0 {
                (world_food_total / Fixed::from_int(expected_food))
                    .clamp(Fixed::from_f64(0.1), Fixed::ONE)
            } else {
                Fixed::from_f64(0.6)
            };
            // §7.2.6 (Iteration 214): smoke_exposure derives from
            // ambient temperature — cold weather means more indoor
            // fires for heating/cooking, increasing smoke exposure.
            // Smoke scale: 0 in summer (temp ≈ 0.9) to 0.6 in winter
            // (temp ≈ 0.2). Formula: (1 − temp) × 0.8, clamped [0, 0.6].
            let smoke_exposure = ((Fixed::ONE - ambient_temperature) * Fixed::from_f64(0.8))
                .clamp(Fixed::ZERO, Fixed::from_f64(0.6));
            // §7.2.6 (Iteration 214): damp_housing derives from
            // rainfall — heavy rain makes houses damper, promoting
            // respiratory irritation and mold-related illness.
            // Damp scale: 0 in drought (rainfall ≈ 0.1) to 0.5 in
            // flood (rainfall ≈ 0.9). Formula: rainfall × 0.55, clamped.
            let damp_housing =
                (weather.rainfall * Fixed::from_f64(0.55)).clamp(Fixed::ZERO, Fixed::from_f64(0.5));
            agents[i].embodied.tick_update(
                threat_level,
                social_safety,
                is_sleeping,
                activity_level,
                ambient_temperature,
                crowding,
                hygiene,
                live_hunger,
                live_thirst,
                nutrition_quality,
                smoke_exposure,
                damp_housing,
                params,
            );
            // Iteration 248 (Arc B — Whitehall gradient): hierarchy
            // position feeds chronic-stress load — the plan's
            // "status → chronic stress → health" loop (G3). Sub-mean
            // status accelerates chronic accumulation; above-mean status
            // earns a slower recovery bonus. Computed in f64 and
            // quantized ONCE per direction (Fixed-4 truncation disease:
            // the raw products sit at ~1e-5/tick). An agent AT the
            // village mean gets exactly zero — midpoint neutrality.
            let status_gap_f = mean_status_f - agents[i].status_v2.effective_status().to_f64();
            let load = agents[i].embodied.endocrine.stress.chronic_load.to_f64();
            if status_gap_f > 0.0 {
                let add = 0.002 * status_gap_f * (1.0 - load);
                agents[i].embodied.endocrine.stress.chronic_load =
                    mindstrata_core::fixed::Fixed::from_f64((load + add).min(1.0));
            } else if status_gap_f < 0.0 {
                let relief = 0.001 * (-status_gap_f) * load;
                agents[i].embodied.endocrine.stress.chronic_load =
                    mindstrata_core::fixed::Fixed::from_f64((load - relief).max(0.0));
            }
            // Sync derived body fields from EmbodiedState back to legacy BodyState.
            // Compute values first to avoid borrow conflicts between embodied and body.
            // Boundary: hunger/thirst/sickness/injury are managed by health.rs and
            // routines.rs, not synced from embodied — they operate on separate tracks.
            let derived_health = agents[i].embodied.derived_health();
            let derived_energy = agents[i].embodied.derived_energy();
            let derived_fatigue = agents[i].embodied.derived_fatigue();
            agents[i].body.health = derived_health;
            agents[i].body.energy = derived_energy;
            agents[i].body.fatigue = derived_fatigue;

            // §16.2: Record hormonal provenance when stress axis level exceeds threshold.
            // This traces cross-system causal influence from biology into psychology.
            let cortisol = agents[i].embodied.endocrine.stress.level;
            if cortisol > HORMONAL_TRACE_THRESHOLD {
                provenance.record_system(crate::provenance::SystemTrace {
                    agent: AgentId::new(i as u64),
                    tick: tick_u64,
                    category: crate::provenance::ProvenanceCategory::Hormonal,
                    description: format!(
                        "Stress axis level ({}) influencing threat appraisal",
                        cortisol.to_f64()
                    ),
                    magnitude: cortisol,
                    cause: "biological_update".into(),
                });
            }
        }
    }
}
