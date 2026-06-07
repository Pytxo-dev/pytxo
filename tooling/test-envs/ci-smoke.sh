#!/usr/bin/env bash
# CI entry: scaffold echo profile + smoke-echo (no API keys).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
export PYTXO_BIN="${PYTXO_BIN:-$ROOT/target/release/pytxo}"

chmod +x "$SCRIPT_DIR/scaffold.sh" "$SCRIPT_DIR/recipes/smoke-echo.sh"

eval "$("$SCRIPT_DIR/scaffold.sh" echo | grep '^export PYTXO_TEST_REPO=')"
"$SCRIPT_DIR/recipes/smoke-echo.sh"
