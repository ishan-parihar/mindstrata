//! Cultural cognition — agents think through cultural categories (§8.1.18).
//!
//! Agents don't just have individual beliefs; they think through cultural
//! categories, prototypes, taboos, honor codes, purity maps, and ritual scripts.
//!
//! ```text
//! Cultural cognition components:
//!   - Categories (prototypes, exemplars, boundaries)
//!   - Taboos (strong prohibitions with disgust response)
//!   - Honor codes (status rules, face-saving, shame triggers)
//!   - Purity maps (sacred/profane boundaries, contamination fears)
//!   - Ritual scripts (expected behavior in ceremonial contexts)
//!
//! Effects:
//!   - outgroup disgust
//!   - sacred boundary defense
//!   - ritual obedience
//!   - cultural creativity
//!   - syncretism
//!   - heresy detection
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A cultural category that the agent uses to interpret the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CulturalCategory {
    /// Category name (e.g., "farmer", "priest", "foreigner").
    pub name: String,
    /// How strongly the agent identifies with this category (0–1).
    pub identification: Fixed,
    /// How rigid the agent's prototype for this category is (0–1).
    /// High rigidity = less tolerance for category ambiguity.
    pub rigidity: Fixed,
    /// Disgust response toward outgroup members of this category (0–1).
    pub outgroup_disgust: Fixed,
    /// Number of exemplars the agent has encountered.
    pub exemplar_count: u32,
}

impl CulturalCategory {
    /// Create a new cultural category.
    pub fn new(name: String) -> Self {
        Self {
            name,
            identification: Fixed::from_f64(0.3),
            rigidity: Fixed::from_f64(0.5),
            outgroup_disgust: Fixed::ZERO,
            exemplar_count: 0,
        }
    }

    /// Encounter an exemplar of this category (strengthens or weakens prototype).
    pub fn encounter(&mut self, positive: bool) {
        self.exemplar_count += 1;
        if positive {
            // Positive encounters slightly reduce rigidity (more nuanced view)
            self.rigidity = (self.rigidity - Fixed::from_f64(0.01)).max(Fixed::from_f64(0.1));
        } else {
            // Negative encounters increase outgroup disgust
            self.outgroup_disgust = (self.outgroup_disgust + Fixed::from_f64(0.02)).clamp_01();
        }
    }
}

/// A taboo — a strong prohibition backed by disgust.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Taboo {
    /// What is forbidden (e.g., "eating pork", "speaking ill of elders").
    pub description: String,
    /// Strength of the prohibition (0–1). Sacred taboos are near 1.0.
    pub strength: Fixed,
    /// Whether violating this taboo triggers disgust rather than just anger.
    pub triggers_disgust: bool,
    /// Whether this taboo is sacred (resistant to change even under pressure).
    pub sacred: bool,
    /// How many agents in the agent's social network share this taboo.
    pub social_reinforcement: u32,
}

impl Taboo {
    /// Create a new taboo.
    pub fn new(description: String, strength: Fixed, sacred: bool) -> Self {
        Self {
            description,
            strength,
            triggers_disgust: true,
            sacred,
            social_reinforcement: 0,
        }
    }

    /// Compute the cost of violating this taboo.
    pub fn violation_cost(&self) -> Fixed {
        let sacred_boost = if self.sacred {
            Fixed::from_f64(0.3)
        } else {
            Fixed::ZERO
        };
        let social_boost = Fixed::from_f64(self.social_reinforcement as f64 * 0.02);
        (self.strength + sacred_boost + social_boost).clamp_01()
    }

    /// Social reinforcement from others sharing this taboo.
    pub fn reinforce(&mut self) {
        self.social_reinforcement += 1;
        self.strength = (self.strength + Fixed::from_f64(0.005)).clamp_01();
    }
}

/// An honor code — rules about status, face, and shame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HonorCode {
    /// Rule name (e.g., "never show fear", "defend family honor").
    pub rule: String,
    /// How central this rule is to the agent's identity (0–1).
    pub centrality: Fixed,
    /// What triggers shame if violated (e.g., "public humiliation").
    pub shame_trigger: String,
    /// Strength of the honor obligation (0–1).
    pub obligation_strength: Fixed,
}

impl HonorCode {
    /// Create a new honor code.
    pub fn new(rule: String, shame_trigger: String) -> Self {
        Self {
            rule,
            centrality: Fixed::from_f64(0.5),
            shame_trigger,
            obligation_strength: Fixed::from_f64(0.6),
        }
    }

