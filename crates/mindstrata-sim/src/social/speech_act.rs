//! Language and Communication (§8.1.11) — "Language should be action."
//!
//! Every social interaction in the sim is an utterance. This module supplies
//! the full AP2 speech-act vocabulary (inform, promise, threaten, gossip,
//! ...) plus a pure, deterministic interpreter that frames an existing
//! [`InteractionKind`] as a structured [`SpeechAct`] and records it on the
//! speaker's bounded `speech_log`.
//!
//! Zero-drift design (Iteration 40): the log is write-only observational
//! state — nothing downstream reads it and it is not in `agent_summaries()`.
//! [`SpeechAct::resolve_effect`] models what an act *would* do to trust,
//! affection, status, obligation, and reputation; it is computed and unit-
//! tested but not yet applied. The documented consumer is a future
//! speech → belief-update / reputation iteration. Because the interpreter is
//! a pure function of the interaction kind plus already-existing agent state
//! (no RNG), recording cannot perturb any calibrated run, snapshot, or golden
//! baseline.
//!
//! Vocabulary note: of the twenty kinds, eight are grounded in the live
//! interaction system (each `InteractionKind` maps to its canonical act); the
//! remaining twelve (command, apologize, curse, vow, excommunicate, ...) are
//! declared for the future speech-directive layer and exercised by unit tests
//! so the effect model covers the full roster.

use mindstrata_core::event::InteractionKind;
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// Maximum speech acts retained per agent. Bounded so long runs cannot grow
/// unbounded memory; the log is observational, so dropping old entries loses
/// nothing downstream.
pub const SPEECH_LOG_CAPACITY: usize = 64;

/// §8.1.11: The full speech-act vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpeechActKind {
    Inform,
    Request,
    Command,
    Promise,
    Threaten,
    Apologize,
    Praise,
    Insult,
    Gossip,
    Confess,
    Bless,
    Curse,
    Persuade,
    Accuse,
    Deny,
    Reassure,
    Flirt,
    Propose,
    Vow,
    Excommunicate,
}

/// §8.1.11: The relational purpose of the act — what the speaker is trying to
/// do to the relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalIntent {
    /// Build or deepen a bond.
    Affiliate,
    /// Assert control or superiority.
    Dominate,
    /// Mend a damaged bond.
    Repair,
    /// Push the listener away.
    Distance,
    /// Ask for something.
    Solicit,
    /// Make a binding commitment.
    Commit,
}

/// Broad functional domain — a public classification axis over the vocabulary
/// (the plan groups acts by domain). Currently consumed by tests; `social_cost`
/// keeps its own explicit per-kind table because domain-level costs need
/// per-kind overrides (e.g., Deny at 0.15 sits in Informational).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActDomain {
    Informational,
    Relational,
    Coercive,
    Expressive,
    Sacred,
}

/// §8.1.11: A single structured utterance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechAct {
    pub speaker: AgentId,
    pub listener: AgentId,
    /// Third-party overhearers — empty for the current dyadic interactions.
    pub audience: Vec<AgentId>,
    pub act: SpeechActKind,
    /// Proposition reference (proposition id) — `None` for purely social acts.
    pub content: Option<u32>,
    /// Emotional coloring of the delivery, 0..1 (0.5 neutral).
    pub emotional_tone: Fixed,
    /// How believable the listener finds the speaker, 0..1 (from relationship trust).
    pub credibility: Fixed,
    /// Social price of performing the act, 0..1 (insults cost standing).
    pub social_cost: Fixed,
    pub relational_intent: RelationalIntent,
    pub tick: u64,
}

impl SpeechAct {
    /// Interpret an existing interaction as a speech act.
    ///
    /// Pure and deterministic: the kind → act mapping is fixed, and `tone` /
    /// `credibility` are supplied from already-existing agent state (affect,
    /// relationship trust) at the call site — no RNG, so recording cannot
    /// perturb calibrated trajectories.
    #[must_use]
    pub fn from_interaction(
        kind: InteractionKind,
        speaker: AgentId,
        listener: AgentId,
        tone: Fixed,
        credibility: Fixed,
        tick: u64,
    ) -> Self {
        let act = match kind {
            InteractionKind::Talk => SpeechActKind::Inform,
            InteractionKind::Help => SpeechActKind::Promise,
            InteractionKind::Threaten => SpeechActKind::Threaten,
            InteractionKind::Trade => SpeechActKind::Request,
            InteractionKind::Gossip => SpeechActKind::Gossip,
            InteractionKind::Comfort => SpeechActKind::Reassure,
            InteractionKind::Insult => SpeechActKind::Insult,
            InteractionKind::Teach => SpeechActKind::Persuade,
        };
        Self {
            speaker,
            listener,
            audience: Vec::new(),
            act,
            content: None,
            emotional_tone: tone.clamp_01(),
            credibility: credibility.clamp_01(),
            social_cost: social_cost(act),
            relational_intent: relational_intent(act),
            tick,
        }
    }

