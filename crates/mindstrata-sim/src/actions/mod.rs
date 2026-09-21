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

/// i351 (A8 closure): the exploration driver's coefficient — utility per
/// unit of motivation-layer novelty pressure for the `Wander` action.
/// Sized from the probe pair (`i351_wander_driver` leg A: novelty pressure
/// p50 0.29–0.33, p90 0.46–0.48 at settlement; leg B: the pre-driver wall
/// mean-loss 1.61, wins 0): at coef 2.0 the term is 0.58–0.96 — competitive
/// with the 0.1–0.4 winning-utility band that holds when biological needs
/// are quiet (the gate's condition), without reaching the 1.2+ wall of a
/// pressed agent (and the gate excludes them anyway). The 0.5–3%-share
/// target band (§4.2) is verified in vivo by `i351_wander_bands`.
const WANDER_NOVELTY_COEF: Fixed = Fixed::from_raw(20_000); // 2.0

/// i351 (A8 closure): the need-quietude gate for the exploration driver.
/// Exploration is what a satiated, restless agent does — a villager with
/// hunger/thirst/fatigue above this gate must provision first (the winner
/// utilities there ride pressure² × 2.0–2.5 and SHOULD beat a driver).
/// 0.5 = the mid-band boundary the goals/needs machinery already treats as
/// "pressing" (§8.1.16 goal-congruence keys on hunger < 0.5).
pub(crate) const WANDER_QUIETUDE_GATE: Fixed = Fixed::from_raw(5000); // 0.5

/// i281: salience-recency window for the polarity-claim social bias, in
/// ticks — one in-sim year (`ticks_per_year = 1000`). The 0.01 coefficient
/// was ratified (i275) against the claim integral a 1–2K horizon produces;
/// the window makes that calibration horizon-invariant instead of letting
/// the bias grow as an unbounded run integral (measured: 0.017 → 0.82 ×
/// social_value over 1K→20K without it).
pub(crate) const CLAIM_SALIENCE_TICKS: u64 = 1000;

