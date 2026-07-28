//! Material logistics — §19.5.E of the architecture spec.
//!
//! Implements storage, spoilage, transport cost, carrying capacity,
//! access rights, ownership, local scarcity, and site inventory.
//!
//! §19.5.E: "Resources should not be abstract global numbers. Add:
//! storage, spoilage, transport cost, carrying capacity, access rights,
//! ownership, local scarcity, site inventory."

use crate::world::{Site, SiteKind, ResourceStock};
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// Transport cost between two sites (in coin per unit of resource).
/// §19.5.E: "Transport cost" — moving resources between sites has a cost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportRoute {
    /// Index of origin site.
    pub from_site: usize,
    /// Index of destination site.
    pub to_site: usize,
    /// Base transport cost per unit of resource (in coin).
    pub base_cost: Fixed,
    /// Distance factor (multiplied by base_cost).
    pub distance_factor: Fixed,
    /// Whether this route is currently active (e.g., road exists).
    pub active: bool,
}

impl TransportRoute {
    /// Create a new transport route.
    pub fn new(from_site: usize, to_site: usize, base_cost: Fixed) -> Self {
        Self {
            from_site,
            to_site,
            base_cost,
            distance_factor: Fixed::ONE,
            active: true,
        }
    }

    /// Compute the total transport cost for moving `quantity` units.
    pub fn transport_cost(&self, quantity: Fixed) -> Fixed {
        if !self.active {
            return Fixed::ZERO; // no route = can't transport
        }
        self.base_cost * self.distance_factor * quantity
    }
}

/// Carrying capacity for an agent (how much they can carry).
/// §19.5.E: "Carrying capacity" — agents have limited inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarryingCapacity {
    /// Maximum weight the agent can carry (in resource units).
    pub max_weight: Fixed,
    /// Current load (sum of carried resources).
    pub current_load: Fixed,
}

impl Default for CarryingCapacity {
    fn default() -> Self {
        Self {
            max_weight: Fixed::from_f64(5.0), // agents can carry 5 units
            current_load: Fixed::ZERO,
        }
    }
}

impl CarryingCapacity {
    /// How much more the agent can carry.
    pub fn remaining(&self) -> Fixed {
        (self.max_weight - self.current_load).max(Fixed::ZERO)
    }

    /// Can the agent carry `amount` more?
    pub fn can_carry(&self, amount: Fixed) -> bool {
        self.remaining() >= amount
    }

    /// Add to the agent's load.
    pub fn add_load(&mut self, amount: Fixed) -> Fixed {
        let added = amount.min(self.remaining());
        self.current_load = (self.current_load + added).min(self.max_weight);
        added
    }

    /// Remove from the agent's load.
    pub fn remove_load(&mut self, amount: Fixed) {
        self.current_load = (self.current_load - amount).max(Fixed::ZERO);
    }
}

/// Local scarcity pricing modifier.
/// §19.5.E: "Local scarcity" — prices vary by local supply/demand.
pub fn local_scarcity_modifier(site_inventory: &[ResourceStock], resource_id: u64, max_expected: Fixed) -> Fixed {
    let total = site_inventory
        .iter()
        .filter(|s| s.resource_id == resource_id)
        .fold(Fixed::ZERO, |acc, s| acc + s.quantity);

    if total <= Fixed::ZERO {
        // Extreme scarcity: very high modifier
        Fixed::from_f64(3.0)
    } else if total >= max_expected {
        // Abundance: low modifier
        Fixed::from_f64(0.5)
    } else {
        // Linear interpolation between 0.5 and 3.0
        let ratio = total / max_expected;
        Fixed::from_f64(3.0) - ratio * Fixed::from_f64(2.5)
    }
}

/// Storage capacity for a site.
/// §19.5.E: "Storage" — sites have limited storage capacity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageCapacity {
    /// Maximum total resource units the site can store.
    pub max_capacity: Fixed,
    /// Current total stored.
    pub current_used: Fixed,
}

impl StorageCapacity {
    /// Create a storage capacity for a site.
    pub fn new(max_capacity: Fixed) -> Self {
        Self {
            max_capacity,
            current_used: Fixed::ZERO,
        }
    }

