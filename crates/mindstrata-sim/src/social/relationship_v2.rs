//! Relationship V2 — enriched relational model with developmental stages.
//!
//! Replaces simple relationship edges with rich relational systems including:
//! - Core affect (trust, affection, respect, fear, obligation)
//! - Deep relational dimensions (intimacy, passion, commitment, solidarity, resentment)
//! - Identity and meaning (shared identity, role expectations)
//! - History (stage progression, betrayal/reconciliation history)
//! - Dynamics (decay, volatility, last interaction)

use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// Relationship stage classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RelationshipStage {
    Unnoticed,
    Noticed,
    Acquaintance,
    Familiar,
    Neighbor,
    Friend,
    CloseFriend,
    Confidant,
    Ally,
    Disliked,
    Rival,
    Enemy,
    Nemesis,
    Kin,
    ParentChild,
    Sibling,
    Cousin,
    InLaw,
    /// §10.3 (AP2): Grandparent/grandchild (or further linear ancestor) ties.
    AncestorDescendant,
    // §10.3 (AP2): Authority branch — assigned from structural relationships
    // (patronage, apprenticeship, cult leadership, household headship), not
    // advanced by the social progression machinery.
    /// Patron/client — protection for support (patronage registry).
    PatronClient,
    /// Lord/vassal — fealty for land/security (no live producer yet — reserved).
    LordVassal,
    /// Master/apprentice — knowledge transmission (education learning events).
    MasterApprentice,
    /// Priest/layperson — spiritual leadership (cult charismatic leader).
    PriestLayperson,
    /// Elder/junior — seniority within a household (household head).
    ElderJunior,
    /// Guard/citizen — protection duty (no live producer yet — reserved).
    GuardCitizen,
}

impl RelationshipStage {
    pub fn next_positive(&self) -> Option<Self> {
        match self {
            Self::Unnoticed => Some(Self::Noticed),
            Self::Noticed => Some(Self::Acquaintance),
            Self::Acquaintance => Some(Self::Familiar),
            Self::Familiar => Some(Self::Neighbor),
            Self::Neighbor => Some(Self::Friend),
            Self::Friend => Some(Self::CloseFriend),
            Self::CloseFriend => Some(Self::Confidant),
            Self::Confidant => Some(Self::Ally),
            _ => None,
        }
    }

    pub fn next_negative(&self) -> Option<Self> {
        match self {
            Self::Neighbor => Some(Self::Disliked),
            Self::Disliked => Some(Self::Rival),
            Self::Rival => Some(Self::Enemy),
            Self::Enemy => Some(Self::Nemesis),
            _ => None,
        }
    }

    pub fn base_trust(&self) -> Fixed {
        match self {
            Self::Unnoticed => Fixed::ZERO,
            Self::Noticed => Fixed::from_f64(0.2),
            Self::Acquaintance => Fixed::from_f64(0.35),
            Self::Familiar => Fixed::from_f64(0.5),
            Self::Neighbor => Fixed::from_f64(0.55),
            Self::Friend => Fixed::from_f64(0.65),
            Self::CloseFriend => Fixed::from_f64(0.8),
            Self::Confidant => Fixed::from_f64(0.9),
            Self::Ally => Fixed::from_f64(0.85),
            Self::Disliked => Fixed::from_f64(0.2),
            Self::Rival => Fixed::from_f64(0.1),
            Self::Enemy => Fixed::from_f64(0.05),
            Self::Nemesis => Fixed::ZERO,
            Self::Kin => Fixed::from_f64(0.6),
            Self::ParentChild => Fixed::from_f64(0.8),
            Self::Sibling => Fixed::from_f64(0.7),
            Self::Cousin => Fixed::from_f64(0.5),
            Self::InLaw => Fixed::from_f64(0.45),
            Self::AncestorDescendant => Fixed::from_f64(0.45),
            // §10.3 authority branch: trust baselines for authority ties.
            Self::PatronClient => Fixed::from_f64(0.6),
            Self::LordVassal => Fixed::from_f64(0.6),
            Self::MasterApprentice => Fixed::from_f64(0.55),
            Self::PriestLayperson => Fixed::from_f64(0.55),
            Self::ElderJunior => Fixed::from_f64(0.5),
            Self::GuardCitizen => Fixed::from_f64(0.5),
        }
    }
}

