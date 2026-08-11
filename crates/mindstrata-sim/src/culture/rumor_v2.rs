//! Rumor v2 — rumors as memes with uncertainty and social stakes (§13.3).
//!
//! Rumors differ from generic memes because they:
//! - Target specific agents or institutions
//! - Carry accusation severity and evidence quality
//! - Track source chains (who told whom)
//! - Can trigger moral panic and scapegoating
//! - Degrade accuracy with each hop (telephone game)
//!
//! ```text
//! Rumor effects:
//!   - reputation damage
//!   - panic
//!   - scapegoating
//!   - market distortion
//!   - faction formation
//!   - institutional crisis
//!
//! Evidence quality degrades:
//!   evidence_quality × fidelity^hops
//!
//! Prevalence grows then decays:
//!   prevalence = host_count / population × emotional_contagion
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A rumor — a meme with a target, evidence chain, and social stakes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RumorV2 {
    /// Unique rumor identifier.
    pub id: usize,
    /// Description of the rumor content.
    pub description: String,
    /// Index of the agent this rumor targets (if any).
    pub target: Option<usize>,
    /// Index of the institution this rumor targets (if any).
    pub institution_target: Option<usize>,
    /// Severity of the accusation (0 = trivial, 1 = life-destroying).
    pub accusation_severity: Fixed,
    /// Quality of evidence supporting the rumor (0 = pure fabrication, 1 = eyewitness).
    pub evidence_quality: Fixed,
    /// Chain of agents who have relayed this rumor (source chain).
    pub source_chain: Vec<usize>,
    /// Emotional charge accumulated through transmission.
    pub emotional_charge: Fixed,
    /// How prevalent this rumor is in the population (0 = unknown, 1 = everyone knows).
    pub prevalence: Fixed,
    /// Moral panic potential (how easily this triggers collective fear/anger).
    pub moral_panic_potential: Fixed,
    /// Tick when this rumor was created.
    pub created_tick: u64,
    /// Tick when this rumor was last transmitted.
    pub last_transmitted_tick: u64,
    /// Number of agents who currently believe this rumor.
    pub believer_count: u32,
    /// Whether this rumor is still active (not debunked or forgotten).
    pub active: bool,
    /// Whether this rumor has been officially debunked.
    pub debunked: bool,
}

impl RumorV2 {
    /// Create a new rumor from an accusation.
    pub fn new(
        id: usize,
        description: String,
        target: Option<usize>,
        accusation_severity: Fixed,
        evidence_quality: Fixed,
        emotional_charge: Fixed,
        tick: u64,
    ) -> Self {
        Self {
            id,
            description,
            target,
            institution_target: None,
            accusation_severity,
            evidence_quality,
            source_chain: Vec::new(),
            emotional_charge,
            prevalence: Fixed::ZERO,
            moral_panic_potential: accusation_severity * evidence_quality,
            created_tick: tick,
            last_transmitted_tick: tick,
            believer_count: 0,
            active: true,
            debunked: false,
        }
    }

    /// Compute transmission probability from source to listener.
    ///
    /// ```text
    /// chance = evidence_quality × emotional_charge
    ///        × listener_susceptibility × (1 - skepticism)
    ///        × target_proximity
    /// ```
    ///
    /// Evidence degradation with hops is baked into the stored
    /// `evidence_quality` field (each [`record_transmission`](Self::record_transmission)
    /// multiplies it by the 0.85 fidelity factor), so no on-the-fly hop factor
    /// is applied here — the stored field is the single source of truth for
    /// the plan's `evidence_quality × fidelity^hops`.
    pub fn transmission_chance(
        &self,
        source_trust: Fixed,
        listener_susceptibility: Fixed,
        skepticism: Fixed,
        population: u32,
    ) -> Fixed {
        // Prevalence creates social proof (bandwagon effect)
        let social_proof = if population > 0 {
            self.prevalence * Fixed::from_f64(0.2)
        } else {
            Fixed::ZERO
        };

        let base = self.evidence_quality
            * source_trust
            * self.emotional_charge
            * listener_susceptibility
            * (Fixed::ONE + social_proof)
            * (Fixed::ONE - skepticism);

        // Target proximity: rumors about known agents spread faster
        let target_bonus = if self.target.is_some() {
            Fixed::from_f64(0.1)
        } else {
            Fixed::ZERO
        };

        (base + target_bonus).clamp_01()
    }

