# Internal mirror - not published, not built

This folder is **not** deployed to pytxo.com and is not read by any build script. It appears to be a stale mirror of the root [`docs/`](../../../docs) Obsidian vault from an earlier docs pipeline.

The current pipeline is:

- [`docs/`](../../../docs) - internal Obsidian vault, source of architecture truth, not published directly.
- [`apps/docs/docs/`](../../docs) - Docusaurus source, the canonical **public** docs.
- `apps/web/public/docs/` - generated build output (`scripts/build-docs.mjs` + `scripts/copy-docs.mjs`), served at `pytxo.com/docs`.

This copy is confirmed stale (verified against `package.json`, `build-docs.mjs`, and `copy-docs.mjs` - none of them reference this path). It is safe to delete this folder in a follow-up cleanup once nothing else in the working tree depends on it.

Do not edit these files for publishing; edit `apps/docs` (public docs) or `docs/` (internal vault) instead.
