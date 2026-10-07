---
title: Pytxo decision ledger
slug: pytxo-decisions
status: active
tags: [context, audit, pytxo]
audience: [human, agent]
layer: meta
created: 2026-09-16
updated: 2026-09-16
related: ["[[PYTXO_HISTORY]]", "[[PYTXO_ARCHITECTURE]]", "[[PYTXO_OPEN_QUESTIONS]]"]
---

# Pytxo decision ledger

Snapshot 2026-09-16. H1–H4 and R1–R13 resolve in [[PYTXO_SOURCE_LEDGER]]. Status is scoped: IMPLEMENTED is source state, VALIDATED needs named evidence, and DECIDED may be an attributed historical product choice. A proposed ADR is not accepted merely because its body says “Decision.”

**Post-audit update:** The user approved the Windows/Codex/free Beta completion
plan. A1/D14 is now IMPLEMENTED with automated Core/IPC/browser validation:
reviewed digest required through Apply, no latest-package fallback, comparison
under the existing domain lease. Native/package acceptance remains BLOCKED.
The C10 contrast below describes the pre-fix audit, not the new call signature.
Flow snapshots now include approved worker authority; changing checks requires
fresh preview. See RELEASE_PLAN/RELEASE_READINESS for remaining work and gates.

## Durable decisions and proposals

