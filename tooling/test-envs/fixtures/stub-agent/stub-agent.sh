#!/usr/bin/env bash
set -euo pipefail

echo "=== pytxo stub-agent ==="
echo "cwd=$(pwd)"
if [[ -n "${PYTXO_CONTEXT_DIR:-}" ]]; then
  echo "PYTXO_CONTEXT_DIR=$PYTXO_CONTEXT_DIR"
  echo "PYTXO_SIGNAL_CORE=${PYTXO_SIGNAL_CORE:-}"
  if [[ -f "$PYTXO_CONTEXT_DIR/manifest.json" ]]; then
    echo "manifest.json:"
    cat "$PYTXO_CONTEXT_DIR/manifest.json"
  else
    echo "manifest.json: (missing)"
    exit 1
  fi
else
  echo "PYTXO_CONTEXT_DIR=(unset)"
  exit 1
fi
echo "stub-agent OK"
