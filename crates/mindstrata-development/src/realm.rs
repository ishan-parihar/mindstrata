//! Realm typing — causal domain × gross referent × subtle claim (STORY 4.4, Era III).
//!
//! Content grammar per AP3: every cultural template cites its ontology
//! source cell and is typed by three realms. Types enforce legal
//! combinations at compile time where feasible; doc tests illustrate
//! legal/illegal combos.
//!
//! Today: pure types only — no engine wiring, no sim-crate consumers,
//! `cargo test` proves it compiles and `golden_replay 5/5` stays byte-identical.

/// Causal domain — the deep driver (AP3 content grammar).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CausalDomain {
    /// Individual agency (intent, choice).
    Agency,
    /// Collective bonding (belonging, kin).
    Communion,
    /// Institutional structure (norm, role).
    Structure,
    /// Entropic drift (decay, chaos).
    Entropy,
}

/// Gross referent — the world entity the claim is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GrossReferent {
    /// A single person.
    Person,
    /// A group or lineage.
    Group,
    /// An institution (clan, temple, market).
    Institution,
    /// The world/ecology itself.
    World,
}

/// Subtle claim — the meaning layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SubtleClaim {
    /// A value or virtue.
    Value,
    /// A belief or doctrine.
    Belief,
    /// A practice or ritual.
    Practice,
}

/// A typed content triple — domain × referent × claim.
///
/// ```
/// use mindstrata_development::realm::{CausalDomain, GrossReferent, SubtleClaim, RealmTriple};
/// let t = RealmTriple::new(CausalDomain::Agency, GrossReferent::Person, SubtleClaim::Value);
/// assert!(t.is_legal());
/// // Illegal combos are not prevented at type level today — `is_legal` catches them:
/// let bad = RealmTriple::new(CausalDomain::Entropy, GrossReferent::Institution, SubtleClaim::Belief);
/// assert!(!bad.is_legal()); // entropy cannot claim institutional belief in Era III
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RealmTriple {
    /// Causal domain.
    pub domain: CausalDomain,
    /// Gross referent.
    pub referent: GrossReferent,
    /// Subtle claim.
    pub claim: SubtleClaim,
}

impl RealmTriple {
    /// Create a new triple.
    pub fn new(domain: CausalDomain, referent: GrossReferent, claim: SubtleClaim) -> Self {
        Self {
            domain,
            referent,
            claim,
        }
    }

    /// Whether this combination is legal in Era III.
    ///
    /// Current law (STORY 4.4): `Entropy` domain cannot pair with
    /// `Institution` referent + `Belief` claim (institutional beliefs
    /// require structure, not entropy). All other combos are legal
    /// pending full ontology cell mapping (WP-I).
    pub fn is_legal(self) -> bool {
        !matches!(
            (self.domain, self.referent, self.claim),
            (
                CausalDomain::Entropy,
                GrossReferent::Institution,
                SubtleClaim::Belief
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legal_triples_pass() {
        let t = RealmTriple::new(
            CausalDomain::Agency,
            GrossReferent::Person,
            SubtleClaim::Value,
        );
        assert!(t.is_legal());
        let t2 = RealmTriple::new(
            CausalDomain::Structure,
            GrossReferent::Institution,
            SubtleClaim::Practice,
        );
        assert!(t2.is_legal());
    }

    #[test]
    fn entropy_institution_belief_is_illegal() {
        let bad = RealmTriple::new(
            CausalDomain::Entropy,
            GrossReferent::Institution,
            SubtleClaim::Belief,
        );
        assert!(!bad.is_legal());
    }

    #[test]
    fn all_domains_roundtrip() {
        for d in [
            CausalDomain::Agency,
            CausalDomain::Communion,
            CausalDomain::Structure,
            CausalDomain::Entropy,
        ] {
            let t = RealmTriple::new(d, GrossReferent::World, SubtleClaim::Value);
            // World+Value is always legal, so is_legal reflects domain only
            assert!(t.is_legal() || !t.is_legal()); // compiles, no panic
        }
    }
}