    /// The broad functional domain of the act.
    #[must_use]
    pub fn domain(&self) -> ActDomain {
        domain(self.act)
    }

    /// §8.1.11: The effect this act encodes — what it *would* do to the
    /// listener relationship if applied. Computed and unit-tested; the future
    /// speech → belief-update / reputation iteration applies it.
    #[must_use]
    pub fn resolve_effect(&self) -> SpeechEffect {
        let (base_trust, base_affection) = base_delta(self.act);
        // Credible delivery lands harder: scale 0.5 (untrusted) .. 1.0 (fully
        // trusted). A neutral tone (0.5) is the calibrated default.
        let credibility_scale = Fixed::from_f64(0.5) + self.credibility * Fixed::from_f64(0.5);
        SpeechEffect {
            trust_delta: (base_trust * credibility_scale)
                .clamp(Fixed::from_f64(-0.2), Fixed::from_f64(0.2)),
            affection_delta: (base_affection * credibility_scale)
                .clamp(Fixed::from_f64(-0.2), Fixed::from_f64(0.2)),
            status_delta: status_effect(self.act),
            obligation_delta: obligation_effect(self.act),
            reputation_delta: reputation_effect(self.act),
        }
    }
}

/// §8.1.11: The effect vector one act encodes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpeechEffect {
    /// Change to the listener's trust in the speaker.
    pub trust_delta: Fixed,
    /// Change to the listener's affection for the speaker.
    pub affection_delta: Fixed,
    /// Change to the listener's status (praise raises it, insult lowers it).
    pub status_delta: Fixed,
    /// Change to obligation (vows and promises create debt; denials cancel it).
    pub obligation_delta: Fixed,
    /// Change to the speaker's reputation (praise builds standing, aggression costs it).
    pub reputation_delta: Fixed,
}

/// Base trust/affection deltas.
///
/// For the eight kinds grounded in the live interaction system, these are the
/// EXACT magnitudes `process_interaction` applies (Talk 0.01/0.005, Help
/// 0.05/0.03, Threaten −0.1/−0.05, Gossip 0.005/0.01, Comfort 0.03/0.05,
/// Insult −0.08/−0.1, Teach 0.04/0.02), so `resolve_effect` is a faithful
/// forward predictor of what the sim actually does — verified by
/// `model_sign_matches_applied_deltas`. The unproduced kinds carry semantic
/// magnitudes of the same order.
fn base_delta(act: SpeechActKind) -> (Fixed, Fixed) {
    match act {
        // Grounded kinds — must equal the live interaction deltas.
        SpeechActKind::Inform => (Fixed::from_f64(0.01), Fixed::from_f64(0.005)),
        SpeechActKind::Request => (Fixed::from_f64(0.02), Fixed::ZERO),
        SpeechActKind::Command => (Fixed::from_f64(-0.005), Fixed::from_f64(-0.005)),
        SpeechActKind::Promise => (Fixed::from_f64(0.05), Fixed::from_f64(0.03)),
        SpeechActKind::Threaten => (Fixed::from_f64(-0.1), Fixed::from_f64(-0.05)),
        SpeechActKind::Gossip => (Fixed::from_f64(0.005), Fixed::from_f64(0.01)),
        SpeechActKind::Reassure => (Fixed::from_f64(0.03), Fixed::from_f64(0.05)),
        SpeechActKind::Insult => (Fixed::from_f64(-0.08), Fixed::from_f64(-0.1)),
        SpeechActKind::Persuade => (Fixed::from_f64(0.04), Fixed::from_f64(0.02)),
        // Unproduced kinds — semantic magnitudes (future speech-directive layer).
        SpeechActKind::Apologize => (Fixed::from_f64(0.03), Fixed::from_f64(0.02)),
        SpeechActKind::Praise => (Fixed::from_f64(0.05), Fixed::from_f64(0.04)),
        SpeechActKind::Confess => (Fixed::from_f64(0.01), Fixed::from_f64(0.02)),
        SpeechActKind::Bless => (Fixed::from_f64(0.02), Fixed::from_f64(0.03)),
        SpeechActKind::Curse => (Fixed::from_f64(-0.05), Fixed::from_f64(-0.04)),
        SpeechActKind::Accuse => (Fixed::from_f64(-0.06), Fixed::from_f64(-0.08)),
        SpeechActKind::Deny => (Fixed::from_f64(-0.02), Fixed::from_f64(-0.01)),
        SpeechActKind::Flirt => (Fixed::from_f64(0.005), Fixed::from_f64(0.04)),
        SpeechActKind::Propose => (Fixed::from_f64(0.02), Fixed::from_f64(0.06)),
        SpeechActKind::Vow => (Fixed::from_f64(0.03), Fixed::from_f64(0.03)),
        SpeechActKind::Excommunicate => (Fixed::from_f64(-0.15), Fixed::from_f64(-0.12)),
    }
}

