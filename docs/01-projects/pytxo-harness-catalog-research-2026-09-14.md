---
title: Pytxo harness catalog research, September 2026
slug: pytxo-harness-catalog-research-2026-09-14
status: active
tags: [project, orchestration, agents, research]
audience: [human, agent]
layer: orchestration
created: 2026-09-14
updated: 2026-09-14
related: [[execution-domains]], [[permission-profile-engine]], [[blast-shield]]
---

# Pytxo harness catalog research, September 2026

## Evidence boundary

This is a first-party documentation and repository review, checked 2026-09-14.
No harness was installed, authenticated, or executed. A documented command is
therefore suitable for registry metadata and adapter tests, not evidence that a
particular local version can complete a Pytxo mission. PATH detection, version
probing, authentication, a write-in-sandbox smoke, cancellation, and exit/output
handling still need validation before an adapter is called ready.

## Grok naming is now unambiguous

xAI publishes an official local coding-agent harness named **Grok Build**. Its
released executable is `grok`, it supports native Windows, and its documented
one-shot shape is `grok --no-auto-update -p "<prompt>"`. `--no-auto-update` is
recommended for automation; structured output and ACP are also official. The
source is Apache-2.0. Authentication belongs to Grok Build through `grok login`
(browser or device flow) or `XAI_API_KEY`; Pytxo must launch it without reading
or copying credentials. Sources: [overview](https://docs.x.ai/build/overview),
[headless contract](https://docs.x.ai/build/cli/headless-scripting),
[CLI reference](https://docs.x.ai/build/cli/reference), and
[official source/license](https://github.com/xai-org/grok-build).

**Grok Bot is not that executable.** It is an official but separate product:
persistent AI teammates on a shared cloud computer, controlled by messaging in
Cursor/xAI apps. It has no documented local `grok-bot` process for Pytxo to
probe or supervise. A Grok model selected inside another harness is also not a
Grok harness installation. Register the product as **Grok Build**, probe
`grok`, and never invent a `grok-bot` adapter. Source:
[Grok Bot overview](https://docs.x.ai/grok-bot/overview).

## Priority catalog

| Canonical product | Executable and official one-shot shape | Authentication owner and license | Pytxo decision |
| --- | --- | --- | --- |
| **Grok Build** | `grok`; `grok --no-auto-update -p "<prompt>"` | Grok Build owns cached OAuth (`grok login`) or consumes `XAI_API_KEY`; Apache-2.0. | **Add now.** Native Windows and explicit headless/ACP contracts are strong. Do not silently add `--always-approve`; map permissions to Pytxo's profile and isolated execution domain. |
| **goose** (AAIF/Linux Foundation; formerly Block) | `goose`; `goose run --no-session -t "<prompt>"` (JSON and stream-JSON are documented) | goose owns provider configuration; users choose provider/subscription/API credentials through goose. Apache-2.0. | **Add now.** Native Windows and a terminating task command are documented. Keep provider secrets and goose configuration outside Pytxo. Sources: [running tasks](https://goose-docs.ai/docs/guides/running-tasks/), [CLI reference](https://goose-docs.ai/docs/guides/goose-cli-commands/), [official repository](https://github.com/aaif-goose/goose). |
| **Factory Droid** | `droid`; read-only `droid exec "<prompt>"`; modifying work requires an explicit tier such as `droid exec --auto low "<prompt>"` | Factory owns login/API-key state; headless CI uses `FACTORY_API_KEY`. The CLI is proprietary/all-rights-reserved (the separate SDK/actions may have their own licenses). | **Add now, explicitly policy-bound.** Native Windows and headless JSON-RPC are documented. The adapter must choose an autonomy tier deliberately and still rely on Pytxo isolation. Sources: [Droid Exec](https://docs.factory.ai/droid-exec/overview), [CLI reference](https://docs.factory.ai/droid-cli/cli-reference), [official repository](https://github.com/Factory-AI/factory). |
| **Cline CLI** | `cline`; `cline --json "<prompt>"` forces non-interactive NDJSON (positional prompt alone can remain interactive on a TTY) | Cline owns `cline auth` state or consumes the selected provider credential; Apache-2.0. | **Add now only inside the existing isolation boundary.** Native Windows is official, but tools are auto-approved by default. Preserve Pytxo's workspace containment and test its exit/NDJSON behavior. Sources: [CLI README](https://github.com/cline/cline/blob/main/apps/cli/README.md), [official repository/license](https://github.com/cline/cline). |
| **Qwen Code** | `qwen`; `qwen -p "<prompt>"` (text, JSON, and stream-JSON) | Qwen Code owns `/auth`/OAuth/provider settings; Apache-2.0. | **Add detection now; gate write-ready status.** Native Windows and headless execution are official. Choose `--approval-mode` only after mapping `default`, `auto-edit`, `auto`, and `yolo` to Pytxo permission profiles; do not guess. Sources: [headless mode](https://qwenlm.github.io/qwen-code-docs/en/users/features/headless/), [auth/providers](https://qwenlm.github.io/qwen-code-docs/en/users/configuration/model-providers/), [official repository/license](https://github.com/QwenLM/qwen-code). |
| **Kimi Code CLI** | `kimi`; `kimi -p "<prompt>"` or add `--output-format stream-json` | Kimi Code owns device-code OAuth via `kimi login`; custom provider secrets belong to its config. Apache-2.0. | **Add now.** Native Windows, one-shot output, and non-interactive `auto` permission behavior are documented; static deny rules remain active. Windows requires Git for Windows/Git Bash, so report that prerequisite rather than “not installed.” Sources: [command reference](https://www.kimi.com/code/docs/en/kimi-code-cli/reference/kimi-command), [Windows setup/auth](https://www.kimi.com/code/docs/en/kimi-code-cli/guides/getting-started), [official repository/license](https://github.com/MoonshotAI/kimi-cli). |
| **Amp CLI** | `amp`; `amp -x "<prompt>"` | Amp owns `amp login`; non-interactive environments require an Amp access token in `AMP_API_KEY`. Proprietary under the [Amp License Terms](https://ampcode.com/terms). | **Defer for native Pytxo Desktop.** Execute mode is real, but official Windows support is WSL-only and Amp runs tools without approval by default. Supporting it truthfully needs a WSL-aware probe, cwd translation, signal/cancellation handling, and explicit execution-domain semantics. Sources: [execute mode](https://ampcode.com/docs/cli/execute-mode), [platform support](https://ampcode.com/docs/cli). |

## Existing-catalog corrections found during the review

- Antigravity CLI now documents `agy -p "<prompt>"` as its one-shot mode. A
  bare `agy` command starts the TUI, so Pytxo's current bare-`agy` command shape
  is not a truthful headless adapter. Source:
  [Antigravity headless mode](https://antigravity.google/docs/cli/headless/).
- Gemini CLI still has a documented `gemini --skip-trust -p "<prompt>"`
  contract, but Google ended individual/free/AI Pro/Ultra service through that
  CLI on 2026-06-18; enterprise and API-key use remain. Keep detection honest
  about that availability boundary rather than presenting every installation as
  usable. Source: [official transition announcement](https://github.com/google-gemini/gemini-cli/discussions/28017).
- Current Cursor Agent documentation uses `cursor-agent -p "<prompt>"` and
  `--force` for unattended modifications; it does not document Pytxo's current
  `--trust` flag. Defer changing that adapter until a locally installed version
  is probed and a contained mutation test establishes the correct versioned
  command. Sources: [headless mode](https://docs.cursor.com/en/cli/headless),
  [parameter reference](https://docs.cursor.com/en/cli/reference/parameters).

## Adapter rule

Adding a row to the registry proves only that Pytxo knows an official executable
and command contract. “Ready” must additionally mean: the exact local version
was probed, harness-owned auth is available, the prompt reaches a terminating
non-interactive run, writes stay inside the assigned execution domain, denial
and cancellation are observable, and Pytxo records output without claiming the
harness's own status as independent verification. Auto-approve/yolo flags are
permission-policy choices, never generic compatibility flags.

## Identity asset implementation

The Desktop catalog now shows transparent compatibility marks for all 14
recognized harnesses. The implementation deliberately separates identity from
readiness: a mark sits beside the text name and never changes an installation,
authentication, or verification claim.

| Reference | Component/file | Adaptation | Evidence and dependency impact |
| --- | --- | --- | --- |
| Official Cursor and OpenAI brand kits | `AdeIdentity.svelte`; paired `*-on-dark.svg` / `*-on-light.svg` assets | Exact theme variants; no recoloring, mask, or background added | Browser tests assert correct theme switching. Static assets only; no runtime dependency. |
| Primer Copilot, Simple Icons Anthropic/Gemini, Kimi, official Cline robot mark | `AdeIdentity.svelte`; transparent path SVGs | CSS masks inherit readable UI color; Gemini uses its catalogued brand purple | Browser tests assert all mapped marks render and no legacy `.ico` is used. Static assets only. |
| Official goose, OpenCode, Qwen assets | `public/ade/*.svg` | Original transparent/color SVG rendered as an image | Browser load and native visual inspection; static assets only. |
| Official Factory favicon | `factory-droid.svg` | Removed the opaque favicon canvas; retained foreground geometry as a current-color mask | Adaptation recorded in provenance; static asset only. |
| Official Antigravity press icon | `antigravity.png` | Exact full-color transparent PNG with its supplied clear space | Local review only; public compatibility use remains approval-gated by Google. |
| Official Aider repository favicon | `aider.png` | Exact transparent 32px PNG | Apache-2.0 source-identification use; static asset only. |
| Official SpaceXAI Grok brand archive | `grok-on-dark.svg`, `grok-on-light.svg` | Exact supplied white/black transparent logomarks switched by theme | No path or color edits; static assets only. |

The remaining public-release caveat is Antigravity: Google's current product
icon rules require approval for a compatibility use. The exact mark is present
for local visual review but must be removed or explicitly approved before public
distribution. Asset provenance and mark-use boundaries live in
`apps/desktop/public/ade/PROVENANCE.md`, `THIRD_PARTY_NOTICES.md`, and
`docs/01-projects/pytxo-final-three-harness-logo-research-2026-09-15.md`.
