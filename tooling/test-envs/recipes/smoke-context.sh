#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=../lib/resolve-pytxo.sh
source "$SCRIPT_DIR/../lib/resolve-pytxo.sh"

if [[ -z "${PYTXO_TEST_REPO:-}" ]]; then
  echo "Run scaffold.sh context first" >&2
  exit 1
fi
cd "$PYTXO_TEST_REPO"

PYTXO="$(resolve_pytxo)"
export PYTXO_BIN="$PYTXO"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
STUB="$SCRIPT_DIR/../fixtures/stub-agent/stub-agent.sh"
chmod +x "$STUB"

echo "=== dry-run ==="
"$PYTXO" run --config pytxo.toml --dry-run

echo "=== execute (stub-agent) ==="
"$PYTXO" run --config pytxo.toml --cmd "bash $STUB"

echo "=== status ==="
"$PYTXO" status

echo "OK: smoke-context complete"
