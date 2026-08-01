//! Patronage system — asymmetric power relations of patron/client type.
//!
//! Architecture §10.3: Authority branch relationships include Patron/Client,
//! Lord/Vassal, Master/Apprentice. Patronage creates asymmetric obligations:
//! the patron provides protection, resources, or status; the client provides
//! loyalty, labor, or political support.
//!
//! ```text
//! Patronage dynamics:
//!   - Patron provides: protection, resources, status endorsement
//!   - Client provides: loyalty, labor, political support
//!   - Power imbalance: patron has resource control, client has dependence
//!   - Obligation asymmetry: patron has less obligation than client
//!   - Dissolution: betrayal, resource loss, status change, death
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Role in a patronage relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatronageRole {
    /// The patron — provides resources and protection.
    Patron,
    /// The client — provides loyalty and labor.
    Client,
}

/// A patronage relationship between two agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatronageRelation {
    /// Index of the patron.
    pub patron: usize,
    /// Index of the client.
    pub client: usize,
    /// Patron's provision level — resources, protection, status (0–1).
    pub provision: Fixed,
    /// Client's loyalty level (0–1).
    pub loyalty: Fixed,
    /// Client's labor contribution (0–1).
    pub labor_contribution: Fixed,
    /// Client's political support (0–1).
    pub political_support: Fixed,
    /// Obligation balance — how much the patron feels obligated to the client (0–1).
    pub patron_obligation: Fixed,
    /// Dependence of client on patron (0–1).
    pub client_dependence: Fixed,
    /// Satisfaction of both parties (0–1).
    pub satisfaction: Fixed,
    /// Duration of the relationship in ticks.
    pub duration: u64,
    /// Tick when the relationship was formed.
    pub formed_tick: u64,
    /// Whether the relationship is currently active.
    pub active: bool,
}

impl PatronageRelation {
    /// Create a new patronage relationship.
    pub fn new(patron: usize, client: usize, tick: u64) -> Self {
        Self {
            patron,
            client,
            provision: Fixed::from_f64(0.3),
            loyalty: Fixed::from_f64(0.4),
            labor_contribution: Fixed::from_f64(0.3),
            political_support: Fixed::from_f64(0.3),
            patron_obligation: Fixed::from_f64(0.2),
            client_dependence: Fixed::from_f64(0.5),
            satisfaction: Fixed::from_f64(0.5),
            duration: 0,
            formed_tick: tick,
            active: true,
        }
    }

    /// Compute the power balance — positive means patron dominates.
    pub fn power_balance(&self) -> Fixed {
        let patron_power = self.provision + self.patron_obligation * Fixed::from_f64(0.3);
        let client_power = self.loyalty + self.labor_contribution + self.political_support;
        (patron_power - client_power).clamp(Fixed::from_f64(-1.0), Fixed::ONE)
    }

    /// Compute overall relationship health.
    pub fn health(&self) -> Fixed {
        (self.provision * Fixed::from_f64(0.25)
            + self.loyalty * Fixed::from_f64(0.25)
            + self.satisfaction * Fixed::from_f64(0.3)
            + self.patron_obligation * Fixed::from_f64(0.1)
            + self.client_dependence * Fixed::from_f64(0.1))
            .clamp_01()
    }

    /// Check if the patronage should dissolve.
    pub fn should_dissolve(&self) -> bool {
        // Dissolve if satisfaction drops too low or if loyalty collapses
        self.satisfaction < Fixed::from_f64(0.1) || self.loyalty < Fixed::from_f64(0.05)
    }

    /// Daily update — grow loyalty with provision, decay satisfaction.
    pub fn daily_update(&mut self) {
        self.duration += 1;
        // Loyalty grows with provision
        self.loyalty = (self.loyalty + self.provision * Fixed::from_f64(0.001)).clamp_01();
        // Labor contribution grows with loyalty
        self.labor_contribution =
            (self.labor_contribution + self.loyalty * Fixed::from_f64(0.0005)).clamp_01();
        // Satisfaction depends on both sides getting what they want
        let patron_satisfied = self.labor_contribution * Fixed::from_f64(0.5)
            + self.political_support * Fixed::from_f64(0.5);
        let client_satisfied = self.provision * Fixed::from_f64(0.7)
            + self.patron_obligation * Fixed::from_f64(0.3);
        self.satisfaction = (patron_satisfied + client_satisfied) * Fixed::from_f64(0.5);
        // Slow decay if not actively reinforced
        self.provision = (self.provision * Fixed::from_f64(0.999)).max(Fixed::ZERO);
    }
}

/// Registry of all patronage relationships.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatronageRegistry {
    pub relations: Vec<PatronageRelation>,
}

impl PatronageRegistry {
    pub fn new() -> Self {
        Self {
            relations: Vec::new(),
        }
    }

    /// Register a new patronage relation.
    pub fn register(&mut self, rel: PatronageRelation) -> usize {
        let idx = self.relations.len();
        self.relations.push(rel);
        idx
    }

    /// Find patronage relations where the given agent is the patron.
    pub fn clients_of(&self, patron: usize) -> Vec<&PatronageRelation> {
        self.relations
            .iter()
            .filter(|r| r.patron == patron && r.active)
            .collect()
    }

    /// Find patronage relations where the given agent is the client.
    pub fn patron_of(&self, client: usize) -> Option<&PatronageRelation> {
        self.relations
            .iter()
            .find(|r| r.client == client && r.active)
    }

    /// Daily update for all active patronage relations.
    pub fn daily_update(&mut self) {
        for rel in &mut self.relations {
            if rel.active {
                rel.daily_update();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patronage_health_computed() {
        let rel = PatronageRelation::new(0, 1, 0);
        let health = rel.health();
        assert!(health > Fixed::ZERO);
        assert!(health <= Fixed::ONE);
    }

    #[test]
    fn patronage_dissolves_on_low_loyalty() {
        let mut rel = PatronageRelation::new(0, 1, 0);
        rel.loyalty = Fixed::ZERO;
        assert!(rel.should_dissolve());
    }

    #[test]
    fn patronage_daily_update_grows_loyalty() {
        let mut rel = PatronageRelation::new(0, 1, 0);
        rel.provision = Fixed::from_f64(0.5);
        let initial_loyalty = rel.loyalty;
        rel.daily_update();
        assert!(rel.loyalty >= initial_loyalty);
    }

    #[test]
    fn patronage_registry_clients_of() {
        let mut registry = PatronageRegistry::new();
        registry.register(PatronageRelation::new(0, 1, 0));
        registry.register(PatronageRelation::new(0, 2, 0));
        registry.register(PatronageRelation::new(1, 3, 0));
        assert_eq!(registry.clients_of(0).len(), 2);
        assert_eq!(registry.clients_of(1).len(), 1);
    }
}
