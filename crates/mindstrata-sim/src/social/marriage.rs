//! Marriage system — PairBond, courtship stages, marriage as institution.
//!
//! Architecture §10.4-§10.5: Marriage is not just a relationship label.
//! It is an institution that affects inheritance, household economics,
//! faction alliances, legitimacy of children, status, sexual norms,
//! religious standing, and kin networks.
//!
//! ```text
//! Romantic Stages:
//!   Awareness → Attraction → Flirtation → Courtship → Exclusivity
//!   → Betrothal → Marriage → Household Formation → Parenthood
//!   → Stabilization/Strain → Reconciliation/Separation/Widowhood
//! ```
//!
//! Marriage dissolution: death, abandonment, annulment, divorce, exile, religious sanction.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Stage of a romantic relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RomanticStage {
    /// No awareness of each other.
    Unaware,
    /// Aware of each other's existence.
    Awareness,
    /// Initial attraction detected.
    Attraction,
    /// Subtle signaling of interest.
    Flirtation,
    /// Active courtship — pursuing the relationship.
    ActiveCourtship,
    /// Testing whether interest is mutual.
    TestingReciprocity,
    /// Social validation — gossip, community awareness.
    SocialValidation,
    /// Mutually agreed exclusivity.
    Exclusivity,
    /// Formal betrothal / engagement.
    Betrothal,
    /// Married / formally bonded.
    Married,
    /// Household formed around the couple.
    HouseholdFormed,
    /// Children present — parenthood phase.
    Parenthood,
    /// Stable partnership with strain accumulating.
    Stabilization,
    /// High strain — reconciliation or separation possible.
    Strain,
}

impl RomanticStage {
    /// Advance to the next stage if conditions are met.
    pub fn next_positive(&self) -> Option<Self> {
        match self {
            Self::Unaware => Some(Self::Awareness),
            Self::Awareness => Some(Self::Attraction),
            Self::Attraction => Some(Self::Flirtation),
            Self::Flirtation => Some(Self::ActiveCourtship),
            Self::ActiveCourtship => Some(Self::TestingReciprocity),
            Self::TestingReciprocity => Some(Self::SocialValidation),
            Self::SocialValidation => Some(Self::Exclusivity),
            Self::Exclusivity => Some(Self::Betrothal),
            Self::Betrothal => Some(Self::Married),
            Self::Married => Some(Self::HouseholdFormed),
            Self::HouseholdFormed => Some(Self::Parenthood),
            Self::Parenthood => Some(Self::Stabilization),
            Self::Stabilization => Some(Self::Strain),
            Self::Strain => None, // terminal — branches to reconciliation or separation
        }
    }

    /// Get the base trust for this stage.
    pub fn base_trust(&self) -> Fixed {
        match self {
            Self::Unaware => Fixed::ZERO,
            Self::Awareness => Fixed::from_f64(0.1),
            Self::Attraction => Fixed::from_f64(0.2),
            Self::Flirtation => Fixed::from_f64(0.3),
            Self::ActiveCourtship => Fixed::from_f64(0.35),
            Self::TestingReciprocity => Fixed::from_f64(0.4),
            Self::SocialValidation => Fixed::from_f64(0.45),
            Self::Exclusivity => Fixed::from_f64(0.55),
            Self::Betrothal => Fixed::from_f64(0.65),
            Self::Married => Fixed::from_f64(0.75),
            Self::HouseholdFormed => Fixed::from_f64(0.8),
            Self::Parenthood => Fixed::from_f64(0.85),
            Self::Stabilization => Fixed::from_f64(0.7),
            Self::Strain => Fixed::from_f64(0.4),
        }
    }
}

/// Type of marriage arrangement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarriageType {
    /// Arranged by families for alliance.
    Arranged,
    /// Chose each other, socially approved.
    Chosen,
    /// Quick ceremony due to pregnancy.
    Shotgun,
    /// Second marriage after widowhood or divorce.
    Remarriage,
}

