//! i390 — office succession: an empty seat is filled from the living.
//!
//! i389 gave the council a voice and the pestilence leg answered 0.00% — with the
//! crisis maximally live (panic active 90.69% of ticks). The reason was structural,
//! not behavioural: `handle_agent_death` vacates every office its holder held
//! (`role.holder = None`) and **nothing ever refills it**. The i390 probe measured
//! the result — the Elder seat vacant for **100% of a pestilence window with a
//! candidate pool present 100% of the time** (a 2-member council, seat empty) —
//! while calm and collapse sit at 0.00% vacancy because nobody dies there.
//!
//! An institution that outlives its holder but cannot replace him is authority
//! that dies with the office-holder: the members, treasury and legitimacy all
//! survive (the Council still reads 0.567 legitimately), so nothing else in the
//! engine notices. This module is the missing lifecycle step.
//!
//! The rule is the *institution's own*: among its living members, the office goes
//! to the one the settlement already respects most — `status_v2.effective_status()`
//! (the §11.1 composite: wealth, office, prestige, network centrality). Ties break
//! on `AgentId`, so the appointment is deterministic without touching an RNG
//! stream. One member cannot hold two seats in the same institution (a repeat
//! appointment would just move the vacancy), and an institution with **no** living
//! members is left vacant — that is a different root cause (recruitment), recorded
//! rather than papered over.

use mindstrata_core::{AgentId, Fixed};
use mindstrata_institutions::institutions::Institution;

use crate::sim::AgentBundle;

/// How quickly a vacancy can be filled, in ticks (1000 ticks = one in-sim year,
/// so within ~18 in-sim days). Fast enough that an office is not dark for a
/// measurable stretch of a crisis, slow enough that the appointment is a
/// *succession* rather than an instantaneous transfer — and cheap: the pass is
/// O(institutions × roles × members) once per cadence.
pub const SUCCESSION_CADENCE_TICKS: u64 = 50;

/// Fill every vacant office whose institution still has a living member to give it
/// to, returning the number of appointments made (the value the pins read).
///
/// `agents` is indexed by `AgentId` — the same convention `remove_member` and the
/// death path use.
pub fn system_office_succession(
    tick: u64,
    agents: &[AgentBundle],
    institutions: &mut [Institution],
) -> usize {
    if agents.is_empty() || !tick.is_multiple_of(SUCCESSION_CADENCE_TICKS) {
        return 0;
    }
    let mut appointments = 0usize;
    for institution in institutions.iter_mut() {
        if !institution.roles.iter().any(|r| r.holder.is_none()) {
            continue;
        }
        // Candidates: living members, best (prestige, then id) first. Members are
        // removed from the roster on death, so this pool is alive by construction.
        let mut pool: Vec<(AgentId, Fixed)> = institution
            .members
            .iter()
            .filter_map(|m| {
                agents
                    .get(m.as_u64() as usize)
                    .map(|a| (*m, a.status_v2.effective_status()))
            })
            .collect();
        pool.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        if pool.is_empty() {
            continue;
        }
        let mut taken: Vec<AgentId> = institution.roles.iter().filter_map(|r| r.holder).collect();
        for role in &mut institution.roles {
            if role.holder.is_some() {
                continue;
            }
            let Some(&(candidate, _)) = pool.iter().find(|(id, _)| !taken.contains(id)) else {
                break; // every member already holds a seat here
            };
            role.holder = Some(candidate);
            taken.push(candidate);
            appointments += 1;
        }
    }
    appointments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{SimConfig, Simulation};
    use mindstrata_institutions::institutions::{InstitutionKind, Role};

    fn council(members: &[u64]) -> Institution {
        let mut inst = Institution::new(1, InstitutionKind::Council, "Council".into());
        inst.members = members.iter().map(|m| AgentId::new(*m)).collect();
        for name in ["Elder", "Guard Captain"] {
            inst.add_role(Role {
                name: name.into(),
                holder: None,
                authority: Fixed::from_f64(0.8),
                obligations: Vec::new(),
            });
        }
        inst
    }

    fn village(n: u32) -> Simulation {
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 10,
            world_width: 8,
            world_height: 8,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();
        sim
    }

    /// Off-cadence is a no-op (authority does not change hands mid-cadence), and
    /// an institution with **no living member** stays vacant — recruitment is a
    /// different root cause, so succession must not invent a holder.
    #[test]
    fn succession_is_cadence_gated_and_needs_a_candidate() {
        let sim = village(4);
        let mut vacancies = council(&[]);
        assert_eq!(
            system_office_succession(
                SUCCESSION_CADENCE_TICKS + 1,
                &sim.agents,
                std::slice::from_mut(&mut vacancies)
            ),
            0,
            "off-cadence: no appointment"
        );
        assert_eq!(
            system_office_succession(
                SUCCESSION_CADENCE_TICKS,
                &sim.agents,
                std::slice::from_mut(&mut vacancies)
            ),
            0,
            "an empty roster cannot succeed its own office"
        );
        assert!(vacancies.roles.iter().all(|r| r.holder.is_none()));
    }

    /// The invariant i389 needed: with living members and a vacancy, every seat is
    /// filled — and no member holds two seats in the same institution (a repeat
    /// appointment would just move the vacancy).
    #[test]
    fn vacant_offices_are_filled_from_the_living_membership() {
        let sim = village(4);
        let mut inst = council(&(0..sim.agents.len() as u64).collect::<Vec<_>>());
        let appointments = system_office_succession(
            SUCCESSION_CADENCE_TICKS,
            &sim.agents,
            std::slice::from_mut(&mut inst),
        );
        assert_eq!(appointments, 2, "both vacant seats are filled");
        let holders: Vec<AgentId> = inst.roles.iter().filter_map(|r| r.holder).collect();
        assert_eq!(holders.len(), 2);
        assert_ne!(
            holders[0], holders[1],
            "one member must not hold two seats in the same institution"
        );
        assert!(
            holders.iter().all(|h| inst.members.contains(h)),
            "an office holder must be a member"
        );
    }

    /// The criterion is the institution's own: the most respected member inherits
    /// the seat — not the first in roster order, and not an arbitrary draw.
    #[test]
    fn the_most_respected_member_takes_the_seat() {
        let mut sim = village(4);
        let favoured = sim.agents.len() - 1;
        let mut member_ids = Vec::new();
        for (i, a) in sim.agents.iter_mut().enumerate() {
            a.status_v2.prestige = Fixed::from_f64(if i == favoured { 1.0 } else { 0.02 });
            member_ids.push(i as u64);
        }
        let mut inst = council(&member_ids);
        let _ = system_office_succession(
            SUCCESSION_CADENCE_TICKS,
            &sim.agents,
            std::slice::from_mut(&mut inst),
        );
        assert_eq!(
            inst.roles[0].holder,
            Some(AgentId::new(favoured as u64)),
            "the prestigious member inherits the seat"
        );
    }
}
