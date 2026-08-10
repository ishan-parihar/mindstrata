//! §2 Behavioral-delta harness — systematic same-seed, consumer on/off proofs.
//!
//! The queue's testing gap (§2): behavioral proofs were historically ad-hoc
//! per-iteration differential probes. This module makes the pattern reusable
//! and self-documenting: run the SAME seed twice — once with default tuning
//! parameters, once with one consumer knob mutated — and either
//!
//! - assert a measurable, directional delta (the consumer is LIVE in this window), or
//! - assert byte-identical metrics (the consumer is DORMANT — its inputs never fire in this window — an honest, probe-pinned zero-blast).
//!
//! Identity at zero is the campaign's calibration contract: a consumer whose
//! inputs are zero must be byte-identical to the baseline, and a consumer
//! whose inputs are live must move a downstream metric.
//!
//! Probe-pinned calibration findings (Iteration 134) — a calm-window inertness
//! sweep across candidate knobs at seed 42:
//!
//! - SATURATED beyond the default (byte-identical): the Iter-127 gratitude consumer `social_gratitude_help_multiplier` (`help_propensity` is `clamp_01()`, so 0.5 → 0.9 is already past the cap; its blast was the 0 → 0.5 turn-on), plus `meme_transmission_multiplier` 1.0 → 2.0 (transmission chance saturates at 1.0), `trust_sync_rate`, and `appraisal_sadness_multiplier` (sadness is ~0 in calm windows).
//! - IDENTITY-AT-ZERO: `endocrine_stress_recovery` 0.05 → 0.2 is byte-identical at seed 42/2000 — the legacy sensitivity test's one-sided `<= baseline + 0.05` bound passes vacuously; the harness's strict delta requirement is what makes such proofs honest.
//! - LIVE: the conflict channel (`conflict_escalation_chance` 0.3 → 0.9 and `conflict_escalation_rate` 1.0 → 3.0) moves events by thousands.
//! - NON-MONOTONIC: `bonding_rate` 0.9 changes relationship quality but in the NEGATIVE direction (a cascade, not a defect) — direction-blind magnitude assertions are the right contract for unknown cascades.
//!
//! Constraint: seed-42 calm windows make most knobs inert, so the demo set is
//! limited to consumers that actually fire here. Consumers that only fire at
//! scale or under stress (the campaign's D3–D6 lesson) need a larger/longer or
//! scenario config — build it directly from the public `SimConfig`/`Simulation`
//! API when probing them.

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::parameters::SimParameters;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::MetricsSnapshot;

use crate::test_helpers::{run_scenario_with_params, run_sim_with_params};

/// Result of a single behavioral-delta experiment.
#[derive(Debug)]
pub struct DeltaReport {
    /// Human-readable name of the mutated knob.
    pub knob: &'static str,
    /// Metric value under default parameters.
    pub baseline: f64,
    /// Metric value under the mutated parameters.
    pub treated: f64,
    /// `treated − baseline`.
    pub delta: f64,
    /// True when the FULL metrics snapshots are byte-identical (JSON-equal).
    pub zero_blast: bool,
}

/// Run a behavioral-delta experiment: same seed, baseline vs mutated params.
pub fn behavioral_delta(
    seed: u64,
    ticks: u64,
    knob: &'static str,
    modify: impl FnOnce(&mut SimParameters),
    metric: impl Fn(&MetricsSnapshot) -> f64,
) -> DeltaReport {
    let baseline = run_sim_with_params(seed, ticks, |_| {}).metrics_snapshot();
    let treated = run_sim_with_params(seed, ticks, modify).metrics_snapshot();
    let b = metric(&baseline);
    let t = metric(&treated);
    DeltaReport {
        knob,
        baseline: b,
        treated: t,
        delta: t - b,
        zero_blast: snapshots_identical(&baseline, &treated),
    }
}

/// Assert the mutated consumer produced a live, directional delta.
pub fn assert_live_delta(report: &DeltaReport, min_magnitude: f64) {
    assert!(
        !report.zero_blast,
        "{}: expected a live delta but the metrics are byte-identical",
        report.knob
    );
    assert!(
        report.delta.abs() >= min_magnitude,
        "{}: delta {:.6} below the minimum magnitude {:.6} \
         (baseline {:.6}, treated {:.6})",
        report.knob,
        report.delta,
        min_magnitude,
        report.baseline,
        report.treated
    );
}

