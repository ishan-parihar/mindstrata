# Runbook — Calibration Audit Checklist v2

Owner: QA (producer probes co-owned with SIM). Supersedes the ad-hoc audit method in
`docs/AUDIT_2026-08-22_EMERGENT_REALISM.md` for per-iteration gating; that doc remains
the deep-audit methodology.

Governing doctrine: AGENTS.md §4 (calibration honesty rules), §5 (systemic hazards).
Adjacent contracts: IC-4 (probe conventions), `runbooks/golden-replay-custody.md`.

## When it runs

Per behavioral iteration, before `scripts/gate --full` (AGENTS.md §2 step 4/5). Pure
refactors skip CA-1…CA-5 but still run CA-6…CA-8 static sweeps (byte-identical goldens
are their own referee).

## Severity scale

| Level    | Meaning                                                        |
|----------|----------------------------------------------------------------|
| CRITICAL | Blocks push. Producer/hazard class — system is lying or dead.  |
| HIGH     | Blocks push until fixed OR recorded as systemic debt w/ plan.  |
| MEDIUM   | Evidence-integrity debt; must close in the next iteration.     |

## Rule index

| ID   | Rule                                   | Source        | Severity |
|------|----------------------------------------|---------------|----------|
| CA-1 | No lucky-seed re-pins                  | §4.1          | HIGH     |
| CA-2 | Dead-producer equilibrium probes       | §4.3          | CRITICAL |
| CA-3 | Re-anchor comment completeness         | §4.2, IC-4    | MEDIUM   |
| CA-4 | Knife-edge flags become debt entries   | §4.5          | HIGH     |
| CA-5 | Founder-variance budget                | §5 (Iter-263) | CRITICAL |
| CA-6 | Fixed-4 truncation disease             | §5            | CRITICAL |
| CA-7 | mem::take write-back discipline        | §5            | CRITICAL |
| CA-8 | RNG append-only stream order           | §5            | HIGH     |

---

## CA-1 — No lucky-seed re-pins

**Statement.** A pin that passes only on its anchor seed after a sweep indicates a broken
system (gate/hazard design), never a seed choice. Re-pins move only with family-level
evidence (see sweep-runner contract below).

**Detection.**
1. *Anchor-seed churn (static):*
   ```sh
   git log -p --follow --since="<10 iterations>" -- crates/mindstrata-tests/src \
     | rg '^\+.*(re-anchor|re-anchors).*seed'
   ```
   Count distinct anchor seeds cited per test symbol. >2 distinct anchors for one pin in
   10 iterations ⇒ flag.
2. *Family pass-rate (dynamic):* run the seed-family sweep runner (below) scoped to the
   pin being re-anchored.

**Pass criteria.**
- Family pass-rate ≥ 11/12 seeds BEFORE the re-anchor lands; the re-anchor comment cites
  the sweep output, not the single anchor run.
- Anchor-seed churn ≤ 2 distinct seeds per pin per 10 iterations.
- A re-pin justified by "only seed N holds" is rejected outright (§4.1; Iter-236
  precedent: 24 red shipped).

**Severity.** HIGH.

---

## CA-2 — Dead-producer equilibrium probes

**Statement.** A producer is dead if its driven state saturates at a bound (fear pinned
at 0.99 everywhere) or collapses to zero spread, even when assertions pass on the
saturated state. Probe equilibrium *distributions*, not just assertions.

**Detection.**
1. *Saturation probe (dynamic, per audited producer):* sample the bounded variable
   (fear/stress/trauma/needs/affect) across all agents at horizon end and across a tick
   window. Compute `sat_frac` = fraction of samples in [0.95, 1.0] ∪ [0, 0.05], and
   temporal std over the window.
   FAIL if `sat_frac > 0.90` **and** window std < 1e-3.
2. *Sub-resolution knob scan (static):* constants below Fixed half-resolution
   (5e-5; `SCALE=10_000`) consumed via `Fixed::mul` per tick quantize to zero —
   the Iter-176 trauma-knob failure mode:
   ```sh
   rg -n 'Fixed::from_f64\(0\.0000[0-4]' crates/ --type rust
   ```
   Every hit must trace to an f64-domain computation with a single terminal quantize
   (or an f64 shadow accumulator), else flag.

**Pass criteria.**
- Producer shows non-degenerate spread at horizon: p10–p90 width > 0.02 (or the pin's
  documented envelope) and `sat_frac < 0.90`.
- Knob tunability: a 3-point parameter sweep moves the equilibrium monotonically (the
  Iter-176 proportional-decay pattern), i.e., the knob has a working range.
- No sub-resolution constant reaches `Fixed::mul` unguarded.

**Severity.** CRITICAL.

---

## CA-3 — Re-anchor comment completeness

**Statement.** Every test re-anchor carries the full IC-4 evidence form:
`measured <value> via <probe-name> at <horizon>, seed(s) <seeds>; old band <x>;
mechanism <why it moved>` — and states explicitly whether it is a **re-pin**
(magnitude drifted, contract unchanged) or a **re-contract** (old assertion invalid;
new invariant guarded per §4.4).

