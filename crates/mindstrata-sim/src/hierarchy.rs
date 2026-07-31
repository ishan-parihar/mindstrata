//! §11.3: Hierarchy Formation, Stabilization, and Destabilization.
//!
//! Hierarchies emerge from competence, age, wealth, violence, religious
//! sanction, inheritance, charisma, network centrality, and crisis response.
//!
//! Stabilization requires legitimacy, ritual, reciprocity, punishment,
//! narrative, and institutional memory.
//!
//! Destabilization emerges from humiliation, corruption, scarcity,
//! betrayal, rival prestige, moral panic, and external shock.

use crate::institutions::{Institution, InstitutionKind};
use crate::person::Personality;
use mindstrata_core::event::SimEvent;
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// Threshold for legitimacy below which a leadership challenge becomes possible.
const CHALLENGE_LEGITIMACY_THRESHOLD: Fixed = Fixed::from_raw(4096); // ~0.4

/// Threshold for corruption above which a coup becomes likely.
const COUP_CORRUPTION_THRESHOLD: Fixed = Fixed::from_raw(6144); // ~0.6

/// Cooldown between leadership challenges (ticks).
const CHALLENGE_COOLDOWN: u64 = 500;

/// Cooldown between elections (ticks).
const ELECTION_COOLDOWN: u64 = 1000;

/// The result of a hierarchy challenge attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeResult {
    /// No challenge occurred this tick.
    None,
    /// A leadership election was triggered (peaceful transfer).
    Election {
        old_leader: AgentId,
        new_leader: AgentId,
        support_margin: Fixed,
    },
    /// A coup was attempted (forceful transfer).
    Coup {
        old_leader: AgentId,
        new_leader: AgentId,
    },
}

/// Compute a leadership score for an agent based on §11.3 hierarchy factors.
///
/// Returns a value in [0, 1] representing suitability for leadership.
pub fn leadership_score(
    ambition: Fixed,
    dominance: Fixed,
    extraversion: Fixed,
    conscientiousness: Fixed,
    age: Fixed,
    wealth: Fixed,
    network_centrality: Fixed,
    moral_reputation: Fixed,
    trauma_load: Fixed,
) -> Fixed {
    // §11.3: Competence, charisma, network centrality, moral reputation
    let competence = conscientiousness * Fixed::from_f64(0.2);
    let charisma = ambition * Fixed::from_f64(0.15) + dominance * Fixed::from_f64(0.1) + extraversion * Fixed::from_f64(0.1);
    let network = network_centrality * Fixed::from_f64(0.15);
    let moral = moral_reputation * Fixed::from_f64(0.15);
    // §11.3: Age provides wisdom but trauma reduces effectiveness
    let age_bonus = if age > Fixed::from_f64(40.0) && age < Fixed::from_f64(65.0) {
        Fixed::from_f64(0.1)
    } else {
        Fixed::ZERO
    };
    let trauma_penalty = trauma_load * Fixed::from_f64(0.1);
    let wealth_bonus = wealth * Fixed::from_f64(0.05);

    (competence + charisma + network + moral + age_bonus + wealth_bonus - trauma_penalty).clamp_01()
}

/// Check if a leadership challenge should occur.
///
/// §11.3: Challenges emerge from low legitimacy, high corruption,
/// rival prestige, and institutional crisis.
pub fn should_challenge(
    institution: &Institution,
    challenger_score: Fixed,
    leader_score: Fixed,
    tick: u64,
    last_challenge_tick: u64,
) -> bool {
    // Cooldown check
    if tick.saturating_sub(last_challenge_tick) < CHALLENGE_COOLDOWN {
        return false;
    }

    // §11.3: Low legitimacy enables challenges
    let legitimacy_pressure = (CHALLENGE_LEGITIMACY_THRESHOLD - institution.legitimacy)
        .max(Fixed::ZERO) * Fixed::from_f64(2.0);

    // §11.3: High corruption enables challenges
    let corruption_pressure = if institution.corruption > COUP_CORRUPTION_THRESHOLD {
        (institution.corruption - COUP_CORRUPTION_THRESHOLD) * Fixed::from_f64(3.0)
    } else {
        Fixed::ZERO
    };

    // §11.3: Rival prestige — challenger must be better than current leader
    let prestige_gap = if challenger_score > leader_score {
        challenger_score - leader_score
    } else {
        Fixed::ZERO
    };

    let total_pressure = legitimacy_pressure + corruption_pressure + prestige_gap;
    total_pressure > Fixed::from_f64(0.3)
}

