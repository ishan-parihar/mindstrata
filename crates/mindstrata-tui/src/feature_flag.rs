//! Feature flags — CLIENT 23 flag integration (2026-08-31).
//!
//! Deterministic, pure, no sim coupling. STORY observability gates behind
//! `Feature::Annals` / `Feature::Lore`.

/// Feature gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Feature {
    /// Annals trace (IC-2).
    Annals,
    /// Lore dossier (DC-2.6).
    Lore,
}

/// Whether `feature` is enabled in `enabled` set.
///
/// ```
/// use mindstrata_tui::feature_flag::{Feature, is_enabled};
/// let flags = vec![Feature::Annals];
/// assert!(is_enabled(Feature::Annals, &flags));
/// assert!(!is_enabled(Feature::Lore, &flags));
/// assert_eq!(is_enabled(Feature::Annals, &flags), is_enabled(Feature::Annals, &flags)); // deterministic
/// ```
pub fn is_enabled(feature: Feature, enabled: &[Feature]) -> bool {
    enabled.contains(&feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annals_enabled_lore_disabled() {
        let flags = vec![Feature::Annals];
        assert!(is_enabled(Feature::Annals, &flags));
        assert!(!is_enabled(Feature::Lore, &flags));
    }

    #[test]
    fn empty_disables_all() {
        assert!(!is_enabled(Feature::Annals, &[]));
        assert!(!is_enabled(Feature::Lore, &[]));
    }

    #[test]
    fn both_enabled_when_set() {
        let flags = vec![Feature::Annals, Feature::Lore];
        assert!(is_enabled(Feature::Annals, &flags));
        assert!(is_enabled(Feature::Lore, &flags));
    }
}
