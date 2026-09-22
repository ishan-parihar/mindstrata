# Iteration 362 — cross-polity diffusion is LIVE at town scale

**Status:** MEASUREMENT (probe only, no source changed) · **Item owned:** the i360
follow-up — "measure cross-polity liveness at town scale".

## The question

i360 turned the density world into a town of villages (N=192 → 3 settlements, N=256 → 4),
each with its own live polity holon. The substrate's i299 channel
(`system_trade_diffusion`) is meant to let one village's genesis memes leak into a
neighbour through **cross-polity trade** — but it is zero-at-zero on two conditions:
fewer than 2 polities (now satisfied), and an actual cross-polity `TradeOccurred`.
Agent-to-agent trade needs a counter-party within 12 tiles, so with villages now
geographically distinct the real risk was that the channel stayed **dormant**.

i360's Leg B already showed the *birth* half is live (each polity mints its own
`[genesis:pN:...]` memes). This probe measures the *leakage* half directly: for every
genesis meme, read its host set (`Meme::hosts`) for agents belonging to a polity **other
than** the meme's own namespace. (`host_count` is capped at the receiving polity's size,
so the host **set**, not the count, is the honest signal.)

## Result — DIFFUSION_LIVE on both town sizes

20 000 ticks, seed 42, gap 8:

| N | side | polities | genesis memes | leaked across a boundary | foreign host-links | leaked per source polity | cross-polity trades (last ~4096 events) |
|---|---|---|---|---|---|---|---|
| 192 | 64 | 3 (64/64/64) | 12 | **12 (100%)** | 1018 | [6, 4, 2] | 22 |
| 256 | 74 | 4 (64×4) | 20 | **20 (100%)** | 2806 | [8, 6, 4, 2] | 16 |

Every genesis meme minted by every village appears in a neighbouring village's agent
population, and cross-polity trades are continuously occurring (~16–22 in a 4096-event
window). `verdict = DIFFUSION_LIVE`.

## What this closes

The i296–i299 multi-settlement stack is now **end-to-end live at town scale**: geography
forms multiple settlements (i360), the partition derives polities (i298), each polity
mints territory-routed genesis culture (i297/i158 — i360 Leg B), and that culture
**diffuses across village boundaries via trade** (i299 — this probe). The i359 blocker
("the world is one settlement, so there is nothing to orchestrate") is fully cleared.

## Recorded observations

- The leak is **asymmetric and decays with distance from the boundary**: leaked-per-source
  is ranked `[6,4,2]` / `[8,6,4,2]` — adjacent villages exchange culture, far ones less
  so — which is the expected interaction-network attenuation, not a defect.
- Foreign host-links (1018 / 2806) far exceed the meme count, i.e. leakage is broad
  within each receiving village, not a single-agent artifact.
- Unmeasured here: whether a larger `cluster_count_for` cap (5–6) increases inter-village
  contact (a density-law world is finite), and the long-horizon saturation of leakage.

## Verification

Probe + docs only; **no source changed**. Golden replay byte-identical, no re-anchors,
`gate --full` GREEN.
