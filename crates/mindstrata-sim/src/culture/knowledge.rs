//! Cultural knowledge — §19.5.I of the architecture spec.
//!
//! Implements practices, rituals, education, knowledge diffusion,
//! taboos, and ideological traditions.
//!
//! §19.5.I: "Practices, rituals, symbols, taboos, education,
//! apprenticeship, innovation, knowledge diffusion, ideological traditions."

#![allow(missing_docs)]

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A cultural practice that agents can participate in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Practice {
    /// Unique identifier for this practice.
    pub id: u64,
    /// Name of the practice.
    pub name: String,
    /// Category of practice.
    pub category: PracticeCategory,
    /// How widespread this practice is (0 = rare, 1 = universal).
    pub prevalence: Fixed,
    /// How strongly agents feel about this practice (0 = indifferent, 1 = fervent).
    pub emotional_charge: Fixed,
    /// Whether this practice is a taboo (forbidden).
    pub is_taboo: bool,
    /// Required resources or conditions to perform this practice.
    pub requirements: Vec<String>,
    /// Effects of performing this practice on agents.
    pub effects: Vec<PracticeEffect>,
}

impl Practice {
    /// Create a new practice.
    pub fn new(id: u64, name: String, category: PracticeCategory) -> Self {
        Self {
            id,
            name,
            category,
            prevalence: Fixed::from_f64(0.3),
            emotional_charge: Fixed::from_f64(0.5),
            is_taboo: false,
            requirements: Vec::new(),
            effects: Vec::new(),
        }
    }
}

/// Categories of cultural practices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PracticeCategory {
    /// Religious or spiritual rituals.
    Ritual,
    /// Social customs and traditions.
    Custom,
    /// Educational practices.
    Education,
    /// Economic practices.
    Economic,
    /// Political practices.
    Political,
    /// Artistic or creative practices.
    Artistic,
    /// Taboo (forbidden) practices.
    Taboo,
}

impl PracticeCategory {
    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            PracticeCategory::Ritual => "Ritual",
            PracticeCategory::Custom => "Custom",
            PracticeCategory::Education => "Education",
            PracticeCategory::Economic => "Economic",
            PracticeCategory::Political => "Political",
            PracticeCategory::Artistic => "Artistic",
            PracticeCategory::Taboo => "Taboo",
        }
    }
}

/// Effects of performing a cultural practice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticeEffect {
    /// What is affected.
    pub target: EffectTarget,
    /// Magnitude of the effect (positive = beneficial, negative = harmful).
    pub magnitude: Fixed,
}

/// What a practice effect targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectTarget {
    /// Affects agent's meaning need.
    Meaning,
    /// Affects agent's social need.
    Social,
    /// Affects agent's trust in others.
    Trust,
    /// Affects agent's morale.
    Morale,
    /// Affects agent's knowledge/skills.
    Knowledge,
    /// Affects agent's identity.
    Identity,
    /// Affects institutional legitimacy.
    Legitimacy,
}

/// Knowledge that agents can learn and share.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Knowledge {
    /// Unique identifier.
    pub id: u64,
    /// Name of the knowledge.
    pub name: String,
    /// Category of knowledge.
    pub category: KnowledgeCategory,
    /// How difficult this knowledge is to acquire (0 = easy, 1 = very hard).
    pub difficulty: Fixed,
    /// How useful this knowledge is (0 = useless, 1 = essential).
    pub utility: Fixed,
    /// How many agents know this knowledge.
    pub holders: u32,
    /// Tick when this knowledge was first discovered.
    pub discovered_tick: u64,
}

impl Knowledge {
    /// Create new knowledge.
    pub fn new(id: u64, name: String, category: KnowledgeCategory) -> Self {
        Self {
            id,
            name,
            category,
            difficulty: Fixed::from_f64(0.5),
            utility: Fixed::from_f64(0.5),
            holders: 0,
            discovered_tick: 0,
        }
    }
}

/// Categories of knowledge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KnowledgeCategory {
    /// Agricultural knowledge.
    Agricultural,
    /// Craft or trade knowledge.
    Craft,
    /// Medical knowledge.
    Medical,
    /// Military knowledge.
    Military,
    /// Religious or philosophical knowledge.
    Philosophical,
    /// Social knowledge (etiquette, customs).
    Social,
    /// Political knowledge (governance, law).
    Political,
}

