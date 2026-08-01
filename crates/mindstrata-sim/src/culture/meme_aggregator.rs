//! §17.4 Meme Aggregation — group-level meme metrics for large populations.
//!
//! For large populations, tracking every individual meme exposure is too expensive.
//! Instead, we aggregate at the group level:
//!
//! ```text
//! MemeAggregator tracks:
//!   - meme prevalence per group (faction, household, institution)
//!   - sampled individual exposure (random subset each tick)
//!   - network centrality for spreaders (hub agents spread more)
//!   - aggregated echo chamber metrics per group
//!
//! Updated periodically (daily or deca tick) to avoid per-tick overhead.
//! ```

use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// §17.4: Group-level meme prevalence snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemePrevalence {
    /// Group identifier (faction index, household index, etc.).
    pub group_id: usize,
    /// Group type label for debugging.
    pub group_type: String,
    /// Number of agents in this group.
    pub agent_count: u32,
    /// Per-meme prevalence in this group (meme_id → prevalence fraction 0–1).
    pub meme_prevalence: Vec<(usize, Fixed)>,
    /// Average emotional charge of memes in this group.
    pub avg_emotional_charge: Fixed,
    /// How homogeneous the group's meme landscape is (0 = diverse, 1 = uniform).
    pub meme_homogeneity: Fixed,
}

/// §17.4: Agent exposure sample for a single tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureSample {
    /// Agent that was exposed.
    pub agent: AgentId,
    /// Meme id that was transmitted.
    pub meme_id: usize,
    /// Transmission success (true = agent accepted the meme).
    pub accepted: bool,
    /// Network centrality of the source agent (0 = peripheral, 1 = hub).
    pub source_centrality: Fixed,
}

/// §17.4: Aggregated meme metrics for the entire population.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMemeMetrics {
    /// Per-group meme prevalence snapshots.
    pub group_prevalences: Vec<GroupMemePrevalence>,
    /// Total exposure samples this tick (capped for performance).
    pub exposure_samples: Vec<ExposureSample>,
    /// Most viral meme id (highest host_count * virality).
    pub most_viral_meme: Option<usize>,
    /// Total active meme count across all groups.
    pub total_active_memes: u32,
    /// Average meme spreader centrality (how central are the agents spreading memes).
    pub avg_spreader_centrality: Fixed,
    /// Aggregate echo chamber strength from meme homogeneity.
    pub aggregate_echo_strength: Fixed,
    /// Tick when this aggregation was computed.
    pub computed_tick: u64,
}

impl Default for AggregatedMemeMetrics {
    fn default() -> Self {
        Self {
            group_prevalences: Vec::new(),
            exposure_samples: Vec::new(),
            most_viral_meme: None,
            total_active_memes: 0,
            avg_spreader_centrality: Fixed::ZERO,
            aggregate_echo_strength: Fixed::ZERO,
            computed_tick: 0,
        }
    }
}

/// §17.4: Meme Aggregation engine — computes group-level meme metrics.
///
/// Designed to be called periodically (not every tick) to avoid overhead.
/// For large populations (100+ agents), this replaces per-agent meme tracking
/// with group-level aggregation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemeAggregator {
    /// Maximum exposure samples per tick (performance cap).
    pub max_exposure_samples: usize,
    /// Tick interval between aggregation passes.
    pub aggregation_interval: u64,
    /// Last tick when aggregation was computed.
    last_aggregation_tick: u64,
    /// Cached aggregated metrics.
    cached_metrics: AggregatedMemeMetrics,
}

impl Default for MemeAggregator {
    fn default() -> Self {
        Self {
            max_exposure_samples: 20,
            aggregation_interval: 10, // deca tick
            last_aggregation_tick: 0,
            cached_metrics: AggregatedMemeMetrics::default(),
        }
    }
}

impl MemeAggregator {
    /// Create a new aggregator with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an aggregator tuned for large populations.
    pub fn for_large_population() -> Self {
        Self {
            max_exposure_samples: 50,
            aggregation_interval: 10,
            last_aggregation_tick: 0,
            cached_metrics: AggregatedMemeMetrics::default(),
        }
    }