/// Attempt a leadership challenge.
///
/// Returns the result of the challenge attempt.
pub fn attempt_challenge(
    institution: &mut Institution,
    old_leader: AgentId,
    new_leader: AgentId,
    challenger_dominance: Fixed,
    leader_dominance: Fixed,
    corruption: Fixed,
) -> ChallengeResult {
    // §11.3: If corruption is extreme, coup is more likely
    if corruption > COUP_CORRUPTION_THRESHOLD {
        // Coup succeeds if challenger is significantly more dominant
        let coup_strength = challenger_dominance - leader_dominance;
        if coup_strength > Fixed::from_f64(0.2) {
            // Successful coup — transfer leadership
            transfer_leadership(institution, old_leader, new_leader);
            ChallengeResult::Coup {
                old_leader,
                new_leader,
            }
        } else {
            // Failed coup — legitimacy damage
            institution.legitimacy = (institution.legitimacy - Fixed::from_f64(0.1)).max(Fixed::ZERO);
            ChallengeResult::None
        }
    } else {
        // Peaceful election — §11.3: legitimacy transfer through ritual
        transfer_leadership(institution, old_leader, new_leader);
        ChallengeResult::Election {
            old_leader,
            new_leader,
            support_margin: challenger_dominance - leader_dominance,
        }
    }
}

/// Transfer leadership from old_leader to new_leader.
fn transfer_leadership(institution: &mut Institution, old_leader: AgentId, new_leader: AgentId) {
    // Find and update the Leader role
    for role in &mut institution.roles {
        if role.name == "Leader" {
            role.holder = Some(new_leader);
            break;
        }
    }

    // Update institution legitimacy based on transfer type
    // §11.3: Smooth transitions preserve legitimacy; violent ones erode it
    institution.legitimacy = (institution.legitimacy + Fixed::from_f64(0.05)).clamp_01();

    // Record the transfer
    institution.records.push(crate::institutions::InstitutionalRecord {
        tick: 0, // caller should set the tick
        action: format!("Leadership transferred from {} to {}", old_leader.as_u64(), new_leader.as_u64()),
        affected: vec![old_leader, new_leader],
        success: true,
    });
}

/// Compute hierarchy destabilization pressure.
///
/// §11.3: Destabilization emerges from humiliation, corruption, scarcity,
/// betrayal, rival prestige, moral panic, and external shock.
pub fn destabilization_pressure(
    legitimacy: Fixed,
    corruption: Fixed,
    cohesion: Fixed,
    morale: Fixed,
    fear: Fixed,
    treasury: Fixed,
) -> Fixed {
    let legitimacy_pressure = (Fixed::from_f64(0.5) - legitimacy).max(Fixed::ZERO) * Fixed::from_f64(0.3);
    let corruption_pressure = corruption * Fixed::from_f64(0.25);
    let cohesion_pressure = (Fixed::from_f64(0.5) - cohesion).max(Fixed::ZERO) * Fixed::from_f64(0.15);
    let morale_pressure = (Fixed::from_f64(0.3) - morale).max(Fixed::ZERO) * Fixed::from_f64(0.1);
    let fear_pressure = fear * Fixed::from_f64(0.1);
    let treasury_pressure = if treasury < Fixed::from_f64(0.1) {
        Fixed::from_f64(0.1)
    } else {
        Fixed::ZERO
    };

    (legitimacy_pressure + corruption_pressure + cohesion_pressure + morale_pressure + fear_pressure + treasury_pressure).clamp_01()
}

