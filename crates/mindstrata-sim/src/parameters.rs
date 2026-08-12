//! §5.1 / Phase 5: Configurable simulation parameters.
//!
//! Centralizes all tuning constants that control simulation behavior.
//! Instead of scattering `Fixed::from_f64(0.3)` across dozens of files,
//! parameters are defined here and referenced by subsystems.
//!
//! ```text
//! Parameter categories:
//!   - Biological: decay rates, thresholds, ceilings
//!   - Psychological: appraisal weights, emotion regulation, cognitive load
//!   - Relational: trust decay, bonding rates, conflict escalation
//!   - Cultural: meme virality, propaganda effectiveness, ritual cohesion
//!   - Economic: price elasticity, trade friction, resource decay
//!   - Institutional: legitimacy decay, policy enforcement, corruption
//!   - Scheduler: tick intervals, phase boundaries
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Top-level simulation parameters — all tuning constants in one place.
///
/// Per Rust best practices (Chapter 1), small Copy types are passed by value.
/// This struct is Copy + Clone + Default for cheap embedding in Simulation.
#[must_use]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SimParameters {
    // ── Biological ─────────────────────────────────────────────
    /// Base hunger decay rate per tick (how fast hunger deficit grows).
    pub hunger_decay_rate: Fixed,
    /// Base thirst decay rate per tick.
    pub thirst_decay_rate: Fixed,
    /// Base fatigue decay rate per tick.
    pub fatigue_decay_rate: Fixed,
    /// Safety need decay rate per tick.
    pub safety_decay_rate: Fixed,
    /// Social need decay rate per tick.
    pub social_decay_rate: Fixed,
    /// Meaning need decay rate per tick.
    pub meaning_decay_rate: Fixed,
    /// Attachment separation distress decay rate (daily).
    pub attachment_decay_rate: Fixed,

    // ── Psychological ─────────────────────────────────────────
    /// Stress smoothing factor (lower = slower adaptation).
    pub stress_smoothing: Fixed,
    /// Fatigue smoothing factor.
    pub fatigue_smoothing: Fixed,
    /// Stress contribution to heuristic bias.
    pub stress_to_heuristic: Fixed,
    /// Fatigue contribution to heuristic bias.
    pub fatigue_to_heuristic: Fixed,
    /// Base heuristic bias (cognitive load floor).
    pub heuristic_bias_floor: Fixed,
    /// Trust sync convergence rate (how fast trust aligns with relationships).
    pub trust_sync_rate: Fixed,
    /// Meme novelty decay rate (daily). Iteration 174: now the LIVE knob —
    /// the daily decay applies `novelty × (1 − rate)`; the redundant
    /// `meme_novelty_decay_factor` complement (0.998) was removed.
    pub meme_novelty_decay: Fixed,
    /// Rumor prevalence decay rate (daily).
    pub rumor_prevalence_decay: Fixed,
    /// Belief resistance decay rate per tick — how fast belief resistance weakens.
    pub belief_resistance_decay: Fixed,
    /// Mental state smoothing factor — how much previous state persists (0.995 = very slow change).
    pub mental_state_smoothing: Fixed,
    /// Mental state accumulation factor — how fast new input is absorbed (0.005 = very slow).
    pub mental_state_accumulation: Fixed,
    /// Base gossip transmission fidelity (0.7 = 70% base accuracy per hop).
    pub gossip_base_fidelity: Fixed,
    /// Gossip emotional distortion multiplier (anger/fear exaggeration).
    pub gossip_emotional_distortion: Fixed,
    /// Gossip acceptance salience threshold (0.15 = rumors below this are ignored).
    pub gossip_acceptance_threshold: Fixed,

    // ── Appraisal ────────────────────────────────────────────
    /// Goal-relevance threshold for triggering emotional response (0.3).
    pub appraisal_goal_relevance_threshold: Fixed,
    /// Sadness multiplier for circumstance-caused events (0.7).
    pub appraisal_sadness_multiplier: Fixed,
    /// Fear multiplier from low coping potential (0.5).
    pub appraisal_fear_coping_multiplier: Fixed,
    /// Low-coping threshold below which fear intensifies (0.3).
    pub appraisal_low_coping_threshold: Fixed,

    // ── Relational / Social ──────────────────────────────────
    /// Reciprocal relationship update factor (0.3 = 30% of direct effect).
    pub social_reciprocal_factor: Fixed,
    /// Low trust threshold — below this, agents threaten/avoid (0.2).
    pub social_low_trust_threshold: Fixed,
    /// High affection threshold — above this, agents comfort/help (0.7).
    pub social_high_affection_threshold: Fixed,
    /// Openness threshold — above this, agents gossip/teach (0.6).
    pub social_openness_threshold: Fixed,
    /// Trust threshold for Friend classification (0.7).
    pub social_friend_trust_threshold: Fixed,
    /// Affection threshold for Friend classification (0.5).
    pub social_friend_affection_threshold: Fixed,
    /// Trust threshold for Rival classification (0.2).
    pub social_rival_trust_threshold: Fixed,
    /// Default trust for new relationships (0.5).
    pub social_default_trust: Fixed,
    /// Default affection for new relationships (0.3).
    pub social_default_affection: Fixed,
    /// Base interaction chance (0.3).
    pub social_interaction_base_chance: Fixed,
    /// Extraversion multiplier for interaction chance (0.4).
    pub social_extraversion_multiplier: Fixed,
    /// §8.1.4 (Iteration 98): loneliness multiplier for interaction chance
    /// (0.3) — a lonely agent seeks social contact more (the emotion gate's
    /// first read of the loneliness family).
    pub social_loneliness_multiplier: Fixed,
    /// §8.1.6 (Iteration 162): the sociability-temperament channel of the
    /// interaction gate — a socially-tempered agent (positive sociability
    /// deviation from its trait-derived baseline, accumulated by the
    /// plasticity pass) clears the gate more often. Deviation is zero at
    /// construction, so this term is byte-identical until life experience
    /// reshapes the temperament layer.
    ///
    /// TUNED (0.3 → 0.15 → 0.08): at 0.3 the probe showed a disproportionate
    /// blast — post-fix sociability deviations reach 0.3–0.7 in calm
    /// windows, so 0.3 × 0.7 = +0.21 on top of a loneliness-saturated gate
    /// (~0.8) clamped the gate to ~1.0 and ERASED the social differentials
    /// (trust-world escalation 17-vs-17, drought vs control farm output
    /// 58.96-vs-58.94). At 0.15 the conception pipeline stalled (seed-44
    /// births collapsed [890,1320,1390,1560,1760] → [310], probe) because
    /// the gate shift re-paces courtship RNG consumption. At 0.08 the
    /// channel adds at most +0.06 — the differentials stay live AND the
    /// pipeline stays healthy (seed-44 births [2890], seed-1 3-chain
    /// [66730, 67850, 93410]) — a genuine nudge that preserves the
    /// calibrated envelope.
    pub social_sociability_multiplier: Fixed,
    /// §8.1.4 (Iteration 99): tenderness multiplier for the help propensity
    /// (0.5) — a tender agent helps neighbors more (the warmth→caregiving
    /// channel; folds into the Help-window consumer, clamped by its
    /// [0.5, 1.0] bound).
    pub social_tenderness_help_multiplier: Fixed,
    /// §8.1.4 (Iteration 127): gratitude multiplier for the help propensity
    /// (0.5, same tier as tenderness) — a grateful agent (recipient of
    /// unexpected positive help) reciprocates by helping more (the
    /// reciprocity→caregiving channel; the appraisal producer is
    /// `positive × (1 − expectedness)`, LIVE in calibrated windows, so this
    /// is a CALIBRATED change — golden + snapshots regenerated).
    pub social_gratitude_help_multiplier: Fixed,
    /// Agreeableness threshold for Teach interaction (0.5).
    pub social_agreeableness_threshold: Fixed,
    /// Friend→Neighbor downgrade threshold (0.4 = Friend downgrades if trust drops below).
    pub social_friend_downgrade_threshold: Fixed,
    /// Rival→Neighbor repair threshold (0.5 = Rival repairs if trust rises above).
    pub social_rival_repair_trust: Fixed,
    /// Trust threshold for friendship classification.
    pub friendship_trust_threshold: Fixed,
    /// Trust threshold for alliance classification.
    pub alliance_trust_threshold: Fixed,
    /// Relationship decay rate when dormant (daily).
    pub relationship_dormant_decay: Fixed,
    /// Emotional event contribution to relationship change.
    pub emotional_event_weight: Fixed,
    /// Bonding rate multiplier — scales all positive interaction deltas.
    /// Default 1.0 preserves original hardcoded behavior; >1.0 amplifies bonding.
    pub bonding_rate: Fixed,
    /// Conflict escalation rate multiplier — scales all negative interaction deltas.
    /// Default 1.0 preserves original hardcoded behavior; >1.0 amplifies conflict.
    pub conflict_escalation_rate: Fixed,

    // ── Belief Update ────────────────────────────────────────
    /// Trust blend factor for blending source_trust with base_trust (0.5).
    pub belief_trust_blend_factor: Fixed,
    /// Identity linkage threshold above which protection kicks in (0.5).
    pub belief_identity_linkage_threshold: Fixed,
    /// Identity protection strength — higher = more resistant (0.3).
    pub belief_identity_protection_strength: Fixed,
    /// Per-tick resistance decay rate (0.001).
    pub belief_resistance_decay_rate: Fixed,
    /// Resistance baseline — below this, no further decay (0.3).
    pub belief_resistance_baseline: Fixed,

    // ── Conflict ─────────────────────────────────────────────
    /// Combat fatigue decay rate per tick when not in combat (0.02).
    pub conflict_combat_fatigue_decay: Fixed,
    /// Trauma decay rate per tick (0.0001 — very slow, years to recover).
    pub conflict_trauma_decay: Fixed,
    /// Combat fatigue accumulation per combat event (0.1).
    pub conflict_combat_fatigue_rate: Fixed,
    /// Dominance weight for aggression calculation (0.3).
    pub conflict_dominance_weight: Fixed,
    /// Risk tolerance weight for aggression calculation (0.2).
    pub conflict_risk_weight: Fixed,
    /// Aggression injury multiplier (0.1).
    pub conflict_aggression_injury_multiplier: Fixed,
    /// Fear sensitivity weight (0.3).
    pub conflict_fear_sensitivity_weight: Fixed,
    /// Fear sensitivity base (0.7).
    pub conflict_fear_sensitivity_base: Fixed,
    /// Trauma multiplier from prior trauma (0.5).
    pub conflict_trauma_multiplier: Fixed,
    /// Lethal injury threshold — injury above this can kill (0.3).
    pub conflict_lethal_injury_threshold: Fixed,
    /// Lethal health threshold — health below this is vulnerable (0.2).
    pub conflict_lethal_health_threshold: Fixed,
    /// Lethal RNG threshold — random chance of death (0.3).
    pub conflict_lethal_rng_threshold: Fixed,
    /// Violence escalation fear threshold — threat fails if target fear below (0.3).
    pub conflict_escalation_fear_threshold: Fixed,
    /// Violence escalation aggression threshold — aggressor must exceed (1.2).
    pub conflict_escalation_aggression_threshold: Fixed,
    /// Violence escalation chance when thresholds met (0.3).
    pub conflict_escalation_chance: Fixed,

    // ── Market / Economic ────────────────────────────────────
    /// Price smoothing factor (0.1 = exponential moving average alpha).
    pub market_price_smoothing: Fixed,
    /// No-supply price ratio (2.0 = price doubles when supply is zero).
    pub market_no_supply_ratio: Fixed,
    /// Trust discount multiplier for direct trades (0.2).
    pub market_trust_discount: Fixed,
    /// Demand weight for need pressure calculation (10.0 ≈ expected per-agent
    /// grain consumption, matching EXPECTED_GRAIN_PER_AGENT). This makes
    /// demand the same order of magnitude as supply so prices can move.
    pub market_demand_weight: Fixed,
    /// Purchasing power divisor (10.0 = coin / 10 = normalized power).
    pub market_purchasing_power_divisor: Fixed,
    /// Scarcity extreme cost multiplier (2.0 = 2x cost when supply is zero).
    pub market_scarcity_extreme: Fixed,
    /// Scarcity abundance cost multiplier (0.5 = 0.5x cost when abundant).
    pub market_scarcity_abundance: Fixed,
    /// Scarcity linear interpolation range (1.5 = 2.0 - 0.5).
    pub market_scarcity_range: Fixed,
    /// Starting grain price (5.0).
    pub market_initial_grain_price: Fixed,
    /// Starting water price (2.0).
    pub market_initial_water_price: Fixed,
    /// Default price for unknown resources (10.0).
    pub market_default_price: Fixed,

    // ── Endocrine (Phase 5 tuning) ─────────────────────────────
    /// Stress axis recovery rate per tick (higher = faster calm-down).
    pub endocrine_stress_recovery: Fixed,
    /// Stress chronic load accumulation rate.
    pub endocrine_stress_chronic_rate: Fixed,
    /// Stress chronic load recovery rate.
    pub endocrine_stress_chronic_recovery: Fixed,
    /// Bonding axis recovery rate per tick.
    pub endocrine_bonding_recovery: Fixed,
    /// Dominance axis response to status change.
    pub endocrine_dominance_response: Fixed,
    /// Arousal axis rise factor.
    pub endocrine_arousal_rise: Fixed,
    /// Arousal axis decay factor.
    pub endocrine_arousal_decay: Fixed,

    // ── Attachment (Phase 5 tuning) ────────────────────────────
    /// Attachment separation distress accrual per daily update for a
    /// partnered agent. Calibrated at 0.02 (Iteration 173): the sweep showed
    /// rates above 0.03 invert the taboo/kin-support/scenario-delta
    /// directionality — the §8.1.14 coupling would dominate, violating the
    /// Phase-5 acceptance — so the envelope is preserved while the knob is
    /// live (previously hardcoded 0.02, now tunable).
    pub attachment_separation_rate: Fixed,
    /// Secure reunion recovery factor.
    pub attachment_secure_recovery: Fixed,
    /// Anxious reunion recovery factor (slower than secure).
    pub attachment_anxious_recovery: Fixed,
    /// Avoidant reunion recovery factor.
    pub attachment_avoidant_recovery: Fixed,
    /// Disorganized reunion recovery factor.
    pub attachment_disorganized_recovery: Fixed,
    /// Secure comfort effectiveness.
    pub attachment_secure_comfort: Fixed,
    /// Anxious comfort effectiveness (partial sooth).
    pub attachment_anxious_comfort: Fixed,
    /// Avoidant comfort effectiveness (may reject).
    pub attachment_avoidant_comfort: Fixed,
    /// Attachment security gain per positive interaction.
    pub attachment_security_gain: Fixed,

    // ── Reproduction / Marriage (Phase 5 tuning) ───────────────
    /// Conception probability multiplier per tick (scales base 0.05).
    pub reproduction_conception_multiplier: Fixed,
    /// Gestation rate multiplier (higher = faster pregnancy progression).
    pub reproduction_gestation_rate: Fixed,
    /// Stress suppression of fertility (0 = no effect, 1 = infertile under stress).
    pub reproduction_stress_suppression: Fixed,
    /// Age-based fertility decline rate per year past 35.
    pub reproduction_age_decline_rate: Fixed,

    // ── Trauma / Recovery (Phase 5 tuning) ─────────────────────
    /// Trauma accumulation rate from sustained high arousal.
    pub nervous_trauma_accumulation: Fixed,
    /// Trauma decay rate per tick (very slow recovery).
    pub nervous_trauma_decay: Fixed,
    /// Sympathetic arousal recovery rate in safety.
    pub nervous_sympathetic_recovery: Fixed,
    /// Parasympathetic buildup rate in safety.
    pub nervous_parasympathetic_buildup: Fixed,

    // ── Meme / Cultural (Phase 5 tuning) ──────────────────────
    /// Meme transmission base chance multiplier.
    pub meme_transmission_multiplier: Fixed,
    /// Meme virality scaling factor (how much emotion+identity boosts
    /// virality). Calibrated at 0.8 (Iteration 174): the knob was previously
    /// dead (seed_initial_memes hardcoded 0.8); wiring it preserved the
    /// probe-verified envelope — all seeded memes active, differentiated
    /// host spread (23/1/3/13/4 of 48 at 10K ticks), novelty held ~0.87 by
    /// transmission reinforcement. A 0.3→1.2 sweep was fully rate-invariant
    /// pre-wiring; the rate-response integration test now proves liveness.
    pub meme_virality_scaling: Fixed,
    /// Meme mutation master multiplier (§13.2) — scales each meme's
    /// per-transmission mutation rate. LIVE by default (0.3: observable
    /// drift — roughly 1-2% of transmissions mutate at seed mutation
    /// rates — while keeping macro-dynamics intact: at 0.5 the anti-
    /// council meme eroded fast enough to pacify politics entirely (no
    /// revolution in 60k ticks), at 0.3 the regime-change cycle still
    /// completes). Set to ZERO to disable entirely, which restores the
    /// identity factor (no decision roll ever drawn → byte-identical
    /// baseline).
    pub meme_mutation_rate_base: Fixed,
    /// Propaganda effectiveness multiplier.
    pub propaganda_effectiveness: Fixed,
    /// Propaganda resistance growth rate per tick (audience fatigue).
    pub propaganda_resistance_growth: Fixed,
    /// Ritual cohesion boost per participation.
    pub ritual_cohesion_boost: Fixed,
    /// Echo chamber emotional charge threshold.
    pub echo_chamber_emotional_threshold: Fixed,

    // ── Economic ──────────────────────────────────────────────
    /// Grain price elasticity (how much price responds to scarcity).
    pub price_elasticity: Fixed,
    /// Water price elasticity.
    pub water_price_elasticity: Fixed,
    /// Trade friction (cost of trading).
    pub trade_friction: Fixed,

    // ── Institutional ─────────────────────────────────────────
    /// Legitimacy decay rate per tick.
    pub legitimacy_decay: Fixed,
    /// Policy enforcement effectiveness.
    pub enforcement_effectiveness: Fixed,
    /// Corruption accumulation rate.
    pub corruption_rate: Fixed,

    // ── Scheduler ─────────────────────────────────────────────
    /// Tick interval for hourly systems.
    pub hourly_interval: u64,
    /// Tick interval for daily systems.
    pub daily_interval: u64,
    /// Tick interval for weekly systems.
    pub weekly_interval: u64,
    /// Tick interval for seasonal systems.
    pub seasonal_interval: u64,
    /// Tick interval for yearly systems.
    pub yearly_interval: u64,
    /// Tier reclassification interval (ticks).
    pub tier_reclassify_interval: u64,
}

