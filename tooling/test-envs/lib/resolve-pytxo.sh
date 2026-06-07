#!/usr/bin/env bash
# Shared pytxo binary resolution for test-env shell scripts.

_pytxo_repo_root() {
  local here
  here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
  cd "$here/../.." && pwd
}

resolve_pytxo() {
  if [[ -n "${PYTXO_BIN:-}" && -x "$PYTXO_BIN" ]]; then
    echo "$PYTXO_BIN"
    return
  fi
  if command -v pytxo >/dev/null 2>&1; then
    command -v pytxo
    return
  fi
  local root
  root="$(_pytxo_repo_root)"
  if [[ -x "$root/target/release/pytxo" ]]; then
    echo "$root/target/release/pytxo"
    return
  fi
  if [[ -x "$root/target/debug/pytxo" ]]; then
    echo "$root/target/debug/pytxo"
    return
  fi
  echo "pytxo not found. Run: npm i -g pytxo or cargo build -p pytxo-cli --release" >&2
  exit 1
}
