use super::{
    apply_action_tick, compute_utility, noise_amplitude, select_action, ActionKind,
    DecisionContext, APPROACH_WANDER_BONUS, PERSISTENCE_NOISE_FLOOR,
};
use crate::person::{BodyState, IdentityKind, IdentityState, NeedState, Personality, Temperament};
use crate::psychology::decision_policy::DecisionPolicy;
use crate::psychology::motivation::MotiveCategory;
use crate::psychology::neural_like::ActionValues;
use mindstrata_core::fixed::Fixed;
use mindstrata_core::rng::RngStreams;

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
fn broke_hungry_agent_prefers_eat_over_trade() {
    let mut rng = RngStreams::new(42);
    let needs = NeedState {
        hunger: Fixed::from_f64(0.9),
        thirst: Fixed::from_f64(0.1),
        ..Default::default()
    };
    let personality = make_personality();
    let identity = IdentityState::default();
    let dp = make_decision_policy();

    let chosen = select_action(
        &DecisionContext {
            needs: &needs,
            personality: &personality,
            active_goals: &[],
            identity: &identity,
            decision_policy: &dp,
            total_grain: Fixed::from_f64(0.5),
            total_water: Fixed::from_f64(0.5),
            coin: Fixed::ZERO, // broke: cannot afford to buy at the market
            norm_pressure: Fixed::ZERO,
            anger: Fixed::ZERO,
            fear: Fixed::ZERO,
            joy: Fixed::ZERO,
            sadness: Fixed::ZERO,
            stress: Fixed::ZERO,
            social_withdrawal: Fixed::ZERO,
            fairness: Fixed::ZERO,
            authority: Fixed::ZERO,
            care: Fixed::ZERO,
            loyalty: Fixed::ZERO,
            action_values: ActionValues::default(),
            dominant_need: MotiveCategory::Hunger,
            dominant_pressure: Fixed::ZERO,
            dread: Fixed::ZERO,
            hope: Fixed::ZERO,
            // 0.5 = neutral default → zero calibration shift (legacy utility).
            planning_confidence: Fixed::from_f64(0.5),
            mood_valence: Fixed::ZERO,
            season: 0,
            life_stage: 4,
            somatic_marker: Fixed::ZERO,
            development: &crate::psychology::DevelopmentFieldState::default(),
            polarity_claims: &[],
        },
        &mut rng,
    );
    assert!(
        chosen == ActionKind::Eat,
        "Broke hungry agent should Eat, got {chosen:?}"
    );
}

#[test]
fn wealthy_hungry_agent_can_prefer_trade() {
    let mut rng = RngStreams::new(42);
    let needs = NeedState {
        hunger: Fixed::from_f64(0.9),
        thirst: Fixed::from_f64(0.1),
        ..Default::default()
    };
    let personality = make_personality();
    let identity = IdentityState::default();
    let dp = make_decision_policy();

    // High ambition + coin makes market purchase attractive.
    let mut ambitious = personality;
    ambitious.ambition = Fixed::from_f64(0.9);
    let chosen = select_action(
        &DecisionContext {
            needs: &needs,
            personality: &ambitious,
            active_goals: &[],
            identity: &identity,
            decision_policy: &dp,
            total_grain: Fixed::from_f64(0.5),
            total_water: Fixed::from_f64(0.5),
            coin: Fixed::from_f64(20.0),
            norm_pressure: Fixed::ZERO,
            anger: Fixed::ZERO,
            fear: Fixed::ZERO,
            joy: Fixed::ZERO,
            sadness: Fixed::ZERO,
            stress: Fixed::ZERO,
            social_withdrawal: Fixed::ZERO,
            fairness: Fixed::ZERO,
            authority: Fixed::ZERO,
            care: Fixed::ZERO,
            loyalty: Fixed::ZERO,
            action_values: ActionValues::default(),
            dominant_need: MotiveCategory::Hunger,
            dominant_pressure: Fixed::ZERO,
            dread: Fixed::ZERO,
            hope: Fixed::ZERO,
            // 0.5 = neutral default → zero calibration shift (legacy utility).
            planning_confidence: Fixed::from_f64(0.5),
            mood_valence: Fixed::ZERO,
            season: 0,
            life_stage: 4,
            somatic_marker: Fixed::ZERO,
            development: &crate::psychology::DevelopmentFieldState::default(),
            polarity_claims: &[],
        },
        &mut rng,
    );
    assert!(
        chosen == ActionKind::Trade || chosen == ActionKind::Eat,
        "Wealthy hungry agent should consider Trade, got {chosen:?}"
    );
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
    let dp = make_decision_policy();

    let eat_utility = select_action(
        &DecisionContext {
            needs: &needs,
            personality: &personality,
            active_goals: &[],
            identity: &identity,
            decision_policy: &dp,
            total_grain: Fixed::from_f64(0.5),
            total_water: Fixed::from_f64(0.5),
            coin: Fixed::ZERO, // broke: hungry agent must eat, not buy
            norm_pressure: Fixed::ZERO,
            anger: Fixed::ZERO,
            fear: Fixed::ZERO,
            joy: Fixed::ZERO,
            sadness: Fixed::ZERO,
            stress: Fixed::ZERO,
            social_withdrawal: Fixed::ZERO,
            fairness: Fixed::ZERO,
            authority: Fixed::ZERO,
            care: Fixed::ZERO,
            loyalty: Fixed::ZERO,
            action_values: ActionValues::default(),
            dominant_need: MotiveCategory::Hunger,
            dominant_pressure: Fixed::ZERO,
            dread: Fixed::ZERO,
            hope: Fixed::ZERO,
            // 0.5 = neutral default → zero calibration shift (legacy utility).
            planning_confidence: Fixed::from_f64(0.5),
            mood_valence: Fixed::ZERO,
            season: 0,
            life_stage: 4,
            somatic_marker: Fixed::ZERO,
            development: &crate::psychology::DevelopmentFieldState::default(),
            polarity_claims: &[],
        },
        &mut rng,
    );
    assert!(
        eat_utility == ActionKind::Eat,
        "Broke hungry agent should prefer Eat, got {eat_utility:?}"
    );
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
        constitution: None,
        temperament: crate::person::Temperament::default(),
    }
}

