---
title: Jev and Pytxo routing research, September 22 2026
slug: jev-typesafe-research-2026-09-22
status: draft
tags: [research, routing, coordinator, evidence]
audience: [human, agent]
layer: orchestration
created: 2026-09-22
updated: 2026-09-22
related: ["[[2026-09-22-jev-routing-design]]", "[[PYTXO_HISTORY]]", "[[ADR-0041-advisory-coordinator-and-routing-boundary]]"]
---

# Jev and Pytxo routing research

This is a source review, not a Jev benchmark or an implementation report. No inference requests, credential inspection, provider enrollment, paid trials, or deployment were performed.

## Conversation and history boundary

Read all seven returned turns of **Compare JEV GPT6 Astra Cost**, conversation `6aafcd1f-f7d8-83ec-84fa-8c27e55dceb7`; the API reported no older page. The original image attachment was unavailable. The text is sufficient for this architecture; no claim is made about that image. The conversation develops cheap semantic routing into event-driven orchestration, then qualifies free routing, execution-account ownership, confidence, evaluation, and training rights. Illustrative scores and savings are not measurements.

Skimmed **Summarize Pytxo Conversations** and **Local Model Harness Setup**, plus the repository's [[PYTXO_SOURCE_LEDGER]], [[PYTXO_HISTORY]], [[PYTXO_DECISIONS]], and [[PYTXO_ARCHITECTURE]]. The first chat reports a reconstructed archive; its PDF was not available in the returned content. The second substantially concerns Phonton: its model-download and adaptive-search ambitions are not Pytxo requirements. The repository's H2/H3 records are attributed historical summaries, not independently recovered approvals.

The surviving Pytxo direction is useful delegation with one worker as a valid starting point, separate model/harness/backend/account capacity, optional model advice, and Core-owned verification and integration. Mandatory swarms, independent unlimited quotas from additional CLIs, and an always-running expensive manager do not follow from that history. The September 13 checkpoint records broader conversational supervision as approved design history but unfinished implementation; this proposal does not relabel it complete.

## First-party Jev findings

