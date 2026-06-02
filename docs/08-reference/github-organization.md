---
title: GitHub organization
slug: github-organization
status: active
tags: [reference, gtm]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[gtm-open-source-loop]], [[mvp-bootstrap]]
---

# GitHub organization

**Canonical org:** [github.com/Pytxo-dev](https://github.com/Pytxo-dev)

Use this organization for all first-party Pytxo repositories, releases, and CI — not personal forks as the source of truth.

## Repositories

| Repo | URL | Contents |
|------|-----|----------|
| **pytxo** | https://github.com/Pytxo-dev/pytxo | Monorepo: `crates/*`, `apps/desktop`, `docs/`, benchmarks |

Future repos (examples, website, infra) should follow the same org and naming (`pytxo-*` where helpful).

## Conventions

- **Issues & PRs:** Open against the repo that contains the code (usually `pytxo`).
- **Cargo / package metadata:** `repository = "https://github.com/Pytxo-dev/pytxo"` at workspace root.
- **Fork workflow:** Fork from `Pytxo-dev/*`, PR back to `Pytxo-dev/*`.
- **User projects:** Developers keep their own app repos; Pytxo runs *in* those repos via `pytxo run --repo`. Only Pytxo *product* code lives under Pytxo-dev.

## Local clone

```bash
git clone https://github.com/Pytxo-dev/pytxo.git
cd pytxo
cargo test --workspace
```

Requires a Git repo with at least one commit before `pytxo run` (worktrees need `HEAD`). See [[first-three-agent-run]].
