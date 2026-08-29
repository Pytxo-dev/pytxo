---
title: ADR-0038 Epistemic state contract for all Pytxo surfaces
slug: adr-0038-epistemic-state-contract
status: accepted
tags: [adr, presentation, desktop, website, trust]
audience: [human, agent]
layer: presentation
created: 2026-08-29
updated: 2026-08-29
adr_id: ADR-0038
related: [[ADR-0037-chroma-aperture-visual-contract]], [[ADR-0039-evidence-ledger-visual-contract]], [[commit-layer]], [[permission-profile-engine]], [[desktop-visual-system]]
---

# ADR-0038: Epistemic state contract for all Pytxo surfaces

## Status

Accepted.

## Context

The orchestration layer already distinguishes what it proved, what it was told,
what it never measured, and what it observed to fail. That distinction is
present in the DTOs that cross the Tauri boundary:

- `EnforcementSurface.status`: `enforced | advisory | unavailable | bypassed`,
  for the run and per agent, across `workspace_isolation`,
  `host_filesystem_boundary`, `network`, and `apply_boundary`
- `RunApplyAttempt.outcome`: `committed | rolled_back | recovery_required |
  interrupted`, alongside `rollback_confirmed`
- `AdeCliStatusDto.auth_state`: `signed_in | signed_out | unknown |
  not_applicable | not_installed`
- `DomainChangesPageDto.cursor_gap`: the system knows when it missed events

The presentation layer discarded nearly all of it and substituted invented
confidence. Two examples were shipped: an identical 72 percent progress fill on
every agent lane regardless of state, for a quantity the orchestrator never
computes, and lane colour assigned from array index while the accepted visual
contract declared that hue encoded execution state.

For a product whose value is a trustworthy claim at a boundary, and whose worst
failure is a receipt certifying a success that did not happen, an interface that
fabricates certainty is a product defect rather than a cosmetic one. The absence
of a stated contract is what allowed it: nothing said that a pixel may not
assert more than its source.

## Decision

1. Every state rendered on any Pytxo surface resolves to exactly one of four
   epistemic states, and each state names the backend conditions that produce
   it.

   | State | Meaning | Representative sources |
   |-------|---------|------------------------|
   | Verified | Mechanically enforced or confirmed | `status="enforced"`, `outcome="committed"`, `rollback_confirmed=true`, digest present |
   | Claimed | Asserted, not mechanically proven | `status="advisory"` |
   | Unknown | No evidence in this snapshot | `status="unavailable"`, `outcome="interrupted"`, `auth_state="unknown"`, `cursor_gap=true`, null digest |
   | Refuted | Evidence of failure or circumvention | `status="bypassed"`, `outcome="rolled_back"`, `outcome="recovery_required"`, `last_apply_error` present |

2. State is legible without colour. Encoding is a non-colour treatment first,
   then hue, and always an accompanying text label: solid fill for verified,
   solid outline for claimed, 45-degree hatch for unknown, solid fill for
   refuted.

3. **Unknown is a state, not a warning.** It carries no hue, no warning glyph,
   and no severity styling. Absence of evidence must not read as a problem, and
   must never read as a pass.

4. **Attention is a separate channel.** Work that requires a human decision gets
   `--state-attention` plus a count, and never shares a hue with unknown.

5. **No element renders a quantity the backend does not compute.** Progress
   visualisation is permitted only where a real numerator and denominator exist,
   such as waves completed over total or files reviewed over total. Everywhere
   else the interface shows a state chip and a timestamp.

6. Inconclusive verification renders as claimed or unknown. It never renders as
   verified.

7. When `cursor_gap` is true or a snapshot is stale, derived boundary checks
   downgrade to unknown rather than displaying last-known-good values.

8. When `rollback_confirmed` is false the interface states plainly that the
   working tree may be partially modified, and offers the concrete recovery
   step.

9. History renders the receipt as of decision time. It is not recomputed against
   current state.

10. The website is bound by the same contract. Any state published on
    pytxo.com uses the same four states with the same encoding, and any
    published measurement carries its non-claims.

## Enforcement

- `packages/chroma` owns `--state-verified`, `--state-claimed`,
  `--state-unknown`, `--state-refuted`, and `--state-attention`. Components may
  not hardcode state colour.
- A stylelint rule bans literal percentage widths inside state and progress
  visuals, so a fabricated fill cannot be reintroduced without a bound source.
- One visual assertion per `EnforcementSurface.status` and
  `RunApplyAttempt.outcome` value confirms each renders a distinct non-colour
  affordance.

## Consequences

The interface becomes strictly less confident than it was, and in places will
show less than before, because it may no longer imply values it does not have.
That is the intended trade: an operator can trust that a filled chip means
something was actually proven.

This contract constrains future features. Any new surface must map its states
into the four before it can be rendered, which is deliberate friction at the
point where fabricated certainty would otherwise enter.

Spectrum is retired from state duty. See
[[ADR-0039-evidence-ledger-visual-contract]] for the visual system that replaces
it, and the partial supersession of
[[ADR-0037-chroma-aperture-visual-contract]].
