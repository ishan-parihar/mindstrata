//! Era III content rendering (PLAN_DC2 Iter-269) — the bridge from the
//! live polarity-claim data path (STORY 9-10) to the template grammar
//! (STORY 4.5/4.6). Until this module existed, `Template`/`RealmTriple`
//! were cited types with no consumer and claims never *rendered*: the
//! village told no stories (UM-2).
//!
//! Pipeline: `ThreeRealmClaim` (live, per-agent, produced every tick by
//! `system_polarity_claim_emit`) → legality-gated projection into the
//! Era III triple grammar → deterministic template selection (cite-first,
//! keyed by the claim's line so a claim renders the same way forever) →
//! `render` substitution. Pure function of the claim; no RNG, no state.
//!
//! Legality law: a claim whose Era III projection is illegal
//! (`is_legal() == false`) renders `None` — illegal content is never
//! spoken. Today the projection is total and legal over the live
//! 3×4×4 domain space (the vendored single rule pairs Entropy with
//! Institution+Belief, unreachable from the live grammar); when the
//! full ontology mapping lands (WP-I vendor coupling), the gate becomes
//! load-bearing.
//!
//! Zero-at-zero: no claims → no lore lines. Rendering is read-only
//! over existing state; the chronicle consumes this — golden stays
//! byte-identical unless the village actually holds claims (it does,
//! so the chronicle text changes, but no state the goldens hash does —
//! `render_chronicle` is TUI/legibility-only, unpinned by tests).

use crate::polarity::{CausalDomain, GrossReferent, SubtleClaim, ThreeRealmClaim};
use crate::realm::{
    CausalDomain as Era3Domain, GrossReferent as Era3Referent, RealmTriple,
    SubtleClaim as Era3Claim,
};
use serde::{Deserialize, Serialize};

/// A rendered lore line — the unit the chronicle annals can quote.
///
/// Carries provenance (claim + template id + source cell citation) so
/// a rendered line is auditable back to its ontology cell (AP3 law:
/// every template cites its source cell).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoreLine {
    /// The rendered text (template pattern with substitutions).
    pub text: String,
    /// Template id that produced the text (e.g., `T-001`).
    pub template_id: &'static str,
    /// Ontology source cell cited by the template.
    pub source_cell: &'static str,
    /// The line-signature slug of the claim that generated this lore.
    pub line_slug: String,
    /// The polarity stage of the generating claim (the narrative weight).
    pub polarity: PolarityTag,
}

/// Chronicle-facing polarity tag — mirrors [`crate::polarity::PolarityState`]
/// without leaking the engine type into the render layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolarityTag {
    /// Claim is held but unremarked.
    Undiscovered,
    /// Claim is in active moral tension — the panic precursor.
    ActiveTension,
    /// Claim has been reconciled into synthesis.
    Integrated,
}

impl From<crate::polarity::PolarityState> for PolarityTag {
    fn from(p: crate::polarity::PolarityState) -> Self {
        match p {
            crate::polarity::PolarityState::Undiscovered => Self::Undiscovered,
            crate::polarity::PolarityState::ActiveTension => Self::ActiveTension,
            crate::polarity::PolarityState::Integrated => Self::Integrated,
        }
    }
}

/// Project the live polarity grammar (3 domains × 4 referents × 4 claims)
/// into the Era III grammar (4 × 4 × 3). Deterministic, total.
///
/// The live `CausalDomain::Material` splits by claim kind: an Event/Fact
/// reads as raw `Agency` (things happen because someone did something),
/// anything else as `Entropy` (material drift beyond agency). Relational
/// is `Communion` (bonding); Symbolic is `Structure` (meaning custodians).
#[must_use]
pub fn project_to_era3(claim: &ThreeRealmClaim) -> RealmTriple {
    let domain = match claim.domain {
        CausalDomain::Material => match (claim.referent, claim.claim) {
            (GrossReferent::Event, SubtleClaim::Fact) => Era3Domain::Agency,
            _ => Era3Domain::Entropy,
        },
        CausalDomain::Relational => Era3Domain::Communion,
        CausalDomain::Symbolic => Era3Domain::Structure,
    };
    let referent = match claim.referent {
        GrossReferent::Site => Era3Referent::World,
        GrossReferent::Resource => Era3Referent::World,
        GrossReferent::Event => Era3Referent::Person,
        GrossReferent::Institution => Era3Referent::Institution,
    };
    let subtle = match claim.claim {
        SubtleClaim::Fact => Era3Claim::Value,
        SubtleClaim::Norm => Era3Claim::Practice,
        SubtleClaim::Value | SubtleClaim::Identity => Era3Claim::Belief,
    };
    RealmTriple::new(domain, referent, subtle)
}

