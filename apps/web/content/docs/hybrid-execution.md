---
title: Pytxo Cloud hybrid execution
slug: hybrid-execution
status: active
tags: [cloud, architecture]
audience: [human, agent]
layer: cloud
created: 2026-06-02
updated: 2026-06-02
related: [sandbox-dispatch](/docs/sandbox-dispatch), [delta-sync](/docs/delta-sync), [three-tier-model](/docs/three-tier-model)
---

# Pytxo Cloud hybrid execution

**Pytxo Cloud** pairs the local-first control plane with **isolated cloud sandboxes** for heavy work—without replacing BYOK or local secrets management.

```text
[Local control plane] ── encrypted TLS / P2P ──► [Cloud sandbox]
  · lean telemetry          · high-core CPU/GPU
  · key handling            · multi-agent builds
  · secrets stay local      · Docker test runs
```

## When to use

- Large parallel swarms
- `docker-compose` integration tests
- Compiler-heavy pipelines that would saturate local silicon

Flow: [sandbox-dispatch](/docs/sandbox-dispatch) → [delta-sync](/docs/delta-sync) → results on [Reality Deck](/docs/presentation-passive-telemetry).
