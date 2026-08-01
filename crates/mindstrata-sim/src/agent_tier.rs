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

/// Simulation tier for an agent — determines which systems run each tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AgentTier {
    /// Full simulation: biology, psychology, relationships, memory, prospection, narrative.
    Focal,
    /// Reduced simulation: simplified biology, heuristic cognition, relationship updates only when relevant.
    Secondary,
    /// Aggregate simulation: household-level behavior, statistical belief updates, minimal memory.
    Background,
}

impl Default for AgentTier {
    fn default() -> Self {
        Self::Secondary
    }
}

impl AgentTier {
    /// Does this tier run the full biological update (endocrine, metabolism, cardiovascular, etc.)?
    pub fn runs_full_biology(&self) -> bool {
        matches!(self, Self::Focal)
    }

    /// Does this tier run simplified biology (just decay/satiation)?
    pub fn runs_simplified_biology(&self) -> bool {
        matches!(self, Self::Focal | Self::Secondary)
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

    /// Does this tier run meme propagation?
    pub fn runs_meme_propagation(&self) -> bool {
        matches!(self, Self::Focal | Self::Secondary)
    }

    /// Does this tier run institutional psychology?
    pub fn runs_institutional_psychology(&self) -> bool {
        matches!(self, Self::Focal)
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
    /// Maximum memory retrievals per tick.
    pub max_memory_retrievals: u32,
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
            max_memory_retrievals: 10,
            max_prospections: 5,
            max_social_inferences: 10,
        }
    }

    /// Reduced budget for secondary agents.
    pub fn secondary() -> Self {
        Self {
            max_appraisals: 5,
            max_memory_retrievals: 3,
            max_prospections: 0,
            max_social_inferences: 2,
        }
    }

    /// Minimal budget for background agents.
    pub fn background() -> Self {
        Self {
            max_appraisals: 1,
            max_memory_retrievals: 0,
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

/// §17.1 + §17.2: Combined tier state stored on each agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTierState {
    /// Current simulation tier.
    pub tier: AgentTier,
    /// Cognitive budget for this tick.
    pub budget: CognitiveBudget,
    /// Narrative importance score (0–1). Higher = more likely to be focal.
    pub narrative_importance: Fixed,
    /// Last tick when tier was reassigned.
    pub last_tier_reassign_tick: u64,
}

impl Default for AgentTierState {
    fn default() -> Self {
        Self {
            tier: AgentTier::Secondary,
            budget: CognitiveBudget::secondary(),
            narrative_importance: Fixed::from_f64(0.3),
            last_tier_reassign_tick: 0,
        }
    }
}

impl AgentTierState {
    /// Create a new tier state for a given tier.
    pub fn new(tier: AgentTier, tick: u64) -> Self {
        Self {
            budget: CognitiveBudget::for_tier(tier),
            tier,
            narrative_importance: Fixed::from_f64(0.3),
            last_tier_reassign_tick: tick,
        }
    }

    /// §17.1: Reclassify an agent's tier based on current state.
    ///
    /// Promotion criteria (Secondary → Focal):
    /// - High status (wealth + role + social > 0.7)
    /// - Faction leadership or council membership
    /// - High narrative importance (> 0.7)
    /// - Crisis involvement (high fear or anger)
    ///
    /// Demotion criteria (Focal → Secondary):
    /// - Low status, no institutional role, low narrative importance
    /// - No crisis involvement for 1000+ ticks
    ///
    /// Demotion to Background:
    /// - Very low narrative importance (< 0.1) and no relationships
    pub fn reclassify(
        &mut self,
        status_effective: Fixed,
        is_institution_member: bool,
        is_faction_leader: bool,
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

        let crisis = fear > Fixed::from_f64(0.6) || anger > Fixed::from_f64(0.6);
        let high_status = status_effective > Fixed::from_f64(0.7);
        let has_institutional_role = is_institution_member || is_faction_leader;

        let new_tier = match self.tier {
            AgentTier::Focal => {
                // Demote if low status, no role, low importance, no crisis
                if !high_status && !has_institutional_role && !crisis
                    && self.narrative_importance < Fixed::from_f64(0.3)
                {
                    AgentTier::Secondary
                } else {
                    AgentTier::Focal
                }
            }
            AgentTier::Secondary => {
                // Promote if high status, institutional role, or crisis
                if high_status || has_institutional_role || crisis
                    || self.narrative_importance > Fixed::from_f64(0.7)
                {
                    AgentTier::Focal
                } else if self.narrative_importance < Fixed::from_f64(0.1)
                    && relationship_count < 2
                {
                    AgentTier::Background
                } else {
                    AgentTier::Secondary
                }
            }
            AgentTier::Background => {
                // Promote if any relevance signal appears
                if high_status || has_institutional_role || crisis
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
        let target = (role_bonus + network_bonus + emotion_bonus + event_bonus + Fixed::from_f64(0.15)).clamp_01();
        self.narrative_importance =
            (self.narrative_importance * Fixed::from_f64(0.9) + target * Fixed::from_f64(0.1))
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
        assert_eq!(CognitiveBudget::for_tier(AgentTier::Focal).max_appraisals, 20);
        assert_eq!(CognitiveBudget::for_tier(AgentTier::Secondary).max_appraisals, 5);
        assert_eq!(CognitiveBudget::for_tier(AgentTier::Background).max_appraisals, 1);
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
    }

    #[test]
    fn reclassify_promotes_secondary_to_focal_on_high_status() {
        let mut state = AgentTierState::new(AgentTier::Secondary, 0);
        state.reclassify(
            Fixed::from_f64(0.8),  // high status
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
        let mut state = AgentTierState::new(AgentTier::Secondary, 0);
        state.reclassify(
            Fixed::from_f64(0.3), // low status
            false,
            false,
            Fixed::from_f64(0.7), // high fear
            Fixed::ZERO,
            0,
            100,
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
}
