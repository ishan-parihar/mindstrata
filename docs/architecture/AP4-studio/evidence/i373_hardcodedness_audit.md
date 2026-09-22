# i373 — the hardcodedness audit: what should become organic, what must stay a constant

**Status:** LANDED (audit + doctrine) · **Method:** enumerated every `const` in the
production crates (sim, world, person/biology, psych, institutions — 150+ sites), then
classified each by one question: **is there a principled law that derives it from
simulator state or operator intent?** If yes → organic candidate (queued with its
re-anchor cost). If no → the constant IS the calibration (realism-preserving), and
hardcoding it is correct.

The framing follows the charter decision "organic values wherever possible so that
everything in the simulation has its own mind": organic ≠ parameterless; it means the
value is a *function of state* (endogenous) rather than a *number fixed at compile time*
(exogenous). Determinism is unaffected either way — both are pure functions of the tick.

## Class A — already organic (the design philosophy, working)

The engine already derives many load-bearing values from state; listed to show the
pattern is established, not aspirational:

| Law | Derives from |
|---|---|
| `houses_for_population(N)` | population |
| `cluster_count_for(houses)` | housing (seed-time; superseded by fission as the count law) |
| Agent tiers (Focal/Secondary/Background) | fear/anger/status/importance gradient |
| Market prices | demand; demand-driven repricing |
| Epidemic R0 | contact structure |
| Faction crisis pressure | accumulated grievance excess |
| Council legitimacy equilibrium | sentiment (§29.2) |
| Treasury hoard equilibrium `T* = R + I/s` | inflow & payout (i363/i370) |
| Council composition on coup | dominance/conscientiousness scores (i370) |
| Narrative importance | contacted degree + crisis (i348) |
| Genesis polity count | settlement structure (i360) |

## Class B — ORGANIC CANDIDATES (queued, each sweep-carrying)

These are hardcoded where a state-driven law exists in the sim already; each is a
candidate to become endogenous. Ordered by expected realism yield.

1. **`COUNCIL_SURPLUS_DIVIDEND_SHARE = 0.25`** (`institutions_impl.rs`) — the strongest
   candidate. Make the payout **legitimacy-coupled**: `s = s₀ + k·(1 − legitimacy)` — a
   nervous/resented council buys goodwill with patronage; a secure one hoards. The hoard
   then becomes a state variable with feedback (hoard → inequality grievance →
   legitimacy ↓ → payout ↑ → hoard ↓). The treasury gets "a mind of its own" and the
   last magic number in the economy goes. Re-anchor: the i365 six-seed Gini band is the
   contract to hold.
2. **Tax/tithe/fee rates** (`COUNCIL_TAX_RATE`, `MARKET_FEE_RATE`,
   `TEMPLE_TITHE_RATE`) — hardcoded policy stances. In a society where the council is an
   office with legitimacy, the *council should debate its own rate* (endogenous policy:
   rate set by legitimacy pressure, war/famine need, or faction composition). This is
   the highest-realism item but also the largest sweep: every wealth pin moves. Queue
   behind the dividend coupling.
3. **`MAX_MARRIAGE_DISTANCE = 20.0`** (`marriage.rs`) — a hard radius on courtship in a
   world whose size is now adaptive (i360 clusters, i371+ fission). Should derive from
   the settlement structure: partners come from the same settlement or an adjacent one
   (endogenous to geography). Small sweep, high coherence.
4. **`PATRONAGE_MAX_CLIENTS_PER_PATRON = 3`** — a patron's *capacity* should scale with
   status/wealth (the rich attract more clients — that IS the mechanism), not be a
   constant. Small sweep.
5. **`CULT_FORM_MEMBERS = 4`, `PATRONAGE_*` thresholds, `FEUD_APPROACH_ANGER`** — these
   are *individual disposition thresholds*; the honest organic form is to derive them
   from the agent's own traits (e.g. a high-openness agent forms/joins cults sooner).
   Realism yield is real but diffuse; queue last.
6. **`TIER_MATURITY_TICKS = 900`** — a pacing constant tied to the importance
   convergence timescale; should derive from the smoothing rate it protects. Low
   priority (invisible in behaviour).

## Class C — REALISM-PRESERVING constants (keep hardcoded, with reasons)

These look like the Class-B pattern but **must not** become endogenous, because the
hard number *is* the natural law being modelled, or because endogenizing creates a
feedback loop with no anchor:

