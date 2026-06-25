#!/usr/bin/env bash
# Phase 57 — overlay vs worktree disk + cold-start on tiny-monorepo.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
FIXTURE="$ROOT/tests/fixtures/tiny-monorepo"
WORKDIR="$(mktemp -d)"
PYTXO="${PYTXO_BIN:-$ROOT/target/debug/pytxo}"

cleanup() { rm -rf "$WORKDIR"; }
trap cleanup EXIT

bytes_human() {
  local n="$1"
  if command -v numfmt >/dev/null 2>&1; then
    numfmt --to=iec-i --suffix=B "$n" 2>/dev/null || echo "${n}B"
  else
    echo "${n} bytes"
  fi
}

dir_bytes() {
  du -sb "$1" 2>/dev/null | awk '{print $1}'
}

setup_repo() {
  cp -r "$FIXTURE" "$WORKDIR/repo"
  cd "$WORKDIR/repo"
  git init -q
  git config user.email "pytxo@bench.local"
  git config user.name "Pytxo Bench"
  git add .
  git commit -q -m "init"
}

bench_worktree() {
  local wt_base="$WORKDIR/repo/.pytxo/worktrees"
  mkdir -p "$wt_base"
  local start end elapsed disk
  start=$(date +%s%N)
  for i in 0 1 2; do
    git worktree add -q "$wt_base/agent-$i" HEAD
  done
  end=$(date +%s%N)
  elapsed=$(( (end - start) / 1000000 ))
  disk=$(dir_bytes "$wt_base")
  echo "worktree_cold_start_ms=$elapsed"
  echo "worktree_disk_bytes=$disk"
  echo "worktree_disk_human=$(bytes_human "$disk")"
}

bench_overlay() {
  cat > pytxo.toml <<'EOF'
max_agents = 3
worktree_dir = ".pytxo/worktrees"
data_dir = ".pytxo/data"
isolation = "overlay"

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

  local start end elapsed disk
  start=$(date +%s%N)
  "$PYTXO" init 2>/dev/null || true
  "$PYTXO" trust orbit 2>/dev/null || true
  "$PYTXO" run --config pytxo.toml --dry-run >/dev/null
  end=$(date +%s%N)
  elapsed=$(( (end - start) / 1000000 ))

  local wt_base="$WORKDIR/repo/.pytxo/worktrees"
  if [[ -d "$wt_base" ]]; then
    disk=$(dir_bytes "$wt_base")
  else
    disk=0
  fi
  echo "overlay_cold_start_ms=$elapsed"
  echo "overlay_disk_bytes=$disk"
  echo "overlay_disk_human=$(bytes_human "$disk")"
}

echo "=== Pytxo overlay vs worktree benchmark (tiny-monorepo) ==="
echo "fixture: $FIXTURE"
echo "pytxo:   $PYTXO"
if [[ ! -x "$PYTXO" ]]; then
  echo "ERROR: build pytxo first: cargo build -p pytxo-cli" >&2
  exit 1
fi

setup_repo
echo ""
echo "--- worktree isolation (3 git worktrees) ---"
bench_worktree

cd "$WORKDIR/repo"
rm -rf .pytxo/worktrees
echo ""
echo "--- overlay isolation (dry-run planner path) ---"
bench_overlay

echo ""
echo "=== summary (paste into competitive-benchmarks.md pinned table) ==="
printf "| backend | cold_start_ms | upper_disk |\n"
printf "|---------|---------------|------------|\n"