/// §10.5 (AP2): How a marriage holds property between the spouses.
///
/// The plan treats property arrangement as an explicit institutional
/// dimension of marriage alongside legitimacy and religious sanction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PropertyArrangement {
    /// No formal arrangement recorded.
    #[default]
    None,
    /// Each spouse retains separate property.
    Separate,
    /// Resources pooled into the shared household.
    CommunalPool,
    /// Bride's family paid a dowry to the groom's family.
    Dowry,
    /// Groom's family paid a bride-price to the bride's family.
    BridePrice,
}

/// §10.5 (AP2): A vow sworn at the marriage ceremony.
///
/// Vows are institutional commitments — breaking them damages legitimacy.
/// Note: `Protection` and `TillDeath` are reserved taxonomy variants — the
/// deterministic `derive_institution` mapping only emits Fidelity/Provision/
/// Care/Honor/Obedience today; the reserved variants exist for future
/// ceremony-type wiring (e.g. martial or religious orders).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum Vow {
    /// Fidelity — sexual and emotional exclusivity.
    #[default]
    Fidelity,
    /// Provision — support the household economically.
    Provision,
    /// Protection — defend the spouse and children.
    Protection,
    /// Honor — uphold the family's reputation.
    Honor,
    /// Care — tend to the spouse in sickness and old age.
    Care,
    /// Obedience — defer to the household head.
    Obedience,
    /// Till-death — permanence of the bond.
    TillDeath,
}

/// A PairBond — the core romantic/sexual relationship between two agents.
///
/// Architecture §10.4: Pair-bonding emerges from repeated positive interaction,
/// intimacy, exclusivity, shared vulnerability, social recognition, cohabitation,
/// children, and economic cooperation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairBond {
    /// Index of partner A.
    pub partner_a: usize,
    /// Index of partner B.
    pub partner_b: usize,
    /// Overall bond strength (0–1).
    pub bond_strength: Fixed,
    /// How exclusive the relationship is (0 = open, 1 = fully exclusive).
    pub exclusivity: Fixed,
    /// Public recognition of the bond (0 = secret, 1 = widely known).
    pub public_recognition: Fixed,
    /// Emotional intimacy level.
    pub emotional_intimacy: Fixed,
    /// Economic cooperation level.
    pub economic_cooperation: Fixed,
    /// Shared children (agent indices).
    pub shared_children: Vec<usize>,
    /// Current strain on the relationship.
    pub strain: Fixed,
    /// Accumulated jealousy load.
    pub jealousy_load: Fixed,
    /// Current romantic stage.
    pub stage: RomanticStage,
    /// Tick when the bond was formed.
    pub formed_tick: u64,
    /// Tick of last positive interaction within the bond.
    pub last_positive_tick: u64,
    /// Tick of last negative interaction within the bond.
    pub last_negative_tick: u64,
}

impl PairBond {
    /// Create a new pair bond between two agents.
    pub fn new(partner_a: usize, partner_b: usize, tick: u64) -> Self {
        Self {
            partner_a,
            partner_b,
            bond_strength: Fixed::from_f64(0.3),
            exclusivity: Fixed::from_f64(0.5),
            public_recognition: Fixed::ZERO,
            emotional_intimacy: Fixed::from_f64(0.2),
            economic_cooperation: Fixed::ZERO,
            shared_children: Vec::new(),
            strain: Fixed::ZERO,
            jealousy_load: Fixed::ZERO,
            stage: RomanticStage::Attraction,
            formed_tick: tick,
            last_positive_tick: tick,
            last_negative_tick: tick,
        }
    }

    /// §10.4 (AP2): Create the pair bond that forms with a marriage — the
    /// couple is already wed, so the bond starts at the Married stage with
    /// the marriage-era strength baseline (not the courtship-era 0.3).
    pub fn new_married(partner_a: usize, partner_b: usize, tick: u64) -> Self {
        let mut bond = Self::new(partner_a, partner_b, tick);
        bond.stage = RomanticStage::Married;
        bond.bond_strength = Fixed::from_f64(0.6);
        bond
    }

    /// Record a positive interaction within the bond.
    pub fn record_positive(&mut self, tick: u64, magnitude: Fixed) {
        self.bond_strength = (self.bond_strength + magnitude * Fixed::from_f64(0.02)).clamp_01();
        self.emotional_intimacy = (self.emotional_intimacy + magnitude * Fixed::from_f64(0.015)).clamp_01();
        self.strain = (self.strain - magnitude * Fixed::from_f64(0.01)).max(Fixed::ZERO);
        self.last_positive_tick = tick;
    }