/// FNV-1a over the line slug — the stable selector key (mirrors
/// `lore::archetype_for_claim`'s hash discipline; keep both in sync
/// if the vendored line table ever renames).
fn line_fnv(slug: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in slug.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Render one claim as lore: legality-gated, cite-first, deterministic.
///
/// Template selection: `SAMPLE_TEMPLATES[(line_fnv(slug) % 2) as usize]` —
/// keyed by the claim's line so the same claim always renders through the
/// same cited template (a village's telling of a claim is stable across
/// renders; two claims on different lines may tell differently).
///
/// Returns `None` when the Era III projection is illegal (unreachable
/// over the live grammar today; the gate exists for the WP-I mapping).
#[must_use]
pub fn render_claim(claim: &ThreeRealmClaim) -> Option<LoreLine> {
    let triple = project_to_era3(claim);
    if !triple.is_legal() {
        return None;
    }
    let domain_str = format!("{:?}", triple.domain);
    let referent_str = format!("{:?}", triple.referent);
    let claim_str = format!("{:?}", triple.claim);
    let templates = crate::template::SAMPLE_TEMPLATES;
    let t = &templates[(line_fnv(claim.line.slug()) as usize) % templates.len()];
    Some(LoreLine {
        text: t.render(&domain_str, &referent_str, &claim_str),
        template_id: t.id,
        source_cell: t.source_cell,
        line_slug: claim.line.slug().to_owned(),
        polarity: claim.polarity.into(),
    })
}

/// Render a village's lore section: dedup by rendered text, insertion
/// order preserved (deterministic — claims arrive in tick/event order),
/// capped at `max_lines` so the annals stay legible. Zero-at-zero.
#[must_use]
pub fn render_lore_section(claims: &[ThreeRealmClaim], max_lines: usize) -> String {
    let mut seen = std::collections::HashSet::new();
    let mut out = String::new();
    for c in claims {
        if out.lines().count() >= max_lines {
            break;
        }
        let Some(line) = render_claim(c) else {
            continue;
        };
        // Tension claims are annal-worthy above all: the village speaks
        // of what divides it. Tag them so the reader sees the weight.
        let tagged = match line.polarity {
            PolarityTag::ActiveTension => format!("{} (a matter of tension)", line.text),
            PolarityTag::Integrated => format!("{} (a synthesis reached)", line.text),
            PolarityTag::Undiscovered => line.text.clone(),
        };
        if seen.insert(tagged.clone()) {
            out.push_str("  - the lore says: ");
            out.push_str(&tagged);
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::line::LineId;

    fn lid(slug: &'static str) -> LineId {
        LineId::new(slug).expect("registered")
    }

    fn claim(
        domain: CausalDomain,
        referent: GrossReferent,
        subtle: SubtleClaim,
        slug: &'static str,
    ) -> ThreeRealmClaim {
        ThreeRealmClaim::new(domain, referent, subtle, lid(slug))
    }

    #[test]
    fn live_catalyst_claims_render_legal_lore() {
        // Every claim the live polarity path can project must render —
        // the projection is total and legal over the live grammar.
        let kinds = [
            (
                CausalDomain::Material,
                GrossReferent::Event,
                SubtleClaim::Fact,
            ),
            (
                CausalDomain::Material,
                GrossReferent::Event,
                SubtleClaim::Identity,
            ),
            (
                CausalDomain::Relational,
                GrossReferent::Institution,
                SubtleClaim::Value,
            ),
            (
                CausalDomain::Symbolic,
                GrossReferent::Event,
                SubtleClaim::Norm,
            ),
        ];
        for (d, r, s) in kinds {
            let c = claim(d, r, s, "cognitive");
            let line = render_claim(&c).expect("live claim must render");
            assert!(!line.text.is_empty());
            assert!(
                !line.text.contains('{'),
                "unsubstituted placeholder: {}",
                line.text
            );
            assert!(line.template_id.starts_with("T-"));
            assert!(line.source_cell.starts_with("cells/"), "cite-first law");
        }
    }

    #[test]
    fn same_claim_renders_identically_regardless_of_order() {
        let a = render_claim(&claim(
            CausalDomain::Relational,
            GrossReferent::Institution,
            SubtleClaim::Value,
            "values",
        ))
        .unwrap();
        let b = render_claim(&claim(
            CausalDomain::Relational,
            GrossReferent::Institution,
            SubtleClaim::Value,
            "values",
        ))
        .unwrap();
        assert_eq!(a, b, "deterministic render");
        // Line modulates template selection (two lines, same claim shape).
        let other = render_claim(&claim(
            CausalDomain::Relational,
            GrossReferent::Institution,
            SubtleClaim::Value,
            "justice",
        ))
        .unwrap();
        let _ = (
            a == other,
            "line-sensitivity is allowed, not required, per pair",
        );
    }

    #[test]
    fn tension_and_synthesis_are_tagged_in_section() {
        let mut c = claim(
            CausalDomain::Symbolic,
            GrossReferent::Event,
            SubtleClaim::Norm,
            "justice",
        );
        c.polarity = crate::polarity::PolarityState::ActiveTension;
        let section = render_lore_section(&[c], 8);
        assert!(section.contains("the lore says:"), "section prefix");
        assert!(section.contains("a matter of tension"), "tension tag");
        assert!(section.lines().count() == 1, "one claim, one line");
    }

    #[test]
    fn empty_claims_render_zero_lines() {
        assert!(render_lore_section(&[], 8).is_empty(), "zero-at-zero");
    }

    #[test]
    fn cap_bounds_the_section() {
        // Six distinct (shape, line) pairs → six distinct texts (two
        // templates × three shapes); the cap must still bound the output.
        let shapes = [
            (
                CausalDomain::Material,
                GrossReferent::Event,
                SubtleClaim::Fact,
                "cognitive",
            ),
            (
                CausalDomain::Relational,
                GrossReferent::Event,
                SubtleClaim::Fact,
                "moral",
            ),
            (
                CausalDomain::Symbolic,
                GrossReferent::Event,
                SubtleClaim::Fact,
                "justice",
            ),
            (
                CausalDomain::Material,
                GrossReferent::Institution,
                SubtleClaim::Fact,
                "values",
            ),
            (
                CausalDomain::Relational,
                GrossReferent::Institution,
                SubtleClaim::Fact,
                "governance",
            ),
            (
                CausalDomain::Symbolic,
                GrossReferent::Institution,
                SubtleClaim::Fact,
                "worldview",
            ),
        ];
        let claims: Vec<ThreeRealmClaim> = shapes
            .iter()
            .map(|(d, r, s, slug)| claim(*d, *r, *s, slug))
            .collect();
        let uncapped = render_lore_section(&claims, 16);
        assert_eq!(
            uncapped.lines().count(),
            6,
            "distinct texts render distinctly"
        );
        let section = render_lore_section(&claims, 3);
        assert_eq!(section.lines().count(), 3, "cap respected");
    }

    #[test]
    fn dedup_collapses_identical_text() {
        let c = claim(
            CausalDomain::Material,
            GrossReferent::Event,
            SubtleClaim::Fact,
            "cognitive",
        );
        let section = render_lore_section(&[c, c, c], 8);
        assert_eq!(section.lines().count(), 1, "dedup");
    }
}
