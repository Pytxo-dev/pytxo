---
title: Three moats
---

# Three moats

Pytxo orchestration routes through three layers — not around them.

| Moat | Codename | Function |
|------|----------|----------|
| Context arbitrage | [Signal Core](/docs/concepts/signal-core) | Scaffold file context so agents spend tokens on work, not noise |
| Copy-on-write sandbox | [Blast Shield](/docs/concepts/blast-shield) | Isolate writes until explicit approval |
| Concurrency guard | [Race Shield](/docs/concepts/race-shield) | Lock-free registry + stdin buffering across swarms |

```text
         Signal Core          Blast Shield         Race Shield
              │                     │                    │
              └────────── orchestration plane ──────────┘
                              │
                         PTY agents
```

On **Galaxy**, [human approvals](/docs/concepts/galaxy-approvals) add a fourth safety layer for risky commands and merges.

Each moat has a dedicated concept page with configuration knobs.