/// Assert the mutated consumer is provably dormant in this window (zero blast).
pub fn assert_zero_blast(report: &DeltaReport) {
    assert!(
        report.zero_blast,
        "{}: expected zero blast but the metrics diverged (baseline {:.6}, treated {:.6}, delta {:.6})",
        report.knob, report.baseline, report.treated, report.delta
    );
}

/// Byte-identity over the FULL metrics snapshot via JSON serialization.
///
/// `MetricsSnapshot` derives only Debug/Clone/Serialize/Deserialize, so the
/// total field-by-field comparison is expressed as JSON equality — which also
/// catches any future field added to the snapshot. `expect` (not `.ok()`)
/// deliberately panics if serialization ever fails (e.g. a NaN metric, which
/// JSON rejects) — `None == None` would falsely report zero-blast.
///
/// In-process determinism is sound: the two snapshots come from identically
/// constructed simulations (same seed, same parameter values, same insert
/// order), so any `HashMap` iteration order is identical within the process,
/// and `Fixed::to_f64()` is deterministic.
pub fn snapshots_identical(a: &MetricsSnapshot, b: &MetricsSnapshot) -> bool {
    let ja = serde_json::to_string(a).expect("baseline snapshot serialization");
    let jb = serde_json::to_string(b).expect("treated snapshot serialization");
    ja == jb
}

// ── Harness validation + demo experiments ───────────────────────────

#[test]
fn harness_is_self_consistent_deterministic() {
    let a = run_sim_with_params(42, 2000, |_| {}).metrics_snapshot();
    let b = run_sim_with_params(42, 2000, |_| {}).metrics_snapshot();
    assert!(
        snapshots_identical(&a, &b),
        "the harness itself must be deterministic"
    );
}

#[test]
fn live_consumer_conflict_escalation_chance_is_measurably_live() {
    // 3× the default escalation chance (0.3 → 0.9) must produce measurably
    // more events at the same seed — the conflict channel is LIVE (probe:
    // +1,704 events at seed 42/3000, 60,792 → 62,496).
    let report = behavioral_delta(
        42,
        3000,
        "conflict_escalation_chance",
        |p| p.conflict_escalation_chance = Fixed::from_f64(0.9),
        |m| m.event_count as f64,
    );
    assert_live_delta(&report, 500.0);
    assert!(
        report.delta > 0.0,
        "more escalation chance must not REDUCE events: {report:?}"
    );
}

#[test]
fn live_consumer_bonding_rate_moves_relationship_quality_direction_blind() {
    // A NON-conflict (social/relationship) subsystem demo. Raising the bonding
    // rate to 0.9 changes mean relationship quality at the same seed — the
    // cascade is documented NON-MONOTONIC (probe: delta −0.0067), so this
    // deliberately uses the direction-blind magnitude contract (the harness
    // mode for unknown cascades) instead of asserting a direction.
    let report = behavioral_delta(
        42,
        5000,
        "bonding_rate",
        |p| p.bonding_rate = Fixed::from_f64(0.9),
        |m| m.avg_relationship_quality,
    );
    assert_live_delta(&report, 0.004);
}

#[test]
fn dormant_consumer_reproduction_gestation_rate_is_zero_blast() {
    // 10× slower gestation is byte-identical in a 2,000-tick window: no
    // pregnancy exists (seed-42 births fire far beyond this horizon), so the
    // knob's consumer never reads it and the RNG stream is untouched. This is
    // the honest "identity-at-zero" half of the calibration contract.
    let report = behavioral_delta(
        42,
        2000,
        "reproduction_gestation_rate",
        |p| p.reproduction_gestation_rate = Fixed::from_f64(0.1),
        |m| m.agent_count as f64,
    );
    assert_zero_blast(&report);
}

// ── §46 Scenario-context behavioral delta (Iteration 138) ────────────

