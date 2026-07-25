# Release changelog (maintainers)

Per-version notes for GitHub Releases live in [`distribution/release-notes/`](distribution/release-notes/).
Workflow uses `distribution/release-notes/v{version}.md` when present.

Maintainer guide: [`docs/07-guides/release-workflow.md`](docs/07-guides/release-workflow.md)

---

# Pytxo v0.8.0

Tray-resident Desktop, full-matrix updater, UI lag fixes, Fumadocs docs. See [`distribution/release-notes/v0.8.0.md`](distribution/release-notes/v0.8.0.md).

Previous: [`v0.7.0`](distribution/release-notes/v0.7.0.md).

---

# Pytxo v0.7.0

Desktop daily-path reliability. See [`distribution/release-notes/v0.7.0.md`](distribution/release-notes/v0.7.0.md).

---

# Pytxo v0.6.0

Desktop Settings / workspaces / chroma accents. See [`distribution/release-notes/v0.6.0.md`](distribution/release-notes/v0.6.0.md).

Previous: [`v0.5.0`](distribution/release-notes/v0.5.0.md).

---

# Pytxo v0.4.0

Pytxo Desktop 2.0, text-first Flow, local-first Voice, and hardened ADE dispatch. See [`distribution/release-notes/v0.4.0.md`](distribution/release-notes/v0.4.0.md).

---

# Pytxo v0.3.5

Reality Deck UX overhaul — setup wizard, agent dashboard, workspace 3D topology, smart CLI install, themes. See [`distribution/release-notes/v0.3.5.md`](distribution/release-notes/v0.3.5.md).

---

# Pytxo v0.3.4

**Production go-live (Phases 50–59)** — inference proxy config split, Link/Cloud/proxy health, Enterprise seats/policy, 3D symbol topology, measured benchmark scripts.

## Highlights

- **ADR-0024** — `billing.inference_proxy_url` separate from Link; doctor checks proxy + cloud health
- **Ultra proxy** — streaming metering, rate limits, `GET /v1/wallet/balance` on Link
- **Cloud hardening** — non-root Docker, Redis cache option, egress allowlist
- **Reality Deck** — CSP allows `proxy.pytxo.com` + `cloud.pytxo.com`; symbol-level 3D nodes
- **Enterprise GA** — org seats migration, policy admin API, audit trail

## Install

```bash
npm i -g pytxo@0.3.4
pytxo doctor
```

---

# Pytxo v0.3.3

**Trust Enter fix + headless CLI** — Windows Enter on workspace trust modal, inline trust errors, and scriptable `pytxo agents`, `pytxo shell --eval`, `pytxo run --ade`.

## Highlights

- **Trust Enter (Windows)** — `is_submit_key` accepts Enter, `\r`, `\n`, and Release events; errors render in modal footer
- **Headless CLI** — `pytxo agents` (`--json`), `pytxo shell --eval "/dry-run"`, `pytxo run --ade cursor`
- **Shell dedup** — `pytxo trust` and `pytxo agents` route through `pytxo-shell::eval_line`
- **Architecture** — `dashboard.rs` extracted from `pytxo-orchestrate`; shared `input.rs` key helper

## Install

```bash
npm i -g pytxo@0.3.3
pytxo agents
pytxo shell --eval "/help"
```

---

# Pytxo v0.3.2

**TUI polish + ADE registry** — fixes nested-Tokio panic on bare `pytxo`, lighter board refresh, ASCII splash, and `/agents` / `/use` / `--ade` for terminal CLIs (cursor, codex, opencode, aider).

## Highlights

- **TUI fix** — async shell loop (no nested `block_on`); `/help` and bare `pytxo` no longer panic
- **Chroma theme** — web-aligned palette; workspace trust agreement for untrusted folders
- **Performance** — light dashboard snapshot (no doctor PTY smoke every 2s); dirty redraw; cached scrollback styling
- **Splash** — dismissible PYTXO banner on first keystroke or command
- **ADE registry** — `pytxo-core::ade_registry`; `/agents`, `/use <ade>`, `/run --ade cursor`
- **Version sync** — `pytxo --version` matches npm package `0.3.2`

