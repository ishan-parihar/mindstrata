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

use crate::fixed::Fixed;
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

    // ── Development / pathology (difficulty row 3) ─────────────
    /// Multiplier on every quadrant's per-tick pathology GROWTH fraction
    /// (`docs/balance/difficulty-levers.md` row 3). 1.0 = canon; the
    /// Resilient band is 0.5, the Brittle band 1.8.
    #[serde(default = "scale_identity")]
    pub pathology_growth_scale: Fixed,
    /// Multiplier on every quadrant's per-tick pathology DECAY fraction.
    /// 1.0 = canon; the Resilient band is 1.2, the Brittle band 0.7.
    #[serde(default = "scale_identity")]
    pub pathology_decay_scale: Fixed,
    /// Multiplier on every quadrant's pathology intensity CEILING (i315;
    /// the row-3 axis the catalog lists as a `0.65–1.0` range and i304
    /// deferred). 1.0 = canon (Q1/Q2 0.80, Q3 0.85, Q4 0.75). The Resilient
    /// band is 0.85 (lower caps), the Brittle band 1.15 (higher caps).
    /// Governing, not cosmetic: growth is `headroom = ceiling − intensity`,
    /// so the ceiling sets each quadrant's equilibrium (live where intensity
    /// approaches the cap — Q2 and Q4 at the calibrated horizons).
    #[serde(default = "scale_identity")]
    pub pathology_ceiling_scale: Fixed,
    /// Multiplier on the five goal-generation fulfillment thresholds
    /// (`docs/balance/difficulty-levers.md` row 2's threshold half, the
    /// companion of the need-decay rates above; i305). 1.0 = canon. A LOWER
    /// multiplier makes the village respond to deficits earlier (standing
    /// deficits stay small — the abundant feel); a higher one makes it tolerate
    /// more deficit before acting (scarcity feel).
    #[serde(default = "scale_identity")]
    pub goal_gate_scale: Fixed,

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
    /// §8.1.6 (Iteration 180): altruism multiplier for the help propensity
    /// (0.23 — a trait tier BELOW the transient-emotion tiers 0.5: the core
    /// trait is a standing disposition drawn 0..1 at birth and present in
    /// EVERY calibrated window, so even a modest multiplier is a uniform
    /// standing shift, not a differential nudge — the Iter-129
    /// floor-pinned framing). The last decision-less core trait gets its
    /// consumer: a high-altruism agent helps neighbors more (the
    /// disposition→caregiving channel; folds into the Help-window consumer
    /// `[0.2, 0.5 × (1 + propensity))`, clamped by its [0.5, 1.0] bound).
    /// ONE-SIDED identity-at-zero: altruism 0 → the norm-only legacy value.
    /// RATE-CALIBRATED BY 2D SWEEP: the standing shift re-paces the shared
    /// interaction RNG stream NON-MONOTONICALLY (the Iter-164 pattern), so
    /// a fine sweep (seed 42, 2000 ticks) was required to find a rate that
    /// preserves ALL THREE directional contracts simultaneously:
    /// tenderness (warm > cold × 1.1), violence-taboo aversion (maxed
    /// taboo suppresses), and taboo-shame tracking (stripped world's extra
    /// acts out-produce). 0.3 flips tenderness (3259 > 2974); 0.2 passes
    /// tenderness but collapses the taboo differentials (70 = 70); the
    /// [0.23, 0.24] band passes all three (0.23 → tenderness 2382 < 2799,
    /// taboo 65 > 63, shame violence 65 < 66 AND 0.451 > 0.328).
    pub social_altruism_help_multiplier: Fixed,
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
    /// i376: daily convergence rate of the legacy v1 trust row onto its dyadic
    /// (`RelationshipV2`) counterpart. 0 = the legacy store is frozen, 1 = it is
    /// set to the dyadic value each daily boundary. Previously this was a
    /// mean-reversion rate toward a hardcoded 0.5 baseline, which the i376 probe
    /// measured as ~14× too weak (v1 trust saturated at 0.887–0.920 with 79–85%
    /// of pairs ≥0.95 while the dyadic store held 0.588–0.628).
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
    /// Violence escalation chance when thresholds met (0.12).
    /// Iteration 185 (P5 emergent re-audit): recalibrated 0.3 → 0.12
    /// (first pass 0.05). At 0.3, ~30% of failed threats escalated to
    /// Violence, and with the pre-fix threat stream (2372 threats / 2000
    /// calm ticks) that meant dozens of beatings per day → every calm
    /// seed's population collapsed (5/6 seeds ≤4/12 alive @20K, 100% of
    /// deaths = Violence, first death on day 1). 0.05 overcorrected: a
    /// failed threat escalated essentially never (probe: 0 violence
    /// events in seed 42's first 2000 ticks, ~1–8 per 5000-tick window),
    /// which starved the violence-response systems (dominance/trust/taboo
    /// escalation differentials) of observable fuel. 0.12 is the
    /// probe-pinned middle ground: violence reachable (seed 42: 3 events
    /// @2K, 9 @9K; 17 across the 3-seed dominance sweep @5K) while a
    /// 5-seed × 20K calm sweep still yields ZERO deaths (injury 0.12 + the
    /// 0.001/tick recovery keeps repeated aggression non-lethal; enemy
    /// clans still double the chance to 0.24 via `escalation_chance`).
    /// Conflict — threats, insults, feuds, trauma, fear — stays the
    /// reachable drama channel while murder remains a rare, legible story
    /// event.
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
    /// Growth axis recovery rate per tick (mean-reversion toward the
    /// life-stage growth target).
    pub endocrine_growth_recovery: Fixed,
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
    /// Conception probability multiplier (scales the per-period birth roll).
    /// Iteration 175: now LIVE — `should_birth` multiplies its period
    /// probability by this (identity at 1.0, so the envelope is preserved);
    /// previously 100%-dead, so the fertility tuning knob was a no-op.
    pub reproduction_conception_multiplier: Fixed,
    /// Base rate of a pair marrying on any given tick (the scalar in the
    /// marriage_chance product: attraction * health * trust * rate * clan).
    /// Iteration 175: now LIVE — the marriage block previously hardcoded
    /// 0.01, so the marriage half of the "tune marriage/fertility" row was
    /// untunable. Default 0.01 preserves the calibrated envelope exactly.
    pub marriage_formation_rate: Fixed,
    /// Gestation rate multiplier (higher = faster pregnancy progression).
    pub reproduction_gestation_rate: Fixed,
    /// Stress suppression of fertility (0 = no effect, 1 = infertile under stress).
    pub reproduction_stress_suppression: Fixed,
    /// Age-based fertility decline rate per year past 35.
    pub reproduction_age_decline_rate: Fixed,

    // ── Trauma / Recovery (Phase 5 tuning) ─────────────────────
    /// Trauma accumulation rate from sustained high arousal.
    pub nervous_trauma_accumulation: Fixed,
    /// Trauma decay FRACTION per tick (proportional since Iteration 176 —
    /// converges to `hot_fraction × accumulation / rate`, so the knob
    /// continuously tunes the recovery envelope; the pre-fix subtractive
    /// decay was a dead knife-edge with no representable range).
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
    /// i374: the legitimacy-coupled council dividend share — BASE term `s0`
    /// of the law `s = s0 + k·(1 − legitimacy)`. The organic replacement for
    /// the hardcoded 0.25 (i363): a nervous/resented council buys goodwill
    /// with patronage, a secure one hoards. Default `s0 = 0.15`, `k = 0.25`
    /// passes exactly through the old constant at the §29.2 equilibrium
    /// legitimacy 0.6 (0.15 + 0.25·0.4 = 0.25 — see `council_dividend_share_k`).
    pub council_dividend_share_s0: Fixed,
    /// i374: legitimacy COUPLING STRENGTH `k` of the dividend law. `k = 0`
    /// degenerates to a constant share `s0` (the pre-i374 shape). Default is
    /// set so the law passes exactly through the old 0.25 constant at the
    /// measured legitimacy equilibrium: 0.25 = 0.15 + k·(1 − 0.6) → k = 0.25.
    pub council_dividend_share_k: Fixed,
    /// Propaganda effectiveness multiplier (§13.4). Iteration 177: now
    /// LIVE — applied to the full effectiveness product in
    /// `compute_effectiveness`. Default recalibrated 0.35 -> 1.0 (identity):
    /// the calibrated envelope was implicitly 1.0 (the knob was dead), and
    /// 0.35 would push mean effectiveness below the 0.1 apply gate,
    /// functionally disabling propaganda.
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
    /// i303: which difficulty-lever band this parameter set carries. Serde
    /// default (Standard) keeps pre-i303 serialized params loading at the
    /// canon band. The six decay rates above are the MATERIALIZED band —
    /// `with_difficulty` writes both together; this field is the honest
    /// provenance record for snapshots/probes, not an independent input.
    #[serde(default)]
    pub difficulty: DifficultyProfile,
}

