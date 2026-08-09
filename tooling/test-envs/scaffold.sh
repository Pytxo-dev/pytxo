#!/usr/bin/env bash
set -euo pipefail

PROFILE="${1:-echo}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib/resolve-pytxo.sh
source "$SCRIPT_DIR/lib/resolve-pytxo.sh"

ROOT="$(_pytxo_repo_root)"
FIXTURE="$SCRIPT_DIR/fixtures/base-repo"
CONFIGS="$SCRIPT_DIR/fixtures/configs"
WORKDIR="${TMPDIR:-/tmp}/pytxo-testenv-$(uuidgen 2>/dev/null || date +%s)"

case "$PROFILE" in
  echo)        CONFIG="pytxo.echo.toml" ;;
  context)     CONFIG="pytxo.echo.toml" ;;
  antigravity) CONFIG="pytxo.antigravity.toml" ;;
  claude)      CONFIG="pytxo.claude.toml" ;;
  codex)       CONFIG="pytxo.codex.toml" ;;
  *)
    echo "Usage: $0 [echo|context|antigravity|claude|codex]" >&2
    exit 1
    ;;
esac

PYTXO="$(resolve_pytxo)"
export PYTXO_BIN="$PYTXO"
mkdir -p "$WORKDIR"
REPO="$WORKDIR/repo"
cp -R "$FIXTURE" "$REPO"
cd "$REPO"

git init -q
git config user.email "pytxo@test.local"
git config user.name "Pytxo Test"
git add .
git commit -q -m "init"

cp "$CONFIGS/$CONFIG" pytxo.toml
"$PYTXO" init
"$PYTXO" doctor
# Initialization intentionally creates the local .pytxo state and may update
# repository ignores. Commit that fixture setup so the smoke run exercises the
# real clean-checkout trust boundary instead of failing on its own bootstrap.
git add -A
git add -f .gitignore pytxo.toml
git commit -q -m "initialize pytxo test fixture"

export PYTXO_TEST_REPO="$REPO"
echo ""
echo "=== scaffold complete ==="
echo "Profile:   $PROFILE"
echo "Repo:      $REPO"
echo "PYTXO_BIN: $PYTXO"
echo "export PYTXO_TEST_REPO=\"$REPO\""
echo ""
echo "Next:"
echo "  cd \"\$PYTXO_TEST_REPO\""
echo "  pytxo run --config pytxo.toml --dry-run"
case "$PROFILE" in
  echo) echo "  $SCRIPT_DIR/recipes/smoke-echo.sh" ;;
  context) echo "  $SCRIPT_DIR/recipes/smoke-context.sh" ;;
  antigravity) echo "  $SCRIPT_DIR/recipes/live-antigravity.sh" ;;
  *) echo "  pytxo run --config pytxo.toml --cmd 'echo pytxo-agent'" ;;
esac
