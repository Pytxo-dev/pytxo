# Desktop export slot

This directory is reserved for **desktop export** artifacts separate from the live app in [`../desktop`](../desktop).

## What goes here

- Release bundles, installer staging, or snapshot trees copied from an external export path.
- Not the canonical source tree — edit and build from `apps/desktop` instead.

## Tauri generated schemas

Committed Tauri capability/schema output lives under:

`apps/desktop/src-tauri/gen/`

Regenerate with `cargo build -p pytxo-desktop` from the repository root when capabilities change.

## Local build outputs

Add large binaries under `dist/` locally; `dist/` is gitignored when present here.
