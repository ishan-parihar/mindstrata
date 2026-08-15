//! Neurological / Nervous system — arousal, autonomic regulation, pain, trauma.
//!
//! Central to making agents feel alive. The nervous system modulates:
//! - attention narrowing under stress
//! - threat sensitivity
//! - social engagement capacity
//! - pain processing
//! - trauma accumulation and effects

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Pain meaning — what pain signifies to the agent.
/// Pain meaning affects emotional response: battle pain → pride, punishment → resentment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PainMeaning {
    /// Pain from labor/effort — expected, tolerable.
    Labor,
    /// Pain from battle — may produce pride or trauma.
    Battle,
    /// Pain from punishment — produces resentment.
    Punishment,
    /// Pain from childbirth — bonding or trauma.
    Childbirth,
    /// Pain from illness — produces fear.
    Illness,
    /// Unknown/unattributed pain.
    Unknown,
}

/// Pain state — more than just injury level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PainState {
    /// Acute pain (0 = none, 1 = excruciating).
    pub acute: Fixed,
    /// Chronic pain (persists after acute resolves).
    pub chronic: Fixed,
    /// How much attention pain commands (0 = ignored, 1 = dominating).
    pub attention_demand: Fixed,
    /// What the agent believes caused the pain.
    pub meaning: PainMeaning,
}

impl Default for PainState {
    fn default() -> Self {
        Self {
            acute: Fixed::ZERO,
            chronic: Fixed::ZERO,
            attention_demand: Fixed::ZERO,
            meaning: PainMeaning::Unknown,
        }
    }
}

impl PainState {
    /// Update pain levels each tick.
    pub fn update(&mut self, injury_input: Fixed) {
        // Acute pain from injury
        self.acute =
            (self.acute * Fixed::from_f64(0.95) + injury_input * Fixed::from_f64(0.3)).clamp_01();
        // Chronic pain accumulates from repeated acute pain
        if self.acute > Fixed::from_f64(0.5) {
            self.chronic = (self.chronic + Fixed::from_f64(0.0005)).clamp_01();
        } else {
            self.chronic = (self.chronic - Fixed::from_f64(0.0002)).max(Fixed::ZERO);
        }
        // Attention demand follows pain level
        self.attention_demand =
            (self.acute * Fixed::from_f64(0.7) + self.chronic * Fixed::from_f64(0.3)).clamp_01();
    }

    /// Total effective pain for cognitive modulation.
    pub fn effective_pain(&self) -> Fixed {
        (self.acute + self.chronic * Fixed::from_f64(0.5)).clamp_01()
    }
}

/// Sensory acuity — how sensitive the agent's senses are.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensoryAcuity {
    /// Visual acuity.
    pub visual: Fixed,
    /// Auditory acuity.
    pub auditory: Fixed,
    /// Overall perceptual sensitivity.
    pub general: Fixed,
}

impl Default for SensoryAcuity {
    fn default() -> Self {
        Self {
            visual: Fixed::from_f64(0.5),
            auditory: Fixed::from_f64(0.5),
            general: Fixed::from_f64(0.5),
        }
    }
}

