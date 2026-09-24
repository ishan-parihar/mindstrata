//! i384 — the economy trade trust pair (read + write) sits on the v1 matrix.
//! What does that cost, and what does migrating it to the dyadic store change?
//!
//! `economy.rs` is the last self-contained **read + write pair** on the legacy
//! `relationships` matrix: the trade price discounts by `trust` read from v1
//! (`1.1 − 0.3·trust`), and the successful trade writes `trust += 0.02` back to
//! v1. i376 measured the two stores ~+0.06 apart (v1 above v2) because v1 gains
//! ~10× faster per act. This probe sizes that split **at the trade site**:
//!
//! Leg A — read-source divergence: for every executed trade, both stores' trust
//!   on the ordered pair (buyer → seller), the resulting price modifier under
//!   each, and the price delta the migration would introduce. Plus the
//!   population-wide store offset (i376's number), re-measured on these worlds.
//! Leg B — the write: how much trust a trade builds in the store it writes, and
//!   whether that is visible against the store's own decay.
//! Leg C — the contracts the economy is pinned by: trades, volume, price, coin
//!   distribution (mean/median/min, Gini), so the blast radius is attributable
//!   rather than guessed.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i384_economy_trade_store`

use mindstrata_sim::sim::{SimConfig, Simulation};
use mindstrata_sim::world::GRAIN_RESOURCE_ID;

/// The shipped v1 trade write, verbatim (`economy.rs`).
const V1_TRADE_GAIN: f64 = 0.02;
/// The shipped trade price discount: `1 − 0.2·trust + 0.1·(1 − trust)`.
fn price_modifier(trust: f64) -> f64 {
    1.0 - trust * 0.2 + (1.0 - trust) * 0.1
}

struct Report {
    label: String,
    trades: usize,
    mean_price: f64,
    mean_qty: f64,
    /// Mean of both stores' trust on the ordered pairs that actually traded.
    trade_v1: f64,
    trade_v2: f64,
    /// Mean `|modifier(v1) − modifier(v2)|` on traded pairs — the read-source
    /// delta the migration removes by construction.
    trade_mod_delta: f64,
    /// Population-wide store offset over all ordered pairs.
    pop_v1: f64,
    pop_v2: f64,
    pop_pairs: usize,
    pop_ge_05: f64,
    /// Contracts.
    total_trades: u64,
    gini: f64,
    avg_wealth: f64,
    median_wealth: f64,
    min_coin: f64,
    destitute: usize,
}