/// Threshold for progressing from one stage to the next.
pub fn stage_progression_threshold(stage: RelationshipStage) -> Fixed {
    match stage {
        RelationshipStage::Unnoticed => Fixed::from_f64(0.3),
        RelationshipStage::Noticed => Fixed::from_f64(0.4),
        RelationshipStage::Acquaintance => Fixed::from_f64(0.5),
        RelationshipStage::Familiar => Fixed::from_f64(0.6),
        RelationshipStage::Neighbor => Fixed::from_f64(0.7),
        RelationshipStage::Friend => Fixed::from_f64(0.8),
        RelationshipStage::CloseFriend => Fixed::from_f64(0.85),
        RelationshipStage::Confidant => Fixed::from_f64(0.9),
        _ => Fixed::ONE,
    }
}

/// §10.2 (AP2): Public relationship label — the §10.3 taxonomy.
///
/// General social stages, the negative branch, and the kin branch map 1:1
/// onto [`RelationshipStage`]; the authority branch (patron/client, lord/
/// vassal, master/apprentice, priest/layperson, elder/junior, guard/citizen)
/// is derived from institutional role expectations. The public label is what
/// the community sees; the private label (see [`RelationshipV2::derive_private_label`])
/// is the emotionally honest view and can diverge from it.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum RelationshipLabel {
    #[default]
    Unnoticed,
    Noticed,
    Acquaintance,
    Familiar,
    Neighbor,
    Friend,
    CloseFriend,
    Confidant,
    Ally,
    Disliked,
    Rival,
    Enemy,
    Nemesis,
    Kin,
    ParentChild,
    Sibling,
    Cousin,
    InLaw,
    AncestorDescendant,
    PatronClient,
    LordVassal,
    MasterApprentice,
    PriestLayperson,
    ElderJunior,
    GuardCitizen,
}

/// §10.2 (AP2): Expected role in this relationship — the §10.3 authority
/// branch. Derived from the relationship stage and structural links (kin vs
/// authority vs peer). `None` when no asymmetric role is expected.
///
/// Only `None`/`Caregiver`/`Ally` are currently derived (kin stages); the
/// authority variants (patron/client, lord/vassal, etc.) are reserved
/// taxonomy for future institutional-role wiring — not live behavior yet.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum RoleExpectation {
    #[default]
    None,
    PatronClient,
    LordVassal,
    MasterApprentice,
    PriestLayperson,
    ElderJunior,
    GuardCitizen,
    Provider,
    Caregiver,
    Protector,
    Ally,
}

/// §10.2 (AP2): Kind of betrayal recorded in a relationship's history.
///
/// Only `Violence` is currently constructed in production (the sim conflict
/// path); the remaining variants are reserved taxonomy for future betrayal
/// mechanics (infidelity, abandonment, theft, deception, broken vows,
/// disloyalty) — not live behavior yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BetrayalKind {
    Infidelity,
    Abandonment,
    Theft,
    Deception,
    Violence,
    BrokenVow,
    Disloyalty,
}

/// §10.2 (AP2): A recorded betrayal event (bounded history).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BetrayalEvent {
    pub tick: u64,
    pub kind: BetrayalKind,
    pub magnitude: Fixed,
}

/// §10.2 (AP2): Kind of reconciliation that repaired a relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ReconciliationKind {
    Apology,
    Reparation,
    Mediation,
    SharedRitual,
    PublicAmends,
}

/// §10.2 (AP2): A recorded reconciliation event (bounded history).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReconciliationEvent {
    pub tick: u64,
    pub kind: ReconciliationKind,
    pub magnitude: Fixed,
}

