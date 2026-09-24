#![allow(missing_docs)]

//! Gossip propagation system — §11.2, §13.5 of the architecture spec.
//!
//! Gossip should mutate information:
//!
//! ```text
//! rumor_accuracy =
//!     source_memory_accuracy
//!   * source_trust
//!   * transmission_fidelity
//!   * emotional_salience_bias
//!   * identity_bias
//! ```
//!
//! This can produce:
//! - moral panics
//! - false accusations
//! - prestige
//! - scapegoating
//! - factional myths

use mindstrata_core::fixed::Fixed;
use mindstrata_person::person::{Belief, DiscreteEmotions, Personality};
use serde::{Deserialize, Serialize};

/// A rumor that has been shared through the gossip network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rumor {
    /// The proposition this rumor is about.
    pub proposition_id: u64,
    /// Current confidence as believed by the spreader.
    pub confidence: Fixed,
    /// How many times this rumor has been relayed (each hop mutates it).
    pub hops: u32,
    /// Original tick when the rumor was created.
    pub origin_tick: u64,
    /// Tick when this rumor was last heard.
    pub last_heard_tick: u64,
    /// Emotional charge accumulated from spreaders' emotions.
    pub emotional_charge: Fixed,
    /// Identity linkage of the original belief.
    pub identity_linkage: Fixed,
    /// Resistance of the original belief (how hard it is to update).
    pub original_resistance: Fixed,
}

impl Rumor {
    /// Create a new rumor from a belief.
    pub fn from_belief(belief: &Belief, tick: u64) -> Self {
        Self {
            proposition_id: belief.proposition_id,
            confidence: belief.confidence,
            hops: 0,
            origin_tick: tick,
            last_heard_tick: tick,
            emotional_charge: belief.emotional_charge,
            identity_linkage: belief.identity_linkage,
            original_resistance: belief.resistance,
        }
    }

    /// Compute the rumor's age in ticks.
    pub fn age(&self, current_tick: u64) -> u64 {
        current_tick.saturating_sub(self.origin_tick)
    }

    /// Compute how salient this rumor is to an agent.
    /// High emotional charge and low hop count = more salient.
    pub fn salience(&self, _current_tick: u64) -> Fixed {
        // Telephone game: each hop reduces accuracy significantly
        let age_penalty = Fixed::from_f64(0.85).powi(self.hops);
        let emotional_boost = self.emotional_charge * Fixed::from_f64(0.3);
        let novelty = if self.hops == 0 {
            Fixed::from_f64(0.2)
        } else {
            Fixed::ZERO
        };
        (age_penalty + emotional_boost + novelty).clamp_01()
    }
}

/// Gossip propagation result — what changed when a rumor was shared.
#[derive(Debug, Clone)]
pub struct GossipResult {
    /// The proposition_id of the rumor shared.
    pub proposition_id: u64,
    /// The mutated confidence after transmission.
    pub mutated_confidence: Fixed,
    /// Emotional charge after distortion.
    pub emotional_charge: Fixed,
    /// How much the listener's belief changed (delta).
    pub belief_delta: Fixed,
    /// Whether the rumor was accepted (passed salience threshold).
    pub accepted: bool,
    /// Resistance of the source belief the rumor was born from (how hard
    /// the ORIGINAL belief was to update) — inherited by a newly-created
    /// listener belief so a rumor from a deeply-held source seeds a more
    /// entrenched belief than one from a weakly-held source.
    pub original_resistance: Fixed,
}