| ID | Decision / status | Approximate chronology and reasoning | Sources | Current code reflects it? | Supersedes / revisitable? |
|---|---|---|---|---|---|
| D01 | Independent outcome enforcement rather than generic approval UI — DECIDED | Undated H1 internal pivot: native permissions/PR/CI substitute for generic review; seek a separate landing boundary | H1; R1/R4 | Prepared package/Apply exists; approval binding qualification A1 | Extra-popup thesis SUPERSEDED; revisit differentiation with user evidence |
| D02 | Useful delegation/control plane is whole-product outcome — DECIDED | H2 broadens commit-only framing; Sep 7 repo vision also makes boundary a primitive | H2, H3; R1 | Planning, scheduling, review present; lower burden not validated | Commit-only product identity SUPERSEDED, not commit integrity; public headline still open |
| D03 | Deterministic mechanisms own correctness-sensitive transitions — DECIDED / IMPLEMENTED in reviewed path | Repeated H2/H3 separation of probabilistic suggestion from authority | H2/H3; R2/R4/R7 | Yes for existing path, not proof against malicious same-user tampering | Model authority REJECTED; foundational invariant |
| D04 | Model ≠ harness ≠ execution backend ≠ capacity pool — DECIDED | H3 corrects cheap-harness quota optimism | H2/H3; R3/R7 | Models/harnesses/backends distinguished; CapacityPool NOT FOUND | “Many CLIs = independent unlimited quotas” SUPERSEDED; pool design open |
| D05 | Single-harness/one-worker value first-class — DECIDED | H2/H3: parallelism is optional optimization | R1/R2/H2/H3 | Single-worker Flow exists; current conservation test passes | Mandatory multi-agent identity REJECTED; expand only with evidence |
| D06 | Preserve tasks; cap concurrency — IMPLEMENTED / VALIDATED in focused test | Exact worker limit must not truncate approved work | H3; R2/R12 | Planner retains tasks; waves bound workers | Any task-truncation interpretation rejected; invariant |
| D07 | No broad automatic fan-out for this increment — DECIDED / DEFERRED beyond it | Sep 7 repo decision, repeated H3 | R1/H3 | Explicit plans/waves; no automatic heterogeneous router established | Swarm-first Beta prerequisite rejected; requires fresh decision/evidence |
| D08 | Cheap-capacity scaling 1→4 before 16/50/100 — EXPERIMENTAL | Sep 15–16: integration, verification and human attention may dominate inference cost | H3 | No named SwarmBench implementation found | 100-worker priority DEFERRED; scale only on verified useful-work gains |
| D09 | Optional provider-neutral coordinator — IMPLEMENTED foundation; ADR PROPOSED | Sep 15 dirty changes replace ambient-key selection/hardcoded managed route | H2; R7/R13 | Explicit direct/managed provider/model; opt-in; OpenAI-compatible adapter | DeepSeek lock-in REJECTED; default profile revisitable through evaluation |
| D10 | Complete eligible execution profiles, rules first — DECIDED direction / PROPOSED operational router | H2: incompatible/local-only/denied routes must be excluded, not down-ranked | H2/H3; R7/R13 | Typed proposal and membership validator only | Opaque global “best AI” router DEFERRED; schema/policy execution open |
| D11 | Local inference is first-class; Core accountless — DECIDED direction / IMPLEMENTED seams | H2 privacy and local-user value | H2; R7/R9 | Local planning, endpoint/provider seams; no full offline/local-model run here | Mandatory Cloud rejected; local inference ≠ air gap |
| D12 | Plugins may extend capabilities, never Core truth — DECIDED direction / PROPOSED implementation | H2 wants foundations in Beta, marketplace later | H2; R7 | Plugin name lists only in new routing contracts; no manager/SDK/runtime established | Marketplace DEFERRED; Beta inclusion still requires scope resolution |
| D13 | Exact combined candidate and fresh required checks — IMPLEMENTED / VALIDATED in focused tests | Sep 6–8 commits harden reviewed candidate semantics | H2/H3; R4/R12 | v3 inventory/recipe, mutations/drift refusal | Worker-local checks insufficient; retain |
| D14 | Review authorization must bind exact state — DECIDED; full validation UNKNOWN | Conceptual H1 → hardened package code | H1/H2/H3; R4/R5 | Stored package matching; missing user-reviewed digest in Apply API | No permission to weaken invariant; A1 next mission |
| D15 | Honest per-surface enforcement and conservative recovery — IMPLEMENTED; runtime scope limited | Repo hardening and H3 reject blanket green safety summaries | R3/R4/H1/H3 | Advisory host/network receipts; journal state refusal | Universal sandbox/undo claims rejected |
| D16 | Mission-first Work/History/Setup, one integrated review — IMPLEMENTED / historically runtime-inspected | Sep 7–15 UI maturation and structural docking | H2/H3; R10/R11 | Current components/IPC exist; current type/style passes | Primary legacy Reality Deck superseded; major redesign remains gated |
| D17 | Topology explains actual state; no new authority — DECIDED; operational map PROPOSED | Sep 15–16 city/raw-AST rejection | H3; R10 | Legacy 3D code exists, new Tasks/Code map not found as exposed feature | City/raw-AST primary UI REJECTED; optional 3D DEFERRED |
| D18 | Link account/control separate from proxy inference and local Core — IMPLEMENTED boundary | Existing services; H2 reaffirms split | H2; R7/R9 | Separate crates/transports; production validation not refreshed | Unnecessary Cloud coupling REJECTED |
| D19 | Provider adapters normalize into Pytxo grants — IMPLEMENTED source / UNKNOWN runtime | Sep 15 dirty commerce work | H2; R8/R13 | Dodo/Paddle adapters, transaction code, migration 008, projection | Provider-specific product logic rejected; schema needs DB evidence |
| D20 | Preserve Paddle coexistence; Dodo intent is not activation — DECIDED constraint; validation historically BLOCKED | H2 runtime gap; Sep 16 H4 user correction | H2/H4; R8/R13 | Both ingress paths present; current provider runtime UNKNOWN | Global Paddle replacement REJECTED; live cutover needs scoped authorization |
| D21 | MBCZ umbrella; Pytxo distinct product; portfolio ≠ billing state — DECIDED constraint | Sep 16 correction to creator-only framing | H4/H2 | Broker integration exists; mbcz-site outside audit | Creator-only/removing commerce SUPERSEDED; exact legal/account status UNKNOWN |
| D22 | Release complete golden path before platform breadth — DECIDED principle | H2 distinguishes vision from release completeness | H2/H3; R11 | Older native evidence; current artifact acceptance incomplete | Test-count/UI-polish readiness rejected; platform/Beta scope needs decision |
| D23 | First users/repeat use plus incremental safety measurement — EXPERIMENTAL | H1 unique catches/overhead; H2/H3 useful work and return | H1/H2/H3; R12 | No outside-cohort result found in inspected evidence | Not a PMF claim or mandatory numeric threshold; revise through research |
| D24 | Separate benchmark families and account for all resources — DECIDED methodology / experiments PROPOSED | H3 replaces dollar-only gauntlet | H2/H3; R12 | Existing scripts/dated observations, no complete new benchmark system | “$25” technical definition SUPERSEDED; content hook may remain |