/// A rich directed relationship between two agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipV2 {
    pub from: AgentId,
    pub to: AgentId,
    pub trust: Fixed,
    pub affection: Fixed,
    pub respect: Fixed,
    pub fear: Fixed,
    pub obligation: Fixed,
    pub intimacy: Fixed,
    pub passion: Fixed,
    pub commitment: Fixed,
    pub attachment_security: Fixed,
    pub dependence: Fixed,
    pub power_balance: Fixed,
    pub solidarity: Fixed,
    pub resentment: Fixed,
    pub admiration: Fixed,
    pub jealousy: Fixed,
    pub gratitude: Fixed,
    pub guilt_toward: Fixed,
    pub moral_debt: Fixed,
    pub shared_identity: Fixed,
    pub stage: RelationshipStage,
    pub stage_progress: Fixed,
    pub interaction_count: u32,
    pub last_positive_tick: u64,
    pub last_negative_tick: u64,
    pub decay_rate: Fixed,
    pub volatility: Fixed,

    // §17.3: Relationship caching — skip full updates for dormant relationships.
    // Updated on every interaction; daily decay pass updates all stale ones.
    pub last_update_tick: u64,
    /// Whether this relationship has pending changes that need processing.
    pub dirty: bool,

    // ── §10.2 (AP2): Identity and meaning ─────────────────────────────
    /// Expected role in this relationship (authority/kin branch of §10.3).
    #[serde(default)]
    pub role_expectation: RoleExpectation,
    /// Public label — what the community sees (§10.3 taxonomy).
    #[serde(default)]
    pub public_label: RelationshipLabel,
    /// Private label — the emotionally honest view, which can diverge from
    /// the public label (resentment downgrades it toward the negative branch).
    #[serde(default)]
    pub private_label: RelationshipLabel,

    // ── §10.2 (AP2): Structure ────────────────────────────────────────
    /// Kinship closeness (0 = stranger, 1 = immediate family). Derived from
    /// the kin stage branch.
    #[serde(default)]
    pub kinship_coefficient: Fixed,
    /// Shared household index, when both agents co-reside.
    #[serde(default)]
    pub household_link: Option<usize>,
    /// Shared faction index, when both belong to the same faction.
    #[serde(default)]
    pub faction_link: Option<usize>,
    /// Shared institution id, when both hold membership in the same body.
    #[serde(default)]
    pub institutional_link: Option<u64>,

    // ── §10.2 (AP2): History ──────────────────────────────────────────
    /// Accumulated weight of positive history (0–1).
    #[serde(default)]
    pub positive_memory_weight: Fixed,
    /// Accumulated weight of negative history (0–1).
    #[serde(default)]
    pub negative_memory_weight: Fixed,
    /// Bounded log of betrayal events (most recent 16).
    #[serde(default)]
    pub betrayal_history: Vec<BetrayalEvent>,
    /// Bounded log of reconciliation events (most recent 16).
    #[serde(default)]
    pub reconciliation_history: Vec<ReconciliationEvent>,
}

impl RelationshipV2 {
    pub fn new(from: AgentId, to: AgentId) -> Self {
        Self {
            from,
            to,
            trust: Fixed::from_f64(0.4),
            affection: Fixed::from_f64(0.3),
            respect: Fixed::from_f64(0.3),
            fear: Fixed::ZERO,
            obligation: Fixed::ZERO,
            intimacy: Fixed::ZERO,
            passion: Fixed::ZERO,
            commitment: Fixed::ZERO,
            attachment_security: Fixed::from_f64(0.3),
            dependence: Fixed::ZERO,
            power_balance: Fixed::ZERO,
            solidarity: Fixed::ZERO,
            resentment: Fixed::ZERO,
            admiration: Fixed::ZERO,
            jealousy: Fixed::ZERO,
            gratitude: Fixed::ZERO,
            guilt_toward: Fixed::ZERO,
            moral_debt: Fixed::ZERO,
            shared_identity: Fixed::ZERO,
            stage: RelationshipStage::Unnoticed,
            stage_progress: Fixed::ZERO,
            interaction_count: 0,
            last_positive_tick: 0,
            last_negative_tick: 0,
            // P5 audit (Iteration 184): recalibrated from 0.0005. The
            // per-tick dirty-window decay (0.0005 × ~72-tick average window
            // ≈ 0.036/day) previously exceeded the now-live interaction
            // gains (9.5 interactions/pair/day × 0.003 ≈ 0.028/day), so
            // every actively-interacting pair collapsed toward Nemesis.
            // At 0.0002 the window decay (~0.014/day) sits below the
            // measured gain — pairs interacting ~10×/day grow, sparse-
            // contact pairs still drift down. Dormant-boundary decay
            // (0.0002/day) remains a slow erosion.
            decay_rate: Fixed::from_f64(0.0002),
            volatility: Fixed::from_f64(0.5),
            last_update_tick: 0,
            dirty: false,
            role_expectation: RoleExpectation::None,
            public_label: RelationshipLabel::default(),
            private_label: RelationshipLabel::default(),
            kinship_coefficient: Fixed::ZERO,
            household_link: None,
            faction_link: None,
            institutional_link: None,
            positive_memory_weight: Fixed::ZERO,
            negative_memory_weight: Fixed::ZERO,
            betrayal_history: Vec::new(),
            reconciliation_history: Vec::new(),
        }
    }

    pub fn quality(&self) -> Fixed {
        (self.trust * Fixed::from_f64(0.3)
            + self.affection * Fixed::from_f64(0.25)
            + self.respect * Fixed::from_f64(0.2)
            + self.intimacy * Fixed::from_f64(0.15)
            + self.commitment * Fixed::from_f64(0.1))
        .clamp_01()
    }

