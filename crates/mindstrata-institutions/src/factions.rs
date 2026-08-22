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

// §29.2 (Iteration 240): Crisis-pressure hazard accumulator. Faction
// formation previously required the council-legitimacy cliff (< 0.5) AND a
// ≥ 0.5 recruitable-grievance mean to hold at the SAME evaluation tick — a
// knife-edge that stopped firing entirely once the Iter-186 legitimacy
// equilibrium (floor 0.6) settled at ~0.53 even under pestilence
// (probe: pestilence seed 5 @30K = 0 factions, 0 revolutions, legit
// 0.525–0.546). Sustained discontent now accumulates deterministic crisis
// pressure instead: grievance above the anchor and council illegitimacy
// below the gate each contribute build; genuine peace bleeds pressure off
// proportionally. Crossing the threshold arms formation through the exact
// legacy path (same recruitment, leader selection, registration,
// provenance). f64 storage: per-tick increments (~1e-5…1e-4) sit below the
// Fixed-4 quantum — the thermal-Iter-239 lesson — so Fixed accumulation
// would stall at zero forever. Pure arithmetic on already-computed
// aggregates: zero RNG-stream consumption, replay determinism preserved.
// The three constants below are the calibration surface.

/// Grievance anchor for the crisis-pressure build term. Deliberately BELOW
/// `FORMATION_GRIEVANCE_THRESHOLD` (0.5) and calibrated against probed pool
/// means: a pristine population runs ≈0.45–0.52 (calm-42 ≈0.455–0.465,
/// epidemic peaks higher), so ordinary worlds accumulate slowly (liveness:
/// mild discontent eventually organizes — a stable village forms ~1 faction
/// per 30–50K instead of never), while the 10K calibrated windows stay far
/// below the 0.5 arm threshold (probe: calm-42 pressure ≈0.1 @10K). NOTE:
/// the anchor applies to the POOL MEAN, which drops sharply once a faction
/// absorbs the most aggrieved agents (probed 0.46 → 0.28) — self-limiting
/// cascade formation.
pub const CRISIS_GRIEVANCE_ANCHOR: f64 = 0.45;

/// Build rate per unit of crisis excess (grievance above `CRISIS_GRIEVANCE_ANCHOR`,
/// council-legitimacy deficit below 0.5), applied per tick. Calibration: a
/// sustained excess of 0.05 (epidemic-grade discontent) arms in ~5K ticks; the
/// calm-world excess (~0.005–0.015) arms in ~25–50K — liveness without
/// clock-coups (the Iter-186 lesson).
pub const CRISIS_PRESSURE_BUILD_RATE: f64 = 0.002;

/// Proportional pressure bleed per tick while NO crisis signal is present
/// (τ ≈ 2000 ticks). A real stretch of peace erases accumulated pressure so
/// transient spikes cannot compound into organization.
pub const CRISIS_PRESSURE_BLEED_RATE: f64 = 0.0005;

/// Accumulated pressure that arms faction formation (through the legacy
/// recruitment/leader path).
pub const FACTION_FORMATION_PRESSURE_THRESHOLD: f64 = 0.5;

/// §29.2 (Iteration 240): Minimum ticks between PRESSURE-ARMED formations.
/// A village that just organized a protest bloc does not immediately spawn
/// another — without this refractory, compounding crisis grievance
/// (epidemic pool-mean probed up to 0.72 → excess 0.27) re-arms the
/// accumulator within ~1K ticks and churns form/dissolve cycles (~20
/// formations / 30K probed). One formation per ~2 months of sustained peak
/// crisis preserves the pressure-valve dynamic while giving each bloc time
/// to act (protests, negotiation, dissolution). The legacy legitimacy-cliff
/// path bypasses this gate (an outright legitimacy collapse always arms).
pub const FACTION_REARM_COOLDOWN_TICKS: u64 = 8640;

/// Minimum tick interval between faction formation attempts.
pub const FORMATION_COOLDOWN: u64 = 100;

/// Minimum tick interval between protests from the same faction.
pub const PROTEST_COOLDOWN: u64 = 50;

/// §7.3 (Iteration 240): Minimum ticks between a faction's formation and its
/// eligibility to stage a revolution — the mobilization/entrenchment window.
/// A freshly formed faction previously carried its full founding size into
/// the revolution score immediately, so an epidemic village (size fraction
/// ~0.8, grievance ~0.7, council legitimacy ~0.52 → score ≈0.66 vs
/// threshold 0.6) formed and overthrew the council within days, making
/// every crisis faction invisible at sampling instants and reducing
/// "politics" to instant coups. One month of organizing (protests,
/// recruitment, negotiation) precedes any revolt attempt.
pub const FACTION_ENTRENCHMENT_MIN_TICKS: u64 = 4320;

/// §29.2 (Iteration 240): Recruitment conviction threshold — a candidate
/// joins an active faction only when `faction.morale × candidate_grievance`
/// clears this bar. At the 0.25 anchor a healthy faction (morale 0.5)
/// recruits agents with grievance ≥ 0.5; a demoralized one (morale 0.2)
/// needs near-certain believers (≥ 0.75) — recruitment self-throttles as
/// the faction weakens, and dead factions recruit nobody.
pub const FACTION_RECRUIT_CONVICTION_THRESHOLD: Fixed = Fixed::from_raw(2500); // 0.25

/// Grievance that triggers a protest event.
pub const PROTEST_GRIEVANCE_THRESHOLD: Fixed = Fixed::from_raw(6500); // 0.65

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
/// Revolution score threshold.
///
/// Iteration 242 recalibration 0.6 -> 0.5: formation no longer requires a
/// legitimacy cliff, so factions organize against a ~0.55-legitimacy council
/// and the old threshold was unreachable in every probed world (pestilence
/// seed 5: 4 blocs / 160K, ZERO revolts - the §7.3 mechanism was dead). At
/// 0.5, sustained multi-crisis worlds cross via grievance + size + pressure
/// momentum, while calm worlds top out ~0.40 (small factions, healthy
/// legitimacy, near-zero pressure) - crisis-driven rebellion preserved.
pub const REVOLUTION_SCORE_THRESHOLD: Fixed = Fixed::from_raw(5000); // 0.5

/// §7.3: Minimum tick before any revolution can occur (allows settlement to stabilize).
pub const REVOLUTION_MIN_TICK: u64 = 500;

/// §7.3: Weight of faction grievance in revolution score.
pub const REVOLUTION_GRIEVANCE_WEIGHT: Fixed = Fixed::from_raw(4000); // 0.4

/// §7.3: Weight of faction size ratio in revolution score.
pub const REVOLUTION_SIZE_WEIGHT: Fixed = Fixed::from_raw(3000); // 0.3

/// §7.3: Weight of council illegitimacy in revolution score.
pub const REVOLUTION_LEGITIMACY_WEIGHT: Fixed = Fixed::from_raw(3000); // 0.3

/// §7.3 (Iteration 242): Weight of the accumulated crisis-pressure tank in
/// the revolution score. Restores the §7.3 firing path after formation
/// stopped requiring a legitimacy cliff: without it no probed world could
/// cross the 0.6 threshold again (pestilence seed 5: 4 protest blocs /
/// 160K, ZERO revolts — the mechanism was dead). A full pressure tank adds
/// +0.2 — enough for a bloc with meaningful grievance and size to revolt
/// against a ~0.55-legitimacy council; calm-world residue (+≤0.06) stays
/// sub-threshold.
pub const REVOLUTION_PRESSURE_WEIGHT: f64 = 0.2;

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
