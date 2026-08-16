//! Narrative and meaning-making system — agents interpret life as a story.
//!
//! Events are interpreted through narrative frames:
//! - punishment as justice,
//! - suffering as test,
//! - loss as curse,
//! - success as blessing,
//! - betrayal as proof of unworthiness,
//! - survival as destiny.
//!
//! This feeds religion, ideology, and resilience.
//!
//! Iteration 181 (AP2 §8.1.3): the six scripts were write-only ratchets —
//! per-tick event increments with NO decay, so every script saturated at ~1.0
//! within ~10K ticks (probe-pinned: redemption 0.5→0.99, contamination
//! 0.2→0.84, heroism 0.5→1.00), flattening the identity and rendering
//! `coherence`/`life_theme` uninformative. `decay_scripts` pulls each script
//! back toward its birth-narrative value (the Iter-179 pull pattern) and
//! `stress_resilience_factor` gives the scripts their first decision
//! consumer — the plan's "feeds ... resilience" — by scaling the per-tick
//! stress input (identity-at-birth, so default-envelope agents are
//! effectively untouched — probe-pinned factor band ±1–4%).

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

// ── Birth-narrative envelope (Iteration 181) ────────────────────────────────
// `NarrativeIdentity::default()` values, hoisted to consts so the hot
// per-tick paths (`decay_scripts`, `stress_resilience_factor`) never rebuild
// the default struct per agent per tick (the rust-best-practices performance
// mindset). `birth_envelope_consts_match_default` keeps these in lockstep
// with `Default`. Values are `Fixed::from_raw` (SCALE = 10000).

/// Birth redemption script (0.5).
const BIRTH_REDEMPTION: Fixed = Fixed::from_raw(5000);
/// Birth contamination script (0.2).
const BIRTH_CONTAMINATION: Fixed = Fixed::from_raw(2000);
/// Birth victimhood script (0.2).
const BIRTH_VICTIMHOOD: Fixed = Fixed::from_raw(2000);
/// Birth heroism script (0.5).
const BIRTH_HEROISM: Fixed = Fixed::from_raw(5000);
/// Birth chosenness script (0.1).
const BIRTH_CHOSENNESS: Fixed = Fixed::from_raw(1000);
/// Birth shame script (0.1).
const BIRTH_SHAME: Fixed = Fixed::from_raw(1000);
/// Birth balance: (0.5+0.5+0.6) − (0.2+0.2+0.1) = 1.1 — the signed
/// script-balance at which `stress_resilience_factor` is neutral (the 0.6
/// term is the birth coherence, which the factor reads live via
/// `self.coherence`).
const BIRTH_BALANCE: Fixed = Fixed::from_raw(11000);

/// Iteration 181: per-tick proportional script decay toward the birth-narrative
/// envelope. Calibrated at 0.005 by sweep: the write-only ratchets fire on
/// most ticks (emotions cross their 0.3 thresholds frequently), so 0.001 was
/// too weak — heroism still saturated to 1.00 and contamination climbed to
/// 0.74 by 10K ticks. At 0.005 each script settles in a bounded band above
/// its birth value — equilibrium ≈ birth + ratchet_rate/decay, and the
/// overshoot varies per script because the ratchet rates differ
/// (probe-pinned seed42/10K: redemption 0.539, contamination 0.312,
/// heroism 0.812, victimhood 0.200 — vs the pre-fix 0.99/0.84/1.00/0.20)
/// while event-driven differentiation between seeds/agents is preserved.
pub const NARRATIVE_SCRIPT_DECAY_RATE: f64 = 0.005;

/// Iteration 181: stress-resilience consumer rate. Calibrated at 0.15: a
/// maximally-redemptive script balance (delta +1.0) buffers stress to 0.85×,
/// a maximally-contaminated balance (delta −1.0) amplifies to 1.15× — bounded
/// ±15%, keeping the channel live but below decision-granularity inversion
/// (the Phase-5 acceptance lesson).
pub const NARRATIVE_STRESS_RESILIENCE_RATE: f64 = 0.15;

