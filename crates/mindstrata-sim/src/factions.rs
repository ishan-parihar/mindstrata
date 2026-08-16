//! Faction dynamics — §29, Phase 8 of the architecture spec.
//!
//! Factions emerge from collective grievance. They are not scripted.
//! They form when:
//!   - enough agents share high grievance (resentment + injustice perception)
//!   - at least one high-ambition agent acts as catalyst/leader
//!   - institutional legitimacy has dropped below a threshold
//!
//! §29.2: "Factions and institutions need mobilization capacity, morale,
//! supplies, leadership, cohesion, casualties, legitimacy of violence."
//!
//! §Phase 8: "Factions form around grievance. Agents recruit others.
//! Institutions respond to unrest."

use crate::institutions::{Institution, InstitutionKind, Role};
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;

/// Minimum grievance threshold for an agent to be recruitable into a faction.
pub const RECRUITMENT_GRIEVANCE_THRESHOLD: Fixed = Fixed::from_raw(4000); // 0.4

/// Minimum average grievance across potential members to trigger faction formation.
pub const FORMATION_GRIEVANCE_THRESHOLD: Fixed = Fixed::from_raw(5000); // 0.5

/// Minimum ambition for a faction leader candidate.
pub const LEADER_AMBITION_THRESHOLD: Fixed = Fixed::from_raw(5500); // 0.55

/// Maximum number of factions that can exist simultaneously.
pub const MAX_FACTIONS: usize = 4;

/// Minimum tick interval between faction formation attempts.
pub const FORMATION_COOLDOWN: u64 = 100;

/// Minimum tick interval between protests from the same faction.
pub const PROTEST_COOLDOWN: u64 = 50;

/// Grievance that triggers a protest event.
pub const PROTEST_GRIEVANCE_THRESHOLD: Fixed = Fixed::from_raw(6500); // 0.65

/// How much a protest reduces institutional legitimacy.
pub const PROTEST_LEGITIMACY_DAMAGE: Fixed = Fixed::from_raw(300); // 0.03

/// How much a successful enforcement response restores legitimacy.
pub const ENFORCEMENT_LEGITIMACY_GAIN: Fixed = Fixed::from_raw(150); // 0.015

/// §7.3: Minimum tick interval between revolutions from the same faction.
pub const REVOLUTION_COOLDOWN: u64 = 200;

/// §7.3 (Iteration 186): post-revolution legitimacy honeymoon window. A
/// fresh regime that just seized power has a popular mandate — its council
/// legitimacy target is floored at 0.5 for this many ticks so the faction
/// trigger (`legitimacy < 0.5`) stays disarmed while the new regime
/// governs. Pre-fix the 0.5 reset decayed back under 0.5 within ~500 ticks
/// (grievance stayed pinned high by the market coin sink), a new faction
/// formed, and calm villages churned 42–129 revolutions per 100K — a
/// revolution every ~800–2400 ticks (≈ 2–7 years of sim time), an absurd
/// regime-change cadence. The honeymoon (≈ 1 sim-month) plus the
/// Iteration-186 coin-sink recirculation let a regime actually govern and
/// grievance genuinely decay before the next crisis can re-arm the
/// trigger.
pub const REVOLUTION_HONEYMOON_TICKS: u64 = 3504;

/// §7.3: Revolution score threshold — when exceeded, faction seizes control.
pub const REVOLUTION_SCORE_THRESHOLD: Fixed = Fixed::from_raw(6000); // 0.6

/// §7.3: Minimum tick before any revolution can occur (allows settlement to stabilize).
pub const REVOLUTION_MIN_TICK: u64 = 500;

/// §7.3: Weight of faction grievance in revolution score.
pub const REVOLUTION_GRIEVANCE_WEIGHT: Fixed = Fixed::from_raw(4000); // 0.4

/// §7.3: Weight of faction size ratio in revolution score.
pub const REVOLUTION_SIZE_WEIGHT: Fixed = Fixed::from_raw(3000); // 0.3

/// §7.3: Weight of council illegitimacy in revolution score.
pub const REVOLUTION_LEGITIMACY_WEIGHT: Fixed = Fixed::from_raw(3000); // 0.3

