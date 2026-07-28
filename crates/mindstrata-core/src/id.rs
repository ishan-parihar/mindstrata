//! Entity and resource ID types.
//!
//! All IDs are newtypes over `u64` to prevent accidental mixing.
//! They are `Copy`, cheap to pass, and serialisable.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for any entity in the world (person, household, site …).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityId(u64);

impl EntityId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Debug for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "E({})", self.0)
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier for an agent (a person / mind-scale entity).
pub type AgentId = EntityId;

/// Identifier for a resource type or resource stock.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceId(u64);

impl ResourceId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Debug for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Res({})", self.0)
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "res_{}", self.0)
    }
}

/// Identifier for a site (house, market, temple …).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SiteId(u64);

impl SiteId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Debug for SiteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Site({})", self.0)
    }
}

impl fmt::Display for SiteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "site_{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_id_display() {
        let id = EntityId::new(42);
        assert_eq!(format!("{id}"), "42");
        assert_eq!(format!("{id:?}"), "E(42)");
    }

    #[test]
    fn ids_are_copy() {
        let a = EntityId::new(1);
        let b = a;
        assert_eq!(a, b);
    }
}
