//! Person entity and its component bundles.

mod body;
mod mind;
mod psyche;
mod social_self;

pub use body::{BodyState, NeedState};
pub use mind::{
    AgentIdentity, Belief, CognitiveState, DerivedMentalState, EvidenceSource, Goal, GoalKind,
    GoalSource, IdentityKind, IdentityState, Intention, MentalStateInput, MoralValues,
};
pub use psyche::{
    Affect, DiscreteEmotions, Personality, PlasticitySignals, Temperament, TraitConstitution,
    CORE_TRAIT_PLASTICITY_RATE, TEMPERAMENT_PLASTICITY_RATE,
};
pub use social_self::{Relationship, RelationshipKind, StatusState};

/// Founder given names (Iteration 245): shared by `populate` and the
/// birth paths so every villager — founder or newborn — draws from one
/// name pool. Newborns append the family surname; founders are single-
/// token roots.
pub const FIRST_NAMES: [&str; 24] = [
    "Anna", "Bran", "Cara", "Dane", "Elise", "Finn", "Greta", "Hans", "Ines", "Jorik", "Kira",
    "Lars", "Mira", "Nils", "Opal", "Poul", "Quinn", "Rosa", "Sven", "Tova", "Ulf", "Vera", "Wulf",
    "Xena",
];

/// Iteration 245 (Arc A heredity): derive a child's full name from a
/// parent's name and a given name. The surname is the parent's family
/// token — everything after their first space — so second-generation
/// children keep the founding line's surname. A founder parent (single
/// token) lends their own given name as the founding family name, which
/// reads as a patronymic lineage from generation one. Replacement
/// newborns pass the deceased's name for household continuity.
///
/// ```
/// use mindstrata_sim::person::inherit_surname;
/// assert_eq!(inherit_surname("Mira Lars", "Tova"), "Tova Lars");
/// assert_eq!(inherit_surname("Anna", "Bran"), "Bran Anna");
/// assert_eq!(inherit_surname("Mira Lars", "Tova") , inherit_surname("Kira Lars", "Tova"));
/// ```
pub fn inherit_surname(parent_name: &str, given_name: &str) -> String {
    let surname = parent_name.split_once(' ').map_or(parent_name, |(_, s)| s);
    format!("{given_name} {surname}")
}

#[cfg(test)]
mod surname_tests {
    use super::inherit_surname;

    #[test]
    fn surnames_form_lineages() {
        assert_eq!(inherit_surname("Mira Lars", "Tova"), "Tova Lars");
        assert_eq!(inherit_surname("Anna", "Bran"), "Bran Anna");
        // Any Lars-parent child carries the Lars line.
        assert_eq!(inherit_surname("Kira Lars", "Ulf"), "Ulf Lars");
        assert_eq!(
            inherit_surname("Mira Lars", "Tova"),
            inherit_surname("Kira Lars", "Tova")
        );
        // Third generation keeps the founding surname through the space rule.
        let gen2 = inherit_surname("Bran Anna", "Cara");
        assert_eq!(gen2, "Cara Anna");
    }
}
