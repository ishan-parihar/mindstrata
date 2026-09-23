//! i346: opt-in census of the action-selection layer.
//!
//! ## Why this exists
//!
//! i338 measured `ActionKind::Wander` and `ActionKind::Move` at **0.00%** of
//! agent-ticks and recorded the whole §6 locomotion subsystem as inert (A8),
//! but the *action histogram* alone cannot say **why** an action is
//! unreachable: `pass_action` selects through a five-deep branch chain
//! (survival-integrity reflex → external command → daily routine → feud
//! approach → utility AI) and then applies two post-transforms (the §8.1.19
//! stress-habit substitution and the i309/i314 exertion veto). An action can
//! be dead because the branch that owns it never runs, or because it runs and
//! loses. Those two failures have completely different fixes, and a census is
//! the only way to tell them apart.
//!
//! ## What a record is (and is not)
//!
//! One record is one **selection decision**, not one agent-tick: `pass_action`
//! only re-decides when the previous action finished (`action_progress == 0`),
//! so an agent holding an 8-tick `Work` walk appears once every 8 ticks. The
//! shares below are therefore shares of *decisions* — i346 measured 42 813
//! decisions at N=12 over 20 000 ticks (17.8% of agent-ticks, i.e. a mean
//! action duration of 5.6 ticks). Duty-cycle shares (what the agent is doing at
//! a random tick) are the `i338`/`i341` measurement, and the two answer
//! different questions: this one answers "what decided", that one "what ran".
//!
//! ## Shape
//!
//! A process-global sink, mirroring the i330/i335 pass profiler
//! ([`crate::sim::Simulation::pass_profile_totals`]): a diagnostic, never
//! serialized, never read by a pass, and therefore structurally incapable of
//! changing behaviour. It is **off by default** — every entry point checks a
//! single relaxed atomic load first — so a normal run pays one predictable
//! branch per agent-tick and records nothing.
//!
//! Enable it with [`enable`] (or `MINDSTRATA_DECISION_CENSUS=1`), read it with
//! [`report`], clear it with [`reset`].
//!
//! The utility leg additionally records how far the loser `Wander` sat from
//! the winning candidate in realized utility units. That number is what turns
//! "Wander never wins" into a *sizing*: the per-candidate jitter is ±0.05, so a
//! gap above 0.05 means no draw could have flipped the decision (the wall is
//! structural), while a gap inside the jitter band means Wander is a coin-flip
//! away (the wall is calibration).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::actions::ActionKind;
use crate::psychology::MotiveCategory;

/// i387: number of motivation categories tracked in the dominance histogram.
pub const MOTIVE_COUNT: usize = 19;

/// i387: names, indexed by [`motive_index`].
pub const MOTIVE_NAMES: [&str; MOTIVE_COUNT] = [
    "Hunger",
    "Thirst",
    "Sleep",
    "Warmth",
    "Health",
    "Safety",
    "Attachment",
    "Belonging",
    "Esteem",
    "Autonomy",
    "Competence",
    "Meaning",
    "Certainty",
    "Novelty",
    "Play",
    "Care",
    "Romance",
    "Justice",
    "Recognition",
];

/// i387: the honest index of a motivation category (the histogram's slot).
#[must_use]
pub fn motive_index(m: MotiveCategory) -> usize {
    match m {
        MotiveCategory::Hunger => 0,
        MotiveCategory::Thirst => 1,
        MotiveCategory::Sleep => 2,
        MotiveCategory::Warmth => 3,
        MotiveCategory::Health => 4,
        MotiveCategory::Safety => 5,
        MotiveCategory::Attachment => 6,
        MotiveCategory::Belonging => 7,
        MotiveCategory::Esteem => 8,
        MotiveCategory::Autonomy => 9,
        MotiveCategory::Competence => 10,
        MotiveCategory::Meaning => 11,
        MotiveCategory::Certainty => 12,
        MotiveCategory::Novelty => 13,
        MotiveCategory::Play => 14,
        MotiveCategory::Care => 15,
        MotiveCategory::Romance => 16,
        MotiveCategory::Justice => 17,
        MotiveCategory::Recognition => 18,
    }
}

