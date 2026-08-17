//! Attachment system — attachment styles, security, anxiety, avoidance.
//!
//! Attachment is central to relationships. It affects friendship, romance,
//! marriage, parenting, faction loyalty, religious devotion, and leader dependence.
//!
//! Under threat:
//! - secure agents seek support and recover,
//! - anxious agents cling and demand reassurance,
//! - avoidant agents withdraw and self-regulate,
//! - disorganized agents oscillate or freeze.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Attachment style classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AttachmentStyle {
    /// Trusting, comfortable with intimacy and independence.
    #[default]
    Secure,
    /// Craves closeness, fears abandonment, hypervigilant to rejection.
    Anxious,
    /// Values independence, uncomfortable with closeness, self-reliant.
    Avoidant,
    /// Oscillates between craving and fearing closeness.
    Disorganized,
}

/// Caregiving style — how the agent nurtures others.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CaregivingStyle {
    /// Responsive, sensitive, attuned.
    #[default]
    Sensitive,
    /// Anxious, intrusive, inconsistent.
    Intrusive,
    /// Dismissive, unavailable.
    Dismissive,
    /// Frightening, unpredictable.
    Frightening,
}

/// Attachment system state for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentSystem {
    /// Current attachment style.
    pub style: AttachmentStyle,
    /// Attachment security (0 = insecure, 1 = fully secure).
    pub security: Fixed,
    /// Attachment anxiety (0 = low, 1 = high).
    pub anxiety: Fixed,
    /// Attachment avoidance (0 = low, 1 = high).
    pub avoidance: Fixed,
    /// Threshold for protest behavior (separation distress).
    pub protest_threshold: Fixed,
    /// Receptivity to soothing from others.
    pub soothing_receptivity: Fixed,
    /// Level of distress when separated from attachment figure.
    pub separation_distress: Fixed,
    /// How the agent cares for others.
    pub caregiving_style: CaregivingStyle,
}

impl Default for AttachmentSystem {
    fn default() -> Self {
        Self {
            style: AttachmentStyle::Secure,
            security: Fixed::from_f64(0.6),
            anxiety: Fixed::from_f64(0.2),
            avoidance: Fixed::from_f64(0.2),
            protest_threshold: Fixed::from_f64(0.5),
            soothing_receptivity: Fixed::from_f64(0.6),
            separation_distress: Fixed::ZERO,
            caregiving_style: CaregivingStyle::Sensitive,
        }
    }
}

impl AttachmentSystem {
    /// Initialize attachment style from developmental history and genetic predisposition.
    pub fn initialize(
        &mut self,
        caregiver_security: Fixed,
        trauma_history: Fixed,
        attachment_vulnerability: Fixed,
    ) {
        // Secure caregiving → secure attachment
        // Trauma → insecure attachment
        // Genetic vulnerability amplifies the effect
        let insecurity = trauma_history * Fixed::from_f64(0.4)
            + (Fixed::ONE - caregiver_security) * Fixed::from_f64(0.3)
            + attachment_vulnerability * Fixed::from_f64(0.3);

        if insecurity < Fixed::from_f64(0.3) {
            self.style = AttachmentStyle::Secure;
            self.security = (Fixed::ONE - insecurity).clamp_01();
            self.anxiety = insecurity * Fixed::from_f64(0.3);
            self.avoidance = insecurity * Fixed::from_f64(0.2);
        } else if insecurity < Fixed::from_f64(0.6) {
            // Split between anxious and avoidant based on caregiver consistency
            if caregiver_security > Fixed::from_f64(0.5) {
                self.style = AttachmentStyle::Anxious;
                self.anxiety = insecurity;
                self.avoidance = Fixed::from_f64(0.2);
            } else {
                self.style = AttachmentStyle::Avoidant;
                self.anxiety = Fixed::from_f64(0.2);
                self.avoidance = insecurity;
            }
            self.security = (Fixed::ONE - insecurity).clamp_01();
        } else {
            self.style = AttachmentStyle::Disorganized;
            self.security = Fixed::from_f64(0.1);
            self.anxiety = insecurity;
            self.avoidance = insecurity * Fixed::from_f64(0.7);
        }

        self.protest_threshold = (Fixed::ONE - self.security) * Fixed::from_f64(0.5);
        self.soothing_receptivity = self.security * Fixed::from_f64(0.8);
    }

    /// React to separation from an attachment figure.
    pub fn on_separation(&mut self, closeness: Fixed, separation_rate: Fixed) {
        let distress = closeness * self.anxiety * separation_rate;
        self.separation_distress = (self.separation_distress + distress).clamp_01();
    }

    /// React to reunion with an attachment figure.
    pub fn on_reunion(
        &mut self,
        secure_recovery: Fixed,
        anxious_recovery: Fixed,
        avoidant_recovery: Fixed,
        disorganized_recovery: Fixed,
    ) {
        match self.style {
            AttachmentStyle::Secure => {
                // Quick recovery
                self.separation_distress = (self.separation_distress * secure_recovery).clamp_01();
                self.security = (self.security + Fixed::from_f64(0.01)).clamp_01();
            }
            AttachmentStyle::Anxious => {
                // Slow recovery, may be angry
                self.separation_distress = (self.separation_distress * anxious_recovery).clamp_01();
            }
            AttachmentStyle::Avoidant => {
                // Appears to recover quickly but internally stressed
                self.separation_distress =
                    (self.separation_distress * avoidant_recovery).clamp_01();
            }
            AttachmentStyle::Disorganized => {
                // Unpredictable — may oscillate
                self.separation_distress =
                    (self.separation_distress * disorganized_recovery).clamp_01();
            }
        }
    }

