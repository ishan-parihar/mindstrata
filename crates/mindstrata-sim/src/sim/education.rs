//! Apprenticeships and school terms.

use super::{AgentId, Fixed, JournalEntryKind, MemoryKind, MemoryTag, SimEvent, Simulation, Tick};

impl Simulation {
    pub(super) fn run_apprenticeship_pass(&mut self, tick_u64: u64, tick: Tick) {
        let n = self.agents.len();
        if n < 2 {
            return;
        }
        for student in 0..n {
            // Find the first knowledge in store order that this student lacks
            // AND for which a capable teacher exists. Scanning candidates
            // (rather than teaching only the first missing item) is essential:
            // a student missing several items may only have a teacher for one.
            let mut chosen: Option<(u64, usize, u32)> = None;
            for knowledge in &self.knowledge_store {
                let knowledge_id = knowledge.id;
                if self.agents[student].education.has_learned(knowledge_id) {
                    continue;
                }
                // §5 (Iteration 148): the technology tree gates education too
                // — a student cannot be taught a node without its prereqs.
                // Deliberately checks `cultural.knowledge` (the same vec the
                // other three gates use) rather than `education.has_learned`:
                // prerequisites live in the knowledge vec, and the two are
                // kept in sync by the apprenticeship bookkeeping below.
                if !self
                    .technology
                    .can_learn(&self.agents[student].cultural.knowledge, knowledge_id)
                {
                    continue;
                }
                // Pick the best teacher among agents who hold this knowledge,
                // can teach, and share a relationship with the student.
                let mut best: Option<(usize, Fixed)> = None;
                for teacher in 0..n {
                    if teacher == student {
                        continue;
                    }
                    if !self.agents[teacher].education.has_learned(knowledge_id) {
                        continue;
                    }
                    if self.agents[teacher].education.teaching_skill < Fixed::from_f64(0.3) {
                        continue;
                    }
                    let rel_quality = self.relationship_quality(teacher, student);
                    if rel_quality < Fixed::from_f64(0.1) {
                        continue;
                    }
                    let skill = self.agents[teacher].education.teaching_skill;
                    if best.is_none_or(|(_, s)| skill > s) {
                        best = Some((teacher, skill));
                    }
                }
                if let Some((teacher, _)) = best {
                    chosen = Some((knowledge_id, teacher, knowledge.holders));
                    break;
                }
            }
            let Some((knowledge_id, teacher, holders)) = chosen else {
                continue;
            };

            // Familiarity: how broadly held the knowledge is (holders / agents).
            let familiarity = Fixed::from_f64(holders as f64 / n as f64).clamp_01();
            let rel_quality = self.relationship_quality(teacher, student);
            let event = crate::culture::education::attempt_teaching(
                teacher,
                student,
                knowledge_id,
                &self.agents[teacher].education,
                &self.agents[student].education,
                familiarity,
                rel_quality,
                tick_u64,
            );

            self.agents[student]
                .education
                .record_learning(event.clone());
            self.agents[teacher]
                .education
                .record_teaching(event.clone());
            if event.success {
                // The student now holds the knowledge in both education state
                // and the shared cultural knowledge vector.
                if !self.agents[student]
                    .cultural
                    .knowledge
                    .contains(&knowledge_id)
                {
                    self.agents[student].cultural.knowledge.push(knowledge_id);
                }
                if let Some(k) = self
                    .knowledge_store
                    .iter_mut()
                    .find(|k| k.id == knowledge_id)
                {
                    k.holders += 1;
                }
                // §8.1.7: Acquired knowledge is evidence exposure — learning
                // desacralizes in proportion to the learning rate and the
                // student's reasoning capacity (same hook as gossip transfer).
                let reasoning = self.agents[student].cognitive.executive_capacity;
                self.agents[student]
                    .sacred_values
                    .desacralize_through_exposure(event.learning_rate, reasoning);
                self.events.push(SimEvent::KnowledgeTransferred {
                    source: AgentId::new(teacher as u64),
                    target: AgentId::new(student as u64),
                    knowledge_id,
                    tick,
                });
                self.provenance
                    .record_institutional(crate::provenance::InstitutionalTrace {
                        institution_name: "Apprenticeship".into(),
                        tick: tick_u64,
                        decision_kind: "apprenticeship_teaching".into(),
                        description: format!(
                            "Agent {teacher} taught knowledge {knowledge_id} to Agent {student}"
                        ),
                        affected: vec![AgentId::new(teacher as u64), AgentId::new(student as u64)],
                        success: true,
                    });

                // §8.1.3: Semantic memory — successfully acquiring knowledge is
                // the canonical semantic episode. Naturally sparse (one pass per
                // tick, success needs a capable teacher and a warm relationship),
                // so the 200-capacity store is not flooded.
                let learner = &mut self.agents[student];
                if learner.agent_tier.tier.runs_memory_encoding()
                    && learner.agent_tier.budget_tracker.can_memory_op()
                {
                    let _ = learner.agent_tier.budget_tracker.consume_memory_op();
                    let emotional =
                        learner.affect.arousal * Fixed::from_f64(0.6) + Fixed::from_f64(0.1);
                    learner.memory.encode(
                        MemoryKind::Semantic,
                        tick_u64,
                        Fixed::from_f64(0.4),
                        emotional,
                        Some(teacher as u32),
                        MemoryTag::LearnedKnowledge,
                    );
                }
            }
        }
    }