/// Number of selection sources tracked (see the `SRC_*` constants).
pub const SOURCE_COUNT: usize = 7;
/// Number of action kinds tracked (the full `select_action` candidate set plus
/// `Move`, which no utility candidate can produce).
pub const ACTION_COUNT: usize = 10;

/// Human-readable names, indexed by the `SRC_*` constants.
pub const SOURCE_NAMES: [&str; SOURCE_COUNT] = [
    "reflex", "command", "routine", "feud", "utility", "habit", "veto",
];
/// Human-readable names, indexed by [`action_index`].
pub const ACTION_NAMES: [&str; ACTION_COUNT] = [
    "Eat",
    "Drink",
    "Rest",
    "Work",
    "Socialize",
    "Worship",
    "Trade",
    "Wander",
    "Move",
    "Idle",
];

/// The §255 survival-integrity reflex layer forced the action.
pub const SRC_REFLEX: usize = 0;
/// An external `GoalSource::Command` directive steered the selection.
pub const SRC_COMMAND: usize = 1;
/// The §10.3 daily routine ladder won.
pub const SRC_ROUTINE: usize = 2;
/// The §19.5.G feud-approach branch produced a `Move`.
pub const SRC_FEUD: usize = 3;
/// The utility AI (`actions::select_action`) ran and its argmax was used.
pub const SRC_UTILITY: usize = 4;
/// The §8.1.19 stress-habit substitution replaced the selected action.
pub const SRC_HABIT: usize = 5;
/// The i309/i314 exertion veto replaced the selected action with `Rest`.
pub const SRC_VETO: usize = 6;

/// The utility gap below which the per-candidate jitter (±0.05) could have
/// flipped the decision on its own.
pub const NOISE_AMPLITUDE: f64 = 0.05;

/// i387: number of utility term buckets a decomposition records.
pub const UTILITY_TERM_COUNT: usize = 14;

/// i387: human-readable bucket names, indexed by the `UT_*` constants.
///
/// The buckets are the *decision-relevant families* of the utility formula,
/// not one entry per line of arithmetic: i380 proved `Socialize` loses the
/// argmax and refuted two single-knob repairs, so the question this ledger
/// answers is **which family of terms decides the winner** — and which family
/// a given candidate is missing entirely.
pub const UTILITY_TERM_NAMES: [&str; UTILITY_TERM_COUNT] = [
    "need",
    "urgency",
    "prospective",
    "identity",
    "norm",
    "trade",
    "cost",
    "learned",
    "explore",
    "driver",
    "goal",
    "policy",
    "affectx",
    "context",
];

/// i387: need-relief block (hunger/thirst/fatigue/social/meaning).
pub const UT_NEED: usize = 0;
/// i387: §8.1.5 dominant-need urgency boost.
pub const UT_URGENCY: usize = 1;
/// i387: §8.1.16/§8.1.12 dread + hope + planning-confidence nudges.
pub const UT_PROSPECTIVE: usize = 2;
/// i387: identity-congruence affinity.
pub const UT_IDENTITY: usize = 3;
/// i387: normative component (pressure × conformity).
pub const UT_NORM: usize = 4;
/// i387: §13.3 trade coin bonus (affordability-gated).
pub const UT_TRADE: usize = 5;
/// i387: energy cost (a penalty).
pub const UT_COST: usize = 6;
/// i387: §9.2 RL action values.
pub const UT_LEARNED: usize = 7;
/// i387: §8.1.6 jitter + approach/withdrawal exploration bonus.
pub const UT_EXPLORE: usize = 8;
/// i387: i351/i356 revival drivers (Wander novelty, Idle recreation).
pub const UT_DRIVER: usize = 9;
/// i387: active-goal alignment bonus.
pub const UT_GOAL: usize = 10;
/// i387: §8.1.20 DecisionPolicy modifiers (emotional, moral, habit).
pub const UT_POLICY: usize = 11;
/// i387: affective/state modulations (ActiveTension social bias, somatic
/// penalty, sleep-debt withdrawal, mood nudge).
pub const UT_AFFECTX: usize = 12;
/// i387: situational nudges (season, age, institution work bonus,
/// development/pathology).
pub const UT_CONTEXT: usize = 13;

