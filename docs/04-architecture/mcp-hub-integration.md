---
title: MCP hub integration
slug: mcp-hub-integration
status: active
tags: [architecture, mcp, integrations]
audience: [human, agent]
layer: orchestration
created: 2026-06-02
updated: 2026-06-02
related: [[agent-os-vs-virtual-workspace]], [[03-resources/integrations/mcp-router]]
---

# MCP hub integration

Pytxo exposes a **stateless, local-first Model Context Protocol (MCP) hub** so existing environments can orchestrate agents without switching IDEs.

## Supported surfaces

- Cursor, VS Code, NeoVim, and standard CLIs that speak MCP.

## Role

- Route tool/context requests to the correct **execution yard** processes.
- Apply [[regex-sanitization]] before context leaves the machine.
- Coordinate with [[dag-flow-engine]] for parallel work.

Deep reference: [[mcp-router]] (when implementing).
