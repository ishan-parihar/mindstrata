# Mindstrata — Exhaustive Development Audit + DC-2 Plan

**Date:** 2026-09-16
**Baseline:** HEAD `392e64a` (post DC-2 Q1–Q4 calibration), main, 790 commits.
**Method:** full git-history review (790 commits, AP1→AP2→AP3→AP4 arcs), docs survey
(ROADMAP, AP4 cycle plan, evidence ledger, balance canon), live verification run of every
gate the doctrine requires, plus targeted reads of the calibration debt sites.

---

## 1. Live verification results (measured at audit time)

| Gate | Command | Result | Verdict |
|---|---|---|---|
| Format | `cargo fmt --all --check` | clean | PASS |
| Lints | `cargo clippy --workspace --quiet` | 0 warnings | PASS |
| Full suite | `cargo test -p mindstrata-tests --lib --release` | **307 passed / 0 failed / 1 ignored**, 161 s | PASS (matches documented signature) |
| Quick gate | `scripts/gate` | GATE GREEN, golden 5/5 (0.42 s) | PASS |
| Full gate | `scripts/gate --full` | GATE GREEN @163 s | PASS |
| Probe law | `scripts/bench_index.py --strict` | 39 ok / 36 legacy / **0 violations** | PASS |
| Toolchain note | `rustup default stable` was unset | 1.98.1 installed to run gates | fixed (environment, not code) |

The one ignored test is the documented long-running trace harness
(`agent_lifetime_trace.rs`, run explicitly with `--ignored`). The 12 `mindstrata-tests`
warnings from `scripts/gate` are pre-existing lint noise in the tests crate, not failures.

**The suite is green at every layer. No development debt is being masked by a red gate.**

---

## 2. What has been completed effectively (evidence-backed)

### 2.1 The four architecture plans, in sequence

