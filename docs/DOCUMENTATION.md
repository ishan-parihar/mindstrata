---
name: documentation-index
description: "The documentation authority map and index. Defines which document owns which scope, the status vocabulary, and the reconciliation rule that keeps docs aligned with the code. Read this before trusting any other doc."
type: Authority
status: AUTHORITY
scope: "documentation governance"
reconciled_commit: 8a4ac31
created: 2026-09-22
owner: PROD (AP4 Studio)
---

# Documentation Authority Map

> **Read order for any session:** this file → the owning doc for your scope →
> [`ENGINE_STATUS.md`](ENGINE_STATUS.md) for the current engine truth.
>
> **Why this exists.** Twice the doctrine pointer went stale and cost iterations:
> AGENTS.md §8 carried the Iteration-247 queue while DC-3 closed and DC-4 opened, and
> later it named the (measured, demoted) sparse store as "the only remaining lever."
> Stale docs are not cosmetic here — a stale ledger sends an agent to rebuild something
> a probe already refuted. This map plus `scripts/doc_index.py` makes drift *detectable*
> instead of discovered-by-accident.

---

## 1. Status vocabulary

Every doc carries exactly one status. The words mean specific things:

| Status | Meaning | May be trusted for decisions? | Needs `reconciled_commit`? |
|---|---|---|---|
| **AUTHORITY** | The current truth for its scope. If it disagrees with anything, it wins. | **Yes** | **Yes** |
| **ACTIVE** | A live program, ledger, or binding contract in force. | Yes, within its scope | Yes (ledgers/plans) |
| **REFERENCE** | Stable spec/canon/theory/runbook. Reviewed at milestones, not per-iteration. | Yes, as spec | No |
| **HISTORICAL** | A frozen record: evidence logs, completed plans, audits, archived waves. | **No** — context only | No |
| **SUPERSEDED** | Replaced by another doc; kept for history. Must carry a banner pointing to the replacement. | **No** | No |

**Rules of use.**
- An agent planning work reads **AUTHORITY → ACTIVE → REFERENCE** only.
- A **HISTORICAL** doc is never used to decide current behaviour; it records what was measured *then*.
- A **SUPERSEDED** doc must start with a banner naming its replacement, or `doc_index.py` will not accept it (see §5).

## 2. Authority map — who owns what

| Scope | Owning document | Notes |
|---|---|---|
| How we work (process, gates, honesty rules) | `AGENTS.md` | Binding, always. The doctrine. |
| Current engine truth (architecture, behaviour, realism, scale) | `docs/ENGINE_STATUS.md` | **Single source.** Supersedes the AP2-era current-state doc. |
| Project trajectory (milestones, layer map) | `docs/ROADMAP.md` | Updated at cycle boundaries. |
| Live work ledger (what is left, per-iteration trail) | `docs/PLAN_DC3_DEVELOPMENT.md` | §3 is the calibration-debt ledger; §7/§9 are execution ledgers. |
| Iteration evidence (probes, measured verdicts) | `docs/architecture/AP4-studio/evidence/` | One doc per iteration. Historical by construction. |
| Organizational model & departments | `docs/architecture/AP4-studio/` | Operating model, charters, contracts, runbooks. |
| Determinism law | `docs/architecture/AP4-studio/contracts/IC-3-determinism.md` | Binding contract. |
| Calibration canon (bands, curves, budgets) | `docs/balance/` | needs-bands, pathology-curves, difficulty-levers, perf budgets. |
| Theory & knowledge base | `docs/architecture/AP3-afa/`, `docs/knowledge-base/` | Reference theory; not implementation status. |
| Historical architectures | `docs/architecture/AP2*.md`, `docs/architecture/archive/` | Completed plans (AP1, AP2, AP3 eras). |

**The historical-doc rule:** a historical doc may be *wrong about the present* — that is
its nature. What it must never do is present itself as current. That is what the status
column prevents.

## 3. Directory conventions (auto-classified)

Everything under these directories is **HISTORICAL/REFERENCE by construction** and is not
listed individually below (178 files):

| Directory | Auto-status | Reason |
|---|---|---|
| `docs/**/evidence/**` | HISTORICAL | iteration evidence logs (166 docs) |
| `docs/architecture/archive/**` | HISTORICAL | completed architecture plans |
| `docs/architecture/AP3-afa/waves/**` | HISTORICAL | completed AP3 wave briefs |
| `docs/knowledge-base/**` | REFERENCE | theory notes, not status |
| `docs/balance/change-orders/**` | HISTORICAL | ratified change orders |

Everything else must be listed in §6 with an explicit status.

## 4. The reconciliation rule

1. **At every milestone gate** (`scripts/gate --full` before a cycle/UM gate), re-verify the
   numbers in `ENGINE_STATUS.md` §1/§5/§7/§8 and bump its `reconciled_commit`.
2. **Any iteration that changes a fact an AUTHORITY or ACTIVE doc asserts** must update that
   doc **in the same commit**. A behaviour change that invalidates a ledger row is not done
   until the row moves.
