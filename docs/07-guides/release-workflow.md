---
title: Release workflow
status: active
tags: [guide, ci]
---

# Release workflow

The **pytxo** monorepo is **private**. Public installs use [Pytxo-dev/pytxo-releases](https://github.com/Pytxo-dev/pytxo-releases) (binaries + install scripts) and [npm](https://www.npmjs.com/package/pytxo).

## One-click release (recommended)

1. Open **Actions → Release → Run workflow** on the private `pytxo` repo.
2. Enter **version** without a `v` prefix (e.g. `0.3.1`).
3. Leave **Publish npm** and **Mirror public** enabled unless you only want a private draft.
4. Run.

The workflow will:

- Build all platform binaries (Linux x64/arm64, macOS x64/arm64, Windows x64)
- Create a GitHub Release on the **private** repo (tag `v{version}`)
- Mirror assets to **public** `pytxo-releases` (if `PYTXO_RELEASES_TOKEN` is set)
- Sync `distribution/pytxo-releases/install.*` to the public repo
- Publish `packages/pytxo` to npm (if `NPM_TOKEN` is set)

## Required secrets (private `pytxo` repo)

| Secret | Purpose |
|--------|---------|
| `NPM_TOKEN` | `npm publish` for `packages/pytxo` |
| `PYTXO_RELEASES_TOKEN` | PAT with `contents: write` on **Pytxo-dev/pytxo-releases** |
| `TAURI_SIGNING_PRIVATE_KEY` | Minisign private key for Desktop auto-updater `.sig` + `latest.json` (see `distribution/tauri/README.md`) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Optional password if the private key is encrypted |

Without `TAURI_SIGNING_PRIVATE_KEY`, Desktop installers still publish, but the updater channel (`latest.json`) is skipped.

## CLI release (alternative)

```bash
git tag v0.3.1
git push origin v0.3.1
```

Pushes the same **Release** workflow via the `v*` tag trigger.

## After release

- Verify [pytxo-releases](https://github.com/Pytxo-dev/pytxo-releases/releases) has all five binaries + `SHA256SUMS.txt`
- Verify `npm i -g pytxo@0.3.1` downloads the correct binary
- Update `RELEASE.md` and `apps/web/src/lib/site.ts` (`PYTXO_VERSION`) before tagging