| Constant | Why it stays |
|---|---|
| Biology rates (`BASE_EMOTION_DECAY_RATE`, `STRESS_RECOVERY_TONE_FLOOR`, `NERVOUS_TONE_*`, `INNATE_IMMUNITY_FLOOR`, `CHRONIC_FEEDBACK`) | Human physiological constants. Making stress-recovery depend on village mood would model a mechanism we don't have; the realism comes from them being *fixed* while the world moves. |
| Genome `MUTATION_NOISE = 0.06` | A mutational rate is exogenous by definition — it must not feed back from state or heredity loses meaning. |
| `CONCEPT_DIMENSIONS = 12` (+ the D_* axis set) | The *representational basis* of cognition (safety, sacredness, status, kinship…). Changing it per-state would make beliefs incomparable across agents/ticks. This is the ontology, not a parameter. |
| `MAX_POPULATION = 256`, `MAX_EVENTS`, buffer caps | Performance envelopes. Deterministic, operator-visible, and required for golden replay; endogenizing a memory cap would make the sim's own footprint chaotic. |
| Escalation-rate family (`HUMILIATION/CONTEMPT/MORAL_OUTRAGE = 0.3`) | Emotion-architecture constants with the same status as decay rates — the *shape* of human affect. |
| `NORM_FIRST_EXPOSURE_FLOOR`, `HYPOCRISY_ENFORCEMENT_CAP` | Social-cognitive invariants of the norm model, probe-anchored; endogenizing would let norms destabilize their own enforcement. |
| `TRADE_DIFFUSION_DAMP`, `LOGISTICS_PRICE_DAMP` | Numerical-statement damping (solver params), not behaviour; organic variants would be numerics pretending to be psychology. |

**The doctrine line (added to AGENTS §4):** a constant is *realism-preserving* when the
hard number is the modelled natural law (physiology, mutation, ontology, solver
damping) or a performance envelope; it is an *organic candidate* when a state variable
in the sim already measures the same thing the constant is guessing at. Every Class-B
promotion is behavioural and sweep-carrying; every Class-C endogenization attempt is
refused by default and needs evidence that the feedback loop anchors.

## Verification

No code changed in this iteration (audit + doctrine only). Suite state carried:
sim **300/300**, tests **310/0/1**, gate GREEN.

---

## Refresh (i377) — post i374 / i375 / i376

The classification is living: every landed organic promotion moves an entry from Class B
to Class A, and every behavioural iteration that measures a *new* gain/law adds one. Three
promotions landed since the first pass, and i376's probe surfaced two items the first pass
classified too coarsely.

### Class A additions (state-derived laws now live)

| Law | Derives from | Landed |
|---|---|---|
| Council dividend share `s = s₀ + k·(1−legitimacy)` | council legitimacy | i374 |
| Patronage capacity `cap = 3 + floor(effective_status × 4)` | patron's effective status | i375 |
| **Legacy v1 trust row** `v1.trust += (v2.trust − v1.trust) × rate` (daily) | **the dyadic `RelationshipV2` store** | i376 |

The i376 entry is the purest instance of the audit's test — *"does a state variable already
measure what the constant guesses at?"* The constant was a hardcoded `0.5` neutral
baseline; the state variable that already measures the right thing was **the other store of
the same quantity**. When the answer is "a parallel store holds it", the organic form is
convergence, not a stronger reversion: the probe showed retuning the rate (0.001 under the
new baseline) still saturates, while changing the baseline cuts the divergence 4× and
roughly doubles the store's discriminating band.

### Class B — remaining queue (unchanged order)

1. **Tax/tithe/fee rates** (`COUNCIL_TAX_RATE` 0.05, `MARKET_FEE_RATE` 0.03,
   `TEMPLE_TITHE_RATE` 0.02, `institutions.rs`) — the council should set its own rate.
   Highest realism yield, largest sweep (every wealth pin). **Design constraint found while
   scoping:** coupling the rate to *legitimacy* creates a positive feedback (low legitimacy
   → higher rate → line-549 legitimacy erosion → lower legitimacy). The anchored form is
   **fiscal need** (treasury vs its operating target), which is self-stabilizing. Do not
   couple it to legitimacy.
2. **`MAX_MARRIAGE_DISTANCE = 20.0`** (`marriage.rs`) — derive from the settlement
   structure (same settlement or adjacent), now that geography is adaptive.
3. **Trait-derived disposition thresholds** (`CULT_FORM_MEMBERS = 4`, `FEUD_APPROACH_ANGER`,
   patronage thresholds) — the honest organic form is the agent's own traits.
