//! Phase 3 audit probe — AP2 §8 Psychological Runtime operationalization.
//!
//! Prints per-system means/min/max and saturation flags (% of agents at 0.0
//! or 1.0) for every psychology subsystem across seeds × horizons ×
//! scenarios, plus categorical distributions (regulation strategy, dominant
//! need, life theme, scenario kind) and an emotion liveness ledger — so the
//! audit can verify the Phase 3 acceptance:
//!
//!   1. No psychology axis saturates (pinned at 1.0 for >50% of agents).
//!   2. Stress worlds (famine/pestilence) show directional psychology shifts
//!      vs calm (fear/stress up, self-esteem/hope down, more rumination).
//!   3. Per-agent spread exists (not all agents identical).
//!   4. Every emotion has a live window OR a documented inert reason.
//!   5. LOD gating: secondary-tier agents keep narrative at the birth
//!      envelope (resilience factor ≈ 1.0) — focal-only containment.
//!
//! Run with: `cargo run -p mindstrata-benches --example psych_probe --release`
//! (release required — 18 runs up to 10K ticks.)

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::agent_tier::AgentTier;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::AgentBundle;
use mindstrata_sim::Simulation;

fn scenario_sim(scenario: Scenario, ticks: u64) -> Simulation {
    let mut sim = Simulation::from_scenario(scenario);
    sim.populate();
    sim.run(ticks);
    sim
}

/// Read one probed psychology field by key — single closure-typed accessor
/// so the probe can iterate field lists without per-closure type mismatches.
fn field(a: &AgentBundle, key: &str) -> Fixed {
    match key {
        // Self-model
        "self.self_esteem" => a.self_model.self_esteem,
        "self.coherence" => a.self_model.coherence,
        "self.security" => a.self_model.security,
        // Narrative
        "narr.redemption" => a.narrative.redemption_script,
        "narr.contamination" => a.narrative.contamination_script,
        "narr.victimhood" => a.narrative.victimhood_script,
        "narr.heroism" => a.narrative.heroism_script,
        "narr.chosenness" => a.narrative.chosenness_script,
        "narr.shame" => a.narrative.shame_script,
        "narr.coherence" => a.narrative.coherence,
        "narr.resilience_factor" => {
            a.narrative.stress_resilience_factor(
                Fixed::from_f64(0.15),
                a.embodied.endocrine.stress.chronic_load,
            )
        }
        // Prospection
        "prosp.hope" => a.prospection.hope,
        "prosp.dread" => a.prospection.dread,
        "prosp.optimism_bias" => a.prospection.optimism_bias,
        "prosp.catastrophic_bias" => a.prospection.catastrophic_bias,
        "prosp.planning_confidence" => a.prospection.planning_confidence,
        // Psychopathology
        "psych.depression" => a.psychopathology.depression_risk,
        "psych.anxiety" => a.psychopathology.anxiety_risk,
        "psych.ptsd" => a.psychopathology.ptsd_risk,
        "psych.paranoia" => a.psychopathology.paranoia_risk,
        "psych.overall_health" => a.psychopathology.overall_health,
        // Moral cognition
        "moral.outrage" => a.moral_cognition.moral_emotions.outrage,
        "moral.shame" => a.moral_cognition.moral_emotions.shame,
        "moral.pride" => a.moral_cognition.moral_emotions.pride,
        "moral.identity" => a.moral_cognition.moral_identity,
        // Motivation
        "motive.hunger" => a.motivation.hunger.deficit,
        "motive.safety" => a.motivation.safety.deficit,
        // Body needs (appraisal inputs)
        "needs.hunger" => a.needs.hunger,
        "needs.thirst" => a.needs.thirst,
        "needs.fatigue" => a.needs.fatigue,
        // Skills
        "skill.automaticity" => a.psych_skills.automaticity,
        // Interoception
        "intero.negative_bias" => a.interoception.negative_bias,
        "intero.sensitivity" => a.interoception.sensitivity,
        // Cultural cognition
        "culture.conservatism" => a.cultural_cognition.conservatism,
        "culture.max_taboo" => a.cultural_cognition.max_taboo_strength(),
        // Attachment
        "attach.security" => a.attachment.security,
        "attach.anxiety" => a.attachment.anxiety,
        "attach.avoidance" => a.attachment.avoidance,
        // Executive function
        "ef.effective_capacity" => a.cognitive_runtime.effective_capacity,
        "ef.effective_inhibition" => a.cognitive_runtime.effective_inhibition,
        "ef.current_impulsivity" => a.cognitive_runtime.current_impulsivity,
        // Regulation
        "reg.capacity" => a.emotion_regulation.capacity,
        "reg.effort" => a.emotion_regulation.current_effort,
        _ => panic!("unknown probe field {key}"),
    }
}

