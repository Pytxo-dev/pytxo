# Pytxo Desktop

Pytxo's optional control UI for preparing delegated work, observing runs,
reviewing combined candidates and applying approved repository changes.
Its three destinations are Work, History and Setup.

**Location:** `apps/desktop` in the [pytxo](https://github.com/Pytxo-dev/pytxo) monorepo.

**Requires:** the in-tree Pytxo workspace crates via path dependencies.

## Prerequisites

- Node.js 22+
- Rust 1.88+ (repo root workspace)
- Tauri system deps on Linux (see root CI workflow)

Choose project folders inside Desktop. The separately installed Pytxo CLI is
optional for Desktop use.

## Build

From repository root:

```bash
cd apps/desktop
npm ci
npm run check
npm run build:native
```

The native development executable is written to the root workspace's
`target/release` directory. For the Windows distribution MSI, use
`npm run build:msi` from this directory. It builds voice support and passes the
shared `.cargo/windows-msvc.toml` policy explicitly to Cargo, so the packaged
app does not depend on separately installed Visual C++ runtime DLLs. Verify
the actual package with `tooling/scripts/verify-windows-msi.ps1` and complete
the clean-machine acceptance in the [release guide](../../docs/07-guides/release-workflow.md).

## Architecture

- Tauri IPC connects the UI to the local Pytxo orchestration and evidence services.
- Presentation layer has no direct filesystem access (ADR-0001).

Export artifacts and release staging: see [`../desktop-export/README.md`](../desktop-export/README.md).
