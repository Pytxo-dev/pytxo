---
title: Pytxo interface upstream research, September 2026
slug: pytxo-interface-upstream-research-2026-09-13
status: active
tags: [project, desktop, ux, motion, research]
audience: [human, agent]
layer: presentation
created: 2026-09-13
updated: 2026-09-13
related: [[desktop-interaction-audit-2026-09-13]], [[astra-ui-feedback-2026-09-13]]
---

# Pytxo interface upstream research, September 2026

## Evidence boundary

This review used first-party documentation available on 2026-09-13. No package,
skill, MCP server, or editor extension was installed or configured. Pytxo
Desktop is a Svelte 5 application and currently declares neither `motion` nor
`animejs` in `apps/desktop/package.json`; it already has CSS transitions and
reduced-motion handling. A reference is therefore not an installation
recommendation.

## Motion: preferred advanced-interaction candidate

Motion's framework-neutral package is installed with `npm install motion` and
imported as `import { animate } from "motion"` or, for the smaller native-Web-
Animations subset, from `"motion/mini"`. The JavaScript API accepts DOM elements
or selectors and supports springs, controls, SVG, staggering, and sequenced
animations ([JavaScript quick start](https://motion.dev/docs/quick-start),
[`animate()` reference](https://motion.dev/docs/animate)). This is the Motion API
that could be evaluated for Pytxo.

Do not confuse it with **Motion for React**. React components and hooks use
imports such as `motion`, `AnimatePresence`, and `useReducedMotion` from
`"motion/react"`; those examples are React-only
([React guide](https://motion.dev/docs/react)). Motion's current platform index
lists React, JavaScript, and Vue, but not Svelte
([docs index](https://motion.dev/docs)). **Inference:** the DOM-based JavaScript
API can be called inside Svelte lifecycle boundaries, but upstream does not
currently document a first-party Svelte component adapter. Any Svelte adoption
needs a small local proof covering cleanup, cancellation, state interruption,
SSR safety, and native WebView behavior before a dependency is added.

Motion's official AI Kit is a separate developer tool. `npx motion-ai` prompts
for project/global scope and supported agents, installs skills, and writes
hosted MCP configuration. Free access covers documentation and example search;
Motion+ adds CSS spring generation, MotionScore audits, a transition editor,
and premium source. Upgrades rewrite the installed skill and MCP configuration
([AI Kit install guide](https://motion.dev/docs/ai-kit-install)). Because this
mutates developer configuration and some features require sign-in, it should be
evaluated and authorized separately from adopting the runtime. It was not run.

Use Motion JavaScript only when CSS becomes brittle for interruptible state,
spatial relocation, dock settling, coordinated progress, or hierarchy changes.
Prefer `transform` and `opacity`, test on the actual Desktop WebView, and retain
Pytxo's reduced-motion path; Motion's own performance guide makes those the
safest rendering properties ([performance guide](https://motion.dev/docs/performance)).

## Anime.js: exceptional choreography, not the default

Anime.js installs as `npm install animejs` and exposes ES modules such as
`animate`, `createTimeline`, `createDraggable`, `createLayout`, SVG utilities,
and a smaller `waapi` path
([installation](https://animejs.com/documentation/getting-started/installation/),
[module imports](https://animejs.com/documentation/getting-started/module-imports/)).
Its timeline can synchronize animations, timers, callbacks, labels, and other
timelines ([timeline reference](https://animejs.com/documentation/timeline/));
its SVG utilities cover line drawing and path-following
([SVG motion path](https://animejs.com/documentation/svg/createmotionpath/)).

That makes Anime.js a candidate only for an unusually choreographed timeline or
SVG explanation that CSS and Motion cannot express cleanly. It should not run
routine hover, list, progress, dock, or Apply interactions. Upstream recommends
WAAPI under CPU/network load or when startup size is critical, and the
JavaScript engine for complex timelines, SVG/DOM attributes, canvas/WebGL, or
advanced callbacks ([WAAPI decision guide](https://animejs.com/documentation/web-animation-api/when-to-use-waapi/)).
**Unavailable/uncertain:** the reviewed first-party getting-started material
documents vanilla JavaScript and React, but no Svelte guide was found. A generic
DOM integration is plausible, not verified Svelte support. If ever evaluated,
scope it to a component and revert instances on teardown; upstream's React
example demonstrates that cleanup contract with `createScope()`
([React integration](https://animejs.com/documentation/getting-started/using-with-react/)).

## Kokonut UI and Bklit UI: references, not ports

Kokonut UI is explicitly a collection of React components built with Tailwind
CSS and Motion. Its shadcn registry copies React source and can automatically
install Tailwind, Lucide, shadcn, and component-specific dependencies
([project overview](https://kokonutui.com/),
[installation](https://kokonutui.com/docs)). Use live examples to study timing,
feedback, compact composition, and interaction states. Do not run its CLI,
copy JSX into Svelte, introduce React, or inherit its visual effects wholesale.

Bklit UI is likewise a React/shadcn source registry focused on charts and data
visualization ([introduction](https://ui.bklit.com/docs),
[installation](https://bklit.com/docs/installation)). It belongs in Pytxo's
future reference set only when a real quantitative question needs a chart—for
example duration or throughput over time—not for task lists, evidence, or
topology decoration. Its chart docs warn that APIs, Studio controls, and docs
may change before release ([bar-chart reference](https://bklit.com/docs/components/bar-chart));
its repository also distinguishes MIT chart source from the proprietary Studio
([repository README](https://github.com/bklit/bklit-ui/blob/main/README.md)).
No Bklit dependency or generated React source should enter Desktop merely to
imitate a screenshot.

## Decision ladder for Pytxo

1. **No animation:** default when state remains clear without it, especially in
   repeated review and Apply work.
2. **CSS:** hover/focus/pressed feedback, short opacity or color transitions,
   and simple transforms with deterministic start and end states.
3. **Motion JavaScript:** interruptible or coordinated spatial/state changes
   where CSS obscures causality; require a measured Svelte/native proof first.
4. **Anime.js:** rare authored timelines or SVG choreography; require a written
   reason that Motion is insufficient.

Every motion proposal must communicate state, causality, spatial relationship,
hierarchy, or progress. It must preserve focus, hit targets, cancellation, and
`prefers-reduced-motion`, and must not slow a frequent workflow.

## First-party interaction references

- VS Code starts with a simple default but supports visible drag destinations,
  remembered placement, reset-to-default, resizing, and keyboard move commands
  ([Custom Layout](https://code.visualstudio.com/docs/configure/custom-layout)).
  Pytxo should offer the same recovery and non-pointer parity for docks.
- Linear's Peek keeps the list context while evidence appears on demand and
  closes predictably with Escape
  ([Peek](https://linear.app/docs/peek)). Pytxo evidence should be similarly
  close, contextual, and dismissible rather than permanently dominant.
- GitHub Desktop leads review with a file list and diff, then lets users change
  diff mode, expand context, and select exact changes
  ([reviewing changes](https://docs.github.com/en/desktop/making-changes-in-a-branch/committing-and-reviewing-changes-to-your-project-in-github-desktop)).
  Pytxo should lead with outcome and safety, then progressively reveal exact
  files and evidence without hiding the candidate boundary.

Back: [[desktop-interaction-audit-2026-09-13]]
