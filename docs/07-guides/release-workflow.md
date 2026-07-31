---
title: Release workflow
status: active
tags: [guide, ci]
---

# Release workflow

The **pytxo** monorepo is **private**. Public installs use [Pytxo-dev/pytxo-releases](https://github.com/Pytxo-dev/pytxo-releases) (binaries + install scripts) and [npm](https://www.npmjs.com/package/pytxo).

## One-click release (recommended)

1. Open **Actions → Release → Run workflow** on the private `pytxo` repo.
2. Enter **version** without a `v` prefix (e.g. `1.0.1`).
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
git tag v1.0.1
git push origin v1.0.1
```

Pushes the same **Release** workflow via the `v*` tag trigger.

## Account/billing-blocked runner fallback

Use this only when GitHub jobs fail before their first step. A code or test
failure is not a reason to bypass CI.

1. Run the complete local gate from `AGENTS.md`, including Desktop browser
   tests, native build, web lint/build/E2E, demo typecheck/render, and secret
   scan.
2. Build the fresh host CLI and Desktop installers. Do not rename a prior binary
   and claim it is current.
3. If other operating systems cannot be built, either omit them or mirror the
   last complete matrix with an explicit compatibility label in release notes
   and public install docs. A mirrored binary keeps its embedded old version.
4. Generate `SHA256SUMS.txt` from the exact staged assets.
5. Create matching `v{version}` releases in the private source repo and public
   distribution repo, sync install scripts, then publish the matching npm
   wrapper.
6. Deploy `apps/web` to production only after the release URLs resolve.
7. Verify downloaded checksums, `pytxo --version`, `npm i -g`, the Windows
   installer, website `/download`, and current-version documentation.

Without a Tauri signing key, publish manual installers only. Do not replace
`latest.json`; the updater channel must continue pointing to the last signed
manifest.

## After release

- Verify [pytxo-releases](https://github.com/Pytxo-dev/pytxo-releases/releases) has all five binaries + `SHA256SUMS.txt`
- Verify `npm i -g pytxo@1.0.1` downloads the correct binary
- Update `RELEASE.md` and `apps/web/src/lib/site.ts` (`PYTXO_VERSION`) before tagging