/// Tunable parameters for nervous system update.
/// Floor on the parasympathetic tone used for sympathetic recovery.
/// Iteration 176: without a floor, sustained threat drains `parasympathetic_tone`
/// toward 0 (drain = arousal × 0.05), which zeroes `recovery = tone × rate`
/// — a structural deadlock where arousal can never calm down once the tone
/// is exhausted. Mirror of the Iteration 172 `STRESS_RECOVERY_TONE_FLOOR`
/// fix on the endocrine stress axis: a floor of 0.3 with the 0.1 recovery
/// rate guarantees a basal recovery channel even at full activation.
/// Defensive rather than load-bearing in the probed windows (aggregate
/// arousal was byte-identical with/without it — the observed trauma ratchet
/// was fixed by the proportional-decay change below) — but without it the
/// channel CAN deadlock in threat-then-safety sequences where the tone has
/// drained to zero, which the targeted unit test pins.
pub const NERVOUS_TONE_RECOVERY_FLOOR: f64 = 0.3;
/// [`NERVOUS_TONE_RECOVERY_FLOOR`] pre-converted to [`Fixed`] (hoisted out of
/// the per-tick per-agent `update` hot path; 0.3 × 10_000 = 3_000).
pub const NERVOUS_TONE_RECOVERY_FLOOR_FIXED: Fixed = Fixed::from_raw(3_000);
/// Ceiling on the parasympathetic tone used for sympathetic recovery
/// (S2-2-1 completion). The symmetric counterpart of
/// [`NERVOUS_TONE_RECOVERY_FLOOR`]: with the logistic tone drain
/// (`drain = coeff × tone`, below) the tone→recovery→arousal→drain coupling
/// is a POSITIVE feedback — tone↑ raises recovery, which lowers arousal,
/// which shrinks the drain coefficient, which lets tone climb further — so
/// an unbounded recovery term ratchets tone to 1.0 and drains arousal to 0
/// (the mirror-image pin; unit-pinned). Capping the recovery tone at 0.6
/// bounds recovery to [0.03, 0.06], giving the axis a stable interior
/// equilibrium: sustained threat 0.5/safety 0.5 converges tone ≈0.67 and
/// arousal ≈0.40 (neither pinned), famine sits low ≈0.08, and true safety
/// still builds tone toward 1.0 (rest/digest).
pub const NERVOUS_TONE_RECOVERY_CEILING: f64 = 0.6;
/// [`NERVOUS_TONE_RECOVERY_CEILING`] pre-converted to [`Fixed`].
pub const NERVOUS_TONE_RECOVERY_CEILING_FIXED: Fixed = Fixed::from_raw(6_000);

///
/// Grouped into a `Copy` struct to avoid transposition-prone positional args
/// (Apollo Rust best practices Ch. 1: prefer structured data over positional
/// args of the same type).
#[must_use]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NervousUpdateParams {
    /// Sympathetic recovery rate in safety (higher = faster calm-down). Range: 0.01–0.3, default 0.1.
    pub sympathetic_recovery_rate: Fixed,
    /// Parasympathetic buildup rate in safety. Range: 0.01–0.2, default 0.06.
    pub parasympathetic_buildup_rate: Fixed,
    /// Trauma accumulation rate from sustained high arousal. Range: 0.0001–0.001, default 0.0003.
    pub trauma_accumulation_rate: Fixed,
    /// Trauma decay fraction per tick (proportional since Iteration 176).
    /// Range: 0.0001–0.005, default 0.0005. The pre-fix range (0.00001–
    /// 0.0002) was entirely below 4-decimal Fixed resolution (0.00001 → raw
    /// 0) — the knob was a dead subtractive knife-edge.
    pub trauma_decay_rate: Fixed,
}

impl Default for NervousUpdateParams {
    fn default() -> Self {
        Self {
            sympathetic_recovery_rate: Fixed::from_f64(0.1),
            parasympathetic_buildup_rate: Fixed::from_f64(0.06),
            trauma_accumulation_rate: Fixed::from_f64(0.0003),
            trauma_decay_rate: Fixed::from_f64(0.0005),
        }
    }
}

/// Nervous system state — autonomic regulation, arousal, pain, trauma.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NervousSystemState {
    /// Sympathetic activation (fight/flight). High = narrowed attention, threat-biased.
    pub sympathetic_arousal: Fixed,
    /// Parasympathetic tone (rest/digest). High = social engagement, trust, recovery.
    pub parasympathetic_tone: Fixed,
    /// Baseline arousal level.
    pub baseline_arousal: Fixed,
    /// Sensory acuity.
    pub sensory: SensoryAcuity,
    /// Current pain state.
    pub pain: PainState,
    /// Pain sensitivity — genetic + developmental.
    pub pain_sensitivity: Fixed,
    /// Accumulated trauma load.
    pub trauma_load: Fixed,
    /// Dissociation risk (extreme trauma response).
    pub dissociation_risk: Fixed,
    /// Startle sensitivity — elevated by trauma.
    pub startle_sensitivity: Fixed,
    /// Neuroplasticity — capacity for learning and change.
    pub neuroplasticity: Fixed,
    /// Sleep pressure — builds without sleep.
    pub sleep_pressure: Fixed,
}

