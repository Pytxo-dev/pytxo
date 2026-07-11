---
title: Repository layout
---

# Repository layout

```
pytxo/
├── apps/
│   ├── web/          # Marketing site (Next.js static export)
│   ├── docs/         # Public docs (Docusaurus → pytxo.com/docs/)
│   └── desktop/      # Pytxo Desktop (Tauri)
├── crates/           # Rust workspace
└── tooling/          # Benchmarks and scripts
```

## Web + docs build

Marketing pages build from `apps/web`. Docusaurus builds from `apps/docs` and copies static output into `apps/web/public/docs/` before the Next.js build.

## Contributing

See [Contributing](/docs/developers/contributing) and `CONTRIBUTING.md` in the repo root.
