//! Action execution system — utility-based selection and execution.
//!
//! Agents choose actions based on utility scores that combine need relief,
//! emotional relief, social effects, identity congruence, cost, and noise.
//! This produces bounded rationality — agents are not perfectly optimal.

use crate::person::{
    BodyState, Goal, GoalKind, IdentityKind, IdentityState, NeedState, Personality, Temperament,
};
use crate::psychology::neural_like::ActionValues;
use crate::psychology::DecisionPolicy;
use crate::psychology::MotiveCategory;
use mindstrata_core::fixed::Fixed;
use mindstrata_core::rng::{RngStream, RngStreams};
use rand::Rng;
use serde::{Deserialize, Serialize};

// ── §8.1.6 (Iteration 162): temperament decision consumers ──────────
/// How strongly a persistence deviation above the trait-derived baseline
/// narrows the ±0.05 decision-noise roll (0.5 → a 0.3 deviation halves the
/// amplitude). Persistence is the plan's "goal adherence under difficulty":
/// a more persistent agent makes more consistent (less noise-flipped)
/// choices. Zero-at-zero (deviation 0 → legacy ±0.05), the RNG draw stays
/// unconditional (only the amplitude changes — replay determinism holds).
const PERSISTENCE_NOISE_REDUCTION: f64 = 0.5;
/// Floor for the persistence-scaled noise amplitude — even a maximally
/// persistent agent keeps a minimal exploration jitter (no full
/// determinism cliff).
const PERSISTENCE_NOISE_FLOOR: f64 = 0.02;
/// Utility bonus per unit of approach-withdrawal deviation for the Wander
/// action — an approach-biased temperament (deviation above the trait-
/// derived baseline) explores the world more; a withdrawal-biased one
/// (negative deviation) wanders less. Sized small (0.05/unit) so it is a
/// genuine nudge, not a reordering lever: at a typical 0.2–0.5 deviation
/// the term adds ±0.01–0.025 — comparable to the dread channel's nudge.
/// Zero-at-zero (deviation 0 → legacy Wander utility), deterministic
/// (pure utility term — no RNG).
const APPROACH_WANDER_BONUS: Fixed = Fixed::from_raw(500); // 0.05

