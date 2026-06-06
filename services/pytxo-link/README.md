# Pytxo Link (reference service)

Minimal HTTP service for Ultra-tier run reconciliation. Matches the monorepo client in [`HttpBillingReconciler`](../crates/pytxo-core/src/billing/link_reconciler.rs).

## Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Liveness |
| POST | `/v1/runs/start` | Run started envelope |
| POST | `/v1/runs/end` | Run ended + usage totals |

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

This reference implementation lives in the monorepo until the private org repo is published. See [pytxo-link-service.md](../docs/08-reference/pytxo-link-service.md).
