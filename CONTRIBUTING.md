# Contributing to Pytxo

Thank you for contributing to Pytxo. This repository pairs **code** with an **Obsidian-friendly docs vault** in `docs/`.

## GitHub organization

All official Pytxo repositories live under **[github.com/Pytxo-dev](https://github.com/Pytxo-dev)**.

| Repository | Purpose |
|------------|---------|
| [Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo) | Rust control plane, docs vault, benchmarks |
| [Pytxo-dev/pytxo-desktop](https://github.com/Pytxo-dev/pytxo-desktop) | Reality Deck (Svelte + Tauri) |

Layout and dependency rules: [`docs/08-reference/repository-layout.md`](docs/08-reference/repository-layout.md).

New first-party repos should be created in the **Pytxo-dev** org. See [`docs/08-reference/github-organization.md`](docs/08-reference/github-organization.md).

## Getting oriented

1. Read [`README.md`](README.md).
2. Open [`docs/00-meta/MOC-home.md`](docs/00-meta/MOC-home.md) in Obsidian or your editor.
3. Agents: read [`AGENTS.md`](AGENTS.md).

## Documentation (docs vault)

### Folder layout (PARA-inspired)

| Folder | Use |
|--------|-----|
| `00-meta/` | MOCs, glossary, style guide, agent context |
| `01-projects/` | Active epics with deadlines |
| `02-areas/` | Ongoing domains (orchestration, security, cloud) |
| `03-resources/` | Stable reference (MCP, ecosystem) |
| `04-architecture/` | System views and C4-style notes (not ADRs) |
| `05-adr/` | Architecture Decision Records |
| `06-product/` | Tiers, positioning, GTM |
| `07-guides/` | Tutorials and how-tos (Diátaxis) |
| `08-reference/` | CLI, IPC, schemas *(when implemented)* |
| `09-decisions-pending/` | Proposed decisions before ADR acceptance |
| `_templates/` | Obsidian note templates |
| `_attachments/` | Diagrams and images |

Keep folders **shallow** (max ~2 levels). Prefer `[[wikilinks]]` over deep nesting.

### Writing a note

1. Copy [`docs/_templates/concept-note.md`](docs/_templates/concept-note.md) or the ADR template.
2. Fill YAML frontmatter (`title`, `slug`, `status`, `tags`, `audience`, `layer`).
3. Link related notes in `related:` and inline `[[wikilinks]]`.
4. Update the relevant MOC.

### ADR lifecycle

1. Draft in `09-decisions-pending/` or directly as `ADR-NNNN-slug.md` with status `proposed`.
2. Review with team; set status `accepted` when decided.
3. **Never edit** an Accepted ADR except typos. To change a decision, add a new ADR that **supersedes** the old one.
4. Maintain [`docs/05-adr/index.md`](docs/05-adr/index.md).

### Obsidian

Open the `docs/` folder as a vault. Committed config: `docs/.obsidian/app.json`. Local workspace state is gitignored.

## Code

- Rust: follow [`AGENTS.md`](AGENTS.md) and `.cursor/rules/rust-core.mdc`.
- UI: Svelte 5 Runes only; no direct filesystem access from the presentation layer (ADR-0001).
- Open PRs against `Pytxo-dev/pytxo` (or the repo named in the issue).

## Pull requests

- Target the **Pytxo-dev** org repository that owns the change (usually `pytxo`).
- Link ADRs or docs updated in the PR description.
- Do not commit secrets or large base64 blobs in markdown.

## Doc changelog

Structural vault changes: note in [`docs/00-meta/changelog-docs.md`](docs/00-meta/changelog-docs.md).