/// Compute the mutated confidence of a rumor during transmission.
///
/// Implements §11.2: "rumor_accuracy = source_memory_accuracy * source_trust
/// * transmission_fidelity * emotional_salience_bias * identity_bias"
pub fn mutate_rumor(
    rumor: &Rumor,
    source_trust: Fixed,
    spreader_emotions: &DiscreteEmotions,
    spreader_personality: &Personality,
    listener_personality: &Personality,
    gossip_base_fidelity: Fixed,
    gossip_emotional_distortion: Fixed,
) -> (Fixed, Fixed) {
    // §11.2 source factors — the relay previously dropped BOTH of them:
    // `source_trust` was received but unused (`_source_trust`), and
    // `original_resistance` (captured at `from_belief`) was never read.
    // A rumor born from a firmly-held belief and spread through a trusted
    // relationship survives the relay with higher fidelity: the source's
    // conviction anchors the message against telephone-game drift, while
    // a weakly-held belief from a low-trust source degrades fast.
    //
    // The anchor applies at hops=0 too (the sim's only condition — rumors
    // are re-created fresh from the spreader's belief each interaction), so
    // a weakly-held belief no longer spreads at full strength: conviction
    // and trust act as fidelity discounts (each ~0.9 at neutral, ~0.98 at
    // strong), keeping the relay near the original strength while still
    // making the source factors genuinely live.
    let conviction_discount =
        Fixed::from_f64(0.9) + rumor.original_resistance * Fixed::from_f64(0.08);
    let trust_discount = Fixed::from_f64(0.9) + source_trust * Fixed::from_f64(0.08);
    let source_fidelity = (conviction_discount * trust_discount).clamp_01();

    // Source memory accuracy: degrades with hops (telephone game)
    let memory_accuracy = Fixed::from_f64(0.95).powi(rumor.hops) * source_fidelity;

    // Transmission fidelity: extraversion and agreeableness improve fidelity
    let transmission_fidelity = gossip_base_fidelity
        + spreader_personality.extraversion * Fixed::from_f64(0.15)
        + spreader_personality.agreeableness * Fixed::from_f64(0.1);

    // Emotional salience bias: angry or fearful spreaders exaggerate
    let emotional_distortion = spreader_emotions.anger * gossip_emotional_distortion
        + spreader_emotions.fear * Fixed::from_f64(0.1)
        - spreader_emotions.joy * Fixed::from_f64(0.05);

    // Identity bias: listeners accept rumors that align with their identity
    let identity_affinity = if listener_personality.traditionalism > Fixed::from_f64(0.6) {
        Fixed::from_f64(0.05) // traditionalists accept authority-aligned rumors
    } else if listener_personality.openness > Fixed::from_f64(0.6) {
        Fixed::from_f64(0.03) // open people accept novel rumors
    } else {
        Fixed::ZERO
    };

    // Combine all factors
    let mutated = rumor.confidence * memory_accuracy * transmission_fidelity
        + emotional_distortion * rumor.confidence
        + identity_affinity * rumor.confidence;

    // Emotional charge accumulates from spreaders
    let new_charge = (rumor.emotional_charge
        + spreader_emotions.anger * Fixed::from_f64(0.1)
        + spreader_emotions.fear * Fixed::from_f64(0.1))
    .clamp_01();

    (mutated.clamp_01(), new_charge)
}

/// Process a gossip event: the spreader shares a rumor with the listener.
///
/// Returns the mutated rumor and whether it was accepted.
pub fn process_gossip(
    rumor: &Rumor,
    source_trust: Fixed,
    spreader_emotions: &DiscreteEmotions,
    spreader_personality: &Personality,
    listener_personality: &Personality,
    listener_beliefs: &[Belief],
    current_tick: u64,
    gossip_base_fidelity: Fixed,
    gossip_emotional_distortion: Fixed,
    gossip_acceptance_threshold: Fixed,
) -> GossipResult {
    let (mutated_confidence, emotional_charge) = mutate_rumor(
        rumor,
        source_trust,
        spreader_emotions,
        spreader_personality,
        listener_personality,
        gossip_base_fidelity,
        gossip_emotional_distortion,
    );

    // Check if listener already has a belief about this proposition
    let existing = listener_beliefs
        .iter()
        .find(|b| b.proposition_id == rumor.proposition_id);

    let belief_delta = if let Some(existing) = existing {
        // Weight by resistance: high-resistance beliefs barely change
        let weight = Fixed::ONE - existing.resistance;
        let new_conf =
            (existing.confidence * existing.resistance + mutated_confidence * weight).clamp_01();
        new_conf - existing.confidence
    } else {
        // New belief — the full mutated confidence
        mutated_confidence
    };

    // Accept if the rumor has enough salience
    let salience = rumor.salience(current_tick);
    let accepted = salience > gossip_acceptance_threshold;

    GossipResult {
        proposition_id: rumor.proposition_id,
        mutated_confidence,
        emotional_charge,
        belief_delta,
        accepted,
        original_resistance: rumor.original_resistance,
    }
}

/// Apply a gossip result to a listener's beliefs.
pub fn apply_gossip(listener_beliefs: &mut Vec<Belief>, result: &GossipResult, tick: u64) {
    if !result.accepted {
        return;
    }

    if let Some(existing) = listener_beliefs
        .iter_mut()
        .find(|b| b.proposition_id == result.proposition_id)
    {
        // Update existing belief
        let weight = Fixed::ONE - existing.resistance;
        existing.confidence = (existing.confidence * existing.resistance
            + result.mutated_confidence * weight)
            .clamp_01();
        existing.emotional_charge = result.emotional_charge;
        existing.last_reinforced_tick = tick;
    } else {
        // Create new belief from rumor
        // §19.5.A: Accuracy depends on how distorted the mutation was.
        // Compare mutated confidence against 0.5 (neutral) as proxy for wild distortion.
        let distortion = (result.mutated_confidence - Fixed::from_f64(0.5)).abs();
        let is_accurate = distortion < Fixed::from_f64(0.3);
        // Iteration 202 (§11.2 source factors): the new belief's resistance
        // inherits the SOURCE's conviction — a rumor born from a deeply-held
        // belief (high original_resistance) seeds a more entrenched belief
        // than one from a weakly-held source. The floor stays 0.3 (rumors
        // are still initially easier to update than direct experience); the
        // source's conviction adds up to +0.4 on top.
        let inherited_resistance =
            (Fixed::from_f64(0.3) + result.original_resistance * Fixed::from_f64(0.4)).clamp_01();
        listener_beliefs.push(Belief {
            proposition_id: result.proposition_id,
            confidence: result.mutated_confidence,
            emotional_charge: result.emotional_charge,
            identity_linkage: Fixed::from_f64(0.2), // rumors start with low identity linkage
            resistance: inherited_resistance,       // inherits the source's conviction
            last_reinforced_tick: tick,
            source: mindstrata_person::person::EvidenceSource::Hearsay,
            social_reinforcement: 0,
            is_accurate,
        });
    }
}