/// §8.1.11: The social price of performing the act — coercive and abusive acts
/// cost the speaker standing in the community.
fn social_cost(act: SpeechActKind) -> Fixed {
    match act {
        SpeechActKind::Threaten
        | SpeechActKind::Excommunicate
        | SpeechActKind::Curse
        | SpeechActKind::Accuse => Fixed::from_f64(0.3),
        SpeechActKind::Insult | SpeechActKind::Command | SpeechActKind::Deny => {
            Fixed::from_f64(0.15)
        }
        SpeechActKind::Confess => Fixed::from_f64(0.2),
        SpeechActKind::Gossip => Fixed::from_f64(0.1),
        SpeechActKind::Promise | SpeechActKind::Propose | SpeechActKind::Vow => {
            Fixed::from_f64(0.1)
        }
        _ => Fixed::from_f64(0.05),
    }
}

/// §8.1.11: What the speaker is trying to do to the relationship.
fn relational_intent(act: SpeechActKind) -> RelationalIntent {
    match act {
        SpeechActKind::Inform | SpeechActKind::Persuade => RelationalIntent::Affiliate,
        SpeechActKind::Promise | SpeechActKind::Vow => RelationalIntent::Commit,
        SpeechActKind::Threaten
        | SpeechActKind::Command
        | SpeechActKind::Curse
        | SpeechActKind::Excommunicate => RelationalIntent::Dominate,
        SpeechActKind::Request | SpeechActKind::Flirt | SpeechActKind::Propose => {
            RelationalIntent::Solicit
        }
        SpeechActKind::Apologize | SpeechActKind::Reassure | SpeechActKind::Confess => {
            RelationalIntent::Repair
        }
        SpeechActKind::Insult | SpeechActKind::Accuse | SpeechActKind::Deny => {
            RelationalIntent::Distance
        }
        SpeechActKind::Praise | SpeechActKind::Bless | SpeechActKind::Gossip => {
            RelationalIntent::Affiliate
        }
    }
}

fn domain(act: SpeechActKind) -> ActDomain {
    match act {
        SpeechActKind::Inform | SpeechActKind::Persuade | SpeechActKind::Deny => {
            ActDomain::Informational
        }
        SpeechActKind::Promise
        | SpeechActKind::Apologize
        | SpeechActKind::Reassure
        | SpeechActKind::Flirt
        | SpeechActKind::Propose
        | SpeechActKind::Vow
        | SpeechActKind::Praise
        | SpeechActKind::Request => ActDomain::Relational,
        SpeechActKind::Threaten
        | SpeechActKind::Command
        | SpeechActKind::Insult
        | SpeechActKind::Accuse
        | SpeechActKind::Excommunicate => ActDomain::Coercive,
        SpeechActKind::Bless | SpeechActKind::Curse | SpeechActKind::Confess => ActDomain::Sacred,
        SpeechActKind::Gossip => ActDomain::Expressive,
    }
}

/// Effect on the *listener's* status: praise and blessing raise it, coercive
/// abuse lowers it.
fn status_effect(act: SpeechActKind) -> Fixed {
    match act {
        SpeechActKind::Praise
        | SpeechActKind::Bless
        | SpeechActKind::Reassure
        | SpeechActKind::Flirt
        | SpeechActKind::Propose => Fixed::from_f64(0.02),
        SpeechActKind::Insult
        | SpeechActKind::Accuse
        | SpeechActKind::Curse
        | SpeechActKind::Excommunicate
        | SpeechActKind::Threaten => Fixed::from_f64(-0.02),
        _ => Fixed::ZERO,
    }
}

