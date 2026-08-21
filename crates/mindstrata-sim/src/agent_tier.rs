//! §17 Performance and Scalability Strategy — Agent Tiers and Cognitive Budget.
//!
//! To keep the simulation scalable, agents are classified into three tiers:
//!
//! - **Focal**: full simulation (biology, psychology, relationships, memory, prospection, narrative).
//! - **Secondary**: reduced simulation (simplified biology, heuristic cognition, relationship updates only when relevant).
//! - **Background**: aggregate simulation (household-level behavior, statistical belief updates, minimal memory).
//!
//! Agents can be promoted/dynamically based on:
//! - player focus,
//! - narrative importance (high status, faction leadership),
//! - institutional role (council member, priest),
//! - crisis involvement (high fear/anger, recent combat),
//! - relationship density (many recent interactions).

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// §17.1 (Iteration 159): minimum time a tier must be held before a Focal
/// demotion may fire. Narrative importance starts near-uniform (~0.31 for
/// everyone at tick 100) and converges slowly (0.1/tick smoothing) toward
/// ~0.45–0.65 by tick 1000; demotions before ~tick 900 fire on the
/// transient, not the differentiated signal, and ratchet the whole village
/// to Secondary (measured at 24: 4F/20S, 48: 5F/43S).
const TIER_MATURITY_TICKS: u64 = 900;

/// Simulation tier for an agent — determines which systems run each tick.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub enum AgentTier {
    /// Full simulation: biology, psychology, relationships, memory, prospection, narrative.
    Focal,
    /// Reduced simulation: simplified biology, heuristic cognition, relationship updates only when relevant.
    #[default]
    Secondary,
    /// Aggregate simulation: household-level behavior, statistical belief updates, minimal memory.
    Background,
}

impl AgentTier {
    /// Does this tier run the full biological update (endocrine, metabolism, cardiovascular, etc.)?
    pub fn runs_full_biology(&self) -> bool {
        matches!(self, Self::Focal)
    }

    /// Does this tier run the full psychological pipeline (appraisal, emotion regulation,
    /// theory of mind, prospection, narrative)?
    pub fn runs_full_psychology(&self) -> bool {
        matches!(self, Self::Focal)
    }

    /// Does this tier run heuristic cognition (basic appraisal + emotion decay)?
    pub fn runs_heuristic_cognition(&self) -> bool {
        matches!(self, Self::Focal | Self::Secondary)
    }

    /// Does this tier run theory of mind (modeling other agents' minds)?
    pub fn runs_theory_of_mind(&self) -> bool {
        matches!(self, Self::Focal)
    }

    /// Does this tier run prospection (mental simulation of futures)?
    pub fn runs_prospection(&self) -> bool {
        matches!(self, Self::Focal)
    }

    /// Does this tier run narrative identity updates?
    pub fn runs_narrative(&self) -> bool {
        matches!(self, Self::Focal)
    }

    /// Does this tier run relationship updates?
    pub fn runs_relationship_updates(&self) -> bool {
        matches!(self, Self::Focal | Self::Secondary)
    }

    /// Does this tier run belief updates?
    pub fn runs_belief_updates(&self) -> bool {
        matches!(self, Self::Focal | Self::Secondary)
    }

    /// Does this tier run memory encoding?
    pub fn runs_memory_encoding(&self) -> bool {
        matches!(self, Self::Focal | Self::Secondary)
    }

    /// Does this tier participate in social interactions?
    pub fn runs_social_interactions(&self) -> bool {
        matches!(self, Self::Focal | Self::Secondary)
    }

    /// Does this tier run action selection (utility AI)?
    pub fn runs_action_selection(&self) -> bool {
        matches!(self, Self::Focal | Self::Secondary)
    }

    /// §17: Numeric tier index for metrics (Focal=0, Secondary=1, Background=2).
    pub fn tier_index(&self) -> f64 {
        match self {
            Self::Focal => 0.0,
            Self::Secondary => 1.0,
            Self::Background => 2.0,
        }
    }
}

