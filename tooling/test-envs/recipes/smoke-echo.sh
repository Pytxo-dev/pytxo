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
# Trust setup writes the local policy metadata used by the real Apply guard.
# Commit it before dispatch so the smoke test reaches execution instead of
# rejecting its own fixture bootstrap as dirty.
git add -A
git add -f .gitignore pytxo.toml
git commit -q -m "record pytxo trust fixture"
"$PYTXO" run --config pytxo.toml --cmd "echo pytxo-agent"

echo "=== status ==="
"$PYTXO" status

echo "OK: smoke-echo complete"
