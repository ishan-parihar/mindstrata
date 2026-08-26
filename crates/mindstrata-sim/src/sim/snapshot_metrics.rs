//! Snapshot capture/save, metrics history and agent summaries.

use super::{
    AttachmentStyle, CaregivingStyle, Deserialize, Fixed, Serialize, Simulation, Snapshot,
    SELF_MODEL_REGULATION_GAIN,
};

/// Maximum number of metric snapshots to retain (recorded every 10 ticks).
///
/// Old value 500 silently dropped the first half of any run longer than
/// 5,000 ticks (ring buffer), so CSV exports started at tick 5010 for a
/// 10,000-tick run. Raised to 20_000 so a full year of sim time
/// (~51,840 ticks ≈ 5,184 rows) is preserved in exports.
pub(super) const MAX_METRIC_HISTORY: usize = 20_000;

/// A snapshot of key simulation metrics at a specific tick.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MetricsSnapshot {
    pub tick: u64,
    // ── Basic need state ──
    pub avg_hunger: f64,
    pub avg_thirst: f64,
    pub avg_fatigue: f64,
    // ── Emotion ──
    pub avg_valence: f64,
    pub avg_joy: f64,
    pub avg_fear: f64,
    // ── Resources ──
    pub total_grain: f64,
    pub total_water: f64,
    // ── System stats ──
    pub event_count: u64,
    pub journal_len: u64,
    pub agent_count: u64,
    // ── §17 Observability: deep metrics for Phase 5 tuning ──
    /// Average stress (fear + anger) across all agents.
    pub avg_stress: f64,
    /// Average health from EmbodiedState.
    pub avg_health: f64,
    /// Average accumulated trauma load across all agents (§7 nervous).
    #[serde(default)]
    pub avg_trauma_load: f64,
    /// Average relationship trust across all relationship_v2 edges.
    pub avg_relationship_trust: f64,
    /// Average relationship quality across all relationship_v2 edges.
    pub avg_relationship_quality: f64,
    /// Number of active memes in the meme registry.
    pub active_meme_count: u64,
    /// Echo chamber polarization index.
    pub polarization_index: f64,
    /// Gini coefficient of coin wealth (0 = equality, 1 = inequality).
    #[serde(default)]
    pub gini: f64,
    /// Mean coin wealth across agents.
    #[serde(default)]
    pub avg_wealth: f64,
    /// Median coin wealth across agents.
    #[serde(default)]
    pub median_wealth: f64,
    /// Cumulative completed trades (market activity).
    #[serde(default)]
    pub total_trades: u64,
    /// Number of active households.
    pub household_count: u64,
    /// Number of active kinship edges.
    pub kinship_edge_count: u64,
    /// Average agent tier (0=Focal, 1=Secondary, 2=Background).
    pub avg_agent_tier: f64,
    /// Number of active feuds across all agents.
    pub total_active_feuds: u64,
    /// Number of clans (kinship-based social groups, §10.8).
    #[serde(default)]
    pub clan_count: u64,
    /// Number of distinct inter-clan relations (alliances + enmities, §10.8).
    #[serde(default)]
    pub clan_relation_count: u64,
    /// Number of active cults (high-intensity groups, §12.4).
    #[serde(default)]
    pub cult_count: u64,
    /// Number of symbolic nodes in the noospheric field (§13).
    #[serde(default)]
    pub noosphere_nodes: u64,
    /// Peak activation of the noospheric field — the dominant symbolic mood
    /// (the §13 belief-ecology zeitgeist).
    #[serde(default)]
    pub noosphere_zeitgeist: f64,
    /// Total shared memories across all groups (§13.5).
    #[serde(default)]
    pub collective_memory_count: u64,
    /// Active patronage relations (§10.9).
    #[serde(default)]
    pub patronage_relation_count: u64,
    /// Iteration 251 (observability): distinct family surnames among
    /// living agents — lineage proliferation over generational time.
    pub family_count: u64,
    /// Mean of each agent's best skill (max of farming/trading/social) —
    /// the village's human-capital curve.
    pub avg_best_skill: f64,
    /// 90th-percentile fear — the distress tail the means hide.
    pub fear_p90: f64,
    /// 90th-percentile joy — the wellbeing tail.
    pub joy_p90: f64,
    /// Population variance of big-five personality traits (Iteration 262):
    /// the quantitative-genetics stationarity monitor promised by the
    /// deepening plan — collapse would signal heredity over-shrinkage.
    pub trait_variance: f64,
    /// Mean genetic relatedness over active kinship edges (Iteration 262).
    pub mean_kinship: f64,
}

