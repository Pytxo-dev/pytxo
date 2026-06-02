#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo test -p pytxo-sanitize
cargo test -p pytxo-scheduler dependency_orders
cargo test --workspace --exclude pytxo-desktop
cargo run -p pytxo-cli -- status --json
if [ -f apps/desktop/package.json ]; then
  cargo build -p pytxo-desktop
else
  echo "desktop skipped (pytxo-desktop repo)"
fi
echo "Phase 2 smoke OK"
