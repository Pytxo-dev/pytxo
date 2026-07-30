#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
cargo test -p pytxo-sanitize
cargo test -p pytxo-scheduler dependency_orders
if [ -f apps/desktop/package.json ]; then
  npm --prefix apps/desktop ci
  npm --prefix apps/desktop run build:native
else
  echo "desktop skipped (apps/desktop missing)"
fi
cargo test --workspace
cargo run -p pytxo-cli -- status --json
echo "Phase 2 smoke OK"
