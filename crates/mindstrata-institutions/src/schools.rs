//! §5 (Iteration 151): Formal schools — cohort instruction beyond the
//! one-to-one apprenticeship.
//!
//! The apprenticeship system (§8.1.18) pairs one master with one apprentice
//! per knowledge, scanned per student per tick. Formal schools scale
//! instruction: a single experienced teacher instructs a small cohort of the
//! youngest students in a single yearly term, teaching the teacher's most
//! advanced knowledge to the whole class at once.
//!
//! The term is fully deterministic — teacher and cohort are chosen by fixed
//! rules and [`crate::culture::education::attempt_teaching`] draws no
//! randomness — so a school term can never perturb any RNG stream, even when
//! a school exists.
//!
//! Zero-blast by construction: the term only runs when a `SiteKind::School`
//! exists, and no default world places one. Calibrated windows (golden,
//! snapshots) contain no school site, so the pass is a structural no-op and
//! leaves every calibrated output byte-identical.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A school term runs once per year, on the same 4320-tick cadence as the
/// monthly ritual, technology discovery, and the diplomacy pass.
pub const SCHOOL_CADENCE: u64 = 4320;

/// Maximum number of students a single teacher instructs per term.
pub const COHORT_SIZE: usize = 5;

/// Minimum teaching skill for an agent to run a school term (mirrors the
/// apprenticeship's competent-teacher threshold).
pub const MIN_TEACHING_SKILL: f64 = 0.3;

/// Running totals across all school terms (observability + test surface).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchoolRegistry {
    /// Number of terms convened.
    pub terms_run: u64,
    /// Total lessons taught (teacher × student pairs).
    pub lessons_taught: u64,
    /// Students who first learned the term's knowledge.
    pub graduates: u64,
}

impl SchoolRegistry {
    /// Create an empty registry (fresh on both simulation constructors —
    /// the `weather` / `legal` / `diplomacy` precedent: no calibrated window
    /// contains a school site, so a fresh restore is always faithful).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether no term has ever convened.
    #[must_use]
    pub fn is_dormant(&self) -> bool {
        self.terms_run == 0
    }
}

/// Pick the instructor for a school term: the highest teaching skill among
/// agents who hold at least one knowledge. Ties break to the lowest index so
/// the choice is deterministic. Returns `None` when nobody is competent.
#[must_use]
pub fn select_teacher(teaching_skill: &[Fixed], knowledge_held: &[bool]) -> Option<usize> {
    let min = Fixed::from_f64(MIN_TEACHING_SKILL);
    let mut best: Option<(usize, Fixed)> = None;
    for (i, (&skill, &held)) in teaching_skill.iter().zip(knowledge_held).enumerate() {
        if !held || skill < min {
            continue;
        }
        if best.is_none_or(|(_, s)| skill > s) {
            best = Some((i, skill));
        }
    }
    best.map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(x: f64) -> Fixed {
        Fixed::from_f64(x)
    }

    #[test]
    fn select_teacher_picks_highest_skill() {
        let skills = [f(0.0), f(0.5), f(0.9), f(0.7)];
        let held = [true, true, true, true];
        assert_eq!(select_teacher(&skills, &held), Some(2));
    }

    #[test]
    fn select_teacher_skips_incompetent_agents() {
        let skills = [f(0.2), f(0.45), f(0.8)];
        let held = [true, true, true];
        // Index 0 is below the teaching threshold.
        assert_eq!(select_teacher(&skills, &held), Some(2));
    }

    #[test]
    fn select_teacher_skips_agents_without_knowledge() {
        let skills = [f(0.9), f(0.8), f(0.7)];
        let held = [false, true, true];
        assert_eq!(select_teacher(&skills, &held), Some(1));
    }

    #[test]
    fn select_teacher_returns_none_when_no_competent_teacher() {
        assert_eq!(select_teacher(&[f(0.1)], &[true]), None);
        assert_eq!(select_teacher(&[], &[]), None);
    }

    #[test]
    fn select_teacher_breaks_ties_to_lowest_index() {
        let skills = [f(0.6), f(0.6), f(0.6)];
        let held = [true, true, true];
        assert_eq!(select_teacher(&skills, &held), Some(0));
    }

    #[test]
    fn registry_starts_dormant_and_counts() {
        let mut reg = SchoolRegistry::new();
        assert!(reg.is_dormant());
        reg.terms_run += 1;
        reg.lessons_taught += 3;
        reg.graduates += 2;
        assert!(!reg.is_dormant());
        assert_eq!(reg.lessons_taught, 3);
        assert_eq!(reg.graduates, 2);
    }
}
