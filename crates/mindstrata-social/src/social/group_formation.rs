//! §12.2: Group formation mechanics — emergent group cohesion and dissociation.
//!
//! Groups form when agents share proximity, repeated interaction, emotional
//! intensity, common threat, common need, shared identity, shared grievance,
//! charismatic focal agent, ritual participation, kinship, and economic
//! interdependence.
//!
//! ```text
//! group_formation_pressure =
//!     shared_grievance
//!   + shared_identity
//!   + emotional_synchrony
//!   + repeated_interaction
//!   + leadership_gravity
//!   + external_threat
//!   - social_cost
//!   - institutional_suppression
//! ```

use mindstrata_core::fixed::Fixed;
use mindstrata_psych::psychology::attachment::AttachmentStyle;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A candidate group that may form from emergent social dynamics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupCandidate {
    /// Indices of agents in the candidate group.
    pub members: Vec<usize>,
    /// Shared grievance that binds the group (0–1).
    pub shared_grievance: Fixed,
    /// Shared identity strength (0–1).
    pub shared_identity: Fixed,
    /// Emotional synchrony — how aligned members' emotions are (0–1).
    pub emotional_synchrony: Fixed,
    /// Frequency of interaction among members (0–1).
    pub repeated_interaction: Fixed,
    /// Strength of the focal leader's gravity (0–1).
    pub leadership_gravity: Fixed,
    /// External threat level perceived by the group (0–1).
    pub external_threat: Fixed,
    /// Social cost of forming/maintaining the group (0–1).
    pub social_cost: Fixed,
    /// Institutional suppression pressure (0–1).
    pub institutional_suppression: Fixed,
    /// §12.2 (AP2, Iteration 121): Shared trauma — how much bodily trauma the
    /// members jointly carry (mean of their nervous `trauma_load`). The
    /// plan's "bonding after shared trauma" emergent effect: trauma binds
    /// like experienced grievance. `#[serde(default)]` so pre-Iter-121 saves
    /// restore.
    #[serde(default)]
    pub shared_trauma: Fixed,
    /// Tick when this candidate was first identified.
    pub identified_tick: u64,
}

/// §12.2 (AP2, Iteration 121): Weight of the shared-trauma bonding term in
/// [`GroupCandidate::formation_pressure`] — deliberately equal to the
/// `shared_grievance` weight (0.2): bodily trauma binds like an experienced
/// grievance. Bounded by `shared_trauma ≤ 1.0`, so the term adds at most 0.2
/// to a candidate's pressure (flipping only candidates in (0.3, 0.5]).
///
/// Scope note: this is the FORMATION entry point of "trauma → group
/// psychology" only; existing groups' cohesion (`compute_group_cohesion`)
/// and `PeerGroup` state deliberately carry no trauma term yet — the natural
/// future extension if trauma-bonded groups should also hold together
/// better.
pub const TRAUMA_BONDING_WEIGHT: f64 = 0.2;

impl GroupCandidate {
    /// Compute the total formation pressure.
    ///
    /// Returns a value > 0.5 suggesting the group should formalize.
    pub fn formation_pressure(&self) -> Fixed {
        let positive = self.shared_grievance * Fixed::from_f64(0.2)
            + self.shared_identity * Fixed::from_f64(0.2)
            + self.emotional_synchrony * Fixed::from_f64(0.15)
            + self.repeated_interaction * Fixed::from_f64(0.15)
            + self.leadership_gravity * Fixed::from_f64(0.15)
            + self.external_threat * Fixed::from_f64(0.15)
            // §12.2 (AP2, Iteration 121): bonding after shared trauma —
            // shared bodily trauma binds like experienced grievance
            // (`TRAUMA_BONDING_WEIGHT` = 0.2, matching shared_grievance),
            // the plan's "bonding after shared trauma" effect from §7.2.
            + self.shared_trauma * Fixed::from_f64(TRAUMA_BONDING_WEIGHT);
        let negative = self.social_cost * Fixed::from_f64(0.15)
            + self.institutional_suppression * Fixed::from_f64(0.15);
        (positive - negative).clamp_01()
    }

