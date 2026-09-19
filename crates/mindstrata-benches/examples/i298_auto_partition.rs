//! i298 — Settlement-based auto-partition (UM-3 leg 2; PLAN_DC3 §8).
//!
//! i297 delivered territory-anchored genesis for OPERATOR-assigned polities.
//! This probe is the exit test for the assignment rule itself: polities
//! derived from settlement geography (home-site clusters) must reproduce the
//! i296/i297 exit contracts without an operator partition:
//!
//! 1. **Single settlement → inert** — the calibrated one-ring village must
//!    auto-partition into NOTHING (legacy default, zero blast).
//! 2. **Two settlements → two polities** — a two-ring geography (agents
//!    re-homed onto two distant houses) must derive a disjoint cover with
//!    sorted membership, no operator input.
//! 3. **Derived ≡ assigned** — the auto-derived partition must step fields
//!    identically to `assign_polities` with the same members (the assignment
//!    rule is pure derivation over the same i296 machinery).
//!
//! Run: cargo run -p mindstrata-benches --release --example i298_auto_partition
use mindstrata_development::collective::{bucket_for_line, CollectiveBucket, CollectiveField};
use mindstrata_sim::sim::{SimConfig, Simulation};

fn bucket_maxima(field: &CollectiveField) -> [f64; 4] {
    let slugs = CollectiveField::line_slugs();
    let mut stage = [0.0f64; 4];
    for (i, line) in field.lines.iter().enumerate() {
        if i >= slugs.len() {
            break;
        }
        let b = bucket_for_line(slugs[i]) as usize;
        if line.stage > stage[b] {
            stage[b] = line.stage;
        }
    }
    stage
}

fn main() {
    let horizon = 10_000u64;
    let seed = 42u64;

    // ── 1. Single settlement (the calibrated geography) → inert.
    let mut sim1 = Simulation::new(SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim1.populate();
    sim1.auto_partition_polities(6);
    println!("i298 auto-partition — {horizon} ticks, seed {seed}, N=12");
    println!();
    println!("1. SINGLE-SETTLEMENT INERTNESS:");
    println!(
        "   polities derived: {} (must be 0 — legacy default)",
        sim1.polity_fields.len()
    );

    // ── 2. Two settlements: re-home agents onto two maximally distant
    //    houses, run, verify auto-derived polities + divergent trajectories.
    let mut sim2 = Simulation::new(SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim2.populate();
    let n = sim2.agents.len();
    let (far, second) = {
        let houses: Vec<usize> = sim2
            .world
            .sites
            .iter()
            .enumerate()
            .filter(|(_, s)| s.kind == mindstrata_sim::world::SiteKind::House)
            .map(|(i, _)| i)
            .collect();
        // Two maximally separated houses by Manhattan distance (site 0 is
        // one pole; the farthest-from-0 house is the other). Ring radius
        // ~4 makes opposite houses ~8-10 apart — genuinely two settlements.
        let (x0, y0) = sim2.world.site_position(0).unwrap_or((0, 0));
        let far = *houses
            .iter()
            .max_by_key(|&&i| {
                let (x, y) = sim2.world.site_position(i).unwrap_or((0, 0));
                (x - x0).abs() + (y - y0).abs()
            })
            .expect("world has houses");
        let (xf, yf) = sim2.world.site_position(far).unwrap_or((0, 0));
        let second = *houses
            .iter()
            .filter(|&&i| i != far)
            .max_by_key(|&&i| {
                let (x, y) = sim2.world.site_position(i).unwrap_or((0, 0));
                (x - xf).abs() + (y - yf).abs()
            })
            .expect("world has a second house");
        (far, second)
    };
    for (i, agent) in sim2.agents.iter_mut().enumerate() {
        agent.home_site = if i < n / 2 { Some(far) } else { Some(second) };
    }
    sim2.auto_partition_polities(6);
    println!();
    println!("2. TWO-SETTLEMENT DERIVATION (houses {far} & {second}):");
    println!("   polities derived: {}", sim2.polity_fields.len());
    for (pid, members) in sim2.polity_members.iter().enumerate() {
        println!("   polity p{pid}: {members:?}");
    }
    let derived = sim2.polity_members.clone();
    sim2.run(horizon);
    let (sa, sb) = (
        bucket_maxima(&sim2.polity_fields[0]),
        bucket_maxima(&sim2.polity_fields[1]),
    );
    println!(
        "   stages p0 S={:.3} R={:.3} / p1 S={:.3} R={:.3}",
        sa[CollectiveBucket::Safety as usize],
        sa[CollectiveBucket::Relational as usize],
        sb[CollectiveBucket::Safety as usize],
        sb[CollectiveBucket::Relational as usize],
    );
    println!(
        "   trajectories diverge: {}",
        if sa != sb {
            "YES"
        } else {
            "no (identical diets)"
        }
    );

    // ── 3. Derived ≡ assigned: same members via assign_polities must give
    //    bit-identical fields after the same horizon.
    let mut sim3 = Simulation::new(SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim3.populate();
    for (i, agent) in sim3.agents.iter_mut().enumerate() {
        agent.home_site = if i < n / 2 { Some(far) } else { Some(second) };
    }
    sim3.assign_polities(derived);
    sim3.run(horizon);
    println!();
    println!("3. DERIVED vs ASSIGNED:");
    let fields_equal = sim2.polity_fields == sim3.polity_fields;
    println!(
        "   fields bit-identical after {horizon} ticks: {}",
        if fields_equal {
            "YES"
        } else {
            "NO — derivation changed behavior"
        }
    );
}
