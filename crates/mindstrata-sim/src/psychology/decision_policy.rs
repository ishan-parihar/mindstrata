//! Architecture-plan-2 §8.1.20 — Decision Policy System.
//!
//! Integrates all psychology into action selection. The policy modulates
//! utility computation based on personality, emotions, moral cognition,
//! risk tolerance, habit strength, and learned experience. This makes
//! agents feel like adaptive intelligence rather than static utility functions.
//!
//! ```text
//! Biology → Needs → Utility Base
//! Personality → Weight Modulation
//! Emotions → Risk/Bias Adjustment
//! Moral Cognition → Fairness/Authority Weight
//! Habits → Routine Inertia
//! Culture → Taboo/Identity Constraints
//! Experience → Learned Weight Adjustment
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Weights that scale different utility components during action selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilityWeights {
    /// How much need relief matters in utility (0–1).
    pub need_relief: Fixed,
    /// How much emotional relief matters (0–1).
    pub emotional_relief: Fixed,
    /// How much social value matters (0–1).
    pub social_value: Fixed,
    /// How much identity congruence matters (0–1).
    pub identity_congruence: Fixed,
    /// How much cost matters as a penalty (0–1).
    pub cost_penalty: Fixed,
    /// How much novelty/stimulation matters (0–1).
    pub novelty: Fixed,
}

impl Default for UtilityWeights {
    fn default() -> Self {
        Self {
            need_relief: Fixed::from_f64(0.4),
            emotional_relief: Fixed::from_f64(0.2),
            social_value: Fixed::from_f64(0.15),
            identity_congruence: Fixed::from_f64(0.1),
            cost_penalty: Fixed::from_f64(0.1),
            novelty: Fixed::from_f64(0.05),
        }
    }
}

/// Moral weights that scale fairness/authority/care contributions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoralWeights {
    /// How much fairness considerations bias action selection (0–1).
    pub fairness: Fixed,
    /// How much authority/obedience considerations bias selection (0–1).
    pub authority: Fixed,
    /// How much care/harm considerations bias selection (0–1).
    pub care: Fixed,
    /// How much loyalty/in-group considerations bias selection (0–1).
    pub loyalty: Fixed,
}

impl Default for MoralWeights {
    fn default() -> Self {
        Self {
            fairness: Fixed::from_f64(0.3),
            authority: Fixed::from_f64(0.2),
            care: Fixed::from_f64(0.3),
            loyalty: Fixed::from_f64(0.2),
        }
    }
}

/// Risk policy — how risk tolerance modulates action selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskPolicy {
    /// Base risk tolerance (0–1). Higher = more risk-seeking.
    pub risk_tolerance: Fixed,
    /// How much fear reduces risk tolerance (0–1).
    pub fear_sensitivity: Fixed,
    /// How much trauma reduces risk tolerance (0–1).
    pub trauma_sensitivity: Fixed,
}

impl Default for RiskPolicy {
    fn default() -> Self {
        Self {
            risk_tolerance: Fixed::from_f64(0.5),
            fear_sensitivity: Fixed::from_f64(0.6),
            trauma_sensitivity: Fixed::from_f64(0.4),
        }
    }
}

/// Social policy — cooperation and reciprocity tendencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialPolicy {
    /// Base cooperation tendency (0–1).
    pub cooperation: Fixed,
    /// Reciprocity strength — how much past help influences future cooperation (0–1).
    pub reciprocity: Fixed,
    /// How much group identity amplifies social utility (0–1).
    pub group_amplification: Fixed,
}

impl Default for SocialPolicy {
    fn default() -> Self {
        Self {
            cooperation: Fixed::from_f64(0.5),
            reciprocity: Fixed::from_f64(0.4),
            group_amplification: Fixed::from_f64(0.3),
        }
    }
}

/// Habit policy — routine adherence and inertia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitPolicy {
    /// How strongly habits influence action selection (0–1).
    pub habit_strength: Fixed,
    /// How much stress increases reliance on habits (0–1).
    pub stress_habit_reliance: Fixed,
}

