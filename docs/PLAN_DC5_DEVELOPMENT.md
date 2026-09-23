---
name: plan-dc5
status: ACTIVE
description: "DC-5 implementation plan — depth closure and inter-community scale. Owns the gap taxonomy (G1–G8) derived from measured shallowness in the engine, the phase/iteration ladder that closes it, and the operator decisions it depends on. Authoritative for what remains after DC-4; ENGINE_STATUS.md stays authoritative for engine truth."
type: Plan
reconciled_commit: f1beca0
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
| Dead genome field | `physical_potential.sensory_acuity` — written at init/blend, **0 consumers** | grep today; `ENGINE_STATUS.md` §6 |
| Skill curve | farming pinned 1.00 population-wide, trading 0.00 (audit E8) — **last measured 2026-08-22, needs re-probe** | `AUDIT_2026-08-22` E8 |
| Fatigue | village-wide synchronized phase; within-seed mean swings 0.05↔0.41 | i321 |
| Wealth | Gini 0.647 → 0.611 (council dividend); endogenous tax not started; i377 constraint: **do not** couple rate to legitimacy | i363, i373 #2, i377 |
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
| **i390** [M] | Write-only genome/person fields are a recurring class (i313 injury, i124 emotion-context, i307 habit gate) and only `sensory_acuity` is currently known | `i390_dead_field_sweep`: field-level write/read census across `Genome`, `EmbodiedState`, `BodyState`, `DevelopmentFieldState` | every writer-only field is either wired or deleted; the sweep script is recorded for re-use |

### W1 — Finish the relationship unification (G2)
| iter | root cause | probe | exit criterion |
|---|---|---|---|
| **i391** | The v1 **writer** gain schedule (`+0.02…+0.10`/act, ~10× the dyadic gain) is the last divergence source; i384 measured that the speech-act effect model must move in the same commit | `i391_speech_act_store` — per-act deltas from both stores on the same interactions | speech-act model + `RelationshipKind` ladder read/write the dyadic store; v1 gain deleted; divergence ≤ the dyadic store's own noise |
| **i392** | Three consumers still read the legacy row (`norms_impl`, `household`, `births_deaths`) — each has its own divergence stake and its own probe (i369/i384 method) | one probe per consumer, re-using the i384 read-source-delta metric | each consumer's read-source delta ≤1%; re-anchors attributed per §4.2 |
| **i393** | With every reader and writer migrated, the v1 matrix + its daily sync pass are pure redundancy | `i393_v1_retirement`: run with the matrix frozen/removed, compare full-run digests | the v1 matrix is deleted (or reduced to a documented projection), the sync pass removed, goldens re-anchored **once** under custody with mechanism evidence |

### W2 — Space, contact, locomotion (G3)
| iter | root cause | probe | exit criterion |
|---|---|---|---|
| **i394** | Locomotion is one step per decision and reachable only through feud approach; §6 names it as the second weakness of the agent layer | `i394_locomotion_cadence` — decision-cadence vs movement rate; contact-graph effect of multi-tile stepping (pathing to a target, returning home) | `Move` is a first-class driver with a measured share; contact breadth rises without breaking the calibrated village (goldens byte-identical at N≤48 or re-anchored with evidence) |
| **i395** | Relationships form from **co-residency**, not encounter: contact is a proxy for sharing a home cell | `i395_encounter_contact` — encounter-driven tie formation (radius-gated, off-home) vs today; re-run i350's saturation metric at a **behavioural** horizon (≥40K), not short windows | touched-R at 40K is materially below the 73.7% co-residency saturation **and** the tie graph still reproduces kinship/courtship/marriage liveness |
| **i396** | Clan count is fixed at 3 regardless of settlement structure (i323) | `i396_clan_formation` — clans derived from settlement proximity + founding kin cores, fission on feud, merge on attrition | clan count varies with N and geography across a ≥12-seed family, with clan enmity/feud channels still live |
| **i397** | World area is decorative below N≈144 (i344) — **operator decision A9 outstanding** | `i397_density_sweep` — the density law at the test matrix {12, 48, 144, 256}, measured as *fidelity* (contact breadth, max co-location) with the perf delta reported honestly | density law adopted (or explicitly rejected) with a coordinated re-anchor sweep of the contact-derived pins |

