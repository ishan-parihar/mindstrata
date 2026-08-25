# IC-3 — Determinism Law

```yaml
provider: PLATFORM
consumer: ALL
owners: [SIM, PLATFORM]
frozen_at: groundwork-c071a3e (ratifying commit stamps final sha per follow-up-stamp mechanism)
version: 1.0.0
status: RATIFIED-P0
change_orders: []
```

## Purpose

Every run of the simulation, on any machine, at any optimization mode (release), with the
same seed and inputs, produces bit-identical state trajectories and artifacts. This is the
property golden replay certifies and every department depends on.

## Surface

1. **Numeric discipline**: per-tick incremental math uses f64 shadows; ONE quantization
   point into `Fixed` (`Fixed::from_f64`) where values enter persistent state or events.
   No raw float accumulation inside Fixed-domain state.

   *Grounding*: `Fixed` is a fixed-point i64 with `SCALE = 10_000`
   (`crates/mindstrata-core/src/fixed.rs:15`, struct at `:30`). `from_f64` performs the
   single rounding step (`(v * SCALE as f64).round() as i64`, `fixed.rs:54–57`); `Mul`
   truncates through an i128 intermediate (`fixed.rs:195–204`) and `Div` likewise
   (`:214–220`). Any per-tick increment below the 1e-4 quantum therefore quantizes to
   zero when accumulated in Fixed-domain state. The canonical precedent is the thermal
   integrator shadow `body_temperature_f64`
   (`crates/mindstrata-person/src/biology/thermal.rs:42–50`): convergence rate 0.001 and
   a metabolic-warmth term near 1e-5/tick were erased every tick by quantized
   accumulation until the f64 shadow carried full precision between ticks.
   **Rule**: sub-resolution rates compute in f64 shadows and quantize exactly once at the
   persistence/event boundary; never accumulate through a quantized field tick over tick.

2. **RNG stream law**: draws are append-only in documented order; birth-path constructors
   consume in field order; new draw sites append at stream END with a comment naming count
   and order. Never insert draws mid-stream.

   *Grounding*: stream machinery lives in `crates/mindstrata-core/src/rng.rs:17–75`
   (`RngStream` enum, stable `ALL` ordering, master-seed construction). The birth path is
   the worked example: child RNG seeded deterministically from config seed + tick +
   child index (`crates/mindstrata-sim/src/sim/births_deaths.rs:576–581`),
   one-draw-per-trait inheritance via `Personality::inherit`
   (`births_deaths.rs:591–597`; implementation `crates/mindstrata-person/src/person/psyche.rs:152–187`,
   single draw per trait at `:163`, field-order consumption `:169–180`, zero extra draws),
   then `EmbodiedState::born` continues the same stream (`births_deaths.rs:613–618`).
   Different range widths consume different byte counts — count-alignment ≠ byte-alignment.

3. **Iteration order**: all passes iterate agents/structures in deterministic index order;
   no HashMap iteration may reach observable behavior.

4. **Pass purity**: tick passes are pure functions of (state snapshot, pre-drawn catalysts);
   no wall-clock, no thread-local entropy, no environment reads.

   *Grounding for the related write-back hazard*: since the mem::take buffer refactor,
   pass writes land in taken buffers that a write-back loop re-attaches — writes into the
   placeholder are silently discarded unless synced during/after that loop
   (`crates/mindstrata-sim/src/sim/core.rs:94–122` buffer extraction, `:171–270` pass
   sequence, `:271–298` write-back loop). Iteration 242 found health-sync dead inside the
   taken placeholder for 24 iterations; the loop now converges legacy fields toward the
   biological envelope after systems deltas each tick. Any new pass writing agent state
   must route through the write-back discipline documented at `core.rs:271–298`.

5. **Save schema**: versioned; migrations are pure old→new transforms; round-trip test is
   part of the gate.

## Guarantees

- PLATFORM maintains the budget table + benchmark harness; determinism regressions are
  release-blocking and bisected via pristine-archive recipe (AGENTS.md §3 / QA runbook).
- Consumers may rely on replay identity for any fixed seed family committed to the registry.

## Obligations

- Consumers NEVER introduce float nondeterminism (no parallel reduction without ordered
  combine), never reorder draws, never iterate unordered collections observably.
- Violations found in foreign territory are reported to PROD, not patched across territories.

## Enforcement checklist (reviewer-run)

1. Every new per-tick increment < 1e-4 computes in an f64 shadow with a named
   quantization point (`Fixed::from_f64` call site) where state persists.