// ── §7.2: Moral Panic / Rumor Cascade ──────────────────────────────────

/// i381: the panic cadence — a moral panic is a discrete crisis, not a per-tick
/// loop. Once it fires, the belief-charge pool relaxes toward its resting level
/// over roughly this window before another can register. Also the unit the
/// anomaly baseline's memory is expressed in (below), so that the trigger's two
/// timescales are derived from one another rather than independently guessed.
pub const MORAL_PANIC_COOLDOWN: u64 = 300;

/// i381: how many propositions the trigger watches — 0 `the_market_is_fair`,
/// 1 `the_council_protects_us`. The simulation keeps **one anomaly baseline per
/// proposition**, because the two charges are driven by different institutions.
pub const MORAL_PANIC_PROPOSITIONS: u64 = 2;

/// i381: the anomaly baseline's memory, in multiples of the panic cadence. The
/// baseline must be far slower than the crisis it measures, or a normal crisis
/// becomes its own reference and the trigger can never see it as anomalous; it
/// must still be fast enough to track a population whose resting charge drifts
/// over an arc. 10 crisis windows ≈ 3 000 ticks ≈ 20 simulated days.
pub const MORAL_PANIC_BASELINE_COOLDOWNS: u64 = 10;

/// i381: ABSOLUTE FLOOR on the average emotional charge — the arm that governs a
/// population whose own level is *low*, and the level assumed about one whose
/// history is still unmeasured.
///
/// This const used to be the whole trigger at **0.55** (`MORAL_PANIC_CHARGE_
/// THRESHOLD`), and i378 measured that bar sitting inside the body of its own
/// input distribution: firing runs cleared it by only 2.5–9.4% while one swept
/// crisis seed missed by 0.5% (0.9955). The absolute form is why the firing seed
/// family moved on every pacing shift (i343/i351/i376 each renamed it).
///
/// **0.47 is the geometric midpoint of the measured decision gap** (i381 probe,
/// 13 worlds × 20K ticks, sampling the trigger's own inputs every tick):
///
/// | side | measured extreme | value | headroom vs 0.47 |
/// |---|---|---|---|
/// | must NOT fire | highest non-firing plateau (`crisis/42`) | 0.4188 | ×1.12 |
/// | must fire | lowest firing spike (`crisis/11`, the i378 knife-edge seed) | 0.5475 | ×1.165 |
///
/// so the floor is the point of equal relative margin between the two sides. That
/// is the honest ceiling of a *level* bar on this corpus: the gap between "fires"
/// and "must stay dark" is only 1.31× wide, and i378's bar sat at one edge of it
/// instead of between them. (For comparison the old 0.55 gives `crisis/11` a
/// margin of 0.9955 — the knife-edge that started this iteration.)
pub const MORAL_PANIC_CHARGE_FLOOR: Fixed = Fixed::from_raw(4700); // 0.47

/// i381: fold an observed population charge into the anomaly baseline.
///
/// `tau` is the time constant in ticks. Deliberately **f64**: the accumulator is
/// recursive, so a `Fixed` update of `(observed − baseline) × (1/tau)` would
/// quantize at 1e-4 per tick, and that error does not average out — it becomes a
/// standing bias of up to `1e-4 × tau` (0.3 at the shipped tau), which is the
/// size of the signal itself. This is §5's fixed-4 truncation hazard applied to
/// *state* rather than to a one-off increment: compute at full precision, quantize
/// once at the point of use (`anomaly_baseline_fixed`).
///
/// Two structural rules, both load-bearing:
///
/// * **A cold baseline adopts its first observation.** An unmeasured population's
///   first impression *is* its normal level. The alternative (starting at 0) makes
///   every young world look maximally anomalous while its history is still
///   unknown — the i381 probe measured exactly that as spurious calm-world panics
///   (a stably-warm calm world firing 3–5 times in its first few thousand ticks
///   before its own baseline had been learned).
/// * **A zero observation carries no information.** `observed == 0` means nobody
///   holds a charged belief about this proposition — not that the population has
///   calmed down — so folding it would drag the baseline toward zero and re-arm
///   the trigger for no reason.
pub fn anomaly_baseline_fold(baseline: f64, observed: Fixed, tau: f64) -> f64 {
    if observed <= Fixed::ZERO {
        return baseline;
    }
    let obs = observed.to_f64();
    if baseline <= 0.0 {
        return obs;
    }
    baseline + (obs - baseline) * (1.0 / tau)
}

