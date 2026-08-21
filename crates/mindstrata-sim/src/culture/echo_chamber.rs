//! Echo chambers and polarization — network-level belief structure (§13.6).
//!
//! Echo chambers emerge when agents sort into homophilous networks,
//! institutions compete, fear rises, gossip punishes dissent, identity
//! fusion increases, and trusted bridges collapse.
//!
//! ```text
//! Echo chamber dynamics:
//!   - Agents interact mostly with similar others
//!   - Dissenting views are suppressed or punished
//!   - Narrative dominance increases within clusters
//!   - Cross-cutting ties weaken over time
//!   - Polarization index rises
//!
//! Polarization emerges from:
//!   - institutional competition
//!   - fear amplification
//!   - gossip punishing dissent
//!   - identity fusion
//!   - trusted bridge collapse
//!   - social sorting
//! ```

use std::collections::BTreeMap;

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// §13.6 (AP2): Narrative-dominance → cluster-assignment fold (Iteration 120).
///
/// [`EchoChamberState::narrative_momentum`] scales the daily pro-institution
/// cluster-assignment heuristic by the strongest narrative in the meme pool
/// — the plan's "narrative dominance increases within clusters" loop, made
/// decisional at assignment time.
/// - `NARRATIVE_DOMINANCE_THRESHOLD`: a narrative must clear this to be
///   "dominant". Anchored at 0.6, ABOVE the probe-pinned calibrated ceiling
///   of 0.52 (the initial memes sit at credibility 0.5 × virality 1.04 =
///   0.52, and the value only decays from there across every seed × horizon
///   swept — 9 combos up to 20,000 ticks), so the channel is identity in
///   every current calibrated world (zero-blast by construction).
/// - `NARRATIVE_MOMENTUM_RATE`: the bonus slope — at full dominance (1.0)
///   the momentum reaches `(1.0 − 0.6) × 0.25 = 0.1`, enough to tip marginal
///   agents (pro_score within 0.1 below the 1.0 boundary) into the
///   pro-institution cluster.
///
/// The momentum is deliberately ONE-SIDED (a village-wide bandwagon toward
/// the institutional pole): it aggregates the single strongest narrative and
/// adds to every agent's pro_score, rather than per-agent alignment with a
/// specific narrative's content — no belief↔meme bridge exists in the
/// codebase, and competing high-dominance narratives (e.g. a pro-temple and
/// an anti-temple meme both at 0.8) still push pro by the max. Acceptable
/// for an assignment-level nudge; a competition-aware (top-2 gap) variant
/// would be a future refinement.
pub const NARRATIVE_DOMINANCE_THRESHOLD: f64 = 0.6;
pub const NARRATIVE_MOMENTUM_RATE: f64 = 0.25;

/// A cluster of agents who share similar beliefs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefCluster {
    /// Unique cluster identifier.
    pub id: usize,
    /// Dominant narrative or belief in this cluster.
    pub dominant_narrative: String,
    /// Agent IDs in this cluster.
    pub members: Vec<usize>,
    /// Internal cohesion — how strongly members agree (0 = fragmented, 1 = unanimous).
    pub cohesion: Fixed,
    /// Average emotional charge of the cluster's beliefs.
    pub emotional_charge: Fixed,
    /// How hostile the cluster is toward outgroup views (0 = tolerant, 1 = hostile).
    pub outgroup_hostility: Fixed,
    /// Degree of identity fusion (how much membership defines personal identity).
    pub identity_fusion: Fixed,
    /// Number of cross-cutting ties to other clusters.
    pub cross_cutting_ties: u32,
}

impl BeliefCluster {
    /// Create a new belief cluster.
    pub fn new(id: usize, dominant_narrative: String) -> Self {
        Self {
            id,
            dominant_narrative,
            members: Vec::new(),
            cohesion: Fixed::from_f64(0.5),
            emotional_charge: Fixed::ZERO,
            outgroup_hostility: Fixed::ZERO,
            identity_fusion: Fixed::ZERO,
            cross_cutting_ties: 0,
        }
    }

    /// Add a member to this cluster.
    pub fn add_member(&mut self, agent_id: usize) {
        if !self.members.contains(&agent_id) {
            self.members.push(agent_id);
        }
    }

    /// Remove a member from this cluster.
    pub fn remove_member(&mut self, agent_id: usize) {
        self.members.retain(|&id| id != agent_id);
    }