    /// Record a negative interaction within the bond.
    pub fn record_negative(&mut self, tick: u64, magnitude: Fixed) {
        self.bond_strength = (self.bond_strength - magnitude * Fixed::from_f64(0.03)).max(Fixed::ZERO);
        self.strain = (self.strain + magnitude * Fixed::from_f64(0.025)).clamp_01();
        self.last_negative_tick = tick;
    }

    /// Daily update — decay strain and jealousy, stabilize bond.
    pub fn daily_update(&mut self) {
        // Strain decays slowly
        self.strain = (self.strain * Fixed::from_f64(0.98)).max(Fixed::ZERO);
        // Jealousy decays slowly
        self.jealousy_load = (self.jealousy_load * Fixed::from_f64(0.97)).max(Fixed::ZERO);
        // Bond slowly converges toward 0.5 only when below it (prevent regression of strong bonds)
        if self.bond_strength < Fixed::from_f64(0.5) {
            let drift = (Fixed::from_f64(0.5) - self.bond_strength) * Fixed::from_f64(0.002);
            self.bond_strength = (self.bond_strength + drift).clamp_01();
        }
    }

    /// Check if the bond should dissolve.
    pub fn should_dissolve(&self) -> bool {
        self.bond_strength < Fixed::from_f64(0.1) && self.strain > Fixed::from_f64(0.8)
    }

    /// §10.4 (AP2): Charge jealousy load from the partners' appraised
    /// jealousy emotion, weighted by relationship dependence.
    ///
    /// The appraisal layer already folds attachment anxiety, status threat
    /// and fear of abandonment into the jealousy emotion; dependence
    /// (partner trust) determines how heavily that threat lands on the bond.
    pub fn charge_jealousy(&mut self, jealousy_emotion: Fixed, dependence: Fixed) {
        let weight = Fixed::from_f64(0.35) + Fixed::from_f64(0.65) * dependence;
        self.jealousy_load = (self.jealousy_load
            + jealousy_emotion * weight * Fixed::from_f64(0.15))
            .clamp_01();
    }

    /// §10.4 (AP2): Advance the post-marriage romantic-stage ladder.
    ///
    /// Married → HouseholdFormed → Parenthood (once a shared child exists) →
    /// Strain when jealousy/strain load is high; Strain → Stabilization on
    /// reconciliation (strain low). Separation / widowhood are decisional
    /// outcomes reserved for a future behavioral iteration.
    pub fn advance_stage(&mut self, has_children: bool) {
        self.stage = match self.stage {
            RomanticStage::Married => RomanticStage::HouseholdFormed,
            RomanticStage::HouseholdFormed => {
                if has_children {
                    RomanticStage::Parenthood
                } else {
                    RomanticStage::HouseholdFormed
                }
            }
            RomanticStage::Parenthood => {
                if self.strain > Fixed::from_f64(0.6) {
                    RomanticStage::Strain
                } else {
                    RomanticStage::Parenthood
                }
            }
            RomanticStage::Strain => {
                if self.strain < Fixed::from_f64(0.35) {
                    RomanticStage::Stabilization
                } else {
                    RomanticStage::Strain
                }
            }
            RomanticStage::Stabilization => {
                if self.strain > Fixed::from_f64(0.6) {
                    RomanticStage::Strain
                } else {
                    RomanticStage::Stabilization
                }
            }
            other => other,
        };
    }
}

/// Outcome of a marriage dissolution event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DissolutionCause {
    /// One partner died.
    Death(usize),
    /// Voluntary separation.
    Separation,
    /// Formal annulment (religious/institutional).
    Annulment,
    /// Exile of one partner.
    Exile(usize),
    /// One partner abandoned the household without consent.
    Abandonment(usize),
    /// Legal divorce — community-recognized dissolution.
    Divorce,
    /// Dissolution by religious sanction (e.g. temple annulment).
    ReligiousSanction,
}

