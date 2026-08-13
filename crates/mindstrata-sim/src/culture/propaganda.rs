//! Propaganda system — institutional narrative campaigns.
//!
//! Institutions can intentionally shape narratives through propaganda.
//! Propaganda uses multiple channels (sermons, edicts, rituals, gossip)
//! to spread institutional messages and compete with民间 narratives.
//!
//! ```text
//! Propaganda effectiveness:
//!   effectiveness =
//!       institutional_legitimacy
//!     × messenger_trust
//!     × repetition
//!     × emotional_fit
//!     × identity_congruence
//!     × audience_fear
//!     - counter_narrative_strength
//!     - hypocrisy_evidence
//! ```
//!
//! Emergent effects:
//! - Legitimate institutions spread propaganda effectively
//! - Corrupt institutions face backlash
//! - Counter-propaganda creates polarization
//! - Repetition creates belief through familiarity
//! - Fear amplifies propaganda reception

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Channel through which propaganda is delivered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropagandaChannel {
    /// Religious sermons or teachings.
    Sermon,
    /// Official edicts or proclamations.
    Edict,
    /// Public rituals or ceremonies.
    Ritual,
    /// Market announcements or economic messaging.
    Market,
    /// Trusted messenger networks.
    Messenger,
    /// Punishment examples (deterrence).
    PunishmentExample,
    /// Songs, chants, or oral traditions.
    Song,
    /// Education or training.
    Education,
}

/// A propaganda campaign sponsored by an institution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagandaCampaign {
    /// Unique campaign identifier.
    pub id: usize,
    /// Index of the sponsoring institution.
    pub sponsor: usize,
    /// Target audience (agent indices).
    pub targets: Vec<usize>,
    /// Description of the narrative being promoted.
    pub narrative: String,
    /// Intensity of the campaign (0 = low effort, 1 = full saturation).
    pub intensity: Fixed,
    /// Channels used for delivery.
    pub channels: Vec<PropagandaChannel>,
    /// Base credibility of the campaign.
    pub credibility: Fixed,
    /// Coercion level (0 = pure persuasion, 1 = threats).
    pub coercion: Fixed,
    /// Duration in ticks.
    pub duration: u64,
    /// Remaining duration.
    pub remaining: u64,
    /// Resistance encountered (accumulated from audience pushback).
    pub resistance: Fixed,
    /// Effectiveness score (updated each tick).
    pub effectiveness: Fixed,
    /// Tick when campaign started.
    pub started_tick: u64,
    /// Whether campaign is still active.
    pub active: bool,
}

impl PropagandaCampaign {
    /// Create a new propaganda campaign.
    pub fn new(
        id: usize,
        sponsor: usize,
        targets: Vec<usize>,
        narrative: String,
        intensity: Fixed,
        channels: Vec<PropagandaChannel>,
        duration: u64,
        tick: u64,
    ) -> Self {
        Self {
            id,
            sponsor,
            targets,
            narrative,
            intensity,
            channels,
            credibility: Fixed::from_f64(0.5),
            coercion: Fixed::ZERO,
            duration,
            remaining: duration,
            resistance: Fixed::ZERO,
            effectiveness: Fixed::ZERO,
            started_tick: tick,
            active: true,
        }
    }

    /// Compute campaign effectiveness based on institutional legitimacy,
    /// audience fear, accumulated resistance, and the global effectiveness
    /// multiplier (Iteration 177: `propaganda_effectiveness`, the §13.4
    /// tuning knob — identity 1.0, so the calibrated envelope is preserved;
    /// the old default 0.35 would have dropped mean effectiveness below the
    /// 0.1 push gate, functionally killing the system).
    ///
    /// ```text
    /// effectiveness = legitimacy × intensity × channel_bonus
    ///               × (1 - resistance) × (1 - coercion_penalty)
    ///               × (1 + fear_boost) × credibility × multiplier
    /// ```
    pub fn compute_effectiveness(
        &mut self,
        institutional_legitimacy: Fixed,
        audience_fear: Fixed,
        effectiveness_multiplier: Fixed,
    ) {
        let channel_bonus = Fixed::from_f64(0.1) * Fixed::from_int(self.channels.len() as i64);
        let coercion_penalty = self.coercion * Fixed::from_f64(0.3);
        let fear_boost = audience_fear * Fixed::from_f64(0.2);

        self.effectiveness = (institutional_legitimacy
            * self.intensity
            * (Fixed::ONE + channel_bonus)
            * (Fixed::ONE - self.resistance)
            * (Fixed::ONE - coercion_penalty)
            * (Fixed::ONE + fear_boost)
            * self.credibility
            * effectiveness_multiplier)
            .clamp_01();
    }

    /// Tick the campaign forward by one tick.
    ///
    /// `resistance_growth` controls how fast audience fatigue accumulates per tick.
    pub fn tick(&mut self, resistance_growth: Fixed) {
        if self.remaining > 0 {
            self.remaining -= 1;
            // Resistance grows slowly as audience gets tired of propaganda
            self.resistance = (self.resistance + resistance_growth).clamp_01();
        }
        // Deactivate when duration fully elapsed
        if self.remaining == 0 {
            self.active = false;
        }
    }

