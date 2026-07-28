//! Simulation orchestrator — the fixed-tick loop.

use crate::actions::{self, ActionKind};
use crate::appraisal::{self, Agency, Appraisal};
use crate::belief_update;
use crate::person::{Affect, BodyState, Belief, DiscreteEmotions, Goal, NeedState, Personality, Relationship};
use crate::scenario::{Scenario, ShockKind};
use crate::social;
use crate::systems::{self, SystemContext};
use crate::world::World;
use crate::world_gen;
use mindstrata_core::clock::{Clock, Tick};
use mindstrata_core::event::SimEvent;
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::{AgentId, EntityId};
use mindstrata_core::rng::RngStreams;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

/// Configuration for a simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    pub seed: u64,
    pub max_ticks: u64,
    pub world_width: u32,
    pub world_height: u32,
    pub num_agents: u32,
    pub snapshot_interval: Option<u64>,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            max_ticks: 1000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: Some(100),
        }
    }
}

/// An agent's full state bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBundle {
    pub body: BodyState,
    pub needs: NeedState,
    pub personality: Personality,
    pub goals: Vec<Goal>,
    pub beliefs: Vec<Belief>,
    pub affect: Affect,
    pub emotions: DiscreteEmotions,
    pub current_action: ActionKind,
    pub action_progress: u32,
    pub name: String,
    pub home_site: Option<usize>,
}

/// The simulation state.
pub struct Simulation {
    config: SimConfig,
    clock: Clock,
    rng: RngStreams,
    pub world: World,
    pub agents: Vec<AgentBundle>,
    pub relationships: Vec<Relationship>,
    events: Vec<SimEvent>,
    scenario: Option<Scenario>,
}

impl Simulation {
    /// Create a new simulation from config.
    pub fn new(config: SimConfig) -> Self {
        let rng = RngStreams::new(config.seed);
        let world = World::new(config.world_width, config.world_height);

        Self {
            config,
            clock: Clock::new(),
            rng,
            world,
            agents: Vec::new(),
            relationships: Vec::new(),
            events: Vec::new(),
            scenario: None,
        }
    }

    /// Create from a scenario definition.
    pub fn from_scenario(scenario: Scenario) -> Self {
        let config = scenario.to_sim_config();
        let mut sim = Self::new(config);
        sim.scenario = Some(scenario);
        sim
    }

    /// Populate the world with terrain, sites, and agents.
    pub fn populate(&mut self) {
        world_gen::generate_village(&mut self.world, &mut self.rng);

        let mut populate_rng =
            rand_chacha::ChaCha8Rng::seed_from_u64(self.config.seed.wrapping_add(1000));

        let names = [
            "Anna", "Bran", "Cara", "Dane", "Elise", "Finn", "Greta", "Hans",
            "Ines", "Jorik", "Kira", "Lars", "Mira", "Nils", "Opal", "Poul",
            "Quinn", "Rosa", "Sven", "Tova", "Ulf", "Vera", "Wulf", "Xena",
        ];

        let house_indices: Vec<usize> = self
            .world
            .sites
            .iter()
            .enumerate()
            .filter(|(_, s)| s.kind == crate::world::SiteKind::House)
            .map(|(i, _)| i)
            .collect();

        for i in 0..self.config.num_agents {
            let body = BodyState::default();
            let needs = NeedState::default();
            let personality = Personality::random(&mut populate_rng);
            let name = names[i as usize % names.len()].to_string();

            let home_site = if house_indices.is_empty() {
                None
            } else {
                Some(house_indices[i as usize % house_indices.len()])
            };

            if let Some(site_idx) = home_site {
                self.world.sites[site_idx].owner =
                    Some(EntityId::new(i as u64));
            }

            let beliefs = vec![
                Belief {
                    proposition_id: 0,
                    confidence: Fixed::from_f64(0.6),
                    emotional_charge: Fixed::ZERO,
                    identity_linkage: Fixed::from_f64(0.3),
                    resistance: Fixed::from_f64(0.5),
                    last_reinforced_tick: 0,
                },
                Belief {
                    proposition_id: 1,
                    confidence: Fixed::from_f64(0.5),
                    emotional_charge: Fixed::ZERO,
                    identity_linkage: Fixed::from_f64(0.2),
                    resistance: Fixed::from_f64(0.4),
                    last_reinforced_tick: 0,
                },
            ];

            self.agents.push(AgentBundle {
                body,
                needs,
                personality,
                goals: Vec::new(),
                beliefs,
                affect: Affect::default(),
                emotions: DiscreteEmotions::default(),
                current_action: ActionKind::Idle,
                action_progress: 0,
                name,
                home_site,
            });
        }

        for i in 0..self.agents.len() {
            for j in 0..self.agents.len() {
                if i != j {
                    let trust = Fixed::from_f64(populate_rng.random_range(0.3..0.7));
                    let affection = Fixed::from_f64(populate_rng.random_range(0.2..0.6));
                    self.relationships.push(Relationship {
                        from: AgentId::new(i as u64),
                        to: AgentId::new(j as u64),
                        trust,
                        affection,
                        respect: Fixed::from_f64(0.3),
                        fear: Fixed::ZERO,
                        obligation: Fixed::ZERO,
                        last_interaction_tick: 0,
                    });
                }
            }
        }

        tracing::info!(
            agents = self.agents.len(),
            sites = self.world.sites.len(),
            relationships = self.relationships.len(),
            "World populated"
        );
    }