## Contradiction analysis

### C1 — Review layer, independent enforcement, or broader control plane?

**Position A:** H1 rejects generic approval and favors independent commit/enforcement.
**Position B:** H2 calls “mainly the commit layer” too narrow; useful delegation must reduce supervision.
**Chronology:** H1 and H2 are undated; internal pivots are explicit, and H2 describes its own commit-first → delegation sequence. Sep 7 repository vision already contains broader framing.
**Repository evidence:** R1/R2/R4 contain both orchestration and guarded integration.
**Canonical interpretation:** Broader product purpose with a precise repository trust primitive. Review remains useful; generic review alone is not differentiation. Do not discard enforcement or turn all coordination aspirations into shipping facts.

### C2 — Success-first versus refusal-first aha

**Position A:** H2 prefers completing useful work over a product that only says no.
**Position B:** Dated H3 centers a five-minute stale-Apply refusal demonstration.
**Chronology:** H3 is Sep 15–16; H2's exact position relative to it is uncertain. No explicit repeal of useful-work value.
**Repository evidence:** R12 records successful native Apply and refusal evidence for older builds.
**Canonical interpretation:** Demonstrate a successful bounded job with a meaningful drift refusal and recovery. Which demonstration leads public messaging remains a user/positioning question.

### C3 — Broad fan-out, 100 workers, versus one-worker-first

**Position A:** H3 records cheap/free capacity and ambitious swarm ideas.
**Position B:** H3 itself corrects this to 1→4 and demand evidence; H2 and Sep 7 repo decisions favor one harness.
**Chronology:** Deliberate refinement within H3, not a current 100-agent requirement.
**Repository evidence:** Explicit tasks/waves; Flow selects one harness; no CapacityPool implementation.
**Canonical interpretation:** Worker count is an experiment variable. Maintain task conservation; no automatic swarm expansion.

### C4 — DeepSeek default versus provider independence

**Position A:** H2 report sets `deepseek-flash` default.
**Position B:** H2/H3 reject DeepSeek-dependent architecture and require local choice.
**Chronology:** Default and independence coexist in Sep 15 proposed ADR-0041.
**Repository evidence:** Config defaults DeepSeek but planner resolves explicit replaceable providers with model egress opt-in.
**Canonical interpretation:** Config default is real; remote availability/pricing/quality are unverified. No contradiction unless default becomes compulsory.

### C5 — Plugins belong in Beta versus a bounded golden-path release

**Position A:** H2 labels plugin foundations/management as Beta architecture.
**Position B:** H3 and repo increment defer marketplace/platform/protocol expansion until a named need.
**Chronology:** Absolute H2/H3 order does not establish cancellation; marketplace deferral is shared.
**Repository evidence:** No Pytxo plugin manager, SDK or generic containment path found; routing fields alone are insufficient.
**Canonical interpretation:** Marketplace deferred. Whether a minimal plugin foundation is a Beta gate remains unresolved; do not silently invent it or mark the earlier desire rejected.

### C6 — Dodo plans/code/“approved brand” versus runtime reality

**Position A:** Proposed ADR-0040 and integration prose use “verified” MBCZ and “approved” Pytxo wording.
**Position B:** H2 says no migration/provider lifecycle proof; H4 says website verification is not approval or live billing.
**Chronology:** Sep 15–16 sources overlap; not a proven account-status reversal.
**Repository evidence:** Dirty adapters and memory tests exist, migration runtime not evidenced; integration doc explicitly disclaims activation.
**Canonical interpretation:** Treat merchant/brand layout as intended, external approval UNKNOWN. Do not repeat unproven legal/account facts; preserve Paddle.

### C7 — Beta blocked by CI billing versus successful hosted jobs

**Position A:** H3 carries historical CI-quota/billing, clean-install and publication blockers.
**Position B:** Sep 12 release reconciliation reports 12 successful jobs for PR31 committed head and says blanket billing blockage is superseded.
**Chronology:** Later H3 is a retrieval of older blockers and explicitly asks to recheck.
**Repository evidence:** Local HEAD now 7287970; local remote-tracking beta ref eb5f73d; dirty source exceeds both. No live query here.
**Canonical interpretation:** Current-source hosted CI remains unproven. Current billing/quota status UNKNOWN, not confirmed blocked. Clean-Windows/publication evidence remains absent for current source.

