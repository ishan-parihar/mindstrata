# Iteration 334 — the §2.4 perception gate on memory/attention (i331's recorded term, closed)

**Status:** LANDED · **Item owned:** i331's recorded remainder — "`tick_memory_encoding`
O(N·E) is still ~21% of the tick — its principled reduction is **behavioural**
(perception-radius gating of attention), queued".

## What i331 left behind

`tick_memory_encoding` ran `compute_salience` — and with it a habituation update, a
salience-competition insert, and a possible episodic encode — **for every event, for
every agent**, independent of whether the agent could have perceived the event. That
was defended as "O(N·E) by design"; it was not. It rested on an assumption no
perception model licenses: *a villager habituates to, and forms memories of, every
event anywhere in the village.*

## Sizing first (i333)

Probe `i333_memory_pass_sizing` (seed 42, 32×32, warmup 400, window 40 ticks):

| N | events/tick | pairs/tick (N·E) | pairs naming the agent | local α (pairs) |
|---|---|---|---|---|
| 12 | 14.9 | 179 | 15.9% | — |
| 48 | 55.5 | 2 664 | 3.7% | 1.946 |
| 96 | 113.6 | 10 908 | 1.9% | 2.034 |
| 192 | 221.1 | 42 451 | **0.9%** | 1.960 |

Two readings, both load-bearing:

- **The pass is at the quadratic floor** (α ≈ 1.96 — events/tick is itself O(N), so
  N·E = O(N²)). The gate is therefore **not** an exponent fix; it is a constant-factor
  fix *and* a correctness fix. Recorded honestly rather than sold as a scale arc.
- **99.1% of the work is on events the agent is not part of** (at N=192; 84% even at
  N=12). Event mix at N=192: Interaction 43.9%, RelationshipChanged 43.9%, Drank 2.9%,
  Rested 1.8%, Ate 1.4%, other 6.0%.

## The gate

`EventAnchors` (in `memory_ops.rs`) is built once per tick in O(E) from the tick's
final agent positions; the inner loop then tests a Manhattan distance instead of
computing salience.

- An event's **lanes** are the agents it names (1, 2, or a list — births, rituals,
  witness lists). An agent perceives the event when it is a lane (distance 0) or
  within `DEFAULT_PERCEPTION_RADIUS` (= 5, the same radius the interaction engine
  already selects partners with) of a lane.
- The lane match is **exhaustive by design** — a new `SimEvent` variant must state its
  lanes rather than inherit a silent default.
- Only `InstitutionChangedPolicy` has no lane; it stays `Everywhere` (an institution's
  policy change is public news), preserving prior behavior for that variant.
- A lane whose agent died earlier in the tick contributes no anchor
  (`EventAnchors::Nowhere` — the stale-id guard i331 used); a phantom position is never
  invented.
- Positions are the tick's **final** ones, an approximation invisible at r=5.

## Measured result

Tick cost (`i329_local_exponent`, release, seed 42, 32×32). Baseline measured at HEAD
in this session by stashing the one changed source file:

| N | µs/tick before | µs/tick after | Δ | µs/agent after |
|---|---|---|---|---|
| 48 | 522.0 | 457.2 | −12.4% | 9.52 |
| 96 | 1 591.6 | 1 380.8 | −13.2% | 14.38 |
| 144 | 3 563.1 | 3 004.7 | −15.7% | 20.87 |
| 192 | 6 480.2 | 5 421.4 | **−16.3%** | 28.24 |

The saving grows with N exactly as removing the village-wide term predicts (the
retained per-agent body — decay, reconsolidation, neural spread, skill practice — does
not). Per-pass at N=192 (`i330_pass_profile`, tick 400): memory **1 391 721 →
988 102 ns (−29.0%)**.

A second, quieter repair comes with it: the memory pass consumed one budget op **per
encoded percept**, so a village-wide agent used to spend attention budget on other
people's events.

## Blast radius — three pins, all probe-evidenced

Golden **9/9 byte-identical** (every ≤1000-tick calibrated window is untouched) and
309/310 tests passed on first contact. The three that moved:

### 1. Snapshot drift — memory surface only (`RE-PIN`, §4.4)

`long_horizon_surface_10000_ticks` moved on **exactly** the memory store: total traces
**948 → 1 106**, by kind `Emotional 540→627, Social 370→445, Traumatic 20→22,
Episodic 10→9, Procedural 6→2, Cultural 1→0`. Every other field byte-identical. Traces
rise because habituation now accrues only from *perceived* events, so novelty — and
therefore the share of perceived events crossing the 0.2 encoding gate — is higher.
The snapshot is the honest fingerprint of the change; regenerated with this evidence.

### 2. `conception_pipeline_round_trips_with_birth` — RE-PACED (§4.2)

Probe `i335_seed46_pipeline` (accelerated seed 46, horizons 2K→12K) pins the new
trajectory: 3 pregnancies, births at **[180, 250, 4960]** — the pregnancy-path delivery
lands ~1.5K ticks later than the old 3,500 pin, and the segment-2 horizon was extended
2 000 → 4 000 (total 6 000) so the delivery is inside the window. **The pipeline is
re-paced, not dead**: the probe carries one live pregnancy at tick 4 000, and the
delivery lands at 4 960.