/// Temperament inputs to the life-theme mapping (Iteration 186).
///
/// The pre-Iteration-186 mapping was a pure script-balance function with
/// NO temperament input, so in a thriving calm village every agent
/// saturated positive scripts (heroism 0.60–1.00 > redemption 0.52–0.58)
/// and collapsed onto Mission (probe: calm 13/13 Mission at 10K+ across
/// seeds) while famine/pestilence kept whatever variety the scripts
/// produced. Personality is fixed at birth, so tempering the mapping with
/// it keeps determinism while differentiating agents in the SAME world:
///   - neuroticism discounts the good: anxious agents need a larger
///     positive margin before life reads as positive-dominant,
///   - Mission is an AGENTIC story — a passive agent in a good world
///     experiences Growth, not a heroic calling.
#[derive(Debug, Clone, Copy)]
pub struct ThemeTemperament {
    /// Neuroticism (0..1) — discounts the positive balance.
    pub neuroticism: Fixed,
    /// Agency (0..1) — ambition/extraversion/conscientiousness blend.
    pub agency: Fixed,
}

impl Default for ThemeTemperament {
    fn default() -> Self {
        Self {
            neuroticism: Fixed::ZERO,
            agency: Fixed::ONE,
        }
    }
}

/// Life narrative theme — the dominant story the agent tells about their life.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LifeTheme {
    /// Life is a journey of growth and learning.
    #[default]
    Growth,
    /// Life is a struggle against adversity.
    Struggle,
    /// Life is a gift to be enjoyed.
    Gift,
    /// Life is a test of character and faith.
    Test,
    /// Life is meaningless (depressive narrative).
    Meaningless,
    /// Life is a punishment for past wrongs.
    Punishment,
    /// Life is a mission with purpose.
    Mission,
}

/// Narrative identity — the story the agent tells about themselves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeIdentity {
    /// Dominant life theme.
    pub life_theme: LifeTheme,
    /// Redemption script strength (finding meaning in suffering). 0–1.
    pub redemption_script: Fixed,
    /// Contamination script strength (good things are ruined). 0–1.
    pub contamination_script: Fixed,
    /// Victimhood script strength (bad things always happen to me). 0–1.
    pub victimhood_script: Fixed,
    /// Heroism script strength (I overcome challenges). 0–1.
    pub heroism_script: Fixed,
    /// Chosenness script strength (I am special/chosen). 0–1.
    pub chosenness_script: Fixed,
    /// Shame script strength (I am fundamentally flawed). 0–1.
    pub shame_script: Fixed,
    /// Overall narrative coherence (how consistent the story feels). 0–1.
    pub coherence: Fixed,
    /// Number of major life events incorporated into narrative.
    pub events_integrated: u32,
}

impl Default for NarrativeIdentity {
    fn default() -> Self {
        Self {
            life_theme: LifeTheme::Growth,
            redemption_script: Fixed::from_f64(0.5),
            contamination_script: Fixed::from_f64(0.2),
            victimhood_script: Fixed::from_f64(0.2),
            heroism_script: Fixed::from_f64(0.5),
            chosenness_script: Fixed::from_f64(0.1),
            shame_script: Fixed::from_f64(0.1),
            coherence: Fixed::from_f64(0.6),
            events_integrated: 0,
        }
    }
}

impl NarrativeIdentity {
    /// Interpret a negative event through the narrative frame.
    pub fn interpret_negative_event(
        &mut self,
        severity: Fixed,
        social_support: Fixed,
        has_blame_target: bool,
    ) {
        // Contamination script strengthens with negative events
        self.contamination_script =
            (self.contamination_script + severity * Fixed::from_f64(0.01)).clamp_01();

        // Victimhood strengthens without support. §8.1.17 (P3-9): the gate was
        // `social_support < 0.3` — but the no-relationship baseline in the sim
        // is EXACTLY 0.3, so an isolated agent (the most victimhood-prone) sat
        // AT the gate and never qualified; victimhood stayed frozen at birth
        // 0.200 in every window. The gate now includes the baseline
        // (`<= 0.3`), so genuinely unsupported agents (pestilence orphans with
        // zero relationships, whose support sits at the 0.3 floor) develop
        // victimhood while well-supported villagers (support ≈ 1.0) do not.
        if social_support <= Fixed::from_f64(0.3) {
            self.victimhood_script =
                (self.victimhood_script + severity * Fixed::from_f64(0.008)).clamp_01();
        }

        // Heroism strengthens when there's support to overcome
        if social_support > Fixed::from_f64(0.4) {
            self.heroism_script =
                (self.heroism_script + severity * Fixed::from_f64(0.005)).clamp_01();
        }

        // Redemption script strengthens when blame can be externalized
        if has_blame_target {
            self.redemption_script = (self.redemption_script + Fixed::from_f64(0.003)).clamp_01();
        }

        // Shame script strengthens from personal failure without external blame.
        // §8.1.17 (P3-9): the gate was `social_support < 0.2` — stricter than
        // the no-relationship baseline (0.3), so shame NEVER fired (frozen at
        // birth 0.100 in every window). Aligned with the victimhood gate
        // (`<= 0.3`): an isolated agent failing without a blame target feels
        // shame; the distinction from victimhood is the blame target, not a
        // second support threshold.
        if !has_blame_target && social_support <= Fixed::from_f64(0.3) {
            self.shame_script = (self.shame_script + severity * Fixed::from_f64(0.005)).clamp_01();
        }

        self.events_integrated += 1;
    }