/// Ideological tradition that agents can follow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ideology {
    /// Unique identifier.
    pub id: u64,
    /// Name of the ideology.
    pub name: String,
    /// Core values of this ideology.
    pub values: Vec<IdeologyValue>,
    /// How many followers this ideology has.
    pub followers: u32,
    /// How rigidly this ideology is held (0 = flexible, 1 = dogmatic).
    pub rigidity: Fixed,
    /// How much this ideology conflicts with other ideologies.
    pub conflict_level: Fixed,
}

impl Ideology {
    /// Create a new ideology.
    pub fn new(id: u64, name: String) -> Self {
        Self {
            id,
            name,
            values: Vec::new(),
            followers: 0,
            rigidity: Fixed::from_f64(0.5),
            conflict_level: Fixed::from_f64(0.3),
        }
    }
}

/// A value within an ideology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeologyValue {
    /// Name of the value.
    pub name: String,
    /// How strongly this value is held (0 = weakly, 1 = strongly).
    pub strength: Fixed,
    /// Whether this value conflicts with other values.
    pub is_conflicting: bool,
}

/// Cultural state for tracking cultural evolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CulturalState {
    /// Practices known by this agent.
    pub practices: Vec<u64>,
    /// Knowledge held by this agent.
    pub knowledge: Vec<u64>,
    /// Ideology followed by this agent.
    pub ideology: Option<u64>,
    /// How traditional this agent is (0 = progressive, 1 = traditional).
    pub traditionalism: Fixed,
    /// How open to new ideas (0 = closed, 1 = open).
    pub openness: Fixed,
}

impl Default for CulturalState {
    fn default() -> Self {
        Self {
            practices: Vec::new(),
            knowledge: Vec::new(),
            ideology: None,
            traditionalism: Fixed::from_f64(0.5),
            openness: Fixed::from_f64(0.5),
        }
    }
}

/// Result of knowledge diffusion between agents.
#[derive(Debug, Clone)]
pub struct DiffusionResult {
    /// Knowledge that was shared.
    pub knowledge_id: u64,
    /// Whether the transfer was successful.
    pub accepted: bool,
    /// How much knowledge was transferred (0 = none, 1 = full).
    pub fidelity: Fixed,
}

/// Attempt to share knowledge between two agents.
///
/// §19.5.I: "Knowledge diffusion" — knowledge spreads through social networks.
pub fn diffuse_knowledge(
    teacher_knowledge: &Knowledge,
    teacher_skill: Fixed,    // teacher's ability to teach
    student_openness: Fixed, // student's willingness to learn
    relationship_trust: Fixed,
    student_existing_knowledge: &[u64],
) -> DiffusionResult {
    // Already know this
    if student_existing_knowledge.contains(&teacher_knowledge.id) {
        return DiffusionResult {
            knowledge_id: teacher_knowledge.id,
            accepted: false,
            fidelity: Fixed::ZERO,
        };
    }

    // Probability of acceptance
    let difficulty_penalty = Fixed::ONE - teacher_knowledge.difficulty;
    let acceptance_chance = (teacher_skill * Fixed::from_f64(0.3)
        + student_openness * Fixed::from_f64(0.3)
        + relationship_trust * Fixed::from_f64(0.2)
        + difficulty_penalty * Fixed::from_f64(0.2))
    .clamp_01();

    // Fidelity depends on teacher skill and difficulty
    let fidelity = (teacher_skill * Fixed::from_f64(0.5)
        + (Fixed::ONE - teacher_knowledge.difficulty) * Fixed::from_f64(0.5))
    .clamp_01();

    DiffusionResult {
        knowledge_id: teacher_knowledge.id,
        accepted: acceptance_chance > Fixed::from_f64(0.5),
        fidelity,
    }
}

/// Compute cultural pressure on an agent to conform to a practice.
pub fn cultural_pressure(
    practice_prevalence: Fixed,
    practice_emotional_charge: Fixed,
    agent_traditionalism: Fixed,
    agent_openness: Fixed,
) -> Fixed {
    // Traditional agents feel more pressure to conform
    let conformity_pressure = practice_prevalence * agent_traditionalism;
    // Open agents feel less pressure
    let resistance = agent_openness * Fixed::from_f64(0.3);
    // Emotional charge increases pressure
    let emotional_pressure = practice_emotional_charge * Fixed::from_f64(0.2);

    (conformity_pressure - resistance + emotional_pressure).clamp_01()
}