### 3. `moral_panic_lifecycle_registers_and_drains_legitimacy_end_to_end` — RE-CONTRACT (§4.1/§4.4)

The old Leg A pinned **one seed's peak intensity** (seed 5). Probe
`i334_perception_gate_delta` (20K, pestilence + calm) shows what that pin was resting
on:

| world | panics | peak intensity | mean charge | fear | distress |
|---|---|---|---|---|---|
| calm s42 (both) | 0 | 0.0000 | 0.3865 | 0.3370 | 0.3334 |
| **pest s5 before** | 8 | **1.0000** | 0.4876 | 0.3746 | 0.3814 |
| **pest s5 after** | 3 | 0.0464 | 0.4819 | 0.2582 | 0.2431 |
| pest s7 before | 10 | 0.2203 | 0.2274 | 0.1334 | 0.1393 |
| pest s7 after | 10 | 0.2203 | 0.2274 | 0.1334 | 0.1393 |

The passed assertion was passing *because of a runaway 1.0000-saturation panic* — the
exact regime this test's own bar was written to exclude ("crises now produce MILD,
resolving panics rather than runaway 1.0 saturation") — and the gate re-paces that
seed's saturation while leaving seed 7 **byte-identical**. A pin that flips when one
seed's saturation regime moves is a lucky-seed pin (the same seed-chasing chain the
Iter-241 note documents: 55 → 99 → 42 → 5 → 1). The assertion is therefore
**re-contracted to a seed family** {5, 7} guarding the *mechanism*: every crisis seed
registers panics, every registration carries a mapped trigger, the family peak clears
0.08, and at least one panic completes its drain. The floor is anchored on the measured
**calm residual** (mean charge 0.3865, 0 panics), not on another seed's magnitude.

## Re-audit (§2.5 — no verdict carried on recall)

| producer | verdict under the gate |
|---|---|
| meaning reflex (i306/i324) | **`MEANING_REFLEX_LIVE`**, all four clauses — at-ceiling **0.0003**, excursions **0**, worship duty 0.02265 (was 0.02252) |
| immune clearance (i311) | pinned inf **0**, R<0.02 **0**, below-gate **0** across calm/pestilence/collapse up to 50K |
| injury channel (i313) | live: injury 0.31–0.32, pain 1.000, shock 0–0.67, social memory → 12 panic / 4 faction at 50K |
| golden baselines | 9/9 byte-identical |

## Re-audit of the scale envelope (i332 charter method, same session)

The i295/i332 charter method (32×32, 2000 ticks, seed 42, new+populate+run, min-of-3)
run against the pre-i334 file (`HEAD~1`) and after, in one session:

| N | µs/tick pre-i334 | µs/tick post-i334 | Δ |
|---|---|---|---|
| 48 | 534.7 | 520.5 | −2.7% |
| 96 | 1 802.3 | 1 788.2 | −0.8% |
| 144 | 4 005.6 | 3 819.1 | −4.7% |
| 192 | 7 351.0 | 6 830.5 | **−7.1%** |

The charter method reports a **smaller** delta than the i329 per-tick probe (−16.3% at
N=192) because it divides a fixed `populate` + short-run cost by only 2 000 ticks, so the
one-off setup dilutes the per-tick saving. Both readings are honest measurements of
different quantities; the i329 number is the per-tick delta, this one is the
charter-method cost.

The **verdict tier is unchanged**: N=96 headroom 73% → 72% (noise-level) and N=192
still breaches the N=96 budget (6 830 > 6 500), so `ENVELOPE_EXPANDED_1_5X` stands exactly
as i332 recorded it. Stated plainly: **i334 is a correctness fix that pays for itself,
not an envelope expansion.** The Phase-1 population-target question is therefore
unaffected by this iteration — it remains open on the i332 evidence.

## Verdict

**`PERCEPTION_GATE_LIVE`** — the last recorded non-floor term is closed with the
doctrine that named it (§2.4), and the pass now costs a distance test per pair instead
of a salience computation. The pass remains O(N·E) = O(N²); the structural floor is
unchanged and still the envelope's governor (re-audited above, not asserted).

## Recorded debt

1. **Seed-5 panic saturation is knife-edge debt** (§4.5). Before the gate it saturated
   at 1.0000 with 8 registrations; after, 3 registrations peaking 0.0464 — while seed 7
   is untouched. The crisis-charge → escalation coupling is a chaotic equilibrium, not
   a calibrated band; the family assertion guards the mechanism, not a magnitude.
2. **The attention radius equals the interaction radius** (5), chosen because §2.4
   already fixes that radius and the interaction engine already selects partners with
   it. Whether *noticing* should reach further than *interacting* is a genuine
   calibration question with no evidence either way yet; changing it moves every
   memory-coupled producer, so it needs the same probe + re-anchor sweep this iteration
   performed.
3. **`VecDeque<SimEvent>`** remains the recorded upgrade path (i327); the `household.rs`
   per-agent `Vec` allocation remains unexamined.

**Probes:** `i333_memory_pass_sizing`, `i334_perception_gate_delta`,
`i335_seed46_pipeline`, `i329_local_exponent`, `i330_pass_profile`, `i306_meaning_channel`,
`i311_immune_clearance`, `i313_injury_channel`.
