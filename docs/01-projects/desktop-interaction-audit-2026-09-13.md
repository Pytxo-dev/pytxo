---
title: Desktop interaction audit, September 2026
slug: desktop-interaction-audit-2026-09-13
status: active
tags: [project, desktop, ux, review]
audience: [human, agent]
layer: presentation
created: 2026-09-13
updated: 2026-09-13
related: [[astra-flat-desktop-02]], [[astra-ui-feedback-2026-09-13]], [[mission-loop]], [[ADR-0038-epistemic-state-contract]]
---

# Desktop interaction audit, September 2026

## Evidence and judgment

The inherited source was stabilized before this audit. The exact native build
(`49904602…4694`) was inspected at 1282×802 against its real local ledger. The
browser matrix covers narrower layouts, but is not native acceptance.

The main loop exists, but the restored session does not explain it in ten
seconds. The user can find **New run**, enter an outcome, and build a plan. That
path competes with a persistent shelf of technical views, disabled voice
controls, old drafts, and a dock that can show evidence from a different run or
project. On History, raw IDs lead the scan and Review sits below a long run
list. On Review, an unverified candidate can be titled **Ready to Apply** while
the next line says **Combined checks: not verified**. The backend rejects a
non-empty unverified package, but the interface advertises permission it does
not have.

Heuristic rating: **4/10**. This is an expert instrument with useful safety
data, not yet a self-explanatory work loop. The rating is a diagnostic, not
user research.

## Severity-ordered findings

1. **Major — contradictory Apply state.** Candidate verification, not agent
   exit status, must control the primary action. A zero-change package is
   “Nothing to apply,” not a ready transaction.
2. **Major — evidence loses context.** Unpinned run views survive navigation
   into New work, History, and Setup, consuming roughly a third of the window
   and presenting unrelated failures. Saved references should survive, but
   only views for the current run should render. Explicitly pinned views may
   cross context while retaining their source identity.
3. **Major — review is buried.** Selecting a completed run should expose its
   result and review action without making the user traverse the entire list.
4. **Moderate — vocabulary describes implementation.** Primary UI should use
   **Project** for the selected codebase/root group, **Work** for the current
   outcome, **Run** for an execution record, and **Agent** for a worker.
   “Workspace” belongs in isolation evidence; “mission” and “task” can remain
   contract/plan terms under details.
5. **Moderate — advanced controls arrive too early.** Docks, layouts, sessions,
   permission mechanics, and unavailable voice are progressive evidence or
   setup. They should not frame the empty outcome composer.

## Authorized structural slice

The first repair makes the UI tell the truth: unverified combined candidates
offer **Verify candidate**, verified candidates offer **Apply**, and empty
packages offer no mutation. The dock renders unpinned views only in their
matching run and hides its tool shelf outside active Work/Review. This removes
wrong-context evidence and wasted space without deleting saved views, changing
runtime authority, or weakening Apply. Broader navigation and terminology
changes remain a separate interaction redesign, not incidental polish.

## Implemented and observed

The structural slice is now implemented. Review presentation derives permission
from the exact combined-candidate check and prepared file count: a non-empty
unverified package offers **Verify candidate**, a verified package offers
**Apply**, and an empty package is **Nothing to Apply** with Apply disabled.
Unpinned dock views render only for their matching run; the task-view shelf and
docks do not render on New run, History, or Setup. New run hides unavailable
voice controls and labels the core selector **Project**.

The final native executable (`edd4e5b9…b0be`) was inspected at 1282×802 against
the real local ledger. Setup, History, New run, Work, and the prepared-run review
were observed. The real empty package showed **Nothing to Apply** and an
accessibility-disabled Apply action. Browser contracts cover the non-empty
unverified-to-verified transition and narrower layouts. Fresh-profile native
onboarding, native DPI variation, and a real non-empty candidate were not
accepted in this pass, so the original heuristic rating is not being promoted
to a user-research result.

Back: [[MOC-home]] · [[astra-ui-feedback-2026-09-13]]