impl Default for HabitPolicy {
    fn default() -> Self {
        Self {
            habit_strength: Fixed::from_f64(0.4),
            stress_habit_reliance: Fixed::from_f64(0.3),
        }
    }
}

/// Emotional policy — how emotions bias action selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalPolicy {
    /// How much anger biases toward aggressive/confrontational actions (0–1).
    pub anger_bias: Fixed,
    /// How much fear biases toward safe/avoidant actions (0–1).
    pub fear_bias: Fixed,
    /// How much joy biases toward prosocial/expanded actions (0–1).
    pub joy_bias: Fixed,
    /// How much sadness biases toward withdrawal (0–1).
    pub sadness_bias: Fixed,
}

impl Default for EmotionalPolicy {
    fn default() -> Self {
        Self {
            anger_bias: Fixed::from_f64(0.3),
            fear_bias: Fixed::from_f64(0.4),
            joy_bias: Fixed::from_f64(0.2),
            sadness_bias: Fixed::from_f64(0.2),
        }
    }
}

/// §8.1.20: Decision Policy — integrates all psychology into action selection.
///
/// The policy is personality-initialized and learns from outcomes.
/// It modulates utility computation based on the full psychological state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPolicy {
    pub utility_weights: UtilityWeights,
    pub moral_weights: MoralWeights,
    pub risk_policy: RiskPolicy,
    pub social_policy: SocialPolicy,
    pub habit_policy: HabitPolicy,
    pub emotional_policy: EmotionalPolicy,
    /// Running average of action success rate (for learning).
    pub success_rate: Fixed,
    /// Running average of action cost efficiency (for learning).
    pub cost_efficiency: Fixed,
    /// Number of decisions made (for learning convergence).
    pub decision_count: u64,
}

impl Default for DecisionPolicy {
    fn default() -> Self {
        Self {
            utility_weights: UtilityWeights::default(),
            moral_weights: MoralWeights::default(),
            risk_policy: RiskPolicy::default(),
            social_policy: SocialPolicy::default(),
            habit_policy: HabitPolicy::default(),
            emotional_policy: EmotionalPolicy::default(),
            success_rate: Fixed::from_f64(0.5),
            cost_efficiency: Fixed::from_f64(0.5),
            decision_count: 0,
        }
    }
}

impl DecisionPolicy {
    /// Initialize from personality traits. Different personality profiles
    /// produce different decision-making styles.
    pub fn from_personality(
        neuroticism: Fixed,
        extraversion: Fixed,
        conscientiousness: Fixed,
        openness: Fixed,
        agreeableness: Fixed,
        risk_tolerance: Fixed,
    ) -> Self {
        Self {
            utility_weights: UtilityWeights {
                // Neurotic agents weight emotional relief more; extraverts weight social value.
                emotional_relief: Fixed::from_f64(0.15) + neuroticism * Fixed::from_f64(0.1),
                social_value: Fixed::from_f64(0.1) + extraversion * Fixed::from_f64(0.1),
                novelty: Fixed::from_f64(0.03) + openness * Fixed::from_f64(0.07),
                identity_congruence: Fixed::from_f64(0.08) + conscientiousness * Fixed::from_f64(0.05),
                ..UtilityWeights::default()
            },
            moral_weights: MoralWeights {
                fairness: Fixed::from_f64(0.2) + agreeableness * Fixed::from_f64(0.15),
                care: Fixed::from_f64(0.2) + agreeableness * Fixed::from_f64(0.15),
                authority: Fixed::from_f64(0.15) + conscientiousness * Fixed::from_f64(0.1),
                loyalty: Fixed::from_f64(0.15) + agreeableness * Fixed::from_f64(0.1),
            },
            risk_policy: RiskPolicy {
                risk_tolerance,
                // Neurotic agents are more fear-sensitive.
                fear_sensitivity: Fixed::from_f64(0.4) + neuroticism * Fixed::from_f64(0.3),
                // Low openness → more trauma-sensitive (rigid response).
                trauma_sensitivity: Fixed::from_f64(0.5) - openness * Fixed::from_f64(0.2),
            },
            social_policy: SocialPolicy {
                // Agreeable agents cooperate more.
                cooperation: Fixed::from_f64(0.3) + agreeableness * Fixed::from_f64(0.3),
                // Extraverts have stronger reciprocity.
                reciprocity: Fixed::from_f64(0.3) + extraversion * Fixed::from_f64(0.2),
                group_amplification: Fixed::from_f64(0.2) + agreeableness * Fixed::from_f64(0.2),
            },
            habit_policy: HabitPolicy {
                // Conscientious agents have stronger habits.
                habit_strength: Fixed::from_f64(0.25) + conscientiousness * Fixed::from_f64(0.25),
                stress_habit_reliance: Fixed::from_f64(0.2) + neuroticism * Fixed::from_f64(0.2),
            },
            emotional_policy: EmotionalPolicy {
                anger_bias: Fixed::from_f64(0.2) + (Fixed::ONE - agreeableness) * Fixed::from_f64(0.2),
                fear_bias: Fixed::from_f64(0.3) + neuroticism * Fixed::from_f64(0.2),
                joy_bias: Fixed::from_f64(0.15) + extraversion * Fixed::from_f64(0.15),
                sadness_bias: Fixed::from_f64(0.15) + neuroticism * Fixed::from_f64(0.15),
            },
            success_rate: Fixed::from_f64(0.5),
            cost_efficiency: Fixed::from_f64(0.5),
            decision_count: 0,
        }
    }

