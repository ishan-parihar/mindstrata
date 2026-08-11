//! Ritual system — structured ceremonial activities that create social cohesion.
//!
//! Rituals are a core technology of social bonding. They synchronize emotions,
//! reinforce norms, create collective effervescence, and mark life transitions.
//!
//! ```text
//! Ritual effects:
//!   - Increases bonding between participants
//!   - Reduces anxiety through predictable structure
//!   - Reinforces norms through repetition
//!   - Creates collective effervescence (shared emotional peak)
//!   - Strengthens institutional legitimacy
//!   - Marks life transitions (birth, marriage, death)
//!   - Encodes cultural memory through repetition
//! ```
//!
//! Emergent effects:
//! - Regular rituals stabilize group cohesion
//! - Ritual failure (e.g., priest corruption) undermines legitimacy
//! - Forced rituals create resentment
//! - Shared rituals bridge household/clan boundaries
//! - Novel rituals emerge from crisis

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Type of ritual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RitualKind {
    /// Harvest festival or thanksgiving.
    HarvestFestival,
    /// Funeral or mourning rite.
    Funeral,
    /// Naming ceremony for newborns.
    Naming,
    /// Marriage ceremony.
    Marriage,
    /// Oath-swearing or covenant.
    OathSwearing,
    /// Public confession or penance.
    Confession,
    /// Excommunication or banishment.
    Excommunication,
    /// Seasonal prayer or observance.
    SeasonalPrayer,
    /// Communal meal.
    CommunalMeal,
    /// Sacrifice (abstract — offering of resources).
    Sacrifice,
}

/// A ritual event or recurring practice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ritual {
    /// Unique ritual identifier.
    pub id: usize,
    /// Type of ritual.
    pub kind: RitualKind,
    /// Human-readable name.
    pub name: String,
    /// Index of the institution sponsoring the ritual.
    pub sponsor: usize,
    /// Indices of participating agents.
    pub participants: Vec<usize>,
    /// Emotional intensity of the ritual (0 = bland, 1 = ecstatic).
    pub emotional_intensity: Fixed,
    /// Degree of behavioral synchrony (0 = chaotic, 1 = perfectly synchronized).
    pub synchrony: Fixed,
    /// Sacredness of the ritual (0 = mundane, 1 = deeply sacred).
    pub sacredness: Fixed,
    /// Identity relevance to participants (0 = irrelevant, 1 = core identity).
    pub identity_relevance: Fixed,
    /// Resource cost of the ritual (per participant).
    pub cost: Fixed,
    /// Bonding effectiveness (0 = no bonding, 1 = maximum bonding).
    pub bonding_effect: Fixed,
    /// Norm reinforcement strength.
    pub norm_reinforcement: Fixed,
    /// Tick when this ritual occurs or last occurred.
    pub last_occurrence: u64,
    /// Interval between occurrences (0 = one-shot).
    pub interval: u64,
    /// Whether this ritual is currently active.
    pub active: bool,
}

impl Ritual {
    /// §12.5 (Iteration 90): institutions declare the norms their collective
    /// life upholds (`Institution.norm_ids`); a ritual hosted by such an
    /// institution reinforces its declared norms preferentially. 1.5× keeps
    /// the temple-declared norm clearly ahead of the base-reinforced norms
    /// (Obey Ruler 0.4 internalization → 0.138/fire vs Respect Elders 0.5 →
    /// 0.115/fire) while staying far below the 1.0 `reinforce_norm` clamp
    /// across the 30K horizon.
    pub const INSTITUTIONAL_NORM_REINFORCEMENT_MULTIPLIER: Fixed = Fixed::from_raw(15_000); // 1.5