    /// Should this group formalize?
    pub fn should_form(&self) -> bool {
        self.formation_pressure() > Fixed::from_f64(0.5)
    }
}

/// Group type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GroupType {
    /// Household. (doc added at S3 extraction)
    Household,
    /// Family. (doc added at S3 extraction)
    Family,
    /// Clan. (doc added at S3 extraction)
    Clan,
    /// Faction. (doc added at S3 extraction)
    Faction,
    /// Cult. (doc added at S3 extraction)
    Cult,
    /// Congregation. (doc added at S3 extraction)
    Congregation,
    /// Guild. (doc added at S3 extraction)
    Guild,
    /// Patronage. (doc added at S3 extraction)
    Patronage,
    /// PeerGroup. (doc added at S3 extraction)
    PeerGroup,
    /// Warband. (doc added at S3 extraction)
    Warband,
}

/// Group-level attachment style — §12.3: individual attachment styles scale
/// upward to whole groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub enum GroupAttachmentStyle {
    /// Trusts leadership, repairs conflict, provides support, tolerates dissent.
    #[default]
    Secure,
    /// Demands reassurance, fears abandonment, clings to leaders, panics under
    /// uncertainty.
    Anxious,
    /// Suppresses emotion, fragments under stress, distrusts dependence,
    /// isolates members.
    Avoidant,
    /// Oscillates between devotion and betrayal — volatile leadership, trauma
    /// bonding, prone to purges.
    Disorganized,
}

impl GroupAttachmentStyle {
    /// Cohesion retained per daily tick (§12.3 behavior).
    ///
    /// Secure groups hold together best; anxious groups lose cohesion slightly
    /// faster under uncertainty; avoidant groups fragment under stress;
    /// disorganized groups are the most volatile.
    pub fn cohesion_retention(self) -> Fixed {
        match self {
            GroupAttachmentStyle::Secure => Fixed::from_f64(0.996),
            GroupAttachmentStyle::Anxious => Fixed::from_f64(0.995),
            GroupAttachmentStyle::Avoidant => Fixed::from_f64(0.993),
            GroupAttachmentStyle::Disorganized => Fixed::from_f64(0.990),
        }
    }
}

/// Aggregate individual attachment styles into a group-level style (§12.3).
///
/// Uses the modal (most prevalent) style; ties resolve to the earlier variant
/// in Secure > Anxious > Avoidant > Disorganized priority order.
/// Deterministic: iterates a slice, no RNG, no hash-map iteration.
pub fn derive_group_attachment_style(member_styles: &[AttachmentStyle]) -> GroupAttachmentStyle {
    if member_styles.is_empty() {
        return GroupAttachmentStyle::Secure;
    }
    let mut counts = [0usize; 4];
    for style in member_styles {
        match style {
            AttachmentStyle::Secure => counts[0] += 1,
            AttachmentStyle::Anxious => counts[1] += 1,
            AttachmentStyle::Avoidant => counts[2] += 1,
            AttachmentStyle::Disorganized => counts[3] += 1,
        }
    }
    let mut best = GroupAttachmentStyle::Secure;
    let mut best_count = counts[0];
    for (idx, candidate_style) in [
        (1, GroupAttachmentStyle::Anxious),
        (2, GroupAttachmentStyle::Avoidant),
        (3, GroupAttachmentStyle::Disorganized),
    ] {
        if counts[idx] > best_count {
            best = candidate_style;
            best_count = counts[idx];
        }
    }
    best
}

/// Type alias for group registry IDs.
pub type GroupId = usize;

/// A formalized peer group that emerged from group formation pressure.
///
/// Architecture §12.2: Groups form when agents share proximity, repeated
/// interaction, emotional synchrony, shared grievance, or external threat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerGroup {
    /// Id. (doc added at S3 extraction)
    pub id: GroupId,
    /// Members. (doc added at S3 extraction)
    pub members: Vec<usize>,
    /// Leader. (doc added at S3 extraction)
    pub leader: Option<usize>,
    /// Group type. (doc added at S3 extraction)
    pub group_type: GroupType,
    /// Shared grievance. (doc added at S3 extraction)
    pub shared_grievance: Fixed,
    /// Shared identity. (doc added at S3 extraction)
    pub shared_identity: Fixed,
    /// Current group cohesion (decays without reinforcement).
    pub cohesion: Fixed,
    /// Formed tick. (doc added at S3 extraction)
    pub formed_tick: u64,
    /// Last interaction tick. (doc added at S3 extraction)
    pub last_interaction_tick: u64,
    /// Active. (doc added at S3 extraction)
    pub active: bool,
    /// Group-level attachment style — modal aggregate of member styles (§12.3).
    #[serde(default)]
    pub attachment_style: GroupAttachmentStyle,
}