/// §17.2: Cognitive budget — limits per-tick processing capacity for each agent.
///
/// Salient events get priority within each budget category.
/// Background agents have zero budgets for most categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveBudget {
    /// Maximum appraisals per tick (events processed through emotion generation).
    pub max_appraisals: u32,
    /// Maximum memory operations (encode + retrieve) per tick.
    pub max_memory_operations: u32,
    /// Maximum prospections (mental simulations of future outcomes) per tick.
    pub max_prospections: u32,
    /// Maximum social inferences (theory-of-mind updates) per tick.
    pub max_social_inferences: u32,
}

impl Default for CognitiveBudget {
    fn default() -> Self {
        Self::focal()
    }
}

impl CognitiveBudget {
    /// Full budget for focal agents.
    pub fn focal() -> Self {
        Self {
            max_appraisals: 20,
            max_memory_operations: 10,
            max_prospections: 5,
            max_social_inferences: 10,
        }
    }

    /// Reduced budget for secondary agents.
    pub fn secondary() -> Self {
        Self {
            max_appraisals: 5,
            max_memory_operations: 3,
            max_prospections: 0,
            max_social_inferences: 2,
        }
    }

    /// Minimal budget for background agents.
    pub fn background() -> Self {
        Self {
            max_appraisals: 1,
            max_memory_operations: 0,
            max_prospections: 0,
            max_social_inferences: 0,
        }
    }

    /// Create a budget appropriate for the given tier.
    pub fn for_tier(tier: AgentTier) -> Self {
        match tier {
            AgentTier::Focal => Self::focal(),
            AgentTier::Secondary => Self::secondary(),
            AgentTier::Background => Self::background(),
        }
    }
}

/// §17.2: Per-tick cognitive budget tracker — enforces processing limits.
///
/// Reset at the start of each tick from the agent's [`CognitiveBudget`].
/// Subsystems check `can_*()` before running expensive operations and
/// call `consume_*()` after successfully performing them.
///
/// When budget is exhausted, the agent falls back to cheaper heuristic
/// paths (e.g., skip prospection, use cached appraisal, skip ToM update).
#[derive(Debug, Clone)]
#[must_use]
#[expect(
    clippy::struct_field_names,
    reason = "all fields track remaining budget for a specific cognitive operation"
)]
pub struct CognitiveBudgetTracker {
    remaining_appraisals: u32,
    remaining_memory_operations: u32,
    remaining_prospections: u32,
    remaining_social_inferences: u32,
}

impl Default for CognitiveBudgetTracker {
    fn default() -> Self {
        Self::from_budget(&CognitiveBudget::default())
    }
}

impl CognitiveBudgetTracker {
    /// Create a tracker from a budget — all counters start full.
    pub fn from_budget(budget: &CognitiveBudget) -> Self {
        Self {
            remaining_appraisals: budget.max_appraisals,
            remaining_memory_operations: budget.max_memory_operations,
            remaining_prospections: budget.max_prospections,
            remaining_social_inferences: budget.max_social_inferences,
        }
    }

    /// Reset all counters to the budget limits (called at tick start).
    pub fn reset(&mut self, budget: &CognitiveBudget) {
        self.remaining_appraisals = budget.max_appraisals;
        self.remaining_memory_operations = budget.max_memory_operations;
        self.remaining_prospections = budget.max_prospections;
        self.remaining_social_inferences = budget.max_social_inferences;
    }

    // ── Check methods ───────────────────────────────────────────────

    /// Can this agent perform another appraisal this tick?
    pub fn can_appraise(&self) -> bool {
        self.remaining_appraisals > 0
    }

    /// Can this agent perform another memory operation (encode or retrieve) this tick?
    pub fn can_memory_op(&self) -> bool {
        self.remaining_memory_operations > 0
    }

    /// Can this agent perform another prospection this tick?
    pub fn can_prospect(&self) -> bool {
        self.remaining_prospections > 0
    }

    /// Can this agent perform another social inference (ToM) this tick?
    pub fn can_social_infer(&self) -> bool {
        self.remaining_social_inferences > 0
    }

    // ── Consume methods ─────────────────────────────────────────────

