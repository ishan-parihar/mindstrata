---
name: engine-status
description: "The authoritative current-state description of the mindstrata engine: architecture, tick orchestration, subsystem depth, agent behaviour, realism verdicts, and the measured scale ceiling. Load this before planning any iteration. Supersedes docs/MINDSTRATA_CURRENT_STATE.md (AP2-era) and docs/REMAINING_WORK_REPORT.md (Iter-134-era)."
type: Authority
status: AUTHORITY
scope: "engine behaviour, architecture, realism, scale"
reconciled_commit: adc5985
created: 2026-09-22
owner: SIM + PROD (AP4 Studio)
---

# Mindstrata — Engine Status

> **This is the single authoritative current-state document for the engine.**
> If it disagrees with any other doc, this one wins and the other is stale —
> report it and fix the other (see [`DOCUMENTATION.md`](DOCUMENTATION.md)).
> Reconciliation rule: every milestone gate must re-verify the numbers in §1
> and re-run the `reconciled_commit` bump.

---

## 1. Verification baseline (measured, not cited)

| Gate | Command | Result |
|---|---|---|
| Format | `cargo fmt --all --check` | clean |
| Lints | `cargo clippy --workspace --quiet` | 0 warnings |
| Full suite | `cargo test -p mindstrata-tests --lib --release` | **310 passed / 0 failed / 1 ignored** |
| Sim unit | `cargo test -p mindstrata-sim --lib --release` | **295 / 295** |
| Golden replay | `scripts/gate` | 5/5 byte-identical |
| Probe law | `scripts/bench_index.py --strict` | 0 violations |

**Scale of the artefact:** 13 crates · ~139,800 LOC · 1,665 test functions · 186 probes
· 137 evidence docs · 36 RON spec files · ~890 commits.

## 2. Architecture

Deterministic agent-based simulator. The **dependency DAG is cargo-enforced**:

```
core ← person ← psych ← {social, institutions, world} ← sim ← {tui, cli, render, tests, benches}
```

`sim` is orchestration only; the five leaf crates own pure domain logic. `mindstrata-core`
holds `Fixed` (fixed-point, determinism), ids, clock, events, rng, parameters. 36 RON
spec files are linted against the code (`spec_lint`).

## 3. Tick orchestration (the pipeline)

One tick is a fixed ~22-stage deterministic sequence. Order is load-bearing; golden
replay is the referee.

| # | Stage | Domain |
|---|---|---|
| 0 | Scenario shocks | world |
| 1 | relationship trust sync/reset | social |
| 2 | **biology pass** | 11 organ systems, metabolism, circadian |
| 3 | **cognitive pass** | attention, emotion regulation, ToM, skill, decision policy |
| 4 | need decay → body update → goal generation | needs |
| 5 | **action pass** | reflex → command → feud → routine → utility → habit → veto |
| 6 | **social pass** | 8 interaction types |
| 7 | **appraisal pass** | Lazarus appraisal → emotion deltas |
| 8 | write-back (`mem::take` re-attach; orchestration-owned) | — |
| 9 | movement, relationship traces | spatial |
| 10 | weather, resource ops | world |
| 11 | memory encoding | psych |
| 12 | ecology (seasons) + health + epidemic + migration + spoilage | world/person |
| 13 | norm evaluation, gossip, socialization, **innovation** | social |
| 14 | institutional psych, faction, **moral panic + revolution**, belief update | institutions/noosphere |
| 15 | social cluster (polarization, echo chambers) | social |
| 16 | **marriage**, births/deaths (deca cadence), rituals | demography |
| 17 | development/pathology, polarity, **collective field**, per-polity holons, trade diffusion, genesis, meme genesis, norm proposals | multiple |
| 18 | kinship + household daily | social |
| 19 | motivation relieve, metrics history, event-buffer bound | infra |

**World:** a 2D grid with terrain, fertility fields, resource stocks and named sites
(well/farm/market/temple/houses), generated per-seed (meandering rivers, jittered site
grammar). Seasons drive fertility; disease transmits by proximity; a market clears
supply/demand endogenously; a black market activates under scarcity.

## 4. Subsystem depth — the ten substrates

| Layer | Contents | Status |
|---|---|---|
| **Biological** | 11 systems: genome, endocrine (6 axes), metabolism, cardiovascular, respiratory, musculoskeletal, nervous (pain/trauma), immune, reproductive, circadian, development | live |
| **Psychological** | 16 systems / 41 modules: interoception, cognitive runtime, emotion regulation (12 strategies), self-model, **theory of mind**, moral cognition (6 foundations), imagination/prospection, narrative identity, attachment, developmental, psychopathology, skill, motivation, cultural cognition, decision policy | live |
| **Social** | 20 modules: relationship_v2 (**20+ dimensions**), attraction (10 factors), courtship, marriage/pair-bond, kinship graph, household, clan, patronage, hierarchy, peer groups, cults, factions, status dimensions, epistemic trust | live |
| **Cultural / Noospheric** | memes (14 types, mutation), rumours v2, propaganda, rituals, collective memory, echo chambers, sacred values, education, narrative frames, ideology, knowledge diffusion, noospheric field, legitimacy field, moral panic | live |
| **Collective** | per-polity `CollectiveField` holons, territory-anchored genesis, cross-polity trade diffusion | live |
| **LOD cognition** | focal → secondary → background → dormant, with per-tick budgets (i348) | live |
| **Provenance** | 13 categories, full `DecisionTrace` audit | live |