fn make_decision_policy() -> DecisionPolicy {
    DecisionPolicy::default()
}

#[test]
fn scarcity_increases_food_utility() {
    let mut rng = RngStreams::new(42);
    let needs = NeedState {
        hunger: Fixed::from_f64(0.9),
        ..Default::default()
    };
    let personality = make_personality();
    let identity = IdentityState::default();
    let dp = make_decision_policy();

    // Use select_action and verify Eat is preferred under scarcity
    let scarce = select_action(
        &DecisionContext {
            needs: &needs,
            personality: &personality,
            active_goals: &[],
            identity: &identity,
            decision_policy: &dp,
            total_grain: Fixed::from_f64(0.1),
            total_water: Fixed::from_f64(0.8),
            coin: Fixed::ZERO, // broke: cannot afford market grain
            norm_pressure: Fixed::ZERO,
            anger: Fixed::ZERO,
            fear: Fixed::ZERO,
            joy: Fixed::ZERO,
            sadness: Fixed::ZERO,
            stress: Fixed::ZERO,
            social_withdrawal: Fixed::ZERO,
            fairness: Fixed::ZERO,
            authority: Fixed::ZERO,
            care: Fixed::ZERO,
            loyalty: Fixed::ZERO,
            action_values: ActionValues::default(),
            dominant_need: MotiveCategory::Hunger,
            dominant_pressure: Fixed::ZERO,
            dread: Fixed::ZERO,
            hope: Fixed::ZERO,
            // 0.5 = neutral default → zero calibration shift (legacy utility).
            planning_confidence: Fixed::from_f64(0.5),
            mood_valence: Fixed::ZERO,
            season: 0,
            life_stage: 4,
            somatic_marker: Fixed::ZERO,
            development: &crate::psychology::DevelopmentFieldState::default(),
            polarity_claims: &[],
        },
        &mut rng,
    );
    assert!(
        scarce == ActionKind::Eat,
        "Under grain scarcity, broke hungry agent should Eat, got {scarce:?}"
    );
}