    /// Record one appraisal performed. Returns false if budget was already exhausted.
    #[must_use]
    pub fn consume_appraisal(&mut self) -> bool {
        if self.remaining_appraisals > 0 {
            self.remaining_appraisals -= 1;
            true
        } else {
            false
        }
    }

    /// Record one memory operation performed (encode or retrieve).
    #[must_use]
    pub fn consume_memory_op(&mut self) -> bool {
        if self.remaining_memory_operations > 0 {
            self.remaining_memory_operations -= 1;
            true
        } else {
            false
        }
    }

    /// Record one prospection performed.
    #[must_use]
    pub fn consume_prospection(&mut self) -> bool {
        if self.remaining_prospections > 0 {
            self.remaining_prospections -= 1;
            true
        } else {
            false
        }
    }

    /// Record one social inference performed.
    #[must_use]
    pub fn consume_social_inference(&mut self) -> bool {
        if self.remaining_social_inferences > 0 {
            self.remaining_social_inferences -= 1;
            true
        } else {
            false
        }
    }

    // ── Inspection methods ──────────────────────────────────────────

    /// Remaining appraisal budget.
    pub fn remaining_appraisals(&self) -> u32 {
        self.remaining_appraisals
    }

    /// Remaining memory operation budget.
    pub fn remaining_memory_operations(&self) -> u32 {
        self.remaining_memory_operations
    }

    /// Remaining prospection budget.
    pub fn remaining_prospections(&self) -> u32 {
        self.remaining_prospections
    }

    /// Remaining social inference budget.
    pub fn remaining_social_inferences(&self) -> u32 {
        self.remaining_social_inferences
    }
}

/// §17.1 + §17.2: Combined tier state stored on each agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTierState {
    /// Current simulation tier.
    pub tier: AgentTier,
    /// Cognitive budget limits for this tier.
    pub budget: CognitiveBudget,
    /// §17.2: Per-tick tracker enforcing the budget limits.
    /// Reset each tick via `reset_tick_budget()`. Serde skips this field;
    /// on deserialization it defaults to focal limits until the first tick reset.
    #[serde(skip)]
    pub budget_tracker: CognitiveBudgetTracker,
    /// Narrative importance score (0–1). Higher = more likely to be focal.
    pub narrative_importance: Fixed,
    /// Last tick when tier was reassigned.
    pub last_tier_reassign_tick: u64,
}

impl Default for AgentTierState {
    fn default() -> Self {
        let budget = CognitiveBudget::secondary();
        Self {
            tier: AgentTier::Secondary,
            budget_tracker: CognitiveBudgetTracker::from_budget(&budget),
            budget,
            narrative_importance: Fixed::from_f64(0.3),
            last_tier_reassign_tick: 0,
        }
    }
}

impl AgentTierState {
    /// Create a new tier state for a given tier.
    pub fn new(tier: AgentTier, tick: u64) -> Self {
        let budget = CognitiveBudget::for_tier(tier);
        Self {
            budget_tracker: CognitiveBudgetTracker::from_budget(&budget),
            budget,
            tier,
            narrative_importance: Fixed::from_f64(0.3),
            last_tier_reassign_tick: tick,
        }
    }

    /// §17.2: Reset the budget tracker for a new tick.
    /// Call at the start of each tick to replenish the agent's cognitive budget.
    pub fn reset_tick_budget(&mut self) {
        self.budget_tracker.reset(&self.budget);
    }