## 5. How agents operate (measured)

Instrument: `sim::decision_census` (opt-in, inert when disabled). A decision is taken on
action completion (mean action duration **5.61 ticks**), so **~18% of agent-ticks are
decisions**.

| Deciding layer | N=12 | N=48 |
|---|---|---|
| Routine | **59.2%** | **54.4%** |
| Utility AI (deliberative) | **32.8%** | **38.1%** |
| Habit | 7.8% | 7.5% |
| Feud approach | 0.55% | 0.37% |
| Pain veto | 0.17% | 0.04% |
| Reflex | 0.08% | 0.08% |
| Command channel | 0% | 0% |

Action distribution (N=12, 20K): Work 39.7%, Rest 20.5%, Trade 15.0%, Eat 8.3%,
Drink 8.1%, Socialize 6.8%, Worship 1.7%, Wander 5.4%.

**Reading:** agency is *layered* and mostly *not* deliberative — about a third of
choices are genuine utility arbitrations; the rest is schedule and habit. That is
plausible for a pre-modern villager, but it means the "rational actor" surface is
smaller than the module count suggests.

## 6. Realism verdicts (dimension by dimension)

| Dimension | Verdict | Evidence / gap |
|---|---|---|
| **Agency** | moderate–good | 5-deep decision chain, bounded rationality under stress, intention commitment, inhibition. Gap: ~59% routine. |
| **Communion / attachment** | good | relationship_v2 20+ dims; attachment styles; tenderness→helping, loneliness→social-seeking, gratitude→help wired. |
| **Relationships** | good, one caveat | courtship→marriage→household→clan pipeline live; witness locality fixed (i349). **Caveat:** two relationship stores (legacy v1 matrix + honest v2) diverge materially — 63% of pairs >0.01 (i352). |
| **Innovation** | present, thin | §19.5.I work-driven discovery, knowledge diffusion (5 types), teaching, meme mutation, education, taboo-damped learning. Gap: not generative; skill saturation was audit finding E8. |
| **Thought processes** | strong | theory of mind, prospection/imagination, 12-proposition belief system, capacity-limited memory (8 kinds), narrative identity with scripts. |
| **Feelings** | strong | 22 discrete emotions, 12 regulation strategies, hedonic setpoint drift. E1 dysphoria plateau dead (i259); valence graded. |
| **Heredity / evolution** | real (Arc A) | genome blending, quantitative-genetics personality, surnames, moral/ideology/sacred transmission. Parent–child **r=0.81**, sibling **r=0.92**. Gap: `sensory_acuity` genome field has no consumer. |
| **Body–mind coupling** | real (Arc B) | interoception into decisions, Whitehall status→stress→health, sleep-debt withdrawal, injury→pain→blood-loss→shock (i313), immune clearance (i311). |
| **Culture / collective** | real (Era III–V) | emergent memes, rumours, rituals, collective memory, moral panics, legitimacy, echo chambers, per-polity genesis. |
| **World interaction** | good | 8 needs drive resource ops against a finite world; seasons, spoilage, disease, market pricing. **World area is decorative below N≈144** (i344) — most contact is co-residency. |

**Summary:** the engine is a genuinely deep simulation of individual minds and
small-group sociality — its strongest dimension. Its weaknesses are (a) the action
surface is routine-dominated, (b) locomotion was dead until i347/i351 and is still slow
(one step per decision), (c) the economy is concentration-prone with weak counter-forces,
and (d) the two-store relationship split makes some consumers read stale values.

## 7. Collective behaviour at scale

- **Factions** emerge from shared resentment + low institutional trust (i348: 6/6 pestilence seeds).
- **Moral panics** register, escalate, drain legitimacy, resolve (seed family {7,11,46}).
- **Revolutions** are regime changes, not repeat loops.
- **Multi-village (UM-3):** `auto_partition_polities` clusters inhabited home sites into disjoint polities; each gets its own collective holon, territory-anchored genesis memes, and cross-polity trade diffusion. Gate: **2 polities, 26 territory-routed memes, 26/26 cross-hosted** (origin-share 0.288).
- **Cults, feuds, patronage, peer groups, clans** all have live producers.

## 8. Scale ceiling (measured)

