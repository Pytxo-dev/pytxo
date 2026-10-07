---
title: Pytxo Desktop 2, missions, and Voice
slug: pytxo-desktop-2-flow-voice
status: active
tags: [desktop, flow, voice, architecture]
audience: [human, agent]
layer: presentation
created: 2026-07-12
updated: 2026-09-21
related: [[desktop-visual-system]], [[presentation-passive-telemetry]], [[dag-flow-engine]], [[permission-profile-engine]], [[execution-domains]], [[ADR-0034-immutable-review-package-and-durable-apply]], [[ADR-0035-desktop-2-quiet-instrument-ia]], [[ADR-0038-epistemic-state-contract]], [[ADR-0039-evidence-ledger-visual-contract]]
---

# Pytxo Desktop 2, missions, and Voice

Pytxo Desktop 2 is a route-driven supervision shell. **Work** is the focused run: ledger plus commit boundary. **History** is the run inventory. **Setup** is configuration. Workspace is title-bar context, not a destination. Approvals are an overlay. This keeps Desktop aligned with [[presentation-passive-telemetry]] instead of turning it into an IDE or terminal wall. Visual contract: [[ADR-0039-evidence-ledger-visual-contract]]. The six-item nav in [[ADR-0035-desktop-2-quiet-instrument-ia]] is retired.

The Svelte application depends on a `DesktopBackend` contract. Production uses Tauri IPC; Storybook and browser tests use deterministic preview data. Canonical hashes are `#/work`, `#/history`, and `#/setup`. Aliases (`#/operations`, `#/missions`, `#/flow`, `#/runs`, `#/integrations`, `#/topology-focus`, `#/run-review`, `#/approvals`) still resolve. The `pytxo-deck-tabs-v1` payload is migrated into recents without deleting the original payload, and the old shell remains available through `desktop_shell_v1` for one rollback release. `pytxo://` is the current deep-link scheme; `pytxo-deck://` remains registered for compatibility. `pytxo://flow` opens New run in-shell.

## Flow boundary

Text missions enter Rust through `pytxo-orchestrate::flow`. A preview requires a selected [[execution-domains|execution domain]] and non-empty mission. Signal Core enriches path context, the scheduler and Race Shield reject collisions, the permission engine checks the domain ceiling, Blast Shield supplies isolation intent, and the ADE registry verifies the requested CLI. A ready plan is persisted in the global catalog. Dispatch is impossible without a persisted ready preview and revalidates domain, permission, isolation, execution backend, ADE availability, and path claims before calling the standard run dispatcher.

Draft metadata and plan JSON live in `flow_drafts`. Dispatched telemetry stays in the selected domain WAL. No quick-run path exists.

## Review and synchronization boundary

Run Review renders the immutable manifest stored for an Orbit or Galaxy run.
It shows exact additions, modifications, and deletions; full base revision and
package digest; task ownership; permission profile; enforcement receipt; and
durable Apply history. Apply, refresh, retry, discard, and reconciliation are
backend intents. Svelte never reads agent workspaces or repository files.

Native mutations emit `pytxo://domain-changed`. Desktop then catches up through
the monotonic `domain_changes` cursor. A full snapshot is reserved for initial
load, domain switch, reconnect or cursor reset, and an infrequent integrity
refresh. Cursor gaps establish a canonical reset boundary before the next
delta is consumed.

Change polling opens an existing domain store read-only, without creating or
migrating it. Missing stores and incompatible change-ledger schemas remain
errors. SQLite polling and full snapshot loading execute on blocking workers
instead of the native event loop. This scheduling boundary does not change
Apply authority or turn observation into a write operation.

## Voice boundary

`pytxo-voice` owns capture state, a bounded in-memory PCM buffer, VAD trimming, transcription contracts, and model verification. Cancellation, device loss, completion, and drop clear the PCM buffer. There is no recording persistence API and no raw-audio catalog column, event, or diagnostic payload.

CPAL capture is enabled through `native-capture`. whisper.cpp support is isolated behind `local-whisper`, allowing default CI to exercise the complete state machine without audio hardware. Native Whisper builds require CMake plus LLVM/libclang; Linux also requires the platform audio development packages used by CPAL. The default `base.en` model URL is revision-pinned and checked against its SHA-256 before installation under `~/.pytxo/models/voice`.

Voice v1 creates missions only. It does not navigate, approve, stop, or automatically dispatch. Remote transcription implementations must use the `RemoteTranscriber` consent contract, name the provider, destination, and retention policy, and pass the returned transcript through Sovereign Shield before later cloud planning.

## Verification artifacts

Reference renders under `docs/_attachments/desktop-2/` cover canonical and alias routes at 1600×1000, 1280×800, and 960×640. The deterministic `marketing-captures.spec.ts` path checks each expected heading before writing the image. Filenames `flow-*` and `integrations-*` remain so the web asset verifier stays stable. Matching Storybook stories use the same production components and preview backend. Release verification includes Svelte diagnostics, Playwright navigation and rollback coverage, Storybook build plus accessibility checks, review-state and cursor regressions, native contract tests, and Tauri compilation.
