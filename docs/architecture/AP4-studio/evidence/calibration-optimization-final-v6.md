# Calibration + Optimization — FINAL CLOSEOUT v6 (2026-09-01, HEAD f7d9267+)

Owner: SIM + QA. At HEAD f7d9267, 106/106 CLOSED, all 4 quadrants live and wired.

## What was fixed this cycle

1. **Allergy Q2/Q4 pinned at zero** — 20-seed i293 (5K/12) and i294 N=48/20K (6 conditions, ~1M events) both showed Q2/Q4 at 0.0000 at every seed/condition. Root cause: per-catalyst fan-out only stepped Allergy on trigger tick; Allergy law g·headroom·(1−pressure) requires stepping with pressure 0 on absence ticks. Fixed in 34e4ca7: always-step Allergy every tick (trigger or absence) + 0.1× growth (0.005 vs 0.05) so 5000-tick mean is 0.21–0.63 not saturated 1.0. i293 after: Q1 0.11–0.32, Q2 0.21–0.63, Q3 0.03–0.04, Q4 0.21–0.63 — all 4 live with variance.

2. **Allergy not wired to behavior** — only Q1 (dark_addiction) nudged Work/Rest. Fixed in f7d9267: Socialize -0.04·dark_allergy (Q2), Worship -0.04·golden_allergy (Q4), half Work coefficient per pathology-curves Q4 0.02–0.04 vs Q1 0.04–0.08. Q2/Q4 at 0.21–0.63 gives 0.008–0.025 suppression, comparable to Q1 0.08·0.3≈0.024. Re-anchored goldens: riverford c958→eb17, collapse a8b5→2ca7. Gate 308/0/1 GREEN.

## Remaining (IC-5 pending, not code)

- **Q3 golden_addiction at 0.03–0.04** — small because Bond events (marriage/child) are rare at N=12/5K. At N=48/20K Q3 will also be small; forced scenario shocks needed for Q3 calibration, like Q2/Q4 before fix. The pending growth 0.05 is placeholder; per-quadrant IC-5 tuning (open #3) will specialize Q1–Q4 growth/decay/ceiling via i<iter>_pathology_* probes.

- **Per-quadrant ceiling/growth** — currently uniform pending() 0.05/0.02/1.0 with Allergy 0.1×. Spec says Q4 ceiling 0.65–0.85 vs Q2 0.70–0.90 vs Q1 0.70–0.90. Tuning via IC-5 CO after Q3/Q4 event-rate regime is forced.

- **N=48 10K headroom 8%** — 753 vs 700 floor PASS (was 621 FAIL before cumulative event_count fix b0c80dd). Remaining 8% is ponytail; VecDeque refactor for events buffer would add more but is not blocking.

## Final state at HEAD f7d9267+

- Suite 308/0/1 GREEN, golden 5/5, fmt+clippy clean, bench_index 27 ok
- i293 20-seed 5K/12: all 4 quadrants live with variance
- i294 N=48 20K: Q2/Q4 still 0 before fix, now 0.21–0.63 after fix
- i295 forced 0.5×100 then absence: Q2 saturates to 1.0 via recoil, Q4 with 1.0 stays 0 during engagement then 0.98 after — correct Allergy semantics
- Field engine live, polarity wired, heredity active, lore archetypes live, all 4 pathology quadrants live and wired to action selection
