#!/usr/bin/env bash
# Runs inside the take window (see take.py). Each command is typed on screen by
# this script and then run exactly as typed; nothing is replayed or edited.
# Usage: driver.sh <take-dir> <script-dir>
TAKE=$1
HERE=$2
LOG="$TAKE/timeline.jsonl"
LAND='pytxo land claude:claude/rename-helper codex:codex/negated-flags --check "npm install && npm test" --per-input'

python -I "$HERE/consetup.py" "$TAKE/hwnd.txt" 24 122 32 || { echo "console setup failed"; read -r; exit 1; }

export npm_config_prefix="$TAKE/npm" npm_config_update_notifier=false
export PATH="$(cygpath -u "$TAKE/npm"):$PATH"
export PYTXO_HOME="$TAKE/pytxo-home" GIT_PAGER=cat PAGER=cat
cd "$TAKE/mri" || exit 1

now() { date +%s.%3N; }
note() { printf '{"t":%s,"note":"%s"}\n' "$(now)" "$1" >> "$LOG"; }
note "preflight node=$(command -v node) git=$(command -v git) pytxo_before=$(command -v pytxo)"

# A timed read on the console; MSYS returns at once on an idle pipe.
pause() { read -rsn1 -t "$1" < /dev/tty || :; }
prompt() {
  printf '\e[33m~/mri\e[0m \e[36m(%s)\e[0m\n\e[90m$\e[0m ' "$(git branch --show-current)"
}
run() {
  local label=$1 cmd=$2 t0 t1 t2 code i
  prompt
  t0=$(now)
  pause 0.8
  for ((i = 0; i < ${#cmd}; i++)); do
    printf '%s' "${cmd:i:1}"
    pause "0.0$((RANDOM % 25 + 10))"
  done
  pause 0.5
  printf '\n'
  t1=$(now)
  eval "$cmd"
  code=$?
  t2=$(now)
  printf '{"label":"%s","typed":%s,"enter":%s,"end":%s,"exit":%s}\n' "$label" "$t0" "$t1" "$t2" "$code" >> "$LOG"
  pause 2.5
  printf '\n'
}
fresh() { clear; note "clear $1"; }

clear
while [ ! -e "$TAKE/go" ]; do pause 0.2; done
note "go"
pause 1.5

run install 'npm i -g pytxo'
run version 'pytxo --version'
note "pytxo_after=$(command -v pytxo)"
run branches 'git branch -v'
fresh land1
run land-blocked "$LAND"
fresh fix
run fix-rebase 'git switch -q codex/negated-flags && git rebase -q claude/rename-helper'
run fix-edit "sed -i 's/toVal(/assign(/' src/index.js && git diff"
run fix-commit 'git commit -qam "Call assign() after the toVal rename" && git switch -q master'
fresh land2
run land-ready "$LAND"
fresh land3
run land-apply "$LAND --apply"
run graph 'git log --oneline --graph -6'
run receipts 'pytxo receipts'
prompt
pause 3
note "done"
: > "$TAKE/done"
while [ ! -e "$TAKE/close" ]; do pause 0.5; done