/// Map an action kind to its census slot.
#[must_use]
pub fn action_index(kind: ActionKind) -> usize {
    match kind {
        ActionKind::Eat => 0,
        ActionKind::Drink => 1,
        ActionKind::Rest => 2,
        ActionKind::Work => 3,
        ActionKind::Socialize => 4,
        ActionKind::Worship => 5,
        ActionKind::Trade => 6,
        ActionKind::Wander => 7,
        ActionKind::Move { .. } => 8,
        ActionKind::Idle => 9,
    }
}

/// How far one watched candidate sits from the winner, across the utility
/// arbitrations sampled.
#[derive(Debug, Default, Clone, Copy)]
pub struct GapStats {
    /// Samples in which this candidate matched the winning utility (won — the
    /// gap is exactly zero for an argmax — or tied it and lost the strict
    /// comparison).
    pub wins: u64,
    /// Losses by no more than [`NOISE_AMPLITUDE`] — a jitter-sized distance
    /// from winning, i.e. reachable by calibration rather than by design.
    pub within_noise: u64,
    /// Sum of the realized `winner − candidate` gaps over the losses.
    pub sum: f64,
    /// Largest realized gap observed.
    pub max: f64,
}

impl GapStats {
    /// Mean realized gap over the losses (0 when the candidate never lost).
    #[must_use]
    pub fn mean_loss(&self, samples: u64) -> f64 {
        let losses = samples.saturating_sub(self.wins);
        if losses == 0 {
            0.0
        } else {
            self.sum / losses as f64
        }
    }

    fn observe(&mut self, won: bool, gap: f64) {
        if won {
            self.wins += 1;
            return;
        }
        if gap <= NOISE_AMPLITUDE {
            self.within_noise += 1;
        }
        self.sum += gap;
        if gap > self.max {
            self.max = gap;
        }
    }
}

#[derive(Debug, Default)]
struct Census {
    sources: [u64; SOURCE_COUNT],
    actions: [u64; ACTION_COUNT],
    /// `cross[source][action]` — which action each source actually produced.
    cross: [[u64; ACTION_COUNT]; SOURCE_COUNT],
    /// Utility-selection samples (one per `select_action` call under census).
    utility_samples: u64,
    /// Distance from the winner for the two candidates that carry no
    /// need-relief term of their own: `Wander` (A8) and `Idle`.
    wander: GapStats,
    idle: GapStats,
    // i387: per-bucket utility sums for the winner, the `Socialize`
    // candidate, and the runner-up (the best non-winner). Sums, not samples,
    // so the ledger is O(1) in memory while still naming the family that
    // decides the arbitration.
    terms_samples: u64,
    /// i387: which motivation category dominated at each arbitration — the
    /// drive the utility leg was asked to serve. A category that never appears
    /// here cannot be "missing a relief channel"; one that appears often with
    /// no mapped action is a dead producer.
    dominant_counts: [u64; MOTIVE_COUNT],
    /// i387: sum/max of the dominant pressure, so a dominance share can be
    /// weighted by how hard the drive was actually pressing.
    dominant_pressure_sum: f64,
    dominant_pressure_max: f64,
    terms_winner: [f64; UTILITY_TERM_COUNT],
    terms_socialize: [f64; UTILITY_TERM_COUNT],
    terms_runner: [f64; UTILITY_TERM_COUNT],
    socialize_gap: GapStats,
    // i351 quiet-window split: the A8 exploration driver only competes when
    // the need-quietude gate is open, so the wall it must clear is the
    // quiet-window winner — not the all-decisions winner (whose mass rides
    // need pressure the gate excludes).
    quiet_samples: u64,
    quiet_wander: GapStats,
    quiet_winner_sum: f64,
    quiet_winner_max: f64,
    quiet_winner_actions: [u64; ACTION_COUNT],
    /// i351 coefficient sweep: per quiet sample, the PRE-driver
    /// winner−Wander gap and the novelty pressure that would multiply the
    /// coefficient. Capped — instrumentation only, never read in production.
    quiet_pairs: Vec<(f64, f64)>,
    /// i356: the same pair shape for the `Idle` candidate — PRE-driver
    /// `winner − Idle` gap against the player's `Play` pressure, so the
    /// recreation driver's coefficient can be swept offline over realized
    /// samples before it is sized (§4.2). Instrumentation only.
    quiet_idle_pairs: Vec<(f64, f64)>,
}

