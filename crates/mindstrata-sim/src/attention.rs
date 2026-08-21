//! Attention and salience system.
//!
//! Agents cannot appraise everything. Attention is limited by:
//! - attention budget (cognitive capacity)
//! - habituation (repeated stimuli become less salient)
//! - emotional relevance (emotionally charged events grab attention)
//! - survival relevance (threats to needs grab attention)
//! - novelty (new things are more salient)
//!
//! §22.5 of the architecture spec.

use crate::person::{Affect, NeedState};
use mindstrata_core::event::{InteractionKind, SimEvent};
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// An agent's attention state.
///
/// Controls which events the agent notices and appraises.
/// §22.5: "Agents cannot appraise everything."
///
/// §8.1.2 (AP2): this is the cognitive gateway — upgraded with the three
/// perceptual-bias dimensions (threat, social, novelty), the salience
/// competition record, and the plan's `attention_capacity()` naming for the
/// budget. The biases are recomputed from existing agent state by pure
/// functions (no RNG) and — since Iteration 97 — **feed back into
/// [`AttentionState::compute_salience`]** as a class-matched multiplicative
/// fold (`bias / 0.5`, exactly 1.0 at the neutral anchor, so a
/// neutral-bias agent is byte-identical to the pre-Iter-97 gate). The
/// salience map remains observational state.
///
/// Generalization note: the plan's signature emergent effects — "fearful
/// agents notice threats", "hungry agents notice food" — are today hardcoded
/// as flat boosts inside [`AttentionState::event_relevance`] (negative affect
/// → threat +0.2, hunger/thirst → resource +0.3). The bias dimensions
/// generalize the *threat/social/novelty* paths: `threat_bias` scales
/// Threat percepts, `social_bias` scales Social percepts, `novelty_bias`
/// scales Other (novel) percepts; Resource and Emotional percepts have no
/// matching dimension and keep the flat boosts untouched (fold 1.0). The
/// flat boosts were NOT removed — removing them would shift the calibrated
/// gate even at the anchor, while the multiplicative fold is zero-at-zero
/// by construction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionState {
    /// Current focus — what the agent is paying attention to.
    pub focus: AttentionFocus,
    /// Attention budget: how much cognitive capacity is available (0..1).
    /// Depleted by stress, fatigue, emotional overwhelm.
    /// §8.1.2: exposed as `attention_capacity()`.
    pub budget: Fixed,
    /// Habituation levels per event category (repeated stimuli become less salient).
    pub habituation: Habituation,
    /// Overall salience bias: shifts how easily things grab attention.
    /// High = easily distracted, low = focused.
    pub salience_bias: Fixed,
    /// Residual attention from the last tick (carries over).
    pub residual: Fixed,
    /// §8.1.2: Threat bias — hypervigilance toward threats. 0.5 is the neutral
    /// anchor; rises with fear, anger, and trauma risk (trauma-triggered
    /// hypervigilance). Observational state.
    #[serde(default = "default_neutral_bias")]
    pub threat_bias: Fixed,
    /// §8.1.2: Social bias — attention to other agents (lovers notice each
    /// other). Rises with extraversion and attachment security.
    #[serde(default = "default_neutral_bias")]
    pub social_bias: Fixed,
    /// §8.1.2: Novelty bias — attention to new stimuli (open agents notice
    /// more). Rises with openness.
    #[serde(default = "default_neutral_bias")]
    pub novelty_bias: Fixed,
    /// §8.1.2: The salience competition record — this tick's percepts ranked
    /// by computed salience (bounded to [`SALIENCE_MAP_CAPACITY`], cleared each
    /// tick). Records every percept the gateway processed — including losers
    /// below the memory-encoding threshold, so the winners (salience ≥ 0.2) are
    /// identifiable within the competition. Observational state.
    #[serde(default)]
    pub salience_map: Vec<SalientItem>,
}

/// What the agent is currently focused on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttentionFocus {
    /// Focused on internal state (needs, emotions).
    Internal,
    /// Focused on a specific agent.
    Agent(AgentId),
    /// Focused on a site/location.
    Site(usize),
    /// Scanning the environment broadly.
    Scanning,
}

