---
title: IDE integration surfaces
slug: cursor-vs-vscode-vs-neovim
status: active
tags: [integrations, mcp]
audience: [human, agent]
layer: meta
created: 2026-06-02
updated: 2026-06-02
related: [[mcp-router]], [[agent-os-vs-virtual-workspace]]
---

# IDE integration surfaces

Pytxo does not require switching editors. Integration is via **MCP** and optional desktop **Reality Deck**.

| Environment | Integration pattern |
|-------------|---------------------|
| **Cursor** | MCP client → Pytxo hub; agent rules in repo `AGENTS.md` / `.cursor/rules/` |
| **VS Code** | MCP extension ecosystem → same hub |
| **NeoVim** | MCP-capable plugins or sidecar CLI |
| **CLI only** | Pytxo Rust CLI + headless execution yard |

## Design rule

Orchestration policy lives in **Rust**, not in editor-specific config. Editor files only **point** agents at Pytxo docs and commands.

See [[mcp-hub-integration]].
