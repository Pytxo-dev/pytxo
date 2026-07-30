---
title: v1 product and interface reference research
slug: v1-product-reference-research
status: active
tags:
  - project
  - research
  - desktop
  - marketing
  - design
audience:
  - human
  - agent
layer: presentation
created: 2026-07-30
updated: 2026-07-30
related:
  - "[[desktop-visual-system]]"
  - "[[v1-provider-auth-onboarding-research]]"
  - "[[market-ready-polish-research]]"
  - "[[product-vision]]"
---

# v1 product and interface reference research

## Verdict

Pytxo should look like a serious local operator tool, not an AI chat product and not a wall of terminals. The strongest current references lead with a concrete outcome, reveal the real product early, and keep account state, activity, and next actions legible. Pytxo's distinctive proof is the mission loop itself: one outcome becomes an inspectable plan, isolated execution waves, evidence, and an explicit apply decision.

Mobbin was requested for pattern research, but its search API returned a paid-plan requirement in this environment. The reference set therefore uses official product pages and documentation rather than inventing Mobbin results.

## Reference set

| Product | Useful pattern for Pytxo |
|---|---|
| [Cursor](https://www.cursor.com/) | Outcome-led headline, product proof in the first viewport |
| [Cursor features](https://www.cursor.com/features) | Capability groups tied to actual workflows |
| [Cursor changelog](https://www.cursor.com/en/changelog) | Shipped-product evidence instead of future-tense promises |
| [ChatGPT Desktop](https://chatgpt.com/download/) | One obvious download action and platform clarity |
| [Claude Desktop](https://claude.com/download) | Restrained product framing and direct install path |
| [Warp](https://www.warp.dev/) | Dense terminal product shown as a composed work surface |
| [Raycast AI](https://www.raycast.com/core-features/ai) | Fast path from intent to useful action |
| [Raycast AI Chat](https://manual.raycast.com/ai/chat) | Clear separation between account access and model/provider choice |
| [Linear](https://linear.app/features) | Calm information density and keyboard-first operations |
| [Wispr Flow setup](https://docs.wisprflow.ai/articles/3152211871-setup-guide) | First-run setup organized around reaching the first success |
| [Wispr Flow navigation](https://docs.wisprflow.ai/articles/5096240724-navigating-the-wispr-flow-app-desktop-ios-and-android) | Small, stable navigation model |
| [Notion AI](https://www.notion.com/product/ai) | AI presented inside a familiar work object |
| [Replit quickstart](https://docs.replit.com/build/your-first-app) | Guided example that creates a real editable project |
| [Replit Project Editor](https://docs.replit.com/learn/projects-and-artifacts/project-editor) | Evidence stays attached to the work, not in a separate demo shell |
| [Superhuman](https://superhuman.com/email) | Confident typography and measurable workflow claims |
| [Ramp](https://ramp.com/platform) | Operational trust and controls made visible |
| [GitHub Copilot](https://github.com/features/copilot) | Existing-tool integration rather than forced workflow replacement |
| [Perplexity Comet](https://www.perplexity.ai/comet) | A focused browser/desktop proposition with one primary promise |

## Applied decisions

Desktop keeps its dense, calm shell. Integrations now reports installed harnesses and vendor-owned session readiness without exposing account identifiers. Providers is a separate API-billing surface with no key inputs. Onboarding includes a real, dependency-free Git example so the user can reach Flow without supplying a credential. Workspaces exposes the same guided-example action after onboarding.

Marketing uses an editorial split: one strong outcome on the left and verified application evidence on the right. Geist remains the type system. Motion is limited to slow product-camera movement, focus transitions, a restrained capability rail, and scroll-pinned product explanation. Decorative gradients and vague "AI-powered" claims do not carry the story.

The demo follows the same hierarchy: mission, connected harnesses, planned waves, isolated execution, approval, verified result. Every visible number must come from a reproducible benchmark result or be labeled as an example state.

## Anti-slop acceptance test

- A first-time viewer can explain Pytxo in one sentence after the hero.
- The first product image contains a decision or state, not generic code.
- Each status has one next action and one credential owner.
- Product copy distinguishes ChatGPT/Codex and Claude Code sessions from metered APIs.
- Effects never obscure text, controls, evidence, or failure states.
- Screenshots come from the current Desktop build and contain no personal data or secrets.