/// A Marriage — an institutional bond between agents.
///
/// Architecture §10.5: Marriage affects inheritance, household economics,
/// faction alliances, legitimacy of children, status, sexual norms,
/// religious standing, and kin networks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marriage {
    /// Index of partner A.
    pub partner_a: usize,
    /// Index of partner B.
    pub partner_b: usize,
    /// Type of marriage arrangement.
    pub marriage_type: MarriageType,
    /// Legitimacy of the marriage in community eyes (0–1).
    pub legitimacy: Fixed,
    /// Household index (if a household has been formed).
    pub household_id: Option<usize>,
    /// Children produced by this marriage.
    pub children: Vec<usize>,
    /// Community recognition (0 = unknown, 1 = universally recognized).
    pub community_recognition: Fixed,
    /// Religious sanction (0 = none, 1 = blessed by temple).
    pub religious_sanction: Fixed,
    /// Current strain on the marriage.
    pub strain: Fixed,
    /// Tick when the marriage was formed.
    pub formed_tick: u64,
    /// Whether the marriage is currently active.
    pub active: bool,
    /// §10.5 (AP2): Kin links forged between the spouses' families
    /// (Spouse / InLaw from the kinship graph).
    #[serde(default)]
    pub kin_alliance: Vec<crate::social::kinship::KinshipLink>,
    /// §10.5 (AP2): How property is held between the spouses.
    #[serde(default)]
    pub property_arrangement: PropertyArrangement,
    /// §10.5 (AP2): Vows sworn at the ceremony.
    #[serde(default)]
    pub vows: Vec<Vow>,
}

impl Marriage {
    /// Create a new marriage.
    pub fn new(
        partner_a: usize,
        partner_b: usize,
        marriage_type: MarriageType,
        tick: u64,
    ) -> Self {
        Self {
            partner_a,
            partner_b,
            marriage_type,
            legitimacy: Fixed::from_f64(0.6),
            household_id: None,
            children: Vec::new(),
            community_recognition: Fixed::from_f64(0.3),
            religious_sanction: Fixed::ZERO,
            strain: Fixed::ZERO,
            formed_tick: tick,
            active: true,
            kin_alliance: Vec::new(),
            property_arrangement: PropertyArrangement::None,
            vows: Vec::new(),
        }
    }

    /// §10.5 (AP2): Derive the institutional dimensions deterministically.
    ///
    /// Property arrangement follows the wealth gap between the spouses
    /// (no RNG — replay-deterministic); vows follow the marriage type.
    /// The kin alliance is supplied from the kinship graph by the caller.
    pub fn derive_institution(
        &mut self,
        kin_alliance: Vec<crate::social::kinship::KinshipLink>,
        wealth_a: Fixed,
        wealth_b: Fixed,
    ) {
        self.kin_alliance = kin_alliance;
        let gap = (wealth_a - wealth_b).abs();
        self.property_arrangement = if wealth_a > wealth_b && gap > Fixed::from_f64(0.5) {
            PropertyArrangement::BridePrice
        } else if wealth_b > wealth_a && gap > Fixed::from_f64(0.5) {
            PropertyArrangement::Dowry
        } else {
            PropertyArrangement::CommunalPool
        };
        self.vows = match self.marriage_type {
            MarriageType::Arranged => vec![Vow::Obedience, Vow::Honor, Vow::Fidelity],
            MarriageType::Chosen => vec![Vow::Fidelity, Vow::Care, Vow::Honor],
            MarriageType::Shotgun => vec![Vow::Provision, Vow::Care],
            MarriageType::Remarriage => vec![Vow::Fidelity, Vow::Provision],
        };
    }

    /// Daily update — decay strain, stabilize recognition.
    pub fn daily_update(&mut self) {
        self.strain = (self.strain * Fixed::from_f64(0.98)).max(Fixed::ZERO);
        // Recognition slowly grows if active
        if self.active {
            self.community_recognition =
                (self.community_recognition + Fixed::from_f64(0.001)).clamp_01();
        }
    }

