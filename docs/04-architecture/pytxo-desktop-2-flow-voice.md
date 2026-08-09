---
title: Pytxo Desktop 2, Flow, and Voice
slug: pytxo-desktop-2-flow-voice
status: active
tags: [desktop, flow, voice, architecture]
audience: [human, agent]
layer: presentation
created: 2026-07-12
updated: 2026-08-01
related: [[desktop-visual-system]], [[presentation-passive-telemetry]], [[dag-flow-engine]], [[permission-profile-engine]], [[execution-domains]], [[ADR-0034-immutable-review-package-and-durable-apply]]
---

# Pytxo Desktop 2, Flow, and Voice

Pytxo Desktop 2 opens on Flow. Compose, Active, History, and contextual Run
Review share that mission surface. Operations, Workspaces, and Settings remain
primary destinations. Approvals and Integrations are secondary system
destinations; structural Focus is contextual mission detail. Legacy Runs and
Run Review deep links redirect into Flow.

The Svelte application depends on a `DesktopBackend` contract. Production uses
Tauri IPC; browser tests use deterministic preview data. `AppRoute` is
persisted separately from workspace recents. The 3D Deck is absent from the
normal production startup path. Its lazy development import requires
`pytxo-developer-deck-v1=true` and the compatibility
`desktop_shell_v1=true` flag. `pytxo://` is the current deep-link scheme;
`pytxo-deck://` remains registered for compatibility.

## Flow boundary

Text Flow enters Rust through `pytxo-orchestrate::flow`. A preview requires a selected [[execution-domains|execution domain]] and non-empty mission. Signal Core enriches path context, the scheduler and Race Shield reject collisions, the permission engine checks the domain ceiling, Blast Shield supplies isolation intent, and the ADE registry verifies the requested CLI. A ready plan is persisted in the global catalog. Dispatch is impossible without a persisted ready preview and revalidates domain, permission, isolation, execution backend, ADE availability, and path claims before calling the standard run dispatcher.

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

## Voice boundary

`pytxo-voice` owns capture state, a bounded in-memory PCM buffer, VAD trimming, transcription contracts, and model verification. Cancellation, device loss, completion, and drop clear the PCM buffer. There is no recording persistence API and no raw-audio catalog column, event, or diagnostic payload.

CPAL capture is enabled through `native-capture`. whisper.cpp support is isolated behind `local-whisper`, allowing default CI to exercise the complete state machine without audio hardware. Native Whisper builds require CMake plus LLVM/libclang; Linux also requires the platform audio development packages used by CPAL. The default `base.en` model URL is revision-pinned and checked against its SHA-256 before installation under `~/.pytxo/models/voice`.

Voice v1 creates Flow missions only. It does not navigate, approve, stop, or automatically dispatch. Remote transcription implementations must use the `RemoteTranscriber` consent contract, name the provider, destination, and retention policy, and pass the returned transcript through Sovereign Shield before later cloud planning.

## Verification artifacts

The 27 reference renders under `docs/_attachments/desktop-2/` cover
Operations, Workspaces, Flow, Approvals, Integrations, Settings, Focus, ready
Run Review, and applied Run Review at 1600×1000, 1280×800, and 960×640.
`marketing-captures.spec.ts` checks the expected state before writing each
image. The web verifier checks dimensions, distinct content, and byte-for-byte
correspondence with `apps/desktop/captures/desktop-2/`.

Release verification includes Svelte diagnostics, keyboard and reduced-motion
coverage, review-state and cursor regressions, all three capture widths,
native contract tests, and the Tauri release build.
