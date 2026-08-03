//! Economic market system — price formation, supply/demand, trade, inequality.
//!
//! §13.3: "Price formation, scarcity hoarding, inequality, specialization,
//! migration due to wages, black markets, debt spirals, firm formation, trade routes."
//!
//! Markets are emergent: prices form from aggregate supply and demand across
//! sites, not from hardcoded values. Agents trade with each other directly
//! and at market sites, with prices modulated by local scarcity.

use crate::person::NeedState;
use crate::world::World;
use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

// ── Price Model ─────────────────────────────────────────────────────────

/// Tracks price for a single resource across the settlement.
/// Price is derived from supply/demand ratio, not hardcoded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceTracker {
    /// Current price (units of coin per unit of resource).
    pub price: Fixed,
    /// 30-tick moving average of supply for smoothing.
    pub avg_supply: Fixed,
    /// 30-tick moving average of demand for smoothing.
    pub avg_demand: Fixed,
    /// Price elasticity — how quickly price responds to imbalance.
    pub elasticity: Fixed,
    /// Minimum price floor (prevents free goods).
    pub price_floor: Fixed,
    /// Maximum price ceiling (prevents infinite prices).
    pub price_ceiling: Fixed,
    /// Stable price anchor — the price a balanced market (demand == supply)
    /// converges toward. Previously the target was `price * ratio`, a
    /// degenerate fixed point that decayed to the floor whenever demand <
    /// supply and to the ceiling whenever demand > supply, so prices never
    /// hovered at intermediate values.
    pub anchor_price: Fixed,
    /// Recent transaction prices for trend analysis.
    pub recent_prices: Vec<Fixed>,
    /// Tick of last price update.
    pub last_update_tick: u64,
}

impl Default for PriceTracker {
    fn default() -> Self {
        Self {
            price: Fixed::from_f64(5.0), // starting price: 5 coins per unit
            avg_supply: Fixed::from_f64(10.0),
            avg_demand: Fixed::from_f64(10.0),
            elasticity: Fixed::from_f64(0.3),
            price_floor: Fixed::from_f64(1.0),
            price_ceiling: Fixed::from_f64(50.0),
            anchor_price: Fixed::from_f64(5.0),
            recent_prices: Vec::new(),
            last_update_tick: 0,
        }
    }
}

impl PriceTracker {
    /// Create a new price tracker with initial price.
    pub fn new(initial_price: Fixed) -> Self {
        Self {
            price: initial_price,
            anchor_price: initial_price,
            ..Default::default()
        }
    }

    /// Update price based on current supply and demand.
    /// Uses exponential moving average for smoothing.
    pub fn update(&mut self, supply: Fixed, demand: Fixed, tick: u64, params: &crate::parameters::SimParameters) {
        // Only update once per tick
        if tick == self.last_update_tick && self.last_update_tick > 0 {
            return;
        }
        self.last_update_tick = tick;

        let alpha = params.market_price_smoothing;

        // Update moving averages
        self.avg_supply = self.avg_supply * (Fixed::ONE - alpha) + supply * alpha;
        self.avg_demand = self.avg_demand * (Fixed::ONE - alpha) + demand * alpha;

        // Price adjustment: high demand / low supply → price rises
        let supply_demand_ratio = if self.avg_supply > Fixed::ZERO {
            self.avg_demand / self.avg_supply
        } else {
            params.market_no_supply_ratio // no supply → high ratio
        };

        // Price converges toward `anchor * ratio` (a stable fixed point), not
        // `price * ratio` (degenerate — decayed to floor/ceiling whenever the
        // ratio departed from 1.0, making prices inert at the bounds).
        let target_price = self.anchor_price * supply_demand_ratio;
        let price_delta = (target_price - self.price) * self.elasticity;
        self.price = (self.price + price_delta)
            .max(self.price_floor)
            .min(self.price_ceiling);

        // Track recent prices (keep last 20)
        self.recent_prices.push(self.price);
        if self.recent_prices.len() > 20 {
            self.recent_prices.remove(0);
        }
    }