| Plan | Scope | Status |
|---|---|---|
| **AP1** | Core deterministic village sim (12-substrate layer cake) | DONE — `archive/AP1-implemented.md` |
| **AP2** (~4,060-line spec) | Deepening: every system wired to decisions | DONE — structurally ~99%→100% by Iter-186; ~170 iterations closed every write-only state, dead producer, and unwired consumer (Iters 82–186 closed the §8.1.10 norm loop end-to-end; Iters 190–235 killed every dead producer; Iter 186's exhaustive audit closed all 7 residual gaps) |
| **AP3** | Attractor-Field Architecture (KosmOS 17-stage ladder, 49 lines, 4 pathology quadrants, Λ-gating) | Eras I–II DONE (WP-A→H landed through i295); Era III content types (realm/template/referent/lore) landed as pure types; **Era III generation + Era IV collective activation remain** |
| **AP4** | Studio operating model (7 departments, DC cycles) | DC-1 **COMPLETE at 106/106 CLOSED** (v18 audit, release-tagged); DC-2 opened |

### 2.2 Scale & structure program (all golden-proven byte-identical)

- Crate ladder S1–S3 complete: `core ← person ← psych ← {social, institutions, world} ← sim`,
  cargo-enforced DAG; 112,777 LOC across the workspace; `mindstrata-development` leaf crate
  born with `deny(missing_docs)`.
- Sim god-file decomposed (17,942 lines → `sim/` hub + six verbatim tick passes);
  Arc-D pass extraction (bio+health → `systems/`) landed verbatim.
- Tests namespaced by domain (`integration_tests/{biology,psychology,social,culture,
  governance,economy,legal,infra}`).

### 2.3 Calibration state at HEAD

**Calibrated (probe-evidenced, ratified):**
- All 4 pathology quadrants live + behaviorally wired: Q1 0.11–0.32, Q2 0.21–0.63,
  Q4 0.21–0.63 at 5K/20 seeds (i293); per-quadrant operator params
  Q1 0.06/0.015/0.80, Q2 0.045/0.022/0.80, Q3 0.07/0.015/0.85, Q4 0.03/0.025/0.75 (66f753b).
- Allergy always-step semantics + 0.1× growth proven by i295 forced-engagement probe.
- Performance floors: N=12 10K 9,644 tps (21% headroom over 8,000), N=48 10K 753 tps
  (8% headroom over 700) — IC-8 PASS after cumulative event_count + ring cap + pre-alloc.
- Golden replay custody: 5/5 baselines byte-verified every gate run.
- Q1→Work/Rest (−0.08/+0.02), Q2→Socialize (−0.04), Q4→Worship (−0.04) nudges live
  and within i282 safe range; goldens re-anchored with mechanism.

**Structurally complete but inert (wired, awaiting activation):**
- `CollectiveField::step_collective` — per-village field mutates from a 4-bucket
  pressure vector but the dev-crate step is intentionally `ponytail:` inert (WP-I debt).
- Era III content grammar — `RealmTriple::is_legal` maps all-but-one triple legal
  "pending WP-I ontology cell mapping"; template/referent/lore are pure types not yet
  generating player-visible content.
- Institutions multiplier (SIM 4.25) — inert groundwork.

---

## 3. What remains — the honest calibration debt ledger

Every item below is **documented debt, not hidden debt**; all have filed evidence.

### 3.1 IC-5 canon values (DESIGN debt, probe-gated)

| Item | Site | Current | Needed |
|---|---|---|---|
| **i275 needs-band sweep** | `docs/balance/needs-bands.md` | 6 bands `CALIBRATION-PENDING(AP3)` | `i275_needs_band_calibration` must measure 11/12 bands with CV ≤ 0.35 to promote `CO-2026-003`; DESIGN 12/12 closed `ponytail:` on this |
| **Per-quadrant pathology ceilings** | `crates/mindstrata-development/src/dynamics.rs` | Spec midpoints ratified; fine-tuning open (#3) | `i<iter>_pathology_*` probes specialize growth/decay/ceiling per quadrant after Q3 event-rate regime is forced |
| **Q3 golden_addiction** | `dynamics.rs` + wiring | 0.03–0.04 at 5K (event-rate limited) | Forced Bond-event scenario (like i295 for Q2/Q4) to calibrate; Q3→action nudge wiring pending |
| **Lambda admission threshold** | `crates/mindstrata-development/src/lambda.rs` | `CALIBRATION-PENDING(AP3)` | Probe-measure smallest behaviorally-sensible threshold |
| **Catalyst magnitudes** | `catalyst.rs`, `catalyst_observers.rs` | Placeholders in [0,1] | Observer-harness sweeps |
| **Cross-line resonance** | `dynamics.rs:23` | same-line v1, cross-line zero | Vendor coupling data → affinity matrix |

### 3.2 Blocked-inert layers (structural debt, one batch)

- **WP-I — village collective field activation**: implement `step_collective` over
  8 collective lines; the highest-impact single conversion (inert layer → first
  per-village emergence source). Unblocks: STORY 14–15 collective behavioral,
  annals browsing depth, Era IV onset, transcendence needs-band (WP-I-dependent).
- **Era III content generation**: template → referent → `RealmTriple` rendering into
  annals/claims (UM-2 "the village tells stories" trajectory).

### 3.3 Known systemic hazards (AGENTS.md §5, standing)

- Fixed-4 truncation, mem::take write-back, midpoint neutrality, RNG stream discipline,
  founder-variance-as-budget (H5, closed as documented ceiling at N=12).
- Knife-edge pins recorded as debt (epidemic R0 resolved at Iter-252).

### 3.4 Performance ponytails (non-blocking)

- VecDeque events buffer (~20 sites, +5–8% N=48 headroom) — deferred.
- Hosted dashboards (TOOLS 5.16) — DC-3.
- 36 legacy bench names grandfathered by naming-law audit.

---

## 4. DC-2 Development Plan

Trajectory context: DC-1 closed at 106/106; the milestone ladder is
**DC-2 → UM-2 "the village tells stories"** (Era III content emergence, generated
cultures seed-disjoint) then DC-3 → UM-3 "the world scales" (Era IV holon, N≥48 budget).
The plan below follows AP4's loop: P0 contract freeze → parallel execution → UM gate.

### Iteration 266 (first DC-2 arc) — WP-I: activate the collective field

**Root cause owned:** the per-village emergence layer is wired but inert.

1. **Probe first.** `crates/mindstrata-benches/examples/i266_collective_wp_i_probe.rs`:
   measure `CollectiveField` altitude trajectories under the 4-bucket pressure vector at
   N=12/5K across the 12-seed family (i280's harness is the base — it already proves the
   wire is inert). Record per-line movement; expect zero today.
2. **Implement `step_collective`** in `crates/mindstrata-development/src/collective.rs`:
   8 collective lines, `all_lines()` ordering, pressure→altitude step reusing the
   4-quadrant operator shape (per-line `OperatorParams` marked `CALIBRATION-PENDING(AP3)`
   with the probe plan named). Respect zero-at-zero: empty window → identity.
3. **Wire a first behavioral consumer** (one only): village-mean collective altitude
   modulates ritual legitimacy gain or moral-panic pressure — probe-swept coefficient,
   `CO-` recorded, golden re-anchor if blast.
4. **Verify:** `gate --full` + golden 5/5 (byte-identical if consumer is inert in
   calibrated windows, else documented re-anchor), `i266` probe shows live field with
   seed variance, zero-at-zero pin passes.

### Iteration 267 — i275 needs-band calibration sweep (unblocks DESIGN debt)

1. **Probe:** `i267_needs_band_calibration.rs` — the named sweep from `needs-bands.md`:
   damage gate 0.8→0.95 (survival), safety retain 0.6→0.8, belonging generate 0.60–0.75,
   esteem scan 0.5→0.7, meaning gate 0.3→0.5 at seed 42/4242 family; measure health
   collapse days, Work/Wander ratio, Socialize/day, kinship edges, wealth Gini, Worship/day.
2. **Gate on the calibration-audit contract:** 11/12 bands with CV ≤ 0.35 → promote
   `CO-2026-003`, replace `CALIBRATION-PENDING(AP3)` with `RATIFIED v1.0.0` in canon,
   lift `difficulty-levers.md` from DRAFT.
3. **Verify:** full suite + golden; re-anchor only with mechanism evidence per §4.2.

### Iteration 268 — Q3 golden-addiction calibration + wiring

1. **Probe:** forced Bond-event scenario (the i295 pattern): shock N=12/20K villages with
   concentrated marriages/childbirths; measure Q3 trajectory vs the natural 0.03–0.04.
2. **Calibrate:** Q3 growth/decay/ceiling within spec band 0.70–0.90; specialize
   `OperatorParams` (Q3 ceiling 0.85 already ratified — tune growth to event regime).
3. **Wire Q3's behavioral nudge** (grasping → Socialize/Worship *positive* nudge per the
   spec note), sweep the coefficient into the i282 safe range, re-anchor goldens with
   mechanism.
4. **Verify:** `gate --full`; i293-style 20-seed sweep shows Q3 live with variance.

### Iteration 269 — Era III content: first generated content in the annals

1. Wire template → referent → `RealmTriple` into the claim-emission path (lore archetype
   is already wired to claims; close the loop so claims *render*).
2. Complete the ontology cell mapping for `is_legal()` (currently one illegal triple
   hardcoded) from the vendored KosmOS `realms.md`.
3. Zero-blast discipline: rendering is read-only over existing state; golden stays
   byte-identical unless a real consumer changes.
4. Probe: `i269_era3_content.rs` — generated entries appear, are seed-deterministic,
   seed-disjoint cultures differentiate (UM-2 criterion).

### Iteration 270+ — batch the structural ponytails + DC-2 continuation

- Lambda admission threshold + catalyst magnitude sweeps (both small, one iteration).
- Cross-line resonance matrix when vendor coupling data lands.
- VecDeque events buffer perf ponytail (only if N=48 headroom binds DC-2 probes).
- Then per the roadmap: DC-2 P0 (contract freeze for the full 106-phase cycle), Era III
  completion, DC-3 planning (N≥48 scale, asset pipeline v0).

### Standing discipline (applies to every iteration above)

- One root cause per iteration; probe before touching; full gate before commit;
  `scripts/gate --full` before push; re-anchors carry measured/old/mechanism;
  knife-edge pins recorded as debt, not flip-flopped.
- Check `git log --oneline -3 && git status` before every session (shared-clone hazard).

### §4 EXECUTION LEDGER — DC-2 landed (2026-09-16)

All five planned iterations executed per doctrine, each ending in a full green gate
+ push:

| Iter | Commit | What landed | Evidence |
|---|---|---|---|
| 266 | `92352fb` | **WP-I collective field alive**: real `step_collective` (press integration, stage advance, fulfillment EMA), vault-`kind` slug-bucket mapping replacing cyclic `i%4`, Safety-fulfillment panic-pacify consumer (anchor 0.25 > calm-window peak 0.19), snapshot v15 + roundtrip pin | `i266_collective_wp_i.md` |
| 267 | `c09543d` | **Dead-producer revival** (§5 Fixed-4 truncation class, Iter-242 family): esteem/autonomy decay computed in f64 + quantize once; Work/Trade relief paths wired; CO-2026-003 landed (0.3695 ± 0.026, CV 0.070) | snapshot re-anchor +0.0002 legitimacy, mechanism named (autonomy → grievance → faction pressure) |
| 268 | `bd6977c` | **Q3 golden-addiction**: forced-Bond probe shows per-subject equilibrium ≈0.75 inside ratified band (no re-pin needed); grasping nudge wired (+0.04 Socialize/Worship) | goldens re-anchored — social memories +65, Friend 0→2, grain 1.81→0.91 (the grasping cost) |
| 269 | `9bcdeb7` | **Era III lore rendering** (UM-2): `render.rs` bridge (legality-gated, cite-first, FNV-stable template selection), chronicle "The lore of the village" annal | `i269_era3_lore_rendering.md` |
| 270 | `989bfca` | **Lambda gate ratified** t=0.05 (probe: all live magnitudes ≥0.311, threshold-invariant across 15× band); catalyst magnitudes censused + ratified; referent doc-test rot fixed | `i270_lambda_catalyst.md` |

**Residual debt (recorded, honest):** full Era III legality mapping blocked at vendor
source (`realms.md` not vendored — WP-I vendor coupling); Grief/Transgression catalyst
magnitudes remain spec-midpoint pins until a mortality-horizon probe can observe them;
`admit(0.35)` is the future pacing knob (would gate 40% of Threat pressure — not pulled
without evidence); `referent.rs`/`line.rs` doc-example classes now green (3/3).

---

## 5. Summary assessment

**Development completed effectively:** the full AP1→AP2 system build (~170 iterations,
1,100+ tests), the AP3 Era I–II attractor-field engine with all four pathology quadrants
calibrated and behaviorally wired, the complete crate-ladder scaling program, and the
entire DC-1 106-phase studio cycle. Every verification gate is green at HEAD with zero
violations.

**DC-2 is now EXECUTED (see §4 ledger):** the deliberately-inert collective layer is
alive (WP-I), the last two dead producers found are revived (esteem/autonomy decay), Q3
is calibrated and behaviorally wired, the village tells its stories (Era III rendering,
UM-2), and the lambda gate + catalyst magnitudes are probe-ratified. The canon's
remaining open items are blocked at the vendor boundary, not at construction —
external inputs (full ontology mapping, mortality-horizon observations) are the only
path to the rest.

**Next horizon (per roadmap):** DC-2 P0 contract freeze for the full 106-phase cycle,
DC-3 planning (N≥48 scale, asset pipeline v0), Era IV collective activation, and the
VecDeque perf ponytail if N≥48 probes bind.

---

## 6. DC-2 CONTINUATION PLAN — Era III completion → UM-2 gate (post-i274)

**Date:** 2026-09-17 · **Baseline:** HEAD `0e7c93e` (i274 Relational feed revival), main.

### 6.1 Live verification at audit time (2026-09-17)

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --quiet` | 0 warnings |
| `cargo test -p mindstrata-tests --lib --release` | **307/0/1** (147.8 s) |
| `mindstrata-development` lib | 80/80 |
| `mindstrata-sim` lib | 221/221 |
| `scripts/gate` (quick, golden 5/5) | GATE GREEN |
| `scripts/bench_index.py --strict` | 48 ok / 36 legacy / 0 violations |

### 6.2 Where DC-2 actually stands (honest vs the UM-2 gate)

The §4/§5 ledger records Iters 266–270. Since then (this session's arcs):

| Iter | Commit | Landed |
|---|---|---|
| 271 | `b2163ad` | Arc-D batches 2+3: appraisal/decay/cognitive passes verbatim into `systems/` |
| 272 | `734b14b` | GriefStruck: grief routes to surviving kin, not replacement newborns |
| 273 | `1dc7e9b` | Era IV activation: stage-gated collective meme genesis (`systems/genesis.rs`) |
| 274 | `0e7c93e` | Relational feed revival: `RitualPerformed` → per-participant Bond catalysts |

**UM-2 exit gate status (AP3 04-waves Era III): "seed-disjoint cultures in 20K-tick
probe; old roster retired."** Not yet met: `seed_initial_memes` still seeds the pool
(population.rs:112), genesis only *appends* bucket-stage memes, and the polarity graph
never reconciles (`reconcile_claims` is pure with zero sim call sites — a dead producer
of the claims graph). The remaining DC-2 work is therefore exactly WP-G2 + WP-H2 + the
tetra-arising half of WP-I, then the i284-style disjointness probe as the gate.

### 6.3 Iteration ladder (one root cause each, doctrine §2)

**i275 — WP-H2: wire polarity reconciliation (the claims-graph dead producer).**
`reconcile_claims`/`in_active_tension`/promotion helpers sit in the dev crate unused;
claims accumulate per-agent but never synthesize or refute. Probe first
(`i275_polarity_reconciliation.rs`): measure claim inventory, same-line collision rate,
and what a minimal reconciliation pass (per-agent same-line pairs, gated by the ratified
lambda admission) would cascade. Wire the smallest consumer that makes
Undiscovered→ActiveTension→Integrated transitions *observable*; pin zero-at-zero.
Probe target: refutation storms produce measurable norm churn then re-crystallization
(the i288 panic-cascade shape, at whatever horizon N=12 supports).

**i276 — WP-G2: compositional content generation (retire the fixed roster's primacy).**
Extend `systems/genesis.rs` from bucket-stage memes to the substrate §5 shape:
domain selection weighted by live collective-line stages × gross referents (existing
sites/institutions/events) × stance from the (now reconciling, post-i275) polarity
state. Every generated item type-checks: cites ≥1 gross entity within exactly one
domain (the legality gate already in `render.rs`). Probe: generated items appear,
are seed-deterministic, cite-valid; roster composition shifts vs the seeded baseline.
Zero-blast discipline: golden horizons keep all lines at 1.0 → generation inert there.

**i277 — WP-I tetra-arising gate: collective stages gate content classes.**
The substrate's collective-activation rule: stage bands on collective lines determine
WHICH content classes the generator may emit; the individual line distribution gates
UPTAKE weight. Wire the read-side gate (midpoint-neutral, zero-blast below stage 2 —
i273's identity floor does most of this already). Probe: forcing a stage band upward
(forced-stage scenario, i295 pattern) opens exactly the gated content class.

**i278 — UM-2 GATE: seed-disjoint cultures probe (`i278_culture_disjoint.rs`).**
Two+ seeds, same founder bands (per audit H5 the founder budget is uniform draws at
N=12), 20K ticks: meme rosters diverge (jaccard < threshold — set it from the measured
cross-seed overlap of the SEEDED pool first, so the gate measures generated divergence,
not seed vocabulary). If pass → UM-2 evidence file + DC-2 unify review. If fail → the
probe names which feed starved (Safety dominance is the known risk; i274 pacing math
says Relational reaches stage 2 only at ~80K).

**i279 — WP-J: institutional altitude coupling.**
Institution behavior parameters become read-side functions of governance/economic-systems
line stages (midpoint-neutral per §5). Probe `i279_institution_shift`: village crossing
amber→green on the governance line (forced-stage) shows pluralistic institution deltas.
Golden stays byte-identical below the crossing band.

**i280 — Identity-bucket feed (the last dead collective producer).**
Grief is mortality-blocked (i272: first natural death ~2.3M ticks). Options, probe-gated:
(a) festival/collective-participation events carry Identity press (substrate table maps
ritual participation to culture-line catalysts — Relational got this in i274; Identity
could take a fraction), (b) grief-proxy events (separation/betrayal at N=12 horizons).
Probe measures which feed produces Identity press without breaking the i274 pins; pick
one, record the other as debt.

**i281 — perf/DC-3 entry.** VecDeque events buffer (~20 sites, +5–8% N=48 headroom)
if the Era III probes bind; else DC-3 P0: N≥48 perf budget doc (i274 baseline: 102 µs/tick
@ N=12 → 928 @ N=48 → 5696 @ N=96, superlinear) + asset pipeline v0 charter.

**i282 — needs-band transcendence sweep (now unblocked).** The self-transcendence band
was deferred "until CollectiveField lands" (needs-bands.md) — it landed at i266. Run the
named sweep; promote or record honestly.

### 6.4 Standing discipline (unchanged)

Probe before touching; one root cause per iteration; full gate before every commit;
`gate --full` before push; re-anchors carry measured/old/mechanism (§4.2); knife-edge
results recorded as debt, not flip-flopped; `git log --oneline -3 && git status` at every
session start (shared-clone hazard).

### §6 EXECUTION LEDGER — continuation arcs landed (2026-09-17, session 2)

| Iter | Commit | What landed | Evidence |
|---|---|---|---|
| 275 | `587970c` + `b747479` | **Polarity graph live**: probe overturned the plan (reconcile pass WAS wired; projection starved it). Fixed TWO dead edges — severity-grounded Threat projection (minor→Fact / major→Identity on one slot) + the mutually-exclusive gate mismatch (`is_active_tension` ≠-domain vs `reconcile_subtle` =-domain — the pair scan could NEVER fire). Action bias re-derived 0.10→0.01 (the DC-2.4 ramp tuned a dead diet; live-diet bias was 10× the audited band). Tension 15/33/206/910, integrated 4/4/40/82 at 1K/2K/5K/20K | `i275_reconciliation_fix.md` + probes |
| 276 | `b84b8e1` | **Referent-grounded genesis** (WP-G2): generated culture cites real sites/institutions, domain-scoped (institution buckets cite institutions; communal buckets cite non-House sites), epoch-rotation binding, zero-blast gate | `i276_referent_grounding.md` |
| 277 | `d90ea77` | **Tetra-arising band gate** (WP-I): stage bands gate content classes (band II ≡ pre-277, III unlocks Political/Song at 4.0, IV Prophecy at 6.0); dedup tag corrected to carry the class | `i277_tetra_arising_gate.md` |
| 278 | `32a6c83` | **UM-2 gate probe: PARTIAL PASS** — seeded control 1.000, generated mean jaccard 0.402, 11/15 pairs disjoint; diversity confined to the Safety bucket = recorded diet debt (i272/i274), not construction. Roster retirement deferred until ≥3 buckets generate | `i278_culture_disjoint.md` |

**New debt recorded this session:** bias reads a horizon-integral (recency window deferred until a 5K+ behavioral pin exists); identity-bucket feed remains the one dead collective producer (mortality-blocked; i280 options listed in §6.3).

### 6.5 Known boundaries (not plannable until inputs land)

- Full Era III legality mapping — blocked: vendor `realms.md` not vendored.
- Grief/Transgression catalyst magnitude ratification — blocked: mortality horizon.
- Cross-line resonance matrix — blocked: vendor coupling attestation.
- VecDeque refactor cost/benefit — re-decide at i281 with fresh N=48 headroom numbers.
