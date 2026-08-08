//! §5.2, §29.2: Faction v2 — upgraded faction system with mobilization, morale,
//! supplies, leadership dynamics, casualties, and legitimacy of violence.
//!
//! Architecture §29.2: "Factions and institutions need mobilization capacity,
//! morale, supplies, leadership, cohesion, casualties, legitimacy of violence."
//!
//! This module extends the existing `factions.rs` (grievance, recruitment,
//! protest, revolution) with richer faction-internal dynamics.
//!
//! ```text
//! Faction v2 dynamics:
//!   - Mobilization capacity: how many agents can the faction field?
//!   - Morale: collective emotional state driving willingness to act
//!   - Supplies: material resources sustaining faction activity
//!   - Leadership: leader competence, trust, succession pressure
//!   - Cohesion: internal unity vs fragmentation
//!   - Casualties: losses from conflict reduce capacity and morale
//!   - Legitimacy of violence: perceived rightfulness of force
//! ```

use crate::psychology::attachment::AttachmentStyle;
use crate::social::group_formation::{derive_group_attachment_style, GroupAttachmentStyle};
use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// §12.3 style-modulation table: how each group-level attachment style
/// rescales the shared daily dynamics. One-point tuning for recalibration.
///
/// - `scarcity_penalty` — morale-loss scale under low supplies (anxious
///   panic, avoidant emotional suppression).
/// - `stress_fragmentation` — daily fragmentation growth at low cohesion
///   (avoidant fragments, secure tolerates dissent).
/// - `cohesion_retention` — daily cohesion multiplier (secure holds best,
///   disorganized most volatile).
/// - `trust_decay` — daily trust-in-leadership loss under an incompetent
///   leader (anxious cling, avoidant distrust of dependence).
struct StyleDynamics {
    scarcity_penalty: Fixed,
    stress_fragmentation: Fixed,
    cohesion_retention: Fixed,
    trust_decay: Fixed,
}

impl StyleDynamics {
    fn of(style: GroupAttachmentStyle) -> Self {
        match style {
            GroupAttachmentStyle::Secure => Self {
                scarcity_penalty: Fixed::from_f64(1.0),
                stress_fragmentation: Fixed::from_f64(0.0005),
                cohesion_retention: Fixed::from_f64(0.9995),
                trust_decay: Fixed::from_f64(0.002),
            },
            GroupAttachmentStyle::Anxious => Self {
                scarcity_penalty: Fixed::from_f64(1.5),
                stress_fragmentation: Fixed::from_f64(0.00075),
                cohesion_retention: Fixed::from_f64(0.999),
                trust_decay: Fixed::from_f64(0.001),
            },
            GroupAttachmentStyle::Avoidant => Self {
                scarcity_penalty: Fixed::from_f64(0.5),
                stress_fragmentation: Fixed::from_f64(0.002),
                cohesion_retention: Fixed::from_f64(0.9985),
                trust_decay: Fixed::from_f64(0.004),
            },
            GroupAttachmentStyle::Disorganized => Self {
                scarcity_penalty: Fixed::from_f64(1.25),
                stress_fragmentation: Fixed::from_f64(0.0015),
                cohesion_retention: Fixed::from_f64(0.998),
                trust_decay: Fixed::from_f64(0.003),
            },
        }
    }
}

