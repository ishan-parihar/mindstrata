---
name: engine-status
description: "The authoritative current-state description of the mindstrata engine: architecture, tick orchestration, subsystem depth, agent behaviour, realism verdicts, and the measured scale ceiling. Load this before planning any iteration. Supersedes docs/MINDSTRATA_CURRENT_STATE.md (AP2-era) and docs/REMAINING_WORK_REPORT.md (Iter-134-era)."
type: Authority
status: AUTHORITY
scope: "engine behaviour, architecture, realism, scale"
reconciled_commit: 8a4ac31
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
| Full suite | `cargo test -p mindstrata-tests --lib --release` | **313 passed / 0 failed / 1 ignored** |
| Sim unit | `cargo test -p mindstrata-sim --lib --release` | **310 / 310** |
| Golden replay | `scripts/gate --full` | GREEN (both baselines byte-identical; last re-anchored at i384) |
| Probe law | `scripts/bench_index.py --strict` | 0 violations |
| Doc structure | `scripts/doc_index.py` | 71 governed docs classified, 0 ghosts |

**Scale of the artefact:** 13 crates · 464 `.rs` · ~148,900 LOC · 1,695 test functions
· 223 probes · 174 evidence docs · 36 RON spec files · 937 commits.

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

Instrument: `sim::decision_census` (opt-in, inert when disabled — verified identical
run folds with the census on and off). **Re-measured at i384**, probe
`i346_decision_census`, seed 42, 32×32, 20 000 ticks after a 500-tick warm-up:

| | N=12 | N=48 |
|---|---|---|
| decisions as share of agent-ticks | 23.2% | 20.8% |
| mean action duration | 4.31 ticks | 4.81 ticks |

| Deciding layer | N=12 | N=48 |
|---|---|---|
| Routine | **52.4%** | **52.2%** |
| Utility AI (deliberative) | **41.1%** | **40.2%** |
| Habit | 5.7% | 7.0% |
| Feud approach | 0.39% | 0.45% |
| Pain veto | 0.30% | 0.10% |
| Reflex | 0.06% | 0.08% |
| Command channel | 0% | 0% |

Action distribution (share of decisions, N=12 / N=48): Work 32.7 / 30.1, Trade 20.5 /
24.3, Rest 17.9 / 16.9, Eat 10.4 / 10.6, Drink 6.6 / 6.4, Socialize 4.9 / 5.1,
Wander 4.4 / 5.4, Idle 1.2 / 0.4, Worship 1.1 / 0.3, Move 0.39 / 0.45.