    /// §11.2: Recompute the power balance of this directed relationship from
    /// B's dependence on A (commitment/attachment), A's status advantage over
    /// B, B's fear of A, and the moral obligations B owes A.
    ///
    /// `power_balance` was declared but never written (the dead-field class
    /// Iteration 39 closes). Positive = A dominates B; negative = B dominates A.
    pub fn update_power_balance(
        &mut self,
        dependence_a_on_b: Fixed,
        commitment_b_to_a: Fixed,
        attachment_b_to_a: Fixed,
        status_advantage: Fixed,
        fear_b_of_a: Fixed,
        obligation_b_to_a: Fixed,
        moral_debt_b_to_a: Fixed,
    ) {
        let power = crate::social::relational_power::RelationalPower::compute(
            dependence_a_on_b,
            commitment_b_to_a,
            attachment_b_to_a,
            status_advantage,
            fear_b_of_a,
            obligation_b_to_a,
            moral_debt_b_to_a,
        );
        self.power_balance = power.power_balance();
    }

    pub fn record_positive(&mut self, tick: u64, magnitude: Fixed) {
        let vol = self.volatility;
        self.trust = (self.trust + magnitude * vol * Fixed::from_f64(0.02)).clamp_01();
        self.affection = (self.affection + magnitude * vol * Fixed::from_f64(0.015)).clamp_01();
        self.gratitude = (self.gratitude + magnitude * Fixed::from_f64(0.01)).clamp_01();
        self.resentment = (self.resentment - magnitude * Fixed::from_f64(0.005)).max(Fixed::ZERO);
        // §10.2 (P5 audit, Iteration 184): intimacy and commitment were
        // declared but NEVER produced — record_positive had zero production
        // call sites and the probe pinned intimacy at 0.000 in every window.
        // Positive interactions now build closeness (intimacy) and
        // persistence (commitment) at a slower rate than trust.
        self.intimacy = (self.intimacy + magnitude * vol * Fixed::from_f64(0.01)).clamp_01();
        self.commitment =
            (self.commitment + magnitude * vol * Fixed::from_f64(0.008)).clamp_01();
        // §10.2: positive history accumulates into the memory weight.
        self.positive_memory_weight =
            (self.positive_memory_weight + magnitude * Fixed::from_f64(0.01)).clamp_01();
        self.last_positive_tick = tick;
        self.last_update_tick = tick;
        self.dirty = true;
        self.interaction_count += 1;
    }

    pub fn record_negative(&mut self, tick: u64, magnitude: Fixed) {
        let vol = self.volatility;
        self.trust = (self.trust - magnitude * vol * Fixed::from_f64(0.03)).max(Fixed::ZERO);
        self.affection =
            (self.affection - magnitude * vol * Fixed::from_f64(0.02)).max(Fixed::ZERO);
        self.resentment = (self.resentment + magnitude * Fixed::from_f64(0.02)).clamp_01();
        self.fear = (self.fear + magnitude * Fixed::from_f64(0.01)).clamp_01();
        // §10.2 (P5 audit, Iteration 184): hostility erodes closeness and
        // persistence — mirrors the positive-side intimacy/commitment gains.
        self.intimacy = (self.intimacy - magnitude * vol * Fixed::from_f64(0.015)).max(Fixed::ZERO);
        self.commitment =
            (self.commitment - magnitude * vol * Fixed::from_f64(0.012)).max(Fixed::ZERO);
        // §10.2: negative history accumulates into the memory weight.
        self.negative_memory_weight =
            (self.negative_memory_weight + magnitude * Fixed::from_f64(0.01)).clamp_01();
        self.last_negative_tick = tick;
        self.last_update_tick = tick;
        self.dirty = true;
        self.interaction_count += 1;
    }

    /// §10.2: Public label derived deterministically from the current stage
    /// (§10.3 taxonomy — general, negative, and kin branches map 1:1).
    pub fn derive_public_label(&self) -> RelationshipLabel {
        match self.stage {
            RelationshipStage::Unnoticed => RelationshipLabel::Unnoticed,
            RelationshipStage::Noticed => RelationshipLabel::Noticed,
            RelationshipStage::Acquaintance => RelationshipLabel::Acquaintance,
            RelationshipStage::Familiar => RelationshipLabel::Familiar,
            RelationshipStage::Neighbor => RelationshipLabel::Neighbor,
            RelationshipStage::Friend => RelationshipLabel::Friend,
            RelationshipStage::CloseFriend => RelationshipLabel::CloseFriend,
            RelationshipStage::Confidant => RelationshipLabel::Confidant,
            RelationshipStage::Ally => RelationshipLabel::Ally,
            RelationshipStage::Disliked => RelationshipLabel::Disliked,
            RelationshipStage::Rival => RelationshipLabel::Rival,
            RelationshipStage::Enemy => RelationshipLabel::Enemy,
            RelationshipStage::Nemesis => RelationshipLabel::Nemesis,
            RelationshipStage::Kin => RelationshipLabel::Kin,
            RelationshipStage::ParentChild => RelationshipLabel::ParentChild,
            RelationshipStage::Sibling => RelationshipLabel::Sibling,
            RelationshipStage::Cousin => RelationshipLabel::Cousin,
            RelationshipStage::InLaw => RelationshipLabel::InLaw,
            RelationshipStage::AncestorDescendant => RelationshipLabel::AncestorDescendant,
            // §10.3 authority branch: each assigned stage carries its label.
            RelationshipStage::PatronClient => RelationshipLabel::PatronClient,
            RelationshipStage::LordVassal => RelationshipLabel::LordVassal,
            RelationshipStage::MasterApprentice => RelationshipLabel::MasterApprentice,
            RelationshipStage::PriestLayperson => RelationshipLabel::PriestLayperson,
            RelationshipStage::ElderJunior => RelationshipLabel::ElderJunior,
            RelationshipStage::GuardCitizen => RelationshipLabel::GuardCitizen,
        }
    }

