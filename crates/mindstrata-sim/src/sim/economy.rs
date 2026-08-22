//! Resource operations, storage overflow and aggregate stock accessors.

use super::{
    ActionKind, AgentId, Fixed, InstitutionKind, JournalEntryKind, ResourceId, SimEvent,
    Simulation, Tick, GRAIN_RESOURCE_ID, LOGISTICS_PRICE_DAMP, TREND_BUY_RATE, WATER_RESOURCE_ID,
};
use crate::health;
use crate::logistics;

impl Simulation {
    /// Get total grain across all farms.
    pub fn total_grain(&self) -> Fixed {
        self.world.total_food()
    }

    /// Get total water across all wells.
    pub fn total_water(&self) -> Fixed {
        self.world.total_water()
    }

    /// §8.1.18: Apprenticeship pass — deliberate knowledge transmission.
    ///
    /// Runs once daily. Each agent attempts to learn one knowledge item they
    /// lack, taught by an agent who holds it with sufficient teaching skill
    /// and a warm relationship. Deterministic: the teacher with the highest
    /// teaching skill is picked (no RNG), the transfer succeeds when the
    /// learning rate clears the `attempt_teaching` threshold, and success
    /// feeds the §8.1.7 desacralization chain (acquired knowledge is evidence
    /// exposure) — the same hook gossip uses.
    /// §19.5.E: Sites have finite storage — goods stored beyond a site's
    /// storage capacity are exposed and spoil at 10x the base seasonal rate.
    /// Transparent while inventories stay under capacity; binds only when
    /// production outpaces consumption (bountiful harvests rot without
    /// adequate storage, creating boom-bust scarcity pressure).
    pub(super) fn apply_storage_overflow(&mut self) {
        let resource_defs = &self.world.resources;
        let spoilage_modifier = self.season.current.spoilage_modifier();
        for site in &mut self.world.sites {
            let mut storage = logistics::StorageCapacity::new(site.storage_capacity);
            storage.update_from_inventory(&site.inventory);
            let overflow = storage.current_used - site.storage_capacity;
            if overflow > Fixed::ZERO && storage.current_used > Fixed::ZERO {
                // Every stock shares the overflow burden proportionally; the
                // exposed portion decays at 10x the normal spoilage rate.
                let spill_ratio = overflow / storage.current_used;
                for stock in &mut site.inventory {
                    if let Some(res_def) = resource_defs.iter().find(|r| r.id == stock.resource_id)
                    {
                        if res_def.perishable && res_def.spoilage_rate > Fixed::ZERO {
                            let loss = stock.quantity
                                * spill_ratio
                                * res_def.spoilage_rate
                                * Fixed::from_f64(10.0)
                                * spoilage_modifier;
                            stock.quantity = (stock.quantity - loss).max(Fixed::ZERO);
                        }
                    }
                }
            }
        }
    }