    /// Compute effective risk tolerance given current emotional state.
    ///
    /// Fear and trauma reduce effective risk tolerance. This means
    /// frightened agents prefer safer actions even if they're normally
    /// risk-seeking.
    pub fn effective_risk_tolerance(&self, fear: Fixed, trauma_load: Fixed) -> Fixed {
        let reduced_by_fear = fear * self.risk_policy.fear_sensitivity;
        let reduced_by_trauma = trauma_load * self.risk_policy.trauma_sensitivity;
        (self.risk_policy.risk_tolerance - reduced_by_fear - reduced_by_trauma).clamp_01()
    }

    /// Compute emotional bias modifier for a given action.
    ///
    /// Returns a modifier in [-0.3, +0.3] that adjusts utility based
    /// on current emotional state and the action's emotional alignment.
    pub fn emotional_modifier(
        &self,
        anger: Fixed,
        fear: Fixed,
        joy: Fixed,
        sadness: Fixed,
        action_is_social: bool,
        action_is_risky: bool,
        action_is_withdrawal: bool,
    ) -> Fixed {
        let mut modifier = Fixed::ZERO;

        // Anger biases toward non-social/risky actions (aggression, confrontation)
        if action_is_risky && !action_is_social {
            modifier += anger * self.emotional_policy.anger_bias;
        }
        // Fear biases toward safe/social actions (retreat, seeking comfort)
        if !action_is_risky {
            modifier += fear * self.emotional_policy.fear_bias;
        }
        // Joy biases toward prosocial/expanded actions
        if action_is_social {
            modifier += joy * self.emotional_policy.joy_bias;
        }
        // Sadness biases toward withdrawal
        if action_is_withdrawal {
            modifier += sadness * self.emotional_policy.sadness_bias;
        }

        // Clamp to prevent extreme emotional takeovers
        modifier.clamp(Fixed::from_f64(-0.3), Fixed::from_f64(0.3))
    }

    /// Compute moral modifier for a given action based on moral values.
    ///
    /// Returns a modifier that adjusts utility based on the action's
    /// alignment with the agent's moral foundations.
    pub fn moral_modifier(
        &self,
        fairness: Fixed,
        authority: Fixed,
        care: Fixed,
        _loyalty: Fixed,
        action_is_prosocial: bool,
        action_is_disobedient: bool,
        action_is_harmful: bool,
    ) -> Fixed {
        let mut modifier = Fixed::ZERO;

        if action_is_prosocial {
            modifier += fairness * self.moral_weights.fairness * Fixed::from_f64(0.2);
            modifier += care * self.moral_weights.care * Fixed::from_f64(0.2);
        }
        if action_is_disobedient {
            modifier -= authority * self.moral_weights.authority * Fixed::from_f64(0.2);
        }
        if action_is_harmful {
            modifier -= care * self.moral_weights.care * Fixed::from_f64(0.3);
        }

        modifier.clamp(Fixed::from_f64(-0.2), Fixed::from_f64(0.2))
    }