impl Default for NervousSystemState {
    fn default() -> Self {
        Self {
            sympathetic_arousal: Fixed::from_f64(0.3),
            parasympathetic_tone: Fixed::from_f64(0.5),
            baseline_arousal: Fixed::from_f64(0.3),
            sensory: SensoryAcuity::default(),
            pain: PainState::default(),
            pain_sensitivity: Fixed::from_f64(0.5),
            trauma_load: Fixed::ZERO,
            dissociation_risk: Fixed::ZERO,
            startle_sensitivity: Fixed::from_f64(0.3),
            neuroplasticity: Fixed::from_f64(0.6),
            sleep_pressure: Fixed::ZERO,
        }
    }
}

impl NervousSystemState {
    /// Update nervous system each tick.
    pub fn update(
        &mut self,
        threat_input: Fixed,
        safety_input: Fixed,
        injury_input: Fixed,
        sleep_tick: bool,
        params: NervousUpdateParams,
    ) {
        // Sympathetic activation from threat.
        // §7.2.5 (S2-2-1 fix): the rise term was `threat × 0.2` against a
        // floored recovery of `tone(≥0.3) × 0.1 = 0.03`. At the calibrated
        // fear equilibrium (~0.3-0.5 in every scenario, including calm), the
        // net was +0.03..+0.07 per tick — the sympathetic channel ratcheted
        // to 1.0 for 9-11/12 agents in every probe window (probe-pinned), and
        // the parasympathetic tone drained to 0 in lockstep. A linear
        // floor/rate calibration cannot bound the axis below 1.0 (rise scales
        // with threat while floored recovery is constant), so the rise is now
        // SATURATING — `threat × 0.2 × (1 − arousal)` — the same logistic
        // pattern as the Iteration-176 trauma decay, the S2-2-2 bonding fix,
        // and the S2-2-3 fitness/conditioning fix. The axis now converges to
        // a stable equilibrium below 1.0 (≈0.63 at threat 0.5 with the 0.1
        // recovery rate), leaving headroom for genuine threat spikes while
        // never pinning. Zero change at zero threat; deterministic, no RNG.
        let sympathetic_delta =
            threat_input * Fixed::from_f64(0.2) * (Fixed::ONE - self.sympathetic_arousal);
        // Iteration 176: floor the parasympathetic tone used for recovery so
        // sustained threat cannot drain it to 0 and zero the recovery channel
        // (the pre-fix trauma ratchet — see NERVOUS_TONE_RECOVERY_FLOOR doc).
        // S2-2-1 completion: also CEIL it (NERVOUS_TONE_RECOVERY_CEILING) —
        // see that constant's doc for the positive-feedback argument.
        let recovery_tone = self
            .parasympathetic_tone
            .max(NERVOUS_TONE_RECOVERY_FLOOR_FIXED)
            .min(NERVOUS_TONE_RECOVERY_CEILING_FIXED);
        let sympathetic_recovery = recovery_tone * params.sympathetic_recovery_rate;
        self.sympathetic_arousal =
            (self.sympathetic_arousal + sympathetic_delta - sympathetic_recovery).clamp_01();

        // Parasympathetic recovery from safety
        let parasympathetic_buildup = safety_input * params.parasympathetic_buildup_rate;
        // §7.2.5 (S2-2-1 fix): the tone drain was `arousal × 0.05` ONLY. With
        // the saturating sympathetic rise (below), sustained threat no longer
        // pins arousal at 1.0 — so the arousal-coupled drain collapsed and the
        // tone ratcheted UP to 1.0 under threat, which over-recovered the
        // sympathetic axis and drained it toward 0 (the mirror-image pin).
        // Threat now drains the tone directly (`threat × 0.05`, same magnitude
        // as the arousal term at the old pinned level), restoring the
        // fight-or-flight premise: sustained threat keeps the parasympathetic
        // tone low (the Iteration-176 floor then supplies basal recovery),
        // and genuine safety builds it. Deterministic, no RNG.
        //
        // (S2-2-1 completion — the drain is now ALSO proportional to the tone
        // itself: `drain_coeff × tone`, the same logistic/saturating pattern
        // as the sympathetic rise. The pre-fix form was a pure accumulator —
        // both buildup and drain exogenous, no tone feedback — so the axis
        // banged to 0 or 1 with no stable middle (probe: 8-11/12 agents at
        // tone 0.0 in EVERY window including calm). With the proportional
        // drain the axis converges to the interior equilibrium
        // `tone* = buildup / drain_coeff`: calm (low threat) settles in
        // rest/digest dominance, famine in fight/flight, with no majority
        // pinning at either extreme.)
        let drain_coeff = self.sympathetic_arousal * Fixed::from_f64(0.05)
            + threat_input * Fixed::from_f64(0.05);
        let parasympathetic_drain = drain_coeff * self.parasympathetic_tone;
        self.parasympathetic_tone = (self.parasympathetic_tone + parasympathetic_buildup
            - parasympathetic_drain)
            .clamp_01();

        // Pain update
        self.pain.update(injury_input);

        // Trauma load accumulation from sustained high sympathetic arousal
        if self.sympathetic_arousal > Fixed::from_f64(0.7) {
            self.trauma_load = (self.trauma_load + params.trauma_accumulation_rate).clamp_01();
        }
        // Trauma load decays proportionally (Iteration 176). The pre-fix
        // subtractive decay was a bang-bang knife-edge: any agent hot
        // (arousal > 0.7) more than ~33% of ticks saturated to the 1.0 cap
        // while everyone else drained to zero, and the documented range
        // (0.00001–0.0002) collapsed to 0.0001–0.0002 in 4-decimal Fixed —
        // the "recovery" knob had no working tuning range at all (probe:
        // 0.00005 ≡ 0.00010 ≡ default envelope, 0.0003 nukes trauma to 0).
        // Proportional decay (the Iteration 164 base-emotion pattern)
        // converges every agent to `hot_fraction × accumulation / rate`,
        // so the knob continuously tunes the whole envelope and trauma
        // differentiates instead of saturating.
        // Computed in f64 for the same reason `should_birth` does (the
        // 1/35040 precedent): rates in the 0.0001–0.005 range are at the
        // edge of 4-decimal Fixed resolution, and round-tripping through
        // Fixed per tick would silently underflow the decay fraction.
        let load = self.trauma_load.to_f64();
        let decay = load * params.trauma_decay_rate.to_f64();
        self.trauma_load = Fixed::from_f64((load - decay).max(0.0));

        // Dissociation risk from extreme trauma
        self.dissociation_risk = (self.trauma_load * Fixed::from_f64(0.8)
            + self.pain.effective_pain() * Fixed::from_f64(0.2))
        .clamp_01();

        // Startle sensitivity elevated by trauma
        self.startle_sensitivity =
            (Fixed::from_f64(0.3) + self.trauma_load * Fixed::from_f64(0.5)).clamp_01();

        // Sleep pressure
        if sleep_tick {
            self.sleep_pressure = (self.sleep_pressure - Fixed::from_f64(0.1)).max(Fixed::ZERO);
        } else {
            self.sleep_pressure = (self.sleep_pressure + Fixed::from_f64(0.001)).clamp_01();
        }

        // Neuroplasticity reduced by chronic stress
        self.neuroplasticity = (Fixed::from_f64(0.6)
            - self.trauma_load * Fixed::from_f64(0.3)
            - self.pain.chronic * Fixed::from_f64(0.1))
        .clamp_01();
    }

