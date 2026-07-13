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

- [x] `DATABASE_URL` set; migrations applied through `005_seats.sql`
- [x] `LINK_REQUIRE_AUTH=1` with Clerk JWKS or API key
- [x] `LINK_ADMIN_KEY` rotated; stored in secrets manager only
- [x] `LINK_ORG_SEATS_DEFAULT` matches contract default (fallback when `org_seats` row missing)
- [x] Smoke: `GET /v1/orgs/{org_id}/seats` returns used/total from Postgres
- [x] Smoke: `PUT /v1/orgs/{org_id}/policy` (admin) writes policy + audit row
- [x] Smoke: `PUT /v1/admin/orgs/{org_id}/seats` updates seat total + audit row
- [x] Backup runbook tested — see [[link-db-backup-runbook]]

## Web account surface

- [x] `NEXT_PUBLIC_LINK_URL` points at production Link
- [x] Signed-in `/account` shows tier, seats, and org policy when `org_id` is provisioned
- [x] Paddle webhooks update entitlements with `org_id`

## Desktop / CLI

- [x] Org policy ceiling fetched for team members (`pytxo doctor` / TUI board)
- [x] Ultra LLM planner (`PYTXO_PLANNER_LLM=1`) only enabled for Ultra billing tenants

## Observability

- [x] Link `/health` and proxy `/health` monitored
- [x] Audit log retention policy documented for `audit_log` table

## Marketing honesty

- [x] Competitive benchmark table in [[competitive-benchmarks]] filled with pinned hardware results
- [x] Pytxo Desktop 3D stills exported per [[deck-3d-stills]]
