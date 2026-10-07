---
title: Release notes draft v1.2.2
slug: release-notes-v1-2-2
status: draft
tags: [project, release, beta]
audience: [human, agent]
layer: meta
created: 2026-10-03
updated: 2026-10-03
related: [[positioning]], [[product-vision]]
---

# Pytxo v1.2.2 — release notes draft

Editorial draft. The release workflow's canonical prepared payload is
`distribution/release-notes/v1.2.2.md`. Not published; publication needs Matt's
approval of that exact payload. Claims match `RELEASE_READINESS.md`.

---

## Many coding agents, one verified change

Pytxo v1.2.2 is a beta candidate that puts several coding-agent CLIs on one job.

**One request, several agents.** New work starts with every ready agent on the
job: Codex, Claude Code, Cursor Agent, OpenCode and Antigravity. Pytxo plans the
request into tasks that own their files, gives each task to an agent (change any
assignment before you run), and runs up to eight at once. Tasks that share a file
run one after another.

**Let an agent split it.** Describe the whole job and choose *Split with Codex*
or *Split with Claude Code*. The agent reads your project read-only and proposes
one task per line, each naming the files it owns. Edit any line, or Undo.

**Watch the fleet.** Work shows one card per task: the agent, its recent output,
the files it owns, and whether it is working, waiting, passed its checks,
finished without changes, or failed.

**Review one change.** Review now opens on a line diff of every changed file,
names the agent that wrote it, and shows the checks Pytxo ran on all the changes
together. *Before & after* still shows each file's exact bytes.

**Apply exactly that.** Apply writes the reviewed files and nothing else. If the
project changed after review, Apply refuses and asks you to refresh.

### Also in this release

- Plainer language across Desktop: Review, History, Approvals and onboarding no
  longer use internal terms.
- Native Windows caption buttons; compact agent setup with each agent's logo.
- Workers settle as soon as they exit; worker output is stored once, one line per
  event.
- OpenCode readiness counts only stored provider credentials. Agents start with a
  clean environment and never receive API keys from your shell.
- Terminal shell opens on a quick start and plain-language status.

### Requirements

- Windows x64 for Desktop. macOS and Linux have the Pytxo CLI.
- At least one agent CLI installed and signed in with its own account. No Pytxo
  account for local work.
- For OpenCode, store a provider with `opencode auth login`.

### Known limitations

- Splitting a job helps when it has independent parts; a small fix is often
  quicker with one agent.
- An agent can finish and pass its checks without changing files; its card says
  "No changes". Name the file and the change in that task's prompt.
- Host filesystem and network controls are advisory; each run's details list
  which protections were enforced.
- The current local installer is unsigned. Final-artifact native acceptance,
  clean install/upgrade, multi-DPI acceptance, hosted CI and signing remain
  release gates; this draft is not a release announcement.