    /// Compute shame from violating this honor code.
    pub fn shame_from_violation(&self, public: bool) -> Fixed {
        let public_multiplier = if public {
            Fixed::from_f64(1.5)
        } else {
            Fixed::from_f64(0.5)
        };
        (self.obligation_strength * self.centrality * public_multiplier).clamp_01()
    }
}

/// A purity map — sacred/profane boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurityMap {
    /// Domain name (e.g., "food", "body", "temple", "language").
    pub domain: String,
    /// Purity sensitivity in this domain (0–1).
    pub sensitivity: Fixed,
    /// What is considered sacred in this domain.
    pub sacred_elements: Vec<String>,
    /// What is considered profane/contaminating.
    pub contaminating_elements: Vec<String>,
}

impl PurityMap {
    /// Create a new purity map.
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            sensitivity: Fixed::from_f64(0.3),
            sacred_elements: Vec::new(),
            contaminating_elements: Vec::new(),
        }
    }

    /// Check if an element is sacred in this domain.
    pub fn is_sacred(&self, element: &str) -> bool {
        self.sacred_elements.iter().any(|s| s == element)
    }

    /// Check if an element is contaminating in this domain.
    pub fn is_contaminating(&self, element: &str) -> bool {
        self.contaminating_elements.iter().any(|c| c == element)
    }

    /// Compute disgust from contamination exposure.
    pub fn contamination_disgust(&self, contaminant: &str) -> Fixed {
        if self.is_contaminating(contaminant) {
            self.sensitivity
        } else {
            Fixed::ZERO
        }
    }
}

/// A ritual script — expected behavior in ceremonial contexts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RitualScript {
    /// Ritual name (e.g., "harvest_prayer", "funeral_mourning").
    pub name: String,
    /// Expected actions in order.
    pub steps: Vec<String>,
    /// How strictly the agent follows this script (0–1).
    pub adherence: Fixed,
    /// Emotional intensity expected during this ritual.
    pub expected_intensity: Fixed,
}

impl RitualScript {
    /// Create a new ritual script.
    pub fn new(name: String) -> Self {
        Self {
            name,
            steps: Vec::new(),
            adherence: Fixed::from_f64(0.6),
            expected_intensity: Fixed::from_f64(0.5),
        }
    }

    /// Compute deviation from expected behavior during ritual.
    pub fn deviation_cost(&self, actual_intensity: Fixed) -> Fixed {
        let intensity_deviation = (actual_intensity - self.expected_intensity).abs();
        let adherence_penalty = (Fixed::ONE - self.adherence) * Fixed::from_f64(0.3);
        (intensity_deviation + adherence_penalty).clamp_01()
    }
}

/// The agent's cultural cognition system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CulturalCognition {
    /// Cultural categories the agent uses.
    pub categories: Vec<CulturalCategory>,
    /// Taboos the agent follows.
    pub taboos: Vec<Taboo>,
    /// Honor codes the agent adheres to.
    pub honor_codes: Vec<HonorCode>,
    /// Purity maps for different domains.
    pub purity_maps: Vec<PurityMap>,
    /// Ritual scripts the agent knows.
    pub ritual_scripts: Vec<RitualScript>,
    /// Overall cultural conservatism (0–1). High = resistant to cultural change.
    pub conservatism: Fixed,
    /// Overall openness to cultural synthesis (0–1).
    pub syncretism_openness: Fixed,
}

impl Default for CulturalCognition {
    fn default() -> Self {
        Self {
            categories: Vec::new(),
            taboos: Vec::new(),
            honor_codes: Vec::new(),
            purity_maps: Vec::new(),
            ritual_scripts: Vec::new(),
            conservatism: Fixed::from_f64(0.5),
            syncretism_openness: Fixed::from_f64(0.3),
        }
    }
}

impl CulturalCognition {
    /// Create from personality traits.
    pub fn from_personality(conservatism_base: Fixed, openness_base: Fixed) -> Self {
        Self {
            conservatism: conservatism_base,
            syncretism_openness: openness_base,
            ..Default::default()
        }
    }