/// Read one emotion field by name.
fn emotion(a: &AgentBundle, name: &str) -> Fixed {
    match name {
        "fear" => a.emotions.fear,
        "anger" => a.emotions.anger,
        "joy" => a.emotions.joy,
        "sadness" => a.emotions.sadness,
        "trust" => a.emotions.trust,
        "shame" => a.emotions.shame,
        "pride" => a.emotions.pride,
        "guilt" => a.emotions.guilt,
        "disgust" => a.emotions.disgust,
        "contempt" => a.emotions.contempt,
        "awe" => a.emotions.awe,
        "gratitude" => a.emotions.gratitude,
        "jealousy" => a.emotions.jealousy,
        "envy" => a.emotions.envy,
        "loneliness" => a.emotions.loneliness,
        "tenderness" => a.emotions.tenderness,
        "humiliation" => a.emotions.humiliation,
        "relief" => a.emotions.relief,
        "hope" => a.emotions.hope,
        "despair" => a.emotions.despair,
        "nostalgia" => a.emotions.nostalgia,
        "moral_outrage" => a.emotions.moral_outrage,
        _ => panic!("unknown emotion {name}"),
    }
}

struct Stats {
    label: String,
    values: Vec<f64>,
}

impl Stats {
    fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            values: Vec::new(),
        }
    }
    fn push(&mut self, v: Fixed) {
        self.values.push(v.to_f64());
    }
    fn report(&self) -> String {
        let n = self.values.len() as f64;
        if n == 0.0 {
            return format!("  {:<28} (no agents)", self.label);
        }
        let mean: f64 = self.values.iter().sum::<f64>() / n;
        let min = self.values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = self.values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let at_zero = self.values.iter().filter(|v| **v <= 0.0).count();
        let at_one = self.values.iter().filter(|v| **v >= 1.0).count();
        let sat = if at_one as f64 > n * 0.5 { " ⚠ SATURATED-HIGH" } else { "" };
        let dead = if max - min < 0.001 { " (no spread)" } else { "" };
        format!(
            "  {:<28} mean={:.3} min={:.3} max={:.3} [0:{}/{}, 1:{}/{}]{}{}",
            self.label,
            mean,
            min,
            max,
            at_zero,
            self.values.len(),
            at_one,
            self.values.len(),
            sat,
            dead
        )
    }
}

fn counts<T: std::fmt::Debug>(label: &str, items: &[T]) -> String {
    let mut tally: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for i in items {
        *tally.entry(format!("{i:?}")).or_insert(0) += 1;
    }
    let mut s = format!("  {label:<28}");
    for (k, v) in tally {
        s.push_str(&format!(" {k}:{v}"));
    }
    s
}

const FIELDS: &[&str] = &[
    "reg.capacity",
    "reg.effort",
    "self.self_esteem",
    "self.coherence",
    "self.security",
    "narr.redemption",
    "narr.contamination",
    "narr.victimhood",
    "narr.heroism",
    "narr.chosenness",
    "narr.shame",
    "narr.coherence",
    "narr.resilience_factor",
    "prosp.hope",
    "prosp.dread",
    "prosp.optimism_bias",
    "prosp.catastrophic_bias",
    "prosp.planning_confidence",
    "psych.depression",
    "psych.anxiety",
    "psych.ptsd",
    "psych.paranoia",
    "psych.overall_health",
    "moral.outrage",
    "moral.shame",
    "moral.pride",
    "moral.identity",
    "motive.hunger",
    "motive.safety",
    "needs.hunger",
    "needs.thirst",
    "needs.fatigue",
    "skill.automaticity",
    "intero.negative_bias",
    "intero.sensitivity",
    "culture.conservatism",
    "culture.max_taboo",
    "attach.security",
    "attach.anxiety",
    "attach.avoidance",
    "ef.effective_capacity",
    "ef.effective_inhibition",
    "ef.current_impulsivity",
];

const EMOTIONS: &[&str] = &[
    "fear", "anger", "joy", "sadness", "trust", "shame", "pride", "guilt", "disgust",
    "contempt", "awe", "gratitude", "jealousy", "envy", "loneliness", "tenderness",
    "humiliation", "relief", "hope", "despair", "nostalgia", "moral_outrage",
];

