---
title: Documentation changelog
slug: changelog-docs
status: active
tags: [meta]
audience: [human]
layer: meta
created: 2026-06-02
updated: 2026-07-31
related: [[MOC-home]]
---

# Documentation changelog

## 2026-07-31 (v1.0.0 release)

- Synchronized install, Desktop setup, first-mission, provider, website, npm,
  Tauri, Cargo, onboarding, and release metadata on **1.0.0**.
- Documented the vendor-owned CLI session boundary for Codex / ChatGPT, Claude
  Code, Cursor Agent, OpenCode, Gemini CLI, and Aider; direct provider keys
  remain a separate native credential surface.
- Added [[v1-provider-auth-onboarding-research]],
  [[v1-product-reference-research]], [[v0.13-demo-benchmark-readiness]], and the
  real-repository Signal / control-plane evidence to the documentation map.
- Added the guided first-mission example, Remotion demo source, ElevenLabs
  voiceover, Screen Studio shot list, and Windows-first distribution caveat.

## 2026-07-27 (architecture research synthesis)

- Added [[pytxo-architecture-research]] — primary-source map of vision, three-tier model, moats, permission profiles, ADRs, crates, and program state; linked from [[MOC-home]].
- Mission Loop Phase 1 docs: [[mission-loop]], [[first-mission]], [[mission-loop-dogfood]], [[ADR-0031-mission-planner-byok-scout]], [[ADR-0032-desktop-2-focus-flow-primary]]; vision/positioning locked to inspectable mission thesis; `pytxo mission` CLI.

## 2026-07-26 (Desktop 0.11.0 dangerous UX + install trust)

- Landed [[desktop-dangerous-ux-2026-07]]; linked from [[MOC-home]] and [[desktop-ui-improvement-backlog]].
- Install trust: Rust CLI install (no PowerShell storm), NSIS logo branding, bundle id `com.pytxo.desktop`.

## 2026-07-09 (Pytxo Desktop rename + plain positioning)

- Product UI name: **Reality Deck** → **Pytxo Desktop** ([[ADR-0028-desktop-product-name]]).
- Visual system note: `reality-deck-visual-system` → [[desktop-visual-system]].
- Vision, glossary, beyond-the-ade, landscape, and compare guides rewritten with plain-language leads.
- Workspaces = modular multi-root projects in Desktop UX docs.

## 2026-06-05 (v0.3.3 trust Enter + headless CLI)

- **TUI:** Windows Enter on trust modal; `input.rs` key normalization; inline trust errors.
- **CLI:** `pytxo agents`, `pytxo shell --eval`, `pytxo run --ade`; trust/agents via `pytxo-shell`.
- **Orchestrate:** `dashboard.rs` module split.

## 2026-06-05 (v0.3.2 TUI + ADE registry)

- **TUI:** nested-runtime fix; light `dashboard_snapshot`; splash overlay; `/agents`, `/use`, `/run --ade`.
- **Core:** `ade_registry` module; `PYTXO_CLI` telemetry on ultra-managed routes.
- **Test envs:** `pytxo.cursor.toml`, `pytxo.opencode.toml` fixtures; ADE table in `tooling/test-envs/README.md`.

## 2026-06-05 (phases 9–16)

- **Phase 9:** [[ADR-0011-modular-project-manifest]]; [[mvp-bootstrap]] synced with Phases 4–8; CI benchmarks (Win+Linux); Deck repo-path dispatch input.
- **Phase 10:** modular projects v1 — `ProjectManifest` (`pytxo-core/src/project.rs`), `pytxo project init|list|paths|run`, store migration 004 (`project_id`/`root_id`), runs tagged per project. [[modular-projects]] status updated.
- **Phase 11:** Galaxy HITL — `HitlQueue` (`pytxo-runner`), `list_hitl`/`hitl_respond` IPC, Deck approval panel, `pytxo hitl` CLI. [[race-shield]] + [[phase-2-reality-deck]] marked shipped.
- **Phase 12:** hypervisor catalog `~/.pytxo/hypervisor.db` (`pytxo_store::Catalog`), upsert on `ensure_domain`, `pytxo domains`, Deck "All projects" home. [[execution-domains]] v2 shipped.
- **Phase 13:** Signal closed-loop retry emits `signal-retry` WAL event; MCP `pytxo_project_run` + `project_id`/`root` read routing. [[mcp-router]] updated; [[blast-shield]] overlay marked deferred spike.
- **Phase 14:** Deck topology consumes design tokens via `getComputedStyle` and sizes nodes by per-agent edited paths/saved tokens (`agent_arbitrage` IPC). [[desktop-visual-system]] updated.
- **Phase 15–16:** `HttpBillingReconciler` builds Link request envelopes (endpoints + bodies); `billing.link_reconcile` documented in `pytxo.toml.example`; [[cloud-sandbox-service]] split into Link vs cloud sandbox separate repos.

