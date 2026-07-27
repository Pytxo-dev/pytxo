---
title: ADR-0031 Mission planner default and BYOK scout
slug: adr-0031-mission-planner-byok-scout
status: accepted
tags: [adr, orchestration, planner, mission]
audience: [human, agent]
layer: orchestration
created: 2026-07-27
updated: 2026-07-27
adr_id: ADR-0031
related: [[mission-loop]], [[signal-core]], [[ADR-0012-hypervisor-shell-default-ux]], [[product-vision]]
---

# ADR-0031: Mission planner default and BYOK scout

## Status

Accepted

## Context

Desktop Flow already decomposes natural-language missions with `SignalBackedPlanner` without requiring `PYTXO_PLANNER`. The Hypervisor Shell and CLI still gated NL planning behind `PYTXO_PLANNER=1` or `[planner] enabled`, and the only LLM planner path required Ultra billing plus the managed inference proxy. That blocked the frictionless loop: `pytxo mission "…"`.

## Decision

1. **`pytxo mission` and Flow** use Signal-backed planning by default (no first-run planner flag).
2. **BYOK scout LLM** may decompose missions when a provider API key is present via the existing provider catalog — not Ultra-proxy-only.
3. Offline / no-key fallback remains Signal-backed, then heuristic, with honest low-confidence warnings.
4. Scout may suggest `verify` shell commands; it never invents passing checks.
5. Operators may still set `[planner] enabled = false` or `PYTXO_PLANNER=0` to force stub/disable for non-mission slash/`run` paths that historically required an explicit enable.

## Consequences

**Positive**

- First-run value without hand-written `[[task]]` rows.
- Aligns CLI with Desktop Flow planning behavior.
- Keeps BYOK as the default commercial posture for planning.

**Negative / tradeoffs**

- Weak offline plans when Signal cannot infer paths (operator must edit the plan).
- BYOK scout quality depends on the operator’s chosen model.

## Links

- Related: [[mission-loop]], [[pytxo-improvement-research]]
- Supersedes: Ultra-only gate for LLM mission decompose (behavioral; ADR-0009 metering unchanged)