/// Internal state of a faction's mobilization and combat readiness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionV2 {
    /// Index of the faction leader agent.
    pub leader: usize,
    /// Member agent indices.
    pub members: Vec<usize>,
    /// Overall grievance level driving the faction (0–1).
    pub grievance: Fixed,
    /// Mobilization capacity — fraction of members willing to act (0–1).
    pub mobilization_capacity: Fixed,
    /// Collective morale — willingness to fight/sacrifice (0–1).
    pub morale: Fixed,
    /// Supply level — material resources available (0–1).
    pub supplies: Fixed,
    /// Supply consumption rate per tick.
    pub supply_consumption: Fixed,
    /// Leadership competence (0–1).
    pub leader_competence: Fixed,
    /// Trust in leadership from members (0–1).
    pub trust_in_leadership: Fixed,
    /// Internal cohesion — unity vs fragmentation (0–1).
    pub cohesion: Fixed,
    /// Fragmentation pressure — dissent, internal rivalry (0–1).
    pub fragmentation: Fixed,
    /// Cumulative casualties (0–1 of original strength).
    pub casualties: Fixed,
    /// Legitimacy of violence — perceived rightfulness of force (0–1).
    pub legitimacy_of_violence: Fixed,
    /// Whether the faction is currently active.
    pub active: bool,
    /// Tick when the faction was formed.
    pub formed_tick: u64,
    /// Tick of last significant event (protest, conflict, etc.).
    pub last_event_tick: u64,
    /// Architecture-plan-2 §12.3: Group-level attachment style — the modal
    /// aggregate of member attachment styles. Factions are the largest group
    /// type, so "attachment styles scale upward" applies here with the
    /// largest behavioral surface: trust in leadership, fragmentation under
    /// stress, and volatility of cohesion all modulate by style.
    #[serde(default)]
    pub attachment_style: GroupAttachmentStyle,
}

impl FactionV2 {
    /// Create a new faction v2 from grievance and initial member count.
    pub fn new(
        leader: usize,
        members: Vec<usize>,
        grievance: Fixed,
        tick: u64,
    ) -> Self {
        // Higher grievance = higher initial morale and mobilization
        let morale = (grievance * Fixed::from_f64(0.8) + Fixed::from_f64(0.2)).clamp_01();
        let mobilization = (grievance * Fixed::from_f64(0.6) + Fixed::from_f64(0.1)).clamp_01();
        // Legitimacy of violence starts low — grows with perceived injustice
        let legitimacy_of_violence = (grievance * Fixed::from_f64(0.3)).clamp_01();

        Self {
            leader,
            members,
            grievance,
            mobilization_capacity: mobilization,
            morale,
            supplies: Fixed::from_f64(0.7), // start with moderate supplies
            supply_consumption: Fixed::from_f64(0.001),
            leader_competence: Fixed::from_f64(0.5),
            trust_in_leadership: Fixed::from_f64(0.6),
            cohesion: (grievance * Fixed::from_f64(0.7) + Fixed::from_f64(0.2)).clamp_01(),
            fragmentation: Fixed::from_f64(0.1),
            casualties: Fixed::ZERO,
            legitimacy_of_violence,
            active: true,
            formed_tick: tick,
            last_event_tick: tick,
            // §12.3: Secure is the neutral default for direct constructions;
            // the sim derives the modal member style at registration.
            attachment_style: GroupAttachmentStyle::Secure,
        }
    }

    /// §12.3: Aggregate member attachment styles into the faction-level style.
    ///
    /// Modal aggregation, deterministic, no RNG — identical to the peer-group
    /// derivation so both group types scale upward by the same rule.
    pub fn derive_attachment_style(&mut self, member_styles: &[AttachmentStyle]) {
        self.attachment_style = derive_group_attachment_style(member_styles);
    }

    /// Compute effective fighting strength = mobilization × morale × (1 − casualties).
    pub fn fighting_strength(&self) -> Fixed {
        let base = self.mobilization_capacity * self.morale;
        let casualty_penalty = Fixed::ONE - self.casualties;
        (base * casualty_penalty).clamp_01()
    }

    /// Compute faction threat level to institutions.
    ///
    /// Threat = fighting_strength × cohesion × grievance × (1 − fragmentation).
    pub fn threat_level(&self) -> Fixed {
        let strength = self.fighting_strength();
        let cohesion_factor = self.cohesion * (Fixed::ONE - self.fragmentation);
        (strength * cohesion_factor * self.grievance).clamp_01()
    }

