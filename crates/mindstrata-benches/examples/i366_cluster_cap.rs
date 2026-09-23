//! i366 — does the `cluster_count_for` cap of 4 ever bind, and what is the real
//! knob?
//!
//! i360 set `cluster_count_for(houses) = (houses/16).clamp(1, 4)` and left an
//! open item: "does raising the cap to 5–6 buy more polities without crowding a
//! density-law world?" Before changing anything, this probe checks the arithmetic
//! in the reachable population range (`MAX_POPULATION = 256`) and measures what
//! the town actually looks like at each reachable size.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i366_cluster_cap`

use mindstrata_sim::sim::{SimConfig, Simulation};
use mindstrata_sim::world::SiteKind;
use mindstrata_sim::world_gen::{cluster_count_for, houses_for_population};

// i392: the i344 density world law is single-sourced in the world generator
// (`world_side_for_population`) — do not re-implement it.
use mindstrata_sim::world_gen::world_side_for_population as density_side;

fn main() {
    println!("i366 — cluster-cap reachability (MAX_POPULATION = 256)\n");
    println!(
        "{:>5} {:>7} {:>5} {:>10} {:>8} {:>10}",
        "N", "side", "houses", "raw(N/16)", "K", "cap binds?"
    );
    for n in [96u32, 116, 144, 192, 256, 320] {
        let side = density_side(n);
        let houses = houses_for_population(n);
        let raw = houses / 16;
        let k = cluster_count_for(houses);
        println!(
            "{:>5} {:>7} {:>5} {:>10} {:>8} {:>10}",
            n,
            side,
            houses,
            raw,
            k,
            if raw > 4 && n <= 256 {
                "yes"
            } else if raw > 4 {
                "yes (>cap N)"
            } else {
                "no"
            }
        );
    }
    // The cap can only bind when houses/16 > 4, i.e. houses >= 80, i.e.
    // N >= 320 — above MAX_POPULATION. Record the exact reachable threshold.
    println!(
        "\ncap binding needs houses >= 80 (raw > 4); reachable max is N=256 => houses {} (raw {})",
        houses_for_population(256),
        houses_for_population(256) / 16
    );

    // What the town actually is, at each reachable size (settlements @ gap 8).
    println!("\nsettlements @ gap 8 and a 2K liveness run:");
    println!(
        "{:>5} {:>6} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "N", "side", "K", "settle", "health", "hunger", "gini"
    );
    for n in [116u32, 144, 192, 256] {
        let side = density_side(n);
        let mut probe = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 0,
            world_width: side,
            world_height: side,
            num_agents: n,
            snapshot_interval: None,
        });
        probe.populate();
        let k = cluster_count_for(houses_for_population(n));
        probe.auto_partition_polities(8);
        let settlements = probe.polity_members.len();

        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 2000,
            world_width: side,
            world_height: side,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();
        sim.run(2000);
        let ms = sim.metrics_snapshot();
        let houses = sim
            .world
            .sites
            .iter()
            .filter(|s| s.kind == SiteKind::House)
            .count();
        let _ = houses;
        println!(
            "{:>5} {:>6} {:>8} {:>8} {:>8.3} {:>8.4} {:>8.3}",
            n, side, k, settlements, ms.avg_health, ms.avg_hunger, ms.gini
        );
    }
}
