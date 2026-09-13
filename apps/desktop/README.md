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

## Mission workspace and docks

Work keeps the mission, progress, Stop and review actions together. Select an
agent to inspect its recorded output; Evidence, Files and Dependencies open
context for the selected run. Agent output is read only. Its default plain-text
view removes terminal formatting without replaying cursor redraws. Open Source
details to switch to raw event text; stored records remain unchanged. Files show the frozen
before/after contents of a prepared package; use the full review to authorize
Apply. Missing records and failed verification remain visible.

Drag tabs between the right and bottom docks, or use View options to move,
reorder, pin and focus them. Separators support arrow keys, Shift for larger
steps, Home/End and Escape to cancel. Layout saves named arrangements and
resets placement without stopping work. Narrow windows remember their own
tab order, visibility and height; expanding restores the wide arrangement.

In native Desktop, Sessions → New workspace terminal explicitly starts your
own shell in the selected project folder. Check its workspace and enable input
before typing. These commands write directly to your workspace, outside reviewed
Apply; repository drift still requires a fresh review. Closing or hiding its
view leaves the shell running. Reopen it from Sessions, or use End session to
stop it. Desktop warns before a full exit with live sessions. Detached commands
may continue independently; sessions do not survive a full application exit.
Restored tabs never restart a shell or restore permission to send input.

Built-in dependency views use recorded plan data. On Windows, Preview opens a
local server address you explicitly enter. It uses separate browser storage and
has no Pytxo command connection. Navigation, frames and filtered resource requests
stay on the selected server; external assets and services may be blocked. This is
not a general network sandbox or evidence that the displayed page is correct.
F6 returns keyboard focus to Pytxo. Pause or Close affects the view, not your server.
If a page fails to load, Retry closes that renderer before reopening the same
explicitly selected address. A failed close leaves the existing view in place
and reports the error; it cannot create a second renderer.
Opening a menu or decision hides the page; saved layouts restore an empty preview
reference without opening a URL. A shared WebView2 storage override causes refusal
before navigation. Interactive agent TUIs remain deferred.

## Architecture

- Tauri IPC connects the UI to the local Pytxo orchestration and evidence services.
- Presentation layer has no direct filesystem access (ADR-0001).

Export artifacts and release staging: see [`../desktop-export/README.md`](../desktop-export/README.md).
