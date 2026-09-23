//! Tick pass 4: action execution effects.

use super::command_goal_action;
use super::decision_census::{
    SRC_COMMAND, SRC_FEUD, SRC_HABIT, SRC_REFLEX, SRC_ROUTINE, SRC_UTILITY, SRC_VETO,
};
use super::{
    ActionKind, AgentBundle, AgentId, DecisionFactor, DecisionTrace, EntityId, Fixed, Goal,
    GoalKind, GoalSource, IdentityKind, SimEvent, Simulation, Tick,
};
use crate::actions;
use crate::institutions;

/// Meaning deficit above which the meaning reflex forces a `Worship` action
/// (i306). Sized just under saturation so the reflex is the LAST resort the
/// routine can trip, not a general worship bias: below it the utility AI and
/// the routine behave exactly as calibrated.
const REFLEX_MEANING_THRESHOLD: Fixed = Fixed::from_raw(9_000);

/// Derived-health level below which a body is treated as too compromised to
/// exert itself (i255's health-critical frame; i309 turned it from a Rest mutex
/// into an exertion veto).
///
/// i312 measured this dormant after i311 (0 below-gate agent-ticks across ~22M):
/// the derived-health distributions overlap across contexts, so no absolute
/// health threshold can separate crisis from calm.
const HEALTH_CRITICAL_THRESHOLD: Fixed = Fixed::from_raw(2_500);

/// Effective-pain level above which a body is treated as at its limits (i314).
///
/// This is the reachable crisis signal the i312 finding asked for. Pain is
/// structurally crisis-only (a wound) and sparse — `i314_pain_veto` measures
/// p90 = 0.0000 in every context. i314 ratified 0.9 (firing 0.04–0.31% of
/// agent-ticks; a 0.7 band drifted 11 pins including BOTH golden baselines
/// under the PRE-driver conflict regime).
///
/// **Re-anchored at i351 on sweep evidence (probe `i351_wound_band`, 8 seeds
/// × 20K, post-Wander-driver):** the exploration driver re-times conflict
/// exposure and the severe-wound ceiling compressed from ~0.9+ to
/// 0.7877–0.8235 across the violence-active seeds (7/8 seeds carry wounds;
/// 43/47/123 stay clean), leaving 0.9 DORMANT — 0 agent-ticks above it, a
/// §4.3 dead guard. The sweep measures firing rates 0.70 → 0.226%,
/// 0.72 → 0.178%, 0.75 → 0.083%, 0.78 → 0.027%, ≥0.85 → 0. **0.75** sits
/// just under the post-driver severe band (all violence seeds' maxima clear
/// it), fires 0.083% of agent-ticks — inside the i314-ratified band — and
/// keeps 2.7× margin above the 0.70 rate that drifted pins pre-driver.
/// Still a rare crisis guard, not a routine governor; pain clears when the
/// wound heals (i313), so non-trapping as before.
const PAIN_VETO_THRESHOLD: Fixed = Fixed::from_raw(7_500);

/// Anger level at which a feuding agent walks toward its feud target (§19.5.G,
/// the only producer of `ActionKind::Move` in the whole selection chain).
///
/// **Re-contracted at i347 on two measurements (not widened for convenience).**
/// The gate used to be `0.4`; `i346_decision_census` measured the branch firing
/// **0 times in 96 000 agent-ticks** while `feuds` were non-empty for 13.3–22.9%
/// of agent-ticks — the feuds were there and the anger was not.
///
/// (1) **The band.** Anger is an *acute* emotion with fast decay, not a slow
/// state: calm `p50 / p90 / p99 / p99.9 = 0.0000 / 0.0089 / 0.1113 / 0.3005`
/// (N=12, 3 seeds × 20K; probe `i347_feud_gate_reach`), so an absolute gate on
/// it is a tail gate by construction and `0.4` sat past p99.9. On the
/// `feuds ∧ anger > t` conjunction the probe measures (calm N=12 / N=48 /
/// collapse / drought): `0.02 → 0.226% / 0.088% / 0.032% / 0.042%` of
/// agent-ticks, `0.05 → 0.110% / 0.038% / 0.020% / 0.020%`, `0.10 → 0.044% /
/// 0.011%`, `0.40 → 0.000% in every leg`. `0.02` is ~p96 of the calm N=12
/// distribution — elevated rather than the noise floor — and is the candidate
/// that stays live in every measured context.
///
/// (2) **The ordering (the actual root cause).** With a reachable gate but the
/// branch *below* routine, the probe counted 45 candidate decisions at N=12 /
/// seed 42 and **45 of them swallowed by `follow_routine`** (and 314 of 505 at a
/// lower gate). The branch is therefore evaluated **above** the §10.3 routine
/// ladder now — see the call-site comment. Acceptance for both halves is the
/// i346 census: the `feud` source row must be non-zero and `Move` must appear.
const FEUD_APPROACH_ANGER: Fixed = Fixed::from_raw(200); // 0.02

