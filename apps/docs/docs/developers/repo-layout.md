---
title: Repository layout
---

# Repository layout

```
pytxo/
├── apps/
│   ├── web/          # Marketing site (Next.js static export)
│   ├── docs/         # Public docs (Docusaurus → pytxo.com/docs/)
│   └── desktop/      # Reality Deck (Tauri)
├── crates/           # Rust workspace
├── docs/             # Internal Obsidian vault (not published verbatim)
└── tooling/          # Benchmarks and scripts
```

## Web + docs build

Marketing pages build from `apps/web`. Docusaurus builds from `apps/docs` and copies static output into `apps/web/public/docs/` before Next.js export.

## Internal vault

The root `docs/` folder is the team knowledge base (ADRs, agent notes, wikilinks). Consumer-facing docs live in `apps/docs/` and are written for humans, not synced automatically.

## Contributing

See [Contributing](/docs/developers/contributing) and `CONTRIBUTING.md` in the repo root.
