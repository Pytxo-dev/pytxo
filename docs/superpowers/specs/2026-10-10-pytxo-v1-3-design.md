---
title: Pytxo v1.3 design — Land
slug: 2026-10-10-pytxo-v1-3-design
status: active
tags: [spec, release, v1.3, land, receipts, pybench, account]
audience: [human, agent]
layer: meta
created: 2026-10-10
updated: 2026-10-10
related: [[pytxo-business-strategy-2026-10-10]], [[product-vision]], [[positioning]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[ADR-0042-landing-gate-and-receipts]]
---

# Pytxo v1.3 design — "Land"

Authorized by Matt on 2026-10-10: "make Pytxo much better with a polished and
repositioned, much more mature, v1.3 … brainstorm and design … and implement
it. Go all in." This spec records the decisions and the build. Publication
steps (merge to main, tag, release, deploy, Link migration, posts) still need
Matt's action-time approval.

## 1. Decisions taken for Matt

| # | Decision (from the strategy note §9) | Taken | Why |
|---|---|---|---|
| 1 | Re-aim at the landing layer | **Yes.** v1.3 ships the first landing-gate surface (`pytxo land`), receipts, and the repositioned copy | The only position with budget and without a free incumbent |
| 2 | License boundary | **Everything in this repository stays MIT.** Future hosted-ledger server code that charges money lives outside this repo and is decided when it exists | A license change during launch week costs more trust than it protects; nothing paid exists yet |
| 3 | Team price | **Local and the free account stay free. Team is "planned" on the site with no number.** Internal anchor stays ~$20/dev/mo | Publishing a price for an unbuilt product creates a commitment and an honesty problem |
| 4 | Founder story | **Later** (Matt) | — |
| 5 | Grants | **Emergent Ventures first** (rolling, 13+), 1517 and Z Fellows after launch numbers exist. Applications are Matt's to send | Fits age and stage; the others want traction |
| 6 | Launch date | **As soon as v1.3 passes its gates** (Matt) | — |
| 7 | Pytxo Pro (added 2026-10-14, Matt approved) | **$12/month through Dodo Payments (merchant of record): private receipts, unlimited shared receipts (free keeps 25), private Pybench.** Gating lives in Link, which stays MIT; this amends #2 and #3 for an individual plan only, Team stays unpriced | Payments needed something real to sell; hosted perks are what a paid individual plan can honestly offer today, and an MIT license does not weaken a hosted service |

Further decisions inside this spec:

- **Login is optional and earns perks.** Mandatory login would break the
  "no Pytxo account for local work" promise printed on the site, docs, fact
  sheet and Product Hunt pack, adds an outage dependency to a local tool, and
  reads badly for an MIT project on HN. Perks are real hosted features, not
  removed local ones.
- **Benchmark name.** The suite is called **Pybench** and runs as
  `pytxo bench`. A PyPI package named `pybench` (scientific benchmarking) and
  CPython's retired `Tools/pybench` already exist, so Pytxo ships no separate
  `pybench` binary or package.

## 2. One sentence

**Pytxo lands agent work: it checks the combined change, shows exactly what
will land, refuses if anything moved, and leaves a receipt.** Whether the
agents ran inside Pytxo (Desktop fleet) or in any other tool (branches from
Orca, Conductor, Codex, Claude Code, Copilot), the end of the run is the same
gate.

Site eyebrow becomes "The landing layer for coding agents"; the H1 "Many
agents. One verified change." stays (it is already the Product Hunt and film
line).

## 3. Scope

### 3.1 `pytxo land` — the landing gate (new crate `pytxo-land`)

```text
pytxo land <ref>... [--base <ref>] [--check <cmd>]... [--per-input]
           [--into <branch>] [--apply] [--json] [--markdown] [--repo <path>]
```

1. **Resolve.** Inputs are commits (branches, tags, SHAs). The target is
   `--into` (default: the branch checked out in `--repo`); `--base` defaults
   to the target's current commit. Every input must contain the base
   (`git merge-base --is-ancestor`) or the gate reports `diverged_input`.
2. **Ownership.** For each input, `git diff --name-status base..input` gives
   its file set. A path changed by two inputs is a *shared file*. For shared
   files in a Signal Core language, base-side symbol spans (functions,
   methods, classes) are intersected with each input's changed hunks: the same
   symbol touched by two inputs is a **symbol collision**; different symbols
   in one file is **same file, separate symbols**.