**Reading:** agency is *layered* — two fifths of choices are genuine utility
arbitrations, half is schedule, ~6–7% habit, and the crisis rungs (feud, veto, reflex)
fire rarely by design. The deliberative surface is **wider than the pre-i347 numbers this
table used to carry** (utility 32.8% → 41.1%), because `Wander`, `Idle` and `Move` — all
dead or near-dead before i347/i351/i356 — now enter the arbitration. **The `Wander` share's
old acceptance band was retired at i392b, and the driver now has a tested contract.** i351's
"0.5–3% of decisions at 20K" was an uncalibrated guess that had stopped being true — the table
above (*this* table, re-measured at i384) already carried Wander **4.4 / 5.4**, and nothing
asserted the claim because i351's instrument is a probe, not a test. Re-measured on the
**12-seed family** at i351's own config (**1.67–8.81%, mean 4.30%**, liveness 12/12), and the
re-contract was decided by the test the band's rationale names: **no Work displacement**
(seed 42 at i384's exact config: Work **33.88% vs 32.70%**, +1.18 pt at N=12; **29.34 vs
30.10**, −0.76 pt at N=48), so a coefficient change would have been a magnitude knob pulled
for nothing. The contract is now **liveness** (reaches the deliberative layer on every seed),
**gate exclusivity** (≥99% of wins inside the need-quietude window — measured 100%, so
exploration never outbids provisioning) and **boundedness** (≤12%, sized between the
calibrated family max 9.20% and the over-drive coefficient 3.0's 13.30%), pinned by
`exploration_driver_is_live_gated_and_bounded_across_the_seed_family` and **proven to trip**
on both halves. The magnitude is recorded rather than asserted: **3–76% of Wander's wins are
decided within the decision-noise amplitude** (mean 32%), which is recorded debt on the action
layer's margin, not something a threshold fixes. Behaviour unchanged — the coefficient is
untouched and both goldens stay byte-identical. Two caveats worth
stating plainly: `Socialize` is now reached by **both** layers — i387 decomposed the
arbitration bucket by bucket and i388 wired the two channels it lacked (the relational
urgency family: `Attachment\|Care\|Romance → Socialize`, `Belonging → Socialize\|Worship`;
and the population-relative goal band `2 × mean_social × goal_gate_scale`), taking
utility-selected `Socialize` from **0 → 109 / 20 121** (village) and **0 → 88 / 96 154**
(town) with the urgency bucket 0.0000 → 0.0865 and live goals 0 → 1/12 and 9/51 — share
**0.54% / 0.09%**, which is *measurement*, not a target: the winner's bar is 0.73–0.96
pressure-driven, and `Socialize`'s own term is small because `needs.social` is (p50 0.005)
— and the **directive channel is no longer nominal (i389)**: the council now speaks
(`systems/decree.rs`) — on a 250-tick cadence, when a crisis duty cycle holds
(famine/mean hunger > 0.6 → `Work`; panic/mean fear > 0.5 → `Worship`) and its mandate is
above 0.35, directing its four nearest villagers with priority `legitimacy × 0.8` as a
`GoalSource::Decree` that **decays** (the operator's `GoalSource::Command` is still the
durable one). Measured: calm **0.00%** (calm golden byte-identical), collapse **0.05%**
(30 selections). The pestilence leg was a measured **office VACANCY** — panic active
90.69% of ticks, Elder seat empty — and **i390 closed it**: `systems/succession.rs` fills
vacant seats from the living membership on a 50-tick cadence, by the institution's own
criterion (`status_v2.effective_status()`, ties on `AgentId`, no RNG stream). Pre-fix the
Elder seat was **100.00% vacant with a candidate pool present 100% of the time** (directive
0.0000%); post-fix vacancy is **0.00%** in all three worlds and the pestilence directive
share rises to **0.0654%** while **calm stays 0.0000%** (authority still silent when nothing
is wrong). Both goldens byte-identical — neither golden window contains an office-holder
death. Orphaned vacancies (a roster with no living member) are counted and left vacant:
recruitment is a separate root cause.
`Move`'s single producer is the §19.5.G feud approach, and its shipped gate
(`FEUD_APPROACH_ANGER = 0.02` + the hunger/thirst guard, branch ABOVE routine since i347)
opens on **0.0887% (N=12) / 0.1085% (N=48)** of agent-ticks over the full 20K window —
the probe leg that reported 0.0000% was measuring the retired `0.4` bar over a 2K window
and has been corrected.

## 6. Realism verdicts (dimension by dimension)

| Dimension | Verdict | Evidence / gap |
|---|---|---|
| **Agency** | moderate–good | 5-deep decision chain, bounded rationality under stress, intention commitment, inhibition. Gap: ~59% routine. |
| **Communion / attachment** | good | relationship_v2 20+ dims; attachment styles; tenderness→helping, loneliness→social-seeking, gratitude→help wired. |
| **Relationships** | good, one caveat | courtship→marriage→household→clan pipeline live; witness locality fixed (i349). **Caveat:** two relationship stores (legacy v1 matrix + honest v2) still coexist — 63% of pairs >0.01 (i352) — though the sites measured as consequential are migrating one per iteration (comfort i369, the v1 *writer* i376, the economy trade read+write pair i384); remaining readers: `norms_impl`, `household`, `births_deaths`. |
| **Innovation** | present, thin | §19.5.I work-driven discovery, knowledge diffusion (5 types), teaching, meme mutation, education, taboo-damped learning. Gap: not generative; skill saturation was audit finding E8. |
| **Thought processes** | strong | theory of mind, prospection/imagination, 12-proposition belief system, capacity-limited memory (8 kinds), narrative identity with scripts. |
| **Feelings** | strong | 22 discrete emotions, 12 regulation strategies, hedonic setpoint drift. E1 dysphoria plateau dead (i259); valence graded. |
| **Heredity / evolution** | real (Arc A) | genome blending, quantitative-genetics personality, surnames, moral/ideology/sacred transmission. Parent–child **r=0.81**, sibling **r=0.92**. Gap: `aggression_threshold` genome field has no consumer (`novelty_seeking`, `sensory_acuity` deliberately inert — measured-and-rejected, records in `i392_novelty_driver.md` / `i394_sensory_acuity.md`). |
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

Per-tick cost, release, 2000 ticks, seed 42, **density world law** `side(N) = max(16,
ceil(√(21.33·N)))` (charter method; envelope re-baselined i332, then expanded by the
housing spread i340/i345, the i352 scan removal, and i356 — re-measured at i359):

| N | side | µs/tick | ticks/sec | vs N=96 budget (6500) |
|---|---|---|---|---|
| 48 | 32 | 405.3 | ~2,470 | −6,095 |
| 96 | 46 | 987.3 | ~1,010 | **85% headroom** |
| 144 | 56 | 1,856.0 | ~540 | **fits** |
| 192 | 64 | **3,239.4** | ~310 | **fits, 50% headroom** |
| 256 | 74 | **5,805.2** | ~170 | **fits** (demographic cap) |

**World-area policy (A9 — DECIDED 2026-09-23, LANDED):** the world's area follows the
population at the engine's own calibrated density —
`world_side_for_population(n) = max(16, ceil(√(21.333·n)))`, **single-sourced in the world
generator** (`mindstrata-world::world_gen`, beside `houses_for_population` /
`cluster_count_for` / `MAX_RING_HOUSE_COUNT`) after seven stale copies across six probes and a
sim test; `SimConfig::for_population(…)` is the caller-side entry point. Because the law is
**anchored on the engine's own two calibrated points** (N=12 in 16×16 and N=48 in 32×32 *both*
give 21.33 cells/agent), it reproduces both **exactly** — the calibrated corpora are
byte-identical under either policy, and it bites only at N ≥ 96.

