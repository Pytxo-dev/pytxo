---
title: ADR-0027 Service observability contract
slug: adr-0027-service-observability-contract
status: accepted
tags: [adr, observability, services]
audience: [human, agent]
layer: architecture
created: 2026-06-24
updated: 2026-06-24
related: [[ADR-0020-cloud-sandbox-runtime]], [[ADR-0021-billing-source-of-truth]], [[ADR-0024-managed-transport-endpoint-split]]
---

# ADR-0027: Service observability contract

## Status

Accepted (2026-06-24)

## Context

Pytxo ships three reference HTTP services (`pytxo-link`, `pytxo-cloud-sandbox`, `pytxo-proxy`) on Railway. Operators need a consistent liveness shape for Railway health checks, CI smoke, and `pytxo doctor`. Logs must be machine-parseable in production without bespoke per-service formatters.

## Decision

### `/health` JSON contract

All three services expose `GET /health` returning **HTTP 200** and JSON:

```json
{
  "status": "ok",
  "uptime_secs": 123,
  "service": "pytxo-link"
}
```

| Field | Type | Meaning |
|-------|------|---------|
| `status` | string | `"ok"` when process is live |
| `uptime_secs` | integer | Seconds since process start |
| `service` | string | Crate/binary name |

Legacy plain-text body `ok` remains accepted by CLI health probes during transition.

### Structured logging

When `RUST_LOG=json`, services emit **JSON lines** via `tracing-subscriber` (severity filter from `PYTXO_LOG_LEVEL`, default `info`). Otherwise `RUST_LOG` follows standard `tracing` env-filter semantics.

Optional sidecar hooks (no in-process exporter yet):

- `OTEL_EXPORTER_OTLP_ENDPOINT` — logged at startup when set
- `SENTRY_DSN` — logged at startup when set

### Link hardening (pytxo-link only)

- **Max request body:** 1 MiB (`LINK_MAX_BODY_BYTES` override)
- **Rate limit:** 120 requests / minute / client IP (`LINK_RATE_LIMIT_PER_MIN` override); excess → HTTP 429

### CI smoke

`deploy-services.yml` starts each binary locally and `curl`s `/health`, asserting JSON `status == "ok"` and numeric `uptime_secs`.

## Consequences

**Positive**

- Railway `healthcheckPath = "/health"` works with JSON parsers.
- `pytxo doctor` and `HttpBillingReconciler::ping` accept JSON via `pytxo_core::service_health_ok`.
- Operators can ship logs to Loki/Datadog without custom adapters when `RUST_LOG=json`.

**Negative**

- Clients that compared body to exact `"ok"` must update (mitigated by `service_health_ok` helper).
- Per-IP rate limits are in-memory only (single replica); horizontal scale needs edge rate limiting later.

## Alternatives considered

- Keep plain-text `/health` — rejected (no uptime, poor structured observability).
- OpenTelemetry SDK in-process — deferred; env hook + sidecar collector is sufficient for reference services.

## Links

- [[link-db-backup-runbook]] — Postgres backup for Link entitlements ledger
- `services/pytxo-link/README.md` — env reference