    /// §8.1.18 (Iteration 165): Seed the shared village taboo set into this
    /// agent's cultural cognition, with each taboo's strength scaled by the
    /// agent's traditionalism — traditional agents internalize prohibitions
    /// more strongly.
    ///
    /// The set mirrors the `specs/culture/taboos_v2.ron` categories (Theft,
    /// Violence, Disrespect, Incest, Sacrilege, Heresy, Adultery, Lying,
    /// BrokenOath) — a compact, culturally grounded subset. Deterministic:
    /// no RNG, a pure function of `traditionalism`, so every run of a given
    /// seed produces an identical taboo profile (replay-safe). This is the
    /// system's first production writer: before Iteration 165, agents were
    /// born with empty `taboos` vecs and the §8.1.18 taboo layer
    /// (`Taboo`, `violation_cost`, `tabo_violated_by`, `change_resistance`)
    /// was write-only dead code — no seeding, no consumers. Honest scope:
    /// `max_taboo_strength` is the only production READ today (the §10.4
    /// `taboo_penalty` channel); the other taboo helpers remain
    /// consumer-free (unit-tested only), now with real seeded data.
    pub fn seed_village_taboos(&mut self, traditionalism: Fixed) {
        // (description, base strength, sacred) — base strengths chosen so
        // the traditionalism-scaled profile lands in a live-but-modest band:
        // the strongest prohibition (Incest, sacred) sits at ~0.35–0.70 for
        // the traditionalism range [0, 1], a real but non-destructive
        // restraint on `total_attraction()`.
        const VILLAGE_TABOOS: &[(&str, f64, bool)] = &[
            ("Incest", 0.7, true),
            ("Adultery", 0.6, true),
            ("Sacrilege", 0.55, true),
            ("Theft", 0.5, false),
            ("Violence", 0.45, false),
            ("Lying", 0.4, false),
            ("Disrespect", 0.35, false),
        ];
        for (description, base, sacred) in VILLAGE_TABOOS {
            // Strength = base × (0.5 + 0.5 × traditionalism) → [0.5×base, base].
            let scale = Fixed::from_f64(0.5) + Fixed::from_f64(0.5) * traditionalism;
            let strength = (Fixed::from_f64(*base) * scale).clamp_01();
            // `add_taboo` deduplicates by description (reinforcing instead of
            // duplicating): re-seeding never grows the vec — but note the
            // dedupe path calls `reinforce()` (+0.005 strength, +1 social
            // reinforcement), so re-seeding is len-stable, NOT
            // strength-idempotent. Production seeds exactly once per agent at
            // populate, so the seeded strengths above are exact in every run.
            self.add_taboo(Taboo::new((*description).to_string(), strength, *sacred));
        }
    }

    /// §8.1.18 (Iteration 165): The strongest taboo the agent holds — the
    /// raw prohibition strength (deliberately excluding the `violation_cost`
    /// sacred boost, which is a violation-*reaction* term, not a standing
    /// restraint). Consumed by the §10.4 `taboo_penalty` courtship channel:
    /// an agent whose culture forbids more, courts more hesitantly.
    ///
    /// Zero when the agent holds no taboos — the identity-at-zero contract
    /// that keeps pre-Iteration-165 snapshots (empty taboo vecs) byte-neutral
    /// on `total_attraction()`. Pure, no RNG.
    pub fn max_taboo_strength(&self) -> Fixed {
        self.taboos
            .iter()
            .map(|t| t.strength)
            .fold(Fixed::ZERO, Fixed::max)
    }

    /// §8.1.18 (Iteration 167): The `violation_cost` of the taboo matching
    /// the given keyword (case-insensitive substring on the description), or
    /// ZERO when the agent holds no matching taboo.
    ///
    /// This is the sacred-severity term: when an act violates a taboo the
    /// agent's culture holds sacred, the transgression carries that taboo's
    /// full violation cost (strength + sacred boost + social reinforcement).
    /// Consumed by the §19.5.D norm-enforcement shame channel: an attacker
    /// whose culture forbids violence more strongly (higher traditionalism →
    /// stronger taboo) feels more shame per violent act.
    ///
    /// ZERO when no taboo matches — the identity-at-zero contract that keeps
    /// pre-Iteration-165 snapshots (empty taboo vecs) byte-neutral on the
    /// consumer. ONE-SIDED: a taboo-free agent's shame is unchanged. Pure,
    /// no RNG.
    pub fn taboo_violation_cost_for(&self, keyword: &str) -> Fixed {
        let needle = keyword.to_lowercase();
        self.taboos
            .iter()
            .find(|t| t.description.to_lowercase().contains(&needle))
            .map_or(Fixed::ZERO, Taboo::violation_cost)
    }

