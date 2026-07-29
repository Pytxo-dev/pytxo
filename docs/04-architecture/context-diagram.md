---
title: System context (C4 L1)
slug: context-diagram
status: active
tags: [architecture, c4]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-07-29
related: [[three-tier-model]], [[c4-container]]
---

# System context (C4 L1)

```mermaid
flowchart LR
  dev[Developer]
  ide[IDE or CLI]
  pytxo[Pytxo control plane]
  agents[Headless agent CLIs]
  cloud[Pytxo Cloud sandbox - configured only]
  llm[LLM providers BYOK]

  dev --> ide
  dev --> pytxo
  ide -->|MCP| pytxo
  pytxo --> agents
  pytxo -.->|non-noop dispatcher| cloud
  agents --> llm
  cloud --> llm
```

Pytxo sits between the developer’s environment and parallel agent processes.
The dashed Cloud path exists only when a non-noop dispatcher is configured;
local execution is the default ([[hybrid-execution]]).