/// The neutral anchor (0.5) for the perceptual biases — identity-at-baseline
/// reference, also the serde default for pre-Iter-41 snapshots.
fn default_neutral_bias() -> Fixed {
    Fixed::from_raw(5000)
}

impl Default for AttentionState {
    fn default() -> Self {
        Self {
            focus: AttentionFocus::Scanning,
            budget: Fixed::from_f64(0.8),
            habituation: Habituation::default(),
            salience_bias: Fixed::from_f64(0.5),
            residual: Fixed::ZERO,
            threat_bias: Fixed::from_raw(5000),
            social_bias: Fixed::from_raw(5000),
            novelty_bias: Fixed::from_raw(5000),
            salience_map: Vec::new(),
        }
    }
}

/// Habituation levels for different event categories.
/// Repeated exposure reduces salience (§22.5).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Habituation {
    /// Habituation to social interactions (talk, trade).
    pub social: Fixed,
    /// Habituation to threats and violence.
    pub threat: Fixed,
    /// Habituation to resource-related events (eating, drinking).
    pub resource: Fixed,
    /// Habituation to emotional events.
    pub emotional: Fixed,
}

/// §8.1.2: The percept category of an event — mirrors the habituation
/// categories, so each percept maps to the dimension it habituates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PerceptKind {
    /// Interactions (talk, trade, help, gossip — non-threatening).
    Social,
    /// Threats and insults.
    Threat,
    /// Eating and drinking.
    Resource,
    /// Relationship changes.
    Emotional,
    /// Everything else.
    Other,
}

impl PerceptKind {
    /// Classify a simulation event into its percept category — mirrors the
    /// `event_habituation` match so classification is always consistent.
    #[must_use]
    pub fn of(event: &SimEvent) -> PerceptKind {
        match event {
            SimEvent::InteractionOccurred {
                kind: InteractionKind::Threaten | InteractionKind::Insult,
                ..
            } => PerceptKind::Threat,
            SimEvent::InteractionOccurred { .. } => PerceptKind::Social,
            SimEvent::AgentAte { .. } | SimEvent::AgentDrank { .. } => PerceptKind::Resource,
            SimEvent::RelationshipChanged { .. } => PerceptKind::Emotional,
            _ => PerceptKind::Other,
        }
    }
}

/// §8.1.2: One entry in the salience competition — an event that competed for
/// the agent's attention this tick and the salience it scored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalientItem {
    /// The percept category.
    pub kind: PerceptKind,
    /// The other agent involved, when the percept is agent-mediated.
    pub agent: Option<AgentId>,
    /// Computed salience (0..1) — the competition score.
    pub salience: Fixed,
    /// Tick the percept occurred.
    pub tick: u64,
}

/// §8.1.2: Maximum entries kept in the salience competition record — the top
/// winners, by computed salience.
pub const SALIENCE_MAP_CAPACITY: usize = 8;

impl AttentionState {
    /// Compute the salience of an event for this agent.
    ///
    /// salience = intensity * novelty * relevance * bias - habituation
    ///
    /// Returns a value in [0, 1] indicating how much attention this event deserves.
    pub fn compute_salience(
        &mut self,
        event: &SimEvent,
        agent_id: AgentId,
        needs: &NeedState,
        affect: &Affect,
    ) -> Fixed {
        let base_intensity = Self::event_base_intensity(event, agent_id);
        let novelty = Self::event_novelty(event, &self.habituation);
        let relevance = Self::event_relevance(event, agent_id, needs, affect);
        let habituation_penalty = Self::event_habituation(event, &mut self.habituation);

        // §8.1.2 (Iteration 97): the perceptual-bias fold — the class-matched
        // bias normalized by the 0.5 neutral anchor (exactly 1.0 at anchor,
        // so a neutral-bias agent is byte-identical to the pre-Iter-97 gate;
        // a hypervigilant agent's threat percepts, an extravert's social
        // percepts, and an open agent's novel percepts score higher).
        let raw_salience = base_intensity
            * novelty
            * relevance
            * self.salience_bias
            * self.percept_bias_factor(event);
        let salience = (raw_salience - habituation_penalty).clamp_01();

        // Update habituation (exposure increases habituation)
        self.update_habituation(event, salience);

        salience
    }