/// Compute taboo violation severity.
pub fn taboo_severity(
    practice: &Practice,
    agent_traditionalism: Fixed,
    witness_count: u32,
) -> Fixed {
    if !practice.is_taboo {
        return Fixed::ZERO;
    }

    let base_severity = practice.emotional_charge;
    let traditionalism_boost = agent_traditionalism * Fixed::from_f64(0.3);
    let witness_multiplier =
        Fixed::ONE + Fixed::from_int(witness_count as i64) * Fixed::from_f64(0.1);

    (base_severity + traditionalism_boost) * witness_multiplier * Fixed::from_f64(0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn practice_creation() {
        let p = Practice::new(0, "Harvest Festival".into(), PracticeCategory::Ritual);
        assert_eq!(p.name, "Harvest Festival");
        assert_eq!(p.category, PracticeCategory::Ritual);
        assert!(!p.is_taboo);
    }

    #[test]
    fn knowledge_creation() {
        let k = Knowledge::new(0, "Crop Rotation".into(), KnowledgeCategory::Agricultural);
        assert_eq!(k.name, "Crop Rotation");
        assert_eq!(k.holders, 0);
    }

    #[test]
    fn ideology_creation() {
        let i = Ideology::new(0, "Harvest Worship".into());
        assert_eq!(i.name, "Harvest Worship");
        assert_eq!(i.followers, 0);
    }

    #[test]
    fn knowledge_diffusion_works() {
        let knowledge = Knowledge::new(0, "Fire Making".into(), KnowledgeCategory::Craft);
        let result = diffuse_knowledge(
            &knowledge,
            Fixed::from_f64(0.8), // good teacher
            Fixed::from_f64(0.7), // open student
            Fixed::from_f64(0.6), // decent trust
            &[],
        );
        assert!(
            result.accepted,
            "Knowledge should be transferred to open student"
        );
        assert!(result.fidelity > Fixed::ZERO);
    }

    #[test]
    fn knowledge_diffusion_rejects_already_known() {
        let knowledge = Knowledge::new(0, "Fire Making".into(), KnowledgeCategory::Craft);
        let result = diffuse_knowledge(
            &knowledge,
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.6),
            &[0], // already knows this
        );
        assert!(
            !result.accepted,
            "Should not transfer already known knowledge"
        );
    }

    #[test]
    fn cultural_pressure_traditional_agents() {
        let pressure = cultural_pressure(
            Fixed::from_f64(0.8), // widespread practice
            Fixed::from_f64(0.6), // emotionally charged
            Fixed::from_f64(0.9), // very traditional agent
            Fixed::from_f64(0.2), // not very open
        );
        assert!(
            pressure > Fixed::from_f64(0.5),
            "Traditional agents should feel strong cultural pressure: {}",
            pressure.to_f64()
        );
    }

    #[test]
    fn cultural_pressure_open_agents() {
        let pressure = cultural_pressure(
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.2), // not traditional
            Fixed::from_f64(0.9), // very open
        );
        assert!(
            pressure < Fixed::from_f64(0.5),
            "Open agents should feel less cultural pressure: {}",
            pressure.to_f64()
        );
    }

    #[test]
    fn taboo_severity_increases_with_witnesses() {
        let practice = Practice {
            id: 0,
            name: "Forbidden Ritual".into(),
            category: PracticeCategory::Taboo,
            prevalence: Fixed::from_f64(0.1),
            emotional_charge: Fixed::from_f64(0.8),
            is_taboo: true,
            requirements: Vec::new(),
            effects: Vec::new(),
        };

        let severity_no_witness = taboo_severity(&practice, Fixed::from_f64(0.5), 0);
        let severity_with_witnesses = taboo_severity(&practice, Fixed::from_f64(0.5), 5);

        assert!(
            severity_with_witnesses > severity_no_witness,
            "More witnesses should increase taboo severity"
        );
    }

    #[test]
    fn non_taboo_has_zero_severity() {
        let practice = Practice::new(0, "Normal Practice".into(), PracticeCategory::Custom);
        let severity = taboo_severity(&practice, Fixed::from_f64(0.5), 10);
        assert_eq!(
            severity,
            Fixed::ZERO,
            "Non-taboo practices should have zero severity"
        );
    }

    #[test]
    fn cultural_state_default() {
        let state = CulturalState::default();
        assert!(state.practices.is_empty());
        assert!(state.knowledge.is_empty());
        assert!(state.ideology.is_none());
    }
}