    /// Compute the cluster's echo chamber strength.
    ///
    /// ```text
    /// echo_strength = cohesion × identity_fusion × outgroup_hostility
    ///               × (1 / (1 + cross_cutting_ties))
    /// ```
    pub fn echo_strength(&self) -> Fixed {
        let tie_penalty =
            Fixed::ONE / (Fixed::ONE + Fixed::from_int(self.cross_cutting_ties as i64));
        (self.cohesion * self.identity_fusion * self.outgroup_hostility * tie_penalty).clamp_01()
    }

    /// Tick update — cohesion changes slowly, ties may weaken.
    pub fn tick_update(&mut self) {
        // Outgroup hostility can grow if emotional charge is high
        if self.emotional_charge > Fixed::from_f64(0.6) {
            self.outgroup_hostility = (self.outgroup_hostility + Fixed::from_f64(0.002)).clamp_01();
        }
        // Identity fusion grows with emotional charge
        self.identity_fusion =
            (self.identity_fusion + self.emotional_charge * Fixed::from_f64(0.001)).clamp_01();
    }
}

/// System-level echo chamber and polarization state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoChamberState {
    /// All belief clusters.
    pub clusters: Vec<BeliefCluster>,
    /// Overall polarization index (0 = consensus, 1 = fully polarized).
    pub polarization_index: Fixed,
    /// Total echo chamber strength across all clusters.
    pub echo_chamber_strength: Fixed,
    /// Total cross-cutting ties across all clusters.
    pub total_cross_cutting_ties: u32,
    /// §13.6 (AP2): Cross-cutting tie RATIO — the plan's `BeliefEcology`
    /// field. Stored as a `Fixed` 0–1 ratio (higher ties = less polarized),
    /// computed by `compute_polarization` as
    /// `Σ cluster cross-cutting ties / max possible ties` (each cross-cluster
    /// high-trust pair counted 2×). Persisted here so the plan-declared field
    /// is observable state rather than a discarded local. Observational only
    /// (no decision consumer) — zero-blast by construction.
    #[serde(default)]
    pub cross_cutting_ties: Fixed,
    /// §13.6 (AP2): Narrative dominance — how strongly each meme narrative
    /// dominates the pool (meme id → dominance 0–1). A `BTreeMap` (not the
    /// plan's `HashMap`) keeps serialization and iteration deterministic.
    /// Derived by the sim's daily meme pass.
    #[serde(default)]
    pub narrative_dominance: BTreeMap<u64, Fixed>,
    /// Next cluster id.
    next_id: usize,
}

impl Default for EchoChamberState {
    fn default() -> Self {
        Self {
            clusters: Vec::new(),
            polarization_index: Fixed::ZERO,
            echo_chamber_strength: Fixed::ZERO,
            total_cross_cutting_ties: 0,
            cross_cutting_ties: Fixed::ZERO,
            narrative_dominance: BTreeMap::new(),
            next_id: 0,
        }
    }
}