/// Compute an agent's grievance level from their existing derived state.
///
/// Grievance combines:
/// - resentment (accumulated from repeated injustice)
/// - fear + anger (current emotional state)
/// - low autonomy and meaning (psychological needs)
/// - justice perception (moral foundation: fairness)
pub fn compute_grievance(
    resentment: Fixed,
    fear: Fixed,
    anger: Fixed,
    autonomy_deficit: Fixed,
    meaning_deficit: Fixed,
    justice_perception: Fixed,
    wealth_inequality: Fixed, // §4.3: Gini coefficient drives fairness grievance
) -> Fixed {
    // Resentment is the primary driver
    let base = resentment;

    // Emotional amplification: fear + anger make grievance feel worse
    let emotional = (fear + anger) * Fixed::from_f64(0.2);

    // Need deficits amplify grievance
    let need_factor = (autonomy_deficit + meaning_deficit) * Fixed::from_f64(0.15);

    // Low justice perception (high fairness need but low fairness experienced)
    let justice_factor = (Fixed::ONE - justice_perception) * Fixed::from_f64(0.1);

    // §4.3: Wealth inequality amplifies grievance — high Gini means unfair distribution
    let inequality_factor = wealth_inequality * Fixed::from_f64(0.15);

    (base + emotional + need_factor + justice_factor + inequality_factor).clamp_01()
}

/// Find agents who are potential faction members (high grievance).
pub fn find_recruitable_agents(
    grievances: &[(AgentId, Fixed)],
    threshold: Fixed,
) -> Vec<(AgentId, Fixed)> {
    grievances
        .iter()
        .filter(|(_, g)| *g >= threshold)
        .copied()
        .collect()
}