3. **Combine.** In a detached temporary worktree at the base, merge inputs in
   argument order with `git merge --no-ff`. A textual conflict aborts that
   merge and marks the input `conflict`.
4. **Check.** Each `--check` runs in the combined worktree (shell, cwd =
   worktree, 15-minute timeout, last 40 output lines kept). With
   `--per-input`, the same checks also run on each input alone, which is what
   exposes "every branch passed, the combination fails".
5. **Decide.** `ready` (no conflict, all combined checks passed), `blocked`
   (conflict, failed combined check, diverged input) or `stale` (target moved
   during the gate). Symbol collisions are reported as risk; they block only
   with `--strict`.
6. **Land (opt-in).** Default is **shadow**: nothing in the repository
   changes; the combined commit is kept at `refs/pytxo/land/<id>` for
   inspection. With `--apply` and a `ready` decision:
   - target not checked out: `git update-ref refs/heads/<into> <combined> <expected-base>`
     (atomic compare-and-swap; refuses if the target moved);
   - target checked out in the repo: requires `HEAD == expected-base`, then
     `git merge --ff-only <combined>` (git refuses to overwrite local changes).
7. **Receipt.** Always written to `.pytxo/receipts/<id>.json` (see §3.3) and
   summarized to stdout, `--json`, or `--markdown` (PR-comment ready).

Exit codes: 0 ready or landed, 1 usage/IO error, 2 blocked, 3 stale.

Not in scope: rewriting history, rebasing inputs, auto-resolving conflicts,
running agents. A GitHub Action wrapper (`actions/land/action.yml`) runs the
same command in shadow mode on pull requests; publication to the Marketplace
waits for approval.

### 3.2 CLI completes the Pytxo loop

macOS and Linux users have the CLI but could not Review or Apply. v1.3 adds:

| Command | Wraps |
|---|---|
| `pytxo review <run-id> [--json]` | run contract + prepared manifest: status, base, digest, files, checks |
| `pytxo apply <run-id> --digest <sha256>` | `apply_run_changes` (digest must match the reviewed package) |
| `pytxo review <run-id> --refresh` / `--discard` | `refresh_run_review` / `discard_run_review` |
| `pytxo receipts [list\|show <id>\|share <id>]` | local receipts; `share` needs an account |

A successful Apply also writes a receipt (kind `apply`).

### 3.3 Receipt format `pytxo.dev/receipt/v1`

An in-toto Statement v1 so supply-chain tooling can read it; unsigned in v1.3
(DSSE signing is a follow-up).

```json
{
  "_type": "https://in-toto.io/Statement/v1",
  "subject": [{ "name": "git:<target>", "digest": { "gitCommit": "<sha>" } }],
  "predicateType": "https://pytxo.com/receipt/v1",
  "predicate": {
    "id": "rcpt_<26 chars>",
    "kind": "land | apply",
    "created_at": "RFC3339",
    "pytxo_version": "1.3.0",
    "decision": "ready | landed | blocked | stale | applied | refused",
    "mode": "shadow | apply",
    "base": "<sha>", "result": "<sha or null>",
    "inputs": [{ "ref": "...", "commit": "<sha>", "files": 3, "status": "merged | conflict | diverged" }],
    "shared_files": [{ "path": "...", "inputs": ["a", "b"], "symbols": [{ "name": "fn", "inputs": ["a","b"] }] }],
    "checks": [{ "scope": "combined | input:<ref>", "command": "...", "exit_code": 0, "duration_ms": 0, "passed": true }],
    "reasons": ["human-readable blocking reasons"],
    "content_digest": "sha256 of the canonical predicate without this field"
  }
}
```

Paths, commands and refs stay local. **Sharing** uploads a redacted copy:
counts, decision, check pass/fail counts, durations, pytxo version, digest —
never file paths, commands, refs, output or repository names unless the user
passes `--include-paths`.

### 3.4 Pybench (`pytxo bench`)

Deterministic, offline, no model calls; each scenario builds a throwaway git
repository in the temp directory.

