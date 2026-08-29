//! Polarity type engine over belief claims (STORY 8-9, FR-030/FR-032).
//!
//! Three-realm grammar: every generated `Subtle` item (meme, norm-proposal,
//! ritual form) type-checks as `Domain(Causal) × Referent(Gross) × Claim(Subtle)`
//! and carries a `LineId` signature. This module is the **read-only** polarity
//! observer: it inspects belief claims without mutating sim state, so it is
//! wiring-free until the `pass_development` consumes its judgments behind a
//! feature flag. Zero-at-zero law: empty inputs produce no judgment.
//!
//! Reconciliation graph (Era III): `Undiscovered → ActiveTension → Integrated`
//! via `reconcile`/`refute` transitions emitted by `belief_update`/`gossip`.
//! This engine just models the transition — the passes will emit it.

use crate::line::LineId;
use serde::{Deserialize, Serialize};

/// Causal explanatory domain of a claim (the `why` layer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CausalDomain {
    /// Material/physical causation (sites, resources).
    Material,
    /// Relational/social causation (institutions, kinship).
    Relational,
    /// Symbolic/cultural causation (meaning, values).
    Symbolic,
}

/// Gross empirical referent the claim cites (the `what` is observed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GrossReferent {
    /// A place or territory cell.
    Site,
    /// A resource stock or flow.
    Resource,
    /// An event (birth, death, verdict, ritual).
    Event,
    /// An institution or faction.
    Institution,
}

/// Subtle interpretive claim layered over the referent (the `so what`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubtleClaim {
    /// Factual assertion.
    Fact,
    /// Value judgment.
    Value,
    /// Identity / belonging assertion.
    Identity,
    /// Norm / prescription.
    Norm,
}

/// Polarity graph state of a claim in the belief network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PolarityState {
    /// Never encountered by this agent.
    Undiscovered,
    /// Contradictory claims co-exist — the moral-panic precursor.
    ActiveTension,
    /// Claims reconciled into a higher synthesis (transcend-and-include).
    Integrated,
}

/// A three-realm belief claim (FR-030) — the unit the polarity engine observes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ThreeRealmClaim {
    /// Causal domain the claim appeals to.
    pub domain: CausalDomain,
    /// Gross referent the claim cites as evidence.
    pub referent: GrossReferent,
    /// Subtle claim asserted.
    pub claim: SubtleClaim,
    /// Which line this claim resonates with (individual-line signature).
    pub line: LineId,
    /// Current polarity state of this claim for the observing agent.
    pub polarity: PolarityState,
}

impl ThreeRealmClaim {
    /// Construct a claim; `polarity` starts `Undiscovered` for new items.
    #[must_use]
    pub const fn new(
        domain: CausalDomain,
        referent: GrossReferent,
        claim: SubtleClaim,
        line: LineId,
    ) -> Self {
        Self {
            domain,
            referent,
            claim,
            line,
            polarity: PolarityState::Undiscovered,
        }
    }
}

/// Pure read-only polarity observation (FR-032): reconcile two claims that
/// share a gross referent into a higher synthesis, if their domains differ
/// (the classic polarity — `Material vs Symbolic` on the same `Event`).
///
/// Returns `Some(synthesized)` when reconciliation is type-legal, `None`
/// when claims are unrelated or already `Integrated`. No mutation, no RNG,
/// deterministic — callers decide whether to emit the transition.
#[must_use]
pub fn reconcile_claims(a: &ThreeRealmClaim, b: &ThreeRealmClaim) -> Option<ThreeRealmClaim> {
    // Zero-at-zero: undiscovered claims never reconcile.
    if a.polarity == PolarityState::Undiscovered || b.polarity == PolarityState::Undiscovered {
        return None;
    }
    // Must cite the same gross referent and resonate the same line — otherwise
    // they are not a polarity pair on the same issue.
    if a.referent != b.referent || a.line != b.line {
        return None;
    }
    // Already integrated — no further synthesis.
    if a.polarity == PolarityState::Integrated || b.polarity == PolarityState::Integrated {
        return None;
    }
    // Opposite causal domains that can be synthesized (Material↔Symbolic,
    // Relational is the mediating domain — any pair with differing domains
    // qualifies; identical domains are just repetition, not polarity).
    if a.domain == b.domain {
        return None;
    }
    // Synthesize: take the more encompassing subtle claim (Norm > Value > Identity > Fact)
    // and mark Integrated. This is the read-only judgment; the caller would
    // advance the belief graph to Integrated on this output.
    let synthesized_claim = match (a.claim, b.claim) {
        (SubtleClaim::Norm, _) | (_, SubtleClaim::Norm) => SubtleClaim::Norm,
        (SubtleClaim::Value, _) | (_, SubtleClaim::Value) => SubtleClaim::Value,
        (SubtleClaim::Identity, _) | (_, SubtleClaim::Identity) => SubtleClaim::Identity,
        _ => SubtleClaim::Fact,
    };
    // Domain of the synthesis is Symbolic (the transcending domain) — the
    // vocabulary's `Symbolic` is the `Causal` layer's synthesis slot.
    Some(ThreeRealmClaim {
        domain: CausalDomain::Symbolic,
        referent: a.referent,
        claim: synthesized_claim,
        line: a.line,
        polarity: PolarityState::Integrated,
    })
}

