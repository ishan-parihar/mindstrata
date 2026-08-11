//! §10.8: Clan system — emergent kinship-plus-alliance groups.
//!
//! Clans emerge from kinship, narrative, and repeated alliance. They are
//! not hardcoded factions but arise from:
//! - marriages connecting households,
//! - shared grievances creating solidarity,
//! - feuds creating boundaries,
//! - myths justifying identity,
//! - elders preserving honor codes.
//!
//! ```text
//! Clan dynamics:
//!   - Prestige rises with successful marriages, wealth, and myth resonance
//!   - Cohesion rises with shared kinship, mutual aid, and collective memory
//!   - Enemies form from feuds, resource competition, and cultural conflict
//!   - Allies form from marriage alliances, trade partnerships, and shared threats
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Identifier for a clan (index into the clans vec).
pub type ClanId = usize;

/// A clan — emergent group from kinship + alliance + narrative.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clan {
    /// Unique clan identifier (index).
    pub id: ClanId,
    /// Core households that belong to this clan.
    pub core_households: Vec<usize>,
    /// Agent index of the clan founder (for founding myth).
    pub founder_memory: Option<usize>,
    /// Shared ancestor that links the clan (if known).
    pub shared_ancestor: Option<usize>,
    /// Clan prestige — rises with successful marriages, wealth, and myth resonance.
    pub prestige: Fixed,
    /// Honor codes that define acceptable behavior.
    pub honor_codes: Vec<HonorCode>,
    /// Enemy clans (feud boundaries).
    pub enemies: Vec<ClanId>,
    /// Allied clans (marriage alliances, trade partnerships).
    pub allies: Vec<ClanId>,
    /// Mythical narratives that justify the clan's identity and legitimacy.
    pub myths: Vec<ClanMyth>,
    /// Internal cohesion — how well members cooperate.
    pub cohesion: Fixed,
    /// Internal grievance — unresolved conflicts within the clan.
    pub grievance: Fixed,
    /// Tick when the clan was formed.
    pub formed_tick: u64,
    /// Tick when the clan last interacted with another clan.
    pub last_interaction_tick: u64,
}

/// An honor code — behavioral expectations for clan members.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HonorCode {
    /// Description of the behavioral expectation.
    pub description: String,
    /// Strength of the code (0–1). Higher = more enforced.
    pub strength: Fixed,
    /// Punishment for violation (0–1).
    pub punishment: Fixed,
}

/// A clan myth — narrative that justifies identity and legitimacy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanMyth {
    /// Description of the myth.
    pub description: String,
    /// Belief strength across clan members (0–1).
    pub belief_strength: Fixed,
    /// Emotional charge — how much the myth motivates action.
    pub emotional_charge: Fixed,
    /// Whether the myth justifies aggression toward enemies.
    pub justifies_aggression: bool,
}

impl Clan {
    /// Create a new clan from an initial set of households.
    pub fn new(
        id: ClanId,
        founder: Option<usize>,
        shared_ancestor: Option<usize>,
        initial_households: Vec<usize>,
        tick: u64,
    ) -> Self {
        Self {
            id,
            core_households: initial_households,
            founder_memory: founder,
            shared_ancestor,
            prestige: Fixed::from_f64(0.3),
            honor_codes: Vec::new(),
            enemies: Vec::new(),
            allies: Vec::new(),
            myths: Vec::new(),
            cohesion: Fixed::from_f64(0.5),
            grievance: Fixed::ZERO,
            formed_tick: tick,
            last_interaction_tick: tick,
        }
    }

    /// Add an honor code to the clan.
    pub fn add_honor_code(&mut self, description: String, strength: Fixed, punishment: Fixed) {
        self.honor_codes.push(HonorCode {
            description,
            strength,
            punishment,
        });
    }

    /// Add a founding myth to the clan.
    pub fn add_myth(
        &mut self,
        description: String,
        belief: Fixed,
        emotional: Fixed,
        aggression: bool,
    ) {
        self.myths.push(ClanMyth {
            description,
            belief_strength: belief,
            emotional_charge: emotional,
            justifies_aggression: aggression,
        });
    }