    /// Record the rumor's originator without degrading evidence.
    ///
    /// The creator is the first entry in the source chain (so the transmission
    /// pass knows who to attribute the first hop to) but has *not* retold the
    /// rumor — the plan's `evidence_quality × fidelity^hops` degradation only
    /// applies to actual retellings, so the hop count (`source_chain.len() - 1`)
    /// stays aligned with the stored evidence penalty.
    pub fn record_source(&mut self, agent_id: usize, tick: u64) {
        self.source_chain.push(agent_id);
        self.last_transmitted_tick = tick;
    }

    /// Record a transmission hop (adds agent to source chain, degrades evidence).
    pub fn record_transmission(&mut self, agent_id: usize, tick: u64) {
        self.source_chain.push(agent_id);
        self.last_transmitted_tick = tick;
        // §13.3: Evidence degrades with each hop (telephone game) — the plan's
        // `evidence_quality × fidelity^hops`. Degrades the *stored* field so
        // hop distortion is observable state, not just a transient in the
        // chance formula.
        self.evidence_quality = (self.evidence_quality * Fixed::from_f64(0.85)).clamp_01();
        // Emotional charge amplifies with each retelling
        self.emotional_charge = (self.emotional_charge + Fixed::from_f64(0.02)).clamp_01();
    }

    /// Compute moral panic pressure from this rumor.
    ///
    /// Panic rises with severity, evidence, and prevalence, then decays.
    pub fn panic_pressure(&self, population: u32) -> Fixed {
        if population == 0 {
            return Fixed::ZERO;
        }
        let prevalence_factor = self.prevalence;
        let severity_factor = self.accusation_severity * self.evidence_quality;
        let contagion = self.moral_panic_potential * prevalence_factor;
        (severity_factor * (Fixed::ONE + contagion)).clamp_01()
    }

    /// Tick update — prevalence decays over time if not reinforced.
    pub fn tick_decay(&mut self, tick: u64) {
        let ticks_since_transmission = tick.saturating_sub(self.last_transmitted_tick);
        // Decay prevalence slowly (rumors fade if not repeated)
        if ticks_since_transmission > 50 {
            let decay_factor = Fixed::from_f64(0.995).powi((ticks_since_transmission / 50) as u32);
            self.prevalence = (self.prevalence * decay_factor).clamp_01();
        }
        // Very old rumors become inactive
        if ticks_since_transmission > 1000 && self.prevalence < Fixed::from_f64(0.05) {
            self.active = false;
        }
    }

    /// Debunk this rumor (mark as false, reduce prevalence).
    pub fn debunk(&mut self) {
        self.debunked = true;
        self.prevalence = (self.prevalence * Fixed::from_f64(0.3)).clamp_01();
        self.active = false;
    }
}

/// Registry of all rumors in the simulation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RumorRegistry {
    /// All rumors.
    pub rumors: Vec<RumorV2>,
    /// Next available rumor id.
    next_id: usize,
}

