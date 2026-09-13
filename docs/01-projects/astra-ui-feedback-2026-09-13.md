---
title: Desktop screenshot feedback implementation
date: 2026-09-13
type: implementation
status: native-acceptance-pending
---

# Desktop screenshot feedback

Matt's six native screenshots authorize a bounded follow-up to the approved
mission workspace: remove the review's blank column, make folder grouping
accessible in Desktop, add generated permission emblems, direct dock dragging,
dismissible workspace feedback, and correctly sized voice/setup controls.

## Implemented locally

- Prepared files use the center width; evidence and ownership follow in a
  responsive row. Support panels size to their content. Review identity,
  prepared-byte inspection, freshness checks and Apply remain unchanged.
- Toolbar views and dock tabs accept pointer dragging to visible right/bottom
  targets, including hidden docks. Escape cancels. Saved layouts, keyboard
  movement, resizing, hiding and reset remain available. New views open in the
  actual drop dock so they cannot replace an unpinned view in the other dock.
- Workspace settings can create a project on first folder attachment, then add
  and remove secondary folders. The native path writes a real manifest and
  associates the primary catalog entry. It never grants trust or changes a
  permission profile. Duplicate/overlapping paths and effective-label collisions
  are refused before writing; existing manifests cannot be overwritten on create.
- Workspace success feedback has an accessible dismiss action. The microphone
  selector reserves room for its arrow and label. CLI discovery and integration
  options no longer use oversized empty/agent-row layouts.
- Four generated transparent emblems identify the permission profiles. They are
  decorative identity assets, not evidence that security mechanisms are enforced.

## Explicit execution limit

Folder grouping is implemented; coordinated multi-folder **Flow missions are
not**. Flow currently rejects labeled-root tasks and dispatches one execution
domain. The settings copy explains this and the mission selector labels a
project's primary folder. The existing project CLI retains coordinated execution.
Do not imply that passing a project ID enables cross-root reviewed Apply. The
next product slice must reconcile that boundary before expanding mission dispatch.

## Evidence and lineage

Evidence lives in `target/astra-ui-feedback-20260913/`. Browser screenshots use
preview data or an explicitly labeled IPC fixture. Rust tests exercise real
temporary-directory metadata writes and native adapter contracts. These are
separate from native window inspection and clean Windows installation.

The earlier review-identity MSI (SHA256 `322489A0…`) predates these changes and is
superseded as a current-source candidate. No new installer, public release,
download verification, CI run, push or deployment is claimed here.

Fresh verification: 38 Desktop Rust tests; Svelte/CSS; warning-free clippy;
frontend build; 49 affected workflow tests; 14 feedback/scaling checks; all six
final feedback regressions. These test matrices overlap.
The native validation EXE uses a separate app ID/profile so the old session stays
open. SHA256 `D54CE7FBEF71AEC9FB3FB1AE8CE212EACE2BBF1F18A550731B879238A43E88D4`.
Native onboarding appeared in accessibility, but the returned screenshot showed
Roblox Studio. Desktop input stopped; exclusive desktop access is the next human
action. Native pointer dragging, folder attachment and Windows visual acceptance
remain unverified. Simulated browser DPI is not a Windows scaling test.

## Asset provenance

The built-in image generator produced four individual transparent PNGs, copied
unaltered into `apps/desktop/public/profiles/*-v1.png`. Originals remain under the
Codex generated_images directory. The common brief requested a sparse geometric
science-fiction instrument emblem readable at 32px, restrained Chroma Aperture
edge accents, no words, labels, shields, locks or verification marks. Subjects:
DeepSpace celestial void; Orbit inclined orbital ring; Galaxy three-arm spiral;
Supernova four-point burst. These generated assets are not native app evidence.

Related: [[astra-native-finish-2026-09-10]], [[astra-local-preview-boundary-2026-09-13]].
