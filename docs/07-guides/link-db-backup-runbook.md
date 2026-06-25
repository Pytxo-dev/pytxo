---
title: Link Postgres backup runbook
slug: link-db-backup-runbook
status: active
tags: [guide, ops, postgres, link]
audience: [human]
layer: guides
created: 2026-06-24
updated: 2026-06-24
related: [[ADR-0021-billing-source-of-truth]], [[ADR-0027-service-observability-contract]]
---

# Link Postgres backup runbook

Operational steps for backing up the **Pytxo Link** Postgres database (`DATABASE_URL` on Railway or self-hosted). Link stores entitlements, org policies, run ledger rows, audit events, and inference usage when Postgres is enabled.

## Prerequisites

- `DATABASE_URL` connection string (owner or read-replica with replication privileges for physical backups)
- `pg_dump` / `psql` client matching server major version (Postgres 15+ on Railway)
- Encrypted object storage or offline vault for dump files (never commit dumps to git)

## Logical backup (recommended for migrations)

```bash
export DATABASE_URL="postgresql://user:pass@host:5432/pytxo_link"
ts=$(date -u +%Y%m%dT%H%M%SZ)
pg_dump "$DATABASE_URL" \
  --format=custom \
  --no-owner \
  --file="pytxo-link-${ts}.dump"
```

Verify the archive:

```bash
pg_restore --list "pytxo-link-${ts}.dump" | head
```

Store `pytxo-link-${ts}.dump` in your backup bucket with SSE enabled. Retain at least **30 daily** and **12 monthly** copies for production.

## Restore to a fresh database

```bash
createdb pytxo_link_restore
pg_restore --dbname="$RESTORE_DATABASE_URL" --no-owner --clean --if-exists "pytxo-link-${ts}.dump"
```

Point a staging Link instance at `RESTORE_DATABASE_URL`, run migrations if needed (`sqlx migrate` ships with the service image), and smoke `/health`:

```bash
curl -fsS http://127.0.0.1:8787/health | jq .
```

## Railway-specific notes

1. Open the Link service → **Variables** → copy `DATABASE_URL` (or use Railway CLI `railway variables`).
2. Run `pg_dump` from a trusted CI runner or local machine with network access to the Railway Postgres host.
3. Schedule backups via GitHub Actions cron or Railway cron job invoking the logical dump command above.
4. After restore, redeploy Link so connection pools recycle.

## Pre-backup checklist

- [ ] Confirm Link `/health` returns `"status":"ok"` (see [[ADR-0027-service-observability-contract]])
- [ ] Pause destructive admin scripts during dump window (optional; custom format is consistent for small DBs)
- [ ] Record dump checksum: `sha256sum pytxo-link-*.dump`

## Post-restore validation

- [ ] `GET /v1/entitlements/status` returns expected tier for a test user
- [ ] Org policy row exists for a known `org_id`
- [ ] Recent `runs` ledger rows visible when `link_reconcile` is enabled

## Disaster recovery

If Link Postgres is lost without backups, re-provision the database, run migrations from `services/pytxo-link/migrations/`, and reconcile entitlements from Paddle/Clerk admin exports per [[ADR-0021-billing-source-of-truth]]. Run ledger history cannot be reconstructed without backups.

Back: [[MOC-home]].
