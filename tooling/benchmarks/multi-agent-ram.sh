#!/usr/bin/env bash
# Phase 57 — sample RSS while a three-agent pytxo run is active.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
FIXTURE="$ROOT/tests/fixtures/tiny-monorepo"
WORKDIR="$(mktemp -d)"
PYTXO="${PYTXO_BIN:-$ROOT/target/debug/pytxo}"

cleanup() {
  if [[ -n "${PYTXO_PID:-}" ]]; then
    kill "$PYTXO_PID" 2>/dev/null || true
    wait "$PYTXO_PID" 2>/dev/null || true
  fi
  rm -rf "$WORKDIR"
}
trap cleanup EXIT

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

"$PYTXO" init
"$PYTXO" trust orbit

# Long-running child so we can sample parent RSS during the swarm.
"$PYTXO" run --config pytxo.toml --cmd "sleep 30" &
PYTXO_PID=$!
sleep 2

peak_kb=0
samples=0
while kill -0 "$PYTXO_PID" 2>/dev/null && [[ $samples -lt 10 ]]; do
  if [[ "$(uname -s)" == "Darwin" ]]; then
    rss_kb=$(ps -o rss= -p "$PYTXO_PID" 2>/dev/null | tr -d ' ' || echo 0)
  else
    rss_kb=$(awk '/VmRSS/{print $2}' "/proc/$PYTXO_PID/status" 2>/dev/null || echo 0)
  fi
  if [[ "$rss_kb" =~ ^[0-9]+$ ]] && [[ "$rss_kb" -gt "$peak_kb" ]]; then
    peak_kb=$rss_kb
  fi
  samples=$((samples + 1))
  sleep 1
done

peak_mb=$((peak_kb / 1024))
echo "=== multi-agent RAM benchmark ==="
echo "samples=$samples"
echo "peak_rss_kb=$peak_kb"
echo "peak_rss_mb=$peak_mb"
echo "OK: RAM sample complete"