    /// Get price trend: positive = rising, negative = falling.
    pub fn trend(&self) -> Fixed {
        if self.recent_prices.len() < 5 {
            return Fixed::ZERO;
        }
        let recent = self.recent_prices[self.recent_prices.len() - 1];
        let older = self.recent_prices[self.recent_prices.len() - 5];
        recent - older
    }
}

// ── Wealth Tracking ─────────────────────────────────────────────────────

/// Per-agent wealth and economic state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WealthState {
    /// Amount of coin the agent possesses.
    pub coin: Fixed,
}

impl Default for WealthState {
    fn default() -> Self {
        Self {
            coin: Fixed::from_f64(10.0), // starting wealth
        }
    }
}

// ── Market State ────────────────────────────────────────────────────────

/// Aggregate market state for the settlement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketState {
    /// Price trackers per resource (indexed by resource_id).
    pub prices: Vec<PriceTracker>,
    /// Total volume traded this tick.
    pub volume_this_tick: Fixed,
    /// Total volume traded in the last 100 ticks.
    pub recent_volume: Vec<Fixed>,
    /// Number of trades this tick.
    pub trade_count: u32,
    /// Cumulative number of completed trades over the whole run.
    /// `trade_count` is reset every tick, so dashboards that read it see 0
    /// even when the market is active; this counter never resets.
    pub total_trades: u64,
    /// Gini coefficient (0 = perfect equality, 1 = perfect inequality).
    pub inequality: Fixed,
    /// Average wealth across all agents.
    pub avg_wealth: Fixed,
    /// Median wealth.
    pub median_wealth: Fixed,
    /// Default price for unknown resources.
    pub default_price: Fixed,
}

impl Default for MarketState {
    fn default() -> Self {
        Self {
            prices: Vec::new(),
            volume_this_tick: Fixed::ZERO,
            recent_volume: Vec::new(),
            trade_count: 0,
            total_trades: 0,
            inequality: Fixed::ZERO,
            avg_wealth: Fixed::ZERO,
            median_wealth: Fixed::ZERO,
            default_price: Fixed::from_f64(10.0),
        }
    }
}

impl MarketState {
    /// Create market state with default prices for grain and water.
    pub fn new(params: &crate::parameters::SimParameters) -> Self {
        Self {
            default_price: params.market_default_price,
            prices: vec![
                PriceTracker::new(params.market_initial_grain_price),
                PriceTracker::new(params.market_initial_water_price),
            ],
            ..Self::default()
        }
    }

    /// Get the current price for a resource.
    pub fn price(&self, resource_id: u64) -> Fixed {
        self.prices
            .get(resource_id as usize)
            .map_or(self.default_price, |p| p.price)
    }

    /// Get price trend for a resource.
    pub fn price_trend(&self, resource_id: u64) -> Fixed {
        self.prices
            .get(resource_id as usize)
            .map_or(Fixed::ZERO, PriceTracker::trend)
    }
}

// ── Trade Mechanics ─────────────────────────────────────────────────────

/// Result of a trade attempt.
#[derive(Debug, Clone)]
pub enum TradeResult {
    Success {
        resource_id: u64,
        quantity: Fixed,
        price: Fixed,
        total_cost: Fixed,
    },
    InsufficientFunds,
    InsufficientStock,
    NoMarket,
    NoPartner,
}

/// Attempt an agent-to-agent trade at the market.
///
/// The buyer pays coin, the seller provides goods.
/// Price is determined by the market's current price tracker.
pub fn execute_trade(
    buyer_wealth: &mut WealthState,
    seller_stock: &mut Fixed, // seller's resource quantity
    resource_id: u64,
    quantity: Fixed,
    market: &mut MarketState,
) -> TradeResult {
    let price = market.price(resource_id);
    let total_cost = price * quantity;

    // Check buyer has enough coin
    if buyer_wealth.coin < total_cost {
        return TradeResult::InsufficientFunds;
    }

    // Check seller has enough stock
    if *seller_stock < quantity {
        return TradeResult::InsufficientStock;
    }

    // Execute trade
    buyer_wealth.coin = (buyer_wealth.coin - total_cost).max(Fixed::ZERO);
    *seller_stock = (*seller_stock - quantity).max(Fixed::ZERO);

    // Update market volume
    market.volume_this_tick += quantity;
    market.trade_count += 1;
    market.total_trades = market.total_trades.saturating_add(1);

    TradeResult::Success {
        resource_id,
        quantity,
        price,
        total_cost,
    }
}