2. Every new RNG draw site appends at stream end with an order/count comment; birth-path
   additions consume in field declaration order.
3. No new HashMap/HashSet iteration reaches events, ordering, or selection without a
   sort/index step.
4. New passes read snapshots, not live partials; any mem::take write follows the
   `core.rs:271–298` re-attach discipline.
5. Golden hashes registered before merge for structural moves; drift bisected via
   pristine-archive before pins are touched.
6. Release-mode suite green (`scripts/gate --full` for behavioral folds).

## Working notes — RNG draw-site inventory & pass-purity audit (task-complete)

Stream registry (`crates/mindstrata-core/src/rng.rs:17–76`): six ChaCha8 streams
`World/Behavior/Social/Economy/Ecology/Narrative`, master-seed offsets `+1,+3,+4,+5,+6,+7`;
streams are **not serialized** — replay always re-seeds from the master seed, so save
artifacts must never carry stream state.

Draw-site inventory across sim + social + world crates (verified by grep at audit time):

| Stream | Sites |
|---|---|
| `World` | `crates/mindstrata-world/src/world_gen.rs:25` only (init-time generation) |
| `Behavior` | actions/mod.rs:647; core.rs:309–310 (spatial wander pair); memory_ops.rs:211 (rehearse); norms_impl.rs:176; pass_scenario.rs:133,188; social_cluster.rs:405 |
| `Social` | core.rs:457,542; clans.rs:436; marriage.rs:141; legal_impl.rs:121; births_deaths.rs:413; pass_health.rs:125,174,182; social_cluster.rs:259,332,644,719,744,815; norms_impl.rs:56; **mindstrata-social** social/interaction.rs:192,222,548 (reached from tick via pass_social.rs:202–213 handing `ctx.rng` to `system_social_interactions`) |
| `Economy` | diplomacy_impl.rs:24 |
| `Ecology` | pass_weather.rs:23 (season advance) |
| `Narrative` | institutions_impl.rs:934 |

Documented exceptions to the stream-per-subsystem rule — four ad-hoc ChaCha instances
plus the birth-path child RNG, all replay-stable by construction:

1. **Birth-path child RNG** (`sim/births_deaths.rs:~29`, replacement-newborn path; the
   newborn path at :576–618 seeds per-child) constructs its own ChaCha instance seeded
   `config.seed + tick + child_idx` rather than drawing from a registered stream —
   deliberate isolation so birth-order changes cannot shift other subsystems.
2. **Ideology inheritance RNG** (`births_deaths.rs:153` and `:730`):
   `ChaCha8Rng::seed_from_u64(seed ^ tick.wrapping_mul(0xA1) ^ idx)` — code-commented as
   deliberate derived streams for ideo inheritance on household death / newborn paths.
3. **Population seeding RNG** (`sim/population.rs:362`): `populate_rng` seeded
   `config.seed.wrapping_add(1000)` — founder generation only.
4. **Cross-stream borrowings**: social_cluster.rs:405 and norms_impl.rs:176 draw
   `Behavior` from social-context code. Legal (any site may use any stream) but each is
   pinned here as intentional.

New-site rule remains: append at the target stream's end with an order/count comment;
census above is the baseline any future append is measured against.

Pass purity spot-check: `tick_biology_pass` (`sim/pass_biology.rs:6–192`) contains zero
RNG draws — pure function of bodies/needs/emotions/weather/params. The remaining passes
consume `ctx.rng` strictly at their fixed position in the core.rs sequence (:171–270),
which preserves byte-identical replay.

No ordering violations found at audit time.

## Tests guarding this contract

- Golden replay suite (`scripts/gate --full`); save/load round-trip harness (QA phase 5);
  seed-family determinism runner (QA phase 6).

## Changelog

| ver | change | approved |
|---|---|---|
| 1.0.0-draft | initial from AP3 doctrine §2 + AGENTS.md §5 | P0 pending |
| 1.0.0-draft | grounding evidence added: Fixed API, thermal shadow precedent, RNG law sites, mem::take trap narrative; enforcement checklist; owners SIM+PLATFORM | P0 pending |
| 1.0.0 | RNG draw-site census (six streams, four ad-hoc ChaCha exceptions + birth-path isolation, biology-pass purity pin), pass-purity audit complete → ratified for DC-1 at the P0 window; frozen against groundwork c071a3e, ratifying commit sha stamped on top | P0 window |