impl EchoChamberState {
    /// Create a new echo chamber state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new belief cluster and return its id.
    pub fn create_cluster(&mut self, dominant_narrative: String) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.clusters
            .push(BeliefCluster::new(id, dominant_narrative));
        id
    }

    /// Get a cluster by id.
    pub fn get_cluster(&self, id: usize) -> Option<&BeliefCluster> {
        self.clusters.iter().find(|c| c.id == id)
    }

    /// Get a mutable reference to a cluster by id.
    pub fn get_cluster_mut(&mut self, id: usize) -> Option<&mut BeliefCluster> {
        self.clusters.iter_mut().find(|c| c.id == id)
    }

    /// §13.6 (AP2): Narrative-dominance momentum — how strongly the most
    /// dominant narrative should pull the village's cluster assignment toward
    /// the institutional pole. Returns `ZERO` when no narrative clears
    /// `threshold` (identity: every calibrated window sits at ≤0.52 < 0.6),
    /// else `(max_dominance − threshold) × rate`, clamped to [0, 1].
    /// Deterministic: `BTreeMap` iteration, `max` of values, no RNG.
    pub fn narrative_momentum(&self, threshold: Fixed, rate: Fixed) -> Fixed {
        let max_dom = self
            .narrative_dominance
            .values()
            .copied()
            .fold(Fixed::ZERO, Fixed::max);
        if max_dom <= threshold {
            return Fixed::ZERO;
        }
        ((max_dom - threshold) * rate).clamp_01()
    }

    /// Compute overall polarization from cluster disagreements.
    ///
    /// ```text
    /// polarization = average_inter_cluster_distance
    ///              × (1 - average_cross_cutting_ties / max_possible_ties)
    /// ```
    pub fn compute_polarization(&mut self) {
        if self.clusters.len() < 2 {
            self.polarization_index = Fixed::ZERO;
            self.echo_chamber_strength = Fixed::ZERO;
            return;
        }

        // Average echo chamber strength across clusters
        let total_echo: Fixed = self
            .clusters
            .iter()
            .map(BeliefCluster::echo_strength)
            .fold(Fixed::ZERO, |acc, e| acc + e);
        let n_clusters = Fixed::from_int(self.clusters.len() as i64);
        self.echo_chamber_strength = (total_echo / n_clusters).clamp_01();

        // Polarization = emotional charge disparity + identity fusion
        let total_emotional_charge: Fixed = self
            .clusters
            .iter()
            .map(|c| c.emotional_charge)
            .fold(Fixed::ZERO, |acc, e| acc + e);
        let avg_emotional_charge = total_emotional_charge / n_clusters;

        let total_fusion: Fixed = self
            .clusters
            .iter()
            .map(|c| c.identity_fusion)
            .fold(Fixed::ZERO, |acc, f| acc + f);
        let avg_fusion = total_fusion / n_clusters;

        // Cross-cutting tie ratio (higher ties = less polarized).
        // Each cross-cluster high-trust pair is counted once per cluster involved
        // (2× total), so the denominator is 2 × Σ(sz_a × sz_b) over cluster pairs —
        // NOT the number of cluster pairs. The old denominator (n_clusters ×
        // (n_clusters − 1)) undercounted by an order of magnitude once trust
        // could actually rise, making tie_ratio > 1 and clamping polarization
        // to 0.0000 forever.
        self.total_cross_cutting_ties = self.clusters.iter().map(|c| c.cross_cutting_ties).sum();
        let tie_ratio = if self.clusters.len() > 1 {
            let mut max_possible: u64 = 0;
            for a in 0..self.clusters.len() {
                for b in (a + 1)..self.clusters.len() {
                    let sz_a = self.clusters[a].members.len() as u64;
                    let sz_b = self.clusters[b].members.len() as u64;
                    max_possible += sz_a * sz_b * 2; // each pair counted 2×
                }
            }
            if max_possible > 0 {
                let ratio = self.total_cross_cutting_ties as f64 / max_possible as f64;
                Fixed::from_f64(ratio.min(1.0))
            } else {
                Fixed::ZERO
            }
        } else {
            Fixed::ZERO
        };
        // §13.6 (Iteration 163): persist the ratio as the plan-declared
        // `cross_cutting_ties` state field (observational — the ratio was
        // previously a discarded local; storing it changes no computation).
        self.cross_cutting_ties = tie_ratio;

        // Additive blend: emotional_charge + fusion + echo chamber contribute,
        // while cross-cutting ties reduce polarization (per §13.6: polarization =
        // inter-cluster distance × (1 − cross-ties/max-ties)). Weights sum to 1
        // so the index is a true 0–1 measure without the old hard 0.67 × 0.5
        // deflation that kept even strong polarization below ~0.33.
        let w_charge = Fixed::from_f64(0.4);
        let w_fusion = Fixed::from_f64(0.3);
        let w_echo = Fixed::from_f64(0.3);
        self.polarization_index = (w_charge * avg_emotional_charge
            + w_fusion * avg_fusion
            + w_echo * self.echo_chamber_strength)
            * (Fixed::ONE - tie_ratio);
        self.polarization_index = self.polarization_index.clamp_01();
    }

    /// Move an agent from one cluster to another.
    /// Returns `false` if either cluster ID is invalid (no-op in that case).
    pub fn move_agent(&mut self, agent_id: usize, from: usize, to: usize) -> bool {
        // Validate both clusters exist before mutating to avoid losing the agent.
        if self.get_cluster(from).is_none() || self.get_cluster(to).is_none() {
            return false;
        }
        if let Some(src) = self.get_cluster_mut(from) {
            src.remove_member(agent_id);
        }
        if let Some(dst) = self.get_cluster_mut(to) {
            dst.add_member(agent_id);
        }
        true
    }

    /// Tick update on all clusters and recompute polarization.
    pub fn tick_update(&mut self) {
        for cluster in &mut self.clusters {
            cluster.tick_update();
        }
        self.compute_polarization();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cluster_has_sane_defaults() {
        let c = BeliefCluster::new(0, "the crown is just".into());
        assert_eq!(c.id, 0);
        assert!(c.members.is_empty());
        assert!(c.cohesion > Fixed::ZERO);
    }

    #[test]
    fn add_and_remove_member() {
        let mut c = BeliefCluster::new(0, "test".into());
        c.add_member(5);
        c.add_member(5); // duplicate ignored
        assert_eq!(c.members.len(), 1);
        c.remove_member(5);
        assert!(c.members.is_empty());
    }

    #[test]
    fn echo_strength_scales_with_fusion_and_hostility() {
        let mut c = BeliefCluster::new(0, "test".into());
        c.cohesion = Fixed::ONE;
        c.identity_fusion = Fixed::ONE;
        c.outgroup_hostility = Fixed::ONE;
        c.cross_cutting_ties = 0;
        let high_echo = c.echo_strength();

        let mut c2 = BeliefCluster::new(1, "test".into());
        c2.cohesion = Fixed::from_f64(0.3);
        c2.identity_fusion = Fixed::from_f64(0.1);
        c2.outgroup_hostility = Fixed::from_f64(0.1);
        c2.cross_cutting_ties = 5;
        let low_echo = c2.echo_strength();

        assert!(high_echo > low_echo);
    }

    #[test]
    fn cross_cutting_ties_reduce_echo_strength() {
        let mut c1 = BeliefCluster::new(0, "test".into());
        c1.cohesion = Fixed::ONE;
        c1.identity_fusion = Fixed::ONE;
        c1.outgroup_hostility = Fixed::ONE;
        c1.cross_cutting_ties = 0;
        let no_ties = c1.echo_strength();

        let mut c2 = c1;
        c2.cross_cutting_ties = 10;
        let many_ties = c2.echo_strength();

        assert!(no_ties > many_ties);
    }

    #[test]
    fn polarization_zero_with_single_cluster() {
        let mut state = EchoChamberState::new();
        state.create_cluster("narrative A".into());
        state.compute_polarization();
        assert_eq!(state.polarization_index, Fixed::ZERO);
    }

    #[test]
    fn polarization_increases_with_more_clusters() {
        let mut state = EchoChamberState::new();
        let c1 = state.create_cluster("narrative A".into());
        let c2 = state.create_cluster("narrative B".into());
        // Set high emotional charge and fusion, no cross-cutting ties
        if let Some(cluster) = state.get_cluster_mut(c1) {
            cluster.emotional_charge = Fixed::from_f64(0.9);
            cluster.identity_fusion = Fixed::from_f64(0.9);
            cluster.outgroup_hostility = Fixed::from_f64(0.8);
            cluster.cohesion = Fixed::from_f64(0.9);
        }
        if let Some(cluster) = state.get_cluster_mut(c2) {
            cluster.emotional_charge = Fixed::from_f64(0.9);
            cluster.identity_fusion = Fixed::from_f64(0.9);
            cluster.outgroup_hostility = Fixed::from_f64(0.8);
            cluster.cohesion = Fixed::from_f64(0.9);
        }
        state.compute_polarization();
        assert!(state.polarization_index > Fixed::ZERO);
    }

    #[test]
    fn move_agent_between_clusters() {
        let mut state = EchoChamberState::new();
        let c1 = state.create_cluster("A".into());
        let c2 = state.create_cluster("B".into());
        state.get_cluster_mut(c1).unwrap().add_member(5);
        assert_eq!(state.get_cluster(c1).unwrap().members.len(), 1);
        state.move_agent(5, c1, c2);
        assert!(state.get_cluster(c1).unwrap().members.is_empty());
        assert_eq!(state.get_cluster(c2).unwrap().members.len(), 1);
    }

    #[test]
    fn registry_create_and_get() {
        let mut state = EchoChamberState::new();
        let id = state.create_cluster("test".into());
        assert_eq!(id, 0);
        assert!(state.get_cluster(0).is_some());
        assert!(state.get_cluster(1).is_none());
    }

    #[test]
    fn narrative_dominance_defaults_to_empty() {
        let state = EchoChamberState::new();
        assert!(state.narrative_dominance.is_empty());
    }

    // ── §13.6 Narrative-dominance momentum tests (Iteration 120) ───────

    #[test]
    fn narrative_momentum_zero_at_and_below_calibrated_ceiling() {
        let mut state = EchoChamberState::new();
        // The probe-pinned calibrated ceiling (0.52) and below: identity.
        state.narrative_dominance.insert(0, Fixed::from_f64(0.52));
        state.narrative_dominance.insert(3, Fixed::from_f64(0.52));
        let m = state.narrative_momentum(
            Fixed::from_f64(NARRATIVE_DOMINANCE_THRESHOLD),
            Fixed::from_f64(NARRATIVE_MOMENTUM_RATE),
        );
        assert_eq!(
            m,
            Fixed::ZERO,
            "at/below the threshold the fold is identity"
        );
        let mut state2 = EchoChamberState::new();
        state2.narrative_dominance.insert(0, Fixed::from_f64(0.2));
        assert_eq!(
            state2.narrative_momentum(
                Fixed::from_f64(NARRATIVE_DOMINANCE_THRESHOLD),
                Fixed::from_f64(NARRATIVE_MOMENTUM_RATE),
            ),
            Fixed::ZERO
        );
    }

    #[test]
    fn narrative_momentum_scales_above_threshold() {
        let mut state = EchoChamberState::new();
        state.narrative_dominance.insert(0, Fixed::from_f64(0.8));
        let m = state.narrative_momentum(
            Fixed::from_f64(NARRATIVE_DOMINANCE_THRESHOLD),
            Fixed::from_f64(NARRATIVE_MOMENTUM_RATE),
        );
        assert_eq!(m, Fixed::from_f64(0.05), "(0.8 − 0.6) × 0.25");
    }

    #[test]
    fn narrative_momentum_empty_map_is_zero() {
        let state = EchoChamberState::new();
        assert_eq!(
            state.narrative_momentum(
                Fixed::from_f64(NARRATIVE_DOMINANCE_THRESHOLD),
                Fixed::from_f64(NARRATIVE_MOMENTUM_RATE),
            ),
            Fixed::ZERO
        );
    }

    #[test]
    fn narrative_momentum_takes_the_maximum_narrative() {
        let mut state = EchoChamberState::new();
        state.narrative_dominance.insert(1, Fixed::from_f64(0.7));
        state.narrative_dominance.insert(2, Fixed::from_f64(0.9));
        let m = state.narrative_momentum(
            Fixed::from_f64(NARRATIVE_DOMINANCE_THRESHOLD),
            Fixed::from_f64(NARRATIVE_MOMENTUM_RATE),
        );
        assert_eq!(m, Fixed::from_f64(0.075), "(0.9 − 0.6) × 0.25");
    }

    #[test]
    fn narrative_dominance_serde_roundtrip_and_old_save_restore() {
        let mut state = EchoChamberState::new();
        state.narrative_dominance.insert(3, Fixed::from_f64(0.7));
        state.narrative_dominance.insert(1, Fixed::from_f64(0.4));
        // BTreeMap keys serialize in sorted order — deterministic JSON.
        let json = serde_json::to_string(&state).unwrap();
        let back: EchoChamberState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.narrative_dominance, state.narrative_dominance);
        // Old save without the field (e.g. pre-Iteration-57) restores empty.
        let old = r#"{"clusters":[],"polarization_index":0,"echo_chamber_strength":0,"total_cross_cutting_ties":0,"next_id":0}"#;
        let restored: EchoChamberState = serde_json::from_str(old).unwrap();
        assert!(restored.narrative_dominance.is_empty());
    }

    /// §13.6 (Iteration 163): `compute_polarization` persists the
    /// cross-cutting tie RATIO into the plan-declared `cross_cutting_ties`
    /// Fixed field (previously a discarded local). With two singleton
    /// clusters and one cross-cluster high-trust pair, the max-possible
    /// denominator is 2 (each pair counted 2×) and the count is 2, so the
    /// ratio pins at exactly 1.0; with no ties it pins at 0.0.
    #[test]
    fn compute_polarization_persists_cross_cutting_tie_ratio() {
        let mut state = EchoChamberState::new();
        let a = state.create_cluster("A".into());
        let b = state.create_cluster("B".into());
        state.clusters[a].add_member(1);
        state.clusters[b].add_member(2);

        // Zero ties → ratio 0.
        state.compute_polarization();
        assert_eq!(state.cross_cutting_ties, Fixed::ZERO);
        assert_eq!(state.total_cross_cutting_ties, 0);

        // One high-trust cross-cluster pair → both clusters count it (2×),
        // max possible is also 2 → ratio exactly 1.0.
        state.clusters[a].cross_cutting_ties = 1;
        state.clusters[b].cross_cutting_ties = 1;
        state.compute_polarization();
        assert_eq!(state.cross_cutting_ties, Fixed::ONE);
        assert_eq!(state.total_cross_cutting_ties, 2);

        // The ratio stays in [0, 1] under an oversized count (clamped).
        state.clusters[a].cross_cutting_ties = 9;
        state.clusters[b].cross_cutting_ties = 9;
        state.compute_polarization();
        assert!(state.cross_cutting_ties <= Fixed::ONE);
    }
}