    /// Record casualties from a conflict or crackdown.
    ///
    /// Also increases grievance and legitimacy_of_violence (martyrdom effect):
    /// casualties radicalize survivors and make violence feel more justified.
    pub fn record_casualties(&mut self, loss_fraction: Fixed) {
        self.casualties = (self.casualties + loss_fraction).clamp_01();
        // Casualties reduce morale and mobilization
        let morale_hit = loss_fraction * Fixed::from_f64(0.5);
        self.morale = (self.morale - morale_hit).max(Fixed::ZERO);
        self.mobilization_capacity = (self.mobilization_capacity - loss_fraction * Fixed::from_f64(0.3))
            .max(Fixed::ZERO);
        // But casualties can increase grievance and legitimacy of violence (martyrdom effect)
        self.grievance = (self.grievance + loss_fraction * Fixed::from_f64(0.1)).clamp_01();
        self.legitimacy_of_violence = (self.legitimacy_of_violence
            + loss_fraction * Fixed::from_f64(0.05))
            .clamp_01();
    }

    /// Daily update — decay morale, consume supplies, adjust cohesion.
    ///
    /// §12.3: Style-specific dynamics modulate the shared baseline — secure
    /// factions trust leadership and hold cohesion best; anxious factions
    /// cling to leaders but panic under scarcity; avoidant factions fragment
    /// under stress and distrust dependence; disorganized factions are the
    /// most volatile (fastest cohesion loss, fastest fragmentation growth).
    pub fn daily_update(&mut self) {
        let dyn_ = StyleDynamics::of(self.attachment_style);
        // Supplies decay with consumption (style-independent)
        self.supplies = (self.supplies - self.supply_consumption).max(Fixed::ZERO);
        // Low supplies reduce morale — scaled by style (anxious panics under
        // uncertainty, avoidant suppresses emotion).
        if self.supplies < Fixed::from_f64(0.3) {
            let supply_penalty = (Fixed::from_f64(0.3) - self.supplies) * Fixed::from_f64(0.1);
            self.morale = (self.morale - supply_penalty * dyn_.scarcity_penalty).max(Fixed::ZERO);
        }
        // Morale slowly decays without reinforcement
        self.morale = (self.morale * Fixed::from_f64(0.998)).max(Fixed::ZERO);
        // Fragmentation grows under low cohesion — scaled by style.
        if self.cohesion < Fixed::from_f64(0.4) {
            self.fragmentation = (self.fragmentation + dyn_.stress_fragmentation).clamp_01();
        }
        // Cohesion slowly decays without shared activity — scaled by style.
        self.cohesion = (self.cohesion * dyn_.cohesion_retention).max(Fixed::ZERO);
        // Trust in leadership decays if leader is incompetent — scaled by style.
        if self.leader_competence < Fixed::from_f64(0.3) {
            self.trust_in_leadership = (self.trust_in_leadership - dyn_.trust_decay).max(Fixed::ZERO);
        }
    }

    /// Check if the faction should dissolve.
    pub fn should_dissolve(&self) -> bool {
        // Dissolve if: too few members, morale collapsed, or casualties too high
        self.members.len() < 2
            || self.morale < Fixed::from_f64(0.1)
            || self.casualties > Fixed::from_f64(0.7)
    }

    /// Compute the recruitment chance for a potential recruit.
    ///
    /// Returns a probability (0–1) that the recruit will join.
    /// The caller should roll against this probability.
    pub fn recruitment_chance(
        &self,
        recruit_grievance: Fixed,
        max_members: usize,
    ) -> Fixed {
        if self.members.len() >= max_members {
            return Fixed::ZERO;
        }
        if recruit_grievance < Fixed::from_f64(0.4) {
            return Fixed::ZERO;
        }
        // Recruitment chance based on faction morale and recruit grievance
        (self.morale * recruit_grievance).clamp_01()
    }
}

/// Registry of all factions v2 in the simulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactionV2Registry {
    pub factions: Vec<FactionV2>,
}