fn main() {
    let scenarios: &[(&str, Scenario)] = &[
        ("calm", Scenario::calm()),
        ("famine", Scenario::famine()),
        ("pestilence", Scenario::pestilence()),
    ];
    let seeds = [42u64, 7u64];
    let horizons = [2000u64, 10000u64];

    // ── Emotion liveness ledger (across ALL runs) ────────────────────
    let mut emotion_ledger: std::collections::BTreeMap<&str, bool> =
        EMOTIONS.iter().map(|e| (*e, false)).collect();

    for (scen_name, scen) in scenarios {
        for seed in seeds {
            for ticks in horizons {
                let sim = scenario_sim(scen.clone(), ticks);

                println!("── {scen_name} seed{seed} @{ticks} ───────────────────────");
                for label in EMOTIONS {
                    let mut st = Stats::new(&format!("emotion.{label}"));
                    for a in &sim.agents {
                        let v = emotion(a, label);
                        if v > Fixed::from_f64(0.001) {
                            *emotion_ledger.get_mut(label).unwrap() = true;
                        }
                        st.push(v);
                    }
                    println!("{}", st.report());
                }
                for key in FIELDS {
                    let mut st = Stats::new(key);
                    for a in &sim.agents {
                        st.push(field(a, key));
                    }
                    println!("{}", st.report());
                }

                let mut st = Stats::new("narr.events_integrated");
                for a in &sim.agents {
                    st.values.push(a.narrative.events_integrated as f64);
                }
                println!("{}", st.report());

                let mut st = Stats::new("prosp.scenario_count");
                for a in &sim.agents {
                    st.values.push(a.prospection.scenarios.len() as f64);
                }
                println!("{}", st.report());

                let mut st = Stats::new("moral.norm_count");
                for a in &sim.agents {
                    st.values.push(a.moral_cognition.internalized_norms.len() as f64);
                }
                println!("{}", st.report());

                let mut st = Stats::new("skill.skill_count");
                let mut st2 = Stats::new("skill.habit_count");
                for a in &sim.agents {
                    st.values.push(a.psych_skills.skills.len() as f64);
                    st2.values.push(a.psych_skills.habits.len() as f64);
                }
                println!("{}", st.report());
                println!("{}", st2.report());

                let mut st = Stats::new("culture.taboo_count");
                for a in &sim.agents {
                    st.values.push(a.cultural_cognition.taboos.len() as f64);
                }
                println!("{}", st.report());

                let mut st = Stats::new("tom.model_count");
                let mut st2 = Stats::new("policy.decision_count");
                for a in &sim.agents {
                    st.values.push(a.mind_models.models.len() as f64);
                    st2.values.push(a.decision_policy.decision_count as f64);
                }
                println!("{}", st.report());
                println!("{}", st2.report());

                println!(
                    "{}",
                    counts(
                        "reg.strategy",
                        &sim.agents
                            .iter()
                            .map(|a| a.emotion_regulation.preferred)
                            .collect::<Vec<_>>()
                    )
                );
                println!(
                    "{}",
                    counts(
                        "narr.life_theme",
                        &sim.agents.iter().map(|a| a.narrative.life_theme).collect::<Vec<_>>()
                    )
                );
                println!(
                    "{}",
                    counts(
                        "prosp.scenario_kind",
                        &sim.agents
                            .iter()
                            .flat_map(|a| a.prospection.scenarios.iter().map(|s| s.kind))
                            .collect::<Vec<_>>()
                    )
                );
                println!(
                    "{}",
                    counts(
                        "motive.dominant",
                        &sim.agents
                            .iter()
                            .map(|a| a.motivation.dominant_need)
                            .collect::<Vec<_>>()
                    )
                );
                let focal = sim
                    .agents
                    .iter()
                    .filter(|a| a.agent_tier.tier == AgentTier::Focal)
                    .count();
                let secondary = sim
                    .agents
                    .iter()
                    .filter(|a| a.agent_tier.tier == AgentTier::Secondary)
                    .count();
                println!("  {:<28} focal:{focal} secondary:{secondary}", "lod.tiers");
                println!();
            }
        }
    }

    println!("── EMOTION LIVENESS LEDGER (nonzero in any window) ──────");
    for (emotion_name, live) in &emotion_ledger {
        let flag = if *live { "LIVE" } else { "DEAD" };
        println!("  {emotion_name:<18} {flag}");
    }
}