| Suite | Measures |
|---|---|
| `land` | 12 scenarios across disjoint files, same file/separate symbols, same-symbol edits, textual conflict, semantic break (rename + new caller: each branch's tests pass, combined fails), test-contract break, stale target, diverged input, three-way fan-in. For each: what a per-branch-CI merge policy would land vs what `pytxo land` decides; broken states prevented; false refusals; detection layer |
| `overhead` | Gate wall time vs plain `git merge` for 2/5/10 inputs × 10/100 files |
| `signal` | Signal Core scaffold-byte reduction on a repository (default: current) |

Checks use `node` (present wherever the npm-installed agent CLIs are); the
suite reports `skipped` with the reason if `node` is missing. Output:
`pybench-<version>-<date>.json` (machine identity, git/node versions, per
scenario results, summary) plus a terminal table and `--markdown`.
`--submit` publishes the summary to the public board (account perk).
Results are synthetic mechanism tests, not real-world failure frequencies;
every surface says so.

### 3.5 Pytxo account (optional, with perks)

| Perk | Where |
|---|---|
| Share a receipt as a public page (`pytxo.com/r/<id>`) | CLI `pytxo receipts share`, Desktop Review/History |
| Keep receipts in your account across machines (Ledger v0) | Web `/account` |
| Publish Pybench results to the public board | `pytxo bench --submit`, web `/bench` |
| Early access to Team (design partners) | Web `/account` |

Sign-in: browser + PKCE. CLI listens on `127.0.0.1:<random>` for the
callback; Desktop uses the existing `pytxo-deck://auth?code&state` deep link.
Link reuses the hashed one-use code and session tables with a new scope
`account:v1` (migration 020; sessions become unique per account *and* scope).
The account credential (`pda1_…`, 30 days) is stored in the OS keyring
(Desktop) or `~/.pytxo/account.json` with user-only permissions (CLI), never
in agent environments.

Link endpoints (new): `POST /v1/account/authorize`, `POST /v1/account/exchange`,
`GET|DELETE /v1/account/session`, `POST /v1/receipts`, `GET /v1/receipts`
(mine), `GET /v1/receipts/{id}` (public), `DELETE /v1/receipts/{id}`,
`POST /v1/bench`, `GET /v1/bench` (public board).

Web: header Sign in/Account, `/account` perks + receipts list, `/r/[id]`,
`/bench`. Deploying Link (migration 020) and the site is approval-gated.

### 3.6 Website and docs

- Fix: FAQ answers clipped (accordion content was pinned to a stale measured
  height and used Base UI `data-open` selectors on a Radix primitive).
- Evidence moves into the docs: `/docs/evidence` (run records, Signal
  benchmark, Pybench results, limits). `/evidence` redirects there; the
  homepage keeps one proof band pointing at it.
- Consistency: one product sentence across metadata, hero, footer, README,
  docs and Desktop; agent support labelled honestly (Codex, Claude Code,
  Cursor Agent verified live; OpenCode and Antigravity beta); proof band uses
  the newest live record (released v1.2.4, 10 Oct) instead of a 2 Oct record
  whose "both causes are fixed" line was contradicted by the 10 Oct run.
- New docs: `pytxo land`, receipts, Pybench, account and perks, CLI review and
  apply, GitHub Action; CLI reference and changelog updated.
- Plans: Free (local), Free account (perks), Team (planned, design partners).

### 3.7 Desktop

- Settings → Account: sign in/out with the account flow, list of perks, state.
- Review: "Share receipt" after Apply when signed in; receipts written for
  every Apply.
- Agent labels: OpenCode and Antigravity marked beta in pickers.
- Version 1.3.0.

### 3.8 Vault and architecture

- ADR-0042: landing gate and receipts.
- Audit note `v1-3-audit-2026-10-10.md`: inconsistencies and weaknesses
  found, each marked fixed in v1.3 or deferred with a reason.
- AGENTS.md, vision, positioning and tiers notes updated to the landing-layer
  sentence; superseded tier/Ultra copy marked historical.

## 4. Acceptance

- `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`,
  `cargo test --workspace` pass; `pytxo-land` tests cover every decision path
  including stale and diverged.
- `pytxo bench --suite land` passes all scenario expectations on this
  machine; results JSON committed and quoted only from that file.
- Web: lint, build, link check, Playwright e2e pass; FAQ answers fully
  visible at 1440×900 and 390×844 (new test).
- Desktop: `npm run check`, release e2e pass; native check of the built exe
  for Settings → Account and Review.
- Link: unit tests for scope, redaction and public receipt reads pass.
- No claim on any surface that is not backed by a shipped path or a recorded
  result.

## 5. Approval boundary

Local branch, commits, builds and tests: authorized. Push, PR merge, tag,
release, npm publish, site deploy, Link deploy/migration, Marketplace
listing, social posts: **Matt's explicit action-time approval**.

Back: [[pytxo-business-strategy-2026-10-10]] · [[MOC-home]]