    /// Create a new ritual.
    pub fn new(
        id: usize,
        kind: RitualKind,
        name: String,
        sponsor: usize,
        emotional_intensity: Fixed,
        sacredness: Fixed,
        cost: Fixed,
        interval: u64,
        tick: u64,
    ) -> Self {
        let bonding_effect = (emotional_intensity + sacredness) * Fixed::from_f64(0.5);
        let norm_reinforcement =
            sacredness * Fixed::from_f64(0.6) + emotional_intensity * Fixed::from_f64(0.4);
        Self {
            id,
            kind,
            name,
            sponsor,
            participants: Vec::new(),
            emotional_intensity,
            synchrony: Fixed::from_f64(0.5),
            sacredness,
            identity_relevance: sacredness * Fixed::from_f64(0.8),
            cost,
            bonding_effect,
            norm_reinforcement,
            last_occurrence: tick,
            interval,
            active: true,
        }
    }

    /// Compute the bonding effect of this ritual on a pair of participants.
    ///
    /// ```text
    /// bonding = bonding_effect × synchrony × (1 + shared_identity)
    /// ```
    pub fn pair_bonding(&self, shared_identity: Fixed) -> Fixed {
        (self.bonding_effect * self.synchrony * (Fixed::ONE + shared_identity)).clamp_01()
    }

    /// Compute norm reinforcement for a single participant.
    pub fn norm_reinforcement_for(&self, internalization: Fixed) -> Fixed {
        self.norm_reinforcement * internalization
    }

    /// §12.5/§19.5.D (Iteration 90): reinforcement honoring the sponsor
    /// institution's declared norms (`Institution.norm_ids` — the default
    /// temple declares "Obey Ruler" = 3, the field's documented purpose
    /// "Obey Ruler norm reinforced by temple" that had zero consumers).
    /// Declared norms are reinforced preferentially (× the institutional
    /// multiplier); everything else keeps the base reinforcement. Pure and
    /// deterministic — no RNG, so the golden baseline is untouched (the
    /// block only runs at monthly ritual fires, beyond every snapshot
    /// horizon; and the temple-declared norm currently has no behavioral
    /// consumer, so the strength change is observational only).
    pub fn norm_reinforcement_for_institutional(
        &self,
        internalization: Fixed,
        declared: bool,
    ) -> Fixed {
        let base = self.norm_reinforcement_for(internalization);
        if declared {
            base * Self::INSTITUTIONAL_NORM_REINFORCEMENT_MULTIPLIER
        } else {
            base
        }
    }

    /// Is this ritual due to occur at the given tick?
    pub fn is_due(&self, tick: u64) -> bool {
        if self.interval == 0 {
            false // one-shot, already occurred
        } else {
            tick >= self.last_occurrence + self.interval
        }
    }

    /// Execute the ritual (update last_occurrence, return bonding delta).
    pub fn execute(&mut self, tick: u64) -> Fixed {
        self.last_occurrence = tick;
        self.bonding_effect
    }
}

/// Registry of all rituals in the simulation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RitualRegistry {
    /// All rituals.
    pub rituals: Vec<Ritual>,
    /// Next available ritual id.
    next_id: usize,
}