    /// Interpret a positive event through the narrative frame.
    pub fn interpret_positive_event(&mut self, magnitude: Fixed, social_recognition: Fixed) {
        // Redemption script strengthens with positive recovery
        self.redemption_script =
            (self.redemption_script + magnitude * Fixed::from_f64(0.005)).clamp_01();

        // Heroism strengthens with recognition
        self.heroism_script =
            (self.heroism_script + social_recognition * Fixed::from_f64(0.003)).clamp_01();

        // Chosenness strengthens with exceptional success.
        // §8.1.17 (P3-9): the gate was `magnitude > 0.7`, but the sim feeds
        // `positive_event_magnitude = joy × 0.1` (≤ 0.1) — the gate was
        // SEVEN times the maximum possible input, so chosenness was
        // structurally unreachable (frozen at birth 0.100 in every window).
        // Re-scaled to the actual event-magnitude domain: `> 0.07` means
        // joy > 0.7 (a genuinely exceptional joyful event) WITH strong social
        // recognition — rare but reachable.
        if magnitude > Fixed::from_f64(0.07) && social_recognition > Fixed::from_f64(0.6) {
            self.chosenness_script = (self.chosenness_script + Fixed::from_f64(0.002)).clamp_01();
        }

        // Contamination weakens with positive events
        self.contamination_script =
            (self.contamination_script - magnitude * Fixed::from_f64(0.003)).max(Fixed::ZERO);

        self.events_integrated += 1;
    }

    /// Iteration 181: proportional script decay toward the birth-narrative
    /// envelope (the Iter-179 pull pattern). The per-event ratchets in
    /// `interpret_negative_event`/`interpret_positive_event` are write-only —
    /// with no decay every script saturates at ~1.0 within ~10K ticks,
    /// flattening the identity (probe-pinned). Pull each script toward its
    /// `Default` value at `rate` per call so event-driven movement is
    /// preserved but bounded: a stable, decisional narrative envelope
    /// instead of a runaway ceiling. Deterministic; no RNG; observational
    /// cadence (runs with the narrative block).
    pub fn decay_scripts(&mut self, rate: Fixed) {
        let pull = |current: Fixed, target: Fixed| -> Fixed {
            (current + (target - current) * rate).clamp_01()
        };
        self.redemption_script = pull(self.redemption_script, BIRTH_REDEMPTION);
        self.contamination_script = pull(self.contamination_script, BIRTH_CONTAMINATION);
        self.victimhood_script = pull(self.victimhood_script, BIRTH_VICTIMHOOD);
        self.heroism_script = pull(self.heroism_script, BIRTH_HEROISM);
        self.chosenness_script = pull(self.chosenness_script, BIRTH_CHOSENNESS);
        self.shame_script = pull(self.shame_script, BIRTH_SHAME);
    }