    /// Base intensity of an event type (how inherently attention-grabbing it is).
    fn event_base_intensity(event: &SimEvent, _agent_id: AgentId) -> Fixed {
        match event {
            // Threats and insults are highly intense
            SimEvent::InteractionOccurred {
                kind: InteractionKind::Threaten | InteractionKind::Insult,
                ..
            } => Fixed::from_f64(0.9),
            // Help and comfort are moderately intense
            SimEvent::InteractionOccurred {
                kind: InteractionKind::Help | InteractionKind::Comfort,
                ..
            } => Fixed::from_f64(0.6),
            // Other interactions are mild
            SimEvent::InteractionOccurred { .. } => Fixed::from_f64(0.4),
            // Resource events are moderate
            SimEvent::AgentAte { .. } | SimEvent::AgentDrank { .. } => Fixed::from_f64(0.3),
            // Relationship changes are moderate
            SimEvent::RelationshipChanged { trust_delta, .. } => {
                // Large trust changes are more intense
                let magnitude = trust_delta.abs();
                Fixed::from_f64(0.3) + magnitude * Fixed::from_f64(0.5)
            }
            // Rumors are moderately intense
            SimEvent::RumorSpread { .. } => Fixed::from_f64(0.5),
            // Everything else is mild
            _ => Fixed::from_f64(0.2),
        }
    }

    /// Novelty factor: new/unexpected events are more salient.
    fn event_novelty(event: &SimEvent, habituation: &Habituation) -> Fixed {
        let habituation_level = match event {
            SimEvent::InteractionOccurred {
                kind: InteractionKind::Threaten | InteractionKind::Insult,
                ..
            } => habituation.threat,
            SimEvent::InteractionOccurred { .. } => habituation.social,
            SimEvent::AgentAte { .. } | SimEvent::AgentDrank { .. } => habituation.resource,
            SimEvent::RelationshipChanged { .. } => habituation.emotional,
            _ => Fixed::from_f64(0.3),
        };
        // Novelty = 1 - habituation (habituated events are less novel)
        Fixed::ONE - habituation_level
    }

    /// §8.1.2 (Iteration 97): the class-matched perceptual-bias fold — the
    /// bias for the event's percept class normalized by the 0.5 neutral
    /// anchor. Exactly [`Fixed::ONE`] at the anchor for every class, so the
    /// fold is zero-at-zero by construction. Threat percepts scale with
    /// `threat_bias` (hypervigilance), Social with `social_bias`, Other
    /// (novel) with `novelty_bias`; Resource and Emotional percepts have no
    /// matching bias dimension and fold at 1.0 (their flat hunger/thirst/
    /// trust boosts in [`AttentionState::event_relevance`] stay untouched).
    fn percept_bias_factor(&self, event: &SimEvent) -> Fixed {
        let bias = match PerceptKind::of(event) {
            PerceptKind::Threat => self.threat_bias,
            PerceptKind::Social => self.social_bias,
            PerceptKind::Other => self.novelty_bias,
            PerceptKind::Resource | PerceptKind::Emotional => return Fixed::ONE,
        };
        // bias / 0.5 — Fixed division keeps the anchor fold exact (5000/5000).
        bias / Fixed::from_f64(0.5)
    }