    /// Declare enmity with another clan.
    pub fn declare_enemy(&mut self, other: ClanId) {
        if !self.enemies.contains(&other) {
            self.enemies.push(other);
            // Enmity reduces cohesion slightly (internal tension from war commitment)
            self.cohesion = (self.cohesion - Fixed::from_f64(0.05)).max(Fixed::ZERO);
        }
    }

    /// §10.8/§19.5.G: Clear an enmity when the underlying feud has fully
    /// decayed — feuds can heal into peace (and later, alliances). Restores
    /// a little cohesion (the end of a war commitment).
    pub fn clear_enemy(&mut self, other: ClanId) {
        if self.enemies.contains(&other) {
            self.enemies.retain(|&e| e != other);
            self.cohesion = (self.cohesion + Fixed::from_f64(0.03)).min(Fixed::ONE);
        }
    }

    /// Declare alliance with another clan.
    pub fn declare_ally(&mut self, other: ClanId) {
        if !self.allies.contains(&other) && !self.enemies.contains(&other) {
            self.allies.push(other);
            // Alliance boosts cohesion slightly (shared purpose)
            self.cohesion = (self.cohesion + Fixed::from_f64(0.05)).min(Fixed::ONE);
        }
    }

    /// Add a household to the clan.
    pub fn add_household(&mut self, household_id: usize) {
        if !self.core_households.contains(&household_id) {
            self.core_households.push(household_id);
        }
    }

    /// Remove a household from the clan (household dissolved or defected).
    pub fn remove_household(&mut self, household_id: usize) {
        self.core_households.retain(|&h| h != household_id);
        // Losing a household reduces cohesion
        self.cohesion = (self.cohesion - Fixed::from_f64(0.1)).max(Fixed::ZERO);
    }

    /// Daily update: decay grievance, adjust cohesion based on myth belief.
    pub fn daily_update(&mut self) {
        // Grievance decays slowly (unresolved conflicts fade over time)
        self.grievance = (self.grievance * Fixed::from_f64(0.98)).max(Fixed::ZERO);

        // Myth belief decays slowly without reinforcement
        for myth in &mut self.myths {
            myth.belief_strength = (myth.belief_strength * Fixed::from_f64(0.995)).max(Fixed::ZERO);
        }

        // Prestige decays slowly (clans must maintain reputation)
        self.prestige = (self.prestige * Fixed::from_f64(0.999)).max(Fixed::ZERO);

        // Cohesion stabilizes toward 0.5 if no external forces
        let cohesion_drift = (Fixed::from_f64(0.5) - self.cohesion) * Fixed::from_f64(0.01);
        self.cohesion = (self.cohesion + cohesion_drift).clamp_01();
    }

    /// §10.8: Daily myth resonance — a clan's living identity (myth belief ×
    /// emotional charge plus honor-code strength) reinforces cohesion, which
    /// in turn sustains prestige. Tiny deterministic coefficients; the
    /// identity layer is observational (no behavioral consumer reads these
    /// fields yet) but the myth/honor-code layer now visibly drives clan
    /// dynamics instead of sitting structurally empty.
    pub fn myth_resonance_day(&mut self) {
        let myth_strength: Fixed = self
            .myths
            .iter()
            .map(|m| m.belief_strength * m.emotional_charge)
            .fold(Fixed::ZERO, |a, b| a + b);
        let honor_strength: Fixed = self
            .honor_codes
            .iter()
            .map(|h| h.strength)
            .fold(Fixed::ZERO, |a, b| a + b);
        let identity_boost =
            myth_strength * Fixed::from_f64(0.002) + honor_strength * Fixed::from_f64(0.001);
        if identity_boost <= Fixed::ZERO {
            // No living identity → no resonance (cohesion/prestige stay put).
            return;
        }
        // Prestige is computed from the pre-boost cohesion so the two
        // effects don't double-count the same identity input.
        let cohesion_before = self.cohesion;
        self.cohesion = (self.cohesion + identity_boost).clamp_01();
        self.prestige = (self.prestige + cohesion_before * Fixed::from_f64(0.0002)).clamp_01();
    }

