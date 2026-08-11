//! §8.1.18: Education system — knowledge transmission between agents.
//!
//! Architecture §8.1.18: Knowledge is transmitted through education events,
//! apprenticeship, institutional teaching, and informal learning. Education
//! is a core technology of cultural transmission.
//!
//! ```text
//! Education dynamics:
//!   - Master/apprentice relationships enable skill transfer
//!   - Institutional education transmits cultural knowledge
//!   - Informal learning through observation and imitation
//!   - Teaching ability depends on skill, patience, and communication
//!   - Learning ability depends on intelligence, motivation, and relationship
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A teaching/learning event between two agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationEvent {
    /// Index of the teacher agent.
    pub teacher: usize,
    /// Index of the student agent.
    pub student: usize,
    /// Knowledge id being transmitted. `u64` to match the sim's knowledge
    /// store (CulturalState.knowledge: Vec<u64>) — the education module was
    /// originally written with `u32` ids and never wired in; alignment is a
    /// precondition of the apprenticeship pass.
    pub knowledge_id: u64,
    /// Quality of the teaching (0–1).
    pub quality: Fixed,
    /// Student's learning rate (depends on aptitude, motivation, relationship).
    pub learning_rate: Fixed,
    /// Tick when this event occurred.
    pub tick: u64,
    /// Whether the teaching was successful (student learned something).
    pub success: bool,
}

/// State of education for a single agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EducationState {
    /// Knowledge ids this agent has learned.
    pub learned: Vec<u64>,
    /// Knowledge ids this agent is currently learning (in progress).
    pub in_progress: Vec<u64>,
    /// Teaching skill — how well this agent can teach others (0–1).
    pub teaching_skill: Fixed,
    /// Learning aptitude — how quickly this agent learns (0–1).
    pub learning_aptitude: Fixed,
    /// Patience for teaching — how much energy the agent puts into teaching (0–1).
    pub teaching_patience: Fixed,
    /// Education events this agent has participated in as teacher.
    pub teaching_events: Vec<EducationEvent>,
    /// Education events this agent has participated in as student.
    pub learning_events: Vec<EducationEvent>,
}

impl EducationState {
    /// Compute the effective teaching quality for a specific knowledge transfer.
    pub fn effective_teaching(&self, knowledge_familiarity: Fixed) -> Fixed {
        (self.teaching_skill * Fixed::from_f64(0.4)
            + knowledge_familiarity * Fixed::from_f64(0.3)
            + self.teaching_patience * Fixed::from_f64(0.3))
        .clamp_01()
    }

    /// Compute the effective learning rate for a specific knowledge transfer.
    pub fn effective_learning(&self, teacher_quality: Fixed, relationship_quality: Fixed) -> Fixed {
        (self.learning_aptitude * Fixed::from_f64(0.3)
            + teacher_quality * Fixed::from_f64(0.4)
            + relationship_quality * Fixed::from_f64(0.3))
        .clamp_01()
    }

    /// Check if this agent has learned a specific knowledge id.
    pub fn has_learned(&self, knowledge_id: u64) -> bool {
        self.learned.contains(&knowledge_id)
    }

    /// Record a successful learning event.
    pub fn record_learning(&mut self, event: EducationEvent) {
        if event.success && !self.learned.contains(&event.knowledge_id) {
            self.learned.push(event.knowledge_id);
        }
        self.in_progress.retain(|&k| k != event.knowledge_id);
        self.learning_events.push(event);
        // Keep only recent events to bound memory
        if self.learning_events.len() > 50 {
            self.learning_events.remove(0);
        }
    }

    /// Record a teaching event.
    pub fn record_teaching(&mut self, event: EducationEvent) {
        self.teaching_events.push(event);
        if self.teaching_events.len() > 50 {
            self.teaching_events.remove(0);
        }
    }

    /// Daily update — decay teaching patience, grow aptitude with practice.
    pub fn daily_update(&mut self) {
        // Teaching patience regenerates with rest
        self.teaching_patience = (self.teaching_patience + Fixed::from_f64(0.001)).clamp_01();
        // Learning aptitude grows slowly with experience
        let event_count = Fixed::from_int(self.learning_events.len() as i64);
        let experience_bonus = (event_count * Fixed::from_f64(0.002)).min(Fixed::from_f64(0.1));
        self.learning_aptitude = (self.learning_aptitude + experience_bonus).clamp_01();
        // Teaching skill grows with teaching practice
        let teach_count = Fixed::from_int(self.teaching_events.len() as i64);
        let teach_bonus = (teach_count * Fixed::from_f64(0.001)).min(Fixed::from_f64(0.05));
        self.teaching_skill = (self.teaching_skill + teach_bonus).clamp_01();
    }
}

/// Attempt to transmit knowledge from teacher to student.
///
/// Returns an EducationEvent with the outcome.
pub fn attempt_teaching(
    teacher_idx: usize,
    student_idx: usize,
    knowledge_id: u64,
    teacher_education: &EducationState,
    student_education: &EducationState,
    knowledge_familiarity: Fixed,
    relationship_quality: Fixed,
    tick: u64,
) -> EducationEvent {
    let quality = teacher_education.effective_teaching(knowledge_familiarity);
    let learning_rate = student_education.effective_learning(quality, relationship_quality);

    // Success chance depends on learning rate and some randomness
    // (caller should provide deterministic roll)
    let success = learning_rate > Fixed::from_f64(0.3);

    EducationEvent {
        teacher: teacher_idx,
        student: student_idx,
        knowledge_id,
        quality,
        learning_rate,
        tick,
        success,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_teaching_depends_on_skill() {
        let high_skill = EducationState {
            teaching_skill: Fixed::from_f64(0.9),
            teaching_patience: Fixed::from_f64(0.7),
            ..EducationState::default()
        };
        let low_skill = EducationState {
            teaching_skill: Fixed::from_f64(0.2),
            teaching_patience: Fixed::from_f64(0.3),
            ..EducationState::default()
        };
        assert!(
            high_skill.effective_teaching(Fixed::from_f64(0.5))
                > low_skill.effective_teaching(Fixed::from_f64(0.5))
        );
    }

    #[test]
    fn learning_rate_depends_on_aptitude() {
        let high_apt = EducationState {
            learning_aptitude: Fixed::from_f64(0.9),
            ..EducationState::default()
        };
        let low_apt = EducationState {
            learning_aptitude: Fixed::from_f64(0.2),
            ..EducationState::default()
        };
        let rate_high = high_apt.effective_learning(Fixed::from_f64(0.5), Fixed::from_f64(0.5));
        let rate_low = low_apt.effective_learning(Fixed::from_f64(0.5), Fixed::from_f64(0.5));
        assert!(rate_high > rate_low);
    }

    #[test]
    fn record_learning_adds_to_learned() {
        let mut edu = EducationState::default();
        let event = EducationEvent {
            teacher: 0,
            student: 1,
            knowledge_id: 42,
            quality: Fixed::from_f64(0.8),
            learning_rate: Fixed::from_f64(0.7),
            tick: 0,
            success: true,
        };
        edu.record_learning(event);
        assert!(edu.has_learned(42));
    }
}
