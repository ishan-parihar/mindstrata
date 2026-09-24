//! i362 — with the world now a town of villages (i360), does cross-polity
//! cultural diffusion actually happen?
//!
//! i360 landed the clustered world generator: N=192 → 3 settlements, N=256 → 4,
//! each with its own live polity holon. The substrate's i299 channel
//! (`system_trade_diffusion`) is supposed to let a village's genesis memes leak
//! into a neighbouring village through **cross-polity trade** — but it is
//! zero-at-zero on two conditions: (a) fewer than 2 polities (now satisfied),
//! and (b) an actual cross-polity trade, i.e. a `TradeOccurred` between agents
//! of different polities. Agent-to-agent trade needs a counter-party within 12
//! tiles, so whether distinct villages ever trade is the real question.
//!
//! This probe measures it directly: build a partitioned town, run it, and read
//! each genesis meme's **host set** (via `Meme::hosts`) for agents belonging to
//! a polity *other than* the meme's own namespace — the exact cross-boundary
//! leakage the pass is meant to produce.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i362_cross_polity_diffusion`

use mindstrata_sim::sim::{SimConfig, Simulation};

// i392: the i344 density world law is single-sourced in the world generator
// (`world_side_for_population`) — do not re-implement it.
use mindstrata_sim::world_gen::world_side_for_population as density_side;

fn main() {
    println!("i362 — cross-polity diffusion liveness (clustered town, seed 42)\n");
    for n in [192u32, 256] {
        let side = density_side(n);
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 20_000,
            world_width: side,
            world_height: side,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();
        sim.auto_partition_polities(8);
        let polities: Vec<Vec<usize>> = sim.polity_members.clone();
        println!(
            "N={n} side={side}: {} polities, members {:?}",
            polities.len(),
            polities.iter().map(std::vec::Vec::len).collect::<Vec<_>>()
        );

        sim.run(20_000);

        // Cross-boundary leakage: a genesis meme of polity K with a host that
        // belongs to another polity. `host_count` is capped at the receiving
        // polity's size, so the host SET — not the count — is the honest signal.
        let polity_of = |agent: usize| polities.iter().position(|p| p.contains(&agent));
        let mut total_memes = 0usize;
        let mut leaked_memes = 0usize;
        let mut foreign_hosts = 0usize;
        let mut per_polity_leak = vec![0usize; polities.len().max(1)];
        for meme in &sim.meme_registry.memes {
            let Some(rest) = meme.description.split("[genesis:p").nth(1) else {
                continue;
            };
            let Some(own) = rest.split(':').next().and_then(|s| s.parse::<usize>().ok()) else {
                continue;
            };
            total_memes += 1;
            let mut leaked = false;
            for &host in &meme.hosts {
                match polity_of(host) {
                    Some(p) if p != own => {
                        leaked = true;
                        foreign_hosts += 1;
                    }
                    _ => {}
                }
            }
            if leaked {
                leaked_memes += 1;
                if own < per_polity_leak.len() {
                    per_polity_leak[own] += 1;
                }
            }
        }
        // Sample recent cross-polity trade events (the channel's sampler).
        let cross_trades = sim
            .recent_events(4096)
            .iter()
            .filter(|ev| match ev {
                mindstrata_core::event::SimEvent::TradeOccurred { buyer, seller, .. } => {
                    let b = polity_of(buyer.as_u64() as usize);
                    let s = polity_of(seller.as_u64() as usize);
                    matches!((b, s), (Some(bp), Some(sp)) if bp != sp)
                }
                _ => false,
            })
            .count();

        println!(
            "  genesis memes {total_memes}  leaked across a boundary {leaked_memes}  foreign host-links {foreign_hosts}"
        );
        println!("  leaked memes per source polity {per_polity_leak:?}  cross-polity trades in last~4096 events {cross_trades}");
        println!(
            "  liveness: {}",
            if leaked_memes > 0 {
                "DIFFUSION_LIVE"
            } else {
                "DIFFUSION_DORMANT (no cross-boundary leakage)"
            }
        );
    }
}
