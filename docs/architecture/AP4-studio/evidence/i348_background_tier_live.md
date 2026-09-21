# Iteration 348 — the Background tier goes live (and its social clause had to go)

**Status:** LANDED (behavioural) · **Root cause owned:** i316 measured the
§17 scaling strategy's cheapest rung — `AgentTier::Background` — at **0
agent-ticks in every run** and deferred the fix as "its own behavioural
iteration". This is that iteration, and it shipped **two** re-contracts:
the entry gate, and the tier's social-participation clause.

## Leg 1 — the entry gate (the deferred fix)

The Secondary→Background edge required `narrative_importance < 0.1`, but the
importance target carries a `+0.15` base plus non-negative bonuses — reachable
floor ≥ 0.15, measured min 0.3000 (i316). Unreachable by arithmetic, not luck.

The first draft re-pinned the gate to 0.32 (from a pre-wiring band table) and
kept a `degree < 2` conjunct on the contacted degree. **The liveness probe
refuted both choices:**

1. The degree conjunct is **unsatisfiable at every runnable N** — contacted
   degree p10 = 2–3, and **0 of the below-gate candidates** across 6 runs
   (3 seeds × N=12/N=48, 10K ticks) had degree < 2. With contact-paced
   relationship building, no village agent is ever socially unembedded.
2. It is also **redundant**: importance's own target already prices gravity
   via `min(contacted_degree × 0.02, +0.2)`.

**And the wiring fix moved the distribution itself.** The §15 reclassification
had been feeding `relationship_v2s.len()` (= N−1 for everyone, the i336/i342
defect) into `update_narrative_importance`, saturating the network bonus at
+0.2 for the whole population. Feeding the honest contacted degree (one O(R)
`contacted_degrees()` pass per reclassify batch) dropped the importance floor
**0.3000 → 0.1943 (N=12) / 0.2110 (N=48)** — importance differentiates for the
first time. Post-wiring band (`i348_background_band`, 3 seeds × calm
N=12/N=48 @10K):

| gate | calm N=12 | calm N=48 |
|---|---|---|
| < 0.10 (old) | 0.000% | 0.000% |
| < 0.20 | 1.995% | 0.000% |
| **< 0.22 (shipped)** | **4.230%** | **11.397%** |
| < 0.25 | 11.731% | 31.465% |
| < 0.30 | 24.578% | 49.618% |

`0.22` sits above the floor-pinned cohort (N=48 min 0.2110 — a 0.20 gate would
re-create the unreachable-gate mistake one notch down) and below N=12 p05
(0.2290). Exit re-contracted to importance-only `> 0.25` (residency band
[0.22, 0.25]): the old `degree >= 2` exit was instantly true for everyone
(churn, not a tier), and `> 0.3` straddled the modal mass. `reclassify`'s
`relationship_count` parameter is removed — entry is importance-only.

## Leg 2 — the social clause (the surprise)

With the gate live, Background held **25–40% of crisis-world agent-ticks**
(pestilence 23–40%, collapse 24.4%) — and the producer sweep found faction
formation collapsed to **3/6 pestilence seeds** (seeds 1/7/42 never formed),
plus panic-legitimacy and echo-chamber liveness anchors failing.

The A/B: `runs_social_interactions()` excluded Background agents from the
social pass — the §17.2 "aggregate simulation" clause. At 25–40% share that
starves every collective producer through it. **Flipping to universal social
presence restored faction formation to 6/6 seeds** (seed 1: 0 → 26 303
live-faction ticks; seed 7: 0 → 25 972; seed 42: 0 → 22 046) and all four
behavioural anchors re-passed unmodified.

The re-contract is principled, not convenient: social withdrawal is already
modelled *behaviourally* (`psychopathology.is_impaired()` gates the same
pass — Iter-187), so the tier must not re-model it as a LOD artifact.
Background stays a **cognitive** rung: zero memory encoding, prospection,
belief updates, theory-of-mind — the predicates that actually cut per-agent
cost (the social pass is not per-agent-cost-dominant; the per-edge passes are,
and those are untouched).

## The sweep, classified

| failure | class | disposition |
|---|---|---|
| golden collapse baseline | re-contract (tier engaged in crisis) | regenerated; `agent_count 12` preserved — mortality invariants intact |
| 10K long-horizon snapshot | re-contract (memory mix shifts: Background no longer encodes; Emotional 436→519, Social 257→343) | reviewed line-by-line: stress 0.366→0.371, health 0.790→0.763, trust −0.015, quality −0.021, polarization 0.061→0.066 — bounded, no saturation, agent_count 12 |
| faction attachment (pestilence seed 1) | **§4.3 starve** (producer dead via social exclusion) | fixed at root (Leg 2), test unchanged and green |
| moral panic, collective fear, echo chambers | §4.3 starve (same mechanism) | fixed at root, green |
| conflict-escalation delta (+47 < 80 on seed 55) | **§4.1 lucky-seed anchor, seventh re-pin** | re-contracted onto a 12-seed family: live on 7/12 (|Δ| ≥ 80, up to +3853), byte-identical on 2, non-monotonic by mechanism. New contract: ≥4/6 family seeds move ≥80, ≥1 positive, ≤1 byte-identical. Probe-delta magnitudes proved construction-sensitive (seed 1: probe −1952 vs harness +2757, both live) — per-seed exact pins are rebuilt treadmills |
| `differentiated_elevated_fear…` unit test | test-construction artifact (calm leg left importance at the unreachable 0.0) | calm leg pinned at 0.4 — fear-channel isolation preserved |

## Honest limits (recorded)

- **Zero re-promotions in vivo** across all 6 liveness runs: bottom-band
  agents are bottom-band *because* their degree stays 3–4; exit needs degree
  ≥ 6 (importance target 0.27+) or an event/emotion lift. Non-trapping is
  pinned at unit level (`reclassify_promotes_background_on_importance_leaving_band`);
  in-vivo promotion is a pacing question for a future iteration, not hidden.
- **Cost**: Leg 2 deliberately gives up part of the LOD saving (Background
  agents still run the social pass). The tier's remaining cuts
  (memory/prospection/belief/ToM) are live and measurable; the social pass was
  never the dominant per-agent cost (i341: the per-edge passes are, at ~58%).
- N=12 Background share is seed-dependent (0–2 agents): the bottom band at
  tiny N is thin. The tier is a scaling mechanism; its target envelope is
  N ≥ 48.

## Probes

- `i348_background_band` — importance distribution + candidate-gate table
  (pre- and post-wiring runs).
- `i348_background_liveness` — entry counts, Background agent-ticks,
  demotion/promotion counts, below-gate degree diagnostics.
- `i348_producer_sweep` — faction-formation + Background share across the
  pestilence seed family and collapse (the Leg-2 A/B instrument).
- `i348_conflict_family` — the 12-seed conflict-delta family table.