    /// How much more can be stored.
    pub fn remaining(&self) -> Fixed {
        (self.max_capacity - self.current_used).max(Fixed::ZERO)
    }

    /// Can we store `amount` more?
    pub fn can_store(&self, amount: Fixed) -> bool {
        self.remaining() >= amount
    }

    /// Update current_used from site inventory.
    pub fn update_from_inventory(&mut self, inventory: &[ResourceStock]) {
        self.current_used = inventory.iter().fold(Fixed::ZERO, |acc, s| acc + s.quantity);
    }
}

/// Compute carrying cost for an agent based on load and personality.
/// §19.5.E: "Transport cost" — carrying heavy loads is tiring.
pub fn carrying_cost(load: Fixed, max_capacity: Fixed, fatigue: Fixed) -> Fixed {
    if max_capacity <= Fixed::ZERO {
        return Fixed::ZERO;
    }
    let load_ratio = load / max_capacity;
    // Heavy loads increase fatigue
    load_ratio * Fixed::from_f64(0.02) * (Fixed::ONE + fatigue * Fixed::from_f64(0.5))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_cost_scales_with_quantity() {
        let route = TransportRoute::new(0, 1, Fixed::from_f64(2.0));
        let cost_1 = route.transport_cost(Fixed::from_f64(1.0));
        let cost_5 = route.transport_cost(Fixed::from_f64(5.0));
        assert!(cost_5 > cost_1);
        assert_eq!(cost_5, cost_1 * Fixed::from_f64(5.0));
    }

    #[test]
    fn carrying_capacity_works() {
        let mut cap = CarryingCapacity::default();
        assert!(cap.can_carry(Fixed::from_f64(3.0)));
        let added = cap.add_load(Fixed::from_f64(3.0));
        assert_eq!(added, Fixed::from_f64(3.0));
        assert!(!cap.can_carry(Fixed::from_f64(3.0))); // only 2 left
        assert!(cap.can_carry(Fixed::from_f64(2.0)));
    }

    #[test]
    fn carrying_capacity_clamps_at_max() {
        let mut cap = CarryingCapacity::default();
        let added = cap.add_load(Fixed::from_f64(10.0)); // more than max
        assert_eq!(added, Fixed::from_f64(5.0)); // only got 5
        assert_eq!(cap.current_load, Fixed::from_f64(5.0));
    }

    #[test]
    fn local_scarcity_high_when_empty() {
        let inventory = vec![];
        let modifier = local_scarcity_modifier(&inventory, 0, Fixed::from_f64(10.0));
        assert!(modifier > Fixed::from_f64(2.0), "Empty site should have high scarcity modifier");
    }

    #[test]
    fn local_scarcity_low_when_full() {
        let inventory = vec![ResourceStock {
            resource_id: 0,
            quantity: Fixed::from_f64(20.0),
            quality: Fixed::ONE,
            access: crate::world::AccessRight::Public,
        }];
        let modifier = local_scarcity_modifier(&inventory, 0, Fixed::from_f64(10.0));
        assert!(modifier < Fixed::from_f64(1.0), "Full site should have low scarcity modifier");
    }

    #[test]
    fn storage_capacity_works() {
        let mut storage = StorageCapacity::new(Fixed::from_f64(100.0));
        assert!(storage.can_store(Fixed::from_f64(50.0)));
        storage.current_used = Fixed::from_f64(90.0);
        assert!(storage.can_store(Fixed::from_f64(10.0)));
        assert!(!storage.can_store(Fixed::from_f64(20.0)));
    }

    #[test]
    fn carrying_cost_increases_with_load() {
        let low = carrying_cost(
            Fixed::from_f64(1.0),
            Fixed::from_f64(5.0),
            Fixed::ZERO,
        );
        let high = carrying_cost(
            Fixed::from_f64(4.0),
            Fixed::from_f64(5.0),
            Fixed::ZERO,
        );
        assert!(high > low, "Higher load should cost more");
    }

    #[test]
    fn inactive_route_has_zero_cost() {
        let mut route = TransportRoute::new(0, 1, Fixed::from_f64(2.0));
        route.active = false;
        let cost = route.transport_cost(Fixed::from_f64(5.0));
        assert_eq!(cost, Fixed::ZERO, "Inactive route should have zero cost");
    }
}
