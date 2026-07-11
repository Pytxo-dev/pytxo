---
title: Three moats
---

# Three moats

Every Pytxo run goes through three safety and efficiency layers.

| What it does | Codename | Function |
|------|----------|----------|
| Smarter context | [Signal Core](/docs/concepts/signal-core) | Send agents code skeletons so they spend tokens on work, not noise |
| Safe sandbox | [Blast Shield](/docs/concepts/blast-shield) | Keep writes isolated until you approve a merge |
| No write collisions | [Race Shield](/docs/concepts/race-shield) | Registry + stdin buffering so swarms do not stomp each other |

```text
         Signal Core          Blast Shield         Race Shield
              │                     │                    │
              └────────── orchestration plane ──────────┘
                              │
                         Agents
```

On **Galaxy**, [human approvals](/docs/concepts/galaxy-approvals) add a fourth layer for risky commands and merges.

Each moat has its own concept page with configuration options.