fn sink() -> &'static Mutex<Census> {
    static SINK: OnceLock<Mutex<Census>> = OnceLock::new();
    SINK.get_or_init(|| Mutex::new(Census::default()))
}

static ENABLED: AtomicBool = AtomicBool::new(false);

/// Turn the census on for the rest of the process (or until [`disable`]).
///
/// Also enabled by `MINDSTRATA_DECISION_CENSUS=1`; the environment is checked
/// once by [`enabled`], so an operator can instrument a binary without a
/// rebuild.
pub fn enable() {
    ENABLED.store(true, Ordering::Relaxed);
}

/// Turn the census off. Purely diagnostic: no recorded number can influence a
/// simulation, so this is safe to call between runs in one process.
pub fn disable() {
    ENABLED.store(false, Ordering::Relaxed);
}

/// Whether `MINDSTRATA_DECISION_CENSUS` was set when the process first asked
/// (read once, cached).
///
/// The environment is an **operator override**: it forces the census on and
/// [`disable`] cannot turn it off, so an instrumented binary keeps its census
/// no matter what the code does.
fn environment_override() -> bool {
    use std::sync::OnceLock;
    static ENV: OnceLock<bool> = OnceLock::new();
    *ENV.get_or_init(|| {
        std::env::var("MINDSTRATA_DECISION_CENSUS")
            .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    })
}

/// Whether the census is currently recording. Cheap enough to call per
/// agent-tick — the `ENABLED` load is a relaxed atomic and the environment
/// latch is a cached bool — which is how the call sites are gated.
#[must_use]
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed) || environment_override()
}

/// Clear every counter (so one probe can measure several populations in one
/// process).
///
/// The sink is process-global, so a caller that resets while another simulation
/// is recording will lose that simulation's counts. Probes are single-threaded
/// and are the intended users; the parallel test harness must therefore keep its
/// census assertions in **one** test case rather than one per property.
pub fn reset() {
    if let Ok(mut sink) = sink().lock() {
        *sink = Census::default();
    }
}

/// Record one finalized action and the source that produced it.
///
/// `source` is the **last** writer: a habit substitution or an exertion veto
/// overwrites the earlier branch, because the census answers "what did the
/// village actually do, and what decided it".
pub fn record(source: usize, action: ActionKind) {
    if !enabled() {
        return;
    }
    // (a selection decision, not an agent-tick — see the module docs)
    let action = action_index(action);
    if source >= SOURCE_COUNT {
        return;
    }
    if let Ok(mut sink) = sink().lock() {
        sink.sources[source] += 1;
        sink.actions[action] += 1;
        sink.cross[source][action] += 1;
    }
}

/// Record one utility-AI arbitration: the realized gap between the winner and
/// each watched candidate.
///
/// Called from [`crate::actions::select_action`] with values it has already
/// computed, so the call draws no RNG and the stream position is untouched.
pub fn record_utility_sample(
    best_minus_wander_pre: f64,
    best_minus_wander: f64,
    best_minus_idle: f64,
    best_minus_idle_pre: f64,
    needs_quiet: bool,
    winner_utility: f64,
    winner_action: usize,
    novelty_pressure: f64,
    play_pressure: f64,
) {
    if !enabled() {
        return;
    }
    if let Ok(mut sink) = sink().lock() {
        sink.utility_samples += 1;
        // A zero gap that is not a win means the candidate tied and lost the
        // strict-inequality comparison; counted as a loss at gap 0, which is
        // the conservative reading (it did not win).
        sink.wander
            .observe(best_minus_wander <= 0.0, best_minus_wander);
        sink.idle.observe(best_minus_idle <= 0.0, best_minus_idle);
        if needs_quiet {
            sink.quiet_samples += 1;
            sink.quiet_wander
                .observe(best_minus_wander <= 0.0, best_minus_wander);
            sink.quiet_winner_sum += winner_utility;
            if winner_utility > sink.quiet_winner_max {
                sink.quiet_winner_max = winner_utility;
            }
            if winner_action < ACTION_COUNT {
                sink.quiet_winner_actions[winner_action] += 1;
            }
            if sink.quiet_pairs.len() < QUIET_PAIR_CAP {
                sink.quiet_pairs
                    .push((best_minus_wander_pre, novelty_pressure));
            }
            if sink.quiet_idle_pairs.len() < QUIET_PAIR_CAP {
                // i356: the PRE-driver `winner − Idle` gap — the quantity the
                // recreation coefficient must clear (the realized gap above
                // already contains the driver term once it lands).
                sink.quiet_idle_pairs
                    .push((best_minus_idle_pre, play_pressure));
            }
        }
    }
}

