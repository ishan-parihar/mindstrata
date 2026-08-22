//! Institutional entities — §12, §26 of the architecture spec.
//!
//! Institutions are first-class entities with roles, legitimacy, cohesion,
//! norms, and collective psychology. They influence agents through:
//! - roles and obligations
//! - permissions and prohibitions
//! - taxes and wages
//! - rituals and laws
//! - norms and sanctions
//! - information channels
//!
//! §26: "Collective psychology should influence agents through roles,
//! messages, sanctions, resources, and institutional decisions — not
//! through magic global variables."

use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// Named constants for institutional mechanics.
pub const INITIAL_PROPOSAL_DELAY: u64 = 100;
pub const POLICY_RECORD_INTERVAL: u64 = 100;
pub const MAX_RECORDS: usize = 1000;
/// §19.5.J: Maximum provenance trace records per vector before trimming.
pub const MAX_PROVENANCE_RECORDS: usize = 5000;
/// Tax rates by institution kind.
pub const COUNCIL_TAX_RATE: f64 = 0.05;
pub const MARKET_FEE_RATE: f64 = 0.03;
pub const TEMPLE_TITHE_RATE: f64 = 0.02;
/// Tax collection interval in ticks.
pub const TAX_COLLECTION_INTERVAL: u64 = 100;
/// Wage payment interval in ticks.
pub const WAGE_PAYMENT_INTERVAL: u64 = 500;
/// Base wage per role holder.
pub const BASE_WAGE: f64 = 2.0;
/// Seed capital for newly created institutions — without a starting treasury,
/// `pay_wages` (which requires treasury >= total wage cost) could never pay
/// out, silently destroying wealth via taxation with zero circulation.
pub const INITIAL_TREASURY: f64 = 100.0;

/// Types of institutions that can exist in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstitutionKind {
    /// A household — shared resources, roles, internal relationships.
    Household,
    /// A market — trade, prices, exchange.
    Market,
    /// A temple — religion, rituals, meaning.
    Temple,
    /// A council — governance, law, legitimacy.
    Council,
    /// A farm — production, labor, resource extraction.
    Farm,
    /// A workshop — crafting, specialization.
    Workshop,
    /// A faction — emergent political group.
    Faction,
}

impl InstitutionKind {
    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            InstitutionKind::Household => "Household",
            InstitutionKind::Market => "Market",
            InstitutionKind::Temple => "Temple",
            InstitutionKind::Council => "Council",
            InstitutionKind::Farm => "Farm",
            InstitutionKind::Workshop => "Workshop",
            InstitutionKind::Faction => "Faction",
        }
    }
}

/// A role within an institution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Role name (e.g., "Elder", "Priest", "Merchant").
    pub name: String,
    /// Agent holding this role (if any).
    pub holder: Option<AgentId>,
    /// Authority level of this role (0..1).
    pub authority: Fixed,
    /// Obligations associated with this role.
    pub obligations: Vec<String>,
}

/// An institution — a persistent social structure with collective psychology.
///
/// §26: "Institutions should not just have variables like legitimacy or cohesion.
/// They need decision procedures, offices, officials, information channels,
/// delays, corruption, enforcement capacity, policy issuance, and record keeping."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Institution {
    /// Unique identifier.
    pub id: u64,
    /// What kind of institution this is.
    pub kind: InstitutionKind,
    /// Human-readable name.
    pub name: String,
    /// Members of this institution.
    pub members: Vec<AgentId>,
    /// Roles within this institution.
    pub roles: Vec<Role>,
    /// Norms enforced by this institution.
    pub norm_ids: Vec<u64>,
    /// Legitimacy: do people believe this institution has the right to govern?
    pub legitimacy: Fixed,
    /// Cohesion: how well do members work together?
    pub cohesion: Fixed,
    /// Corruption: how much does the institution abuse its power?
    pub corruption: Fixed,
    /// Enforcement capacity: can the institution punish norm violations?
    pub enforcement_capacity: Fixed,
    /// Communication capacity: how efficiently does information flow?
    pub communication_capacity: Fixed,
    /// Collective psychology — derived from member states each tick.
    pub collective: CollectivePsychology,
    // ── §19.5.C: Institutional Decision Layer ─────────────────────────
    /// Pending policies awaiting implementation.
    pub pending_policies: Vec<Policy>,
    /// Implemented policies currently active.
    pub active_policies: Vec<Policy>,
    /// Official record of institutional actions.
    pub records: Vec<InstitutionalRecord>,
    /// Treasury balance (coin held by institution).
    pub treasury: Fixed,
    /// Bureaucratic inertia: delays policy implementation.
    pub inertia: Fixed,
    /// Monotonic counter for unique policy IDs.
    pub policy_counter: u64,
    /// §7.3 (Iteration 240): Tick this institution was created. Drives the
    /// faction entrenchment requirement — a freshly formed faction cannot
    /// revolt the same tick it organizes (movements need time to mobilize;
    /// probed: formation → successful coup within < 1K ticks made every
    /// crisis faction invisible at sampling instants). Serde default keeps
    /// pre-240 snapshots loadable (founding institutions read as ageless).
    #[serde(default)]
    pub formed_tick: u64,
}

