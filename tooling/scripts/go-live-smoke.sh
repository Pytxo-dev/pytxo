#!/usr/bin/env bash
# Go-live smoke: Link health, entitlements, run-ledger round-trip.
# Usage:
#   LINK_BASE=https://link.pytxo.com LINK_API_KEY=... ./tooling/scripts/go-live-smoke.sh
#   PROXY_BASE=https://proxy.pytxo.com ./tooling/scripts/go-live-smoke.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

LINK_BASE="${LINK_BASE:-${LINK_BASE_URL:-http://127.0.0.1:8787}}"
PROXY_BASE="${PROXY_BASE:-${PROXY_BASE_URL:-http://127.0.0.1:8790}}"
LINK_API_KEY="${LINK_API_KEY:-}"
LINK_ADMIN_KEY="${LINK_ADMIN_KEY:-}"
PYTXO_ULTRA_SESSION="${PYTXO_ULTRA_SESSION:-$LINK_API_KEY}"
RUN_ID="go-live-$(date +%s)"
DOMAIN_ID="${PYTXO_DOMAIN_ID:-go-live-smoke}"

auth_args=()
if [[ -n "$PYTXO_ULTRA_SESSION" ]]; then
  auth_args=(-H "Authorization: Bearer ${PYTXO_ULTRA_SESSION}")
fi

echo "==> Link health ($LINK_BASE)"
health="$(curl -fsS "${LINK_BASE%/}/health")"
[[ "$health" == "ok" || "$health" == *'"status":"ok"'* || "$health" == *'"status": "ok"'* ]] || { echo "unexpected health: $health"; exit 1; }

echo "==> Proxy health ($PROXY_BASE)"
proxy_health="$(curl -fsS "${PROXY_BASE%/}/health")"
[[ "$proxy_health" == "ok" || "$proxy_health" == *'"status":"ok"'* || "$proxy_health" == *'"status": "ok"'* ]] || { echo "unexpected proxy health: $proxy_health"; exit 1; }
echo "  $proxy_health"
if [[ "$proxy_health" != *deepseek* ]]; then
  echo "  WARNING: deepseek not in providers_configured - set DEEPSEEK_API_KEY on pytxo-proxy" >&2
fi

if [[ -n "$LINK_BASE" && -n "$PYTXO_ULTRA_SESSION" ]]; then
  echo "==> Entitlements status"
  curl -fsS "${auth_args[@]}" "${LINK_BASE%/}/v1/entitlements/status" | tee /dev/stderr
  echo

  echo "==> Wallet balance"
  if ! curl -fsS "${auth_args[@]}" "${LINK_BASE%/}/v1/wallet/balance" | tee /dev/stderr; then
    echo "  WARNING: wallet/balance failed - check Link DB migrations" >&2
  fi
  echo
else
  echo "==> Skipping entitlements/wallet (set LINK_BASE + LINK_API_KEY or PYTXO_ULTRA_SESSION)"
fi

echo "==> Run ledger round-trip (idempotent retry simulation)"
start_body="$(printf '{"domain_id":"%s","run_id":"%s"}' "$DOMAIN_ID" "$RUN_ID")"
end_body="$(printf '{"domain_id":"%s","run_id":"%s","usage":{"tokens_in_billed":12,"tokens_in_sent":8,"tokens_out":4,"saved_tokens":4,"cost_micro_usd":250}}' "$DOMAIN_ID" "$RUN_ID")"

for attempt in 1 2; do
  curl -fsS -X POST "${auth_args[@]}" \
    -H "Content-Type: application/json" \
    -d "$start_body" \
    "${LINK_BASE%/}/v1/runs/start" >/dev/null
  echo "  runs/start attempt $attempt OK"
done

for attempt in 1 2; do
  curl -fsS -X POST "${auth_args[@]}" \
    -H "Content-Type: application/json" \
    -d "$end_body" \
    "${LINK_BASE%/}/v1/runs/end" >/dev/null
  echo "  runs/end attempt $attempt OK"
done

if [[ -n "$LINK_ADMIN_KEY" && -n "${PYTXO_ORG_ID:-}" ]]; then
  echo "==> Enterprise org seats"
  curl -fsS -H "Authorization: Bearer ${LINK_ADMIN_KEY}" \
    "${LINK_BASE%/}/v1/orgs/${PYTXO_ORG_ID}/seats" | tee /dev/stderr
  echo

  echo "==> Enterprise org policy (GET)"
  curl -fsS -H "Authorization: Bearer ${LINK_ADMIN_KEY}" \
    "${LINK_BASE%/}/v1/orgs/${PYTXO_ORG_ID}/policy" | tee /dev/stderr
  echo

  echo "==> Enterprise org audit (recent)"
  curl -fsS -H "Authorization: Bearer ${LINK_ADMIN_KEY}" \
    "${LINK_BASE%/}/v1/orgs/${PYTXO_ORG_ID}/audit?limit=5" | tee /dev/stderr
  echo
elif [[ -n "$LINK_ADMIN_KEY" ]]; then
  echo "==> Skipping org seats/policy (set PYTXO_ORG_ID for Enterprise smoke)"
fi

echo "Go-live smoke OK (run_id=$RUN_ID)"