    /// §17.1: Reclassify an agent's tier based on current state.
    ///
    /// Promotion criteria (Secondary → Focal):
    /// - High status (wealth + role + social > 0.7)
    /// - Faction leadership or council membership
    /// - High narrative importance (> 0.6)
    /// - Crisis involvement (acute anger — Iteration-159 re-anchor)
    ///
    /// Demotion criteria (Focal → Secondary):
    /// - Low status, no institutional role, low narrative importance (< 0.5)
    /// - No acute crisis
    ///
    /// Demotion to Background:
    /// - Very low narrative importance (< 0.1) and no relationships
    ///   (structurally unreachable at village scale — every agent builds a
    ///   full relationship graph; reserved for sparse multi-settlement runs)
    pub fn reclassify(
        &mut self,
        status_effective: Fixed,
        is_institution_member: bool,
        is_faction_leader: bool,
        // Iteration 164: the queued §8.1.4 core-emotion decay landed, so
        // fear now differentiates (0.3–0.7 band with real per-agent spread
        // instead of the 0.83–1.0 saturation that motivated the Iteration
        // 159 anger-only anchor). The fear-based crisis return reserved in
        // the Iteration-159 comment is now wired: crisis = acute anger OR
        // elevated fear. Threshold 0.6 for fear reproduces the designed
        // §17.1 gradient — probe-pinned crisis counts 2/5/10/23/18 at
        // 6/12/24/48/48 agents, which combine with the status/role/
        // importance promotions to land the 4F/9F/11F/23F/19F envelope
        // (measured post-anchor 6: 6F, 12: 9F, 24: 17F, 48: 40F at 0.5 —
        // over-firing; 0.6 is the calibrated band) — a genuine
        // Focal/Secondary split with the differentiated tail, not the
        // all-Focal saturation (fear-anchored pre-159) nor the
        // mass-demotion ratchet (anger-only post-159, 4F/20S at 24).
        fear: Fixed,
        anger: Fixed,
        relationship_count: usize,
        tick: u64,
        min_reassign_interval: u64,
    ) {
        // Don't reclassify too often
        if tick.saturating_sub(self.last_tier_reassign_tick) < min_reassign_interval {
            return;
        }

        // §17.1 (Iteration 159 → Iteration 164): crisis is acute anger OR
        // elevated fear. Iteration 159 re-anchored to anger only because
        // chronic fear saturated at 0.83–1.0 (no proportional decay on the
        // base emotions), firing universally and promoting everyone to
        // Focal; the comment reserved the fear return for the queued
        // core-emotion decay iteration. That iteration (164) is what makes
        // fear a differentiated signal again: anger still marks acute
        // outrage/humiliation, fear now marks genuine distress in the
        // 0.3–0.7 band. Zero-fear calm worlds still fall back to the
        // role + narrative-importance gradient.
        let crisis = fear > Fixed::from_f64(0.6) || anger > Fixed::from_f64(0.6);
        let high_status = status_effective > Fixed::from_f64(0.7);
        let has_institutional_role = is_institution_member || is_faction_leader;

        let new_tier = match self.tier {
            AgentTier::Focal => {
                // Demote if low status, no role, no acute crisis, and low
                // importance — gated on tier MATURITY so the demotion only
                // fires once narrative importance has converged.
                // Iteration-159 probe: importance starts at ~0.31 for
                // everyone and converges (0.1/tick smoothing) toward
                // ~0.45–0.65 by tick 1000; an ungated threshold ratcheted
                // the whole village at the tick-100 interval (24: 4F/20S,
                // 48: 5F/43S — the converged floor ~0.5 never re-triggers a
                // > 0.6 promotion). With the 900-tick maturity gate the
                // demotion only fires on the converged distribution; the
                // hysteresis band [0.45, 0.5] against the promotion rule
                // prevents interval-to-interval oscillation.
                if tick.saturating_sub(self.last_tier_reassign_tick) >= TIER_MATURITY_TICKS
                    && !high_status
                    && !has_institutional_role
                    && !crisis
                    && self.narrative_importance < Fixed::from_f64(0.45)
                {
                    AgentTier::Secondary
                } else {
                    AgentTier::Focal
                }
            }
            AgentTier::Secondary => {
                // Promote if high status, institutional role, acute crisis,
                // or high importance (> 0.5 — was 0.7, Iteration 159; the
                // measured convergence floor for non-role agents is ~0.5
                // (universal fear + full relationship graph), so 0.7/0.55
                // were unreachable and only role-holders ever became Focal
                // (5F/43S at the 48-cap — the demotion path was irrelevant
                // because nobody was Focal to demote). Agents whose
                // importance crosses 0.5 as it converges are promoted, and
                // the low tail that stalls below 0.45 stays Secondary — a
                // genuine, stable gradient).
                if high_status
                    || has_institutional_role
                    || crisis
                    || self.narrative_importance > Fixed::from_f64(0.5)
                {
                    AgentTier::Focal
                } else if self.narrative_importance < Fixed::from_f64(0.1) && relationship_count < 2
                {
                    AgentTier::Background
                } else {
                    AgentTier::Secondary
                }
            }
            AgentTier::Background => {
                // Promote if any relevance signal appears
                if high_status
                    || has_institutional_role
                    || crisis
                    || self.narrative_importance > Fixed::from_f64(0.3)
                    || relationship_count >= 2
                {
                    AgentTier::Secondary
                } else {
                    AgentTier::Background
                }
            }
        };

        if new_tier != self.tier {
            self.tier = new_tier;
            self.budget = CognitiveBudget::for_tier(new_tier);
            // §17.2: Sync tracker limits when tier changes
            self.budget_tracker.reset(&self.budget);
            self.last_tier_reassign_tick = tick;
        }
    }

