//! Domain-grouped unit tests (split from the former monolithic tests.rs;
//! pure moves — fn bodies untouched). Child modules inherit compilation
//! from the cfg(test)-gated `mod tests;` declaration in mod.rs.

mod biology;
mod conflict;
mod culture;
mod development;
mod economy;
mod family;
mod governance;
mod legal;
mod psychology;

use super::Simulation;

// Shared test helper (used by family + conflict domains).
/// §10.8: Find two agents in different seeded clans (home-site parity
/// seeds 2 clans during populate).
pub(crate) fn cross_clan_pair(sim: &Simulation) -> (usize, usize) {
    let clans = &sim.clan_registry.clans;
    assert!(clans.len() >= 2, "two clans must be seeded");
    assert!(!clans[0].core_households.is_empty(), "clan 0 has members");
    assert!(!clans[1].core_households.is_empty(), "clan 1 has members");
    (clans[0].core_households[0], clans[1].core_households[0])
}
