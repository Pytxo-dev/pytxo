---
title: ADR-0030 Public docs on Fumadocs in Next.js
slug: adr-0030-public-docs-fumadocs-next
status: accepted
tags: [adr, docs, web]
audience: [human, agent]
layer: meta
created: 2026-07-25
updated: 2026-07-25
adr_id: ADR-0030
related: [[repository-layout]], [[ADR-0029-chroma-shared-design-tokens]]
---

# ADR-0030: Public docs on Fumadocs in Next.js

## Status

Accepted

## Context

Public user docs previously lived in `apps/docs` (Docusaurus 3) and were copied into `apps/web/public/docs` during the Vercel `prebuild`. That dual pipeline drifted from the marketing site, needed a separate Node install, and forced HTML rewrites for static Docusaurus output.

## Decision

1. Host public docs as **Fumadocs MDX** inside `apps/web` (Next.js App Router) at `/docs`.
2. Content source: `apps/web/content/docs/**/*.mdx`.
3. Retire the Docusaurus app and the `build:docs` / `copy:docs` / `bundle-docs` scripts from the web deploy path.
4. Keep the Obsidian vault at repo-root `docs/` as architecture source of truth; public MDX remains a curated user subset.

## Consequences

**Positive**

- One deploy artifact for marketing + docs
- Shared Chroma / Tailwind tokens with the site
- Native Next search route and App Router pages

**Negative / tradeoffs**

- Docs changes require a web app rebuild
- Contributors edit MDX under `apps/web/content/docs`, not `apps/docs`

## Supersedes

Implicit choice of Docusaurus for `apps/docs` (no prior ADR).