impl MetricsSnapshot {
    /// §5.1/§19: CSV header for exporting metrics for analysis.
    pub fn csv_header() -> &'static str {
        "tick,avg_hunger,avg_thirst,avg_fatigue,avg_valence,avg_joy,avg_fear,total_grain,total_water,event_count,journal_len,agent_count,avg_stress,avg_health,avg_trauma_load,avg_relationship_trust,avg_relationship_quality,active_meme_count,polarization_index,gini,avg_wealth,median_wealth,total_trades,household_count,kinship_edge_count,avg_agent_tier,total_active_feuds,clan_count,clan_relation_count,cult_count,noosphere_nodes,noosphere_zeitgeist,collective_memory_count,patronage_relation_count,family_count,avg_best_skill,fear_p90,joy_p90,trait_variance,mean_kinship"
    }

    /// §5.1/§19: One CSV line for this snapshot.
    pub fn to_csv_line(&self) -> String {
        format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.tick,
            self.avg_hunger, self.avg_thirst, self.avg_fatigue,
            self.avg_valence, self.avg_joy, self.avg_fear,
            self.total_grain, self.total_water,
            self.event_count, self.journal_len, self.agent_count,
            self.avg_stress, self.avg_health, self.avg_trauma_load,
            self.avg_relationship_trust, self.avg_relationship_quality,
            self.active_meme_count, self.polarization_index,
            self.gini, self.avg_wealth, self.median_wealth, self.total_trades,
            self.household_count, self.kinship_edge_count,
            self.avg_agent_tier, self.total_active_feuds,
            self.clan_count, self.clan_relation_count, self.cult_count,
            self.noosphere_nodes,
            self.noosphere_zeitgeist, self.collective_memory_count,
            self.patronage_relation_count,
            self.family_count,
            self.avg_best_skill,
            self.fear_p90,
            self.joy_p90,
            self.trait_variance,
            self.mean_kinship,
        )
    }
}

/// A lightweight summary of an agent's state for display.
#[derive(Debug, Clone)]
pub struct AgentSummary {
    pub index: usize,
    pub name: String,
    pub hunger: Fixed,
    pub thirst: Fixed,
    pub fatigue: Fixed,
    pub health: Fixed,
    pub energy: Fixed,
    pub valence: Fixed,
    pub joy: Fixed,
    pub fear: Fixed,
    pub anger: Fixed,
    pub goal_count: usize,
    pub current_action: String,
    pub attention_budget: Fixed,
    pub has_intention: bool,
    /// Architecture-plan-2 §8.1.14: Attachment style for the live agent view.
    pub attachment_style: AttachmentStyle,
    /// Architecture-plan-2 §8.1.14: Attachment security (0 = insecure, 1 = secure).
    pub attachment_security: Fixed,
    /// Architecture-plan-2 §8.1.14: Attachment anxiety.
    pub attachment_anxiety: Fixed,
    /// Architecture-plan-2 §8.1.14: Attachment avoidance.
    pub attachment_avoidance: Fixed,
    /// Architecture-plan-2 §8.1.14: Protest-behavior threshold.
    pub attachment_protest_threshold: Fixed,
    /// Architecture-plan-2 §8.1.14: Receptivity to soothing from others.
    pub attachment_soothing_receptivity: Fixed,
    /// Architecture-plan-2 §8.1.14: Separation distress.
    pub attachment_separation_distress: Fixed,
    /// Architecture-plan-2 §8.1.14: Caregiving style for the live agent view.
    pub attachment_caregiving_style: CaregivingStyle,
    /// Architecture-plan-2 §7: Accumulated trauma load (§17 observability).
    pub trauma_load: Fixed,
}

