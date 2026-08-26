//! Tick pass 4: action execution effects.

use super::command_goal_action;
use super::{
    ActionKind, AgentBundle, AgentId, DecisionFactor, DecisionTrace, EntityId, Fixed, Goal,
    GoalKind, GoalSource, IdentityKind, SimEvent, Simulation, Tick,
};
use crate::actions;
use crate::institutions;

impl Simulation {
    pub(super) fn tick_action_pass(
        ctx: &mut crate::systems::SystemContext,
        agents: &mut [AgentBundle],
        tick_u64: u64,
        bodies: &mut [crate::person::BodyState],
        emotions: &mut [crate::person::DiscreteEmotions],
        goals: &mut [Vec<crate::person::Goal>],
        needs: &mut [crate::person::NeedState],
        personalities: &[crate::person::Personality],
        tick: Tick,
        institutions_reg: &[crate::institutions::Institution],
        norms: &crate::norms::NormRegistry,
        provenance_x: &mut crate::provenance::CausalProvenance,
        season: &crate::ecology::SeasonTracker,
        tick_action_starts: &mut Vec<(usize, crate::actions::ActionKind)>,
    ) {
        // ── 4. Action execution (per-tick effects) ────────────────
        for i in 0..agents.len() {
            // Track causal provenance_x flags per agent per tick
            let mut was_interrupted_by_critical_needs = false;
            let mut intention_abandoned_this_tick = false;

            // §24.5: Intention commitment — check if current intention should be abandoned
            if let Some(ref intention) = agents[i].intention {
                if intention.completed {
                    // §3.2: Goal completed — move to completed list for learning
                    let completed_goal = Goal {
                        kind: intention.goal_kind,
                        priority: Fixed::from_f64(0.5),
                        commitment: Fixed::ONE,
                        created_tick: intention.formed_tick,
                        source: GoalSource::Identity, // default source for completed goals
                    };
                    agents[i].completed_goals.push(completed_goal);
                    if agents[i].completed_goals.len() > 50 {
                        agents[i].completed_goals.remove(0);
                    }
                    agents[i].intention = None;
                    intention_abandoned_this_tick = true;
                } else {
                    let stress = emotions[i].fear + emotions[i].anger;
                    let emotional_shock = emotions[i].anger > Fixed::from_f64(0.5)
                        || emotions[i].fear > Fixed::from_f64(0.5);
                    if intention.should_abandon(tick_u64, stress, emotional_shock) {
                        // §3.2: Goal abandoned — move to rejected list for learning
                        let rejected_goal = Goal {
                            kind: intention.goal_kind,
                            priority: Fixed::from_f64(0.5),
                            commitment: Fixed::ZERO,
                            created_tick: intention.formed_tick,
                            source: GoalSource::Identity,
                        };
                        agents[i].rejected_goals.push(rejected_goal);
                        if agents[i].rejected_goals.len() > 50 {
                            agents[i].rejected_goals.remove(0);
                        }
                        agents[i].intention = None;
                        agents[i].action_progress = 0; // force reselection
                        intention_abandoned_this_tick = true;
                    }
                }
            }

            // Action interruption: force address critical needs (> 0.9)
            if agents[i].action_progress > 0 {
                let critical = needs[i].hunger > Fixed::from_f64(0.9)
                    || needs[i].thirst > Fixed::from_f64(0.9)
                    || needs[i].fatigue > Fixed::from_f64(0.95);
                if critical {
                    // §3.2: Goal interrupted by critical needs — move to rejected list
                    if let Some(ref intention) = agents[i].intention {
                        let rejected_goal = Goal {
                            kind: intention.goal_kind,
                            priority: Fixed::from_f64(0.5),
                            commitment: Fixed::ZERO,
                            created_tick: intention.formed_tick,
                            source: GoalSource::Identity,
                        };
                        agents[i].rejected_goals.push(rejected_goal);
                        if agents[i].rejected_goals.len() > 50 {
                            agents[i].rejected_goals.remove(0);
                        }
                    }
                    agents[i].action_progress = 0;
                    agents[i].intention = None; // critical needs override intentions
                    was_interrupted_by_critical_needs = true;
                    intention_abandoned_this_tick = true;
                }
            }

            if agents[i].action_progress == 0 {
                let total_grain = ctx.world.total_food();
                let total_water = ctx.world.total_water();
                // Compute norm pressure using the registry's formula (identity-aware)
                let conformity = personalities[i].conformity;
                let avg_identity_strength = agents[i]
                    .identity
                    .strength_of(IdentityKind::Farmer)
                    .max(agents[i].identity.strength_of(IdentityKind::Parent))
                    .max(agents[i].identity.strength_of(IdentityKind::Believer));
                // §12: Institution enforcement amplifies norm pressure.
                // Council enforcement capacity multiplies the pressure felt by agents.
                let enforcement_multiplier = institutions_reg
                    .iter()
                    .filter(|i| i.kind == institutions::InstitutionKind::Council)
                    .map(|i| Fixed::ONE + i.enforcement_capacity * Fixed::from_f64(0.5))
                    .fold(Fixed::ONE, std::cmp::Ord::max);

                // §22.1: Moral values modulate norm pressure.
                // High fairness/authority → stronger norm compliance.
                // High liberty → weaker norm compliance.
                let moral_modifier = agents[i].moral_values.fairness * Fixed::from_f64(0.15)
                    + agents[i].moral_values.authority * Fixed::from_f64(0.1)
                    - agents[i].moral_values.liberty * Fixed::from_f64(0.08);
                let norm_pressure = norms
                    .norms()
                    .iter()
                    .map(|n| norms.compute_pressure(conformity, avg_identity_strength, n.id))
                    .fold(Fixed::ZERO, |a, b| a + b)
                    * enforcement_multiplier
                    - moral_modifier;

                // §10.3: Routine bias — agents follow daily schedules when stable.
                // §24: Personality modulates routine adherence — conscientiousness
                // increases it, openness decreases it.
                let (routine_action, routine_strength) = agents[i].routine.preferred_action(tick);
                let stress = emotions[i].fear + emotions[i].anger;
                let follow_routine = agents[i].routine.should_follow(
                    needs[i].hunger,
                    needs[i].thirst,
                    needs[i].fatigue,
                    stress,
                );
                // Personality-modulated routine strength: conscientiousness boosts, openness reduces
                let personality_routine_modifier = personalities[i].conscientiousness
                    * Fixed::from_f64(0.3)
                    - personalities[i].openness * Fixed::from_f64(0.2);
                let effective_routine_strength =
                    (routine_strength + personality_routine_modifier).clamp_01();

                // §22.1: When stressed, agents favor habit/routine over utility.
                // High heuristic bias → agents follow routine more readily.
                let effective_routine_threshold =
                    if agents[i].cognitive.heuristic_bias > Fixed::from_f64(0.5) {
                        Fixed::from_f64(0.3) // stressed agents follow routine at lower threshold
                    } else {
                        Fixed::from_f64(0.5) // normal agents need stronger routine signal
                    };

                // Iteration 255 (audit Phase 3 — survival-integrity
                // reflexes): the physiological override layer BENEATH the
                // utility AI. Critical deficits force the relief action
                // directly — no utility contest, no habit substitution, no
                // command override can outrank a body at its limits. This
                // makes the audit's E3 inversion (agent starving while
                // working) impossible BY CONSTRUCTION rather than by
                // calibration. Health-critical agents restrict to
                // rest/recovery instead. Deterministic, RNG-free, and
                // unreachable in calibrated calm windows (thirst tops out
                // ~0.62 there), so golden stays byte-identical.
                let reflex_override = if needs[i].thirst > Fixed::from_f64(0.9)
                    && needs[i].thirst >= needs[i].hunger
                {
                    Some(ActionKind::Drink)
                } else if needs[i].hunger > Fixed::from_f64(0.9) {
                    Some(ActionKind::Eat)
                } else if needs[i].fatigue > Fixed::from_f64(0.95) {
                    Some(ActionKind::Rest)
                } else if agents[i].body.health < Fixed::from_f64(0.25) {
                    // Health-critical: restrict to recovery actions only.
                    Some(ActionKind::Rest)
                } else {
                    None
                };
                let mut action = if let Some(forced) = reflex_override {
                    forced
                } else if let Some((cmd_action, cmd_kind)) =
                    command_goal_action(&goals[i], &needs[i])
                {
                    // §5 (Iteration 155): an external directive takes
                    // priority over routine and internal drives, and is
                    // consumed the moment it steers a selection — a
                    // one-shot nudge that returns the agent to
                    // autonomy. Gated on `GoalSource::Command` goals —
                    // which only exist after `command_agent` is called
                    // (never in a calibrated window), so this branch is
                    // a structural no-op everywhere calibrated and draws
                    // zero RNG.
                    goals[i].retain(|g| !(g.source == GoalSource::Command && g.kind == cmd_kind));
                    cmd_action
                } else if follow_routine && effective_routine_strength > effective_routine_threshold
                {
                    // §10.3: Routine creates behavioral stability — prefer scheduled action
                    routine_action
                } else if !agents[i].feuds.is_empty()
                    && emotions[i].anger > Fixed::from_f64(0.4)
                    && needs[i].hunger < Fixed::from_f64(0.85)
                    && needs[i].thirst < Fixed::from_f64(0.85)
                {
                    // §19.5.G: Angry feuding agents Move toward their feud target, NOT when critical needs demand attention
                    let feud_target = agents[i].feuds[0];
                    ActionKind::Move {
                        target_x: agents[feud_target].position.x,
                        target_y: agents[feud_target].position.y,
                    }
                } else {
                    // §12.3: Institution collective morale modulates norm compliance.
                    // Members of institutions_reg with high morale are more norm-compliant.
                    // Note: norm_pressure is negative = compliant, positive = violating.
                    let mut adjusted_pressure = norm_pressure;
                    for inst in institutions_reg {
                        if inst.has_member(AgentId::new(i as u64)) {
                            // High morale → more compliant (pressure becomes more negative)
                            let morale_bonus = inst.collective.morale * Fixed::from_f64(0.1);
                            adjusted_pressure -= morale_bonus;
                        }
                    }
                    actions::select_action(
                        &actions::DecisionContext {
                            needs: &needs[i],
                            personality: &personalities[i],
                            active_goals: &goals[i],
                            identity: &agents[i].identity,
                            decision_policy: &agents[i].decision_policy,
                            total_grain,
                            total_water,
                            coin: agents[i].wealth.coin,
                            norm_pressure: adjusted_pressure,
                            anger: emotions[i].anger,
                            fear: emotions[i].fear,
                            joy: emotions[i].joy,
                            sadness: emotions[i].sadness,
                            stress,
                            fairness: agents[i].moral_values.fairness,
                            authority: agents[i].moral_values.authority,
                            care: agents[i].moral_values.care,
                            loyalty: agents[i].moral_values.loyalty,
                            // §9.2 (Iteration 94): the agent's learned
                            // RL valuation weights feed selection.
                            action_values: agents[i].neural_like.values,
                            // §8.1.5 (Iteration 96): the full-pressure
                            // motivation argmax biases selection toward
                            // actions that relieve the dominant need.
                            dominant_need: agents[i].motivation.dominant_need,
                            dominant_pressure: agents[i].motivation.dominant_pressure(),
                            // §8.1.16 (Iteration 103): the scenario-
                            // grounded dread (regrounded on the daily
                            // phase, before selection in the same tick)
                            // drives precautionary provisioning — Work/
                            // Trade up, Rest down. Zero-at-zero: agents
                            // whose tiers do not run prospection hold
                            // dread 0 → legacy utility.
                            dread: agents[i].prospection.dread,
                            // §8.1.16 (Iteration 203): the scenario-
                            // grounded hope drives aspirational
                            // engagement — Socialize/Worship up, Idle
                            // down (the positive mirror of dread).
                            hope: agents[i].prospection.hope,
                            // §8.1.12 (Iteration 204): the blended
                            // emotion + executive-function planning
                            // confidence drives deferred-gratification
                            // calibration — Work up / Idle down when
                            // confident (baseline-corrected at 0.5, so
                            // default populations stay byte-identical).
                            planning_confidence: agents[i].prospection.planning_confidence,
                            // Iteration 232: mood drift — valence
                            // from affect module ([-1,1]) drives
                            // social/exploration vs withdrawal.
                            mood_valence: agents[i].affect.valence,
                            // Iteration 233: seasonal behavioral modulation
                            season: season.current as u8,
                            // Iteration 236: age-related behavioral modulation
                            life_stage: agents[i].embodied.development.life_stage as u8,
                            // Iteration 248 (Arc B): sleep-debt social
                            // withdrawal — zero below the deprivation
                            // threshold (0.5), scaling above; rested
                            // agents are byte-identical.
                            social_withdrawal: {
                                let debt = agents[i].embodied.circadian.sleep_debt;
                                if debt > Fixed::from_f64(0.5) {
                                    debt - Fixed::from_f64(0.5)
                                } else {
                                    Fixed::ZERO
                                }
                            },
                            // Iteration 247 (Arc B — interoception): the
                            // somatic marker — how much worse than a
                            // default interoceptor the agent feels (fatigue
                            // + pain). Biases risky actions down; exactly
                            // zero for default configurations, so calm
                            // worlds stay byte-identical.
                            somatic_marker: agents[i]
                                .interoception
                                .somatic_risk_bias(needs[i].fatigue, agents[i].embodied.injury),
                            // AP3 DC-1 (task 3.4): development gating —
                            // fulfillment thresholds via needs bands.
                            // Zero at neutral (task 3.1 newborn/3.2 virgin)
                            // so goldens stay byte-identical until field moves.
                            development: &agents[i].development,
                        },
                        ctx.rng,
                    )
                };
                // §8.1.19 (P3-1, August 14, 2026): habitual fallback
                // under stress — an agent whose automaticity is high
                // (habits formed through ~100 practice ticks at the
                // 0.1-proficiency milestone) and who is stressed
                // substitutes its strongest habit for the deliberated
                // action ("under stress, agents fall back on habits").
                // Zero-blast in calibrated windows: automaticity stays
                // < 0.5 until habits actually form (base strength 0.3 →
                // ≈0.29 at golden stress levels), so the golden/snapshot
                // horizons are untouched and only long-horizon runs where
                // practice has accumulated see the fallback. Deterministic,
                // no RNG.
                if stress > Fixed::from_f64(0.5)
                    && agents[i].psych_skills.automaticity > Fixed::from_f64(0.5)
                {
                    let trigger = match agents[i].motivation.dominant_need {
                        crate::psychology::motivation::MotiveCategory::Hunger => "hunger",
                        crate::psychology::motivation::MotiveCategory::Thirst => "thirst",
                        crate::psychology::motivation::MotiveCategory::Sleep => "fatigue",
                        crate::psychology::motivation::MotiveCategory::Belonging
                        | crate::psychology::motivation::MotiveCategory::Attachment
                        | crate::psychology::motivation::MotiveCategory::Romance => "social",
                        crate::psychology::motivation::MotiveCategory::Meaning => "meaning",
                        _ => "",
                    };
                    if let Some(habit) = agents[i].psych_skills.execute_habit(trigger, tick_u64) {
                        action = match habit.as_str() {
                            "Work" => ActionKind::Work,
                            "Trade" => ActionKind::Trade,
                            "Socialize" => ActionKind::Socialize,
                            "Worship" => ActionKind::Worship,
                            "Eat" => ActionKind::Eat,
                            _ => action,
                        };
                    }
                }
                agents[i].current_action = action;
                agents[i].action_progress = action.definition().duration_ticks;
                tick_action_starts.push((i, action));

                let agent_id = AgentId::new(i as u64);

                // §34: Record decision trace for causal provenance_x
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
                            description: format!(
                                "Routine strength: {:.2}",
                                effective_routine_strength.to_f64()
                            ),
                        });
                    }
                    provenance_x.record_decision(DecisionTrace {
                        agent: agent_id,
                        tick: tick_u64,
                        action_name: format!("{action:?}"),
                        factors,
                        from_routine: follow_routine
                            && effective_routine_strength > Fixed::from_f64(0.5),
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
                agents[i].intention = Some(crate::person::Intention::new(
                    goal_kind,
                    tick_u64,
                    personalities[i].conscientiousness,
                ));

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

            let action = agents[i].current_action;
            actions::apply_action_tick(action, &mut bodies[i], &mut needs[i]);

            agents[i].action_progress = agents[i].action_progress.saturating_sub(1);
        }
    }
}
