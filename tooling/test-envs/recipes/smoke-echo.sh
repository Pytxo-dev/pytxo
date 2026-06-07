#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=../lib/resolve-pytxo.sh
source "$SCRIPT_DIR/../lib/resolve-pytxo.sh"

if [[ -z "${PYTXO_TEST_REPO:-}" ]]; then
  echo "Run scaffold.sh first (sets PYTXO_TEST_REPO)" >&2
  exit 1
fi
cd "$PYTXO_TEST_REPO"

PYTXO="$(resolve_pytxo)"
export PYTXO_BIN="$PYTXO"

echo "=== dry-run ==="
"$PYTXO" run --config pytxo.toml --dry-run

echo "=== execute (echo) ==="
"$PYTXO" trust orbit
"$PYTXO" run --config pytxo.toml --cmd "echo pytxo-agent"

echo "=== status ==="
"$PYTXO" status

echo "OK: smoke-echo complete"
