---
title: Pytxo presentation implementation references
slug: pytxo-presentation-references-2026-09-14
status: active
tags: [project, desktop, website, motion, research]
audience: [human, agent]
layer: presentation
created: 2026-09-14
updated: 2026-09-14
related: [[pytxo-interface-upstream-research-2026-09-13]], [[desktop-interaction-audit-2026-09-13]]
---

# Pytxo presentation implementation references

Primary sources inspected September 14. This note recommends implementation;
it is not runtime acceptance. No package, tooling, or configuration was installed
by this research task. Local manifests report Svelte 5 Desktop and Next 16.2.12 /
React 19.2.4 / Tailwind 4 website.

## Motion: bounded DOM integration

Use `animate` from `motion/mini` for opacity and complete CSS transforms on a
dedicated evidence wrapper. Use `motion` only when independent axes, sequences,
or other hybrid features are needed. Neither requires React rendering.
`stop()` commits current animated styles and cannot restart; `cancel()` restores
the initial state; `complete()` reaches the endpoint immediately. Do not infer
React layout/drag APIs from this DOM API.
([quick start](https://motion.dev/docs/quick-start),
[animate controls](https://motion.dev/docs/animate#controls)).

Pytxo integration recommendations, inferred from those APIs:

- Own one animation per wrapper. Stop on retarget, then start from its current
  computed state; avoid a jump back to the original opening keyframe.
- On destroy, invalidate pending completion callbacks, cancel active effects,
  remove listeners, and restore only properties owned by the action. Never
  overwrite a caller's entire inline-style attribute.
- Guard callbacks by current generation/run identity. Replace obsolete evidence
  synchronously; an outgoing animation must never retain another run's content.
- Keep Apply guards, accessibility visibility, and focus independent of animation
  completion. Do not share transforms with pointer-drag positioning.
- Use a Svelte action lifecycle with cleanup. Listen to
  `matchMedia('(prefers-reduced-motion: reduce)')` changes; settle immediately
  when enabled, retaining the same state and controls.
  ([Svelte lifecycle](https://svelte.dev/docs/svelte/use),
  [media-query observation](https://developer.mozilla.org/en-US/docs/Web/API/Window/matchMedia)).

Native proof must cover rapid toggle, switch-run during animation, keyboard
focus, reduced motion, teardown, and pointer docking. Compare built bundle impact;
the library's published size is not Pytxo's measured increase.

Registry inspection: `motion@13.2.0`, `framer-motion@13.2.0`,
`motion-dom@13.2.0`, and `motion-utils@13.0.0` declare MIT; `tslib@2.8.1`
declares 0BSD. React/ReactDOM peers are optional. None of these inspected
manifests declares preinstall/install/postinstall scripts. Build/prepack and
postpublish scripts exist; do not execute them. Preserve notices and inspect the
actual resolved lockfile. ([Motion manifest](https://registry.npmjs.org/motion/13.2.0),
[transitive manifest](https://registry.npmjs.org/framer-motion/13.2.0),
[license](https://github.com/motiondivision/motion/blob/main/LICENSE.md)).

## Motion AI tooling scope

Free hosted documentation/example-metadata search works without an account.
Motion+ gates premium source, spring generation, audits, and the transition
editor. `motion-ai@14.1.0` is MIT; its interactive installer copies bundled skills
and configures both `https://mcp.motion.dev` and `/plus`. Codex skills map to
`.agents/skills`; project scope passes `local: true` to `add-mcp`. Prefer only
the free endpoint in supported project configuration, preserving existing entries;
do not run a broad installer to obtain documentation already readable publicly.
No private code or recordings belong in documentation queries.
([install instructions](https://motion.dev/docs/ai-kit-install),
[installer source](https://unpkg.com/motion-ai@14.1.0/dist/install.js),
[agent paths](https://unpkg.com/motion-ai@14.1.0/dist/agents.js)).

## Kokonut: inspect and adapt

Both selected patterns are MIT; preserve the copyright/permission notice if
copying substantial source. The registry copies components and dependencies;
avoid whole-project CLI initialization. Website stack compatibility is plausible,
not runtime proof. Desktop needs native Svelte implementation.
([installation](https://kokonutui.com/docs),
[license at inspected revision](https://github.com/kokonut-labs/kokonutui/blob/83eec6d982d400a18438001a8efdbac1f159dd43/LICENSE)).

| Reference | Proposed Pytxo adaptation | Source findings and required evidence |
|---|---|---|
| [Smooth Tab source](https://github.com/kokonut-labs/kokonutui/blob/83eec6d982d400a18438001a8efdbac1f159dd43/components/kokonutui/smooth-tab.tsx) | Website screenshot walkthrough: stable frame and moving selection indicator | Source has no arrow navigation despite inactive tabs being unfocusable, and missing panel targets. Implement complete [tab semantics](https://www.w3.org/WAI/ARIA/apg/patterns/tabs/). Remove blur, looping waves, 400ms slides. Test keyboard/mobile/reduced motion. |
| [AI Input Search source](https://github.com/kokonut-labs/kokonutui/blob/83eec6d982d400a18438001a8efdbac1f159dd43/components/kokonutui/ai-input-search.tsx) | Desktop composer: outcome area with compact contextual action row | Avoid duplicate textbox wrapper, unnamed Send button, unconditional draft clearing, and missing blank/IME guards. Keep real supported controls, draft behavior, and explanatory disabled state. Test submit/newline/focus. |

Attach actual component paths and final evidence to the mission implementation
map; these proposals alone do not establish resource adoption.

[Bklit](https://bklit.com/docs) is deferred: no defined visualization requirement
with real data; [Anime.js](https://animejs.com/documentation) is deferred: no
exceptional timeline/SVG requirement justifies a second engine.
