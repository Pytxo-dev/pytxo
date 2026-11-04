# Pytxo 2.0: Impulse

Status: in progress (2026-10-28). Strategy: the October 2026 market research and
"Pytxo Business Plan: Impulse". Reversible choices are logged in
[DECISIONS.md](DECISIONS.md).

One product: a check that merges several agents' branches in a throwaway
worktree, runs the project's real tests on the combined result, and blocks the
merge if the combination breaks. Bar: from first sight to a verdict on your own
repo in under 60 seconds, one command, no flags, no config, no account.

## What exists (verified in code, not docs)

- `pytxo land` already merges inputs in memory, runs checks in a temp worktree,
  re-runs a failed combined check, and exits 0/2/3. It needs explicit branches
  and `--check`, and prints receipt-heavy output.
- `pytxo` with no arguments opens the TUI. Nothing detects agent branches.
- Pybench builds throwaway repos with Node-only tests (`price()`/`unitPrice()`).
- The Action needs `inputs`, a label and a manual fetch loop, writes only the
  job summary, and reports errors as "blocked".
- The npm launcher downloads a 30 MB binary that bundles the TUI, PTY runner,
  SQLite and an HTTP stack. Measured: cold `npx pytxo@1.3.6 --version` 9.2 s,
  of which 3.2 s is the download.
- No Claude Code plugin, no Codex hook, no license code, no telemetry.

## Phase 1: the check (crates)

- `crates/pytxo-land/src/detect.rs` (new): target branch, agent branches
  (prefixes, remote-tracking, linked worktrees; configurable), test command
  from the target's tree, uncommitted-work notes, state fingerprint for hooks.
- `crates/pytxo-land/src/checks.rs`: more ecosystems; skip installs when
  `package.json` has no dependencies; read from a git tree, not the checkout.
- `crates/pytxo-land/src/lib.rs`: `PerInput::OnFailure` (combined first,
  each branch alone only after a failure), `core.longpaths` on Windows.
- `crates/pytxo-land/src/render.rs`: `verdict()` plus a 3-line plain render;
  Markdown leads with the verdict, details folded, install line last.
- `crates/pytxo-land/src/bench.rs`: public demo repo builder.
- `crates/pytxo-cli/src/main.rs`, `check_cmd.rs`, `demo.rs`, `hook.rs`:
  `pytxo` = `pytxo check` (alias `land`), `pytxo demo`, `pytxo hook`;
  `--verbose` shows the old detail; usage errors exit 1, never 2.
- `crates/pytxo-cli/src/legacy.rs` + `Cargo.toml` feature `full`: frozen
  commands (shell, missions, fleets, tiers) compile only with `full`; the npm
  binary is built with `--no-default-features`.

Tests: detection (prefixes, remotes, worktrees, merged/squashed, config),
test-command detection per ecosystem, verdict lines for every outcome, demo
end to end, Windows paths with spaces and linked worktrees.

## Phase 2: surfaces

- `actions/land/action.yml` + `discover.mjs` (+ `discover.test.mjs`): one
  `uses:` line, PR discovery by prefix or bot author, sticky comment with
  job-summary fallback, `pull_request` only, no token in the check step.
- `integrations/claude-code/` (plugin, Stop hook) + `.claude-plugin/marketplace.json`;
  `integrations/codex/` (hooks.json + steps).
- `pytxo fix` if time allows.
- `crates/pytxo-cli/src/license.rs`: Pro gate for private repos in CI
  (14-day trial, then a license key secret), compiled in but switched off.

## Phase 3: story and release prep

- `POSITIONING.md`, `README.md`, `packages/pytxo/{package.json,README.md}`.
- `apps/web`: hero, homepage sections (Desktop moves to `/desktop`), plans
  (Free; Pro $29 once for the first 500, then $5/month or $48/year; Team
  planned), docs navigation with an Advanced section, new docs pages, e2e.
- `.github/workflows/{ci,release}.yml`: lean CLI build, action/integration
  tests, `v2` tag safe. Version 2.0.0 and release notes, unpublished.
- Fresh-subagent review of the full diff against "Done means", then a final
  report with timings and the exact ship commands.
