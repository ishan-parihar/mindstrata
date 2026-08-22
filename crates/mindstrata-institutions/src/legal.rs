//! Legal system — §5 (AP2, Iteration 149).
//!
//! Courts, trials, and the judicial layer over the existing ownership and
//! enforcement machinery. The Council's "Arbitrate disputes" obligation has
//! existed as text since day one but was never implemented: a caught theft
//! produced a fine, an event, and a provenance record — but no case, no
//! verdict, no record of justice. This module closes that gap.
//!
//! - **Property**: `Site.owner` already exists (homes are assigned at
//!   populate, `AccessRight::OwnerOnly` gates some stocks). The legal layer
//!   treats an owned site's theft as strong evidence — property rights are
//!   what make a crime *provable*.
//! - **Courts**: the adjudication authority is the registry itself — it
//!   materializes on the first prosecuted violation (`established_tick`),
//!   mirroring how a council of elders convenes only when a dispute is
//!   brought before it. No new `InstitutionKind` is introduced, so the
//!   institutions snapshot surface is untouched.
//! - **Trials**: `Simulation::prosecute_violation` (the court's entry point,
//!   called by `enforce_theft` for every caught theft) files a `LegalCase`,
//!   weighs evidence deterministically — NO RNG draw, so adjudication can
//!   never perturb any RNG stream — and returns a verdict + sentence.
//!   Evidence = `0.6 × council enforcement + 0.25 × owned site + 0.1 ×
//!   repeat offenses`; verdict Guilty at evidence ≥ 0.5; sentence =
//!   `enforcement_fine × (0.5 + evidence)` as a supplemental court fine.
//!
//! Blast contract: the trial path executes ONLY when a theft is actually
//! caught, and theft is probe-pinned at ZERO `NormViolated` events in every
//! calibrated window — so golden and snapshots stay byte-identical today.
//! Note this immunity is BEHAVIORAL (no thefts fire in windows), not
//! structural like the technology tree's virgin-stream isolation: if a
//! future scenario (famine, drought) starts generating caught thefts, the
//! court activates — correct feature behavior, but it will surface as
//! calibrated drift then. The new journal variant (`LegalVerdict`) only
//! serializes when present, i.e. never inside a calibrated window.
//!
//! Fine magnitude (the stacking is deliberate punitive justice): the field
//! fine is `taken × price × 2`; the court adds `field_fine × (0.5 +
//! evidence)` — for an owned-site theft (evidence 0.55) the TOTAL is ≈ 4.1×
//! the taken value, ≈ 4.3× for a repeat offender (evidence 0.65). If theft
//! ever becomes live in a window, that is a large wealth shock — the
//! Iter-147 lesson (tiny economy shifts cascade) applies.

use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// Evidence at or above this level returns a Guilty verdict.
pub const EVIDENCE_GUILTY_THRESHOLD: f64 = 0.5;
/// Evidence granted when the victim is the site's owner (property rights
/// make the crime provable).
pub const OWNED_SITE_EVIDENCE_BONUS: f64 = 0.25;
/// Evidence granted per prior conviction, capped at three.
pub const REPEAT_OFFENDER_EVIDENCE_BONUS: f64 = 0.1;
/// Evidence multiplier applied to the councils' combined enforcement
/// capacity (the court's own investigative power).
pub const ENFORCEMENT_EVIDENCE_WEIGHT: f64 = 0.6;
/// The supplemental court fine is `enforcement_fine × (0.5 + evidence)`.
pub const SENTENCE_BASE: f64 = 0.5;

/// A trial's outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    /// Guilty. (doc added at S3 extraction)
    Guilty,
    /// Innocent. (doc added at S3 extraction)
    Innocent,
}

/// A single adjudicated case — the court's record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalCase {
    /// Case id. (doc added at S3 extraction)
    pub case_id: u64,
    /// Norm id. (doc added at S3 extraction)
    pub norm_id: u64,
    /// Accused. (doc added at S3 extraction)
    pub accused: AgentId,
    /// The victim — the site's owner when the theft targeted owned property.
    pub victim: Option<AgentId>,
    /// Site idx. (doc added at S3 extraction)
    pub site_idx: Option<usize>,
    /// Tick filed. (doc added at S3 extraction)
    pub tick_filed: u64,
    /// Deterministic evidence weight in [0, 1].
    pub evidence_strength: Fixed,
    /// Verdict. (doc added at S3 extraction)
    pub verdict: Option<Verdict>,
    /// Supplemental court fine (zero for an Innocent verdict).
    pub sentence: Fixed,
    /// Whether the accused had a prior conviction at filing time.
    pub repeat_offender: bool,
}