impl Default for SimParameters {
    fn default() -> Self {
        Self {
            // Biological — base decay_rate = 0.001, matching original system_need_decay
            hunger_decay_rate: Fixed::from_f64(0.001),
            thirst_decay_rate: Fixed::from_f64(0.002), // decay_rate * 2
            fatigue_decay_rate: Fixed::from_f64(0.0005), // decay_rate * 0.5
            safety_decay_rate: Fixed::from_f64(0.0003), // decay_rate * 0.3
            social_decay_rate: Fixed::from_f64(0.0002), // decay_rate * 0.2
            meaning_decay_rate: Fixed::from_f64(0.00015), // decay_rate * 0.15
            attachment_decay_rate: Fixed::from_f64(0.05),

            // Psychological
            stress_smoothing: Fixed::from_f64(0.1),
            fatigue_smoothing: Fixed::from_f64(0.05),
            stress_to_heuristic: Fixed::from_f64(0.6),
            fatigue_to_heuristic: Fixed::from_f64(0.2),
            heuristic_bias_floor: Fixed::from_f64(0.2),
            trust_sync_rate: Fixed::from_f64(0.1),
            meme_novelty_decay: Fixed::from_f64(0.002),
            rumor_prevalence_decay: Fixed::from_f64(0.01),
            belief_resistance_decay: Fixed::from_f64(0.001), // matches original BELIEF_RESISTANCE_DECAY
            mental_state_smoothing: Fixed::from_f64(0.995),  // matches original
            mental_state_accumulation: Fixed::from_f64(0.005), // matches original
            gossip_base_fidelity: Fixed::from_f64(0.7),      // matches original
            gossip_emotional_distortion: Fixed::from_f64(0.15), // matches original
            gossip_acceptance_threshold: Fixed::from_f64(0.15), // matches original
            appraisal_goal_relevance_threshold: Fixed::from_f64(0.3),
            appraisal_sadness_multiplier: Fixed::from_f64(0.7),
            appraisal_fear_coping_multiplier: Fixed::from_f64(0.5),
            appraisal_low_coping_threshold: Fixed::from_f64(0.3),
            social_reciprocal_factor: Fixed::from_f64(0.3),
            social_low_trust_threshold: Fixed::from_f64(0.2),
            social_high_affection_threshold: Fixed::from_f64(0.7),
            social_openness_threshold: Fixed::from_f64(0.6),
            social_friend_trust_threshold: Fixed::from_f64(0.7),
            social_friend_affection_threshold: Fixed::from_f64(0.5),
            social_rival_trust_threshold: Fixed::from_f64(0.2),
            social_default_trust: Fixed::from_f64(0.5),
            social_default_affection: Fixed::from_f64(0.3),
            social_interaction_base_chance: Fixed::from_f64(0.3),
            social_extraversion_multiplier: Fixed::from_f64(0.4),
            social_loneliness_multiplier: Fixed::from_f64(0.3),
            // §8.1.6 (Iteration 162): the sociability channel of the
            // interaction gate. TUNED to 0.08 (was 0.3, then 0.15 — the
            // pipeline-stall blast, see the field doc): post-fix deviations
            // reach 0.3–0.7, so 0.08 adds +0.03–0.06 to the gate — a
            // genuine nudge that keeps the differentials live AND the
            // conception pipeline healthy.
            social_sociability_multiplier: Fixed::from_f64(0.08),
            social_tenderness_help_multiplier: Fixed::from_f64(0.5),
            social_gratitude_help_multiplier: Fixed::from_f64(0.5),
            social_agreeableness_threshold: Fixed::from_f64(0.5),
            social_friend_downgrade_threshold: Fixed::from_f64(0.4),
            social_rival_repair_trust: Fixed::from_f64(0.5),

            // Relational
            friendship_trust_threshold: Fixed::from_f64(0.5),
            alliance_trust_threshold: Fixed::from_f64(0.7),
            relationship_dormant_decay: Fixed::from_f64(0.001),
            emotional_event_weight: Fixed::from_f64(0.3),
            bonding_rate: Fixed::ONE, // identity: preserves original hardcoded deltas
            conflict_escalation_rate: Fixed::ONE, // identity: preserves original hardcoded deltas

            // Belief Update
            belief_trust_blend_factor: Fixed::from_f64(0.5),
            belief_identity_linkage_threshold: Fixed::from_f64(0.5),
            belief_identity_protection_strength: Fixed::from_f64(0.3),
            belief_resistance_decay_rate: Fixed::from_f64(0.001),
            belief_resistance_baseline: Fixed::from_f64(0.3),
            // Conflict
            conflict_combat_fatigue_decay: Fixed::from_f64(0.02),
            conflict_trauma_decay: Fixed::from_f64(0.0001),
            conflict_combat_fatigue_rate: Fixed::from_f64(0.1),
            conflict_dominance_weight: Fixed::from_f64(0.3),
            conflict_risk_weight: Fixed::from_f64(0.2),
            conflict_aggression_injury_multiplier: Fixed::from_f64(0.1),
            conflict_fear_sensitivity_weight: Fixed::from_f64(0.3),
            conflict_fear_sensitivity_base: Fixed::from_f64(0.7),
            conflict_trauma_multiplier: Fixed::from_f64(0.5),
            conflict_lethal_injury_threshold: Fixed::from_f64(0.3),
            conflict_lethal_health_threshold: Fixed::from_f64(0.2),
            conflict_lethal_rng_threshold: Fixed::from_f64(0.3),
            conflict_escalation_fear_threshold: Fixed::from_f64(0.3),
            conflict_escalation_aggression_threshold: Fixed::from_f64(1.2),
            conflict_escalation_chance: Fixed::from_f64(0.3),
            // Market / Economic
            market_price_smoothing: Fixed::from_f64(0.1),
            market_no_supply_ratio: Fixed::from_f64(2.0),
            market_trust_discount: Fixed::from_f64(0.2),
            market_demand_weight: Fixed::from_f64(10.0),
            market_purchasing_power_divisor: Fixed::from_f64(10.0),
            market_scarcity_extreme: Fixed::from_f64(2.0),
            market_scarcity_abundance: Fixed::from_f64(0.5),
            market_scarcity_range: Fixed::from_f64(1.5),
            market_initial_grain_price: Fixed::from_f64(5.0),
            market_initial_water_price: Fixed::from_f64(2.0),
            market_default_price: Fixed::from_f64(10.0),
            // Endocrine (Phase 5 tuning). 0.10 pairs with the Iter-172
            // STRESS_RECOVERY_TONE_FLOOR (0.3): recovery = 0.10 × max(tone,
            // 0.3) keeps the stress axis in a differentiated equilibrium
            // (0.42–0.58 mean) instead of pinning at 1.0.
            endocrine_stress_recovery: Fixed::from_f64(0.10),
            endocrine_stress_chronic_rate: Fixed::from_f64(0.001),
            endocrine_stress_chronic_recovery: Fixed::from_f64(0.0005),
            endocrine_bonding_recovery: Fixed::from_f64(0.02),
            endocrine_dominance_response: Fixed::from_f64(0.1),
            endocrine_arousal_rise: Fixed::from_f64(0.3),
            endocrine_arousal_decay: Fixed::from_f64(0.1),

            // Attachment (Phase 5 tuning)
            attachment_separation_rate: Fixed::from_f64(0.02),
            attachment_secure_recovery: Fixed::from_f64(0.3),
            attachment_anxious_recovery: Fixed::from_f64(0.6),
            attachment_avoidant_recovery: Fixed::from_f64(0.4),
            attachment_disorganized_recovery: Fixed::from_f64(0.5),
            attachment_secure_comfort: Fixed::from_f64(0.3),
            attachment_anxious_comfort: Fixed::from_f64(0.15),
            attachment_avoidant_comfort: Fixed::from_f64(0.1),
            attachment_security_gain: Fixed::from_f64(0.005),

            // Meme / Cultural (Phase 5 tuning)
            // Reproduction / Marriage (Phase 5 tuning)
            reproduction_conception_multiplier: Fixed::from_f64(1.0),
            reproduction_gestation_rate: Fixed::from_f64(1.0),
            reproduction_stress_suppression: Fixed::from_f64(0.3),
            reproduction_age_decline_rate: Fixed::from_f64(0.03),

            // Trauma / Recovery (Phase 5 tuning)
            nervous_trauma_accumulation: Fixed::from_f64(0.0003),
            nervous_trauma_decay: Fixed::from_f64(0.00005),
            nervous_sympathetic_recovery: Fixed::from_f64(0.1),
            nervous_parasympathetic_buildup: Fixed::from_f64(0.06),

            meme_transmission_multiplier: Fixed::from_f64(1.2),
            meme_virality_scaling: Fixed::from_f64(0.8),
            meme_mutation_rate_base: Fixed::from_f64(0.3),
            propaganda_effectiveness: Fixed::from_f64(0.35),
            propaganda_resistance_growth: Fixed::from_f64(0.002),
            ritual_cohesion_boost: Fixed::from_f64(0.12),
            echo_chamber_emotional_threshold: Fixed::from_f64(0.55),

            // Economic
            price_elasticity: Fixed::from_f64(0.5),
            water_price_elasticity: Fixed::from_f64(0.4),
            trade_friction: Fixed::from_f64(0.05),

            // Institutional
            legitimacy_decay: Fixed::from_f64(0.001),
            enforcement_effectiveness: Fixed::from_f64(0.5),
            corruption_rate: Fixed::from_f64(0.002),

            // Scheduler — §6 tick intervals
            hourly_interval: 6,
            daily_interval: 144,
            weekly_interval: 1008,
            seasonal_interval: 4320,
            yearly_interval: 51840,
            tier_reclassify_interval: 100,
        }
    }
}

