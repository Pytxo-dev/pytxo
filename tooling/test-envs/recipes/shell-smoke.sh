#!/usr/bin/env bash
# Shell router smoke: dry-run + status via pytxo CLI (no TUI).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
# shellcheck source=../lib/resolve-pytxo.sh
source "$SCRIPT_DIR/../lib/resolve-pytxo.sh"

cd "${PYTXO_TEST_REPO:?PYTXO_TEST_REPO not set}"

echo "== shell smoke: trust =="
"$PYTXO_BIN" trust orbit

echo "== shell smoke: doctor =="
"$PYTXO_BIN" doctor

echo "== shell smoke: providers =="
provider_count=$("$PYTXO_BIN" providers --json | jq 'length')
test "$provider_count" -ge 5

echo "== shell smoke: models static =="
model_count=$("$PYTXO_BIN" models list --provider deepseek --json | jq 'length')
test "$model_count" -ge 1

echo "== shell smoke: dry-run =="
# Drain stdout so an early match cannot close the CLI's pipe mid-write.
"$PYTXO_BIN" run --dry-run --agents 2 | grep waves > /dev/null

echo "== shell smoke: run echo =="
"$PYTXO_BIN" run --agents 1 --cmd "echo pytxo-shell-smoke"

echo "== shell smoke: status =="
"$PYTXO_BIN" status --limit 3

echo "shell-smoke OK"
