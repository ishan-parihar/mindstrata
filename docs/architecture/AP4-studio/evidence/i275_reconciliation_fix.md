# Iteration 275 — WP-H2 reconciliation fix (implementation evidence)

**Status:** LANDED · **Doctrine:** probe-first; two root causes + one re-derivation, one system.

## 3. Action-bias re-derivation (0.10 → 0.01, §4.2)

The 0.10 coefficient (DC-2.4 ramp) was ratified against a DEAD tension diet
(0–3 sporadic counts). With the diet live, the first full gate showed real
behavioral blast: the seed-46 demographic pipeline re-timed (births
[2890] → [240, 280]), 7 snapshots moved, both goldens moved, and the
noospheric conviction test's weak-leg rose past its floor (0.19 → 0.2175).

**Re-derivation:** 0.01 × measured counts (1.25–2.75/agent at 1–2K) → bias
≤ 0.0275 × social_value — inside the audited 0–0.03 magnitude band,
restoring the ratified "5–20% of the social driver" contract. The ramp's
evidence base was void (it tuned against a channel that never fired); the
original DC-2.1 value is correct FOR THE LIVE REGIME. Recorded here as a
re-derivation with mechanism, not a re-pin.

### Re-anchor trail at 0.01 (all mechanism: tie-break re-timing under a
bounded, live bias)

- 7 snapshots: 500/2K metrics show small tie-break shifts (hunger ±0.006–0.025,
  stress −0.008, trust +0.0005); 10K surface shows demographic re-timing
  (agent_count 13 → 12, envy 0.002 → 0.011); 3 snapshots byte-identical.
- Both goldens: all three hashes move, agent_count stable (12) — schedules
  re-timed, population structure intact.
- Conviction test: legs measured by probe (high 0.4860 / low 0.2175, delta
  0.2685 — the differential WIDENED 2.24×; relay reinforcement helps the
  confident ecology more). Floors re-anchored to the measured legs.

### Debt recorded

- The bias reads a horizon-integral of tension claims (grows unboundedly);
  at 5K+ the count reaches 17–76/agent. A recency window is the principled
  fix — deferred until a 5K+ behavioral pin exists to re-anchor against.

## What was fixed (two dead edges in one graph)

### 1. The projection starved the tension gate

`project_catalyst` mapped every Threat event to `Material/Event/Fact/cognitive`
— mono-subtle per (referent, line), so `advance_to_active_tension` never saw
a sibling with a different claim. Probe i275 (pre-fix): 1,292 claims at 20K,
zero ActiveTension.

**Fix — severity-grounded Threat projection** (`project_catalyst_severity`):
- MINOR conflict (verbal Threat/Intimidation) → Fact ("something bad happened")
- MAJOR conflict (Violence/Combat/Revolution/MoralPanic, incl. FeudFormed) →
  Identity ("we are the kind of people who live under threat")

Both on the SAME (Material, Event, cognitive) slot: an agent exposed to both
severities holds the polarity pair the graph exists to synthesize. All other
kinds project unchanged (one root cause at a time). Zero-at-zero preserved:
minor-only windows project exactly what v1 projected.

### 2. The pair-scan gate was mutually exclusive with the synthesizer

The reconciliation scan called `is_active_tension` (requires
`a.domain != b.domain`) then `reconcile_subtle` (requires
`a.domain == b.domain`). **Both can never pass** — the scan was provably dead
code since DC-2.1, invisible while the projection kept the pool empty.

**Fix:** gate the scan on `reconcile_subtle`'s own contract (same
domain/referent/line, both past Undiscovered, different subtle claims).
Also fixed a multi-pair removal hazard (dedupe scheduled indices before
removal).

## Measured equilibria (seed 42, N=12)

| Horizon | Pre-fix | Post-fix (tension) | Post-fix (integrated) |
|---|---|---|---|
| 1K (golden) | 0 / 0 | 15 total (~1.25/agent) | 4 |
| 2K (snapshot) | 0 / 0 | 33 (~2.75/agent) | 4 |
| 5K | 0 / 0 | 206 | 40 |
| 20K | 0 / 0 | 910 | 82 |

**The action-bias contract holds at calibration horizons:** the 0.10
coefficient was ratified (DC-2.4) against a 0–3 tension-per-agent regime;
the measured regime at 1–2K is 1.25–2.75 — inside the ratified envelope.
The bias's cumulative-count growth at 5K+ (mean 17→76/agent) is recorded
debt: the consumer reads a horizon-integral; a recency window is the
principled fix, deferred until a 5K+ behavioral pin exists to re-anchor
against (no constant pulled without evidence).

## The interoception test trail (honest §4.2 record)

With the projection fixed but reconciliation still dead, the unbounded pool
drove the action bias hard enough to INVERT `emotional_body tone`'s pin
(arousal 0.204 vs 0.231 — measured via `i275_interoception_diag`). After
closing the loop, the original equilibrium is RESTORED (arousal 0.206 vs
0.196) and conflict exposure is identical across sensitivity variants
(136 = 136: bounded bias, no chaotic divergence). The test now pins BOTH:
identical conflict exposure (loop-closure invariant) and the original
arousal ordering (physiological channel), with the full trail in the test
comment. **This is a re-contract per §4.4 rule 4**: the old single pin was
true under both the dead-channel regime AND the post-closure regime, but
its meaning changed — it now certifies a live, bounded polarity graph.

## New unit evidence

`polarity_tension_reconciles_to_integrated`: minor→Fact, major→Identity on
the same slot; joint window synthesizes Integrated (the check that would
have caught BOTH dead edges).

## Files

- `crates/mindstrata-development/src/polarity.rs` — `project_catalyst_severity`
- `crates/mindstrata-sim/src/systems/development.rs` — 4-tuple catalysts with
  severity discriminator; scan gate on reconcile_subtle; dedupe removal
- `crates/mindstrata-sim/src/sim/tests/psychology.rs` — re-contracted pin
- `crates/mindstrata-sim/src/sim/tests/development.rs` — liveness test
- Probes: `i275_polarity_reconciliation.rs`, `i275_tension_regime.rs`,
  `i275_interoception_diag.rs`