/// Whether two claims are in `ActiveTension` (co-exist without synthesis).
#[must_use]
pub fn is_active_tension(a: &ThreeRealmClaim, b: &ThreeRealmClaim) -> bool {
    a.referent == b.referent
        && a.line == b.line
        && a.domain != b.domain
        && a.polarity == PolarityState::ActiveTension
        && b.polarity == PolarityState::ActiveTension
}

/// Pure projection: which `(domain, referent, claim, line)` quartet a
/// catalyst kind produces. FR-031 data-path wire — the sim crate
/// applies this per `CatalystEvent` and appends the resulting claim
/// to the agent's `polarity_claims` list. Deterministic and pure: no
/// RNG, no state.
///
/// Mapping (DC-1, kept minimal — STORY 9-10 expands with agent-line
/// resonance):
/// - `Threat`    → Material / Event / Fact / safety
/// - `Bond`      → Relational / Institution / Value / attachment
/// - `Transgression` → Symbolic / Event / Norm / justice
/// - `Grief`     → Material / Event / Identity / safety
///
/// All emitted claims start in `Undiscovered` polarity — the agent
/// has encountered the catalyst but has not yet formed a polarized
/// position. Subsequent exposures (same line × same referent) can
/// trigger reconciliation.
#[must_use]
pub fn project_catalyst(kind: crate::catalyst::CatalystKind) -> ThreeRealmClaim {
    use crate::catalyst::CatalystKind;
    let (domain, referent, claim, line_slug) = match kind {
        CatalystKind::Threat => (
            CausalDomain::Material,
            GrossReferent::Event,
            SubtleClaim::Fact,
            "safety",
        ),
        CatalystKind::Bond => (
            CausalDomain::Relational,
            GrossReferent::Institution,
            SubtleClaim::Value,
            "attachment",
        ),
        CatalystKind::Transgression => (
            CausalDomain::Symbolic,
            GrossReferent::Event,
            SubtleClaim::Norm,
            "justice",
        ),
        CatalystKind::Grief => (
            CausalDomain::Material,
            GrossReferent::Event,
            SubtleClaim::Identity,
            "safety",
        ),
    };
    let line = LineId::new(line_slug).unwrap_or_else(|| {
        // safety/attachment/justice are all in the vendored registry;
        // if a future addition drops one, fall back to a known stable line.
        LineId::new("cognitive").expect("cognitive line is in registry")
    });
    ThreeRealmClaim::new(domain, referent, claim, line)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lid(slug: &'static str) -> LineId {
        LineId::new(slug).expect("registered")
    }

    fn claim(
        domain: CausalDomain,
        referent: GrossReferent,
        subtle: SubtleClaim,
        line: &'static str,
        polarity: PolarityState,
    ) -> ThreeRealmClaim {
        ThreeRealmClaim {
            domain,
            referent,
            claim: subtle,
            line: lid(line),
            polarity,
        }
    }

    #[test]
    fn three_realm_typing_is_structurally_enforced() {
        let c = ThreeRealmClaim::new(
            CausalDomain::Material,
            GrossReferent::Event,
            SubtleClaim::Fact,
            lid("cognitive"),
        );
        assert_eq!(c.domain, CausalDomain::Material);
        assert_eq!(c.polarity, PolarityState::Undiscovered);
    }

    #[test]
    fn reconcile_requires_shared_referent_and_line_and_active_tension() {
        let a = claim(
            CausalDomain::Material,
            GrossReferent::Event,
            SubtleClaim::Fact,
            "cognitive",
            PolarityState::ActiveTension,
        );
        let b = claim(
            CausalDomain::Symbolic,
            GrossReferent::Event,
            SubtleClaim::Value,
            "cognitive",
            PolarityState::ActiveTension,
        );
        let syn = reconcile_claims(&a, &b).expect("polarity pair should reconcile");
        assert_eq!(syn.polarity, PolarityState::Integrated);
        assert_eq!(syn.domain, CausalDomain::Symbolic);
        assert_eq!(syn.claim, SubtleClaim::Value);
    }

    #[test]
    fn reconcile_rejects_identical_domains_and_undiscovered() {
        let a = claim(
            CausalDomain::Material,
            GrossReferent::Event,
            SubtleClaim::Fact,
            "cognitive",
            PolarityState::ActiveTension,
        );
        let b_same_domain = claim(
            CausalDomain::Material,
            GrossReferent::Event,
            SubtleClaim::Fact,
            "cognitive",
            PolarityState::ActiveTension,
        );
        assert!(reconcile_claims(&a, &b_same_domain).is_none());
        let undiscovered = claim(
            CausalDomain::Symbolic,
            GrossReferent::Event,
            SubtleClaim::Fact,
            "cognitive",
            PolarityState::Undiscovered,
        );
        assert!(reconcile_claims(&a, &undiscovered).is_none());
    }

    #[test]
    fn reconcile_rejects_different_referents_or_lines() {
        let a = claim(
            CausalDomain::Material,
            GrossReferent::Event,
            SubtleClaim::Fact,
            "cognitive",
            PolarityState::ActiveTension,
        );
        let b_diff_ref = claim(
            CausalDomain::Symbolic,
            GrossReferent::Site,
            SubtleClaim::Fact,
            "cognitive",
            PolarityState::ActiveTension,
        );
        assert!(reconcile_claims(&a, &b_diff_ref).is_none());
        let b_diff_line = claim(
            CausalDomain::Symbolic,
            GrossReferent::Event,
            SubtleClaim::Fact,
            "values",
            PolarityState::ActiveTension,
        );
        assert!(reconcile_claims(&a, &b_diff_line).is_none());
    }

    #[test]
    fn reconcile_is_read_only_and_deterministic() {
        let a = claim(
            CausalDomain::Material,
            GrossReferent::Institution,
            SubtleClaim::Norm,
            "values",
            PolarityState::ActiveTension,
        );
        let b = claim(
            CausalDomain::Relational,
            GrossReferent::Institution,
            SubtleClaim::Fact,
            "values",
            PolarityState::ActiveTension,
        );
        let first = reconcile_claims(&a, &b);
        let second = reconcile_claims(&a, &b);
        assert_eq!(first, second);
        // Inputs unchanged — read-only
        assert_eq!(a.polarity, PolarityState::ActiveTension);
    }

    #[test]
    fn active_tension_predicate_matches_reconcile_preconditions() {
        let a = claim(
            CausalDomain::Material,
            GrossReferent::Event,
            SubtleClaim::Fact,
            "cognitive",
            PolarityState::ActiveTension,
        );
        let b = claim(
            CausalDomain::Symbolic,
            GrossReferent::Event,
            SubtleClaim::Fact,
            "cognitive",
            PolarityState::ActiveTension,
        );
        assert!(is_active_tension(&a, &b));
        assert!(reconcile_claims(&a, &b).is_some());
    }

    #[test]
    fn integrated_claims_do_not_reconcile_further() {
        let a = claim(
            CausalDomain::Symbolic,
            GrossReferent::Event,
            SubtleClaim::Fact,
            "cognitive",
            PolarityState::Integrated,
        );
        let b = claim(
            CausalDomain::Material,
            GrossReferent::Event,
            SubtleClaim::Fact,
            "cognitive",
            PolarityState::ActiveTension,
        );
        assert!(reconcile_claims(&a, &b).is_none());
    }
}