/// §8.1: Regulation-capacity support contributed by self-esteem (deviation
/// from the 0.5 baseline × gain). Zero at baseline; positive for healthy
/// self-models, negative for eroded ones. Pure and deterministic.
pub(super) fn self_esteem_support(self_esteem: Fixed) -> Fixed {
    (self_esteem - Fixed::from_f64(0.5)) * SELF_MODEL_REGULATION_GAIN
}

impl Simulation {
    /// §16.1: Capture a snapshot of the current simulation state.
    pub fn capture_snapshot(&self) -> crate::snapshot::Snapshot {
        crate::snapshot::Snapshot::capture(&crate::snapshot::CaptureContext {
            config: &self.config,
            tick: self.clock.tick(),
            master_seed: self.config.seed,
            world: &self.world,
            agents: &self.agents,
            relationships: &self.relationships,
            events: &self.events,
            journal: &self.journal,
            norms: &self.norms,
            institutions: &self.institutions,
            provenance: &self.provenance,
            season: &self.season,
            ecology_config: &self.ecology_config,
            health_config: &self.health_config,
            demography_config: &self.demography_config,
            agent_diseases: &self.agent_diseases,
            market: &self.market,
            knowledge_store: &self.knowledge_store,
            metric_history: &self.metric_history,
            last_revolution_tick: self.last_revolution_tick,
            last_moral_panic_tick: self.last_moral_panic_tick,
            last_cult_formation_tick: self.last_cult_formation_tick,
            black_market: &self.black_market,
            site_work_ticks: &self.site_work_ticks,
            group_registry: &self.group_registry,
            collective_memory_registry: &self.collective_memory_registry,
            meme_registry: &self.meme_registry,
            kinship_graph: &self.kinship_graph,
        })
    }

    /// Get a snapshot of agent summaries.
    pub fn agent_summaries(&self) -> Vec<AgentSummary> {
        self.agents
            .iter()
            .enumerate()
            .map(|(i, a)| AgentSummary {
                index: i,
                name: a.name.clone(),
                hunger: a.needs.hunger,
                thirst: a.needs.thirst,
                fatigue: a.needs.fatigue,
                health: a.body.health,
                energy: a.body.energy,
                valence: a.affect.valence,
                joy: a.emotions.joy,
                fear: a.emotions.fear,
                anger: a.emotions.anger,
                goal_count: a.goals.len(),
                current_action: format!("{:?}", a.current_action),
                attention_budget: a.attention.budget,
                has_intention: a.intention.is_some(),
                attachment_style: a.attachment.style,
                attachment_security: a.attachment.security,
                attachment_anxiety: a.attachment.anxiety,
                attachment_avoidance: a.attachment.avoidance,
                attachment_protest_threshold: a.attachment.protest_threshold,
                attachment_soothing_receptivity: a.attachment.soothing_receptivity,
                attachment_separation_distress: a.attachment.separation_distress,
                attachment_caregiving_style: a.attachment.caregiving_style,
                trauma_load: a.embodied.nervous.trauma_load,
            })
            .collect()
    }

    /// §16.1: Capture a full deterministic snapshot of the simulation state.
    /// RNG streams are not serialized — only the master seed is stored.
    /// §16.1: Convenience alias — delegates to capture_snapshot.
    pub fn save_snapshot(&self) -> Snapshot {
        self.capture_snapshot()
    }