| | fixed 32×32 | density law |
|---|---|---|
| near-pair share (≤5 tiles), N=192 | 11.3% | **4.7%** |
| contacted-row share | 11.5% | **5.3%** |
| mean partners/agent | 22.1 | **10.2** |
| contacted α (144→192) | 1.276 | **0.866** |
| µs/tick, N=96 / 144 / 192 | — | **+9.1% / −2.3% / +5.6%** |

**Attribution correction (i392):** the `max co-location 19 → 4` result belongs to **A11**
(i345's spacing-aware area packing, one house per tile), not to A9 — i344 measured the 19 in
the **ring** world and queued A11 as a separate item. A9's own contribution is **contact
dilution** (the table above), and the two items are complementary. **Interim, not organic:**
this is still an input keyed on a chosen population; the target that retires it (carrying
capacity, causal space, encounter-driven contact, fission) is recorded in
`PLAN_DC5_DEVELOPMENT.md` §4 and `evidence/a9_density_policy.md` §5.

`ENVELOPE_EXPANDED_2_7X` — the N=96 charter budget covers **N=256**. Liveness holds at
the cap (i359: N=256 health 0.777, hunger 0.045, zero-coin 0, all partnered). Hard
demographic cap: `MAX_POPULATION = 256`. World size is operator-set via `SimConfig`
(default **16×16**; the charter/perf harness uses 32×32 at the N=48 tier; the density law
above drives the scale probes) — the density law (i344) is a *fidelity* option, not a
speed one. **Test-population policy (i368):** the budget is a **ceiling**, not a test
mandate — every test/probe runs **the smallest N that exhibits its phenomenon**, with the
default matrix **{12, 48, 144}** and N=256 reserved as the town-scale stress tier. The rolling event buffer is **bounded** by i327
(amortized bulk drop, `MAX_EVENTS = 262_144`, peak ≈28 MiB), so **long horizons are
available now** — i354 measured a 250 000-tick village run at 55.3 s (N=12→36). The
`VecDeque` conversion stays deferred; its only remaining trigger is jitter-free ticks.

| World class | Feasible now? | Why |
|---|---|---|
| **Village** (10–50) | ✅ yes, comfortably — the calibrated regime | 2,470 tps @48 |
| **Small town** (up to ~256) | ✅ yes | **310 tps @192, 170 tps @256**; cap 256; capacity is no longer the blocker |
| **Multi-settlement town** | ✅ yes | i360 landed the **clustered world generator** (above 24 houses the world places `clamp(houses/16, 1, 4)` village centres → N=192 **3** / N=256 **4** settlements at the natural gap, was 1), each with its own live polity holon; i362 measured the stack **end-to-end live** — **100% of genesis memes leak across a village boundary** via cross-polity trade (12/12 at N=192, 20/20 at N=256). Calibrated range (<25 houses) byte-identical |
| **City** (10³–10⁴) | ❌ no | needs the sparse relationship store (**demoted** by i338/i344/i350), multi-settlement orchestration, larger world; the Ω(N²) relationship floor is structural |
| **Country** | ❌ no | requires hierarchical polities (the per-polity holon is only a seed) + regional aggregation |
| **Planet** | ❌ no | requires a different scaling architecture (partitioning, LOD at every layer, distributed execution) |

**Verdict:** mindstrata today is an exquisitely detailed simulator that runs a
**256-agent, multi-settlement town** (i359 capacity + i360 structure) — what it lacks
beyond that is the city/country/planet architecture (the Ω(N²) relationship floor is
structural, and hierarchical polities are only a seed).

## 9. Calibration debt (the honest ledger)

**Office vacancy (i389 → CLOSED i390):** the Elder seat was never refilled when its holder
died, so the authority channel was structurally dark in mortality worlds. `systems/succession.rs`
now fills vacant seats from the living membership (measured: pestilence vacancy 100.00% →
**0.00%**, directive share 0.0000% → **0.0654%**, calm still 0.0000%; both goldens
byte-identical). **Residual (recorded, not fixed):** an institution whose members have ALL
died stays vacant — that is recruitment, a different mechanism — and the long-vacancy
institutional effects (treasury accumulating with no holder to spend it, legitimacy drift)
are unmeasured. It would have blocked the inter-community layer's offices (W3).


**Dead genes (i391 census; rows 1 and 3 CLOSED, rows 2 and 4 REJECTED, row 5 open):**
`aggression_threshold`, `novelty_seeking`, `chronic_pain_risk`, `sensory_acuity` and
`puberty_age` were drawn at founder creation, defaulted and blended at `inherit`, and **read by
nothing outside `genome.rs`** — heritable trait variation that changed no behaviour.
**`puberty_age` is wired (i392 row 1):** the reproductive clock reads
`ReproductiveUpdateParams.puberty_age` from the genome, with the retired constant as the
default (13.0 = the gene's midpoint, so the wiring is midpoint-neutral). Its measured scope is
honest and narrow: **0 agent-ticks in age [10,16) over 240K + 987K agent-ticks** (founders
≥18, the ramp opens at ~385 000 ticks), so the fix is invisible in every corpus — both goldens
byte-identical, 0 re-anchors — and its target is the next generation at long horizons.
**`sensory_acuity` was measured against its class-consistent consumer and rejected (i392 row 4):**
the census-named consumer (the i334 perception radius) is an integer staircase — disqualified
before probing. The attention system's hardcoded `salience_bias = 0.5` (numerically the gene
draw's mean, U(0.2,0.8) → 0.5) was wired and probed instead: the anchor was exact (pinned-0.5
reproduced both goldens byte for byte), but the **encode threshold (0.2) sits inside the
gene's draw range** — fresh own-help salience ≈ 0.216 rides the band edge, so memory occupancy
spread 13× across the gene range (a threshold lottery, not a gradient), and the revolution
liveness family collapsed to 1/3 via shared Behavior-stream re-timing. Reverted per the row-2
precedent (`i394_sensory_acuity.md`); the trait stays deliberately inert until a consumer
exists with no acceptance band inside the draw range.
**`chronic_pain_risk` is wired (i392 row 3, `i393_chronic_pain_gene.md`):** it now scales
the skeletal accumulation law through `SkeletalUpdateParams` (the row-1 params-struct
pattern). Both §4.15 gates cleared: the band it governs is **closed in every calibrated
corpus** (2.29 M agent-ticks, five corpora including pestilence and collapse — severe injury
never occurs, so the wiring is invisible to goldens *by construction*, recorded as
measured-scope), and the response is **exactly linear** (±40% gene ⇒ ±40% state — the
proportional shape row 2's elastic consumer lacked). Anchored at the **draw midpoint 0.25**,
not the gene's `Default` 0.2 — the Default anchor would have baked a +8% population drift
into every future injury world. Same-row finding: the `organs.ron` spec field
`chronic_pain_accumulation_rate` is **inert** — it declares 0.001, which cannot work at the
state's 1e-4 resolution (5e-5 < quantum), the operative 0.005 was correct, and the
misattributing comment is now a documented divergence (spec debt in `PLAN_DC5`).
**`novelty_seeking` was measured against its named consumer and rejected (i392 row 2):** the
wiring onto the i351 exploration driver was live and monotone (pinned-gene A/B 132 → 2 464 →
3 927 Wander decisions) and midpoint neutrality was **proven at machine precision** (the
gene-pinned-0.5 control reproduced the stored golden `metric_hash` `0xb2a1be3b18fbb46d` byte
for byte; the natural-gene divergence began at tick 657 for agent 0 alone), but the surface's
response is **elastic (≈4)** — so aggregate neutrality hid per-agent redistribution, and the
**revolution liveness family fell to 1 of 3 seeds** (a dead producer, which §2.3 forbids
re-pinning). The candidate was reverted, not re-anchored. The rejection exposed the act's
**precondition (i392b, new)**: the exploration driver's own acceptance band is stale —
`i351_wander_bands` reads **5.44% (s42) / 3.53% (s7)** of decisions at 20K against the
documented **0.5–3%** target, and *no test pins it* (the instrument is a probe), so the claim
the coefficient was chosen for has drifted unenforced. Rows 3–5 wait for that band to be
re-contracted or re-calibrated, and each must clear both gates: the shadowed constant sits at
the gene's midpoint **and** the response to that constant is proportional (doctrine §4.15).
The instrument (`scripts/field_census.py`, doctrine §5) is recorded for re-use; it found six
suspects and one is the documented false-positive class (an aggregate consumed by its own
file).

The live iteration ledger is [`PLAN_DC5_DEVELOPMENT.md`](PLAN_DC5_DEVELOPMENT.md) §3;
it is authoritative for *work*, this section is the summary.

1. **v1→v2 relationship migration (charter DECIDED: finish it subsystem by subsystem)** — i353 measured per-pair divergence (90% of pairs >0.01 at N=48/20K). **i369 landed the first migration:** the comfort/soothing path now reads the dyadic v2 store (three-way runnable pin; collapse golden one birth flipped 13→12, in-contract). Remaining readers queued: norms_impl, household, births_deaths; marriage pass REFUTED (i353); the cognitive mean-reversion writer landed (i376). **i384 migrated the economy trade pair (read + write)** — the last self-contained v1 pair — after its probe measured the divergence at 6–40× the population average on exactly the pairs that trade (v1 0.9174/0.8068/0.7892 vs v2 0.8242/0.7036/0.6551) and a 2.8–4.0% price error on the busiest dyads; post-migration the read-source delta is 0.34–0.52%, traded-pair v2 trust sits at 0.93–0.98, and `trade_price_reads_the_dyadic_store_and_writes_it` guards the store choice. The remaining v1 writers' gain schedule (speech-act model + `RelationshipKind` ladder) is the next migration commit.
2. **A9 — world area at constant density** — a charter/fidelity decision (co-location 19→4), not throughput.
3. **§17 tier-gate residual — RESOLVED (i372): the dead predicates are DELETED.** `runs_full_biology()`/`runs_action_selection()` had zero call sites (i316), the whole gate payoff was ≈5% (i328), and the Background tier is dark in calm towns (i367). The *cognitive* rungs stay wired and real.
4. **Per-edge pass pacing** — ~58% of the tick; remaining reduction is behavioural (§17.3 dirty-window pacing).

**Queue additions:** **organic-mechanisms arc (i373 audit + i374/i375 landings)** — hardcodedness classified into organic candidates vs realism-preserving constants (doctrine §4.6); the legitimacy-coupled dividend `s = s0 + k·(1−legit)` (i374) and status-scaled patronage capacity `cap = 3 + floor(status×4)` (i375) are LANDED, and the **gate selectivity audit (i379)** added the third method — measure the share of samples that OPEN each shipped gate over a distribution × context grid — settling four classes: **quantile gates over the fixed founder draw are robust by construction** (trait thresholds open 31–54%, stable across calm/crisis × village/town) and must stay hardcoded; **crisis gates on bounded need scales are dark by design** (`hunger > 0.85` opens only in famine); **gain/scale mismatches** are the class that hides behind passing tests — and i380 split them: `needs.social × social_value` is not merely small but **inert** (the utility leg selected `Socialize` **0 times at N=12 and 1 at N=48** across 112 669 arbitrations; every social decision comes from the routine), and neither a ×6 gain nor a ×20 need accrual revives it, because the blocking quantity is the **arbitration bar** (measured winner utility 0.692 mean) rather than the term's scale — so the repair belongs to the channels its siblings have and it lacks. **That decomposition is now LANDED (i387) and its verdict acted on (i388):** the utility is a labelled bucket vector and the census records the winner's, the runner-up's and the `Socialize` candidate's vectors per arbitration — the candidate's one big term (`policy` 0.31) is nearly the winner's (0.38), so it was never under-weighted; the deficit sat in buckets it had **no channel for** (`urgency` 0.053/0.066, `goal` 0.062/0.080, `trade` 0.109/0.188, `driver` 0.091/0.095), and the dominant motive is relational in **40.1%** of calm-village arbitrations (`Attachment` 34.6% + `Belonging` 5.5%) while the §8.1.5 map sent every one through `_ => false` — the dead-dominant-motive class i351/i356 closed for `Novelty`/`Play`, at twice their size. i388 closed both channels (relational urgency family + the relative band `2 × mean_social × goal_gate_scale`, the i382/i383 self-normalizing form; the three producers' absolute bars 0.7/0.4/0.3 all sat above the channel's own p99 0.032–0.039 and were dark by construction). Blast radius attributed and re-anchored with evidence: both goldens (`agent_count` 12 → 12), 7 snapshots reviewed, and three pins re-contracted — the pain veto band from a single-seed max to reachability over a family (i319 probe extended: family max 1.0000, **2/12** seeds at 20K), the revolution family rediscovered as **{5, 42, 12345}** (7/5/3 revolutions; producer stronger, re-timed), and the tenderness Help pin from a single-seed ±10% magnitude to the mechanism over a family (+15.2%). **Recorded debt: the revolution family is knife-edge** ({5,11,42} → {42,7,23} → {5,42,12345} over three pacing changes). **Measured, unexplained (carried to W2):** action duration rose (4.31 → 5.40 ticks/decision) and the 2K relationship-stage distribution shifted shallower (`Unnoticed` 20 → 26, `Friend` 8 → 5) — hypothesis: serving the drive selects stay-put social actions *instead of* movement, so encounter volume falls as social selection rises; the sparse-store contact work must measure this before it is interpretable. The queue now leads with **absolute thresholds on self-driven aggregates** drift (`council legitimacy < 0.50` — which **i382 resolved**: the audit's 0.0% reading was one seed's sample; across 15 worlds the bar opened 0.00% of ticks in **8** and 0.07–4.26% in the other **7**, against a 0.54–0.58 mandate equilibrium, and it fed **both** the instant cliff arm and the i240 accumulator's deficit, so the whole legitimacy channel was seed-decided. Both sites now read a baseline the council's own history established (`LEGITIMACY_DEBT_TAU_TICKS = 100` derived from the 0.01/tick convergence rate, `LEGITIMACY_COLLAPSE_MARGIN = 0.05` = the offset the deleted bar measured empirically, read before the fold). Liveness went from **0.0000 pressure units in 8/15 worlds to 0.021–0.233 in 15/15**; formations 8 → 12 with every collapse-only one preserved; **zero calibrated blast radius** — goldens byte-identical, no pin re-anchored). **The remaining emotion-side item then closed (i383):** `emotions.anger > 0.50` — flagged by the same census — was measured opening for **0.00–0.30% of agent-ticks** at **both** of its sites (the intention-abandonment shock, ORed with a live fear arm, and the anger→Work "aggressive productivity" emitter, where anger is the only gate and so the producer was dead), while `fear > 0.5` beside it opened **15.2–38.8%** and needed no change. Both anger arms now read **1.25 × the population's own mean anger** (`EMOTION_SHOCK_RATIO`, the i381 anomaly multiple; the emitter's priority is the excess over that bar), which measures **3.1–19.6% open in every world** and lifts anger-sourced Work goals from **0 in 7 of 10 worlds to 0.020–0.184 per agent-tick in all 10**. One collapse golden re-anchored (mortality unchanged at 12); calm golden byte-identical, no snapshot drift. The queue continues with endogenous tax policy (council sets its own rate — biggest sweep), geography-derived marriage distance, and trait-derived disposition thresholds. **The panic-trigger redesign that the audit's Class-4 ruling demanded is now LANDED (i381):** the §7.2 trigger fires on `avg_charge ≥ max(population_baseline × 1.25, 0.47) AND panic_ratio ≥ 0.30` — the floor is the geometric midpoint of the measured decision gap between the highest must-not-fire crisis plateau (0.4188) and the lowest must-fire spike (0.5475), and the per-proposition baseline is a slow EWMA (τ = 10 × the 300-tick panic cadence) that adopts its first observation on a cold start. i378's knife-edge seed now clears the bar by 16% instead of missing it by 0.5%, and the relative arm gives the form a *memory*: a population that warms into a sustained level fires a burst and then goes quiet (a panic is an event, not a new equilibrium), and a chronically charged population has no sudden collapse to have. Two contracts re-anchored with attribution (attachment separation distress re-contracted to a 2/3 population-wide supermajority; the revolution family discovered as `{42,7,23}` by a 10-seed pestilence sweep at 70K) — goldens and snapshots byte-identical, the trigger's first in-corpus fire sitting at tick 3014, past every calibrated window. **i376 closed the v1→v2 divergence source:** the legacy `relationships` trust row was saturating (mean 0.887–0.920, 79–85% of pairs ≥0.95 at 50K) because its 0.001/day mean reversion lost to the interaction gains by ~14×, while the dyadic store held 0.588–0.628 — so the two disagreed by 0.27–0.31 mean across 55–63% of pairs and every v1 trust gate (`memory_ops` encoding, speech-act credibility, marriage/economy partner trust, norm enforcement) read a dead signal. The row now converges onto its dyadic counterpart (`× relationship_dormant_decay` daily, default 1.0 = full sync); measure the trust-derived surfaces against the differentiated store from here. **Closed:** the treasury-ceiling item (superseded — i370's office-based council bounds the hoard at 182–350 @50K) and the cluster-divisor sweep (superseded by settlement fission).