impl PeerGroup {
    /// Create a new peer group from a GroupCandidate.
    pub fn from_candidate(candidate: &GroupCandidate, id: GroupId, tick: u64) -> Self {
        let leader = candidate.members.first().copied();
        let cohesion = compute_group_cohesion(
            candidate.shared_identity,
            candidate.repeated_interaction,
            candidate.external_threat,
            Fixed::ZERO,
            0,
        );
        Self {
            id,
            members: candidate.members.clone(),
            leader,
            group_type: GroupType::PeerGroup,
            shared_grievance: candidate.shared_grievance,
            shared_identity: candidate.shared_identity,
            cohesion,
            formed_tick: tick,
            last_interaction_tick: tick,
            active: true,
            // Callers derive the modal member style (sim.rs formation pass);
            // Secure is the neutral default for direct constructions.
            attachment_style: GroupAttachmentStyle::Secure,
        }
    }

    /// Daily update — decay cohesion (style-dependent, §12.3) and check
    /// dissolution.
    pub fn daily_update(&mut self) {
        self.cohesion = (self.cohesion * self.attachment_style.cohesion_retention()).clamp_01();
        if self.cohesion < Fixed::from_f64(0.1) || self.members.len() < 2 {
            self.active = false;
        }
    }
}

/// Registry for all active peer groups in the simulation.
///
/// Maintains an O(1) membership cache for fast agent-in-group lookups.
/// The cache is rebuilt on register/dissolve operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRegistry {
    /// Groups. (doc added at S3 extraction)
    pub groups: Vec<PeerGroup>,
    /// O(1) membership cache: set of agent indices currently in active groups.
    #[serde(skip)]
    active_members: HashSet<usize>,
}

impl Default for GroupRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupRegistry {
    /// Fn. (doc added at S3 extraction)
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            active_members: HashSet::new(),
        }
    }

    /// Fn. (doc added at S3 extraction)
    pub fn register(&mut self, group: PeerGroup) -> GroupId {
        let id = self.groups.len();
        // Update membership cache for new group.
        if group.active {
            for &member in &group.members {
                self.active_members.insert(member);
            }
        }
        self.groups.push(group);
        id
    }

    /// Fn. (doc added at S3 extraction)
    pub fn get(&self, id: GroupId) -> Option<&PeerGroup> {
        self.groups.get(id)
    }

    /// Fn. (doc added at S3 extraction)
    pub fn get_mut(&mut self, id: GroupId) -> Option<&mut PeerGroup> {
        self.groups.get_mut(id)
    }

    /// Return all active groups.
    pub fn active(&self) -> Vec<(GroupId, &PeerGroup)> {
        self.groups
            .iter()
            .enumerate()
            .filter(|(_, g)| g.active)
            .collect()
    }

    /// Daily update for all groups — decay cohesion and dissolve inactive groups.
    /// Rebuilds the membership cache after dissolution.
    pub fn daily_update(&mut self) {
        self.active_members.clear();
        for group in &mut self.groups {
            if group.active {
                group.daily_update();
            }
            // Rebuild cache for groups that remain active.
            if group.active {
                for &member in &group.members {
                    self.active_members.insert(member);
                }
            }
        }
    }

    /// Check if an agent is already in an active group.
    /// O(1) lookup via the membership cache.
    pub fn agent_in_active_group(&self, agent_idx: usize) -> bool {
        self.active_members.contains(&agent_idx)
    }

    /// Rebuild the membership cache from scratch.
    /// Call after deserializing a snapshot (serde skips the cache).
    pub fn rebuild_cache(&mut self) {
        self.active_members.clear();
        for group in &self.groups {
            if group.active {
                for &member in &group.members {
                    self.active_members.insert(member);
                }
            }
        }
    }
}

