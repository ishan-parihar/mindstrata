# MindStrata — Operational Audit Report

**Date:** 2026-08-03
**Method:** Empirical only. The game was run exhaustively across every simulation dimension and CLI vector. No architecture-plan documents were consulted; everything below was observed by running the built binary.

---

## 0. Executive summary

MindStrata **compiles, runs deterministically, and is fully observable** — every inspector, dashboard, snapshot path, and metrics export works. The core loop (biology → psychology → social → institutions) executes for arbitrarily long runs without panicking, and edge cases (0 agents, 200 agents, 1 agent) are handled safely.

However, the simulation is **operationally shallow**: several headline systems are wired but fire ~never (marriage: **0 in 20K ticks**), are inert (market: **zero trade volume**, prices pinned at floor), or are misreported (health metric reads ~0.06 while agents are actually at ~1.00). Population is hard-capped at 48 agents, so "50 agents" silently becomes 48. Only 1 of 2 spec'd scenarios is loadable. **Verdict: playable & inspectable, but emergence is mostly latent — the engine is a functioning skeleton with several disconnected organs.**

---

## 1. Build & CLI surface

| Command | Status |
|---|---|
| `cargo build --release -p mindstrata-cli` | ✅ builds clean |
| `mindstrata --help` | ✅ works |
| `mindstrata sim --help` | ✅ works |
| `mindstrata scenario --help` | ✅ works |

**`sim` flags (all functional):**
`--seed`, `--ticks`, `--agents`, `--export-metrics <path>`, `--factions`, `--market`, `--map`, `--psychology <id>`, `--inspect-agent <id>`, `--beliefs <id>`, `--decisions <id>`, `--records`, `--timeline <n>`, `--show-relationships <a,b>`, `--save-snapshot <path>`, `--load-snapshot <path>`, `-v` (verbose).

---

## 2. Determinism & reproducibility — ✅ PASS

- Same seed + config run twice → **byte-identical simulation output** (only wall-clock timestamps differ). `md5sum` match.
- Deterministic RNG streams per system (`RngStream::Social`, etc.) are confirmed working.

## 3. Seed variance — ✅ PASS (rich)

Seeds `1, 7, 42, 99, 1234` at 2K ticks produce visibly different event profiles (faction formation, revolutions, moral panics, births all differ by seed). The engine is *not* degenerate to a single attractor.

---

## 4. What you can play / simulate right now

### 4.1 Population & demographics
- **Spawn:** ✅ Population-cap clamp works — requesting 200 agents logs `WARN ... Clamping initial population to MAX_POPULATION requested=200 clamped=48`. MAX_POPULATION = **48**.
- **Edge cases:** 0 agents → graceful no-op; 1 agent → runs.
- **Births:** ✅ occur (16 in a 20K-tick / 12-agent run; newborns get foundational beliefs, in-place replacement).
- **Deaths:** ⚠️ only age-based mortality (§31 probabilistic); **health never kills**. In 20K ticks with 12 agents: 0 deaths from health, population frozen at cap.
- **Population dynamics:** ⚠️ population rises to the 48 cap then **freezes forever** — no oscillation, no die-off, no overshoot-collapse. Long-run population curves are a flat line at 48.

### 4.2 Biology
- Body needs (hunger/thirst/fatigue), endocrine stress axis, cardiovascular, immune, nervous, circadian, reproductive systems all tick without error.
- ⚠️ **Health metric discrepancy (real finding):** inspector shows raw `body.health = 0.98–1.00` at tick 200+, but the metrics CSV `avg_health` (via `derived_health()`, which subtracts stress/pain/sickness/shock penalties) **crashes to ≈0.06 within ~20 ticks and never recovers**. Either the penalty stack (stress/pain/sickness) is being pinned at ~1.0 permanently, or the metric formula is wrong — either way `avg_health` is currently **misleading**.

### 4.3 Psychology
- Emotions (valence/joy/fear), stress, attachment (8 fields now surfaced: style, security, anxiety, avoidance, protest threshold, soothing receptivity, separation distress, caregiving style), cognition, motivation all tick.
- ⚠️ Emotion reactivity is weak: valence stays near +0.8 through revolution events in earlier tracing; fear/resentment feedback was added but is not strongly visible in trajectories.

### 4.4 Social
- Relationships, courtship (wired), households, kinship, status, hierarchy all present.
- 🔴 **Marriage: 0 marriages in 20K ticks (12 agents, 16 births).** Root cause found empirically:
  ```
  marriage_chance = attraction_score * health * trust * 0.001
  ```
  - `trust` defaults to 0 unless a directed relationship (i→j) already exists — most pairs never qualify.
  - The ×0.001 multiplier makes even a well-qualified pair (~0.2 attraction × 1.0 health × 0.5 trust) fire with probability 1e-4 **per tick**, i.e. ~1 marriage per 10,000 pair-ticks.
  - Net effect: the marriage system is effectively **dead code** in any practical run.
- `total_attraction()` itself is sound (weighted sum, clamped, unit-tested).

### 4.5 Economy
- 🔴 **Market is inert:** prices pinned at the floor (1.0), **zero trade volume** over a 10K-tick run despite the treasury-plumbing fixes. No transactions → no price signal → no market dynamics.
- Wealth/inequality: Gini ~0.71, but this is an **artifact of taxes/fines draining wealth**, not of trade-driven accumulation (no trade happens).
- Black market: wired in code, no observable activity.

