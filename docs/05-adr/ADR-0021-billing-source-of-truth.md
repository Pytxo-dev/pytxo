---
title: ADR-0021 Billing source of truth and run ledger
slug: adr-0021-billing-source-of-truth
status: accepted
tags: [adr, billing, commercial]
audience: [human, agent]
layer: architecture
created: 2026-06-22
updated: 2026-06-22
related: [[ADR-0009-ultra-managed-metering]], [[pytxo-link-service]]
---

# ADR-0021: Billing source of truth and run ledger

## Status

Accepted (2026-06-22)

## Context

Ultra billing used an in-memory `HashMap` on Pytxo Link for `/v1/runs/start` and `/v1/runs/end`. Local `TokenWallet` in `pytxo-store` held reservations but had no durable cross-device ledger. Production requires idempotent reconcile and survivable restarts.

## Decision

1. **Link Postgres `runs` table** is the authoritative run ledger for Ultra reconcile (`migrations/003_runs.sql`).
2. **Local `TokenWallet`** remains an offline cache and reservation engine; on reconcile success, local and Link totals should align.
3. **Idempotency**: `run_id` is the primary key; `ON CONFLICT DO NOTHING` on start; end upserts usage JSON.
4. **Production Link** requires `DATABASE_URL` and `LINK_REQUIRE_AUTH=1`.

## Consequences

- Link restarts do not lose run history when Postgres is configured.
- CLI `doctor` must HTTP-ping Link `/health` when `link_reconcile` is enabled (not URL-parse only).
- Paddle webhooks on Link must verify HMAC signatures when `PADDLE_WEBHOOK_SECRET` is set.

## Alternatives considered

- Local wallet as sole source of truth — rejected (no cross-device billing).
- Event-sourcing only — deferred; simple upsert ledger sufficient for v1.
