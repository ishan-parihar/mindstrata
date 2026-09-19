//! i299 — Cross-polity cultural diffusion via trade (UM-3 leg 3; PLAN_DC3 §8).
//!
//! Exit evidence for `system_trade_diffusion`. The channel mechanics are
//! pinned at unit level (4 pins in `systems/trade_diffusion.rs`: cross-polity
//! trade diffuses the sender's memes; intra-polity trade does not; no
//! polities → no-op; a trade never diffuses the receiver's own memes). This
//! probe measures the channel's IN-VIVO diet and documents the measurement
//! boundary discovered while building it:
//!
//! 1. **Zero-at-zero in vivo** — no polities → zero namespaced memes (the
//!    pass never fires; legacy runs byte-identical).
//! 2. **Cross-boundary diet** — trading between ring-pole polities is dense
//!    (~1900 cross trades / 10K ticks): the shared market/farm sites are
//!    natural contact boundaries, and wandering agents meet across the
//!    partition regardless of home-site geometry.
//! 3. **Measurement boundary (recorded debt)** — `host_count` cannot isolate
//!    diffusion from intra-polity gossip (both grow the same counter, both
//!    saturate at the population cap), and an A/B geometry run fails because
//!    wandering erases static separation and shared sites force contact.
//!    The clean signal is per-agent meme hosting (`meme.hosts`), a frozen
//!    schema change → recorded for the i300 gate design.
//!
//! Run: cargo run -p mindstrata-benches --release --example i299_trade_diffusion
use mindstrata_sim::sim::{SimConfig, Simulation};

fn setup(pole_shift: i32) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: 0,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    // Two opposite-ring houses as settlement poles.
    let houses: Vec<usize> = sim
        .world
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.kind == mindstrata_sim::world::SiteKind::House)
        .map(|(i, _)| i)
        .collect();
    let (far, second) = {
        let (x0, y0) = sim.world.site_position(0).unwrap_or((0, 0));
        let far = *houses
            .iter()
            .max_by_key(|&&i| {
                let (x, y) = sim.world.site_position(i).unwrap_or((0, 0));
                (x - x0).abs() + (y - y0).abs()
            })
            .unwrap();
        let (xf, yf) = sim.world.site_position(far).unwrap_or((0, 0));
        let second = *houses
            .iter()
            .filter(|&&i| i != far)
            .max_by_key(|&&i| {
                let (x, y) = sim.world.site_position(i).unwrap_or((0, 0));
                (x - xf).abs() + (y - yf).abs()
            })
            .unwrap();
        (far, second)
    };
    let n = 12usize;
    for (i, agent) in sim.agents.iter_mut().enumerate() {
        let home = if i < n / 2 { far } else { second };
        agent.home_site = Some(home);
        if let Some((x, y)) = sim.world.site_position(home) {
            // pole_shift separates the second polity's initial positions —
            // the wandering pass erases this within days (measured), which is
            // itself a finding: contact cannot be geometry-blocked at N=12.
            let (dx, dy) = if i < n / 2 {
                (0, 0)
            } else {
                (pole_shift, pole_shift)
            };
            agent.position = mindstrata_sim::sim::Position::new(x + dx, y + dy);
        }
    }
    sim.auto_partition_polities(6);
    sim
}

fn main() {
    let horizon = 10_000u64;
    let seed = 42u64;
    println!("i299 trade diffusion — {horizon} ticks, seed {seed}, N=12");

    // ── 1. Zero-at-zero in vivo: no polities → no namespaced memes.
    let mut sim0 = Simulation::new(SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim0.populate();
    sim0.run(horizon);
    let ns0 = sim0
        .meme_registry
        .memes
        .iter()
        .filter(|m| m.description.contains("[genesis:p"))
        .count();
    println!();
    println!("1. ZERO-AT-ZERO (no polities): p-namespaced memes = {ns0} (must be 0)");

    // ── 2. Contact run: two polities, natural trade, 10K ticks.
    let mut sim2 = setup(0);
    let polities = sim2.polity_members.len();
    sim2.run(horizon);

    let mut total_trades = 0usize;
    let mut cross_trades = 0usize;
    let window: Vec<_> = sim2.recent_events(100_000).to_vec();
    for ev in &window {
        let (a, b) = match ev {
            mindstrata_core::event::SimEvent::TradeOccurred { buyer, seller, .. } => {
                total_trades += 1;
                (buyer.as_u64() as usize, seller.as_u64() as usize)
            }
            mindstrata_core::event::SimEvent::InteractionOccurred {
                from,
                to,
                kind: mindstrata_core::event::InteractionKind::Trade,
                ..
            } => {
                total_trades += 1;
                (from.as_u64() as usize, to.as_u64() as usize)
            }
            _ => continue,
        };
        let pa = sim2.polity_members.iter().position(|m| m.contains(&a));
        let pb = sim2.polity_members.iter().position(|m| m.contains(&b));
        if pa.is_some() && pb.is_some() && pa != pb {
            cross_trades += 1;
        }
    }

    println!();
    println!("2. CHANNEL DIET (two ring-pole polities):");
    println!("   polities: {polities}");
    println!("   [last-50K-window] trade events: {total_trades}, cross-polity: {cross_trades}");
    for ns in ["p0", "p1"] {
        let memes: Vec<(u64, u32)> = sim2
            .meme_registry
            .memes
            .iter()
            .filter(|m| m.description.contains(&format!("[genesis:{ns}:")))
            .map(|m| (m.created_tick, m.host_count))
            .collect();
        let hosts: Vec<u32> = memes.iter().map(|(_, h)| *h).collect();
        println!("   {ns} genesis memes: {} hosts {hosts:?}", memes.len());
    }

    // ── 3. Verdict + recorded debt.
    println!();
    println!("3. VERDICT:");
    if ns0 == 0 && cross_trades > 0 {
        println!(
            "   zero-at-zero: PASS; cross-boundary trade diet: LOADED ({cross_trades} in window)"
        );
        println!("   the diffusion pass is live and fed (unit pins prove mechanics);");
        println!("   per-polity isolation of the diffusion CONTRIBUTION needs meme.hosts schema");
        println!("   (per-agent hosting sets) — recorded for the i300 UM-3 gate design.");
    } else {
        println!(
            "   unexpected state: ns0={ns0}, cross={cross_trades} — investigate before landing"
        );
    }
}