    /// Compute habit modifier — how much routine/habit influences this action.
    ///
    /// Higher stress increases habit reliance (agents fall back on routines).
    pub fn habit_modifier(&self, is_routine_action: bool, stress: Fixed) -> Fixed {
        if !is_routine_action {
            return Fixed::ZERO;
        }
        let base = self.habit_policy.habit_strength;
        let stress_boost = stress * self.habit_policy.stress_habit_reliance;
        (base + stress_boost).clamp_01() * Fixed::from_f64(0.3)
    }

    /// Learn from an action outcome. Adjusts success_rate and cost_efficiency
    /// using exponential moving average (converges after ~20 decisions).
    pub fn learn_from_outcome(&mut self, success: bool, cost: Fixed) {
        self.decision_count += 1;
        let alpha = Fixed::from_f64(0.05); // learning rate

        if success {
            self.success_rate = self.success_rate + alpha * (Fixed::ONE - self.success_rate);
        } else {
            self.success_rate = self.success_rate - alpha * self.success_rate;
        }

        // Cost efficiency: lower cost = higher efficiency
        let efficiency = (Fixed::ONE - cost).max(Fixed::ZERO);
        self.cost_efficiency = self.cost_efficiency + alpha * (efficiency - self.cost_efficiency);
    }

    /// Update the policy's weight vectors based on long-term learning.
    /// Called daily to slowly shift decision-making style based on experience.
    pub fn daily_update(&mut self) {
        // Successful agents become slightly more cost-conscious (learn efficiency)
        let cost_learn = (self.success_rate - Fixed::from_f64(0.5)) * Fixed::from_f64(0.001);
        self.utility_weights.cost_penalty =
            (self.utility_weights.cost_penalty + cost_learn).clamp_01();

        // Agents with high cost efficiency slightly increase social weighting
        // (cooperation tends to be efficient in village settings)
        let social_learn = (self.cost_efficiency - Fixed::from_f64(0.5)) * Fixed::from_f64(0.0005);
        self.social_policy.cooperation =
            (self.social_policy.cooperation + social_learn).clamp_01();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_decision_policy_has_balanced_weights() {
        let dp = DecisionPolicy::default();
        assert!(dp.utility_weights.need_relief > Fixed::ZERO);
        assert!(dp.risk_policy.risk_tolerance > Fixed::ZERO);
        assert_eq!(dp.success_rate, Fixed::from_f64(0.5));
    }

    #[test]
    fn from_personality_neurotic_has_higher_fear_sensitivity() {
        let dp_high = DecisionPolicy::from_personality(
            Fixed::from_f64(0.8), // neuroticism
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
        );
        let dp_low = DecisionPolicy::from_personality(
            Fixed::from_f64(0.2), // neuroticism
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
        );
        assert!(dp_high.risk_policy.fear_sensitivity > dp_low.risk_policy.fear_sensitivity);
        assert!(dp_high.emotional_policy.fear_bias > dp_low.emotional_policy.fear_bias);
    }

    #[test]
    fn from_personality_agreeable_has_higher_cooperation() {
        let dp = DecisionPolicy::from_personality(
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.8), // agreeableness
            Fixed::from_f64(0.5),
        );
        assert!(dp.social_policy.cooperation > Fixed::from_f64(0.5));
        assert!(dp.moral_weights.care > Fixed::from_f64(0.3));
    }

    #[test]
    fn effective_risk_tolerance_reduces_under_fear() {
        let dp = DecisionPolicy::default();
        let base = dp.risk_policy.risk_tolerance;
        let under_fear = dp.effective_risk_tolerance(Fixed::from_f64(0.5), Fixed::ZERO);
        assert!(under_fear < base);
    }

