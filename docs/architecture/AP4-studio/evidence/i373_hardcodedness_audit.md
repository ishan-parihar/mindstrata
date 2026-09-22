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
