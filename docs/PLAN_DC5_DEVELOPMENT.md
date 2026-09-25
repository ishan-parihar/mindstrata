---
name: plan-dc5
status: ACTIVE
description: "DC-5 implementation plan — depth closure and inter-community scale. Owns the gap taxonomy (G1–G8) derived from measured shallowness in the engine, the phase/iteration ladder that closes it, and the operator decisions it depends on. Authoritative for what remains after DC-4; ENGINE_STATUS.md stays authoritative for engine truth."
type: Plan
reconciled_commit: a09cfbe
created: 2026-09-23
---

# Mindstrata — DC-5: depth closure and inter-community scale

> **What this document is.** The DC-3 ledger (`PLAN_DC3_DEVELOPMENT.md`) closed DC-3 and
> carried DC-4's ledger. This is the **next cycle's plan**: it takes the gaps that the
> engine's own audits, probes and status doc already measured, classifies them, and turns
> them into a probe-first iteration ladder with named exit criteria.
>
> **What it is not.** Not a rewrite of `ENGINE_STATUS.md` (engine truth), not a re-litigation
> of closed findings (`ENGINE_STATUS.md` §9's "closed (do not re-open)" list stands), and not
> a licence for magnitude knobs: every item below is probe-first, §4.4-compliant, and carries
> its own re-anchor evidence rule.
>
> **Frame.** The engine is deep where it is *individual* and *small-group* (agency layers,
> 22 emotions, theory of mind, heredity r=0.81/0.92, culture, panics, revolutions) and thin
> where it is *collective*, *spatial* and *inter-community*. DC-3 proved the world can hold a
> multi-settlement town; DC-5's job is to make the town's parts actually interact, and to
> close the small set of dead-or-inert surfaces that a probe has already named.

---

## 1. Evidence base (measured, not remembered)

Everything below was re-verified at `f1beca0` for this plan; the numbers are the plan's
premises, and any iteration that moves one must move this file (§ENGINE_STATUS §10 rule).

| Fact | Value | Source |
|---|---|---|
| Scale envelope | N=256 @ ~170 tps, N=192 @ ~310 tps, cap `MAX_POPULATION = 256` | `ENGINE_STATUS.md` §8 (i359) |
| Tick cost at N=192 | **4 028 µs/tick**; cognitive 21.3%, memory 17.8%, appraisal 9.0%, trust_sync 8.8%, derived+belief 8.3%, social_pass 6.8%, rel_traces 6.0%, kinship_daily 3.9%, social_cluster 3.7%, biology 3.5% | re-measured today, `i330_pass_profile -- 192 400 200` |
| Decision surface | routine **52.4%**, utility AI **41.1%**, habit 5.7%; decisions on 23.2% of agent-ticks at 4.31 ticks/action | `ENGINE_STATUS.md` §5 (i385) |
| `Socialize` | **0 utility selections in 112 669 arbitrations** at N=12, 1 at N=48 — routine-only; `needs.social` inert to ×6 gain and ×20 accrual | i380 |
| Directive channel | `GoalSource::Command` has an injection API (`sim/api.rs:79`) and 4 tests, **zero in-sim producers** | `ENGINE_STATUS.md` §5 caveat; grep today |
| Inter-community layer | diplomacy runs on **3 fixed off-map neighbors** (`Riverside`/`Millbrook`/`Stonegate`), 4320-tick cadence, ~1%/neighbor/pass — while the *world* holds 3–4 on-map polities with per-polity holons | `institutions/src/diplomacy.rs` (i150); i297/i298/i360 |
| Clan structure | **pinned at exactly 3 clans at every horizon and every seed** (100% of ticks ≥2) | i323 measurement |
| Relationship store | v1 matrix + dyadic v2 coexist; **3 readers left** (`norms_impl`, `household`, `births_deaths`) + the speech-act effect model and v1 `RelationshipKind` ladder | i384 |
| Contact | world area **decorative below N≈144** — most contact is co-residency; touched-R saturates 13.3%→73.7% by 40K | i344, i350 |
| LOD tier | `Background` dark in calm towns (0.0% of agent-ticks at N≥96); forced-all-Background payoff −3.6…−5.3% | i367, i328 |
| Dead genome fields | **act i392 CLOSED — no inert gene left unwired-by-accident.** Wired: `puberty_age` (row 1), `chronic_pain_risk` (row 3). Measured-and-rejected, kept deliberately inert with records: `novelty_seeking` (row 2), `sensory_acuity` (row 4), `aggression_threshold` (row 5) | `scripts/field_census.py`; `ENGINE_STATUS.md` §6 |
| Exploration-driver band | **stale band RETIRED (i392b)**: measured **1.67–8.81%, mean 4.30%** over the 12-seed family at i351's config, against a documented 0.5–3% target that the doc's own i384 table had already breached (Wander 4.4/5.4). No displacement (Work +1.18/−0.76 pt), so re-contracted: **liveness + gate exclusivity + ≤12%**, pinned by test and proven to trip | `i392b_wander_band`; `exploration_driver_is_live_gated_and_bounded_across_the_seed_family` |
| Wander's decision margin | **noise-decided in 3–76% of wins** across the family (mean 32%) — the driver's volume is defensible, its determinism at the margin is not | `i392b_wander_band` leg D |
| Consumer response elasticity | the exploration coefficient's response is **elasticity ≈ 4** (1.92 → 2.16 buys 1 853 → 3 256 Wander wins; the gene's full range is a 4.1× spread) while the population-mean multiplier stays within ±5% of 1.0 — so *aggregate* neutrality hides *per-agent* redistribution | `i392_novelty_driver` leg D |
| Skill curve | farming pinned 1.00 population-wide, trading 0.00 (audit E8) — **last measured 2026-08-22, needs re-probe** | `AUDIT_2026-08-22` E8 |
| Fatigue | village-wide synchronized phase; within-seed mean swings 0.05↔0.41 | i321 |
| Wealth | Gini 0.647 → 0.611 (council dividend); endogenous tax not started; i377 constraint: **do not** couple rate to legitimacy | i363, i373 #2, i377 |
| Town-tier throughput (2026-09-25 A/B) | **attribution RESOLVED — host-state spikes, not code**: load-pinned `558538d` vs HEAD interleave (3 reps/arm) reads head/old **1.00–1.17** at every uncontaminated position (mean ≈1.08 at town tiers); yesterday's 1.85×–3.24× readings were samples of a spike state the head arm enters 2-of-3 runs (7,270 @192 / 21,990 @256) and the old arm never (spread ≤3%). Residual ≈8–9% arc+world-law delta (not attributable to any single iteration). **Open finding: the spike state itself** — i423 must record its frequency, not just quiet medians | `evidence/i423_ab_town_scale_attribution.md` |
| Founder shape | uniform draws **load-bearing at N=12**; shaping needs larger N + coordinated sweep | AGENTS §5 (Iter-263) |
| Doc enforcement | `doc_index.py` checks structure, **not citations** (i386 found 5 drift classes green-lighted) | i386 |

---

## 2. The gap taxonomy (G1–G8)

Ordered by *realism yield per unit of blast radius*, with the verdict class each item belongs
to (the i379 vocabulary: **A** already organic · **B** organic candidate · **C** realism-
preserving · **D** dead/inert surface · **E** architectural).

| # | Gap | Class | Measured symptom | Cost |
|---|---|---|---|---|
| **G1** | **Action surface is routine-dominated, with two dead/inert surfaces** — `needs.social` never wins an arbitration; the directive channel has no in-sim producer | D+B | Socialize 0/112 669 utility wins; Command 0.0% of decisions | S–M |
| **G2** | **Two relationship stores** | E | 3 readers + the writer's gain schedule still on v1; 17–25% of pairs >0.05 apart | M |
| **G3** | **Space is decorative; contact is co-residency; locomotion is one step/decision; clan count fixed at 3** | E+B | area law only bites ≥N≈144; 3 clans forever; `Move` reachable only through feud approach | M–L |
| **G4** | **The inter-community layer is off-map and disconnected from the on-map polity graph** — the single largest architectural gap for any scale beyond a town | E | 3 fixed abstract neighbors vs 3–4 real polities that already share trade, memes and territory | L |
| **G5** | **No hierarchical polity layer** — per-polity holon is a seed; no regional aggregation, no treaties/federation, no inter-settlement war | E | i296/i297 landed single-order polities only | L |
| **G6** | **Scale architecture: the top of the envelope is superlinear and the LOD tier cannot carry a background population** | E | cognitive+memory = 39% of tick at N=192; Background dark in calm worlds; cap 256 | M–L |
| **G7** | **Thin deep layers** — skill curve saturates (E8), fatigue lacks per-agent differentiation, innovation is not generative, one genome field is dead | B+D | farming 1.00 / trading 0.00; synchronized fatigue; tier-2 technology chains rare | M |
| **G8** | **Verification surfaces have holes** — citation drift is unchecked; longitudinal charts partial; no standing envelope beyond N=96 | E | i386 found 5 drift classes with the gate green | S |