    /// §5 (Iteration 151): One formal school term — a single competent
    /// teacher instructs a small cohort of the youngest students in the
    /// teacher's most advanced knowledge. Gated on a `SiteKind::School`
    /// existing (no default world places one, so this is a structural no-op
    /// in every calibrated window). Fully deterministic — teacher and cohort
    /// selection follow fixed rules and `attempt_teaching` draws no
    /// randomness — so a term can never perturb any RNG stream.
    pub fn tick_school_term(&mut self, tick_u64: u64) {
        // Zero-blast gate: without a schoolhouse there is nothing to convene.
        if !self
            .world
            .sites
            .iter()
            .any(|s| s.kind == crate::world::SiteKind::School)
        {
            return;
        }
        let n = self.agents.len();
        if n < 2 {
            return;
        }

        // Instructor: the most experienced teacher holding at least one
        // knowledge (highest teaching skill, ties → lowest index).
        let skills: Vec<Fixed> = self
            .agents
            .iter()
            .map(|a| a.education.teaching_skill)
            .collect();
        let held: Vec<bool> = self
            .agents
            .iter()
            .map(|a| !a.cultural.knowledge.is_empty())
            .collect();
        let Some(teacher_idx) = crate::schools::select_teacher(&skills, &held) else {
            return;
        };

        // Lesson topic: the teacher's most advanced knowledge — the last
        // element of the store-ordered knowledge vector. `select_teacher`
        // only returns knowledge-holding indices, but we still degrade
        // gracefully instead of panicking.
        let Some(&knowledge_id) = self.agents[teacher_idx].cultural.knowledge.last() else {
            return;
        };

        // Cohort: the youngest students who lack the topic and whose
        // technology prerequisites are met — the same gate the apprenticeship
        // applies, so schools can never bypass the tech tree.
        let mut students: Vec<usize> = (0..n)
            .filter(|&s| s != teacher_idx)
            .filter(|&s| !self.agents[s].education.has_learned(knowledge_id))
            .filter(|&s| {
                self.technology
                    .can_learn(&self.agents[s].cultural.knowledge, knowledge_id)
            })
            .collect();
        students.sort_by_key(|&s| self.agents[s].age);
        students.truncate(crate::schools::COHORT_SIZE);
        if students.is_empty() {
            return;
        }

        let cohort = students.len() as u64;
        let mut graduates: u64 = 0;
        // Familiarity: how broadly held the knowledge is (holders / agents);
        // an unseeded innovation scores zero.
        let familiarity = Fixed::from_f64(
            self.knowledge_store
                .iter()
                .find(|k| k.id == knowledge_id)
                .map_or(0.0, |k| k.holders as f64 / n as f64),
        )
        .clamp_01();

        for &student in &students {
            let rel_quality = self.relationship_quality(teacher_idx, student);
            let event = crate::culture::education::attempt_teaching(
                teacher_idx,
                student,
                knowledge_id,
                &self.agents[teacher_idx].education,
                &self.agents[student].education,
                familiarity,
                rel_quality,
                tick_u64,
            );
            self.agents[student]
                .education
                .record_learning(event.clone());
            self.agents[teacher_idx]
                .education
                .record_teaching(event.clone());
            if !event.success {
                continue;
            }
            graduates += 1;
            // The student now holds the knowledge in both education state
            // and the shared cultural knowledge vector.
            if !self.agents[student]
                .cultural
                .knowledge
                .contains(&knowledge_id)
            {
                self.agents[student].cultural.knowledge.push(knowledge_id);
            }
            if let Some(k) = self
                .knowledge_store
                .iter_mut()
                .find(|k| k.id == knowledge_id)
            {
                k.holders += 1;
            }
            // §8.1.7: Acquired knowledge is evidence exposure — learning
            // desacralizes in proportion to the learning rate and the
            // student's reasoning capacity (same hook as gossip transfer).
            let reasoning = self.agents[student].cognitive.executive_capacity;
            self.agents[student]
                .sacred_values
                .desacralize_through_exposure(event.learning_rate, reasoning);
            self.events.push(SimEvent::KnowledgeTransferred {
                source: AgentId::new(teacher_idx as u64),
                target: AgentId::new(student as u64),
                knowledge_id,
                tick: mindstrata_core::clock::Tick::new(tick_u64),
            });
            self.provenance.record_institutional(crate::provenance::InstitutionalTrace {
                institution_name: "School".into(),
                tick: tick_u64,
                decision_kind: "school_teaching".into(),
                description: format!(
                    "Agent {teacher_idx} taught knowledge {knowledge_id} to Agent {student} in the school term"
                ),
                affected: vec![AgentId::new(teacher_idx as u64), AgentId::new(student as u64)],
                success: true,
            });

            // §8.1.3: Semantic memory — the same sparse-encoding discipline
            // as the apprenticeship (one term per year, small cohort), so the
            // 200-capacity store is not flooded.
            let learner = &mut self.agents[student];
            if learner.agent_tier.tier.runs_memory_encoding()
                && learner.agent_tier.budget_tracker.can_memory_op()
            {
                let _ = learner.agent_tier.budget_tracker.consume_memory_op();
                let emotional =
                    learner.affect.arousal * Fixed::from_f64(0.6) + Fixed::from_f64(0.1);
                learner.memory.encode(
                    MemoryKind::Semantic,
                    tick_u64,
                    Fixed::from_f64(0.4),
                    emotional,
                    Some(teacher_idx as u32),
                    MemoryTag::LearnedKnowledge,
                );
            }
        }

        self.school.terms_run += 1;
        self.school.lessons_taught += students.len() as u64;
        self.school.graduates += graduates;
        self.journal.record(
            tick_u64,
            AgentId::new(teacher_idx as u64),
            JournalEntryKind::SchoolTerm {
                teacher: teacher_idx as u64,
                cohort,
                graduates,
            },
        );
    }
}
