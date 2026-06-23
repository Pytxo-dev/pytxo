# Pytxo Link (reference service)

Minimal HTTP service for Ultra-tier run reconciliation. Matches the monorepo client in [`HttpBillingReconciler`](../crates/pytxo-core/src/billing/link_reconciler.rs).

## Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Liveness |
| POST | `/v1/runs/start` | Run started envelope |
| POST | `/v1/runs/end` | Run ended + usage totals |

## Production defaults

| Variable | Default | Purpose |
|----------|---------|---------|
| `LINK_BIND` | `127.0.0.1:8787` | Listen address (local) |
| `PORT` | Railway injects | Hosted listen port |
| `LINK_API_KEY` | unset (dev open) | Bearer token for API routes |
| `LINK_REQUIRE_AUTH` | `true` when `LINK_API_KEY` set | Reject unsigned requests in prod |
| `DATABASE_URL` | unset | Postgres for entitlements + org policies |
| `PADDLE_WEBHOOK_SECRET` | unset | Verify Paddle billing webhooks |

Hosted production URL: `https://link.pytxo.com/v1`

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