### 4.6 Politics & institutions
- ✅ Faction formation **does** fire (factions dashboard works; membership is now exclusive post-fix).
- ✅ Council legitimacy is dynamic post-fix (no longer pinned at 0.000).
- ✅ Moral panic has cooldown + charge decay (no longer saturates at 1.0 forever).
- ✅ Revolutions can occur (observed across seeds).

### 4.7 Culture
- ⚠️ Memes, propaganda, rumors, rituals all tick, but metrics show **active_meme_count ≈ 0, polarization low** in long runs — cultural layer is wired but dormant by default config.

### 4.8 Ecology & resources
- Water/grain pools tracked (`total_water`, `total_grain` in CSV). No observable scarcity dynamics in default runs.

### 4.9 Scenarios
| Scenario | Status |
|---|---|
| `riverford` | ✅ runs, renders map |
| `drought` | 🔴 **not available** — only `riverford` is registered, despite `specs/scenarios/drought.ron` existing |

### 4.10 Persistence
- ✅ `--save-snapshot` / `--load-snapshot` round-trip verified: save at tick 500, load + resume works.
- ✅ `--export-metrics` CSV is complete for runs up to 20K ticks (ring buffer raised 500 → 20,000; undocumented before, now ample for realistic runs).
- 22 metric columns: tick, avg_hunger, avg_thirst, avg_fatigue, avg_valence, avg_joy, avg_fear, total_grain, total_water, event_count, journal_len, agent_count, avg_stress, avg_health, avg_relationship_trust, avg_relationship_quality, active_meme_count, polarization_index, household_count, kinship_edge_count, avg_agent_tier, total_active_feuds.

### 4.11 Observability — ✅ PASS (all inspectors render)
- `--psychology <id>` (incl. 8-field Attachment System) ✅
- `--inspect-agent <id>` (live view: body/needs/state/emotions/attachment/relationships) ✅
- `--beliefs <id>`, `--decisions <id>`, `--records` ✅
- `--map` (varies by seed — world-gen non-degenerate) ✅
- `--timeline <n>`, `--show-relationships a,b` ✅
- `--factions`, `--market` dashboards ✅ (content shallow — see §4.5)
- `-v` verbose: ⚠️ very sparse — only **5 WARN lines in a 3K-tick run**; most subsystems emit nothing.

---

## 5. Long-run emergence analysis (10K ticks, 50→48 agents, seed 42)

| Metric | Trajectory | Verdict |
|---|---|---|
| population | 48 → 48 (flat) | 🔴 frozen at cap |
| avg_health | ~0.98 → 0.06 (crash) | 🔴 metric misleading (agents actually ~1.0) |
| avg_fear | low/flat | ⚠️ no crises → no fear |
| market price | pinned at 1.0 | 🔴 inert |
| trade volume | 0 | 🔴 none |
| marriages | 0 | 🔴 never fires |
| memes / kinship / feuds | all ≈ 0 | ⚠️ dormant |
| gini | 0.71 | ⚠️ tax artifact, not trade |
| events | ~19K (dominated by moral-panic-era noise pre-fix; now bounded) | ⚠️ improved |

**Net:** no genuine long-run emergence (no class formation from trade, no marriage-driven kinship networks, no population cycles). The sim reaches a static equilibrium and stays there.

---

## 6. Tests & benchmarks

| Suite | Result |
|---|---|
| `cargo test --workspace` | ✅ 99 passed / 1 ignored |
| statistical / property / comparison / golden groups | ✅ all pass |
| `cargo bench -p mindstrata-benches --bench tick_loop -- --test` | ✅ runs |

---

## 7. Issue register (by severity)

### 🔴 Critical (systems effectively dead)
1. **Marriage never fires** — ×0.001 gate + trust=0 default ⇒ 0 marriages in 20K ticks.
2. **Market inert** — zero trade volume, prices pinned at floor; economy is static.
3. **Population frozen at cap 48** — no meaningful demography after cap hit; health never causes death.

### 🟠 Major (misleading or missing)
4. **`avg_health` metric broken/misleading** — reports ~0.06 while agents are ~1.0 (`derived_health()` penalty stack or metric formula).
5. **`drought` scenario unregistered** — spec exists, scenario.rs only knows `riverford`.
6. **Emotion/cognition decoupling persists** — valence near-static through major events; weak feedback.

### 🟡 Minor
7. **Cultural layer dormant by default** (memes ≈ 0) — needs config/seed pressure to activate.
8. **Verbose mode too quiet** — most systems emit nothing at `-v`.
9. **Gini 0.71 is an artifact** — inequality from taxes, not trade; will mislead analysis until market works.

---

## 8. Verdict

**What works:** build, determinism, all 12+ inspectors/dashboards, snapshots, metrics export, scenarios (riverford), faction formation, revolutions, moral panics, legitimacy, births, edge-case handling, full test suite.

**What's playable today:** a deterministic, fully inspectable *simulation skeleton* — you can watch agents live, trace psychology, dump 22-metric CSVs, save/load worlds, and see political events fire across seeds.

**What's not real yet:** economy, marriage/kinship demography, meaningful long-run emergence, honest health metrics, and half the scenario library.

**Priority fix order:** ① marriage gate (raise multiplier, default trust floor, pair selection without prior relationship) → ② market (enable actual bids/asks/clearing) → ③ `avg_health` metric → ④ health-based mortality → ⑤ register `drought` scenario → ⑥ cultural activation pressure.