/// Run a behavioral-delta experiment in a scenario context (non-calm world
/// where shocks fire and consumers that are inert at seed 42 may become live).
pub fn scenario_behavioral_delta(
    scenario: &Scenario,
    ticks: u64,
    knob: &'static str,
    modify: impl FnOnce(&mut SimParameters),
    metric: impl Fn(&MetricsSnapshot) -> f64,
) -> DeltaReport {
    let baseline = run_scenario_with_params(scenario, ticks, |_| {}).metrics_snapshot();
    let treated = run_scenario_with_params(scenario, ticks, modify).metrics_snapshot();
    let b = metric(&baseline);
    let t = metric(&treated);
    DeltaReport {
        knob,
        baseline: b,
        treated: t,
        delta: t - b,
        zero_blast: snapshots_identical(&baseline, &treated),
    }
}

/// Shared probe: conflict-escalation-chance delta in a scenario context.
/// Extracted (Iteration 140) so every scenario-context differential uses the
/// identical knob/metric and no test duplicates the drought computation.
fn conflict_delta_in(scenario: &Scenario) -> DeltaReport {
    scenario_behavioral_delta(
        scenario,
        3000,
        "conflict_escalation_chance",
        |p| p.conflict_escalation_chance = Fixed::from_f64(0.9),
        |m| m.event_count as f64,
    )
}

/// §46 (Iteration 138): The behavioral-delta harness works in scenario
/// contexts (non-calm worlds with shocks). The drought scenario's water
/// shock at tick 500 creates a structurally different world: the conflict
/// channel remains LIVE but with DIFFERENT baseline conditions.
///
/// Probe-pinned finding: drought DAMPENS the conflict-escalation delta
/// (calm +1704 events vs drought +1435 events at 3000 ticks, same knob
/// 0.3→0.9). Explanation: thirst depresses health, shifting behavior from
/// conflict to survival-mode gathering — the same escalation chance bump
/// has less fuel in a weaker population.
#[test]
fn scenario_delta_is_live_and_contexts_differ() {
    let vanilla = behavioral_delta(
        42,
        3000,
        "conflict_escalation_chance (vanilla seed 42)",
        |p| p.conflict_escalation_chance = Fixed::from_f64(0.9),
        |m| m.event_count as f64,
    );
    let drought = conflict_delta_in(&Scenario::drought());

    // Both contexts are live (the harness works in scenarios).
    assert_live_delta(&vanilla, 500.0);
    assert_live_delta(&drought, 500.0);

    // The scenario world has different baseline conditions. Probe-pinned:
    // vanilla 60,792 vs drought 61,170 at seed 42/3000 (gap 378 events) — the
    // drought shock at tick 500 depletes water but doesn't collapse the event
    // rate (agents stay active in survival mode). The difference is small but
    // structural.
    let baseline_gap = (vanilla.baseline - drought.baseline).abs();
    assert!(
        baseline_gap > 50.0,
        "vanilla baseline ({:.0} events) must differ from drought baseline \
         ({:.0} events) — gap {:.0} — the drought scenario structurally \
         changes the world",
        vanilla.baseline, drought.baseline, baseline_gap
    );

    // The drought delta is direction-blind smaller (survival-mode damping).
    // Both are positive (more escalation = more events), but the gap is a
    // probe-pinned calibration observation: vanilla +{:.0} vs drought +{:.0}.
    assert!(
        vanilla.delta > 0.0 && drought.delta > 0.0,
        "both contexts must show positive deltas (more escalation = more events): \
         vanilla={:.0} drought={:.0}",
        vanilla.delta, drought.delta
    );
}

/// §46 (Iteration 140): The Calm scenario (no shocks) produces a structurally
/// different baseline from the Drought scenario, proving the scenario battery
/// is useful for control-vs-treatment differential testing.
#[test]
fn calm_scenario_baseline_differs_from_drought() {
    let calm = conflict_delta_in(&Scenario::calm());
    let drought = conflict_delta_in(&Scenario::drought());

    // Both are live (the harness works with the new Calm scenario).
    assert_live_delta(&calm, 500.0);
    assert_live_delta(&drought, 500.0);

    // The calm baseline (no shocks) differs from the drought baseline.
    let gap = (calm.baseline - drought.baseline).abs();
    assert!(
        gap > 50.0,
        "calm baseline ({:.0} events) must differ from drought \
         baseline ({:.0} events) — gap {:.0}",
        calm.baseline, drought.baseline, gap
    );

    // Both deltas are positive (more escalation = more events).
    assert!(
        calm.delta > 0.0 && drought.delta > 0.0,
        "both contexts must show positive deltas: calm delta={:.0} drought delta={:.0}",
        calm.delta, drought.delta
    );
}