    /// Total violation cost across EVERY taboo a described change violates.
    ///
    /// The first production consumer of `tabo_violated_by` (the §8.1.18
    /// taboo-resolution helper, previously consumer-free): where
    /// `taboo_violation_cost_for` reads the single strongest matching
    /// prohibition, this sums the full weight of every taboo the change
    /// violates — the total cultural gravity of the transgression. Consumed
    /// by the §19.5.H failed-threat escalation aversion (Iteration 169): an
    /// agent whose culture forbids violence hesitates before escalating a
    /// failed threat.
    ///
    /// ZERO when no taboo matches — the identity-at-zero contract. Pure,
    /// no RNG.
    pub fn taboo_violation_cost_sum(&self, change_description: &str) -> Fixed {
        self.tabo_violated_by(change_description)
            .iter()
            .map(|t| t.violation_cost())
            .fold(Fixed::ZERO, |acc, c| acc + c)
    }

    /// Add a taboo and reinforce existing ones that overlap.
    pub fn add_taboo(&mut self, taboo: Taboo) {
        // Check for overlapping taboos
        let existing = self
            .taboos
            .iter_mut()
            .find(|t| t.description == taboo.description);
        if let Some(existing) = existing {
            existing.reinforce();
        } else {
            self.taboos.push(taboo);
        }
    }

    /// Compute the agent's resistance to cultural change.
    pub fn change_resistance(&self) -> Fixed {
        let taboo_strength: Fixed = self
            .taboos
            .iter()
            .map(Taboo::violation_cost)
            .fold(Fixed::ZERO, |acc, t| acc + t);
        let category_rigidity: Fixed = if self.categories.is_empty() {
            Fixed::ZERO
        } else {
            let total: Fixed = self
                .categories
                .iter()
                .map(|c| c.rigidity)
                .fold(Fixed::ZERO, |acc, r| acc + r);
            total / Fixed::from_int(self.categories.len() as i64)
        };
        (self.conservatism * Fixed::from_f64(0.3)
            + category_rigidity * Fixed::from_f64(0.3)
            + taboo_strength.min(Fixed::ONE) * Fixed::from_f64(0.4))
        .clamp_01()
    }

    /// Check if a proposed cultural change would violate taboos.
    pub fn tabo_violated_by(&self, change_description: &str) -> Vec<&Taboo> {
        self.taboos
            .iter()
            .filter(|t| {
                change_description
                    .to_lowercase()
                    .contains(&t.description.to_lowercase())
            })
            .collect()
    }

    /// Compute outgroup disgust for a target based on cultural categories.
    /// Returns the outgroup_disgust of the category matching the target name,
    /// or zero if no matching category exists.
    pub fn outgroup_disgust_for(&self, target_category: &str) -> Fixed {
        self.categories
            .iter()
            .find(|c| c.name == target_category)
            .map_or(Fixed::ZERO, |c| c.outgroup_disgust)
    }

    /// Tick update — conservatism shifts based on exposure.
    /// Positive exposure reduces conservatism; negative exposure increases it.
    pub fn tick_update(&mut self, positive_exposure: Fixed, negative_exposure: Fixed) {
        // Positive cultural exposure slightly reduces conservatism
        self.conservatism = (self.conservatism
            - positive_exposure * Fixed::from_f64(0.001) * (Fixed::ONE - self.conservatism))
            .max(Fixed::ZERO);
        // Negative exposure (threat, trauma) increases conservatism
        self.conservatism = (self.conservatism
            + negative_exposure * Fixed::from_f64(0.0015) * (Fixed::ONE - self.conservatism))
            .clamp_01();
        // Taboo strength decays slowly for non-sacred taboos
        for taboo in &mut self.taboos {
            if !taboo.sacred {
                taboo.strength = (taboo.strength - Fixed::from_f64(0.0005)).max(Fixed::ZERO);
            }
        }
    }
}