/// i381: the gate's view of the anomaly baseline — quantized exactly once, here.
pub fn anomaly_baseline_fixed(baseline: f64) -> Fixed {
    Fixed::from_f64(baseline.max(0.0))
}

/// i381: the ANOMALY leg — the population's charge must exceed this multiple of
/// its own slow baseline to read as a cascade rather than as its normal level.
///
/// This is the relative form the audit's Class-4 ruling calls for: an absolute
/// threshold on a self-driven aggregate has an operating point that drifts with
/// the distribution, so its pins churn. A *relative* threshold adapts to the
/// population it is testing, so a world whose charge is stably high is NOT
/// anomalous while one whose charge climbs above its own recent level IS.
///
/// Two consequences the floor alone cannot deliver, and the reason this arm ships
/// even though the floor is the operating point everywhere in the calibration
/// corpus (whose warmest *stable* population, `calm/42`, sits at 0.373 — just
/// under this law's crossover of `floor / ratio` = 0.376):
///
/// 1. **A sustained high level stops firing.** Under the old law a population
///    whose charge plateaus above the bar fires every cooldown forever; here the
///    baseline rises to meet it and the cascade goes quiet after its burst — a
///    panic is an *event*, not a new equilibrium. Measured on synthetic shapes by
///    the i381 probe: a population that steps from 0.20 to a sustained 0.65 fires
///    **50** times under the old law and **13** here (a burst, then silence —
///    `tick_moral_panic_and_revolution`'s 300-tick cooldown over the ~4 800 ticks
///    it takes the baseline to catch up). A world that never left 0.65 fires 67
///    times there and **0** here: a chronically charged population has no
///    *sudden* collapse to have, which is the §7.2 semantics the absolute bar
///    never expressed.
/// 2. **The bar follows an arc.** As a settlement's charge drifts over thousands of
///    ticks the bar drifts with it, so the trigger keeps a constant *relative*
///    sensitivity instead of needing a re-anchor at every pacing shift.
pub const MORAL_PANIC_ANOMALY_RATIO: Fixed = Fixed::from_raw(12500); // 1.25

/// Moral panic result — what happens when gossip accumulates enough emotional charge.
#[derive(Debug, Clone)]
pub struct MoralPanicResult {
    /// Whether a moral panic was triggered.
    pub triggered: bool,
    /// The proposition_id of the institution being gossiped about.
    pub proposition_id: u64,
    /// The average emotional charge that triggered the panic.
    pub avg_charge: Fixed,
    /// How much institutional legitimacy should drop.
    pub legitimacy_damage: Fixed,
    /// How much faction grievance should increase.
    pub grievance_boost: Fixed,
}