**Detection.**
```sh
rg -n 're-anchor' crates/mindstrata-tests/src --type rust -B2 -A6
```
For each hit, verify presence of: (a) iteration id, (b) measured value, (c) old band /
prior value, (d) mechanism clause, (e) probe citation matching an existing file in
`crates/mindstrata-benches/examples/`, (f) the words re-pin or re-contract.
*(Assisted-manual: prose forms vary across the legacy corpus; a lint script lands with
bench-index tooling. Until then this is a checklist pass, not a pure exit-code check.)*

**Pass criteria.** All six fields present. Missing (b)/(c)/(d) = "widen until green"
smell — reject. Missing (f) = unverifiable citation — reject. Legacy comments predating
this runbook are grandfathered until next touch.

**Severity.** MEDIUM.

---

## CA-4 — Knife-edge flags become debt entries

**Statement.** A pin sitting on an unstable equilibrium (epidemic R0≈1 flipping
TRANSIENT↔ENDEMIC nine times) is systemic debt. It is recorded once, with a plan —
never flip-flopped pin-by-pin.

**Detection.**
1. *Outcome-class instability (dynamic):* sweep runner reports the outcome-class label
   per seed. Mixed labels across the family at constant code ⇒ knife-edge.
2. *Debt-entry existence:* search for the greppable tag:
   ```sh
   rg -n 'SYSTEMIC-DEBT\(knife-edge\)' docs/
   ```
   Every flagged pin must have a matching entry naming the pin, the flip signature, and
   the structural fix direction (hazard accumulation per AUDIT recommendations).

**Pass criteria.**
- Zero outcome-class mixing among pins NOT carrying a debt entry.
- No pin re-anchored more than twice for the same flip signature — third occurrence
  forces the debt entry (§4.5).
- Debt entries cite the sweep evidence that exposed them.

**Severity.** HIGH.

---

## CA-5 — Founder-variance budget

**Statement.** Founder trait draws are U(0,1) and load-bearing at N=12 (audit H5,
Iter-263): any reshaping away from uniform starves extreme-driven producers and broke
13 liveness pins even when variance-preserving. Reshaping is a coordinated program, not
a local edit.

**Detection.**
1. *Diff-scope guard (static):*
   ```sh
   git diff --name-only <last-green>..HEAD -- crates/mindstrata-sim/src/sim/population.rs \
     crates/mindstrata-person/src
   ```
   Any touch to founder seeding/draw-shaping paths triggers the full dynamic battery.
2. *Uniformity probe (dynamic):* draw the founder trait battery across the sweep family;
   per trait assert support coverage (family-min < 0.25 AND family-max > 0.75) and
   variance within [0.06, 0.115] (U(0,1) var = 1/12 ≈ 0.083).
3. *Liveness battery (dynamic):* full standing liveness suite green — fear contagion,
   plasticity deltas, courtships, faction formation, motivation context, violence
   window, prediction-error seeds (the 13-pin Iter-263 casualty list).

**Pass criteria.** All three green. Any distribution-shaping change additionally
requires: a systemic-debt entry (`SYSTEMIC-DEBT(founder-variance):`), a larger founding
population decision, and a coordinated re-anchor sweep across ALL extreme-driven
producers — piecemeal attempts are rejected (§5).

**Severity.** CRITICAL.

---

## CA-6 — Fixed-4 truncation disease

**Statement.** `Fixed::mul` truncates at `SCALE=10_000`; any per-tick increment below
5e-5 quantizes to zero. Killed thermal convergence (239), faction pressure (240),
gestation advance (242), epidemic exposure (244). Law: sub-resolution rates compute in
f64 and quantize once (`Fixed::from_f64(x)`), or accumulate in f64 shadow fields.

**Detection.**
1. *Rate-literal scan (static):* see CA-2 detection #2 (same scan, hazard-specific
   disposition: every hit names its f64-domain guard in a comment).
2. *Accumulation liveness (dynamic, per rate-driven system):* probe asserts monotone
   nonzero progress over the pinned horizon — e.g., gestation advances, exposure
   accumulates, pressure converges — comparing Fixed accumulation against an f64 shadow
   integral within 2× resolution × ticks tolerance.

**Pass criteria.**
- Every sub-resolution rate has an f64-domain guard (comment names ceiling + upgrade
  path per §9 ponytail convention).
- Accumulation probes show strictly positive advance at the pinned horizon; zero
  advance with a passing assertion = dead producer (also routes to CA-2).

**Severity.** CRITICAL.

---

## CA-7 — mem::take write-back discipline

**Statement.** Every `mem::take` extraction in the tick pipeline must have a symmetric
write-back in the same pass (Iteration-218 trap: writes into taken buffers were silently
discarded; health-sync dead for 24 iterations).