/// §19.5.C: A policy issued by an institution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Unique policy identifier.
    pub id: u64,
    /// Human-readable name.
    pub name: String,
    /// Tick when the policy was proposed.
    pub proposed_tick: u64,
    /// Tick when the policy will be implemented (delayed by inertia).
    pub implement_tick: u64,
    /// Whether the policy is currently active.
    pub active: bool,
    /// Effect strength of the policy (e.g., tax rate, fine amount).
    pub effect: Fixed,
}

/// §19.5.C: A record of an institutional action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionalRecord {
    /// Tick when the action occurred.
    pub tick: u64,
    /// Description of the action.
    pub action: String,
    /// Agent(s) affected by the action.
    pub affected: Vec<AgentId>,
    /// Whether the action was successful.
    pub success: bool,
}

/// Derived collective psychology of an institution.
///
/// §23: "Collective psychology should influence agents through roles,
/// messages, sanctions, resources, and institutional decisions — not
/// through magic global variables."
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CollectivePsychology {
    /// Overall morale of the institution.
    pub morale: Fixed,
    /// How united are the members?
    pub unity: Fixed,
    /// Collective fear or anxiety.
    pub fear: Fixed,
    /// Collective ambition or drive.
    pub ambition: Fixed,
    /// Trust in leadership (derived from legitimacy).
    pub trust_in_leadership: Fixed,
    /// Ideological rigidity: resistance to change.
    pub ideological_rigidity: Fixed,
}

impl Institution {
    /// Create a new institution.
    pub fn new(id: u64, kind: InstitutionKind, name: String) -> Self {
        Self {
            id,
            kind,
            name,
            members: Vec::new(),
            roles: Vec::new(),
            norm_ids: Vec::new(),
            legitimacy: Fixed::from_f64(0.6),
            cohesion: Fixed::from_f64(0.5),
            corruption: Fixed::ZERO,
            enforcement_capacity: Fixed::from_f64(0.3),
            communication_capacity: Fixed::from_f64(0.5),
            collective: CollectivePsychology::default(),
            pending_policies: Vec::new(),
            active_policies: Vec::new(),
            records: Vec::new(),
            treasury: Fixed::ZERO,
            inertia: Fixed::from_f64(0.3),
            policy_counter: 0,
            formed_tick: 0,
        }
    }

    // ── §19.5.C: Institutional Decision Methods ───────────────────────

    /// Propose a new policy. It will be implemented after a delay.
    pub fn propose_policy(&mut self, name: String, effect: Fixed, current_tick: u64) {
        let delay = (self.inertia * Fixed::from_f64(100.0)).to_f64() as u64;
        let id = self.policy_counter;
        self.policy_counter += 1;
        self.pending_policies.push(Policy {
            id,
            name,
            proposed_tick: current_tick,
            implement_tick: current_tick + delay.max(1),
            active: false,
            effect,
        });
    }

    /// Process pending policies — move ready ones to active.
    pub fn process_policies(&mut self, current_tick: u64) {
        let mut still_pending = Vec::new();
        for policy in self.pending_policies.drain(..) {
            if current_tick >= policy.implement_tick {
                self.active_policies.push(Policy {
                    active: true,
                    ..policy
                });
            } else {
                still_pending.push(policy);
            }
        }
        self.pending_policies = still_pending;
    }

    /// Record an institutional action.
    pub fn record_action(
        &mut self,
        tick: u64,
        action: String,
        affected: Vec<AgentId>,
        success: bool,
    ) {
        self.records.push(InstitutionalRecord {
            tick,
            action,
            affected,
            success,
        });
    }