impl RumorRegistry {
    /// Register a new rumor and return its id.
    pub fn register(&mut self, rumor: RumorV2) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let mut rumor = rumor;
        rumor.id = id;
        self.rumors.push(rumor);
        id
    }

    /// Get a rumor by id.
    pub fn get(&self, id: usize) -> Option<&RumorV2> {
        self.rumors.iter().find(|r| r.id == id)
    }

    /// Get a mutable reference to a rumor by id.
    pub fn get_mut(&mut self, id: usize) -> Option<&mut RumorV2> {
        self.rumors.iter_mut().find(|r| r.id == id)
    }

    /// Get all active rumors targeting a specific agent.
    pub fn rumors_targeting(&self, agent_id: usize) -> Vec<&RumorV2> {
        self.rumors
            .iter()
            .filter(|r| r.active && r.target == Some(agent_id))
            .collect()
    }

    /// Get all active rumors about an institution.
    pub fn rumors_about_institution(&self, inst_id: usize) -> Vec<&RumorV2> {
        self.rumors
            .iter()
            .filter(|r| r.active && r.institution_target == Some(inst_id))
            .collect()
    }

    /// Tick decay on all active rumors.
    pub fn tick_all(&mut self, tick: u64) {
        for rumor in &mut self.rumors {
            if rumor.active {
                rumor.tick_decay(tick);
            }
        }
    }

    /// §13.3: Deterministic daily transmission pass — each active rumor spreads
    /// to its single most receptive listener.
    ///
    /// For every active rumor whose last transmitter is `source`, the pass
    /// scans all agents not already in the source chain, computes
    /// `transmission_chance` for each (scaled by the §12.3 group-escalation
    /// factor of the transmitter — anxious groups escalate rumors, avoidant
    /// groups suppress them), and transmits to the argmax listener when that
    /// chance clears the spread floor. The argmax tie-breaks to the lowest
    /// index, so the pass is **fully deterministic (no RNG)** — the golden
    /// baseline stays byte-identical. `record_transmission` then grows the
    /// source chain (degrading stored evidence) and the believer count, and
    /// prevalence tracks believers per population (the plan's
    /// `host_count / population × emotional_contagion`).
    ///
    /// Returns the number of transmission hops taken.
    pub fn transmission_pass(
        &mut self,
        trust_matrix: &[Vec<Fixed>],
        susceptibility: &[Fixed],
        skepticism: &[Fixed],
        escalation: &[Fixed],
        population: u32,
        tick: u64,
    ) -> usize {
        let n = trust_matrix.len();
        self.transmission_pass_lazy(
            n,
            |listener, source| trust_matrix[listener][source],
            susceptibility,
            skepticism,
            escalation,
            population,
            tick,
        )
    }

    /// §17.4 lazy variant: like [`Self::transmission_pass`] but the trust
    /// value for a (listener, source) pair is produced by a closure instead of
    /// being read from a dense n×n matrix. Callers with a packed/sparse trust
    /// store (e.g. the sim's per-agent `relationship_v2s`) can avoid the
    /// O(n²) matrix allocation + fill on every daily pass. The closure is
    /// invoked exactly once per (rumor, listener) scan — same count as the
    /// matrix reads it replaces — so the pass is bit-identical when the
    /// closure returns the same values the matrix would.
    pub fn transmission_pass_lazy<F>(
        &mut self,
        n: usize,
        trust: F,
        susceptibility: &[Fixed],
        skepticism: &[Fixed],
        escalation: &[Fixed],
        population: u32,
        tick: u64,
    ) -> usize
    where
        F: Fn(usize, usize) -> Fixed,
    {
        const SPREAD_FLOOR: f64 = 0.02;
        let mut hops = 0;
        for i in 0..self.rumors.len() {
            if !self.rumors[i].active {
                continue;
            }
            let source = *self.rumors[i].source_chain.last().unwrap_or(&usize::MAX);
            if source >= n {
                continue;
            }
            let source_escalation = escalation.get(source).copied().unwrap_or(Fixed::ONE);
            // Deterministic argmax over non-chain listeners.
            let mut best: Option<(usize, Fixed)> = None;
            for listener in 0..n {
                if self.rumors[i].source_chain.contains(&listener) {
                    continue;
                }
                let chance = self.rumors[i].transmission_chance(
                    trust(listener, source),
                    susceptibility[listener],
                    skepticism[listener],
                    population,
                ) * source_escalation;
                if chance > Fixed::from_f64(SPREAD_FLOOR)
                    && best.is_none_or(|(_, best_chance)| chance > best_chance)
                {
                    best = Some((listener, chance));
                }
            }
            if let Some((listener, _)) = best {
                self.rumors[i].record_transmission(listener, tick);
                self.rumors[i].believer_count += 1;
                // Plan: prevalence = host_count / population × emotional_contagion.
                let believers = Fixed::from_int(self.rumors[i].believer_count as i64);
                let contagion = Fixed::ONE + self.rumors[i].emotional_charge;
                self.rumors[i].prevalence = if population > 0 {
                    (believers / Fixed::from_int(population as i64) * contagion).clamp_01()
                } else {
                    Fixed::ZERO
                };
                hops += 1;
            }
        }
        hops
    }

    /// Number of active rumors.
    pub fn active_count(&self) -> usize {
        self.rumors.iter().filter(|r| r.active).count()
    }

    /// Sum the panic pressure from all active rumors.
    pub fn total_panic_pressure(&self, population: u32) -> Fixed {
        self.rumors
            .iter()
            .filter(|r| r.active)
            .map(|r| r.panic_pressure(population))
            .fold(Fixed::ZERO, |acc, p| (acc + p).clamp_01())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rumor_has_sane_defaults() {
        let r = RumorV2::new(
            0,
            "the baker stole grain".into(),
            Some(5),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.7),
            0,
        );
        assert_eq!(r.id, 0);
        assert!(r.active);
        assert!(!r.debunked);
        assert_eq!(r.target, Some(5));
    }

    #[test]
    fn transmission_degrades_with_hops() {
        let mut r = RumorV2::new(
            0,
            "test".into(),
            None,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.5),
            0,
        );
        let chance_no_hops = r.transmission_chance(Fixed::ONE, Fixed::ONE, Fixed::ZERO, 100);
        r.record_transmission(1, 1);
        r.record_transmission(2, 2);
        let chance_2_hops = r.transmission_chance(Fixed::ONE, Fixed::ONE, Fixed::ZERO, 100);
        assert!(chance_no_hops > chance_2_hops);
    }

    #[test]
    fn record_transmission_degrades_stored_evidence() {
        // §13.3: stored evidence_quality must degrade with each hop (the
        // plan's `evidence_quality × fidelity^hops`), not just transiently.
        let mut r = RumorV2::new(
            0,
            "test".into(),
            None,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.5),
            0,
        );
        r.record_transmission(1, 1);
        let after_1 = r.evidence_quality;
        r.record_transmission(2, 2);
        assert!(r.evidence_quality < after_1);
        assert!(r.evidence_quality < Fixed::from_f64(0.9));
    }

    #[test]
    fn transmission_pass_spreads_to_most_receptive_listener() {
        // A rumor from source 0: listener 1 has the highest trust (0.9) and
        // lowest skepticism; listener 2 has low trust (0.1).
        let mut reg = RumorRegistry::default();
        reg.register(RumorV2::new(
            0,
            "test".into(),
            None,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.6),
            0,
        ));
        reg.rumors[0].record_source(0, 0); // originator 0
        let trust = vec![
            vec![Fixed::ZERO, Fixed::from_f64(0.5), Fixed::from_f64(0.5)],
            vec![Fixed::from_f64(0.9), Fixed::ZERO, Fixed::from_f64(0.5)],
            vec![Fixed::from_f64(0.1), Fixed::from_f64(0.5), Fixed::ZERO],
        ];
        let susceptibility = vec![Fixed::from_f64(0.8); 3];
        let skepticism = vec![Fixed::ZERO; 3];
        let escalation = vec![Fixed::ONE; 3];
        let hops = reg.transmission_pass(&trust, &susceptibility, &skepticism, &escalation, 10, 1);
        assert_eq!(hops, 1);
        // Listener 1 (trust 0.9) was chosen; chain grew and prevalence rose.
        assert_eq!(reg.rumors[0].source_chain, vec![0, 1]);
        assert_eq!(reg.rumors[0].believer_count, 1);
        assert!(reg.rumors[0].prevalence > Fixed::ZERO);
    }

    #[test]
    fn lazy_pass_is_bit_identical_to_matrix_pass() {
        // §17.4: the lazy closure variant must produce byte-identical rumor
        // state to the dense-matrix API — the sim's O(n²) trust-matrix build
        // is replaced by a direct packed-relationship read, so this equality
        // is the contract that keeps the golden baseline stable.
        fn build() -> RumorRegistry {
            let mut reg = RumorRegistry::default();
            reg.register(RumorV2::new(
                0,
                "a".into(),
                None,
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.9),
                Fixed::from_f64(0.6),
                0,
            ));
            reg.register(RumorV2::new(
                1,
                "b".into(),
                Some(2),
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.7),
                Fixed::from_f64(0.5),
                0,
            ));
            reg.rumors[0].record_source(0, 0);
            reg.rumors[1].record_source(3, 0);
            reg
        }
        let trust = vec![
            vec![
                Fixed::ZERO,
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.5),
            ],
            vec![
                Fixed::from_f64(0.9),
                Fixed::ZERO,
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.5),
            ],
            vec![
                Fixed::from_f64(0.1),
                Fixed::from_f64(0.5),
                Fixed::ZERO,
                Fixed::from_f64(0.5),
            ],
            vec![
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.5),
                Fixed::ZERO,
            ],
        ];
        let susceptibility = vec![Fixed::from_f64(0.8); 4];
        let skepticism = vec![Fixed::ZERO; 4];
        let escalation = vec![Fixed::ONE; 4];

        let mut matrix_reg = build();
        let matrix_hops =
            matrix_reg.transmission_pass(&trust, &susceptibility, &skepticism, &escalation, 10, 1);

        let mut lazy_reg = build();
        let lazy_hops = lazy_reg.transmission_pass_lazy(
            4,
            |l, s| trust[l][s],
            &susceptibility,
            &skepticism,
            &escalation,
            10,
            1,
        );

        assert_eq!(lazy_hops, matrix_hops);
        assert_eq!(lazy_reg.rumors.len(), matrix_reg.rumors.len());
        for (a, b) in lazy_reg.rumors.iter().zip(&matrix_reg.rumors) {
            assert_eq!(a.source_chain, b.source_chain);
            assert_eq!(a.believer_count, b.believer_count);
            assert_eq!(a.prevalence, b.prevalence);
            assert_eq!(a.active, b.active);
        }
        // Hard-coded anchor: the matrix result is exactly the expected spread
        // (rumor 0 from source 0 reaches listener 1 at trust 0.9; rumor 1 from
        // source 3 reaches listener 0 — all its trust entries are 0.5, so the
        // argmax tie-breaks to the lowest index). Guards the shared body
        // against a refactor that keeps the two variants "equal" but wrong.
        assert_eq!(matrix_hops, 2);
        assert_eq!(matrix_reg.rumors[0].source_chain, vec![0, 1]);
        assert_eq!(matrix_reg.rumors[0].believer_count, 1);
        assert_eq!(matrix_reg.rumors[1].source_chain, vec![3, 0]);
        assert_eq!(matrix_reg.rumors[1].believer_count, 1);

        // Closure-consulted leg: a closure returning ZERO trust everywhere
        // must suppress the UNTARGETED rumor (its chance = base + 0, trust is
        // the only driver) while the TARGETED rumor still spreads at its 0.1
        // target-proximity floor — proves the lazy pass actually reads the
        // closure rather than ignoring it (and that the two rumors are
        // distinguished by the closure's values, not by a shared constant).
        let mut zero_reg = build();
        let zero_hops = zero_reg.transmission_pass_lazy(
            4,
            |_, _| Fixed::ZERO,
            &susceptibility,
            &skepticism,
            &escalation,
            10,
            1,
        );
        assert_eq!(zero_hops, 1);
        assert_eq!(zero_reg.rumors[0].source_chain, vec![0]);
        assert_eq!(zero_reg.rumors[1].source_chain, vec![3, 0]);
    }

    #[test]
    fn transmission_pass_is_deterministic_and_avoids_chain_repeats() {
        let mut reg = RumorRegistry::default();
        reg.register(RumorV2::new(
            0,
            "test".into(),
            None,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.6),
            0,
        ));
        reg.rumors[0].record_source(0, 0);
        let trust = vec![
            vec![Fixed::ZERO, Fixed::from_f64(0.7), Fixed::from_f64(0.7)],
            vec![Fixed::from_f64(0.7), Fixed::ZERO, Fixed::from_f64(0.7)],
            vec![Fixed::from_f64(0.7), Fixed::from_f64(0.7), Fixed::ZERO],
        ];
        let susceptibility = vec![Fixed::from_f64(0.8); 3];
        let skepticism = vec![Fixed::ZERO; 3];
        let escalation = vec![Fixed::ONE; 3];
        // Same inputs → same result (no RNG).
        let hops_a =
            reg.transmission_pass(&trust, &susceptibility, &skepticism, &escalation, 10, 1);
        let chain_a = reg.rumors[0].source_chain.clone();
        let hops_b =
            reg.transmission_pass(&trust, &susceptibility, &skepticism, &escalation, 10, 2);
        assert_eq!(hops_a, 1);
        assert_eq!(hops_b, 1);
        // The second pass must pick a *different* listener (no repeats), and
        // the full sequence is deterministic.
        assert_eq!(chain_a, vec![0, 1]);
        assert_eq!(reg.rumors[0].source_chain, vec![0, 1, 2]);
    }

    #[test]
    fn anxious_escalation_boosts_spread_over_secure() {
        // Same rumor, same listeners; only the transmitter's escalation factor
        // differs (anxious 1.5× vs secure 1.0×). The anxious variant must
        // clear the spread floor and transmit where the secure one would not.
        let build = |escalation: Fixed| {
            let mut reg = RumorRegistry::default();
            reg.register(RumorV2::new(
                0,
                "test".into(),
                None,
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.4),
                Fixed::from_f64(0.3),
                0,
            ));
            reg.rumors[0].record_source(0, 0);
            let trust = vec![
                vec![Fixed::ZERO, Fixed::from_f64(0.5), Fixed::from_f64(0.5)],
                vec![Fixed::from_f64(0.5), Fixed::ZERO, Fixed::from_f64(0.5)],
                vec![Fixed::from_f64(0.5), Fixed::from_f64(0.5), Fixed::ZERO],
            ];
            let susceptibility = vec![Fixed::from_f64(0.5); 3];
            let skepticism = vec![Fixed::from_f64(0.4); 3];
            let escalations = vec![escalation, Fixed::ONE, Fixed::ONE];
            reg.transmission_pass(&trust, &susceptibility, &skepticism, &escalations, 10, 1)
        };
        let secure_hops = build(Fixed::from_f64(1.0));
        let anxious_hops = build(Fixed::from_f64(1.5));
        assert!(anxious_hops > secure_hops);
        assert_eq!(secure_hops, 0); // weak rumor does not spread from a secure source
    }

    #[test]
    fn skepticism_reduces_transmission() {
        let r = RumorV2::new(
            0,
            "test".into(),
            None,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.6),
            0,
        );
        let no_skepticism = r.transmission_chance(Fixed::ONE, Fixed::ONE, Fixed::ZERO, 100);
        let high_skepticism =
            r.transmission_chance(Fixed::ONE, Fixed::ONE, Fixed::from_f64(0.8), 100);
        assert!(no_skepticism > high_skepticism);
    }

    #[test]
    fn prevalence_decays_over_time() {
        let mut r = RumorV2::new(
            0,
            "test".into(),
            None,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            0,
        );
        r.prevalence = Fixed::from_f64(0.5);
        r.last_transmitted_tick = 0;
        r.tick_decay(200);
        assert!(r.prevalence < Fixed::from_f64(0.5));
    }

    #[test]
    fn debunk_reduces_prevalence() {
        let mut r = RumorV2::new(
            0,
            "test".into(),
            None,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            0,
        );
        r.prevalence = Fixed::from_f64(0.8);
        r.debunk();
        assert!(r.debunked);
        assert!(r.prevalence < Fixed::from_f64(0.3));
        assert!(!r.active);
    }

    #[test]
    fn registry_register_and_get() {
        let mut reg = RumorRegistry::default();
        let r = RumorV2::new(
            0,
            "test".into(),
            None,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            0,
        );
        let id = reg.register(r);
        assert_eq!(id, 0);
        assert!(reg.get(0).is_some());
        assert!(reg.get(1).is_none());
    }

    #[test]
    fn rumors_targeting_filters_correctly() {
        let mut reg = RumorRegistry::default();
        reg.register(RumorV2::new(
            0,
            "about alice".into(),
            Some(0),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            0,
        ));
        reg.register(RumorV2::new(
            0,
            "about bob".into(),
            Some(1),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            0,
        ));
        let about_0 = reg.rumors_targeting(0);
        assert_eq!(about_0.len(), 1);
        assert_eq!(about_0[0].target, Some(0));
    }

    #[test]
    fn panic_pressure_scales_with_prevalence() {
        let mut r = RumorV2::new(
            0,
            "test".into(),
            None,
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.5),
            0,
        );
        r.prevalence = Fixed::from_f64(0.1);
        let low_panic = r.panic_pressure(100);
        r.prevalence = Fixed::from_f64(0.9);
        let high_panic = r.panic_pressure(100);
        assert!(high_panic > low_panic);
    }
}
