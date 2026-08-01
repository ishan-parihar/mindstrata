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

/// Role within a household.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
        }
    }

    /// Add a member to the household.
    pub fn add_member(&mut self, agent: usize) {
        if !self.members.contains(&agent) {
            self.members.push(agent);
        }
    }

    /// Remove a member from the household.
    pub fn remove_member(&mut self, agent: usize) {
        self.members.retain(|&m| m != agent);
        if self.head == Some(agent) {
            self.head = self.members.first().copied();
        }
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
}