    /// Compute clan power — combination of prestige, cohesion, and household count.
    /// Household contribution is normalized to [0, 1] before weighting.
    pub fn power(&self) -> Fixed {
        let raw_count = Fixed::from_int(self.core_households.len() as i64);
        let household_factor = (raw_count * Fixed::from_f64(0.1)).min(Fixed::ONE);
        self.prestige * Fixed::from_f64(0.4)
            + self.cohesion * Fixed::from_f64(0.3)
            + household_factor * Fixed::from_f64(0.3)
    }

    /// Check if two clans are enemies.
    pub fn is_enemy(&self, other: ClanId) -> bool {
        self.enemies.contains(&other)
    }

    /// Check if two clans are allies.
    pub fn is_ally(&self, other: ClanId) -> bool {
        self.allies.contains(&other)
    }
}

/// Registry of all clans in the simulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClanRegistry {
    /// All clans in the simulation.
    pub clans: Vec<Clan>,
}

impl ClanRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self { clans: Vec::new() }
    }

    /// Register a new clan and return its id.
    pub fn register(&mut self, clan: Clan) -> ClanId {
        let id = self.clans.len();
        self.clans.push(clan);
        id
    }

    /// Find a clan by id.
    pub fn get(&self, id: ClanId) -> Option<&Clan> {
        self.clans.get(id)
    }

    /// Find a clan by id (mutable).
    pub fn get_mut(&mut self, id: ClanId) -> Option<&mut Clan> {
        self.clans.get_mut(id)
    }

    /// Daily update for all clans.
    pub fn daily_update(&mut self) {
        for clan in &mut self.clans {
            clan.daily_update();
        }
    }

    /// §10.8: Daily myth resonance for all clans (call after `daily_update`).
    pub fn myth_resonance_day(&mut self) {
        for clan in &mut self.clans {
            clan.myth_resonance_day();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clan_creation_structure() {
        let mut clan = Clan::new(0, Some(0), Some(0), vec![0, 1], 0);
        clan.add_honor_code(
            "Do not steal".into(),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.5),
        );
        clan.add_myth(
            "We descend from the mountain kings".into(),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.6),
            false,
        );

        assert_eq!(clan.core_households, vec![0, 1]);
        assert_eq!(clan.honor_codes.len(), 1);
        assert_eq!(clan.myths.len(), 1);
        assert_eq!(clan.prestige, Fixed::from_f64(0.3));
        assert_eq!(clan.cohesion, Fixed::from_f64(0.5));
    }

    #[test]
    fn clan_enemy_declaration() {
        let mut clan_a = Clan::new(0, None, None, vec![0], 0);
        let mut clan_b = Clan::new(1, None, None, vec![1], 0);

        clan_a.declare_enemy(1);
        clan_b.declare_enemy(0);

        assert!(clan_a.is_enemy(1));
        assert!(clan_b.is_enemy(0));
        assert!(!clan_a.is_enemy(0)); // no self-enmity
                                      // Enmity reduces cohesion
        assert!(clan_a.cohesion < Fixed::from_f64(0.5));
    }

    #[test]
    fn clan_ally_declaration() {
        let mut clan_a = Clan::new(0, None, None, vec![0], 0);

        clan_a.declare_ally(1);
        assert!(clan_a.is_ally(1));
        // Alliance boosts cohesion
        assert!(clan_a.cohesion > Fixed::from_f64(0.5));
    }

    #[test]
    fn clan_cannot_ally_with_enemy() {
        let mut clan_a = Clan::new(0, None, None, vec![0], 0);
        clan_a.declare_enemy(1);
        clan_a.declare_ally(1);
        // Should not be an ally if already an enemy
        assert!(!clan_a.is_ally(1));
        assert!(clan_a.is_enemy(1));
    }

    #[test]
    fn clan_daily_update_decay() {
        let mut clan = Clan::new(0, None, None, vec![0], 0);
        clan.grievance = Fixed::from_f64(0.5);
        clan.prestige = Fixed::from_f64(0.8);
        clan.cohesion = Fixed::from_f64(0.3);

        clan.daily_update();

        // Grievance should decay
        assert!(clan.grievance < Fixed::from_f64(0.5));
        // Prestige should decay
        assert!(clan.prestige < Fixed::from_f64(0.8));
        // Cohesion should drift toward 0.5
        assert!(clan.cohesion > Fixed::from_f64(0.3));
    }

    #[test]
    fn clan_power_computation() {
        let mut clan = Clan::new(0, None, None, vec![0, 1, 2], 0);
        clan.prestige = Fixed::from_f64(0.8);
        clan.cohesion = Fixed::from_f64(0.7);

        let power = clan.power();
        assert!(
            power > Fixed::from_f64(0.3),
            "Clan with prestige and cohesion should have significant power"
        );
    }

    #[test]
    fn clan_household_management() {
        let mut clan = Clan::new(0, None, None, vec![0], 0);
        clan.add_household(1);
        clan.add_household(2);
        assert_eq!(clan.core_households.len(), 3);

        // Adding duplicate should not create duplicate
        clan.add_household(1);
        assert_eq!(clan.core_households.len(), 3);

        clan.remove_household(1);
        assert_eq!(clan.core_households.len(), 2);
        assert!(!clan.core_households.contains(&1));
        // Removing a household reduces cohesion
        assert!(clan.cohesion < Fixed::from_f64(0.5));
    }

    #[test]
    fn clan_registry_operations() {
        let mut registry = ClanRegistry::new();
        let clan_a = Clan::new(0, Some(0), None, vec![0], 0);
        let clan_b = Clan::new(1, Some(1), None, vec![1], 0);

        let id_a = registry.register(clan_a);
        let id_b = registry.register(clan_b);

        assert_eq!(id_a, 0);
        assert_eq!(id_b, 1);
        assert!(registry.get(0).is_some());
        assert!(registry.get(1).is_some());
        assert!(registry.get(2).is_none());

        // Daily update should not panic
        registry.daily_update();
    }

    #[test]
    fn myth_belief_decay() {
        let mut clan = Clan::new(0, None, None, vec![0], 0);
        clan.add_myth(
            "Test myth".into(),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.5),
            false,
        );

        clan.daily_update();

        let myth = &clan.myths[0];
        assert!(
            myth.belief_strength < Fixed::from_f64(0.8),
            "Myth belief should decay without reinforcement"
        );
    }

    // ── §10.8 Clan identity (myth resonance) tests ────────────────────

    /// Living myths and honored codes visibly lift cohesion and prestige.
    #[test]
    fn clan_myth_resonance_lifts_cohesion_and_prestige() {
        let mut clan = Clan::new(0, Some(0), None, vec![0], 0);
        clan.add_myth(
            "The river sustains us".into(),
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.8),
            false,
        );
        clan.add_honor_code(
            "Honor the elders".into(),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.4),
        );
        let cohesion_before = clan.cohesion;
        let prestige_before = clan.prestige;
        clan.myth_resonance_day();
        assert!(
            clan.cohesion > cohesion_before,
            "myth resonance must lift cohesion"
        );
        assert!(
            clan.prestige > prestige_before,
            "myth resonance must lift prestige"
        );
        // Still bounded.
        assert!(clan.cohesion <= Fixed::ONE && clan.prestige <= Fixed::ONE);
    }

    /// A clan with no identity layer has no resonance (cohesion AND prestige
    /// stay put — prestige must not drift without a living myth).
    #[test]
    fn clan_without_identity_has_no_resonance() {
        let mut clan = Clan::new(0, None, None, vec![0], 0);
        let cohesion_before = clan.cohesion;
        let prestige_before = clan.prestige;
        clan.myth_resonance_day();
        assert_eq!(clan.cohesion, cohesion_before);
        assert_eq!(clan.prestige, prestige_before);
    }
}