/// Serde default for the row-3 pathology scale multipliers: the canon
/// identity (1.0), so parameter payloads written before i304 load in the band
/// every calibrated window was measured in. (`Fixed::default()` is ZERO —
/// harmless for a decay rate, catastrophic for a scale multiplier, which is
/// why this is an explicit helper rather than a bare `#[serde(default)]`.)
fn scale_identity() -> Fixed {
    Fixed::ONE
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
            // Identity multipliers — the canon band (see `with_difficulty`).
            pathology_growth_scale: Fixed::from_f64(1.0),
            pathology_decay_scale: Fixed::from_f64(1.0),
            pathology_ceiling_scale: Fixed::from_f64(1.0),
            goal_gate_scale: Fixed::from_f64(1.0),

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
            social_altruism_help_multiplier: Fixed::from_f64(0.23),
            social_agreeableness_threshold: Fixed::from_f64(0.5),
            social_friend_downgrade_threshold: Fixed::from_f64(0.4),
            social_rival_repair_trust: Fixed::from_f64(0.5),

            // Relational
            friendship_trust_threshold: Fixed::from_f64(0.5),
            alliance_trust_threshold: Fixed::from_f64(0.7),
            relationship_dormant_decay: Fixed::from_f64(1.0),
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
            conflict_escalation_chance: Fixed::from_f64(0.12),
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
            endocrine_growth_recovery: Fixed::from_f64(0.001), // slow per-tick drift
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
            marriage_formation_rate: Fixed::from_f64(0.01),
            reproduction_gestation_rate: Fixed::from_f64(1.0),
            reproduction_stress_suppression: Fixed::from_f64(0.3),
            reproduction_age_decline_rate: Fixed::from_f64(0.03),

            // Trauma / Recovery (Phase 5 tuning)
            nervous_trauma_accumulation: Fixed::from_f64(0.0003),
            nervous_trauma_decay: Fixed::from_f64(0.0005),
            nervous_sympathetic_recovery: Fixed::from_f64(0.1),
            nervous_parasympathetic_buildup: Fixed::from_f64(0.06),

            meme_transmission_multiplier: Fixed::from_f64(1.2),
            meme_virality_scaling: Fixed::from_f64(0.8),
            meme_mutation_rate_base: Fixed::from_f64(0.3),
            // i374: s = 0.15 + 0.25·(1 − legit) passes through 0.25 at the
            // §29.2 legitimacy equilibrium 0.6 — the constant law at the
            // measured midpoint, coupled away from it.
            council_dividend_share_s0: Fixed::from_f64(0.15),
            council_dividend_share_k: Fixed::from_f64(0.25),
            propaganda_effectiveness: Fixed::from_f64(1.0),
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

            //            // Scheduler — §6 tick intervals
            hourly_interval: 6,
            daily_interval: 144,
            weekly_interval: 1008,
            seasonal_interval: 4320,
            yearly_interval: 51840,
            tier_reclassify_interval: 100,
            // i303: canon difficulty band (the Materialized default —
            // `with_difficulty` rewrites the six rates above for other bands).
            difficulty: DifficultyProfile::Standard,
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

    /// Difficulty-lever surface (i303, DC-4 entry "b"; row 3 added in i304):
    /// the ratified levers catalog (`docs/balance/difficulty-levers.md`) row 2
    /// ("Need decay", bands 0.6× / 1.0× / 1.4×) and row 3 ("Pathology
    /// growth/ceiling", bands Resilient growth 0.5× decay 1.2× / Standard 1.0×
    /// / Brittle growth 1.8× decay 0.7×) promoted from DRAFT to a live
    /// setting. Standard is BYTE-IDENTICAL to today's canon by construction —
    /// it returns `default()` untouched, so every golden/snapshot window stays
    /// zero-blast; Lenient/Harsh rescale the six need-decay rates and the two
    /// pathology scale multipliers per the catalog's candidate bands.
    ///
    /// §5 quantize-once discipline: band values are computed here ONCE at
    /// construction (f64 → one `Fixed::from_f64`), never re-derived per tick,
    /// so no sub-resolution per-tick increment can truncate to zero. The
    /// effective raw bands are pinned in `parameters::tests` — the meaning
    /// channel's Lenient band collapses onto Standard at Fixed-4 resolution
    /// (raw 1 both; 0.00015 × 0.6 = 0.00009 rounds back to raw 1), recorded
    /// honestly rather than reshaped: the lever bites on hunger/thirst/
    /// fatigue/safety/social and on meaning only at Harsh (raw 1 → 2). If a
    /// 3-way distinct meaning band is ever required, the upgrade path is the
    /// standing §5 f64-shadow accumulator, a behavioral iteration of its own.
    ///
    /// Composes with nothing: presets like `fast()`/`stable()` that override
    /// the same six rates must be applied AFTER difficulty (or their overrides
    /// re-applied) — difficulty is the world-level contract, presets are dev
    /// knobs. The derived esteem/autonomy rate stays coupled at 2/3 of
    /// meaning per `system_need_decay_with_params`; at Fixed-4 resolution its
    /// raw rate is 1 in every band (0.0001×0.6667 and 0.0002×0.6667 both
    /// round to raw 1) — also pinned in tests.
    pub fn with_difficulty(profile: DifficultyProfile) -> Self {
        let mut p = Self {
            difficulty: profile,
            ..Self::default()
        };
        match profile {
            // Identity by construction: never re-derive the canon rates.
            DifficultyProfile::Standard => {}
            DifficultyProfile::Lenient => {
                p.hunger_decay_rate = Fixed::from_f64(0.0006); // 0.0010 × 0.6
                p.thirst_decay_rate = Fixed::from_f64(0.0012); // 0.0020 × 0.6
                p.fatigue_decay_rate = Fixed::from_f64(0.0003); // 0.0005 × 0.6
                p.safety_decay_rate = Fixed::from_f64(0.0002); // 0.0003 × 0.6 (1.8 → 2)
                p.social_decay_rate = Fixed::from_f64(0.0001); // 0.0002 × 0.6 (1.2 → 1)
                p.meaning_decay_rate = Fixed::from_f64(0.0001); // 0.00015 × 0.6 (0.9 → 1, sub-resolution)
                                                                // Row 3, Resilient band: slow accumulation, fast recovery,
                                                                // and a lower ceiling — 0.85 × canon (Q1/Q2 0.68, Q3 0.7225,
                                                                // Q4 0.6375), inside the catalog's 0.65–1.0 range.
                p.pathology_growth_scale = Fixed::from_f64(0.5);
                p.pathology_decay_scale = Fixed::from_f64(1.2);
                p.pathology_ceiling_scale = Fixed::from_f64(0.85);
                // Row 2 threshold half: respond to deficits earlier (i305).
                p.goal_gate_scale = Fixed::from_f64(0.6);
            }
            DifficultyProfile::Harsh => {
                p.hunger_decay_rate = Fixed::from_f64(0.0014); // 0.0010 × 1.4
                p.thirst_decay_rate = Fixed::from_f64(0.0028); // 0.0020 × 1.4
                p.fatigue_decay_rate = Fixed::from_f64(0.0007); // 0.0005 × 1.4
                p.safety_decay_rate = Fixed::from_f64(0.0004); // 0.0003 × 1.4 (4.2 → 4)
                p.social_decay_rate = Fixed::from_f64(0.0003); // 0.0002 × 1.4 (2.8 → 3)
                p.meaning_decay_rate = Fixed::from_f64(0.0002); // 0.00015 × 1.4 (2.1 → 2)
                                                                // Row 3, Brittle band: fast accumulation, slow recovery, and
                                                                // a higher ceiling — 1.15 × canon (Q1/Q2 0.92, Q3 0.9775,
                                                                // Q4 0.8625), inside the catalog's 0.65–1.0 range.
                p.pathology_growth_scale = Fixed::from_f64(1.8);
                p.pathology_decay_scale = Fixed::from_f64(0.7);
                p.pathology_ceiling_scale = Fixed::from_f64(1.15);
                // Row 2 threshold half: tolerate more deficit before acting.
                p.goal_gate_scale = Fixed::from_f64(1.4);
            }
        }
        p
    }
}
/// Difficulty-lever profile (i303; row 3 wired i304; row 2's threshold
/// half wired i305): which ratified band the run sits in across the
/// promoted rows of `docs/balance/difficulty-levers.md` — row 2 (need decay
/// AND the goal-generation fulfillment thresholds, both 0.6×/1.0×/1.4×) and
/// row 3 pathology growth+decay (Resilient/Standard/Brittle). Standard
/// is the canon default and the only value every calibrated window was
/// measured at; Lenient/Harsh are the catalog's Low/High candidate bands
/// promoted to a runtime surface with probe evidence (i303 + i304 probes
/// and the `parameters`/`systems::development` pins).
///
/// The two rows move together by design: a lenient world is abundant in
/// provisioning AND forgiving in its pathology operator, a harsh world is
/// scarce AND brittle. If a future consumer needs them decoupled, split
/// this into per-row profiles rather than adding a third enum.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DifficultyProfile {
    /// Canon default — the calibrated 1.0× decay band; byte-identical to the
    /// unparameterized defaults every golden/snapshot window pins.
    #[default]
    Standard,
    /// Low band — 0.6× need decay: deficits accumulate slower, the village
    /// feels abundant (fewer survival-driven goals). Catalog hypothesis band.
    Lenient,
    /// High band — 1.4× need decay: deficits accumulate faster, the village
    /// feels scarcity-driven (survival work crowds out social/worship).
    /// Catalog hypothesis band.
    Harsh,
}

