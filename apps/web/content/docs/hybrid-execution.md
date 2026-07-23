---
title: Pytxo Cloud hybrid execution
slug: hybrid-execution
status: active
tags: [cloud, architecture]
audience: [human, agent]
layer: cloud
created: 2026-06-02
updated: 2026-07-23
related: [[sandbox-dispatch]], [[delta-sync]], [[three-tier-model]], [[pytxo-improvement-research]]
---

# Pytxo Cloud hybrid execution

**Pytxo Cloud** is the optional path that pairs the local-first control plane with **isolated cloud sandboxes** for heavy work—without replacing BYOK or local secrets management.

**Capability gate:** the default orchestration path uses `NoopCloudDispatcher`. Treat Cloud as **configured when an HTTP dispatcher (or equivalent) is wired** — not as a GA Max-tier default. Maturity: [[pytxo-improvement-research]].

```text
[Local control plane] ── encrypted TLS / P2P ──► [Cloud sandbox]
  · lean telemetry          · high-core CPU/GPU
  · key handling            · multi-agent builds
  · secrets stay local      · Docker test runs
```

## When to use

- Large parallel swarms that saturate local silicon
- `docker-compose` integration tests
- Compiler-heavy pipelines

Only after Cloud dispatch is configured for the environment. Until then, stay on the local execution yard.

Flow (when configured): [[sandbox-dispatch]] → [[delta-sync]] → results on [[presentation-passive-telemetry|Pytxo Desktop]].