/// Attempt agent-to-agent direct trade (no market site required).
///
/// Two agents exchange goods directly based on their needs and resources.
/// Prices are influenced by the market but can be negotiated via relationship trust.
pub fn direct_trade(
    buyer_wealth: &mut WealthState,
    seller_stock: &mut Fixed,
    resource_id: u64,
    quantity: Fixed,
    trust: Fixed, // relationship trust between buyer and seller
    market: &mut MarketState,
    params: &crate::parameters::SimParameters,
) -> TradeResult {
    let base_price = market.price(resource_id);

    // Trust discount: high trust → lower price, low trust → higher price
    // Trust 1.0 → 80% of base price, Trust 0.0 → 120% of base price
    let trust_modifier = Fixed::ONE - trust * params.market_trust_discount + (Fixed::ONE - trust) * params.market_trust_discount;
    let price = (base_price * trust_modifier).max(market.prices.get(resource_id as usize).map_or(Fixed::ONE, |p| p.price_floor));

    let total_cost = price * quantity;

    if buyer_wealth.coin < total_cost {
        return TradeResult::InsufficientFunds;
    }

    if *seller_stock < quantity {
        return TradeResult::InsufficientStock;
    }

    // Execute direct trade
    buyer_wealth.coin = (buyer_wealth.coin - total_cost).max(Fixed::ZERO);
    *seller_stock = (*seller_stock - quantity).max(Fixed::ZERO);

    market.volume_this_tick += quantity;
    market.trade_count += 1;
    market.total_trades = market.total_trades.saturating_add(1);

    TradeResult::Success {
        resource_id,
        quantity,
        price,
        total_cost,
    }
}

// ── Supply/Demand Computation ───────────────────────────────────────────

/// Compute aggregate supply of a resource across all sites.
pub fn compute_supply(world: &World, resource_id: u64) -> Fixed {
    world
        .sites
        .iter()
        .flat_map(|s| &s.inventory)
        .filter(|r| r.resource_id == resource_id)
        .fold(Fixed::ZERO, |acc, r| acc + r.quantity)
}

/// Compute aggregate demand for a resource based on agent needs.
///
/// Demand is weighted by need pressure: agents with high hunger
/// demand more grain, etc.
pub fn compute_demand(agents: &[(NeedState, WealthState)], resource_id: u64, params: &crate::parameters::SimParameters) -> Fixed {
    agents
        .iter()
        .fold(Fixed::ZERO, |acc, (needs, wealth)| {
            // Linear need pressure scaled by demand weight. The weight is set
            // to the expected per-agent consumption (EXPECTED_GRAIN_PER_AGENT
            // ≈ 10), so demand is the same order of magnitude as aggregate
            // supply — without this, hunger²·2 was ~2 units vs ~100 units of
            // supply, pinning prices at the floor forever.
            let need_pressure = match resource_id {
                0 => needs.hunger * params.market_demand_weight, // grain
                1 => needs.thirst * params.market_demand_weight, // water
                _ => Fixed::ZERO,
            };
            // Only demand if agent can afford something
            let purchasing_power = (wealth.coin / params.market_purchasing_power_divisor).clamp_01();
            acc + need_pressure * purchasing_power
        })
}

// ── Inequality Metrics ──────────────────────────────────────────────────

/// Compute Gini coefficient from a list of wealth values.
///
/// Gini = 0 means perfect equality, Gini = 1 means one person has everything.
pub fn compute_gini(wealths: &[Fixed]) -> Fixed {
    if wealths.is_empty() || wealths.len() == 1 {
        return Fixed::ZERO;
    }

    let n = Fixed::from_int(wealths.len() as i64);
    let mean = wealths.iter().fold(Fixed::ZERO, |a, b| a + *b) / n;

    if mean <= Fixed::ZERO {
        return Fixed::ZERO; // all zero wealth → no inequality
    }

    let mut total_diff = Fixed::ZERO;
    for (i, w1) in wealths.iter().enumerate() {
        for w2 in wealths.iter().skip(i + 1) {
            total_diff += (*w1 - *w2).abs();
        }
    }

    let num_pairs = Fixed::from_int(((wealths.len() * (wealths.len() - 1)) / 2) as i64);
    total_diff / (num_pairs * mean * Fixed::from_f64(2.0))
}