    /// Collect taxes from members. Deducts from member wealth and adds to treasury.
    /// Returns total collected.
    ///
    /// INVARIANT: AgentId::new(i) == index i. The member_wealth vec must be built
    /// from institution.members using agent_id.as_u64() as the index.
    pub fn collect_taxes(
        &mut self,
        tax_rate: Fixed,
        member_wealth: &mut [(AgentId, Fixed)],
    ) -> Fixed {
        let mut total = Fixed::ZERO;
        for (agent, wealth) in member_wealth.iter_mut() {
            if self.has_member(*agent) {
                let tax = *wealth * tax_rate;
                *wealth = (*wealth - tax).max(Fixed::ZERO);
                total += tax;
            }
        }
        self.treasury += total;
        total
    }

    /// Pay wages to role holders. Returns total paid.
    /// Only pays if treasury can cover ALL wages. Returns ZERO if insolvent.
    ///
    /// INVARIANT: AgentId::new(i) == index i. The member_wealth vec must be built
    /// from institution.members using agent_id.as_u64() as the index.
    pub fn pay_wages(&mut self, wage: Fixed, member_wealth: &mut [(AgentId, Fixed)]) -> Fixed {
        // Count how many role holders will receive wages
        let role_holder_count = self.roles.iter().filter(|r| r.holder.is_some()).count();
        let total_wage_cost = wage * Fixed::from_int(role_holder_count as i64);
        // Treasury solvency check: must cover ALL wages before paying any
        if self.treasury < total_wage_cost {
            return Fixed::ZERO;
        }
        let mut total = Fixed::ZERO;
        for role in &self.roles {
            if let Some(holder) = role.holder {
                if let Some(entry) = member_wealth.iter_mut().find(|(a, _)| *a == holder) {
                    entry.1 += wage;
                    total += wage;
                }
            }
        }
        self.treasury -= total;
        total
    }

    /// Add a member to this institution.
    pub fn add_member(&mut self, agent: AgentId) {
        if !self.members.contains(&agent) {
            self.members.push(agent);
        }
    }

    /// Remove a member from this institution.
    pub fn remove_member(&mut self, agent: AgentId) {
        self.members.retain(|m| *m != agent);
    }

    /// Check if an agent is a member.
    pub fn has_member(&self, agent: AgentId) -> bool {
        self.members.contains(&agent)
    }

    /// Add a role to this institution.
    pub fn add_role(&mut self, role: Role) {
        self.roles.push(role);
    }

    /// Assign an agent to a role by name.
    pub fn assign_role(&mut self, role_name: &str, agent: AgentId) {
        if let Some(role) = self.roles.iter_mut().find(|r| r.name == role_name) {
            role.holder = Some(agent);
        }
    }

    /// Get the agent holding a specific role.
    pub fn get_role_holder(&self, role_name: &str) -> Option<AgentId> {
        self.roles
            .iter()
            .find(|r| r.name == role_name)
            .and_then(|r| r.holder)
    }

    /// Derive collective psychology from member states.
    ///
    /// §23: "Collective psychology should be derived from:
    /// member states, leadership, resources, recent victories/defeats,
    /// internal conflict, external threats, legitimacy, communication quality."
    pub fn derive_collective_psychology(
        &mut self,
        member_morales: &[Fixed],
        member_trusts: &[Fixed],
    ) {
        if member_morales.is_empty() {
            return;
        }

        let n = Fixed::from_int(member_morales.len() as i64);

        // Clamp member morales to [0, 1] before averaging (valence can be negative)
        let clamped_morales: Vec<Fixed> = member_morales.iter().map(|m| m.clamp_01()).collect();

        // Morale = average member morale + legitimacy bonus
        let avg_morale: Fixed = clamped_morales.iter().fold(Fixed::ZERO, |a, b| a + *b) / n;
        self.collective.morale = (avg_morale + self.legitimacy * Fixed::from_f64(0.2)).clamp_01();

        // Unity = 1 - variance of member trusts (high variance = low unity)
        let avg_trust: Fixed = member_trusts.iter().fold(Fixed::ZERO, |a, b| a + *b) / n;
        let variance: Fixed = member_trusts
            .iter()
            .map(|t| {
                let diff = *t - avg_trust;
                diff * diff
            })
            .fold(Fixed::ZERO, |a, b| a + b)
            / n;
        self.collective.unity = (Fixed::ONE - variance).clamp_01();

        // Fear = corruption + low legitimacy
        self.collective.fear =
            (self.corruption + (Fixed::ONE - self.legitimacy) * Fixed::from_f64(0.3)).clamp_01();

        // Ambition = morale * (1 - fear)
        self.collective.ambition =
            (self.collective.morale * (Fixed::ONE - self.collective.fear)).clamp_01();

        // Trust in leadership = legitimacy * (1 - corruption)
        self.collective.trust_in_leadership =
            (self.legitimacy * (Fixed::ONE - self.corruption)).clamp_01();

        // Ideological rigidity = high cohesion + high legitimacy
        self.collective.ideological_rigidity = (self.cohesion * Fixed::from_f64(0.5)
            + self.legitimacy * Fixed::from_f64(0.5))
        .clamp_01();
    }

