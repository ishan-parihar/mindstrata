//! Action execution system — utility-based selection and execution.
//!
//! Agents choose actions based on utility scores that combine need relief,
//! emotional relief, social effects, identity congruence, cost, and noise.
//! This produces bounded rationality — agents are not perfectly optimal.

use crate::person::{BodyState, Goal, GoalKind, NeedState, Personality};
use mindstrata_core::fixed::Fixed;
use mindstrata_core::rng::{RngStreams, RngStream};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// An action that an agent can take.
///
/// **Field semantics for `per_tick_effects()`:**
/// - Fields marked *total* are divided by `duration_ticks` to get per-tick values.
/// - Fields marked *per-tick* are used as-is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDef {
    pub kind: ActionKind,
    pub duration_ticks: u32,
    /// Total hunger relief over the action duration (divided by duration). *total*
    pub hunger_relief: Fixed,
    /// Total thirst relief over the action duration (divided by duration). *total*
    pub thirst_relief: Fixed,
    /// Total fatigue relief over the action duration (divided by duration). *total*
    pub fatigue_relief: Fixed,
    /// Social interaction value used in utility computation (NOT divided). *per-tick*
    pub social_value: Fixed,
    /// Total energy cost over the action duration (divided by duration). *total*
    pub energy_cost: Fixed,
    /// Additional fatigue relief per tick beyond base fatigue_relief. *per-tick*
    pub bonus_fatigue_relief: Fixed,
    /// Additional energy recovery per tick. *per-tick*
    pub bonus_energy_recovery: Fixed,
    /// Additional social need reduction per tick. *per-tick*
    pub bonus_social_relief: Fixed,
    /// Additional meaning need reduction per tick. *per-tick*
    pub bonus_meaning_relief: Fixed,
}

/// Kinds of actions agents can perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    Eat,
    Drink,
    Rest,
    Work,
    Socialize,
    Worship,
    Wander,
    Idle,
}

impl ActionKind {
    /// Get the action definition for this kind.
    pub fn definition(self) -> ActionDef {
        match self {
            ActionKind::Eat => ActionDef {
                kind: self,
                duration_ticks: 4,
                hunger_relief: Fixed::from_f64(0.4),
                thirst_relief: Fixed::ZERO,
                fatigue_relief: Fixed::ZERO,
                social_value: Fixed::ZERO,
                energy_cost: Fixed::from_f64(0.02),
                bonus_fatigue_relief: Fixed::ZERO,
                bonus_energy_recovery: Fixed::ZERO,
                bonus_social_relief: Fixed::ZERO,
                bonus_meaning_relief: Fixed::ZERO,
            },
            ActionKind::Drink => ActionDef {
                kind: self,
                duration_ticks: 2,
                hunger_relief: Fixed::ZERO,
                thirst_relief: Fixed::from_f64(0.5),
                fatigue_relief: Fixed::ZERO,
                social_value: Fixed::ZERO,
                energy_cost: Fixed::from_f64(0.01),
                bonus_fatigue_relief: Fixed::ZERO,
                bonus_energy_recovery: Fixed::ZERO,
                bonus_social_relief: Fixed::ZERO,
                bonus_meaning_relief: Fixed::ZERO,
            },
            ActionKind::Rest => ActionDef {
                kind: self,
                duration_ticks: 8,
                hunger_relief: Fixed::ZERO,
                thirst_relief: Fixed::ZERO,
                fatigue_relief: Fixed::from_f64(0.3),
                social_value: Fixed::ZERO,
                energy_cost: Fixed::ZERO,
                bonus_fatigue_relief: Fixed::from_f64(0.1),
                bonus_energy_recovery: Fixed::from_f64(0.06),
                bonus_social_relief: Fixed::ZERO,
                bonus_meaning_relief: Fixed::ZERO,
            },
            ActionKind::Work => ActionDef {
                kind: self,
                duration_ticks: 8,
                hunger_relief: Fixed::ZERO,
                thirst_relief: Fixed::ZERO,
                fatigue_relief: Fixed::ZERO,
                social_value: Fixed::ZERO,
                energy_cost: Fixed::from_f64(0.05),
                bonus_fatigue_relief: Fixed::ZERO,
                bonus_energy_recovery: Fixed::ZERO,
                bonus_social_relief: Fixed::ZERO,
                bonus_meaning_relief: Fixed::ZERO,
            },
            ActionKind::Socialize => ActionDef {
                kind: self,
                duration_ticks: 4,
                hunger_relief: Fixed::ZERO,
                thirst_relief: Fixed::ZERO,
                fatigue_relief: Fixed::ZERO,
                social_value: Fixed::from_f64(0.3),
                energy_cost: Fixed::from_f64(0.01),
                bonus_fatigue_relief: Fixed::ZERO,
                bonus_energy_recovery: Fixed::ZERO,
                bonus_social_relief: Fixed::from_f64(0.1),
                bonus_meaning_relief: Fixed::ZERO,
            },
            ActionKind::Worship => ActionDef {
                kind: self,
                duration_ticks: 4,
                hunger_relief: Fixed::ZERO,
                thirst_relief: Fixed::ZERO,
                fatigue_relief: Fixed::ZERO,
                social_value: Fixed::from_f64(0.1),
                energy_cost: Fixed::from_f64(0.01),
                bonus_fatigue_relief: Fixed::ZERO,
                bonus_energy_recovery: Fixed::ZERO,
                bonus_social_relief: Fixed::ZERO,
                bonus_meaning_relief: Fixed::from_f64(0.1),
            },
            ActionKind::Wander => ActionDef {
                kind: self,
                duration_ticks: 2,
                hunger_relief: Fixed::ZERO,
                thirst_relief: Fixed::ZERO,
                fatigue_relief: Fixed::ZERO,
                social_value: Fixed::ZERO,
                energy_cost: Fixed::from_f64(0.02),
                bonus_fatigue_relief: Fixed::ZERO,
                bonus_energy_recovery: Fixed::ZERO,
                bonus_social_relief: Fixed::ZERO,
                bonus_meaning_relief: Fixed::ZERO,
            },
            ActionKind::Idle => ActionDef {
                kind: self,
                duration_ticks: 1,
                hunger_relief: Fixed::ZERO,
                thirst_relief: Fixed::ZERO,
                fatigue_relief: Fixed::from_f64(0.05),
                social_value: Fixed::ZERO,
                energy_cost: Fixed::ZERO,
                bonus_fatigue_relief: Fixed::ZERO,
                bonus_energy_recovery: Fixed::ZERO,
                bonus_social_relief: Fixed::ZERO,
                bonus_meaning_relief: Fixed::ZERO,
            },
        }
    }

