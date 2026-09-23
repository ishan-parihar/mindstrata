//! i389 — the directive channel gets an in-sim producer: council decrees.
//!
//! DC-5's G1 lists authority as **nominal**. The offices exist (`Elder`, `Guard
//! Captain`), the directive API exists (`Simulation::command_agent` + five unit
//! tests), the consumption path exists (`command_goal_action` overrides routine
//! and utility, gated so a pressing need still wins), and the census counts the
//! layer (`SRC_COMMAND`) — but until i389 **no shipped producer emitted a
//! directive goal**, so the layer measured 0.00% of selections, 0 directive
//! holders, in every probed world (`i389_command_channel`: calm village,
//! collapse, pestilence — all 0.00%).
//!
//! This module is that producer. Two design decisions carry it:
//!
//! * **A decree is its own goal source, and it DECAYS.** `GoalSource::Command`
//!   is the *operator's* channel: exempt from goal decay (`system_goal_generation`
//!   skips it) because a TUI command persists until satisfied. An in-sim decree
//!   must be the opposite — bounded — so it ships as `GoalSource::Decree`, which
//!   rides the ordinary priority decay (`age × 0.001`/tick). A decree that is
//!   never acted on fades on its own; there is no clearing logic to get wrong and
//!   no way for a stale decree to steer a village years later.
//! * **The mandate scales the ask.** A decree's priority is
//!   `legitimacy × DECREE_MANDATE_SHARE`, so a resented council's directive is
//!   weak and fades fastest — authority that has lost its mandate stops being
//!   heard without a special case, and a council below `DECREE_MANDATE_FLOOR`
//!   does not ask at all.
//!
//! Calm is a no-op by construction: both crisis conditions must be false, which
//! is the item's exit criterion (0% in calm, >0% in crisis).

use mindstrata_core::fixed::Fixed;
use mindstrata_institutions::institutions::{Institution, InstitutionKind};

use crate::person::{Goal, GoalKind, GoalSource};
use crate::sim::AgentBundle;

/// How often the council may speak, in ticks (1000 ticks = one in-sim year, so
/// four decrees a year at most). The cadence is the *rate* bound: without it a
/// crisis would re-issue every tick and the directive would act as a permanent
/// override rather than an ask.
pub const DECREE_CADENCE_TICKS: u64 = 250;

/// How many villagers one decree reaches — the nearest cohort to the office
/// holder. The reach is what makes a decree a *signal* rather than a global
/// state change: the rest of the village learns only by seeing it acted on.
pub const DECREE_MAX_TARGETS: usize = 4;

/// Population mean hunger above which the granary reads short.
///
/// `0.6` is the sibling band the goal machinery already treats as pressing
/// (`GoalGates::CANON.eat` 0.5 creates the Eat goal; hunger > 0.85 is the
/// famine-strength band the gate audit found dark by design). Measured context
/// (`i389_command_channel`): the calm village sits at mean hunger **0.026** and
/// the collapse window at **0.040**, so this bar is a genuine crisis line, not a
/// background term — the item demands 0% in calm.
pub const DECREE_HUNGER_BAR: Fixed = Fixed::from_raw(6_000);

/// Population mean fear above which the village reads frightened (the §7.2
/// panic family's own band — the fear contagion gates and the panic trigger both
/// live at 0.5).
pub const DECREE_FEAR_BAR: Fixed = Fixed::from_raw(5_000);

/// Legitimacy below which the council stops being heard at all. A council that
/// has lost its mandate cannot ask; the measured mandate equilibrium is
/// 0.54–0.58 (`i382`), so this floor sits at roughly two thirds of a healthy
/// mandate — the point where the institution is already losing its grip, not the
/// point where it is normal.
pub const DECREE_MANDATE_FLOOR: Fixed = Fixed::from_raw(3_500);

/// The share of legitimacy a decree's priority carries. A full-mandate council
/// asks at 0.8 priority — above the routine's own goals, below a critical need,
/// and short enough of 1.0 that the goal decays in ~800 ticks rather than
/// persisting like an operator directive.
pub const DECREE_MANDATE_SHARE: Fixed = Fixed::from_raw(8_000);

/// The decree a crisis calls for: a short granary asks for WORK, a frightened
/// village asks for WORSHIP (public ritual — the legitimacy-restoring response
/// the institution itself already models). Neither crisis ⇒ no decree.
#[must_use]
pub fn decree_for(hunger_crisis: bool, fear_crisis: bool) -> Option<GoalKind> {
    if hunger_crisis {
        Some(GoalKind::Work)
    } else if fear_crisis {
        Some(GoalKind::Worship)
    } else {
        None
    }
}

