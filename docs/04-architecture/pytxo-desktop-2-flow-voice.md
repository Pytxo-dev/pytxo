---
title: Pytxo Desktop 2, missions, and Voice
slug: pytxo-desktop-2-flow-voice
status: active
tags: [desktop, flow, voice, architecture]
audience: [human, agent]
layer: presentation
created: 2026-07-12
updated: 2026-08-13
related: [[desktop-visual-system]], [[presentation-passive-telemetry]], [[dag-flow-engine]], [[permission-profile-engine]], [[execution-domains]], [[ADR-0035-desktop-2-quiet-instrument-ia]]
---

# Pytxo Desktop 2, missions, and Voice

Pytxo Desktop 2 is a route-driven supervision shell. **Ops** is today (needs-you, running work, spend). **Missions** is the core object: inventory, New mission composer, and Plan / Live / Review detail. **Approvals** is the decision inbox. Workspaces, Agents, and Settings are catalogs. Agents, logs, diffs, and file lists remain contextual. This keeps Desktop aligned with [[presentation-passive-telemetry]] instead of turning it into an IDE or terminal wall. IA: [[ADR-0035-desktop-2-quiet-instrument-ia]].

The Svelte application depends on a `DesktopBackend` contract. Production uses Tauri IPC; Storybook and browser tests use deterministic preview data. Canonical hashes (`#/missions`, `#/agents`) persist; aliases (`#/flow`, `#/runs`, `#/integrations`, `#/topology-focus`, `#/run-review`) still resolve. The `pytxo-deck-tabs-v1` payload is migrated into recents without deleting the original payload, and the old shell remains available through `desktop_shell_v1` for one rollback release. `pytxo://` is the current deep-link scheme; `pytxo-deck://` remains registered for compatibility. `pytxo://flow` opens New mission in-shell.

## Flow boundary

Text missions enter Rust through `pytxo-orchestrate::flow`. A preview requires a selected [[execution-domains|execution domain]] and non-empty mission. Signal Core enriches path context, the scheduler and Race Shield reject collisions, the permission engine checks the domain ceiling, Blast Shield supplies isolation intent, and the ADE registry verifies the requested CLI. A ready plan is persisted in the global catalog. Dispatch is impossible without a persisted ready preview and revalidates domain, permission, isolation, execution backend, ADE availability, and path claims before calling the standard run dispatcher.

Draft metadata and plan JSON live in `flow_drafts`. Dispatched telemetry stays in the selected domain WAL. No quick-run path exists.

## Voice boundary

`pytxo-voice` owns capture state, a bounded in-memory PCM buffer, VAD trimming, transcription contracts, and model verification. Cancellation, device loss, completion, and drop clear the PCM buffer. There is no recording persistence API and no raw-audio catalog column, event, or diagnostic payload.

CPAL capture is enabled through `native-capture`. whisper.cpp support is isolated behind `local-whisper`, allowing default CI to exercise the complete state machine without audio hardware. Native Whisper builds require CMake plus LLVM/libclang; Linux also requires the platform audio development packages used by CPAL. The default `base.en` model URL is revision-pinned and checked against its SHA-256 before installation under `~/.pytxo/models/voice`.

Voice v1 creates missions only. It does not navigate, approve, stop, or automatically dispatch. Remote transcription implementations must use the `RemoteTranscriber` consent contract, name the provider, destination, and retention policy, and pass the returned transcript through Sovereign Shield before later cloud planning.

## Verification artifacts

Reference renders under `docs/_attachments/desktop-2/` cover canonical and alias routes at 1600×1000, 1280×800, and 960×640. The deterministic `marketing-captures.spec.ts` path checks each expected heading before writing the image. Filenames `flow-*` and `integrations-*` remain so the web asset verifier stays stable. Matching Storybook stories use the same production components and preview backend. Release verification includes Svelte diagnostics, Playwright navigation and rollback coverage, Storybook build plus accessibility checks, Rust Flow/store/Voice suites, native platform smoke tests, and Tauri compilation.