4. **`TIER_MATURITY_TICKS = 900`** — should derive from the smoothing rate it protects.
   Low priority (invisible in behaviour).

### Class B **newly found** by i376's probe (the audit's second payoff)

These are *divergent gains for one quantity* — the same class as the constants above, but
found by measuring a store rather than by grepping a `const`:

5. **The v1 interaction gains** (`social/interaction.rs` per-kind deltas +0.02…+0.10,
   `marriage.rs` +0.2 on marriage, `economy.rs` +0.02 on trade). The dyadic store applies
   `record_positive`'s `magnitude × volatility × 0.02` — about **10× smaller** for the same
   act. That is why the convergence leaves a residual +0.06–0.08 offset: the legacy row
   sits one day of (larger) gains above the store it now follows. Two candidate organic
   forms: (a) derive the legacy gains from the dyadic ones, (b) finish the migration and
   delete the legacy write path — which is the charter's stated end state. **Queued with
   the offset measured.**
6. **`social_reciprocal_factor`** (`interaction.rs`) — a second, independent gain applied
   only to the reciprocal v1 row, with no dyadic counterpart. Same class: a per-quantity
   gain that should either be derived or disappear with the legacy path.

### Class C — one promotion into, and one clarification

- **`relationship_dormant_decay` is now Class C.** Re-purposed by i376 from a behavioural
  reversion strength to a **convergence timescale** (0 = frozen, 1 = full sync each day).
  Under doctrine §4.6 this is solver/damping numerics, not a modelled disposition — the
  same category as `TRADE_DIFFUSION_DAMP`. Endogenizing a timescale would only produce
  numerics pretending to be psychology. Refused by default.
- **Test scaffold is not simulator configuration.** The seed *families* that probes and
  integration tests re-anchor (panic seeds, golden-window seeds, liveness seeds) look like
  hardcoded values but are not: they select which already-emergent world a pin is measured
  on. Re-anchoring them is legitimate **only** with a sweep — and i376 did exactly that
  (10-seed panic sweep → family `{7,11,46}`→`{7,1,23}`; seeds 1–70 sweep → golden window
  back to canonical seed 42). The rule that keeps this honest: never re-anchor a family
  without a runnable sweep, and record the *mechanism* that moved it.