/// The priority a decree carries at this legitimacy, or `None` when the council
/// has no mandate to ask at all.
#[must_use]
pub fn decree_priority(legitimacy: Fixed) -> Option<Fixed> {
    if legitimacy < DECREE_MANDATE_FLOOR {
        return None;
    }
    Some((legitimacy * DECREE_MANDATE_SHARE).clamp_01())
}

/// The `n` agents nearest `origin`, nearest first — ties broken by index so the
/// choice is deterministic without touching an RNG stream.
#[must_use]
fn nearest_agents(agents: &[AgentBundle], origin: (i32, i32), n: usize) -> Vec<usize> {
    let mut scored: Vec<(i64, usize)> = agents
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let dx = i64::from(a.position.x) - i64::from(origin.0);
            let dy = i64::from(a.position.y) - i64::from(origin.1);
            (dx * dx + dy * dy, i)
        })
        .collect();
    scored.sort_unstable();
    scored.into_iter().take(n).map(|(_, i)| i).collect()
}

/// The i389 producer: on the cadence, an office-holding council whose mandate
/// holds and whose settlement is in crisis directs its nearest villagers.
///
/// Returns the number of decrees issued (0 when calm, off-cadence, unmandated,
/// or when no `Elder` holds the office) — the value the probe and the pins read,
/// so "the authority never asks" is measurable rather than assumed.
pub fn system_council_decrees(
    tick: u64,
    agents: &mut [AgentBundle],
    institutions: &[Institution],
    famine_open: bool,
    panic_active: bool,
) -> usize {
    if agents.is_empty() || !tick.is_multiple_of(DECREE_CADENCE_TICKS) {
        return 0;
    }
    let Some(council) = institutions
        .iter()
        .find(|i| i.kind == InstitutionKind::Council)
    else {
        return 0;
    };
    let Some(priority) = decree_priority(council.legitimacy) else {
        return 0;
    };
    let Some(elder) = council.get_role_holder("Elder") else {
        return 0;
    };

    let n = agents.len() as f64;
    let mean_hunger = agents.iter().map(|a| a.needs.hunger.to_f64()).sum::<f64>() / n;
    let mean_fear = agents.iter().map(|a| a.emotions.fear.to_f64()).sum::<f64>() / n;
    let Some(kind) = decree_for(
        famine_open || mean_hunger > DECREE_HUNGER_BAR.to_f64(),
        panic_active || mean_fear > DECREE_FEAR_BAR.to_f64(),
    ) else {
        return 0;
    };

    let elder_idx = elder.as_u64() as usize;
    let Some(holder) = agents.get(elder_idx) else {
        return 0;
    };
    let origin = (holder.position.x, holder.position.y);
    let mut issued = 0usize;
    for i in nearest_agents(agents, origin, DECREE_MAX_TARGETS + 1) {
        if i == elder_idx {
            continue; // the holder is not its own subject
        }
        let agent = &mut agents[i];
        // Upsert: a repeat decree of the same kind replaces the old one, and a
        // decree of the other kind is superseded (the crisis changed).
        agent.goals.retain(|g| g.source != GoalSource::Decree);
        agent.goals.push(Goal {
            kind,
            priority,
            commitment: priority,
            created_tick: tick,
            source: GoalSource::Decree,
        });
        issued += 1;
    }
    issued
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The mandate gate and the mandate share are the whole authority law: a
    /// council below the floor does not ask, and the ask scales with what
    /// mandate remains (a nervous council's decree is weak and fades fastest).
    #[test]
    fn decree_priority_reads_the_mandate() {
        assert!(
            decree_priority(DECREE_MANDATE_FLOOR - Fixed::from_raw(1)).is_none(),
            "a council without a mandate must not speak"
        );
        let full = decree_priority(Fixed::ONE).expect("full mandate may ask");
        assert_eq!(full, DECREE_MANDATE_SHARE);
        let weak = decree_priority(Fixed::from_f64(0.4)).expect("a weak mandate may still ask");
        assert!(
            weak < full / Fixed::from_int(2),
            "a 0.4 mandate asks at less than half strength ({weak:?} vs {full:?})"
        );
    }

    /// Crisis → a decree; calm → nothing. Hunger outranks fear (a short granary
    /// is the more urgent ask), and the two are the only crisis classes.
    #[test]
    fn decree_for_maps_crisis_to_an_ask() {
        assert_eq!(decree_for(false, false), None);
        assert_eq!(decree_for(true, false), Some(GoalKind::Work));
        assert_eq!(decree_for(false, true), Some(GoalKind::Worship));
        assert_eq!(decree_for(true, true), Some(GoalKind::Work));
    }
}