    /// Dissolve the marriage.
    pub fn dissolve(&mut self, cause: &DissolutionCause) {
        self.active = false;
        self.strain = Fixed::ONE;
        match cause {
            DissolutionCause::Death(_) => {
                // Legitimacy preserved — community respects widowhood
                self.legitimacy = (self.legitimacy * Fixed::from_f64(0.8)).clamp_01();
            }
            DissolutionCause::Separation => {
                // Legitimacy damaged
                self.legitimacy = (self.legitimacy - Fixed::from_f64(0.2)).max(Fixed::ZERO);
            }
            DissolutionCause::Annulment => {
                // Religious sanction revoked
                self.religious_sanction = Fixed::ZERO;
                self.legitimacy = (self.legitimacy - Fixed::from_f64(0.3)).max(Fixed::ZERO);
            }
            DissolutionCause::Exile(_) => {
                self.legitimacy = (self.legitimacy * Fixed::from_f64(0.5)).clamp_01();
            }
            DissolutionCause::Abandonment(_) => {
                // Abandonment is a public shame — recognition and legitimacy fall.
                self.legitimacy = (self.legitimacy - Fixed::from_f64(0.25)).max(Fixed::ZERO);
                self.community_recognition =
                    (self.community_recognition - Fixed::from_f64(0.15)).max(Fixed::ZERO);
            }
            DissolutionCause::Divorce => {
                // Legal divorce halves legitimacy.
                self.legitimacy = (self.legitimacy * Fixed::from_f64(0.5)).clamp_01();
            }
            DissolutionCause::ReligiousSanction => {
                // Religious sanction revokes the blessing entirely.
                self.religious_sanction = Fixed::ZERO;
                self.legitimacy = (self.legitimacy - Fixed::from_f64(0.4)).max(Fixed::ZERO);
            }
        }
    }
}

/// Registry of all pair bonds and marriages in the simulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarriageRegistry {
    /// All active pair bonds.
    pub pair_bonds: Vec<PairBond>,
    /// All marriages (active and dissolved).
    pub marriages: Vec<Marriage>,
}