**Closed (do not re-open without new evidence):** the **cluster-cap raise** (i366 — never binds below N=320; the divisor is the knob), the **collapse-resilience concern** (i364 — A/B shows the cascade kills 2 agents on both builds; `agent_count` 12→13 is a birth), the **wealth-tail fix** (i363 — council surplus dividend: Gini 0.647→0.611, hoard 163 602→241, `WEALTH_TAIL_BENT`; the i358-scoped progressive tax was refuted by i361), the **clustered world generator** (i360 — landed: N=192→3, N=256→4 settlements, per-polity holons live, calibrated range byte-identical), `Idle` (i356 — the `Play` recreation driver made the last dead action live, 0.01%–3.67% of decisions), the A8 `Wander` driver (i351), the **locomotion-pace** premise (i357 — `Move` already steps one tile per tick; the limiters are the anger gate and co-location/A9), and the dual-store **migration** (i353/i356 — architectural redundancy, not a liveness fault).

**Deferred with triggers:** event `VecDeque` (>250K ticks), recent-claims index (same),
H5 founder-variance shaping (needs larger N + coordinated sweep).
**Vendor-blocked:** realms.md ontology, resonance attestation, cult-liveliness regime.

**Systemic debt register:** epidemic R0≈1 fragility (structurally mitigated), fatigue as a
village-wide synchronized phase, the revolution-liveness family (knife-edge: three
re-anchors by discovery across three pacing changes — the fix is a per-seed route pin,
not another sweep), deliberative sociality vs encounter volume (i388 measured action
duration 4.31 → 5.40 ticks and a shallower 2K relationship-stage distribution — W2 must
decide whether contact falls when the drive is served), the dual-store split (both halves are now closed at the sites measured: i376 converged the
v1 row onto the dyadic store, and **i384 migrated the economy trade read+write pair** — the
last self-contained v1 pair — where the two stores sat +0.09…+0.13 apart on exactly the
pairs that trade, 6–40× the population-wide offset, so trade now reads and writes one
store and its price no longer discounts by a stale scalar; the residual is the v1
interaction-gain schedule itself (+0.02…+0.10/act, ~10× the dyadic gain) whose removal is a
subsystem migration — the speech-act effect model and the v1 `RelationshipKind` ladder must
move together — plus the three remaining readers `norms_impl`, `household`,
`births_deaths`),
the moral-panic trigger's knife-edge (i378 sized it, **i381 resolved it** with the
RELATIVE/anomaly form — `avg_charge ≥ max(population_baseline × 1.25, 0.47) AND
panic_ratio ≥ 0.30` — and its pin fragility is structural too: both panic tests hold a
fixed 10-seed family and discover its firing members, so a pacing shift moves which member
carries the downstream legs instead of renaming the family),
the proportional council dividend's residual hoard (RESOLVED by i370 — offices, not rosters: the hoard equilibrium is bounded by the 3-member tax base; i374's legitimacy coupling additionally makes a resented council spend more),
and the LOD tier being **dark in calm towns** (i367: Background 0.0 % of agent-ticks at N≥96 calm, 7.5–13.9 % under crisis — a crisis modulator, not a scale lever).

## 10. How to keep this document current

- Any iteration that changes a number in §1, §5, §7, or §8 **must** update this file in the
  same commit and bump `reconciled_commit` to the iteration's commit hash.
- Any doc that contradicts this one is stale: either fix it or mark it `SUPERSEDED` and
  point it here (see [`DOCUMENTATION.md`](DOCUMENTATION.md) for the taxonomy).
- `scripts/doc_index.py` (wired into `scripts/gate`) fails the build if this document is
  unlisted or its `reconciled_commit` does not resolve in git. It checks **structure, not
  citations** — a `file:line` here is a pointer, not a contract: re-point it whenever the
  file moves (AGENTS.md §4.14), and verify a "not landed" claim by opening the file before
  planning work off it (i386 found two balance docs reporting live levers as unlanded).