/// Evaluate whether a set of agents should form a peer group.
pub fn evaluate_peer_group(
    agent_indices: &[usize],
    agent_ages: &[Fixed],
    agent_openness: &[Fixed],
    avg_interaction_frequency: Fixed,
    _tick: u64,
) -> Option<GroupCandidate> {
    if agent_indices.len() < 3 {
        return None;
    }
    let mean_age: Fixed = agent_ages.iter().fold(Fixed::ZERO, |acc, a| acc + *a)
        / Fixed::from_int(agent_ages.len() as i64);
    let age_variance: Fixed = agent_ages
        .iter()
        .map(|a| {
            let diff = *a - mean_age;
            diff * diff
        })
        .fold(Fixed::ZERO, |acc, v| acc + v)
        / Fixed::from_int(agent_ages.len() as i64);
    let age_similarity = (Fixed::ONE - age_variance * Fixed::from_f64(0.1)).clamp_01();
    let avg_openness: Fixed = agent_openness.iter().fold(Fixed::ZERO, |acc, o| acc + *o)
        / Fixed::from_int(agent_openness.len() as i64);
    Some(GroupCandidate {
        members: agent_indices.to_vec(),
        shared_grievance: Fixed::ZERO,
        shared_identity: avg_openness * age_similarity,
        emotional_synchrony: Fixed::from_f64(0.3),
        repeated_interaction: avg_interaction_frequency,
        leadership_gravity: Fixed::from_f64(0.2),
        external_threat: Fixed::ZERO,
        social_cost: Fixed::from_f64(0.1),
        institutional_suppression: Fixed::ZERO,
        shared_trauma: Fixed::ZERO,
        identified_tick: 0,
    })
}

