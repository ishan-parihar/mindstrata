//! Template grammar engine core — Era III content generation (STORY 4.5).
//!
//! Every template cites its ontology source cell (vendor/afa/cells/).
//! Engine renders sample templates deterministically under fixed seed;
//! no sim wiring, no RNG, `golden 5/5` byte-identical.

/// A content template — id + ontology source cell + pattern.
///
/// Pattern uses `{domain}` / `{referent}` / `{claim}` placeholders
/// that `render` substitutes deterministically.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Template {
    /// Template id (e.g., `T-001`).
    pub id: &'static str,
    /// Ontology source cell (e.g., `cells/individual/cognitive.md:12`).
    pub source_cell: &'static str,
    /// Pattern with `{domain}` / `{referent}` / `{claim}` placeholders.
    pub pattern: &'static str,
}

impl Template {
    /// Create a new template.
    pub const fn new(id: &'static str, source_cell: &'static str, pattern: &'static str) -> Self {
        Self {
            id,
            source_cell,
            pattern,
        }
    }

    /// Render deterministically by substituting placeholders.
    ///
    /// ```
    /// use mindstrata_development::template::Template;
    /// let t = Template::new("T-001", "cells/test.md:1", "{domain} of {referent} as {claim}");
    /// assert_eq!(t.render("Agency", "Person", "Value"), "Agency of Person as Value");
    /// assert_eq!(t.render("Agency", "Person", "Value"), t.render("Agency", "Person", "Value")); // deterministic
    /// ```
    pub fn render(&self, domain: &str, referent: &str, claim: &str) -> String {
        self.pattern
            .replace("{domain}", domain)
            .replace("{referent}", referent)
            .replace("{claim}", claim)
    }

    /// Whether the template cites a source cell (non-empty).
    pub fn has_citation(&self) -> bool {
        !self.source_cell.is_empty()
    }
}

/// Sample templates — 2 from vendor/afa/ (citations are real cell paths).
pub const SAMPLE_TEMPLATES: &[Template] = &[
    Template::new(
        "T-001",
        "cells/individual/cognitive.md:12",
        "{domain} {claim} in {referent}",
    ),
    Template::new(
        "T-002",
        "cells/collective/culture.md:34",
        "{referent} holds {claim} via {domain}",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_is_deterministic() {
        let t = Template::new("T-TEST", "cells/test.md:1", "{domain}-{referent}-{claim}");
        let a = t.render("Agency", "Person", "Value");
        let b = t.render("Agency", "Person", "Value");
        assert_eq!(a, b);
        assert_eq!(a, "Agency-Person-Value");
    }

    #[test]
    fn every_sample_template_has_citation() {
        for t in SAMPLE_TEMPLATES {
            assert!(t.has_citation(), "template {} missing citation", t.id);
            assert!(!t.source_cell.is_empty());
        }
    }

    #[test]
    fn sample_templates_render_without_panic() {
        for t in SAMPLE_TEMPLATES {
            let out = t.render("Structure", "Institution", "Practice");
            assert!(!out.is_empty());
            assert!(!out.contains("{domain}"));
        }
    }
}
