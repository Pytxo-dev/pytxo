---
title: MCP router (reference)
slug: mcp-router
status: active
tags: [mcp, reference]
audience: [agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-02
related: [[mcp-hub-integration]], [[cursor-vs-vscode-vs-neovim]]
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

**Phase 2 stub:** `crates/pytxo-mcp` (stdio JSON-RPC). Tools: `pytxo_dry_run`, `pytxo_run`, `pytxo_status`, `pytxo_logs`. Setup: [[mcp-cursor-setup]], [[cursor-mcp-pytxo]].

Full bidirectional routing to arbitrary agent MCP servers remains Phase 3.

Architecture: [[mcp-hub-integration]].