## Install

```bash
npm i -g pytxo@0.3.2
pytxo
```

---

# Pytxo v0.3.0

**Trusted Shell + multi-provider BYOK** — folder trust tier picker, 15+ provider registry, model catalog search, and routing fixes so `[[agent]]` model/provider settings apply at runtime. Builds on the v0.2 Hypervisor Shell (board, scrollback, operator prompt).

## Highlights

- **Folder trust** — VS Code–style tier picker on first launch; `/trust` and `pytxo trust`; blocks `/run` until trusted
- **Multi-provider BYOK** — direct APIs + OpenRouter gateway; `pytxo providers`; env-key injection for generic CLIs
- **Model catalog** — `pytxo models list|search|refresh` and `/models` in the shell (`pytxo-catalog` crate)
- **Routing fix** — per-agent `model`, `provider`, `cli_adapter`, `api_key_env` honored in `RunContext`
- **TUI polish** — trust modal, minimalist board header, Tab slash completion, `RunFinished` events
- **Docs** — [Folder trust](apps/web/content/docs/getting-started/folder-trust.mdx), [Providers & BYOK](apps/web/content/docs/reference/providers-byok.mdx), [Models](apps/web/content/docs/reference/models.mdx), ADR-0013, ADR-0014

## Install

```bash
npm i -g pytxo@0.3.0
pytxo trust orbit
pytxo models search deepseek --provider openrouter
```

---

# Pytxo v0.2.0

**Hypervisor Shell** — mission control with board, scrollback, and operator prompt. Shared `pytxo-shell` router; optional `pytxo-planner` behind `PYTXO_PLANNER=1` or `[planner] enabled`. Shipped inside v0.3.0 (not published separately to npm).

## Highlights

- **Hypervisor Shell** — default `pytxo` TUI: `/help`, `/doctor`, `/dry-run`, `/run`, `/status`, `/logs`, `/stop`
- **pytxo-shell** — slash command parser + orchestrate dispatch wrappers (TUI, future MCP)
- **Runtime tasks** — `RunOptions.tasks`, `task_cmd_template`, `task_prompts` for shell-driven runs
- **pytxo-planner** — stub + heuristic decomposer when planner flag is on
- **Docs** — [Hypervisor Shell guide](apps/web/content/docs/getting-started/hypervisor-shell.mdx), ADR-0012

---

# Pytxo v0.1.0

First public release of the **Pytxo** agent hypervisor — local-first PTY orchestration, SQLite telemetry, and an interactive terminal dashboard.

## Highlights

- **CLI** — `init`, `doctor`, `run`, `status`, `logs`, `stop`, `project`, `hitl`, `domains`
- **Default TUI** — run `pytxo` with no subcommand for a live dashboard (doctor, runs, domains, HITL)
- **Three moats** — Signal Core scaffolding, Blast Shield worktrees, Race Shield scheduling
- **MCP** — `pytxo-mcp` for Cursor and IDE integration
- **Reality Deck** — optional Tauri desktop telemetry (`apps/desktop`)

## Install

```bash
npm i -g pytxo@0.3.0
pytxo doctor
```

Or use the public install script ([pytxo-releases](https://github.com/Pytxo-dev/pytxo-releases)):

```bash
curl -fsSL https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/install.sh | bash
```

Prebuilt binaries: [github.com/Pytxo-dev/pytxo-releases/releases](https://github.com/Pytxo-dev/pytxo-releases/releases)

## Assets

Prebuilt binaries are attached to this release: `pytxo-linux-x64`, `pytxo-linux-arm64`, `pytxo-darwin-arm64`, `pytxo-darwin-x64`, `pytxo-windows-x64.exe`.

Verify with `SHA256SUMS.txt`.