## 2026-06-04 (modular projects)

- Added [[modular-projects]] — multi-path Pytxo projects (Antigravity-style roots); linked from [[product-vision]], [[execution-domains]], [[glossary]], [[MOC-home]].

## 2026-06-04 (audit + GitHub visibility)

- Code: `PytxoConfig::db_path_at` / `state_path_at`; Deck poll cursors per domain+agent; `billing.link_reconcile` → `HttpBillingReconciler`; benchmark scripts resolve monorepo root.
- Docs: [[race-shield]] status table (PTY stdin shipped, HITL planned); [[phase-2-reality-deck]] IPC v2 shipped vs planned; [[github-organization]] public/private matrix from live `gh repo list`.

## 2026-06-02 (Pytxo Ultra metering)

- [[ADR-0009-ultra-managed-metering]]; Ultra tier in [[tiers-hobbyist-pro-max]]; dual-ledger arbitrage in [[token-arbitrage]].
- Rust: `pytxo-core::billing`, store migration 003, runner `ArbitrageProfiler`, orchestrate hybrid wallet.

## 2026-06-02 (monorepo reorganization)

- Pytxo Desktop at `apps/desktop`; export slot `apps/desktop-export`; `tooling/scripts` and `tooling/benchmarks`.
- Updated [[repository-layout]], [[github-organization]], [[mvp-bootstrap]], root `README.md`, `AGENTS.md`, `CONTRIBUTING.md`.

## 2026-06-02 (permission profile implementation)

- Rust: `PermissionProfile`, `PermissionEngine`, `HypervisorRegistry` in crates; updated enforcement tables in [[permission-profile-engine]] and [[execution-domains]].

## 2026-06-02 (permission profile + hypervisor)

- Added [[ADR-0008-local-permission-profile-four-tiers]], [[permission-profile-engine]], [[execution-domains]].
- Updated [[blast-shield]], [[race-shield]], [[sqlite-wal-logging]], [[presentation-passive-telemetry]], [[phase-2-reality-deck]], [[three-tier-model]], [[product-vision]], [[agent-os-vs-virtual-workspace]].
- Updated [[adr-index]] (ADR-0006–0008), [[glossary]], [[architecture-index]], [[MOC-home]], [[pytxo-toml]], `AGENTS.md`, `crates/README.md`.

## 2026-06-02 (vision)

- Added [[product-vision]] — agent hypervisor thesis, three moats ([[signal-core]], [[blast-shield]], [[race-shield]]), Pytxo Desktop aesthetic, BYOK cloud.
- Added [[desktop-visual-system]]; updated [[glossary]], [[MOC-home]], [[architecture-index]], [[three-tier-model]], root `README.md`, `AGENTS.md`.

## 2026-06-02

- Canonical GitHub org: [Pytxo-dev](https://github.com/Pytxo-dev); see [[github-organization]].
- Phase 2 docs: Pytxo Desktop, MCP setup, ADR-0006/0007, updated [[mvp-bootstrap]].

- Implemented Phase 0–1 Rust CLI (`crates/pytxo-*`).
- Added ADR-0005 worktree MVP isolation, CLI/pytxo.toml reference, first-three-agent-run tutorial, benchmark scripts.
- Bootstrapped `docs/` Obsidian vault (PARA-inspired layout).
- Atomized Pytxo Architecture & Strategy v2026.2 into linked notes.
- Added ADR-0001 through ADR-0004 (Accepted).
- Added `AGENTS.md`, `CONTRIBUTING.md`, `.cursor/rules/`, root `README.md`.
- Added compare/operations/ecosystem guides for 2026 agent-team landscape.