    /// §10.2: Private label — the emotionally honest view. Starts from the
    /// public label, then resentment dominating trust downgrades it toward
    /// the negative branch (a "friend" the agent privately resents is labeled
    /// Disliked; a Rival becomes an Enemy). Deterministic, no RNG.
    ///
    /// §11.2: *being dominated* feeds the same channel — an agent who holds
    /// the weaker hand in an asymmetric relationship privately resents the
    /// subordination even when the public stage looks friendly. A smooth ramp
    /// (no threshold cliff) maps each unit of domination (negative
    /// `power_balance`) onto 0.5 units of effective resentment, so the
    /// divergence is continuous in the underlying power differential.
    pub fn derive_private_label(&self) -> RelationshipLabel {
        let public = self.derive_public_label();
        // §11.2: domination (power_balance < 0 ⇒ the other side holds the
        // leverage) feeds the resentment channel that drives the downgrade.
        let domination = (Fixed::ZERO - self.power_balance).max(Fixed::ZERO);
        let effective_resentment = (self.resentment + domination * Fixed::from_f64(0.5)).clamp_01();
        if effective_resentment > self.trust && effective_resentment > Fixed::from_f64(0.4) {
            match public {
                RelationshipLabel::Unnoticed
                | RelationshipLabel::Noticed
                | RelationshipLabel::Acquaintance
                | RelationshipLabel::Familiar
                | RelationshipLabel::Neighbor
                | RelationshipLabel::Friend
                | RelationshipLabel::CloseFriend
                | RelationshipLabel::Confidant
                | RelationshipLabel::Ally
                | RelationshipLabel::Kin => RelationshipLabel::Disliked,
                RelationshipLabel::Disliked => RelationshipLabel::Rival,
                RelationshipLabel::Rival => RelationshipLabel::Enemy,
                RelationshipLabel::Enemy => RelationshipLabel::Nemesis,
                other => other,
            }
        } else {
            public
        }
    }

    /// §10.2: Role expectation from the §10.3 authority/kin branch — kin
    /// stages imply caregiving/alliance roles; peers expect none.
    pub fn derive_role_expectation(&self) -> RoleExpectation {
        match self.stage {
            RelationshipStage::ParentChild => RoleExpectation::Caregiver,
            RelationshipStage::AncestorDescendant => RoleExpectation::Caregiver,
            RelationshipStage::Sibling => RoleExpectation::Ally,
            RelationshipStage::Kin | RelationshipStage::Cousin | RelationshipStage::InLaw => {
                RoleExpectation::Ally
            }
            // §10.3 authority branch: each stage maps onto its role expectation.
            RelationshipStage::PatronClient => RoleExpectation::PatronClient,
            RelationshipStage::LordVassal => RoleExpectation::LordVassal,
            RelationshipStage::MasterApprentice => RoleExpectation::MasterApprentice,
            RelationshipStage::PriestLayperson => RoleExpectation::PriestLayperson,
            RelationshipStage::ElderJunior => RoleExpectation::ElderJunior,
            RelationshipStage::GuardCitizen => RoleExpectation::GuardCitizen,
            _ => RoleExpectation::None,
        }
    }

    /// §10.2: Kinship coefficient — blood closeness implied by the kin stage
    /// branch (0 = stranger, 1 = immediate family).
    pub fn derive_kinship_coefficient(&self) -> Fixed {
        match self.stage {
            RelationshipStage::ParentChild => Fixed::from_f64(0.8),
            RelationshipStage::Sibling => Fixed::from_f64(0.5),
            RelationshipStage::Kin => Fixed::from_f64(0.5),
            RelationshipStage::Cousin => Fixed::from_f64(0.25),
            RelationshipStage::InLaw => Fixed::from_f64(0.3),
            RelationshipStage::AncestorDescendant => Fixed::from_f64(0.25),
            _ => Fixed::ZERO,
        }
    }

