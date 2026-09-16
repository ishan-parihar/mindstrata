# Iter-266 WP-I — Collective Field Activation (2026-09-16)

Owner: SIM. Probes: `i266_collective_wp_i` + `i266_catalyst_census`. Baseline HEAD `392e64a`.

## What landed

1. **`CollectiveField::step_collective` is LIVE** (`crates/mindstrata-development/src/collective.rs`):
   press integrates `p·growth` per line, shadow stage advances on press saturation
   (transcend-and-include, carry-over surplus), fulfillment EMA tracks pressure at 0.02.
   Zero-at-zero preserved (empty window → bit-identity, pin stays green).
2. **Slug-based bucket mapping** replaces the DC-1 v1 cyclic `i%4` distribution:
   vault-`kind` affinity (culture→Relational 12, system→Safety 14,
   collective-system→Identity 1, consciousness→Meaning 2) via
   `collective::pressure_vector(...)`. CALIBRATION-PENDING(AP3) pending vendor coupling data.
3. **First behavioral consumer** (`norms_impl.rs::tick_moral_panic_lifecycle`):
   Safety-bucket fulfillment pacifies moral-panic escalation —
   `pressure × (1 − (fulfillment − 0.25) × 0.2)` above the 0.25 anchor, identity below.
4. **Snapshot persistence v14→v15**: `Snapshot.collective_field` with `#[serde(default)]`
   (pre-v15 saves load at neutral = old semantics). Restore path uses the captured state.
   The kinship_graph v10 precedent: behavioral state on Simulation root must round-trip.

## Probe evidence

### i266_collective_wp_i (12 seeds × 5K ticks, N=12)

- **moved_lines=28/29 at every seed** — family_moved_rate=1.0000 (was `is_neutral()=true`
  at all 12 seeds pre-WP-I per i280).
- max_press 0.35 (seed 7) → 0.99 (seed 77); max_stage=2 in 6/12 seeds (stage advance fires).
- Per-bucket fulfillment: rel 0.0008–0.0072, saf 0.13–0.19, ide 0.0 (no deaths in calm
  windows), mea 0.014–0.019. Bucket spread > 0.13 at 12/12 seeds — the mapping differentiates.
- The one unmoved line per seed: the Identity-bucket knowledge-systems line (no Grief
  catalysts in calm windows) — correct bucket isolation.

### i266_catalyst_census (seed 42 × 2000 ticks)

- Catalysts arrive in bursts: **70/2000 ticks carry catalysts** (12 Bond, 146 Threat,
  0 Grief/Transgression at seed 42 calm).
- Worst-tick safety pressure = 6 catalysts / 12 agents = 0.50 per line → per-tick press
  burst 0.025; the field integrates the burst train, explaining max_press 0.61 @2000.
- Safety fulfillment peak 0.19 in calm windows → **consumer anchor 0.25 sits above every
  calm observation** (Iter-112 pattern), identity in golden (1000) + snapshot (≤2000)
  horizons by construction.

## Pre-existing red test found + re-contracted

`dynamics::tests::zero_pressure_zero_intensity_is_bit_identity` FAILED at pristine HEAD —
broken by commit `34e4ca7` (Allergy always-step) without a suite run (the 34e4ca7/f7d9267
closeout audited `gate --full` 307/0/1, which does not include the dev-crate lib tests —
documented as `final-tail-closure.md` 69/69 at an earlier HEAD).

RE-CONTRACT per AGENTS.md §4.4, not a re-pin: the old assertion (bulk `PathologyField::step`
bit-identity at all-zero pressure) tests something the always-step Allergy law legitimately
invalidates (absence growth 0.05×0.1×headroom = 0.005/tick is the mechanism that revived
Q2/Q4). The replacement pin guards the real invariants: Addiction zero-at-zero bit-identity,
exact Allergy absence rate, monotone accumulation under the ceiling.

## Verification signature at this iteration

- `cargo test -p mindstrata-development --lib`: **74/74**
- `cargo test -p mindstrata-sim --lib`: **210/210** (new: collective_field_catalyst_liveness_safety_bucket, snapshot_roundtrip_preserves_collective_field)
- `cargo test -p mindstrata-tests --lib --release`: **307/0/1** (golden byte-identical — zero-blast proven)
- `bash scripts/gate --full`: **GREEN @161s** (golden 5/5, fmt+clippy clean)
- `python3 scripts/bench_index.py --strict`: **41 ok / 36 legacy / 0 violations** (2 new probes law-compliant)

## Remaining (recorded, not blocking)

- Crisis-window sweep `i<iter>_panic_*` to measure the pacify equilibrium shift before the
  0.2 rate is ratified (CALIBRATION-PENDING).
- Cross-line resonance + bucket affinity await vendor coupling data (same debt as dynamics).
- Lambda admission threshold + catalyst magnitudes (Iter-270 batch).
