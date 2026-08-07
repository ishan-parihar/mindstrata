//! §10.3 Relationship stage logic — thresholds, advancement, and regression.
//!
//! This module provides the computational logic for relationship stage transitions,
//! working with the `RelationshipStage` enum defined in `relationship_v2`.
//!
//! Architecture §10.3: Relationships progress through stages that encode
//! social meaning beyond numeric trust/affection values. Stage transitions
//! depend on interaction count, trust, affection, fear, and context.

use mindstrata_core::fixed::Fixed;
use super::kinship::KinshipLink;
use super::relationship_v2::RelationshipStage;

/// Minimum interaction count to advance beyond each stage.
pub fn min_interactions_for_stage(stage: RelationshipStage) -> u32 {
    match stage {
        RelationshipStage::Unnoticed => 0,
        RelationshipStage::Noticed => 0,
        RelationshipStage::Acquaintance => 1,
        RelationshipStage::Familiar => 3,
        RelationshipStage::Neighbor => 8,
        RelationshipStage::Friend => 15,
        RelationshipStage::CloseFriend => 30,
        RelationshipStage::Confidant => 50,
        RelationshipStage::Ally => 40,
        // Negative stages don't advance via positive interactions
        RelationshipStage::Disliked => 0,
        RelationshipStage::Rival => 0,
        RelationshipStage::Enemy => 0,
        RelationshipStage::Nemesis => 0,
        // Kin stages are assigned, not advanced
        RelationshipStage::Kin => 0,
        RelationshipStage::ParentChild => 0,
        RelationshipStage::Sibling => 0,
        RelationshipStage::Cousin => 0,
        RelationshipStage::InLaw => 0,
        RelationshipStage::AncestorDescendant => 0,
    }
}

/// Minimum trust to advance beyond each stage.
pub fn trust_threshold_for_stage(stage: RelationshipStage) -> Fixed {
    match stage {
        RelationshipStage::Unnoticed => Fixed::ZERO,
        RelationshipStage::Noticed => Fixed::from_f64(0.05),
        RelationshipStage::Acquaintance => Fixed::from_f64(0.15),
        RelationshipStage::Familiar => Fixed::from_f64(0.3),
        RelationshipStage::Neighbor => Fixed::from_f64(0.4),
        RelationshipStage::Friend => Fixed::from_f64(0.55),
        RelationshipStage::CloseFriend => Fixed::from_f64(0.7),
        RelationshipStage::Confidant => Fixed::from_f64(0.8),
        RelationshipStage::Ally => Fixed::from_f64(0.6),
        RelationshipStage::Disliked => Fixed::from_f64(0.3),
        RelationshipStage::Rival => Fixed::from_f64(0.2),
        RelationshipStage::Enemy => Fixed::from_f64(0.1),
        RelationshipStage::Nemesis => Fixed::from_f64(0.05),
        RelationshipStage::Kin => Fixed::from_f64(0.5),
        RelationshipStage::ParentChild => Fixed::from_f64(0.6),
        RelationshipStage::Sibling => Fixed::from_f64(0.5),
        RelationshipStage::Cousin => Fixed::from_f64(0.35),
        RelationshipStage::InLaw => Fixed::from_f64(0.3),
        RelationshipStage::AncestorDescendant => Fixed::from_f64(0.35),
    }
}

/// Minimum affection to advance beyond each stage.
pub fn affection_threshold_for_stage(stage: RelationshipStage) -> Fixed {
    match stage {
        RelationshipStage::Unnoticed => Fixed::ZERO,
        RelationshipStage::Noticed => Fixed::ZERO,
        RelationshipStage::Acquaintance => Fixed::from_f64(0.05),
        RelationshipStage::Familiar => Fixed::from_f64(0.15),
        RelationshipStage::Neighbor => Fixed::from_f64(0.25),
        RelationshipStage::Friend => Fixed::from_f64(0.4),
        RelationshipStage::CloseFriend => Fixed::from_f64(0.55),
        RelationshipStage::Confidant => Fixed::from_f64(0.7),
        RelationshipStage::Ally => Fixed::from_f64(0.2),
        RelationshipStage::Disliked => Fixed::from_f64(0.2),
        RelationshipStage::Rival => Fixed::from_f64(0.1),
        RelationshipStage::Enemy => Fixed::from_f64(0.05),
        RelationshipStage::Nemesis => Fixed::from_f64(0.0),
        RelationshipStage::Kin => Fixed::from_f64(0.3),
        RelationshipStage::ParentChild => Fixed::from_f64(0.4),
        RelationshipStage::Sibling => Fixed::from_f64(0.3),
        RelationshipStage::Cousin => Fixed::from_f64(0.2),
        RelationshipStage::InLaw => Fixed::from_f64(0.15),
        RelationshipStage::AncestorDescendant => Fixed::from_f64(0.2),
    }
}