    /// Relevance: how important is this event to the agent's current state?
    fn event_relevance(
        event: &SimEvent,
        agent_id: AgentId,
        needs: &NeedState,
        affect: &Affect,
    ) -> Fixed {
        let mut relevance = Fixed::from_f64(0.3); // base relevance

        match event {
            // Events involving the agent directly are highly relevant
            SimEvent::InteractionOccurred { from, to, .. }
            | SimEvent::RelationshipChanged { from, to, .. } => {
                if *from == agent_id || *to == agent_id {
                    relevance = Fixed::from_f64(0.9);
                }
            }
            SimEvent::AgentAte { agent, .. } | SimEvent::AgentDrank { agent, .. }
                if *agent == agent_id =>
            {
                relevance = Fixed::from_f64(0.8);
            }
            _ => {}
        }

        // Hungry agents find food-related events more relevant
        if needs.hunger > Fixed::from_f64(0.7) && matches!(event, SimEvent::AgentAte { .. }) {
            relevance = (relevance + Fixed::from_f64(0.3)).clamp_01();
        }

        // Thirsty agents find water-related events more relevant
        if needs.thirst > Fixed::from_f64(0.7) && matches!(event, SimEvent::AgentDrank { .. }) {
            relevance = (relevance + Fixed::from_f64(0.3)).clamp_01();
        }

        // Negative affect makes threat events more relevant (anxiety bias)
        if affect.valence < Fixed::ZERO
            && matches!(
                event,
                SimEvent::InteractionOccurred {
                    kind: InteractionKind::Threaten | InteractionKind::Insult,
                    ..
                }
            )
        {
            relevance = (relevance + Fixed::from_f64(0.2)).clamp_01();
        }

        relevance
    }

    /// Habituation penalty for a specific event type.
    fn event_habituation(event: &SimEvent, habituation: &mut Habituation) -> Fixed {
        match event {
            SimEvent::InteractionOccurred {
                kind: InteractionKind::Threaten | InteractionKind::Insult,
                ..
            } => habituation.threat,
            SimEvent::InteractionOccurred { .. } => habituation.social,
            SimEvent::AgentAte { .. } | SimEvent::AgentDrank { .. } => habituation.resource,
            SimEvent::RelationshipChanged { .. } => habituation.emotional,
            _ => Fixed::ZERO,
        }
    }

    /// Update habituation after exposure to an event.
    fn update_habituation(&mut self, event: &SimEvent, salience: Fixed) {
        let increment = salience * Fixed::from_f64(0.02); // slow habituation
        match event {
            SimEvent::InteractionOccurred {
                kind: InteractionKind::Threaten | InteractionKind::Insult,
                ..
            } => {
                self.habituation.threat = (self.habituation.threat + increment).clamp_01();
            }
            SimEvent::InteractionOccurred { .. } => {
                self.habituation.social = (self.habituation.social + increment).clamp_01();
            }
            SimEvent::AgentAte { .. } | SimEvent::AgentDrank { .. } => {
                self.habituation.resource = (self.habituation.resource + increment).clamp_01();
            }
            SimEvent::RelationshipChanged { .. } => {
                self.habituation.emotional = (self.habituation.emotional + increment).clamp_01();
            }
            _ => {}
        }
    }

    /// Decay habituation over time (agents "forget" to ignore things).
    pub fn decay_habituation(&mut self, rate: Fixed) {
        self.habituation.social = (self.habituation.social - rate).max(Fixed::ZERO);
        self.habituation.threat = (self.habituation.threat - rate).max(Fixed::ZERO);
        self.habituation.resource = (self.habituation.resource - rate).max(Fixed::ZERO);
        self.habituation.emotional = (self.habituation.emotional - rate).max(Fixed::ZERO);
    }

    /// §8.1.2: The plan's name for the attention capacity — the existing budget.
    #[must_use]
    pub fn attention_capacity(&self) -> Fixed {
        self.budget
    }