/// i387: record one arbitration's per-bucket decomposition.
///
/// `winner`, `socialize` and `runner_up` are the bucket vectors of the argmax,
/// the `Socialize` candidate, and the best non-winning candidate respectively;
/// `socialize_gap` is `winner_total − socialize_total`. Read-only bookkeeping:
/// the buckets are values the selection loop has already computed, so an
/// instrumented run stays byte-identical to an uninstrumented one.
pub fn record_utility_terms(
    winner: &[f64; UTILITY_TERM_COUNT],
    socialize: &[f64; UTILITY_TERM_COUNT],
    runner_up: &[f64; UTILITY_TERM_COUNT],
    socialize_gap: f64,
) {
    if !enabled() {
        return;
    }
    if let Ok(mut sink) = sink().lock() {
        sink.terms_samples += 1;
        for i in 0..UTILITY_TERM_COUNT {
            sink.terms_winner[i] += winner[i];
            sink.terms_socialize[i] += socialize[i];
            sink.terms_runner[i] += runner_up[i];
        }
        sink.socialize_gap
            .observe(socialize_gap <= 0.0, socialize_gap);
    }
}

/// i387: record the dominant motivation at one arbitration.
pub fn record_dominant(need: MotiveCategory, pressure: f64) {
    if !enabled() {
        return;
    }
    if let Ok(mut sink) = sink().lock() {
        sink.dominant_counts[motive_index(need)] += 1;
        sink.dominant_pressure_sum += pressure;
        if pressure > sink.dominant_pressure_max {
            sink.dominant_pressure_max = pressure;
        }
    }
}

/// Cap on the per-sample sweep pairs (instrumentation memory bound).
const QUIET_PAIR_CAP: usize = 200_000;

/// i356: offline coefficient sweep for the `Idle` recreation driver — wins
/// at coefficient `c` are the recorded quiet samples whose pre-driver
/// `winner − Idle` gap is ≤ `play_pressure × c`. Mirrors [`quiet_sweep`];
/// reads only recorded values, so the run is unaffected.
#[must_use]
pub fn quiet_idle_sweep(coefs: &[f64]) -> Vec<(f64, u64)> {
    let Ok(sink) = sink().lock() else {
        return Vec::new();
    };
    coefs
        .iter()
        .map(|&c| {
            let wins = sink
                .quiet_idle_pairs
                .iter()
                .filter(|&&(gap, p)| gap <= p * c)
                .count() as u64;
            (c, wins)
        })
        .collect()
}

/// i356: number of recorded quiet-window idle pairs (the sweep denominator).
#[must_use]
pub fn quiet_idle_pair_count() -> usize {
    sink().lock().map_or(0, |s| s.quiet_idle_pairs.len())
}

/// i356: the recorded quiet-window `winner − Idle` gaps (unsorted), so a probe
/// can report the distribution the coefficient must clear. Instrumentation
/// only; copies the recorded column.
#[must_use]
pub fn quiet_idle_gaps() -> Vec<f64> {
    sink().lock().map_or(Vec::new(), |s| {
        s.quiet_idle_pairs.iter().map(|&(gap, _)| gap).collect()
    })
}

/// i351: offline coefficient sweep over the RECORDED quiet samples — wins at
/// coefficient `c` are the samples whose pre-driver gap ≤ pressure × c. Reads
/// only recorded values; the run itself is unaffected.
#[must_use]
pub fn quiet_sweep(coefs: &[f64]) -> Vec<(f64, u64)> {
    let Ok(sink) = sink().lock() else {
        return Vec::new();
    };
    coefs
        .iter()
        .map(|&c| {
            let wins = sink
                .quiet_pairs
                .iter()
                .filter(|&&(gap, p)| gap <= p * c)
                .count() as u64;
            (c, wins)
        })
        .collect()
}

