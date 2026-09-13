---
title: Local preview renderer boundary
date: 2026-09-13
status: implementation-decision
tags: [desktop, security, approval]
---

# Local preview renderer boundary

The approved mission-workspace proposal includes an explicitly selected local URL
inside a movable dock. This note resolves how to isolate that content; it does not
reopen the layout decision. The raw renderer and dock are now implemented locally;
native acceptance is tracked separately from source and browser contract tests.

## September 13 implementation follow-up

`local_preview.rs` creates an unregistered Wry child, initially blank and hidden.
It checks the actual Environment7 data folder against its fresh private directory
before navigation. Native messaging and host objects are disabled; no Tauri callback
or custom protocols are installed. Main-webview identity controls all commands.
Popup, download and permission callbacks deny access; exact-origin top-level/frame
navigation and worker-aware resource filters require supported native interfaces.
F6/Tab exhaustion hands focus to the main webview. Failed loads and blocked requests
are surfaced without recording private request URLs. This is not a general network
sandbox; real runtime coverage remains necessary.

The dock starts only on explicit Open. Restored references contain neither URL nor
renderer identity. Visibility updates carry revisions, native leases expire, and
decision/menu barriers await hiding. Hide failure attempts native close before
allowing a decision; failed hide and close produce a visible recovery message.
Inline disclosures remain usable, while drag/resize gestures suppress the child.

Fresh source/contract evidence and actual native captures are under
`target/astra-native-dock-20260913`. The original storage-override profile was
preserved and produced the expected native refusal before page loading (capture 14,
EXE 8b77c28d). The first relative-dataDirectory validation build (ba39bb3d) was
stopped when process metadata revealed it used the default profile. Locked
tauri-runtime 2.11.1 omits data_directory in WindowConfig conversion. The narrowly
conditional main_profile.rs workaround now applies explicit safe relative main
profiles through the native builder; default startup remains unchanged. A stopped
test-profile copy is prepared for renewed validation. Actual process paths must
confirm isolation before positive testing; this is not clean-install acceptance.

Follow-up DD1EF49B executable verified separate actual process directories and
rendered the disposable loopback fixture. Captures 15/16 establish live rendering
and menu hiding. Fixture reports, server request logs and process paths show
rejected custom-IPC/alternate-server fetches, blocked popup/alternate frame and a
working same-server frame. Inert ipc.postMessage returned without a result; absence
of a result alone is not a security proof. Native bottom-dock inspection found a
header-density defect (17), repaired in a subsequent bounded UI follow-up.

## Current evidence and decision

The locked Tauri 2.11.2 child-webview API requires `unstable`. Its managed webviews
receive the framework IPC integration. Custom app commands can bypass ACL checks
for local app origins when no app ACL manifest exists; a development preview could
share the app's dev URL. Existing capabilities select the `main` and `flow` OS
windows, not only their privileged webviews. The internal channel fetch path also
has an ACL exception. These are source observations, not a demonstrated exploit
of the running app. They prevent a claim that origin checks alone isolate previews.

**Recommended:** a raw Wry child renderer, with no IPC handler and no Tauri/custom
protocol registration, created inside the existing dock. Wry 0.55.1 is already in
Cargo.lock and the native dependency graph. Use that exact existing version as a
direct Desktop dependency; do not upgrade it or enable Tauri `unstable`. Keep the
main CSP and capabilities unchanged. A dependency-only proposed diff is staged
under `target/astra-native-dock-20260913/preview-dependency-proposal.patch`.

Independent review concluded that directly reusing this already-locked rendering
dependency is not inherently a substantial new dependency. The narrowly isolated
implementation fits the approved preview scope without a new layout approval.
Permission/CSP expansion, shared authentication, a new dependency/version or a
weaker boundary would require the existing explicit approval gate. Native tests
must demonstrate the boundary before enabling the integration.

## Bounded implementation

1. Only the actual main webview can create/control previews; sharing its OS window
   is insufficient. `local_preview_policy.rs` provides tested caller and URL rules.
   Renderer commands now integrate these rules in `local_preview.rs`.
2. The user selects an HTTP(S) URL on localhost, 127.0.0.1 or [::1]. Show the
   canonical target before opening it. Credentials/control characters and other
   hosts/schemes are rejected. Subsequent navigation remains on the exact selected
   scheme/host/port; another local server also requires a new explicit selection.
3. A separate private WebContext/storage directory must prevent sharing main-app cookies
   and storage; verify the actual native data path. Incognito alone can silently
   fall back to an ordinary controller on older WebView2 environments. No inherited sign-in, clipboard integration, native command handling,
   arbitrary file access, downloads, popups or implicit external navigation.
4. Keep raw renderer handles on the native UI thread. Native ownership and bounds
   checks constrain each child to its active dock viewport. Hiding, tab switching,
   workspace switching, modal opening and window shutdown must hide or dispose
   children before they can cover mission controls or intercept input. A renderer
   crash becomes a visible error, not an invisible overlay.
5. Restored layouts contain references only. Opening the app never navigates or
   starts a server. Close destroys the renderer; it does not stop a user server.
   No IDE, agent TUI, arbitrary split, floating browser or extension work is added.

## Acceptance before release

Use a disposable loopback test page and harmless read-only probes. Confirm that
Pytxo custom IPC, framework/plugin IPC and internal channel-fetch access are
unavailable even if a page imitates framework globals or shares the dev URL.
Confirm storage separation; block remote and alternate-port redirects, popup and
download requests. A probe failing to display output is not proof of denial.
Check nested frames, reload, renderer failure, hidden docks, modal overlap,
resizing, keyboard focus and Windows scaling against the actual native renderer.
Keep Stop, Review and Apply outside preview authority. Retain raw results and
exact executable identity. URL-policy unit tests are not native isolation proof.

The independent review found and reproduced an origin-only check accepting a
`blob:` destination. Navigation now also requires HTTP(S), with a regression test.
Raw Wry injects an inert `window.ipc.postMessage` object even without a callback;
do not mistake its existence for Pytxo command access or claim that no IPC-shaped
object exists. Disable unnecessary devtools, context menus, autofill and permission
grants explicitly. Top-level navigation callbacks are not a subresource firewall.
Nested-frame and resource behavior must be measured and described honestly.

Existing terminal-input, fresh run/Apply, installer, demo and publication gates
remain in [[astra-execution-2026-09-07]] and the root RELEASE_PLAN.md/CHECKPOINT.md records.
