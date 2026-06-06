# Documentation style guide

## Frontmatter (required on substantive notes)

```yaml
---
title: Human-readable title
slug: kebab-case-slug
status: draft | active | archived
tags: [domain, topic]
audience: [human] | [human, agent] | [agent]
layer: presentation | orchestration | execution | cloud | security | meta
created: YYYY-MM-DD
updated: YYYY-MM-DD
related: wikilink, another-note
---
```

ADRs add: `adr_id: ADR-NNNN` and use `status: proposed | accepted | superseded | deprecated`.

## Files

- **kebab-case.md** for notes; **ADR-NNNN-short-title.md** for ADRs.
- One concept per file; target **300–700 words**.
- Prefer `wikilinks` over long paths in prose.

## Diagrams

- **Mermaid** in fenced blocks for architecture.
- Binary assets in `docs/_attachments/`—never `data:image` base64 in git.

## Voice

- Precise, engineering-first; separate **claims** from **validated benchmarks** ([competitive-benchmarks](/docs/competitive-benchmarks)).
- Document token cost and competitor capabilities honestly.

## Agents

- Update [MOC-home](/docs/moc-home) when adding a new top-level concept.
- Irreversible architecture → new ADR in `05-adr/`.

Templates: `docs/_templates/`.
