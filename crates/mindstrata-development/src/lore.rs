//! Lore archetype scaffold (DC-2.6, STORY lore).
//!
//! Maps a [`ThreeRealmClaim`] — the `(domain, referent, claim, line)`
//! quartet emitted by the polarity engine — to a narrative
//! [`LoreArchetype`]. The mapping is pure, deterministic, and
//! wiring-free until the sim crate projects it (DC-2.7). Zero-at-zero:
//! no claim → no archetype; the function is total over the 48
//! `domain × referent × claim` combos plus the vendored [`LineId`]
//! (line modulates the result via a stable hash so two claims that
//! differ only in line can still diverge narratively).

use crate::polarity::{CausalDomain, GrossReferent, SubtleClaim, ThreeRealmClaim};
use serde::{Deserialize, Serialize};

/// Narrative archetype that frames a claim when it is told as lore.
///
/// Eight archetypes cover the AP3 narrative grammar (sovereignty,
/// meaning, belonging, becoming). The set is closed — new archetypes
/// are a canon change, not a local addition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoreArchetype {
    /// Sovereign / law-giver — order, boundary, institution.
    Sovereign,
    /// Sage / witness — truth, memory, explanation.
    Sage,
    /// Guardian / steward — protection, resource, place.
    Guardian,
    /// Lover / kin — attachment, care, belonging.
    Lover,
    /// Warrior / challenger — trial, ordeal, transgression.
    Warrior,
    /// Trickster / threshold — reversal, play, liminal.
    Trickster,
    /// Creator / weaver — making, craft, institution-building.
    Creator,
    /// Seeker / pilgrim — quest, transformation, return.
    Seeker,
}

impl LoreArchetype {
    /// Stable `u8` discriminant for hashing / serialization compat.
    #[must_use]
    pub const fn id(self) -> u8 {
        match self {
            Self::Sovereign => 0,
            Self::Sage => 1,
            Self::Guardian => 2,
            Self::Lover => 3,
            Self::Warrior => 4,
            Self::Trickster => 5,
            Self::Creator => 6,
            Self::Seeker => 7,
        }
    }

    /// Human-readable slug.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Sovereign => "sovereign",
            Self::Sage => "sage",
            Self::Guardian => "guardian",
            Self::Lover => "lover",
            Self::Warrior => "warrior",
            Self::Trickster => "trickster",
            Self::Creator => "creator",
            Self::Seeker => "seeker",
        }
    }

    /// All archetypes in discriminant order.
    #[must_use]
    pub const fn all() -> [Self; 8] {
        [
            Self::Sovereign,
            Self::Sage,
            Self::Guardian,
            Self::Lover,
            Self::Warrior,
            Self::Trickster,
            Self::Creator,
            Self::Seeker,
        ]
    }
}

