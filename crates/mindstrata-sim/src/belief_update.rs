//! Belief update system — Bayesian-ish updating with psychological bias.
//!
//! Agents update beliefs based on evidence, but with psychological
//! resistance: identity-linked, emotionally charged, and socially
//! reinforced beliefs resist change.  This produces ideology,
//! propaganda, rumor, and polarization.

use crate::person::Belief;
use mindstrata_core::fixed::Fixed;

/// Update a single belief based on new evidence.
///
/// Uses a simplified Bayesian update with psychological resistance:
///
/// ```text
/// new_confidence =
///     old_confidence * resistance
///   + effective_evidence * blended_trust
///   + emotional_reinforcement
///   + social_reinforcement
/// ```
/// where `effective_evidence = evidence_strength * (2 − rigidity_factor)`
/// (§10.1.3, Iteration 109 — the noospheric dogmatism dampening; identity
/// when `rigidity_factor` is 1.0).
pub fn update_belief(
    belief: &mut Belief,
    evidence_strength: Fixed,
    source_trust: Fixed,
    emotional_reinforcement: Fixed,
    social_reinforcement_delta: Fixed,
    current_tick: u64,
    params: &crate::parameters::SimParameters,
    rigidity_factor: Fixed,
) {
    let resistance = belief.resistance;

    // §10.1.3 (Iteration 109): the noospheric field's mean belief
    // confidence dampens incoming evidence — an agent holding its beliefs
    // with high mean confidence weights counter- and confirming evidence
    // less (dogmatism / closed-mindedness). Identity at rigidity_factor
    // 1.0 (no beliefs → byte-identical; ×1.0 is exact). NOTE: we dampen
    // the EVIDENCE term, NOT `resistance` — the formula weights old
    // confidence by resistance, so scaling resistance up would AMPLIFY
    // positive updates (verified empirically); the evidence dampening is
    // the honest closed-mindedness direction. Multiplication (not
    // division) keeps the dampening exact fixed-point. The raw evidence
    // still gates reinforcement decay.
    let effective_evidence = evidence_strength * (Fixed::from_f64(2.0) - rigidity_factor);

    // §19.5.A: Blend explicit source_trust with belief's source base_trust.
    // This means the evidence source of the belief itself modulates trust.
    let blended_trust =
        (source_trust + belief.source.base_trust()) * params.belief_trust_blend_factor;

    // §19.5.A: Track social reinforcement count when social evidence confirms belief.
    if social_reinforcement_delta > Fixed::ZERO {
        belief.social_reinforcement = belief.social_reinforcement.saturating_add(1);
    }

    // Base update: weighted combination
    let base_update = belief.confidence * resistance
        + effective_evidence * blended_trust
        + emotional_reinforcement
        + social_reinforcement_delta;

    // Identity protection bias: beliefs with high identity linkage resist change
    let identity_protection = if belief.identity_linkage > params.belief_identity_linkage_threshold
    {
        let protection =
            Fixed::ONE - belief.identity_linkage * params.belief_identity_protection_strength;
        belief.confidence * (Fixed::ONE - protection) + base_update * protection
    } else {
        base_update
    };

    // Clamp to [0, 1]
    belief.confidence = identity_protection.clamp_01();

    // Reinforcement decays resistance over time
    let time_since_reinforce = current_tick.saturating_sub(belief.last_reinforced_tick);
    if time_since_reinforce > 0 && evidence_strength > Fixed::ZERO {
        let decay =
            params.belief_resistance_decay_rate * Fixed::from_int(time_since_reinforce as i64);
        belief.resistance = (belief.resistance - decay).clamp_01();
        belief.last_reinforced_tick = current_tick;
    }
}

/// Batch update all beliefs for an agent.
///
/// `cultural_dampening` (0..=1) scales the EVIDENCE term of every update —
/// the §8.1.18 sacred-boundary defense: an agent whose culture is more
/// conservative/taboo-bound absorbs less counter- and confirming evidence,
/// so its beliefs move less per interaction (the same honest
/// closed-mindedness direction as the `rigidity_factor` dogmatism term —
/// evidence dampening, never `resistance` scaling, per the Iteration-109
/// lesson). At dampening 1.0 (no cultural resistance) the formula is
/// byte-identical to the pre-Iteration-168 behavior. ONE-SIDED: a dampening
/// factor is always ≤ 1.0 and can only shrink the evidence magnitude,
/// never flip its sign.
pub fn update_beliefs(
    beliefs: &mut [Belief],
    evidence: &[(u64, Fixed, Fixed)], // (proposition_id, evidence_strength, source_trust)
    emotional_reinforcement: Fixed,
    social_reinforcement: Fixed,
    current_tick: u64,
    params: &crate::parameters::SimParameters,
    rigidity_factor: Fixed,
    cultural_dampening: Fixed,
) {
    for (prop_id, strength, trust) in evidence {
        if let Some(belief) = beliefs.iter_mut().find(|b| b.proposition_id == *prop_id) {
            update_belief(
                belief,
                *strength * cultural_dampening,
                *trust,
                emotional_reinforcement,
                social_reinforcement,
                current_tick,
                params,
                rigidity_factor,
            );
        }
    }
}