    /// Record audience pushback (increases resistance).
    pub fn record_pushback(&mut self, amount: Fixed) {
        self.resistance = (self.resistance + amount).clamp_01();
    }
}

/// Registry of all propaganda campaigns.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PropagandaRegistry {
    /// All campaigns.
    pub campaigns: Vec<PropagandaCampaign>,
    /// Next available campaign id.
    next_id: usize,
}

impl PropagandaRegistry {
    /// Register a new campaign and return its id.
    pub fn register(&mut self, campaign: PropagandaCampaign) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let mut campaign = campaign;
        campaign.id = id;
        self.campaigns.push(campaign);
        id
    }

    /// Get a campaign by id.
    pub fn get(&self, id: usize) -> Option<&PropagandaCampaign> {
        self.campaigns.iter().find(|c| c.id == id)
    }

    /// Tick all active campaigns.
    ///
    /// `resistance_growth` controls audience fatigue accumulation per tick.
    pub fn tick_all(&mut self, resistance_growth: Fixed) {
        for campaign in &mut self.campaigns {
            if campaign.active {
                campaign.tick(resistance_growth);
            }
        }
    }

    /// Number of active campaigns.
    pub fn active_count(&self) -> usize {
        self.campaigns.iter().filter(|c| c.active).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_campaign_has_sane_defaults() {
        let c = PropagandaCampaign::new(
            0,
            0,
            vec![1, 2],
            "test narrative".into(),
            Fixed::from_f64(0.7),
            vec![PropagandaChannel::Sermon],
            100,
            0,
        );
        assert_eq!(c.id, 0);
        assert!(c.active);
        assert_eq!(c.remaining, 100);
    }

    #[test]
    fn effectiveness_scales_with_legitimacy() {
        let mut c = PropagandaCampaign::new(
            0,
            0,
            vec![],
            "test".into(),
            Fixed::from_f64(0.5),
            vec![PropagandaChannel::Sermon],
            100,
            0,
        );
        c.compute_effectiveness(Fixed::from_f64(0.9), Fixed::ZERO, Fixed::ONE);
        let high_legit = c.effectiveness;
        c.compute_effectiveness(Fixed::from_f64(0.2), Fixed::ZERO, Fixed::ONE);
        let low_legit = c.effectiveness;
        assert!(high_legit > low_legit);
    }

    #[test]
    fn effectiveness_scales_with_multiplier_knob() {
        // Iteration 177: the §13.4 propaganda_effectiveness knob must be
        // LIVE — the pre-fix multiplier was 100%-dead (zero references
        // outside parameters.rs; probe-pinned rate-invariant across
        // 0.35–2.0). Higher multiplier -> higher effectiveness, and the
        // identity 1.0 preserves the calibrated envelope.
        let mk = |mult: f64| {
            let mut c = PropagandaCampaign::new(
                0,
                0,
                vec![],
                "test".into(),
                Fixed::from_f64(0.5),
                vec![PropagandaChannel::Sermon],
                100,
                0,
            );
            c.compute_effectiveness(Fixed::from_f64(0.9), Fixed::ZERO, Fixed::from_f64(mult));
            c.effectiveness
        };
        let base = mk(1.0);
        let boosted = mk(2.0);
        let dampened = mk(0.5);
        assert!(
            boosted > base,
            "2x multiplier must raise effectiveness: base {base} boosted {boosted}"
        );
        assert!(
            dampened < base,
            "0.5x multiplier must lower effectiveness: base {base} dampened {dampened}"
        );
    }

    #[test]
    fn tick_decreases_remaining() {
        let mut c = PropagandaCampaign::new(
            0,
            0,
            vec![],
            "test".into(),
            Fixed::from_f64(0.5),
            vec![],
            10,
            0,
        );
        c.tick(Fixed::from_f64(0.002));
        assert_eq!(c.remaining, 9);
    }

    #[test]
    fn campaign_deactivates_when_expired() {
        let mut c = PropagandaCampaign::new(
            0,
            0,
            vec![],
            "test".into(),
            Fixed::from_f64(0.5),
            vec![],
            1,
            0,
        );
        c.tick(Fixed::from_f64(0.002));
        assert!(!c.active);
    }

    #[test]
    fn pushback_increases_resistance() {
        let mut c = PropagandaCampaign::new(
            0,
            0,
            vec![],
            "test".into(),
            Fixed::from_f64(0.5),
            vec![],
            100,
            0,
        );
        c.record_pushback(Fixed::from_f64(0.3));
        assert!(c.resistance > Fixed::ZERO);
    }

    #[test]
    fn registry_register_and_get() {
        let mut reg = PropagandaRegistry::default();
        let c = PropagandaCampaign::new(
            0,
            0,
            vec![],
            "test".into(),
            Fixed::from_f64(0.5),
            vec![],
            100,
            0,
        );
        let id = reg.register(c);
        assert_eq!(id, 0);
        assert!(reg.get(0).is_some());
    }
}