    /// §8.1.2: Recompute the perceptual biases from existing agent state.
    ///
    /// Pure and deterministic (no RNG). Since Iteration 97 the biases feed
    /// back into [`AttentionState::compute_salience`] via
    /// [`AttentionState::percept_bias_factor`]; they stay within [0.3, 0.7]
    /// around the 0.5 neutral anchor. threat_bias rises with fear, anger,
    /// and trauma risk (trauma-triggered hypervigilance); social_bias with
    /// extraversion and attachment security; novelty_bias with openness.
    ///
    /// §7.2.2 (Iteration 187 — the S2-2-2 arousal consumer): the biological
    /// endocrine arousal axis (`endocrine.arousal.level`, adrenaline-like)
    /// also feeds threat bias — acute physiological stress is hypervigilance.
    /// Mean-zero at the 0.5 anchor: `(arousal − 0.5) × 0.5` adds at most
    /// ±0.25 to `threat_input` at the extremes, so a baseline-aroused agent
    /// is byte-identical to the pre-Iter-187 gate and the fold is
    /// ONE-SIDED (arousal can only push threat bias up or down, never
    /// through a sign flip of the input).
    pub fn recompute_biases(
        &mut self,
        fear: Fixed,
        anger: Fixed,
        trauma_risk: Fixed,
        extraversion: Fixed,
        openness: Fixed,
        attachment_security: Fixed,
        prediction_error: Fixed,
        arousal: Fixed,
    ) {
        let threat_input =
            fear + anger + trauma_risk + (arousal - Fixed::from_f64(0.5)) * Fixed::from_f64(0.5);
        self.threat_bias = (Fixed::from_raw(5000)
            + (threat_input - Fixed::ONE) * Fixed::from_f64(0.2))
        .clamp(Fixed::from_f64(0.3), Fixed::from_f64(0.7));
        self.social_bias = (Fixed::from_raw(5000)
            + (extraversion - Fixed::from_f64(0.5)) * Fixed::from_f64(0.3)
            + (attachment_security - Fixed::from_f64(0.5)) * Fixed::from_f64(0.2))
        .clamp(Fixed::from_f64(0.3), Fixed::from_f64(0.7));
        // §9.2 (Iteration 183d): large prediction error increases attention —
        // surprise raises novelty-seeking bias (a violated world model
        // reorients the agent toward novel percepts). The caller passes
        // `last_prediction_error` (the prior tick's surprise), which is 0.0
        // in calm windows, so calibrated trajectories are byte-identical. At
        // 0.2 gain a 0.3 surprise lifts the bias by 0.06, inside the
        // [0.3, 0.7] clamp.
        self.novelty_bias = (Fixed::from_raw(5000)
            + (openness - Fixed::from_f64(0.5)) * Fixed::from_f64(0.3)
            + prediction_error * Fixed::from_f64(0.2))
        .clamp(Fixed::from_f64(0.3), Fixed::from_f64(0.7));
    }

    /// §8.1.2: Record a percept into the salience competition. Keeps the top
    /// [`SALIENCE_MAP_CAPACITY`] entries by computed salience, always sorted
    /// descending so the ranking contract is uniform at any size.
    pub fn record_salience(
        &mut self,
        kind: PerceptKind,
        agent: Option<AgentId>,
        salience: Fixed,
        tick: u64,
    ) {
        self.salience_map.push(SalientItem {
            kind,
            agent,
            salience,
            tick,
        });
        self.salience_map
            .sort_by_key(|s| std::cmp::Reverse(s.salience));
        if self.salience_map.len() > SALIENCE_MAP_CAPACITY {
            self.salience_map.truncate(SALIENCE_MAP_CAPACITY);
        }
    }

    /// Replenish attention budget (rest, low stress).
    pub fn replenish_budget(&mut self, stress: Fixed, fatigue: Fixed) {
        let replenishment = Fixed::from_f64(0.05) * (Fixed::ONE - stress) * (Fixed::ONE - fatigue);
        self.budget = (self.budget + replenishment).clamp_01();
    }