/// The court: case records, verdict tallies, and the establishment tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalRegistry {
    /// Cases. (doc added at S3 extraction)
    pub cases: Vec<LegalCase>,
    /// Next case id. (doc added at S3 extraction)
    pub next_case_id: u64,
    /// Convictions. (doc added at S3 extraction)
    pub convictions: u64,
    /// Acquittals. (doc added at S3 extraction)
    pub acquittals: u64,
    /// Set on the first prosecuted violation — the court convenes when a
    /// dispute is brought before it.
    pub established_tick: Option<u64>,
}

impl Default for LegalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LegalRegistry {
    #[must_use]
    /// Fn. (doc added at S3 extraction)
    pub fn new() -> Self {
        Self {
            cases: Vec::new(),
            next_case_id: 0,
            convictions: 0,
            acquittals: 0,
            established_tick: None,
        }
    }

    /// Prior convictions of the accused — a repeat offender faces stronger
    /// evidence (a documented record) and, via the evidence term, a harsher
    /// sentence.
    #[must_use]
    pub fn prior_convictions(&self, accused: AgentId) -> u32 {
        self.cases
            .iter()
            .filter(|c| c.accused == accused && c.verdict == Some(Verdict::Guilty))
            .count() as u32
    }

    /// The evidence available against the accused — deterministic, no RNG.
    /// `enforcement` is the combined Council enforcement capacity in [0, 1].
    #[must_use]
    pub fn evidence_strength(
        &self,
        owned_site: bool,
        prior_convictions: u32,
        enforcement: Fixed,
    ) -> Fixed {
        let owned = if owned_site {
            OWNED_SITE_EVIDENCE_BONUS
        } else {
            0.0
        };
        let repeat = REPEAT_OFFENDER_EVIDENCE_BONUS * prior_convictions.min(3) as f64;
        let total = enforcement.to_f64() * ENFORCEMENT_EVIDENCE_WEIGHT + owned + repeat;
        Fixed::from_f64(total).clamp_01()
    }

