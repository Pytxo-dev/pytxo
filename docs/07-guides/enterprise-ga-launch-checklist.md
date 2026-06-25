---
title: Enterprise GA launch checklist
slug: enterprise-ga-launch-checklist
status: active
tags: [guide, enterprise, launch]
audience: [human, operator]
layer: guide
created: 2026-06-24
updated: 2026-06-24
related: [[competitive-benchmarks]], [[link-db-backup-runbook]]
---

# Enterprise GA launch checklist

Use this before enabling Enterprise org features in production.

## Pytxo Link (Postgres)

- [ ] `DATABASE_URL` set; migrations applied through `005_seats.sql`
- [ ] `LINK_REQUIRE_AUTH=1` with Clerk JWKS or API key
- [ ] `LINK_ADMIN_KEY` rotated; stored in secrets manager only
- [ ] `LINK_ORG_SEATS_DEFAULT` matches contract default (fallback when `org_seats` row missing)
- [ ] Smoke: `GET /v1/orgs/{org_id}/seats` returns used/total from Postgres
- [ ] Smoke: `PUT /v1/orgs/{org_id}/policy` (admin) writes policy + audit row
- [ ] Smoke: `PUT /v1/admin/orgs/{org_id}/seats` updates seat total + audit row
- [ ] Backup runbook tested — see [[link-db-backup-runbook]]

## Web account surface

- [ ] `NEXT_PUBLIC_LINK_URL` points at production Link
- [ ] Signed-in `/account` shows tier, seats, and org policy when `org_id` is provisioned
- [ ] Paddle webhooks update entitlements with `org_id`

## Desktop / CLI

- [ ] Org policy ceiling fetched for team members (`pytxo doctor` / TUI board)
- [ ] Ultra LLM planner (`PYTXO_PLANNER_LLM=1`) only enabled for Ultra billing tenants

## Observability

- [ ] Link `/health` and proxy `/health` monitored
- [ ] Audit log retention policy documented for `audit_log` table

## Marketing honesty

- [ ] Competitive benchmark table in [[competitive-benchmarks]] filled with pinned hardware results
- [ ] Reality Deck 3D stills exported per [[deck-3d-stills]]