/// Whether a body is too compromised to exert itself (i255's frame): either its
/// derived health is below [`HEALTH_CRITICAL_THRESHOLD`] (dormant safety net,
/// i312) or it is in acute pain at/above [`PAIN_VETO_THRESHOLD`] (the live,
/// reachable signal, i314). Pure and RNG-free.
#[must_use]
fn exertion_vetoed(health: Fixed, pain: Fixed) -> bool {
    health < HEALTH_CRITICAL_THRESHOLD || pain >= PAIN_VETO_THRESHOLD
}

/// Whether an action spends the body's reserves — the actions a health-critical
/// agent must not take (i309). `Work` is heavy labour (energy cost 0.05) and
/// `Wander` roams (energy cost 0.02, and it is the classified risky action).
/// Everything else — eat, drink, rest, socialize, worship, trade, idle — is
/// compatible with a body in crisis, which is why the veto is a filter and not
/// a forced action.
pub(super) fn is_exerting(action: ActionKind) -> bool {
    matches!(action, ActionKind::Work | ActionKind::Wander)
}

impl Simulation {
    pub(super) fn tick_action_pass(
        ctx: &mut crate::systems::SystemContext,
        agents: &mut [AgentBundle],
        tick_u64: u64,
        bodies: &mut [crate::person::BodyState],
        emotions: &mut [crate::person::DiscreteEmotions],
        goals: &mut [Vec<crate::person::Goal>],
        needs: &mut [crate::person::NeedState],
        personalities: &[crate::person::Personality],
        tick: Tick,
        institutions_reg: &[crate::institutions::Institution],
        norms: &crate::norms::NormRegistry,
        collective_field: &mindstrata_development::collective::CollectiveField,
        provenance_x: &mut crate::provenance::CausalProvenance,
        season: &crate::ecology::SeasonTracker,
        tick_action_starts: &mut Vec<(usize, crate::actions::ActionKind)>,
    ) {
        // WP-J (Iteration-280): governance/economic-systems stages are
        // tick-constant — resolve the slug lookup once per pass, not per
        // agent (hot path; §6 no-allocation-in-per-tick-passes).
        let (wpj_gov_stage, wpj_eco_stage) = {
            let slugs = mindstrata_development::collective::CollectiveField::line_slugs();
            let mut gov = 1.0_f64;
            let mut eco = 1.0_f64;
            for (line, s) in collective_field.lines.iter().zip(slugs) {
                match s.slug() {
                    "governance" => gov = line.stage,
                    "economic-systems" => eco = line.stage,
                    _ => {}
                }
            }
            (gov, eco)
        };
        let wpj_compliance_mult = crate::systems::institutions_multiplier::compliance_multiplier(
            wpj_gov_stage,
            wpj_eco_stage,
        );

        // i383: the population's own mean anger — the reference the shock gate's
        // ANGER arm reads (§4.10's Class-4 ruling: an absolute bar on a
        // self-driven aggregate). Hoisted out of the agent loop (§6/i336: never
        // re-fold a population aggregate per agent).
        let pop_mean_anger = crate::psychology::emotion_regulation::mean_anger(emotions);
        let anger_shock_bar =
            pop_mean_anger * crate::psychology::emotion_regulation::EMOTION_SHOCK_RATIO;

        // ── 4. Action execution (per-tick effects) ────────────────
        for i in 0..agents.len() {
            // Track causal provenance_x flags per agent per tick
            let mut was_interrupted_by_critical_needs = false;
            let mut intention_abandoned_this_tick = false;

            // §24.5: Intention commitment — check if current intention should be abandoned
            if let Some(ref intention) = agents[i].intention {
                if intention.completed {
                    // §3.2: Goal completed — move to completed list for learning
                    let completed_goal = Goal {
                        kind: intention.goal_kind,
                        priority: Fixed::from_f64(0.5),
                        commitment: Fixed::ONE,
                        created_tick: intention.formed_tick,
                        source: GoalSource::Identity, // default source for completed goals
                    };
                    agents[i].completed_goals.push(completed_goal);
                    if agents[i].completed_goals.len() > 50 {
                        agents[i].completed_goals.remove(0);
                    }
                    agents[i].intention = None;
                    intention_abandoned_this_tick = true;
                } else {
                    let stress = emotions[i].fear + emotions[i].anger;
                    // i383: the anger arm is RELATIVE to the population's own anger.
                    // The absolute `anger > 0.5` opened for 0.00–0.30% of
                    // agent-ticks over the i383 corpus (0.00% in 7 of 10 worlds) —
                    // a decoration on a predicate whose fear arm alone opened for
                    // 15.2–38.8%; the relative bar opens 3.1–19.6% in every world,
                    // so the anger channel decides again. The FEAR arm keeps its
                    // measured-discriminating bar (see EMOTION_SHOCK_RATIO).
                    let emotional_shock = emotions[i].anger > anger_shock_bar
                        || emotions[i].fear > Fixed::from_f64(0.5);
                    if intention.should_abandon(tick_u64, stress, emotional_shock) {
                        // §3.2: Goal abandoned — move to rejected list for learning
                        let rejected_goal = Goal {
                            kind: intention.goal_kind,
                            priority: Fixed::from_f64(0.5),
                            commitment: Fixed::ZERO,
                            created_tick: intention.formed_tick,
                            source: GoalSource::Identity,
                        };
                        agents[i].rejected_goals.push(rejected_goal);
                        if agents[i].rejected_goals.len() > 50 {
                            agents[i].rejected_goals.remove(0);
                        }
                        agents[i].intention = None;
                        agents[i].action_progress = 0; // force reselection
                        intention_abandoned_this_tick = true;
                    }
                }
            }

            // Action interruption: force address critical needs (> 0.9)
            if agents[i].action_progress > 0 {
                let critical = needs[i].hunger > Fixed::from_f64(0.9)
                    || needs[i].thirst > Fixed::from_f64(0.9)
                    || needs[i].fatigue > Fixed::from_f64(0.95);
                if critical {
                    // §3.2: Goal interrupted by critical needs — move to rejected list
                    if let Some(ref intention) = agents[i].intention {
                        let rejected_goal = Goal {
                            kind: intention.goal_kind,
                            priority: Fixed::from_f64(0.5),
                            commitment: Fixed::ZERO,
                            created_tick: intention.formed_tick,
                            source: GoalSource::Identity,
                        };
                        agents[i].rejected_goals.push(rejected_goal);
                        if agents[i].rejected_goals.len() > 50 {
                            agents[i].rejected_goals.remove(0);
                        }
                    }
                    agents[i].action_progress = 0;
                    agents[i].intention = None; // critical needs override intentions
                    was_interrupted_by_critical_needs = true;
                    intention_abandoned_this_tick = true;
                }
            }

            if agents[i].action_progress == 0 {
                let total_grain = ctx.world.total_food();
                let total_water = ctx.world.total_water();
                // Compute norm pressure using the registry's formula (identity-aware)
                let conformity = personalities[i].conformity;
                let avg_identity_strength = agents[i]
                    .identity
                    .strength_of(IdentityKind::Farmer)
                    .max(agents[i].identity.strength_of(IdentityKind::Parent))
                    .max(agents[i].identity.strength_of(IdentityKind::Believer));
                // §12: Institution enforcement amplifies norm pressure.
                // Council enforcement capacity multiplies the pressure felt by agents.
                let enforcement_multiplier = institutions_reg
                    .iter()
                    .filter(|i| i.kind == institutions::InstitutionKind::Council)
                    .map(|i| Fixed::ONE + i.enforcement_capacity * Fixed::from_f64(0.5))
                    .fold(Fixed::ONE, std::cmp::Ord::max);

                // §22.1: Moral values modulate norm pressure.
                // High fairness/authority → stronger norm compliance.
                // High liberty → weaker norm compliance.
                let moral_modifier = agents[i].moral_values.fairness * Fixed::from_f64(0.15)
                    + agents[i].moral_values.authority * Fixed::from_f64(0.1)
                    - agents[i].moral_values.liberty * Fixed::from_f64(0.08);
                let norm_pressure = norms
                    .norms()
                    .iter()
                    .map(|n| norms.compute_pressure(conformity, avg_identity_strength, n.id))
                    .fold(Fixed::ZERO, |a, b| a + b)
                    * enforcement_multiplier
                    - moral_modifier;

                // §10.3: Routine bias — agents follow daily schedules when stable.
                // §24: Personality modulates routine adherence — conscientiousness
                // increases it, openness decreases it.
                let (routine_action, routine_strength) = agents[i].routine.preferred_action(tick);
                let stress = emotions[i].fear + emotions[i].anger;
                let follow_routine = agents[i].routine.should_follow(
                    needs[i].hunger,
                    needs[i].thirst,
                    needs[i].fatigue,
                    stress,
                );
                // Personality-modulated routine strength: conscientiousness boosts, openness reduces
                let personality_routine_modifier = personalities[i].conscientiousness
                    * Fixed::from_f64(0.3)
                    - personalities[i].openness * Fixed::from_f64(0.2);
                let effective_routine_strength =
                    (routine_strength + personality_routine_modifier).clamp_01();

                // §22.1: When stressed, agents favor habit/routine over utility.
                // High heuristic bias → agents follow routine more readily.
                let effective_routine_threshold =
                    if agents[i].cognitive.heuristic_bias > Fixed::from_f64(0.5) {
                        Fixed::from_f64(0.3) // stressed agents follow routine at lower threshold
                    } else {
                        Fixed::from_f64(0.5) // normal agents need stronger routine signal
                    };

                // Iteration 255 (audit Phase 3 — survival-integrity
                // reflexes): the physiological override layer BENEATH the
                // utility AI. Critical deficits force the relief action
                // directly — no utility contest, no habit substitution, no
                // command override can outrank a body at its limits. This
                // makes the audit's E3 inversion (agent starving while
                // working) impossible BY CONSTRUCTION rather than by
                // calibration. Health-critical agents restrict to
                // rest/recovery instead. Deterministic, RNG-free, and
                // unreachable in calibrated calm windows (thirst tops out
                // ~0.62 there), so golden stays byte-identical.
                // Iteration 309 (audit finding i309): the health-critical case
                // used to force `Rest` — and that made a permanent trap out of
                // a chronic state. `body.health` is the DERIVED value
                // (`derived_health` = base × immune − 0.2 × stress level −
                // 0.15 × chronic load − pain − sickness − shock − …, all times
                // the skeletal factor), so any chronically stressed agent sits
                // permanently below the 0.25 gate, and Rest reduces exactly
                // none of those penalties. Measured (i309_rest_plateau_mechanism):
                // the affected agents spent 78–89% of their lives in Rest —
                // 81% for pestilence-era seed 123 agent 6 — with social and
                // meaning needs pinned at 1.0 because Rest was the ONLY legal
                // action, and thirst allowed to climb to 0.9 before the thirst
                // reflex could fire. i308 had already excluded energy, sleep,
                // habit-substitution and the emotional modifiers; the trace
                // (`[pass] agent 6 … reflex Some(Rest)`) named this branch.
                //
                // The i255 intent was "a body at its limits does not exert
                // itself", not "a body at its limits loses all agency". This
                // flag keeps that intent and drops the mutex: the agent goes on
                // choosing (drink, eat, rest, socialize, worship — every
                // non-exerting option), and only the EXERTING actions are
                // refused below.
                // Iteration 312 (post-i311 re-audit, AGENTS §2.5): the guard is
                // currently DORMANT. i311 fixed the immune-clearance pin that
                // used to hold ~3% of agents below 0.25, so `i312_health_reflex_
                // liveness` measures 0 below-gate agent-ticks across ~22M
                // (pestilence/collapse/drought/calm, 4320–50K; ever-crossed 0).
                // The derived-health floor is now ≈0.40 and decomposes as
                // `base×immune 0.74 − stress 0.16 − chronic 0.135 − sickness 0.05`
                // with `pain` and `shock` contributing EXACTLY 0.0000 — two of
                // the five penalty channels are dead, and the floor is set by
                // stress+chronic saturating at a fixed ~0.29 total. No absolute
                // threshold can restore liveness: the per-scenario distributions
                // overlap (p1 0.47–0.62 in every context). Kept as a correct
                // safety veto; restoring a reachable crisis band is queued as its
                // own behavioural iteration (needs live pain/shock channels), not
                // a threshold re-pin (AGENTS §4.5).
                // i314: the veto fires on EITHER a compromised derived health
                // (dormant since i311/312, kept as a safety net) OR severe pain
                // — the live, reachable crisis signal. A body in acute pain at
                // its limits does not exert itself; the wound heals (i313) so
                // this never traps the way the i255 Rest mutex did.
                let pain = agents[i].embodied.nervous.pain.effective_pain();
                let health_critical = exertion_vetoed(agents[i].body.health, pain);
                let reflex_override = if needs[i].thirst > Fixed::from_f64(0.9)
                    && needs[i].thirst >= needs[i].hunger
                {
                    Some(ActionKind::Drink)
                } else if needs[i].hunger > Fixed::from_f64(0.9) {
                    Some(ActionKind::Eat)
                } else if needs[i].fatigue > Fixed::from_f64(0.95) {
                    Some(ActionKind::Rest)
                } else if needs[i].meaning > REFLEX_MEANING_THRESHOLD {
                    // Iteration 306 (audit finding i306): the MEANING reflex.
                    //
                    // The routine override below the reflex layer assumes the
                    // daily schedule covers every need channel, but the routine
                    // template has no worship slot — so an agent whose meaning
                    // need saturates is locked into Work/Rest while its Worship
                    // goal sits at priority 1.0 and never fires. Measured
                    // (i306_meaning_channel, 20K x 12 seeds): agents at meaning
                    // 1.0000 with a Worship goal present 70–85% of ticks
                    // performed ZERO Worship ticks, and 27% of all agent-ticks
                    // sat at the meaning ceiling — the channel was dead for the
                    // very agents who needed it most.
                    //
                    // The reflex layer exists to make "an agent at its limits
                    // acts, whatever the schedule says" true by construction
                    // (Iteration 255, audit Phase 3); a need pinned at its
                    // ceiling is at its limit. Sits BELOW every physiological
                    // reflex (body still outranks soul) and ABOVE routine and
                    // utility. Deterministic, RNG-free. The threshold keeps
                    // calibrated short windows untouched by construction:
                    // meaning tops out ~0.30 at 2K ticks, so golden stays
                    // byte-identical.
                    Some(ActionKind::Worship)
                } else {
                    None
                };
                let (mut action, mut action_source) = if let Some(forced) = reflex_override {
                    (forced, SRC_REFLEX)
                } else if let Some((cmd_action, cmd_kind)) =
                    command_goal_action(&goals[i], &needs[i])
                {
                    // §5 (Iteration 155): an external directive takes
                    // priority over routine and internal drives, and is
                    // consumed the moment it steers a selection — a
                    // one-shot nudge that returns the agent to
                    // autonomy. Gated on `GoalSource::Command` goals —
                    // which only exist after `command_agent` is called
                    // (never in a calibrated window), so this branch is
                    // a structural no-op everywhere calibrated and draws
                    // zero RNG.
                    goals[i].retain(|g| !(g.source == GoalSource::Command && g.kind == cmd_kind));
                    (cmd_action, SRC_COMMAND)
                } else if !agents[i].feuds.is_empty()
                    && emotions[i].anger > FEUD_APPROACH_ANGER
                    && needs[i].hunger < Fixed::from_f64(0.85)
                    && needs[i].thirst < Fixed::from_f64(0.85)
                {
                    // §19.5.G: Angry feuding agents Move toward their feud target,
                    // NOT when critical needs demand attention.
                    //
                    // i347: this branch sits ABOVE the daily routine, and that
                    // ordering is the fix, not an accident of layout. Placed
                    // below routine it was shadowed: with a reachable anger
                    // gate (0.02) the i347 probe measured **45 of 45**
                    // candidate decisions swallowed by `follow_routine` — an
                    // angry agent mid-schedule never approached, which is the
                    // opposite of what the clause above says. Routine is the
                    // schedule for *ordinary* business; a live feud is not
                    // ordinary business, and the branch's own needs guard
                    // (hunger/thirst < 0.85) is exactly the "NOT when critical
                    // needs demand attention" carve-out. External commands
                    // still outrank it (§5), and the physiological reflexes
                    // still outrank everything (i255).
                    let feud_target = agents[i].feuds[0];
                    (
                        ActionKind::Move {
                            target_x: agents[feud_target].position.x,
                            target_y: agents[feud_target].position.y,
                        },
                        SRC_FEUD,
                    )
                } else if follow_routine && effective_routine_strength > effective_routine_threshold
                {
                    // §10.3: Routine creates behavioral stability — prefer scheduled action
                    (routine_action, SRC_ROUTINE)
                } else {
                    // §12.3: Institution collective morale modulates norm compliance.
                    // Members of institutions_reg with high morale are more norm-compliant.
                    // Note: norm_pressure is negative = compliant, positive = violating.
                    // WP-J (Iteration-280): the morale→compliance transmission
                    // depth scales with the village's governance/economic-
                    // systems collective stage (band-III gated, identity at
                    // all pinned horizons — see systems/institutions_multiplier).
                    let wpj_mult = wpj_compliance_mult;
                    // WP-J (i280): channel #2 — the live read-side
                    // surface. Σ member_work_bonus over the agent's
                    // institutions (zero below the band-III gate, dread-
                    // class nudge above). Channel #1 (morale→compliance
                    // below) is retained: its surface is provably dead
                    // at N=12 (i280 violations sweep) but is the honest
                    // §12.3 semantics and revives at violation-positive
                    // regimes.
                    let mut institution_work_bonus = Fixed::ZERO;
                    let mut adjusted_pressure = norm_pressure;
                    for inst in institutions_reg {
                        if inst.has_member(AgentId::new(i as u64)) {
                            // High morale → more compliant (pressure becomes more negative);
                            // WP-J: transmission scaled by the governance band.
                            let morale_bonus =
                                inst.collective.morale * Fixed::from_f64(0.1) * wpj_mult;
                            adjusted_pressure -= morale_bonus;
                            institution_work_bonus +=
                                crate::systems::institutions_multiplier::member_work_bonus(
                                    inst.collective.morale,
                                    wpj_mult,
                                );
                        }
                    }
                    (
                        actions::select_action(
                            &actions::DecisionContext {
                                needs: &needs[i],
                                personality: &personalities[i],
                                active_goals: &goals[i],
                                identity: &agents[i].identity,
                                decision_policy: &agents[i].decision_policy,
                                total_grain,
                                total_water,
                                coin: agents[i].wealth.coin,
                                norm_pressure: adjusted_pressure,
                                anger: emotions[i].anger,
                                fear: emotions[i].fear,
                                joy: emotions[i].joy,
                                sadness: emotions[i].sadness,
                                stress,
                                fairness: agents[i].moral_values.fairness,
                                authority: agents[i].moral_values.authority,
                                care: agents[i].moral_values.care,
                                loyalty: agents[i].moral_values.loyalty,
                                // §9.2 (Iteration 94): the agent's learned
                                // RL valuation weights feed selection.
                                action_values: agents[i].neural_like.values,
                                // §8.1.5 (Iteration 96): the full-pressure
                                // motivation argmax biases selection toward
                                // actions that relieve the dominant need.
                                dominant_need: agents[i].motivation.dominant_need,
                                dominant_pressure: agents[i].motivation.dominant_pressure(),
                                // i351 (A8 closure): the exploration driver
                                // inputs — the motivation layer's novelty
                                // pressure (full formula, the same number
                                // `update_dominant` compares) and the need-
                                // quietude gate. Zero-at-identity: the driver
                                // term is 0 unless the gate opens AND novelty
                                // pressure is live.
                                novelty_pressure: agents[i].motivation.pressure_full(
                                    crate::psychology::motivation::MotiveCategory::Novelty,
                                ),
                                // i356 (Idle revival): the recreation driver's
                                // input — the same full-formula pressure
                                // `update_dominant` compares for `Play`.
                                // Instrumentation-only until the driver term
                                // lands (probe i356 sizes it first).
                                play_pressure: agents[i].motivation.pressure_full(
                                    crate::psychology::motivation::MotiveCategory::Play,
                                ),
                                needs_quiet: needs[i].hunger < actions::WANDER_QUIETUDE_GATE
                                    && needs[i].thirst < actions::WANDER_QUIETUDE_GATE
                                    && needs[i].fatigue < actions::WANDER_QUIETUDE_GATE,
                                // §8.1.16 (Iteration 103): the scenario-
                                // grounded dread (regrounded on the daily
                                // phase, before selection in the same tick)
                                // drives precautionary provisioning — Work/
                                // Trade up, Rest down. Zero-at-zero: agents
                                // whose tiers do not run prospection hold
                                // dread 0 → legacy utility.
                                dread: agents[i].prospection.dread,
                                // §8.1.16 (Iteration 203): the scenario-
                                // grounded hope drives aspirational
                                // engagement — Socialize/Worship up, Idle
                                // down (the positive mirror of dread).
                                hope: agents[i].prospection.hope,
                                // §8.1.12 (Iteration 204): the blended
                                // emotion + executive-function planning
                                // confidence drives deferred-gratification
                                // calibration — Work up / Idle down when
                                // confident (baseline-corrected at 0.5, so
                                // default populations stay byte-identical).
                                planning_confidence: agents[i].prospection.planning_confidence,
                                // Iteration 232: mood drift — valence
                                // from affect module ([-1,1]) drives
                                // social/exploration vs withdrawal.
                                mood_valence: agents[i].affect.valence,
                                // Iteration 233: seasonal behavioral modulation
                                season: season.current as u8,
                                // Iteration 236: age-related behavioral modulation
                                life_stage: agents[i].embodied.development.life_stage as u8,
                                // Iteration 248 (Arc B): sleep-debt social
                                // withdrawal — zero below the deprivation
                                // threshold (0.5), scaling above; rested
                                // agents are byte-identical.
                                social_withdrawal: {
                                    let debt = agents[i].embodied.circadian.sleep_debt;
                                    if debt > Fixed::from_f64(0.5) {
                                        debt - Fixed::from_f64(0.5)
                                    } else {
                                        Fixed::ZERO
                                    }
                                },
                                // Iteration 247 (Arc B — interoception): the
                                // somatic marker — how much worse than a
                                // default interoceptor the agent feels (fatigue
                                // + pain). Biases risky actions down; exactly
                                // zero for default configurations, so calm
                                // worlds stay byte-identical.
                                somatic_marker: agents[i]
                                    .interoception
                                    .somatic_risk_bias(needs[i].fatigue, agents[i].embodied.injury),
                                // AP3 DC-1 (task 3.4): development gating —
                                // fulfillment thresholds via needs bands.
                                // Zero at neutral (task 3.1 newborn/3.2 virgin)
                                // so goldens stay byte-identical until field moves.
                                development: &agents[i].development,
                                // DC-2 Era III lite: polarity_claims list
                                // (read-only; the action selector applies the
                                // i282 safe-coefficient 0.01 bias to social
                                // actions based on ActiveTension count).
                                polarity_claims: &agents[i].polarity_claims,
                                // WP-J (i280): institution-membership work
                                // bonus (zero below the band-III gate).
                                institution_work_bonus,
                                // i281: anchors the polarity-claim salience
                                // window (1000-tick recency integral).
                                current_tick: tick_u64,
                            },
                            ctx.rng,
                        ),
                        SRC_UTILITY,
                    )
                };
                // §8.1.19 (P3-1, August 14, 2026): habitual fallback
                // under stress — an agent whose automaticity is high
                // (habits formed through ~100 practice ticks at the
                // 0.1-proficiency milestone) and who is stressed
                // substitutes its strongest habit for the deliberated
                // action ("under stress, agents fall back on habits").
                // Zero-blast in calibrated windows: automaticity stays
                // < 0.5 until habits actually form (base strength 0.3 →
                // ≈0.29 at golden stress levels), so the golden/snapshot
                // horizons are untouched and only long-horizon runs where
                // practice has accumulated see the fallback. Deterministic,
                // no RNG.
                //
                // Iteration 307 (audit finding i307): this block used to
                // overwrite the action UNCONDITIONALLY — including an action
                // the survival-integrity reflex layer had just forced. That
                // violates i255's own contract above verbatim ("no utility
                // contest, NO HABIT SUBSTITUTION, no command override can
                // outrank a body at its limits"), and it was not theoretical:
                // the substituted habit set is {Work, Trade, Socialize,
                // Worship, Eat} — it cannot produce `Drink` or `Rest` at all,
                // so a chronically stressed, habituated agent was trapped
                // in Trade/Work while its thirst and fatigue sat pinned at
                // 1.0 for tens of thousands of ticks (measured:
                // i307_physio_saturation, seed 99 agent 2 — thirst above the
                // reflex for 49 212/50 000 ticks, 4 Drink ticks in the whole
                // run, action census Trade 0.997). The physiological reflex
                // layer is therefore gated off from habit substitution, the
                // same way it is gated off from the routine path.
                if reflex_override.is_none()
                    && stress > Fixed::from_f64(0.5)
                    && agents[i].psych_skills.automaticity > Fixed::from_f64(0.5)
                {
                    let trigger = match agents[i].motivation.dominant_need {
                        crate::psychology::motivation::MotiveCategory::Hunger => "hunger",
                        crate::psychology::motivation::MotiveCategory::Thirst => "thirst",
                        crate::psychology::motivation::MotiveCategory::Sleep => "fatigue",
                        crate::psychology::motivation::MotiveCategory::Belonging
                        | crate::psychology::motivation::MotiveCategory::Attachment
                        | crate::psychology::motivation::MotiveCategory::Romance => "social",
                        crate::psychology::motivation::MotiveCategory::Meaning => "meaning",
                        _ => "",
                    };
                    if let Some(habit) = agents[i].psych_skills.execute_habit(trigger, tick_u64) {
                        let substituted = match habit.as_str() {
                            "Work" => ActionKind::Work,
                            "Trade" => ActionKind::Trade,
                            "Socialize" => ActionKind::Socialize,
                            "Worship" => ActionKind::Worship,
                            "Eat" => ActionKind::Eat,
                            _ => action,
                        };
                        // i346 census: only a habit that actually changed the
                        // action counts as the deciding source.
                        if substituted != action {
                            action_source = SRC_HABIT;
                        }
                        action = substituted;
                    }
                }
                // Iteration 309: a body at its limits does not exert itself.
                // The health-critical frame vetoes the exerting actions and
                // leaves every restorative/expressive option available, instead
                // of the old Rest mutex (see the `health_critical` comment
                // above). Applied AFTER the habit fallback so a stress habit
                // cannot smuggle `Work` past the veto either.
                if health_critical && is_exerting(action) {
                    action = ActionKind::Rest;
                    action_source = SRC_VETO;
                }
                agents[i].current_action = action;
                // i346: record the finalized (source, action) pair. Off by
                // default — one relaxed atomic load per agent-tick when the
                // census has never been enabled.
                super::decision_census::record(action_source, action);
                agents[i].action_progress = action.definition().duration_ticks;
                tick_action_starts.push((i, action));

                let agent_id = AgentId::new(i as u64);

                // §34: Record decision trace for causal provenance_x
                {
                    let mut factors = Vec::new();
                    factors.push(DecisionFactor {
                        kind: "need_hunger".into(),
                        magnitude: needs[i].hunger,
                        description: format!("Hunger: {:.2}", needs[i].hunger.to_f64()),
                    });
                    factors.push(DecisionFactor {
                        kind: "need_thirst".into(),
                        magnitude: needs[i].thirst,
                        description: format!("Thirst: {:.2}", needs[i].thirst.to_f64()),
                    });
                    factors.push(DecisionFactor {
                        kind: "need_fatigue".into(),
                        magnitude: needs[i].fatigue,
                        description: format!("Fatigue: {:.2}", needs[i].fatigue.to_f64()),
                    });
                    factors.push(DecisionFactor {
                        kind: "norm_pressure".into(),
                        magnitude: norm_pressure,
                        description: format!("Norm pressure: {:.2}", norm_pressure.to_f64()),
                    });
                    if follow_routine && effective_routine_strength > Fixed::from_f64(0.5) {
                        factors.push(DecisionFactor {
                            kind: "routine".into(),
                            magnitude: effective_routine_strength,
                            description: format!(
                                "Routine strength: {:.2}",
                                effective_routine_strength.to_f64()
                            ),
                        });
                    }
                    provenance_x.record_decision(DecisionTrace {
                        agent: agent_id,
                        tick: tick_u64,
                        action_name: format!("{action:?}"),
                        factors,
                        from_routine: follow_routine
                            && effective_routine_strength > Fixed::from_f64(0.5),
                        interrupted_by_critical_needs: was_interrupted_by_critical_needs,
                        intention_abandoned: intention_abandoned_this_tick,
                    });
                }

                // §24.5: Create intention for the selected action's goal
                let goal_kind = match action {
                    ActionKind::Eat => GoalKind::Eat,
                    ActionKind::Drink => GoalKind::Drink,
                    ActionKind::Rest => GoalKind::Rest,
                    ActionKind::Work => GoalKind::Work,
                    ActionKind::Socialize => GoalKind::Socialize,
                    ActionKind::Worship => GoalKind::Worship,
                    _ => GoalKind::Eat, // fallback
                };
                agents[i].intention = Some(crate::person::Intention::new(
                    goal_kind,
                    tick_u64,
                    personalities[i].conscientiousness,
                ));

                // Push events (no resource ops yet — those happen after ctx drops)
                match action {
                    ActionKind::Eat => {
                        ctx.events.push(SimEvent::AgentAte {
                            agent: agent_id,
                            food: EntityId::new(0),
                            tick,
                        });
                    }
                    ActionKind::Drink => {
                        ctx.events.push(SimEvent::AgentDrank {
                            agent: agent_id,
                            source: EntityId::new(1),
                            tick,
                        });
                    }
                    ActionKind::Rest => {
                        ctx.events.push(SimEvent::AgentRested {
                            agent: agent_id,
                            tick,
                        });
                    }
                    _ => {}
                }
            }

            let action = agents[i].current_action;
            actions::apply_action_tick(action, &mut bodies[i], &mut needs[i]);

            agents[i].action_progress = agents[i].action_progress.saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod exertion_veto_tests {
    use super::*;

    #[test]
    fn exertion_veto_fires_on_severe_pain() {
        // i314: pain is the reachable crisis signal (i312 found the health
        // clause dormant). A body in acute pain does not exert itself.
        // i351 re-anchor (0.9 → 0.75, probe `i351_wound_band`): the test
        // straddles the threshold on both sides — just-below and just-above
        // — so the boundary move is exercised, not assumed.
        assert!(!exertion_vetoed(Fixed::ONE, Fixed::ZERO));
        assert!(!exertion_vetoed(Fixed::ONE, Fixed::from_f64(0.74)));
        assert!(exertion_vetoed(Fixed::ONE, Fixed::from_raw(7_500)));
        assert!(exertion_vetoed(Fixed::ONE, Fixed::ONE));
    }

    #[test]
    fn exertion_veto_keeps_the_health_safety_net() {
        // The i255 clause is dormant (i312) but retained: a body whose derived
        // health collapses below the gate must still be refused exertion.
        assert!(exertion_vetoed(Fixed::from_f64(0.24), Fixed::ZERO));
        assert!(!exertion_vetoed(Fixed::from_f64(0.25), Fixed::ZERO));
    }
}