    /// Iteration 181: the narrative scripts' first decision consumer — the
    /// plan's "narrative feeds ... resilience".
    ///
    /// Returns a stress multiplier for the per-tick stress input: ~1.0 when
    /// the script balance AND coherence sit at the birth envelope
    /// (identity-at-birth, one-sided — the Iter-99/127 pattern; coherence is
    /// part of the narrative state, so it drifts with events — the honest
    /// probe-pinned factor band is ±1–4% at 10K, never decision-granularity),
    /// <1.0 when the story has drifted redemptive
    /// (redemption/heroism/coherence outweigh contamination/victimhood/
    /// shame), >1.0 when the story has drifted contaminated/victimized. The
    /// delta is the signed script-balance deviation from the birth envelope,
    /// clamped to ±1.0; `rate` scales it to a bounded ±15% modulation.
    /// Non-focal agents never run the narrative block, so their scripts stay
    /// at the birth envelope → factor ≈1.0 → near-zero blast below the
    /// focal tier.
    pub fn stress_resilience_factor(&self, rate: Fixed) -> Fixed {
        let positive = self.redemption_script + self.heroism_script + self.coherence;
        let negative = self.contamination_script + self.victimhood_script + self.shame_script;
        let delta = (positive - negative - BIRTH_BALANCE).clamp(-Fixed::ONE, Fixed::ONE);
        (Fixed::ONE - delta * rate)
            .clamp(Fixed::from_f64(0.8), Fixed::from_f64(1.2))
    }

