---
title: Pytxo 2.1, the check that says why
status: Approved for build
date: 2026-10-29
tags: [land, cli, action, hooks]
---

# Pytxo 2.1: the check that says why

## Outcome

When agent branches pass alone and fail together, Pytxo names the change
that broke them and the branch that has to change, and hands that branch's
agent a ready-to-paste fix prompt. With three or more branches it proves which
pair fails, so the other branches and their pull requests are not blocked. In
CI, Copilot's pull requests join the combined check once a person has let
their run start. Pytxo still calls no model and sends nothing anywhere.

Example, the demo's two branches:

```
BLOCKED  claude/rename-price and codex/add-discount pass npm test alone but fail together.
why      claude/rename-price renamed price to unitPrice; codex/add-discount still uses price (src/discount.js:1).
next     rebase codex/add-discount onto claude/rename-price and use unitPrice, then run npx pytxo again.
```

## 1. Cause: a name one branch removed and another still uses

New module `crates/pytxo-land/src/cause.rs`. It runs when the combined check
fails for real (it failed again on the rerun, no timeout) and every merged
input passed alone, which is the "together" verdict. It also ranks the pair
for section 2.

Terms: base `B`; input `i` at commit `C_i` with merge base
`M_i = merge-base(B, C_i)`; combined commit `R`.

1. **Files.** The input's changed files (`ResolvedInput.files`) that
   pytxo-signal parses (`language.rs`). At most 200 files per input; skip
   files over 1 MB.
2. **Gone names.** A name defined (`symbol_spans`, last segment of a qualified
   name, at least 3 characters) in `M_i`'s version of a changed file and not in
   `C_i`'s version. It only counts when `git grep -q -w -F -e <name> C_i` finds
   nothing: the input removed every mention, so the input itself no longer
   uses the name. This keeps common words such as `get` out.
3. **Renamed or removed.** In `git diff -U0 M_i C_i -- <file>`, a removed line
   that contains the name as a word, and an added line in the same file that
   equals it (whitespace trimmed) once that word is replaced by another word
   `Y` that `C_i`'s version of the file defines: "renamed `X` to `Y`".
   Otherwise "removed `X`".
4. **Stale uses.** For every other merged input `j`: the added lines of
   `git diff -U0 M_j C_j` that contain the name as a word, with their path and
   line number in `C_j` (what the user sees on that branch). It is not a cause
   if `R` still defines the name: `git grep -l -w -F -e <name> R`, then parse
   those files at `R`.
5. **Confirmed.** The failing output tails of the combined runs contain the
   name as a word, or a `path:line` equal to one of the stale uses.
6. **Pick one.** Confirmed first, then the most stale uses, then input order.

Bounds: at most 50 gone names per input and 100 `git grep` calls in all; a
cap reached means no cause, never a guess. Git calls are stoppable like the
rest of `land()`. Windows diffs may carry `\r`; strip it.

`LandOutcome` gets `cause: Option<Cause>`:

```rust
pub struct Cause { pub by: usize, pub user: usize, pub name: String,
                   pub renamed_to: Option<String>, pub uses: Vec<Use>, pub confirmed: bool }
pub struct Use { pub path: String, pub line: u32 }
```

The cause is a reading of the runs, not a record of them, so it stays out of
the receipt and its digest.

Why definitions and not every word: tree-sitter is already in the lean build,
a renamed or removed definition is what breaks another branch's new code in
the common case, and it needs a handful of `git grep` calls instead of a scan
of every word in the diff. Renamed config keys, CSS classes and changed
signatures are out of scope.

## 2. The culprit pair (three or more merged inputs)

After a real "together" failure with three or more merged inputs, when a cause
(confirmed or not) names a pair `by` and `user`:

1. Run the failing command on `B` + `by` + `user` in the `-solo` scratch
   worktree (receipt scope `pair`). If it passes, the guess was wrong: the
   verdict stays as in 2.0, and the note says the pair passed.
2. If it fails, run it on `B` + every other merged input (scope `rest`). If
   that passes, only `by` and `user` are implicated. If it fails, everyone
   stays implicated and the verdict still names the proven pair.
3. Only the first failing command, at most two extra runs, never after a
   timeout, and never when the hook deadline would pass before them.
4. No cause, no extra runs.

The Action needs no change for this: it already passes a pull request whose
index is not implicated, with the note that it is not the cause.

Words when the pair is proven and the rest passes:

```
BLOCKED  claude/rename-price and codex/add-discount fail together; agent/readme passes without them.
```

With two or more others: "the other 3 branches pass without them."

## 3. Words, prompt and surfaces

The three lines stay three lines.

- **Confirmed, renamed:** as in the outcome.
- **Confirmed, removed:** `why  <by> removed <X>; <user> still uses it (<path>:<line>).`
  `next  rebase <user> onto <by> and stop using <X>, or keep <X> on <by>, then run npx pytxo again.`
