---
title: Cloud sandbox service (separate repo)
slug: cloud-sandbox-service
status: active
tags: [cloud, reference]
audience: [human, agent]
layer: cloud
created: 2026-06-04
updated: 2026-06-04
related: [[hybrid-execution]], [[sandbox-dispatch]]
---

# Cloud sandbox service (separate repo)

Pytxo **Max Swarm** hosted sandboxes are implemented as a **separate deployable service**, not inside the `pytxo` monorepo control plane.

## Monorepo scope (this repository)

- Local hypervisor, PTY execution yard, Pytxo Desktop, MCP, Ultra **local** wallet ledger
- [`HttpBillingReconciler`](../../crates/pytxo-core/src/billing/link_reconciler.rs) builds the Pytxo Link request **envelopes** (endpoints + JSON bodies for `runs/start` and `runs/end`) and is wired behind `billing.link_reconcile`. The HTTP transport itself is attached when the Link service is deployed, so the monorepo stays network-free and offline-testable.

## Pytxo Link service (separate private repo — Phase 15)

- Auth, run start/end ingestion, usage reconciliation, provider webhooks
- Consumes the envelopes above; deploy + document the base URL in `pytxo.toml.example` (`billing.proxy_url`)

## Cloud sandbox service (separate repo — shipped POC)

Repository: **`pytxo-cloud-sandbox`** (sibling to this monorepo; local dev `http://127.0.0.1:8788`).

- Axum API: `/health`, `/v1/sandboxes/*`, `/v1/cache/scaffold`
- Worker POC for remote `exec` / delta `sync`
- Monorepo client: `ExecutionBackend::Cloud`, `[cloud]` in `pytxo.toml`, `PYTXO_CLOUD_SESSION` auth
- Pro context cache: read-through / write-through in runner `prepare_agent_context` (`content_hash` keys)

When deployed, link production URL from [[github-organization]] and set `cloud.sandbox_url` in `pytxo.toml`.
