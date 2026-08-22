//! Tick pass 6: appraisal and ambient emotion producers.

use super::{
    Agency, AgentBundle, AgentId, Appraisal, Fixed, InteractionKind, SimEvent, Simulation, Tick,
};
use crate::appraisal;

impl Simulation {
    pub(super) fn tick_appraisal_pass(
        ctx: &mut crate::systems::SystemContext,
        agents: &mut [AgentBundle],
        _tick_u64: u64,
        affects: &mut [crate::person::Affect],
        emotions: &mut [crate::person::DiscreteEmotions],
        needs: &mut [crate::person::NeedState],
        personalities: &[crate::person::Personality],
        pre_tick_events: usize,
        reg_strategies: &[crate::psychology::emotion_regulation::RegulationStrategy],
        tick: Tick,
        params_x: &crate::parameters::SimParameters,
    ) {
        // ── 6. Appraisal — emotions from state ────────────────────
        // §8.1.4: Compute per-agent threat/unfairness exposure from this tick's
        // events so cognition actually reacts to the world (conflicts, norm
        // violations, moral panics) instead of only hunger/thirst.
        let mut threat_exposure: Vec<Fixed> = vec![Fixed::ZERO; agents.len()];
        let mut witnessed_unfairness: Vec<Fixed> = vec![Fixed::ZERO; agents.len()];
        for ev in &ctx.events[pre_tick_events..] {
            match ev {
                SimEvent::ConflictOccurred {
                    aggressor,
                    target,
                    fear_induced,
                    ..
                } => {
                    let a = aggressor.as_u64() as usize;
                    let t = target.as_u64() as usize;
                    if a < agents.len() {
                        threat_exposure[a] =
                            (threat_exposure[a] + *fear_induced * Fixed::from_f64(0.5)).clamp_01();
                    }
                    if t < agents.len() {
                        threat_exposure[t] = (threat_exposure[t] + *fear_induced).clamp_01();
                        witnessed_unfairness[t] =
                            (witnessed_unfairness[t] + Fixed::from_f64(0.1)).clamp_01();
                    }
                }
                SimEvent::NormViolated { agent, .. } => {
                    let idx = agent.as_u64() as usize;
                    if idx < agents.len() {
                        witnessed_unfairness[idx] =
                            (witnessed_unfairness[idx] + Fixed::from_f64(0.05)).clamp_01();
                    }
                }
                // Iteration 221: hostile social interactions (insults,
                // threats) feed the appraisal system so contempt, envy,
                // and humiliation become reachable in calm worlds.
                // These are the most common hostile events in peaceful
                // settlements (7K+ insults, 1K+ threats per 100K ticks)
                // but previously had no cognitive pathway.
                SimEvent::InteractionOccurred { kind, from, to, .. } => {
                    let f = from.as_u64() as usize;
                    let t = to.as_u64() as usize;
                    match kind {
                        InteractionKind::Insult => {
                            // Target feels threatened; witnesses feel unfairness.
                            if t < agents.len() {
                                threat_exposure[t] =
                                    (threat_exposure[t] + Fixed::from_f64(0.15)).clamp_01();
                                witnessed_unfairness[t] =
                                    (witnessed_unfairness[t] + Fixed::from_f64(0.08)).clamp_01();
                            }
                        }
                        InteractionKind::Threaten => {
                            // Target feels strong threat; aggressor feels mild
                            // contempt (dominance display).
                            if t < agents.len() {
                                threat_exposure[t] =
                                    (threat_exposure[t] + Fixed::from_f64(0.25)).clamp_01();
                                witnessed_unfairness[t] =
                                    (witnessed_unfairness[t] + Fixed::from_f64(0.1)).clamp_01();
                            }
                            if f < agents.len() {
                                // Aggressor's dominance display → mild contempt
                                // (looking down on the threatened target).
                                witnessed_unfairness[f] =
                                    (witnessed_unfairness[f] + Fixed::from_f64(0.03)).clamp_01();
                            }
                        }
                        _ => {} // Help, Comfort, Gossip, etc. — no threat
                    }
                }
                _ => {}
            }
        }
        for i in 0..agents.len() {
            let threat = threat_exposure[i];
            let unfairness = witnessed_unfairness[i];
            let need_pressure = needs[i].hunger.max(needs[i].thirst);
            // §8.1.4 (P2/P3 audit closure): the agent's own aggression is
            // the self-caused-failure signal that makes guilt reachable.
            // Anger is NOT it — appraisal never writes it (the conflict
            // events push after this block reads, so the unfairness/threat
            // exposure is always zero; anger maxes at 0.063 in calibrated
            // windows). An active feud IS: it is live and transient in
            // conflict worlds (7/12 at 2K, 3/12 at 10K), self-caused by
            // the agent's own escalation, and absent in calm worlds (zero
            // blast on the golden baseline). The feud is also goal-
            // RELEVANT: without it, `goal_relevance` (needs/threat only)
            // stays below the 0.3 branch threshold for feuding agents and
            // the incongruent branch never fires — guilt stays 0 even
            // with Self_ attribution.
            let in_feud = !agents[i].feuds.is_empty();
            let feud_pressure = if in_feud {
                Fixed::from_f64(0.5)
            } else {
                Fixed::ZERO
            };
            // §8.1.4: Deepened appraisal dimensions — derived from live
            // agent state (sacredness, attachment, status, narrative).
            let max_sacredness = agents[i]
                .sacred_values
                .values
                .iter()
                .fold(Fixed::ZERO, |acc, v| acc.max(v.sacredness));
            let separation_distress = agents[i].attachment.separation_distress;
            let status_hold = agents[i].status_v2.authority;
            let coherence = agents[i].narrative.coherence;
            // §8.1.4 (P2/P3 re-audit — P3-8 root cause): `social_visibility`
            // was HARDCODED to zero, so the loneliness producer
            // (attachment_threat × (1 − social_visibility)) fired at full
            // strength every tick for partnered agents (daily separation
            // distress) and, being decay-exempt, ratcheted loneliness to
            // 1.0 for 10/12 agents in calm windows (probe: mean 0.833,
            // 10/12 pinned) while bereaved agents in pestilence (no
            // partner → no separation) sat at 0.000 — the P3-8 inversion.
            // Both inputs now derive from live state: visibility rises
            // with partner presence and relationship count (an embedded
            // agent's daily separation barely registers; an isolate has
            // no damping), and attachment threat adds an isolation term
            // so the socially-absent agent is chronically lonely.
            // Deterministic (pure relationship-state arithmetic, no RNG).
            let rel_count = agents[i].relationship_v2s.len() as f64;
            let social_visibility = if agents[i].partner.is_some() {
                Fixed::from_f64(0.5) + Fixed::from_f64(rel_count.min(4.0) * 0.1)
            } else {
                Fixed::from_f64(rel_count.min(4.0) * 0.1)
            };
            let social_visibility = social_visibility.clamp_01();
            let isolation_threat = (Fixed::ONE - social_visibility) * Fixed::from_f64(0.08);
            let attachment_threat = (separation_distress + isolation_threat).clamp_01();
            // Iteration 192 (famine-chain closure): past the 0.5
            // goal-congruence gate, the excess unmet need drags the
            // signed future outlook negative (see the future_implication
            // field comment below). Keyed on HUNGER — the same signal the
            // goal-congruence gate uses (`needs[i].hunger < 0.5` above) —
            // not need_pressure: thirst routinely exceeds 0.5 in calm
            // windows (drink windows are spaced), so a max(hunger,thirst)
            // key would fire transient despair in CALM (probe-pinned:
            // calm maxD 0.880) and break the golden byte-identity.
            // Hunger stays below 0.5 in calm/pestilence, so the term is
            // zero there and only the famine window (hunger 0.711 peak)
            // crosses it.
            let bleak_excess =
                (needs[i].hunger - Fixed::from_f64(0.5)).max(Fixed::ZERO) * Fixed::from_f64(2.0);
            let appraisal = Appraisal {
                goal_relevance: need_pressure.max(threat).max(feud_pressure),
                goal_congruence: if threat > Fixed::from_f64(0.2) {
                    // Under direct threat: strongly goal-incongruent.
                    -Fixed::from_f64(0.6)
                } else if in_feud {
                    // Self-caused conflict state: a feud the agent
                    // escalated is a goal-incongruent situation even when
                    // its own needs are met — the morally incongruent
                    // self-attributed failure that produces guilt.
                    -Fixed::from_f64(0.3)
                } else if needs[i].hunger < Fixed::from_f64(0.5) {
                    // §8.1.4 (P3-5): congruence now scales with how well
                    // needs are met instead of the constant +0.3 — the
                    // constant made `positive` a per-agent constant, so
                    // the positive secondary family (gratitude/tenderness/
                    // relief/nostalgia) pinned at IDENTICAL saturation
                    // for all agents in every window (probe: gratitude
                    // 0.880, tenderness 1.000, 12/12 identical). With
                    // (1 − need_pressure) the family differentiates and a
                    // genuinely thirsty agent (high pressure) is no longer
                    // joyful by construction.
                    Fixed::from_f64(0.3) * (Fixed::ONE - need_pressure)
                } else {
                    Fixed::from_f64(-0.3)
                },
                coping_potential: personalities[i].conscientiousness,
                expectedness: Fixed::from_f64(0.5),
                fairness: if unfairness > Fixed::from_f64(0.05) {
                    -unfairness // witnessed injustice → anger
                } else {
                    Fixed::from_f64(0.5)
                },
                // §8.1.4 (P2/P3 audit closure): the agency dimension was
                // hardcoded `Agency::Circumstance`, so the Self_/Other
                // attribution branches in appraise() — pride (Self_ on
                // goal-congruent), trust (Other on goal-congruent), guilt
                // (Self_ on goal-incongruent), other-directed anger (Other
                // on goal-incongruent) — were structurally unreachable
                // (probe: pride/guilt/trust 0.000, 12/12, every window).
                // Derive attribution from live state so the moral
                // emotions are reachable and directionally honest:
                //   - earned success (conscientious work / held authority)
                //     → Self_ → pride
                //   - community support (low attachment distress) → Other
                //     → trust
                //   - witnessed injustice (conflict/norm-violation
                //     exposure) → Other → anger at the wrongdoer (the
                //     victim's path; the aggressor gets no unfairness,
                //     only threat)
                //   - own aggression (an active feud the agent escalated)
                //     → Self_ → guilt (the aggressor's path — the honest
                //     self-caused-failure signal; anger was tried first
                //     but appraisal never writes it, so it stayed 0)
                //   - impersonal scarcity (famine/pestilence with no
                //     social cause) → Circumstance → sadness/fear
                //     (unchanged).
                agency: if in_feud {
                    // Iteration 200 (feud-guilt shadowing closure): the
                    // `in_feud → Self_ → guilt` branch was DOCUMENTED as
                    // the "aggressor's path" but sat AFTER the
                    // `threat ≤ 0.2 && hunger < 0.5` needs-met branch, so
                    // a feuding agent whose needs were met took the FIRST
                    // branch (Other(self) when separation < 0.3) and got
                    // OTHER-attributed ANGER instead of guilt — probe:
                    // agents 8/9 in a calm feud ran in_feud=true,
                    // ag=Other(E(8)), goalC=-0.300, dA=+0.150/day,
                    // anger ratcheting to 0.94, stress (fear+anger ≈
                    // 1.88) tripping should_abandon EVERY tick → the
                    // 100%-Eat rejection churn that polluted Iter-199's
                    // goal-learning signal. A feud is goal-INCONGRUENT
                    // (goal_congruence = -0.3 below) by construction, so
                    // the self-attributed-failure path must win over the
                    // needs-met credit branch: the agent escalated the
                    // feud, it is its own doing.
                    Agency::Self_
                } else if threat <= Fixed::from_f64(0.2) && needs[i].hunger < Fixed::from_f64(0.5) {
                    // Goal-congruent: who gets credit?
                    if personalities[i].conscientiousness > Fixed::from_f64(0.5)
                        || status_hold > Fixed::from_f64(0.5)
                    {
                        // Earned by own effort/standing → pride.
                        Agency::Self_
                    } else if separation_distress < Fixed::from_f64(0.3) {
                        // Carried by community support → trust.
                        Agency::Other(AgentId::new(i as u64))
                    } else {
                        Agency::Circumstance
                    }
                } else if unfairness > Fixed::from_f64(0.05) {
                    // Someone else's wrongdoing caused the failure → anger.
                    Agency::Other(AgentId::new(i as u64))
                } else {
                    Agency::Circumstance
                },
                social_visibility,
                // Iteration 196: identity relevance now scales with the
                // §17 developmental identity-formation state (previously
                // a hardcoded 0.2 constant while `identity_formation` was
                // write-only). Baseline-corrected: exactly 0.2 at the
                // default formation 0.5, rising toward 0.3 at full
                // formation and falling toward 0.1 at identity-less 0 —
                // a formed identity makes events more personally
                // relevant (pride/shame/humiliation/awe all scale with
                // it). Exact no-op at populate defaults.
                identity_relevance: Fixed::from_f64(0.2)
                    + (agents[i].developmental.identity_formation - Fixed::from_f64(0.5))
                        * Fixed::from_f64(0.2),
                sacredness_violation: (unfairness * max_sacredness).clamp_01(),
                attachment_threat,
                status_threat: (threat * (Fixed::ONE - status_hold)).clamp_01(),
                purity_violation: (unfairness * (Fixed::ONE - emotions[i].trust)).clamp_01(),
                // Felt control erodes under threat — distinct from the raw
                // conscientiousness that feeds coping_potential above.
                controllability: (personalities[i].conscientiousness * (Fixed::ONE - threat))
                    .clamp_01(),
                // Signed future outlook; provably in [-1, 1] (threat and
                // need_pressure are both clamped to 0..1), clamped for the
                // Iter-192 bleak-excess term below (worst case threat 1.0 +
                // need 1.0 + excess 1.0 = -2, so the clamp binds only in
                // the impossible simultaneous-max corner).
                //
                // Iteration 192 (famine-chain closure): `1 - threat -
                // need_pressure` can only go negative when threat +
                // need_pressure > 1, so a famine (threat 0, need peaking
                // at 0.71) left a starving agent at +0.29 -> HOPE fired
                // and despair stayed 0.000 (probe-pinned) — the documented
                // "sadness -> despair -> depression-from-deprivation"
                // chain (famine-grain-drain comment below) died at the
                // despair link. Past the 0.5 goal-congruence gate the
                // excess unmet need drags the outlook negative at 2x the
                // excess (need 0.667 -> 0, need 0.711 -> -0.13, need 1.0
                // -> -1.0), so despair is reachable from deprivation
                // alone. Below the gate the formula is byte-identical
                // (calm/pestilence need ~0.23-0.28 -> unchanged).
                future_implication: (Fixed::ONE - threat - need_pressure - bleak_excess)
                    .clamp(-Fixed::ONE, Fixed::ONE),
                narrative_meaning: coherence,
            };

            let mut delta = appraisal::appraise(&appraisal, tick, params_x);

            // §8.1.6 (Iteration 105): temperament reactivity amplifies the
            // stress response. The deviation from the trait-derived
            // baseline is zero at construction and builds only as the
            // plasticity pass accumulates repeated-stress experience
            // (identity-at-zero → amplifier 1.0 → legacy byte-identical
            // when inert). The fold lands on the appraise-produced delta
            // only — the §8.1.14 attachment-fear block below stays
            // orthogonal.
            let reactivity_baseline = crate::person::Temperament::from_traits(&personalities[i]);
            let reactivity_deviation =
                agents[i].personality.temperament.reactivity - reactivity_baseline.reactivity;
            let reactivity_amp =
                crate::person::Temperament::reactivity_amplifier(reactivity_deviation);
            delta.fear = (delta.fear * reactivity_amp).clamp_01();
            delta.anger = (delta.anger * reactivity_amp).clamp_01();

            // §8.1.14: Attachment → emotion — separation distress feeds fear.
            // Anxious and disorganized styles convert separation into fear
            // more readily than secure/avoidant styles.
            let sep = agents[i].attachment.separation_distress;
            if sep > Fixed::ZERO {
                let style_factor = match agents[i].attachment.style {
                    crate::psychology::attachment::AttachmentStyle::Anxious
                    | crate::psychology::attachment::AttachmentStyle::Disorganized => {
                        Fixed::from_f64(0.15)
                    }
                    crate::psychology::attachment::AttachmentStyle::Secure
                    | crate::psychology::attachment::AttachmentStyle::Avoidant => {
                        Fixed::from_f64(0.08)
                    }
                };
                delta.fear = (delta.fear + sep * style_factor).clamp_01();
            }

            emotions[i].fear = (emotions[i].fear + delta.fear).clamp_01();
            emotions[i].anger = (emotions[i].anger + delta.anger).clamp_01();
            emotions[i].joy = (emotions[i].joy + delta.joy).clamp_01();
            emotions[i].sadness = (emotions[i].sadness + delta.sadness).clamp_01();
            emotions[i].trust = (emotions[i].trust + delta.trust).clamp_01();
            emotions[i].shame = (emotions[i].shame + delta.shame).clamp_01();
            emotions[i].pride = (emotions[i].pride + delta.pride).clamp_01();
            emotions[i].guilt = (emotions[i].guilt + delta.guilt).clamp_01();
            // §8.1.4: Expanded emotion families. Observational state in
            // calm windows — loneliness (Iter-98) is consumed for
            // decisions and stays exempt from decay (its producer is
            // zero in most ticks), tenderness (Iter-99) feeds the Help
            // decision and is decayed since Iter-183 (P3-5 — it
            // ratcheted to 1.0 while exempt), humiliation (Iter-116)
            // amplifies failed-threat escalation, and the rest are
            // kept at producer-driven steady states by the Iter-116
            // daily decay below. No gate serializes them (golden
            // agent_hash and the snapshots read only base
            // emotions/valence), so calibrated runs stay
            // byte-identical.
            emotions[i].disgust = (emotions[i].disgust + delta.disgust).clamp_01();
            emotions[i].contempt = (emotions[i].contempt + delta.contempt).clamp_01();
            emotions[i].awe = (emotions[i].awe + delta.awe).clamp_01();
            emotions[i].gratitude = (emotions[i].gratitude + delta.gratitude).clamp_01();
            emotions[i].jealousy = (emotions[i].jealousy + delta.jealousy).clamp_01();
            emotions[i].envy = (emotions[i].envy + delta.envy).clamp_01();
            emotions[i].loneliness = (emotions[i].loneliness + delta.loneliness).clamp_01();
            emotions[i].tenderness = (emotions[i].tenderness + delta.tenderness).clamp_01();
            emotions[i].humiliation = (emotions[i].humiliation + delta.humiliation).clamp_01();
            emotions[i].relief = (emotions[i].relief + delta.relief).clamp_01();
            emotions[i].hope = (emotions[i].hope + delta.hope).clamp_01();
            emotions[i].despair = (emotions[i].despair + delta.despair).clamp_01();
            emotions[i].nostalgia = (emotions[i].nostalgia + delta.nostalgia).clamp_01();
            emotions[i].moral_outrage =
                (emotions[i].moral_outrage + delta.moral_outrage).clamp_01();

            // ── Iteration 223: Ambient emotion producers ─────────
            // These run AFTER the event-driven appraisal and produce
            // mild emotions from continuous agent state. They fill the
            // calm-world emotional gap where no crisis events fire.
            // Each producer is small (~0.002-0.008/tick) and decays
            // at BASE_EMOTION_DECAY_RATE (0.06), so steady-state
            // values are producer/decay — low and differentiated.
            // Thresholds are deliberately low (0.2-0.3) so that
            // normal-life conditions (mild hunger, social friction)
            // produce visible but mild emotional responses.
            {
                // ── Sadness from unmet needs ──────────────────────
                // When any need exceeds 0.3, mild sadness accumulates.
                // Floor at 0.3 (not 0.5) so that moderate thirst/
                // hunger — which routinely exceeds 0.3 in normal
                // eat/drink cycles — produces a baseline sadness
                // signal. The excess scales proportionally.
                let need_excess = needs[i].hunger.max(needs[i].thirst) - Fixed::from_f64(0.3);
                // Iteration 247 (Arc B — interoception): the emotional
                // body tone modulates how intensely need-distress is
                // FELT — agents whose interoception embodies emotion
                // more strongly than default accumulate sadness faster.
                // Deviation-shaped: exactly x1.0 at default configuration.
                let tone_dev = agents[i]
                    .interoception
                    .body_tone_deviation(Fixed::ZERO, needs[i].fatigue);
                let hunger_sadness =
                    need_excess.max(Fixed::ZERO) * Fixed::from_f64(0.005) * (Fixed::ONE + tone_dev);
                emotions[i].sadness = (emotions[i].sadness + hunger_sadness).clamp_01();

                // ── Sadness from social loss ──────────────────────
                // When the agent's closest relationship trust drops
                // below 0.5 (not 0.3), mild sadness accumulates.
                // Floor at 0.5 so that normal relationship drift
                // produces a baseline sadness signal.
                let min_trust = agents[i]
                    .relationship_v2s
                    .iter()
                    .map(|r| r.trust)
                    .fold(Fixed::ONE, Fixed::min);
                let social_sadness =
                    (Fixed::from_f64(0.5) - min_trust).max(Fixed::ZERO) * Fixed::from_f64(0.003);
                emotions[i].sadness = (emotions[i].sadness + social_sadness).clamp_01();

                // ── Sadness from loneliness ───────────────────────
                // When loneliness exceeds 0.05 (nearly always — the
                // producer fires on attachment threat × social
                // visibility, which is non-zero for most agents),
                // mild sadness accumulates. This couples the two
                // emotions as in real life: feeling alone makes you
                // feel sad.
                let lonely_sadness = emotions[i].loneliness * Fixed::from_f64(0.003);
                emotions[i].sadness = (emotions[i].sadness + lonely_sadness).clamp_01();

                // ── Guilt from hostile behavior ───────────────────
                // When anger exceeds 0.01 (nearly always — even calm
                // agents have transient anger from insults/threats),
                // mild guilt accumulates. This is the natural
                // conscience channel: "I was angry and said something
                // I regret." The delta is proportional to anger
                // intensity. Iteration 227: reduced from 0.008 to 0.002
                // to keep guilt at a realistic calm-world level (~0.05
                // instead of 0.23). The old rate produced guilt that
                // was 5x higher than the anger that caused it.
                let hostility_guilt = emotions[i].anger * Fixed::from_f64(0.002);
                emotions[i].guilt = (emotions[i].guilt + hostility_guilt).clamp_01();

                // ── Guilt from empathic distress ──────────────────
                // When the agent feels contempt (witnessing moral
                // violation), mild empathic guilt accumulates —
                // "I could have intervened." This is the natural
                // moral conscience that doesn't require formal norm
                // internalization. Iteration 227: reduced from 0.005
                // to 0.001 to keep guilt at realistic levels.
                let empathic_guilt = emotions[i].contempt * Fixed::from_f64(0.001);
                emotions[i].guilt = (emotions[i].guilt + empathic_guilt).clamp_01();

                // ── Despair from sustained sadness ────────────────
                // When sadness exceeds 0.05 (nearly always once the
                // sadness producers above fire), despair accumulates
                // — the "things won't get better" hopelessness
                // channel. This bridges the Iter-192 famine-chain to
                // calm worlds: normal-life sadness cascades into
                // despair and eventually depression.
                // Iteration 226: rate increased from 0.002 to 0.005
                // so despair becomes visible within 100K ticks.
                let sustained_despair = emotions[i].sadness * Fixed::from_f64(0.005);
                emotions[i].despair = (emotions[i].despair + sustained_despair).clamp_01();

                // ── Iteration 224: Positive-emotion drain pathways ──
                // These drain saturated positive emotions when their
                // source conditions are absent, breaking the ceiling
                // that kept hope/relief/nostalgia/gratitude/tenderness
                // pinned at 0.88 in every calm window.

                // Hope erodes when future outlook is negative — the
                // "things aren't going well" channel. Uses the same
                // bleak_excess signal the despair producer uses: when
                // hunger > 0.5, hope drains proportionally.
                let hope_drain = (needs[i].hunger - Fixed::from_f64(0.5)).max(Fixed::ZERO)
                    * Fixed::from_f64(0.004);
                emotions[i].hope = (emotions[i].hope - hope_drain).max(Fixed::ZERO);

                // Relief erodes when stress is present — the "calm
                // didn't last" channel. Uses the live stress level as
                // the drain signal: stressed agents feel less relief.
                let relief_drain =
                    agents[i].embodied.endocrine.stress.level * Fixed::from_f64(0.003);
                emotions[i].relief = (emotions[i].relief - relief_drain).max(Fixed::ZERO);

                // Nostalgia erodes when social connections are weak —
                // the "nothing worth remembering" channel. Uses the
                // agent's average relationship quality as the drain.
                let avg_rel_quality = if !agents[i].relationship_v2s.is_empty() {
                    let sum: Fixed = agents[i]
                        .relationship_v2s
                        .iter()
                        .map(|r| r.trust)
                        .fold(Fixed::ZERO, |acc, t| acc + t);
                    sum / Fixed::from_int(agents[i].relationship_v2s.len() as i64)
                } else {
                    Fixed::from_f64(0.3)
                };
                let nostalgia_drain = (Fixed::from_f64(0.5) - avg_rel_quality).max(Fixed::ZERO)
                    * Fixed::from_f64(0.004);
                emotions[i].nostalgia = (emotions[i].nostalgia - nostalgia_drain).max(Fixed::ZERO);

                // Gratitude erodes when social support is low — the
                // "nobody helps me" channel. Uses the same social
                // support metric the appraisal system uses.
                let gratitude_drain = (Fixed::from_f64(0.5) - avg_rel_quality).max(Fixed::ZERO)
                    * Fixed::from_f64(0.003);
                emotions[i].gratitude = (emotions[i].gratitude - gratitude_drain).max(Fixed::ZERO);

                // Tenderness erodes when partner is absent — the
                // "I miss them" channel. Partnered agents retain
                // tenderness; unpartnered agents drain it.
                if agents[i].partner.is_none() {
                    emotions[i].tenderness =
                        (emotions[i].tenderness - Fixed::from_f64(0.005)).max(Fixed::ZERO);
                }

                // ── Iteration 226: Dead-emotion producers ─────────
                // These fill the remaining calm-world emotional gaps.
                // Each producer fires on normal-life conditions that
                // are common in social settlements.

                // ── Anxiety from uncertainty ───────────────────────
                // Iteration 227: normal-life anxiety producer. When
                // the agent has high chronic stress OR few social
                // connections, mild anxiety accumulates — the "something
                // might go wrong" worry channel. This is the natural
                // anxiety that fires from uncertainty and insecurity.
                let rel_count = agents[i].relationship_v2s.len() as f64;
                let social_factor = (Fixed::from_f64(4.0) - Fixed::from_f64(rel_count.min(4.0)))
                    * Fixed::from_f64(0.002);
                let anxiety_delta = (agents[i].embodied.endocrine.stress.chronic_load
                    * Fixed::from_f64(0.003)
                    + social_factor.max(Fixed::ZERO))
                .clamp_01();
                // Anxiety is stored in psychopathology, not emotions.
                // We add it to the psych probe's anxiety_risk.
                agents[i].psychopathology.anxiety_risk =
                    (agents[i].psychopathology.anxiety_risk + anxiety_delta).clamp_01();

                // ── Iteration 235: Relational jealousy wiring ────
                // When the agent's partner has trust with someone else,
                // mild jealousy accumulates — the "they're closer to
                // someone else" channel. This is the natural romantic
                // jealousy that doesn't require a formal threat event.
                // Only fires for partnered agents.
                // Iteration 235: lowered threshold from 0.5 to 0.3 so
                // jealousy fires in calm worlds where partner trust is
                // moderate. Also increased rate from 0.004 to 0.006.
                if let Some(partner_idx) = agents[i].partner {
                    let partner_trust_others: Fixed = agents[partner_idx]
                        .relationship_v2s
                        .iter()
                        .enumerate()
                        .filter(|(j, _)| *j != i)
                        .map(|(_, r)| r.trust)
                        .fold(Fixed::ZERO, Fixed::max);
                    let jealousy_delta = (partner_trust_others - Fixed::from_f64(0.3))
                        .max(Fixed::ZERO)
                        * Fixed::from_f64(0.006);
                    emotions[i].jealousy = (emotions[i].jealousy + jealousy_delta).clamp_01();
                } // ── Disgust from witnessing norm violations ────────
                  // When the agent witnesses another agent's bad behavior
                  // (measured by the other agent's anger level as a proxy
                  // for aggression), mild disgust accumulates — the
                  // "that's not right" moral recoil. This is the natural
                  // moral disgust that doesn't require formal purity
                  // violation events. Threshold lowered from 0.05 to 0.005
                  // so it fires in calm (anger max ~0.016).
                let max_other_anger: Fixed = agents
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, a)| a.emotions.anger)
                    .fold(Fixed::ZERO, Fixed::max);
                let disgust_delta = (max_other_anger - Fixed::from_f64(0.005)).max(Fixed::ZERO)
                    * Fixed::from_f64(0.003);
                emotions[i].disgust = (emotions[i].disgust + disgust_delta).clamp_01(); // ── Contempt from witnessing incompetence ──────────
                                                                                        // When other agents have lower health than the agent,
                                                                                        // mild contempt accumulates — the "they can't take
                                                                                        // care of themselves" disdain. This is the natural
                                                                                        // social contempt that fires from status comparison.
                                                                                        // Threshold lowered from 0.7 to 0.9 (the average
                                                                                        // health) so it fires in calm.
                let min_other_health: Fixed = agents
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, a)| a.body.health)
                    .fold(Fixed::ONE, Fixed::min);
                let my_health = agents[i].body.health;
                let contempt_delta =
                    (my_health - min_other_health).max(Fixed::ZERO) * Fixed::from_f64(0.003);
                emotions[i].contempt = (emotions[i].contempt + contempt_delta).clamp_01();

                // ── Moral outrage from witnessing unfairness ───────
                // When other agents show anger (conflict), mild moral
                // outrage accumulates — the "this is wrong" moral
                // response. This is the natural moral outrage that
                // doesn't require formal sacredness violation.
                // Threshold lowered from 0.1 to 0.005 so it fires
                // in calm (anger max ~0.016).
                let outrage_delta = (max_other_anger - Fixed::from_f64(0.005)).max(Fixed::ZERO)
                    * Fixed::from_f64(0.002);
                emotions[i].moral_outrage = (emotions[i].moral_outrage + outrage_delta).clamp_01();

                // ── Anger from social friction ─────────────────────
                // When the agent's minimum relationship trust is low
                // (social tension), mild anger accumulates — the
                // "people are being difficult" frustration. This is
                // the natural anger that fires from social friction.
                let social_anger =
                    (Fixed::from_f64(0.4) - min_trust).max(Fixed::ZERO) * Fixed::from_f64(0.002);
                emotions[i].anger = (emotions[i].anger + social_anger).clamp_01();

                // ── Shame from social comparison ───────────────────
                // When the agent's status is below average, mild shame
                // accumulates — the "I'm not as good as others"
                // feeling. This is the natural shame that fires from
                // social comparison. Uses the agent's status vs the
                // average status of others.
                let my_status = agents[i].status_v2.effective_status();
                let avg_status: Fixed = agents
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, a)| a.status_v2.effective_status())
                    .fold(Fixed::ZERO, |acc, s| acc + s)
                    / Fixed::from_int((agents.len() - 1) as i64);
                let shame_delta =
                    (avg_status - my_status).max(Fixed::ZERO) * Fixed::from_f64(0.002);
                emotions[i].shame = (emotions[i].shame + shame_delta).clamp_01();

                // ── Humiliation from public failure ────────────────
                // When the agent has low status AND witnesses others'
                // anger (public conflict), mild humiliation accumulates
                // — the "everyone sees me failing" feeling. This is
                // the natural humiliation that fires from public shame.
                // Threshold lowered from 0.05 to 0.005 so it fires
                // in calm.
                if my_status < avg_status && max_other_anger > Fixed::from_f64(0.005) {
                    let humiliation_delta =
                        (avg_status - my_status) * max_other_anger * Fixed::from_f64(0.003);
                    emotions[i].humiliation =
                        (emotions[i].humiliation + humiliation_delta).clamp_01();
                }

                // ── Envy from status comparison ────────────────────
                // When other agents have higher status, mild envy
                // accumulates — the "they have what I want" coveting.
                // This is the natural envy that fires from social
                // comparison. Proportional to the status deficit.
                let max_other_status: Fixed = agents
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, a)| a.status_v2.effective_status())
                    .fold(Fixed::ZERO, Fixed::max);
                let envy_delta =
                    (max_other_status - my_status).max(Fixed::ZERO) * Fixed::from_f64(0.002);
                emotions[i].envy = (emotions[i].envy + envy_delta).clamp_01();

                // ── Iteration 231: Trust from positive social interactions ─
                // Trust was permanently 0.000 in calm because the only
                // producer was the appraisal system's rare positive-
                // interaction events. This ambient producer fires when
                // the agent has good relationships (high avg trust),
                // creating a "people are reliable" channel.
                let agent_avg_trust: Fixed = agents[i]
                    .relationship_v2s
                    .iter()
                    .map(|r| r.trust)
                    .fold(Fixed::ZERO, |acc, t| acc + t)
                    / Fixed::from_int(agents[i].relationship_v2s.len().max(1) as i64);
                if agent_avg_trust > Fixed::from_f64(0.3) {
                    let trust_delta =
                        (agent_avg_trust - Fixed::from_f64(0.3)) * Fixed::from_f64(0.002);
                    emotions[i].trust = (emotions[i].trust + trust_delta).clamp_01();
                }

                // ── Increased despair rate ─────────────────────────
                // Iteration 226: the old 0.002 rate was too slow for
                // calm worlds (sadness ~0.007 → despair ~0.000014/tick).
                // Increased to 0.005 so despair becomes visible within
                // 100K ticks. Also added a direct despair producer from
                // chronic stress — the "I'm worn down" channel.
                // (The sadness cascade above already contributes;
                // this is an additional direct channel.)
                let chronic_despair =
                    agents[i].embodied.endocrine.stress.chronic_load * Fixed::from_f64(0.001);
                emotions[i].despair = (emotions[i].despair + chronic_despair).clamp_01();

                // ── Iteration 229: Moral emotion ambient producers ─
                // moral.shame and moral.pride were permanently 0.000
                // because update_moral_emotions() was gated on base
                // emotions (guilt/shame/pride) that were themselves
                // near-zero in calm. These ambient producers bypass
                // that chicken-and-egg problem by feeding moral
                // cognition directly from social dynamics.
                let moral_id = agents[i].moral_cognition.moral_identity;
                // Moral shame: social comparison below average →
                // "I failed morally by being worse than others."
                let moral_shame_delta =
                    (avg_status - my_status).max(Fixed::ZERO) * moral_id * Fixed::from_f64(0.001);
                agents[i].moral_cognition.moral_emotions.shame =
                    (agents[i].moral_cognition.moral_emotions.shame + moral_shame_delta).clamp_01();
                // Moral pride: high status + good relationships →
                // "I earned this through moral behavior."
                let avg_trust: Fixed = agents[i]
                    .relationship_v2s
                    .iter()
                    .map(|r| r.trust)
                    .fold(Fixed::ZERO, |acc, t| acc + t)
                    / Fixed::from_int(agents[i].relationship_v2s.len().max(1) as i64);
                let moral_pride_delta = (my_status - avg_status).max(Fixed::ZERO)
                    * avg_trust
                    * moral_id
                    * Fixed::from_f64(0.001);
                agents[i].moral_cognition.moral_emotions.pride =
                    (agents[i].moral_cognition.moral_emotions.pride + moral_pride_delta).clamp_01();
            }

            // §8.1.4: Valence is SIGNED (-1..1). The old `.clamp_01()` floored
            // negative affect at 0, so fear/anger/sadness could never move
            // valence below zero — an agent could live through a revolution
            // while maintaining +0.8 valence. Fixed is signed, so we clamp to
            // the full range instead.
            affects[i].valence =
                (emotions[i].joy - emotions[i].sadness - emotions[i].fear - emotions[i].anger)
                    .clamp(-Fixed::ONE, Fixed::ONE);
            affects[i].arousal =
                (emotions[i].fear + emotions[i].anger + emotions[i].joy) * Fixed::from_f64(0.5);
            // §8.1.4: Apply emotion regulation AFTER appraisal.
            // Now apply_strategy() uses the fresh, appraisal-derived affect values
            // as input — the skill-scaled boost is computed from the correct target state.
            if i < reg_strategies.len() {
                // §8.1: Embodied emotions (high body tone from interoceptive
                // sensitivity) resist cognitive regulation — scale the
                // strategy's effect. Mean-zero at the default sensitivity.
                let reg_scale = agents[i]
                    .interoception
                    .regulation_scale(affects[i].valence, affects[i].arousal);
                let (reg_vd, reg_ad) = agents[i].emotion_regulation.apply_strategy(
                    reg_strategies[i],
                    affects[i].valence,
                    affects[i].arousal,
                );
                affects[i].valence =
                    (affects[i].valence + reg_vd * reg_scale).clamp(-Fixed::ONE, Fixed::ONE);
                affects[i].arousal = (affects[i].arousal + reg_ad * reg_scale).clamp_01();
            }
        }
    }
}