/// Bundled context for action selection — replaces the 18-parameter signature.
///
/// All fields are either references (borrowed from agent/world state) or
/// `Copy` scalars (emotional readings, moral values, resource levels).
/// The `rng` field is kept separate because it requires `&mut` access.
#[derive(Debug)]
pub struct DecisionContext<'a> {
    /// Agent's current need deficits.
    pub needs: &'a NeedState,
    /// Agent's personality traits.
    pub personality: &'a Personality,
    /// Agent's active goals.
    pub active_goals: &'a [Goal],
    /// Agent's identity state.
    pub identity: &'a IdentityState,
    /// Decision policy integrating all psychology into action.
    pub decision_policy: &'a DecisionPolicy,
    /// World-level grain scarcity (0 = abundant, 1 = depleted).
    pub total_grain: Fixed,
    /// World-level water scarcity (0 = abundant, 1 = depleted).
    pub total_water: Fixed,
    /// Agent's spendable coin — gates Trade utility (broke agents can't buy).
    pub coin: Fixed,
    /// Aggregate norm pressure from all active norms.
    pub norm_pressure: Fixed,
    // ── Emotional readings ──────────────────────────────────────────
    /// Agent's current anger level.
    pub anger: Fixed,
    /// Agent's current fear level.
    pub fear: Fixed,
    /// Agent's current joy level.
    pub joy: Fixed,
    /// Agent's current sadness level.
    pub sadness: Fixed,
    /// Agent's current stress level (fear + anger).
    pub stress: Fixed,
    // ── Moral foundations ───────────────────────────────────────────
    /// Fairness foundation strength.
    pub fairness: Fixed,
    /// Authority foundation strength.
    pub authority: Fixed,
    /// Care foundation strength.
    pub care: Fixed,
    /// Loyalty foundation strength.
    pub loyalty: Fixed,
    // ── §9.2 (Iteration 94): neural-like RL action values ───────────
    /// The agent's learned valuation weights (need/emotional/social/identity
    /// relief), EMA-updated from successful outcomes. The selection loop
    /// folds `learned_delta` against each candidate's outcome profile into
    /// the utility, closing the plan's §9.2 learning → action loop. Passed
    /// by value (Copy, 5 Fixed fields).
    pub action_values: ActionValues,
    // ── §8.1.5 (Iteration 96): dominant-need urgency ────────────────
    /// The argmax of the full five-factor motivation pressure formula
    /// (need × personality × emotion × legitimacy × affordance). Selection
    /// boosts actions that relieve the dominant need.
    pub dominant_need: MotiveCategory,
    /// Pressure of the dominant need (full formula) — scales the urgency boost.
    pub dominant_pressure: Fixed,
    // ── §8.1.16 (Iteration 103): prospection dread ──────────────────
    /// The agent's scenario-grounded dread (0–1) — how much it fears the
    /// imagined bad future. Drives the precautionary-provisioning term.
    pub dread: Fixed,
}

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
    Move {
        target_x: i32,
        target_y: i32,
    },
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

    /// §9.2 (Iteration 94): the outcome-relief profile `(need, emotional,
    /// social, identity)` this action delivers on success — the single
    /// source of truth shared by the RL learning site (`sim.rs`
    /// `learn_from_outcome`) and the Iteration-94 selection consumer, so
    /// what an agent learns is exactly what biases its future choices.
    pub fn outcome_profile(self) -> [Fixed; 4] {
        match self {
            ActionKind::Work => [
                Fixed::from_f64(0.4),
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::from_f64(0.1),
            ],
            ActionKind::Eat | ActionKind::Drink => [
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.1),
                Fixed::ZERO,
                Fixed::ZERO,
            ],
            ActionKind::Socialize => [
                Fixed::ZERO,
                Fixed::from_f64(0.1),
                Fixed::from_f64(0.4),
                Fixed::ZERO,
            ],
            ActionKind::Worship => [
                Fixed::ZERO,
                Fixed::from_f64(0.2),
                Fixed::from_f64(0.1),
                Fixed::from_f64(0.3),
            ],
            ActionKind::Trade => [
                Fixed::from_f64(0.2),
                Fixed::ZERO,
                Fixed::from_f64(0.1),
                Fixed::from_f64(0.1),
            ],
            _ => [
                Fixed::from_f64(0.05),
                Fixed::from_f64(0.05),
                Fixed::from_f64(0.05),
                Fixed::from_f64(0.05),
            ],
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
            hunger_relief: if def.hunger_relief > Fixed::ZERO {
                def.hunger_relief / d
            } else {
                Fixed::ZERO
            },
            thirst_relief: if def.thirst_relief > Fixed::ZERO {
                def.thirst_relief / d
            } else {
                Fixed::ZERO
            },
            fatigue_relief: if def.fatigue_relief > Fixed::ZERO {
                def.fatigue_relief / d
            } else {
                Fixed::ZERO
            },
            social_value: def.social_value,
            energy_cost: if def.energy_cost > Fixed::ZERO {
                def.energy_cost / d
            } else {
                Fixed::ZERO
            },
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
    coin: Fixed,
    action_values: ActionValues,
    dominant_need: MotiveCategory,
    dominant_pressure: Fixed,
    dread: Fixed,
) -> Fixed {
    let mut utility = Fixed::ZERO;

    // §9.1: Nonlinear need pressure — deficit^exponent * personality_modifier.
    // Impulsivity amplifies hunger/thirst pressure, conscientiousness dampens fatigue.
    if action.hunger_relief > Fixed::ZERO {
        let hunger_pressure = needs.hunger * needs.hunger * Fixed::from_f64(2.0); // deficit^1.5 approx
        let hunger_pressure =
            hunger_pressure * (Fixed::ONE + personality.impulsivity * Fixed::from_f64(0.5));
        let mut hunger_util = hunger_pressure * action.hunger_relief * Fixed::from_f64(2.0);
        let scarcity = (Fixed::ONE - total_grain).clamp_01();
        hunger_util += scarcity * needs.hunger * Fixed::from_f64(0.5);
        utility += hunger_util;
    }
    if action.thirst_relief > Fixed::ZERO {
        let thirst_pressure = needs.thirst * needs.thirst * Fixed::from_f64(2.0);
        let thirst_pressure =
            thirst_pressure * (Fixed::ONE + personality.impulsivity * Fixed::from_f64(0.5));
        let mut thirst_util = thirst_pressure * action.thirst_relief * Fixed::from_f64(2.5);
        let scarcity = (Fixed::ONE - total_water).clamp_01();
        thirst_util += scarcity * needs.thirst * Fixed::from_f64(0.5);
        utility += thirst_util;
    }
    if action.fatigue_relief > Fixed::ZERO {
        let fatigue_pressure = needs.fatigue
            * needs.fatigue
            * (Fixed::ONE - personality.conscientiousness * Fixed::from_f64(0.3));
        utility += fatigue_pressure * action.fatigue_relief * Fixed::from_f64(1.5);
    }
    if action.social_value > Fixed::ZERO {
        utility += needs.social * action.social_value * personality.extraversion;
    }
    if action.bonus_meaning_relief > Fixed::ZERO {
        utility += needs.meaning * action.bonus_meaning_relief * Fixed::from_f64(1.2);
    }

    // §8.1.5 (Iteration 96): dominant-need urgency boost — the argmax of
    // the full five-factor pressure formula (need × personality × emotion
    // × legitimacy × affordance) now biases selection. Only actions that
    // relieve the dominant need receive a boost scaled by its pressure.
    // Abstract dominant needs with no relief channel (Safety, Esteem, ...)
    // yield zero boost by design: a fear-dominated agent's selection is
    // steered by its fear emotion modifier (withdrawal bias toward Rest,
    // risk aversion) rather than a food-fixation bonus — the argmax
    // exclusivity withholds the urgency nudge rather than actively
    // suppressing need-seeking.
    if dominant_pressure > Fixed::ZERO {
        let urgent = match dominant_need {
            MotiveCategory::Hunger => action.hunger_relief > Fixed::ZERO,
            MotiveCategory::Thirst => action.thirst_relief > Fixed::ZERO,
            MotiveCategory::Sleep => action.fatigue_relief > Fixed::ZERO,
            MotiveCategory::Meaning => action.bonus_meaning_relief > Fixed::ZERO,
            _ => false,
        };
        if urgent {
            utility += dominant_pressure * Fixed::from_f64(0.4);
        }
    }

    // §8.1.16 (Iteration 103): precautionary provisioning — an agent who
    // daily imagines the harvest failing (scenario-grounded dread) prepares
    // for it: Work and Trade (grain-seeking, stocking up) gain utility and
    // Rest loses it. Zero-at-zero (dread 0 → exact legacy utility),
    // deterministic (pure utility term — the RNG stream is untouched), and
    // sized small (0.2/0.1) so it is a genuine nudge, not a reordering
    // lever: a dread 0.4 agent's Work gains 0.08, comparable to the §8.1.5
    // urgency boost at low pressure.
    if dread > Fixed::ZERO {
        match action.kind {
            ActionKind::Work | ActionKind::Trade => {
                utility += dread * Fixed::from_f64(0.2);
            }
            ActionKind::Rest => {
                utility -= dread * Fixed::from_f64(0.1);
            }
            _ => {}
        }
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

    // §13.3: Trade utility — agents trade when they have coin to spend and
    // needs are pressing. Previously this bonus was tiny (ambition*0.3 +
    // needs*0.4*0.5) so Trade never beat Eat/Drink/Work and the market had
    // zero volume. Trade is now genuinely competitive — but ONLY for agents
    // who can afford it: an agent with zero coin must not prefer buying
    // over eating what it can forage.
    if action.kind == ActionKind::Trade && coin > Fixed::ZERO {
        let coin_pressure = (needs.hunger + needs.thirst) * Fixed::from_f64(0.8);
        let coin_utility = personality.ambition * Fixed::from_f64(0.4);
        let need_pressure_bonus = coin_pressure * Fixed::from_f64(0.8);
        utility += coin_utility + need_pressure_bonus;
    }

    utility -= action.energy_cost * Fixed::from_f64(0.5);

    // §9.2 (Iteration 94): RL action values feed selection — the agent's
    // learned valuation weights (EMA-updated from successful outcomes in
    // `sim.rs`) bias each candidate's utility via `learned_delta` against
    // that action's outcome profile. Zero at the neutral prior (tick-0
    // inert), deterministic (no RNG — only a utility term), so the learned
    // signal shifts the argmax without perturbing the RNG stream. Each
    // profile is normalized by its own sum inside `learned_delta`, so the
    // baseline is a uniform 0.5 across candidates and the relative signal
    // (profiles matching what the agent learned to value) is what
    // differentiates them.
    utility += action_values.learned_delta(action.kind.outcome_profile());

    // §8.1.6 (Iteration 162): temperament decision consumers — persistence
    // narrows the decision-noise amplitude and approach-withdrawal biases
    // exploration. Both read the deviation of the live temperament layer
    // from its trait-derived baseline (`Temperament::from_traits`), which
    // is exactly 0 at construction and drifts only as the plasticity pass
    // accumulates life experience — so calibrated runs stay byte-identical
    // until the layer moves. Deterministic (pure utility/amplitude terms;
    // the noise RNG draw stays unconditional — only its range changes, so
    // the Behavior-stream position is untouched).
    let temperament_baseline = Temperament::from_traits(personality);
    let persistence_dev = personality.temperament.persistence - temperament_baseline.persistence;
    let approach_dev =
        personality.temperament.approach_withdrawal - temperament_baseline.approach_withdrawal;

    // Persistence: a persistent agent's choices are more stable — the
    // ±0.05 exploration jitter shrinks with the deviation (floored so even
    // maximal persistence keeps a minimal jitter). Zero at zero deviation.
    let amplitude = noise_amplitude(persistence_dev);
    let noise_roll: f64 = rng.get_mut(RngStream::Behavior).random_range(-amplitude..amplitude);
    utility += Fixed::from_f64(noise_roll);

    // Approach/withdrawal: an approach-biased agent explores more, a
    // withdrawal-biased one wanders less.
    if action.kind == ActionKind::Wander && approach_dev != Fixed::ZERO {
        utility += approach_dev * APPROACH_WANDER_BONUS;
    }

    utility
}

/// §8.1.6 (Iteration 162): the decision-noise amplitude for a given
/// persistence deviation from the trait-derived baseline.
///
/// The legacy ±0.05 jitter shrinks by `deviation × PERSISTENCE_NOISE_REDUCTION`
/// (a 0.5 deviation halves it), floored at [`PERSISTENCE_NOISE_FLOOR`] so even
/// maximal persistence keeps a minimal exploration jitter. Zero-at-zero:
/// deviation 0 → exactly 0.05 (legacy amplitude, byte-identical). Deviations
/// are non-negative in practice (plasticity pushes persistence toward
/// baseline + goal-striving signal), but negative values are treated as the
/// legacy amplitude defensively. Deterministic — the caller's RNG draw stays
/// unconditional (only the range changes).
fn noise_amplitude(persistence_dev: Fixed) -> f64 {
    if persistence_dev <= Fixed::ZERO {
        return 0.05;
    }
    (0.05 - persistence_dev.to_f64() * PERSISTENCE_NOISE_REDUCTION * 0.05)
        .max(PERSISTENCE_NOISE_FLOOR)
}

/// Classify an action for DecisionPolicy modifier lookup.
fn action_traits(kind: ActionKind) -> (bool, bool, bool, bool, bool, bool) {
    // (is_social, is_risky, is_withdrawal, is_prosocial, is_disobedient, is_harmful)
    match kind {
        ActionKind::Eat | ActionKind::Drink => (false, false, false, false, false, false),
        ActionKind::Rest => (false, false, true, false, false, false),
        ActionKind::Work => (false, false, false, true, false, false),
        ActionKind::Socialize | ActionKind::Trade => (true, false, false, true, false, false),
        ActionKind::Worship => (true, false, false, true, false, false),
        ActionKind::Wander => (false, true, false, false, true, false),
        ActionKind::Move { .. } => (false, false, false, false, false, false),
        ActionKind::Idle => (false, false, true, false, true, false),
    }
}

/// Select the best action for an agent based on utility.
///
/// DecisionPolicy modifiers (emotional, moral, habit) are applied to each
/// candidate's utility, making agents feel like adaptive intelligence rather
/// than static utility functions.
/// Select the best action for an agent based on utility.
///
/// DecisionPolicy modifiers (emotional, moral, habit) are applied to each
/// candidate's utility, making agents feel like adaptive intelligence rather
/// than static utility functions.
pub fn select_action(ctx: &DecisionContext<'_>, rng: &mut RngStreams) -> ActionKind {
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
        let mut utility = compute_utility(
            &def,
            ctx.needs,
            ctx.personality,
            rng,
            ctx.total_grain,
            ctx.total_water,
            ctx.identity,
            ctx.norm_pressure,
            ctx.coin,
            ctx.action_values,
            ctx.dominant_need,
            ctx.dominant_pressure,
            ctx.dread,
        );

        for goal in ctx.active_goals {
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

        // Architecture-plan-2 §8.1.20: Apply DecisionPolicy modifiers.
        // These modulate utility based on emotional state, moral values,
        // and habit strength — making agents feel adaptive rather than static.
        let (is_social, is_risky, is_withdrawal, is_prosocial, is_disobedient, is_harmful) =
            action_traits(*kind);
        let emo = ctx.decision_policy.emotional_modifier(
            ctx.anger,
            ctx.fear,
            ctx.joy,
            ctx.sadness,
            is_social,
            is_risky,
            is_withdrawal,
        );
        let moral = ctx.decision_policy.moral_modifier(
            ctx.fairness,
            ctx.authority,
            ctx.care,
            ctx.loyalty,
            is_prosocial,
            is_disobedient,
            is_harmful,
        );
        // Habit modifier: routine actions get a boost under stress
        let is_routine = matches!(
            kind,
            ActionKind::Work
                | ActionKind::Eat
                | ActionKind::Drink
                | ActionKind::Rest
                | ActionKind::Worship
        );
        let habit = ctx.decision_policy.habit_modifier(is_routine, ctx.stress);
        utility += emo + moral + habit;

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
                fairness: Fixed::ZERO,
                authority: Fixed::ZERO,
                care: Fixed::ZERO,
                loyalty: Fixed::ZERO,
                action_values: ActionValues::default(),
                dominant_need: MotiveCategory::Hunger,
                dominant_pressure: Fixed::ZERO,
                dread: Fixed::ZERO,
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
                fairness: Fixed::ZERO,
                authority: Fixed::ZERO,
                care: Fixed::ZERO,
                loyalty: Fixed::ZERO,
                action_values: ActionValues::default(),
                dominant_need: MotiveCategory::Hunger,
                dominant_pressure: Fixed::ZERO,
                dread: Fixed::ZERO,
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
                fairness: Fixed::ZERO,
                authority: Fixed::ZERO,
                care: Fixed::ZERO,
                loyalty: Fixed::ZERO,
                action_values: ActionValues::default(),
                dominant_need: MotiveCategory::Hunger,
                dominant_pressure: Fixed::ZERO,
                dread: Fixed::ZERO,
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
                fairness: Fixed::ZERO,
                authority: Fixed::ZERO,
                care: Fixed::ZERO,
                loyalty: Fixed::ZERO,
                action_values: ActionValues::default(),
                dominant_need: MotiveCategory::Hunger,
                dominant_pressure: Fixed::ZERO,
                dread: Fixed::ZERO,
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
                fairness: Fixed::ZERO,
                authority: Fixed::ZERO,
                care: Fixed::ZERO,
                loyalty: Fixed::ZERO,
                action_values: ActionValues::default(),
                dominant_need: MotiveCategory::Hunger,
                dominant_pressure: Fixed::ZERO,
                dread: Fixed::ZERO,
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
                        fairness: Fixed::ZERO,
                        authority: Fixed::ZERO,
                        care: Fixed::ZERO,
                        loyalty: Fixed::ZERO,
                        action_values: ActionValues::default(),
                        dominant_need: MotiveCategory::Hunger,
                        dominant_pressure: Fixed::ZERO,
                        dread,
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

    // ── §8.1.6 (Iteration 162): temperament decision consumers ─────────

    /// §8.1.6 (Iteration 162): `noise_amplitude` shrinks the ±0.05 jitter
    /// with persistence deviation — zero at zero (byte-identical legacy),
    /// monotone decreasing, floored so maximal persistence keeps a minimal
    /// exploration jitter (no full determinism cliff).
    #[test]
    fn persistence_narrows_decision_noise_amplitude() {
        assert_eq!(noise_amplitude(Fixed::ZERO), 0.05);
        assert_eq!(noise_amplitude(-Fixed::from_f64(0.2)), 0.05, "negative deviation is defensively legacy");
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
        approach.temperament.approach_withdrawal =
            baseline.approach_withdrawal + Fixed::from_f64(0.4);
        // Withdrawal-biased: −0.3 deviation.
        let mut withdraw = control.clone();
        withdraw.temperament.approach_withdrawal =
            baseline.approach_withdrawal - Fixed::from_f64(0.3);

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
        approach.temperament.approach_withdrawal =
            baseline.approach_withdrawal + Fixed::from_f64(0.4);

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
            )
        };
        assert_eq!(
            work_util(&approach),
            work_util(&control),
            "approach deviation must not touch non-Wander utility"
        );
    }
}