**Detection.**
```sh
# Extraction sites (start-of-tick parallel arrays)
rg -n 'mem::take\(&mut (a|agents\[i\]|agent)\.(body|needs|goals|affect|emotions)\)' \
  crates/mindstrata-sim/src/sim/ --type rust
# Write-back sites (end-of-pass restore)
rg -n '(body|needs|goals|affect|emotions)\[i\] = std::mem::take' \
  crates/mindstrata-sim/src/sim/ --type rust
```
Counts must match per buffer family; any NEW take site outside `sim/core.rs`'s paired
extract/write-back block (currently ~L94–121 extract, ~L271–297 write-back) is flagged
for manual pairing review.

**Dynamic:** the standing cross-tick persistence tests (health-sync family in
`integration_tests/biology.rs`) must stay green — they exist precisely because this trap
is invisible to compile-time checks.

**Pass criteria.** Extract/write-back counts match per family; persistence tests green;
no orphan `mem::take` introduced.

**Severity.** CRITICAL.

---

## CA-8 — RNG append-only stream order

**Statement.** Six ChaCha8 streams (`World Behavior Social Economy Ecology Narrative`,
seeded master+{1,3,4,5,6,7}; streams are not serialized — replay re-seeds from master).
Existing draw sites are append-only per stream: inserting or reordering a draw shifts
every downstream consumer and re-paces all shared-stream pins (the historical
re-anchor treadmill, AUDIT 3.7). Birth-path constructors consume draws in field order;
`Personality::inherit` consumes exactly one draw per trait — count-alignment ≠
byte-alignment across range widths.

**Detection.**
1. *Draw-site diff scoping (static):*
   ```sh
   git diff -U0 <last-green>..HEAD -- crates/ | \
     rg '^[+-].*\.(gen_range|gen_bool|gen_f64|gen_ratio|sample)\('
   ```
   Any `-` line (removed/moved draw) or `+` line inserted ABOVE an existing draw in the
   same stream's consumer requires justification: either pure tail-append, or a declared
   coordinated re-anchor (cite sweep evidence).
2. *Birth-path parity (dynamic):* unit test asserting parent/child constructor draw-count
   parity per trait and field-order consumption (existing inheritance tests cover; keep
   them green).
3. *Golden referee:* any undetected reorder surfaces as a golden `agent_hash` break —
   route through `runbooks/golden-replay-custody.md` bisect, never regenerate baselines
   to absorb it.

**Pass criteria.** Diff shows tail-appends only, or each exception carries sweep-backed
re-anchor evidence; parity tests green; goldens byte-identical for structural commits.

**Severity.** HIGH.

---

## Seed-family sweep runner contract

The runner is the shared instrument behind CA-1/CA-2/CA-4/CA-5. One canonical spec:

- **Shape:** `crates/mindstrata-benches/examples/i<iter>_seed_family_sweep.rs`
  (IC-4 naming law). Release-mode only:
  `cargo run --release -p mindstrata-benches --example i<iter>_seed_family_sweep`.
  Prints stable `key=value` rows: `seed=<n> pin=<symbol> passed=<0|1> value=<f64>
  class=<label>` — machine-greppable per IC-4.
- **N seeds = 12, fixed family:** `{1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345}`.
  Rationale: superset of every anchor seed in the historical re-anchor corpus, so
  regressions resurface where they historically lived. The family is versioned HERE;
  changing it is a runbook change, not a per-run choice.
- **Which pins (scoped run = iteration audit):**
  1. Every pin whose assertion or comment the iteration's diff touches.
  2. Every pin carrying a `SYSTEMIC-DEBT(knife-edge)` entry.
  3. The standing liveness battery headline metrics.
  Full-suite runs (pre-push gate) extend to all magnitude/liveness pins enumerated by
  the bench-index tooling when it lands.
- **Variance statistics & thresholds (per pin):**

  | Statistic                 | Definition                                  | Threshold            |
  |---------------------------|---------------------------------------------|----------------------|
  | Family pass-rate          | passed=1 count / 12                         | ≥ 11/12 (≥ 92%)      |
  | Magnitude dispersion      | CV = std/mean of `value` over passing seeds | ≤ 0.35               |
  | Outcome-class unimodality | distinct `class` labels                     | exactly 1            |
  | Anchor churn (per 10 iters)| distinct anchor seeds cited in history     | ≤ 2                  |

  Rationale: the Iter-263 trapezoid shifted fear contagion 0.33→0.24 (~27% relative) and
  broke pins — CV 0.35 sits just above legitimate coupling noise but below
  starvation-class drift. Class mixing is NEVER absorbed by thresholds; it routes to
  CA-4 debt recording.
- **Failure handling:** any threshold breach blocks the re-anchor that motivated the run
  (CA-1) and opens the corresponding CA-2/CA-4 investigation. Fix the system; the seed
  family is not a menu.

## Audit record

Each iteration's audit appends one block to the commit message: rules checked, sweep
command + headline rows, violations found and their disposition (fixed / debt-tagged /
grandfathered). Observable output only (§3) — no "audited, looks fine" without the
runner rows.
