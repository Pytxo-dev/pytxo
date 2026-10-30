#!/usr/bin/env bash
# Check the Pytxo GitHub Action on github.com before anything tells people to
# install it. It creates a PUBLIC throwaway repository under your account and a
# fork of it under <fork-owner>, then:
#   1. opens claude/rename and codex/discount, which pass `npm test` alone and
#      fail together, then agent/readme, which only touches the README and
#      must pass: the check proves the other two fail without it;
#   2. pushes again to codex/discount: its comment must be updated, not doubled;
#   3. opens a pull request from the fork: that run gets a read-only token, so
#      the verdict must go to the job summary and no comment may appear;
#   4. runs `npx -y pytxo@<version> demo` on Ubuntu, macOS and Windows.
# It prints what GitHub shows, then the commands that delete both repositories.
#
# Usage:
#   ./tooling/scripts/action-e2e.sh <action-ref> <fork-owner>
#   ./tooling/scripts/action-e2e.sh v2.0.0 my-org
# <fork-owner> is an organization you can fork into (not your own account).
# PYTXO_VERSION overrides the npm version the demo runs (default: the ref
# without its "v"). Needs git and gh, logged in with the repo and workflow
# scopes (gh auth refresh -s workflow).
set -euo pipefail

ref="${1:?usage: action-e2e.sh <action-ref> <fork-owner>}"
fork_owner="${2:?usage: action-e2e.sh <action-ref> <fork-owner>}"
version="${PYTXO_VERSION:-${ref#v}}"
owner="$(gh api user --jq .login)"
[ "$fork_owner" != "$owner" ] || { echo "fork into an organization, not your own account ($owner)" >&2; exit 1; }
repo="pytxo-action-e2e-$(date +%Y%m%d-%H%M%S)"
full="$owner/$repo"
marker='<!-- pytxo-check -->'
fails=0

note() { printf '\n== %s\n' "$*"; }
pass() { echo "PASS  $*"; }
fail() { echo "FAIL  $*"; fails=$((fails + 1)); }

# The finished run of workflow <file> for commit <sha>; prints its id.
run_for() {
  local id=""
  for _ in $(seq 1 90); do
    id="$(gh run list -R "$full" --workflow "$1" --commit "$2" --json databaseId --jq '.[0].databaseId // empty')"
    [ -n "$id" ] && break
    sleep 10
  done
  [ -n "$id" ] || { echo "no $1 run for $2 after 15 minutes" >&2; return 1; }
  gh run watch -R "$full" "$id" > /dev/null 2>&1 || true
  echo "$id"
}

conclusion() { gh run view -R "$full" "$1" --json conclusion --jq .conclusion; }

# A field of each Pytxo sticky comment on pull request <n>, one per line.
sticky() { gh api "repos/$full/issues/$1/comments" --paginate --jq ".[] | select(.body | startswith(\"$marker\")) | .$2"; }
comment_count() { sticky "$1" id | grep -c . || true; }

open_pr() { # open_pr <head> <title>: prints the pull request number
  local url
  url="$(gh pr create -R "$full" --head "$1" --base main --title "$2" --body "Throwaway check of the Pytxo Action. Safe to delete.")"
  echo "${url##*/}"
}

work="$(mktemp -d)"
cd "$work"
note "Creating $full (public) in $work"
git init -q -b main
mkdir -p test .github/workflows
cat > package.json <<'EOF'
{
  "name": "pytxo-action-e2e",
  "version": "1.0.0",
  "private": true,
  "scripts": { "test": "node --test" }
}
EOF
cat > price.js <<'EOF'
function price(item) {
  return item.cents / 100;
}

module.exports = { price };
EOF
cat > cart.js <<'EOF'
const { price } = require("./price");

function total(items) {
  return items.reduce((sum, item) => sum + price(item), 0);
}

module.exports = { total };
EOF
cat > test/cart.test.js <<'EOF'
const test = require("node:test");
const assert = require("node:assert");
const { total } = require("../cart");

test("total adds up the prices", () => {
  assert.strictEqual(total([{ cents: 150 }, { cents: 250 }]), 4);
});
EOF
printf '# Pytxo Action check\n\nA throwaway repository for testing the Pytxo GitHub Action. Safe to delete.\n' > README.md
cat > .github/workflows/pytxo.yml <<'EOF'
name: Pytxo
on: pull_request
permissions:
  contents: read
  pull-requests: write
concurrency:
  group: pytxo-${{ github.event.pull_request.number }}
  cancel-in-progress: true
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
EOF
printf '      - uses: Pytxo-dev/pytxo/actions/land@%s\n' "$ref" >> .github/workflows/pytxo.yml
cat > .github/workflows/demo.yml <<'EOF'
name: Demo
on:
  push:
    branches: [main]
permissions:
  contents: read
jobs:
  demo:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - name: npx pytxo demo
        shell: bash
        run: |
          start=$(date +%s)
          npx -y "pytxo@$PYTXO_VERSION" demo 2>&1 | tee demo.txt
          echo "took $(( $(date +%s) - start )) s"
          grep -q BLOCKED demo.txt
EOF
printf '        env:\n          PYTXO_VERSION: "%s"\n' "$version" >> .github/workflows/demo.yml
git add -A
git commit -qm "shop: prices and totals"
base="$(git rev-parse HEAD)"
gh repo create "$full" --public --source . --push > /dev/null

note "Agent branches"
git switch -q -c claude/rename "$base"
cat > price.js <<'EOF'
function cost(item) {
  return item.cents / 100;
}

