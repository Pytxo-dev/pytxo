#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=../lib/resolve-pytxo.sh
source "$SCRIPT_DIR/../lib/resolve-pytxo.sh"

if [[ -z "${PYTXO_TEST_REPO:-}" ]]; then
  echo "Run scaffold.sh antigravity first" >&2
  exit 1
fi
cd "$PYTXO_TEST_REPO"

if ! command -v agy >/dev/null 2>&1; then
  echo "agy not found on PATH. Install Antigravity CLI first." >&2
  exit 1
fi

PYTXO="$(resolve_pytxo)"
export PYTXO_BIN="$PYTXO"

echo "=== pytxo ==="
echo "PYTXO_BIN=$PYTXO"
"$PYTXO" --version
agy --version 2>&1 || agy --help 2>&1 | head -1

echo "=== dry-run ==="
"$PYTXO" run --config pytxo.toml --dry-run

echo "=== execute (agy --help smoke) ==="
"$PYTXO" run --config pytxo.toml --cmd "agy --help"

echo "=== status ==="
"$PYTXO" status

echo ""
echo "OK: live-antigravity smoke complete"
echo "Next: replace --cmd with your non-interactive agy invocation"