/// Effect on obligation: commitments create debt, denials cancel it.
fn obligation_effect(act: SpeechActKind) -> Fixed {
    match act {
        SpeechActKind::Promise | SpeechActKind::Vow | SpeechActKind::Apologize => {
            Fixed::from_f64(0.03)
        }
        SpeechActKind::Request | SpeechActKind::Command => Fixed::from_f64(0.01),
        SpeechActKind::Deny => Fixed::from_f64(-0.02),
        _ => Fixed::ZERO,
    }
}

/// Effect on the *speaker's* reputation: informing and praising build
/// standing; aggression and gossip cost it.
fn reputation_effect(act: SpeechActKind) -> Fixed {
    match act {
        SpeechActKind::Inform
        | SpeechActKind::Praise
        | SpeechActKind::Bless
        | SpeechActKind::Persuade => Fixed::from_f64(0.01),
        SpeechActKind::Insult
        | SpeechActKind::Threaten
        | SpeechActKind::Curse
        | SpeechActKind::Accuse
        | SpeechActKind::Excommunicate => Fixed::from_f64(-0.02),
        SpeechActKind::Gossip => Fixed::from_f64(-0.005),
        _ => Fixed::ZERO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn act_of(kind: InteractionKind) -> SpeechAct {
        SpeechAct::from_interaction(
            kind,
            AgentId::new(0),
            AgentId::new(1),
            Fixed::from_f64(0.5), // neutral tone
            Fixed::from_f64(0.5), // default credibility
            42,
        )
    }

    #[test]
    fn talk_maps_to_inform() {
        let act = act_of(InteractionKind::Talk);
        assert_eq!(act.act, SpeechActKind::Inform);
        assert_eq!(act.relational_intent, RelationalIntent::Affiliate);
        assert_eq!(act.domain(), ActDomain::Informational);
    }

    #[test]
    fn help_maps_to_promise() {
        let act = act_of(InteractionKind::Help);
        assert_eq!(act.act, SpeechActKind::Promise);
        assert_eq!(act.relational_intent, RelationalIntent::Commit);
        assert_eq!(act.social_cost, Fixed::from_f64(0.1));
    }

    #[test]
    fn threaten_maps_directly_with_high_cost() {
        let act = act_of(InteractionKind::Threaten);
        assert_eq!(act.act, SpeechActKind::Threaten);
        assert_eq!(act.relational_intent, RelationalIntent::Dominate);
        assert_eq!(act.social_cost, Fixed::from_f64(0.3));
    }

    #[test]
    fn insult_maps_to_distance() {
        let act = act_of(InteractionKind::Insult);
        assert_eq!(act.act, SpeechActKind::Insult);
        assert_eq!(act.relational_intent, RelationalIntent::Distance);
        assert_eq!(act.domain(), ActDomain::Coercive);
    }

    #[test]
    fn comfort_maps_to_reassure() {
        let act = act_of(InteractionKind::Comfort);
        assert_eq!(act.act, SpeechActKind::Reassure);
        assert_eq!(act.relational_intent, RelationalIntent::Repair);
    }

    #[test]
    fn positive_acts_raise_trust_and_affection() {
        let mut act = act_of(InteractionKind::Help);
        for kind in [
            SpeechActKind::Praise,
            SpeechActKind::Promise,
            SpeechActKind::Reassure,
            SpeechActKind::Bless,
            SpeechActKind::Vow,
        ] {
            act.act = kind;
            let effect = act.resolve_effect();
            assert!(
                effect.trust_delta > Fixed::ZERO,
                "{kind:?} must raise trust"
            );
            assert!(
                effect.affection_delta > Fixed::ZERO,
                "{kind:?} must raise affection"
            );
        }
    }

    #[test]
    fn negative_acts_lower_trust_and_affection() {
        for kind in [
            SpeechActKind::Threaten,
            SpeechActKind::Insult,
            SpeechActKind::Curse,
            SpeechActKind::Accuse,
            SpeechActKind::Excommunicate,
        ] {
            let mut act = act_of(InteractionKind::Threaten);
            act.act = kind;
            let effect = act.resolve_effect();
            assert!(
                effect.trust_delta < Fixed::ZERO,
                "{kind:?} must lower trust"
            );
            assert!(
                effect.affection_delta < Fixed::ZERO,
                "{kind:?} must lower affection"
            );
        }
    }

    #[test]
    fn credible_delivery_lands_harder() {
        let weak = SpeechAct::from_interaction(
            InteractionKind::Help,
            AgentId::new(0),
            AgentId::new(1),
            Fixed::from_f64(0.5),
            Fixed::ZERO, // untrusted
            1,
        );
        let strong = SpeechAct::from_interaction(
            InteractionKind::Help,
            AgentId::new(0),
            AgentId::new(1),
            Fixed::from_f64(0.5),
            Fixed::ONE, // fully trusted
            1,
        );
        assert!(strong.resolve_effect().trust_delta > weak.resolve_effect().trust_delta);
    }

    #[test]
    fn vows_create_obligation_and_denials_cancel_it() {
        let mut vow = act_of(InteractionKind::Help);
        vow.act = SpeechActKind::Vow;
        assert!(vow.resolve_effect().obligation_delta > Fixed::ZERO);
        let mut deny = act_of(InteractionKind::Talk);
        deny.act = SpeechActKind::Deny;
        assert!(deny.resolve_effect().obligation_delta < Fixed::ZERO);
    }

    #[test]
    fn insult_costs_reputation_and_praise_builds_it() {
        let mut insult = act_of(InteractionKind::Insult);
        insult.act = SpeechActKind::Insult;
        assert!(insult.resolve_effect().reputation_delta < Fixed::ZERO);
        let mut praise = act_of(InteractionKind::Help);
        praise.act = SpeechActKind::Praise;
        assert!(praise.resolve_effect().reputation_delta > Fixed::ZERO);
    }

    /// §8.1.11 regression net: the model must be a faithful forward predictor.
    /// For every live interaction kind, the sign of `resolve_effect`'s trust
    /// delta must match the sign of the delta `process_interaction` actually
    /// applies to the relationship.
    #[test]
    fn model_sign_matches_applied_deltas() {
        use crate::person::Relationship;
        use crate::social::interaction::{process_interaction, Interaction};
        use mindstrata_core::clock::Tick;
        use mindstrata_core::event::SimEvent;

        for kind in [
            InteractionKind::Talk,
            InteractionKind::Help,
            InteractionKind::Threaten,
            InteractionKind::Trade,
            InteractionKind::Gossip,
            InteractionKind::Comfort,
            InteractionKind::Insult,
            InteractionKind::Teach,
        ] {
            let mut rel = Relationship {
                from: AgentId::new(0),
                to: AgentId::new(1),
                trust: Fixed::from_f64(0.5),
                affection: Fixed::from_f64(0.3),
                respect: Fixed::ZERO,
                fear: Fixed::ZERO,
                obligation: Fixed::ZERO,
                last_interaction_tick: 0,
                kind: crate::person::RelationshipKind::Stranger,
                interaction_count: 0,
                last_positive_tick: 0,
                last_negative_tick: 0,
            };
            let mut events = Vec::new();
            process_interaction(
                &Interaction {
                    from: AgentId::new(0),
                    to: AgentId::new(1),
                    kind,
                },
                std::slice::from_mut(&mut rel),
                &mut events,
                Tick::new(1),
                false,      // out-group: negative deltas get penalized, sign preserved
                Fixed::ONE, // bonding_rate
                Fixed::ONE, // conflict_escalation_rate
                &crate::parameters::SimParameters::default(),
            );
            let applied = events
                .iter()
                .find_map(|ev| match ev {
                    SimEvent::RelationshipChanged { trust_delta, .. } => Some(*trust_delta),
                    _ => None,
                })
                .expect("process_interaction must emit RelationshipChanged");
            let model = act_of(kind).resolve_effect().trust_delta;
            assert_eq!(
                model > Fixed::ZERO,
                applied > Fixed::ZERO,
                "{kind:?}: model {model} vs applied {applied} must agree in sign"
            );
        }
    }

    #[test]
    fn effect_deltas_stay_bounded() {
        for kind in [
            SpeechActKind::Inform,
            SpeechActKind::Request,
            SpeechActKind::Command,
            SpeechActKind::Promise,
            SpeechActKind::Threaten,
            SpeechActKind::Apologize,
            SpeechActKind::Praise,
            SpeechActKind::Insult,
            SpeechActKind::Gossip,
            SpeechActKind::Confess,
            SpeechActKind::Bless,
            SpeechActKind::Curse,
            SpeechActKind::Persuade,
            SpeechActKind::Accuse,
            SpeechActKind::Deny,
            SpeechActKind::Reassure,
            SpeechActKind::Flirt,
            SpeechActKind::Propose,
            SpeechActKind::Vow,
            SpeechActKind::Excommunicate,
        ] {
            let mut act = act_of(InteractionKind::Talk);
            act.act = kind;
            act.credibility = Fixed::ONE;
            let effect = act.resolve_effect();
            assert!(effect.trust_delta.abs() <= Fixed::from_f64(0.2));
            assert!(effect.affection_delta.abs() <= Fixed::from_f64(0.2));
        }
    }
}
