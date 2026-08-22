//! Household system — primary social and economic unit.
//!
//! Households emerge from kinship, marriage, and co-residence.
//! They pool resources, share labor, raise children, and care for elders.
//!
//! ```text
//! Household dynamics:
//!   - Resource pooling (shared food, wealth, tools)
//!   - Division of labor (cooking, farming, crafting, childcare)
//!   - Domestic conflict (disputes over resources, roles, authority)
//!   - Inheritance (property passes through household lines)
//!   - Hospitality (hosting outsiders builds reputation)
//!   - Shame/pride (household reputation affects all members)
//! ```
//!
//! A household has:
//! - Members (agent indices)
//! - Head (decision-maker, usually eldest or wealthiest)
//! - Residence (site index)
//! - Pooled resources
//! - Cohesion (how well members cooperate)
//! - Conflict (internal tension)
//! - Reputation (how the settlement views them)
//!
//! Emergent effects:
//! - Households compete for resources and status
//! - Domestic abuse creates trauma and flight
//! - Inheritance disputes fracture families
//! - Large households gain economic advantage
//! - Household reputation affects marriage prospects

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// §10.7 (AP2): Household food-pooling constants (Iteration 119).
///
/// `Simulation::tick_household_food_pooling` uses these to make the plan's
/// "resource pooling" and "division of labor" dimensions decisional (see the
/// fold's doc for the full narrative). Values are deliberately small — the
/// fold is an emergency buffer for hungry households, not a full subsistence
/// model.
/// - `HOUSEHOLD_POOL_RATE`: an adult's daily surplus contribution, scaled by
///   how far they sit below the feed threshold (≈0.005/day at half hunger).
/// - `HOUSEHOLD_FEED_RELIEF`: a dependent's daily ration (hunger relief).
/// - `HOUSEHOLD_ADULT_FEED_RATIO`: hungry adults receive this fraction of the
///   dependent ration from the residual pot — dependents eat first and
///   better (childcare/elder care).
/// - `HOUSEHOLD_HUNGER_FEED_THRESHOLD`: only members at or above this hunger
///   are fed. Calibrated windows sit at ≤0.33 max hunger; malnutrition
///   starts at 0.7 — the fold engages only genuinely hungry members.
pub const HOUSEHOLD_POOL_RATE: f64 = 0.02;
/// Const. (doc added at S3 extraction)
pub const HOUSEHOLD_FEED_RELIEF: f64 = 0.1;
/// Const. (doc added at S3 extraction)
pub const HOUSEHOLD_ADULT_FEED_RATIO: f64 = 0.5;
/// Const. (doc added at S3 extraction)
pub const HOUSEHOLD_HUNGER_FEED_THRESHOLD: f64 = 0.35;

/// Role within a household.
///
/// §10.7 (AP2): the plan's `Household` carries `roles: Vec<HouseholdRole>`;
/// this enum was declared but never stored on a household (dead code) until
/// Iteration 53 wired it — roles are derived deterministically from agent
/// state each daily pass and stored parallel to `members`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HouseholdRole {
    /// Head of household (decision-maker).
    Head,
    /// Co-head or spouse.
    Partner,
    /// Adult member.
    Adult,
    /// Dependent child.
    Child,
    /// Elder (retired, respected).
    Elder,
    /// Servant or dependent.
    Dependent,
}

/// A household — the primary social and economic unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Household {
    /// Unique household identifier.
    pub id: usize,
    /// Agent indices of members.
    pub members: Vec<usize>,
    /// Index of the household head (decision-maker).
    pub head: Option<usize>,
    /// Site index where the household resides.
    pub residence: Option<usize>,
    /// Pooled food reserves.
    pub food_reserves: Fixed,
    /// Pooled coin/wealth.
    pub wealth_reserves: Fixed,
    /// Internal cohesion (0 = dysfunctional, 1 = perfectly cooperative).
    pub cohesion: Fixed,
    /// Internal conflict (0 = peaceful, 1 = at war with each other).
    pub conflict: Fixed,
    /// Reputation in the settlement (0 = notorious, 1 = honored).
    pub reputation: Fixed,
    /// Tick when household was formed.
    pub founded_tick: u64,
    /// §10.7 (AP2): Role of each member, parallel to `members` — the plan's
    /// division-of-labor dimension. Derived deterministically from agent
    /// state (head, partner, age) on the daily pass; `#[serde(default)]` so
    /// pre-Iter-53 saves restore.
    #[serde(default)]
    pub roles: Vec<HouseholdRole>,
    /// §10.7 (AP2): PracticeIds this household collectively maintains — the
    /// plan's `traditions` dimension. The deterministic union of members'
    /// known practices (sorted), refreshed on the daily pass.
    #[serde(default)]
    pub traditions: Vec<u64>,
}