/// Find the best leader candidate from a group of agents.
///
/// Leader selection considers:
/// - ambition (primary: leaders must want to lead)
/// - dominance (secondary: leaders must be assertive)
/// - extraversion (tertiary: leaders must be social)
/// - anger (negative: overly angry agents are poor leaders)
pub fn select_leader(
    candidates: &[(AgentId, Fixed, Fixed, Fixed, Fixed)], // (id, ambition, dominance, extraversion, anger)
) -> Option<AgentId> {
    if candidates.is_empty() {
        return None;
    }

    // Score each candidate: ambition * 0.4 + dominance * 0.3 + extraversion * 0.2 - anger * 0.1
    let best = candidates.iter().max_by(|a, b| {
        let score_a =
            a.1 * Fixed::from_f64(0.4) + a.2 * Fixed::from_f64(0.3) + a.3 * Fixed::from_f64(0.2)
                - a.4 * Fixed::from_f64(0.1);
        let score_b =
            b.1 * Fixed::from_f64(0.4) + b.2 * Fixed::from_f64(0.3) + b.3 * Fixed::from_f64(0.2)
                - b.4 * Fixed::from_f64(0.1);
        score_a
            .partial_cmp(&score_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    best.map(|c| c.0)
}

/// Create a new faction institution from a leader and initial members.
pub fn create_faction(
    id: u64,
    leader: AgentId,
    members: Vec<AgentId>,
    grievance_level: Fixed,
) -> Institution {
    let mut faction = Institution::new(id, InstitutionKind::Faction, format!("Faction #{id}"));

    // Add leader role
    faction.add_role(Role {
        name: "Leader".into(),
        holder: Some(leader),
        authority: Fixed::from_f64(0.8),
        obligations: vec![
            "Rally members".into(),
            "Negotiate with institutions".into(),
            "Lead protests if needed".into(),
        ],
    });

    // Add all members including leader
    faction.add_member(leader);
    for member in members {
        faction.add_member(member);
    }

    // Initial faction properties based on grievance
    // High grievance = high cohesion (shared enemy) but low legitimacy
    faction.cohesion = (grievance_level * Fixed::from_f64(1.2)).clamp_01();
    faction.legitimacy = Fixed::from_f64(0.3); // factions start with low institutional legitimacy
    faction.enforcement_capacity = Fixed::from_f64(0.1); // no enforcement initially
    faction.communication_capacity = Fixed::from_f64(0.4);

    // Set initial collective psychology
    faction.collective.morale = grievance_level;
    faction.collective.unity = (grievance_level * Fixed::from_f64(0.8)).clamp_01();
    faction.collective.fear = Fixed::from_f64(0.2);
    faction.collective.ambition = grievance_level;
    faction.collective.trust_in_leadership = Fixed::from_f64(0.6);
    faction.collective.ideological_rigidity = Fixed::from_f64(0.3);

    faction
}

/// Check if a protest should occur based on faction grievance and cohesion.
pub fn should_protest(grievance: Fixed, cohesion: Fixed, tick: u64) -> bool {
    // Protests require high grievance AND sufficient cohesion
    // Also add some periodicity: protests are more likely on "round" ticks
    let cohesion_factor = cohesion * Fixed::from_f64(0.3);
    let combined = grievance + cohesion_factor;
    combined >= PROTEST_GRIEVANCE_THRESHOLD && tick.is_multiple_of(PROTEST_COOLDOWN)
}

/// Compute the effect of a protest on institutional legitimacy.
pub fn protest_legitimacy_effect(protest_size: usize, total_population: usize) -> Fixed {
    if total_population == 0 {
        return Fixed::ZERO;
    }
    // Larger protests cause more legitimacy damage
    let size_ratio =
        Fixed::from_int(protest_size as i64) / Fixed::from_int(total_population as i64);
    (PROTEST_LEGITIMACY_DAMAGE * size_ratio * Fixed::from_f64(2.0)).clamp_01()
}

/// Compute how the council responds to a protest.
/// Returns true if the protest is suppressed (council has enough enforcement capacity).
///
/// `protest_strength` is the protesting faction's v2 fighting strength (0–1)
/// — Architecture-plan-2 §29.2: mobilization capacity, morale, cohesion, and
/// casualties determine a faction's ability to actually press a challenge.
/// An armed faction is harder to suppress: the effective protest size ratio
/// scales up with strength. A zero strength (no v2 record / unarmed crowd)
/// is byte-identical to the legacy suppression logic.
pub fn council_response(
    council_enforcement: Fixed,
    protest_size: usize,
    total_population: usize,
    protest_strength: Fixed,
) -> (bool, Fixed) {
    if total_population == 0 {
        return (false, Fixed::ZERO);
    }

    let size_ratio =
        Fixed::from_int(protest_size as i64) / Fixed::from_int(total_population as i64);
    // §29.2: armed factions resist suppression — effective size grows with
    // fighting strength (mobilization × morale × (1 − casualties)).
    let effective_ratio = size_ratio * (Fixed::ONE + protest_strength.clamp_01());

    // Suppression succeeds if enforcement capacity outweighs protest size
    let suppression_threshold = council_enforcement - effective_ratio;
    let suppressed = suppression_threshold > Fixed::ZERO;

    // Legitimacy effect: suppressing a large protest costs legitimacy
    // but failing to suppress also costs legitimacy
    let legitimacy_effect = if suppressed {
        // Successful suppression: small legitimacy gain from showing strength,
        // but cost proportional to protest size (repression is unpopular)
        let gain = ENFORCEMENT_LEGITIMACY_GAIN;
        let repression_cost = size_ratio * Fixed::from_f64(0.02);
        (gain - repression_cost).clamp_01()
    } else {
        // Failed suppression: legitimacy loss proportional to protest size
        -(size_ratio * Fixed::from_f64(0.04)).clamp_01()
    };

    (suppressed, legitimacy_effect)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grievance_computation_basic() {
        let g = compute_grievance(
            Fixed::from_f64(0.8), // resentment
            Fixed::from_f64(0.2), // fear
            Fixed::from_f64(0.3), // anger
            Fixed::from_f64(0.1), // autonomy deficit
            Fixed::from_f64(0.2), // meaning deficit
            Fixed::from_f64(0.5), // justice perception
            Fixed::from_f64(0.3), // wealth inequality (Gini)
        );
        // resentment (0.8) + emotional (0.2*0.2=0.04) + need (0.3*0.15=0.045) + justice (0.5*0.1=0.05) + inequality (0.3*0.15=0.045) = ~0.98
        assert!(g > Fixed::from_f64(0.8));
        assert!(g <= Fixed::ONE);
    }

    #[test]
    fn grievance_no_resentment_is_low() {
        let g = compute_grievance(
            Fixed::ZERO, // no resentment
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ONE,  // perfect justice
            Fixed::ZERO, // no inequality
        );
        assert_eq!(g, Fixed::ZERO);
    }

    #[test]
    fn recruitablity_filters_by_threshold() {
        let agents = vec![
            (AgentId::new(0), Fixed::from_f64(0.3)),
            (AgentId::new(1), Fixed::from_f64(0.6)),
            (AgentId::new(2), Fixed::from_f64(0.7)),
            (AgentId::new(3), Fixed::from_f64(0.2)),
        ];
        let recruited = find_recruitable_agents(&agents, Fixed::from_f64(0.5));
        assert_eq!(recruited.len(), 2);
        assert_eq!(recruited[0].0, AgentId::new(1));
        assert_eq!(recruited[1].0, AgentId::new(2));
    }

    #[test]
    fn leader_selection_prefers_high_ambition() {
        let candidates = vec![
            (
                AgentId::new(0),
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.1),
            ),
            (
                AgentId::new(1),
                Fixed::from_f64(0.9),
                Fixed::from_f64(0.7),
                Fixed::from_f64(0.6),
                Fixed::from_f64(0.2),
            ),
            (
                AgentId::new(2),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.4),
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.1),
            ),
        ];
        let leader = select_leader(&candidates);
        assert_eq!(leader, Some(AgentId::new(1)));
    }

    #[test]
    fn leader_selection_empty_candidates() {
        assert!(select_leader(&[]).is_none());
    }

    #[test]
    fn faction_creation_structure() {
        let faction = create_faction(
            100,
            AgentId::new(0),
            vec![AgentId::new(1), AgentId::new(2)],
            Fixed::from_f64(0.7),
        );

        assert_eq!(faction.kind, InstitutionKind::Faction);
        assert_eq!(faction.members.len(), 3);
        assert_eq!(faction.get_role_holder("Leader"), Some(AgentId::new(0)));
        assert!(faction.cohesion > Fixed::from_f64(0.5));
        assert!(faction.legitimacy < Fixed::from_f64(0.5));
    }

    #[test]
    fn protest_requires_sufficient_grievance_and_cohesion() {
        // High grievance + high cohesion = protest
        assert!(should_protest(
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.6),
            100,
        ));

        // Low grievance = no protest
        assert!(!should_protest(
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.8),
            100,
        ));

        // High grievance but not on a round tick = no protest
        assert!(!should_protest(
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.6),
            101,
        ));
    }

    #[test]
    fn protest_legitimacy_damage_scales_with_size() {
        let small = protest_legitimacy_effect(2, 12);
        let large = protest_legitimacy_effect(8, 12);
        assert!(large > small);
    }

    #[test]
    fn council_suppression_works() {
        // Strong council, small protest = suppressed
        let (suppressed, _) = council_response(Fixed::from_f64(0.7), 2, 12, Fixed::ZERO);
        assert!(suppressed);

        // Weak council, large protest = not suppressed
        let (suppressed, _) = council_response(Fixed::from_f64(0.2), 8, 12, Fixed::ZERO);
        assert!(!suppressed);
    }

    #[test]
    fn zero_protest_strength_is_legacy_identical() {
        // With protest_strength = 0 the armed-scale factor is 1.0 — the
        // effective ratio equals the plain size ratio, so suppression and
        // legitimacy outcomes must match the pre-§29.2 logic exactly.
        let (suppressed_a, effect_a) = council_response(Fixed::from_f64(0.5), 3, 12, Fixed::ZERO);
        let (suppressed_b, effect_b) =
            council_response(Fixed::from_f64(0.5), 3, 12, Fixed::from_f64(0.001));
        assert_eq!(suppressed_a, suppressed_b);
        assert_eq!(effect_a, effect_b);
    }

    #[test]
    fn armed_faction_resists_suppression() {
        // Borderline suppression: enforcement 0.5 vs size ratio 3/12 = 0.25.
        // Unarmed (0.0): threshold 0.5 − 0.25 = 0.25 > 0 → suppressed.
        // Armed (1.0): effective ratio 0.25 × 2.0 = 0.5 → threshold 0.0 →
        // not suppressed. The same crowd with fighting strength resists.
        let (suppressed_unarmed, _) = council_response(Fixed::from_f64(0.5), 3, 12, Fixed::ZERO);
        let (suppressed_armed, _) = council_response(Fixed::from_f64(0.5), 3, 12, Fixed::ONE);
        assert!(suppressed_unarmed);
        assert!(!suppressed_armed);
    }

    #[test]
    fn higher_strength_is_never_easier_to_suppress() {
        for strength_a in [0.0f64, 0.3, 0.6, 0.9] {
            for strength_b in [0.1f64, 0.4, 0.7, 1.0] {
                let (suppressed_a, _) =
                    council_response(Fixed::from_f64(0.5), 4, 12, Fixed::from_f64(strength_a));
                let (suppressed_b, _) =
                    council_response(Fixed::from_f64(0.5), 4, 12, Fixed::from_f64(strength_b));
                // A stronger faction is never suppressed while a weaker one is
                // not (stronger factions only ever resist suppression).
                assert!(
                    !(strength_b > strength_a && suppressed_b && !suppressed_a),
                    "strength {strength_b} suppressed but weaker {strength_a} free"
                );
            }
        }
    }
}