impl SimParameters {
    /// Create parameters tuned for faster simulation (compressed timescales).
    pub fn fast() -> Self {
        Self {
            hunger_decay_rate: Fixed::from_f64(0.002),
            thirst_decay_rate: Fixed::from_f64(0.004),
            fatigue_decay_rate: Fixed::from_f64(0.001),
            ..Self::default()
        }
    }

    /// Create parameters tuned for slow, stable simulation.
    pub fn stable() -> Self {
        Self {
            hunger_decay_rate: Fixed::from_f64(0.0005),
            thirst_decay_rate: Fixed::from_f64(0.001),
            fatigue_decay_rate: Fixed::from_f64(0.00025),
            stress_smoothing: Fixed::from_f64(0.05),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_parameters_are_sane() {
        let p = SimParameters::default();
        assert!(p.hunger_decay_rate > Fixed::ZERO);
        assert!(p.thirst_decay_rate > Fixed::ZERO);
        assert!(p.stress_smoothing > Fixed::ZERO && p.stress_smoothing < Fixed::ONE);
        assert!(p.trust_sync_rate > Fixed::ZERO && p.trust_sync_rate < Fixed::ONE);
        assert!(p.friendship_trust_threshold > Fixed::ZERO);
        assert!(p.friendship_trust_threshold < p.alliance_trust_threshold);
        assert_eq!(p.daily_interval, 144);
        assert_eq!(p.yearly_interval, 51840);
    }

    #[test]
    fn fast_parameters_have_higher_decay() {
        let default = SimParameters::default();
        let fast = SimParameters::fast();
        assert!(fast.hunger_decay_rate > default.hunger_decay_rate);
        assert!(fast.thirst_decay_rate > default.thirst_decay_rate);
    }

    #[test]
    fn stable_parameters_have_lower_decay() {
        let default = SimParameters::default();
        let stable = SimParameters::stable();
        assert!(stable.hunger_decay_rate < default.hunger_decay_rate);
        assert!(stable.stress_smoothing < default.stress_smoothing);
    }

    #[test]
    fn parameters_are_copy() {
        let p1 = SimParameters::default();
        let p2 = p1; // Copy, not move
        assert_eq!(p1.hunger_decay_rate, p2.hunger_decay_rate);
    }

    #[test]
    fn parameters_are_serializable() {
        let p = SimParameters::default();
        let json = serde_json::to_string(&p).unwrap();
        let deserialized: SimParameters = serde_json::from_str(&json).unwrap();
        assert_eq!(p.hunger_decay_rate, deserialized.hunger_decay_rate);
        assert_eq!(p.daily_interval, deserialized.daily_interval);
    }
}