/// Base obligation multiplier from each stage.
pub fn obligation_multiplier_for_stage(stage: RelationshipStage) -> Fixed {
    match stage {
        RelationshipStage::Unnoticed => Fixed::ZERO,
        RelationshipStage::Noticed => Fixed::from_f64(0.01),
        RelationshipStage::Acquaintance => Fixed::from_f64(0.05),
        RelationshipStage::Familiar => Fixed::from_f64(0.1),
        RelationshipStage::Neighbor => Fixed::from_f64(0.2),
        RelationshipStage::Friend => Fixed::from_f64(0.35),
        RelationshipStage::CloseFriend => Fixed::from_f64(0.5),
        RelationshipStage::Confidant => Fixed::from_f64(0.7),
        RelationshipStage::Ally => Fixed::from_f64(0.6),
        RelationshipStage::Disliked => Fixed::from_f64(0.0),
        RelationshipStage::Rival => Fixed::from_f64(0.0),
        RelationshipStage::Enemy => Fixed::from_f64(0.0),
        RelationshipStage::Nemesis => Fixed::from_f64(0.0),
        RelationshipStage::Kin => Fixed::from_f64(0.4),
        RelationshipStage::ParentChild => Fixed::from_f64(0.8),
        RelationshipStage::Sibling => Fixed::from_f64(0.5),
        RelationshipStage::Cousin => Fixed::from_f64(0.25),
        RelationshipStage::InLaw => Fixed::from_f64(0.2),
        RelationshipStage::AncestorDescendant => Fixed::from_f64(0.4),
    }
}

/// §10.3 (AP2): Map a kinship-graph link onto the kin-branch stage it
/// implies. `Spouse` maps to `None` — marriage is a separate institution
/// (§10.5), not a kin stage; ritual/structural kin (adoptive, godparent,
/// oath-sibling) map to the generic `Kin` stage.
pub fn kin_stage_for_link(link: KinshipLink) -> Option<RelationshipStage> {
    match link {
        KinshipLink::ParentChild => Some(RelationshipStage::ParentChild),
        KinshipLink::Sibling => Some(RelationshipStage::Sibling),
        KinshipLink::InLaw => Some(RelationshipStage::InLaw),
        KinshipLink::Adoptive
        | KinshipLink::Godparent
        | KinshipLink::OathSibling => Some(RelationshipStage::Kin),
        KinshipLink::Spouse => None,
    }
}

/// §10.3 (AP2): Is this stage a kin-branch stage (assigned from the kinship
/// graph, not advanced by the social progression machinery)?
pub fn is_kin_stage(stage: RelationshipStage) -> bool {
    matches!(
        stage,
        RelationshipStage::Kin
            | RelationshipStage::ParentChild
            | RelationshipStage::Sibling
            | RelationshipStage::Cousin
            | RelationshipStage::InLaw
            | RelationshipStage::AncestorDescendant
    )
}

/// Try to advance a relationship stage based on current metrics.
///
/// Returns the new stage if advancement occurred, None otherwise.
pub fn try_advance_stage(
    current: RelationshipStage,
    interactions: u32,
    trust: Fixed,
    affection: Fixed,
) -> Option<RelationshipStage> {
    let next = current.next_positive()?;

    if interactions >= min_interactions_for_stage(next)
        && trust >= trust_threshold_for_stage(next)
        && affection >= affection_threshold_for_stage(next)
    {
        Some(next)
    } else {
        None
    }
}