impl DifficultyProfile {
    /// Catalog decay multiplier (the `match difficulty { Low => 0.85, … }`
    /// pure-data shape the levers doc prescribes). Standard is exactly 1.0 —
    /// the identity that keeps every calibrated window untouched.
    #[must_use]
    pub fn decay_multiplier(self) -> f64 {
        match self {
            Self::Lenient => 0.6,
            Self::Standard => 1.0,
            Self::Harsh => 1.4,
        }
    }
}

impl std::str::FromStr for DifficultyProfile {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "lenient" => Ok(Self::Lenient),
            "standard" => Ok(Self::Standard),
            "harsh" => Ok(Self::Harsh),
            other => Err(format!(
                "unknown difficulty profile '{other}' — expected lenient|standard|harsh"
            )),
        }
    }
}

impl std::fmt::Display for DifficultyProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Lenient => "lenient",
            Self::Standard => "standard",
            Self::Harsh => "harsh",
        })
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

    // ── i303 difficulty-lever bands (docs/balance/difficulty-levers.md row 2) ──

    /// STANDARD IS BYTE-IDENTICAL to the canon defaults — the zero-blast
    /// contract that lets the lever ship without touching a single pin.
    /// `with_difficulty(Standard)` must return exactly `default()`.
    #[test]
    fn standard_difficulty_is_identity_with_canon_defaults() {
        let d = SimParameters::default();
        let s = SimParameters::with_difficulty(DifficultyProfile::Standard);
        assert_eq!(d.hunger_decay_rate, s.hunger_decay_rate);
        assert_eq!(d.thirst_decay_rate, s.thirst_decay_rate);
        assert_eq!(d.fatigue_decay_rate, s.fatigue_decay_rate);
        assert_eq!(d.safety_decay_rate, s.safety_decay_rate);
        assert_eq!(d.social_decay_rate, s.social_decay_rate);
        assert_eq!(d.meaning_decay_rate, s.meaning_decay_rate);
        assert_eq!(d.difficulty, DifficultyProfile::Standard);
        // Row 3 (i304) and row 2's threshold half (i305) ride the same
        // identity contract.
        assert_eq!(d.pathology_growth_scale, s.pathology_growth_scale);
        assert_eq!(d.pathology_decay_scale, s.pathology_decay_scale);
        assert_eq!(d.pathology_ceiling_scale, s.pathology_ceiling_scale);
        assert_eq!(d.goal_gate_scale, s.goal_gate_scale);
        assert_eq!(s.pathology_growth_scale, Fixed::ONE);
        assert_eq!(s.pathology_decay_scale, Fixed::ONE);
        assert_eq!(s.goal_gate_scale, Fixed::ONE);
        // Full-struct identity: difficulty is the ONLY field the mapping may
        // differ on, and for Standard it does not differ at all.
        let json_a = serde_json::to_string(&d).unwrap();
        let json_b = serde_json::to_string(&s).unwrap();
        assert_eq!(
            json_a, json_b,
            "Standard must serialize identical to default"
        );
    }

    /// §5 quantize-once pin: the EFFECTIVE raw (Fixed-4) bands after the
    /// 0.6×/1.4× mapping. The levers doc's candidate bands quantize
    /// non-uniformly at 4-decimal resolution — these are the honest measured
    /// bands, each pinned with its derivation so no future "fix" re-derives
    /// them differently. Notes:
    ///   - hunger 0.0006→raw 6 (0.0010×0.6); thirst 0.0012→raw 12;
    ///     fatigue 0.0003→raw 3; safety 0.00018→raw 2 (rounds up); social
    ///     0.00012→raw 1 (rounds DOWN from 1.2 — lenient social decay is
    ///     HALF canon, not 0.6×);
    ///   - meaning 0.00009→raw 1 = CANON — the Lenient meaning band collapses
    ///     onto Standard at Fixed-4 (recorded, not reshaped; upgrade path is
    ///     the §5 f64-shadow accumulator);
    ///   - harsh social 0.00028→raw 3, safety 0.00042→raw 4, meaning
    ///     0.00021→raw 2 (all round-to-nearest).
    #[test]
    fn difficulty_bands_quantize_to_pinned_raw_values() {
        let len = SimParameters::with_difficulty(DifficultyProfile::Lenient);
        let harsh = SimParameters::with_difficulty(DifficultyProfile::Harsh);
        let std = SimParameters::with_difficulty(DifficultyProfile::Standard);
        // Lenient raw band (raw units: 1 raw = 1e-4).
        assert_eq!(len.hunger_decay_rate.to_raw(), 6);
        assert_eq!(len.thirst_decay_rate.to_raw(), 12);
        assert_eq!(len.fatigue_decay_rate.to_raw(), 3);
        assert_eq!(len.safety_decay_rate.to_raw(), 2);
        assert_eq!(len.social_decay_rate.to_raw(), 1);
        assert_eq!(
            len.meaning_decay_rate.to_raw(),
            1,
            "sub-resolution collapse: 0.9 rounds back to canon 1"
        );
        // Harsh raw band.
        assert_eq!(harsh.hunger_decay_rate.to_raw(), 14);
        assert_eq!(harsh.thirst_decay_rate.to_raw(), 28);
        assert_eq!(harsh.fatigue_decay_rate.to_raw(), 7);
        assert_eq!(harsh.safety_decay_rate.to_raw(), 4);
        assert_eq!(harsh.social_decay_rate.to_raw(), 3);
        assert_eq!(
            harsh.meaning_decay_rate.to_raw(),
            2,
            "harsh meaning bites: canon 1 → 2"
        );
        // Monotonic band ordering, every channel.
        assert!(len.hunger_decay_rate < std.hunger_decay_rate);
        assert!(std.hunger_decay_rate < harsh.hunger_decay_rate);
        assert!(len.thirst_decay_rate < std.thirst_decay_rate);
        assert!(std.thirst_decay_rate < harsh.thirst_decay_rate);
        assert!(len.fatigue_decay_rate < std.fatigue_decay_rate);
        assert!(std.fatigue_decay_rate < harsh.fatigue_decay_rate);
        assert!(len.safety_decay_rate < std.safety_decay_rate);
        assert!(std.safety_decay_rate < harsh.safety_decay_rate);
        assert!(len.social_decay_rate < std.social_decay_rate);
        assert!(std.social_decay_rate < harsh.social_decay_rate);
        // Meaning: lenient == standard (documented collapse), harsh > standard.
        assert_eq!(len.meaning_decay_rate, std.meaning_decay_rate);
        assert!(std.meaning_decay_rate < harsh.meaning_decay_rate);
    }

    /// The catalog's pure-data multiplier contract: Standard is exactly 1.0
    /// (identity), Lenient/Harsh carry the 0.6×/1.4× candidate bands.
    #[test]
    fn difficulty_multipliers_match_catalog() {
        assert_eq!(DifficultyProfile::Lenient.decay_multiplier(), 0.6);
        assert_eq!(DifficultyProfile::Standard.decay_multiplier(), 1.0);
        assert_eq!(DifficultyProfile::Harsh.decay_multiplier(), 1.4);
        // Parsing (CLI surface) is total over the catalog vocabulary.
        use std::str::FromStr;
        assert_eq!(
            DifficultyProfile::from_str("harsh"),
            Ok(DifficultyProfile::Harsh)
        );
        assert_eq!(
            DifficultyProfile::from_str("Standard"),
            Ok(DifficultyProfile::Standard)
        );
        assert_eq!(
            DifficultyProfile::from_str("LENIENT"),
            Ok(DifficultyProfile::Lenient)
        );
        assert!(DifficultyProfile::from_str("extreme").is_err());
    }

    // ── i304 difficulty-lever row 3 (pathology growth/decay) ──

    /// Quantize-once pin for the row-3 bands. These multipliers survive
    /// Fixed-4 exactly (0.5/1.2/1.8/0.7 are all representable), so unlike
    /// row 2 there is NO sub-resolution collapse here — the honest record is
    /// exactness. Growth falls resilient → standard → brittle; recovery
    /// (decay) rises the other way, which is what makes the band a band.
    #[test]
    fn pathology_bands_scale_growth_up_and_decay_down() {
        let res = SimParameters::with_difficulty(DifficultyProfile::Lenient);
        let std = SimParameters::with_difficulty(DifficultyProfile::Standard);
        let bri = SimParameters::with_difficulty(DifficultyProfile::Harsh);
        assert_eq!(res.pathology_growth_scale.to_raw(), 5_000);
        assert_eq!(res.pathology_decay_scale.to_raw(), 12_000);
        assert_eq!(std.pathology_growth_scale.to_raw(), 10_000);
        assert_eq!(std.pathology_decay_scale.to_raw(), 10_000);
        assert_eq!(bri.pathology_growth_scale.to_raw(), 18_000);
        assert_eq!(bri.pathology_decay_scale.to_raw(), 7_000);
        assert!(res.pathology_growth_scale < std.pathology_growth_scale);
        assert!(std.pathology_growth_scale < bri.pathology_growth_scale);
        assert!(res.pathology_decay_scale > std.pathology_decay_scale);
        assert!(std.pathology_decay_scale > bri.pathology_decay_scale);
        // f64 resolution: each multiplier is exact, so the operator's
        // per-tick growth/decay fractions scale by the pinned factor with no
        // rounding step of its own.
        assert_eq!(res.pathology_growth_scale.to_f64(), 0.5);
        assert_eq!(bri.pathology_decay_scale.to_f64(), 0.7);
    }

    // ── i315 difficulty-lever row 3, ceiling half ──

    /// The ceiling band (i315) rides the row-3 multipliers: lower caps in the
    /// Resilient band, higher in the Brittle band, and Fixed-4-exact
    /// (0.85 → 8 500, 1.15 → 11 500).
    #[test]
    fn pathology_ceiling_band_lowers_resilient_and_raises_brittle() {
        let res = SimParameters::with_difficulty(DifficultyProfile::Lenient);
        let std = SimParameters::with_difficulty(DifficultyProfile::Standard);
        let bri = SimParameters::with_difficulty(DifficultyProfile::Harsh);
        assert_eq!(res.pathology_ceiling_scale.to_raw(), 8_500);
        assert_eq!(std.pathology_ceiling_scale.to_raw(), 10_000);
        assert_eq!(bri.pathology_ceiling_scale.to_raw(), 11_500);
        assert!(res.pathology_ceiling_scale < std.pathology_ceiling_scale);
        assert!(std.pathology_ceiling_scale < bri.pathology_ceiling_scale);
        assert_eq!(res.pathology_ceiling_scale.to_f64(), 0.85);
        assert_eq!(bri.pathology_ceiling_scale.to_f64(), 1.15);
    }

    // ── i305 difficulty-lever row 2, threshold half (goal gates) ──

    /// The goal-gate band rides the same multipliers as the row-2 decay half
    /// (0.6/1.0/1.4) and, like the row-3 scales, survives Fixed-4 exactly — no
    /// sub-resolution collapse class exists on this half either.
    #[test]
    fn goal_gate_band_scales_the_fulfillment_thresholds() {
        let len = SimParameters::with_difficulty(DifficultyProfile::Lenient);
        let std = SimParameters::with_difficulty(DifficultyProfile::Standard);
        let harsh = SimParameters::with_difficulty(DifficultyProfile::Harsh);
        assert_eq!(len.goal_gate_scale.to_raw(), 6_000);
        assert_eq!(std.goal_gate_scale.to_raw(), 10_000);
        assert_eq!(harsh.goal_gate_scale.to_raw(), 14_000);
        assert!(len.goal_gate_scale < std.goal_gate_scale);
        assert!(std.goal_gate_scale < harsh.goal_gate_scale);
        // Fixed-4 is exact for all three multipliers, so the scale that reaches
        // the gate math is the catalog factor itself. Canon gates 0.5/0.6/0.7
        // and the retain gate 0.3 stay exactly representable when scaled
        // (0.3/0.36/0.42/0.18 lenient; 0.7/0.84/0.98/0.42 harsh).
        for (gate, expect_len, expect_harsh) in [
            (0.5_f64, 0.3, 0.7),
            (0.6, 0.36, 0.84),
            (0.7, 0.42, 0.98),
            (0.3, 0.18, 0.42),
        ] {
            let canon = Fixed::from_f64(gate);
            assert_eq!((canon * len.goal_gate_scale).to_f64(), expect_len);
            assert_eq!((canon * harsh.goal_gate_scale).to_f64(), expect_harsh);
            assert_eq!((canon * std.goal_gate_scale).to_f64(), gate);
        }
    }

    /// Serde back-compat: a param payload serialized before i304 (no
    /// pathology-scale keys) loads at the canon identity — the same
    /// serde-default pattern every Snapshot field addition has used since v8.
    /// The i305 `goal_gate_scale` key is covered by the same assertion.
    #[test]
    fn legacy_params_json_defaults_pathology_scales_to_canon() {
        let json = serde_json::to_string(&SimParameters::default()).unwrap();
        let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = v.as_object_mut().unwrap();
        obj.remove("pathology_growth_scale");
        obj.remove("pathology_decay_scale");
        obj.remove("pathology_ceiling_scale");
        obj.remove("goal_gate_scale");
        let restored: SimParameters = serde_json::from_value(v).unwrap();
        assert_eq!(restored.pathology_growth_scale, Fixed::ONE);
        assert_eq!(restored.pathology_decay_scale, Fixed::ONE);
        assert_eq!(restored.pathology_ceiling_scale, Fixed::ONE);
        assert_eq!(restored.goal_gate_scale, Fixed::ONE);
    }

    /// Serde back-compat: a param payload serialized before i303 (no
    /// `difficulty` key) loads at the Standard band — the same serde-default
    /// pattern every Snapshot field addition has used since v8.
    #[test]
    fn legacy_params_json_defaults_to_standard_band() {
        let mut p = SimParameters::default();
        p.difficulty = DifficultyProfile::Harsh;
        let json = serde_json::to_string(&p).unwrap();
        // Simulate pre-i303 bytes: strip the difficulty key.
        let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
        v.as_object_mut().unwrap().remove("difficulty");
        let restored: SimParameters = serde_json::from_value(v).unwrap();
        assert_eq!(restored.difficulty, DifficultyProfile::Standard);
        assert_eq!(
            restored.hunger_decay_rate,
            SimParameters::default().hunger_decay_rate
        );
    }
}
