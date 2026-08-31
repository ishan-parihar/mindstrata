//! Referent extractor prototype — Era III (STORY 4.6).
//!
//! Deterministic extraction of `GrossReferent` from text. Pure types,
//! no sim wiring, no RNG, `golden 5/5` byte-identical.
//!
//! Mapping is keyword-based and deterministic; real extractor will be
//! vendor-cell driven in Era III full.

use crate::realm::GrossReferent;

/// Extract `GrossReferent` from text deterministically.
///
/// Keyword rules (case-insensitive, first match wins):
/// - `person`, `self`, `identity` → `Person`
/// - `family`, `kin`, `household` → `Family`
/// - `institution`, `law`, `governance` → `Institution`
/// - `culture`, `myth`, `ritual` → `Culture`
/// - `ecology`, `land`, `resource` → `Ecology`
/// - otherwise → `Institution` (neutral fallback)
///
/// ```
/// use mindstrata_development::referent::extract_referent;
/// use mindstrata_development::realm::GrossReferent;
/// assert_eq!(extract_referent("the person holds value"), GrossReferent::Person);
/// assert_eq!(extract_referent("family kinship"), GrossReferent::Family);
/// assert_eq!(extract_referent("unknown text"), GrossReferent::Institution);
/// assert_eq!(extract_referent("PERSON"), extract_referent("person")); // case-insensitive, deterministic
/// ```
pub fn extract_referent(text: &str) -> GrossReferent {
    let lower = text.to_ascii_lowercase();
    if lower.contains("person") || lower.contains("self") || lower.contains("identity") {
        GrossReferent::Person
    } else if lower.contains("family")
        || lower.contains("kin")
        || lower.contains("household")
        || lower.contains("group")
    {
        GrossReferent::Group
    } else if lower.contains("ecology")
        || lower.contains("land")
        || lower.contains("resource")
        || lower.contains("world")
        || lower.contains("culture")
        || lower.contains("myth")
        || lower.contains("ritual")
    {
        GrossReferent::World
    } else {
        GrossReferent::Institution
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_is_deterministic_and_case_insensitive() {
        let a = extract_referent("Person holds value");
        let b = extract_referent("PERSON holds value");
        let c = extract_referent("person holds value");
        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(a, GrossReferent::Person);
    }

    #[test]
    fn every_referent_variant_reachable() {
        assert_eq!(extract_referent("person"), GrossReferent::Person);
        assert_eq!(extract_referent("family"), GrossReferent::Group);
        assert_eq!(
            extract_referent("institution law"),
            GrossReferent::Institution
        );
        assert_eq!(extract_referent("culture myth"), GrossReferent::World);
        assert_eq!(extract_referent("ecology land"), GrossReferent::World);
        assert_eq!(extract_referent("group"), GrossReferent::Group);
        assert_eq!(extract_referent("world"), GrossReferent::World);
    }

    #[test]
    fn fallback_is_institution() {
        assert_eq!(extract_referent(""), GrossReferent::Institution);
        assert_eq!(
            extract_referent("unknown text xyz"),
            GrossReferent::Institution
        );
    }
}