    /// Compute per-tick effect by dividing total relief by duration.
    /// All effects (including bonuses) are derived from the definition.
    pub fn per_tick_effects(self) -> ActionDef {
        let def = self.definition();
        let d = Fixed::from_int(def.duration_ticks as i64);
        ActionDef {
            kind: self,
            duration_ticks: 1,
            hunger_relief: if def.hunger_relief > Fixed::ZERO { def.hunger_relief / d } else { Fixed::ZERO },
            thirst_relief: if def.thirst_relief > Fixed::ZERO { def.thirst_relief / d } else { Fixed::ZERO },
            fatigue_relief: if def.fatigue_relief > Fixed::ZERO { def.fatigue_relief / d } else { Fixed::ZERO },
            social_value: def.social_value,
            energy_cost: if def.energy_cost > Fixed::ZERO { def.energy_cost / d } else { Fixed::ZERO },
            // Bonus fields are already per-tick values — do not divide by duration.
            bonus_fatigue_relief: def.bonus_fatigue_relief,
            bonus_energy_recovery: def.bonus_energy_recovery,
            bonus_social_relief: def.bonus_social_relief,
            bonus_meaning_relief: def.bonus_meaning_relief,
        }
    }
}

/// Compute the utility of an action for a given agent state.
pub fn compute_utility(
    action: &ActionDef,
    needs: &NeedState,
    personality: &Personality,
    rng: &mut RngStreams,
) -> Fixed {
    let mut utility = Fixed::ZERO;

    if action.hunger_relief > Fixed::ZERO {
        utility += needs.hunger * action.hunger_relief * Fixed::from_f64(2.0);
    }
    if action.thirst_relief > Fixed::ZERO {
        utility += needs.thirst * action.thirst_relief * Fixed::from_f64(2.5);
    }
    if action.fatigue_relief > Fixed::ZERO {
        utility += needs.fatigue * action.fatigue_relief * Fixed::from_f64(1.5);
    }
    if action.social_value > Fixed::ZERO {
        utility += needs.social * action.social_value * personality.extraversion;
    }

    utility -= action.energy_cost * Fixed::from_f64(0.5);

    let noise_roll: f64 = rng
        .get_mut(RngStream::Behavior)
        .random_range(-0.05..0.05);
    utility += Fixed::from_f64(noise_roll);

    utility
}

/// Select the best action for an agent based on utility.
pub fn select_action(
    needs: &NeedState,
    personality: &Personality,
    active_goals: &[Goal],
    rng: &mut RngStreams,
) -> ActionKind {
    let candidates = [
        ActionKind::Eat,
        ActionKind::Drink,
        ActionKind::Rest,
        ActionKind::Work,
        ActionKind::Socialize,
        ActionKind::Worship,
        ActionKind::Wander,
        ActionKind::Idle,
    ];

    let mut best_action = ActionKind::Idle;
    let mut best_utility = Fixed::MIN;

    for kind in &candidates {
        let def = kind.definition();
        let mut utility = compute_utility(&def, needs, personality, rng);

        for goal in active_goals {
            let goal_aligned = matches!(
                (kind, goal.kind),
                (ActionKind::Eat, GoalKind::Eat)
                    | (ActionKind::Drink, GoalKind::Drink)
                    | (ActionKind::Rest, GoalKind::Rest)
                    | (ActionKind::Work, GoalKind::Work)
                    | (ActionKind::Socialize, GoalKind::Socialize)
                    | (ActionKind::Worship, GoalKind::Worship)
            );
            if goal_aligned {
                utility += goal.priority * Fixed::from_f64(0.5);
            }
        }

        if utility > best_utility {
            best_utility = utility;
            best_action = *kind;
        }
    }

    best_action
}