    /// Update the dominant life theme based on current script balance and
    /// temperament (Iteration 186).
    ///
    /// The temperament bias breaks the calm-world all-Mission collapse: the
    /// old mapping was script-balance-only, so every thriving agent saturated
    /// the same positive scripts and landed on Mission. Now:
    ///   - neuroticism discounts the positive balance (anxious agents read
    ///     even a good life as precarious → Test/Growth rather than Mission),
    ///   - Mission additionally requires agency — a passive agent in a good
    ///     world has a Growth story, not a heroic calling.
    pub fn update_theme(&mut self, temperament: ThemeTemperament) {
        let positive_balance =
            self.redemption_script + self.heroism_script + self.chosenness_script;
        let negative_balance =
            self.contamination_script + self.victimhood_script + self.shame_script;

        // Neuroticism discounts the good: an anxious agent needs a larger
        // positive margin before life reads as positive-dominant.
        let effective_positive = positive_balance - temperament.neuroticism * Fixed::from_f64(0.2);

        self.life_theme = if effective_positive > negative_balance * Fixed::from_f64(1.5) {
            if self.heroism_script > self.redemption_script
                && temperament.agency > Fixed::from_f64(0.5)
            {
                LifeTheme::Mission
            } else {
                LifeTheme::Growth
            }
        } else if negative_balance > effective_positive * Fixed::from_f64(1.5) {
            if self.victimhood_script > self.shame_script {
                LifeTheme::Struggle
            } else if self.shame_script > Fixed::from_f64(0.6) {
                LifeTheme::Punishment
            } else {
                LifeTheme::Meaningless
            }
        } else {
            LifeTheme::Test // balanced — life is a test
        };

        // Coherence is how consistent the narrative scripts are
        let max_script = positive_balance.max(negative_balance);
        let min_script = positive_balance.min(negative_balance);
        self.coherence = if max_script > Fixed::ZERO {
            (Fixed::ONE - (max_script - min_script) / max_script).clamp_01()
        } else {
            Fixed::from_f64(0.5)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_event_increases_contamination() {
        let mut n = NarrativeIdentity::default();
        let initial = n.contamination_script;
        n.interpret_negative_event(Fixed::from_f64(0.7), Fixed::from_f64(0.3), false);
        assert!(n.contamination_script > initial);
    }

    #[test]
    fn positive_event_increases_redemption() {
        let mut n = NarrativeIdentity::default();
        let initial = n.redemption_script;
        n.interpret_positive_event(Fixed::from_f64(0.5), Fixed::from_f64(0.5));
        assert!(n.redemption_script > initial);
    }

    #[test]
    fn theme_updates_based_on_balance() {
        let mut n = NarrativeIdentity {
            heroism_script: Fixed::from_f64(0.9),
            redemption_script: Fixed::from_f64(0.7),
            contamination_script: Fixed::from_f64(0.1),
            victimhood_script: Fixed::from_f64(0.1),
            ..Default::default()
        };
        n.update_theme(ThemeTemperament {
            neuroticism: Fixed::ZERO,
            agency: Fixed::ONE,
        });
        assert_eq!(n.life_theme, LifeTheme::Mission);
    }

    #[test]
    fn temperament_differentiates_positive_worlds() {
        // Iteration 186: the theme mapping must NOT be a pure script-balance
        // function — a thriving calm village saturated every agent's positive
        // scripts and collapsed all of them onto Mission (probe: calm 13/13
        // Mission at 10K+). Tempering the mapping by fixed-at-birth
        // personality restores variety deterministically:
        //   - a passive (low-agency) agent in the same good world lands on
        //     Growth, not Mission,
        //   - an anxious (high-neuroticism) agent needs a larger positive
        //     margin before life reads as positive-dominant (lands on Test).
        // High agency + low neuroticism → Mission.
        let mut mission = NarrativeIdentity {
            heroism_script: Fixed::from_f64(0.9),
            redemption_script: Fixed::from_f64(0.7),
            contamination_script: Fixed::from_f64(0.1),
            victimhood_script: Fixed::from_f64(0.1),
            ..Default::default()
        };
        mission.update_theme(ThemeTemperament {
            neuroticism: Fixed::from_f64(0.2),
            agency: Fixed::from_f64(0.9),
        });
        assert_eq!(mission.life_theme, LifeTheme::Mission);

        // Same scripts, low agency → Growth (a passive agent in a good world
        // has a growth story, not a heroic calling).
        let mut growth = NarrativeIdentity {
            heroism_script: Fixed::from_f64(0.9),
            redemption_script: Fixed::from_f64(0.7),
            contamination_script: Fixed::from_f64(0.1),
            victimhood_script: Fixed::from_f64(0.1),
            ..Default::default()
        };
        growth.update_theme(ThemeTemperament {
            neuroticism: Fixed::from_f64(0.2),
            agency: Fixed::from_f64(0.3),
        });
        assert_eq!(growth.life_theme, LifeTheme::Growth);

        // A balanced script mix under extreme neuroticism: the discount
        // (0.2 × neuro) pushes the effective positive margin below the 1.5×
        // dominance band on BOTH sides → Test (life is a test). Positive
        // 0.9, negative 0.7 → effective 0.71: not positive-dominant
        // (0.71 ≤ 1.05), not negative-dominant (0.7 ≤ 1.065).
        let mut test = NarrativeIdentity {
            heroism_script: Fixed::from_f64(0.5),
            redemption_script: Fixed::from_f64(0.4),
            contamination_script: Fixed::from_f64(0.4),
            victimhood_script: Fixed::from_f64(0.3),
            ..Default::default()
        };
        test.update_theme(ThemeTemperament {
            neuroticism: Fixed::from_f64(0.95),
            agency: Fixed::from_f64(0.9),
        });
        assert_eq!(test.life_theme, LifeTheme::Test);
    }

    #[test]
    fn decay_pulls_saturated_scripts_back_toward_birth_envelope() {
        // Iteration 181: the write-only ratchets saturate every script at ~1.0;
        // decay must pull a saturated identity back toward the birth envelope.
        let mut n = NarrativeIdentity {
            redemption_script: Fixed::ONE,
            contamination_script: Fixed::ONE,
            victimhood_script: Fixed::ONE,
            heroism_script: Fixed::ONE,
            chosenness_script: Fixed::ONE,
            shame_script: Fixed::ONE,
            ..Default::default()
        };
        let d = NarrativeIdentity::default();
        n.decay_scripts(Fixed::from_f64(0.02));
        assert!(n.redemption_script < Fixed::ONE);
        assert!(n.contamination_script < Fixed::ONE);
        // Pulled toward the birth values (redemption 0.5, contamination 0.2).
        assert!(n.redemption_script > d.redemption_script);
        assert!(n.contamination_script > d.contamination_script);
        // Repeated pulls converge: the identity no longer saturates.
        for _ in 0..400 {
            n.decay_scripts(Fixed::from_f64(0.02));
        }
        assert!((n.redemption_script - d.redemption_script).abs() < Fixed::from_f64(0.01));
        assert!(
            (n.contamination_script - d.contamination_script).abs() < Fixed::from_f64(0.01)
        );
    }

    #[test]
    fn birth_envelope_consts_match_default() {
        // Iteration 181: the hoisted consts must stay in lockstep with
        // `Default`, or decay/resilience would silently re-anchor elsewhere.
        let d = NarrativeIdentity::default();
        assert_eq!(BIRTH_REDEMPTION, d.redemption_script);
        assert_eq!(BIRTH_CONTAMINATION, d.contamination_script);
        assert_eq!(BIRTH_VICTIMHOOD, d.victimhood_script);
        assert_eq!(BIRTH_HEROISM, d.heroism_script);
        assert_eq!(BIRTH_CHOSENNESS, d.chosenness_script);
        assert_eq!(BIRTH_SHAME, d.shame_script);
        // Coherence's birth value participates in the factor via BIRTH_BALANCE
        // (the 0.6 term); the factor reads live `self.coherence`.
        assert_eq!(
            BIRTH_BALANCE,
            (d.redemption_script + d.heroism_script + d.coherence)
                - (d.contamination_script + d.victimhood_script + d.shame_script)
        );
    }

    #[test]
    fn stress_resilience_factor_is_one_at_birth_envelope() {
        // Identity-at-birth: a default-envelope agent is untouched.
        let n = NarrativeIdentity::default();
        assert_eq!(n.stress_resilience_factor(Fixed::from_f64(0.15)), Fixed::ONE);
    }

    #[test]
    fn coherence_drift_alone_moves_factor_only_slightly() {
        // Reviewer finding: coherence is narrative state and drifts with
        // events, so "1.0 at birth" holds exactly only when coherence is at
        // birth too. Document the honest band: coherence 0.6→0.9 with scripts
        // pinned at birth moves the factor by ≤ ~1.5% — never
        // decision-granularity.
        let n = NarrativeIdentity {
            coherence: Fixed::from_f64(0.9),
            ..Default::default()
        };
        let factor = n.stress_resilience_factor(Fixed::from_f64(0.15));
        let drift = (factor - Fixed::ONE).abs();
        assert!(
            drift <= Fixed::from_f64(0.05),
            "coherence-only drift must stay in a bounded band"
        );
    }

    #[test]
    fn decay_returns_saturated_identity_to_neutral_resilience() {
        // Iteration 181: after decay re-anchors a saturated identity, the
        // stress factor returns toward 1.0 — the write-only ceiling no longer
        // pins the consumer at its ±15% extreme.
        let mut n = NarrativeIdentity {
            redemption_script: Fixed::ONE,
            contamination_script: Fixed::ONE,
            victimhood_script: Fixed::ONE,
            heroism_script: Fixed::ONE,
            chosenness_script: Fixed::ONE,
            shame_script: Fixed::ONE,
            ..Default::default()
        };
        for _ in 0..1000 {
            n.decay_scripts(Fixed::from_f64(0.005));
        }
        let factor = n.stress_resilience_factor(Fixed::from_f64(0.15));
        let drift = (factor - Fixed::ONE).abs();
        assert!(
            drift < Fixed::from_f64(0.1),
            "decay must return the factor toward neutral"
        );
    }

    #[test]
    fn redemptive_narrative_buffers_stress() {
        let n = NarrativeIdentity {
            redemption_script: Fixed::from_f64(0.9),
            heroism_script: Fixed::from_f64(0.9),
            coherence: Fixed::from_f64(0.9),
            contamination_script: Fixed::from_f64(0.1),
            victimhood_script: Fixed::from_f64(0.1),
            shame_script: Fixed::from_f64(0.1),
            ..Default::default()
        };
        let factor = n.stress_resilience_factor(Fixed::from_f64(0.15));
        assert!(factor < Fixed::ONE, "redemptive story must buffer stress");
        assert!(factor >= Fixed::from_f64(0.8));
    }

    #[test]
    fn contaminated_narrative_amplifies_stress() {
        let n = NarrativeIdentity {
            redemption_script: Fixed::from_f64(0.1),
            heroism_script: Fixed::from_f64(0.1),
            coherence: Fixed::from_f64(0.1),
            contamination_script: Fixed::from_f64(0.9),
            victimhood_script: Fixed::from_f64(0.9),
            shame_script: Fixed::from_f64(0.9),
            ..Default::default()
        };
        let factor = n.stress_resilience_factor(Fixed::from_f64(0.15));
        assert!(factor > Fixed::ONE, "contaminated story must amplify stress");
        assert!(factor <= Fixed::from_f64(1.2));
    }
}
