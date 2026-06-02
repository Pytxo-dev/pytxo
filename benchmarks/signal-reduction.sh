#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

fixture="tests/fixtures/tiny-monorepo/src/a.ts"
if [[ ! -f "$fixture" ]]; then
  echo "fixture not found: $fixture" >&2
  exit 1
fi

echo "== Signal Core scaffold report =="
json=$(cargo run -q -p pytxo-signal --example scaffold_report -- "$fixture" low)
echo "$json"

pct=$(echo "$json" | python -c "import json,sys; print(json.load(sys.stdin)['stats']['token_reduction_pct'])")
echo "token_reduction_pct: $pct"

python -c "import sys; sys.exit(0 if float('$pct') > 0 else 1)"

echo "Signal reduction benchmark OK"