3. **A new doc must be classified** in §6 in the commit that adds it — the gate enforces this.
4. **Citations are part of the contract** (i386, AGENTS.md §4.14). A doc that names a file,
   a symbol anchor, or an artifact must be re-pointed **in the commit that moves it** —
   `FROZEN` freezes the rule, never the path. The `doc_index.py` gate checks *structure*
   (classified, no ghosts, marker resolves) and **cannot** see citation drift: it will pass
   a doc that cites a file which never existed, or that says a value is "not landed" after
   the code shipped it under a different name. Treat every such claim as a probe to run
   before you plan work off it. The i386 sweep (2026-09-23) is the worked example: 2 ghost
   contract filenames, 2 moved pass files, 1 crate-moved symbol, and 2 balance docs
   reporting live levers as unlanded.
5. **When a doc is replaced**, set its status to SUPERSEDED and add a banner:
   ```
   > **SUPERSEDED (YYYY-MM-DD, iNNN):** replaced by `path/to/replacement.md`. Kept for history.
   ```

## 5. Enforcement

`scripts/doc_index.py` (wired into `scripts/gate`, step 2.6) fails the build when:

- a non-exempt doc under `docs/` or a root `*.md` is **not** listed in §6 (unclassified drift);
- a listed path **does not exist** (ghost entry);
- an AUTHORITY/ACTIVE doc under `docs/` lacks a `reconciled_commit:` that resolves in git.

Run it directly with `python3 scripts/doc_index.py --check`; add `--list` to print what is
missing. There is no `--write`: classification is a judgement, not a generation.

## 6. Index — every doc that is not auto-classified