    /// Run the simulation for `n` ticks.
    pub fn run(&mut self, n: u64) {
        for _ in 0..n {
            self.tick();
        }
    }

    /// Execute a single simulation tick.
    pub fn tick(&mut self) {
        let tick = self.clock.advance();
        let tick_u64 = tick.as_u64();

        // Snapshot event count before this tick
        let pre_tick_events = self.events.len();

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
                        for site in self.world.sites.iter_mut() {
                            for stock in site.inventory.iter_mut() {
                                if stock.resource_id == 0 {
                                    stock.quantity =
                                        (stock.quantity - shock.magnitude * Fixed::from_f64(10.0))
                                            .clamp_01();
                                }
                            }
                        }
                    }
                    ShockKind::Festival => {
                        for agent in self.agents.iter_mut() {
                            agent.emotions.joy =
                                (agent.emotions.joy + shock.magnitude * Fixed::from_f64(0.3))
                                    .clamp_01();
                        }
                    }
                }
            }
        }

        // Prepare system context
        let mut ctx = SystemContext {
            tick: tick_u64,
            rng: &mut self.rng,
            world: &mut self.world,
            events: &mut self.events,
        };

        // Extract agent data for systems
        let mut bodies: Vec<BodyState> = self.agents.iter().map(|a| a.body.clone()).collect();
        let mut needs: Vec<NeedState> = self.agents.iter().map(|a| a.needs.clone()).collect();
        let personalities: Vec<Personality> =
            self.agents.iter().map(|a| a.personality.clone()).collect();
        let mut goals: Vec<Vec<Goal>> = self.agents.iter().map(|a| a.goals.clone()).collect();
        let mut affects: Vec<Affect> = self.agents.iter().map(|a| a.affect.clone()).collect();
        let mut emotions: Vec<DiscreteEmotions> =
            self.agents.iter().map(|a| a.emotions.clone()).collect();

        // ── 1. Need decay ─────────────────────────────────────────────
        systems::system_need_decay(&mut ctx, &bodies, &mut needs);

        // ── 2. Body state update ──────────────────────────────────────
        systems::system_body_update(&mut ctx, &mut bodies, &needs);

        // ── 3. Goal generation ────────────────────────────────────────
        systems::system_goal_generation(&mut ctx, &personalities, &needs, &mut goals);

        // ── 4. Action execution (per-tick effects) ────────────────────
        for i in 0..self.agents.len() {
            if self.agents[i].action_progress == 0 {
                let action = actions::select_action(
                    &needs[i],
                    &personalities[i],
                    &goals[i],
                    ctx.rng,
                );
                self.agents[i].current_action = action;
                self.agents[i].action_progress = action.definition().duration_ticks;

                let agent_id = AgentId::new(i as u64);
                match action {
                    ActionKind::Eat => {
                        ctx.events.push(SimEvent::AgentAte {
                            agent: agent_id,
                            food: EntityId::new(0),
                            tick,
                        });
                    }
                    ActionKind::Drink => {
                        ctx.events.push(SimEvent::AgentDrank {
                            agent: agent_id,
                            source: EntityId::new(1),
                            tick,
                        });
                    }
                    ActionKind::Rest => {
                        ctx.events.push(SimEvent::AgentRested {
                            agent: agent_id,
                            tick,
                        });
                    }
                    _ => {}
                }
            }

            let action = self.agents[i].current_action;
            actions::apply_action_tick(action, &mut bodies[i], &mut needs[i]);

            self.agents[i].action_progress =
                self.agents[i].action_progress.saturating_sub(1);
        }

        // ── 5. Social interactions ────────────────────────────────────
        {
            let agent_info: Vec<_> = self
                .agents
                .iter()
                .enumerate()
                .map(|(i, a)| {
                    (
                        AgentId::new(i as u64),
                        a.personality.openness,
                        a.personality.agreeableness,
                        a.personality.extraversion,
                    )
                })
                .collect();

            social::system_social_interactions(
                &agent_info,
                &mut self.relationships,
                ctx.events,
                tick,
                ctx.rng,
            );
        }

        // ── 6. Appraisal — emotions from state ────────────────────────
        for i in 0..self.agents.len() {
            let appraisal = Appraisal {
                goal_relevance: needs[i].hunger.max(needs[i].thirst),
                goal_congruence: if needs[i].hunger < Fixed::from_f64(0.5) {
                    Fixed::from_f64(0.3)
                } else {
                    Fixed::from_f64(-0.3)
                },
                coping_potential: self.agents[i].personality.conscientiousness,
                expectedness: Fixed::from_f64(0.5),
                fairness: Fixed::from_f64(0.5),
                agency: Agency::Circumstance,
                social_visibility: Fixed::ZERO,
                identity_relevance: Fixed::from_f64(0.2),
            };

            let delta = appraisal::appraise(&appraisal, ctx.rng, tick);

            emotions[i].fear = (emotions[i].fear + delta.fear).clamp_01();
            emotions[i].anger = (emotions[i].anger + delta.anger).clamp_01();
            emotions[i].joy = (emotions[i].joy + delta.joy).clamp_01();
            emotions[i].sadness = (emotions[i].sadness + delta.sadness).clamp_01();
            emotions[i].trust = (emotions[i].trust + delta.trust).clamp_01();
            emotions[i].shame = (emotions[i].shame + delta.shame).clamp_01();
            emotions[i].pride = (emotions[i].pride + delta.pride).clamp_01();
            emotions[i].guilt = (emotions[i].guilt + delta.guilt).clamp_01();

            affects[i].valence = (emotions[i].joy - emotions[i].sadness - emotions[i].fear
                - emotions[i].anger)
                .clamp_01();
            affects[i].arousal = (emotions[i].fear + emotions[i].anger + emotions[i].joy)
                * Fixed::from_f64(0.5);
        }

        // ── 7. Emotion decay ──────────────────────────────────────────
        for emotion in emotions.iter_mut() {
            let decay = Fixed::from_f64(0.01);
            emotion.fear = (emotion.fear - decay).clamp_01();
            emotion.anger = (emotion.anger - decay).clamp_01();
            emotion.joy = (emotion.joy - decay).clamp_01();
            emotion.sadness = (emotion.sadness - decay).clamp_01();
            emotion.shame = (emotion.shame - decay).clamp_01();
            emotion.pride = (emotion.pride - decay).clamp_01();
            emotion.guilt = (emotion.guilt - decay).clamp_01();
        }

        // ── 8. Belief resistance decay ────────────────────────────────
        for agent_beliefs in self.agents.iter_mut() {
            belief_update::decay_belief_resistance(
                &mut agent_beliefs.beliefs,
                Fixed::from_f64(0.001),
            );
        }

        // ── Write back ────────────────────────────────────────────────
        for (i, agent) in self.agents.iter_mut().enumerate() {
            agent.body = bodies[i].clone();
            agent.needs = needs[i].clone();
            agent.goals = goals[i].clone();
            agent.affect = affects[i].clone();
            agent.emotions = emotions[i].clone();
        }

        // Periodic logging
        if tick_u64.is_multiple_of(100) && tick_u64 > 0 {
            let avg_hunger: f64 = self
                .agents
                .iter()
                .map(|a| a.needs.hunger.to_f64())
                .sum::<f64>()
                / self.agents.len() as f64;
            let avg_joy: f64 = self
                .agents
                .iter()
                .map(|a| a.emotions.joy.to_f64())
                .sum::<f64>()
                / self.agents.len() as f64;
            let avg_valence: f64 = self
                .agents
                .iter()
                .map(|a| a.affect.valence.to_f64())
                .sum::<f64>()
                / self.agents.len() as f64;
            tracing::info!(
                tick = tick_u64,
                day = tick.day(),
                agents = self.agents.len(),
                avg_hunger = format!("{:.3}", avg_hunger),
                avg_joy = format!("{:.3}", avg_joy),
                avg_valence = format!("{:.3}", avg_valence),
                events = self.events.len(),
                "Tick summary"
            );
        }

        // ── 9. Belief updates from this tick's social interactions ────
        // After ctx is dropped, process events added this tick
        for ev in &self.events[pre_tick_events..] {
            if let SimEvent::InteractionOccurred { from, to, .. } = ev {
                let from_idx = from.as_u64() as usize;
                let to_idx = to.as_u64() as usize;
                if from_idx < self.agents.len() && to_idx < self.agents.len() {
                    let trust = self.relationships
                        .iter()
                        .find(|r| r.from == *from && r.to == *to)
                        .map(|r| r.trust)
                        .unwrap_or(Fixed::from_f64(0.5));

                    let evidence_strength = trust - Fixed::from_f64(0.5);
                    let source_trust = Fixed::from_f64(0.6);

                    belief_update::update_beliefs(
                        &mut self.agents[from_idx].beliefs,
                        &[(2, evidence_strength, source_trust)],
                        Fixed::ZERO,
                        Fixed::ZERO,
                        tick_u64,
                    );
                }
            }
        }
    }

    /// Access the current tick.
    pub fn current_tick(&self) -> Tick {
        self.clock.tick()
    }

    /// Number of agents.
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Total events generated.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Get a reference to the world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get recent events (last n).
    pub fn recent_events(&self, n: usize) -> &[SimEvent] {
        let start = self.events.len().saturating_sub(n);
        &self.events[start..]
    }

    /// Get a snapshot of agent summaries.
    pub fn agent_summaries(&self) -> Vec<AgentSummary> {
        self.agents
            .iter()
            .enumerate()
            .map(|(i, a)| AgentSummary {
                index: i,
                name: a.name.clone(),
                hunger: a.needs.hunger,
                thirst: a.needs.thirst,
                fatigue: a.needs.fatigue,
                health: a.body.health,
                energy: a.body.energy,
                valence: a.affect.valence,
                joy: a.emotions.joy,
                fear: a.emotions.fear,
                anger: a.emotions.anger,
                goal_count: a.goals.len(),
                current_action: format!("{:?}", a.current_action),
            })
            .collect()
    }
}

/// A lightweight summary of an agent's state for display.
#[derive(Debug, Clone)]
pub struct AgentSummary {
    pub index: usize,
    pub name: String,
    pub hunger: Fixed,
    pub thirst: Fixed,
    pub fatigue: Fixed,
    pub health: Fixed,
    pub energy: Fixed,
    pub valence: Fixed,
    pub joy: Fixed,
    pub fear: Fixed,
    pub anger: Fixed,
    pub goal_count: usize,
    pub current_action: String,
}