### W3 — Inter-community architecture (G4 + G5) — the country path
| iter | root cause | probe | exit criterion |
|---|---|---|---|
| **i398** | Diplomacy is an off-map abstraction of three fixed names; the on-map polity graph (i297/i298/i360) is not the diplomacy graph | `i398_polity_diplomacy` — derive neighbors from actual polities (distance, shared trade/kinship history), relations updated by real cross-polity events; off-map names retained only as an explicit "outside world" fallback | every on-map polity has relations derived from world state; `raids`/`caravans` counters move from real inter-polity events; N=192/256 runs show ≥2 polities with distinct relation trajectories |
| **i399** | Raids and caravans still conjure/remove grain from an abstraction; a real settlement's loss must be another's gain | `i399_on_map_trade_conflict`: grain transferred between settlement stores, militia (`military.readiness`) tied to the raided polity, war/peace states between polities | conservation holds (Σgrain before/after within the transfer's bookkeeping), `military` roster/musters scale with the raiding polity, journal entries name the actual settlement |
| **i400** | No hierarchical layer: polities have holons but no relations, no federation, no regional council | `i400_regional_polity`: second-order holon over polities (regional legitimacy, treaty/federation state), regional aggregation of memes/technology | a run at N=256 produces measurable regional-vs-local divergence (a meme or norm adopted regionally but not locally, or vice versa) with liveness pins |
| **i401** [M] | Demography at town scale is unmeasured against settlement dynamics (fission/migration exist but their settlement-level effect is not) | `i401_settlement_demography`: does fission actually occur? migration between settlements? measure N=256 over 100K | a settlement-level demographic verdict with numbers, feeding i402 |

### W4 — Scale architecture (G6)
| iter | root cause | probe | exit criterion |
|---|---|---|---|
| **i402** | cognitive (21.3%) + memory (17.8%) = 39% of the N=192 tick, and i334's perception gate only covered the memory pass | `i402_cognitive_gate`: the same perception/edge gating applied to the cognitive pass's row sweep (11.3% of tick by itself), plus a §17.3 dirty-window pacing probe | tick cost at N=192 down materially (recorded number, not a vibe) **and** behaviour re-anchored only where the gate legitimately changes perception (a *behavioural* change, probed) |
| **i403** | The LOD tier is a crisis modulator, not a scale lever: Background is dark in calm towns and never carries work | `i403_living_background`: a background population that actually acts (budgeted cognition + social presence + coarse actions) rather than being dark | at N=192/256, Background carries a measured share of agent-ticks in **calm** worlds with per-agent budget caps; behaviour at N≤48 unchanged |
| **i404** | `MAX_POPULATION = 256` is a perf envelope, not a law — and H5's founder-shape debt unlocks only at larger N | `i404_cap_envelope`: re-baseline the envelope with i403's tier live, at 384/512 | cap raise adopted (charter) with a measured envelope, or recorded as not-yet |
| **i405** [M] | City/planet architecture has never been prototyped; the Ω(N²) relationship floor is confirmed but the *partitioned* alternative is not measured | `i405_partition_spike`: a prototype in which cross-settlement state is explicit and sparse (per-settlement tick slices, no global dense matrix), measured against the same N | a documented verdict: partition feasible / not, with the numbers that decide it — a spike, never a half-landed refactor |

### W5 — Economy and demography counter-forces (G7 partial)
| iter | root cause | probe | exit criterion |
|---|---|---|---|
| **i406** | Tax rate is a hardcoded policy stance; the council has legitimacy and treasury state but no fiscal policy | `i406_endogenous_tax`: rate set from fiscal need (treasury vs expected outlay, famine/war pressure) — **i377's constraint: not coupled to legitimacy** | rate varies with treasury/famine state across a ≥12-seed family; wealth pins re-anchored with the i365 Gini band as the contract |
| **i407** | Wealth concentration has weak counter-forces: inheritance/charity/redistribution absence is the audit's E6 residual | `i407_inheritance_charity`: wealth-at-death splitting, charity/redistribution norms, measured Gini trajectory to 100K | Gini does not drift monotonically upward; the counter-forces are attributable (per-mechanism delta) |
| **i408** | Founder-shape realism is the oldest recorded debt, blocked on "larger N + coordinated re-anchor sweep" — i404 is that unlock | `i408_founder_shape`: shaped founder draws at the raised N with the full re-anchor sweep (13 liveness pins were the Iter-263 casualty) | shaped draws land with **zero** liveness regressions at the new N, or the debt is re-recorded with the new evidence |

### W6 — Thin deep layers (G7)
| iter | root cause | probe | exit criterion |
|---|---|---|---|
| **i409** | Skill curve saturates (E8: farming 1.00, trading 0.00) — the last measured state is 2026-08-22 | `i409_skill_curve`: re-measure the distribution; add diminishing returns at the top and gradient at the bottom, with decay pressure | no skill pinned population-wide; distribution has an interior mode; skill-gated work/innovation channels remain live |
| **i410** | Fatigue is a village-wide synchronized phase (i321) — individual workload differentiation is lost | `i410_fatigue_differentiation`: per-agent workload weighting of the circadian/work cycle | within-seed fatigue dispersion rises with measured per-agent workload; bounds (no ceiling approach) preserved |
| **i411** | Innovation is present but not generative (tier-2 chains rare; technology tree depth 0–1 in most runs) | `i411_innovation_ratchet`: measure tier-2/3 chain firing naturally; make discovery depend on practice/teaching accumulation (skills feed technology) | tier-2 innovations fire in a majority of ≥12-seed 100K runs, attributable to measured practice accumulation |
| **i412** | Vendor-blocked trio (realms ontology, resonance matrix, cult-liveliness) + the audit's ritual post-formation residual | `i412_vendor_gate`: one probe measuring what each blocked channel would unlock, and whether a seedable fallback exists | each blocked item is either unblocked by a vendored/seedable substitute or re-parked with a date-stamped reason |

### W7 — Verification and observability (G8)
| iter | root cause | probe | exit criterion |
|---|---|---|---|
| **i413** | Citation drift is unchecked by the gate (i386 found 5 classes with `doc_index.py` green) | extend `scripts/doc_index.py` with a **citation check** (every `*.md|rs|py|sh` token in a governed doc must resolve from the repo root, the doc's directory, or a basename index; exit non-zero on a true ghost) | the gate fails on a planted ghost citation (the check is proven to trip, per the i170 subsystem-gate lesson) |
| **i414** | Longitudinal observability is partial: lineage/tail metrics landed at i251 with chart scaffolding, but the operator-facing longitudinal view is unfinished | `i414_charts`: finish the TUI longitudinal charts (population, stress, Gini, belief ecology, lineage) from cached metrics | charts render deterministically from a capture; zero mechanical effect (read-only pin) |
| **i415** | No standing perf envelope above N=96, and the per-pass profile is probe-only | `i415_envelope`: standing warn-only envelopes at 144/192/256 + a profile readout surfaced through the CLI | `scripts/gate` reports the envelope at the town tiers; a regression at N=192 is visible without running the probe by hand |

---

## 4. Sequencing and dependencies

```
W0 (i387→i390)  dead/inert surfaces  ── feeds the action layer for W2/W3
W1 (i391→i393)  relationship unification  ── must precede W2 (contact changes the store)
W2 (i394→i397)  space/contact  ── must precede W3 (diplomacy needs a settlement graph)
      i397 (A9) is an OPERATOR GATE: W2's clan/contact items can land either way,
      but the density decision should be taken before i398 measures a polity graph
W3 (i398→i401)  inter-community layer  ── the country path; i400 needs i398/i399 landed
W4 (i402→i405)  scale architecture  ── i402/i403 unblock i404 (cap) which unblocks i408 (H5)
W5 (i406→i408)  economy/demography  ── i406/i407 independent; i408 depends on i404
W6 (i409→i412)  thin layers  ── independent; the cheapest realism-yield per iteration
W7 (i413→i415)  verification  ── i413 is independent and can land any time (recommended early)
```

**Recommended first arc (the next five iterations, in order):** **i387 → i388 → i389
(DONE) → i390 → i391**, plus the new **office-succession** item i389 exposed (an Elder
who dies is never replaced — the pestilence leg's authority is structurally absent, and
succession blocks i399/i400's polity layer as much as it blocks this one). Rationale: i387 is a *measurement* that decides the action layer's fate; i388/i389
close two surfaces a probe has already named (a Class-D inert term and a nominal authority
channel); i390 sweeps the write-only-field class that has produced four separate dead-leg
bugs; i391 starts the migration whose reader-side is already half-done. All five are
low-blast, and together they remove the last known false affordances below the social layer —
after which W2/W3 can measure space and inter-community behaviour on a clean action surface.

---

## 5. Operator decisions this plan depends on

1. **A9 — fixed 32×32 vs constant density.** i344 measured it as a *fidelity* policy (max
   co-location 19→4, contacted share halved), not throughput. Adopting it changes calibrated
   village structure, so it is a charter call. **Needed before i397.**
2. **Cap raise (`MAX_POPULATION` 256 → 384/512).** Needed for H5's unlock (i408) and for any
   city-path claim. Requires a measured envelope (i403/i404) and a golden/harness review.
   **Needed before i404.**
3. **Does DC-5 own depth *and* scale, or should the city path be split out?** This plan
   proposes one cycle ("depth closure and inter-community scale") because the two are coupled:
   the inter-community layer (W3) is what makes town scale meaningful, and the LOD/cap work
   (W4) is what makes it run. If you prefer, W4/W5 become DC-6 and DC-5 ends at W3.
4. **The directive channel (i389): wire or delete?** Wiring gives offices real authority
   (a realism gain, a re-anchor cost); deleting is honest and free (i372 precedent). The plan
   defaults to **wire**, because the offices already exist and no other layer can express
   hierarchical authority.
5. **`Socialize` (i388): revive or re-contract?** i380's evidence says the term lacks the
   channels its siblings have; reviving buys deliberate sociality, re-contracting keeps the
   routine as the social carrier. Default: decide on i387's decomposition, not now.

---

## 6. Non-goals (explicit)

- **No magnitude-knob compensation** (§4.4 / i279). Ever.
- **No piecemeal founder-shape change**: it lands only with i404's larger N and the full
  coordinated sweep (§5 of AGENTS; Iter-263's 13 broken liveness pins are the receipt).
- **No sparse relationship store** as a perf fix: demoted by i326/i338/i344/i350. i395 is the
  *behavioural* question (do encounters, not co-residency, form ties), not a cache.
- **No `VecDeque` event conversion**: trigger is jitter-free ticks only (i354).
- **No planet-scale distributed execution**: i405 is a spike with a verdict, nothing more.
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