/// Try to regress a relationship stage based on deteriorating metrics.
///
/// For negative-escalation stages (Neighbor→Disliked→Rival→etc.), uses `next_negative()`.
/// For positive stages above Neighbor, falls back to the previous positive stage.
///
/// Returns the new stage if regression occurred, None otherwise.
pub fn try_regress_stage(
    current: RelationshipStage,
    trust: Fixed,
    fear: Fixed,
) -> Option<RelationshipStage> {
    // Regress if trust drops below the threshold for the CURRENT stage
    // or if fear is high enough to push toward negative stages (§10.3)
    let trust_pressure = trust < trust_threshold_for_stage(current);
    let fear_pressure = fear > Fixed::from_f64(0.6);

    if trust_pressure || fear_pressure {
        // Try negative escalation first (Neighbor → Disliked → Rival → ...)
        if let Some(neg) = current.next_negative() {
            return Some(neg);
        }
        // For positive stages with no negative path, regress to previous positive stage
        // e.g. Friend → Neighbor, CloseFriend → Friend, etc.
        match current {
            RelationshipStage::Ally => Some(RelationshipStage::Friend),
            RelationshipStage::Confidant => Some(RelationshipStage::CloseFriend),
            RelationshipStage::CloseFriend => Some(RelationshipStage::Friend),
            RelationshipStage::Friend => Some(RelationshipStage::Neighbor),
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_advances_with_sufficient_metrics() {
        let result = try_advance_stage(
            RelationshipStage::Unnoticed,
            1,
            Fixed::from_f64(0.1),
            Fixed::from_f64(0.05),
        );
        assert_eq!(result, Some(RelationshipStage::Noticed));
    }

    #[test]
    fn stage_does_not_advance_without_enough_interactions() {
        let result = try_advance_stage(
            RelationshipStage::Familiar,
            2, // needs 8 for Neighbor
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.3),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn stage_regresses_on_low_trust() {
        let result = try_regress_stage(
            RelationshipStage::Friend,
            Fixed::from_f64(0.3), // below Friend threshold of 0.55
            Fixed::ZERO,
        );
        assert!(result.is_some());
    }

    #[test]
    fn obligation_increases_with_stage() {
        let low = obligation_multiplier_for_stage(RelationshipStage::Acquaintance);
        let high = obligation_multiplier_for_stage(RelationshipStage::Confidant);
        assert!(high > low);
    }

    #[test]
    fn kin_stage_mapping_covers_all_links() {
        use super::super::kinship::KinshipLink;
        assert_eq!(
            kin_stage_for_link(KinshipLink::ParentChild),
            Some(RelationshipStage::ParentChild)
        );
        assert_eq!(
            kin_stage_for_link(KinshipLink::Sibling),
            Some(RelationshipStage::Sibling)
        );
        assert_eq!(
            kin_stage_for_link(KinshipLink::InLaw),
            Some(RelationshipStage::InLaw)
        );
        assert_eq!(
            kin_stage_for_link(KinshipLink::Adoptive),
            Some(RelationshipStage::Kin)
        );
        assert_eq!(
            kin_stage_for_link(KinshipLink::OathSibling),
            Some(RelationshipStage::Kin)
        );
        // Marriage is an institution, not a kin stage.
        assert_eq!(kin_stage_for_link(KinshipLink::Spouse), None);
    }

    #[test]
    fn kin_stages_are_not_advanceable() {
        for stage in [
            RelationshipStage::Kin,
            RelationshipStage::ParentChild,
            RelationshipStage::Sibling,
            RelationshipStage::Cousin,
            RelationshipStage::InLaw,
            RelationshipStage::AncestorDescendant,
        ] {
            assert!(is_kin_stage(stage));
            assert_eq!(stage.next_positive(), None, "{stage:?} must not advance");
            assert_eq!(stage.next_negative(), None, "{stage:?} must not regress");
            assert_eq!(min_interactions_for_stage(stage), 0);
        }
    }

    #[test]
    fn ancestor_descendant_has_kin_tables() {
        let s = RelationshipStage::AncestorDescendant;
        assert!(trust_threshold_for_stage(s) > Fixed::ZERO);
        assert!(affection_threshold_for_stage(s) > Fixed::ZERO);
        assert!(obligation_multiplier_for_stage(s) > Fixed::ZERO);
        // Grandparent ties carry the cousin-level kinship coefficient (0.25)
        // via the relationship-v2 derivation — close family, more distant
        // than parent-child (0.8) or sibling (0.5).
        let mut rv2 = super::super::relationship_v2::RelationshipV2::new(
            mindstrata_core::id::AgentId::new(0),
            mindstrata_core::id::AgentId::new(1),
        );
        rv2.stage = s;
        assert_eq!(rv2.derive_kinship_coefficient(), Fixed::from_f64(0.25));
        assert_eq!(rv2.derive_role_expectation(), super::super::relationship_v2::RoleExpectation::Caregiver);
    }
}