/// Compute group cohesion dynamics for an existing group.
pub fn compute_group_cohesion(
    shared_identity: Fixed,
    repeated_interaction: Fixed,
    external_threat: Fixed,
    institutional_support: Fixed,
    time_since_formation: u64,
) -> Fixed {
    let base = shared_identity * Fixed::from_f64(0.3)
        + repeated_interaction * Fixed::from_f64(0.3)
        + external_threat * Fixed::from_f64(0.2)
        + institutional_support * Fixed::from_f64(0.2);
    let time_bonus =
        Fixed::from_f64(time_since_formation as f64 * 0.0001).min(Fixed::from_f64(0.1));
    (base + time_bonus).clamp_01()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formation_pressure_computed() {
        let candidate = GroupCandidate {
            members: vec![0, 1, 2],
            shared_grievance: Fixed::from_f64(0.7),
            shared_identity: Fixed::from_f64(0.6),
            emotional_synchrony: Fixed::from_f64(0.5),
            repeated_interaction: Fixed::from_f64(0.4),
            leadership_gravity: Fixed::from_f64(0.3),
            external_threat: Fixed::from_f64(0.6),
            social_cost: Fixed::from_f64(0.1),
            institutional_suppression: Fixed::from_f64(0.1),
            shared_trauma: Fixed::ZERO,
            identified_tick: 0,
        };
        assert!(candidate.formation_pressure() > Fixed::from_f64(0.3));
    }

    // ── §12.2 Shared-trauma bonding tests (Iteration 121) ────────────

    #[test]
    fn shared_trauma_increases_formation_pressure() {
        let base = GroupCandidate {
            members: vec![0, 1, 2],
            shared_grievance: Fixed::from_f64(0.5),
            shared_identity: Fixed::from_f64(0.5),
            emotional_synchrony: Fixed::from_f64(0.5),
            repeated_interaction: Fixed::from_f64(0.5),
            leadership_gravity: Fixed::ZERO,
            external_threat: Fixed::ZERO,
            social_cost: Fixed::ZERO,
            institutional_suppression: Fixed::ZERO,
            shared_trauma: Fixed::ZERO,
            identified_tick: 0,
        };
        let mut bonded = base.clone();
        bonded.shared_trauma = Fixed::ONE;
        let p0 = base.formation_pressure();
        let p1 = bonded.formation_pressure();
        assert!(p1 > p0, "trauma adds bonding pressure");
        assert_eq!(
            p1,
            p0 + Fixed::from_f64(0.2),
            "trauma weighs like grievance (0.2)"
        );
    }

    #[test]
    fn shared_trauma_can_flip_should_form() {
        // A candidate just below the 0.5 boundary bonds over it at full trauma.
        let mut candidate = GroupCandidate {
            members: vec![0, 1, 2],
            shared_grievance: Fixed::from_f64(0.5),
            shared_identity: Fixed::from_f64(0.5),
            emotional_synchrony: Fixed::from_f64(0.5),
            repeated_interaction: Fixed::from_f64(0.5),
            leadership_gravity: Fixed::ZERO,
            external_threat: Fixed::ZERO,
            social_cost: Fixed::from_f64(0.2),
            institutional_suppression: Fixed::ZERO,
            shared_trauma: Fixed::ZERO,
            identified_tick: 0,
        };
        // 0.5×0.2 + 0.5×0.2 + 0.5×0.15 + 0.5×0.15 − 0.2×0.15 = 0.35 − 0.03 = 0.32
        assert!(!candidate.should_form());
        candidate.shared_trauma = Fixed::ONE; // +0.2 → 0.52
        assert!(
            candidate.should_form(),
            "shared trauma bonds the group into form"
        );
    }

    #[test]
    fn shared_trauma_serde_restores_zero() {
        // Pre-Iter-121 saves lack the field and must restore 0.
        let old = r#"{"members":[],"shared_grievance":0,"shared_identity":0,"emotional_synchrony":0,"repeated_interaction":0,"leadership_gravity":0,"external_threat":0,"social_cost":0,"institutional_suppression":0,"identified_tick":0}"#;
        let restored: GroupCandidate = serde_json::from_str(old).unwrap();
        assert_eq!(restored.shared_trauma, Fixed::ZERO);
    }

    #[test]
    fn group_cohesion_increases_with_shared_identity() {
        let low = compute_group_cohesion(
            Fixed::from_f64(0.2),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.1),
            Fixed::from_f64(0.1),
            100,
        );
        let high = compute_group_cohesion(
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.1),
            Fixed::from_f64(0.1),
            100,
        );
        assert!(high > low);
    }

    #[test]
    fn peer_group_needs_minimum_members() {
        let result = evaluate_peer_group(
            &[0, 1],
            &[Fixed::from_f64(25.0), Fixed::from_f64(26.0)],
            &[Fixed::from_f64(0.5), Fixed::from_f64(0.6)],
            Fixed::from_f64(0.3),
            0,
        );
        assert!(result.is_none());
    }

    #[test]
    fn group_registry_register_and_active() {
        let mut reg = GroupRegistry::new();
        let candidate = GroupCandidate {
            members: vec![0, 1, 2],
            shared_grievance: Fixed::from_f64(0.7),
            shared_identity: Fixed::from_f64(0.6),
            emotional_synchrony: Fixed::from_f64(0.5),
            repeated_interaction: Fixed::from_f64(0.4),
            leadership_gravity: Fixed::from_f64(0.3),
            external_threat: Fixed::from_f64(0.6),
            social_cost: Fixed::from_f64(0.1),
            institutional_suppression: Fixed::from_f64(0.1),
            shared_trauma: Fixed::ZERO,
            identified_tick: 0,
        };
        let group = PeerGroup::from_candidate(&candidate, 0, 100);
        let id = reg.register(group);
        assert_eq!(id, 0);
        assert_eq!(reg.active().len(), 1);
    }

    #[test]
    fn peer_group_dissolves_on_low_cohesion() {
        let mut group = PeerGroup {
            id: 0,
            members: vec![0, 1],
            leader: Some(0),
            group_type: GroupType::PeerGroup,
            shared_grievance: Fixed::ZERO,
            shared_identity: Fixed::ZERO,
            cohesion: Fixed::from_f64(0.05),
            formed_tick: 0,
            last_interaction_tick: 0,
            active: true,
            attachment_style: GroupAttachmentStyle::Secure,
        };
        group.daily_update();
        assert!(!group.active);
    }

    #[test]
    fn group_registry_cache_works_after_deserialization() {
        // Register a group, serialize, deserialize (cache skipped), rebuild, verify.
        let mut reg = GroupRegistry::new();
        let candidate = GroupCandidate {
            members: vec![0, 1, 2],
            shared_grievance: Fixed::from_f64(0.7),
            shared_identity: Fixed::from_f64(0.6),
            emotional_synchrony: Fixed::from_f64(0.5),
            repeated_interaction: Fixed::from_f64(0.4),
            leadership_gravity: Fixed::from_f64(0.3),
            external_threat: Fixed::from_f64(0.6),
            social_cost: Fixed::from_f64(0.1),
            institutional_suppression: Fixed::from_f64(0.1),
            shared_trauma: Fixed::ZERO,
            identified_tick: 0,
        };
        let group = PeerGroup::from_candidate(&candidate, 0, 100);
        reg.register(group);

        // Cache is populated after register.
        assert!(reg.agent_in_active_group(0));
        assert!(reg.agent_in_active_group(1));
        assert!(!reg.agent_in_active_group(99));

        // Serialize and deserialize — cache is skipped.
        let json = serde_json::to_string(&reg).expect("serialize");
        let mut restored: GroupRegistry = serde_json::from_str(&json).expect("deserialize");

        // Cache is empty after deserialization.
        assert!(!restored.agent_in_active_group(0));
        assert!(!restored.agent_in_active_group(1));
        assert!(!restored.agent_in_active_group(2));
        assert_eq!(restored.groups.len(), 1);
        assert!(restored.groups[0].active);

        // Rebuild cache — membership is restored.
        restored.rebuild_cache();
        assert!(restored.agent_in_active_group(0));
        assert!(restored.agent_in_active_group(1));
        assert!(restored.agent_in_active_group(2));
        assert!(!restored.agent_in_active_group(99));
    }

    #[test]
    fn group_registry_daily_update_rebuilds_cache() {
        let mut reg = GroupRegistry::new();
        let candidate = GroupCandidate {
            members: vec![0, 1, 2],
            shared_grievance: Fixed::from_f64(0.7),
            shared_identity: Fixed::from_f64(0.6),
            emotional_synchrony: Fixed::from_f64(0.5),
            repeated_interaction: Fixed::from_f64(0.4),
            leadership_gravity: Fixed::from_f64(0.3),
            external_threat: Fixed::from_f64(0.6),
            social_cost: Fixed::from_f64(0.1),
            institutional_suppression: Fixed::from_f64(0.1),
            shared_trauma: Fixed::ZERO,
            identified_tick: 0,
        };
        reg.register(PeerGroup::from_candidate(&candidate, 0, 100));
        assert!(reg.agent_in_active_group(0));

        // After daily_update, cache is rebuilt and agent is still in active group.
        reg.daily_update();
        assert!(reg.agent_in_active_group(0));
        assert!(reg.agent_in_active_group(1));
    }

    #[test]
    fn group_registry_dissolution_removes_from_cache() {
        let mut reg = GroupRegistry::new();
        // Register a group with very low cohesion so it dissolves on daily_update.
        let group = PeerGroup {
            id: 0,
            members: vec![0, 1],
            leader: Some(0),
            group_type: GroupType::PeerGroup,
            shared_grievance: Fixed::ZERO,
            shared_identity: Fixed::ZERO,
            cohesion: Fixed::from_f64(0.05), // below 0.1 threshold
            formed_tick: 0,
            last_interaction_tick: 0,
            active: true,
            attachment_style: GroupAttachmentStyle::Secure,
        };
        reg.register(group);
        assert!(reg.agent_in_active_group(0));
        assert!(reg.agent_in_active_group(1));

        // daily_update dissolves the group — members should leave the cache.
        reg.daily_update();
        assert!(!reg.agent_in_active_group(0));
        assert!(!reg.agent_in_active_group(1));
        assert!(!reg.groups[0].active);
    }

    #[test]
    fn group_attachment_style_derives_modal_member_style() {
        use mindstrata_psych::psychology::attachment::AttachmentStyle::{
            Anxious, Avoidant, Disorganized, Secure,
        };
        // Majority anxious → Anxious.
        assert_eq!(
            derive_group_attachment_style(&[Secure, Anxious, Anxious, Avoidant]),
            GroupAttachmentStyle::Anxious
        );
        // Majority avoidant → Avoidant.
        assert_eq!(
            derive_group_attachment_style(&[Avoidant, Avoidant, Secure, Disorganized]),
            GroupAttachmentStyle::Avoidant
        );
        // All disorganized → Disorganized.
        assert_eq!(
            derive_group_attachment_style(&[Disorganized, Disorganized]),
            GroupAttachmentStyle::Disorganized
        );
        // Empty → Secure default.
        assert_eq!(
            derive_group_attachment_style(&[]),
            GroupAttachmentStyle::Secure
        );
    }

    #[test]
    fn group_attachment_style_ties_resolve_to_higher_priority() {
        use mindstrata_psych::psychology::attachment::AttachmentStyle::{
            Anxious, Avoidant, Secure,
        };
        // 1v1v1v1 tie → Secure (highest priority).
        assert_eq!(
            derive_group_attachment_style(&[Secure, Anxious, Avoidant]),
            GroupAttachmentStyle::Secure
        );
        // 2v2 tie Anxious vs Avoidant → Anxious.
        assert_eq!(
            derive_group_attachment_style(&[Anxious, Anxious, Avoidant, Avoidant]),
            GroupAttachmentStyle::Anxious
        );
    }

    #[test]
    fn group_attachment_style_retention_ordering_is_secure_first() {
        let secure = GroupAttachmentStyle::Secure.cohesion_retention();
        let anxious = GroupAttachmentStyle::Anxious.cohesion_retention();
        let avoidant = GroupAttachmentStyle::Avoidant.cohesion_retention();
        let disorganized = GroupAttachmentStyle::Disorganized.cohesion_retention();
        // Secure holds cohesion best; disorganized worst.
        assert!(secure > anxious);
        assert!(anxious > avoidant);
        assert!(avoidant > disorganized);
    }

    #[test]
    fn disorganized_groups_lose_cohesion_faster_than_secure() {
        let candidate = GroupCandidate {
            members: vec![0, 1, 2],
            shared_grievance: Fixed::from_f64(0.7),
            shared_identity: Fixed::from_f64(0.6),
            emotional_synchrony: Fixed::from_f64(0.5),
            repeated_interaction: Fixed::from_f64(0.4),
            leadership_gravity: Fixed::from_f64(0.3),
            external_threat: Fixed::from_f64(0.6),
            social_cost: Fixed::from_f64(0.1),
            institutional_suppression: Fixed::from_f64(0.1),
            shared_trauma: Fixed::ZERO,
            identified_tick: 0,
        };
        let mut secure = PeerGroup::from_candidate(&candidate, 0, 0);
        secure.attachment_style = GroupAttachmentStyle::Secure;
        let mut disorganized = PeerGroup::from_candidate(&candidate, 1, 0);
        disorganized.attachment_style = GroupAttachmentStyle::Disorganized;
        secure.cohesion = Fixed::from_f64(0.9);
        disorganized.cohesion = Fixed::from_f64(0.9);
        for _ in 0..10 {
            secure.daily_update();
            disorganized.daily_update();
        }
        assert!(disorganized.cohesion < secure.cohesion);
        assert!(secure.active);
        assert!(disorganized.active);
    }

    #[test]
    fn peer_group_attachment_style_old_snapshots_default_to_secure() {
        // Old snapshots (no field) must deserialize to Secure. Fixed values
        // are the raw scaled i64 (0.5 → 5000).
        let json = r#"{"id":0,"members":[0,1],"leader":0,"group_type":"PeerGroup","shared_grievance":0,"shared_identity":0,"cohesion":5000,"formed_tick":0,"last_interaction_tick":0,"active":true}"#;
        let group: PeerGroup = serde_json::from_str(json).unwrap();
        assert_eq!(group.attachment_style, GroupAttachmentStyle::Secure);
    }
}
