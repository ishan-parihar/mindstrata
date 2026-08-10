//! §5 (Iteration 152): Religious mechanics — theological beliefs and the
//! religious calendar.
//!
//! The ritual system (§culture/ritual.rs) already fires communal ceremonies
//! and sacred values already track desacralization — what the plan row
//! ("Religious mechanics — theological beliefs, calendar (rituals are live)")
//! calls out as missing is a *theology*: an explicit religion whose doctrine
//! agents convert to, hold convictions about, and celebrate on an annual
//! festival calendar.
//!
//! A world opts in by seeding a [`Religion`] (a deity + a doctrine naming a
//! sacred value). Until one is seeded, [`TheologyRegistry`] is dormant and
//! the yearly pass is a structural no-op — no default world seeds a
//! religion, so every calibrated window (golden, snapshots) is untouched.
//! Once seeded, the pass is still fully deterministic (fixed conversion and
//! drift rules, no RNG drawn anywhere), so it can never perturb any RNG
//! stream.
//!
//! Known limitation: beliefs are keyed by agent slot, and the generational
//! replacement path fills a dead agent's slot in place without clearing its
//! belief — so in a religion-bearing long run a replacement is born a
//! believer with a stale `since_tick`. Moot in every calibrated window (no
//! religion is ever seeded there); a follow-up could hook the replacement
//! path to reset the slot's belief.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Half-year cadence for religious events: conversion + conviction drift on
/// the yearly (4320) mark, festivals on the mid-year (2160) mark. Mirrors
/// the ritual/technology/diplomacy/school cadences.
pub const RELIGIOUS_CADENCE: u64 = 2160;

/// Age at which an agent is an "elder" early adopter of a new religion.
pub const ELDER_AGE: f64 = 30.0;

/// How the faithful view the deity — the theodicy dimension of a belief.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Temperament {
    /// The deity rewards the faithful and punishes the wicked.
    Benevolent,
    /// The deity is remote and unconcerned with mortal affairs.
    Indifferent,
    /// The deity is wrathful; misfortune is divine punishment.
    Malevolent,
}

/// A named deity with a theodicy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deity {
    pub name: String,
    pub temperament: Temperament,
}

/// A religion's moral code — a name, precepts, and the sacred value it
/// hallows (sacralized at festivals).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doctrine {
    pub name: String,
    pub precepts: Vec<String>,
    pub sacred_value: String,
}

/// A complete religion: who is worshipped and what the faithful believe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Religion {
    pub deity: Deity,
    pub doctrine: Doctrine,
}

impl Religion {
    /// A convenience constructor for worlds that opt in.
    #[must_use]
    pub fn seeded(
        deity_name: &str,
        temperament: Temperament,
        doctrine_name: &str,
        precepts: Vec<String>,
        sacred_value: &str,
    ) -> Self {
        Self {
            deity: Deity {
                name: deity_name.into(),
                temperament,
            },
            doctrine: Doctrine {
                name: doctrine_name.into(),
                precepts,
                sacred_value: sacred_value.into(),
            },
        }
    }
}

/// One agent's theology: how strongly they believe and how they see the
/// deity, plus when they converted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheologicalBelief {
    /// Conviction in [0, 1] — how firmly the agent holds the faith.
    pub conviction: Fixed,
    /// The believer's theodicy (their view of the deity).
    pub temperament_held: Temperament,
    /// Tick the agent converted.
    pub since_tick: u64,
}

/// World-level religious state: the seeded religion (if any), per-agent
/// beliefs, and festival/conversion tallies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TheologyRegistry {
    /// The world's religion. `None` → the whole system is dormant.
    pub religion: Option<Religion>,
    /// Per-agent beliefs, indexed by agent index (agents never reindex).
    pub beliefs: Vec<Option<TheologicalBelief>>,
    /// Festivals actually held (with at least one believer attending).
    pub festivals_held: u64,
    /// Total conversions recorded.
    pub converts: u64,
}

impl TheologyRegistry {
    /// Create an empty registry (fresh on both simulation constructors —
    /// the `weather` / `legal` / `diplomacy` / `school` precedent: no
    /// calibrated window seeds a religion, so a fresh restore is faithful).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the theology system is dormant (no religion seeded).
    #[must_use]
    pub fn is_dormant(&self) -> bool {
        self.religion.is_none()
    }

    /// Number of believers.
    #[must_use]
    pub fn believer_count(&self) -> u64 {
        self.beliefs.iter().filter(|b| b.is_some()).count() as u64
    }
}

/// Whether an unconverted agent converts this year: elders convert freely;
/// everyone else needs social contact with an existing believer (the
/// default-neutral relationship view makes contagion spread fast once a
/// believer exists).
#[must_use]
pub fn should_convert(is_elder: bool, has_believer_contact: bool) -> bool {
    is_elder || has_believer_contact
}

/// Initial conviction of a new convert: elders start stronger, tempered by
/// age, clamped to [0, 1].
#[must_use]
pub fn seed_conviction(is_elder: bool, age: Fixed) -> Fixed {
    let base = if is_elder {
        Fixed::from_f64(0.6)
    } else {
        Fixed::from_f64(0.4)
    };
    (base + age * Fixed::from_f64(0.002)).clamp_01()
}

/// One year of conviction drift: pull the belief toward the community mean
/// (grudges and fervor cool alike), clamped to [0, 1].
#[must_use]
pub fn drift_conviction(current: Fixed, community_mean: Fixed, rate: f64) -> Fixed {
    (current + (community_mean - current) * Fixed::from_f64(rate)).clamp_01()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(x: f64) -> Fixed {
        Fixed::from_f64(x)
    }

    #[test]
    fn new_registry_is_dormant() {
        let reg = TheologyRegistry::new();
        assert!(reg.is_dormant());
        assert_eq!(reg.believer_count(), 0);
        assert_eq!(reg.converts, 0);
        assert_eq!(reg.festivals_held, 0);
    }

    #[test]
    fn seeded_religion_wakes_the_registry() {
        let mut reg = TheologyRegistry::new();
        reg.religion = Some(Religion::seeded(
            "The Shepherd", Temperament::Benevolent,
            "The Way", vec!["Tend the flock".into()], "The Flock",
        ));
        assert!(!reg.is_dormant());
        assert_eq!(reg.religion.as_ref().unwrap().deity.temperament, Temperament::Benevolent);
        assert_eq!(reg.religion.as_ref().unwrap().doctrine.sacred_value, "The Flock");
    }

    #[test]
    fn conversion_rule_is_elder_or_contact() {
        assert!(should_convert(true, false), "elders convert freely");
        assert!(should_convert(false, true), "contact with a believer converts");
        assert!(!should_convert(false, false), "isolated young stay unconverted");
    }

    #[test]
    fn conviction_seed_prefers_elders_and_clamps() {
        assert!(seed_conviction(true, f(40.0)) > seed_conviction(false, f(20.0)));
        // A very old elder cannot exceed 1.0.
        assert!(seed_conviction(true, f(500.0)) <= Fixed::ONE);
    }

    #[test]
    fn conviction_drift_pulls_toward_the_mean() {
        let high = drift_conviction(f(0.9), f(0.5), 0.1);
        let low = drift_conviction(f(0.1), f(0.5), 0.1);
        assert!(high < f(0.9), "fervor cools toward the mean");
        assert!(low > f(0.1), "weak faith firms up toward the mean");
        assert!(drift_conviction(f(0.9), f(0.5), 0.1) >= Fixed::ZERO);
    }
}