- **Not confirmed:** the why line stays the error line and a note follows:
  `note  <user> uses <X>, which <by> renamed to <Y> (<path>:<line>).`
- **Markdown:** the same sentences with names in code spans, then
  `<details><summary>Fix prompt for <user></summary>` holding the prompt in a
  fenced `text` block.
- **JSON:** `verdict.cause` (`by` and `user` as branch names, `name`,
  `renamed_to`, `uses`, `confirmed`) and `verdict.fix_prompt`.
- **Hook:** the system message stays the three lines. With `--block`, the
  reason is the fix prompt, addressed to the agent whose worktree the hook runs
  in: "Your branch `<user>` …" in the user's worktree; in the `by` branch's
  worktree it says the fix belongs on `<user>` and that keeping `<X>` as an
  alias is the other way out; elsewhere the plain prompt.

The fix prompt, renamed variant:

```
Pytxo merged <by> and <user> and ran <command>: each passes alone, together they fail.
<by> renamed <X> to <Y>. <user> still uses <X> at <path>:<line>, <path>:<line>.
Fix it on <user>: rebase it onto <by>, replace <X> with <Y>, and run <command> until it passes. Leave <by>'s changes as they are.
Failing output:
<the last 15 lines of the combined run's output tail>
```

Removed variant: "<by> removed <X>. … Fix it on <user>: rebase it onto <by>
and stop using <X>. If <user> needs <X>, ask for it back on <by> instead."
At most three uses are listed. The output tail is the sanitized one the
receipt already keeps.

## 4. A better error line

When there is no confirmed cause, the why line is still the first useful
error line. `why_lines` now prefers, in order: an error or exception name
(`SomethingError:`, `Error:`, `error[`, `panicked at`, `Traceback`,
`assert`), then a `FAIL <file>` line, then any other line with error or
fail. It skips test-runner count lines such as `ℹ fail 1`, `# fail 1`,
`Tests: 1 failed` and `1 failed`. Today node:test's spec reporter gives
"ℹ fail 1".

## 5. The Action

### 5.1 Copilot's pull requests (supersedes decision 17's exclusion)

A pull request opened by Copilot, `copilot-swe-agent[bot]` or
`github-actions[bot]` joins another pull request's run at the newest of its
commits that has a workflow run a person let start: a run on this repository,
event `pull_request`, for that head SHA, whose conclusion is not
`action_required`. Approval restarts the held run as attempt 2; a repository
that turned Copilot's approval off starts it at attempt 1. A manual re-run
looks the same, and also needs write access.

- Calls, per held pull request: `GET /repos/{repo}/actions/runs?event=pull_request&branch=<head ref>&per_page=100`
  and `GET /repos/{repo}/pulls/{n}/commits?per_page=100`, to order the SHAs
  and keep only this pull request's commits. They need `actions: read`.
- On 403 or 404: today's exclusion, with the note "Add `actions: read` to the
  workflow's permissions to check Copilot's pull requests once someone lets
  their run start."
- The SHA is fetched like every other pull request's. If it can't be fetched
  (force-pushed away), the pull request is left out with a note.
- "Checked together" names the commit: `#12 copilot/fix-x at abc1234`.
- The docs' setup adds `actions: read`.

### 5.2 Licensing, still switched off

Dodo Payments license keys replace the offline Ed25519 keys, because a
monthly subscription needs a key that stops working when it lapses. Only on a
private repository, only after the 14-day trial, only with `LICENSING` on:
`POST https://live.dodopayments.com/licenses/validate` with
`{"license_key": "<key>"}` returns `{"valid": true|false}`. `valid: false`
gives the trial-over refusal. A network error, 429 or 5xx runs the check
anyway with a warning. Only the key is sent. The docs say so before it is
switched on.

## 6. Not in 2.1

Remembering past runs (reusing alone results by commit and check), flaky-test
memory, `pytxo fix`, any model call, a pair search without a cause, and
causes other than a renamed or removed definition.

## 7. Done means

- `pytxo demo` prints the renamed cause and the fix, and still exits 0; the
  pinned demo text in `crates/pytxo-cli/tests/cli.rs` follows. The README,
  docs, site e2e and hero evidence follow at 2.1's promotion, like 2.0.
- Tests: cause (renamed, removed, the user defines the name itself, an
  unsupported language, not confirmed, CRLF); culprit pair (pair proven and
  rest passes, pair passes, rest fails, deadline); `why_lines` on node:test,
  jest, pytest and cargo output; the hook's `--block` reason in each worktree;
  the Action's held pull request joining, left out on 403, and left out when
  its commit is gone; license validate paths.
- `cargo fmt --all --check`; clippy `-D warnings` on pytxo-land and
  pytxo-cli, full and `--no-default-features`; `cargo test -p pytxo-land -p pytxo-cli`;
  `node --test actions/land/pytxo-action.test.mjs`.
- The lean binary grows by less than 200 KB.