    /// The full trial lifecycle: file → weigh evidence → verdict → sentence.
    /// Returns the recorded case. `base_fine` is the field-enforcement fine
    /// already applied; the court adds a supplemental sentence on a Guilty
    /// verdict. Deterministic — adjudication draws no RNG.
    pub fn prosecute(
        &mut self,
        norm_id: u64,
        accused: AgentId,
        victim: Option<AgentId>,
        site_idx: Option<usize>,
        owned_site: bool,
        enforcement: Fixed,
        base_fine: Fixed,
        tick: u64,
    ) -> LegalCase {
        let prior = self.prior_convictions(accused);
        let evidence = self.evidence_strength(owned_site, prior, enforcement);
        let guilty = evidence >= Fixed::from_f64(EVIDENCE_GUILTY_THRESHOLD);
        let sentence = if guilty {
            base_fine * (Fixed::from_f64(SENTENCE_BASE) + evidence)
        } else {
            Fixed::ZERO
        };
        let case = LegalCase {
            case_id: self.next_case_id,
            norm_id,
            accused,
            victim,
            site_idx,
            tick_filed: tick,
            evidence_strength: evidence,
            verdict: Some(if guilty {
                Verdict::Guilty
            } else {
                Verdict::Innocent
            }),
            sentence,
            repeat_offender: prior > 0,
        };
        self.next_case_id += 1;
        if self.established_tick.is_none() {
            self.established_tick = Some(tick);
        }
        match case.verdict {
            Some(Verdict::Guilty) => self.convictions += 1,
            Some(Verdict::Innocent) => self.acquittals += 1,
            None => {}
        }
        self.cases.push(case.clone());
        case
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: AgentId = AgentId::new(1);
    const V: AgentId = AgentId::new(2);

    #[test]
    fn evidence_strength_scales_with_enforcement_and_owned_site() {
        let reg = LegalRegistry::new();
        let none = Fixed::ZERO;
        let half = Fixed::from_f64(0.5);
        assert_eq!(
            reg.evidence_strength(false, 0, none),
            Fixed::ZERO,
            "no court power, communal site"
        );
        let communal = reg.evidence_strength(false, 0, half);
        assert_eq!(communal.to_f64(), 0.3, "0.6 × 0.5 enforcement");
        let owned = reg.evidence_strength(true, 0, half);
        assert_eq!(owned.to_f64(), 0.55, "communal + 0.25 owned-site bonus");
        assert!(owned > communal, "property rights strengthen evidence");
    }

    #[test]
    fn evidence_strength_repeat_offender_bonus_caps_at_three() {
        let reg = LegalRegistry::new();
        let half = Fixed::from_f64(0.5);
        let once = reg.evidence_strength(true, 1, half);
        let thrice = reg.evidence_strength(true, 3, half);
        let ten = reg.evidence_strength(true, 10, half);
        assert!(
            once > reg.evidence_strength(true, 0, half),
            "repeat record adds evidence"
        );
        assert_eq!(
            thrice, ten,
            "the repeat bonus saturates at three prior convictions"
        );
    }

    #[test]
    fn owned_site_theft_with_court_power_is_convicted() {
        let mut reg = LegalRegistry::new();
        let fine = Fixed::from_f64(10.0);
        let case = reg.prosecute(
            7,
            A,
            Some(V),
            Some(3),
            true,
            Fixed::from_f64(0.5),
            fine,
            100,
        );
        assert_eq!(case.verdict, Some(Verdict::Guilty), "evidence 0.55 ≥ 0.5");
        assert!(
            case.sentence > Fixed::ZERO,
            "a Guilty verdict carries a court fine"
        );
        assert_eq!(case.sentence.to_f64(), 10.0 * 1.05, "fine × (0.5 + 0.55)");
        assert_eq!(case.evidence_strength.to_f64(), 0.55);
        assert_eq!(reg.convictions, 1);
        assert_eq!(
            reg.established_tick,
            Some(100),
            "the court convenes on the first case"
        );
    }

    #[test]
    fn weak_evidence_acquits_without_sentence() {
        let mut reg = LegalRegistry::new();
        let case = reg.prosecute(
            7,
            A,
            None,
            None,
            false,
            Fixed::ZERO,
            Fixed::from_f64(10.0),
            50,
        );
        assert_eq!(case.verdict, Some(Verdict::Innocent), "evidence 0 < 0.5");
        assert_eq!(case.sentence, Fixed::ZERO, "an acquittal carries no fine");
        assert_eq!(reg.acquittals, 1);
        assert_eq!(reg.convictions, 0);
    }

    #[test]
    fn repeat_offender_faces_harsher_sentence() {
        let mut reg = LegalRegistry::new();
        let fine = Fixed::from_f64(10.0);
        let first = reg.prosecute(
            7,
            A,
            Some(V),
            Some(3),
            true,
            Fixed::from_f64(0.5),
            fine,
            100,
        );
        assert!(!first.repeat_offender, "first offense is not a repeat");
        let second = reg.prosecute(
            7,
            A,
            Some(V),
            Some(3),
            true,
            Fixed::from_f64(0.5),
            fine,
            200,
        );
        assert!(second.repeat_offender, "second offense is a repeat");
        assert!(
            second.sentence > first.sentence,
            "repeat evidence 0.65 > 0.55 raises the fine"
        );
        // Both offenses are on the record now; the repeat_offender flag above
        // (computed at filing time, before the second case was pushed) is what
        // proves the timing semantics.
        assert_eq!(reg.prior_convictions(A), 2, "both offenses are on record");
        assert_eq!(reg.convictions, 2);
    }

    #[test]
    fn prior_convictions_counts_only_guilty_verdicts() {
        let mut reg = LegalRegistry::new();
        reg.prosecute(
            7,
            A,
            None,
            None,
            false,
            Fixed::ZERO,
            Fixed::from_f64(10.0),
            50,
        ); // acquittal
        reg.prosecute(
            7,
            A,
            Some(V),
            Some(3),
            true,
            Fixed::from_f64(0.5),
            Fixed::from_f64(10.0),
            100,
        ); // conviction
        assert_eq!(
            reg.prior_convictions(A),
            1,
            "acquittals do not count against the accused"
        );
        assert_eq!(reg.cases.len(), 2);
        assert_eq!(reg.next_case_id, 2);
    }
}