impl FactionV2Registry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            factions: Vec::new(),
        }
    }

    /// Register a new faction and return its index.
    pub fn register(&mut self, faction: FactionV2) -> usize {
        let idx = self.factions.len();
        self.factions.push(faction);
        idx
    }

    /// Get all active factions.
    pub fn active(&self) -> Vec<(usize, &FactionV2)> {
        self.factions
            .iter()
            .enumerate()
            .filter(|(_, f)| f.active)
            .collect()
    }

    /// Deactivate every faction whose leader matches `leader`.
    ///
    /// Used after a successful revolution: the faction institution dissolves
    /// (its members take the council), so its FactionV2 record must not keep
    /// reporting an active faction that no longer exists as an institution
    /// (that would feed stale threat/agent-tier state).
    pub fn deactivate_by_leader(&mut self, leader: usize) {
        for faction in &mut self.factions {
            if faction.leader == leader {
                faction.active = false;
            }
        }
    }

    /// §5.2 (Iteration 89): Deactivate every record — the coup path
    /// dissolves *all* v1 factions (`retain`), so every active v2 record
    /// whose institution is being wiped becomes stale and must stop
    /// reporting as active (threat/agent-tier surface). Complements
    /// `deactivate_by_leader`, which only matches the revolting leader and
    /// left the other factions' records stale-active, breaking the v1↔v2
    /// 1:1 registration invariant after a coup.
    pub fn deactivate_all(&mut self) {
        for faction in &mut self.factions {
            faction.active = false;
        }
    }

    /// Daily update for all active factions.
    pub fn daily_update(&mut self) {
        for faction in &mut self.factions {
            if faction.active {
                faction.daily_update();
                if faction.should_dissolve() {
                    faction.active = false;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fighting_strength_computed() {
        let f = FactionV2::new(0, vec![0, 1, 2], Fixed::from_f64(0.7), 0);
        let strength = f.fighting_strength();
        assert!(strength > Fixed::ZERO);
        assert!(strength <= Fixed::ONE);
    }

    #[test]
    fn threat_level_includes_cohesion() {
        let mut f1 = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        let mut f2 = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        f1.cohesion = Fixed::from_f64(0.9);
        f2.cohesion = Fixed::from_f64(0.2);
        assert!(f1.threat_level() > f2.threat_level());
    }

    #[test]
    fn casualties_reduce_strength() {
        let mut f = FactionV2::new(0, vec![0, 1, 2], Fixed::from_f64(0.7), 0);
        let before = f.fighting_strength();
        f.record_casualties(Fixed::from_f64(0.3));
        assert!(f.fighting_strength() < before);
    }

    #[test]
    fn casualties_increase_grievance() {
        let mut f = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.5), 0);
        let before = f.grievance;
        f.record_casualties(Fixed::from_f64(0.2));
        assert!(f.grievance > before);
    }

    #[test]
    fn dissolve_on_low_morale() {
        let mut f = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        f.morale = Fixed::from_f64(0.05);
        assert!(f.should_dissolve());
    }

    #[test]
    fn daily_update_consumes_supplies() {
        let mut f = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        let before = f.supplies;
        f.daily_update();
        assert!(f.supplies < before);
    }

    #[test]
    fn recruit_requires_grievance() {
        let f = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        // Low grievance recruit should return zero chance
        assert_eq!(f.recruitment_chance(Fixed::from_f64(0.2), 10), Fixed::ZERO);
    }

    #[test]
    fn recruit_fails_at_capacity() {
        let f = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        // max_members=2, already has 2 members → zero chance
        assert_eq!(f.recruitment_chance(Fixed::from_f64(0.8), 2), Fixed::ZERO);
    }

    /// §5.2 (Iteration 89): `deactivate_all` marks every record inactive —
    /// the coup path dissolves all v1 factions, so each active v2 record
    /// (not just the revolting leader's) must stop reporting as active to
    /// preserve the v1↔v2 1:1 registration invariant.
    #[test]
    fn registry_deactivate_all_marks_every_record_inactive() {
        let mut reg = FactionV2Registry::new();
        reg.register(FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0));
        reg.register(FactionV2::new(1, vec![2, 3], Fixed::from_f64(0.7), 0));
        assert_eq!(reg.active().len(), 2, "both records start active");
        // Leader-only deactivation leaves the second record stale-active —
        // the exact asymmetry the coup path exposed.
        reg.deactivate_by_leader(0);
        assert_eq!(reg.active().len(), 1);
        reg.deactivate_all();
        assert_eq!(
            reg.active().len(),
            0,
            "deactivate_all must clear every record, not just the leader's"
        );
    }

    #[test]
    fn registry_daily_update_deactivates_dead_factions() {
        let mut reg = FactionV2Registry::new();
        let mut f = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        f.morale = Fixed::from_f64(0.05); // will dissolve
        reg.register(f);
        reg.daily_update();
        assert!(!reg.factions[0].active);
    }

    #[test]
    fn attachment_style_derives_modal_member_style() {
        use crate::psychology::attachment::AttachmentStyle::{Anxious, Secure};
        let mut f = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        f.derive_attachment_style(&[Secure, Anxious, Anxious]);
        assert_eq!(f.attachment_style, GroupAttachmentStyle::Anxious);
    }

    #[test]
    fn old_snapshots_default_to_secure_attachment_style() {
        // Pre-§12.3 snapshots carry no attachment_style field — serde default
        // must restore Secure (the neutral construction default).
        let json = r#"{"leader":0,"members":[0,1],"grievance":5000,"mobilization_capacity":4000,"morale":5000,"supplies":7000,"supply_consumption":100,"leader_competence":5000,"trust_in_leadership":6000,"cohesion":4000,"fragmentation":1000,"casualties":0,"legitimacy_of_violence":3000,"active":true,"formed_tick":0,"last_event_tick":0}"#;
        let f: FactionV2 = serde_json::from_str(json).unwrap();
        assert_eq!(f.attachment_style, GroupAttachmentStyle::Secure);
    }

    #[test]
    fn secure_factions_hold_cohesion_best() {
        let mut secure = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        let mut disorganized = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        secure.attachment_style = GroupAttachmentStyle::Secure;
        disorganized.attachment_style = GroupAttachmentStyle::Disorganized;
        secure.cohesion = Fixed::from_f64(0.9);
        disorganized.cohesion = Fixed::from_f64(0.9);
        for _ in 0..50 {
            secure.daily_update();
            disorganized.daily_update();
        }
        assert!(secure.cohesion > disorganized.cohesion);
        assert!(secure.cohesion > Fixed::from_f64(0.85));
    }

    #[test]
    fn anxious_factions_cling_to_leader_under_incompetence() {
        let mut anxious = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        let mut avoidant = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        anxious.attachment_style = GroupAttachmentStyle::Anxious;
        avoidant.attachment_style = GroupAttachmentStyle::Avoidant;
        anxious.leader_competence = Fixed::from_f64(0.2);
        avoidant.leader_competence = Fixed::from_f64(0.2);
        for _ in 0..10 {
            anxious.daily_update();
            avoidant.daily_update();
        }
        // Anxious clings (slower trust decay); avoidant distrusts dependence.
        assert!(anxious.trust_in_leadership > avoidant.trust_in_leadership);
    }

    #[test]
    fn avoidant_factions_fragment_more_under_low_cohesion() {
        let mut avoidant = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        let mut secure = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        avoidant.attachment_style = GroupAttachmentStyle::Avoidant;
        secure.attachment_style = GroupAttachmentStyle::Secure;
        avoidant.cohesion = Fixed::from_f64(0.3);
        secure.cohesion = Fixed::from_f64(0.3);
        for _ in 0..50 {
            avoidant.daily_update();
            secure.daily_update();
        }
        assert!(avoidant.fragmentation > secure.fragmentation);
    }

    #[test]
    fn anxious_factions_panic_harder_under_scarcity() {
        let mut anxious = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        let mut avoidant = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        anxious.attachment_style = GroupAttachmentStyle::Anxious;
        avoidant.attachment_style = GroupAttachmentStyle::Avoidant;
        anxious.morale = Fixed::from_f64(0.5);
        avoidant.morale = Fixed::from_f64(0.5);
        // Force both into scarcity immediately (below the 0.3 supply floor).
        anxious.supplies = Fixed::from_f64(0.2);
        avoidant.supplies = Fixed::from_f64(0.2);
        for _ in 0..10 {
            anxious.daily_update();
            avoidant.daily_update();
        }
        // Anxious panics harder under scarcity (1.5× penalty vs 0.5×).
        assert!(anxious.morale < avoidant.morale);
    }
}