    /// Capture a snapshot of key metrics at the current tick.
    pub fn metrics_snapshot(&self) -> MetricsSnapshot {
        let summaries = self.agent_summaries();
        let n = summaries.len() as f64;
        let n_inv = if n > 0.0 { 1.0 / n } else { 0.0 };

        // §17: Deep observability metrics for Phase 5 tuning
        let avg_stress: f64 = if self.agents.is_empty() {
            0.0
        } else {
            self.agents
                .iter()
                .map(|a| (a.emotions.fear + a.emotions.anger).to_f64())
                .sum::<f64>()
                * n_inv
        };
        let avg_health: f64 = if self.agents.is_empty() {
            0.0
        } else {
            self.agents
                .iter()
                .map(|a| a.embodied.derived_health().to_f64())
                .sum::<f64>()
                * n_inv
        };
        let (trust_sum, quality_sum, rel_count) = self
            .agents
            .iter()
            .flat_map(|a| a.relationship_v2s.iter())
            .fold((0.0f64, 0.0f64, 0u64), |(tq, qq, c), rv2| {
                (tq + rv2.trust.to_f64(), qq + rv2.quality().to_f64(), c + 1)
            });
        let rel_n = if rel_count > 0 { rel_count as f64 } else { 1.0 };
        let avg_relationship_trust = trust_sum / rel_n;
        let avg_relationship_quality = quality_sum / rel_n;
        let active_meme_count = self.meme_registry.active_count() as u64;
        let polarization_index = self.echo_chamber.polarization_index.to_f64();
        // §13.3: Inequality metrics — computed by the market system earlier in
        // this tick (tick_social_cluster → system_market), so values are fresh.
        let gini = self.market.inequality.to_f64();
        let avg_wealth = self.market.avg_wealth.to_f64();
        let median_wealth = self.market.median_wealth.to_f64();
        let total_trades = self.market.total_trades;
        let household_count = self.households.len() as u64;
        let kinship_edge_count = self.kinship_graph.active_count() as u64;
        let avg_agent_tier: f64 = if self.agents.is_empty() {
            0.0
        } else {
            self.agents
                .iter()
                .map(|a| a.agent_tier.tier.tier_index())
                .sum::<f64>()
                * n_inv
        };
        let total_active_feuds: u64 = self.agents.iter().map(|a| a.feuds.len() as u64).sum();
        // §10.8/§12.4: Collective-structure observability — clan count and
        // active cult count.
        let clan_count = self.clan_registry.clans.len() as u64;
        // §10.8: Distinct inter-clan relations — each alliance/enmity edge is
        // recorded symmetrically on both clans, so divide by 2.
        let clan_relation_count = {
            let edges: usize = self
                .clan_registry
                .clans
                .iter()
                .map(|c| c.allies.len() + c.enemies.len())
                .sum();
            (edges / 2) as u64
        };
        let cult_count = self.cult_registry.cults.iter().filter(|c| c.active).count() as u64;
        let noosphere_nodes = self.noospheric_field.nodes.len() as u64;
        let noosphere_zeitgeist = self
            .noospheric_field
            .nodes
            .iter()
            .map(|n| n.effective_activation().to_f64())
            .fold(0.0f64, f64::max);
        let collective_memory_count = self
            .collective_memory_registry
            .entries
            .iter()
            .map(|e| e.memories.len() as u64)
            .sum();
        let patronage_relation_count = self
            .patronage_registry
            .relations
            .iter()
            .filter(|r| r.active)
            .count() as u64;

        // Iteration 251 (observability): lineage + tail metrics.
        // family_count: distinct surnames ("Given Surname", Iter-245);
        // single-token founder names don't count as families.
        let family_count = self
            .agents
            .iter()
            .filter_map(|a| a.name.split_once(' ').map(|(_, sur)| sur))
            .collect::<std::collections::HashSet<_>>()
            .len() as u64;
        let avg_best_skill = if self.agents.is_empty() {
            0.0
        } else {
            self.agents
                .iter()
                .map(|a| {
                    a.skills
                        .farming
                        .to_f64()
                        .max(a.skills.trading.to_f64())
                        .max(a.skills.social.to_f64())
                })
                .sum::<f64>()
                / self.agents.len() as f64
        };
        // Nearest-rank percentile over sorted per-agent emotion values.
        let percentile = |mut vals: Vec<f64>, q: f64| -> f64 {
            if vals.is_empty() {
                return 0.0;
            }
            vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let idx = (((vals.len() as f64) * q).ceil() as usize)
                .saturating_sub(1)
                .min(vals.len() - 1);
            vals[idx]
        };
        let fear_p90 = percentile(summaries.iter().map(|x| x.fear.to_f64()).collect(), 0.9);
        let joy_p90 = percentile(summaries.iter().map(|x| x.joy.to_f64()).collect(), 0.9);

        // Iteration 262 (observability completion): the two Iter-251
        // promises that never shipped.
        // trait_variance: mean per-trait population variance across the
        // big five — the stationarity monitor for quantitative-genetics
        // heredity (collapse would mean over-shrinkage toward the mean).
        let trait_variance = if self.agents.is_empty() {
            0.0
        } else {
            let n = self.agents.len() as f64;
            let accessors = [
                |a: &crate::sim::AgentBundle| a.personality.openness.to_f64(),
                |a: &crate::sim::AgentBundle| a.personality.conscientiousness.to_f64(),
                |a: &crate::sim::AgentBundle| a.personality.extraversion.to_f64(),
                |a: &crate::sim::AgentBundle| a.personality.agreeableness.to_f64(),
                |a: &crate::sim::AgentBundle| a.personality.neuroticism.to_f64(),
            ];
            let mut total = 0.0;
            for accessor in accessors {
                let vals: Vec<f64> = self.agents.iter().map(accessor).collect();
                let m = vals.iter().sum::<f64>() / n;
                total += vals.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / n;
            }
            total / 5.0
        };
        // mean_kinship: average genetic relatedness across active edges
        // (0.5 parent-child/sibling by construction; spouses/in-laws 0).
        let mean_kinship = {
            let act: Vec<_> = self
                .kinship_graph
                .edges
                .iter()
                .filter(|e| e.active)
                .collect();
            if act.is_empty() {
                0.0
            } else {
                act.iter().map(|e| e.coefficient.to_f64()).sum::<f64>() / act.len() as f64
            }
        };

        MetricsSnapshot {
            tick: self.current_tick().as_u64(),
            avg_hunger: summaries.iter().map(|s| s.hunger.to_f64()).sum::<f64>() * n_inv,
            avg_thirst: summaries.iter().map(|s| s.thirst.to_f64()).sum::<f64>() * n_inv,
            avg_fatigue: summaries.iter().map(|s| s.fatigue.to_f64()).sum::<f64>() * n_inv,
            avg_valence: summaries.iter().map(|s| s.valence.to_f64()).sum::<f64>() * n_inv,
            avg_joy: summaries.iter().map(|s| s.joy.to_f64()).sum::<f64>() * n_inv,
            avg_fear: summaries.iter().map(|s| s.fear.to_f64()).sum::<f64>() * n_inv,
            total_grain: self.total_grain().to_f64(),
            total_water: self.total_water().to_f64(),
            event_count: self.event_count() as u64,
            journal_len: self.journal_len() as u64,
            agent_count: self.agent_count() as u64,
            avg_stress,
            avg_health,
            avg_trauma_load: summaries
                .iter()
                .map(|s| s.trauma_load.to_f64())
                .sum::<f64>()
                * n_inv,
            avg_relationship_trust,
            avg_relationship_quality,
            active_meme_count,
            polarization_index,
            gini,
            avg_wealth,
            median_wealth,
            total_trades,
            household_count,
            kinship_edge_count,
            avg_agent_tier,
            total_active_feuds,
            clan_count,
            clan_relation_count,
            cult_count,
            noosphere_nodes,
            noosphere_zeitgeist,
            collective_memory_count,
            patronage_relation_count,
            family_count,
            avg_best_skill,
            fear_p90,
            joy_p90,
            trait_variance,
            mean_kinship,
        }
    }
}