/// A read-only snapshot of the census.
#[derive(Debug, Clone)]
pub struct Report {
    /// Decisions per selection source (`SOURCE_NAMES` order).
    pub sources: [u64; SOURCE_COUNT],
    /// Decisions per final action (`ACTION_NAMES` order).
    pub actions: [u64; ACTION_COUNT],
    /// `cross[source][action]`.
    pub cross: [[u64; ACTION_COUNT]; SOURCE_COUNT],
    /// Utility-AI arbitrations sampled.
    pub utility_samples: u64,
    /// Distance from the winner for the `Wander` candidate.
    pub wander: GapStats,
    /// Distance from the winner for the `Idle` candidate.
    pub idle: GapStats,
    /// Arbitrations where the need-quietude gate was open.
    pub quiet_samples: u64,
    /// i387: arbitrations carrying a per-bucket decomposition.
    pub terms_samples: u64,
    /// i387: dominant motivation per arbitration (`MOTIVE_NAMES` order).
    pub dominant_counts: [u64; MOTIVE_COUNT],
    /// i387: sum/max of the dominant pressure across arbitrations.
    pub dominant_pressure_sum: f64,
    /// i387: max dominant pressure seen.
    pub dominant_pressure_max: f64,
    /// i387: summed bucket contributions for the winner.
    pub terms_winner: [f64; UTILITY_TERM_COUNT],
    /// i387: summed bucket contributions for the `Socialize` candidate.
    pub terms_socialize: [f64; UTILITY_TERM_COUNT],
    /// i387: summed bucket contributions for the runner-up.
    pub terms_runner: [f64; UTILITY_TERM_COUNT],
    /// i387: distance from the winner for the `Socialize` candidate.
    pub socialize_gap: GapStats,
    /// Quiet-window `Wander` gap stats (the driver's actual arena).
    pub quiet_wander: GapStats,
    /// Sum/Max of the winner's absolute utility at quiet windows.
    pub quiet_winner_sum: f64,
    pub quiet_winner_max: f64,
    /// Which action won each quiet-window arbitration.
    pub quiet_winner_actions: [u64; ACTION_COUNT],
}

impl Report {
    /// Total decisions recorded (the sum over sources).
    #[must_use]
    pub fn total(&self) -> u64 {
        self.sources.iter().sum()
    }
}

/// Read the accumulated census.
#[must_use]
pub fn report() -> Report {
    let Ok(sink) = sink().lock() else {
        return Report {
            sources: [0; SOURCE_COUNT],
            actions: [0; ACTION_COUNT],
            cross: [[0; ACTION_COUNT]; SOURCE_COUNT],
            utility_samples: 0,
            wander: GapStats::default(),
            idle: GapStats::default(),
            terms_samples: 0,
            dominant_counts: [0; MOTIVE_COUNT],
            dominant_pressure_sum: 0.0,
            dominant_pressure_max: 0.0,
            terms_winner: [0.0; UTILITY_TERM_COUNT],
            terms_socialize: [0.0; UTILITY_TERM_COUNT],
            terms_runner: [0.0; UTILITY_TERM_COUNT],
            socialize_gap: GapStats::default(),
            quiet_samples: 0,
            quiet_wander: GapStats::default(),
            quiet_winner_sum: 0.0,
            quiet_winner_max: 0.0,
            quiet_winner_actions: [0; ACTION_COUNT],
        };
    };
    Report {
        sources: sink.sources,
        actions: sink.actions,
        cross: sink.cross,
        utility_samples: sink.utility_samples,
        wander: sink.wander,
        idle: sink.idle,
        terms_samples: sink.terms_samples,
        dominant_counts: sink.dominant_counts,
        dominant_pressure_sum: sink.dominant_pressure_sum,
        dominant_pressure_max: sink.dominant_pressure_max,
        terms_winner: sink.terms_winner,
        terms_socialize: sink.terms_socialize,
        terms_runner: sink.terms_runner,
        socialize_gap: sink.socialize_gap,
        quiet_samples: sink.quiet_samples,
        quiet_wander: sink.quiet_wander,
        quiet_winner_sum: sink.quiet_winner_sum,
        quiet_winner_max: sink.quiet_winner_max,
        quiet_winner_actions: sink.quiet_winner_actions,
    }
}