    /// Receive comfort from an attachment figure.
    pub fn receive_comfort(
        &mut self,
        comfort_quality: Fixed,
        secure_comfort: Fixed,
        anxious_comfort: Fixed,
        avoidant_comfort: Fixed,
        disorganized_comfort: Fixed,
        security_gain: Fixed,
    ) {
        let effectiveness = self.soothing_receptivity * comfort_quality;
        match self.style {
            AttachmentStyle::Secure => {
                self.separation_distress =
                    (self.separation_distress - effectiveness * secure_comfort).max(Fixed::ZERO);
                self.security = (self.security + effectiveness * security_gain).clamp_01();
            }
            AttachmentStyle::Anxious => {
                // Partially soothed but anxiety remains
                self.separation_distress =
                    (self.separation_distress - effectiveness * anxious_comfort).max(Fixed::ZERO);
            }
            AttachmentStyle::Avoidant => {
                // May reject comfort, but still benefits slightly
                self.separation_distress =
                    (self.separation_distress - effectiveness * avoidant_comfort).max(Fixed::ZERO);
            }
            AttachmentStyle::Disorganized => {
                // Unpredictable response
                self.separation_distress = (self.separation_distress
                    - effectiveness * disorganized_comfort)
                    .max(Fixed::ZERO);
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secure_caregiving_creates_secure_attachment() {
        let mut att = AttachmentSystem::default();
        att.initialize(Fixed::from_f64(0.8), Fixed::ZERO, Fixed::from_f64(0.3));
        assert_eq!(att.style, AttachmentStyle::Secure);
        assert!(att.security > Fixed::from_f64(0.5));
    }

    #[test]
    fn trauma_creates_insecure_attachment() {
        let mut att = AttachmentSystem::default();
        att.initialize(
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.7),
        );
        assert_ne!(att.style, AttachmentStyle::Secure);
        assert!(att.security < Fixed::from_f64(0.5));
    }

    #[test]
    fn separation_increases_distress() {
        let mut att = AttachmentSystem {
            security: Fixed::from_f64(0.8),
            anxiety: Fixed::from_f64(0.5),
            ..Default::default()
        };
        att.on_separation(Fixed::from_f64(0.7), Fixed::from_f64(0.3));
        assert!(att.separation_distress > Fixed::ZERO);
    }

    #[test]
    fn reunion_reduces_distress_for_secure() {
        let mut att = AttachmentSystem {
            style: AttachmentStyle::Secure,
            separation_distress: Fixed::from_f64(0.5),
            ..Default::default()
        };
        att.on_reunion(
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.4),
            Fixed::from_f64(0.5),
        );
        assert!(att.separation_distress < Fixed::from_f64(0.5));
    }

    #[test]
    fn comfort_reduces_distress_scaled_by_receptivity() {
        // Iteration 191: `receive_comfort` is now wired at the Comfort-
        // interaction pass (the active soothing path — pre-Iter-191 it had
        // zero call sites and separation distress only decayed passively).
        // The effectiveness is `soothing_receptivity × comfort_quality`, so
        // a high-receptivity agent soothes more than a low-receptivity one.
        // `soothing_receptivity` is computed in `initialize` (= security × 0.8),
        // so the two agents are built through that path to get honest,
        // differentiated receptivity rather than a struct-literal default.
        let mut open = AttachmentSystem::default();
        open.initialize(Fixed::from_f64(0.8), Fixed::ZERO, Fixed::from_f64(0.3));
        open.separation_distress = Fixed::from_f64(0.6);
        let mut guarded = AttachmentSystem::default();
        guarded.initialize(Fixed::from_f64(0.2), Fixed::from_f64(0.7), Fixed::from_f64(0.8));
        guarded.separation_distress = Fixed::from_f64(0.6);
        open.receive_comfort(
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.4),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.05),
        );
        guarded.receive_comfort(
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.4),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.05),
        );
        assert!(
            open.separation_distress < guarded.separation_distress,
            "a higher-receptivity agent must soothe more: {} vs {}",
            open.separation_distress.to_f64(),
            guarded.separation_distress.to_f64()
        );
        assert!(
            open.security > Fixed::from_f64(0.8),
            "the secure style gains security from comfort"
        );
    }

    #[test]
    fn distress_decays_proportionally_to_rate() {
        // Iteration 173: the daily decay in the sim applies
        // `distress * (1 - attachment_decay_rate)`; this unit test pins the
        // proportional-decay contract (slower decay rate ⟹ slower drain).
        let mut slow = AttachmentSystem {
            separation_distress: Fixed::from_f64(0.5),
            ..Default::default()
        };
        let mut fast = slow.clone();
        for _ in 0..30 {
            slow.separation_distress =
                (slow.separation_distress * (Fixed::ONE - Fixed::from_f64(0.05))).clamp_01();
            fast.separation_distress =
                (fast.separation_distress * (Fixed::ONE - Fixed::from_f64(0.30))).clamp_01();
        }
        assert!(fast.separation_distress < slow.separation_distress);
        assert!(
            slow.separation_distress > Fixed::ZERO,
            "slow decay must not zero distress in 30 days"
        );
    }
}
