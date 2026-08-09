#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
FIXTURE="$ROOT/tests/fixtures/tiny-monorepo"
WORKDIR="$(mktemp -d)"
PYTXO="${PYTXO_BIN:-$ROOT/target/debug/pytxo}"

cp -r "$FIXTURE" "$WORKDIR/repo"
cd "$WORKDIR/repo"
git init -q
git config user.email "pytxo@bench.local"
git config user.name "Pytxo Bench"
git add .
git commit -q -m "init"

cat > pytxo.toml <<'EOF'
max_agents = 3
worktree_dir = ".pytxo/worktrees"
data_dir = ".pytxo/data"

[[task]]
id = "task-a"
agent = "builder"
paths = ["src/a.ts"]

[[task]]
id = "task-b"
agent = "builder"
paths = ["package.json"]

[[task]]
id = "task-c"
agent = "builder"
paths = ["package.json"]
EOF

echo "=== dry-run plan ==="
"$PYTXO" run --config pytxo.toml --dry-run

echo "=== execute ==="
"$PYTXO" init
"$PYTXO" trust orbit
git add -A
git add -f .gitignore pytxo.toml
git diff --cached --quiet || git commit -q -m "record pytxo benchmark trust fixture"
"$PYTXO" run --config pytxo.toml --cmd "echo pytxo-agent"

echo "=== status ==="
"$PYTXO" status

echo "OK: benchmark complete"