/// AP3 DC-1 signature #1 + Iteration-268 (i268 forced-Bond probe):
/// pathology → action nudges, extracted as a pure helper so the pin can
/// test all four quadrants without a full `DecisionContext`.
///
/// Q1 dark_addiction (deficit fixation): suppresses Work, mild Rest bias —
/// the agent stops provisioning for a future it is fixated on. Q2
/// dark_allergy (recoil from contradiction): suppresses Socialize —/// withdrawal from the line that delivered contradiction. Q3
/// golden_addiction (grasping): POSITIVE Socialize/Worship bias — the
/// grasping signature reaches for connection/transcendence; the only
/// positive pathology channel. Q4 golden_allergy (refusal of opening):
/// suppresses Worship — avoiding the genuine opening.
///
/// Per-subject Q3 equilibrium measured ≈0.75 at forced Bond rates, inside
/// the ratified ceiling band [0.70,0.90] (growth 0.07 / decay 0.015 /
/// ceiling 0.85 need no re-pin). Coefficient 0.04 matches the allergy
/// channels; at natural Q3 mean 0.038–0.046 the shift is ≤0.0019 — a
/// genuine nudge, not a reordering lever. Zero-at-zero: neutral field ⇒
/// 0 for every kind. Deterministic (pure function, no RNG).
pub fn development_pathology_nudge(
    kind: ActionKind,
    pathology: &mindstrata_development::dynamics::PathologyField,
) -> Fixed {
    let dark = pathology.dark_addiction.intensity;
    let dark_allergy = pathology.dark_allergy.intensity;
    let golden_addiction = pathology.golden_addiction.intensity;
    let golden_allergy = pathology.golden_allergy.intensity;
    match kind {
        ActionKind::Work => -Fixed::from_f64(dark * 0.08),
        ActionKind::Rest => Fixed::from_f64(dark * 0.02),
        ActionKind::Socialize => Fixed::from_f64(golden_addiction * 0.04 - dark_allergy * 0.04),
        ActionKind::Worship => Fixed::from_f64(golden_addiction * 0.04 - golden_allergy * 0.04),
        _ => Fixed::ZERO,
    }
}

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
    // ── i351 (A8 closure): the exploration driver ───────────────────
    /// The agent's motivation-layer NOVELTY pressure (full formula —
    /// deficit × urgency × amplifiers, exactly what `update_dominant`
    /// compares). Drives the Wander driver term: a restless agent with
    /// quiet biological needs explores. Zero when novelty is saturated-
    /// relieved, so identity holds at novelty 0.
    pub novelty_pressure: Fixed,
    /// `true` when the agent's biological needs are quiet enough to
    /// explore (`max(hunger, thirst, fatigue) < WANDER_QUIETUDE_GATE`).
    /// The driver term is gated on this: exploration is what a SATIATED
    /// restless agent does — a hungry agent's high-novelty pressure must
    /// not pull it off provisioning (the wall is correct for them).
    pub needs_quiet: bool,
    // ── §8.1 (Iteration 248): sleep-debt social withdrawal ──────────
    /// How much circadian sleep debt suppresses social participation.
    /// Zero below the `sleep_deprived()` threshold (0.5), scaling above —
    /// an exhausted agent declines gatherings before it neglects work.
    pub social_withdrawal: Fixed,
    // ── §8.1 (Iteration 247): interoceptive somatic marker ──────────
    /// How much worse than a default interoceptor the agent feels right
    /// now (fatigue + pain amplification). Biases risky actions down —
    /// a body in distress votes against gambles. Exactly zero for
    /// default configurations, so calm-world selection is unchanged.
    pub somatic_marker: Fixed,
    // ── §8.1.16 (Iteration 103): prospection dread ──────────────────
    /// The agent's scenario-grounded dread (0–1) — how much it fears the
    /// imagined bad future. Drives the precautionary-provisioning term.
    pub dread: Fixed,
    // ── §8.1.16 (Iteration 203): prospection hope ───────────────────
    /// The agent's scenario-grounded hope (0–1) — how much it looks
    /// forward to the imagined good future. Drives the
    /// aspirational-engagement term (the positive mirror of dread's
    /// precautionary provisioning).
    pub hope: Fixed,
    // ── §8.1.12 (Iteration 204): planning confidence ────────────────
    /// The agent's confidence in its ability to plan and execute
    /// (0–1, 0.5 = neutral default) — the blended emotion term +
    /// executive-function planning depth. Drives the deferred-
    /// gratification calibration term (§8.1.12 "high executive
    /// function enables long-term planning").
    pub planning_confidence: Fixed,
    // ── Iteration 232: mood drift ──────────────────────────────────
    /// Agent's current mood valence (-1 to 1, derived from affect).
    /// Positive mood boosts social/exploration actions; negative mood
    /// boosts withdrawal/work. This is the plan's "mood affects behavior"
    /// channel — making agents feel adaptive rather than static.
    pub mood_valence: Fixed,
    // ── Iteration 233: seasonal behavioral modulation ──────────────
    /// Current season (0=Spring, 1=Summer, 2=Autumn, 3=Winter).
    /// Modulates action utility: winter boosts social/worship,
    /// summer boosts work/trade.
    pub season: u8,
    // ── Iteration 236: age-related behavioral modulation ───────────
    /// Current life stage (0=Infant, 1=Child, 2=Adolescent, 3=YoungAdult,
    /// 4=Adult, 5=Mature, 6=Elder). Modulates action utility: youth
    /// boosts exploration, elders boost social/worship.
    pub life_stage: u8,
    // ── AP3 DC-1 (task 3.4): development gating — fulfillment thresholds
    // read via canon `needs` band map (docs/balance/needs-bands.md v1,
    // CALIBRATION-PENDING). Zero at neutral DevelopmentFieldState
    // (altitudes 0.0, pathology neutral) so goldens stay byte-identical
    // until the field moves.
    /// Per-agent attractor-field state — altitudes + pathology.
    pub development: &'a crate::psychology::DevelopmentFieldState,
    // DC-2 Era III lite: per-agent polarity_claims list (read-only).
    // The action selector counts `ActiveTension` claims and applies
    // a small bias `+0.01 × action.social_value × count` to social
    // actions. Coefficient 0.01 is the i282 safe-range start
    // (expected shift ~0.025, within 1σ of natural variance 0.1265).
    /// Per-agent three-realm polarity claims (read-only; derived
    /// deterministically from the catalyst stream by the daily pass).
    pub polarity_claims: &'a [crate::development::ThreeRealmClaim],
    /// Current simulation tick (i281: anchors the polarity-claim
    /// salience-recency window at the social-bias site).
    pub current_tick: u64,
    // ── WP-J (Iteration 280): institution-membership work bonus ────
    /// Aggregate Work-utility bonus from the agent's institution
    /// memberships (Σ over live institutions of `member_work_bonus(morale,
    /// mult)`). Zero below the WP-J band-III gate — pinned horizons are
    /// byte-identical; above the gate a functioning institution makes
    /// provisioning more attractive to its members (dread-class nudge).
    /// Zero-at-zero: no memberships or all-dead institutions → 0.
    pub institution_work_bonus: Fixed,
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
                // Iteration 190: 0.5 → 0.7 — a drink action (2 ticks, the
                // cheapest action) now mostly quenches, mirroring Eat's
                // 0.4-per-4-ticks efficiency. Combined with the routine's
                // daily 07:00-08:00 drink slot, thirst settles near ~0.2
                // (probe-pinned) instead of the pre-Iter-190 ~0.55 chronic
                // dehydration.
                thirst_relief: Fixed::from_f64(0.7),
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
    hope: Fixed,
    planning_confidence: Fixed,
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
            // i351 (A8 closure): `MotiveCategory::Novelty` competes in
            // `update_dominant` but had no relief mapping — a dead dominant
            // motive (§4.3): it could WIN the argmax and nothing responded.
            // The exploration driver is its response.
            MotiveCategory::Novelty => action.kind == ActionKind::Wander,
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

    // §8.1.16 (Iteration 203): aspirational engagement — an agent who
    // daily imagines the village thriving (scenario-grounded hope) builds
    // toward that future: Socialize and Worship (community and collective
    // meaning) gain utility and Idle loses it. The positive mirror of
    // dread's precautionary provisioning (Iteration 103): dread stocks up
    // (Work/Trade), hope engages (Socialize/Worship); the channels are
    // disjoint so they compose without double-counting. Zero-at-zero
    // (hope 0 → exact legacy utility), deterministic (pure utility term —
    // the RNG stream is untouched), and sized small (0.2/0.1, same as the
    // dread nudge): a hope 0.4 agent's Socialize gains 0.08 — a genuine
    // nudge, not a reordering lever.
    if hope > Fixed::ZERO {
        match action.kind {
            ActionKind::Socialize | ActionKind::Worship => {
                utility += hope * Fixed::from_f64(0.2);
            }
            ActionKind::Idle => {
                utility -= hope * Fixed::from_f64(0.1);
            }
            _ => {}
        }
    }

    // §8.1.12 (Iteration 204): deferred-gratification calibration — an
    // agent that trusts its ability to plan and execute (the blended
    // emotion term + executive-function planning depth) commits to
    // future-facing Work and wastes less time Idle; a low-confidence
    // agent hedges (Work down, Idle up) because it cannot trust the
    // long-horizon payoff. Baseline-corrected around the 0.5 neutral
    // default (exactly zero effect at 0.5 — the Iter-197 zero-drift
    // pattern, so default populations stay byte-identical),
    // deterministic (pure utility term — the RNG stream is untouched),
    // and sized small (0.2/0.1, the same nudge scale as the dread/hope
    // channels): a confident 0.65 agent's Work gains 0.03.
    let pc_shift = planning_confidence - Fixed::from_f64(0.5);
    if pc_shift != Fixed::ZERO {
        match action.kind {
            ActionKind::Work => {
                utility += pc_shift * Fixed::from_f64(0.2);
            }
            ActionKind::Idle => {
                utility -= pc_shift * Fixed::from_f64(0.1);
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
    let noise_roll: f64 = rng
        .get_mut(RngStream::Behavior)
        .random_range(-amplitude..amplitude);
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
    // i346 census: the realized utility of the two candidates that carry no
    // need-relief term of their own (`Wander` — ledger A8 — and `Idle`).
    // Recording where they finish is what turns "never selected" into a
    // sizing — see `sim::decision_census`.
    let mut wander_utility: Option<Fixed> = None;
    let mut idle_utility: Option<Fixed> = None;

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
            ctx.hope,
            ctx.planning_confidence,
        );

        // i351 (A8 closure): the exploration driver, applied in the
        // selection loop (where `ctx` lives). `Wander` was imported with
        // zero relief on every channel and lost all 107 085 measured
        // arbitrations by 1.2–1.6 (i346 §4.3) — and the motivation layer's
        // novelty need grew with NO behavioural outlet, so
        // `MotiveCategory::Novelty` could dominate an agent with nothing to
        // do about it. The design act A8 asked for: **a villager may roam,
        // and does so when restless and unpressured**. The term is gated on
        // need quietude: the wall (winning utilities ride pressure² ×
        // 2.0–2.5) is CORRECT for a pressed agent — the driver only
        // competes when the biological needs are quiet, where the measured
        // winner sits at mean 0.71 / max 1.40 (`i351_wander_diag`: quiet
        // windows are 70.1% of utility samples, and Work 53% / Worship 30%
        // still ride production+meaning pressure there). Sizing is the
        // offline sweep over the same realized samples (census
        // `quiet_sweep`): c=2.0 wins 4.9% of quiet windows (56/1141) —
        // landing Wander's overall share in the sub-1% band (comparable to
        // the §4.2 target under Worship 1.74%), while c=3.0 already jumps
        // to 19.5% (over-drive). First draft shipped coef 0.2 by a
        // Fixed::from_raw unit error (2000 raw = 0.2, not 2.0) — the
        // in-vivo census (0 wins despite the sweep predicting 56) is what
        // exposed it. Zero-at-zero: novelty pressure 0 or gate closed →
        // exactly the pre-i351 utility. Deterministic (pure utility term,
        // no RNG).
        if *kind == ActionKind::Wander && ctx.novelty_pressure > Fixed::ZERO && ctx.needs_quiet {
            utility += ctx.novelty_pressure * WANDER_NOVELTY_COEF;
        }

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
        // DC-2 Era III lite (FR-030/FR-032 wiring): bias social actions
        // by the agent's ActiveTension polarity count. Coefficient RE-
        // DERIVED 0.10 → 0.01 (Iteration-275, §4.2): the 0.05/0.10 ramp
        // (DC-2.4) was ratified against a DEAD tension diet (v1 mono-subtle
        // projection: 0–3 sporadic counts per i282). i275 made the diet
        // live (severity-grounded Threat: mean 1.25–2.75/agent at 1–2K,
        // probe i275_tension_regime), so 0.10 produced bias ~0.13–0.28 ×
        // social_value — 10× the audited 0–0.03 magnitude band, re-timing
        // seed-46 demographic schedules (births [2890] → [240, 280]) and
        // inverting the interoception pin pre-loop-closure. 0.01 at the
        // live diet's counts lands the bias at ≤0.0275 × social_value —
        // inside the audited band, restoring the ratified "5–20% of the
        // social driver" contract. Only social actions are biased
        // (`is_social` is true) so the bias is scoped to the social
        // action family. Identity-at-zero preserved: an agent with no
        // ActiveTension claims contributes 0 to the bias.
        if is_social {
            // i281 salience-recency window: only claims emitted within the
            // last CLAIM_SALIENCE_TICKS count. The original coefficient
            // (0.10, DC-2.4) and its re-derivation (0.01, i275) were both
            // ratified against 1–2K-horizon claim integrals; i281 measured
            // the count as an unbounded run integral (bias 0.017 → 0.82 ×
            // social_value over 1K→20K, 27× the audited 0–0.03 band). A
            // 1000-tick window (ticks_per_year) makes the count a bounded
            // RECENCY integral whose equilibrium matches exactly the diet
            // the coefficient was ratified against — the honest §4.2
            // re-contract: the 0.01 coefficient stands; the integral it
            // multiplies is restored to its calibrated horizon. Zero-at-
            // zero unchanged: no recent ActiveTension claims → no bias.
            let now = ctx.current_tick.saturating_sub(CLAIM_SALIENCE_TICKS);
            let active_tension_count = ctx
                .polarity_claims
                .iter()
                .filter(|c| {
                    c.polarity == crate::development::PolarityState::ActiveTension
                        && c.created_tick >= now
                })
                .count();
            utility += Fixed::from_f64(0.01 * active_tension_count as f64) * def.social_value;
        }
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

        // Iteration 247 (Arc B — interoception): the somatic marker
        // biases risky actions DOWN — a body in distress votes against
        // gambles (Wander is the classified risky action). Zero for
        // default interoceptors, so calm worlds are byte-identical.
        if is_risky && ctx.somatic_marker > Fixed::ZERO {
            utility -= ctx.somatic_marker * Fixed::from_f64(0.1);
        }

        // Iteration 248 (Arc B): sleep-debt withdrawal — social actions
        // lose utility as unpaid sleep accumulates. Zero below the
        // deprivation threshold, so rested agents are byte-identical.
        if is_social && ctx.social_withdrawal > Fixed::ZERO {
            utility -= ctx.social_withdrawal * Fixed::from_f64(0.08);
        }

        // Iteration 232: mood drift — positive mood boosts social/
        // exploration actions; negative mood boosts withdrawal/work.
        // Sized at ±0.03 (comparable to dread/hope nudge) so it's a
        // genuine behavioral nudge, not a reordering lever.
        let mood_nudge = if is_social && ctx.mood_valence > Fixed::ZERO {
            ctx.mood_valence * Fixed::from_f64(0.03)
        } else if is_withdrawal && ctx.mood_valence < Fixed::ZERO {
            (-ctx.mood_valence) * Fixed::from_f64(0.03)
        } else {
            Fixed::ZERO
        };
        utility += mood_nudge;

        // Iteration 233: seasonal behavioral modulation.
        // Winter boosts social/worship (+0.02), summer boosts work (+0.02).
        // Sized at ±0.02 so it's a genuine nudge, not a reordering lever.
        let season_nudge = match ctx.season {
            3 => {
                // Winter: boost social, reduce work
                if is_social {
                    Fixed::from_f64(0.02)
                } else if matches!(kind, ActionKind::Work) {
                    Fixed::from_f64(-0.01)
                } else {
                    Fixed::ZERO
                }
            }
            1 => {
                // Summer: boost work, reduce social
                if matches!(kind, ActionKind::Work) {
                    Fixed::from_f64(0.02)
                } else if is_social {
                    Fixed::from_f64(-0.01)
                } else {
                    Fixed::ZERO
                }
            }
            _ => Fixed::ZERO, // Spring/Autumn: neutral
        };
        utility += season_nudge;

        // WP-J (Iteration 280): institution-membership work bonus — a
        // functioning institution (positive morale) makes provisioning
        // more attractive to its members, scaled by the governance/
        // economic-systems collective band. Zero below the band-III gate
        // (all pinned horizons), so calm worlds are byte-identical.
        if ctx.institution_work_bonus > Fixed::ZERO && matches!(kind, ActionKind::Work) {
            utility += ctx.institution_work_bonus;
        }

        // Iteration 236: age-related behavioral modulation.
        // Youth (Adolescent/YoungAdult) boost exploration (Wander).
        // Elders boost social/worship, reduce work.
        let age_nudge = match ctx.life_stage {
            2 | 3 => {
                // Adolescent/YoungAdult: boost exploration
                if matches!(kind, ActionKind::Wander) {
                    Fixed::from_f64(0.02)
                } else {
                    Fixed::ZERO
                }
            }
            6 => {
                // Elder: boost social, reduce work
                if is_social {
                    Fixed::from_f64(0.02)
                } else if matches!(kind, ActionKind::Work) {
                    Fixed::from_f64(-0.01)
                } else {
                    Fixed::ZERO
                }
            }
            _ => Fixed::ZERO, // Other stages: neutral
        };
        utility += age_nudge;

        // AP3 DC-1 (tasks 3.4/3.5): development gating — fulfillment thresholds
        // via `needs` band map (docs/balance/needs-bands.md) + pathology
        // signatures (docs/balance/pathology-curves.md). Zero-at-zero:
        // neutral pathology ⇒ 0, so goldens stay byte-identical until a
        // catalyst actually steps pathology (FR-023).
        let dev_nudge = development_pathology_nudge(*kind, &ctx.development.pathology);
        utility += dev_nudge;

        if matches!(kind, ActionKind::Wander) {
            wander_utility = Some(utility);
        } else if matches!(kind, ActionKind::Idle) {
            idle_utility = Some(utility);
        }

        if utility > best_utility {
            best_utility = utility;
            best_action = *kind;
        }
    }

    // i346: hand the arbitration to the census. `record_utility_sample` is
    // gated on the census being enabled and only reads values the loop has
    // already computed — no RNG draw, no state write — so an instrumented run
    // is byte-identical to an uninstrumented one. i351: the quiet-window
    // split records the driver's actual arena — the winner it must beat when
    // the need-quietude gate is open — plus the pre-driver gap and novelty
    // pressure so coefficients can be swept offline (§4.2 sizing).
    if let (Some(wander_utility), Some(idle_utility)) = (wander_utility, idle_utility) {
        // i351 instrumentation: the exact PRE-driver gap, derived by
        // subtracting the driver term (known in full at this point) from
        // whichever candidate carried it. Lets the census sweep coefficients
        // offline over realized samples (§4.2 sizing). Read-only, no RNG.
        let term_current = if ctx.novelty_pressure > Fixed::ZERO && ctx.needs_quiet {
            ctx.novelty_pressure * WANDER_NOVELTY_COEF
        } else {
            Fixed::ZERO
        };
        let best_utility_pre = if matches!(best_action, ActionKind::Wander) {
            best_utility - term_current
        } else {
            best_utility
        };
        let pre_gap = (best_utility_pre - (wander_utility - term_current)).to_f64();
        crate::sim::decision_census::record_utility_sample(
            pre_gap,
            (best_utility - wander_utility).to_f64(),
            (best_utility - idle_utility).to_f64(),
            ctx.needs_quiet,
            best_utility.to_f64(),
            crate::sim::decision_census::action_index(best_action),
            ctx.novelty_pressure.to_f64(),
        );
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

    // Iteration-267 (i275 needs-band calibration): esteem/autonomy relief
    // paths, the missing half of the revived decay. The decay revive
    // (systems/mod.rs) is an unconditional ratchet +1e-4/tick (Fixed-4
    // truncation disease, §5) — with no relief channel the band would pin
    // at 1.0 everywhere within 8K ticks (§4.3 dead-producer hazard). The
    // design-cited relieve path (needs-bands.md Esteem row: "Work/Trade
    // relieve paths") uses competence-action relief sized so a full-time
    // worker (ρ≈0.65) equilibria mid-band: relief 0.0002/tick balances
    // decay at ρ = decay/relief = 0.5, well inside the draft [0.38,0.58]
    // pacing band. Zero-at-zero: an idle agent accumulates deficit as
    // designed; equilibrium position is workload-dependent, which IS the
    // competence signal.
    let competence_relief = Fixed::from_f64(0.0002);
    match action {
        ActionKind::Work | ActionKind::Trade => {
            needs.esteem = (needs.esteem - competence_relief).clamp_01();
            needs.autonomy = (needs.autonomy - competence_relief).clamp_01();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests;
