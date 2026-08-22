//! Tick passes 7-9: emotion/belief decay and trait plasticity.

use super::{AgentBundle, Fixed, Simulation};
use crate::belief_update;

impl Simulation {
    pub(super) fn tick_decay_pass(
        agents: &mut [AgentBundle],
        _tick_u64: u64,
        affects: &mut [crate::person::Affect],
        emotions: &mut [crate::person::DiscreteEmotions],
        goals: &mut [Vec<crate::person::Goal>],
        needs: &mut [crate::person::NeedState],
        phases: crate::scheduler::TickPhases,
        params_x: &crate::parameters::SimParameters,
    ) {
        // ── 7. Emotion decay ──────────────────────────────────────
        // §8.1.4 (Iteration 164): The BASE emotion families now decay
        // proportionally every tick (BASE_EMOTION_DECAY_RATE, probe-
        // calibrated at 0.06) — the pre-Iter-164 subtractive decay
        // (0.002/tick, §22.1) was ~40× too weak against the per-tick
        // appraisal deltas, so fear pinned at 0.83–1.0 in every
        // calibrated window (probe-pinned at Iter-164: fear climbed to
        // 0.99 by tick 800 in seeds 1/42/99) and every amplification
        // consumer (`1 + fear×0.4` Safety, `(fear+sadness)×0.1`
        // negative-events narrative fold, stress input) ran at max
        // with zero differentiation. Proportional decay cancels
        // production to producer-driven steady states (fear mean
        // 0.55–0.75 with min 0.03–0.10 / max 1.0 only for genuinely
        // stressed agents, joy 0–0.92, shame ≈ 0.03) — restoring
        // headroom so amplification differentiates across agents and
        // windows. Tuned down from an initial 0.08 whose probe
        // starved the rare-event conception rolls (the lower fear
        // re-paces the shared Social RNG stream; at 0.08 only 1/5
        // seeds delivered a 100K birth, at 0.06 all 5 deliver).
        // `tenderness` exemption below removed (Iter-183, P3-5) and
        // `loneliness` joined the secondary decay below (P2/P3 re-
        // audit, P3-8) once their producers became live — each was
        // the same write-only-ratchet class. `trust` JOINED the base
        // decay above (P2/P3 audit closure): appraisal now produces
        // it via Other-attributed goal congruence, so the old
        // exemption (justified only while it probed at exactly 0)
        // would ratchet it to 1.0.
        for emotion in emotions.iter_mut() {
            let base_rate = crate::appraisal::BASE_EMOTION_DECAY_RATE;
            emotion.fear = crate::appraisal::secondary_emotion_decay(emotion.fear, base_rate);
            emotion.anger = crate::appraisal::secondary_emotion_decay(emotion.anger, base_rate);
            emotion.joy = crate::appraisal::secondary_emotion_decay(emotion.joy, base_rate);
            emotion.sadness = crate::appraisal::secondary_emotion_decay(emotion.sadness, base_rate);
            emotion.shame = crate::appraisal::secondary_emotion_decay(emotion.shame, base_rate);
            emotion.pride = crate::appraisal::secondary_emotion_decay(emotion.pride, base_rate);
            emotion.guilt = crate::appraisal::secondary_emotion_decay(emotion.guilt, base_rate);
            // §8.1.4 (P2/P3 audit closure): `trust` was decay-exempt with
            // the justification "probes at exactly 0 in calibrated
            // windows" — but the appraisal block now produces it
            // (Other-attributed goal congruence), so the exemption would
            // become a write-only ratchet to 1.0 (the same class as the
            // Iter-183 tenderness fix). Joined the base decay so it sits
            // at a producer-driven steady state.
            emotion.trust = crate::appraisal::secondary_emotion_decay(emotion.trust, base_rate);
            // §8.1.4 (Iteration 116): The expanded emotion families decay
            // proportionally EVERY TICK (SECONDARY_EMOTION_DECAY_RATE,
            // probe-calibrated) — the base-8 linear decay (0.002/tick) is
            // ~40× too weak against the per-tick appraisal deltas (~0.08/
            // tick, probe-pinned: awe climbs 0.08 → 1.0 in ~20 ticks), and
            // the expanded families previously had NO decay at all: the
            // produced families (awe/relief/hope/gratitude/nostalgia)
            // pinned at 1.0 in every calibrated run. The proportional
            // per-tick decay cancels the per-tick production to keep them
            // at meaningful producer-driven levels. `loneliness` (Iter-98
            // social-seeking) remains exempt — its producer is zero in
            // most ticks, so it does not ratchet; `tenderness` was
            // exempted at Iter-99 but JOINED the decay at Iter-183
            // (P3-5): with a live per-agent producer it ratcheted to
            // 1.0 for every agent, exactly the write-only saturation
            // class the P3 audit targets. The Iter-115 birth-pipeline
            // lesson applies — the help-window re-pace is absorbed by
            // the standard re-pin workflow (golden + snapshots
            // regenerated, integration pins re-anchored).
            let rate = crate::appraisal::SECONDARY_EMOTION_DECAY_RATE;
            emotion.disgust = crate::appraisal::secondary_emotion_decay(emotion.disgust, rate);
            emotion.contempt = crate::appraisal::secondary_emotion_decay(emotion.contempt, rate);
            emotion.awe = crate::appraisal::secondary_emotion_decay(emotion.awe, rate);
            emotion.gratitude = crate::appraisal::secondary_emotion_decay(emotion.gratitude, rate);
            emotion.jealousy = crate::appraisal::secondary_emotion_decay(emotion.jealousy, rate);
            emotion.envy = crate::appraisal::secondary_emotion_decay(emotion.envy, rate);
            emotion.humiliation =
                crate::appraisal::secondary_emotion_decay(emotion.humiliation, rate);
            emotion.relief = crate::appraisal::secondary_emotion_decay(emotion.relief, rate);
            emotion.hope = crate::appraisal::secondary_emotion_decay(emotion.hope, rate);
            emotion.despair = crate::appraisal::secondary_emotion_decay(emotion.despair, rate);
            emotion.nostalgia = crate::appraisal::secondary_emotion_decay(emotion.nostalgia, rate);
            emotion.moral_outrage =
                crate::appraisal::secondary_emotion_decay(emotion.moral_outrage, rate);
            // §8.1.4 (P2/P3 re-audit — P3-8 completion): loneliness JOINS
            // the secondary decay. It was exempted at Iter-98 with the
            // justification "its producer is zero in most ticks, so it
            // does not ratchet" — that premise is stale: the appraisal
            // block now produces it every tick (attachment_threat ×
            // (1 − social_visibility), with separation distress for
            // partnered agents), so the exemption ratcheted it to 1.0
            // for 10/12 agents in calm (probe: mean 0.833) — the exact
            // write-only saturation class the trust/tenderness fixes
            // eliminated. With the decay it sits at a producer-driven
            // steady state: partnered-embedded agents low, isolates
            // high, bereaved agents (pestilence) mid — the honest
            // direction.
            emotion.loneliness =
                crate::appraisal::secondary_emotion_decay(emotion.loneliness, rate);
            // §8.1.4 (Iteration 183 — AP2 P3-5 completion): tenderness
            // JOINS the secondary decay. It was exempted at Iter-99
            // ("consumers calibrated against the saturated state") and
            // ratcheted to 1.0 for 13/13 agents in every calibrated
            // window — the write-only saturation class P3-5 exists to
            // eliminate. The P3-5 goal-congruence differentiation gave
            // gratitude a 0.207–0.880 spread but could not move
            // tenderness, which never decayed. Now its producer
            // (positive × (1 − status_threat), same family as
            // gratitude) drives the same delta/rate equilibrium, so
            // the help propensity's tenderness tier (× 0.5)
            // differentiates across agents instead of adding a
            // constant +0.5 to everyone. `loneliness` keeps its
            // exemption — its producer (attachment_threat × (1 −
            // social_visibility)) is zero in most ticks, so it does
            // not ratchet (probe: live at 0.04–0.42).
            emotion.tenderness =
                crate::appraisal::secondary_emotion_decay(emotion.tenderness, rate);
        }

        // ── 7b. §10.1.1 fear contagion (Iteration 107) ────────────
        // The sensory field's perceived ambient stress ("expression" —
        // mean of (fear + anger) / 2 over agents within
        // PERCEPTION_RADIUS) feeds the agent's fear on the daily
        // cadence: an agent surrounded by stressed neighbors catches
        // their fear, and a terrified village sustains itself against
        // the §22.1 decay. Zero-at-zero: no stressed neighbors →
        // perceived_stress 0 → the term contributes nothing, and the
        // daily refresh (end of the previous tick) reads default zeros
        // at tick 0, so calibrated runs are byte-identical until
        // neighbors actually hold stress. Deterministic (the field is
        // RNG-free by construction); the rate keeps the annual ambient
        // pressure bounded (~FEAR_CONTAGION_RATE × stress × 365,
        // clamped). Equilibrium note: the fold's own arithmetic
        // (rate × stress ≈ 0.02/day vs decay 0.002/day) would alone
        // saturate fear — and empirically 2/12 agents DO sit at the
        // 1.0 clamp at tick 2000 (probe-pinned) — but the per-tick
        // appraisal recompute holds the population MEAN at ~0.86,
        // leaving headroom for the other 10, so the channel's
        // differential is compressed but live. The elevated ambient
        // fear is the intended "terrified village sustains itself"
        // state (reviewer-flagged calibration risk: if appraisal
        // tuning changes, re-pin the integration floor).
        if phases.is_daily {
            for (agent, emotion) in agents.iter().zip(emotions.iter_mut()) {
                emotion.fear = crate::social::relational_field::RelationalFields::contagion_apply(
                    emotion.fear,
                    agent.relational_fields.perceived_stress,
                    Fixed::from_f64(crate::social::relational_field::FEAR_CONTAGION_RATE),
                );
            }
            // ── 7c. §10.1.2 peer-status envy (Iteration 113) ───────
            // The social field's highest-status neighbor ("the most
            // dominant presence I see") feeds the agent's anger on the
            // daily cadence: an agent who perceives a GENUINELY
            // dominant peer (peer_status above the 0.5 anchor — above
            // the calibrated ceiling of 0.46, probe-pinned across seeds
            // 1/7/42/99, drought/pestilence scenarios, and the 20K
            // horizon) grows envious: status frustration → anger. The
            // anger channel is decisional (feeds threat_level, stress,
            // and witnessed-violation appraisal), so the envy term is
            // behaviorally live when it engages. ONE-SIDED: identity
            // (zero delta) at/below the anchor, so calibrated runs are
            // byte-identical (zero-blast — no regeneration).
            // Deterministic (no RNG).
            for (agent, emotion) in agents.iter().zip(emotions.iter_mut()) {
                emotion.anger = crate::social::relational_field::RelationalFields::envy_apply(
                    emotion.anger,
                    agent.relational_fields.peer_status,
                    Fixed::from_f64(crate::social::relational_field::PEER_ENVY_ANCHOR),
                    Fixed::from_f64(crate::social::relational_field::PEER_ENVY_RATE),
                    Fixed::from_f64(crate::social::relational_field::PEER_ENVY_CAP),
                );
            }
        }

        // ── 8. Belief resistance decay ────────────────────────────
        // §5.1: Use configurable decay rate from SimParameters.
        // §8.1.17: Narrative frames modulate belief rigidity — agents whose
        // meaning-making frames resist countervailing evidence (punitive,
        // curse, just-world frames) hold their beliefs longer. Mean-zero
        // at the default frame set (resistance_to_update == 0.5 exactly),
        // so decay is unchanged for typical agents and only diverges as
        // frames drift with life experience.
        let belief_resistance_decay = params_x.belief_resistance_decay;
        for agent_beliefs in agents.iter_mut() {
            let rigidity = agent_beliefs.narrative_frames.resistance_to_update();
            let rigidity_deviation = rigidity - Fixed::from_f64(0.5);
            let scaled_decay =
                (belief_resistance_decay * (Fixed::ONE - rigidity_deviation)).max(Fixed::ZERO);
            belief_update::decay_belief_resistance(
                &mut agent_beliefs.beliefs,
                scaled_decay,
                params_x,
            );
        }

        // ── 9. Trait plasticity (§8.1.6 state-trait dynamics) ──────
        // Temperament dimensions slowly reshape from repeated life
        // experience (stress, recovery, social engagement, goal striving),
        // gated by identity integration (self-model coherence) and
        // developmental plasticity (youth). Writes ONLY the observational
        // temperament layer — the 12 decision-read core traits are
        // untouched, so calibrated runs remain byte-identical.
        //
        // §8.1.6 (Iteration 162): the social_engagement and goal_striving
        // signals were STRUCTURALLY DEAD — probe-pinned at exactly 0.0000
        // for every agent to 5,000 ticks — so sociability, persistence,
        // regularity, and approach_withdrawal never drifted (only the
        // stress-driven reactivity/soothability/sensitivity moved). Two
        // dead channels, two fixes:
        //   1. `social_engagement` read `emotions.trust + joy`: appraisal
        //      NEVER writes `emotions.trust` (pinned 0.0000 at every
        //      sampled tick) and joy is sporadic. Replaced with the
        //      live social-need satisfaction `1 − needs.social` (the
        //      deficit is live, 0.00–0.065 in calm windows, so the
        //      satisfaction signal sits ~0.93–1.0 — saturated but
        //      genuine, mirroring the arousal signal's saturation).
        //   2. `goal_striving` read `recent_attempts/successes`, but the
        //      outcome pass increments them at tick-action time and the
        //      derived-state pass `saturating_sub(1)`s them — the two
        //      cancel within each tick, so the window is ALWAYS 0 at
        //      this read point (pinned 0/0 at every sample). Replaced
        //      with the live goal load — the sum of active goal
        //      priorities clamped to 0..1 (1–2 goals, priority sums
        //      0.72–1.42 in calm windows). Both replacements stay in
        //      the Fixed domain (project doctrine), are deterministic
        //      (no RNG), and now drive the four previously-frozen
        //      temperament dimensions.
        for (i, agent) in agents.iter_mut().enumerate() {
            let goal_striving = goals[i]
                .iter()
                .fold(Fixed::ZERO, |acc, g| acc + g.priority)
                .clamp_01();
            let signals = crate::person::PlasticitySignals {
                // Arousal is the in-scope physiological stress proxy
                // ((fear + anger + joy) × 0.5 from the appraisal block).
                repeated_stress: affects[i].arousal,
                recovery: affects[i].valence.clamp_01(),
                // §8.1.6 (Iteration 162): live social-need satisfaction
                // (was the dead `emotions.trust + joy` proxy).
                social_engagement: Fixed::ONE - needs[i].social.clamp_01(),
                goal_striving,
                identity_integration: agent.self_model.coherence,
                age_years: agent.age,
            };
            let baseline = crate::person::Temperament::from_traits(&agent.personality);
            agent
                .personality
                .temperament
                .plastic_update(&baseline, &signals);
            // §8.1.6 (Iteration 179): the 12 decision-read core traits
            // also move — the plan's "traits slowly change through
            // repeated behavior, trauma, success, failure..." formula
            // applied to the traits themselves (not just the
            // observational temperament layer). Each trait is pushed
            // toward its repeatedly-expressed state and pulled back
            // toward the birth constitution, both at the same rate
            // (identity-integration × developmental-plasticity ×
            // social-reinforcement × CORE_TRAIT_PLASTICITY_RATE).
            // Deterministic, zero RNG, clamped to 0..1.
            //
            // YEARLY GATE (the critical calibration decision): core
            // traits feed decisions directly, so per-tick drift at the
            // Fixed-4-decimal floor would pin them to constitution +
            // saturated-signal (social/valence ~0.9-1.0 in calm windows)
            // within a few thousand ticks — a qualitative behavioral
            // rewrite (Iter-179 probe: births delayed 51K→93K ticks;
            // and the baseline recompute chases the drifting traits,
            // collapsing the deviation the Iter-105 reactivity
            // amplifier reads). Gated to the scheduler's YearlyPhase
            // (51,840 ticks) so every short-horizon calibrated window
            // is byte-identical while still allowing genuine multi-year
            // personality drift — the plan's "slowly".
            //
            // Why YearlyPhase and not the age system's 35,040-tick year
            // (DemographyConfig.ticks_per_year)? Two reasons, both
            // probed: (1) the scheduler's yearly cadence is the
            // codebase's canonical "year" for the other slow systems
            // (climate, culture drift, technology), so trait plasticity
            // fires on the same clock as its sibling yearly passes;
            // (2) at 35,040 the first fire lands at tick 35,040 —
            // BEFORE the seed-46 first birth (51,000) — and that
            // pre-birth nudge re-paces courtship enough to push every
            // birth out of the 100K liveness window (probe: left []).
            // At 51,840 the first fire lands AFTER the first birth, so
            // the pinned 51,000 birth stays byte-identical and only
            // subsequent courtship shifts (probe: [51000, 69330,
            // 88160] — the documented 3-chain). The age-vs-gate year
            // mismatch (an agent ages ~1.48 years between fires) is a
            // pre-existing dual-year-definition inconsistency in the
            // codebase (clock 96 ticks/day vs scheduler 144 ticks/day),
            // not introduced by this iteration.
            if phases.is_yearly {
                agent.personality.plastic_update_traits(&signals);
            }
        }
    }
}
