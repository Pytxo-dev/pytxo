---
title: ADR-0029 Chroma shared design tokens
slug: adr-0029-chroma-shared-design-tokens
status: accepted
tags: [adr, design, presentation]
audience: [human, agent]
layer: meta
created: 2026-06-09
updated: 2026-07-18
adr_id: ADR-0029
related: [[desktop-visual-system]], [[product-vision]]
---

# ADR-0029: Chroma shared design tokens

## Status

Accepted

> **Renumber (Phase 73):** Previously filed as a duplicate `ADR-0014` alongside [[ADR-0014-multi-provider-byok-catalog]]. ID **ADR-0029** is canonical; decision body unchanged.

## Context

Chroma brand tokens (void background, magenta/gold/cyan/violet cycle) were duplicated across `apps/web`, `apps/desktop`, and `crates/pytxo-tui`, risking drift.

## Decision

Extract a shared `packages/chroma` package:

- `tokens.css` — CSS custom properties
- `utilities.css` — framework-agnostic Chroma utility classes
- `tokens.json` — machine-readable palette for Rust TUI alignment

Web and desktop import from `@pytxo/chroma`. TUI constants in `theme.rs` must match `tokens.json`.

## Consequences

**Positive**

- Single source of truth for brand palette
- Desktop can use full Chroma utilities without Tailwind

**Negative**

- TUI still requires manual sync or a future codegen step from `tokens.json`

## Links

- [[desktop-visual-system]]
- `packages/chroma/`