- **The panic channel's trust-range coupling is recorded as debt, not a constant.** i376
  halved panic firing density (3/5 → 3/10 swept crisis seeds) by de-saturating the trust
  field. That is a *coupling* to re-examine in its own iteration (the rumor/credibility
  path's dependence on the wide trust range), and explicitly not a threshold to re-pin.

### Method note

The first pass found candidates by enumerating `const` sites. The higher-yield method this
round was to **probe a store and ask whether its equilibrium is doing work** (doctrine
§4.3). That found a 14×-mistuned gain, a saturated reader family, and two divergent-gain
constants — none of which a `const` census would surface, because they are *numbers inside
call sites*, not named constants. Future refreshes should alternate the two methods.

---

## Refresh (i378) — the panic threshold, and a knife-edge that no earlier class held

The i373 census enumerated 150+ `const` sites; this one was in the list but unclassified,
and i378 shows the reason classification alone would not have caught its *problem*.

### New Class B candidate — the panic trigger's threshold

`MORAL_PANIC_CHARGE_THRESHOLD = 0.55` + `panic_ratio ≥ 0.30` (`gossip.rs`) — the biggest
organic candidate the first two passes missed. It is an **absolute** threshold on a
population-level distribution, which is why the seed family that registers moves on every
pacing shift (i343, i351, i376; i378 measured the firing legs clearing by only 2.5–9.4%
with one swept seed missing by 0.5%).

**The organic form is a relative / anomaly-based trigger:** a panic when the population's
charge is anomalous against *its own* baseline (with an absolute floor against quiet-world
noise). That satisfies the i373 test literally — the state variable that already measures
the right thing is the population's own charge history — and, unlike the absolute form, it
is **structurally free of knife-edge**, because a relative threshold adapts to the
distribution it is testing instead of sitting inside it. Queued as its own iteration
(behavioural, sweep-carrying); sized by `i378_panic_threshold_headroom`.

### The class this reveals: a new category, not a Class C

The existing taxonomy asks "is there a principled law that derives this from state?" It did
not carry a slot for *"the number is fine, but it is an absolute threshold on an
endogenous distribution, so its effective operating point drifts with the distribution."*
That is a distinct defect class — **threshold-on-a-moving-distribution** — and its symptom
is a pin family that keeps moving (the §4.5 knife-edge family), which is easy to
misdiagnose as "the producer died" and repair in the wrong direction.

**Rule of thumb added to the refresh above:** when a pin's *membership* keeps changing but
the mechanism demonstrably still fires, measure the trigger's **headroom** before touching
anything. 2–9% margin ⇒ the threshold is the fault (endogenize it, or restructure the
contract); a large negative margin ⇒ the producer is starved (revive it). The two look
identical from a firing count and want opposite fixes.

Note that neither the `const` census nor the store-equilibrium probe would have produced
this item alone: the census lists the constant, and the probe (i376) surfaced the symptom
(the re-anchored family) — it took both, plus a third measurement (headroom) to classify it
correctly.

---

## Refresh (i379) — the third method, and the four classes it settles

A **gate selectivity census** now sits alongside the `const` census (i373) and the
store-equilibrium probe (i376): measure the share of samples that OPEN each shipped gate,
over a **distribution type × context** grid. Full table and analysis:
`evidence/i379_gate_selectivity_audit.md`.

Why a third method was needed: the first two cannot see a gate whose *reachable band* sits
entirely on one side of its threshold. i379 found three such gates — `needs.social > 0.30`
(p95 0.03 in every world), `emotions.anger > 0.50` (p95 0.005–0.071 at town scale) and
`council legitimacy < 0.50` (never fires; the legitimacy band is 0.54–0.69) — plus one
near-dead at the calibrated N (`v1 trust > 0.60` opens 82.9% at N=12 but 42.3% at N=48,
which is how i376's fix is only visible at scale).

The refreshed classification replaces "Class B = organic candidates" with four rules:

1. **Quantile gates over the fixed founder draw** — trait thresholds are self-balancing
   (measured 31–54% open, *stable across all four worlds*). Keep hardcoded; their robustness
   comes from the uniformity of the draw, which §5's H5 already protects.
2. **Crisis gates on bounded need scales** — dark in calm IS the semantics (`hunger > 0.85`
   opens only in famine). Keep hardcoded. **Always read a 0.0% open-rate against the world
   that should open it.**
3. **Gain/scale mismatches** — a gain whose *common case* rounds to nothing
   (`needs.social × …` contributes ~0.5% of typical action utility; i376's v1 trust gains
   were ~10× the dyadic ones). Rescale or derive. This is the class that hides behind
   passing tests.
4. **Absolute thresholds on self-driven aggregates** — the operating point drifts with the
   distribution (panic charge i378; the faction legitimacy arm). Make relative/anomaly-based.

The queue this yields is in the i379 doc §"implementation queue": the `needs.social` utility
scale, the anger arm of the negativity channel, the faction legitimacy arm, and the
relative-trigger redesign.

---

## Refresh (i380) — the first Class-3 item is measured, and its two obvious repairs are refuted

i379's Class 3 (gain/scale mismatches) predicted that `needs.social`'s action-utility term
rounds to nothing. `i380_socialize_reachability` confirms it is not merely small but
**inert**: across 112 669 utility arbitrations the utility leg selected `Socialize` once
(0 at N=12, 1 at N=48), and all 12 645 social decisions came from the daily routine.

The audit's *diagnosis* held; the audit's *implied repair* did not. Both principled
single-knob fixes were measured and both fail:

* a `×6` gain in family with the 2.0/2.5/1.5 siblings → 0 and 2 selections;
* a `×20` social-need accrual (band p50 0.006→0.096, p95 0.021→0.292) → 9 of 19 359.

**Class 3 therefore splits in two, and the distinction matters:**

* **3a — scale mismatch** (the term is small but the channel is alive): rescale and pin the
  outcome. Still the right model for i376's v1 trust gains, where rescaling *did* work.
* **3b — structural argmax disadvantage** (the candidate loses on the terms it does NOT
  have, not on the one it does): rescaling is a decoy. `Socialize`'s measured bar is the
  0.692 mean winner utility, and no plausible value of its own term reaches it, so the
  repair belongs to whichever channel (goal-alignment, dominant-need urgency, relief
  mapping) its siblings have and it lacks.

**Rule of thumb added:** before rescaling a term that "rounds to nothing", compute the
measured WINNER's utility and ask whether any plausible value of the term reaches it. If
not, the term is not the defect and rescaling is wasted work — the census is the instrument
that answers it (`winner_utility`, already recorded per arbitration).
