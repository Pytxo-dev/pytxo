---
title: ADR-0041 Advisory coordinator and routing boundary
slug: ADR-0041-advisory-coordinator-and-routing-boundary
adr_id: ADR-0041
status: proposed
tags: [adr, orchestration, coordinator, routing, models]
audience: [human, agent]
layer: orchestration
created: 2026-09-15
updated: 2026-09-15
related: [[mission-loop]], [[ADR-0031-mission-planner-byok-scout]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[permission-profile-engine]], [[pytxo-link-service]]
---

# ADR-0041: Advisory coordinator and routing boundary

## Status

Proposed. The configuration and validation seams exist as an opt-in foundation;
this record does not approve an autonomous router or hosted coordinator service.

## Context

Pytxo already has worker provider metadata, CLI harness discovery, Signal-backed
planning, MCP registration, execution backends, permission profiles, mission
state, verification, and reviewed Apply. The optional LLM planner, however,
selected whichever supported API key happened to be present and hard-coded a
managed DeepSeek route. Extending that path directly into Routing would create a
second, model-owned control plane and duplicate systems Pytxo Core already owns.

The desired coordinator may help reason about a task, propose a plan, rank
routes, and diagnose failure. Future routes may combine a model, harness,
skills, MCP/tools, plugins, and execution setup. Those choices can be
probabilistic; authority and state transitions cannot be.

## Decision

The coordinator is a logical advisory role, not a mandatory central service.
It has an explicit provider, model, and transport profile independent from
worker-agent profiles. `direct` calls a selected provider or local
OpenAI-compatible endpoint. `managed` calls the optional Pytxo inference proxy
and requires its existing entitlement boundary. Local Core, heuristic planning,
Ollama, and LM Studio do not require Pytxo Cloud or Pytxo Link.

The initial profile is DeepSeek `deepseek-flash`, while model egress remains
off until the caller explicitly enables it. This is a replaceable operational
default, not a protocol dependency. DeepSeek's 2026-09-10 update documents
Flash as available and says the V4.1 Pro endpoint temporarily routes to Flash,
so Pytxo must not encode “Pro” as a guaranteed capability:
[DeepSeek update](https://api-docs.deepseek.com/news/news260910/).

Routing follows a narrow pipeline:

1. Core discovers current models, harnesses, skills, tools, plugins, execution
   backends, and isolation mechanisms.
2. Core applies deterministic compatibility, entitlement, trust, and policy
   filters to produce an eligible catalog snapshot.
3. The coordinator receives bounded task context and that snapshot, then emits
   a typed advisory proposal.
4. Core rejects any component absent from the current snapshot, rechecks policy,
   materializes the route, and records the inputs and decision evidence.
5. Existing mission and execution machinery owns dispatch and state. Existing
   verification, immutable candidate identity, approval, and Apply gates remain
   unchanged.

The initial `RouteProposal` contains only model, harness, skills, tools, plugins,
execution backend, and isolation choice. It deliberately has no permission
profile, mission-state mutation, approval, verification result, candidate ID,
or Apply authority. A diagnosis is evidence or advice until deterministic code
or a human accepts a resulting action.

Pytxo Link remains an optional account/control API for identity, organization
policy, commerce-derived entitlements, and hosted-usage accounting. It is not
the source of local mission truth. The managed inference proxy remains a
separate secret-bearing transport. A future hosted API can reuse versioned
coordinator request/response contracts without moving local safety state into
the cloud.

## Consequences

Provider and model replacement no longer changes planner logic. Ambient API
keys cannot silently choose a coordinator. Local endpoints remain first-class.
Future Routing can add catalog descriptors, scoring evidence, and provider-
specific adapters incrementally without building a marketplace or managed
inference platform first.

This foundation does not yet perform route ranking, benchmark models, install
plugins, discover arbitrary tools, or host coordinator sessions. The current
network adapter covers OpenAI-compatible endpoints only; other provider
protocols require explicit adapters behind the same advisory contract.
