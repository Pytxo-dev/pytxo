---
title: Multiple accounts per agent CLI
status: Implementing
date: 2026-10-17
tags: [desktop, runner, setup, accounts]
---

# Multiple accounts per agent CLI

## Outcome

A user keeps several sign-ins per agent CLI (two Claude Max accounts, two
ChatGPT accounts for Codex), sees each account's
sign-in state in Desktop Setup and in `pytxo accounts list`, and picks which
account new work uses, per CLI or per configured agent. Each account is the
vendor CLI's own config directory. Pytxo never opens, copies or stores the
credential files inside it.

## Research (primary sources, 2026-10-17)

| CLI | Isolation | Source | Shipped in v1 |
|-----|-----------|--------|---------------|
| Claude Code | `CLAUDE_CONFIG_DIR`; credentials file and macOS Keychain entry are both keyed to the directory | code.claude.com/docs/en/authentication ("Log in with multiple accounts", "Credential management") | Yes |
| Codex | `CODEX_HOME`; `auth.json` under it, keyring key is `cli|sha256(canonical CODEX_HOME)[..16]` | learn.chatgpt.com/docs/auth; `openai/codex` `codex-rs/login/src/auth/storage.rs` `compute_store_key` | Yes |
| Cursor Agent | `CURSOR_CONFIG_DIR` relocates `cli-config.json` only; the sign-in lives elsewhere | cursor.com/docs/cli/reference/configuration; local check below | No |
| Antigravity (`agy`) | None: Windows Credential Manager entry, no relocation variable (`AGY_ACCOUNT` selects connector accounts for sandboxed tools, not the Google sign-in) | local `agy.exe` strings, `agy --help` | No |
| Gemini CLI | `GEMINI_CLI_HOME` moves files, but the keychain entry is fixed (`gemini-cli-oauth` / `main-account`) | geminicli.com/docs/reference/configuration; `google-gemini/gemini-cli` `oauth-credential-storage.ts` | No |

Local check on Windows: with a fresh `CLAUDE_CONFIG_DIR`, `claude auth status
--json` reports `loggedIn: false` while the default home stays signed in; with a
fresh `CODEX_HOME`, `codex login status` reports `Not logged in`.
With a fresh, empty `CURSOR_CONFIG_DIR`, `cursor-agent status` still reports
`Logged in`, so Cursor's sign-in is per OS user and `cursor-agent logout` from
an "account" would sign the real session out.
`codex login status` still says `Logged in using ChatGPT` when the refresh
token is dead, so expiry only shows up at run time.

BridgeMind: bridgemind.ai returns 403 to fetchers and no public page documents
its account feature, so this design copies no specific BridgeMind behavior.

### Terms that shape the design

- Anthropic, Claude Code legal page: sign-in "must complete through
  Anthropic's own flow" and developers "may not collect, store, or intermediate
  Claude.ai credentials or session tokens"; Pro/Max limits "assume ordinary,
  individual usage". Consumer Terms §2: no sharing login or making an account
  available to anyone else; §3: no "bypassing any of our systems or protective
  measures".
- OpenAI Terms of Use: no sharing credentials; may not "circumvent any rate
  limits or restrictions".
- Google Antigravity Additional Terms §6: "Using third party software, tools, or
  services to access the Service ... is a breach"; accounts were suspended in
  waves in 2026, with reported false positives.

Consequences: Pytxo runs only the vendor's own sign-in command in the vendor
terminal; every account must be the user's own; **no automatic switching or
load spreading when an account hits a limit** (that is the conduct both
terms name). A limit is reported as a limit, with no "try another account"
prompt. Antigravity stays single-account.

## Design

**Where accounts live.** `~/.pytxo/agent-accounts/<cli>/<name>/` is the
vendor config directory for that account (`PYTXO_HOME` overrides the home, as
elsewhere). `default` means the CLI's own home (`~/.claude`, `~/.codex`) and
sets no variable. `~/.pytxo/agent-accounts/<cli>/.active`
holds the account name new work uses. Names: `[a-z0-9][a-z0-9-]{0,31}`,
`default` reserved. No database rows, no copied files.

**Registry.** `pytxo_core::agent_accounts::account_env(cli)`:
`CLAUDE_CONFIG_DIR`, `CODEX_HOME`; `None` for every other CLI.

**Runner (per launch).** After the agent's command is resolved, the runner
identifies the CLI from the command's first word, picks the account
(`[[agent]] account = "work"` in `pytxo.toml` for that agent, else the CLI's
active account), and sets the one variable in `ChildLaunchEnv`. A missing
account directory fails that agent before spawn with the fix command. The run
log gets one `account` event naming the CLI and account name, never the path.
After a failed exit, the runner scans the output tail for the vendor's
expired-login and usage-limit messages and appends one `pytxo:` line naming
the account and the fix.

Permission profile: applies identically under DeepSpace, Orbit, Galaxy and
Supernova; the variable is a path, not a secret, and the profile's existing
environment policy still runs after it. Execution domain: account homes are
host-global, outside every domain; the choice is read at each launch and
recorded in that domain's run log.

**Sign-in and status.** One shared probe in `pytxo-core` (moved from Desktop)
runs `claude auth status --json` or `codex login status` with the account's variable set, in the background, no window. Sign-in
runs `claude auth login` or `codex login` with the variable set: in the current terminal for the CLI, in a new vendor terminal
for Desktop. States: signed in, signed out, unknown. Expired and limit
reached are run-time states (above).

**CLI.**
```
pytxo accounts list [--json]
pytxo accounts add <cli> <name>          # creates the folder, opens vendor sign-in
pytxo accounts login <cli> [<name>]
pytxo accounts use <cli> <name|default>
pytxo accounts remove <cli> <name>       # vendor logout, then deletes the folder
```

**Desktop Setup.** Under Claude Code and Codex in Agent CLIs,
an Accounts list: Default plus named accounts, each with its state, a "Selected"
marker or a Select button (distinct from the CLI's "Use in new work"), Sign in when not signed in, Remove for
named accounts, and an Add account field. The CLI row's own status reflects
the active account, so Work's readiness gate checks the account that will
actually run. Other CLIs show nothing new.

## Not in v1

Per-run account choice inside the Work composer (Setup's choice applies to
new work), automatic switching, sharing settings or MCP config between
account folders (each account starts with the vendor's defaults), email
display (keeps addresses out of demo footage), Cursor, Gemini and Antigravity
accounts.

## Acceptance

- Core unit tests: name validation, add/list/use/remove, launch env per CLI,
  missing account refused, default sets nothing.
- Runner: env carries the account variable for a bound agent; failure lines
  recognised.
- Sanitizer redacts Anthropic `sk-ant-` keys and bare JWTs.
- `pytxo accounts` round trip against a scratch `PYTXO_HOME`.
- Desktop `npm run check`; native Setup shows Default and a named account
  with real probe states; screenshots inspected.
