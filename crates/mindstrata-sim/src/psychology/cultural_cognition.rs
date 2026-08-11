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
}
