//! Simulation orchestrator — the fixed-tick loop.

use crate::actions::{self, ActionKind};
use crate::appraisal::{self, Agency, Appraisal};
use crate::belief_update;
use crate::journal::{EventJournal, JournalEntryKind};
use crate::memory::{MemoryKind, MemoryStore, MemoryTag};
use crate::norms::{self, NormRegistry};
use crate::attention::AttentionState;
use crate::factions;
use crate::institutions::{self, Institution, InstitutionKind};
use crate::routines::DailyRoutine;
pub use crate::attention;
pub use crate::person::Intention;
use crate::person::{Affect, BodyState, Belief, CognitiveState, DerivedMentalState, DiscreteEmotions, Goal, GoalKind, IdentityKind, IdentityState, MoralValues, NeedState, Personality, Relationship};
use crate::provenance::{CausalProvenance, DecisionFactor, DecisionTrace};
use crate::scenario::{Scenario, ShockKind};
use crate::snapshot::Snapshot;
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
    /// §10.3: Daily routine creating behavioral stability.
    pub routine: DailyRoutine,
    /// §22.1: Moral foundations affecting norm compliance and emotional appraisal.
    pub moral_values: MoralValues,
    /// §22.1: Bounded rationality — stress, fatigue reduce planning and increase heuristic reliance.
    pub cognitive: CognitiveState,
    /// §22: Derived mental states — trauma, depression, ambition, resilience, resentment.
    pub derived: DerivedMentalState,
    /// Track success rate for derived mental state computation.
    pub recent_successes: u32,
    pub recent_attempts: u32,
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
    /// §34: Causal provenance — tracks decision traces and event causality chains.
    provenance: CausalProvenance,
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
            provenance: CausalProvenance::new(),
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
                routine: DailyRoutine::village_routine(),
                moral_values: MoralValues::random(&mut populate_rng),
                cognitive: CognitiveState::default(),
                derived: DerivedMentalState::default(),
                recent_successes: 0,
                recent_attempts: 0,
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

            // ── 0. Cognitive state update (§22.1) ────────────────────
            // §22.1: Stress reduces planning horizon, increases heuristic bias.
            // This must happen before action selection to affect decision-making.
            for i in 0..self.agents.len() {
                let stress = emotions[i].fear + emotions[i].anger;
                let need_fatigue = needs[i].fatigue.max(needs[i].hunger).max(needs[i].thirst);
                self.agents[i].cognitive.update(stress, need_fatigue);
            }

            // ── 1. Need decay (nonlinear pressure, §9.1) ────────────────
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
                // Track causal provenance flags per agent per tick
                let mut was_interrupted_by_critical_needs = false;
                let mut intention_abandoned_this_tick = false;

                // §24.5: Intention commitment — check if current intention should be abandoned
                if let Some(ref intention) = self.agents[i].intention {
                    if intention.completed {
                        self.agents[i].intention = None;
                        intention_abandoned_this_tick = true;
                    } else {
                        let stress = emotions[i].fear + emotions[i].anger;
                        let emotional_shock = emotions[i].anger > Fixed::from_f64(0.5) || emotions[i].fear > Fixed::from_f64(0.5);
                        if intention.should_abandon(tick_u64, stress, emotional_shock) {
                            self.agents[i].intention = None;
                            self.agents[i].action_progress = 0; // force reselection
                            intention_abandoned_this_tick = true;
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
                        was_interrupted_by_critical_needs = true;
                        intention_abandoned_this_tick = true;
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

                    // §22.1: Moral values modulate norm pressure.
                    // High fairness/authority → stronger norm compliance.
                    // High liberty → weaker norm compliance.
                    let moral_modifier = self.agents[i].moral_values.fairness
                        * Fixed::from_f64(0.15)
                        + self.agents[i].moral_values.authority * Fixed::from_f64(0.1)
                        - self.agents[i].moral_values.liberty * Fixed::from_f64(0.08);
                    let norm_pressure = self.norms
                        .norms()
                        .iter()
                        .map(|n| self.norms.compute_pressure(conformity, avg_identity_strength, n.id))
                        .fold(Fixed::ZERO, |a, b| a + b) * enforcement_multiplier
                        - moral_modifier;

                    // §10.3: Routine bias — agents follow daily schedules when stable.
                    // §24: Personality modulates routine adherence — conscientiousness
                    // increases it, openness decreases it.
                    let (routine_action, routine_strength) = self.agents[i].routine.preferred_action(tick);
                    let stress = emotions[i].fear + emotions[i].anger;
                    let follow_routine = self.agents[i].routine.should_follow(
                        needs[i].hunger, needs[i].thirst, needs[i].fatigue, stress
                    );
                    // Personality-modulated routine strength: conscientiousness boosts, openness reduces
                    let personality_routine_modifier = personalities[i].conscientiousness
                        * Fixed::from_f64(0.3)
                        - personalities[i].openness * Fixed::from_f64(0.2);
                    let effective_routine_strength = (routine_strength + personality_routine_modifier).clamp_01();

                    // §22.1: When stressed, agents favor habit/routine over utility.
                    // High heuristic bias → agents follow routine more readily.
                    let effective_routine_threshold = if self.agents[i].cognitive.heuristic_bias > Fixed::from_f64(0.5) {
                        Fixed::from_f64(0.3) // stressed agents follow routine at lower threshold
                    } else {
                        Fixed::from_f64(0.5) // normal agents need stronger routine signal
                    };

                    let action = if follow_routine && effective_routine_strength > effective_routine_threshold {
                        // §10.3: Routine creates behavioral stability — prefer scheduled action
                        routine_action
                    } else {
                        // §12.3: Institution collective morale modulates norm compliance.
                        // Members of institutions with high morale are more norm-compliant.
                        // Note: norm_pressure is negative = compliant, positive = violating.
                        let mut adjusted_pressure = norm_pressure;
                        for inst in &self.institutions {
                            if inst.has_member(AgentId::new(i as u64)) {
                                // High morale → more compliant (pressure becomes more negative)
                                let morale_bonus = inst.collective.morale * Fixed::from_f64(0.1);
                                adjusted_pressure -= morale_bonus;
                            }
                        }
                        actions::select_action(
                            &needs[i],
                            &personalities[i],
                            &goals[i],
                            ctx.rng,
                            total_grain,
                            total_water,
                            &self.agents[i].identity,
                            adjusted_pressure,
                        )
                    };
                    self.agents[i].current_action = action;
                    self.agents[i].action_progress = action.definition().duration_ticks;
                    action_starts.push((i, action));

                    let agent_id = AgentId::new(i as u64);

                    // §34: Record decision trace for causal provenance
                    {
                        let mut factors = Vec::new();
                        factors.push(DecisionFactor {
                            kind: "need_hunger".into(),
                            magnitude: needs[i].hunger,
                            description: format!("Hunger: {:.2}", needs[i].hunger.to_f64()),
                        });
                        factors.push(DecisionFactor {
                            kind: "need_thirst".into(),
                            magnitude: needs[i].thirst,
                            description: format!("Thirst: {:.2}", needs[i].thirst.to_f64()),
                        });
                        factors.push(DecisionFactor {
                            kind: "need_fatigue".into(),
                            magnitude: needs[i].fatigue,
                            description: format!("Fatigue: {:.2}", needs[i].fatigue.to_f64()),
                        });
                        factors.push(DecisionFactor {
                            kind: "norm_pressure".into(),
                            magnitude: norm_pressure,
                            description: format!("Norm pressure: {:.2}", norm_pressure.to_f64()),
                        });
                        if follow_routine && effective_routine_strength > Fixed::from_f64(0.5) {
                            factors.push(DecisionFactor {
                                kind: "routine".into(),
                                magnitude: effective_routine_strength,
                                description: format!("Routine strength: {:.2}", effective_routine_strength.to_f64()),
                            });
                        }
                        self.provenance.record_decision(DecisionTrace {
                            agent: agent_id,
                            tick: tick_u64,
                            action_name: format!("{:?}", action),
                            factors,
                            from_routine: follow_routine && effective_routine_strength > Fixed::from_f64(0.5),
                            interrupted_by_critical_needs: was_interrupted_by_critical_needs,
                            intention_abandoned: intention_abandoned_this_tick,
                        });
                    }

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
                    // §19.5.E: Use access-checking method to enforce resource access rights.
                    if let Some(farm_idx) = self.world.accessible_farm_with_grain(agent_id) {
                        let taken = self.world.consume_resource(farm_idx, GRAIN_RESOURCE_ID, amount);
                        self.journal.record(tick_u64, agent_id, JournalEntryKind::Consumed { resource: "grain".into(), amount: taken.to_f64() });
                        taken > Fixed::ZERO
                    } else {
                        false
                    }
                }
                ActionKind::Drink => {
                    let amount = Fixed::from_f64(0.15);
                    // §19.5.E: Use access-checking method to enforce resource access rights.
                    if let Some(well_idx) = self.world.accessible_well_with_water(agent_id) {
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
            // §22: Track success rate for derived mental state computation
            self.agents[*agent_idx].recent_attempts += 1;
            if action_succeeded {
                self.agents[*agent_idx].recent_successes += 1;
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
                            // §22.1: Moral outrage modulated by moral foundations.
                            // High care/fairness → more shame from own violations.
                            let outrage = self.agents[from_idx].moral_values.moral_outrage(punishment);
                            self.agents[from_idx].emotions.shame = (self.agents[from_idx]
                                .emotions
                                .shame
                                + outrage * Fixed::from_f64(0.10))
                                .clamp_01();
                        }
                        // §12.4: Successful norm enforcement increases institutional legitimacy
                        // Modulated by enforcement_capacity — low-capacity councils gain less.
                        for inst in self.institutions.iter_mut() {
                            if inst.kind == institutions::InstitutionKind::Council {
                                let legitimacy_gain = inst.enforcement_capacity * Fixed::from_f64(0.005);
                                inst.increase_legitimacy(legitimacy_gain);
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
        }        // ── 15. Faction dynamics — grievance, formation, recruitment, protests (Phase 8) ──
        // §29.2: Factions emerge from collective grievance. §Phase 8: legitimacy crisis → rebellion or reform.
        {
            let faction_count = self.institutions.iter()
                .filter(|i| i.kind == InstitutionKind::Faction)
                .count();

            // Compute grievance for each agent
            let grievances: Vec<(AgentId, Fixed)> = self.agents.iter().enumerate()
                .map(|(i, agent)| {
                    let g = factions::compute_grievance(
                        agent.derived.resentment,
                        agent.emotions.fear,
                        agent.emotions.anger,
                        agent.needs.autonomy,
                        agent.needs.meaning,
                        agent.moral_values.fairness,
                    );
                    (AgentId::new(i as u64), g)
                })
                .collect();

            // Find council legitimacy (average across councils)
            let avg_council_legitimacy: Fixed = {
                let councils: Vec<_> = self.institutions.iter()
                    .filter(|i| i.kind == InstitutionKind::Council)
                    .collect();
                if councils.is_empty() {
                    Fixed::from_f64(0.5)
                } else {
                    let sum: Fixed = councils.iter().map(|c| c.legitimacy).fold(Fixed::ZERO, |a, b| a + b);
                    sum / Fixed::from_int(councils.len() as i64)
                }
            };

            // Faction formation: when grievance is high and council legitimacy is low
            let should_form = faction_count < factions::MAX_FACTIONS
                && avg_council_legitimacy < Fixed::from_f64(0.5)
                && tick_u64 > factions::FORMATION_COOLDOWN;

            if should_form {
                let recruitable = factions::find_recruitable_agents(
                    &grievances, factions::RECRUITMENT_GRIEVANCE_THRESHOLD,
                );

                let avg_grievance = if recruitable.is_empty() {
                    Fixed::ZERO
                } else {
                    let sum: Fixed = recruitable.iter().map(|(_, g)| *g).fold(Fixed::ZERO, |a, b| a + b);
                    sum / Fixed::from_int(recruitable.len() as i64)
                };

                if avg_grievance >= factions::FORMATION_GRIEVANCE_THRESHOLD && recruitable.len() >= 2 {
                    // Find leader candidates: high ambition, high dominance, low anger
                    let leader_candidates: Vec<_> = recruitable.iter()
                        .filter_map(|(id, _)| {
                            let idx = id.as_u64() as usize;
                            if idx < self.agents.len() {
                                let a = &self.agents[idx];
                                if a.personality.ambition >= factions::LEADER_AMBITION_THRESHOLD {
                                    Some((*id, a.personality.ambition, a.personality.dominance,
                                          a.personality.extraversion, a.emotions.anger))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();

                    if let Some(leader) = factions::select_leader(&leader_candidates) {
                        let members: Vec<AgentId> = recruitable.iter()
                            .map(|(id, _)| *id)
                            .collect();

                        let new_faction = factions::create_faction(
                            1000 + faction_count as u64,
                            leader,
                            members.clone(),
                            avg_grievance,
                            tick_u64,
                        );

                        tracing::warn!(
                            leader = leader.as_u64(),
                            members = members.len(),
                            grievance = avg_grievance.to_f64(),
                            tick = tick_u64,
                            "Faction formed"
                        );

                        // Record in provenance
                        self.provenance.record_decision(crate::provenance::DecisionTrace {
                            agent: leader,
                            tick: tick_u64,
                            action_name: "FactionFormation".into(),
                            factors: vec![
                                crate::provenance::DecisionFactor {
                                    kind: "grievance".into(),
                                    magnitude: avg_grievance,
                                    description: format!("Avg grievance: {:.2}", avg_grievance.to_f64()),
                                },
                                crate::provenance::DecisionFactor {
                                    kind: "council_legitimacy".into(),
                                    magnitude: avg_council_legitimacy,
                                    description: format!("Council legitimacy: {:.2}", avg_council_legitimacy.to_f64()),
                                },
                            ],
                            from_routine: false,
                            interrupted_by_critical_needs: false,
                            intention_abandoned: false,
                        });

                        self.institutions.push(new_faction);
                    }
                }
            }

            // Faction dynamics: check for protests from existing factions
            for inst_idx in 0..self.institutions.len() {
                if self.institutions[inst_idx].kind != InstitutionKind::Faction {
                    continue;
                }

                let faction_grievance = self.institutions[inst_idx].collective.morale;
                let faction_cohesion = self.institutions[inst_idx].collective.unity;

                if factions::should_protest(faction_grievance, faction_cohesion, tick_u64) {
                    let protest_size = self.institutions[inst_idx].members.len();
                    let total_pop = self.agents.len();

                    tracing::warn!(
                        faction = self.institutions[inst_idx].name.as_str(),
                        protest_size,
                        total_pop,
                        grievance = faction_grievance.to_f64(),
                        tick = tick_u64,
                        "Protest occurred"
                    );

                    // Council response
                    let council_legitimacy = avg_council_legitimacy;
                    let council_enforcement = self.institutions.iter()
                        .filter(|i| i.kind == InstitutionKind::Council)
                        .map(|i| i.enforcement_capacity)
                        .fold(Fixed::ZERO, |a, b| a.max(b));

                    let (suppressed, legitimacy_effect) = factions::council_response(
                        council_legitimacy, council_enforcement, protest_size, total_pop,
                    );

                    // Apply legitimacy effect to council
                    for inst in self.institutions.iter_mut() {
                        if inst.kind == InstitutionKind::Council {
                            if legitimacy_effect > Fixed::ZERO {
                                inst.increase_legitimacy(legitimacy_effect);
                            } else {
                                inst.decay_legitimacy(-legitimacy_effect);
                            }
                        }
                    }

                    // If not suppressed, faction gains legitimacy and morale boost
                    if !suppressed {
                        if let Some(faction) = self.institutions.get_mut(inst_idx) {
                            faction.increase_legitimacy(Fixed::from_f64(0.02));
                            faction.collective.morale = (faction.collective.morale + Fixed::from_f64(0.05)).clamp_01();
                        }
                    }

                    // Record in provenance (use leader of the protesting faction)
                    let protest_agent = self.institutions[inst_idx].get_role_holder("Leader")
                        .unwrap_or(AgentId::new(0));
                    self.provenance.record_decision(crate::provenance::DecisionTrace {
                        agent: protest_agent,
                        tick: tick_u64,
                        action_name: "Protest".into(),
                        factors: vec![
                            crate::provenance::DecisionFactor {
                                kind: "protest_size".into(),
                                magnitude: Fixed::from_int(protest_size as i64),
                                description: format!("Protest size: {protest_size}"),
                            },
                            crate::provenance::DecisionFactor {
                                kind: "suppressed".into(),
                                magnitude: if suppressed { Fixed::ONE } else { Fixed::ZERO },
                                description: format!("Suppressed: {suppressed}"),
                            },
                        ],
                        from_routine: false,
                        interrupted_by_critical_needs: false,
                        intention_abandoned: false,
                    });
                }
            }
        }

        // ── 14. Derived mental state computation (§22) ─────────────
        for agent in self.agents.iter_mut() {
            let stress = agent.emotions.fear + agent.emotions.anger;
            let need_deficit_avg = (agent.needs.hunger + agent.needs.thirst + agent.needs.fatigue + agent.needs.safety) * Fixed::from_f64(0.25);
            let social_support = Fixed::ONE - agent.needs.social;
            let success_rate = if agent.recent_attempts > 0 {
                Fixed::from_int(agent.recent_successes as i64) / Fixed::from_int(agent.recent_attempts as i64)
            } else {
                Fixed::from_f64(0.5)
            };
            let justice_perception = agent.moral_values.fairness;
            agent.derived.compute(&crate::person::MentalStateInput {
                stress,
                coping_potential: agent.personality.conscientiousness,
                social_support,
                need_deficit_avg,
                meaning: agent.needs.meaning,
                autonomy: agent.needs.autonomy,
                success_rate,
                neuroticism: agent.personality.neuroticism,
                justice_perception,
            });
            // Decay success tracking window
            agent.recent_attempts = agent.recent_attempts.saturating_sub(1);
            agent.recent_successes = agent.recent_successes.saturating_sub(1);
        }

        // ── 13. Belief updates from this tick's social interactions ────
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

    /// §34: Get a reference to the causal provenance store.
    pub fn provenance(&self) -> &CausalProvenance {
        &self.provenance
    }

    /// §16.1: Capture a full deterministic snapshot of the simulation state.
    /// RNG streams are not serialized — only the master seed is stored.
    pub fn save_snapshot(&self) -> Snapshot {
        Snapshot::capture(
            &self.config,
            self.clock.tick(),
            self.config.seed,
            &self.world,
            &self.agents,
            &self.relationships,
            &self.events,
            &self.journal,
            &self.norms,
            &self.institutions,
            &self.provenance,
        )
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