    /// Effective arousal for cognitive modulation.
    /// High arousal narrows attention, increases threat bias.
    pub fn effective_arousal(&self) -> Fixed {
        self.sympathetic_arousal
    }

    /// Social engagement capacity — requires parasympathetic dominance.
    pub fn social_engagement_capacity(&self) -> Fixed {
        self.parasympathetic_tone
    }

    /// Attention narrowing factor from arousal.
    /// Returns 0–1 where 1 = fully narrowed (tunnel vision).
    pub fn attention_narrowing(&self) -> Fixed {
        (self.sympathetic_arousal * Fixed::from_f64(0.8)).clamp_01()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pain_decays_without_input() {
        let mut pain = PainState {
            acute: Fixed::from_f64(0.8),
            ..PainState::default()
        };
        pain.update(Fixed::ZERO);
        assert!(pain.acute < Fixed::from_f64(0.8));
    }

    #[test]
    fn pain_increases_with_injury() {
        let mut pain = PainState::default();
        pain.update(Fixed::from_f64(0.9));
        assert!(pain.acute > Fixed::ZERO);
    }

    #[test]
    fn trauma_accumulates_from_sustained_arousal() {
        let mut ns = NervousSystemState::default();
        let params = NervousUpdateParams::default();
        for _ in 0..100 {
            ns.update(
                Fixed::from_f64(0.9),
                Fixed::ZERO,
                Fixed::ZERO,
                false,
                params,
            );
        }
        assert!(ns.trauma_load > Fixed::ZERO);
    }

    #[test]
    fn parasympathetic_recoveres_in_safety() {
        let mut ns = NervousSystemState {
            parasympathetic_tone: Fixed::from_f64(0.1),
            sympathetic_arousal: Fixed::from_f64(0.8),
            ..NervousSystemState::default()
        };
        let params = NervousUpdateParams::default();
        for _ in 0..50 {
            ns.update(
                Fixed::ZERO,
                Fixed::from_f64(0.8),
                Fixed::ZERO,
                false,
                params,
            );
        }
        assert!(ns.parasympathetic_tone > Fixed::from_f64(0.1));
    }

    #[test]
    fn social_engagement_requires_parasympathetic() {
        let ns_high = NervousSystemState {
            parasympathetic_tone: Fixed::from_f64(0.8),
            ..NervousSystemState::default()
        };
        assert!(ns_high.social_engagement_capacity() > Fixed::from_f64(0.5));
        let ns_low = NervousSystemState {
            parasympathetic_tone: Fixed::from_f64(0.1),
            ..NervousSystemState::default()
        };
        assert!(ns_low.social_engagement_capacity() < Fixed::from_f64(0.3));
    }

    #[test]
    fn trauma_decays_proportionally_not_subtractively() {
        // Iteration 176: proportional decay must remove a FRACTION of the
        // current load, so a small load decays slower in absolute terms than
        // a large load — the pre-fix subtractive knife-edge (0.0001/tick,
        // which saturated anyone hot >33% of ticks and could not be tuned in
        // 4-decimal Fixed) is replaced by a continuously tunable envelope.
        let mut ns = NervousSystemState {
            trauma_load: Fixed::from_f64(0.5),
            ..NervousSystemState::default()
        };
        let params = NervousUpdateParams {
            trauma_decay_rate: Fixed::from_f64(0.001),
            ..NervousUpdateParams::default()
        };
        ns.update(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO, false, params);
        let after_one = ns.trauma_load;
        assert!(
            after_one < Fixed::from_f64(0.5),
            "proportional decay must lower the load"
        );
        assert!(
            after_one > Fixed::from_f64(0.49),
            "one tick at 0.1% must remove only ~0.1% (got {after_one})"
        );
        // Half-life: 0.5 -> ~0.25 needs ln(2)/ln(1/(1-0.001)) ~ 693 ticks.
        for _ in 0..700 {
            ns.update(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO, false, params);
        }
        assert!(
            ns.trauma_load < Fixed::from_f64(0.3),
            "700 ticks at 0.1% proportional decay must halve the load"
        );
    }

    #[test]
    fn trauma_recovery_rate_is_continuously_tunable() {
        // Iteration 176: the recovery knob must produce distinct equilibrium
        // levels across its range — the pre-fix subtractive knob was
        // byte-identical from 0.00005 to 0.0002 and nuked trauma to zero at
        // 0.0003 (no working range). Two agents under identical sustained
        // threat with different decay rates must converge to different loads.
        let mk = |decay: f64| {
            let mut ns = NervousSystemState::default();
            let params = NervousUpdateParams {
                trauma_decay_rate: Fixed::from_f64(decay),
                ..NervousUpdateParams::default()
            };
            for _ in 0..5000 {
                ns.update(
                    Fixed::from_f64(0.9),
                    Fixed::from_f64(0.1),
                    Fixed::ZERO,
                    false,
                    params,
                );
            }
            ns.trauma_load
        };
        let slow = mk(0.0005);
        let fast = mk(0.002);
        assert!(
            slow > fast,
            "lower decay rate must retain more trauma: slow {slow} vs fast {fast}"
        );
        assert!(
            slow < Fixed::ONE && fast < Fixed::ONE,
            "neither may saturate: slow {slow} fast {fast}"
        );
        assert!(
            slow > Fixed::from_f64(0.1),
            "slow decay must retain a meaningful load: {slow}"
        );
    }

    #[test]
    fn sympathetic_arousal_does_not_pin_under_sustained_threat() {
        // S2-2-1 regression: the old linear rise (`threat × 0.2`) against
        // the floored recovery (0.03) ratcheted sympathetic arousal to 1.0
        // for the majority in every scenario — probe: 9-11/12 agents pinned
        // in calm runs too. The saturating rise converges to a stable
        // equilibrium well below 1.0 under a sustained threat level.
        let mut ns = NervousSystemState::default();
        let params = NervousUpdateParams::default();
        // Sustained moderate threat (the calibrated fear mean ~0.5).
        for _ in 0..5000 {
            ns.update(
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.5),
                Fixed::ZERO,
                false,
                params,
            );
        }
        let after = ns.sympathetic_arousal.to_f64();
        assert!(
            after < 0.95,
            "saturating rise must keep sympathetic arousal below 1.0 (got {after})"
        );
        assert!(
            after > 0.2,
            "sustained threat must still register meaningfully (got {after})"
        );
        // And it is stable: a further block does not creep higher.
        let before = ns.sympathetic_arousal;
        for _ in 0..1000 {
            ns.update(
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.5),
                Fixed::ZERO,
                false,
                params,
            );
        }
        assert!(ns.sympathetic_arousal <= before + Fixed::from_f64(0.01));
    }

    #[test]
    fn parasympathetic_tone_drains_under_threat() {
        // S2-2-1 regression (mirror-image): with the saturating sympathetic
        // rise, a threat-only drain on tone (the old arousal-only drain)
        // collapsed to nothing and the tone ratcheted to 1.0 under sustained
        // threat — over-recovering the sympathetic axis. The threat-coupled
        // drain keeps tone low while danger persists (the Iteration-176 floor
        // then supplies basal recovery).
        let mut ns = NervousSystemState::default();
        let params = NervousUpdateParams::default();
        // Sustained threat with genuine (not maximal) safety: tone must NOT
        // climb to 1.0 — it should hover low (floor-sustained) instead.
        for _ in 0..5000 {
            ns.update(
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.5),
                Fixed::ZERO,
                false,
                params,
            );
        }
        let after = ns.parasympathetic_tone.to_f64();
        assert!(
            after < 0.8,
            "sustained threat must keep parasympathetic tone from pinning (got {after})"
        );
        // True safety (no threat) still builds the tone toward engagement.
        let mut safe = NervousSystemState::default();
        for _ in 0..5000 {
            safe.update(Fixed::ZERO, Fixed::ONE, Fixed::ZERO, false, params);
        }
        assert!(
            safe.parasympathetic_tone > Fixed::from_f64(0.5),
            "safety must build parasympathetic tone: {}",
            safe.parasympathetic_tone
        );
    }

    #[test]
    fn tone_recovery_floor_prevents_arousal_pinning() {
        // Iteration 176: the parasympathetic tone used for sympathetic
        // recovery is floored, so sustained threat cannot drain the tone to
        // zero and zero the recovery channel (the pre-fix trauma ratchet).
        // An agent with zero tone under threat must still see its arousal
        // fall once threat recedes (recovery = floored_tone x rate).
        let mut ns = NervousSystemState {
            parasympathetic_tone: Fixed::ZERO,
            sympathetic_arousal: Fixed::from_f64(0.9),
            ..NervousSystemState::default()
        };
        let params = NervousUpdateParams::default();
        // Threat recedes to a low level; safety stays low so the tone itself
        // cannot rebuild — only the floor can supply recovery.
        for _ in 0..50 {
            ns.update(Fixed::from_f64(0.05), Fixed::ZERO, Fixed::ZERO, false, params);
        }
        assert!(
            ns.sympathetic_arousal < Fixed::from_f64(0.9),
            "floored recovery must pull arousal down from 0.9: {}",
            ns.sympathetic_arousal
        );
    }
}