    /// Check if aggregation should run this tick.
    pub fn should_aggregate(&self, tick: u64) -> bool {
        tick > 0
            && tick >= self.last_aggregation_tick + self.aggregation_interval
            && tick % self.aggregation_interval == 0
    }

    /// Get the cached metrics from the last aggregation.
    pub fn metrics(&self) -> &AggregatedMemeMetrics {
        &self.cached_metrics
    }

    /// §17.4: Compute per-group meme prevalence.
    ///
    /// For each group, count how many agents host each meme and compute
    /// the prevalence fraction (hosted / group_size).
    pub fn compute_group_prevalence(
        &self,
        agent_meme_hosts: &[Vec<usize>], // agent_idx → list of meme ids hosted
        agent_groups: &[Option<usize>],  // agent_idx → group_id
        meme_ids: &[usize],              // all active meme ids
        group_labels: &[(usize, String)], // (group_id, type_label)
        meme_emotional_charges: &[Fixed], // meme_id → emotional_charge (parallel to meme_ids)
    ) -> Vec<GroupMemePrevalence> {
        let mut result = Vec::new();

        for &(group_id, ref group_type) in group_labels {
            // Collect agents in this group
            let group_agents: Vec<usize> = agent_groups
                .iter()
                .enumerate()
                .filter(|(_, gid)| **gid == Some(group_id))
                .map(|(idx, _)| idx)
                .collect();

            let agent_count = group_agents.len() as u32;
            if agent_count == 0 {
                continue;
            }

            // Count meme prevalence
            let mut meme_counts: Vec<(usize, u32)> = Vec::new();
            for &meme_id in meme_ids {
                let count = group_agents
                    .iter()
                    .filter(|&&agent_idx| {
                        agent_meme_hosts
                            .get(agent_idx)
                            .map_or(false, |memes| memes.contains(&meme_id))
                    })
                    .count() as u32;
                if count > 0 {
                    let prevalence =
                        Fixed::from_int(count as i64) / Fixed::from_int(agent_count as i64);
                    meme_counts.push((meme_id, count));
                    let _ = prevalence; // used below
                }
            }

            // Compute prevalence fractions
            let prevalence_vec: Vec<(usize, Fixed)> = meme_counts
                .iter()
                .map(|&(mid, count)| {
                    (
                        mid,
                        Fixed::from_int(count as i64) / Fixed::from_int(agent_count as i64),
                    )
                })
                .collect();

            // §17.4: Average emotional charge from meme data (not prevalence proxy)
            let avg_charge = if prevalence_vec.is_empty() {
                Fixed::ZERO
            } else {
                let total_charge: Fixed = prevalence_vec
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, (mid, _))| {
                        meme_ids.iter().position(|id| id == mid)
                            .and_then(|pos| meme_emotional_charges.get(pos))
                            .map(|c| *c)
                    })
                    .fold(Fixed::ZERO, |acc, c| acc + c);
                let charge_count = prevalence_vec.len().max(1) as i64;
                (total_charge / Fixed::from_int(charge_count)).clamp_01()
            };
            let total_prevalence: Fixed = prevalence_vec
                .iter()
                .map(|(_, p)| *p)
                .fold(Fixed::ZERO, |acc, p| acc + p);

            // Meme homogeneity: how uniform is the prevalence distribution?
            // High homogeneity = most agents share the same memes
            let avg_prevalence = if prevalence_vec.is_empty() {
                Fixed::ZERO
            } else {
                total_prevalence / Fixed::from_int(prevalence_vec.len() as i64)
            };
            let variance: Fixed = prevalence_vec
                .iter()
                .map(|(_, p)| {
                    let diff = *p - avg_prevalence;
                    diff * diff
                })
                .fold(Fixed::ZERO, |acc, v| acc + v);
            let homogeneity = if prevalence_vec.is_empty() {
                Fixed::ZERO
            } else {
                (Fixed::ONE
                    - (variance / Fixed::from_int(prevalence_vec.len() as i64)).min(Fixed::ONE))
                    .max(Fixed::ZERO)
            };

