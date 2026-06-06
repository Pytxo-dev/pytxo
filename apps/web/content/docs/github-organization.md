---
title: GitHub organization
slug: github-organization
status: active
tags: [reference, gtm]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-04
related: [gtm-open-source-loop](/docs/gtm-open-source-loop), [mvp-bootstrap](/docs/mvp-bootstrap), [repository-layout](/docs/repository-layout), [cloud-sandbox-service](/docs/cloud-sandbox-service)
---

# GitHub organization

**Canonical org:** [github.com/Pytxo-dev](https://github.com/Pytxo-dev)

Use this organization for all first-party Pytxo repositories, releases, and CI — not personal forks as the source of truth.

## Repository inventory (live)

Last enumerated with `gh repo list Pytxo-dev` (2026-06-04). Re-run that command when adding repos.

| Repository | Visibility | Archived | Purpose | Relationship to monorepo |
|------------|------------|----------|---------|---------------------------|
| [pytxo](https://github.com/Pytxo-dev/pytxo) | **Public** | no | Agentic control plane + Reality Deck | **Canonical** — `crates/*`, `apps/desktop`, `docs/`, `tooling/` |
| [pytxo-desktop](https://github.com/Pytxo-dev/pytxo-desktop) | **Public** | no | Legacy split UI repo | **Mirror / legacy** — canonical UI is `apps/desktop` in **pytxo** |

No **private** or **internal** repositories were returned for this org at enumeration time. If your token cannot see private org repos, confirm visibility in GitHub org settings before assuming they do not exist.

## Public vs private policy

| What | Where | Visibility |
|------|--------|------------|
| OSS control plane (Rust crates, CLI, MCP, local hypervisor, Deck) | **pytxo** monorepo | Public |
| Docs vault (product + architecture) | **pytxo** `docs/` | Public |
| CI, smoke scripts, benchmarks | **pytxo** `tooling/` | Public |
| Hosted cloud sandboxes (Max Swarm workers) | Separate service repo | **Planned private** (or org-private) — see [cloud-sandbox-service](/docs/cloud-sandbox-service) |
| Pytxo Link (Ultra billing HTTP reconcile) | Separate service repo | **Planned** — local stub in monorepo until deployed |
| Customer secrets / BYOK vault | Never in public repos | Private infrastructure only |

**Rule of thumb:** Anything that runs untrusted user code at scale, holds payment webhooks, or stores API keys stays **out of the public monorepo**. The monorepo ships local execution, telemetry, and integration **stubs** (e.g. `HttpBillingReconciler`).

## Planned repositories (not on GitHub yet)

These names appear in product/architecture docs only. Do not treat them as existing until `gh repo list` shows them.

| Planned name | Role | Doc |
|--------------|------|-----|
| `pytxo-link` (working title) | Ultra metering HTTP API + reconcile webhooks | [ADR-0009-ultra-managed-metering](/docs/adr-0009-ultra-managed-metering), `billing.link_reconcile` in `pytxo.toml` |
| `pytxo-cloud-sandbox` | Hosted sandbox dispatch + delta sync + context cache (POC shipped) | [cloud-sandbox-service](/docs/cloud-sandbox-service) |

Optional later: **pytxo-docs** if the vault needs its own release cycle (still public OSS unless you split commercial docs).

## Conventions

- **Issues & PRs:** Open against **pytxo** for control plane, docs, and Reality Deck unless tracking mirror-only work on **pytxo-desktop**.
- **Cargo metadata:** `repository = "https://github.com/Pytxo-dev/pytxo"` at workspace root.
- **Fork workflow:** Fork from `Pytxo-dev/*`, PR back to `Pytxo-dev/*`.
- **User projects:** Developers keep their own app repos; Pytxo runs *in* those repos via `pytxo run --repo`. Only Pytxo *product* code lives under Pytxo-dev.

See [repository-layout](/docs/repository-layout) for path boundaries. Desktop and control plane share workspace `0.1.x` via path deps.

## Local clone

```bash
git clone https://github.com/Pytxo-dev/pytxo.git
cd pytxo
cargo run -p pytxo-cli -- doctor
cargo test --workspace
```

Reality Deck (same repo):

```bash
cd apps/desktop
npm ci && npm run check
cd ../..
cargo build -p pytxo-desktop
```

Requires a Git repo with at least one commit before `pytxo run` (worktrees need `HEAD`). See [first-three-agent-run](/docs/first-three-agent-run).

## Refreshing this page

```bash
gh repo list Pytxo-dev --limit 200 --json name,visibility,description,url,isArchived
```

Update the inventory table when repos are added, archived, or visibility changes.