    /// Deplete attention budget (high stress, emotional overwhelm).
    pub fn deplete_budget(&mut self, amount: Fixed) {
        self.budget = (self.budget - amount).max(Fixed::ZERO);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mindstrata_core::clock::Tick;

    #[test]
    fn threat_events_are_highly_salient() {
        let mut attention = AttentionState::default();
        let needs = NeedState::default();
        let affect = Affect::default();
        let agent_id = AgentId::new(0);

        let threat_event = SimEvent::InteractionOccurred {
            from: AgentId::new(1),
            to: AgentId::new(0),
            kind: InteractionKind::Threaten,
            tick: Tick::new(1),
        };

        let salience = attention.compute_salience(&threat_event, agent_id, &needs, &affect);
        // base_intensity(0.9) * novelty(1.0) * relevance(0.9) * bias(0.5) = 0.405
        // This is the highest salience category — threat events dominate.
        assert!(
            salience > Fixed::from_f64(0.3),
            "Threat events should be highly salient, got {}",
            salience.to_f64()
        );
    }

    #[test]
    fn direct_events_are_more_relevant() {
        let mut attention = AttentionState::default();
        let needs = NeedState::default();
        let affect = Affect::default();
        let agent_id = AgentId::new(0);

        // Event involving the agent
        let direct_event = SimEvent::InteractionOccurred {
            from: AgentId::new(1),
            to: AgentId::new(0),
            kind: InteractionKind::Talk,
            tick: Tick::new(1),
        };

        // Event not involving the agent
        let indirect_event = SimEvent::InteractionOccurred {
            from: AgentId::new(2),
            to: AgentId::new(3),
            kind: InteractionKind::Talk,
            tick: Tick::new(1),
        };

        let salience_direct = attention.compute_salience(&direct_event, agent_id, &needs, &affect);
        let salience_indirect =
            attention.compute_salience(&indirect_event, agent_id, &needs, &affect);

        assert!(
            salience_direct > salience_indirect,
            "Direct events should be more salient: direct={}, indirect={}",
            salience_direct.to_f64(),
            salience_indirect.to_f64()
        );
    }

    #[test]
    fn habituation_reduces_salience() {
        let mut attention = AttentionState::default();
        let needs = NeedState::default();
        let affect = Affect::default();
        let agent_id = AgentId::new(0);

        let event = SimEvent::InteractionOccurred {
            from: AgentId::new(1),
            to: AgentId::new(0),
            kind: InteractionKind::Talk,
            tick: Tick::new(1),
        };

        let salience_first = attention.compute_salience(&event, agent_id, &needs, &affect);
        let salience_second = attention.compute_salience(&event, agent_id, &needs, &affect);

        assert!(
            salience_second <= salience_first,
            "Habituation should reduce salience: first={}, second={}",
            salience_first.to_f64(),
            salience_second.to_f64()
        );
    }

    #[test]
    fn hunger_increases_food_relevance() {
        let mut attention = AttentionState::default();
        let agent_id = AgentId::new(0);
        let affect = Affect::default();

        let hungry_needs = NeedState {
            hunger: Fixed::from_f64(0.9),
            ..Default::default()
        };
        let satiated_needs = NeedState {
            hunger: Fixed::from_f64(0.1),
            ..Default::default()
        };

        let event = SimEvent::AgentAte {
            agent: AgentId::new(2),
            food: mindstrata_core::id::EntityId::new(0),
            tick: Tick::new(1),
        };

        let salience_hungry = attention.compute_salience(&event, agent_id, &hungry_needs, &affect);
        let salience_satiated =
            attention.compute_salience(&event, agent_id, &satiated_needs, &affect);

        assert!(
            salience_hungry > salience_satiated,
            "Hungry agents should find food events more salient: hungry={}, satiated={}",
            salience_hungry.to_f64(),
            salience_satiated.to_f64()
        );
    }

    #[test]
    fn habituation_decays() {
        let mut attention = AttentionState::default();
        attention.habituation.social = Fixed::from_f64(0.5);

        attention.decay_habituation(Fixed::from_f64(0.1));
        assert_eq!(attention.habituation.social, Fixed::from_f64(0.4));
    }

    #[test]
    fn budget_depletes_and_replenishes() {
        let mut attention = AttentionState::default();
        let initial = attention.budget;

        attention.deplete_budget(Fixed::from_f64(0.3));
        assert!(attention.budget < initial);

        attention.replenish_budget(Fixed::ZERO, Fixed::ZERO); // no stress, no fatigue
        assert!(attention.budget > initial - Fixed::from_f64(0.3));
    }

    // ── §8.1.2: Perception and Attention upgrade ───────────────────

    #[test]
    fn perception_biases_default_to_neutral() {
        let attention = AttentionState::default();
        assert_eq!(attention.threat_bias, Fixed::from_f64(0.5));
        assert_eq!(attention.social_bias, Fixed::from_f64(0.5));
        assert_eq!(attention.novelty_bias, Fixed::from_f64(0.5));
        assert!(attention.salience_map.is_empty());
        assert_eq!(attention.attention_capacity(), attention.budget);
    }

    #[test]
    fn threat_bias_rises_with_fear_and_trauma() {
        let mut attention = AttentionState::default();
        attention.recompute_biases(
            Fixed::from_f64(0.9), // fear
            Fixed::from_f64(0.5), // anger
            Fixed::from_f64(0.7), // trauma_risk
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::from_f64(0.5), // arousal at the mean-zero anchor
        );
        assert!(attention.threat_bias > Fixed::from_f64(0.5));
        assert!(attention.threat_bias <= Fixed::from_f64(0.7));
        // Calm agents sit at (or below) the neutral anchor.
        attention.recompute_biases(
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::from_f64(0.5),
        );
        assert!(attention.threat_bias < Fixed::from_f64(0.5));
    }

    #[test]
    fn threat_bias_rises_with_endocrine_arousal() {
        // §7.2.2 (Iteration 187): the biological arousal axis (adrenaline)
        // feeds hypervigilance — a physiologically aroused agent perceives
        // more threat at identical fear/anger/trauma inputs.
        let mut low = AttentionState::default();
        let mut high = AttentionState::default();
        low.recompute_biases(
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::from_f64(0.3), // low arousal
        );
        high.recompute_biases(
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::from_f64(0.9), // high arousal
        );
        assert!(high.threat_bias > low.threat_bias);
        // The anchor is identity: at arousal 0.5 the fold term is exactly 0,
        // so the bias equals the legacy formula 0.5 + (0 − 1)×0.2 = 0.3
        // (clamped) — byte-identical to the pre-Iter-187 value.
        let mut anchor = AttentionState::default();
        anchor.recompute_biases(
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::from_f64(0.5),
        );
        assert_eq!(anchor.threat_bias, Fixed::from_f64(0.3));
        assert!(
            high.threat_bias <= Fixed::from_f64(0.7),
            "stays in the clamp"
        );
    }

    #[test]
    fn social_bias_rises_with_extraversion_and_security() {
        let mut attention = AttentionState::default();
        attention.recompute_biases(
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.9), // extraversion
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.9), // attachment security
            Fixed::ZERO,
            Fixed::from_f64(0.5),
        );
        assert!(attention.social_bias > Fixed::from_f64(0.5));
        assert!(attention.social_bias <= Fixed::from_f64(0.7));
    }

    #[test]
    fn novelty_bias_rises_with_openness() {
        let mut attention = AttentionState::default();
        attention.recompute_biases(
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.9), // openness
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::from_f64(0.5),
        );
        assert!(attention.novelty_bias > Fixed::from_f64(0.5));
        assert!(attention.novelty_bias <= Fixed::from_f64(0.7));
    }

    #[test]
    fn novelty_bias_rises_with_prediction_error() {
        let mut attention = AttentionState::default();
        attention.recompute_biases(
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5), // prediction_error
            Fixed::from_f64(0.5),
        );
        assert!(attention.novelty_bias > Fixed::from_f64(0.5));
        assert!(attention.novelty_bias <= Fixed::from_f64(0.7));
        // Zero prediction error leaves the openness-anchored baseline.
        attention.recompute_biases(
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::from_f64(0.5),
        );
        assert_eq!(attention.novelty_bias, Fixed::from_f64(0.5));
    }

    #[test]
    fn salience_map_keeps_top_percepts() {
        let mut attention = AttentionState::default();
        let scores = [0.1, 0.9, 0.4, 0.7, 0.2, 0.8, 0.3, 0.6, 0.5, 0.95];
        for (i, s) in scores.iter().enumerate() {
            attention.record_salience(
                PerceptKind::Social,
                Some(AgentId::new(i as u64)),
                Fixed::from_f64(*s),
                i as u64,
            );
        }
        // Bounded to the top SALIENCE_MAP_CAPACITY winners, sorted descending.
        assert_eq!(attention.salience_map.len(), SALIENCE_MAP_CAPACITY);
        assert_eq!(attention.salience_map[0].salience, Fixed::from_f64(0.95));
        assert!(attention
            .salience_map
            .iter()
            .all(|s| s.salience >= Fixed::from_f64(0.3)));
        let mut prev = Fixed::ONE;
        for item in &attention.salience_map {
            assert!(item.salience <= prev);
            prev = item.salience;
        }
    }

    #[test]
    fn percept_kind_classifies_events() {
        let threat = SimEvent::InteractionOccurred {
            from: AgentId::new(0),
            to: AgentId::new(1),
            kind: InteractionKind::Threaten,
            tick: Tick::new(1),
        };
        assert_eq!(PerceptKind::of(&threat), PerceptKind::Threat);
        let talk = SimEvent::InteractionOccurred {
            from: AgentId::new(0),
            to: AgentId::new(1),
            kind: InteractionKind::Talk,
            tick: Tick::new(1),
        };
        assert_eq!(PerceptKind::of(&talk), PerceptKind::Social);
    }

    // ── §8.1.2 (Iteration 97): perceptual-bias feedback ────────────

    fn threat_event() -> SimEvent {
        SimEvent::InteractionOccurred {
            from: AgentId::new(1),
            to: AgentId::new(0),
            kind: InteractionKind::Threaten,
            tick: Tick::new(1),
        }
    }

    fn talk_event() -> SimEvent {
        SimEvent::InteractionOccurred {
            from: AgentId::new(1),
            to: AgentId::new(0),
            kind: InteractionKind::Talk,
            tick: Tick::new(1),
        }
    }

    fn ate_event() -> SimEvent {
        SimEvent::AgentAte {
            agent: AgentId::new(2),
            food: mindstrata_core::id::EntityId::new(0),
            tick: Tick::new(1),
        }
    }

    #[test]
    fn bias_fold_is_exactly_one_at_neutral_anchor() {
        let attention = AttentionState::default();
        assert_eq!(attention.percept_bias_factor(&threat_event()), Fixed::ONE);
        assert_eq!(attention.percept_bias_factor(&talk_event()), Fixed::ONE);
        // Resource/Emotional percepts fold at 1.0 regardless of any bias.
        assert_eq!(attention.percept_bias_factor(&ate_event()), Fixed::ONE);
    }

    #[test]
    fn threat_bias_amplifies_threat_salience() {
        // Fresh, identical states — only the bias differs — so habituation
        // starts identical and the comparison isolates the fold.
        let mut neutral = AttentionState::default();
        let mut vigilant = AttentionState {
            threat_bias: Fixed::from_f64(0.7),
            ..Default::default()
        };
        let needs = NeedState::default();
        let affect = Affect::default();

        let s_neutral = neutral.compute_salience(&threat_event(), AgentId::new(0), &needs, &affect);
        let s_vigilant =
            vigilant.compute_salience(&threat_event(), AgentId::new(0), &needs, &affect);
        assert!(
            s_vigilant > s_neutral,
            "Threat-biased agent should notice threats more: {s_vigilant} vs {s_neutral}"
        );
    }

    #[test]
    fn social_bias_amplifies_social_salience() {
        let mut neutral = AttentionState::default();
        let mut extravert = AttentionState {
            social_bias: Fixed::from_f64(0.7),
            ..Default::default()
        };
        let needs = NeedState::default();
        let affect = Affect::default();

        let s_neutral = neutral.compute_salience(&talk_event(), AgentId::new(0), &needs, &affect);
        let s_extravert =
            extravert.compute_salience(&talk_event(), AgentId::new(0), &needs, &affect);
        assert!(
            s_extravert > s_neutral,
            "Socially biased agent should notice interactions more: {s_extravert} vs {s_neutral}"
        );
    }

    #[test]
    fn resource_events_ignore_perceptual_biases() {
        // Resource percepts have no matching bias dimension — the fold stays
        // 1.0 even at extreme biases, so eating salience is bias-invariant.
        let mut low = AttentionState {
            threat_bias: Fixed::from_f64(0.3),
            ..Default::default()
        };
        let mut high = AttentionState {
            threat_bias: Fixed::from_f64(0.7),
            ..Default::default()
        };
        let needs = NeedState::default();
        let affect = Affect::default();

        let s_low = low.compute_salience(&ate_event(), AgentId::new(0), &needs, &affect);
        let s_high = high.compute_salience(&ate_event(), AgentId::new(0), &needs, &affect);
        assert_eq!(
            s_low, s_high,
            "Resource percepts must ignore the perceptual biases"
        );
    }
}
