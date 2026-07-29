---
title: Container diagram (C4 L2)
slug: c4-container
status: active
tags: [architecture, c4]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-07-29
related: [[three-tier-model]], [[context-diagram]]
---

# Container diagram (C4 L2)

```mermaid
flowchart TB
  subgraph presentation [Presentation]
    ui[Svelte 5 Pytxo Desktop]
  end
  subgraph orchestration [Orchestration]
    core[Rust core]
    wal[(SQLite WAL)]
    scaffold[Semantic scaffolder]
    dag[DAG flow engine]
  end
  subgraph execution [Execution]
    yard[Local execution yard]
    overlay[Worktree or sparse copy-layer]
  end
  subgraph cloud [Pytxo Cloud - configured only]
    sandbox[Cloud sandbox]
  end

  ui <-->|Tauri IPC| core
  core --> wal
  core --> scaffold
  core --> dag
  core --> yard
  yard --> overlay
  core -.->|non-noop dispatcher over TLS| sandbox
```

The dashed Cloud edge is absent on the default `NoopCloudDispatcher` path.
Kernel FUSE / ProjFS remains a north star; the shipping overlay path is the
sparse copy-layer. See [[three-tier-model]], [[hybrid-execution]], and
[[sparse-overlay-fs]] for narrative detail.