module.exports = { cost };
EOF
cat > cart.js <<'EOF'
const { cost } = require("./price");

function total(items) {
  return items.reduce((sum, item) => sum + cost(item), 0);
}

module.exports = { total };
EOF
git commit -qam "rename price to cost"
git push -q -u origin claude/rename

git switch -q -c codex/discount "$base"
cat > discount.js <<'EOF'
const { price } = require("./price");

function discounted(item, percent) {
  return price(item) * (1 - percent / 100);
}

module.exports = { discounted };
EOF
cat > test/discount.test.js <<'EOF'
const test = require("node:test");
const assert = require("node:assert");
const { discounted } = require("../discount");

test("discounted takes the percentage off", () => {
  assert.strictEqual(discounted({ cents: 200 }, 25), 1.5);
});
EOF
git add -A
git commit -qm "add discounts"
git push -q -u origin codex/discount

git switch -q -c agent/readme "$base"
printf '\nRun `npm test` to check the shop code.\n' >> README.md
git commit -qam "docs: how to run the tests"
git push -q -u origin agent/readme

note "1. Two pull requests that break together, then an innocent one"
pr_rename="$(open_pr claude/rename "Rename price to cost")"
run="$(run_for pytxo.yml "$(git rev-parse claude/rename)")"
[ "$(conclusion "$run")" = success ] && pass "#$pr_rename alone: nothing to combine, the job passes" \
  || fail "#$pr_rename alone: expected success, got $(conclusion "$run")"
[ "$(comment_count "$pr_rename")" = 0 ] && pass "#$pr_rename alone: no comment" || fail "#$pr_rename alone: a comment appeared"

pr_discount="$(open_pr codex/discount "Add discounts")"
run="$(run_for pytxo.yml "$(git rev-parse codex/discount)")"
[ "$(conclusion "$run")" = failure ] && pass "#$pr_discount with #$pr_rename: the job fails" \
  || fail "#$pr_discount with #$pr_rename: expected failure, got $(conclusion "$run")"
[ "$(comment_count "$pr_discount")" = 1 ] && pass "#$pr_discount: one sticky comment" \
  || fail "#$pr_discount: expected one sticky comment, found $(comment_count "$pr_discount")"
first_update="$(sticky "$pr_discount" updated_at | head -1)"
echo "--- #$pr_discount comment (first 25 lines)"
sticky "$pr_discount" body | head -25
echo "---"

pr_readme="$(open_pr agent/readme "Explain how to run the tests")"
run="$(run_for pytxo.yml "$(git rev-parse agent/readme)")"
[ "$(conclusion "$run")" = success ] && pass "#$pr_readme (README only) with both: the other two are to blame, the job passes" \
  || fail "#$pr_readme (README only) with both: expected success, got $(conclusion "$run")"
sticky "$pr_readme" body | grep -q 'passes without them' && pass "#$pr_readme: the comment says it passes without the pair" \
  || fail "#$pr_readme: the comment does not clear it"

note "2. A new push updates the comment"
git switch -q codex/discount
printf '\n// Percent is a number from 0 to 100.\n' >> discount.js
git commit -qam "discount: document the percent"
git push -q
run="$(run_for pytxo.yml "$(git rev-parse HEAD)")"
count="$(comment_count "$pr_discount")"
[ "$count" = 1 ] && pass "#$pr_discount after a push: still one comment" || fail "#$pr_discount after a push: $count comments"
[ "$(sticky "$pr_discount" updated_at | head -1)" != "$first_update" ] && pass "#$pr_discount after a push: the comment was updated" \
  || fail "#$pr_discount after a push: the comment was not updated"

note "3. A pull request from a fork"
gh repo fork "$full" --org "$fork_owner" --clone=false > /dev/null
git remote add fork "https://github.com/$fork_owner/$repo.git"
git switch -q -c claude/fork-note "$base"
printf 'Prices are stored in cents.\n' > NOTES.md
git add -A
git commit -qm "notes: prices are in cents"
for _ in 1 2 3 4 5 6; do git push -q fork claude/fork-note && break; sleep 10; done
pr_fork="$(open_pr "$fork_owner:claude/fork-note" "Notes from a fork")"
run="$(run_for pytxo.yml "$(git rev-parse HEAD)")"
[ "$(comment_count "$pr_fork")" = 0 ] && pass "#$pr_fork from a fork: no comment (read-only token)" \
  || fail "#$pr_fork from a fork: a comment appeared"
echo "INFO  #$pr_fork's verdict should be in its job summary: $(gh run view -R "$full" "$run" --json url --jq .url)"

note "4. npx pytxo@$version demo on three systems"
run="$(run_for demo.yml "$base")"
gh run view -R "$full" "$run" --json jobs --jq '.jobs[] | "      \(.name): \(.conclusion)"'
gh run view -R "$full" "$run" --log 2>/dev/null | grep -E 'took [0-9]+ s' | grep -v echo \
  | awk -F'\t' '{print "      " $1 ": " $NF}' | sed -E 's/ [0-9T:.-]+Z / /' || true
[ "$(conclusion "$run")" = success ] && pass "the demo prints BLOCKED on all three" || fail "the demo failed on at least one system"

note "Done: $fails failed"
echo "Open: https://github.com/$full/pulls"
echo "Held pull requests (Copilot) are not covered here: add actions: read to the workflow and assign an issue to Copilot to see one."
echo "Delete both repositories afterwards (needs: gh auth refresh -s delete_repo):"
echo "  gh repo delete $full --yes"
echo "  gh repo delete $fork_owner/$repo --yes"
[ "$fails" = 0 ]