impl Household {
    /// Create a new household with a single founding member.
    pub fn new(founder: usize, residence: Option<usize>, tick: u64) -> Self {
        Self {
            id: 0, // assigned externally
            members: vec![founder],
            head: Some(founder),
            residence,
            food_reserves: Fixed::from_f64(10.0),
            wealth_reserves: Fixed::from_f64(5.0),
            cohesion: Fixed::from_f64(0.6),
            conflict: Fixed::ZERO,
            reputation: Fixed::from_f64(0.5),
            founded_tick: tick,
            roles: vec![HouseholdRole::Head],
            traditions: Vec::new(),
        }
    }

    /// Add a member to the household (default role: Adult).
    pub fn add_member(&mut self, agent: usize) {
        if !self.members.contains(&agent) {
            self.members.push(agent);
            self.roles.push(HouseholdRole::Adult);
        }
    }

    /// Remove a member from the household (and their role).
    pub fn remove_member(&mut self, agent: usize) {
        if let Some(pos) = self.members.iter().position(|&m| m == agent) {
            self.members.remove(pos);
            self.roles.remove(pos);
        }
        if self.head == Some(agent) {
            self.head = self.members.first().copied();
        }
    }

    /// §10.7 (AP2): Deterministically derive each member's role from agent
    /// state — head → Head, head's spouse → Partner, minors → Child, elders
    /// (>55) → Elder, everyone else Adult. Writes only the new `roles` field
    /// from a pure function of existing state (no RNG), so calibrated runs
    /// stay byte-identical.
    pub fn derive_roles(&mut self, ages: &[Fixed], partners: &[Option<usize>]) {
        let mut roles = Vec::with_capacity(self.members.len());
        for &member in &self.members {
            let role = if self.head == Some(member) {
                HouseholdRole::Head
            } else if self.head.is_some() && partners.get(member) == Some(&self.head) {
                HouseholdRole::Partner
            } else if ages
                .get(member)
                .is_some_and(|age| *age < Fixed::from_f64(14.0))
            {
                HouseholdRole::Child
            } else if ages
                .get(member)
                .is_some_and(|age| *age > Fixed::from_f64(55.0))
            {
                HouseholdRole::Elder
            } else {
                HouseholdRole::Adult
            };
            roles.push(role);
        }
        self.roles = roles;
    }

    /// §10.7 (AP2): Deterministically recompute the household's traditions as
    /// the sorted union of members' known practice ids — the plan's
    /// `traditions: Vec<PracticeId>` dimension. Writes only the new field.
    pub fn collect_traditions(&mut self, practices_by_agent: &[Vec<u64>]) {
        let mut seen = std::collections::BTreeSet::new();
        for &member in &self.members {
            if let Some(practices) = practices_by_agent.get(member) {
                seen.extend(practices.iter().copied());
            }
        }
        self.traditions = seen.into_iter().collect();
    }

    /// Number of members.
    pub fn size(&self) -> usize {
        self.members.len()
    }

    /// Is the given agent a member?
    pub fn is_member(&self, agent: usize) -> bool {
        self.members.contains(&agent)
    }

    /// Is the given agent the head?
    pub fn is_head(&self, agent: usize) -> bool {
        self.head == Some(agent)
    }

    /// Pool food from a member's contribution.
    pub fn pool_food(&mut self, amount: Fixed) {
        self.food_reserves = (self.food_reserves + amount).max(Fixed::ZERO);
    }

    /// Distribute food to a member (returns amount actually given).
    pub fn distribute_food(&mut self, amount: Fixed) -> Fixed {
        let given = amount.min(self.food_reserves);
        self.food_reserves -= given;
        given
    }