    /// Decay legitimacy over time (institutions lose legitimacy without reinforcement).
    pub fn decay_legitimacy(&mut self, rate: Fixed) {
        self.legitimacy = (self.legitimacy - rate).max(Fixed::ZERO);
    }

    /// Increase legitimacy (e.g., from successful outcomes).
    pub fn increase_legitimacy(&mut self, amount: Fixed) {
        self.legitimacy = (self.legitimacy + amount).clamp_01();
    }
}

/// Create default institutions for a settlement.
pub fn default_institutions() -> Vec<Institution> {
    let mut institutions = Vec::new();

    // Council — governance
    let mut council = Institution::new(0, InstitutionKind::Council, "Village Council".into());
    council.legitimacy = Fixed::from_f64(0.7);
    council.enforcement_capacity = Fixed::from_f64(0.5);
    council.add_role(Role {
        name: "Elder".into(),
        holder: None,
        authority: Fixed::from_f64(0.8),
        obligations: vec!["Arbitrate disputes".into(), "Set taxes".into()],
    });
    council.add_role(Role {
        name: "Guard Captain".into(),
        holder: None,
        authority: Fixed::from_f64(0.6),
        obligations: vec!["Enforce laws".into(), "Protect settlement".into()],
    });
    institutions.push(council);

    // Temple — religion and meaning
    let mut temple = Institution::new(1, InstitutionKind::Temple, "Village Temple".into());
    temple.legitimacy = Fixed::from_f64(0.6);
    temple.add_role(Role {
        name: "Priest".into(),
        holder: None,
        authority: Fixed::from_f64(0.4),
        obligations: vec!["Lead rituals".into(), "Provide counsel".into()],
    });
    temple.norm_ids = vec![3]; // "Obey Ruler" norm reinforced by temple
    institutions.push(temple);

    // Market — trade and economy
    let mut market = Institution::new(2, InstitutionKind::Market, "Village Market".into());
    market.legitimacy = Fixed::from_f64(0.5);
    market.add_role(Role {
        name: "Merchant".into(),
        holder: None,
        authority: Fixed::from_f64(0.3),
        obligations: vec!["Maintain fair prices".into()],
    });
    institutions.push(market);

    // §19.5.C: Seed treasuries so wage payments and public goods can actually
    // flow. Previously treasuries started at zero and `pay_wages` (insolvency
    // check) never paid anyone — taxes drained agent wealth with no return.
    for inst in &mut institutions {
        inst.treasury = Fixed::from_f64(INITIAL_TREASURY);
    }

    institutions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn institution_creation() {
        let inst = Institution::new(0, InstitutionKind::Council, "Test".into());
        assert_eq!(inst.kind, InstitutionKind::Council);
        assert!(inst.members.is_empty());
        assert!(inst.legitimacy > Fixed::ZERO);
    }

    #[test]
    fn member_management() {
        let mut inst = Institution::new(0, InstitutionKind::Household, "Test".into());
        let agent = AgentId::new(0);
        inst.add_member(agent);
        assert!(inst.has_member(agent));
        inst.remove_member(agent);
        assert!(!inst.has_member(agent));
    }

    #[test]
    fn role_assignment() {
        let mut inst = Institution::new(0, InstitutionKind::Council, "Test".into());
        inst.add_role(Role {
            name: "Elder".into(),
            holder: None,
            authority: Fixed::from_f64(0.8),
            obligations: vec![],
        });
        assert!(inst.get_role_holder("Elder").is_none());
        inst.assign_role("Elder", AgentId::new(0));
        assert_eq!(inst.get_role_holder("Elder"), Some(AgentId::new(0)));
    }

    #[test]
    fn collective_psychology_derivation() {
        let mut inst = Institution::new(0, InstitutionKind::Council, "Test".into());
        let morales = vec![
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.7),
        ];
        let trusts = vec![
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
        ];
        inst.derive_collective_psychology(&morales, &trusts);

        assert!(inst.collective.morale > Fixed::ZERO);
        assert!(inst.collective.unity > Fixed::ZERO);
    }

    #[test]
    fn legitimacy_decay() {
        let mut inst = Institution::new(0, InstitutionKind::Council, "Test".into());
        let initial = inst.legitimacy;
        inst.decay_legitimacy(Fixed::from_f64(0.1));
        assert!(inst.legitimacy < initial);
    }

    #[test]
    fn default_institutions_count() {
        let institutions = default_institutions();
        assert_eq!(institutions.len(), 3);
        assert!(institutions
            .iter()
            .any(|i| i.kind == InstitutionKind::Council));
        assert!(institutions
            .iter()
            .any(|i| i.kind == InstitutionKind::Temple));
        assert!(institutions
            .iter()
            .any(|i| i.kind == InstitutionKind::Market));
    }

    #[test]
    fn propose_policy_assigns_unique_ids() {
        let mut inst = Institution::new(0, InstitutionKind::Council, "Test".into());
        inst.propose_policy("Policy A".into(), Fixed::from_f64(0.1), 0);
        inst.propose_policy("Policy B".into(), Fixed::from_f64(0.2), 0);
        assert_eq!(inst.pending_policies.len(), 2);
        assert_ne!(inst.pending_policies[0].id, inst.pending_policies[1].id);
        assert_eq!(inst.policy_counter, 2);
    }

    #[test]
    fn process_policies_moves_ready_to_active() {
        let mut inst = Institution::new(0, InstitutionKind::Council, "Test".into());
        inst.propose_policy("Test".into(), Fixed::from_f64(0.1), 0);
        // Not ready yet — delay is based on inertia
        inst.process_policies(0);
        assert_eq!(inst.pending_policies.len(), 1);
        assert_eq!(inst.active_policies.len(), 0);
        // Process far in the future — should be ready
        inst.process_policies(10000);
        assert_eq!(inst.pending_policies.len(), 0);
        assert_eq!(inst.active_policies.len(), 1);
        assert!(inst.active_policies[0].active);
    }

    #[test]
    fn record_action_stores_record() {
        let mut inst = Institution::new(0, InstitutionKind::Council, "Test".into());
        inst.record_action(42, "Did something".into(), vec![AgentId::new(0)], true);
        assert_eq!(inst.records.len(), 1);
        assert_eq!(inst.records[0].tick, 42);
        assert!(inst.records[0].success);
    }

    #[test]
    fn collect_taxes_deducts_from_members() {
        let mut inst = Institution::new(0, InstitutionKind::Council, "Test".into());
        inst.add_member(AgentId::new(0));
        inst.add_member(AgentId::new(1));
        let mut wealth = vec![
            (AgentId::new(0), Fixed::from_f64(100.0)),
            (AgentId::new(1), Fixed::from_f64(50.0)),
        ];
        let collected = inst.collect_taxes(Fixed::from_f64(0.1), &mut wealth);
        assert!(collected > Fixed::ZERO);
        assert!(wealth[0].1 < Fixed::from_f64(100.0)); // taxed
        assert!(wealth[1].1 < Fixed::from_f64(50.0)); // taxed
        assert_eq!(inst.treasury, collected);
    }

    #[test]
    fn pay_wages_adds_to_role_holders() {
        let mut inst = Institution::new(0, InstitutionKind::Council, "Test".into());
        inst.add_role(Role {
            name: "Elder".into(),
            holder: Some(AgentId::new(0)),
            authority: Fixed::from_f64(0.8),
            obligations: vec![],
        });
        inst.treasury = Fixed::from_f64(100.0);
        let mut wealth = vec![(AgentId::new(0), Fixed::from_f64(50.0))];
        let paid = inst.pay_wages(Fixed::from_f64(10.0), &mut wealth);
        assert!(paid > Fixed::ZERO);
        assert!(wealth[0].1 > Fixed::from_f64(50.0)); // paid
        assert!(inst.treasury < Fixed::from_f64(100.0)); // treasury reduced
    }

    #[test]
    fn pay_wages_multiple_role_holders() {
        let mut inst = Institution::new(0, InstitutionKind::Council, "Test".into());
        inst.add_role(Role {
            name: "Elder".into(),
            holder: Some(AgentId::new(0)),
            authority: Fixed::from_f64(0.8),
            obligations: vec![],
        });
        inst.add_role(Role {
            name: "Guard Captain".into(),
            holder: Some(AgentId::new(1)),
            authority: Fixed::from_f64(0.6),
            obligations: vec![],
        });
        inst.treasury = Fixed::from_f64(100.0);
        let mut wealth = vec![
            (AgentId::new(0), Fixed::from_f64(50.0)),
            (AgentId::new(1), Fixed::from_f64(30.0)),
        ];
        let paid = inst.pay_wages(Fixed::from_f64(5.0), &mut wealth);
        assert_eq!(paid, Fixed::from_f64(10.0)); // 2 holders × 5.0
        assert!(wealth[0].1 > Fixed::from_f64(50.0));
        assert!(wealth[1].1 > Fixed::from_f64(30.0));
        assert!(inst.treasury < Fixed::from_f64(100.0));
    }

    #[test]
    fn pay_wages_skips_empty_treasury() {
        let mut inst = Institution::new(0, InstitutionKind::Council, "Test".into());
        inst.add_role(Role {
            name: "Elder".into(),
            holder: Some(AgentId::new(0)),
            authority: Fixed::from_f64(0.8),
            obligations: vec![],
        });
        inst.treasury = Fixed::from_f64(1.0); // not enough for wage of 5.0
        let mut wealth = vec![(AgentId::new(0), Fixed::from_f64(50.0))];
        let paid = inst.pay_wages(Fixed::from_f64(5.0), &mut wealth);
        assert_eq!(paid, Fixed::ZERO); // can't afford
        assert_eq!(wealth[0].1, Fixed::from_f64(50.0)); // wealth unchanged
    }

    #[test]
    fn collect_taxes_with_different_rates() {
        let mut council = Institution::new(0, InstitutionKind::Council, "Council".into());
        council.add_member(AgentId::new(0));
        let mut wealth = vec![(AgentId::new(0), Fixed::from_f64(100.0))];
        let collected = council.collect_taxes(Fixed::from_f64(COUNCIL_TAX_RATE), &mut wealth);
        // 5% of 100 = 5.0
        assert!((collected.to_f64() - 5.0).abs() < 0.1);
        assert!((wealth[0].1.to_f64() - 95.0).abs() < 0.1);

        let mut market = Institution::new(1, InstitutionKind::Market, "Market".into());
        market.add_member(AgentId::new(0));
        let mut wealth2 = vec![(AgentId::new(0), Fixed::from_f64(100.0))];
        let collected2 = market.collect_taxes(Fixed::from_f64(MARKET_FEE_RATE), &mut wealth2);
        // 3% of 100 = 3.0
        assert!((collected2.to_f64() - 3.0).abs() < 0.1);
    }

    #[test]
    fn treasury_accumulates_from_taxes_then_pays_wages() {
        let mut inst = Institution::new(0, InstitutionKind::Council, "Test".into());
        inst.add_member(AgentId::new(0));
        inst.add_role(Role {
            name: "Elder".into(),
            holder: Some(AgentId::new(0)),
            authority: Fixed::from_f64(0.8),
            obligations: vec![],
        });
        // Collect taxes first
        let mut wealth = vec![(AgentId::new(0), Fixed::from_f64(100.0))];
        let _ = inst.collect_taxes(Fixed::from_f64(0.1), &mut wealth);
        assert!(inst.treasury > Fixed::ZERO); // 10.0
                                              // Then pay wages from treasury
        let paid = inst.pay_wages(Fixed::from_f64(5.0), &mut wealth);
        assert_eq!(paid, Fixed::from_f64(5.0));
        assert!((inst.treasury.to_f64() - 5.0).abs() < 0.1); // 10.0 - 5.0
    }

    #[test]
    fn tax_rate_constants_are_consistent() {
        assert!((COUNCIL_TAX_RATE - 0.05).abs() < f64::EPSILON);
        assert!((MARKET_FEE_RATE - 0.03).abs() < f64::EPSILON);
        assert!((TEMPLE_TITHE_RATE - 0.02).abs() < f64::EPSILON);
        assert_eq!(TAX_COLLECTION_INTERVAL, 100);
        assert_eq!(WAGE_PAYMENT_INTERVAL, 500);
        assert!((BASE_WAGE - 2.0).abs() < f64::EPSILON);
    }
}