### C8 — Public v1.2.1 versus local v1.2.2 / “latest verified candidate”

**Position A:** H3 observed public v1.2.1; old release sections call v1.2.2 locally verified.
**Position B:** Newer checkpoint contains additional dirty UI changes and newer executables.
**Chronology:** Sep 13 MSI precedes Sep 15 visual executable; same version does not imply identical source.
**Repository evidence:** Source 1.2.2, Web public constant 1.2.1, different artifact receipts/hashes.
**Canonical interpretation:** No current-tree release proof; verify exact source/artifact. Public status is a historical/local declaration until fetched.

### C9 — Topology mock semantics versus real Desktop

**Position A:** Generated H3 revamp shows ETA, counts, broad green isolation, review readiness.
**Position B:** H3 later rejects the mock as specification; host/network remain advisory.
**Chronology:** Explicit correction inside H3.
**Repository evidence:** R3/R10 show per-surface receipts and current Work/History/Setup; legacy 3D remains.
**Canonical interpretation:** Keep useful visual ideas separate from authority and supported behavior. Do not claim no topology code exists.

### C10 — “Exact approval implemented” versus the current caller contract

**Position A:** Historical hardening summaries imply exact review/approval binding.
**Position B:** Current Desktop/IPC Apply signature contains no user-reviewed digest.
**Chronology:** Source observation is current; no evidence this is a deliberate product reversal.
**Repository evidence:** R5 call chain; R4 validates the current stored contract, not the old client's displayed identity.
**Canonical interpretation:** Exact bytes/verification binding is real; stale-user-authorization resistance is partial and needs targeted proof/repair. Avoid categorical “all approvals are bound” language.

## Rejected / superseded / deferred register

| Do not casually rebuild | Status and source | Revisit condition |
|---|---|---|
| Generic extra approval popup product | REJECTED H1 | New measured unmet need, not renaming existing prompts |
| Pure safety product that only refuses work | REJECTED H2 | Preserve safety as part of useful completed work |
| Commit boundary as entire product identity | SUPERSEDED H2/R1 | Boundary remains critical; public wording can be tested |
| Mandatory lots-of-agents identity | REJECTED H2 | Evidence of task need, not agent-count marketing |
| Broad automatic fan-out this increment | REJECTED for increment R1; DEFERRED broader H3 | Fresh bounded product decision and comparative evidence |
| 100-agent near-term architecture | DEFERRED H3 | 1→4 useful-work scaling established first |
| Giant opaque AI router | DEFERRED/REJECTED initial design H2/H3 | Eligible compatibility/policy foundations and interpretable evidence |
| DeepSeek architectural lock-in | REJECTED H2/H3 | Provider selection may change, authority boundary may not |
| Mandatory Cloud for local Core | REJECTED H2 | Only explicitly selected hosted functionality requires it |
| Provider-specific entitlement logic throughout product | REJECTED H2 | Normalize at adapter boundary |
| Full plugin marketplace/enterprise platform before Beta | DEFERRED H2/H3 | Real users and bounded need; plugin foundation scope still open |
| Cyberpunk city/raw giant AST as core UI | REJECTED H3 | New evidence-led direction approval; optional 3D remains deferred |
| Unsupported ETA/progress/fake enforcement in generated mocks | REJECTED AS SPEC H3 | Actual data/mechanism and validated semantics required |
| Dollar-only “$25 business” technical benchmark | SUPERSEDED H3 | Defined resource envelope; building software still not demand proof |
| Separate harnesses imply separate unlimited quotas | SUPERSEDED H3 | Measure real provider/account/project capacity |
| Creator-only MBCZ site/removing Paddle | SUPERSEDED/REJECTED H4 | Separate explicitly authorized business change |
| Active promotion of discontinued ProdVerdict | REJECTED H4 user correction | New explicit status evidence; do not infer status of other products |

These are historical dispositions, not a new roadmap or permission to delete legacy code.