| Path | Status | Owns / note |
|---|---|---|
| `AGENTS.md` | AUTHORITY | development doctrine; binding process, gates, honesty rules |
| `README.md` | AUTHORITY | project entry point and public framing |
| `docs/ROADMAP.md` | AUTHORITY | milestone ladder and layer map |
| `CHANGELOG.md` | HISTORICAL | frozen at Iteration 214; superseded in practice by the git log and the ledgers |
| `OPERATIONAL_AUDIT.md` | HISTORICAL | empirical audit snapshot, 2026-08-03; pre-dates most systems |
| `docs/ENGINE_STATUS.md` | AUTHORITY | **current engine truth** — architecture, behaviour, realism, scale |
| `docs/DOCUMENTATION.md` | AUTHORITY | this file; documentation governance |
| `docs/MINDSTRATA_CURRENT_STATE.md` | SUPERSEDED | → `ENGINE_STATUS.md` (AP2-era state) |
| `docs/REMAINING_WORK_REPORT.md` | SUPERSEDED | → `ENGINE_STATUS.md` + `PLAN_DC3_DEVELOPMENT.md` (Iter-134-era) |
| `docs/PLAN_DC3_DEVELOPMENT.md` | ACTIVE | DC-3 ledger (closed) + the DC-4 execution ledger; §3 calibration debt |
| `docs/PLAN_DC5_DEVELOPMENT.md` | ACTIVE | **the live plan** — DC-5 gap taxonomy G1–G8 + the iteration ladder (i387+) |
| `docs/PLAN_DC2_DEVELOPMENT.md` | HISTORICAL | DC-2 complete (WP-H3 closed i286) |
| `docs/PLAN_BIO_PSYCH_DEEPENING.md` | HISTORICAL | Arcs A–D complete (program close i259) |
| `docs/PLAN_SCALING_FOUNDATION.md` | HISTORICAL | crate ladder S1–S3 landed; remaining infra tracked in PLAN_DC3 |
| `docs/AUDIT_2026-08-22_EMERGENT_REALISM.md` | HISTORICAL | realism audit findings H1–H6 / E1–E8; most since closed |
| `docs/scaling/coupling_map.md` | REFERENCE | S1 crate-boundary coupling survey |
| `docs/architecture/AP2.md` | HISTORICAL | AP2 spec (implemented) |
| `docs/architecture/AP2_AUDIT_PLAN.md` | HISTORICAL | AP2 audit plan |
| `docs/architecture/AP2_AUDIT_FINDINGS.md` | HISTORICAL | AP2 audit findings |
| `docs/architecture/AP2_FINAL_AUDIT.md` | HISTORICAL | AP2 closeout |
| `docs/architecture/AP3-afa/PLAN.md` | SUPERSEDED | → `ENGINE_STATUS.md` for status; AP3 Era I–V all landed |
| `docs/architecture/AP3-afa/01-doctrine.md` | REFERENCE | AP3 doctrine (field engine invariants) |
| `docs/architecture/AP3-afa/02-theory-map.md` | REFERENCE | theory mapping |
| `docs/architecture/AP3-afa/03-substrate.md` | REFERENCE | substrate definitions |
| `docs/architecture/AP3-afa/04-waves.md` | HISTORICAL | wave schedule (executed) |
| `docs/architecture/AP3-afa/refs/kosmos-index.md` | REFERENCE | upstream ontology index |
| `docs/architecture/AP3-afa/refs/sim-inventory.md` | REFERENCE | sim inventory snapshot (may lag; ENGINE_STATUS wins) |
| `docs/architecture/AP4-studio/PLAN.md` | ACTIVE | studio operating-model entry point |
| `docs/architecture/AP4-studio/01-operating-model.md` | REFERENCE | studio loop and roles |
| `docs/architecture/AP4-studio/02-departments.md` | REFERENCE | department definitions |
| `docs/architecture/AP4-studio/03-interlock-map.md` | REFERENCE | territory ledger |
| `docs/architecture/AP4-studio/04-cycle-plan-DC1.md` | HISTORICAL | DC-1 cycle plan (106/106 closed) |
| `docs/architecture/AP4-studio/05-deployment-manifest.md` | REFERENCE | agent deployment packets |
| `docs/architecture/AP4-studio/06-swarm-integration.md` | REFERENCE | swarm loading procedure |
| `docs/architecture/AP4-studio/08-spec-source.md` | REFERENCE | FR catalog → spec procedure |
| `docs/architecture/AP4-studio/charters/ASSET-PIPELINE-v0.md` | REFERENCE | asset schema v0 (binding, i301) |
| `docs/architecture/AP4-studio/charters/CLIENT.md` | REFERENCE | CLIENT department charter |
| `docs/architecture/AP4-studio/charters/DC3-P0-perf-budget.md` | REFERENCE | perf budgets (binding) |
| `docs/architecture/AP4-studio/charters/DESIGN.md` | REFERENCE | DESIGN department charter |
| `docs/architecture/AP4-studio/charters/PLATFORM.md` | REFERENCE | PLATFORM department charter |
| `docs/architecture/AP4-studio/charters/QA.md` | REFERENCE | QA department charter |
| `docs/architecture/AP4-studio/charters/SIM.md` | REFERENCE | SIM department charter |
| `docs/architecture/AP4-studio/charters/STORY.md` | REFERENCE | STORY department charter |
| `docs/architecture/AP4-studio/charters/TOOLS.md` | REFERENCE | TOOLS department charter |
| `docs/architecture/AP4-studio/contracts/IC-1-catalysts.md` | REFERENCE | catalyst observation contract |
| `docs/architecture/AP4-studio/contracts/IC-2-annals.md` | REFERENCE | annals contract |
| `docs/architecture/AP4-studio/contracts/IC-3-determinism.md` | REFERENCE | determinism law (binding) |
| `docs/architecture/AP4-studio/contracts/IC-4-probes.md` | REFERENCE | probe contract (binding) |
| `docs/architecture/AP4-studio/contracts/IC-5-canon.md` | REFERENCE | canon-values contract |
| `docs/architecture/AP4-studio/contracts/IC-6-gates.md` | REFERENCE | gate definitions |
| `docs/architecture/AP4-studio/contracts/IC-7-modding.md` | REFERENCE | modding surface contract |
| `docs/architecture/AP4-studio/contracts/IC-8-render-budget.md` | REFERENCE | render budget |
| `docs/architecture/AP4-studio/runbooks/beat-dashboards.md` | REFERENCE | dashboards runbook |
| `docs/architecture/AP4-studio/runbooks/calibration-audit-v2.md` | REFERENCE | calibration-audit method |
| `docs/architecture/AP4-studio/runbooks/ci-aa-mode.md` | REFERENCE | CI AA-mode runbook |
| `docs/architecture/AP4-studio/runbooks/devex-check-loops.md` | REFERENCE | dev loop runbook |
| `docs/architecture/AP4-studio/runbooks/dossier-polish-19-22.md` | HISTORICAL | completed polish pass |
| `docs/architecture/AP4-studio/runbooks/golden-custody-transfer-checklist.md` | REFERENCE | golden custody checklist |
| `docs/architecture/AP4-studio/runbooks/golden-replay-custody.md` | REFERENCE | golden replay custody |
| `docs/architecture/AP4-studio/runbooks/modding-surface.md` | REFERENCE | modding runbook |
| `docs/architecture/AP4-studio/runbooks/suite-segmentation.md` | REFERENCE | suite segmentation |
| `docs/architecture/AP4-studio/templates/evidence-schema-v1.md` | REFERENCE | evidence doc schema |
| `docs/architecture/AP4-studio/templates/INTERLOCK.md` | REFERENCE | interlock template |
| `docs/architecture/AP4-studio/templates/PHASE.md` | REFERENCE | phase template |
| `docs/balance/canon-inventory.md` | REFERENCE | canon value inventory |
| `docs/balance/difficulty-levers.md` | REFERENCE | difficulty-lever catalog |
| `docs/balance/needs-bands.md` | REFERENCE | needs canon bands (CO-2026-003) |
| `docs/balance/pacing-model.md` | REFERENCE | pacing model |
| `docs/balance/pathology-curves.md` | REFERENCE | pathology quadrant curves |
| `docs/balance/perf-budget.md` | REFERENCE | perf budget canon |
| `docs/balance/render-perf.md` | REFERENCE | render perf canon |