**Priority call:** G4+G5 are the *architectural* shallowness the operator asked about — the
engine simulates a town as if it were a village with extra houses. G1/G2/G8 are cheap, high-
confidence closures that remove dead surfaces and stale readers. G3/G6 are the scale
preconditions. G7 is depth polish with the best narrative payoff.

---

## 3. Workstreams

Each iteration owns **one root cause** (§2.1), writes its probe **first** (`<iter>_*.rs`),
runs `gate --full` before push, and lands with an evidence doc. Measurement iterations are
marked **[M]** — they change no behaviour, so they carry no re-anchor risk and can be
batched tightly.

### W0 — Close the named dead/inert surfaces (cheap, unblocks the action layer)
| iter | root cause | probe | exit criterion |
|---|---|---|---|
| **i387** [M] | **DONE** (`i387_utility_decomposition.md`) | the utility is a labelled bucket vector + the census records winner/runner-up/`Socialize` per arbitration | named the missing channels: `urgency`, `goal`, `trade`, `driver` at 0.000 on the social candidate while `policy` (its own big term) was already 0.31 vs the winner's 0.38; `Attachment`+`Belonging` = 40.1% of dominances |
| **i388** | **DONE** (`i388_relational_outlet.md`) — revivable, and the channel was the problem: the relational urgency family (`Attachment\|Care\|Romance → Socialize`, `Belonging → Socialize\|Worship`) + the population-relative goal band (`2 × mean_social × goal_gate_scale`, retain at ×0.43) | `i388_relational_outlet` (reach + ratio sweep, pre/post census) | channel live: `Socialize` urgency 0.0000 → **0.0865**, utility-selected `Socialize` **0 → 109** (village) / **0 → 88** (town), live goals 0 → 1/12 and 9/51. **Exit criterion re-contracted**: utility share measured **0.54% (village) / 0.09% (town)** against the pre-measurement guess of ≥1% — a 1% share needs a band wide enough to stop discriminating (§4.10), and the winner's bar is 0.73–0.96 pressure-driven. The pinned invariant is *the drive reaches the deliberative layers*, not a share |
| **i389** | **DONE** (`i389_command_channel.md`) — authority was nominal; `systems/decree.rs` gives the council a bounded, legitimacy-scaled ask (`GoalSource::Decree`: a directive that DECAYS, unlike the operator's durable `Command`), crisis-keyed (`famine`/hunger → Work, `panic`/fear → Worship) and capped at the four nearest villagers | `i389_command_channel` (producer census + per-tick crisis duty cycles, pre/post) | **calm 0.00% (0 selections, and the calm golden is byte-identical)** ✓, **collapse 0.05%** (30 selections, 181 decree agent-ticks) ✓. The pestilence leg measures **0.00% for a measured reason, not a dark producer**: its panic duty cycle is **90.69%** and its Elder seat is **EMPTY** — after a pestilence the council has no office holder and nothing appoints a successor. Succession is recorded as the next root cause (one per iteration); the producer's conditional liveness is pinned with the seat filled |
| **i390** | **DONE** (`i390_office_succession.md`) — the lifecycle gap i389 exposed: `handle_agent_death` nulls every office its holder held and nothing ever refilled it, so the pestilence council had **no office holder at all** (authority structurally absent while members/treasury/legitimacy survived). `systems/succession.rs` fills vacant seats from the living membership on a 50-tick cadence, by the institution's own criterion (`status_v2.effective_status()`, ties on `AgentId`, no RNG stream) | `i390_office_succession` (vacancy share + candidate-pool share + directive share, pass enabled vs disabled) | pre-fix pestilence Elder seat **100.00% vacant with a pool present 100% of the time**, directive **0.0000%**; post-fix **0.00% / 0.00% / 0.00%** and directive **0.0654%** while **calm stays 0.0000%** (authority still silent when nothing is wrong) ✓ — **both goldens byte-identical** (no office-holder death in either golden window). Orphaned vacancies (no members) counted at 0 and left vacant: recruitment is a separate root cause |
| **i391** [M] | **DONE** (`i391_dead_field_sweep.md`) — the recurring write-only class (i313 injury, i124 emotion-context, i307 habit gate) is now found by census. `scripts/field_census.py` counts writes/literals/reads per field and treats a read inside the field's OWN file as plumbing, so the instrument's key is the **cross-module read** | `scripts/field_census.py --file <defs> [--sites]` | **6 suspects, 5 confirmed dead genes**: `aggression_threshold`, `novelty_seeking`, `chronic_pain_risk`, `sensory_acuity`, `puberty_age` — each drawn at `random()`, defaulted, blended at `inherit`, and read by nothing outside `genome.rs`; the 6th (`EmbodiedState.metabolic`) is the instrument's documented false-positive class (a consumer in its own file). The remaining 53 fields have consumer reads — the class is five genes, not systemic rot. Script recorded for re-use ✓ |
| **i392** | The five dead genes are **false affordances**: the engine carries heritable trait variation that changes no behaviour (and the H5 distribution debt is measured over inert draws). Each is a behavioural change, so each gets its own probe inside one act | per-gene probes; `puberty_age` first (its consumer **already exists and was hardcoded over the gene** — `reproductive.rs:168` read a village-wide `13.0` while the gene draws 11.0–15.0) | every row wired midpoint-neutrally (§4.6) **and** cleared against the response-elasticity gate i392b measured (§7 of `i392_novelty_driver.md`: a consumer whose response to its constant is elastic turns a heritable spread into a population-level lottery), or deleted; birth timing / violence / perception paths re-anchored with attribution; no inert gene left |
| **i392 · row 1** | **DONE** (`i392_puberty_clock.md`) — the clock reads `ReproductiveUpdateParams.puberty_age`, fed from `genome.fertility_predispositions.puberty_age`; the default is the retired constant (13.0 = the gene's midpoint), so the wiring is **midpoint-neutral by construction** | `i392_puberty_clock` (gene distribution, band occupancy, per-agent manipulation check) | measured: gene spread 3.3–3.9 yr across 3 worlds; **0 agent-ticks in age [10,16) over 240K + 987K agent-ticks** (founders ≥18, the ramp opens at ~385K ticks) so the fix is invisible in every corpus; live at the clock (12-yr-old with gene 11.0 → `Early`/0.1428 vs gene 15.0 → `Prepubescent`/0, gene 13.0 → identical to pre-fix); **both goldens byte-identical, 0 re-anchors** |
| **i392 · row 2** | **MEASURED AND REJECTED** (`i392_novelty_driver.md`) — the plan's named consumer was the i351 `Wander` driver, and the wiring was built, measured and reverted. Channel live and monotone (pinned-gene A/B: 132 → 2 464 → 3 927 Wander decisions), §4.6 **proven at machine precision** (the gene-pinned-0.5 control reproduces the stored golden metric_hash `0xb2a1be3b18fbb46d` byte for byte), attribution exact (first divergence tick 657, agent 0 alone, gene 0.8381) — **and still rejected**, because the response surface is **elastic (≈4)**: the population mean multiplier (0.94) hides a 4.1× spread in exploration between carriers, and with it in tree the **revolution liveness family collapses to 1 of 3 seeds** (`[(5,0,2,0),(42,1,3,1),(12345,0,3,0)]`, `firing >= 2` fails) plus 2 goldens and 5 snapshots | `i392_novelty_driver` legs A–E (manipulation, pinned-gene A/B with the pre-i392 control arm, response curve, golden attribution) | rejection recorded with the numbers; **the trait is not wired and not deleted**; reverted tree 313/0/1, both goldens byte-identical ✓ |
| **i392b** | **DONE** (`i392b_wander_band.md`) — the act's precondition, discharged. The stale band was measured on the **12-seed family** at i351's own config: **1.67–8.81%, mean 4.30%** (8/12 seeds above the documented 0.5–3% ceiling, seed 99 above the 8% guard too), and the breach was **already in `ENGINE_STATUS` §5's own i384 table** (Wander 4.4/5.4) with nothing asserting it. Decided **re-contract, not re-calibrate**, on the displacement test the band's rationale names: at i384's exact config Work reads **33.88% vs 32.70%** (+1.18 pt) at N=12 and **29.34 vs 30.10** (−0.76 pt) at N=48 — nothing is displaced, so a coefficient change would be a magnitude knob pulled for nothing (§4.4). The new contract is three invariants, pinned by `exploration_driver_is_live_gated_and_bounded_across_the_seed_family`: **liveness** (6/6 seeds), **gate exclusivity** (≥99% of wins inside the quiet window; measured 100%, i.e. exploration never outbids provisioning), **boundedness** (≤12% — sized between the calibrated family max 9.20% and the over-drive coefficient 3.0's 13.30%). Magnitude recorded, not asserted (§4.10). Both gate halves **proven to trip** (coefficient 0 → liveness; 5.0 → bound at 15.31%) | `i392b_wander_band` legs A–D + the new pin | contract enforced by a **test** for the first time; **behaviour unchanged** (coefficient untouched, goldens byte-identical). New debt: **3–76% of wins are noise-decided** (mean 32%) — the driver's determinism at the margin is an action-layer question, recorded not papered over |
| **i392 · row 3** | **DONE** (`i393_chronic_pain_gene.md`) — `chronic_pain_risk` reaches the skeletal accumulation law through `SkeletalUpdateParams` (row-1's params-struct pattern). Two §4.15 gates: the band is **closed in every calibrated corpus** (leg B: 2.29 M agent-ticks over five corpora incl. pestilence/collapse — severe injury never occurs, so the wiring is invisible to goldens *by construction*, recorded as measured-scope) and the response is **exactly linear** (leg D: ±40% gene ⇒ ±40% state at fixed risk — no elasticity cliff, unlike row 2). Anchored at the **draw midpoint 0.25**, not the Default 0.2 (§4.6): the draw U(0,0.5) has mean 0.25, so the Default anchor would have baked a +8% population drift into every future injury world. Second finding: **the spec field is inert** — `organs.ron` declares 0.001, which cannot work (5e-5 < the 1e-4 quantum), the code's 0.005 was right, and the comment misattributing to the spec is now a documented divergence + spec debt. Two probe-harness traps fixed before trusting numbers: final-snapshot blindness (decay empties the state between spike and sample) and the saturated harness (driving injury through `tick_update` pins every carrier at cp 1.0 — the §4.13 trap) | `i393_chronic_pain_gene` legs A–D + two unit tests (gene ordering; midpoint identity) | person 125/125, sim 310/310, integration 314/0/1, gate GREEN, **both goldens byte-identical** (measured-scope, not luck) |
| **i392 · row 5** | **MEASURED AND REJECTED** (`i395_aggression_threshold.md`) — the last inert gene, and the only row whose locus the probe *confirmed* before wiring (legs A–C: the 1.2 escalation gate is live — 4–7 of 12 agents per seed already above it on `dominance + risk_tolerance` alone; δ\* mean 0.18–0.24; and the law's response is **smooth** in the pool share, 41 → 37 → 31 → 11 → 6 across the gene range, unlike rows 2/4). The law `threshold = 1.2 × (1 + (gene − 0.55) × 1.5)` (§4.6-anchored at the `U(0.2,0.9)` draw midpoint) was wired and failed the pre-registered checklist on **four independent families**: revolution liveness `[(5,0,1,0),(42,0,2,0),(12345,1,3,1)]` (seed 5 → **0** in 70K), fear-contagion presence **10/12** (band 11–12), prediction-error **3/6** (needs majority), founding `kinship_penalty` **0.5 ≠ 0** (seed 43) — **plus both goldens and 7 snapshots**. Attribution exact: the same 9 tests pass on the reverted tree. **Structural finding (the row's real output):** every coupling that survives the golden gate in this engine is *dormant in the calibrated window* (the repeated `identity at zero` device); a founder-time gene on a tick-0-live gate cannot be, so rows 1/3 could land (their consumers sit inside dormant states) and rows 2/4/5 could not. Recorded upgrade path: gate the modulation on a state that is zero at tick 0 (`norm_resistance("no-violence") > 0`, zero before tick 4320) so dormancy is structural, then re-run the same checklist — a design-act candidate, not debt | `i395_aggression_threshold` legs A–E + unit pin (built, passed) | wiring + its four constants reverted; tree 314/0/1, both goldens byte-identical ✓; trait inert by decision |
| **i392 · row 4** | **MEASURED AND REJECTED** (`i394_sensory_acuity.md`) — the census-named consumer (i334 perception radius) is an **integer staircase**, disqualified before probing; the class-consistent candidate was the attention system's hardcoded `salience_bias = 0.5` — numerically the **mean of the gene's own draw** (U(0.2,0.8) → 0.5). Built as `AttentionState::with_acuity` at the three construction sites, measured, reverted. Anchor exact (pinned-0.5 reproduces both goldens byte for byte — including collapse, once the probe itself used `from_scenario`; the first divergent reading was a probe bug, the scenario's `drought_until` pre-set), but the **encode threshold 0.2 sits inside the gene's draw range** (fresh own-help salience ≈ 0.216 rides the band edge): memory occupancy spreads **13×** across the gene range (79 → 1 132 traces) — a threshold lottery, not a gradient — and the **revolution liveness family fell to 1/3** (0/0/2, the i388 family config) via Behavior-stream re-timing | `i394_sensory_acuity` legs A–D (golden control, attribution, response curve, liveness family) | wiring reverted, tree byte-identical, goldens MATCH/MATCH on the reverted tree ✓; trait stays **deliberately inert**; future-consumer qualification recorded (continuous, no band edge inside the draw range, RNG-stream isolation or same-iteration family re-sweep) |

### W1 — Finish the relationship unification (G2)
| iter | root cause | probe | exit criterion |
|---|---|---|---|
| **i400** | **MEASURED AND REVERTED** (`i400_cluster_trust.md`) — the last two self-contained v1 trust readers in `social_cluster.rs` (§8.1.9 ToM pair trust, §19.5.I knowledge-diffusion `source_trust`; the comfort path had already moved at i369). The probe gave the cleanest shape a reader migration can have — **zero fallback misses on either store** (so only the store changes), mean \|v1−v2\| 0.0096/0.0195 — but with an **unbounded tail** (max 0.70/1.00) directly on a live band edge (`infer_intent` is Friendly only above trust 0.5; 0.8–1.8% of sampled pairs straddle it). The suite then answered: **304/10/1 in 211 s** vs **314/0/1 in 165 s** reverted — both goldens, 6 snapshots, the `knowledge_acquisition_desacralizes_sacred_values` fixture (v1 pin 0.9 vs the v2 stranger prior 0.4 under the 0.5 acceptance floor), and **two deliberate dormancy contracts** (`pregnancy_state_refactor_keeps_lifecycle_dormant` — "Lars became pregnant"; `conception_pregnancy_…` — "no pregnancy may exist in the golden window"). A ~0.01-mean accuracy change on a soft consumer is not worth widening a design contract (§4.1), so the reads stay on v1 with the finding recorded at both sites, and the retry's precondition is now sharper than the i393 sequence: **the ToM/knowledge trust path and the interaction-gain deletion must move in ONE commit**, so the dormancy windows and the acceptance floor are re-derived once against the whole new trust surface instead of perturbed three separate times | `i400_cluster_trust` legs A–C + two suite runs | reverted; tree green at 314/0/1, goldens byte-identical |
| **i399** | **LANDED (behavioural — the sweep's first non-inert landing)** (`i399_marriage_store.md`) — `marriage.rs` was not a reader but a **closed loop inside the ghost store**: the formation gate reads v1 affection/trust, the bond boost (+0.2/+0.3, both directions) writes v1, and the jealousy pass reads v1 trust — while v1 `affection` has **no other production reader and no decay**. Probe: affection divergence mean 0.043/0.081 (N=12/48), married-pair end affection **v1 0.937 (pinned flat) vs v2 0.878 (decayed, differentiated)** — the bond the engine forged was invisible to the machinery built to model it. Migrated read+write together per i384. Three measured consequences: (1) **seven snapshots + both goldens** re-baselined with mechanism direction (trust 0.6345→0.6475, quality 0.4124→0.4217, stage distribution **deepens to `Confidant`**, 10K stress rises 0.3605→0.3679 — bonds now counted, not saturating); (2) the revolution family re-anchored after the **sweep proved the producer not starved** (total **15 → 18** revolutions, **3/10 → 5/10** firing seeds; members {12345, 7, 11}); (3) **a stale fixture hung the suite for >300 s** — `marriage_forges_spouse_and_inlaw_kinship` pinned only v1, so no pair was suppressed and agent 0 paired with agent 1 (900-step window exhausted). `interaction_count`/`is_contacted` semantics unchanged (v2 rows are complete and seeded from v1 at populate, so the migration is value-equal at tick 0). **Operational lesson recorded: grep for v1-only pins before migrating the next reader.** | `i399_marriage_store` legs A–C + `i399_revolution_sweep` (10-seed before/after) | sim 310/310, integration 314/0/1, goldens regenerated, gate GREEN |
| **i398** | **READER LANDED ZERO-BLAST; WRITERS MEASURED AND DEFERRED** (`i398_norms_store.md`) — the reader-first sweep's first item. The `norms_impl` belief-evidence channel (`rel_pos` → `trust − 0.5`) now reads the dyadic store: **goldens byte-identical, 314/0/1** — provably inert, not luckily (the probe shows the stores agree at every pinned horizon ≤0.017 after the early pre-boundary spike of 0.082; the belief consumer is linear toward equilibrium; evidence is symmetric around 0). The file's three v1 writers (violence −0.3/−0.2 ×2 directions, punishment −0.15×p) were moved and **reverted with the mechanism understood**: the v1 write was transient (i376's daily sync erases it within a day) so on v2 the same −0.3 is PERSISTENT, re-arming the Iter-185 low-trust→threat spiral — measured: one extra death by 10K (13→12), `peer_status` 0.2255 (floor 0.3), fear-contagion presence 9/12, a pinned conception slid. **Doctrine nugget: a transient write made persistent is a magnitude change, not a store change.** The write stays on v1 with the finding recorded at the site; the magnitude gets re-sized for persistence by a violence-family sweep (i273 family + the death-spiral horizon) before the move | `i398_norms_store` legs A–B + three suite runs | reader landed; writers deferred with a named next iteration |
| **i393** | **PROBE LANDED + INERT HALF LANDED; MIGRATION IS ORDER-CONSTRAINED** (`i393_speech_act_store.md`) — this iteration retired the write-only §19.5.G ladder (both goldens byte-identical, 314/0/1, i.e. proven zero-blast by the reader census) and ran the migration itself **twice, in two shapes, both reverted with measurements**: (1) writer-deleted-first → **299/15/1 in 656 s** (5× slowdown; `revolution` + `attachment` ×2 + `bonding_rate` + belief/memory liveness pins red), because every v1 reader is calibrated against the ~0.9 level the deleted writer produced and §2.3 forbids re-pinning a producer that goes dark; the decisive insight is that **`bonding_rate`/`conflict_escalation_rate` are `Fixed::ONE` defaults whose only consumer was that write**, so the repair is to re-host them on the dyadic application (value-neutral at defaults), never a re-anchor. (2) folds-moved-only → **301/13/1 in 143 s** (normal speed, 4 behavioural pins of 4 different kinds: a `contagion_delta` **Fixed-4 truncation** hazard the move exposed, a kin-**labelling** correctness break, a `peer_status` band edge, and a §4.13 manipulation failure). **Corrected sequence: readers first (norms_impl, household, births_deaths, marriage/attachment), then delete the v1 application WITH the rate re-hosting, then the folds.** Original probe findings stand (below) |
| **i393 · probe** | **PROBE LANDED; DELIVERABLE CORRECTED TWICE** (`i393_speech_act_store.md`) — the v1 writer gain schedule (`+0.02…+0.10`/act, 3–10× the dyadic gain) is the last divergence source, and the probe sized it: **the divergence is a mid-day phenomenon** (trust agrees to 0.0009–0.0039 on a daily boundary and rebuilds to 0.169 mean at +72 ticks — leg A's first draft tried `Δtrust/Δacts` and could not work, because i376's sync dominates a v1 row's movement); **affection is never synced at all** (0.187 mean / max 1.000 at 50K, unbounded in horizon); and the three-regime projection picks the shape — importing the v1 deltas (regimes 2/3) raises dyadic saturation 50% → 61% at 50K, i.e. it re-imports the exact condition i376 diagnosed as v1's failure, so **regime 1 (delete the gain, keep the dyadic schedule)**. Two census findings correct the plan: **`Relationship.kind` has NO production reader** (write-only §19.5.G ladder — the i391 dead-field class; retire it, don't migrate it; `RelationshipV2.stage` is the live replacement) and **deleting the interaction gain does not make v1 derived** (`legal_impl` −0.2 and `marriage` +0.2/+0.3 keep writing it, above the deleted gain's magnitude) | `i393_speech_act_store` legs A–D | contract: delete the interaction gain, re-point the speech-act model's `base_delta` doc+guard at the schedule actually applied, retire the write-only ladder and pin `stage` liveness instead, then re-anchor trust-gated pins with attribution. **The `legal_impl`/`marriage` v1 writers are deliberately out of scope** (separate behavioural moves, §2.1) and **`affection` must not be left unsynced** |
| **i394** | The remaining legacy-row readers — **the queue over-counted this list**: `household` and `births_deaths` already read the dyadic store, so the real set is `social_cluster`'s ToM + knowledge-diffusion trust and the `memory_ops` `trust > 0.6` status fold | one probe per consumer, re-using the i384 read-source-delta metric **plus a band-edge leg** (i400 found the divergence tail lands on the 0.5 `infer_intent` / acceptance verdict) | each consumer's read-source delta ≤1% **and** its band-edge straddle share measured; re-anchors attributed per §4.2 |
| **i395** | With every reader and writer migrated, the v1 matrix + its daily sync pass are pure redundancy | `i395_v1_retirement`: run with the matrix frozen/removed, compare full-run digests | the v1 matrix is deleted (or reduced to a documented projection), the sync pass removed, goldens re-anchored **once** under custody with mechanism evidence |
| **i401** | **MEASURED AND REVERTED — the bundle does not land** (`i401_bundled_closeout.md`, probe `i401_attachment_mechanism`). Built as i400 asked (writer deletion + rate re-host + `affection` sync leg + fold move + parameter retirement + the `social_cluster` reads, one commit): **304/16/1 in 656 s** — 2 goldens, 7 snapshots, 7 behavioural pins (conception, attachment, meme transmission, fear contagion, noospheric conviction, relational-field refresh, revolution). Decomposed by re-running with single pieces backed out: **(A)** the `social_cluster` read move alone kills the conception producer (`conception_pipeline_round_trips_with_birth` PASSES with only that file reverted — zero pregnancies in 2 000 accelerated seed-46 ticks), so i400's revert is now proven *under the bundle it asked for* and that move belongs in a behavioural iteration with its own re-pacing probe; **(B)** the writer deletion alone leaves the attachment coupling visibly quieter on all three seeds (nonzero distress 38/29/43 → **15/10/13**, means 0.0079/0.0041/0.0120 → 0.0022/0.0022/**0.0016**, under the 0.002 floor) and the probe localises the cause **one hop upstream**: `choose_interaction` decides the interaction *kind* from the **v1** trust/affection pair, and the deleted write was a **ratchet** (v1 only ever converged toward v2 via the daily sync, never down), so it parked pairs in the 0.7+ Comfort branch and *out* of the Gossip branch — the branch that fires `on_reunion`. Co-residency is ruled out as the explanation (co-resident couples FELL 10 → 4 and mean partner distance ROSE 14.75 → 15.42 while distress fell); **(C)** the `social_status_counts` fold move is orthogonal to attachment (27/48 with it and without it) — a clean small landing once the schedule is settled; **(D)** `social_reciprocal_factor`'s retirement is coupled to the writer deletion and cannot precede it (a dead parameter is a hazard, not an early win). **Doctrine nugget: a reader whose calibration rode a write's ratchet is not a reader — it is a producer's calibration surface; migrate it with a kind-mix probe, not a band widening.** | `i401_attachment_mechanism` (3 seeds, partition + distance + co-residency) + three suite runs + two isolation re-runs | reverted; tree green at 314/0/1, goldens byte-identical |
| **i402** | **DONE — measured deferral, folded into i403** (`evidence/i402_kind_schedule.md`, 2026-09-25). The kind-mix probe ran: the ratchet is **real in sign** (v1−v2 affection +0.002…+0.033) but the surface is **saturated on both stores** (contacted-pair affection v1 p50 = 1.0000, v2 p50 = 0.9986–0.9997) — `affection > 0.70` opens for 98–99% of interactions on v1 AND v2, branch flips 0.0/2.9/3.5%. The gate itself is the defect (§4.10 absolute-threshold-on-self-driven-aggregate); the 0.34/0.45 "matching quantiles" are NOT adopted. Read move + gate redesign (relative/cohort candidate) land inside i403, re-derived against the post-deletion surface | `i402_kind_schedule` (ran; probe adopted — one harness bug fixed: the latch leg's `run(t+step)` executed Σ = 10.5× the intended horizon before the fix) | verdict recorded with legs A/B/C/D numbers; the sequence decision (i403 first, then re-run probe, then one commit) documented in the evidence doc §Verdict |
| **i403** | **MEASURED REJECTION** (`evidence/i403_writer_deletion.md`, 2026-09-25) — the full arc was built and run: writer deletion + rate re-host + affection sync + reciprocal retirement + the folded kind-schedule read move + credibility move + social_cluster ToM/source_trust reads + the contacted_degrees re-point. **The crisis route starves on the honest trust surface**: revolution sweep total 18 → 2 (1-of-10 firing), panics 0 on 9-of-10 pestilence seeds, and the i381 panic gap COLLAPSED (calm peaks 0.49–0.51 on top of crisis peaks 0.54 — the calm/crisis charge distributions now overlap, so no re-sizable bar exists); the belief ecology's conviction halved (0.486 → 0.215). Three in-arc findings: (a) the **degree-freeze bug** — `contacted_degrees` counted v1 bookkeeping the deletion froze → mass Background demotions → 35–65% volume collapse (fixed by the dyadic re-point; lesson = AGENTS §4.17); (b) the **witness re-host reverted by measurement** (persistent-on-v2 pinned trust at 1.0, i398 class); (c) the **contagion f64 fix exonerated by A/B** (identical per-seed revolution counts in both arms) — **landed separately as i425** with custody (`evidence/i425_contagion_f64.md`: only the collapse golden moved, +2.3 quanta avg_fear from fold-on-final-tick phase alignment; riverford bit-identical). **Re-attempt preconditions** (§6 of the evidence doc): belief-charge deficit formation-time-vs-steady-state probe FIRST, then the belief-channel re-derivation (knowledge-diffusion floor, ToM band, gossip confidence, panic charge law) against the honest surface, THEN the stashed arc re-applies | one commit was attempted; the stash ("i403 measured rejection") carries the preserved arc | the crisis engine's liveness (panics → breakdown → revolutions, ≥2 firing seeds) restored BEFORE any deletion lands; no starved producer re-pinned (§2.3) |

| **i425** | **DONE — LANDED WITH CUSTODY** (`evidence/i425_contagion_f64.md`, 2026-09-25) — the §5 Fixed-4 truncation fix the i393 migration exposed and the i403 arc preserved: `contagion_delta` computed in f64 and quantized once, so the low-stress range (`perceived_stress < 0.002` at `FEAR_CONTAGION_RATE = 0.05`) stops truncating to zero. Both A/B arms measured by `i425_contagion_calm_band`: the repaired band `[0.001, 0.002)` fires 0–1 of 72 agent-days per calm seed, the floor→nearest round-up class 30–53 (the dominant per-fold class, ≤ 1 quantum per agent-fold); the fold increment PERSISTS (not washed; riverford t=144 mean fear 0.287800 → 0.287900, +1q, re-aligning within ~7 ticks). **End-state visibility is PHASE-DEPENDENT — the decisive finding**: all three calm 1000-tick end states are IDENTICAL across arms (the riverford golden reproduces bit-for-bit), while collapse (4320 = 30×144, a fold fires inside its final tick) ends +2.3 quanta avg_fear (0.294131 → 0.294154; agent_count 13 and event_count 40 170 identical). Custody: collapse `metric_hash` 8993835882449929447 → 11698195916283842933 (the ONLY field that moved); 2 snapshots +1q avg_stress; the 10K snapshot's memory-kind stock redistributed at a CONSTANT 789 traces (Emotional 447→443, Episodic 15→14, Social 309→314) — named survivorship chain: fold ≤1q fear → appraisal `arousal=(fear+anger+joy)×0.5` → encode `charge=arousal×0.6+0.1` → decay `emotional_bonus=charge×0.1` + `evict_weakest` quantum-margin tie-breaks. Crisis corpora untouched (identical per-seed revolution counts). 307/4 → **311/0/1** | `i425_contagion_calm_band` (synthetic band discriminators, 3-seed occupancy, riverford fold-window, collapse final-window + end state; ran on both arms) | no behavioural pin moved; no calibration constant touched; the i403 re-attempt preconditions unchanged |

### W2 — Space, contact, locomotion (G3)
| iter | root cause | probe | exit criterion |
|---|---|---|---|
| **i396** | Locomotion is one step per decision and reachable only through feud approach; §6 names it as the second weakness of the agent layer | `i396_locomotion_cadence` — decision-cadence vs movement rate; contact-graph effect of multi-tile stepping (pathing to a target, returning home) | `Move` is a first-class driver with a measured share; contact breadth rises without breaking the calibrated village (goldens byte-identical at N≤48 or re-anchored with evidence) |
| **i397** | Relationships form from **co-residency**, not encounter: contact is a proxy for sharing a home cell | `i397_encounter_contact` — encounter-driven tie formation (radius-gated, off-home) vs today; re-run i350's saturation metric at a **behavioural** horizon (≥40K), not short windows | touched-R at 40K is materially below the 73.7% co-residency saturation **and** the tie graph still reproduces kinship/courtship/marriage liveness |
| **i398** | Clan count is fixed at 3 regardless of settlement structure (i323) | `i398_clan_formation` — clans derived from settlement proximity + founding kin cores, fission on feud, merge on attrition | clan count varies with N and geography across a ≥12-seed family, with clan enmity/feud channels still live |
| **i399** | World area is decorative below N≈144 (i344) — **operator decision A9 outstanding** | `i399_density_sweep` — the density law at the test matrix {12, 48, 144, 256}, measured as *fidelity* (contact breadth, max co-location) with the perf delta reported honestly | density law adopted (or explicitly rejected) with a coordinated re-anchor sweep of the contact-derived pins |

### W3 — Inter-community architecture (G4 + G5) — the country path
| iter | root cause | probe | exit criterion |
|---|---|---|---|
| **i400** | Diplomacy is an off-map abstraction of three fixed names; the on-map polity graph (i297/i298/i360) is not the diplomacy graph | `i400_polity_diplomacy` — derive neighbors from actual polities (distance, shared trade/kinship history), relations updated by real cross-polity events; off-map names retained only as an explicit "outside world" fallback | every on-map polity has relations derived from world state; `raids`/`caravans` counters move from real inter-polity events; N=192/256 runs show ≥2 polities with distinct relation trajectories |
| **i401** | Raids and caravans still conjure/remove grain from an abstraction; a real settlement's loss must be another's gain | `i401_on_map_trade_conflict`: grain transferred between settlement stores, militia (`military.readiness`) tied to the raided polity, war/peace states between polities | conservation holds (Σgrain before/after within the transfer's bookkeeping), `military` roster/musters scale with the raiding polity, journal entries name the actual settlement |
| **i402** | No hierarchical layer: polities have holons but no relations, no federation, no regional council | `i402_regional_polity`: second-order holon over polities (regional legitimacy, treaty/federation state), regional aggregation of memes/technology | a run at N=256 produces measurable regional-vs-local divergence (a meme or norm adopted regionally but not locally, or vice versa) with liveness pins |
| **i403** [M] | Demography at town scale is unmeasured against settlement dynamics (fission/migration exist but their settlement-level effect is not) | `i403_settlement_demography`: does fission actually occur? migration between settlements? measure N=256 over 100K | a settlement-level demographic verdict with numbers, feeding i404 |

### W4 — Scale architecture (G6)
| iter | root cause | probe | exit criterion |
|---|---|---|---|
| **i404** | cognitive (21.3%) + memory (17.8%) = 39% of the N=192 tick, and i334's perception gate only covered the memory pass | `i404_cognitive_gate`: the same perception/edge gating applied to the cognitive pass's row sweep (11.3% of tick by itself), plus a §17.3 dirty-window pacing probe | tick cost at N=192 down materially (recorded number, not a vibe) **and** behaviour re-anchored only where the gate legitimately changes perception (a *behavioural* change, probed) |
| **i405** | The LOD tier is a crisis modulator, not a scale lever: Background is dark in calm towns and never carries work | `i405_living_background`: a background population that actually acts (budgeted cognition + social presence + coarse actions) rather than being dark | at N=192/256, Background carries a measured share of agent-ticks in **calm** worlds with per-agent budget caps; behaviour at N≤48 unchanged |
| **i406** | `MAX_POPULATION = 256` is a perf envelope, not a law — and H5's founder-shape debt unlocks only at larger N | `i406_cap_envelope`: re-baseline the envelope with i405's tier live, at 384/512 | cap raise adopted (charter) with a measured envelope, or recorded as not-yet |
| **i407** [M] | City/planet architecture has never been prototyped; the Ω(N²) relationship floor is confirmed but the *partitioned* alternative is not measured | `i407_partition_spike`: a prototype in which cross-settlement state is explicit and sparse (per-settlement tick slices, no global dense matrix), measured against the same N | a documented verdict: partition feasible / not, with the numbers that decide it — a spike, never a half-landed refactor |

### W5 — Economy and demography counter-forces (G7 partial)
| iter | root cause | probe | exit criterion |
|---|---|---|---|
| **i408** | Tax rate is a hardcoded policy stance; the council has legitimacy and treasury state but no fiscal policy | `i408_endogenous_tax`: rate set from fiscal need (treasury vs expected outlay, famine/war pressure) — **i377's constraint: not coupled to legitimacy** | rate varies with treasury/famine state across a ≥12-seed family; wealth pins re-anchored with the i365 Gini band as the contract |
| **i409** | Wealth concentration has weak counter-forces: inheritance/charity/redistribution absence is the audit's E6 residual | `i409_inheritance_charity`: wealth-at-death splitting, charity/redistribution norms, measured Gini trajectory to 100K | Gini does not drift monotonically upward; the counter-forces are attributable (per-mechanism delta) |
| **i410** | Founder-shape realism is the oldest recorded debt, blocked on "larger N + coordinated re-anchor sweep" — i406 is that unlock | `i410_founder_shape`: shaped founder draws at the raised N with the full re-anchor sweep (13 liveness pins were the Iter-263 casualty) | shaped draws land with **zero** liveness regressions at the new N, or the debt is re-recorded with the new evidence |

### W6 — Thin deep layers (G7)
| iter | root cause | probe | exit criterion |
|---|---|---|---|
| **i411** | Skill curve saturates (E8: farming 1.00, trading 0.00) — the last measured state is 2026-08-22 | `i411_skill_curve`: re-measure the distribution; add diminishing returns at the top and gradient at the bottom, with decay pressure | no skill pinned population-wide; distribution has an interior mode; skill-gated work/innovation channels remain live |
| **i412** | Fatigue is a village-wide synchronized phase (i321) — individual workload differentiation is lost | `i412_fatigue_differentiation`: per-agent workload weighting of the circadian/work cycle | within-seed fatigue dispersion rises with measured per-agent workload; bounds (no ceiling approach) preserved |
| **i413** | Innovation is present but not generative (tier-2 chains rare; technology tree depth 0–1 in most runs) | `i413_innovation_ratchet`: measure tier-2/3 chain firing naturally; make discovery depend on practice/teaching accumulation (skills feed technology) | tier-2 innovations fire in a majority of ≥12-seed 100K runs, attributable to measured practice accumulation |
| **i414** | Vendor-blocked trio (realms ontology, resonance matrix, cult-liveliness) + the audit's ritual post-formation residual | `i414_vendor_gate`: one probe measuring what each blocked channel would unlock, and whether a seedable fallback exists | each blocked item is either unblocked by a vendored/seedable substitute or re-parked with a date-stamped reason |

### W7 — Verification and observability (G8)
| iter | root cause | probe | exit criterion |
|---|---|---|---|
| **i415** | Citation drift is unchecked by the gate (i386 found 5 classes with `doc_index.py` green) | extend `scripts/doc_index.py` with a **citation check** (every `*.md|rs|py|sh` token in a governed doc must resolve from the repo root, the doc's directory, or a basename index; exit non-zero on a true ghost) | the gate fails on a planted ghost citation (the check is proven to trip, per the i170 subsystem-gate lesson) |
| **i416** | Longitudinal observability is partial: lineage/tail metrics landed at i251 with chart scaffolding, but the operator-facing longitudinal view is unfinished | `i416_charts`: finish the TUI longitudinal charts (population, stress, Gini, belief ecology, lineage) from cached metrics | charts render deterministically from a capture; zero mechanical effect (read-only pin) |
| **i417** | No standing perf envelope above N=96, and the per-pass profile is probe-only | `i417_envelope`: standing warn-only envelopes at 144/192/256 + a profile readout surfaced through the CLI | `scripts/gate` reports the envelope at the town tiers; a regression at N=192 is visible without running the probe by hand |

---

## 4. Sequencing and dependencies

> **Operator sequencing decision (2026-09-23): depth before scale.** The depth items
> (**W5/W6** — endogenous tax, inheritance/charity, skill curve, fatigue differentiation,
> generative innovation) are taken **ahead of** the scale items (**W3/W4** — inter-community
> architecture, LOD/cap). W0/W1 (i392b, i393) are unaffected and stay first, since they are
> in flight and i393 is the last structural inconsistency in the paired-store layer.
> The **scope** question (one cycle owning both, vs the city path splitting into DC-6) is
> *not* settled by this — see §5 item 3.
>
> **A9 is DECIDED and landed (2026-09-23): the constant-density law is the interim
> world-area policy** (`world_side_for_population`, now single-sourced in the world
> generator; evidence `a9_density_policy.md`). It affects **only N ≥ 96** — the law is
> anchored on the engine's own two calibrated points, so the village and the N=48 tier are
> byte-identical under either policy. **The organic target it is interim *for*, in dependency
> order** (keep this in view through W2/W3 — it is why i396/i397 matter and what finally
> retires the law):
>
> 1. **population is an output bounded by carrying capacity** — the ecology should set what
>    the world supports, and growth should *feel* it (crowding, disease, conflict over space,
>    out-migration) instead of hitting `MAX_POPULATION`;
> 2. **space is causal** — travel costs time and energy, so distance to well/farm/market/
>    temple defines a villager's real daily reach (i396);
> 3. **contact comes from encounters, not assignment** — ties form where activity puts people
>    together, not because two agents share a home cell (i397);
> 4. **density is a pressure with both ends wired** — `crowding` already feeds infection
>    exposure, and the axis extends to stress, housing quality and conflict over space, then to
>    the demographic response: **fission**, a settlement past local capacity founding a new
>    site, which makes the settlement-size distribution emergent instead of
>    `cluster_count_for(houses)`.
>
> End state: `side(N)` **disappears** — the question becomes how many agents the world
> supports, not how big to make the world for a chosen N.


```
W0 (i387→i392)  dead/inert surfaces  ── CLOSED (DONE; i390 succession, i391 census)
W1 (i393→i401)  relationship unification  ── IN FLIGHT; five landings, close-out at i401
W2             space/contact  ── must precede W3 (diplomacy needs a settlement graph)
W3             inter-community layer  ── the country path; needs W2's settlement graph
W4             scale architecture  ── the tier work unblocks the cap which unblocks founder shape
W5             economy/demography counter-forces  ── founder-shape row depends on the cap raise
W6             thin layers  ── independent; the cheapest realism-yield per iteration
W7             verification  ── the citation check is independent and can land any time
```

> **Iteration-number drift, resolved (i400):** the per-arc tables below were drafted with
> ordinal labels (W2 started at "i396", W3 at "i400" …) but real iterations consumed
> i393–i401 on the W1 close-out, so those labels no longer identify iterations. **§4.1 below is
> the authoritative schedule and numbering from i401 onward**; the arc tables are kept as arc
> *definitions* (root cause, probe, exit criterion) and their i-labels are draft ordinals.

### 4.1 Live queue at i400 — the recommended order

Sequenced by the operator's standing **depth-before-scale** decision, with dependencies made
explicit. Each row is one iteration and carries its own probe; nothing here is a batch.

| # | item | arc | depends on |
|---|---|---|---|
| **i401** | ~~Bundled close-out of the dual-store split~~ — **DONE AS A MEASURED REJECTION**: 304/16/1, decomposed into four findings, reverted. See the W1 table row | W1 | i400's band-edge finding (done) |
| **i402** | **DONE — measured deferral, folded into i403; NOTE: i403 subsequently REJECTED the fold** (see the i403 row below, `evidence/i403_writer_deletion.md`) — the read move's own measurement stands (0.0–3.5% branch flips, both stores saturated), but its landing awaits the belief-charge re-derivation named in the i403 preconditions (`evidence/i402_kind_schedule.md`, 2026-09-25): the kind-mix probe ran on the concurrent session's draft (taken over after ~80 min silence; gate-blocking fmt/lint + a latch-leg `run(n)` stepping bug fixed). **The premise is refuted by measurement**: the schedule already reads a saturated ratcheted signal — affection p50 = 1.0000 on v1, 0.9986–0.9997 on v2 across contacted pairs; the ratchet sign is real (v1−v2 +0.002…+0.033) but branch flips are 0.0/2.9/3.5%, and `affection > 0.70` opens for 98–99% of interactions on BOTH stores — §4.10's absolute-threshold-on-a-self-driven-aggregate class, as vacuous as the deleted `trust < 0.5` arm. The 0.34/0.45 "matching quantiles" are NOT adopted (quantile-matching a 98.5% share of a point mass re-codes the saturation). **Sequence decided**: i403 first (writer deletion + rate re-host — the surface change; leg D proves the latch is write-side: 100.00% above-gate persistence, below-gate rise 4–5%/kyr against 0.0002/tick decay), then this probe re-run post-deletion, then the read move + gate redesign in ONE commit — leading candidate the **relative/cohort form** (top-quartile of the current population's contacted-pair affection, N-sized) per §4.9/§4.10 + the i381/i383 precedents, decided by the post-deletion measurement | W1 | i401 §3 (the ratchet calibration) — **confirmed in sign, quantitatively irrelevant to branch selection** |
| **i402b** | **Locomotion cadence** — `Move` reaches the deliberative layer only via the feud approach (share 0.0887%); measure decision cadence vs movement and whether contact breadth rises | W2 | — |
| **i403** | **Encounter-driven tie formation** — ties today form from co-residency, not encounter; re-measure the i350 saturation metric at a behavioural horizon (≥40K) | W2 | i402b (contact volume must be a *measured* quantity first) |
| **i404** | **Carrying capacity + fission** — population becomes an output: crowding → stress/housing/conflict, and a settlement past local capacity founds a new site | W2 | i403 |
| **i405** | **Retire A9** — with capacity and fission causal, `world_side_for_population` disappears; the question becomes how many agents the world supports | W2, W4 | i404 |
| **i406** | **Skill curve** (E8: farming pinned at 1.00, trading at 0.00) — diminishing returns at the top, gradient at the bottom, decay pressure | W6 | — |
| **i407** | **Fatigue differentiation** — the village-wide synchronized phase loses per-agent workload signal | W6 | — |
| **i408** | **Generative innovation** — make discovery depend on practice/teaching accumulation so tier-2 chains fire naturally | W6 | i406 (skills feed technology) |
| **i409** | **Vendor gate** — realms ontology, resonance matrix, cult-liveliness: unblock via a seedable substitute or re-park with a date-stamped reason | W6 | — |
| **i410** | **Endogenous tax policy** — the council sets its own rate from fiscal need (not legitimacy, per i377's constraint) | W5 | — |
| **i411** | **Inheritance + charity** — the audit's E6 residual; measure Gini to 100K and attribute each counter-force | W5 | — |
| **i412** | **Living background tier** — the LOD Background is dark in calm towns; make it carry a budgeted share of work | W4 | — |
| **i413** | **Cap envelope at 384/512** — re-baseline with i412 live, then adopt or record | W4 | i412 |
| **i414** | **Founder-shape realism (H5)** — shaped draws at the raised N with the full re-anchor sweep; this is the item the Iter-263 casualty blocks | W5 | i413 (larger N) |
| **i415** | **Polity diplomacy from world state** — derive neighbours from actual polities; off-map names only as an "outside world" fallback | W3 | i403/i404 (a real settlement graph) |
| **i416** | **On-map trade and conflict** — grain conserved between settlement stores; militia tied to the raided polity | W3 | i415 |
| **i417** | **Settlement demography verdict [M]** — does fission occur, does migration move people between settlements (N=256 × 100K)? | W3 | i404 |
| **i418** | **Regional second-order holon** — regional legitimacy/treaty state; regional-vs-local meme divergence | W3 | i415, i416 |
| **i419** | **Cognitive pass perception gate** — the i334 gate covered memory only; cognitive + memory are ~39% of the N=192 tick | W4 | — |
| **i420** | **Partition spike [M]** — prototype explicit sparse cross-settlement state against the Ω(N²) floor; a verdict, never a half-landed refactor | W4 | i418 |
| **i421** | **DONE** (`evidence/i421_citation_gate.md`, 2026-09-24) — doc_index check 4: every `*.md\|rs\|py\|sh` token cited in an AUTHORITY/ACTIVE doc must resolve through the project's path-convention ladder (repo/docs/doc-parent/AP4-shorthand/crate-relative/bare-basename); HISTORICAL/REFERENCE docs exempt — frozen records, the same scope rule freshness uses. Historical/vendor citations are curated as **(doc, token, line-anchor)** triples — the exemption is line-scoped, so a future occurrence of the same token still fails. Found+fixed **2 real drift sites** in AGENTS §7 (pre-split example module names the split landed under other names) and **1 extractor false-positive class** (a symbol name whose `.sh`-prefixed middle matched the extension — now requires a word boundary). **Proven to trip**: planted ghost → exit 1 (both shorthand and repo-relative classes), and the trip + exemption-scope checks re-run in the gate as `doc_index.py --selftest`. ~130 tokens now gated across the 8 must-be-true docs | W7 | — |
| **i422** | **TUI longitudinal charts** — finish the i251 scaffolding (population, stress, Gini, belief ecology, lineage) | W7 | — |
| **i423** | **Standing perf envelope** at 144/192/256 + a CLI profile readout, so a regression is visible without running a probe by hand. **2026-09-25: the drift attribution is RESOLVED** (`evidence/i423_ab_town_scale_attribution.md` — quiet-state paired head/old 1.00–1.17, yesterday's ratios were spike-state samples; residual ≈8–9% arc+world delta, no single-iteration claim citable), and the row's requirement sharpens with it: **pin load AND record spike-state frequency + trigger conditions** (head arm entered a sustained 2–3.6× state in 2-of-3 interleaved runs while the old arm never spiked — a median-only envelope would certify a machine fine on average and unusable a third of the time); the gate's N=12/96 rungs remain structurally blind to town tiers | W7 | — |

**Cheap wins available at any time:** i406, i407, i409, i410, i411, i421, i422, i423 (no
dependencies). **The two structural walls** are i403/i404 (contact and capacity — they retire
both the density law and the co-residency proxy) and i419/i420 (throughput and partition — they
decide whether a city is reachable at all).

**Infrastructure queue pointer (plan-rust-craft C5, 2026-09-24):** the sim-glue detangle
queue that AGENTS.md §7 carries now has a **measured complexity inventory**: 193 functions
≥10 cyclomatic complexity, worst `tick_kinship_household_daily` at cc 130 (full table:
`docs/PLAN_RUST_CRAFT_AUDIT.md` F9). Any `*_impl` detangle iteration should pull its targets
from F9 rather than by hunt.

**First arc status: i387 → i388 → i389 → i390 → i391 → i392 CLOSED (rows 1 and 3 DONE; rows 2,
4 and 5 rejected with records), precondition i392b DONE. All five inert genes are settled —
two wired, three deliberately inert — and the act's structural lesson is recorded: a coupling
survives the golden gate only if it is dormant in the calibrated window. The next act item is
**i393** (the speech-act store migration).** i392's row 2 (`novelty_seeking` → the exploration driver) was
built, measured and **rejected** (`i392_novelty_driver.md`), and its rejection produced the
act's precondition: the driver's own acceptance band is stale, so the surface has no valid
calibration for a heritable trait to ride — **i392b** re-contracts or re-calibrates it with a
test behind the claim, and rows 3–5 wait for that. **i393** then starts the speech-act /
`RelationshipKind` migration.
Rationale as planned, with two insertions: i387 was a *measurement* that decided the action
layer's fate; i388/i389 closed two surfaces a probe had already named (a Class-D inert term
and a nominal authority channel); **i390 was the lifecycle item i389 exposed** — an office
holder who dies is never replaced, so the pestilence leg's authority was structurally absent
while its crisis ran at a 90.69% duty cycle, and succession blocks the inter-community
layer's offices as much as it blocked the directive channel. i391 swept the write-only-field
class (`scripts/field_census.py`: five dead genes, one documented false-positive class) and
found that the **consumer of one of them already exists** — `puberty_age` is drawn, inherited
and blended while `reproductive.rs` reads a village-wide `13.0` — so the sweep scheduled
**i392** as the wiring act, which owns the five inert genes. That act's first row landed with
zero blast (the band it governs is empty until ~385K ticks); its second row did not, and the
rejection is the more valuable of the two findings: a consumer's **response elasticity** is
part of its suitability, and a live, midpoint-neutral, machine-verified wiring can still be
the wrong place to put a heritable trait. Together with i393 the arc removes the last known
false affordances below the social layer — after which W2/W3 can measure space and
inter-community behaviour on a clean action surface.

---

## 5. Operator decisions this plan depends on

1. **A9 — fixed 32×32 vs constant density. DECIDED (2026-09-23): ADOPTED as the interim
   world-area policy** (`world_side_for_population`, single-sourced in the world generator
   after seven stale copies; evidence `a9_density_policy.md`). i344 measured it as a
   *fidelity* policy — **contact dilution** (near-pair share 11.3% → 4.7%, mean partners/agent
   22.1 → 10.2, contacted α 1.276 → 0.866) — not throughput (**+5.6% cost at N=192**; refuted
   as a perf lever). **Correction to the earlier note here: the `max co-location 19 → 4`
   result is A11's (i345's area packing), not A9's** — the two are complementary, not
   redundant. **Corrected premise (i392 row 2 re-read of i344):** the earlier
   "adopting it changes calibrated village structure" was wrong — the law
   `side(N) = max(16, ceil(√(21.333·N)))` is *anchored on the simulator's own calibrated
   points* (N=12 in 16×16 and N=48 in 32×32 both give 21.33 cells/agent), so it reproduces
   both **exactly** and the calibrated corpora are untouched by construction; the change bites
   only at N ≥ 96 (max co-location 19 → 4, near-pair share 11.3% → 4.7%, mean partners/agent
   22.1 → 10.2, contacted α 1.276 → 0.866). The re-anchor cost is therefore confined to the
   scale tiers' contact-derived pins, not the village. Still a charter call — it changes what
   the *town* is. **Needed before i399.**
2. **Cap raise (`MAX_POPULATION` 256 → 384/512).** Needed for H5's unlock (i410) and for any
   city-path claim. Requires a measured envelope (i405/i406) and a golden/harness review.
   **Needed before i406.**
3. **Does DC-5 own depth *and* scale, or should the city path be split out?** This plan
   proposes one cycle ("depth closure and inter-community scale") because the two are coupled:
   the inter-community layer (W3) is what makes town scale meaningful, and the LOD/cap work
   (W4) is what makes it run. If you prefer, W4/W5 become DC-6 and DC-5 ends at W3.
4. **The directive channel (i389): DECIDED — wired** (and i390 made it *live* in pestilence
   by giving the office a successor). Wiring gives offices real authority
   (a realism gain, a re-anchor cost); deleting is honest and free (i372 precedent). The plan
   defaults to **wire**, because the offices already exist and no other layer can express
   hierarchical authority.
5. **`Socialize` (i388): revive or re-contract?** i380's evidence says the term lacks the
   channels its siblings have; reviving buys deliberate sociality, re-contracting keeps the
   routine as the social carrier. Default: decide on i387's decomposition, not now.

---

## 6. Non-goals (explicit)

- **No magnitude-knob compensation** (§4.4 / i279). Ever.
- **No piecemeal founder-shape change**: it lands only with i406's larger N and the full
  coordinated sweep (§5 of AGENTS; Iter-263's 13 broken liveness pins are the receipt).
- **No sparse relationship store** as a perf fix: demoted by i326/i338/i344/i350. i397 is the
  *behavioural* question (do encounters, not co-residency, form ties), not a cache.
- **No `VecDeque` event conversion**: trigger is jitter-free ticks only (i354).
- **No planet-scale distributed execution**: i407 is a spike with a verdict, nothing more.
- **No dead-code deletion without a probe**: a surface is deleted only when its absence is
  measured (i372's rule), never by grep alone.

---

## 7. Exit criteria (the UM-5 gate, as this plan defines it)

DC-5 is done when all of the following are measured, not asserted:

1. **One relationship store**: the v1 matrix is gone or documented as a pure projection; every
   consumer's read-source delta ≤1% (W1).
2. **A living settlement graph**: at N≥192, ≥3 on-map polities with mutually-derived relations,
   grain-conserving inter-settlement trade/raids, and a regional (second-order) holon (W3).
3. **A measured town envelope** with the LOD tier carrying calm-world background work, and a
   documented cap decision (W4).
4. **No known dead surface**: every writer-only field, inert utility term and unreachable gate
   is either wired or deleted, with its probe recorded (W0, W6).
5. **The gate checks citations**, not just structure, and a standing perf envelope covers the
   town tiers (W7).
6. **Suite and goldens green throughout**, with every re-anchor carrying measured value, old
   band and mechanism (§4.2) and every iteration ending in commit + push.

---

## 8. Standing discipline (unchanged, restated because it is the plan's spine)

- Probe before touching; fix the producer, never re-pin a dead assertion (§2.3).
- One root cause per iteration; behavioural changes never mixed with structural ones.
- `cargo fmt --all && cargo clippy --workspace --quiet && cargo test -p mindstrata-tests --lib
  --release`, then `scripts/gate --full` before push.
- Re-anchors carry measured value / old band / mechanism in the comment (§4.2); knife-edge
  flags are recorded as debt, not flip-flopped (§4.5); the manipulation is checked before the
  response is re-pinned (§4.13); a doc's citations are re-pointed in the commit that moves the
  file (§4.14).
- Evidence doc per iteration under `docs/architecture/AP4-studio/evidence/`; ledgers updated in
  the same commit; `ENGINE_STATUS.md` re-measured whenever §1/§5/§7/§8 numbers move.