/// Analyze accumulated beliefs for signs of moral panic.
///
/// When enough agents hold high-emotional-charge beliefs about an institution,
/// it can trigger a sudden collapse in trust — a moral panic.
/// This implements §7.2: "Gossip about institutions spreads through the
/// social graph. Distorted rumors can create moral panics."
pub fn detect_moral_panic(
    beliefs: &[&[Belief]],
    proposition_id: u64,
    charge_baseline: Fixed,
) -> MoralPanicResult {
    // Collect emotional charges from all agents' beliefs about this proposition
    let charges: Vec<Fixed> = beliefs
        .iter()
        .filter_map(|agent_beliefs| {
            agent_beliefs
                .iter()
                .find(|b| b.proposition_id == proposition_id)
                .map(|b| b.emotional_charge)
        })
        .collect();

    if charges.is_empty() {
        return MoralPanicResult {
            triggered: false,
            proposition_id,
            avg_charge: Fixed::ZERO,
            legitimacy_damage: Fixed::ZERO,
            grievance_boost: Fixed::ZERO,
        };
    }

    let avg_charge =
        charges.iter().fold(Fixed::ZERO, |a, b| a + *b) / Fixed::from_int(charges.len() as i64);

    // Also check: how many agents have HIGH emotional charge (> 0.4)?
    let high_charge_count = charges
        .iter()
        .filter(|c| **c > Fixed::from_f64(0.4))
        .count();
    let panic_ratio =
        Fixed::from_int(high_charge_count as i64) / Fixed::from_int(charges.len() as i64);

    // Moral panic requires: an ANOMALOUS average charge (above the population's
    // own slow baseline by `MORAL_PANIC_ANOMALY_RATIO`, and above an absolute
    // floor so a quiet world cannot trip it) AND widespread charge (≥30% of
    // holders emotionally charged — the leg i378 measured as the clean
    // discriminator). A zero baseline (a fresh world, or a caller that has no
    // history) reduces the bar to the absolute floor, which is the pre-i381
    // behaviour with a lower bar rather than a different mechanism.
    let elevation_bar = (charge_baseline * MORAL_PANIC_ANOMALY_RATIO).max(MORAL_PANIC_CHARGE_FLOOR);
    let triggered = avg_charge >= elevation_bar && panic_ratio >= Fixed::from_f64(0.3); // at least 30% of agents are emotionally charged

    let legitimacy_damage = if triggered {
        // Damage scales with how intense the panic is
        (avg_charge * Fixed::from_f64(0.4) + panic_ratio * Fixed::from_f64(0.3)).clamp_01()
    } else {
        Fixed::ZERO
    };

    let grievance_boost = if triggered {
        // Panic increases grievance proportionally
        (avg_charge * Fixed::from_f64(0.2)).clamp_01()
    } else {
        Fixed::ZERO
    };

    MoralPanicResult {
        triggered,
        proposition_id,
        avg_charge,
        legitimacy_damage,
        grievance_boost,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_belief(prop_id: u64, confidence: f64) -> Belief {
        Belief {
            proposition_id: prop_id,
            confidence: Fixed::from_f64(confidence),
            emotional_charge: Fixed::from_f64(0.3),
            identity_linkage: Fixed::from_f64(0.4),
            resistance: Fixed::from_f64(0.5),
            last_reinforced_tick: 0,
            source: mindstrata_person::person::EvidenceSource::PersonalExperience,
            social_reinforcement: 0,
            is_accurate: true,
        }
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
            temperament: mindstrata_person::person::Temperament::default(),
        }
    }

    fn make_emotions() -> DiscreteEmotions {
        DiscreteEmotions {
            fear: Fixed::from_f64(0.2),
            anger: Fixed::from_f64(0.1),
            joy: Fixed::from_f64(0.3),
            sadness: Fixed::ZERO,
            trust: Fixed::from_f64(0.5),
            shame: Fixed::ZERO,
            pride: Fixed::ZERO,
            guilt: Fixed::ZERO,
            disgust: Fixed::ZERO,
            contempt: Fixed::ZERO,
            awe: Fixed::ZERO,
            gratitude: Fixed::ZERO,
            jealousy: Fixed::ZERO,
            envy: Fixed::ZERO,
            loneliness: Fixed::ZERO,
            tenderness: Fixed::ZERO,
            humiliation: Fixed::ZERO,
            relief: Fixed::ZERO,
            hope: Fixed::ZERO,
            despair: Fixed::ZERO,
            nostalgia: Fixed::ZERO,
            moral_outrage: Fixed::ZERO,
        }
    }

    #[test]
    fn rumor_from_belief() {
        let belief = make_belief(0, 0.7);
        let rumor = Rumor::from_belief(&belief, 100);
        assert_eq!(rumor.proposition_id, 0);
        assert_eq!(rumor.confidence, Fixed::from_f64(0.7));
        assert_eq!(rumor.hops, 0);
        assert_eq!(rumor.origin_tick, 100);
    }

    #[test]
    fn rumor_salience_higher_when_new() {
        let rumor = Rumor {
            proposition_id: 0,
            confidence: Fixed::from_f64(0.7),
            hops: 0,
            origin_tick: 100,
            last_heard_tick: 100,
            emotional_charge: Fixed::from_f64(0.5),
            identity_linkage: Fixed::ZERO,
            original_resistance: Fixed::from_f64(0.5),
        };
        let salience_new = rumor.salience(100);

        let mut old_rumor = rumor;
        old_rumor.hops = 5;
        let salience_old = old_rumor.salience(100);

        assert!(
            salience_new > salience_old,
            "New rumors should be more salient than old ones"
        );
    }

    #[test]
    fn mutate_applies_emotional_distortion() {
        let rumor = Rumor {
            proposition_id: 0,
            confidence: Fixed::from_f64(0.7),
            hops: 0,
            origin_tick: 0,
            last_heard_tick: 0,
            emotional_charge: Fixed::from_f64(0.3),
            identity_linkage: Fixed::ZERO,
            original_resistance: Fixed::from_f64(0.5),
        };

        let mut angry_emotions = make_emotions();
        angry_emotions.anger = Fixed::from_f64(0.8);

        let calm_emotions = make_emotions();

        let personality = make_personality();

        let (mutated_angry, _) = mutate_rumor(
            &rumor,
            Fixed::from_f64(0.7),
            &angry_emotions,
            &personality,
            &personality,
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.15),
        );
        let (mutated_calm, _) = mutate_rumor(
            &rumor,
            Fixed::from_f64(0.7),
            &calm_emotions,
            &personality,
            &personality,
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.15),
        );

        // Angry spreaders should distort confidence differently
        assert_ne!(
            mutated_angry, mutated_calm,
            "Emotional state should affect mutation"
        );
    }

    #[test]
    fn gossip_updates_existing_belief() {
        let rumor = Rumor::from_belief(&make_belief(0, 0.8), 100);
        let listener_beliefs = vec![make_belief(0, 0.5)];

        let result = process_gossip(
            &rumor,
            Fixed::from_f64(0.7),
            &make_emotions(),
            &make_personality(),
            &make_personality(),
            &listener_beliefs,
            100,
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.15),
            Fixed::from_f64(0.15),
        );

        assert!(result.accepted, "Gossip should be accepted");
        assert_ne!(result.belief_delta, Fixed::ZERO, "Belief should change");
    }

    #[test]
    fn gossip_creates_new_belief() {
        let rumor = Rumor::from_belief(&make_belief(5, 0.6), 100);
        let listener_beliefs: Vec<Belief> = vec![];

        let result = process_gossip(
            &rumor,
            Fixed::from_f64(0.7),
            &make_emotions(),
            &make_personality(),
            &make_personality(),
            &listener_beliefs,
            100,
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.15),
            Fixed::from_f64(0.15),
        );

        assert!(result.accepted);
        assert!(result.belief_delta > Fixed::ZERO);
    }

    #[test]
    fn apply_gossip_modifies_belief() {
        let mut beliefs = vec![make_belief(0, 0.5)];
        let result = GossipResult {
            proposition_id: 0,
            mutated_confidence: Fixed::from_f64(0.8),
            emotional_charge: Fixed::from_f64(0.4),
            belief_delta: Fixed::from_f64(0.15),
            accepted: true,
            original_resistance: Fixed::from_f64(0.5),
        };

        apply_gossip(&mut beliefs, &result, 200);

        assert!(
            beliefs[0].confidence > Fixed::from_f64(0.5),
            "Confidence should increase"
        );
        assert_eq!(beliefs[0].last_reinforced_tick, 200);
    }

    #[test]
    fn apply_gossip_creates_new_belief() {
        let mut beliefs: Vec<Belief> = vec![];
        let result = GossipResult {
            proposition_id: 5,
            mutated_confidence: Fixed::from_f64(0.6),
            emotional_charge: Fixed::from_f64(0.3),
            belief_delta: Fixed::from_f64(0.6),
            accepted: true,
            original_resistance: Fixed::from_f64(0.8),
        };

        apply_gossip(&mut beliefs, &result, 200);

        assert_eq!(beliefs.len(), 1);
        assert_eq!(beliefs[0].proposition_id, 5);
    }

    #[test]
    fn rejected_gossip_does_not_apply() {
        let mut beliefs = vec![make_belief(0, 0.5)];
        let result = GossipResult {
            proposition_id: 0,
            mutated_confidence: Fixed::from_f64(0.9),
            emotional_charge: Fixed::from_f64(0.5),
            belief_delta: Fixed::from_f64(0.2),
            accepted: false,
            original_resistance: Fixed::from_f64(0.5),
        };

        apply_gossip(&mut beliefs, &result, 200);

        assert_eq!(
            beliefs[0].confidence,
            Fixed::from_f64(0.5),
            "Rejected gossip should not change belief"
        );
    }

    #[test]
    fn high_resistance_belief_resists_gossip() {
        let rumor = Rumor::from_belief(&make_belief(0, 0.9), 100);
        let high_resistance_belief = Belief {
            proposition_id: 0,
            confidence: Fixed::from_f64(0.8),
            emotional_charge: Fixed::from_f64(0.5),
            identity_linkage: Fixed::from_f64(0.8),
            resistance: Fixed::from_f64(0.9), // very resistant
            last_reinforced_tick: 0,
            source: mindstrata_person::person::EvidenceSource::PersonalExperience,
            social_reinforcement: 0,
            is_accurate: true,
        };

        let result = process_gossip(
            &rumor,
            Fixed::from_f64(0.7),
            &make_emotions(),
            &make_personality(),
            &make_personality(),
            &[high_resistance_belief],
            100,
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.15),
            Fixed::from_f64(0.15),
        );

        // High resistance should make the belief barely change
        assert!(
            result.belief_delta.abs() < Fixed::from_f64(0.1),
            "High resistance belief should barely change from gossip, delta={}",
            result.belief_delta.to_f64()
        );
    }

    #[test]
    fn source_conviction_anchors_relay_fidelity() {
        // §11.2 source factors (Iteration 202): a rumor born from a
        // deeply-held belief (high original_resistance) relayed through a
        // trusted relationship survives the telephone game with higher
        // fidelity than a weakly-held one from a low-trust source.
        let strong_rumor = Rumor {
            proposition_id: 0,
            confidence: Fixed::from_f64(0.7),
            hops: 0,
            origin_tick: 0,
            last_heard_tick: 0,
            emotional_charge: Fixed::from_f64(0.3),
            identity_linkage: Fixed::ZERO,
            original_resistance: Fixed::from_f64(0.9),
        };
        let weak_rumor = Rumor {
            original_resistance: Fixed::from_f64(0.1),
            ..strong_rumor
        };
        let personality = make_personality();

        let (strong_fixed, _) = mutate_rumor(
            &strong_rumor,
            Fixed::from_f64(0.9), // high trust
            &make_emotions(),
            &personality,
            &personality,
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.15),
        );
        let (weak_fixed, _) = mutate_rumor(
            &weak_rumor,
            Fixed::from_f64(0.2), // low trust
            &make_emotions(),
            &personality,
            &personality,
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.15),
        );

        assert!(
            strong_fixed > weak_fixed,
            "Deeply-held belief via trusted source should relay stronger: strong={} weak={}",
            strong_fixed.to_f64(),
            weak_fixed.to_f64()
        );
        // The anchor is live at hops=0: a weakly-held rumor must NOT spread
        // at full strength — its relayed confidence drops below the source's.
        assert!(
            weak_fixed < strong_rumor.confidence,
            "Weakly-held rumor should degrade through the relay: relayed={} source={}",
            weak_fixed.to_f64(),
            strong_rumor.confidence.to_f64()
        );
    }

    #[test]
    fn new_belief_inherits_source_resistance() {
        // §11.2 source factors (Iteration 202): a rumor born from a
        // deeply-held belief seeds a MORE entrenched new belief than one
        // from a weakly-held source (floor 0.3 stays — rumors are still
        // easier to update than direct experience).
        let weak_result = GossipResult {
            proposition_id: 5,
            mutated_confidence: Fixed::from_f64(0.6),
            emotional_charge: Fixed::from_f64(0.3),
            belief_delta: Fixed::from_f64(0.6),
            accepted: true,
            original_resistance: Fixed::from_f64(0.1),
        };
        let strong_result = GossipResult {
            original_resistance: Fixed::from_f64(0.9),
            ..weak_result
        };

        let mut weak_beliefs: Vec<Belief> = vec![];
        let mut strong_beliefs: Vec<Belief> = vec![];
        apply_gossip(&mut weak_beliefs, &weak_result, 200);
        apply_gossip(&mut strong_beliefs, &strong_result, 200);

        assert_eq!(weak_beliefs.len(), 1);
        assert_eq!(strong_beliefs.len(), 1);
        // Floor 0.3 + 0.1*0.4 = 0.34 vs 0.3 + 0.9*0.4 = 0.66
        assert!(
            strong_beliefs[0].resistance > weak_beliefs[0].resistance,
            "Deeply-held source should seed a more resistant new belief: strong={} weak={}",
            strong_beliefs[0].resistance.to_f64(),
            weak_beliefs[0].resistance.to_f64()
        );
        assert!(
            weak_beliefs[0].resistance >= Fixed::from_f64(0.3),
            "Rumor-seeded beliefs keep the 0.3 floor"
        );
    }

    #[test]
    fn anomaly_baseline_raises_the_bar_above_a_stable_population() {
        // The same population charge, judged against two different histories:
        // a cold baseline leaves the absolute floor in charge, while a baseline
        // at or above the charge itself makes the level stop being anomalous.
        // This is the whole point of the relative form — i378's absolute bar sat
        // inside the population's own input distribution (2.5–9.4% headroom).
        let mk = |charge: f64| {
            let mut b = make_belief(1, 0.8);
            b.emotional_charge = Fixed::from_f64(charge);
            vec![b]
        };
        let a = mk(0.7);
        let b = mk(0.7);
        let c = mk(0.7);
        let beliefs: Vec<&[Belief]> = vec![&a[..], &b[..], &c[..]];

        // Cold history: the absolute floor governs and a charged population fires.
        assert!(detect_moral_panic(&beliefs, 1, Fixed::ZERO).triggered);
        // Warm history: the population has *lived* at 0.8, so 0.7 is not news.
        let warm = detect_moral_panic(&beliefs, 1, Fixed::from_f64(0.8));
        assert!(
            !warm.triggered,
            "a charge no higher than the population's own baseline must not read as a cascade"
        );
    }

    #[test]
    fn anomaly_baseline_fold_seeds_cold_and_ignores_an_empty_population() {
        // A cold baseline adopts its first observation rather than starting at
        // zero — a young world must not read as maximally anomalous.
        assert_eq!(
            anomaly_baseline_fold(0.0, Fixed::from_f64(0.42), 3000.0),
            0.42
        );
        // Zero holders carry no information: folding must not drag the baseline
        // down, or an empty proposition would re-arm the trigger for nothing.
        assert_eq!(anomaly_baseline_fold(0.42, Fixed::ZERO, 3000.0), 0.42);
        // Once seeded, the fold is slow.
        assert!(anomaly_baseline_fold(0.42, Fixed::from_f64(0.9), 3000.0) < 0.4202);
    }

    #[test]
    fn anomaly_baseline_fold_is_slow_and_never_truncates_to_zero() {
        // Slow: one fold moves a seeded baseline by only a fraction of the gap.
        let one = anomaly_baseline_fold(0.30, Fixed::from_f64(0.6), 3000.0);
        assert!((0.30..0.301).contains(&one), "one tick moved {one}");
        // Convergence: after one time constant the baseline has covered ~63% of
        // the gap (1 − 1/e), which is what "slow moving average" means.
        let mut acc = 0.30f64;
        for _ in 0..3000 {
            acc = anomaly_baseline_fold(acc, Fixed::from_f64(0.6), 3000.0);
        }
        assert!((0.48..0.50).contains(&acc), "after one tau: {acc}");
        // The §5 hazard this design exists to avoid: the SAME update expressed in
        // `Fixed` quantizes to zero for a near-baseline observation, so a recursive
        // accumulator built out of `Fixed` would silently freeze — a standing bias
        // of up to `1e-4 × tau` (0.3 here), the size of the signal itself.
        let delta = 0.301 - 0.30;
        assert_eq!(
            Fixed::from_f64(delta * (1.0 / 3000.0)),
            Fixed::ZERO,
            "if this ever stops truncating, the f64 accumulator is no longer load-bearing"
        );
        assert!(anomaly_baseline_fold(0.30, Fixed::from_f64(0.301), 3000.0) > 0.30);
    }

    #[test]
    fn moral_panic_triggers_with_high_charge() {
        // Create beliefs with high emotional charge across multiple agents
        let b1 = {
            let mut b = make_belief(1, 0.8);
            b.emotional_charge = Fixed::from_f64(0.7); // above the absolute floor
            vec![b]
        };
        let b2 = {
            let mut b = make_belief(1, 0.8);
            b.emotional_charge = Fixed::from_f64(0.7);
            vec![b]
        };
        let b3 = {
            let mut b = make_belief(1, 0.8);
            b.emotional_charge = Fixed::from_f64(0.7);
            vec![b]
        };
        let beliefs: Vec<&[Belief]> = vec![&b1[..], &b2[..], &b3[..]];

        // A cold baseline: the absolute floor is the whole bar (a fresh world).
        let result = detect_moral_panic(&beliefs, 1, Fixed::ZERO);
        assert!(
            result.triggered,
            "Moral panic should trigger with high emotional charge"
        );
        assert!(result.legitimacy_damage > Fixed::ZERO);
        assert!(result.grievance_boost > Fixed::ZERO);
    }

    #[test]
    fn moral_panic_does_not_trigger_with_low_charge() {
        let b1 = {
            let mut b = make_belief(1, 0.5);
            b.emotional_charge = Fixed::from_f64(0.2); // below the absolute floor
            vec![b]
        };
        let beliefs: Vec<&[Belief]> = vec![&b1[..]];

        let result = detect_moral_panic(&beliefs, 1, Fixed::ZERO);
        assert!(
            !result.triggered,
            "Moral panic should not trigger with low emotional charge"
        );
        assert_eq!(result.legitimacy_damage, Fixed::ZERO);
        assert_eq!(result.grievance_boost, Fixed::ZERO);
    }

    #[test]
    fn moral_panic_no_beliefs_returns_no_trigger() {
        let empty_beliefs: Vec<&[Belief]> = vec![];
        let result = detect_moral_panic(&empty_beliefs, 1, Fixed::ZERO);
        assert!(!result.triggered);
        assert_eq!(result.avg_charge, Fixed::ZERO);
    }

    #[test]
    fn moral_panic_boundary_high_charge_low_ratio() {
        // avg_charge >= 0.55 but panic_ratio < 0.3 (only 1 of 5 agents affected)
        let mut high_charge = make_belief(1, 0.8);
        high_charge.emotional_charge = Fixed::from_f64(0.7);
        let low_charge = make_belief(1, 0.5); // default emotional_charge = 0.3

        let b1 = [high_charge];
        let b2 = [low_charge.clone()];
        let b3 = [low_charge.clone()];
        let b4 = [low_charge.clone()];
        let b5 = [low_charge];
        let beliefs: Vec<&[Belief]> = vec![&b1[..], &b2[..], &b3[..], &b4[..], &b5[..]];

        let result = detect_moral_panic(&beliefs, 1, Fixed::ZERO);
        // avg_charge = (0.7 + 0.3 + 0.3 + 0.3 + 0.3) / 5 = 0.38 — below the floor
        assert!(
            !result.triggered,
            "Should NOT trigger when avg charge is below threshold even with one high agent"
        );
    }
}