    #[test]
    fn effective_risk_tolerance_reduces_under_trauma() {
        let dp = DecisionPolicy::default();
        let base = dp.risk_policy.risk_tolerance;
        let under_trauma = dp.effective_risk_tolerance(Fixed::ZERO, Fixed::from_f64(0.5));
        assert!(under_trauma < base);
    }

    #[test]
    fn emotional_modifier_biases_social_actions_under_joy() {
        let dp = DecisionPolicy::default();
        let mod_social = dp.emotional_modifier(
            Fixed::ZERO, Fixed::ZERO,
            Fixed::from_f64(0.7), Fixed::ZERO,
            true, false, false,
        );
        let mod_nonsocial = dp.emotional_modifier(
            Fixed::ZERO, Fixed::ZERO,
            Fixed::from_f64(0.7), Fixed::ZERO,
            false, false, false,
        );
        assert!(mod_social > mod_nonsocial);
    }

    #[test]
    fn emotional_modifier_biases_safe_actions_under_fear() {
        let dp = DecisionPolicy::default();
        let mod_safe = dp.emotional_modifier(
            Fixed::ZERO, Fixed::from_f64(0.7),
            Fixed::ZERO, Fixed::ZERO,
            false, false, false,
        );
        let mod_risky = dp.emotional_modifier(
            Fixed::ZERO, Fixed::from_f64(0.7),
            Fixed::ZERO, Fixed::ZERO,
            false, true, false,
        );
        assert!(mod_safe > mod_risky);
    }

    #[test]
    fn moral_modifier_reduces_harmful_actions() {
        let dp = DecisionPolicy::default();
        let mod_harmful = dp.moral_modifier(
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.5),
            false, false, true,
        );
        assert!(mod_harmful < Fixed::ZERO);
    }

    #[test]
    fn moral_modifier_boosts_prosocial_actions() {
        let dp = DecisionPolicy::default();
        let mod_prosocial = dp.moral_modifier(
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.5),
            true, false, false,
        );
        assert!(mod_prosocial > Fixed::ZERO);
    }

    #[test]
    fn habit_modifier_boosts_routine_actions_under_stress() {
        let dp = DecisionPolicy::default();
        let mod_routine_stress = dp.habit_modifier(true, Fixed::from_f64(0.7));
        let mod_routine_calm = dp.habit_modifier(true, Fixed::from_f64(0.1));
        assert!(mod_routine_stress > mod_routine_calm);
    }

    #[test]
    fn habit_modifier_zero_for_non_routine() {
        let dp = DecisionPolicy::default();
        let mod_nonroutine = dp.habit_modifier(false, Fixed::from_f64(0.9));
        assert_eq!(mod_nonroutine, Fixed::ZERO);
    }

    #[test]
    fn learn_from_outcome_increases_success_rate() {
        let mut dp = DecisionPolicy::default();
        let initial = dp.success_rate;
        for _ in 0..10 {
            dp.learn_from_outcome(true, Fixed::from_f64(0.3));
        }
        assert!(dp.success_rate > initial);
        assert_eq!(dp.decision_count, 10);
    }

    #[test]
    fn learn_from_outcome_decreases_success_rate_on_failure() {
        let mut dp = DecisionPolicy::default();
        let initial = dp.success_rate;
        for _ in 0..10 {
            dp.learn_from_outcome(false, Fixed::from_f64(0.3));
        }
        assert!(dp.success_rate < initial);
    }

    #[test]
    fn daily_update_slowly_shifts_weights() {
        let mut dp = DecisionPolicy {
            success_rate: Fixed::from_f64(0.8), // successful agent
            ..Default::default()
        };
        let initial_cost_penalty = dp.utility_weights.cost_penalty;
        for _ in 0..100 {
            dp.daily_update();
        }
        // Successful agents should become slightly more cost-conscious
        assert!(dp.utility_weights.cost_penalty > initial_cost_penalty);
    }
}