/// Decay belief resistance over time (beliefs become slightly easier to update).
pub fn decay_belief_resistance(
    beliefs: &mut [Belief],
    decay_rate: Fixed,
    params: &crate::parameters::SimParameters,
) {
    for belief in beliefs.iter_mut() {
        let baseline = params.belief_resistance_baseline;
        if belief.resistance > baseline {
            belief.resistance = (belief.resistance - decay_rate).clamp_01();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn belief_update_with_strong_evidence() {
        let mut belief = Belief {
            proposition_id: 0,
            confidence: Fixed::from_f64(0.3),
            emotional_charge: Fixed::ZERO,
            identity_linkage: Fixed::ZERO,
            resistance: Fixed::from_f64(0.5),
            last_reinforced_tick: 0,
            source: crate::person::EvidenceSource::PersonalExperience,
            social_reinforcement: 0,
            is_accurate: true,
        };

        update_belief(
            &mut belief,
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.8),
            Fixed::ZERO,
            Fixed::ZERO,
            100,
            &crate::parameters::SimParameters::default(),
            // §10.1.3 (Iteration 109): identity factor — legacy tests keep
            // rigidity 1.0 so the pre-consumer expectations hold exactly.
            Fixed::ONE,
        );

        assert!(
            belief.confidence > Fixed::from_f64(0.3),
            "Confidence should increase with strong evidence"
        );
    }

    #[test]
    fn identity_linked_belief_resists_change() {
        let mut belief = Belief {
            proposition_id: 0,
            confidence: Fixed::from_f64(0.8),
            emotional_charge: Fixed::from_f64(0.5),
            identity_linkage: Fixed::from_f64(0.9),
            resistance: Fixed::from_f64(0.7),
            last_reinforced_tick: 0,
            source: crate::person::EvidenceSource::PersonalExperience,
            social_reinforcement: 0,
            is_accurate: true,
        };

        update_belief(
            &mut belief,
            Fixed::from_f64(-0.5),
            Fixed::from_f64(0.8),
            Fixed::ZERO,
            Fixed::ZERO,
            100,
            &crate::parameters::SimParameters::default(),
            // §10.1.3 (Iteration 109): identity factor — legacy tests keep
            // rigidity 1.0 so the pre-consumer expectations hold exactly.
            Fixed::ONE,
        );

        // Identity-linked beliefs resist change, but should still move somewhat
        assert!(
            belief.confidence > Fixed::from_f64(0.3),
            "Identity-linked beliefs should resist contradictory evidence, got {}",
            belief.confidence.to_f64()
        );
    }

    #[test]
    fn rigid_belief_updates_move_less() {
        // §10.1.3 (Iteration 109): the dogmatism factor scales update
        // resistance — under the same evidence, a belief updated with a
        // rigidity factor > 1.0 must move strictly less than with 1.0.
        let params = crate::parameters::SimParameters::default();
        let make = |confidence: f64| Belief {
            proposition_id: 0,
            confidence: Fixed::from_f64(confidence),
            emotional_charge: Fixed::ZERO,
            identity_linkage: Fixed::ZERO,
            resistance: Fixed::from_f64(0.5),
            last_reinforced_tick: 0,
            source: crate::person::EvidenceSource::PersonalExperience,
            social_reinforcement: 0,
            is_accurate: true,
        };

        let mut loose = make(0.5);
        update_belief(
            &mut loose,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.8),
            Fixed::ZERO,
            Fixed::ZERO,
            100,
            &params,
            Fixed::ONE,
        );

        let mut rigid = make(0.5);
        update_belief(
            &mut rigid,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.8),
            Fixed::ZERO,
            Fixed::ZERO,
            100,
            &params,
            Fixed::from_f64(1.275),
        );

        assert!(
            (loose.confidence - Fixed::from_f64(0.5)).abs()
                > (rigid.confidence - Fixed::from_f64(0.5)).abs(),
            "the rigid belief must move strictly less under identical evidence (loose {} vs rigid {})",
            loose.confidence.to_f64(),
            rigid.confidence.to_f64()
        );
        assert!(
            rigid.confidence < loose.confidence,
            "rigidity must dampen the upward movement"
        );
    }

    #[test]
    fn cultural_dampening_scales_evidence_one_sided() {
        // §8.1.18 (Iteration 168): the cultural change-resistance dampening
        // scales the EVIDENCE term only — under identical input, a belief
        // updated with dampening < 1.0 moves strictly less than with
        // identity (1.0), and the direction never flips (ONE-SIDED: a
        // dampening factor is always ≤ 1.0, so it can only shrink evidence
        // magnitude, never flip its sign).
        let params = crate::parameters::SimParameters::default();
        let make = || Belief {
            proposition_id: 2,
            confidence: Fixed::from_f64(0.5),
            emotional_charge: Fixed::ZERO,
            identity_linkage: Fixed::ZERO,
            resistance: Fixed::from_f64(0.5),
            last_reinforced_tick: 0,
            source: crate::person::EvidenceSource::PersonalExperience,
            social_reinforcement: 0,
            is_accurate: true,
        };
        let evidence = [(2u64, Fixed::from_f64(0.6), Fixed::from_f64(0.8))];

        let mut undamped = vec![make()];
        update_beliefs(
            &mut undamped,
            &evidence,
            Fixed::ZERO,
            Fixed::ZERO,
            100,
            &params,
            Fixed::ONE,
            Fixed::ONE,
        );
        let mut damped = vec![make()];
        update_beliefs(
            &mut damped,
            &evidence,
            Fixed::ZERO,
            Fixed::ZERO,
            100,
            &params,
            Fixed::ONE,
            Fixed::from_f64(0.5),
        );

        assert!(
            undamped[0].confidence > Fixed::from_f64(0.5),
            "undamped evidence must move confidence up (got {})",
            undamped[0].confidence.to_f64()
        );
        assert!(
            damped[0].confidence > Fixed::from_f64(0.5),
            "dampened evidence must still move in the same direction (got {})",
            damped[0].confidence.to_f64()
        );
        assert!(
            damped[0].confidence < undamped[0].confidence,
            "the dampened belief must move strictly less (damped {} vs undamped {})",
            damped[0].confidence.to_f64(),
            undamped[0].confidence.to_f64()
        );
    }
}
