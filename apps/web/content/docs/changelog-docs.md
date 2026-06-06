# Documentation changelog

## 2026-06-05 (phases 9–16)

- **Phase 9:** [ADR-0011-modular-project-manifest](/docs/adr-0011-modular-project-manifest); [mvp-bootstrap](/docs/mvp-bootstrap) synced with Phases 4–8; CI benchmarks (Win+Linux); Deck repo-path dispatch input.
- **Phase 10:** modular projects v1 — `ProjectManifest` (`pytxo-core/src/project.rs`), `pytxo project init|list|paths|run`, store migration 004 (`project_id`/`root_id`), runs tagged per project. [modular-projects](/docs/modular-projects) status updated.
- **Phase 11:** Galaxy HITL — `HitlQueue` (`pytxo-runner`), `list_hitl`/`hitl_respond` IPC, Deck approval panel, `pytxo hitl` CLI. [race-shield](/docs/race-shield) + [phase-2-reality-deck](/docs/phase-2-reality-deck) marked shipped.
- **Phase 12:** hypervisor catalog `~/.pytxo/hypervisor.db` (`pytxo_store::Catalog`), upsert on `ensure_domain`, `pytxo domains`, Deck "All projects" home. [execution-domains](/docs/execution-domains) v2 shipped.
- **Phase 13:** Signal closed-loop retry emits `signal-retry` WAL event; MCP `pytxo_project_run` + `project_id`/`root` read routing. [mcp-router](/docs/mcp-router) updated; [blast-shield](/docs/blast-shield) overlay marked deferred spike.
- **Phase 14:** Deck topology consumes design tokens via `getComputedStyle` and sizes nodes by per-agent edited paths/saved tokens (`agent_arbitrage` IPC). [reality-deck-visual-system](/docs/reality-deck-visual-system) updated.
- **Phase 15–16:** `HttpBillingReconciler` builds Link request envelopes (endpoints + bodies); `billing.link_reconcile` documented in `pytxo.toml.example`; [cloud-sandbox-service](/docs/cloud-sandbox-service) split into Link vs cloud sandbox separate repos.

## 2026-06-04 (modular projects)

- Added [modular-projects](/docs/modular-projects) — multi-path Pytxo projects (Antigravity-style roots); linked from [product-vision](/docs/product-vision), [execution-domains](/docs/execution-domains), [glossary](/docs/glossary), [MOC-home](/docs/moc-home).

## 2026-06-04 (audit + GitHub visibility)

- Code: `PytxoConfig::db_path_at` / `state_path_at`; Deck poll cursors per domain+agent; `billing.link_reconcile` → `HttpBillingReconciler`; benchmark scripts resolve monorepo root.
- Docs: [race-shield](/docs/race-shield) status table (PTY stdin shipped, HITL planned); [phase-2-reality-deck](/docs/phase-2-reality-deck) IPC v2 shipped vs planned; [github-organization](/docs/github-organization) public/private matrix from live `gh repo list`.

## 2026-06-02 (Pytxo Ultra metering)

- [ADR-0009-ultra-managed-metering](/docs/adr-0009-ultra-managed-metering); Ultra tier in [tiers-hobbyist-pro-max](/docs/tiers-hobbyist-pro-max); dual-ledger arbitrage in [token-arbitrage](/docs/token-arbitrage).
- Rust: `pytxo-core::billing`, store migration 003, runner `ArbitrageProfiler`, orchestrate hybrid wallet.

## 2026-06-02 (monorepo reorganization)

- Reality Deck at `apps/desktop`; export slot `apps/desktop-export`; `tooling/scripts` and `tooling/benchmarks`.
- Updated [repository-layout](/docs/repository-layout), [github-organization](/docs/github-organization), [mvp-bootstrap](/docs/mvp-bootstrap), root `README.md`, `AGENTS.md`, `CONTRIBUTING.md`.

## 2026-06-02 (permission profile implementation)

- Rust: `PermissionProfile`, `PermissionEngine`, `HypervisorRegistry` in crates; updated enforcement tables in [permission-profile-engine](/docs/permission-profile-engine) and [execution-domains](/docs/execution-domains).

## 2026-06-02 (permission profile + hypervisor)

- Added [ADR-0008-local-permission-profile-four-tiers](/docs/adr-0008-local-permission-profile-four-tiers), [permission-profile-engine](/docs/permission-profile-engine), [execution-domains](/docs/execution-domains).
- Updated [blast-shield](/docs/blast-shield), [race-shield](/docs/race-shield), [sqlite-wal-logging](/docs/sqlite-wal-logging), [presentation-passive-telemetry](/docs/presentation-passive-telemetry), [phase-2-reality-deck](/docs/phase-2-reality-deck), [three-tier-model](/docs/three-tier-model), [product-vision](/docs/product-vision), [agent-os-vs-virtual-workspace](/docs/agent-os-vs-virtual-workspace).
- Updated [adr-index](/docs/adr-index) (ADR-0006–0008), [glossary](/docs/glossary), [architecture-index](/docs/architecture-index), [MOC-home](/docs/moc-home), [pytxo-toml](/docs/pytxo-toml), `AGENTS.md`, `crates/README.md`.

## 2026-06-02 (vision)

- Added [product-vision](/docs/product-vision) — agent hypervisor thesis, three moats ([signal-core](/docs/signal-core), [blast-shield](/docs/blast-shield), [race-shield](/docs/race-shield)), Reality Deck aesthetic, BYOK cloud.
- Added [reality-deck-visual-system](/docs/reality-deck-visual-system); updated [glossary](/docs/glossary), [MOC-home](/docs/moc-home), [architecture-index](/docs/architecture-index), [three-tier-model](/docs/three-tier-model), root `README.md`, `AGENTS.md`.

## 2026-06-02

- Canonical GitHub org: [Pytxo-dev](https://github.com/Pytxo-dev); see [github-organization](/docs/github-organization).
- Phase 2 docs: Reality Deck, MCP setup, ADR-0006/0007, updated [mvp-bootstrap](/docs/mvp-bootstrap).

- Implemented Phase 0–1 Rust CLI (`crates/pytxo-*`).
- Added ADR-0005 worktree MVP isolation, CLI/pytxo.toml reference, first-three-agent-run tutorial, benchmark scripts.
- Bootstrapped `docs/` Obsidian vault (PARA-inspired layout).
- Atomized Pytxo Architecture & Strategy v2026.2 into linked notes.
- Added ADR-0001 through ADR-0004 (Accepted).
- Added `AGENTS.md`, `CONTRIBUTING.md`, `.cursor/rules/`, root `README.md`.
- Added compare/operations/ecosystem guides for 2026 agent-team landscape.