/// Apply per-tick effects of an action to agent state.
/// All effects are derived from the definition via `per_tick_effects()`.
pub fn apply_action_tick(action: ActionKind, body: &mut BodyState, needs: &mut NeedState) {
    let fx = action.per_tick_effects();

    needs.hunger = (needs.hunger - fx.hunger_relief).clamp_01();
    needs.thirst = (needs.thirst - fx.thirst_relief).clamp_01();
    needs.fatigue = (needs.fatigue - fx.fatigue_relief - fx.bonus_fatigue_relief).clamp_01();
    body.energy = (body.energy - fx.energy_cost + fx.bonus_energy_recovery).clamp_01();
    needs.social = (needs.social - fx.bonus_social_relief).clamp_01();
    needs.meaning = (needs.meaning - fx.bonus_meaning_relief).clamp_01();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eat_reduces_hunger() {
        let mut body = BodyState::default();
        let mut needs = NeedState {
            hunger: Fixed::from_f64(0.8),
            ..Default::default()
        };

        for _ in 0..4 {
            apply_action_tick(ActionKind::Eat, &mut body, &mut needs);
        }

        assert!(needs.hunger < Fixed::from_f64(0.5));
    }

    #[test]
    fn utility_prefers_food_when_hungry() {
        let mut rng = RngStreams::new(42);
        let needs = NeedState {
            hunger: Fixed::from_f64(0.9),
            thirst: Fixed::from_f64(0.1),
            ..Default::default()
        };

        let personality = Personality {
            openness: Fixed::from_f64(0.5),
            conscientiousness: Fixed::from_f64(0.5),
            extraversion: Fixed::from_f64(0.5),
            agreeableness: Fixed::from_f64(0.5),
            neuroticism: Fixed::from_f64(0.5),
            risk_tolerance: Fixed::from_f64(0.5),
            conformity: Fixed::from_f64(0.5),
            ambition: Fixed::from_f64(0.5),
            altruism: Fixed::from_f64(0.5),
            traditionalism: Fixed::from_f64(0.5),
            dominance: Fixed::from_f64(0.5),
            impulsivity: Fixed::from_f64(0.5),
        };

        let eat_utility = compute_utility(&ActionKind::Eat.definition(), &needs, &personality, &mut rng);
        let drink_utility = compute_utility(&ActionKind::Drink.definition(), &needs, &personality, &mut rng);

        assert!(eat_utility > drink_utility);
    }

    #[test]
    fn per_tick_effects_match_definition() {
        let fx = ActionKind::Eat.per_tick_effects();
        let def = ActionKind::Eat.definition();
        let d = Fixed::from_int(def.duration_ticks as i64);
        assert_eq!(fx.hunger_relief, def.hunger_relief / d);
        assert_eq!(fx.energy_cost, def.energy_cost / d);
    }

    #[test]
    fn rest_reduces_fatigue_with_bonus() {
        let mut body = BodyState::default();
        let mut needs = NeedState {
            fatigue: Fixed::from_f64(0.8),
            ..Default::default()
        };

        for _ in 0..8 {
            apply_action_tick(ActionKind::Rest, &mut body, &mut needs);
        }

        assert!(needs.fatigue < Fixed::from_f64(0.3), "Rest should significantly reduce fatigue");
        assert!(body.energy > Fixed::from_f64(0.9), "Rest should recover energy");
    }

    #[test]
    fn socialize_reduces_social_need() {
        let mut body = BodyState::default();
        let mut needs = NeedState {
            social: Fixed::from_f64(0.8),
            ..Default::default()
        };

        for _ in 0..4 {
            apply_action_tick(ActionKind::Socialize, &mut body, &mut needs);
        }

        assert!(needs.social < Fixed::from_f64(0.5), "Socialize should reduce social need");
    }

    #[test]
    fn worship_reduces_meaning_need() {
        let mut body = BodyState::default();
        let mut needs = NeedState {
            meaning: Fixed::from_f64(0.8),
            ..Default::default()
        };

        for _ in 0..4 {
            apply_action_tick(ActionKind::Worship, &mut body, &mut needs);
        }

        assert!(needs.meaning < Fixed::from_f64(0.5), "Worship should reduce meaning deficit");
    }
}
