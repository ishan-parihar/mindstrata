//! Action execution system — utility-based selection and execution.
//!
//! Agents choose actions based on utility scores that combine need relief,
//! emotional relief, social effects, identity congruence, cost, and noise.
//! This produces bounded rationality — agents are not perfectly optimal.

use crate::person::{BodyState, Goal, GoalKind, IdentityKind, IdentityState, NeedState, Personality};
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
    /// §6: Move toward a specific target position (Manhattan distance).
    Move { target_x: i32, target_y: i32 },
    /// §13.3: Trade goods with another agent at the market.
    Trade,
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
            // §6: Move duration = 1 tick per 2 Manhattan distance units
            ActionKind::Move { .. } => ActionDef {
                kind: self,
                duration_ticks: 1, // actual ticks set dynamically in sim.rs
                hunger_relief: Fixed::ZERO,
                thirst_relief: Fixed::ZERO,
                fatigue_relief: Fixed::ZERO,
                social_value: Fixed::ZERO,
                energy_cost: Fixed::from_f64(0.01),
                bonus_fatigue_relief: Fixed::ZERO,
                bonus_energy_recovery: Fixed::ZERO,
                bonus_social_relief: Fixed::ZERO,
                bonus_meaning_relief: Fixed::ZERO,
            },
            ActionKind::Trade => ActionDef {
                kind: self,
                duration_ticks: 3,
                hunger_relief: Fixed::from_f64(0.3), // grain trade relieves hunger
                thirst_relief: Fixed::ZERO,
                fatigue_relief: Fixed::ZERO,
                social_value: Fixed::from_f64(0.2),
                energy_cost: Fixed::from_f64(0.02),
                bonus_fatigue_relief: Fixed::ZERO,
                bonus_energy_recovery: Fixed::ZERO,
                bonus_social_relief: Fixed::from_f64(0.05),
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

/// Identity-action affinity: which identities prefer which actions.
fn identity_affinity(action: ActionKind, identity: &IdentityState) -> Fixed {
    let kind = match action {
        ActionKind::Work => IdentityKind::Farmer,
        ActionKind::Worship => IdentityKind::Believer,
        ActionKind::Socialize | ActionKind::Trade => IdentityKind::Parent,
        ActionKind::Eat | ActionKind::Drink | ActionKind::Rest => IdentityKind::Parent,
        _ => return Fixed::ZERO,
    };
    identity.strength_of(kind) * Fixed::from_f64(0.3)
}

/// Compute the utility of an action for a given agent state.
///
/// Resource scarcity modifier: when grain/water stocks are low,
/// Eat/Drink get higher utility to create economic pressure.
/// Identity-congruent actions get higher utility.
/// Norm-compliant actions get bonus; antisocial actions get penalty.
#[expect(clippy::too_many_arguments)]
pub fn compute_utility(
    action: &ActionDef,
    needs: &NeedState,
    personality: &Personality,
    rng: &mut RngStreams,
    total_grain: Fixed,
    total_water: Fixed,
    identity: &IdentityState,
    norm_pressure: Fixed,
) -> Fixed {
    let mut utility = Fixed::ZERO;

    // §9.1: Nonlinear need pressure — deficit^exponent * personality_modifier.
    // Impulsivity amplifies hunger/thirst pressure, conscientiousness dampens fatigue.
    if action.hunger_relief > Fixed::ZERO {
        let hunger_pressure = needs.hunger * needs.hunger * Fixed::from_f64(2.0); // deficit^1.5 approx
        let hunger_pressure = hunger_pressure * (Fixed::ONE + personality.impulsivity * Fixed::from_f64(0.5));
        let mut hunger_util = hunger_pressure * action.hunger_relief * Fixed::from_f64(2.0);
        let scarcity = (Fixed::ONE - total_grain).clamp_01();
        hunger_util += scarcity * needs.hunger * Fixed::from_f64(0.5);
        utility += hunger_util;
    }
    if action.thirst_relief > Fixed::ZERO {
        let thirst_pressure = needs.thirst * needs.thirst * Fixed::from_f64(2.0);
        let thirst_pressure = thirst_pressure * (Fixed::ONE + personality.impulsivity * Fixed::from_f64(0.5));
        let mut thirst_util = thirst_pressure * action.thirst_relief * Fixed::from_f64(2.5);
        let scarcity = (Fixed::ONE - total_water).clamp_01();
        thirst_util += scarcity * needs.thirst * Fixed::from_f64(0.5);
        utility += thirst_util;
    }
    if action.fatigue_relief > Fixed::ZERO {
        let fatigue_pressure = needs.fatigue * needs.fatigue * (Fixed::ONE - personality.conscientiousness * Fixed::from_f64(0.3));
        utility += fatigue_pressure * action.fatigue_relief * Fixed::from_f64(1.5);
    }
    if action.social_value > Fixed::ZERO {
        utility += needs.social * action.social_value * personality.extraversion;
    }
    if action.bonus_meaning_relief > Fixed::ZERO {
        utility += needs.meaning * action.bonus_meaning_relief * Fixed::from_f64(1.2);
    }

    // Identity congruence bonus
    utility += identity_affinity(action.kind, identity);

    // Normative component: negative pressure = compliant (bonus for prosocial), positive = violating (bonus for antisocial)
    let normative = match action.kind {
        // Prosocial actions: negate pressure so compliant agents (negative pressure) get bonus
        ActionKind::Work => -norm_pressure * personality.conformity * Fixed::from_f64(0.15),
        ActionKind::Socialize => -norm_pressure * personality.conformity * Fixed::from_f64(0.10),
        ActionKind::Worship => -norm_pressure * personality.conformity * Fixed::from_f64(0.12),
        // Antisocial actions: positive pressure = violating agents prefer these
        ActionKind::Wander => norm_pressure * personality.conformity * Fixed::from_f64(0.05),
        ActionKind::Idle => norm_pressure * personality.conformity * Fixed::from_f64(0.08),
        // §13.3: Trade is prosocial — compliant agents prefer it
        ActionKind::Trade => -norm_pressure * personality.conformity * Fixed::from_f64(0.08),
        _ => Fixed::ZERO,
    };
    utility += normative;

    // §13.3: Trade utility — agents benefit from trade when they have coin and pressing needs
    if action.kind == ActionKind::Trade {
        // Trade is more useful when agent has coin to spend and needs are pressing
        let coin_pressure = (needs.hunger + needs.thirst) * Fixed::from_f64(0.5);
        let coin_utility = personality.ambition * Fixed::from_f64(0.3);
        let need_pressure_bonus = coin_pressure * Fixed::from_f64(0.4);
        utility += coin_utility + need_pressure_bonus;
    }

    utility -= action.energy_cost * Fixed::from_f64(0.5);

    let noise_roll: f64 = rng
        .get_mut(RngStream::Behavior)
        .random_range(-0.05..0.05);
    utility += Fixed::from_f64(noise_roll);

    utility
}

/// Select the best action for an agent based on utility.
#[expect(clippy::too_many_arguments)]
pub fn select_action(
    needs: &NeedState,
    personality: &Personality,
    active_goals: &[Goal],
    rng: &mut RngStreams,
    total_grain: Fixed,
    total_water: Fixed,
    identity: &IdentityState,
    norm_pressure: Fixed,
) -> ActionKind {
    let candidates = [
        ActionKind::Eat,
        ActionKind::Drink,
        ActionKind::Rest,
        ActionKind::Work,
        ActionKind::Socialize,
        ActionKind::Worship,
        ActionKind::Trade,
        ActionKind::Wander,
        ActionKind::Idle,
    ];

    let mut best_action = ActionKind::Idle;
    let mut best_utility = Fixed::MIN;

    for kind in &candidates {
        let def = kind.definition();
        let mut utility = compute_utility(&def, needs, personality, rng, total_grain, total_water, identity, norm_pressure);

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
        let personality = make_personality();
        let identity = IdentityState::default();

        let eat_utility = compute_utility(&ActionKind::Eat.definition(), &needs, &personality, &mut rng, Fixed::from_f64(0.5), Fixed::from_f64(0.5), &identity, Fixed::ZERO);
        let drink_utility = compute_utility(&ActionKind::Drink.definition(), &needs, &personality, &mut rng, Fixed::from_f64(0.5), Fixed::from_f64(0.5), &identity, Fixed::ZERO);

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
    fn rest_per_tick_effects() {
        let fx = ActionKind::Rest.per_tick_effects();
        let def = ActionKind::Rest.definition();
        let d = Fixed::from_int(def.duration_ticks as i64);
        // Base fatigue_relief is divided by duration
        assert_eq!(fx.fatigue_relief, def.fatigue_relief / d);
        // Bonus fields are NOT divided (they're already per-tick)
        assert_eq!(fx.bonus_fatigue_relief, def.bonus_fatigue_relief);
        assert_eq!(fx.bonus_energy_recovery, def.bonus_energy_recovery);
    }

    #[test]
    fn socialize_per_tick_effects() {
        let fx = ActionKind::Socialize.per_tick_effects();
        assert_eq!(fx.bonus_social_relief, Fixed::from_f64(0.1));
        assert_eq!(fx.social_value, Fixed::from_f64(0.3)); // social_value is NOT divided
    }

    #[test]
    fn worship_per_tick_effects() {
        let fx = ActionKind::Worship.per_tick_effects();
        assert_eq!(fx.bonus_meaning_relief, Fixed::from_f64(0.1));
    }

    fn make_personality() -> Personality {
        Personality {
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
        }
    }

    #[test]
    fn scarcity_increases_food_utility() {
        let mut rng = RngStreams::new(42);
        let needs = NeedState {
            hunger: Fixed::from_f64(0.5),
            ..Default::default()
        };
        let personality = make_personality();
        let identity = IdentityState::default();

        let normal = compute_utility(&ActionKind::Eat.definition(), &needs, &personality, &mut rng, Fixed::from_f64(0.8), Fixed::from_f64(0.8), &identity, Fixed::ZERO);
        let scarce = compute_utility(&ActionKind::Eat.definition(), &needs, &personality, &mut rng, Fixed::from_f64(0.1), Fixed::from_f64(0.8), &identity, Fixed::ZERO);

        assert!(scarce > normal, "Scarcity should increase food utility: scarce={scarce}, normal={normal}");
    }

    #[test]
    fn scarcity_increases_water_utility() {
        let mut rng = RngStreams::new(42);
        let needs = NeedState {
            thirst: Fixed::from_f64(0.5),
            ..Default::default()
        };
        let personality = make_personality();
        let identity = IdentityState::default();

        let normal = compute_utility(&ActionKind::Drink.definition(), &needs, &personality, &mut rng, Fixed::from_f64(0.8), Fixed::from_f64(0.8), &identity, Fixed::ZERO);
        let scarce = compute_utility(&ActionKind::Drink.definition(), &needs, &personality, &mut rng, Fixed::from_f64(0.8), Fixed::from_f64(0.1), &identity, Fixed::ZERO);

        assert!(scarce > normal, "Scarcity should increase water utility: scarce={scarce}, normal={normal}");
    }

    #[test]
    fn farmer_identity_increases_work_utility() {
        let mut rng = RngStreams::new(42);
        let needs = NeedState::default();
        let personality = make_personality();

        let farmer_identity = IdentityState {
            identities: vec![crate::person::AgentIdentity {
                kind: IdentityKind::Farmer,
                strength: Fixed::from_f64(0.8),
            }],
        };
        let no_identity = IdentityState::default();

        let u_farmer = compute_utility(&ActionKind::Work.definition(), &needs, &personality, &mut rng, Fixed::from_f64(0.5), Fixed::from_f64(0.5), &farmer_identity, Fixed::ZERO);
        let u_none = compute_utility(&ActionKind::Work.definition(), &needs, &personality, &mut rng, Fixed::from_f64(0.5), Fixed::from_f64(0.5), &no_identity, Fixed::ZERO);

        assert!(u_farmer > u_none, "Farmer identity should increase Work utility");
    }

    #[test]
    fn compliant_pressure_increases_work_utility() {
        let mut rng = RngStreams::new(42);
        let needs = NeedState::default();
        let personality = make_personality();
        let identity = IdentityState::default();

        // Negative pressure = compliant agent (compute_pressure returns negative for compliant)
        let no_pressure = compute_utility(&ActionKind::Work.definition(), &needs, &personality, &mut rng, Fixed::from_f64(0.5), Fixed::from_f64(0.5), &identity, Fixed::ZERO);
        let compliant_pressure = compute_utility(&ActionKind::Work.definition(), &needs, &personality, &mut rng, Fixed::from_f64(0.5), Fixed::from_f64(0.5), &identity, -Fixed::from_f64(0.5));

        assert!(compliant_pressure > no_pressure, "Compliant (negative) pressure should increase Work utility");
    }

    #[test]
    fn violating_pressure_decreases_work_utility() {
        let mut rng = RngStreams::new(42);
        let needs = NeedState::default();
        let personality = make_personality();
        let identity = IdentityState::default();

        // Positive pressure = violating agent
        let no_pressure = compute_utility(&ActionKind::Work.definition(), &needs, &personality, &mut rng, Fixed::from_f64(0.5), Fixed::from_f64(0.5), &identity, Fixed::ZERO);
        let violating_pressure = compute_utility(&ActionKind::Work.definition(), &needs, &personality, &mut rng, Fixed::from_f64(0.5), Fixed::from_f64(0.5), &identity, Fixed::from_f64(0.5));

        assert!(violating_pressure < no_pressure, "Violating (positive) pressure should decrease Work utility");
    }

    #[test]
    fn violating_pressure_increases_idle_utility() {
        let mut rng = RngStreams::new(42);
        let needs = NeedState::default();
        let personality = make_personality();
        let identity = IdentityState::default();

        // Positive pressure = violating agent prefers idle
        let no_pressure = compute_utility(&ActionKind::Idle.definition(), &needs, &personality, &mut rng, Fixed::from_f64(0.5), Fixed::from_f64(0.5), &identity, Fixed::ZERO);
        let violating_pressure = compute_utility(&ActionKind::Idle.definition(), &needs, &personality, &mut rng, Fixed::from_f64(0.5), Fixed::from_f64(0.5), &identity, Fixed::from_f64(0.5));

        assert!(violating_pressure > no_pressure, "Violating (positive) pressure should increase Idle utility");
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