/// Compute median wealth from a list.
pub fn compute_median(wealths: &mut [Fixed]) -> Fixed {
    if wealths.is_empty() {
        return Fixed::ZERO;
    }
    wealths.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = wealths.len() / 2;
    if wealths.len().is_multiple_of(2) {
        (wealths[mid - 1] + wealths[mid]) / Fixed::from_f64(2.0)
    } else {
        wealths[mid]
    }
}

// ── Market System ───────────────────────────────────────────────────────

/// Run the market system each tick.
///
/// Updates prices based on supply/demand, computes inequality metrics,
/// and resets per-tick counters.
pub fn system_market(
    world: &World,
    agents: &[(NeedState, WealthState)],
    market: &mut MarketState,
    tick: u64,
    params: &crate::parameters::SimParameters,
) {
    // Update prices for each tracked resource
    for (resource_id, tracker) in market.prices.iter_mut().enumerate() {
        let supply = compute_supply(world, resource_id as u64);
        let demand = compute_demand(agents, resource_id as u64, params);
        tracker.update(supply, demand, tick, params);
    }

    // Compute inequality
    let mut wealths: Vec<Fixed> = agents.iter().map(|(_, w)| w.coin).collect();
    market.inequality = compute_gini(&wealths);
    market.avg_wealth = if !agents.is_empty() {
        wealths.iter().fold(Fixed::ZERO, |a, b| a + *b) / Fixed::from_int(agents.len() as i64)
    } else {
        Fixed::ZERO
    };
    market.median_wealth = compute_median(&mut wealths);

    // Track volume history
    market.recent_volume.push(market.volume_this_tick);
    if market.recent_volume.len() > 100 {
        market.recent_volume.remove(0);
    }

    // Reset per-tick counters
    market.volume_this_tick = Fixed::ZERO;
    market.trade_count = 0;
}

// ── Supply/Demand Price Modifier for Actions ────────────────────────────

/// Compute a scarcity-based price modifier for utility calculations.
///
/// When a resource is scarce, the effective "cost" of acquiring it increases,
/// making alternatives more attractive. This creates economic pressure.
pub fn scarcity_modifier(supply: Fixed, max_expected: Fixed, params: &crate::parameters::SimParameters) -> Fixed {
    if supply <= Fixed::ZERO {
        params.market_scarcity_extreme // extreme scarcity → 2x cost
    } else if supply >= max_expected {
        params.market_scarcity_abundance // abundance → 0.5x cost
    } else {
        // Linear interpolation between 0.5 and 2.0
        let ratio = supply / max_expected;
        params.market_scarcity_extreme - ratio * params.market_scarcity_range
    }
}

#[cfg(test)]
mod tests {
    use crate::parameters::SimParameters;
    use super::*;
    use crate::world::GRAIN_RESOURCE_ID;

    #[test]
    fn price_rises_with_scarcity() {
        let mut tracker = PriceTracker::new(Fixed::from_f64(5.0));
        // High demand, low supply → price should rise
        tracker.update(Fixed::from_f64(1.0), Fixed::from_f64(10.0), 1, &SimParameters::default());
        assert!(tracker.price > Fixed::from_f64(5.0), "Price should rise with scarcity");
    }

    #[test]
    fn price_falls_with_abundance() {
        let mut tracker = PriceTracker::new(Fixed::from_f64(5.0));
        // Low demand, high supply → price should fall
        tracker.update(Fixed::from_f64(10.0), Fixed::from_f64(1.0), 1, &SimParameters::default());
        assert!(tracker.price < Fixed::from_f64(5.0), "Price should fall with abundance");
    }

    #[test]
    fn gini_zero_for_equal_wealth() {
        let wealths = vec![
            Fixed::from_f64(10.0),
            Fixed::from_f64(10.0),
            Fixed::from_f64(10.0),
        ];
        let gini = compute_gini(&wealths);
        assert!(gini < Fixed::from_f64(0.01), "Gini should be near 0 for equal wealth");
    }