    /// Update narrative importance based on recent activity.
    ///
    /// Narrative importance rises when:
    /// - agent is in a faction or institution
    /// - agent has many relationships
    /// - agent is experiencing strong emotions
    /// - agent has recent story-worthy events
    pub fn update_narrative_importance(
        &mut self,
        has_institutional_role: bool,
        relationship_count: usize,
        emotional_intensity: Fixed,
        recent_event_count: u32,
    ) {
        let role_bonus = if has_institutional_role {
            Fixed::from_f64(0.2)
        } else {
            Fixed::ZERO
        };
        let network_bonus = (Fixed::from_f64(relationship_count as f64) * Fixed::from_f64(0.02))
            .min(Fixed::from_f64(0.2));
        let emotion_bonus = emotional_intensity * Fixed::from_f64(0.15);
        let event_bonus = (Fixed::from_f64(recent_event_count as f64) * Fixed::from_f64(0.05))
            .min(Fixed::from_f64(0.15));

        // Slowly move toward the target importance (smoothing factor 0.1)
        let target =
            (role_bonus + network_bonus + emotion_bonus + event_bonus + Fixed::from_f64(0.15))
                .clamp_01();
        self.narrative_importance = (self.narrative_importance * Fixed::from_f64(0.9)
            + target * Fixed::from_f64(0.1))
        .clamp_01();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_states_have_correct_budgets() {
        let focal = CognitiveBudget::focal();
        let secondary = CognitiveBudget::secondary();
        let background = CognitiveBudget::background();

        assert!(focal.max_appraisals > secondary.max_appraisals);
        assert!(secondary.max_appraisals > background.max_appraisals);
        assert_eq!(background.max_prospections, 0);
        assert_eq!(background.max_social_inferences, 0);
    }

    #[test]
    fn for_tier_matches_individual_constructors() {
        assert_eq!(
            CognitiveBudget::for_tier(AgentTier::Focal).max_appraisals,
            20
        );
        assert_eq!(
            CognitiveBudget::for_tier(AgentTier::Secondary).max_appraisals,
            5
        );
        assert_eq!(
            CognitiveBudget::for_tier(AgentTier::Background).max_memory_operations,
            0
        );
        assert_eq!(
            CognitiveBudget::for_tier(AgentTier::Background).max_appraisals,
            1
        );
    }

    #[test]
    fn tier_methods_match_expected_tiers() {
        assert!(AgentTier::Focal.runs_full_biology());
        assert!(!AgentTier::Secondary.runs_full_biology());
        assert!(!AgentTier::Background.runs_full_biology());

        assert!(AgentTier::Focal.runs_full_psychology());
        assert!(!AgentTier::Secondary.runs_full_psychology());
        assert!(!AgentTier::Background.runs_full_psychology());

        assert!(AgentTier::Focal.runs_theory_of_mind());
        assert!(!AgentTier::Secondary.runs_theory_of_mind());

        assert!(AgentTier::Focal.runs_prospection());
        assert!(!AgentTier::Secondary.runs_prospection());

        assert!(AgentTier::Focal.runs_relationship_updates());
        assert!(AgentTier::Secondary.runs_relationship_updates());
        assert!(!AgentTier::Background.runs_relationship_updates());

        assert!(AgentTier::Focal.runs_action_selection());
        assert!(AgentTier::Secondary.runs_action_selection());
        assert!(!AgentTier::Background.runs_action_selection());

        // §17.2 (Iteration 158): the social-participation gate now has a
        // production consumer — Background agents must be excluded from the
        // individual interaction pass (aggregate simulation).
        assert!(AgentTier::Focal.runs_social_interactions());
        assert!(AgentTier::Secondary.runs_social_interactions());
        assert!(!AgentTier::Background.runs_social_interactions());
    }

    #[test]
    fn reclassify_promotes_secondary_to_focal_on_high_status() {
        let mut state = AgentTierState::new(AgentTier::Secondary, 0);
        state.reclassify(
            Fixed::from_f64(0.8), // high status
            false,
            false,
            Fixed::ZERO,
            Fixed::ZERO,
            0,
            100,
            10, // min interval
        );
        assert_eq!(state.tier, AgentTier::Focal);
    }

    #[test]
    fn reclassify_respects_min_interval() {
        let mut state = AgentTierState::new(AgentTier::Secondary, 50);
        state.reclassify(
            Fixed::from_f64(0.8),
            false,
            false,
            Fixed::ZERO,
            Fixed::ZERO,
            0,
            100,
            100, // min interval = 100
        );
        // tick=100, last_reassign=50, diff=50 < 100 → no change
        assert_eq!(state.tier, AgentTier::Secondary);
    }

    #[test]
    fn reclassify_promotes_on_crisis() {
        // Iteration 159: crisis is acute-anger-driven (fear saturates
        // chronically at ~1.0 in every calibrated world — a fear threshold
        // fired universally and blocked the LOD gradient).
        let mut state = AgentTierState::new(AgentTier::Secondary, 0);
        state.reclassify(
            Fixed::from_f64(0.3), // low status
            false,
            false,
            Fixed::ZERO,          // fear alone no longer triggers crisis
            Fixed::from_f64(0.7), // high anger = acute crisis
            0,
            100,
            10,
        );
        assert_eq!(state.tier, AgentTier::Focal);
    }

    #[test]
    fn differentiated_elevated_fear_triggers_crisis_after_decay() {
        // Iteration 159: saturated chronic fear must NOT count as crisis —
        // otherwise every agent promotes and the gradient never forms.
        // That held while fear was permanently saturated (no base-emotion
        // decay). Iteration 164: the queued §8.1.4 core-emotion decay
        // landed, fear differentiates again, and the fear-based crisis
        // return reserved in the Iter-159 comment is now wired — crisis =
        // fear > 0.6 OR anger > 0.6. A DIFFERENTIATED elevated fear
        // (e.g. 0.7, genuinely distressed in the 0.3–0.7 band) now counts
        // as crisis and protects the agent from demotion; the test re-pins
        // that designed behavior. (The all-Focal saturation that motivated
        // the Iter-159 anger-only anchor cannot recur: the proportional
        // decay keeps fear at producer-driven steady states, so 0.99 fear
        // only exists for genuinely stressed agents.)
        let mut state = AgentTierState::new(AgentTier::Secondary, 0);
        state.reclassify(
            Fixed::from_f64(0.3),
            false,
            false,
            Fixed::from_f64(0.7), // elevated fear — the differentiated band
            Fixed::ZERO,          // but not acutely angry
            0,
            100,
            10,
        );
        assert_eq!(
            state.tier,
            AgentTier::Focal,
            "elevated fear must count as crisis (Iter-164 fear-crisis anchor)"
        );
        // Same state at a sub-threshold fear: no crisis → stays Secondary
        // (the role/importance gradient alone drives the split).
        let mut calm = AgentTierState::new(AgentTier::Secondary, 0);
        calm.reclassify(
            Fixed::from_f64(0.3),
            false,
            false,
            Fixed::from_f64(0.4), // sub-threshold fear
            Fixed::ZERO,
            0,
            100,
            10,
        );
        assert_eq!(calm.tier, AgentTier::Secondary);
    }

    #[test]
    fn reclassify_demotes_focal_below_importance_threshold() {
        // Iteration 159: the demotion anchor moved 0.3 → 0.45 (with a 300
        // tick maturity gate) so the Focal/Secondary gradient materializes
        // against the measured importance clustering without ratcheting the
        // whole village at the tick-100 interval.
        let mut state = AgentTierState::new(AgentTier::Focal, 0);
        state.narrative_importance = Fixed::from_f64(0.4);
        state.reclassify(
            Fixed::from_f64(0.2),
            false,
            false,
            Fixed::ZERO,
            Fixed::ZERO,
            0,
            1000, // mature (>= 300 ticks since last reassign)
            10,
        );
        assert_eq!(state.tier, AgentTier::Secondary);
    }

    #[test]
    fn reclassify_keeps_focal_above_importance_threshold() {
        // The hysteresis band [0.45, 0.55]: an agent at 0.5 stays Focal
        // (no demotion) without re-promotion churn.
        let mut state = AgentTierState::new(AgentTier::Focal, 0);
        state.narrative_importance = Fixed::from_f64(0.5);
        state.reclassify(
            Fixed::from_f64(0.2),
            false,
            false,
            Fixed::ZERO,
            Fixed::ZERO,
            0,
            1000,
            10,
        );
        assert_eq!(state.tier, AgentTier::Focal);
    }

    #[test]
    fn reclassify_demotion_respects_maturity_gate() {
        // Iteration 159: an immature tier (held < 300 ticks) must NOT be
        // demoted — early importance is uniformly low (~0.31) and would
        // ratchet the whole village to Secondary.
        let mut state = AgentTierState::new(AgentTier::Focal, 100);
        state.narrative_importance = Fixed::from_f64(0.3);
        state.reclassify(
            Fixed::from_f64(0.2),
            false,
            false,
            Fixed::ZERO,
            Fixed::ZERO,
            0,
            200, // 200 - 100 = 100 < 300 → no demotion
            10,
        );
        assert_eq!(state.tier, AgentTier::Focal);
    }

    #[test]
    fn reclassify_demotes_focal_to_secondary() {
        let mut state = AgentTierState::new(AgentTier::Focal, 0);
        state.narrative_importance = Fixed::from_f64(0.1);
        state.reclassify(
            Fixed::from_f64(0.2), // low status
            false,
            false,
            Fixed::ZERO,
            Fixed::ZERO,
            0,
            1000,
            10,
        );
        assert_eq!(state.tier, AgentTier::Secondary);
    }

    #[test]
    fn reclassify_demotes_secondary_to_background_on_low_relevance() {
        // The §17.1 Secondary → Background demotion path — an agent with
        // near-zero narrative importance and no relationships (no social
        // gravity, no story presence) drops to the aggregate tier. The
        // budget must follow (Background = zero memory/prospection/social
        // budgets) so the per-tick gates actually bite.
        let mut state = AgentTierState::new(AgentTier::Secondary, 0);
        state.narrative_importance = Fixed::from_f64(0.05);
        state.reclassify(
            Fixed::from_f64(0.2), // low status
            false,
            false,
            Fixed::ZERO,
            Fixed::ZERO,
            1, // relationship_count < 2
            1000,
            10,
        );
        assert_eq!(state.tier, AgentTier::Background);
        assert_eq!(state.budget.max_memory_operations, 0);
        assert_eq!(state.budget.max_prospections, 0);
        assert_eq!(state.budget.max_social_inferences, 0);
    }

    #[test]
    fn reclassify_keeps_secondary_when_relationships_exist() {
        // Guard: relationship_count >= 2 must block the demotion even at
        // near-zero narrative importance — an agent with friends is not
        // aggregate material.
        let mut state = AgentTierState::new(AgentTier::Secondary, 0);
        state.narrative_importance = Fixed::from_f64(0.05);
        state.reclassify(
            Fixed::from_f64(0.2),
            false,
            false,
            Fixed::ZERO,
            Fixed::ZERO,
            2, // relationship_count >= 2
            1000,
            10,
        );
        assert_eq!(state.tier, AgentTier::Secondary);
    }

    #[test]
    fn narrative_importance_increases_with_institutional_role() {
        let mut state = AgentTierState::new(AgentTier::Secondary, 0);
        state.narrative_importance = Fixed::from_f64(0.2);
        state.update_narrative_importance(
            true, // has institutional role
            5,
            Fixed::from_f64(0.3),
            2,
        );
        // Should have increased
        assert!(state.narrative_importance > Fixed::from_f64(0.2));
    }

    // ── §17.2 CognitiveBudgetTracker tests ─────────────────────────

    #[test]
    fn tracker_from_budget_has_full_counts() {
        let budget = CognitiveBudget::focal();
        let tracker = CognitiveBudgetTracker::from_budget(&budget);
        assert_eq!(tracker.remaining_appraisals(), 20);
        assert_eq!(tracker.remaining_memory_operations(), 10);
        assert_eq!(tracker.remaining_prospections(), 5);
        assert_eq!(tracker.remaining_social_inferences(), 10);
    }

    #[test]
    fn tracker_consume_decrements_remaining() {
        let budget = CognitiveBudget::focal();
        let mut tracker = CognitiveBudgetTracker::from_budget(&budget);
        assert!(tracker.can_appraise());
        assert!(tracker.consume_appraisal());
        assert_eq!(tracker.remaining_appraisals(), 19);
        assert!(tracker.can_prospect());
        assert!(tracker.consume_prospection());
        assert_eq!(tracker.remaining_prospections(), 4);
    }

    #[test]
    fn tracker_exhaustion_prevents_further_consumption() {
        let budget = CognitiveBudget::secondary(); // max_appraisals = 5
        let mut tracker = CognitiveBudgetTracker::from_budget(&budget);
        for _ in 0..5 {
            assert!(tracker.consume_appraisal());
        }
        assert!(!tracker.can_appraise());
        assert!(!tracker.consume_appraisal());
        assert_eq!(tracker.remaining_appraisals(), 0);
    }

    #[test]
    fn tracker_reset_restores_full_budget() {
        let budget = CognitiveBudget::focal();
        let mut tracker = CognitiveBudgetTracker::from_budget(&budget);
        for _ in 0..20 {
            let _ = tracker.consume_appraisal();
        }
        assert!(!tracker.can_appraise());
        tracker.reset(&budget);
        assert!(tracker.can_appraise());
        assert_eq!(tracker.remaining_appraisals(), 20);
    }

    #[test]
    fn tracker_background_has_zero_prospection() {
        let budget = CognitiveBudget::background();
        let tracker = CognitiveBudgetTracker::from_budget(&budget);
        assert!(!tracker.can_prospect());
        assert!(!tracker.can_social_infer());
        assert!(!tracker.can_memory_op());
        assert!(tracker.can_appraise()); // background gets 1 appraisal
    }

    #[test]
    fn tier_state_reset_tick_budget_works() {
        let mut state = AgentTierState::new(AgentTier::Focal, 0);
        // Exhaust some budget
        let _ = state.budget_tracker.consume_appraisal();
        let _ = state.budget_tracker.consume_appraisal();
        assert_eq!(state.budget_tracker.remaining_appraisals(), 18);
        // Reset tick budget
        state.reset_tick_budget();
        assert_eq!(state.budget_tracker.remaining_appraisals(), 20);
    }

    #[test]
    fn tier_reclassification_updates_tracker() {
        let mut state = AgentTierState::new(AgentTier::Secondary, 0);
        assert_eq!(state.budget_tracker.remaining_prospections(), 0);
        // Promote to focal
        state.reclassify(
            Fixed::from_f64(0.8),
            false,
            false,
            Fixed::ZERO,
            Fixed::ZERO,
            0,
            100,
            10,
        );
        assert_eq!(state.tier, AgentTier::Focal);
        // Budget tracker should now have focal limits
        assert_eq!(state.budget_tracker.remaining_prospections(), 5);
    }
}
