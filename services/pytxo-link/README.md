# Pytxo Link (reference service)

Minimal HTTP service for Ultra-tier run reconciliation. Matches the monorepo client in [`HttpBillingReconciler`](../crates/pytxo-core/src/billing/link_reconciler.rs).

## Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Liveness |
| GET | `/v1/entitlements/status` | Current tier and agent limits |
| GET | `/v1/wallet/balance` | Ultra wallet balance (microcredits) |
| POST | `/v1/runs/start` | Run started envelope |
| POST | `/v1/runs/end` | Run ended + usage totals |

## Production defaults

| Variable | Default | Purpose |
|----------|---------|---------|
| `LINK_BIND` | `127.0.0.1:8787` | Listen address (local) |
| `PORT` | Railway injects | Hosted listen port |
| `LINK_API_KEY` | unset (loopback dev only) | Bearer token for API routes |
| `LINK_REQUIRE_AUTH` | `true` when an API key or JWKS validator is configured | Reject unsigned requests; mandatory for any non-loopback or `PORT` bind |
| `DATABASE_URL` | unset | Postgres for entitlements, org policies, and durable Paddle event IDs |
| `PADDLE_WEBHOOK_SECRET` | unset | Required to enable the Paddle webhook; missing secrets fail closed |
| `PADDLE_PRICE_PRO` | unset | Allowlisted Paddle price ID for Pro |
| `PADDLE_PRICE_MAX` | unset | Allowlisted Paddle price ID for Max |
| `PADDLE_PRICE_ULTRA` | unset | Allowlisted Paddle price ID for Ultra |

Hosted production URL: `https://link.pytxo.com/v1`

Pytxo Link refuses startup when a public bind is unauthenticated or when
authentication is required without an API key or complete Clerk JWKS
configuration. Bind values must use an IP socket address such as
`127.0.0.1:8787` or `0.0.0.0:8787`.

Client env:

| Variable | Purpose |
|----------|---------|
| `PYTXO_ULTRA_SESSION` | Bearer token for reconcile + entitlements |
| `PYTXO_ORG_ID` | Optional org id for policy ceiling fetch |

`pytxo.toml` Ultra defaults (`billing.mode = "ultra"`):

```toml
[billing]
mode = "ultra"
proxy_url = "https://link.pytxo.com"
# link_reconcile defaults to true for ultra (set link_reconcile = false to disable)
```

The CLI ships with `link-http` enabled via `pytxo-orchestrate` default features.

`POST /v1/webhooks/paddle` is unavailable unless `DATABASE_URL`, a non-empty
`PADDLE_WEBHOOK_SECRET`, and at least one allowlisted `PADDLE_PRICE_*` value are
configured. Accepted Paddle signatures must be no more than five minutes old.
`subscription.created` binds Paddle subscription and customer IDs to the
signed `custom_data.user_id`; later updates and cancellations must match that
persisted binding. Event claims, bindings, and entitlement changes share one
Postgres transaction. Paid tiers are derived only from allowlisted price IDs.

## Local dev

```bash
cd services/pytxo-link
cargo run
```

Optional auth:

```bash
export LINK_API_KEY=dev-secret
export PYTXO_ULTRA_SESSION=dev-secret
```

Point `pytxo.toml`:

```toml
[billing]
mode = "ultra"
link_reconcile = true
proxy_url = "http://127.0.0.1:8787/v1"
```

## Docker

```bash
docker compose up --build
```

## Production

Hosted at `https://link.pytxo.com`. Deploy steps: [`distribution/railway/README.md`](../../distribution/railway/README.md).

See [pytxo-link-service.md](../docs/08-reference/pytxo-link-service.md) for the service contract.