impl RitualRegistry {
    /// Register a new ritual and return its id.
    pub fn register(&mut self, ritual: Ritual) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let mut ritual = ritual;
        ritual.id = id;
        self.rituals.push(ritual);
        id
    }

    /// Get a ritual by id.
    pub fn get(&self, id: usize) -> Option<&Ritual> {
        self.rituals.iter().find(|r| r.id == id)
    }

    /// Get all rituals due at the current tick.
    pub fn due_rituals(&self, tick: u64) -> Vec<&Ritual> {
        self.rituals
            .iter()
            .filter(|r| r.active && r.is_due(tick))
            .collect()
    }

    /// Number of active rituals.
    pub fn active_count(&self) -> usize {
        self.rituals.iter().filter(|r| r.active).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ritual_has_sane_defaults() {
        let r = Ritual::new(
            0,
            RitualKind::HarvestFestival,
            "Harvest".into(),
            0,
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.1),
            144,
            0,
        );
        assert_eq!(r.id, 0);
        assert!(r.active);
        assert!(r.bonding_effect > Fixed::ZERO);
    }

    #[test]
    fn pair_bonding_scales_with_synchrony() {
        let r = Ritual::new(
            0,
            RitualKind::CommunalMeal,
            "Meal".into(),
            0,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.05),
            0,
            0,
        );
        let low_sync = r.pair_bonding(Fixed::ZERO);
        let mut r2 = r;
        r2.synchrony = Fixed::ONE;
        let high_sync = r2.pair_bonding(Fixed::ZERO);
        assert!(high_sync > low_sync);
    }

    #[test]
    fn is_due_after_interval() {
        let r = Ritual::new(
            0,
            RitualKind::SeasonalPrayer,
            "Prayer".into(),
            0,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.05),
            144,
            0,
        );
        assert!(!r.is_due(50));
        assert!(r.is_due(144));
    }

    #[test]
    fn execute_updates_last_occurrence() {
        let mut r = Ritual::new(
            0,
            RitualKind::SeasonalPrayer,
            "Prayer".into(),
            0,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.05),
            144,
            0,
        );
        r.execute(100);
        assert_eq!(r.last_occurrence, 100);
    }

    #[test]
    fn registry_register_and_get() {
        let mut reg = RitualRegistry::default();
        let r = Ritual::new(
            0,
            RitualKind::HarvestFestival,
            "Harvest".into(),
            0,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.05),
            144,
            0,
        );
        let id = reg.register(r);
        assert_eq!(id, 0);
        assert!(reg.get(0).is_some());
    }

    #[test]
    fn due_rituals_returns_only_due() {
        let mut reg = RitualRegistry::default();
        let r1 = Ritual::new(
            0,
            RitualKind::SeasonalPrayer,
            "Prayer".into(),
            0,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.05),
            100,
            0,
        );
        let r2 = Ritual::new(
            0,
            RitualKind::CommunalMeal,
            "Meal".into(),
            0,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.05),
            200,
            0,
        );
        reg.register(r1);
        reg.register(r2);
        let due = reg.due_rituals(150);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].kind, RitualKind::SeasonalPrayer);
    }

    /// §19.5.D (Iteration 90): a norm declared by the sponsor institution
    /// (`Institution.norm_ids`) is reinforced preferentially — exactly the
    /// institutional multiplier on top of the base; a non-declared norm
    /// keeps the base reinforcement (legacy identity); zero internalization
    /// yields zero either way.
    #[test]
    fn institutional_norm_reinforcement_weights_declared_norms() {
        let ritual = Ritual::new(
            0,
            RitualKind::SeasonalPrayer,
            "Seasonal Prayer".into(),
            1,
            Fixed::from_f64(0.2),
            Fixed::from_f64(0.25),
            Fixed::from_f64(0.05),
            4320,
            0,
        );
        let base = ritual.norm_reinforcement_for(Fixed::from_f64(0.4));
        let declared = ritual.norm_reinforcement_for_institutional(Fixed::from_f64(0.4), true);
        let undeclared = ritual.norm_reinforcement_for_institutional(Fixed::from_f64(0.4), false);
        assert!(base > Fixed::ZERO, "the seeded ritual must reinforce");
        assert_eq!(
            undeclared, base,
            "a non-declared norm keeps the legacy base reinforcement"
        );
        assert_eq!(
            declared,
            base * Ritual::INSTITUTIONAL_NORM_REINFORCEMENT_MULTIPLIER,
            "a declared norm gets the institutional multiplier on top of base"
        );
        assert!(
            declared > base,
            "the multiplier must strictly increase the reinforcement"
        );
        assert_eq!(
            ritual.norm_reinforcement_for_institutional(Fixed::ZERO, true),
            Fixed::ZERO,
            "zero internalization stays zero even when declared"
        );
    }
}