/// §8.1.18 (Iteration 166): the taboo knowledge-resistance factor — how
/// much an agent's strongest internalized taboo dampens absorption of
/// novel knowledge (the plan's "sacred boundary defense" / heresy
/// resistance: a culture that forbids more accepts fewer innovations).
///
/// `1 − max_taboo × rate`, floored at `floor` — ONE-SIDED identity at
/// zero: a taboo-free agent's factor is exactly 1.0 (no dampening), and
/// the floor guarantees absorption is slowed but never fully blocked
/// (knowledge can still spread, just more slowly for taboo-bound agents).
/// Pure, no RNG, replay-deterministic.
///
/// Mirrors the `awe_reverence_factor` one-sided-factor pattern (Iteration
/// 130): the rate is the dampening strength, the floor is the never-fully-
/// cancels guard.
pub fn taboo_knowledge_factor(max_taboo_strength: Fixed, rate: Fixed, floor: Fixed) -> Fixed {
    (Fixed::ONE - max_taboo_strength * rate).max(floor).clamp_01()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_category_has_sane_defaults() {
        let c = CulturalCategory::new("farmer".into());
        assert_eq!(c.name, "farmer");
        assert!(c.rigidity > Fixed::ZERO);
        assert_eq!(c.exemplar_count, 0);
    }

    #[test]
    fn positive_encounter_reduces_rigidity() {
        let mut c = CulturalCategory::new("priest".into());
        let before = c.rigidity;
        c.encounter(true);
        assert!(c.rigidity < before);
        assert_eq!(c.exemplar_count, 1);
    }

    #[test]
    fn negative_encounter_increases_outgroup_disgust() {
        let mut c = CulturalCategory::new("foreigner".into());
        c.encounter(false);
        assert!(c.outgroup_disgust > Fixed::ZERO);
    }

    #[test]
    fn taboo_violation_cost_increases_with_sacredness() {
        let mut sacred = Taboo::new("never eat pork".into(), Fixed::from_f64(0.7), true);
        sacred.reinforce();
        let sacred_cost = sacred.violation_cost();
        let profane = Taboo::new("no whistling at night".into(), Fixed::from_f64(0.3), false);
        let profane_cost = profane.violation_cost();
        assert!(sacred_cost > profane_cost);
    }

    #[test]
    fn honor_shame_increases_with_publicity() {
        let h = HonorCode::new("never show fear".into(), "public humiliation".into());
        let private_shame = h.shame_from_violation(false);
        let public_shame = h.shame_from_violation(true);
        assert!(public_shame > private_shame);
    }

    #[test]
    fn purity_map_detects_contamination() {
        let mut p = PurityMap::new("food".into());
        p.contaminating_elements.push("pork".into());
        p.sacred_elements.push("bread".into());
        assert!(p.is_contaminating("pork"));
        assert!(!p.is_contaminating("beef"));
        assert!(p.is_sacred("bread"));
        assert!(p.contamination_disgust("pork") > Fixed::ZERO);
    }

    #[test]
    fn ritual_deviation_cost_scales_with_adherence() {
        let mut r = RitualScript::new("harvest_prayer".into());
        r.adherence = Fixed::from_f64(0.9);
        r.expected_intensity = Fixed::from_f64(0.7);
        let low_deviation = r.deviation_cost(Fixed::from_f64(0.65));
        let high_deviation = r.deviation_cost(Fixed::from_f64(0.2));
        assert!(high_deviation > low_deviation);
    }

    #[test]
    fn change_resistance_increases_with_taboos() {
        let mut cc = CulturalCognition::default();
        let no_taboo = cc.change_resistance();
        cc.add_taboo(Taboo::new("sacred rule".into(), Fixed::from_f64(0.8), true));
        let with_taboo = cc.change_resistance();
        assert!(with_taboo > no_taboo);
    }

    #[test]
    fn conservatism_decays_with_positive_exposure() {
        let mut cc = CulturalCognition {
            conservatism: Fixed::from_f64(0.7),
            ..Default::default()
        };
        cc.tick_update(Fixed::ONE, Fixed::ZERO);
        assert!(cc.conservatism < Fixed::from_f64(0.7));
    }

    #[test]
    fn conservatism_increases_with_negative_exposure() {
        let mut cc = CulturalCognition {
            conservatism: Fixed::from_f64(0.3),
            ..Default::default()
        };
        cc.tick_update(Fixed::ZERO, Fixed::ONE);
        assert!(cc.conservatism > Fixed::from_f64(0.3));
    }

    #[test]
    fn from_personality_sets_base_values() {
        let cc = CulturalCognition::from_personality(Fixed::from_f64(0.8), Fixed::from_f64(0.2));
        assert_eq!(cc.conservatism, Fixed::from_f64(0.8));
        assert_eq!(cc.syncretism_openness, Fixed::from_f64(0.2));
    }

    /// §8.1.18 (Iteration 165): seeding a shared village taboo set fills the
    /// previously-empty taboos vec, scales strength by traditionalism, and is
    /// deterministic (pure function of traditionalism — replay-safe).
    #[test]
    fn seed_village_taboos_populates_scales_and_is_deterministic() {
        let mut cc = CulturalCognition::default();
        assert!(cc.taboos.is_empty(), "pre-Iter-165 agents hold no taboos");
        cc.seed_village_taboos(Fixed::from_f64(0.5));
        assert_eq!(cc.taboos.len(), 7, "the shared village set has 7 taboos");
        // Sacred Incest is the strongest: 0.7 × (0.5 + 0.5×0.5) = 0.525.
        let incest = cc
            .taboos
            .iter()
            .find(|t| t.description == "Incest")
            .expect("Incest is part of the village set");
        assert!(incest.sacred);
        assert_eq!(incest.strength, Fixed::from_f64(0.525));
        // Determinism: identical traditionalism → identical profile.
        let mut cc2 = CulturalCognition::default();
        cc2.seed_village_taboos(Fixed::from_f64(0.5));
        assert_eq!(cc.taboos.len(), cc2.taboos.len());
        for (a, b) in cc.taboos.iter().zip(cc2.taboos.iter()) {
            assert_eq!(a.description, b.description);
            assert_eq!(a.strength, b.strength);
            assert_eq!(a.sacred, b.sacred);
        }
        // Traditional agents internalize taboos more strongly.
        let mut trad = CulturalCognition::default();
        trad.seed_village_taboos(Fixed::ONE);
        let mut open = CulturalCognition::default();
        open.seed_village_taboos(Fixed::ZERO);
        assert!(
            trad.max_taboo_strength() > open.max_taboo_strength(),
            "traditionalism must scale taboo strength"
        );
        // Len-stability: re-seeding dedupes by description (never grows the
        // vec) — though it reinforces strength via `add_taboo`'s dedupe path,
        // so only the length is stable, not the strengths (production seeds
        // once per agent, so this is a defensive property).
        cc.seed_village_taboos(Fixed::from_f64(0.5));
        assert_eq!(cc.taboos.len(), 7, "re-seed must not duplicate");
        // The strongest taboo is unchanged by a re-seed in strength ORDER —
        // Incest (0.7 base) still dominates after reinforce's +0.005.
        assert_eq!(cc.max_taboo_strength(), Fixed::from_f64(0.53));
    }

    /// §8.1.18 (Iteration 165): `max_taboo_strength` is the strongest raw
    /// prohibition — zero for an empty vec (identity-at-zero, the
    /// pre-Iter-165 snapshot contract).
    #[test]
    fn max_taboo_strength_is_zero_for_empty_and_picks_strongest() {
        let cc = CulturalCognition::default();
        assert_eq!(cc.max_taboo_strength(), Fixed::ZERO);
        let mut seeded = CulturalCognition::default();
        seeded.seed_village_taboos(Fixed::from_f64(0.5));
        let max = seeded.max_taboo_strength();
        assert_eq!(max, Fixed::from_f64(0.525), "Incest dominates at trad 0.5");
        // Strengthening a different taboo moves the max.
        let mut cc = CulturalCognition::default();
        cc.add_taboo(Taboo::new("Theft".into(), Fixed::from_f64(0.8), false));
        assert_eq!(cc.max_taboo_strength(), Fixed::from_f64(0.8));
    }

    /// §8.1.18 (Iteration 166): `taboo_knowledge_factor` is ONE-SIDED
    /// identity at zero (no taboo → exactly 1.0), scales down with the max
    /// taboo strength, and is floored so absorption is never fully blocked.
    #[test]
    fn taboo_knowledge_factor_is_one_sided_and_floored() {
        let rate = Fixed::from_f64(0.2);
        let floor = Fixed::from_f64(0.8);
        // Identity at zero: a taboo-free agent is unaffected.
        assert_eq!(taboo_knowledge_factor(Fixed::ZERO, rate, floor), Fixed::ONE);
        // 1 − 0.525 × 0.2 = 0.895 at the mean-seeded strength.
        assert_eq!(
            taboo_knowledge_factor(Fixed::from_f64(0.525), rate, floor),
            Fixed::from_f64(0.895)
        );
        // 1 − 0.68 × 0.2 = 0.864 at a high max taboo.
        assert_eq!(
            taboo_knowledge_factor(Fixed::from_f64(0.68), rate, floor),
            Fixed::from_f64(0.864)
        );
        // Higher taboo → lower factor (the dampening is monotonic).
        assert!(
            taboo_knowledge_factor(Fixed::from_f64(0.68), rate, floor)
                < taboo_knowledge_factor(Fixed::from_f64(0.3), rate, floor)
        );
        // Floor binds: an extreme taboo cannot push the factor below 0.8.
        assert_eq!(
            taboo_knowledge_factor(Fixed::ONE, rate, floor),
            floor,
            "the floor must prevent fully blocking absorption"
        );
        // Clamp on the upside for pathological inputs.
        assert_eq!(taboo_knowledge_factor(Fixed::from_f64(-0.5), rate, floor), Fixed::ONE);
    }

    /// §8.1.18 (Iteration 167): `taboo_violation_cost_for` returns the
    /// matching taboo's full violation cost, ZERO for empty/non-matching
    /// (identity-at-zero), is case-insensitive, and scales with
    /// traditionalism through the seeded strength.
    #[test]
    fn taboo_violation_cost_for_matches_case_insensitively_and_is_zero_at_empty() {
        // Empty vec → zero (the identity-at-zero contract for legacy saves).
        let cc = CulturalCognition::default();
        assert_eq!(cc.taboo_violation_cost_for("violence"), Fixed::ZERO);
        assert_eq!(cc.taboo_violation_cost_for("theft"), Fixed::ZERO);
        // Non-matching keyword → zero (ONE-SIDED).
        let mut seeded = CulturalCognition::default();
        seeded.seed_village_taboos(Fixed::from_f64(0.5));
        assert_eq!(seeded.taboo_violation_cost_for("fishing"), Fixed::ZERO);
        // Matching keyword returns violation_cost = strength + sacred_boost.
        // Violence (secular, base 0.45) at trad 0.5: 0.45 × 0.75 = 0.3375.
        let violence_cost = seeded.taboo_violation_cost_for("violence");
        assert_eq!(violence_cost, Fixed::from_f64(0.3375));
        // Case-insensitive: uppercase keyword matches.
        assert_eq!(seeded.taboo_violation_cost_for("VIOLENCE"), violence_cost);
        // Theft (secular, base 0.5) at trad 0.5: 0.5 × 0.75 = 0.375.
        assert_eq!(seeded.taboo_violation_cost_for("theft"), Fixed::from_f64(0.375));
        // Sacred taboo gets the sacred boost: Incest at trad 0.5 has strength
        // 0.525 and violation_cost = 0.525 + 0.3 = 0.825.
        assert_eq!(
            seeded.taboo_violation_cost_for("incest"),
            Fixed::from_f64(0.825)
        );
        // Traditional agents hold stronger taboos → higher violation cost.
        let mut trad = CulturalCognition::default();
        trad.seed_village_taboos(Fixed::ONE);
        assert!(
            trad.taboo_violation_cost_for("violence")
                > seeded.taboo_violation_cost_for("violence"),
            "higher traditionalism must raise the sacred-severity term"
        );
    }

    #[test]
    fn taboo_violation_cost_sum_sums_every_matching_taboo() {
        // §8.1.18 (Iteration 169): the summed-cost aggregation over
        // `tabo_violated_by` — the total cultural gravity of a change,
        // not just the single strongest prohibition.
        let cc = CulturalCognition::default();
        assert_eq!(cc.taboo_violation_cost_sum("violence"), Fixed::ZERO);

        let mut seeded = CulturalCognition::default();
        seeded.seed_village_taboos(Fixed::from_f64(0.5));
        // Single match → exactly that taboo's cost (Violence 0.45 × 0.75
        // = 0.3375, secular — no sacred boost).
        assert_eq!(
            seeded.taboo_violation_cost_sum("violence"),
            Fixed::from_f64(0.3375)
        );
        // Multi-match → the SUM across the whole forbidden set (Violence
        // 0.3375 + Theft 0.5 × 0.75 = 0.375 → 0.7125).
        assert_eq!(
            seeded.taboo_violation_cost_sum("violence and theft"),
            Fixed::from_f64(0.7125)
        );
        // Non-match → zero (ONE-SIDED identity).
        assert_eq!(seeded.taboo_violation_cost_sum("farming"), Fixed::ZERO);
    }
}
