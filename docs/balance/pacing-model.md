# Pacing Model — What the Player Experiences (v1 measured)

Owner: DESIGN → CLIENT playability bar (IC-6 G4) + PLATFORM perf guidance (IC-8).
Companion to canon-inventory row 6; feeds `IC-6 G4` UM-1 gate and `IC-5` difficulty levers.
Status: **RATIFIED v1 at 2026-08-29 — measured basis `i270`/`i271`/`i272` release.**

## Measured basis (release, same host as budgets)

* **Tick rate:** `i270_perf_snapshot` `N=12 10.7K tps (93µs/tick)` → wall-clock `10K≈0.93s`, `50K≈4.7s`, `100K≈9.3s` (budget `≤15s`, `i270 --quick 10056≥8000` PASS). `N=48 865 tps (1.16 ms)` → `100K≈116s` (`≤180s`). `snapshot JSON 3.3→10.1 MB` (ceiling `6/18`).
* **Render:** `i271_render_perf` `Trends 9.7/22.6/170µs`, `dashboard 0.9µs`, `list 10.4µs` → per-frame `tick+render ≈118µs@12/1.23ms@48` (budget `Trends ≤1 ms`, `IC-8 @1dea277`). TUI `delay_ms=120` dominates frame.
* **Differentiation:** `i272_differentiation` `inter 0.1817 / intra 0.0723 PASS` (altitude delta `1.20` cognitive) — seeded vs neutral clusters measurable already at `10K`.
* **Goldens:** `collapse`/`riverford_minor` deterministic replay `5/5` — same `100K` horizon seeds the long probe `p5_100k_probe.rs`.

## Horizons (player-visible milestones — titrated to wall-clock)

| Horizon | Wall-clock @N=12 | Player sees by this tick | Probe / artifact that will prove it | DESIGN note |
|---|---|---|---|---|
| 10K | `~0.9s` | households formed, 1–2 factions visible, first norm verdicts, lineage lane draws first stage positions | `i272` clusters measurable; `p5` family probes; `golden` early ticks; `TUI --annals` lane smoke | Short session — tutorial-length; difficulty levers (row 1 of levers doc) tune founding variance & need decay so the player can distinguish seeded cultures within this window |
| 50K | `~4.7s` | faction crisis-pressure cycles, marriage webs, courtship→birth pipeline, epidemic windows, first ideology drift signals | `i<iter>_midcourse` 50K sweep (future); `World.tick()` 50K snapshot; `annals.jsonl` 500 intervals | Mid session — the village "lives"; pathology quadrants begin to diverge from `pending()` if pressure sustained (WP-C) |
| 100K | `~9.3s` | multi-generation lineage, culture-disjointness across seeds, full AP3 Era II field altitudes, annals chronicle depth | `p5_100k_probe` + `culture-disjointness` (`i<iter>_culture_divergence` planned) + `FINAL-AUDIT` suite `100K@N=48 116s` ceiling | Full session — `UM-1` target; wall-clock fits a `10–15s` automated playthrough, `2–3 min` at `N=48` village-scale (DC-3) |

Session-length target for `UM-1` playability `G4`: **a new player can witness distinct
village fates (2+ clusters) within a `15s` unattended `100K` run** — the `i272 0.18` gate
is the mechanical proxy until human playtesting replaces it.

## Difficulty levers (pointer to `difficulty-levers.md`)

Three levers pace the above horizons without touching structure (all via `IC-5` `CO-`):
1. **Founding variance** (founder `LineId` distribution spread via `needs` line) — moves `10K` cluster distance `inter` (`AGENTS §5` debt, `i272` floor `0.12`).
2. **Need decay / fulfillment thresholds** (`needs-bands.md` 6 bands `0.25–0.92`) — moves `50K` goal-generation pacing (Wander vs Socialize vs Worship ratios).
3. **Pathology growth/ceiling** (`pathology-curves.md` 4 quadrants dark/golden×addiction/allergy) — moves `100K` lineage divergence half-life (WP-C `growth 0.02–0.10/decay 0.01–0.05`).

## Residual open questions (carried as DC-2 debt, not blockers)

1. At which horizon does AP3 stage visibility first change a player's decision vs
   merely the lane reading? → needs `CLIENT 19-22` annals browsing depth + playtest.
2. Should `N=48` `116s` be presented as a single session or chapterized saves (`IC-3 save schema` `v13`)? → `PLATFORM 8-10` save/CI question.
3. Exact `50K` crisis-pressure cadence — awaiting `SIM 12-13` pathology signature behavioral wiring beyond `pending()` identity.