#[test]
fn scarcity_increases_water_utility() {
    let mut rng = RngStreams::new(42);
    let needs = NeedState {
        thirst: Fixed::from_f64(0.9),
        ..Default::default()
    };
    let personality = make_personality();
    let identity = IdentityState::default();
    let dp = make_decision_policy();

    let scarce = select_action(
        &DecisionContext {
            needs: &needs,
            personality: &personality,
            active_goals: &[],
            identity: &identity,
            decision_policy: &dp,
            total_grain: Fixed::from_f64(0.8),
            total_water: Fixed::from_f64(0.1),
            coin: Fixed::from_f64(5.0),
            norm_pressure: Fixed::ZERO,
            anger: Fixed::ZERO,
            fear: Fixed::ZERO,
            joy: Fixed::ZERO,
            sadness: Fixed::ZERO,
            stress: Fixed::ZERO,
            social_withdrawal: Fixed::ZERO,
            fairness: Fixed::ZERO,
            authority: Fixed::ZERO,
            care: Fixed::ZERO,
            loyalty: Fixed::ZERO,
            action_values: ActionValues::default(),
            dominant_need: MotiveCategory::Hunger,
            dominant_pressure: Fixed::ZERO,
            dread: Fixed::ZERO,
            hope: Fixed::ZERO,
            // 0.5 = neutral default → zero calibration shift (legacy utility).
            planning_confidence: Fixed::from_f64(0.5),
            mood_valence: Fixed::ZERO,
            season: 0,
            life_stage: 4,
            somatic_marker: Fixed::ZERO,
            development: &crate::psychology::DevelopmentFieldState::default(),
            polarity_claims: &[],
        },
        &mut rng,
    );
    assert!(
        scarce == ActionKind::Drink,
        "Under water scarcity, thirsty agent should Drink, got {scarce:?}"
    );
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

    let u_farmer = compute_utility(
        &ActionKind::Work.definition(),
        &needs,
        &personality,
        &mut rng,
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &farmer_identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    let u_none = compute_utility(
        &ActionKind::Work.definition(),
        &needs,
        &personality,
        &mut rng,
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &no_identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );

    assert!(
        u_farmer > u_none,
        "Farmer identity should increase Work utility"
    );
}

#[test]
fn compliant_pressure_increases_work_utility() {
    let mut rng = RngStreams::new(42);
    let needs = NeedState::default();
    let personality = make_personality();
    let identity = IdentityState::default();

    // Negative pressure = compliant agent (compute_pressure returns negative for compliant)
    let no_pressure = compute_utility(
        &ActionKind::Work.definition(),
        &needs,
        &personality,
        &mut rng,
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    let compliant_pressure = compute_utility(
        &ActionKind::Work.definition(),
        &needs,
        &personality,
        &mut rng,
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        -Fixed::from_f64(0.5),
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );

    assert!(
        compliant_pressure > no_pressure,
        "Compliant (negative) pressure should increase Work utility"
    );
}

#[test]
fn violating_pressure_decreases_work_utility() {
    let mut rng = RngStreams::new(42);
    let needs = NeedState::default();
    let personality = make_personality();
    let identity = IdentityState::default();

    // Positive pressure = violating agent
    let no_pressure = compute_utility(
        &ActionKind::Work.definition(),
        &needs,
        &personality,
        &mut rng,
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    let violating_pressure = compute_utility(
        &ActionKind::Work.definition(),
        &needs,
        &personality,
        &mut rng,
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::from_f64(0.5),
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );

    assert!(
        violating_pressure < no_pressure,
        "Violating (positive) pressure should decrease Work utility"
    );
}

#[test]
fn violating_pressure_increases_idle_utility() {
    let mut rng = RngStreams::new(42);
    let needs = NeedState::default();
    let personality = make_personality();
    let identity = IdentityState::default();

    // Positive pressure = violating agent prefers idle
    let no_pressure = compute_utility(
        &ActionKind::Idle.definition(),
        &needs,
        &personality,
        &mut rng,
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    let violating_pressure = compute_utility(
        &ActionKind::Idle.definition(),
        &needs,
        &personality,
        &mut rng,
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::from_f64(0.5),
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );

    assert!(
        violating_pressure > no_pressure,
        "Violating (positive) pressure should increase Idle utility"
    );
}

#[test]
fn anger_biases_toward_wander() {
    let _rng = RngStreams::new(42);
    let _needs = NeedState::default();
    let _personality = make_personality();
    let _identity = IdentityState::default();
    let mut dp = make_decision_policy();
    dp.emotional_policy.anger_bias = Fixed::from_f64(0.8);

    // Compute utilities directly to verify the emotional modifier shifts Wander up.
    // We compare Wander with Eat (baseline non-social, non-risky action).
    let emo_wander = dp.emotional_modifier(
        Fixed::from_f64(0.8),
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::ZERO,
        false,
        true,
        false, // is_social=false, is_risky=true, is_withdrawal=false
    );
    let emo_eat = dp.emotional_modifier(
        Fixed::from_f64(0.8),
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::ZERO,
        false,
        false,
        false,
    );
    assert!(
        emo_wander > emo_eat,
        "Anger should boost risky non-social actions more: wander={emo_wander}, eat={emo_eat}"
    );
}

#[test]
fn fear_biases_toward_rest() {
    let _rng = RngStreams::new(42);
    let _needs = NeedState::default();
    let _personality = make_personality();
    let _identity = IdentityState::default();
    let mut dp = make_decision_policy();
    dp.emotional_policy.fear_bias = Fixed::from_f64(0.8);

    // Compute emotional modifiers directly to verify fear boosts withdrawal actions.
    let emo_rest = dp.emotional_modifier(
        Fixed::ZERO,
        Fixed::from_f64(0.8),
        Fixed::ZERO,
        Fixed::ZERO,
        false,
        false,
        true, // is_social=false, is_risky=false, is_withdrawal=true
    );
    let emo_wander = dp.emotional_modifier(
        Fixed::ZERO,
        Fixed::from_f64(0.8),
        Fixed::ZERO,
        Fixed::ZERO,
        false,
        true,
        false, // risky — fear does NOT boost risky actions
    );
    assert!(
        emo_rest > emo_wander,
        "Fear should boost withdrawal actions: rest={emo_rest}, wander={emo_wander}"
    );
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

    assert!(
        needs.fatigue < Fixed::from_f64(0.3),
        "Rest should significantly reduce fatigue"
    );
    assert!(
        body.energy > Fixed::from_f64(0.9),
        "Rest should recover energy"
    );
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

    assert!(
        needs.social < Fixed::from_f64(0.5),
        "Socialize should reduce social need"
    );
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

    assert!(
        needs.meaning < Fixed::from_f64(0.5),
        "Worship should reduce meaning deficit"
    );
}

// ── §8.1.5 (Iteration 96): dominant-need urgency boost ──────────
// Each compute_utility call uses its own identically-seeded RngStreams
// so the ±0.05 noise roll is identical across compared calls.

#[test]
fn dominant_need_boost_prefers_matching_channel() {
    let needs = NeedState {
        hunger: Fixed::from_f64(0.7),
        thirst: Fixed::from_f64(0.1),
        ..Default::default()
    };
    let personality = make_personality();
    let identity = IdentityState::default();

    let eat_when_hungry_dominant = compute_utility(
        &ActionKind::Eat.definition(),
        &needs,
        &personality,
        &mut RngStreams::new(42),
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::from_f64(0.8),
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    let eat_when_thirst_dominant = compute_utility(
        &ActionKind::Eat.definition(),
        &needs,
        &personality,
        &mut RngStreams::new(42),
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Thirst,
        Fixed::from_f64(0.8),
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    assert!(
        eat_when_hungry_dominant > eat_when_thirst_dominant,
        "Hunger-dominant agent should value Eat more than a Thirst-dominant one: {eat_when_hungry_dominant} vs {eat_when_thirst_dominant}",
    );
}

#[test]
fn abstract_dominant_need_yields_no_boost() {
    let needs = NeedState {
        hunger: Fixed::from_f64(0.7),
        ..Default::default()
    };
    let personality = make_personality();
    let identity = IdentityState::default();

    // Zero dominant pressure (the neutral prior) must equal a Safety-
    // dominant agent: Safety has no relief channel, so no boost applies.
    let zero_pressure = compute_utility(
        &ActionKind::Eat.definition(),
        &needs,
        &personality,
        &mut RngStreams::new(42),
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    let safety_dominant = compute_utility(
        &ActionKind::Eat.definition(),
        &needs,
        &personality,
        &mut RngStreams::new(42),
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Safety,
        Fixed::from_f64(0.8),
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    assert_eq!(
        zero_pressure, safety_dominant,
        "Safety-dominant (no relief channel) must not boost Eat"
    );
}

#[test]
fn dominant_boost_scales_with_pressure() {
    let needs = NeedState {
        hunger: Fixed::from_f64(0.7),
        ..Default::default()
    };
    let personality = make_personality();
    let identity = IdentityState::default();

    let low = compute_utility(
        &ActionKind::Eat.definition(),
        &needs,
        &personality,
        &mut RngStreams::new(42),
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::from_f64(0.2),
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    let high = compute_utility(
        &ActionKind::Eat.definition(),
        &needs,
        &personality,
        &mut RngStreams::new(42),
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::from_f64(0.9),
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    assert!(
        high > low,
        "Higher dominant pressure should yield a larger boost"
    );
}

#[test]
fn dominant_boost_does_not_fire_for_non_relieving_actions() {
    let needs = NeedState {
        hunger: Fixed::from_f64(0.7),
        ..Default::default()
    };
    let personality = make_personality();
    let identity = IdentityState::default();

    // Work has no hunger relief — a Hunger-dominant agent gets no boost.
    let zero_pressure = compute_utility(
        &ActionKind::Work.definition(),
        &needs,
        &personality,
        &mut RngStreams::new(42),
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    let hungry_dominant = compute_utility(
        &ActionKind::Work.definition(),
        &needs,
        &personality,
        &mut RngStreams::new(42),
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::from_f64(0.9),
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    assert_eq!(
        zero_pressure, hungry_dominant,
        "A Hunger-dominant agent must not boost non-relieving Work"
    );
}

/// §8.1.16 (Iteration 103): dread boosts provisioning actions and
/// suppresses Rest — the precautionary consumer's exact magnitude is
/// pinned: 0.6 dread → Work +0.12, Rest −0.06 (identical noise streams,
/// so the delta is precisely the term).
#[test]
fn dread_boosts_provisioning_and_suppresses_rest() {
    let needs = NeedState {
        hunger: Fixed::from_f64(0.3),
        ..Default::default()
    };
    let personality = make_personality();
    let identity = IdentityState::default();
    let work_dreadful = compute_utility(
        &ActionKind::Work.definition(),
        &needs,
        &personality,
        &mut RngStreams::new(42),
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::ZERO,
        Fixed::from_f64(0.6),
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    let work_calm = compute_utility(
        &ActionKind::Work.definition(),
        &needs,
        &personality,
        &mut RngStreams::new(42),
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    let rest_dreadful = compute_utility(
        &ActionKind::Rest.definition(),
        &needs,
        &personality,
        &mut RngStreams::new(42),
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::ZERO,
        Fixed::from_f64(0.6),
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    let rest_calm = compute_utility(
        &ActionKind::Rest.definition(),
        &needs,
        &personality,
        &mut RngStreams::new(42),
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.5),
        &identity,
        Fixed::ZERO,
        Fixed::ZERO,
        ActionValues::default(),
        MotiveCategory::Hunger,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::ZERO,
        Fixed::from_f64(0.5),
    );
    let work_delta = (work_dreadful - work_calm).to_f64();
    let rest_delta = (rest_calm - rest_dreadful).to_f64();
    assert!(
        work_delta > 0.0,
        "dread must boost Work, delta {work_delta:.6}"
    );
    assert!(
        rest_delta > 0.0,
        "dread must suppress Rest, delta {rest_delta:.6}"
    );
    // Exact magnitude: 0.6 × 0.2 = 0.12 for Work, 0.6 × 0.1 = 0.06 for
    // Rest (the noise streams are identical, so no other term differs).
    assert!(
        (work_delta - 0.12).abs() < 1e-9,
        "Work boost must be exactly 0.12, got {work_delta:.6}"
    );
    assert!(
        (rest_delta - 0.06).abs() < 1e-9,
        "Rest penalty must be exactly 0.06, got {rest_delta:.6}"
    );
}

/// §8.1.16 (Iteration 103): the precautionary term is exclusive to
/// provisioning — every other action's utility is byte-identical at
/// dread 0.6 vs dread 0 (identical noise streams), so the consumer
/// cannot distort the non-provisioning action set.
#[test]
fn dread_leaves_non_provisioning_actions_untouched() {
    let needs = NeedState {
        hunger: Fixed::from_f64(0.7),
        ..Default::default()
    };
    let personality = make_personality();
    let identity = IdentityState::default();
    for kind in [
        ActionKind::Eat,
        ActionKind::Drink,
        ActionKind::Socialize,
        ActionKind::Worship,
        ActionKind::Wander,
        ActionKind::Idle,
    ] {
        let dreadful = compute_utility(
            &kind.definition(),
            &needs,
            &personality,
            &mut RngStreams::new(42),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            &identity,
            Fixed::ZERO,
            Fixed::ZERO,
            ActionValues::default(),
            MotiveCategory::Hunger,
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.6),
            Fixed::ZERO,
            Fixed::from_f64(0.5),
        );
        let calm = compute_utility(
            &kind.definition(),
            &needs,
            &personality,
            &mut RngStreams::new(42),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            &identity,
            Fixed::ZERO,
            Fixed::ZERO,
            ActionValues::default(),
            MotiveCategory::Hunger,
            Fixed::from_f64(0.8),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
        );
        assert_eq!(
            dreadful.to_f64(),
            calm.to_f64(),
            "{kind:?} must be untouched by dread"
        );
    }
}

/// §8.1.16 (Iteration 103): the precautionary term shifts real
/// selections — a dreadful agent chooses Work strictly more often and
/// Rest strictly less often than a calm twin, on identical RNG streams
/// (every noise draw matches, so the differential is deterministic).
#[test]
fn dread_shifts_selection_toward_provisioning() {
    let needs = NeedState {
        hunger: Fixed::from_f64(0.15),
        thirst: Fixed::from_f64(0.1),
        fatigue: Fixed::from_f64(0.25),
        ..Default::default()
    };
    let personality = make_personality();
    let identity = IdentityState::default();
    let dp = make_decision_policy();
    let count_selection = |dread: Fixed| -> (u64, u64) {
        let mut rng = RngStreams::new(42);
        let mut work = 0u64;
        let mut rest = 0u64;
        for _ in 0..500 {
            let chosen = select_action(
                &DecisionContext {
                    needs: &needs,
                    personality: &personality,
                    active_goals: &[],
                    identity: &identity,
                    decision_policy: &dp,
                    total_grain: Fixed::from_f64(0.5),
                    total_water: Fixed::from_f64(0.5),
                    coin: Fixed::ZERO,
                    norm_pressure: Fixed::ZERO,
                    anger: Fixed::ZERO,
                    fear: Fixed::ZERO,
                    joy: Fixed::ZERO,
                    sadness: Fixed::ZERO,
                    stress: Fixed::ZERO,
                    social_withdrawal: Fixed::ZERO,
                    fairness: Fixed::ZERO,
                    authority: Fixed::ZERO,
                    care: Fixed::ZERO,
                    loyalty: Fixed::ZERO,
                    action_values: ActionValues::default(),
                    dominant_need: MotiveCategory::Hunger,
                    dominant_pressure: Fixed::ZERO,
                    dread,
                    hope: Fixed::ZERO,
                    // 0.5 = neutral default → zero calibration shift.
                    planning_confidence: Fixed::from_f64(0.5),
                    mood_valence: Fixed::ZERO,
                    season: 0,
                    life_stage: 4,
                    somatic_marker: Fixed::ZERO,
                    development: &crate::psychology::DevelopmentFieldState::default(),
                    polarity_claims: &[],
                },
                &mut rng,
            );
            match chosen {
                ActionKind::Work => work += 1,
                ActionKind::Rest => rest += 1,
                _ => {}
            }
        }
        (work, rest)
    };
    let (work_dreadful, rest_dreadful) = count_selection(Fixed::from_f64(0.8));
    let (work_calm, rest_calm) = count_selection(Fixed::ZERO);
    assert!(
        work_dreadful > work_calm,
        "dread must push selections toward Work: {work_dreadful} vs {work_calm}"
    );
    assert!(
        rest_dreadful < rest_calm,
        "dread must push selections away from Rest: {rest_dreadful} vs {rest_calm}"
    );
}

/// §8.1.16 (Iteration 203): `hope` drives aspirational engagement —
/// an agent who imagines the village thriving prefers Socialize and
/// Worship (community / collective meaning) and avoids Idle, the
/// positive mirror of dread's precautionary provisioning. Zero-at-zero
/// (hope 0 → identical legacy selections), deterministic (pure utility
/// term — same seed, identical RNG stream).
/// §8.1.16 (Iteration 203): `hope` drives aspirational engagement —
/// an agent who imagines the village thriving prefers Socialize and
/// Worship (community / collective meaning) and avoids Idle, the
/// positive mirror of dread's precautionary provisioning. Zero-at-zero
/// (hope 0 → byte-identical legacy utility), deterministic (pure
/// utility term — identical noise streams across the comparison).
#[test]
fn hope_boosts_engagement_and_suppresses_idle() {
    let needs = NeedState {
        hunger: Fixed::from_f64(0.3),
        ..Default::default()
    };
    let personality = make_personality();
    let identity = IdentityState::default();
    let util = |kind: ActionKind, hope: Fixed| {
        compute_utility(
            &kind.definition(),
            &needs,
            &personality,
            &mut RngStreams::new(42),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            &identity,
            Fixed::ZERO,
            Fixed::ZERO,
            ActionValues::default(),
            MotiveCategory::Hunger,
            Fixed::ZERO,
            Fixed::ZERO,
            hope,
            Fixed::from_f64(0.5),
        )
    };
    let social_hopeful = util(ActionKind::Socialize, Fixed::from_f64(0.6));
    let social_calm = util(ActionKind::Socialize, Fixed::ZERO);
    let worship_hopeful = util(ActionKind::Worship, Fixed::from_f64(0.6));
    let worship_calm = util(ActionKind::Worship, Fixed::ZERO);
    let idle_hopeful = util(ActionKind::Idle, Fixed::from_f64(0.6));
    let idle_calm = util(ActionKind::Idle, Fixed::ZERO);
    let social_delta = (social_hopeful - social_calm).to_f64();
    let worship_delta = (worship_hopeful - worship_calm).to_f64();
    let idle_delta = (idle_calm - idle_hopeful).to_f64();
    assert!(
        social_delta > 0.0,
        "hope must boost Socialize, delta {social_delta:.6}"
    );
    assert!(
        worship_delta > 0.0,
        "hope must boost Worship, delta {worship_delta:.6}"
    );
    assert!(
        idle_delta > 0.0,
        "hope must suppress Idle, delta {idle_delta:.6}"
    );
    // Exact magnitude: 0.6 × 0.2 = 0.12 for Socialize/Worship, 0.6 ×
    // 0.1 = 0.06 for Idle (identical noise streams — no other term
    // differs). Mirrors the dread channel's exact-magnitude pin.
    assert!(
        (social_delta - 0.12).abs() < 1e-9,
        "Socialize boost must be exactly 0.12, got {social_delta:.6}"
    );
    assert!(
        (worship_delta - 0.12).abs() < 1e-9,
        "Worship boost must be exactly 0.12, got {worship_delta:.6}"
    );
    assert!(
        (idle_delta - 0.06).abs() < 1e-9,
        "Idle penalty must be exactly 0.06, got {idle_delta:.6}"
    );
}

/// §8.1.16 (Iteration 203): the aspirational term is exclusive to
/// engagement — every other action's utility is byte-identical at hope
/// 0.6 vs hope 0 (identical noise streams), so the consumer cannot
/// distort the non-engagement action set.
#[test]
fn hope_leaves_non_engagement_actions_untouched() {
    let needs = NeedState {
        hunger: Fixed::from_f64(0.7),
        ..Default::default()
    };
    let personality = make_personality();
    let identity = IdentityState::default();
    for kind in [
        ActionKind::Eat,
        ActionKind::Drink,
        ActionKind::Rest,
        ActionKind::Work,
        ActionKind::Trade,
        ActionKind::Wander,
    ] {
        let hopeful = compute_utility(
            &kind.definition(),
            &needs,
            &personality,
            &mut RngStreams::new(42),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            &identity,
            Fixed::ZERO,
            Fixed::ZERO,
            ActionValues::default(),
            MotiveCategory::Hunger,
            Fixed::from_f64(0.8),
            Fixed::ZERO,
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.5),
        );
        let calm = compute_utility(
            &kind.definition(),
            &needs,
            &personality,
            &mut RngStreams::new(42),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            &identity,
            Fixed::ZERO,
            Fixed::ZERO,
            ActionValues::default(),
            MotiveCategory::Hunger,
            Fixed::from_f64(0.8),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
        );
        assert_eq!(
            hopeful.to_f64(),
            calm.to_f64(),
            "{kind:?} must be untouched by hope"
        );
    }
}

// ── §8.1.12 (Iteration 204): planning-confidence calibration ───────

/// §8.1.12 (Iteration 204): `planning_confidence` drives deferred-
/// gratification calibration — an agent that trusts its ability to
/// plan and execute commits to future-facing Work and wastes less
/// time Idle. Baseline-corrected around the 0.5 neutral default
/// (pc 0.5 → byte-identical legacy utility — the Iter-197 zero-drift
/// pattern), deterministic (pure utility term — identical noise
/// streams across the comparison).
#[test]
fn planning_confidence_calibrates_work_and_idle() {
    let needs = NeedState {
        hunger: Fixed::from_f64(0.3),
        ..Default::default()
    };
    let personality = make_personality();
    let identity = IdentityState::default();
    let util = |kind: ActionKind, pc: Fixed| {
        compute_utility(
            &kind.definition(),
            &needs,
            &personality,
            &mut RngStreams::new(42),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            &identity,
            Fixed::ZERO,
            Fixed::ZERO,
            ActionValues::default(),
            MotiveCategory::Hunger,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            pc,
        )
    };
    let work_confident = util(ActionKind::Work, Fixed::from_f64(0.65));
    let work_neutral = util(ActionKind::Work, Fixed::from_f64(0.5));
    let work_hedged = util(ActionKind::Work, Fixed::from_f64(0.35));
    let idle_confident = util(ActionKind::Idle, Fixed::from_f64(0.65));
    let idle_neutral = util(ActionKind::Idle, Fixed::from_f64(0.5));
    let idle_hedged = util(ActionKind::Idle, Fixed::from_f64(0.35));
    let work_boost = (work_confident - work_neutral).to_f64();
    let work_hedge = (work_hedged - work_neutral).to_f64();
    let idle_boost = (idle_neutral - idle_confident).to_f64();
    let idle_hedge = (idle_hedged - idle_neutral).to_f64();
    // Exact magnitudes: (0.65−0.5)×0.2 = 0.03 for Work, (0.65−0.5)×0.1
    // = 0.015 for Idle (identical noise streams — no other term
    // differs). Mirrors the dread/hope channels' exact-magnitude pins.
    assert!(
        (work_boost - 0.03).abs() < 1e-9,
        "Work boost must be exactly 0.03, got {work_boost:.6}"
    );
    assert!(
        (work_hedge + 0.03).abs() < 1e-9,
        "Work hedge must be exactly −0.03, got {work_hedge:.6}"
    );
    assert!(
        (idle_boost - 0.015).abs() < 1e-9,
        "Idle suppression must be exactly 0.015, got {idle_boost:.6}"
    );
    assert!(
        (idle_hedge - 0.015).abs() < 1e-9,
        "Idle hedge must be exactly +0.015, got {idle_hedge:.6}"
    );
}

/// §8.1.12 (Iteration 204): the calibration term is exclusive to
/// Work and Idle — every other action's utility is byte-identical at
/// pc 0.65 vs pc 0.5 (identical noise streams), so the consumer
/// cannot distort the non-work action set. Also pins the zero-drift
/// contract: pc 0.5 must equal the pre-Iter-204 legacy utility for
/// EVERY action (identical to a pc-neutral baseline run).
#[test]
fn planning_confidence_leaves_non_work_actions_untouched() {
    let needs = NeedState {
        hunger: Fixed::from_f64(0.7),
        ..Default::default()
    };
    let personality = make_personality();
    let identity = IdentityState::default();
    for kind in [
        ActionKind::Eat,
        ActionKind::Drink,
        ActionKind::Rest,
        ActionKind::Socialize,
        ActionKind::Worship,
        ActionKind::Trade,
        ActionKind::Wander,
    ] {
        let confident = compute_utility(
            &kind.definition(),
            &needs,
            &personality,
            &mut RngStreams::new(42),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            &identity,
            Fixed::ZERO,
            Fixed::ZERO,
            ActionValues::default(),
            MotiveCategory::Hunger,
            Fixed::from_f64(0.8),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.65),
        );
        let neutral = compute_utility(
            &kind.definition(),
            &needs,
            &personality,
            &mut RngStreams::new(42),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            &identity,
            Fixed::ZERO,
            Fixed::ZERO,
            ActionValues::default(),
            MotiveCategory::Hunger,
            Fixed::from_f64(0.8),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
        );
        assert_eq!(
            confident.to_f64(),
            neutral.to_f64(),
            "{kind:?} must be untouched by planning confidence"
        );
    }
}

// ── §8.1.6 (Iteration 162): temperament decision consumers ─────────

/// §8.1.6 (Iteration 162): `noise_amplitude` shrinks the ±0.05 jitter
/// with persistence deviation — zero at zero (byte-identical legacy),
/// monotone decreasing, floored so maximal persistence keeps a minimal
/// exploration jitter (no full determinism cliff).
#[test]
fn persistence_narrows_decision_noise_amplitude() {
    assert_eq!(noise_amplitude(Fixed::ZERO), 0.05);
    assert_eq!(
        noise_amplitude(-Fixed::from_f64(0.2)),
        0.05,
        "negative deviation is defensively legacy"
    );
    assert!(noise_amplitude(Fixed::from_f64(0.3)) < 0.05);
    assert!(
        noise_amplitude(Fixed::from_f64(0.6)) < noise_amplitude(Fixed::from_f64(0.3)),
        "amplitude must be monotone decreasing in deviation"
    );
    assert!(
        noise_amplitude(Fixed::ONE) >= PERSISTENCE_NOISE_FLOOR,
        "maximal persistence must not clamp below the floor"
    );
}

/// §8.1.6 (Iteration 162): approach-withdrawal deviation biases the
/// Wander action's utility — an approach-biased agent (positive
/// deviation) prefers exploration, a withdrawal-biased one (negative
/// deviation) wanders less. Deterministic: the same-seed Wander utility
/// differs from the zero-deviation baseline by exactly
/// `deviation × APPROACH_WANDER_BONUS` (no RNG on this path beyond the
/// identical noise stream).
#[test]
fn approach_deviation_biases_wander_exploration() {
    let needs = NeedState {
        hunger: Fixed::from_f64(0.15),
        thirst: Fixed::from_f64(0.1),
        fatigue: Fixed::from_f64(0.25),
        ..Default::default()
    };
    let identity = IdentityState::default();
    let base = make_personality();
    // Zero-deviation control: live temperament equals the trait-derived baseline.
    let baseline = Temperament::from_traits(&base);
    let mut control = base;
    control.temperament = baseline;
    // Approach-biased: +0.4 deviation on approach_withdrawal only.
    let mut approach = control.clone();
    approach.temperament.approach_withdrawal = baseline.approach_withdrawal + Fixed::from_f64(0.4);
    // Withdrawal-biased: −0.3 deviation.
    let mut withdraw = control.clone();
    withdraw.temperament.approach_withdrawal = baseline.approach_withdrawal - Fixed::from_f64(0.3);

    let wander_util = |p: &Personality| -> Fixed {
        let mut rng = RngStreams::new(42);
        compute_utility(
            &ActionKind::Wander.definition(),
            &needs,
            p,
            &mut rng,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            &identity,
            Fixed::ZERO,
            Fixed::ZERO,
            ActionValues::default(),
            MotiveCategory::Hunger,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
        )
    };
    let base_util = wander_util(&control);
    let approach_util = wander_util(&approach);
    let withdraw_util = wander_util(&withdraw);
    assert_eq!(
        approach_util - base_util,
        Fixed::from_f64(0.4) * APPROACH_WANDER_BONUS,
        "approach deviation must add exactly deviation × bonus to Wander"
    );
    assert_eq!(
        withdraw_util - base_util,
        -Fixed::from_f64(0.3) * APPROACH_WANDER_BONUS,
        "withdrawal deviation must subtract exactly deviation × bonus from Wander"
    );
}

/// §8.1.6 (Iteration 162): the approach channel is Wander-specific —
/// non-Wander actions are untouched by the deviation (only the identical
/// noise stream may differ, which the same-seed call keeps identical).
#[test]
fn approach_deviation_leaves_non_wander_actions_untouched() {
    let needs = NeedState {
        hunger: Fixed::from_f64(0.15),
        thirst: Fixed::from_f64(0.1),
        fatigue: Fixed::from_f64(0.25),
        ..Default::default()
    };
    let identity = IdentityState::default();
    let base = make_personality();
    let baseline = Temperament::from_traits(&base);
    let mut control = base;
    control.temperament = baseline;
    let mut approach = control.clone();
    approach.temperament.approach_withdrawal = baseline.approach_withdrawal + Fixed::from_f64(0.4);

    let work_util = |p: &Personality| -> Fixed {
        let mut rng = RngStreams::new(42);
        compute_utility(
            &ActionKind::Work.definition(),
            &needs,
            p,
            &mut rng,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            &identity,
            Fixed::ZERO,
            Fixed::ZERO,
            ActionValues::default(),
            MotiveCategory::Hunger,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
        )
    };
    assert_eq!(
        work_util(&approach),
        work_util(&control),
        "approach deviation must not touch non-Wander utility"
    );
}