    /// Update household dynamics for one tick.
    pub fn tick_update(&mut self) {
        // Cohesion slowly decays without positive reinforcement
        self.cohesion = (self.cohesion * Fixed::from_f64(0.999)).max(Fixed::from_f64(0.1));
        // Conflict slowly decays (grudges fade)
        self.conflict = (self.conflict * Fixed::from_f64(0.995)).clamp_01();
        // Reputation slowly converges toward 0.5 (neutral)
        let drift = (Fixed::from_f64(0.5) - self.reputation) * Fixed::from_f64(0.001);
        self.reputation = (self.reputation + drift).clamp_01();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_household_has_one_member() {
        let h = Household::new(0, Some(5), 100);
        assert_eq!(h.size(), 1);
        assert!(h.is_member(0));
        assert!(h.is_head(0));
        assert_eq!(h.residence, Some(5));
    }

    #[test]
    fn add_member() {
        let mut h = Household::new(0, None, 0);
        h.add_member(1);
        h.add_member(2);
        assert_eq!(h.size(), 3);
        assert!(h.is_member(1));
    }

    #[test]
    fn add_duplicate_member_is_noop() {
        let mut h = Household::new(0, None, 0);
        h.add_member(0);
        assert_eq!(h.size(), 1);
    }

    #[test]
    fn remove_member() {
        let mut h = Household::new(0, None, 0);
        h.add_member(1);
        h.remove_member(1);
        assert_eq!(h.size(), 1);
        assert!(!h.is_member(1));
    }

    #[test]
    fn remove_head_assigns_new_head() {
        let mut h = Household::new(0, None, 0);
        h.add_member(1);
        h.remove_member(0);
        assert_eq!(h.head, Some(1));
    }

    #[test]
    fn pool_and_distribute_food() {
        let mut h = Household::new(0, None, 0);
        let initial = h.food_reserves;
        h.pool_food(Fixed::from_f64(5.0));
        assert!(h.food_reserves > initial);
        let given = h.distribute_food(Fixed::from_f64(3.0));
        assert_eq!(given, Fixed::from_f64(3.0));
    }

    #[test]
    fn distribute_food_caps_at_reserve() {
        let mut h = Household::new(0, None, 0);
        h.food_reserves = Fixed::from_f64(2.0);
        let given = h.distribute_food(Fixed::from_f64(10.0));
        assert_eq!(given, Fixed::from_f64(2.0));
        assert_eq!(h.food_reserves, Fixed::ZERO);
    }

    #[test]
    fn tick_update_decays_conflict() {
        let mut h = Household::new(0, None, 0);
        h.conflict = Fixed::from_f64(0.8);
        h.tick_update();
        assert!(h.conflict < Fixed::from_f64(0.8));
    }

    #[test]
    fn cohesion_stays_above_floor() {
        let mut h = Household::new(0, None, 0);
        h.cohesion = Fixed::from_f64(0.15);
        for _ in 0..1000 {
            h.tick_update();
        }
        assert!(h.cohesion >= Fixed::from_f64(0.1));
    }

    // ── §10.7 Roles + Traditions Tests ────────────────────────────────

    #[test]
    fn new_household_has_head_role() {
        let h = Household::new(0, None, 0);
        assert_eq!(h.roles, vec![HouseholdRole::Head]);
        assert!(h.traditions.is_empty());
    }

    #[test]
    fn add_member_keeps_roles_parallel() {
        let mut h = Household::new(0, None, 0);
        h.add_member(1);
        h.add_member(2);
        assert_eq!(h.members.len(), h.roles.len());
        assert_eq!(h.roles[1], HouseholdRole::Adult);
    }

    #[test]
    fn remove_member_removes_role_too() {
        let mut h = Household::new(0, None, 0);
        h.add_member(1);
        h.remove_member(1);
        assert_eq!(h.members, vec![0]);
        assert_eq!(h.roles, vec![HouseholdRole::Head]);
    }

    #[test]
    fn derive_roles_assigns_by_state() {
        let mut h = Household::new(0, None, 0);
        h.add_member(1); // partner
        h.add_member(2); // child
        h.add_member(3); // elder
        h.add_member(4); // adult
        let ages = vec![
            Fixed::from_f64(40.0), // 0 head
            Fixed::from_f64(38.0), // 1 partner
            Fixed::from_f64(8.0),  // 2 child
            Fixed::from_f64(70.0), // 3 elder
            Fixed::from_f64(30.0), // 4 adult
        ];
        let partners = vec![Some(1usize), Some(0), None, None, None];
        h.derive_roles(&ages, &partners);
        assert_eq!(h.roles[0], HouseholdRole::Head);
        assert_eq!(h.roles[1], HouseholdRole::Partner);
        assert_eq!(h.roles[2], HouseholdRole::Child);
        assert_eq!(h.roles[3], HouseholdRole::Elder);
        assert_eq!(h.roles[4], HouseholdRole::Adult);
    }

    #[test]
    fn collect_traditions_unions_member_practices_sorted() {
        let mut h = Household::new(0, None, 0);
        h.add_member(1);
        let practices = vec![vec![3u64, 1], vec![2], vec![]];
        h.collect_traditions(&practices);
        assert_eq!(
            h.traditions,
            vec![1, 2, 3],
            "sorted union of member practices"
        );
    }

    #[test]
    fn traditions_drop_when_members_change() {
        let mut h = Household::new(0, None, 0);
        h.add_member(1);
        h.collect_traditions(&[vec![1u64], vec![2]]);
        assert_eq!(h.traditions, vec![1, 2]);
        h.remove_member(1);
        h.collect_traditions(&[vec![1u64], vec![2]]);
        assert_eq!(
            h.traditions,
            vec![1],
            "traditions recompute from remaining members"
        );
    }
}