| Finding checked on September 22 | Architectural consequence | Source |
|---|---|---|
| `jev-1.13.0`; input price $0.042 per million tokens; output uncharged; text input; 64k total context and 32k state plus longest question. Listed limits are 1,200 requests/minute and 250,000 tokens/second, explicitly subject to change. | Pin the evaluated model, keep packets small, meter returned usage, and configure upstream limits rather than assuming permanent capacity. | [Models](https://docs.typesafe.ai/models) |
| `POST https://api.typesafe.ai/v1/systemone` accepts state and typed questions, returning answers, resolved model, and token usage. | A dedicated adapter is required; changing the model name in Pytxo's chat-completions planner is insufficient. | [API reference](https://docs.typesafe.ai/api) |
| Choice selects supplied alternatives; Score evaluates an ordered rubric; Noul reports a yes/no probability. Questions in a request share state but are evaluated independently. | Batch independent questions. A dependent second evaluation requires explicit code and a new request. | [Primitives](https://docs.typesafe.ai/primitives) |
| Choice/Score confidence is derived from the answer distribution. Noul has no separate confidence field. | Confidence is neither coding-success probability nor permission to execute. Calibrate each question/version separately. | [Confidence](https://docs.typesafe.ai/confidence) |
| Documented weaknesses include arithmetic, indirection, irrelevant context, adversarial state, and text generation. Separate questions need not satisfy expected probability identities. | Keep budgets, clocks, graph validation, state transitions and verification in code; use a generative planner for new task descriptions. | [Jev 1.13 limitations](https://docs.typesafe.ai/model-jaggedness/jev-1.13) |
| The documentation describes no customer fine-tuning/LoRA route. Enterprise ZDR is a separate offering. | Design request templates and a replaceable adapter; do not promise private trained Jev or universal zero retention. | [Models](https://docs.typesafe.ai/models), [Legal](https://docs.typesafe.ai/legal) |

The September 19 [customer agreement](https://typesafe.ai/legal/mca) permits API integration into customer applications and restricts standalone service access, distillation, imitation training, and similar/competing product development using the service or outputs. Its data terms distinguish model training from telemetry and other processing. An included Pytxo feature appears consistent with the application model, but this is a design interpretation, not confirmation of Pytxo's specific rights.

There is a relevant nuance: TypeSafe's [feature-discovery cookbook](https://docs.typesafe.ai/cookbooks/autoresearch_feature_discovery) demonstrates downstream supervised learning using Jev-derived features. That is not blanket permission to distill a substitute or operate a competing routing service. Keep independently observed execution outcomes separate from Jev outputs, and clarify intended training/service uses in writing before relying on them commercially. No contact was made during this review.

General searches returned unofficial Jev-branded sites with different endpoints and prices. They were not used as technical evidence. No current Astra price claim is taken from the historical chat; execution economics require the actual selected account and billing mode.

## Current checkout evidence

Inspected working tree at HEAD `72879702f90f2b74eece888bb117df59608f9b56`, with 361 pre-existing status entries. Relevant dirty and untracked source was included; HEAD alone does not identify this state.

| Source and symbol | What exists / limit |
|---|---|
| `crates/pytxo-core/src/coordinator.rs`: `CoordinatorConfig`, `RouteProposal`, `validate_route_proposal` | Explicit coordinator profile and component-list membership validation. No production caller of the validator was found in the inspected Core, service or Desktop source. No complete compatibility catalog is established. |
| `crates/pytxo-planner/src/lib.rs`: `LlmPlanner`, `coordinator_endpoint` | Explicit opt-in generative planning using an OpenAI-compatible endpoint; local endpoint support and separate managed entitlement path. |
| `crates/pytxo-orchestrate/src/flow.rs`: Flow dispatch | Selects one ADE and shared task command template for the run; rechecks the reviewed plan and checkout. |
| `crates/pytxo-core/src/task.rs`: `AgentSpec` | Lower-level per-agent model/provider/CLI metadata is broader than Flow's selection UI. That does not establish operational cross-harness routing. |
| `crates/pytxo-scheduler/src/dag.rs`, runner dependency tests | Dependency/path scheduling and failed-upstream handling exist. Arbitrary semantic or external-resource independence is not proven. |
| `crates/pytxo-orchestrate/src/lib.rs`: `apply_run_changes`; Desktop IPC | Current source requires the expected reviewed package digest. The September 16 context pack's missing-digest observation is stale. |
| `crates/pytxo-store/src/store.rs`: `RunContractRecord`, `EventRecord` | Durable run/review/event seams exist; a routing decision/attempt ledger remains an addition. |
| `services/pytxo-link/src/limits.rs`, `inference.rs` | Existing per-process IP limiting and usage insertion are not atomic sponsored-token reservation or request idempotency. |

Representative SHA-256 identities at inspection:

- `coordinator.rs`: `F1BC8598B66DDDA9634663FE8CE69E940F8A5A4C23F12870BCE3B55889834FCE`
- planner `lib.rs`: `980A59C473B8EE9D20B2722BBFD1BB8AF37B93615898BA1333A7EA6D8D21FAB6`
- orchestrate `flow.rs`: `D396B1049587529800B5EB3191F40F2568F36EA5D519C49821FD1C5C8D1480F0`
- orchestrate `lib.rs`: `6946416852E75897CF64488F0CA669831D17DF5712430DEC57BEEBACBCE70943`
- store `store.rs`: `F56ABD64D272B150BA2DDFE2D224F14E5F190B854611307BBAC6F349D2AC7B55`

[[ADR-0041-advisory-coordinator-and-routing-boundary]] remains **proposed**. Existing billing `ModelRouter` is transport/billing routing, not the proposed task allocation system. No Jev integration, active execution-profile selector, shared capacity pool, or portable attempt-handoff subsystem was found. Source presence and test definitions are the evidence level here; no existing runtime or suite was revalidated in this design-only task.

## Second-pass implementation stress review

The same source identities were rechecked for [[2026-09-22-jev-routing-stress-review]]. Additional inspection found an internal Signal retry in runner/run.rs, all-result candidate aggregation in orchestrate/lib.rs, Codex/Orbit/single-worker Desktop Beta restrictions, separate SQLite/process-file crash boundaries, and proxy authentication/limiting that cannot be reused unchanged for sponsorship. The revised design records these as implementation seams, not claims that new routing already exists.

Reopened TypeSafe's official API and model pages on September 22. Their documented context budget and returned usage do not explicitly provide a complete upper billing guarantee for ambiguous requests; the first draft's unconditional 64k reservation claim has been narrowed in [[2026-09-22-jev-routing-service]]. No upstream evaluation was made. The earlier broad Jev skill/failure/dependency advice is now later work; V1a tests only the initial semantic profile decision in [[2026-09-22-jev-routing-benchmark]].
