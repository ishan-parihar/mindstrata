//! Simulation orchestrator — the fixed-tick loop.

use crate::actions::{self, ActionKind};
use crate::appraisal::{self, Agency, Appraisal};
use crate::belief_update;
use crate::journal::{EventJournal, JournalEntryKind};
use crate::memory::{MemoryKind, MemoryStore, MemoryTag};
use crate::norms::{self, NormRegistry};
use crate::attention::AttentionState;
use crate::faction::Faction;
use crate::institutions::{self, Institution};
pub use crate::attention;
pub use crate::person::Intention;
use crate::person::{Affect, BodyState, Belief, DiscreteEmotions, Goal, GoalKind, IdentityKind, IdentityState, NeedState, Personality, Relationship};
use crate::scenario::{Scenario, ShockKind};
use crate::social;
use crate::systems::{self, SystemContext};
use crate::world::{GRAIN_RESOURCE_ID, WATER_RESOURCE_ID, World};
use crate::world_gen;
use mindstrata_core::clock::{Clock, Tick};
use mindstrata_core::event::SimEvent;
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::{AgentId, EntityId};
use mindstrata_core::rng::{RngStream, RngStreams};
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
    pub memory: MemoryStore,
    pub identity: IdentityState,
    /// §22.5: Attention controls which events the agent notices.
    pub attention: AttentionState,
    /// §24.5: Current intention — a committed plan to achieve a goal.
    pub intention: Option<crate::person::Intention>,
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
    journal: EventJournal,
    norms: NormRegistry,
    /// §12: Institutions are first-class entities with legitimacy, roles, and collective psychology.
    pub institutions: Vec<Institution>,
    /// §26: Factions are emergent political groups with collective identity.
    pub factions: Vec<Faction>,
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
            journal: EventJournal::new(),
            norms: NormRegistry::new(),
            institutions: Vec::new(),
            factions: Vec::new(),
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
                memory: MemoryStore::new(200),
                identity: IdentityState {
                    identities: vec![
                        crate::person::AgentIdentity {
                            kind: IdentityKind::Farmer,
                            strength: Fixed::from_f64(populate_rng.random_range(0.3..0.8)),
                        },
                        crate::person::AgentIdentity {
                            kind: IdentityKind::Parent,
                            strength: Fixed::from_f64(populate_rng.random_range(0.0..0.5)),
                        },
                        crate::person::AgentIdentity {
                            kind: IdentityKind::Believer,
                            strength: Fixed::from_f64(populate_rng.random_range(0.2..0.7)),
                        },
                    ],
                },
                attention: AttentionState::default(),
                intention: None,
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

        // Register default norms
        for norm in norms::default_norms() {
            self.norms.register(norm);
        }

        // §12: Create default institutions
        self.institutions = institutions::default_institutions();

        // §12: Assign agents to institutional roles based on personality traits
        // Elder: high conscientiousness + high agreeableness
        // Priest: high traditionalism + high agreeableness
        if self.agents.len() >= 3 {
            // Find best Elder candidate (highest conscientiousness + agreeableness)
            let elder_candidate = self.agents.iter().enumerate()
                .max_by_key(|(_, a)| {
                    let score = a.personality.conscientiousness + a.personality.agreeableness;
                    score.to_raw() as u64
                })
                .map(|(i, _)| i);

            // Find best Priest candidate (highest traditionalism + agreeableness, not already elder)
            let priest_candidate = self.agents.iter().enumerate()
                .filter(|(i, _)| Some(*i) != elder_candidate)
                .max_by_key(|(_, a)| {
                    let score = a.personality.traditionalism + a.personality.agreeableness;
                    score.to_raw() as u64
                })
                .map(|(i, _)| i);

            if let Some(elder_idx) = elder_candidate {
                if let Some(council) = self.institutions.iter_mut().find(|i| i.kind == institutions::InstitutionKind::Council) {
                    council.assign_role("Elder", AgentId::new(elder_idx as u64));
                    council.add_member(AgentId::new(elder_idx as u64));
                    // Add a second member (next highest score)
                    let second = self.agents.iter().enumerate()
                        .filter(|(i, _)| *i != elder_idx)
                        .max_by_key(|(_, a)| (a.personality.conscientiousness + a.personality.agreeableness).to_raw() as u64)
                        .map(|(i, _)| i);
                    if let Some(second_idx) = second {
                        council.add_member(AgentId::new(second_idx as u64));
                    }
                }
            }

            if let Some(priest_idx) = priest_candidate {
                if let Some(temple) = self.institutions.iter_mut().find(|i| i.kind == institutions::InstitutionKind::Temple) {
                    temple.assign_role("Priest", AgentId::new(priest_idx as u64));
                    temple.add_member(AgentId::new(priest_idx as u64));
                }
            }
        }

        tracing::info!(
            agents = self.agents.len(),
            sites = self.world.sites.len(),
            relationships = self.relationships.len(),
            norms = self.norms.norms().len(),
            institutions = self.institutions.len(),
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

        // Capture event count before systems run (before SystemContext borrows self.events)
        let pre_tick_events = self.events.len();

        // Collect action starts to defer resource operations outside the ctx block
        let mut action_starts: Vec<(usize, ActionKind)> = Vec::new();

        // Prepare system context and run systems that need mutable borrow on events/rng
        {
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

            // ── 1. Need decay ─────────────────────────────────────────
            tracing::debug!(tick = tick_u64, "systems::need_decay");
            systems::system_need_decay(&mut ctx, &bodies, &mut needs);

            // ── 2. Body state update ──────────────────────────────────
            tracing::debug!(tick = tick_u64, "systems::body_update");
            systems::system_body_update(&mut ctx, &mut bodies, &needs);

            // ── 3. Goal generation ────────────────────────────────────
            tracing::debug!(tick = tick_u64, "systems::goal_generation");
            systems::system_goal_generation(&mut ctx, &personalities, &needs, &mut goals);

            // ── 4. Action execution (per-tick effects) ────────────────
            for i in 0..self.agents.len() {
                // §24.5: Intention commitment — check if current intention should be abandoned
                if let Some(ref intention) = self.agents[i].intention {
                    if intention.completed {
                        self.agents[i].intention = None;
                    } else {
                        let stress = emotions[i].fear + emotions[i].anger;
                        let emotional_shock = emotions[i].anger > Fixed::from_f64(0.5) || emotions[i].fear > Fixed::from_f64(0.5);
                        if intention.should_abandon(tick_u64, stress, emotional_shock) {
                            self.agents[i].intention = None;
                            self.agents[i].action_progress = 0; // force reselection
                        }
                    }
                }

                // Action interruption: force address critical needs (> 0.9)
                if self.agents[i].action_progress > 0 {
                    let critical = needs[i].hunger > Fixed::from_f64(0.9)
                        || needs[i].thirst > Fixed::from_f64(0.9)
                        || needs[i].fatigue > Fixed::from_f64(0.95);
                    if critical {
                        self.agents[i].action_progress = 0;
                        self.agents[i].intention = None; // critical needs override intentions
                    }
                }

                if self.agents[i].action_progress == 0 {
                    let total_grain = ctx.world.total_food();
                    let total_water = ctx.world.total_water();
                    // Compute norm pressure using the registry's formula (identity-aware)
                    let conformity = personalities[i].conformity;
                    let avg_identity_strength = self.agents[i].identity.strength_of(IdentityKind::Farmer)
                        .max(self.agents[i].identity.strength_of(IdentityKind::Parent))
                        .max(self.agents[i].identity.strength_of(IdentityKind::Believer));
                    // §12: Institution enforcement amplifies norm pressure.
                    // Council enforcement capacity multiplies the pressure felt by agents.
                    let enforcement_multiplier = self.institutions.iter()
                        .filter(|i| i.kind == institutions::InstitutionKind::Council)
                        .map(|i| Fixed::ONE + i.enforcement_capacity * Fixed::from_f64(0.5))
                        .fold(Fixed::ONE, |a, b| a.max(b));
                    let norm_pressure = self.norms
                        .norms()
                        .iter()
                        .map(|n| self.norms.compute_pressure(conformity, avg_identity_strength, n.id))
                        .fold(Fixed::ZERO, |a, b| a + b) * enforcement_multiplier; // Negative = compliant, positive = violating
                    let action = actions::select_action(
                        &needs[i],
                        &personalities[i],
                        &goals[i],
                        ctx.rng,
                        total_grain,
                        total_water,
                        &self.agents[i].identity,
                        norm_pressure,
                    );
                    self.agents[i].current_action = action;
                    self.agents[i].action_progress = action.definition().duration_ticks;
                    action_starts.push((i, action));

                    // §24.5: Create intention for the selected action's goal
                    let goal_kind = match action {
                        ActionKind::Eat => GoalKind::Eat,
                        ActionKind::Drink => GoalKind::Drink,
                        ActionKind::Rest => GoalKind::Rest,
                        ActionKind::Work => GoalKind::Work,
                        ActionKind::Socialize => GoalKind::Socialize,
                        ActionKind::Worship => GoalKind::Worship,
                        _ => GoalKind::Eat, // fallback
                    };
                    self.agents[i].intention = Some(
                        crate::person::Intention::new(goal_kind, tick_u64, personalities[i].conscientiousness)
                    );

                    // Push events (no resource ops yet — those happen after ctx drops)
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

            // ── 5. Social interactions ────────────────────────────────
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

            // ── 6. Appraisal — emotions from state ────────────────────
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

            // ── 7. Emotion decay ──────────────────────────────────────
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

            // ── 8. Belief resistance decay ────────────────────────────
            for agent_beliefs in self.agents.iter_mut() {
                belief_update::decay_belief_resistance(
                    &mut agent_beliefs.beliefs,
                    Fixed::from_f64(0.001),
                );
            }

            // ── Write back ────────────────────────────────────────────
            for (i, agent) in self.agents.iter_mut().enumerate() {
                agent.body = bodies[i].clone();
                agent.needs = needs[i].clone();
                agent.goals = goals[i].clone();
                agent.affect = affects[i].clone();
                agent.emotions = emotions[i].clone();
            }

        } // ctx is dropped here, releasing mutable borrows on self.events and self.rng

        // ── 4b. Resource operations, journal recording, intention tracking ──
        for (agent_idx, action) in &action_starts {
            let agent_id = AgentId::new(*agent_idx as u64);
            // §24.5: Track intention success/failure when action completes.
            // The resource operation outcome (e.g., Eat failing because no grain)
            // is known here, so we record intention success/failure based on
            // the actual resource operation result.
            let action_succeeded = match action {
                ActionKind::Eat => {
                    let amount = Fixed::from_f64(0.1);
                    if let Some(farm_idx) = self.world.farm_with_grain() {
                        let taken = self.world.consume_resource(farm_idx, GRAIN_RESOURCE_ID, amount);
                        self.journal.record(tick_u64, agent_id, JournalEntryKind::Consumed { resource: "grain".into(), amount: taken.to_f64() });
                        taken > Fixed::ZERO
                    } else {
                        false
                    }
                }
                ActionKind::Drink => {
                    let amount = Fixed::from_f64(0.15);
                    if let Some(well_idx) = self.world.well_with_water() {
                        let taken = self.world.consume_resource(well_idx, WATER_RESOURCE_ID, amount);
                        self.journal.record(tick_u64, agent_id, JournalEntryKind::Consumed { resource: "water".into(), amount: taken.to_f64() });
                        taken > Fixed::ZERO
                    } else {
                        false
                    }
                }
                ActionKind::Work => {
                    let productivity = self.agents[*agent_idx].personality.conscientiousness * Fixed::from_f64(0.05);
                    if let Some(farm_idx) = self.world.best_farm_for_work() {
                        self.world.produce_resource(farm_idx, GRAIN_RESOURCE_ID, productivity);
                        self.journal.record(tick_u64, agent_id, JournalEntryKind::Worked { productivity: productivity.to_f64() });
                        true
                    } else {
                        false
                    }
                }
                ActionKind::Rest => {
                    self.journal.record(tick_u64, agent_id, JournalEntryKind::Rested);
                    true
                }
                ActionKind::Worship => {
                    self.journal.record(tick_u64, agent_id, JournalEntryKind::Worshiped);
                    true
                }
                _ => false,
            };

            // Record intention success/failure based on actual resource outcome
            if let Some(ref mut intention) = self.agents[*agent_idx].intention {
                if action_succeeded {
                    intention.record_success();
                } else {
                    intention.record_failure();
                }
            }
        }

        // ── 9. Memory encoding from this tick's events ───────────────
        // §22.5: Use attention system to compute salience instead of hardcoded values.
        // Only events that pass the attention threshold are encoded into memory.
        for (i, agent) in self.agents.iter_mut().enumerate() {
            for ev in &self.events[pre_tick_events..] {
                // §22.5: Attention computes salience based on intensity, novelty, relevance
                let salience = agent.attention.compute_salience(
                    ev,
                    AgentId::new(i as u64),
                    &agent.needs,
                    &agent.affect,
                );

                // Only encode events that exceed the attention threshold
                if salience < Fixed::from_f64(0.2) {
                    continue;
                }

                // Emotional intensity scales with arousal
                let emotional = agent.affect.arousal * Fixed::from_f64(0.6) + Fixed::from_f64(0.1);
                match ev {
                    SimEvent::AgentAte { agent: a, .. } if a.as_u64() == i as u64 => {
                        agent.memory.encode(MemoryKind::Consumption, tick_u64, salience, emotional, None, MemoryTag::AteFood);
                    }
                    SimEvent::AgentDrank { agent: a, .. } if a.as_u64() == i as u64 => {
                        agent.memory.encode(MemoryKind::Consumption, tick_u64, salience, emotional, None, MemoryTag::DrankWater);
                    }
                    SimEvent::AgentRested { agent: a, .. } if a.as_u64() == i as u64 => {
                        agent.memory.encode(MemoryKind::Positive, tick_u64, salience, emotional, None, MemoryTag::Rested);
                    }
                    SimEvent::InteractionOccurred { from, to, kind, .. } => {
                        if from.as_u64() == i as u64 {
                            let tag = match kind {
                                mindstrata_core::event::InteractionKind::Help => MemoryTag::HelpedBy,
                                mindstrata_core::event::InteractionKind::Threaten => MemoryTag::ThreatenedBy,
                                mindstrata_core::event::InteractionKind::Insult => MemoryTag::InsultedBy,
                                mindstrata_core::event::InteractionKind::Gossip => MemoryTag::GossipedAbout,
                                mindstrata_core::event::InteractionKind::Trade => MemoryTag::TradedWith,
                                _ => MemoryTag::TalkedTo,
                            };
                            let kind = match kind {
                                mindstrata_core::event::InteractionKind::Threaten | mindstrata_core::event::InteractionKind::Insult => MemoryKind::Negative,
                                mindstrata_core::event::InteractionKind::Help | mindstrata_core::event::InteractionKind::Comfort => MemoryKind::Positive,
                                _ => MemoryKind::Social,
                            };
                            agent.memory.encode(kind, tick_u64, salience, emotional, Some(to.as_u64() as u32), tag);
                        }
                        if to.as_u64() == i as u64 {
                            agent.memory.encode(MemoryKind::Social, tick_u64, salience, emotional, Some(from.as_u64() as u32), MemoryTag::TalkedTo);
                        }
                    }
                    _ => {}
                }
            }
            // Probabilistic memory rehearsal (every 10 ticks)
            if tick_u64.is_multiple_of(10) {
                agent.memory.rehearse_random(self.rng.get_mut(RngStream::Behavior));
            }
            // Decay memories
            agent.memory.decay(tick_u64);

            // §22.5: Attention maintenance — decay habituation, replenish budget
            agent.attention.decay_habituation(Fixed::from_f64(0.005));
            let stress = agent.emotions.fear + agent.emotions.anger;
            agent.attention.replenish_budget(stress, agent.needs.fatigue);
        }

        // ── 10. Resource spoilage (perishable goods degrade each tick) ──
        // Extract resources ref before mutable borrow of sites (split borrowing)
        let resource_defs = &self.world.resources;
        for site in self.world.sites.iter_mut() {
            for stock in site.inventory.iter_mut() {
                if let Some(res_def) = resource_defs.iter().find(|r| r.id == stock.resource_id) {
                    if res_def.perishable && res_def.spoilage_rate > Fixed::ZERO {
                        let spoilage = stock.quantity * res_def.spoilage_rate;
                        stock.quantity = (stock.quantity - spoilage).max(Fixed::ZERO);
                    }
                }
            }
        }

        // ── 11. Norm evaluation: threats and insults are norm violations ──
        for ev in &self.events[pre_tick_events..].to_vec() {
            if let SimEvent::InteractionOccurred {
                from,
                to,
                kind: mindstrata_core::event::InteractionKind::Threaten
                | mindstrata_core::event::InteractionKind::Insult,
                ..
            } = ev
            {
                let from_id = *from;
                let to_id = *to;
                let punishment = self.norms.check_violation(1, from_id, tick_u64);                    if punishment > Fixed::ZERO {
                        if let Some(rel) = self
                            .relationships
                            .iter_mut()
                            .find(|r| r.from == to_id && r.to == from_id)
                        {
                            rel.trust =
                                (rel.trust - punishment * Fixed::from_f64(0.15)).max(Fixed::ZERO);
                        }
                        let from_idx = from_id.as_u64() as usize;
                        if from_idx < self.agents.len() {
                            self.agents[from_idx].emotions.shame = (self.agents[from_idx]
                                .emotions
                                .shame
                                + punishment * Fixed::from_f64(0.10))
                                .clamp_01();
                        }
                        // §12.4: Successful norm enforcement increases institutional legitimacy
                        for inst in self.institutions.iter_mut() {
                            if inst.kind == institutions::InstitutionKind::Council {
                                inst.increase_legitimacy(Fixed::from_f64(0.005));
                            }
                        }
                    }
            }
        }

        // ── 11b. Gossip propagation: when agents gossip, share a random belief ──
        for ev in &self.events[pre_tick_events..].to_vec() {
            if let SimEvent::InteractionOccurred {
                from,
                to,
                kind: mindstrata_core::event::InteractionKind::Gossip,
                ..
            } = ev
            {
                let from_idx = from.as_u64() as usize;
                let to_idx = to.as_u64() as usize;
                if from_idx < self.agents.len() && to_idx < self.agents.len() {
                    // Extract data from source to avoid borrow conflicts
                    let shared = {
                        let from_beliefs = &self.agents[from_idx].beliefs;
                        if from_beliefs.is_empty() {
                            None
                        } else {
                            let pick = self.rng.get_mut(RngStream::Social).random_range(0..from_beliefs.len());
                            let b = &from_beliefs[pick];
                            let source_trust = self.relationships
                                .iter()
                                .find(|r| r.from == *from && r.to == *to)
                                .map(|r| r.trust)
                                .unwrap_or(Fixed::from_f64(0.5));
                            // Use discrete emotions for stronger emotional distortion
                            let anger_bias = self.agents[from_idx].emotions.anger * Fixed::from_f64(0.2);
                            let joy_bias = self.agents[from_idx].emotions.joy * Fixed::from_f64(0.15);
                            let emotional_bias = joy_bias - anger_bias;
                            let mutated_confidence = (b.confidence * source_trust + emotional_bias).clamp_01();
                            Some((b.proposition_id, mutated_confidence, b.emotional_charge, b.identity_linkage, b.resistance))
                        }
                    };
                    // Now apply to listener (immutable borrow on source is dropped)
                    if let Some((prop_id, mutated_conf, e_charge, i_linkage, res)) = shared {
                        // Weight by resistance: high-resistance beliefs barely change
                        let weight = Fixed::ONE - res;
                        if let Some(existing) = self.agents[to_idx].beliefs.iter_mut()
                            .find(|b| b.proposition_id == prop_id)
                        {
                            existing.confidence = (existing.confidence * res + mutated_conf * weight).clamp_01();
                            existing.last_reinforced_tick = tick_u64;
                        } else {
                            self.agents[to_idx].beliefs.push(Belief {
                                proposition_id: prop_id,
                                confidence: mutated_conf,
                                emotional_charge: e_charge,
                                identity_linkage: i_linkage,
                                resistance: res,
                                last_reinforced_tick: tick_u64,
                            });
                        }
                    }
                }
            }
        }

        // ── 12. Institutional collective psychology derivation ────
        // §23: Derive collective psychology from member states each tick.
        for institution in self.institutions.iter_mut() {
            let member_morales: Vec<Fixed> = institution
                .members
                .iter()
                .filter_map(|m| {
                    let idx = m.as_u64() as usize;
                    if idx < self.agents.len() {
                        Some(self.agents[idx].affect.valence)
                    } else {
                        None
                    }
                })
                .collect();
            let member_trusts: Vec<Fixed> = institution
                .members
                .iter()
                .filter_map(|m| {
                    let idx = m.as_u64() as usize;
                    if idx < self.agents.len() {
                        Some(self.agents[idx].emotions.trust)
                    } else {
                        None
                    }
                })
                .collect();
            institution.derive_collective_psychology(&member_morales, &member_trusts);
            // Slow legitimacy decay without reinforcement
            institution.decay_legitimacy(Fixed::from_f64(0.0001));
        }

        // ── 13. Faction mobilization derivation ────
        for faction in self.factions.iter_mut() {
            faction.derive_mobilization();
        }

        // ── 14. Belief updates from this tick's social interactions ────
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
                attention_budget: a.attention.budget,
                has_intention: a.intention.is_some(),
            })
            .collect()
    }

    /// Get a reference to the event journal.
    pub fn journal(&self) -> &EventJournal {
        &self.journal
    }

    /// Get journal entry count.
    pub fn journal_len(&self) -> usize {
        self.journal.len()
    }

    /// Get total grain across all farms.
    pub fn total_grain(&self) -> Fixed {
        self.world.total_food()
    }

    /// Get total water across all wells.
    pub fn total_water(&self) -> Fixed {
        self.world.total_water()
    }

    /// Get a reference to the norm registry.
    pub fn norms(&self) -> &NormRegistry {
        &self.norms
    }

    /// Capture a snapshot of key metrics at the current tick.
    pub fn metrics_snapshot(&self) -> MetricsSnapshot {
        let summaries = self.agent_summaries();
        let n = summaries.len() as f64;
        MetricsSnapshot {
            tick: self.current_tick().as_u64(),
            avg_hunger: summaries.iter().map(|s| s.hunger.to_f64()).sum::<f64>() / n,
            avg_thirst: summaries.iter().map(|s| s.thirst.to_f64()).sum::<f64>() / n,
            avg_fatigue: summaries.iter().map(|s| s.fatigue.to_f64()).sum::<f64>() / n,
            avg_valence: summaries.iter().map(|s| s.valence.to_f64()).sum::<f64>() / n,
            avg_joy: summaries.iter().map(|s| s.joy.to_f64()).sum::<f64>() / n,
            avg_fear: summaries.iter().map(|s| s.fear.to_f64()).sum::<f64>() / n,
            total_grain: self.total_grain().to_f64(),
            total_water: self.total_water().to_f64(),
            event_count: self.event_count() as u64,
            journal_len: self.journal_len() as u64,
            agent_count: self.agent_count() as u64,
        }
    }
}

/// A snapshot of key simulation metrics at a specific tick.
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub tick: u64,
    pub avg_hunger: f64,
    pub avg_thirst: f64,
    pub avg_fatigue: f64,
    pub avg_valence: f64,
    pub avg_joy: f64,
    pub avg_fear: f64,
    pub total_grain: f64,
    pub total_water: f64,
    pub event_count: u64,
    pub journal_len: u64,
    pub agent_count: u64,
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
    pub attention_budget: Fixed,
    pub has_intention: bool,
}