fn run_world(
    label: &str,
    seed: u64,
    ticks: u64,
    width: u32,
    height: u32,
    agent_count: u32,
) -> Report {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: width,
        world_height: height,
        num_agents: agent_count,
        snapshot_interval: None,
    });
    sim.populate();

    let mut trades = 0usize;
    let mut price_sum = 0.0;
    let mut qty_sum = 0.0;
    let mut v1_sum = 0.0;
    let mut v2_sum = 0.0;
    let mut mod_delta_sum = 0.0;
    let mut last_total = sim.event_count();

    for _ in 0..ticks {
        sim.run(1);
        let now = sim.event_count();
        let new = now - last_total;
        last_total = now;
        if new == 0 {
            continue;
        }
        for ev in sim.recent_events(new) {
            let mindstrata_core::event::SimEvent::TradeOccurred {
                buyer,
                seller,
                good,
                quantity,
                price,
                ..
            } = ev
            else {
                continue;
            };
            if *good != mindstrata_core::id::ResourceId::new(GRAIN_RESOURCE_ID) {
                continue;
            }
            let buyer_idx = buyer.as_u64() as usize;
            let seller_idx = seller.as_u64() as usize;
            // v1 read, first-occurrence semantics (the same element `rel_pos`
            // resolves) — the probe cannot call the crate-private accessor.
            let v1_trade_trust = sim
                .relationships
                .iter()
                .find(|rel| {
                    rel.from.as_u64() as usize == buyer_idx
                        && rel.to.as_u64() as usize == seller_idx
                })
                .map_or(0.5, |rel| rel.trust.to_f64());
            let v2_trade_trust = sim
                .relationship_v2_between(buyer_idx, seller_idx)
                .map_or(0.5, |rel| rel.trust.to_f64());
            trades += 1;
            price_sum += price.to_f64();
            qty_sum += quantity.to_f64();
            v1_sum += v1_trade_trust;
            v2_sum += v2_trade_trust;
            mod_delta_sum +=
                (price_modifier(v1_trade_trust) - price_modifier(v2_trade_trust)).abs();
        }
    }

    // Population-wide store offset over every ordered pair.
    let mut pop_v1 = 0.0;
    let mut pop_v2 = 0.0;
    let mut pop_pairs = 0usize;
    let mut pop_ge_05 = 0.0;
    for rel in &sim.relationships {
        let from_idx = rel.from.as_u64() as usize;
        let to_idx = rel.to.as_u64() as usize;
        let Some(v2_pair_trust) = sim.relationship_v2_between(from_idx, to_idx) else {
            continue;
        };
        let v1_pair_trust = rel.trust.to_f64();
        let v2_trust = v2_pair_trust.trust.to_f64();
        pop_v1 += v1_pair_trust;
        pop_v2 += v2_trust;
        pop_pairs += 1;
        if (v1_pair_trust - v2_trust).abs() >= 0.05 {
            pop_ge_05 += 1.0;
        }
    }

    let coins: Vec<f64> = sim.agents.iter().map(|a| a.wealth.coin.to_f64()).collect();
    let min_coin = coins.iter().copied().fold(f64::INFINITY, f64::min);
    let destitute = coins.iter().filter(|coin| **coin < 1.0).count();

    let trade_norm = trades.max(1) as f64;
    Report {
        label: label.to_string(),
        trades,
        mean_price: price_sum / trade_norm,
        mean_qty: qty_sum / trade_norm,
        trade_v1: v1_sum / trade_norm,
        trade_v2: v2_sum / trade_norm,
        trade_mod_delta: mod_delta_sum / trade_norm,
        pop_v1: pop_v1 / pop_pairs.max(1) as f64,
        pop_v2: pop_v2 / pop_pairs.max(1) as f64,
        pop_pairs,
        pop_ge_05: pop_ge_05 / pop_pairs.max(1) as f64 * 100.0,
        total_trades: sim.market.total_trades,
        gini: sim.market.inequality.to_f64(),
        avg_wealth: sim.market.avg_wealth.to_f64(),
        median_wealth: sim.market.median_wealth.to_f64(),
        min_coin,
        destitute,
    }
}

fn print(r: &Report) {
    println!("── {} ──", r.label);
    println!(
        "  trades {:>7}  mean price {:.3}  mean qty {:.3}  market.total {:>7}",
        r.trades, r.mean_price, r.mean_qty, r.total_trades
    );
    println!(
        "  traded-pair trust: v1 {:.4}  v2 {:.4}  Δ {:.4}  |Δ price modifier| {:.4} ({:.2}%)",
        r.trade_v1,
        r.trade_v2,
        r.trade_v1 - r.trade_v2,
        r.trade_mod_delta,
        r.trade_mod_delta * 100.0
    );
    println!(
        "  all-pair store: v1 {:.4}  v2 {:.4}  Δ {:+.4}  pairs {}  ≥0.05 apart {:.1}%",
        r.pop_v1,
        r.pop_v2,
        r.pop_v1 - r.pop_v2,
        r.pop_pairs,
        r.pop_ge_05
    );
    println!(
        "  contracts: gini {:.4}  avg coin {:.2}  median {:.2}  min {:.2}  destitute(<1) {}",
        r.gini, r.avg_wealth, r.median_wealth, r.min_coin, r.destitute
    );
    println!(
        "  v1 trade gain/act {:+.4} vs dyadic record_positive(1.0) {:+.4} (volatility 0.5)",
        V1_TRADE_GAIN,
        0.5 * 0.02
    );
}

fn main() {
    let worlds: Vec<(&str, u64, u32, u32, u32)> = vec![
        ("village 16x16 N=12 seed42", 42, 16, 16, 12),
        ("town    46x46 N=48 seed42", 42, 46, 46, 48),
        ("town    46x46 N=48 seed07", 7, 46, 46, 48),
    ];
    let ticks = 20_000;
    let mut reports = Vec::new();
    for (label, seed, w, h, n) in worlds {
        let r = run_world(label, seed, ticks, w, h, n);
        print(&r);
        reports.push(r);
    }
    println!("\n=== summary ===");
    let tot: usize = reports.iter().map(|r| r.trades).sum();
    let mod_delta: f64 =
        reports.iter().map(|r| r.trade_mod_delta).sum::<f64>() / reports.len() as f64;
    println!("total trades across worlds: {tot}");
    println!("mean |price-modifier delta| the migration removes: {mod_delta:.4}");
}