    /// §10.2: Refresh the identity layer (public/private labels, role
    /// expectation, kinship coefficient) from current state. Deterministic —
    /// a pure function of existing fields, so calibrated runs stay
    /// byte-identical.
    pub fn update_identity_metadata(&mut self) {
        self.public_label = self.derive_public_label();
        self.private_label = self.derive_private_label();
        self.role_expectation = self.derive_role_expectation();
        self.kinship_coefficient = self.derive_kinship_coefficient();
    }

    /// §10.2: Record a betrayal event into the bounded history and raise the
    /// negative memory weight. Only touches the new §10.2 fields — the trust/
    /// affection damage is applied by the conflict system that calls this, so
    /// calibrated runs stay byte-identical.
    pub fn record_betrayal(&mut self, tick: u64, kind: BetrayalKind, magnitude: Fixed) {
        const MAX_BETRAYALS: usize = 16;
        if self.betrayal_history.len() >= MAX_BETRAYALS {
            self.betrayal_history.remove(0);
        }
        self.betrayal_history.push(BetrayalEvent {
            tick,
            kind,
            magnitude,
        });
        self.negative_memory_weight = (self.negative_memory_weight + magnitude).clamp_01();
    }

    /// §10.2: Record a reconciliation event into the bounded history and
    /// raise the positive memory weight. Only touches new §10.2 fields.
    pub fn record_reconciliation(&mut self, tick: u64, kind: ReconciliationKind, magnitude: Fixed) {
        const MAX_RECONCILIATIONS: usize = 16;
        if self.reconciliation_history.len() >= MAX_RECONCILIATIONS {
            self.reconciliation_history.remove(0);
        }
        self.reconciliation_history.push(ReconciliationEvent {
            tick,
            kind,
            magnitude,
        });
        self.positive_memory_weight = (self.positive_memory_weight + magnitude).clamp_01();
    }

    pub fn decay(&mut self, ticks_elapsed: u64) {
        let decay = self.decay_rate * Fixed::from_int(ticks_elapsed as i64);
        self.trust = (self.trust - decay).max(Fixed::ZERO);
        self.affection = (self.affection - decay * Fixed::from_f64(0.5)).max(Fixed::ZERO);
        self.intimacy = (self.intimacy - decay * Fixed::from_f64(0.3)).max(Fixed::ZERO);
        // NOTE: dirty is NOT set here — decay is passive, not an interaction.
        // dirty is only set by record_positive/record_negative (active interactions).
    }

    /// §17.3: Check if this relationship needs decay this tick.
    ///
    /// Relationships are "dormant" when no interaction, rumor, or emotional
    /// event has touched them recently. Dormant relationships only receive
    /// the daily decay pass (every 144 ticks). Active (dirty) relationships
    /// decay every tick until cleared.
    pub fn is_active_this_tick(&self, current_tick: u64) -> bool {
        // Active if recently interacted with (dirty)
        if self.dirty {
            return true;
        }
        // Also active on daily boundary for the bulk decay pass
        current_tick > 0 && current_tick.is_multiple_of(144)
    }

    /// §17.3: Clear dirty flag after processing.
    pub fn clear_dirty(&mut self, tick: u64) {
        self.dirty = false;
        self.last_update_tick = tick;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_computed() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        r.trust = Fixed::from_f64(0.8);
        r.affection = Fixed::from_f64(0.7);
        r.respect = Fixed::from_f64(0.6);
        r.intimacy = Fixed::from_f64(0.5);
        r.commitment = Fixed::from_f64(0.4);
        let q = r.quality();
        assert!(q > Fixed::from_f64(0.4));
    }