    pub(super) fn tick_resource_operations(
        &mut self,
        action_starts: &[(usize, ActionKind)],
        _pre_tick_events: usize,
        tick_u64: u64,
        tick: Tick,
    ) {
        // ── 4b. Resource operations, journal recording, intention tracking ──

        for (agent_idx, action) in action_starts {
            let agent_id = AgentId::new(*agent_idx as u64);
            // §24.5: Track intention success/failure when action completes.
            // The resource operation outcome (e.g., Eat failing because no grain)
            // is known here, so we record intention success/failure based on
            // the actual resource operation result.
            let action_succeeded = match action {
                ActionKind::Eat => {
                    let amount = Fixed::from_f64(0.1);
                    // §19.5.E: Use access-checking method to enforce resource access rights.
                    // §8.1.4 (P3-6): a FAILED Eat must not relieve hunger — the
                    // per-tick relief in `apply_action_tick` is unconditional, so
                    // an agent whose Eat finds no grain anywhere still "ate" and
                    // hunger never accumulated in famine/pestilence (probe:
                    // needs.hunger 0.006–0.041 in EVERY scenario, famine ≈ calm
                    // — the goal-incongruence branch was unreachable). The
                    // failure paths below revert the relief applied this tick and
                    // cancel the action so its remaining duration ticks don't
                    // relieve either: no food consumed, no satiation.
                    //
                    // A portion is a full meal: `accessible_farm_with_grain`
                    // matches ANY nonzero stock, and `consume_resource` takes
                    // `min(amount, stock)` — so with 0.015 grain left, an Eat
                    // "succeeded" and fully relieved hunger (crumbs counted as
                    // meals, keeping hunger pinned near 0 through the famine
                    // window). Success now requires the FULL portion; a farm
                    // holding less than a meal's worth fails the Eat.
                    if let Some(farm_idx) = self.world.accessible_farm_with_grain_amount(
                        agent_id,
                        &self.institutions,
                        amount,
                    ) {
                        let taken =
                            self.world
                                .consume_resource(farm_idx, GRAIN_RESOURCE_ID, amount);
                        // §13.3: Consumers pay coin for resources consumed. Coin is not
                        // destroyed — it flows into the Market treasury (which then
                        // funds wages), closing the wealth-destruction loop.
                        let cost = taken * self.market.price(GRAIN_RESOURCE_ID);
                        let paid = cost.min(self.agents[*agent_idx].wealth.coin);
                        self.agents[*agent_idx].wealth.coin =
                            (self.agents[*agent_idx].wealth.coin - paid).max(Fixed::ZERO);
                        if let Some(market) = self
                            .institutions
                            .iter_mut()
                            .find(|i| i.kind == InstitutionKind::Market)
                        {
                            market.treasury = (market.treasury + paid).max(Fixed::ZERO);
                        }
                        self.journal.record(
                            tick_u64,
                            agent_id,
                            JournalEntryKind::Consumed {
                                resource: "grain".into(),
                                amount: taken.to_f64(),
                            },
                        );
                        // §7.2.7 (Iteration 212): food quality now derives
                        // from world food abundance (same ratio as the biology
                        // tick_update). In famine, digestible quality drops.
                        if taken > Fixed::ZERO {
                            let total = self.world.total_food();
                            let exp = crate::sim::EXPECTED_GRAIN_PER_AGENT as i64
                                * self.agents.len() as i64;
                            let food_quality = if exp > 0 {
                                (total / Fixed::from_int(exp))
                                    .clamp(Fixed::from_f64(0.1), Fixed::ONE)
                            } else {
                                Fixed::from_f64(0.6)
                            };
                            self.agents[*agent_idx]
                                .embodied
                                .digestive
                                .consume_food(food_quality);
                        }
                        taken > Fixed::ZERO
                    } else {
                        // §19.5.D: Theft detection — no accessible farm with a
                        // full portion, but inaccessible farms holding one exist?
                        // (P3-6: amount-gated like the accessible path — a crumb
                        // is not worth stealing and must not feed the thief.)
                        if let Some(thief_idx) = self.world.inaccessible_farm_with_grain_amount(
                            agent_id,
                            &self.institutions,
                            amount,
                        ) {
                            self.enforce_theft(
                                *agent_idx,
                                agent_id,
                                thief_idx,
                                GRAIN_RESOURCE_ID,
                                amount,
                                tick_u64,
                                tick,
                            )
                        } else {
                            false
                        }
                    }
                }
                ActionKind::Drink => {
                    let amount = Fixed::from_f64(0.15);
                    // §19.5.E: Use access-checking method to enforce resource access rights.
                    if let Some(well_idx) = self
                        .world
                        .accessible_well_with_water(agent_id, &self.institutions)
                    {
                        let taken =
                            self.world
                                .consume_resource(well_idx, WATER_RESOURCE_ID, amount);
                        // §13.3: Consumers pay coin for water consumed. Coin flows into
                        // the Market treasury instead of vanishing.
                        let cost = taken * self.market.price(WATER_RESOURCE_ID);
                        let paid = cost.min(self.agents[*agent_idx].wealth.coin);
                        self.agents[*agent_idx].wealth.coin =
                            (self.agents[*agent_idx].wealth.coin - paid).max(Fixed::ZERO);
                        if let Some(market) = self
                            .institutions
                            .iter_mut()
                            .find(|i| i.kind == InstitutionKind::Market)
                        {
                            market.treasury = (market.treasury + paid).max(Fixed::ZERO);
                        }
                        self.journal.record(
                            tick_u64,
                            agent_id,
                            JournalEntryKind::Consumed {
                                resource: "water".into(),
                                amount: taken.to_f64(),
                            },
                        );
                        taken > Fixed::ZERO
                    } else {
                        // §19.5.D: Water theft detection — no accessible well, but inaccessible wells with water exist?
                        if let Some(thief_idx) = self
                            .world
                            .inaccessible_well_with_water(agent_id, &self.institutions)
                        {
                            self.enforce_theft(
                                *agent_idx,
                                agent_id,
                                thief_idx,
                                WATER_RESOURCE_ID,
                                amount,
                                tick_u64,
                                tick,
                            )
                        } else {
                            false
                        }
                    }
                }
                ActionKind::Work => {
                    // §4.2: Productivity = base (conscientiousness) + skill bonus
                    let skill_bonus =
                        self.agents[*agent_idx].skills.farming * Fixed::from_f64(0.02);
                    let base = (self.agents[*agent_idx].personality.conscientiousness
                        * Fixed::from_f64(0.05)
                        + skill_bonus)
                        .clamp_01();
                    // §7.5: Sickness depresses output — a plague must hurt the
                    // food economy, not just the population. Healthy agents are
                    // untouched (work_impairment = 1.0 with no diseases), so
                    // calibrated baseline runs don't drift; only outbreaks bite.
                    let sickness_factor = health::work_impairment(&self.agent_diseases[*agent_idx]);
                    // §7.2.3: Frailty/injury depresses output too — mobility is
                    // exactly 1.0 for healthy adults, so calibrated runs carry
                    // zero drift; elders and the severely injured produce less.
                    let mobility_factor = self.agents[*agent_idx]
                        .embodied
                        .skeletal
                        .effective_mobility();
                    // §5 (Iteration 147): weather scales farm output — the
                    // growth factor is ≈ identity in normal weather (mild
                    // calibrated drift) and ≈0.6 during an emergent drought
                    // (genuine famine pressure).
                    // §8.1.4 (P3-6): an active famine window additionally
                    // suppresses grain output to `1 − magnitude` of the shock
                    // (a crop failure — Work keeps laboring but the fields
                    // yield little). The factor is exactly 1.0 outside a
                    // famine, so calibrated windows (riverford/calm/pestilence
                    // and the golden) are untouched; only the Famine and
                    // Collapse scenarios ever open a window.
                    let famine_factor = if tick_u64 < self.famine_until {
                        self.famine_production_factor
                    } else {
                        Fixed::ONE
                    };
                    let productivity = base
                        * sickness_factor
                        * mobility_factor
                        * self.weather.growth_factor()
                        * famine_factor;
                    if let Some(farm_idx) = self.world.best_farm_for_work() {
                        self.world
                            .produce_resource(farm_idx, GRAIN_RESOURCE_ID, productivity);
                        // §13.3: Workers earn coin proportional to productivity.
                        // Wage must cover the cost of eating (0.1 × price per
                        // Eat); the old ×0.3 factor left agents with a net coin
                        // drain (median wealth collapsed to ~0.1, killing trade).
                        let wage = productivity * self.market.price(GRAIN_RESOURCE_ID);
                        self.agents[*agent_idx].wealth.coin += wage;

                        self.journal.record(
                            tick_u64,
                            agent_id,
                            JournalEntryKind::Worked {
                                productivity: productivity.to_f64(),
                            },
                        );
                        true
                    } else {
                        false
                    }
                }
                ActionKind::Rest => {
                    self.journal
                        .record(tick_u64, agent_id, JournalEntryKind::Rested);
                    true
                }
                ActionKind::Worship => {
                    self.journal
                        .record(tick_u64, agent_id, JournalEntryKind::Worshiped);
                    true
                }
                ActionKind::Trade => {
                    // §13.3: Agent-to-agent trade — find a counter-party to trade with.
                    // Trade happens at the settlement Market: any other agent within
                    // a generous radius of the market square counts as a partner.
                    // The old `distance <= 3` filter made trade effectively
                    // impossible (agents rarely stood that close), so volume was 0.
                    let buyer_coin = self.agents[*agent_idx].wealth.coin;
                    let seller_info = self
                        .agents
                        .iter()
                        .enumerate()
                        .find(|(j, a)| {
                            *j != *agent_idx
                                && self.agents[*agent_idx]
                                    .position
                                    .manhattan_distance(&a.position)
                                    <= 12
                        })
                        .map(|(j, _)| j);
                    // Grain source: an accessible farm with a full portion
                    // (prefer the buyer's own accessible farm; fall back to any
                    // farm). §8.1.4 (P3-6): amount-gated like the Eat path — a
                    // trade must move a full portion; crumbs are not marketable
                    // grain and must not feed the buyer through Trade's hunger
                    // relief.
                    let quantity = Fixed::from_f64(0.1);
                    let farm_idx = self
                        .world
                        .accessible_farm_with_grain_amount(agent_id, &self.institutions, quantity)
                        .or_else(|| self.world.farm_with_grain_amount(quantity));
                    if let (Some(seller), Some(farm_idx)) = (seller_info, farm_idx) {
                        let trust = self
                            .relationships
                            .iter()
                            .find(|r| {
                                r.from == AgentId::new(*agent_idx as u64)
                                    && r.to == AgentId::new(seller as u64)
                            })
                            .map_or(Fixed::from_f64(0.5), |r| r.trust);
                        // §13.3: Execute trade — buyer pays coin, seller's farm provides grain
                        let base_price = self.market.price(GRAIN_RESOURCE_ID);
                        // Trust discount: high trust → lower price
                        let trust_modifier = Fixed::from_f64(1.0) - trust * Fixed::from_f64(0.2)
                            + (Fixed::ONE - trust) * Fixed::from_f64(0.1);
                        // §29 (Iteration 187 — the price-trend consumer): the
                        // market's 5-sample price trend was write-only (audit
                        // FINDING 4). A falling price lets the buyer haggle
                        // down; a rising price has them pay up (scarcity
                        // urgency). Mean-zero: trend 0 → modifier exactly
                        // 1.0, byte-identical to the pre-Iter-187 price.
                        let trend = self.market.price_trend(GRAIN_RESOURCE_ID);
                        let trend_modifier = (Fixed::ONE + trend * TREND_BUY_RATE)
                            .clamp(Fixed::from_f64(0.9), Fixed::from_f64(1.1));
                        // §19.5.E (Iteration 187 — the logistics consumer):
                        // the module's `local_scarcity_modifier` was fully
                        // orphaned (audit FINDING 6). The site's own grain
                        // stock now prices its grain — a scarce farm
                        // commands a premium, an overflowing one discounts.
                        // Damped from the raw [0.5, 3.0] band to [0.95, 1.2].
                        let local_modifier = crate::logistics::local_scarcity_modifier(
                            &self.world.sites[farm_idx].inventory,
                            GRAIN_RESOURCE_ID,
                            Fixed::from_f64(1.0),
                        );
                        let local_price_mod =
                            (local_modifier - Fixed::ONE) * LOGISTICS_PRICE_DAMP + Fixed::ONE;
                        let price =
                            (base_price * trust_modifier * trend_modifier * local_price_mod)
                                .max(Fixed::from_f64(1.0));
                        let cost = price * quantity;
                        if buyer_coin >= cost {
                            let taken =
                                self.world
                                    .consume_resource(farm_idx, GRAIN_RESOURCE_ID, quantity);
                            if taken > Fixed::ZERO {
                                let actual_cost = price * taken;
                                // Buyer pays coin to seller (farm owner)
                                self.agents[*agent_idx].wealth.coin =
                                    (self.agents[*agent_idx].wealth.coin - actual_cost)
                                        .max(Fixed::ZERO);
                                self.agents[seller].wealth.coin += actual_cost;
                                self.events.push(SimEvent::TradeOccurred {
                                    buyer: AgentId::new(*agent_idx as u64),
                                    seller: AgentId::new(seller as u64),
                                    good: ResourceId::new(GRAIN_RESOURCE_ID),
                                    quantity: taken,
                                    price,
                                    tick,
                                });
                                // §13.3: Track volume so the market dashboard and
                                // inequality metrics reflect real trade activity.
                                self.market.volume_this_tick =
                                    (self.market.volume_this_tick + taken).max(Fixed::ZERO);
                                self.market.trade_count = self.market.trade_count.saturating_add(1);
                                self.market.total_trades =
                                    self.market.total_trades.saturating_add(1);
                                self.journal.record(
                                    tick_u64,
                                    agent_id,
                                    JournalEntryKind::Consumed {
                                        resource: "grain_via_trade".into(),
                                        amount: taken.to_f64(),
                                    },
                                );
                                // §19.5.J: Record relationship trace — trade builds trust
                                if let Some(rel) = self.relationships.iter_mut().find(|r| {
                                    r.from == AgentId::new(*agent_idx as u64)
                                        && r.to == AgentId::new(seller as u64)
                                }) {
                                    let old_trust = rel.trust;
                                    rel.trust = (rel.trust + Fixed::from_f64(0.02)).clamp_01();
                                    self.provenance.record_relationship(
                                        crate::provenance::RelationshipTrace {
                                            from: AgentId::new(*agent_idx as u64),
                                            to: AgentId::new(seller as u64),
                                            tick: tick_u64,
                                            cause: "trade".into(),
                                            old_trust,
                                            new_trust: rel.trust,
                                            old_affection: rel.affection,
                                            new_affection: rel.affection,
                                            description: format!(
                                                "Trade built trust ({} -> {})",
                                                old_trust.to_f64(),
                                                rel.trust.to_f64()
                                            ),
                                        },
                                    );
                                }
                                true
                            } else {
                                false
                            }
                        } else {
                            false // insufficient funds
                        }
                    } else {
                        false // no nearby seller with grain
                    }
                }
                _ => false,
            };

            // §8.1.4 (P3-6): a FAILED Eat must not relieve hunger. The
            // per-tick relief in `apply_action_tick` is unconditional, so
            // without this revert an agent whose Eat found no grain anywhere
            // still "ate" — hunger never accumulated in famine/pestilence
            // (probe: needs.hunger 0.006–0.041 in EVERY scenario, famine ≈
            // calm; the goal-incongruence branch was unreachable). Revert
            // the single tick of relief already applied and cancel the
            // action so its remaining duration ticks don't relieve either:
            // no food consumed, no satiation.
            if *action == ActionKind::Eat && !action_succeeded {
                let fx = ActionKind::Eat.per_tick_effects();
                let agent = &mut self.agents[*agent_idx];
                agent.needs.hunger = (agent.needs.hunger + fx.hunger_relief).clamp_01();
                agent.action_progress = 0;
                agent.current_action = ActionKind::Idle;
            }

            // Record intention success/failure based on actual resource outcome
            if let Some(ref mut intention) = self.agents[*agent_idx].intention {
                if action_succeeded {
                    intention.record_success();
                } else {
                    intention.record_failure();
                }
            }
            // §22: Track success rate for derived mental state computation
            self.agents[*agent_idx].recent_attempts += 1;
            if action_succeeded {
                self.agents[*agent_idx].recent_successes += 1;
            }
            // Architecture-plan-2 §8.1.20: Decision policy learns from action outcome.
            // EMA learning adjusts utility weights toward successful action patterns.
            let action_cost = match action {
                ActionKind::Work => Fixed::from_f64(0.1),
                ActionKind::Eat | ActionKind::Drink => Fixed::from_f64(0.05),
                ActionKind::Move { .. } | ActionKind::Wander => Fixed::from_f64(0.08),
                ActionKind::Socialize => Fixed::from_f64(0.03),
                _ => Fixed::from_f64(0.02),
            };
            self.agents[*agent_idx]
                .decision_policy
                .learn_from_outcome(action_succeeded, action_cost);

            // §9.2: Neural-like RL values — learn the plan's four value
            // components from the outcome. The outcome profile is the
            // shared `ActionKind::outcome_profile` (single source of
            // truth with the Iteration-94 selection consumer), so what an
            // agent learns is exactly what biases its future choices.
            let profile = action.outcome_profile();
            self.agents[*agent_idx]
                .neural_like
                .values
                .learn_from_outcome(
                    action_succeeded,
                    profile[0],
                    profile[1],
                    profile[2],
                    profile[3],
                );
        }
    }
}
