# i342 — The "social connections" proxy is dead, and the fix's blast is measured

**How it surfaced.** Building i341's §7 coupling map (classify every per-edge read site
before any sparsification surgery), two appraisal channels turned out to read the agent's
relationship-list LENGTH as its number of social connections:

- `social_visibility` (the attachment/isolation term):
  `0.5 | 0.0 + 0.1·min(len, 4)`
- normal-life anxiety's `social_factor`: `(4 − min(len, 4))·0.002` — "few connections ⇒ worry"

i335/i336 pinned that list at exactly **N−1 rows for every agent** (a complete directed
graph). So the length measures the **population**, not the agent's sociality.

**Probe.** `crates/mindstrata-benches/examples/i342_social_count_proxy.rs` (seed 42, 32×32,
5 000 ticks), reading the production census `Simulation::contacted_degrees`:

| N | list length | contacted mean | contacted max | `min(len,4)==4` | `social_visibility` | anxiety `social_factor` |
|---|-------------|----------------|---------------|-----------------|---------------------|-------------------------|
| 12 | 11 | 6.2 | 10 | **yes, all agents** | **0.900 / 0.400** | **0.000** |
| 48 | 47 | 6.3 | 14 | **yes, all agents** | **0.900 / 0.400** | **0.000** |
| 96 | 95 | 9.8 | 28 | **yes, all agents** | **0.900 / 0.400** | **0.000** |

The §4.3 dead-producer shape exactly — live code, zero discrimination:

- `min(len, 4)` is **4 for every agent at every N** (lists of 11/47/95 all clamp), so the only
  variation in `social_visibility` is partner status: an isolate and the village's
  best-connected agent score **identically**. The list length cannot fall below 4 for any
  N ≥ 5, so the channel has no reachable isolation state *by construction*.
- The anxiety term evaluates to **exactly 0.000 for the entire village**: the
  "few connections ⇒ worry" channel has no isolation input at all.
- Meanwhile the honest count — contacted degree — varies 0–28 across the same runs.

**Contrast with a saturated producer (§4.3).** This is the same failure mode as fear pinned
at 0.99: tests pass, the value is finite, and the channel carries no information. Probing the
equilibrium value (not the assertions) is what exposed it.

## The fix, measured and held back

The repair is one line per site: read `Simulation::contacted_degrees()` (rows with
`interaction_count > 0`) instead of `relationship_v2s.len()`. The clamp and the 0.1 weight are
untouched, so **connected agents keep the calibrated magnitude** (`min(degree, 4) == 4` for
degree ≥ 4, the common case at every N) and only genuine isolates move — the term finally
doing what it describes.

Wired and measured, the blast is **13 tests**: both golden replays, 7 snapshots
(endocrine/agent-state/institution/relationship-stages/long-horizon at 500–10 000 ticks), and
4 behavioural integration tests (governance ×2, moral-panic lifecycle, the conception
pipeline). That is a **re-contract (§4.4)**, not a re-pin: the channel's definition changes,
so the charter baselines legitimately move — and moving a golden is a documented-evidence step
(§3), not something to wave through at the end of a session.

**Held back, deliberately.** This iteration ships the *survey*: the census accessor (production
code, pinned by `contacted_degrees_counts_only_rows_with_interaction_state`), the probe, and
the two sites annotated with the finding and the reason they are untouched. The wiring lands as
its own iteration with the sweep it needs — re-anchor the goldens and snapshots with the
mechanism documented, examine the 4 behavioural tests for re-pin vs re-contract, and re-audit
downstream equilibria (§2.5). `i342_social_count_proxy` and the 13-test blast radius are that
iteration's starting evidence.

## Why this matters beyond the two sites

- It is an independent confirmation of i341's finding that **per-edge "degree/mean over every
  other agent" folds are semantics, not accidents**: here the semantic was silently wrong, and
  three channels (visualisation, anxiety, and the §17 tier gate's `relationship_count`) all
  consume a quantity that cannot vary.
- It gives the queued sparse store (i343) its first concrete contract: whatever replaces the
  complete graph must still answer "how many social connections does this agent have?", and
  `contacted_degrees` is the measured answer.

## Reproduce

```
cargo run --release -p mindstrata-benches --example i342_social_count_proxy
```