    #[test]
    fn gini_high_for_unequal_wealth() {
        let wealths = vec![
            Fixed::from_f64(100.0),
            Fixed::from_f64(0.0),
            Fixed::from_f64(0.0),
        ];
        let gini = compute_gini(&wealths);
        assert!(gini > Fixed::from_f64(0.5), "Gini should be high for unequal wealth");
    }

    #[test]
    fn trade_reduces_buyer_coin() {
        let mut buyer = WealthState { coin: Fixed::from_f64(20.0) };
        let mut seller_stock = Fixed::from_f64(5.0);
        let mut market = MarketState::new(&SimParameters::default());

        let result = execute_trade(&mut buyer, &mut seller_stock, 0, Fixed::from_f64(1.0), &mut market);
        match result {
            TradeResult::Success { total_cost, .. } => {
                assert!(buyer.coin < Fixed::from_f64(20.0));
                assert!(buyer.coin == Fixed::from_f64(20.0) - total_cost);
            }
            _ => panic!("Trade should succeed"),
        }
    }

    #[test]
    fn trade_fails_with_insufficient_funds() {
        let mut buyer = WealthState { coin: Fixed::from_f64(1.0) };
        let mut seller_stock = Fixed::from_f64(5.0);
        let mut market = MarketState::new(&SimParameters::default());

        let result = execute_trade(&mut buyer, &mut seller_stock, 0, Fixed::from_f64(1.0), &mut market);
        assert!(matches!(result, TradeResult::InsufficientFunds));
    }

    #[test]
    fn direct_trade_respects_trust() {
        let mut buyer_high_trust = WealthState { coin: Fixed::from_f64(100.0) };
        let mut buyer_low_trust = WealthState { coin: Fixed::from_f64(100.0) };
        let mut stock1 = Fixed::from_f64(10.0);
        let mut stock2 = Fixed::from_f64(10.0);
        let mut market = MarketState::new(&SimParameters::default());

        let r1 = direct_trade(&mut buyer_high_trust, &mut stock1, 0, Fixed::from_f64(1.0), Fixed::from_f64(0.9), &mut market, &SimParameters::default());
        let r2 = direct_trade(&mut buyer_low_trust, &mut stock2, 0, Fixed::from_f64(1.0), Fixed::from_f64(0.1), &mut market, &SimParameters::default());

        match (r1, r2) {
            (TradeResult::Success { total_cost: cost1, .. }, TradeResult::Success { total_cost: cost2, .. }) => {
                assert!(cost1 < cost2, "High trust should get lower price");
            }
            _ => panic!("Both trades should succeed"),
        }
    }

    #[test]
    fn scarcity_modifier_increases_with_scarcity() {
        let abundant = scarcity_modifier(Fixed::from_f64(10.0), Fixed::from_f64(10.0), &SimParameters::default());
        let scarce = scarcity_modifier(Fixed::from_f64(1.0), Fixed::from_f64(10.0), &SimParameters::default());
        assert!(scarce > abundant, "Scarcity modifier should increase with scarcity");
    }

    #[test]
    fn compute_supply_sums_resources() {
        let world = World::new(4, 4);
        // Empty world should have 0 supply
        let supply = compute_supply(&world, GRAIN_RESOURCE_ID);
        assert_eq!(supply, Fixed::ZERO);
    }

    #[test]
    fn price_floor_prevents_negative() {
        let mut tracker = PriceTracker::new(Fixed::from_f64(2.0));
        // Extreme abundance should not drop below floor
        for tick in 1..1000 {
            tracker.update(Fixed::from_f64(100.0), Fixed::from_f64(0.01), tick, &SimParameters::default());
        }
        assert!(tracker.price >= tracker.price_floor, "Price should not go below floor");
    }

    #[test]
    fn price_ceiling_prevents_infinite() {
        let mut tracker = PriceTracker::new(Fixed::from_f64(5.0));
        // Extreme scarcity should not exceed ceiling
        for tick in 1..1000 {
            tracker.update(Fixed::from_f64(0.01), Fixed::from_f64(100.0), tick, &SimParameters::default());
        }
        assert!(tracker.price <= tracker.price_ceiling, "Price should not exceed ceiling");
    }
}