Per-tick cost, release, 2000 ticks, seed 42 (envelope re-baselined i332; further improved
by i352):

| N | µs/tick | ticks/sec | vs N=96 budget (6500) |
|---|---|---|---|
| 12 | 96.7 | ~10,300 | −6,403 |
| 48 | 531.4 | ~1,880 | −5,969 |
| 96 | 1,853.0 | ~540 | **71% headroom** |
| 144 | 4,157.9 | ~240 | **fits** |
| 192 | 7,700.8 | ~130 | breaches by **16%** |

Hard demographic cap: `MAX_POPULATION = 256`. World default fixed **32×32**; a
constant-density law (`side(N) = max(16, ceil(√(21.33·N)))`, i344) exists but is a
*fidelity* option, not a speed one. The rolling event buffer is **bounded** by i327
(amortized bulk drop, `MAX_EVENTS = 262_144`, peak ≈28 MiB), so **long horizons are
available now** — i354 measured a 250 000-tick village run at 55.3 s (N=12→36). The
`VecDeque` conversion stays deferred; its only remaining trigger is jitter-free ticks.

| World class | Feasible now? | Why |
|---|---|---|
| **Village** (10–50) | ✅ yes, comfortably — the calibrated regime | 540–10,300 tps |
| **Small town** (up to ~192–256) | ✅ yes, slowly | 130 tps @192; cap 256; LOD tiers engage |
| **City** (10³–10⁴) | ❌ no | needs the sparse relationship store (**demoted** by i338/i344/i350), multi-settlement orchestration, larger world; the Ω(N²) relationship floor is structural |
| **Country** | ❌ no | requires hierarchical polities (the per-polity holon is only a seed) + regional aggregation |
| **Planet** | ❌ no | requires a different scaling architecture (partitioning, LOD at every layer, distributed execution) |

**Verdict:** mindstrata today is an exquisitely detailed **village** simulator that
stretches to a **small town**. It is not yet a city/country/planet simulator.

## 9. Calibration debt (the honest ledger)

The live iteration ledger is [`PLAN_DC3_DEVELOPMENT.md`](PLAN_DC3_DEVELOPMENT.md) §3;
it is authoritative for *work*, this section is the summary.

1. **v1→v2 relationship dual-store** — architectural redundancy, **not a liveness fault**. i353 measured per-pair divergence (90% of pairs >0.01 at N=48/20K) but i356 measured the *equilibria*: both stores settle at 0.69/0.70 mean trust and **neither is dead** (v2 dead-row share 0.00%). The two stores model overlapping quantities with independent physics, so unifying them is a redesign (charter-level), not a migration. Consumers are split (appraisal/cognitive read v2; `social_cluster`/`economy`/`norms` read v1); the marriage pass is refuted as a target (fires only in the synced opening window).
2. **A9 — world area at constant density** — a charter/fidelity decision (co-location 19→4), not throughput.
3. **§17 tier-gate residual** — cosmetic rename + `runs_action_selection()` has zero call sites (i355 documented as recorded debt).
4. **Per-edge pass pacing** — ~58% of the tick; remaining reduction is behavioural (§17.3 dirty-window pacing).

**Queue additions:** the **wealth-tail progressive tax** (i358 — the distribution is bounded at ~0.65 and nobody is destitute, but `collect_taxes` is proportional and scale-invariant, so it cannot dent the one agent holding ~50% of the coin; fix is a surcharge above the membership median, behavioural + sweep-carrying) and **multi-settlement at N≥192** (the city-path experiment on the i344 density world law).

**Closed (do not re-open without new evidence):** `Idle` (i356 — the `Play` recreation driver made the last dead action live, 0.01%–3.67% of decisions), the A8 `Wander` driver (i351), the **locomotion-pace** premise (i357 — `Move` already steps one tile per tick; the limiters are the anger gate and co-location/A9), and the dual-store **migration** (i353/i356 — architectural redundancy, not a liveness fault).

**Deferred with triggers:** event `VecDeque` (>250K ticks), recent-claims index (same),
H5 founder-variance shaping (needs larger N + coordinated sweep).
**Vendor-blocked:** realms.md ontology, resonance attestation, cult-liveliness regime.

**Systemic debt register:** epidemic R0≈1 fragility (structurally mitigated), fatigue as a
village-wide synchronized phase, Background-tier in-vivo promotion pace, the dual-store split.

## 10. How to keep this document current

- Any iteration that changes a number in §1, §5, §7, or §8 **must** update this file in the
  same commit and bump `reconciled_commit` to the iteration's commit hash.
- Any doc that contradicts this one is stale: either fix it or mark it `SUPERSEDED` and
  point it here (see [`DOCUMENTATION.md`](DOCUMENTATION.md) for the taxonomy).
- `scripts/doc_index.py` (wired into `scripts/gate`) fails the build if this document is
  unlisted or its `reconciled_commit` does not resolve in git.
