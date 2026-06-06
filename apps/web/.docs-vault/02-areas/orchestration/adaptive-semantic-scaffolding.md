---
title: Adaptive semantic scaffolding
slug: adaptive-semantic-scaffolding
status: active
tags: [orchestration, context, tree-sitter]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-02
related: [[closed-loop-fidelity]], [[sparse-overlay-fs]], [[signal-core]]
---

# Adaptive semantic scaffolding

Implementation detail of **[[signal-core]]**. Naively stripping all function bodies destroys an agent’s understanding of dependencies. Pytxo uses an **Adaptive Semantic Scaffolder** powered by **tree-sitter**.

## Fidelity levels

Context is sliced into dynamic tiers:

| Tier | Typical consumer | Content |
|------|------------------|---------|
| Low | Orchestrator agents | Imports, signatures, module shape |
| Medium | Feature agents | Docstrings, type contracts |
| High | Builder / fix agents | Targeted implementation blocks |

## Closed loop

On compile or test failure, fidelity **automatically increases** for the next iteration—injecting implementation blocks for failing dependencies. See [[closed-loop-fidelity]].

## Pipeline position

```text
Disk → [[sparse-overlay-fs]] → Scaffolder → [[dag-flow-engine]] → Execution
```