impl MarriageRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            pair_bonds: Vec::new(),
            marriages: Vec::new(),
        }
    }

    /// Find a pair bond between two agents (either direction).
    pub fn find_bond(&self, a: usize, b: usize) -> Option<&PairBond> {
        self.pair_bonds.iter().find(|pb| {
            (pb.partner_a == a && pb.partner_b == b) || (pb.partner_a == b && pb.partner_b == a)
        })
    }

    /// Find an active marriage between two agents.
    pub fn find_marriage(&self, a: usize, b: usize) -> Option<&Marriage> {
        self.marriages.iter().find(|m| {
            m.active
                && ((m.partner_a == a && m.partner_b == b)
                    || (m.partner_a == b && m.partner_b == a))
        })
    }

    /// Daily update for all bonds and marriages.
    pub fn daily_update(&mut self) {
        for bond in &mut self.pair_bonds {
            bond.daily_update();
        }
        for marriage in &mut self.marriages {
            if marriage.active {
                marriage.daily_update();
            }
        }
        // Remove dissolved pair bonds where the bond has fully decayed
        self.pair_bonds
            .retain(|pb| pb.bond_strength > Fixed::ZERO);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn romantic_stage_advances() {
        assert_eq!(
            RomanticStage::Unaware.next_positive(),
            Some(RomanticStage::Awareness)
        );
        assert_eq!(
            RomanticStage::Married.next_positive(),
            Some(RomanticStage::HouseholdFormed)
        );
        assert_eq!(RomanticStage::Strain.next_positive(), None);
    }

    #[test]
    fn pair_bond_positive_increases_strength() {
        let mut bond = PairBond::new(0, 1, 0);
        let initial = bond.bond_strength;
        bond.record_positive(10, Fixed::from_f64(0.5));
        assert!(bond.bond_strength > initial);
    }

    #[test]
    fn pair_bond_negative_increases_strain() {
        let mut bond = PairBond::new(0, 1, 0);
        bond.record_negative(10, Fixed::from_f64(0.5));
        assert!(bond.strain > Fixed::ZERO);
    }

    #[test]
    fn pair_bond_jealousy_charges_with_dependence() {
        // §10.4: dependence (partner trust) amplifies how hard the appraised
        // jealousy emotion lands on the bond.
        let mut low = PairBond::new(0, 1, 0);
        low.charge_jealousy(Fixed::from_f64(0.5), Fixed::ZERO);
        let mut high = PairBond::new(0, 1, 0);
        high.charge_jealousy(Fixed::from_f64(0.5), Fixed::ONE);
        assert!(high.jealousy_load > low.jealousy_load,
            "dependence must amplify jealousy charge");
        assert!(high.jealousy_load > Fixed::ZERO && high.jealousy_load <= Fixed::ONE);
    }

    #[test]
    fn pair_bond_jealousy_load_clamps_at_one() {
        let mut bond = PairBond::new(0, 1, 0);
        for _ in 0..50 {
            bond.charge_jealousy(Fixed::ONE, Fixed::ONE);
        }
        assert_eq!(bond.jealousy_load, Fixed::ONE);
    }

    #[test]
    fn pair_bond_jealousy_load_decays_daily() {
        let mut bond = PairBond::new(0, 1, 0);
        bond.jealousy_load = Fixed::from_f64(0.5);
        bond.daily_update();
        assert!(bond.jealousy_load < Fixed::from_f64(0.5) && bond.jealousy_load > Fixed::ZERO);
    }

    #[test]
    fn pair_bond_stage_advances_post_marriage_ladder() {
        // §10.4: Married → HouseholdFormed → Parenthood → Strain, and
        // reconciliation back to Stabilization once strain abates.
        let mut bond = PairBond::new(0, 1, 0);
        bond.stage = RomanticStage::Married;
        bond.advance_stage(false);
        assert_eq!(bond.stage, RomanticStage::HouseholdFormed);
        bond.advance_stage(true);
        assert_eq!(bond.stage, RomanticStage::Parenthood);
        bond.strain = Fixed::from_f64(0.7);
        bond.advance_stage(true);
        assert_eq!(bond.stage, RomanticStage::Strain);
        bond.strain = Fixed::from_f64(0.2);
        bond.advance_stage(true);
        assert_eq!(bond.stage, RomanticStage::Stabilization);
    }

    #[test]
    fn pair_bond_dissolves_on_strength_collapse_and_strain() {
        let mut bond = PairBond::new(0, 1, 0);
        bond.bond_strength = Fixed::from_f64(0.05);
        bond.strain = Fixed::from_f64(0.9);
        assert!(bond.should_dissolve());
        bond.strain = Fixed::from_f64(0.5);
        assert!(!bond.should_dissolve());
    }

    #[test]
    fn marriage_dissolution_sets_inactive() {
        let mut marriage = Marriage::new(0, 1, MarriageType::Chosen, 100);
        assert!(marriage.active);
        marriage.dissolve(&DissolutionCause::Separation);
        assert!(!marriage.active);
    }

    #[test]
    fn marriage_registry_find_bond() {
        let mut registry = MarriageRegistry::new();
        registry.pair_bonds.push(PairBond::new(0, 1, 0));
        assert!(registry.find_bond(0, 1).is_some());
        assert!(registry.find_bond(1, 0).is_some());
        assert!(registry.find_bond(0, 2).is_none());
    }

    #[test]
    fn marriage_new_initializes_institutional_fields_empty() {
        let m = Marriage::new(0, 1, MarriageType::Chosen, 100);
        assert!(m.kin_alliance.is_empty(), "no kin alliance before derivation");
        assert_eq!(
            m.property_arrangement,
            PropertyArrangement::None,
            "no arrangement before derivation"
        );
        assert!(m.vows.is_empty(), "no vows before derivation");
    }

    #[test]
    fn derive_institution_populates_all_dimensions() {
        let mut m = Marriage::new(2, 3, MarriageType::Chosen, 100);
        m.derive_institution(
            vec![
                crate::social::kinship::KinshipLink::Spouse,
                crate::social::kinship::KinshipLink::InLaw,
            ],
            Fixed::from_f64(10.0),
            Fixed::from_f64(10.0),
        );
        assert_eq!(
            m.kin_alliance,
            vec![
                crate::social::kinship::KinshipLink::Spouse,
                crate::social::kinship::KinshipLink::InLaw,
            ],
            "kin alliance carries the Spouse/InLaw links"
        );
        assert_eq!(
            m.property_arrangement,
            PropertyArrangement::CommunalPool,
            "equal wealth pools resources"
        );
        assert_eq!(
            m.vows,
            vec![Vow::Fidelity, Vow::Care, Vow::Honor],
            "Chosen marriages swear fidelity/care/honor"
        );
    }

    #[test]
    fn derive_institution_wealth_gap_selects_bride_price_or_dowry() {
        let mut groom_richer = Marriage::new(0, 1, MarriageType::Arranged, 100);
        groom_richer.derive_institution(
            Vec::new(),
            Fixed::from_f64(20.0),
            Fixed::from_f64(5.0),
        );
        assert_eq!(
            groom_richer.property_arrangement,
            PropertyArrangement::BridePrice,
            "groom richer → bride-price paid to bride's family"
        );

        let mut bride_richer = Marriage::new(0, 1, MarriageType::Arranged, 100);
        bride_richer.derive_institution(
            Vec::new(),
            Fixed::from_f64(5.0),
            Fixed::from_f64(20.0),
        );
        assert_eq!(
            bride_richer.property_arrangement,
            PropertyArrangement::Dowry,
            "bride richer → dowry paid by bride's family"
        );
    }

    #[test]
    fn dissolution_causes_cover_plan_list() {
        // §10.5 plan: death, abandonment, annulment, divorce, exile, religious
        // sanction — all six must have a dissolve() arm.
        let causes = [
            DissolutionCause::Death(0),
            DissolutionCause::Abandonment(0),
            DissolutionCause::Annulment,
            DissolutionCause::Divorce,
            DissolutionCause::Exile(0),
            DissolutionCause::ReligiousSanction,
        ];
        for cause in causes {
            let mut m = Marriage::new(0, 1, MarriageType::Chosen, 100);
            m.dissolve(&cause);
            assert!(!m.active, "{cause:?} must dissolve the marriage");
        }
        // Religious sanction revokes the blessing.
        let mut m = Marriage::new(0, 1, MarriageType::Chosen, 100);
        m.religious_sanction = Fixed::from_f64(0.8);
        m.dissolve(&DissolutionCause::ReligiousSanction);
        assert_eq!(m.religious_sanction, Fixed::ZERO);
        // Abandonment damages recognition.
        let mut m = Marriage::new(0, 1, MarriageType::Chosen, 100);
        m.community_recognition = Fixed::from_f64(0.7);
        m.dissolve(&DissolutionCause::Abandonment(0));
        assert!(m.community_recognition < Fixed::from_f64(0.7));
    }

    #[test]
    fn institutional_fields_serde_roundtrip() {
        let mut m = Marriage::new(0, 1, MarriageType::Remarriage, 100);
        m.derive_institution(
            vec![crate::social::kinship::KinshipLink::InLaw],
            Fixed::from_f64(5.0),
            Fixed::from_f64(20.0),
        );
        let json = serde_json::to_string(&m).unwrap();
        let restored: Marriage = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.kin_alliance, m.kin_alliance);
        assert_eq!(restored.property_arrangement, m.property_arrangement);
        assert_eq!(restored.vows, m.vows);
        assert_eq!(restored.active, m.active);
    }

    #[test]
    fn old_save_without_institutional_fields_restores_defaults() {
        // A save from before the §10.5 fields existed must deserialize to
        // sensible defaults (serde(default) on each new field).
        // Fixed serializes as raw scaled i64 (SCALE = 10_000), so
        // 0.6 → 6000, 0.3 → 3000.
        let old_json = r#"{
            "partner_a": 0,
            "partner_b": 1,
            "marriage_type": "Chosen",
            "legitimacy": 6000,
            "household_id": null,
            "children": [],
            "community_recognition": 3000,
            "religious_sanction": 0,
            "strain": 0,
            "formed_tick": 100,
            "active": true
        }"#;
        let restored: Marriage = serde_json::from_str(old_json).unwrap();
        assert!(restored.kin_alliance.is_empty());
        assert_eq!(restored.property_arrangement, PropertyArrangement::None);
        assert!(restored.vows.is_empty());
    }
}