/// Stabilize hierarchy through ritual and legitimacy building.
///
/// §11.3: Stabilization requires legitimacy, ritual, reciprocity,
/// punishment, narrative, and institutional memory.
pub fn stabilize_hierarchy(
    institution: &mut Institution,
    ritual_participation: Fixed,
    punishment_effectiveness: Fixed,
    narrative_strength: Fixed,
) {
    // §11.3: Ritual participation builds legitimacy
    institution.legitimacy = (institution.legitimacy + ritual_participation * Fixed::from_f64(0.02)).clamp_01();

    // §11: Punishment effectiveness reduces corruption
    institution.corruption = (institution.corruption - punishment_effectiveness * Fixed::from_f64(0.01)).max(Fixed::ZERO);

    // §11.3: Narrative strength builds cohesion
    institution.cohesion = (institution.cohesion + narrative_strength * Fixed::from_f64(0.01)).clamp_01();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leadership_score_increases_with_charisma() {
        let score_low = leadership_score(
            Fixed::from_f64(0.2), Fixed::from_f64(0.2), Fixed::from_f64(0.2),
            Fixed::from_f64(0.5), Fixed::from_f64(30.0), Fixed::from_f64(0.3),
            Fixed::from_f64(0.3), Fixed::from_f64(0.5), Fixed::ZERO,
        );
        let score_high = leadership_score(
            Fixed::from_f64(0.8), Fixed::from_f64(0.8), Fixed::from_f64(0.8),
            Fixed::from_f64(0.5), Fixed::from_f64(30.0), Fixed::from_f64(0.3),
            Fixed::from_f64(0.3), Fixed::from_f64(0.5), Fixed::ZERO,
        );
        assert!(score_high > score_low, "High charisma should produce higher leadership score");
    }

    #[test]
    fn leadership_score_reduces_with_trauma() {
        let score_no_trauma = leadership_score(
            Fixed::from_f64(0.5), Fixed::from_f64(0.5), Fixed::from_f64(0.5),
            Fixed::from_f64(0.5), Fixed::from_f64(30.0), Fixed::from_f64(0.3),
            Fixed::from_f64(0.3), Fixed::from_f64(0.5), Fixed::ZERO,
        );
        let score_high_trauma = leadership_score(
            Fixed::from_f64(0.5), Fixed::from_f64(0.5), Fixed::from_f64(0.5),
            Fixed::from_f64(0.5), Fixed::from_f64(30.0), Fixed::from_f64(0.3),
            Fixed::from_f64(0.3), Fixed::from_f64(0.5), Fixed::from_f64(0.8),
        );
        assert!(score_no_trauma > score_high_trauma, "Trauma should reduce leadership score");
    }

    #[test]
    fn destabilization_pressure_increases_with_corruption() {
        let pressure_low = destabilization_pressure(
            Fixed::from_f64(0.7), Fixed::from_f64(0.1), Fixed::from_f64(0.7),
            Fixed::from_f64(0.6), Fixed::ZERO, Fixed::from_f64(0.5),
        );
        let pressure_high = destabilization_pressure(
            Fixed::from_f64(0.7), Fixed::from_f64(0.8), Fixed::from_f64(0.7),
            Fixed::from_f64(0.6), Fixed::ZERO, Fixed::from_f64(0.5),
        );
        assert!(pressure_high > pressure_low, "High corruption should increase destabilization pressure");
    }

    #[test]
    fn stabilize_hierarchy_builds_legitimacy() {
        let mut inst = Institution::new(0, InstitutionKind::Council, "Test".into());
        let old_legitimacy = inst.legitimacy;
        stabilize_hierarchy(&mut inst, Fixed::from_f64(0.5), Fixed::from_f64(0.3), Fixed::from_f64(0.4));
        assert!(inst.legitimacy > old_legitimacy, "Stabilization should increase legitimacy");
    }

    #[test]
    fn challenge_requires_cooldown() {
        let mut inst = Institution::new(0, InstitutionKind::Council, "Test".into());
        inst.legitimacy = Fixed::ZERO; // very low legitimacy
        let challenger = Fixed::from_f64(0.8);
        let leader = Fixed::from_f64(0.3);

        assert!(should_challenge(&inst, challenger, leader, 600, 0), "Should challenge at tick 600 (> cooldown of 500)");
        assert!(!should_challenge(&inst, challenger, leader, 800, 600), "Should NOT challenge during cooldown");
        assert!(should_challenge(&inst, challenger, leader, 1200, 600), "Should challenge after cooldown");
    }
}