    #[test]
    fn positive_increases_trust() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        let initial = r.trust;
        r.record_positive(0, Fixed::from_f64(0.5));
        assert!(r.trust > initial);
    }

    #[test]
    fn negative_decreases_trust() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        r.trust = Fixed::from_f64(0.8);
        r.record_negative(0, Fixed::from_f64(0.5));
        assert!(r.trust < Fixed::from_f64(0.8));
    }

    #[test]
    fn stage_progression() {
        let next = RelationshipStage::Unnoticed.next_positive().unwrap();
        assert_eq!(next, RelationshipStage::Noticed);
    }

    // ── §17.3 Relationship Caching Tests ──────────────────────────────

    #[test]
    fn decay_does_not_set_dirty() {
        // §17.3: decay() is passive — it must NOT set the dirty flag.
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        assert!(!r.dirty);
        r.decay(1);
        assert!(!r.dirty, "decay() must not set dirty flag");
    }

    #[test]
    fn record_positive_sets_dirty() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        assert!(!r.dirty);
        r.record_positive(10, Fixed::from_f64(0.5));
        assert!(r.dirty, "record_positive must set dirty");
        assert_eq!(r.last_update_tick, 10);
    }

    #[test]
    fn record_negative_sets_dirty() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        assert!(!r.dirty);
        r.record_negative(20, Fixed::from_f64(0.5));
        assert!(r.dirty, "record_negative must set dirty");
        assert_eq!(r.last_update_tick, 20);
    }

    #[test]
    fn dormant_relationship_skips_decay() {
        // §17.3: A non-dirty relationship at a non-daily-boundary tick
        // should NOT be active (i.e., should skip decay).
        let r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        // Tick 50 is neither dirty nor a daily boundary (144)
        assert!(
            !r.is_active_this_tick(50),
            "Dormant relationship should not be active at non-daily tick"
        );
    }

    #[test]
    fn daily_boundary_activates_all() {
        // §17.3: On daily boundary (tick % 144 == 0), all relationships
        // should be active for the bulk decay pass.
        let r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        assert!(
            r.is_active_this_tick(144),
            "Daily boundary should activate all relationships"
        );
        assert!(
            r.is_active_this_tick(288),
            "Daily boundary should activate all relationships"
        );
    }

    #[test]
    fn dirty_relationship_stays_active_until_cleared() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        r.record_positive(5, Fixed::from_f64(0.5));
        // At tick 100 (non-daily), still active because dirty
        assert!(
            r.is_active_this_tick(100),
            "Dirty relationship should be active at any tick"
        );
        // Clear dirty
        r.clear_dirty(100);
        assert!(
            !r.is_active_this_tick(100),
            "After clear_dirty, relationship should be dormant"
        );
    }

    #[test]
    fn clear_dirty_resets_flag() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        r.record_positive(5, Fixed::from_f64(0.5));
        assert!(r.dirty);
        r.clear_dirty(10);
        assert!(!r.dirty);
        assert_eq!(r.last_update_tick, 10);
    }

    // ── §10.2 Relationship Identity + History Tests ───────────────────

    #[test]
    fn new_initializes_section_10_2_fields() {
        let r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        assert_eq!(r.public_label, RelationshipLabel::Unnoticed);
        assert_eq!(r.private_label, RelationshipLabel::Unnoticed);
        assert_eq!(r.role_expectation, RoleExpectation::None);
        assert_eq!(r.kinship_coefficient, Fixed::ZERO);
        assert_eq!(r.household_link, None);
        assert_eq!(r.faction_link, None);
        assert_eq!(r.institutional_link, None);
        assert!(r.betrayal_history.is_empty());
        assert!(r.reconciliation_history.is_empty());
    }

    #[test]
    fn public_label_follows_stage() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        r.stage = RelationshipStage::Friend;
        assert_eq!(r.derive_public_label(), RelationshipLabel::Friend);
        r.stage = RelationshipStage::Enemy;
        assert_eq!(r.derive_public_label(), RelationshipLabel::Enemy);
        r.stage = RelationshipStage::Sibling;
        assert_eq!(r.derive_public_label(), RelationshipLabel::Sibling);
    }

    #[test]
    fn private_label_downgrades_under_resentment() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        // Public: Friend. Private: also Friend while resentment is low.
        r.stage = RelationshipStage::Friend;
        r.trust = Fixed::from_f64(0.7);
        r.resentment = Fixed::from_f64(0.1);
        assert_eq!(r.derive_private_label(), RelationshipLabel::Friend);
        // Resentment dominates trust → the private view downgrades.
        r.trust = Fixed::from_f64(0.2);
        r.resentment = Fixed::from_f64(0.6);
        assert_eq!(r.derive_private_label(), RelationshipLabel::Disliked);
        // A public Rival with high resentment becomes privately an Enemy.
        r.stage = RelationshipStage::Rival;
        assert_eq!(r.derive_private_label(), RelationshipLabel::Enemy);
    }

    #[test]
    fn private_label_downgrades_under_domination() {
        // §11.2: a dominated agent privately resents the subordination even
        // with zero overt resentment — the honest view diverges from public.
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        r.stage = RelationshipStage::Friend;
        r.trust = Fixed::from_f64(0.3);
        // Balanced power → the private view matches the public one.
        r.power_balance = Fixed::ZERO;
        assert_eq!(r.derive_private_label(), RelationshipLabel::Friend);
        // Strong domination (pb = -0.9 → effective resentment 0.45, above the
        // 0.4 threshold and above trust) downgrades even with zero resentment.
        r.power_balance = Fixed::from_f64(-0.9);
        assert_eq!(r.derive_private_label(), RelationshipLabel::Disliked);
        // Moderate domination (effective resentment 0.25) stays public — the
        // ramp is continuous, not a threshold cliff.
        r.power_balance = Fixed::from_f64(-0.5);
        assert_eq!(r.derive_private_label(), RelationshipLabel::Friend);
    }

    #[test]
    fn power_balance_fills_directed_leverage_and_drives_label() {
        // A depends on B heavily, B barely on A, B holds the status edge → A
        // is dominated (negative balance); the mirror relationship is positive.
        let mut subordinate = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        subordinate.update_power_balance(
            Fixed::from_f64(0.95),
            Fixed::from_f64(0.05),
            Fixed::from_f64(0.05),
            Fixed::from_f64(-1.0),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
        );
        assert!(subordinate.power_balance < Fixed::ZERO);
        assert!(subordinate.power_balance > Fixed::from_f64(-1.0));

        let mut dominant = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        dominant.update_power_balance(
            Fixed::from_f64(0.1),
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.5),
        );
        assert!(dominant.power_balance > Fixed::from_f64(0.5));

        // The dominated view diverges from the public Friend; the dominant
        // view matches it.
        subordinate.stage = RelationshipStage::Friend;
        subordinate.trust = Fixed::from_f64(0.3);
        dominant.stage = RelationshipStage::Friend;
        dominant.trust = Fixed::from_f64(0.3);
        assert_eq!(
            subordinate.derive_private_label(),
            RelationshipLabel::Disliked
        );
        assert_eq!(dominant.derive_private_label(), RelationshipLabel::Friend);
    }

    #[test]
    fn role_expectation_derived_from_kin_stage() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        r.stage = RelationshipStage::ParentChild;
        assert_eq!(r.derive_role_expectation(), RoleExpectation::Caregiver);
        r.stage = RelationshipStage::Sibling;
        assert_eq!(r.derive_role_expectation(), RoleExpectation::Ally);
        r.stage = RelationshipStage::Friend;
        assert_eq!(r.derive_role_expectation(), RoleExpectation::None);
    }

    #[test]
    fn kinship_coefficient_derived_from_kin_stage() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        r.stage = RelationshipStage::ParentChild;
        assert!(r.derive_kinship_coefficient() > Fixed::from_f64(0.7));
        r.stage = RelationshipStage::Cousin;
        assert!(r.derive_kinship_coefficient() > Fixed::ZERO);
        r.stage = RelationshipStage::Friend;
        assert_eq!(r.derive_kinship_coefficient(), Fixed::ZERO);
    }

    #[test]
    fn update_identity_metadata_fills_all_fields() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        r.stage = RelationshipStage::Friend;
        r.update_identity_metadata();
        assert_eq!(r.public_label, RelationshipLabel::Friend);
        assert_eq!(r.private_label, RelationshipLabel::Friend);
        assert_eq!(r.role_expectation, RoleExpectation::None);
        assert_eq!(r.kinship_coefficient, Fixed::ZERO);
    }

    #[test]
    fn record_betrayal_appends_and_raises_negative_weight() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        r.record_betrayal(10, BetrayalKind::Violence, Fixed::from_f64(0.3));
        assert_eq!(r.betrayal_history.len(), 1);
        assert_eq!(r.betrayal_history[0].kind, BetrayalKind::Violence);
        assert_eq!(r.betrayal_history[0].tick, 10);
        assert!(r.negative_memory_weight > Fixed::ZERO);
        // Does not touch trust (conflict system applies the damage).
        assert_eq!(r.trust, Fixed::from_f64(0.4));
    }

    #[test]
    fn record_reconciliation_appends_and_raises_positive_weight() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        r.record_reconciliation(20, ReconciliationKind::Apology, Fixed::from_f64(0.5));
        assert_eq!(r.reconciliation_history.len(), 1);
        assert_eq!(
            r.reconciliation_history[0].kind,
            ReconciliationKind::Apology
        );
        assert_eq!(r.reconciliation_history[0].tick, 20);
        assert!(r.positive_memory_weight > Fixed::ZERO);
    }

    #[test]
    fn histories_are_bounded() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        for t in 0..40u64 {
            r.record_betrayal(t, BetrayalKind::Theft, Fixed::from_f64(0.1));
            r.record_reconciliation(t, ReconciliationKind::Mediation, Fixed::from_f64(0.1));
        }
        assert_eq!(
            r.betrayal_history.len(),
            16,
            "betrayal history capped at 16"
        );
        assert_eq!(
            r.reconciliation_history.len(),
            16,
            "reconciliation history capped at 16"
        );
        // The most recent event survives (FIFO eviction).
        assert_eq!(r.betrayal_history.last().map(|e| e.tick), Some(39));
    }
}
