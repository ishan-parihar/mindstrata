//! i390 — the empty seat: does authority survive its holder?
//!
//! i389's producer census found the pestilence leg's directive channel at 0.00%
//! **with the crisis maximally live** (panic active 90.69% of ticks) and the Elder
//! office **empty**. Reading the death path explains it: `handle_agent_death`
//! calls `inst.remove_member(id)` and then, for every role, `role.holder = None`
//! — the seat is deliberately vacated and **nothing ever refills it**. An office
//! that empties stays empty, so the institution survives (members, treasury,
//! legitimacy) while its authority is structurally absent.
//!
//! This probe is the item's before/after ledger. It measures:
//!   * the share of ticks with a vacant role in any institution,
//!   * the share of ticks with the Council's Elder seat vacant,
//!   * the candidate pool at those moments (an institution with no members cannot
//!     succeed its own office — that is a different root cause),
//!   * and, linking back to i389, the directive channel's share in the same run.
//!
//! Measured with `system_office_succession` disabled (the "before" column): the
//! pestilence window reads **Elder seat 100.00% vacant, candidate pool present
//! 100% of the time, directive share 0.0000%**; calm and collapse read 0.00%
//! vacancy (nobody dies in either corpus). With the pass wired (the "after"
//! column) the pestilence leg collapses to **0.00% vacancy** and the directive
//! channel comes alive at **0.0654%** while calm stays at 0.00% — authority
//! returns without speaking when nothing is wrong.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i390_office_succession`

use mindstrata_sim::institutions::InstitutionKind;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::decision_census::{self, SRC_COMMAND};
use mindstrata_sim::sim::{SimConfig, Simulation};

const WARMUP: u64 = 500;
const WINDOW: u64 = 20_000;

enum World {
    Calm,
    Collapse,
    Pestilence,
}

fn build(world: &World, seed: u64) -> Simulation {
    let mut sim = match world {
        World::Calm => Simulation::new(SimConfig {
            seed,
            max_ticks: WARMUP + WINDOW,
            world_width: 32,
            world_height: 32,
            num_agents: 12,
            snapshot_interval: None,
        }),
        World::Collapse => {
            let mut sc = Scenario::collapse();
            sc.seed = seed;
            sc.ticks = WARMUP + WINDOW;
            Simulation::from_scenario(sc)
        }
        World::Pestilence => {
            let mut sc = Scenario::pestilence();
            sc.seed = seed;
            sc.ticks = WARMUP + WINDOW;
            Simulation::from_scenario(sc)
        }
    };
    sim.populate();
    sim
}

fn run(label: &str, world: &World, seed: u64) {
    let mut sim = build(world, seed);
    sim.run(WARMUP);
    decision_census::reset();
    decision_census::enable();

    let mut ticks_any_vacant = 0u64;
    let mut ticks_elder_vacant = 0u64;
    let mut ticks_elder_vacant_with_pool = 0u64;
    let mut vacancies_orphaned = 0u64; // vacant AND no member to appoint
    for _ in 0..WINDOW {
        sim.run(1);
        let mut any_vacant = false;
        let mut elder_vacant = false;
        let mut pool_nonempty = false;
        for inst in &sim.institutions {
            let vacant: usize = inst.roles.iter().filter(|r| r.holder.is_none()).count();
            if vacant > 0 {
                any_vacant = true;
                if inst.members.is_empty() {
                    vacancies_orphaned += vacant as u64;
                }
            }
            if inst.kind == InstitutionKind::Council {
                let elder = inst
                    .roles
                    .iter()
                    .find(|r| r.name == "Elder")
                    .is_some_and(|r| r.holder.is_none());
                if elder {
                    elder_vacant = true;
                    if !inst.members.is_empty() {
                        pool_nonempty = true;
                    }
                }
            }
        }
        if any_vacant {
            ticks_any_vacant += 1;
        }
        if elder_vacant {
            ticks_elder_vacant += 1;
        }
        if elder_vacant && pool_nonempty {
            ticks_elder_vacant_with_pool += 1;
        }
    }
    decision_census::disable();
    let r = decision_census::report();
    let total = r.total().max(1) as f64;

    let council = sim
        .institutions
        .iter()
        .find(|i| i.kind == InstitutionKind::Council);
    let (members, roles, filled) = match council {
        Some(c) => (
            c.members.len(),
            c.roles.len(),
            c.roles.iter().filter(|r| r.holder.is_some()).count(),
        ),
        None => (0, 0, 0),
    };

    println!("══ {label} (seed {seed}) ══");
    println!(
        "  vacancy share over {WINDOW} ticks: any role {:.2}% · Elder seat {:.2}% · \
         Elder vacant WITH a candidate pool {:.2}% · orphaned vacancies (no members) {vacancies_orphaned}",
        ticks_any_vacant as f64 / WINDOW as f64 * 100.0,
        ticks_elder_vacant as f64 / WINDOW as f64 * 100.0,
        ticks_elder_vacant_with_pool as f64 / WINDOW as f64 * 100.0,
    );
    println!(
        "  council at end: {members} members · {filled}/{roles} offices filled · \
         directive share {:.4}% ({} selections)",
        r.sources[SRC_COMMAND] as f64 / total * 100.0,
        r.sources[SRC_COMMAND]
    );
    println!();
}

fn main() {
    println!("i390 — office succession (warmup {WARMUP}, window {WINDOW})\n");
    run("calm village", &World::Calm, 42);
    run("collapse", &World::Collapse, 42);
    run("pestilence", &World::Pestilence, 7);
}
