---
title: Three moats
---

# Three moats

Future orchestration in Pytxo routes through three layers — not around them.

| Moat | Codename | Function |
|------|----------|----------|
| Context arbitrage | [Signal Core](/docs/concepts/signal-core) | Scaffold file context so agents spend tokens on work, not noise |
| Copy-on-write sandbox | [Blast Shield](/docs/concepts/blast-shield) | Isolate writes until explicit approval |
| Concurrency guard | [Race Shield](/docs/concepts/race-shield) | Lock-free registry + stdin buffering across swarms |

Implementation status varies by crate — see the [repo layout](/docs/developers/repo-layout) and crate READMEs for what ships today vs planned.

```text
         Signal Core          Blast Shield         Race Shield
              │                     │                    │
              └────────── orchestration plane ──────────┘
                              │
                         PTY agents
```

Each moat has a dedicated concept page with behavior details and configuration knobs.
