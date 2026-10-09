---
title: Desktop and website redesign — October 2026
status: approved
tags: [design, desktop, web, auth]
---

# Desktop and website redesign — October 2026

Approved by Matt on 2026-10-08 ("approve, go") before the demo film. Scope:
Pytxo Desktop and pytxo.com together, a Clerk audit, and backend fixes that
users feel. Chroma Aperture stays; this is hierarchy, density and copy, not a
new visual language. Safety meanings in [[pytxo-interface-engineering]] are
unchanged: only a verified, fresh, non-empty candidate enables Apply.

## Findings (inspected 2026-10-08)

Desktop (current captures in `apps/desktop/captures/desktop-2`):

- **Work header**: six equal-weight buttons (Runs, Details, Decision needed,
  Review changes, Stop, panel). The next action is not obvious.
- **Work canvas** opens at 130% with three small cards in a dotted void; the
  floating "ui · Needs plan · Unblocks tests" chip is cryptic.
- **Status bar** repeats the Ctrl K search hint that the sidebar already shows.
- **New work**: Project and Lead agent selectors sit far right of the heading,
  away from the request; "OpenAI Codex: ready · Check again" is noise once
  ready; each plan task is a tall card with an always-open textarea, so four
  tasks need a long scroll; panels nest boxes inside boxes.
- **Review**: "Applies to" appears three times; "Change set pkg-…" is jargon.
- **Errors**: every backend error reaches the UI as `[code] message`
  (e.g. "[orchestrate] codex did not finish…").

Website and Clerk (live pytxo.com):

- **Sign-in/up** is the stock Clerk card: "Sign in to pytxo" (lowercase app
  name), generic "Welcome back! Please sign in to continue", a bare "Pytxo"
  text header without the mark, no explanation of what an account is for.
- **/account** speaks in deployment terms ("entitlements provided by this
  deployment", "organization policy"). A visitor can't tell why they would
  sign in; Local Core needs no account.
- Clerk wiring is sound: `clerkMiddleware` behind `SITE_AUTH_BACKEND_ENABLED`,
  PKCE Desktop bridge (`deck_callback`) carried through sign-in/up redirects.
  Problems are presentation and copy, not auth logic. Sign-up and real sign-in
  were not exercised (no test account).

## Changes

Desktop

1. Work header: one state-driven primary action (Decide / Review changes /
   Stop), secondary actions quiet; Runs and Details move into an overflow.
2. Work opens on Fleet; Canvas fits its content when chosen; plain-language
   dependency chip.
3. New work: Project and Lead agent move into a context row above the
   request; readiness row appears only when the lead is not ready; plan tasks
   render as compact rows with Edit on demand; flatter panels.
4. Review: one "Applies to"; "Prepared change" instead of "Change set".
5. Status bar without the duplicate search hint.
6. IPC errors keep their code on the error object, not in the message.

Website

7. Sign-in/up: Pytxo mark in the auth header, Clerk appearance on Chroma
   tokens, "Sign in to Pytxo" with a one-line reason (connect Desktop, manage
   your account), and a note that local use needs no account.
8. /account in plain language for signed-out and signed-in visitors.

## Acceptance

svelte-check clean; full Desktop e2e green with updated expectations;
regenerated captures inspected at 1280 and 1600; web lint, build and e2e
green; before/after screenshots in `D:/pytxo-native-acceptance/audit-20261008`.
Demo film is re-recorded after these land.

## Phase 3: terminal fleet (2026-10-09)

Matt asked to go further on Desktop and pytxo.com, "especially the fleet UI",
with ASCII art and a UI style of my choosing, as long as it stays good to use
every day. The vocabulary is recorded in [[pytxo-interface-engineering]]
under Terminal language.

Desktop

- Fleet: a monitor pane (ASCII scope plus one activity row per task with CLI,
  sparkline, state and clock) above TUI worker panes. Queued tasks render as
  compact panes and open into terminals when their worker starts; the board
  follows a newly started worker unless the viewer just scrolled. The activity
  timeline disclosure is gone; the sparklines replace it.
- New work: a fleet preview (selected agents on the scope, lead first, and the
  describe → plan → work → verify → apply loop) fills the empty right side;
  plan tasks are compact TUI panes, so a four-task plan fits on one screen.
- Status line and sidebar: mono status segments, a `| / - \` spinner for live
  workers, an ASCII meter for finished agents.

Website

- Hero: an ASCII aperture sphere with the five agent CLIs on an orbit behind
  the headline (pointer-reactive, still under reduced motion), and a clip of
  Desktop playing the scripted sample run over the hero capture.
- Footer: the wordmark rasterised into the aperture ramp. Step kickers and
  the run record use the same terminal framing.

Safety meanings are unchanged; the sparkline and scope are activity, not
progress or verification.