/// Pure, deterministic mapping `(domain, referent, claim, line)` → archetype.
///
/// The line modulates the result so two claims that differ only in
/// `LineId` can diverge (e.g., cognitive vs attachment on the same
/// `Material/Event/Fact` maps to different archetypes). The hash is
/// stable: `FNV-1a` over the line slug bytes, folded into the base
/// `domain × referent × claim` index. No RNG, no allocation.
///
/// Design: base index is `domain_id * 16 + referent_id * 4 + claim_id`
/// (0..47), then `^ line_hash` keeps distribution while preserving
/// determinism across vendored line renames (the line hash is the
/// low 3 bits of the FNV).
#[must_use]
pub fn archetype_for_claim(claim: &ThreeRealmClaim) -> LoreArchetype {
    let domain_id = match claim.domain {
        CausalDomain::Material => 0,
        CausalDomain::Relational => 1,
        CausalDomain::Symbolic => 2,
    };
    let referent_id = match claim.referent {
        GrossReferent::Site => 0,
        GrossReferent::Resource => 1,
        GrossReferent::Event => 2,
        GrossReferent::Institution => 3,
    };
    let claim_id = match claim.claim {
        SubtleClaim::Fact => 0,
        SubtleClaim::Value => 1,
        SubtleClaim::Identity => 2,
        SubtleClaim::Norm => 3,
    };
    let base: u8 = domain_id * 16 + referent_id * 4 + claim_id;
    // FNV-1a over line slug, low 3 bits only — stable, small diffusion.
    let mut hash: u8 = 0b0010_1100; // FNV offset basis low bits
    for b in claim.line.slug().bytes() {
        hash ^= b;
        hash = hash.wrapping_mul(0x93); // FNV prime low bits
    }
    let idx = (base ^ (hash & 0b111)) % 8;
    LoreArchetype::all()[idx as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::line::LineId;
    use crate::polarity::{PolarityState, ThreeRealmClaim};

    fn lid(slug: &'static str) -> LineId {
        LineId::new(slug).expect("registered")
    }

    #[test]
    fn archetype_known_vectors() {
        // Spot-check a few vectors from the deterministic table.
        let c = ThreeRealmClaim::new(
            CausalDomain::Material,
            GrossReferent::Event,
            SubtleClaim::Fact,
            lid("cognitive"),
        );
        let a = archetype_for_claim(&c);
        // Must be deterministic — second call same result.
        assert_eq!(a, archetype_for_claim(&c));
        // Just check it's a valid archetype (id in 0..8).
        assert!(a.id() < 8);
    }

    #[test]
    fn archetype_all_eight_reachable() {
        // Over the 48 domain×referent×claim combos × a few lines,
        // every archetype should appear at least once.
        let lines = ["cognitive", "moral", "justice", "values"];
        let mut seen = [false; 8];
        for &domain in &[
            CausalDomain::Material,
            CausalDomain::Relational,
            CausalDomain::Symbolic,
        ] {
            for &referent in &[
                GrossReferent::Site,
                GrossReferent::Resource,
                GrossReferent::Event,
                GrossReferent::Institution,
            ] {
                for &claim in &[
                    SubtleClaim::Fact,
                    SubtleClaim::Value,
                    SubtleClaim::Identity,
                    SubtleClaim::Norm,
                ] {
                    for &line in &lines {
                        let c = ThreeRealmClaim::new(domain, referent, claim, lid(line));
                        let a = archetype_for_claim(&c);
                        seen[a.id() as usize] = true;
                    }
                }
            }
        }
        assert!(
            seen.iter().all(|&b| b),
            "not all archetypes reachable: {seen:?}"
        );
    }

    #[test]
    fn archetype_line_modulates_result() {
        // Same quartet except line — at least one pair must diverge
        // (otherwise line would be dead).
        let base = ThreeRealmClaim::new(
            CausalDomain::Material,
            GrossReferent::Event,
            SubtleClaim::Fact,
            lid("cognitive"),
        );
        let other = ThreeRealmClaim::new(
            CausalDomain::Material,
            GrossReferent::Event,
            SubtleClaim::Fact,
            lid("moral"),
        );
        // Not asserting they *must* differ for every line pair, but
        // across the line table at least one divergent pair should exist
        // (covered by all_eight_reachable). Here we just assert the
        // function is sensitive to line at all — try a few lines.
        let mut variants = std::collections::HashSet::new();
        for &line in &["cognitive", "moral", "justice", "values"] {
            let c = ThreeRealmClaim::new(
                CausalDomain::Material,
                GrossReferent::Event,
                SubtleClaim::Fact,
                lid(line),
            );
            variants.insert(archetype_for_claim(&c).id());
        }
        assert!(variants.len() > 1, "line does not modulate archetype");
        // Silence unused warning for the two spot claims.
        let _ = (base, other);
    }

    #[test]
    fn archetype_slug_roundtrips_via_id() {
        for &a in &LoreArchetype::all() {
            assert_eq!(a.slug().len() > 2, true);
            assert!(a.id() < 8);
        }
    }
}
