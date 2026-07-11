---
title: MCP router (reference)
slug: mcp-router
status: active
tags: [mcp, reference]
audience: [agent]
layer: orchestration
created: 2026-06-02
updated: 2026-07-10
related: [[mcp-hub-integration]], [[cursor-vs-vscode-vs-neovim]], [[modular-projects]]
---

# MCP router (reference)

Planned component: **local-first MCP hub** in the Rust orchestration layer.

## Responsibilities (target)

- Register tools/resources exposed by execution yard processes.
- Route IDE MCP client requests to the correct agent session.
- Apply [[regex-sanitization]] on outbound context.
- Emit audit events to [[sqlite-wal-logging]].

## Non-goals

- Replacing the IDE’s own MCP servers where not needed.
- Storing provider API keys server-side (BYOK stays with developer).

## Implementation status

**stdio MCP server (shipped):** `crates/pytxo-mcp` (JSON-RPC). Tools include `pytxo_dry_run`, `pytxo_run`, `pytxo_status`, `pytxo_logs`, `pytxo_read`, `pytxo_read_scaffolded`, `pytxo_stdin`, plus hub/routing tools below. Setup: [[mcp-cursor-setup]], [[cursor-mcp-pytxo]].

**Modular project routing (shipped):** `pytxo_project_run` runs a command across a project's writable roots; `pytxo_read` / `pytxo_read_scaffolded` accept `project_id` (+ optional `root` label) to resolve the target folder from the project manifest ([[modular-projects]], [[ADR-0011-modular-project-manifest]]) instead of a raw `repo` path.

**MCP hub v1 (shipped):** `pytxo_list_live_agents` reads the domain Race Shield registry; `pytxo_route_stdin` validates the agent is live before enqueue; MCP tool calls emit `mcp-tool` WAL events. Sanitized reads when `sanitize = true`.

**MCP hub v2 (shipped):** `pytxo_mcp_proxy` forwards JSON-RPC to live child MCP sessions registered in the domain hub; `pytxo_mcp_tools_list` aggregates child tools as `agent:{id}/{tool}`. Runner registers/deregisters on agent lifecycle when `[mcp_hub].enabled`. WAL `mcp-tool` audit preserved.

**MCP hub v3 (shipped, Phase 67):** resource subscriptions — `subscribe_resource` / `notify_resource_updated` in runner `mcp_hub.rs`.

**Deferred:** multi-hop routing across remote MCP hubs; streaming resource updates to IDE clients beyond in-process notify.