            result.push(GroupMemePrevalence {
                group_id,
                group_type: group_type.clone(),
                agent_count,
                meme_prevalence: prevalence_vec,
                avg_emotional_charge: avg_charge,
                meme_homogeneity: homogeneity,
            });
        }

        result
    }

    /// §17.4: Sample individual meme exposures for metrics.
    ///
    /// Randomly samples a subset of agents and records their meme transmission
    /// outcomes. The `source_centrality` parameter should come from a
    /// precomputed network centrality measure.
    pub fn sample_exposures(
        &self,
        agent_idx: usize,
        meme_ids_hosted: &[usize],
        source_centrality: Fixed,
        accepted: bool,
        samples: &mut Vec<ExposureSample>,
    ) {
        if samples.len() >= self.max_exposure_samples {
            return;
        }
        for &meme_id in meme_ids_hosted.iter().take(2) {
            // cap at 2 per agent to spread sampling
            samples.push(ExposureSample {
                agent: AgentId::new(agent_idx as u64),
                meme_id,
                accepted,
                source_centrality,
            });
            if samples.len() >= self.max_exposure_samples {
                break;
            }
        }
    }

    /// §17.4: Run full aggregation pass.
    ///
    /// This is the main entry point called periodically from the tick loop.
    /// It computes group prevalence, samples exposures, and aggregates metrics.
    /// Run full aggregation pass.
    ///
    /// `meme_emotional_charges` is parallel to `meme_ids` — the emotional
    /// charge of each meme (for accurate group-level charge metrics).
    ///
    /// Note: `should_aggregate(tick)` returns false for tick 0, but this
    /// method always runs on tick 0 to produce initial metrics.
    pub fn aggregate(
        &mut self,
        tick: u64,
        agent_meme_hosts: &[Vec<usize>],
        agent_groups: &[Option<usize>],
        meme_ids: &[usize],
        meme_emotional_charges: &[Fixed],
        group_labels: &[(usize, String)],
        agent_centrality: &[Fixed],
    ) -> &AggregatedMemeMetrics {
        // Always run on tick 0 for initialization; otherwise respect interval.
        if tick != 0 && !self.should_aggregate(tick) {
            return &self.cached_metrics;
        }

        // 1. Compute group prevalence with actual emotional charges
        let group_prevalences = self.compute_group_prevalence(
            agent_meme_hosts, agent_groups, meme_ids, group_labels, meme_emotional_charges,
        );

        // 2. Find most viral meme
        let most_viral = meme_ids
            .iter()
            .max_by_key(|&&mid| {
                // Approximate virality from prevalence across groups
                let total_hosts: u32 = group_prevalences
                    .iter()
                    .filter_map(|gp| {
                        gp.meme_prevalence
                            .iter()
                            .find(|(id, _)| *id == mid)
                            .map(|(_, p)| {
                                (*p * Fixed::from_int(gp.agent_count as i64)).to_raw() as u32
                            })
                    })
                    .sum();
                total_hosts as u64
            })
            .copied();

        // 3. Compute average spreader centrality (only meme hosts, not all agents)
        let meme_host_centralities: Vec<Fixed> = agent_meme_hosts
            .iter()
            .enumerate()
            .filter(|(_, hosts)| !hosts.is_empty())
            .filter_map(|(idx, _)| agent_centrality.get(idx).copied())
            .collect();
        let avg_centrality = if meme_host_centralities.is_empty() {
            Fixed::ZERO
        } else {
            let total: Fixed = meme_host_centralities.iter().fold(Fixed::ZERO, |acc, c| acc + *c);
            total / Fixed::from_int(meme_host_centralities.len() as i64)
        };

        // 4. Aggregate echo chamber strength from group homogeneities
        let total_homogeneity: Fixed = group_prevalences
            .iter()
            .map(|gp| gp.meme_homogeneity)
            .fold(Fixed::ZERO, |acc, h| acc + h);
        let echo_strength = if group_prevalences.is_empty() {
            Fixed::ZERO
        } else {
            (total_homogeneity / Fixed::from_int(group_prevalences.len() as i64)).clamp_01()
        };

        self.cached_metrics = AggregatedMemeMetrics {
            group_prevalences,
            exposure_samples: Vec::new(), // filled by tick loop
            most_viral_meme: most_viral,
            total_active_memes: meme_ids.len() as u32,
            avg_spreader_centrality: avg_centrality,
            aggregate_echo_strength: echo_strength,
            computed_tick: tick,
        };
        self.last_aggregation_tick = tick;

        &self.cached_metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregator_should_aggregate_at_interval() {
        let agg = MemeAggregator::new();
        assert!(!agg.should_aggregate(0));
        assert!(agg.should_aggregate(10));
        assert!(!agg.should_aggregate(11));
        assert!(agg.should_aggregate(20));
    }

    #[test]
    fn group_prevalence_computed_correctly() {
        let agg = MemeAggregator::new();
        // 4 agents, group 0 has agents 0,1 and group 1 has agents 2,3
        let agent_meme_hosts = vec![vec![0], vec![0], vec![1], vec![1]];
        let agent_groups = vec![Some(0), Some(0), Some(1), Some(1)];
        let meme_ids = vec![0, 1];
        let meme_charges = vec![Fixed::from_f64(0.7), Fixed::from_f64(0.4)];
        let group_labels = vec![(0, "faction".into()), (1, "faction".into())];

        let result = agg.compute_group_prevalence(
            &agent_meme_hosts,
            &agent_groups,
            &meme_ids,
            &group_labels,
            &meme_charges,
        );

        assert_eq!(result.len(), 2);
        // Group 0: meme 0 prevalence = 2/2 = 1.0, meme 1 = 0/2 = 0
        let g0 = &result[0];
        assert_eq!(g0.agent_count, 2);
        let p0 = g0.meme_prevalence.iter().find(|(id, _)| *id == 0);
        assert!(p0.is_some());
        assert!((p0.unwrap().1 - Fixed::ONE).to_raw().unsigned_abs() < 2);

        // Group 1: meme 1 prevalence = 2/2 = 1.0
        let g1 = &result[1];
        let p1 = g1.meme_prevalence.iter().find(|(id, _)| *id == 1);
        assert!(p1.is_some());
    }

    #[test]
    fn exposure_sampling_respects_limit() {
        let agg = MemeAggregator::new(); // max 20 samples
        let mut samples = Vec::new();
        for i in 0..30 {
            agg.sample_exposures(i, &[0, 1], Fixed::from_f64(0.5), true, &mut samples);
        }
        assert!(samples.len() <= 20);
    }

    #[test]
    fn full_aggregation_populates_metrics() {
        let mut agg = MemeAggregator::new();
        let agent_meme_hosts = vec![vec![0], vec![0], vec![1]];
        let agent_groups = vec![Some(0), Some(0), Some(1)];
        let meme_ids = vec![0, 1];
        let meme_charges = vec![Fixed::from_f64(0.7), Fixed::from_f64(0.4)];
        let group_labels = vec![(0, "faction".into()), (1, "faction".into())];
        let centrality = vec![
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.4),
            Fixed::from_f64(0.6),
        ];

        let metrics = agg.aggregate(
            10,
            &agent_meme_hosts,
            &agent_groups,
            &meme_ids,
            &meme_charges,
            &group_labels,
            &centrality,
        );

        assert_eq!(metrics.total_active_memes, 2);
        assert!(metrics.most_viral_meme.is_some());
        assert!(metrics.avg_spreader_centrality > Fixed::ZERO);
        assert_eq!(metrics.computed_tick, 10);
    }

    #[test]
    fn for_large_population_has_higher_sample_cap() {
        let agg = MemeAggregator::for_large_population();
        assert_eq!(agg.max_exposure_samples, 50);
    }
}
